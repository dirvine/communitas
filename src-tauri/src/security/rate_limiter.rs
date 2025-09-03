//! Rate limiting module to prevent DoS attacks and abuse
//!
//! This module provides:
//! - Request rate limiting per user/IP
//! - Sliding window rate limiting
//! - Different limits for different operations
//! - Automatic cleanup of old entries

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    LimitExceeded,
    #[error("Rate limiter lock error")]
    LockError,
    #[error("Configuration error")]
    ConfigError,
}

/// Default rate limits
pub const DEFAULT_REQUESTS_PER_MINUTE: u32 = 60;
pub const AUTH_REQUESTS_PER_MINUTE: u32 = 5; // Lower limit for auth operations
pub const DHT_REQUESTS_PER_MINUTE: u32 = 30; // Moderate limit for DHT operations
pub const MESSAGE_REQUESTS_PER_MINUTE: u32 = 120; // Higher limit for messages

/// Time window for rate limiting
pub const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

/// Cleanup interval for rate limiter entries
pub const CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

/// Rate limiter entry tracking requests within a time window
#[derive(Debug, Clone)]
struct RateLimitEntry {
    requests: Vec<Instant>,
    last_cleanup: Instant,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Add a new request and clean up old ones
    fn add_request(&mut self, now: Instant, window: Duration) {
        // Clean up old requests outside the window
        let cutoff = now - window;
        self.requests.retain(|&request_time| request_time > cutoff);

        // Add the new request
        self.requests.push(now);
        self.last_cleanup = now;
    }

    /// Check if adding a new request would exceed the limit
    fn would_exceed_limit(&self, limit: u32, now: Instant, window: Duration) -> bool {
        let cutoff = now - window;
        let current_requests = self
            .requests
            .iter()
            .filter(|&&request_time| request_time > cutoff)
            .count();

        current_requests >= limit as usize
    }

    /// Get current request count within the window
    fn current_count(&self, now: Instant, window: Duration) -> u32 {
        let cutoff = now - window;
        self.requests
            .iter()
            .filter(|&&request_time| request_time > cutoff)
            .count() as u32
    }
}

/// Rate limiter implementation
#[derive(Debug, Clone)]
pub struct RateLimiter {
    entries: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    last_cleanup: Arc<RwLock<Instant>>,
    default_limit: u32,
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter with default settings
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
            default_limit: DEFAULT_REQUESTS_PER_MINUTE,
            window: RATE_LIMIT_WINDOW,
        }
    }

    /// Create a rate limiter with custom settings
    pub fn with_limit(limit: u32, window: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
            default_limit: limit,
            window,
        }
    }

    /// Check if a request is allowed for the given key (user ID, IP, etc.)
    pub fn is_allowed(&self, key: &str) -> Result<bool> {
        self.check_rate_limit(key, self.default_limit)
            .map_err(|e| anyhow::anyhow!("Rate limit check failed: {}", e))
    }

    /// Check if a request is allowed with a custom limit
    pub fn check_rate_limit(&self, key: &str, limit: u32) -> Result<bool, RateLimitError> {
        let now = Instant::now();

        let mut entries = self
            .entries
            .write()
            .map_err(|_| RateLimitError::LockError)?;

        let entry = entries
            .entry(key.to_string())
            .or_insert_with(RateLimitEntry::new);

        let allowed = !entry.would_exceed_limit(limit, now, self.window);

        if allowed {
            entry.add_request(now, self.window);
            // Trigger cleanup if needed
            drop(entries); // Release the write lock before cleanup
            let _ = self.cleanup_old_entries(); // Ignore cleanup errors
            Ok(true)
        } else {
            Err(RateLimitError::LimitExceeded)
        }
    }

    /// Record a request (for when you want to check and record separately)
    pub fn record_request(&self, key: &str) -> Result<()> {
        let now = Instant::now();

        let mut entries = self
            .entries
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire rate limiter lock"))?;

        let entry = entries
            .entry(key.to_string())
            .or_insert_with(RateLimitEntry::new);

        entry.add_request(now, self.window);
        Ok(())
    }

    /// Get current request count for a key
    pub fn get_current_count(&self, key: &str) -> Result<u32> {
        let now = Instant::now();

        let entries = self
            .entries
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire rate limiter lock"))?;

        let count = entries
            .get(key)
            .map(|entry| entry.current_count(now, self.window))
            .unwrap_or(0);

        Ok(count)
    }

    /// Get remaining requests for a key
    pub fn get_remaining(&self, key: &str, limit: u32) -> Result<u32> {
        let current = self.get_current_count(key)?;
        Ok(limit.saturating_sub(current))
    }

    /// Clean up old entries that haven't been used recently
    fn cleanup_old_entries(&self) -> Result<()> {
        let now = Instant::now();

        // Check if cleanup is needed
        {
            let last_cleanup = self
                .last_cleanup
                .read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cleanup lock"))?;

            if now.duration_since(*last_cleanup) < CLEANUP_INTERVAL {
                return Ok(()); // Cleanup not needed yet
            }
        }

        // Perform cleanup
        {
            let mut entries = self
                .entries
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire rate limiter lock"))?;

            let cutoff = now - self.window - CLEANUP_INTERVAL;
            entries.retain(|_, entry| entry.last_cleanup > cutoff);
        }

        // Update last cleanup time
        {
            let mut last_cleanup = self
                .last_cleanup
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cleanup lock"))?;
            *last_cleanup = now;
        }

        Ok(())
    }

    /// Reset rate limit for a specific key (admin function)
    pub fn reset_key(&self, key: &str) -> Result<()> {
        let mut entries = self
            .entries
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire rate limiter lock"))?;

        entries.remove(key);
        Ok(())
    }

    /// Get statistics about the rate limiter
    pub fn get_stats(&self) -> Result<RateLimiterStats> {
        let entries = self
            .entries
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire rate limiter lock"))?;

        let total_keys = entries.len();
        let total_requests: usize = entries.values().map(|entry| entry.requests.len()).sum();

        Ok(RateLimiterStats {
            total_keys,
            total_requests,
            window_seconds: self.window.as_secs(),
            default_limit: self.default_limit,
        })
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter statistics
#[derive(Debug)]
pub struct RateLimiterStats {
    pub total_keys: usize,
    pub total_requests: usize,
    pub window_seconds: u64,
    pub default_limit: u32,
}

/// Specialized rate limiters for different operation types
#[derive(Debug)]
pub struct RateLimiters {
    pub default: RateLimiter,
    pub auth: RateLimiter,
    pub dht: RateLimiter,
    pub messages: RateLimiter,
}

impl RateLimiters {
    pub fn new() -> Self {
        Self {
            default: RateLimiter::new(),
            auth: RateLimiter::with_limit(AUTH_REQUESTS_PER_MINUTE, RATE_LIMIT_WINDOW),
            dht: RateLimiter::with_limit(DHT_REQUESTS_PER_MINUTE, RATE_LIMIT_WINDOW),
            messages: RateLimiter::with_limit(MESSAGE_REQUESTS_PER_MINUTE, RATE_LIMIT_WINDOW),
        }
    }

    /// Check authentication rate limit
    pub fn check_auth(&self, key: &str) -> Result<bool> {
        self.auth.is_allowed(key)
    }

    /// Check DHT operation rate limit
    pub fn check_dht(&self, key: &str) -> Result<bool> {
        self.dht.is_allowed(key)
    }

    /// Check message operation rate limit
    pub fn check_messages(&self, key: &str) -> Result<bool> {
        self.messages.is_allowed(key)
    }
}

impl Default for RateLimiters {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro for checking rate limits in Tauri commands
#[macro_export]
macro_rules! check_rate_limit {
    ($rate_limiter:expr, $key:expr) => {
        match $rate_limiter.is_allowed($key) {
            Ok(true) => {}
            Ok(false) => return Err("Rate limit exceeded. Please try again later.".to_string()),
            Err(e) => return Err(format!("Rate limit check failed: {}", e)),
        }
    };

    ($rate_limiter:expr, $key:expr, $limit:expr) => {
        match $rate_limiter.check_rate_limit($key, $limit) {
            Ok(true) => {}
            Ok(false) => return Err("Rate limit exceeded. Please try again later.".to_string()),
            Err(e) => return Err(format!("Rate limit check failed: {}", e)),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::with_limit(2, Duration::from_secs(60));
        let key = "test_user";

        assert!(limiter.is_allowed(key).unwrap());
        assert!(limiter.is_allowed(key).unwrap());
        assert!(!limiter.is_allowed(key).unwrap()); // Should be blocked on third request
    }

    #[test]
    fn test_rate_limiter_window() {
        let limiter = RateLimiter::with_limit(1, Duration::from_millis(100));
        let key = "test_user";

        assert!(limiter.is_allowed(key).unwrap());
        assert!(!limiter.is_allowed(key).unwrap()); // Should be blocked

        thread::sleep(Duration::from_millis(150));
        assert!(limiter.is_allowed(key).unwrap()); // Should be allowed after window expires
    }

    #[test]
    fn test_rate_limiter_different_keys() {
        let limiter = RateLimiter::with_limit(1, Duration::from_secs(60));

        assert!(limiter.is_allowed("user1").unwrap());
        assert!(limiter.is_allowed("user2").unwrap()); // Different user should be allowed
        assert!(!limiter.is_allowed("user1").unwrap()); // Original user should be blocked
    }

    #[test]
    fn test_get_current_count() {
        let limiter = RateLimiter::new();
        let key = "test_user";

        assert_eq!(limiter.get_current_count(key).unwrap(), 0);

        limiter.record_request(key).unwrap();
        assert_eq!(limiter.get_current_count(key).unwrap(), 1);

        limiter.record_request(key).unwrap();
        assert_eq!(limiter.get_current_count(key).unwrap(), 2);
    }

    #[test]
    fn test_get_remaining() {
        let limiter = RateLimiter::with_limit(5, Duration::from_secs(60));
        let key = "test_user";

        assert_eq!(limiter.get_remaining(key, 5).unwrap(), 5);

        limiter.record_request(key).unwrap();
        assert_eq!(limiter.get_remaining(key, 5).unwrap(), 4);

        limiter.record_request(key).unwrap();
        assert_eq!(limiter.get_remaining(key, 5).unwrap(), 3);
    }

    #[test]
    fn test_rate_limiter_cleanup() {
        let limiter = RateLimiter::with_limit(1, Duration::from_millis(100));
        let key = "test_user";

        // Make a request
        assert!(limiter.is_allowed(key).unwrap());

        // Should be blocked immediately
        assert!(!limiter.is_allowed(key).unwrap());

        // Wait for cleanup interval (simulated)
        thread::sleep(Duration::from_millis(150));

        // Should be allowed again after window expires
        assert!(limiter.is_allowed(key).unwrap());
    }

    #[test]
    fn test_rate_limiter_stats() {
        let limiter = RateLimiter::new();
        let key1 = "user1";
        let key2 = "user2";

        // Make some requests
        limiter.record_request(key1).unwrap();
        limiter.record_request(key1).unwrap();
        limiter.record_request(key2).unwrap();

        let stats = limiter.get_stats().unwrap();
        assert_eq!(stats.total_keys, 2);
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.default_limit, DEFAULT_REQUESTS_PER_MINUTE);
    }

    #[test]
    fn test_rate_limiter_reset() {
        let limiter = RateLimiter::with_limit(2, Duration::from_secs(60));
        let key = "test_user";

        // Use up the limit
        assert!(limiter.is_allowed(key).unwrap());
        assert!(limiter.is_allowed(key).unwrap());
        assert!(!limiter.is_allowed(key).unwrap());

        // Reset the key
        limiter.reset_key(key).unwrap();

        // Should be allowed again
        assert!(limiter.is_allowed(key).unwrap());
    }

    #[test]
    fn test_rate_limiter_concurrent_access() {
        let limiter = RateLimiter::with_limit(10, Duration::from_secs(60));
        let key = "concurrent_user";
        let mut handles = vec![];

        // Spawn multiple threads making requests
        for _ in 0..5 {
            let limiter_clone = limiter.clone();
            let key_clone = key.to_string();
            let handle = thread::spawn(move || {
                for _ in 0..2 {
                    let _ = limiter_clone.is_allowed(&key_clone);
                    thread::sleep(Duration::from_millis(10));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Check that requests were properly tracked
        let current_count = limiter.get_current_count(key).unwrap();
        assert_eq!(current_count, 10); // 5 threads * 2 requests each
    }

    #[test]
    fn test_rate_limiter_edge_cases() {
        let limiter = RateLimiter::with_limit(1, Duration::from_secs(60));

        // Test with empty key
        assert!(limiter.is_allowed("").unwrap());
        assert!(!limiter.is_allowed("").unwrap());

        // Test with very long key
        let long_key = "a".repeat(1000);
        assert!(limiter.is_allowed(&long_key).unwrap());
        assert!(!limiter.is_allowed(&long_key).unwrap());

        // Test with special characters in key
        let special_key = "user@domain.com!#$%^&*()";
        assert!(limiter.is_allowed(special_key).unwrap());
        assert!(!limiter.is_allowed(special_key).unwrap());
    }

    #[test]
    fn test_rate_limiter_zero_limit() {
        let limiter = RateLimiter::with_limit(0, Duration::from_secs(60));
        let key = "zero_limit_user";

        // Should always be blocked with zero limit
        assert!(!limiter.is_allowed(key).unwrap());
        assert!(!limiter.is_allowed(key).unwrap());
    }

    #[test]
    fn test_rate_limiter_very_short_window() {
        let limiter = RateLimiter::with_limit(1, Duration::from_millis(1));
        let key = "short_window_user";

        assert!(limiter.is_allowed(key).unwrap());
        assert!(!limiter.is_allowed(key).unwrap());

        // Wait for window to expire
        thread::sleep(Duration::from_millis(10));
        assert!(limiter.is_allowed(key).unwrap());
    }
}

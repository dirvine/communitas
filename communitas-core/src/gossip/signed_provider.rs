// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Provider Summary Anti-Spam
//!
//! saorsa-gossip-rendezvous::ProviderSummary has built-in sign/verify methods.
//! This module adds rate limiting and collection helpers.
//!
//! ## Note
//! ProviderSummary signature verification is handled by saorsa-gossip-rendezvous.
//! We just add:
//! - Rate limiting per target (prevent flooding)
//! - Statistics tracking
//! - TTL enforcement

use saorsa_gossip_rendezvous::ProviderSummary;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::debug;

/// Rate limiter for provider summaries
///
/// Tracks message rates per target to prevent flooding
pub struct ProviderRateLimiter {
    /// Message counts per target (target_id → (count, window_start))
    #[allow(clippy::type_complexity)]
    counts: Arc<RwLock<HashMap<[u8; 32], (u32, u64)>>>,

    /// Maximum messages per window
    max_per_window: u32,

    /// Window duration in milliseconds
    window_ms: u64,
}

impl ProviderRateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_per_window` - Maximum messages per time window (default: 10)
    /// * `window_ms` - Window duration in milliseconds (default: 1000)
    pub fn new(max_per_window: u32, window_ms: u64) -> Self {
        Self {
            counts: Arc::new(RwLock::new(HashMap::new())),
            max_per_window,
            window_ms,
        }
    }

    /// Check if a message should be accepted
    ///
    /// Returns true if within rate limit, false if should be dropped
    pub async fn check_and_update(&self, target_id: &[u8; 32]) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        let mut counts = self.counts.write().await;

        let (count, window_start) = counts.entry(*target_id).or_insert((0, now));

        // Check if we're in a new window
        if now - *window_start > self.window_ms {
            // Reset window
            *count = 1;
            *window_start = now;
            return true;
        }

        // Check if over limit
        if *count >= self.max_per_window {
            debug!(
                "Rate limit exceeded for target {:?}",
                hex::encode(target_id)
            );
            return false;
        }

        // Increment count
        *count += 1;
        true
    }

    /// Clean up old entries (call periodically)
    pub async fn cleanup(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        let mut counts = self.counts.write().await;
        counts.retain(|_, (_, window_start)| {
            now - *window_start < self.window_ms * 2 // Keep entries for 2 windows
        });
    }
}

impl Default for ProviderRateLimiter {
    fn default() -> Self {
        Self::new(10, 1000) // 10 messages per second
    }
}

/// Provider summary collector with anti-spam
///
/// Collects ProviderSummary messages for a target with rate limiting.
/// Signature verification is delegated to rendezvous client.
pub struct ProviderCollector {
    /// Target we're collecting for
    target_id: [u8; 32],

    /// Verified provider summaries
    providers: Arc<RwLock<Vec<ProviderSummary>>>,

    /// Rate limiter
    rate_limiter: Arc<ProviderRateLimiter>,

    /// Statistics
    stats: Arc<RwLock<CollectorStats>>,
}

/// Collector statistics
#[derive(Debug, Clone, Default)]
pub struct CollectorStats {
    pub total_received: u64,
    pub signature_valid: u64,
    pub signature_invalid: u64,
    pub rate_limited: u64,
    pub expired: u64,
    pub accepted: u64,
}

impl ProviderCollector {
    /// Create a new collector
    ///
    /// # Arguments
    /// * `target_id` - The target (BLAKE3 hash of public key) we're collecting for
    pub fn new(target_id: [u8; 32]) -> Self {
        Self {
            target_id,
            providers: Arc::new(RwLock::new(Vec::new())),
            rate_limiter: Arc::new(ProviderRateLimiter::default()),
            stats: Arc::new(RwLock::new(CollectorStats::default())),
        }
    }

    /// Process incoming provider summary with rate limiting
    ///
    /// NOTE: Signature verification should be done by caller before passing to this method.
    /// This focuses on rate limiting and deduplication.
    ///
    /// # Returns
    /// true if accepted, false if rejected (rate limited/wrong target)
    pub async fn process(&self, summary: ProviderSummary, is_valid: bool) -> bool {
        let mut stats = self.stats.write().await;
        stats.total_received += 1;
        drop(stats);

        // Early filter: target must match
        if summary.target != self.target_id {
            return false; // Wrong target, ignore silently
        }

        // Check signature status (caller verifies)
        if !is_valid {
            let mut stats = self.stats.write().await;

            // Check if expired
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                .as_millis() as u64;

            if summary.exp <= now {
                stats.expired += 1;
            } else {
                stats.signature_invalid += 1;
            }
            return false;
        }

        // Rate limit check
        if !self.rate_limiter.check_and_update(&self.target_id).await {
            let mut stats = self.stats.write().await;
            stats.rate_limited += 1;
            return false;
        }

        // Track valid signature
        let mut stats = self.stats.write().await;
        stats.signature_valid += 1;
        drop(stats);

        // Add to collection
        let mut providers = self.providers.write().await;

        // Deduplicate: remove old entry from same provider
        providers.retain(|p| p.provider != summary.provider);

        // Add new entry
        providers.push(summary);

        let mut stats = self.stats.write().await;
        stats.accepted += 1;

        true
    }

    /// Get collected providers
    pub async fn get_providers(&self) -> Vec<ProviderSummary> {
        self.providers.read().await.clone()
    }

    /// Get statistics
    pub async fn stats(&self) -> CollectorStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_gossip_types::PeerId;

    fn create_test_summary(
        target: [u8; 32],
        provider: PeerId,
        validity_ms: u64,
    ) -> ProviderSummary {
        ProviderSummary::new(
            target,
            provider,
            vec![saorsa_gossip_rendezvous::Capability::Site],
            validity_ms,
        )
    }

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = ProviderRateLimiter::new(3, 1000); // 3 per second
        let target = [1u8; 32];

        // First 3 should pass
        assert!(limiter.check_and_update(&target).await);
        assert!(limiter.check_and_update(&target).await);
        assert!(limiter.check_and_update(&target).await);

        // 4th should be rejected
        assert!(!limiter.check_and_update(&target).await);
    }

    #[tokio::test]
    async fn test_rate_limiter_window_reset() {
        let limiter = ProviderRateLimiter::new(2, 100); // 2 per 100ms
        let target = [2u8; 32];

        // First 2 should pass
        assert!(limiter.check_and_update(&target).await);
        assert!(limiter.check_and_update(&target).await);

        // 3rd rejected
        assert!(!limiter.check_and_update(&target).await);

        // Wait for window to reset
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Should accept again
        assert!(limiter.check_and_update(&target).await);
    }

    #[tokio::test]
    async fn test_collector_accepts_valid_summary() {
        let target = [1u8; 32];
        let collector = ProviderCollector::new(target);

        let summary = create_test_summary(target, PeerId::new([2u8; 32]), 60000);

        // Process as valid (signature verification done elsewhere)
        assert!(collector.process(summary, true).await);

        let providers = collector.get_providers().await;
        assert_eq!(providers.len(), 1);

        let stats = collector.stats().await;
        assert_eq!(stats.accepted, 1);
        assert_eq!(stats.signature_valid, 1);
    }

    #[tokio::test]
    async fn test_collector_rejects_invalid_summary() {
        let target = [1u8; 32];
        let collector = ProviderCollector::new(target);

        let summary = create_test_summary(target, PeerId::new([2u8; 32]), 60000);

        // Process as invalid (signature verification failed)
        assert!(!collector.process(summary, false).await);

        let stats = collector.stats().await;
        assert_eq!(stats.accepted, 0);
        assert_eq!(stats.signature_invalid, 1);
    }

    #[tokio::test]
    async fn test_collector_enforces_rate_limit() {
        let target = [1u8; 32];
        let collector = ProviderCollector::new(target);

        // Send 15 valid summaries rapidly
        let mut accepted = 0;
        for i in 0..15 {
            let summary = create_test_summary(target, PeerId::new([i as u8; 32]), 60000);
            if collector.process(summary, true).await {
                accepted += 1;
            }
        }

        // Should only accept 10 (default rate limit)
        assert_eq!(accepted, 10);

        let stats = collector.stats().await;
        assert_eq!(stats.rate_limited, 5);
    }

    #[tokio::test]
    async fn test_collector_deduplicates_by_provider() {
        let target = [1u8; 32];
        let collector = ProviderCollector::new(target);
        let provider = PeerId::new([2u8; 32]);

        // Send 3 summaries from same provider
        for _ in 0..3 {
            let summary = create_test_summary(target, provider, 60000);
            collector.process(summary, true).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Should only have 1 provider (latest)
        let providers = collector.get_providers().await;
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider, provider);
    }
}

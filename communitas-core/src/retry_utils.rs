// Copyright (c) 2025 Saorsa Labs Limited
//
// Retry utilities with exponential backoff for resilient networking
//
// Implements adaptive retry behavior as specified in MESH_CAPABILITIES.md §3.2
// to handle intermittent connectivity and network degradation gracefully.

use std::time::Duration;
use tokio_retry::Retry;
use tokio_retry::strategy::{ExponentialBackoff, jitter};
use tracing::{debug, warn};

/// Default retry configuration for network operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Initial delay before first retry (default: 100ms)
    pub initial_delay: Duration,

    /// Maximum delay between retries (default: 60 seconds)
    pub max_delay: Duration,

    /// Maximum number of retry attempts (default: 10)
    pub max_retries: usize,

    /// Backoff multiplier (default: 2.0 for exponential)
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            max_retries: 10,
            backoff_multiplier: 2.0,
        }
    }
}

/// Backoff configuration
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    pub initial: Duration,
    pub max: Duration,
    pub multiplier: f64,
}

impl BackoffConfig {
    pub fn into_strategy(self) -> impl Iterator<Item = Duration> {
        let mut current = self.initial.as_millis() as u64;
        let max_ms = self.max.as_millis() as u64;
        
        std::iter::from_fn(move || {
            let delay = Duration::from_millis(current);
            current = (current as f64 * self.multiplier) as u64;
            if current > max_ms {
                current = max_ms;
            }
            
            let jitter = (rand::random::<f64>() * 0.1 - 0.05) * current as f64;
            Some(Duration::from_millis((current as f64 + jitter) as u64))
        })
    }
}

/// Result type for retry operations
pub type RetryResult<T> = Result<T, anyhow::Error>;

/// Retry an async operation with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    config: RetryConfig,
) -> RetryResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = RetryResult<T>>,
{
    let strategy = config.build_strategy();

    Retry::spawn(strategy, operation).await
}

impl RetryConfig {
    /// Create config for fast retries (low latency operations)
    pub fn fast() -> Self {
        Self {
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(5),
            max_retries: 5,
            backoff_multiplier: 2.0,
        }
    }

    /// Create config for slow retries (expensive operations)
    pub fn slow() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), // 5 minutes
            max_retries: 15,
            backoff_multiplier: 2.0,
        }
    }

    /// Create config for critical operations (more attempts)
    pub fn critical() -> Self {
        Self {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(120), // 2 minutes
            max_retries: 20,
            backoff_multiplier: 2.0,
        }
    }

    /// Build tokio-retry strategy from config
    pub fn build_strategy(&self) -> impl Iterator<Item = Duration> {
        let backoff = ExponentialBackoff::from_millis(self.initial_delay.as_millis() as u64)
            .max_delay(self.max_delay)
            .take(self.max_retries + 1);

        backoff.map(jitter)
    }
}

/// Retry a network dial operation with logging
pub async fn retry_dial<F, Fut, T, E>(
    peer_id: &str,
    config: RetryConfig,
    mut dial_fn: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let strategy = config.build_strategy();

    for delay in strategy {
        attempt += 1;

        match dial_fn().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Dial to {} succeeded on attempt {}", peer_id, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt <= config.max_retries {
                    debug!(
                        "Dial to {} failed (attempt {}): {} - retrying in {:?}",
                        peer_id, attempt, e, delay
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    warn!(
                        "Dial to {} failed after {} attempts: {}",
                        peer_id, attempt, e
                    );
                    return Err(e);
                }
            }
        }
    }

    // This should never be reached due to take() on strategy
    unreachable!("Retry strategy exhausted")
}

/// Retry a coordinator discovery operation with appropriate backoff
pub async fn retry_coordinator_discovery<F, Fut, T>(
    config: RetryConfig,
    operation: F,
) -> RetryResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = RetryResult<T>>,
{
    retry_with_backoff(operation, config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_eventually() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            max_retries: 5,
            backoff_multiplier: 2.0,
        };

        let result = retry_with_backoff(|| {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(anyhow::anyhow!("Not yet"))
                } else {
                    Ok("Success")
                }
            }
        }, config)
        .await;

        assert_eq!(result, Ok("Success"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            max_attempts: 3,
            jitter: false,
        };

        let result = retry_with_backoff(config, || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("Always fails")
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_config_presets() {
        let fast = RetryConfig::fast();
        assert_eq!(fast.initial_delay, Duration::from_millis(50));
        assert_eq!(fast.max_attempts, 5);

        let slow = RetryConfig::slow();
        assert_eq!(slow.initial_delay, Duration::from_secs(1));
        assert_eq!(slow.max_attempts, 15);

        let critical = RetryConfig::critical();
        assert_eq!(critical.max_attempts, 20);
    }

    #[tokio::test]
    async fn test_retry_dial_with_logging() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            max_attempts: 3,
            jitter: false,
        };

        let result = retry_dial("test-peer", config, || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 1 {
                    Err("Connection refused")
                } else {
                    Ok(())
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}

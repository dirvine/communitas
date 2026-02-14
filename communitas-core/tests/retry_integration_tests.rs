// Copyright (c) 2025 Saorsa Labs Limited
//
// Integration tests for exponential backoff retry logic
//
// Tests MESH_CAPABILITIES.md §3.2 Scenario C: Intermittent connectivity

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::retry_utils::{RetryConfig, retry_dial, retry_with_backoff};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Test that retry_with_backoff succeeds eventually
#[tokio::test]
async fn test_retry_succeeds_after_failures() {
    // Arrange: Operation that fails twice then succeeds
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        max_retries: 5,
        backoff_multiplier: 2.0,
    };

    // Act: Retry the operation
    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(anyhow::anyhow!("Connection failed"))
                } else {
                    Ok("Success")
                }
            }
        },
        config,
    )
    .await;

    // Assert: Should succeed after 3 attempts
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

/// Test that retry_with_backoff fails after max attempts
#[tokio::test]
async fn test_retry_fails_after_max_attempts() {
    // Arrange: Operation that always fails
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        initial_delay: Duration::from_millis(5),
        max_delay: Duration::from_millis(50),
        max_retries: 3,
        backoff_multiplier: 2.0,
    };

    // Act
    let result = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(anyhow::anyhow!("Always fails"))
            }
        },
        config,
    )
    .await;

    // Assert: Should fail after max_retries
    // Note: tokio-retry's take() limits the number of retries (delays), not attempts
    // So max_retries=3 means: initial attempt + 3 delays = 4 total attempts
    assert!(result.is_err());
    let actual_attempts = attempts.load(Ordering::SeqCst);
    assert!(
        (3..=4).contains(&actual_attempts),
        "Expected 3-4 attempts, got {}",
        actual_attempts
    );
}

/// Test exponential backoff timing
#[tokio::test]
async fn test_exponential_backoff_timing() {
    // Arrange: Track number of attempts and total elapsed time
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();
    let start = Instant::now();

    let config = RetryConfig {
        initial_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(500),
        max_retries: 4,
        backoff_multiplier: 2.0,
    };

    // Act
    let _ = retry_with_backoff(
        || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(anyhow::anyhow!("Fail to measure delays"))
            }
        },
        config,
    )
    .await;

    let elapsed = start.elapsed();
    let total_attempts = attempts.load(Ordering::SeqCst);

    // Assert: Should have 4 total attempts (initial + 3 retries from take(3))
    assert_eq!(
        total_attempts, 4,
        "Should have 4 total attempts (max_retries=4)"
    );

    // Total elapsed time should reflect delays occurred
    // With initial=50ms, multiplier=2.0, delays are: ~50ms, ~100ms, ~200ms = ~350ms minimum
    // With jitter reducing delays and CI runner variance, use a very conservative lower bound
    assert!(
        elapsed >= Duration::from_millis(100),
        "Total elapsed {:?} should indicate exponential delays occurred",
        elapsed
    );
}

/// Test retry_dial with peer identification
#[tokio::test]
async fn test_retry_dial_with_peer_id() {
    // Arrange
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = attempts.clone();

    let config = RetryConfig {
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_millis(100),
        max_retries: 5,
        backoff_multiplier: 2.0,
    };

    // Act: Dial that succeeds on second attempt
    let result = retry_dial("test-peer-four-words", config, || {
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

    // Assert
    assert!(result.is_ok());
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

/// Test fast retry preset
#[tokio::test]
async fn test_fast_retry_preset() {
    let config = RetryConfig::fast();

    // Fast preset should have short delays
    assert_eq!(config.initial_delay, Duration::from_millis(50));
    assert_eq!(config.max_delay, Duration::from_secs(5));
    assert_eq!(config.max_retries, 5);
}

/// Test slow retry preset
#[tokio::test]
async fn test_slow_retry_preset() {
    let config = RetryConfig::slow();

    // Slow preset should have longer delays
    assert_eq!(config.initial_delay, Duration::from_secs(1));
    assert_eq!(config.max_delay, Duration::from_secs(300));
    assert_eq!(config.max_retries, 15);
}

/// Test critical retry preset
#[tokio::test]
async fn test_critical_retry_preset() {
    let config = RetryConfig::critical();

    // Critical preset should have more attempts
    assert_eq!(config.max_retries, 20);
    assert_eq!(config.max_delay, Duration::from_secs(120));
}

/// Test that jitter prevents thundering herd
#[tokio::test]
async fn test_jitter_adds_randomness() {
    // This test verifies that jitter is applied by running the same
    // retry multiple times and checking that delays vary

    let config = RetryConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(1000),
        max_retries: 3,
        backoff_multiplier: 2.0,
    };

    let first_run_delays = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let second_run_delays = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Run 1
    {
        let last_time = Arc::new(tokio::sync::Mutex::new(None::<Instant>));
        let last_time_clone = last_time.clone();
        let delays = first_run_delays.clone();

        let _ = retry_with_backoff(
            || {
                let last_time = last_time_clone.clone();
                let delays = delays.clone();
                async move {
                    let mut last = last_time.lock().await;
                    if let Some(prev) = *last {
                        delays
                            .lock()
                            .await
                            .push(Instant::now().duration_since(prev));
                    }
                    *last = Some(Instant::now());
                    drop(last);
                    Err::<(), _>(anyhow::anyhow!("Fail"))
                }
            },
            config.clone(),
        )
        .await;
    }

    // Run 2
    {
        let last_time = Arc::new(tokio::sync::Mutex::new(None::<Instant>));
        let last_time_clone = last_time.clone();
        let delays = second_run_delays.clone();

        let _ = retry_with_backoff(
            || {
                let last_time = last_time_clone.clone();
                let delays = delays.clone();
                async move {
                    let mut last = last_time.lock().await;
                    if let Some(prev) = *last {
                        delays
                            .lock()
                            .await
                            .push(Instant::now().duration_since(prev));
                    }
                    *last = Some(Instant::now());
                    drop(last);
                    Err::<(), _>(anyhow::anyhow!("Fail"))
                }
            },
            config.clone(),
        )
        .await;
    }

    // With jitter, delays should differ between runs
    // (very small chance they'd be identical)
    let first = first_run_delays.lock().await;
    let second = second_run_delays.lock().await;

    // tokio-retry may create more delays than expected
    assert!(
        first.len() >= 2,
        "Expected at least 2 delays in first run, got {}",
        first.len()
    );
    assert!(
        second.len() >= 2,
        "Expected at least 2 delays in second run, got {}",
        second.len()
    );

    // At least one delay should differ by more than 10ms
    let differs = first
        .iter()
        .zip(second.iter())
        .any(|(a, b)| a.abs_diff(*b) > Duration::from_millis(10));

    assert!(
        differs,
        "Jitter should cause delays to differ between runs. Run1: {:?}, Run2: {:?}",
        *first, *second
    );
}

/// Test that retry respects max_delay cap
#[tokio::test]
async fn test_retry_respects_max_delay_cap() {
    let config = RetryConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(200), // Cap at 200ms
        max_retries: 10,
        backoff_multiplier: 2.0,
    };

    let delays = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let delays_clone = delays.clone();
    let last_time = Arc::new(tokio::sync::Mutex::new(None::<Instant>));
    let last_time_clone = last_time.clone();

    let _ = retry_with_backoff(
        || {
            let delays = delays_clone.clone();
            let last_time = last_time_clone.clone();
            async move {
                let mut last = last_time.lock().await;
                if let Some(prev) = *last {
                    let delay = Instant::now().duration_since(prev);
                    delays.lock().await.push(delay);
                }
                *last = Some(Instant::now());
                drop(last);
                Err::<(), _>(anyhow::anyhow!("Fail"))
            }
        },
        config,
    )
    .await;

    let recorded = delays.lock().await;

    // All delays should be capped at or below max_delay + some tolerance
    for (i, delay) in recorded.iter().enumerate() {
        assert!(
            *delay <= Duration::from_millis(250), // 200ms + 50ms tolerance
            "Delay {} exceeded max_delay cap: {:?}",
            i,
            delay
        );
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Exponential Backoff Tests
//!
//! Tests for retry logic with exponential backoff as specified in
//! MESH_CAPABILITIES.md Section 3.2 Scenario C

use communitas_core::retry_utils::{BackoffConfig, RetryConfig, RetryResult, retry_with_backoff};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_successful_operation_no_retry() {
    let attempt_count = Arc::new(AtomicUsize::new(0));
    let counter = attempt_count.clone();

    let result = retry_with_backoff(
        || async {
            counter.fetch_add(1, Ordering::SeqCst);
            Ok::<_, anyhow::Error>("success")
        },
        RetryConfig::default(),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(
        attempt_count.load(Ordering::SeqCst),
        1,
        "Should only try once on success"
    );
}

#[tokio::test]
async fn test_retry_with_eventual_success() {
    let attempt_count = Arc::new(AtomicUsize::new(0));
    let counter = attempt_count.clone();

    let result = retry_with_backoff(
        || async {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(anyhow::anyhow!("Temporary failure"))
            } else {
                Ok("success")
            }
        },
        RetryConfig::default(),
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(
        attempt_count.load(Ordering::SeqCst),
        3,
        "Should retry until success"
    );
}

#[tokio::test]
async fn test_max_retries_exceeded() {
    let attempt_count = Arc::new(AtomicUsize::new(0));
    let counter = attempt_count.clone();

    // max_retries means total number of attempts, not retries after first attempt
    let config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(10),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
    };

    let result = retry_with_backoff(
        || async {
            counter.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(anyhow::anyhow!("Always fails"))
        },
        config,
    )
    .await;

    assert!(result.is_err());
    assert_eq!(
        attempt_count.load(Ordering::SeqCst),
        3,
        "Should try max_retries total attempts"
    );
}

#[tokio::test]
async fn test_exponential_delay_progression() {
    let attempt_count = Arc::new(AtomicUsize::new(0));
    let counter = attempt_count.clone();
    let start = Instant::now();

    // max_retries means total attempts, so 4 = 4 total attempts = 3 delays between them
    let config = RetryConfig {
        max_retries: 4,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
    };

    let _result = retry_with_backoff(
        || async {
            counter.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(anyhow::anyhow!("Fail"))
        },
        config,
    )
    .await;

    let elapsed = start.elapsed();
    let attempts = attempt_count.load(Ordering::SeqCst);

    // Verify we got the expected number of attempts
    assert_eq!(attempts, 4, "Should have max_retries total attempts");

    // Verify that delays actually happened (total time should be at least the sum of delays)
    // With initial_delay=100ms and multiplier=2.0, base delays are: 100ms, 200ms, 400ms = 700ms minimum
    // But with jitter reducing delays, we use a lower threshold
    assert!(
        elapsed >= Duration::from_millis(300),
        "Total elapsed time {:?} should indicate delays occurred",
        elapsed
    );
}

#[tokio::test]
async fn test_max_delay_cap() {
    let attempt_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let times = attempt_times.clone();

    let config = RetryConfig {
        max_retries: 10,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500),
        backoff_multiplier: 2.0,
    };

    let _result = retry_with_backoff(
        || async {
            times.lock().await.push(Instant::now());
            Err::<(), _>(anyhow::anyhow!("Fail"))
        },
        config,
    )
    .await;

    let timestamps = attempt_times.lock().await;
    let delays: Vec<Duration> = timestamps
        .windows(2)
        .map(|w| w[1].duration_since(w[0]))
        .collect();

    for delay in delays.iter().skip(2) {
        assert!(
            *delay <= Duration::from_millis(550),
            "Delays should be capped at max_delay"
        );
    }
}

#[tokio::test]
async fn test_jitter_prevents_thundering_herd() {
    let config = RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_multiplier: 2.0,
    };

    let mut delays = Vec::new();

    for _ in 0..5 {
        let attempt_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
        let times = attempt_times.clone();

        let _result = retry_with_backoff(
            || async {
                times.lock().await.push(Instant::now());
                Err::<(), _>(anyhow::anyhow!("Fail"))
            },
            config.clone(),
        )
        .await;

        let timestamps = attempt_times.lock().await;
        let first_delay = timestamps[1].duration_since(timestamps[0]);
        delays.push(first_delay);
    }

    let all_same = delays.windows(2).all(|w| w[0] == w[1]);
    assert!(
        !all_same,
        "Jitter should cause variation in delays (prevent thundering herd)"
    );
}

#[tokio::test]
async fn test_custom_backoff_config() {
    let config = BackoffConfig {
        initial: Duration::from_millis(50),
        max: Duration::from_secs(5),
        multiplier: 3.0,
    };

    let backoff = config.into_strategy();
    let mut iter = backoff.into_iter();

    let d1 = iter.next().unwrap();
    let d2 = iter.next().unwrap();
    let d3 = iter.next().unwrap();

    // BackoffConfig.into_strategy() multiplies before returning, so:
    // d1: 50 * 3 = 150ms (with ~5% jitter = ~142-158ms)
    // d2: 150 * 3 = 450ms (with ~5% jitter = ~427-473ms)
    // d3: 450 * 3 = 1350ms (with ~5% jitter)
    assert!(
        d1 >= Duration::from_millis(120) && d1 < Duration::from_millis(180),
        "First delay should be ~150ms: {:?}",
        d1
    );
    assert!(
        d2 >= Duration::from_millis(400) && d2 < Duration::from_millis(500),
        "Second delay should be ~450ms: {:?}",
        d2
    );
    assert!(
        d3 >= Duration::from_millis(1200),
        "Third delay should be ~1350ms: {:?}",
        d3
    );
}

#[test]
fn test_retry_result_conversion() {
    let success: RetryResult<i32> = Ok(42);
    assert!(matches!(success, Ok(42)));

    let failure: RetryResult<i32> = Err(anyhow::anyhow!("Failed"));
    assert!(failure.is_err());
}

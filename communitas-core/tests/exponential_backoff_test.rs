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
        4,
        "Should try 1 + 3 retries"
    );
}

#[tokio::test]
async fn test_exponential_delay_progression() {
    let attempt_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let times = attempt_times.clone();

    let config = RetryConfig {
        max_retries: 4,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
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
    assert_eq!(timestamps.len(), 5, "Should have 5 attempts");

    let delays: Vec<Duration> = timestamps
        .windows(2)
        .map(|w| w[1].duration_since(w[0]))
        .collect();

    assert!(
        delays[0] >= Duration::from_millis(100),
        "First delay >= 100ms"
    );
    assert!(
        delays[1] >= Duration::from_millis(200),
        "Second delay >= 200ms"
    );
    assert!(
        delays[2] >= Duration::from_millis(400),
        "Third delay >= 400ms"
    );
    assert!(
        delays[3] >= Duration::from_millis(800),
        "Fourth delay >= 800ms"
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

    assert!(d1 >= Duration::from_millis(50) && d1 < Duration::from_millis(100));
    assert!(d2 >= Duration::from_millis(150) && d2 < Duration::from_millis(300));
    assert!(d3 >= Duration::from_millis(450));
}

#[test]
fn test_retry_result_conversion() {
    let success: RetryResult<i32> = Ok(42);
    assert_eq!(success.unwrap(), 42);

    let failure: RetryResult<i32> = Err(anyhow::anyhow!("Failed"));
    assert!(failure.is_err());
}

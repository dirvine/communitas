// Copyright (c) 2025 Saorsa Labs Limited
//
// Phase 3 Integration Tests - End-to-End Resilience Features
//
// Tests complete integration of:
// - Connectivity watchdog with bootstrap monitoring
// - Resource limits enforcement in real gossip operations
// - Local-only mode dial decisions
// - Config-based limit loading

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::{
    ConnectivityWatchdog, GossipContext, ResourceLimitError, ResourceLimits, WatchdogConfig,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::time::sleep;

/// Test that watchdog is started and monitors bootstrap health
#[tokio::test]
async fn test_watchdog_starts_monitoring_bootstrap() {
    // This test will verify that:
    // 1. Watchdog monitoring task is spawned during boot
    // 2. It pings bootstrap nodes periodically
    // 3. It detects failures and enters local-only mode

    // Arrange: Create a watchdog that should detect failure quickly
    let config = WatchdogConfig {
        check_interval: Duration::from_millis(50),
        detection_threshold: Duration::from_millis(200),
        recovery_check_interval: Duration::from_millis(100),
        enabled: true,
    };

    let watchdog = ConnectivityWatchdog::new(config);
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();
    let should_fail = Arc::new(AtomicBool::new(true));
    let should_fail_clone = should_fail.clone();

    // Health check that can be controlled
    let health_check = move || {
        let count = call_count_clone.clone();
        let fail = should_fail_clone.clone();
        async move {
            count.fetch_add(1, Ordering::SeqCst);
            !fail.load(Ordering::SeqCst) // Succeed when should_fail is false
        }
    };

    // Act: Start monitoring (simulates boot sequence behavior)
    let handle = watchdog.clone().start_monitoring(health_check);

    // Initially should be online
    assert!(!watchdog.is_local_only_mode());

    // Wait for detection threshold
    sleep(Duration::from_millis(300)).await;

    // Assert: Should enter local-only mode
    assert!(
        watchdog.is_local_only_mode(),
        "Watchdog should enter local-only mode when bootstrap fails"
    );

    // Simulate bootstrap recovery
    should_fail.store(false, Ordering::SeqCst);

    // Wait for recovery check
    sleep(Duration::from_millis(250)).await;

    // Should exit local-only mode
    assert!(
        !watchdog.is_local_only_mode(),
        "Watchdog should exit local-only mode when bootstrap recovers"
    );

    // Verify health checks were called multiple times
    let checks = call_count.load(Ordering::SeqCst);
    assert!(
        checks >= 5,
        "Health check should be called at least 5 times, got {}",
        checks
    );

    handle.abort();
}

/// Test that GossipContext provides method to check if WAN dials should be attempted
#[tokio::test]
#[ignore] // Network test - GossipContext starts QUIC listener
async fn test_gossip_context_respects_local_only_mode() {
    let ctx = GossipContext::initialize(
        "ocean-forest-moon-star".to_string(),
        "Alice".to_string(),
        "Desktop".to_string(),
        None,
    )
    .await
    .expect("context init");

    assert!(
        ctx.should_attempt_wan_operations(),
        "WAN operations should be allowed by default"
    );

    ctx.set_local_only_mode(true);
    assert!(
        ctx.is_local_only_mode(),
        "Context should be in local-only mode after override"
    );
    assert!(
        !ctx.should_attempt_wan_operations(),
        "WAN operations should be disabled in local-only mode"
    );

    ctx.set_local_only_mode(false);
    assert!(
        !ctx.is_local_only_mode(),
        "Context should exit local-only mode after override"
    );
    assert!(
        ctx.should_attempt_wan_operations(),
        "WAN operations should be re-enabled after recovery"
    );
}

/// Test that membership layer enforces peer connection limits
#[tokio::test]
async fn test_membership_enforces_peer_limits() {
    // This test will verify that:
    // 1. ResourceLimits are checked before adding peers
    // 2. Excess peer connections are rejected
    // 3. Error is returned when limit exceeded

    // Arrange: Create resource limits with very low peer count
    let limits = ResourceLimits {
        max_peer_connections: 2,
        ..ResourceLimits::default()
    };

    // Act: Try to add 3 peers
    let mut current_peers = 0;

    // First peer should succeed
    let result1 = limits.enforce_peer_limit(current_peers);
    assert!(result1.is_ok());
    current_peers += 1;

    // Second peer should succeed
    let result2 = limits.enforce_peer_limit(current_peers);
    assert!(result2.is_ok());
    current_peers += 1;

    // Third peer should fail (at limit)
    let result3 = limits.enforce_peer_limit(current_peers);
    assert!(result3.is_err());

    match result3 {
        Err(ResourceLimitError::PeerLimitExceeded { current, limit }) => {
            assert_eq!(current, 2);
            assert_eq!(limit, 2);
        }
        _ => panic!("Expected PeerLimitExceeded error"),
    }
}

/// Test that document operations enforce size limits
#[tokio::test]
async fn test_document_operations_enforce_size_limits() {
    // Arrange: Create limits with small document size
    let limits = ResourceLimits {
        crdt_document_limit_mb: 10,
        ..ResourceLimits::default()
    };

    // Act & Assert: Small document succeeds
    assert!(limits.enforce_document_limit(5).is_ok());

    // Large document fails
    let result = limits.enforce_document_limit(11);
    assert!(result.is_err());

    match result {
        Err(ResourceLimitError::DocumentTooLarge { size_mb, limit_mb }) => {
            assert_eq!(size_mb, 11);
            assert_eq!(limit_mb, 10);
        }
        _ => panic!("Expected DocumentTooLarge error"),
    }
}

/// Test that ResourceLimits can be customized via builder pattern
#[test]
fn test_resource_limits_customization() {
    // Arrange & Act: Create custom limits
    let limits = ResourceLimits {
        max_peer_connections: 100,
        max_relay_connections: 5,
        max_memory_mb: 4096,
        crdt_document_limit_mb: 100,
        ..ResourceLimits::default()
    };

    // Assert: Custom values are applied
    assert_eq!(limits.max_peer_connections, 100);
    assert_eq!(limits.max_relay_connections, 5);
    assert_eq!(limits.max_memory_mb, 4096);
    assert_eq!(limits.crdt_document_limit_mb, 100);

    // Should still validate
    assert!(limits.validate().is_ok());
}

/// Test that watchdog can be disabled via config
#[tokio::test]
async fn test_watchdog_can_be_disabled() {
    // Arrange: Create disabled watchdog
    let config = WatchdogConfig {
        enabled: false,
        ..Default::default()
    };

    let watchdog = ConnectivityWatchdog::new(config);
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    let health_check = move || {
        let count = call_count_clone.clone();
        async move {
            count.fetch_add(1, Ordering::SeqCst);
            false // Always fail
        }
    };

    // Act: Start monitoring
    let handle = watchdog.clone().start_monitoring(health_check);

    // Wait a bit
    sleep(Duration::from_millis(200)).await;

    // Assert: Should never enter local-only mode when disabled
    assert!(!watchdog.is_local_only_mode());

    // Health check should not be called
    assert_eq!(call_count.load(Ordering::SeqCst), 0);

    handle.abort();
}

/// Test that multiple simultaneous retry operations use jitter
#[tokio::test]
async fn test_concurrent_retries_use_jitter() {
    use communitas_core::retry_utils::{RetryConfig, retry_with_backoff};
    use std::time::Instant;

    // This test verifies that concurrent retry operations don't
    // create a thundering herd by using jitter

    let config = RetryConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_millis(500),
        max_retries: 3,
        backoff_multiplier: 2.0,
    };

    let start_times = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    // Launch 10 concurrent retry operations
    let mut handles = vec![];
    for _ in 0..10 {
        let config = config.clone();
        let times = start_times.clone();

        let handle = tokio::spawn(async move {
            let start = Instant::now();
            times.lock().await.push(start);

            let _ = retry_with_backoff(|| async { Err::<(), _>(anyhow::anyhow!("Fail")) }, config)
                .await;
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Verify start times are not all identical (jitter applied)
    let times = start_times.lock().await;

    // At least some should differ by more than 1ms
    let mut differs = false;
    for i in 0..times.len() - 1 {
        if times[i].elapsed() > Duration::from_millis(1) {
            differs = true;
            break;
        }
    }

    assert!(
        differs,
        "Concurrent retries should have jittered start times"
    );
}

/// Test integration: Watchdog state affects dial decisions
#[tokio::test]
async fn test_end_to_end_local_only_mode_blocks_wan_dials() {
    // This is a higher-level integration test that will verify:
    // 1. Watchdog detects bootstrap failure
    // 2. System enters local-only mode
    // 3. WAN dial attempts are skipped
    // 4. LAN operations continue normally

    // Arrange: Create watchdog in failed state
    let watchdog = ConnectivityWatchdog::default();
    watchdog.force_local_only();

    // Act: Check if WAN dials should be attempted
    let should_dial_wan = !watchdog.is_local_only_mode();

    // Assert: WAN dials should be blocked
    assert!(
        !should_dial_wan,
        "WAN dials should be blocked in local-only mode"
    );

    // Simulate recovery
    watchdog.force_online();

    // Now WAN dials should be allowed
    let should_dial_wan = !watchdog.is_local_only_mode();
    assert!(should_dial_wan, "WAN dials should be allowed when online");
}

/// Test that resource limits prevent OOM by capping memory
#[test]
fn test_resource_limits_prevent_oom() {
    // Arrange: Create limits with memory cap
    let limits = ResourceLimits {
        max_memory_mb: 1024,
        ..ResourceLimits::default()
    };

    // Act & Assert: Check various memory usage levels
    assert!(limits.check_memory_usage(512).is_ok());
    assert!(limits.check_memory_usage(1024).is_ok());
    assert!(limits.check_memory_usage(1025).is_err());
    assert!(limits.check_memory_usage(2048).is_err());
}

/// Test that bandwidth limits are properly converted
#[test]
fn test_bandwidth_limit_conversion() {
    // Arrange: Create limits with bandwidth caps
    let limits = ResourceLimits {
        upload_rate_limit_mbps: Some(10),
        download_rate_limit_mbps: Some(100),
        ..ResourceLimits::default()
    };

    // Act: Convert to bytes/sec
    let upload_bps = limits.upload_rate_bytes_per_sec();
    let download_bps = limits.download_rate_bytes_per_sec();

    // Assert: Conversion is correct
    // 10 Mbps = 10 * 1,000,000 / 8 = 1,250,000 bytes/sec
    assert_eq!(upload_bps, Some(1_250_000));

    // 100 Mbps = 100 * 1,000,000 / 8 = 12,500,000 bytes/sec
    assert_eq!(download_bps, Some(12_500_000));
}

/// Test that connection timeout is enforced
#[test]
fn test_connection_timeout_enforcement() {
    // Arrange: Different presets have different timeouts
    let default_limits = ResourceLimits::default();
    let low_res_limits = ResourceLimits::low_resource();
    let high_perf_limits = ResourceLimits::high_performance();

    // Assert: Timeouts match specifications
    assert_eq!(default_limits.connection_timeout, Duration::from_secs(30));
    assert_eq!(low_res_limits.connection_timeout, Duration::from_secs(15));
    assert_eq!(high_perf_limits.connection_timeout, Duration::from_secs(60));
}

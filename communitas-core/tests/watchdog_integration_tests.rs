// Copyright (c) 2025 Saorsa Labs Limited
//
// Integration tests for connectivity watchdog
//
// Tests MESH_CAPABILITIES.md §3.2 Scenario A: Internet collapse detection

use communitas_core::{ConnectivityWatchdog, WatchdogConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Test that watchdog detects bootstrap failure and enters local-only mode
#[tokio::test]
async fn test_watchdog_detects_bootstrap_failure_and_enters_local_only() {
    // Arrange: Create watchdog with short threshold for testing
    let config = WatchdogConfig {
        check_interval: Duration::from_millis(50),
        detection_threshold: Duration::from_millis(200),
        recovery_check_interval: Duration::from_millis(100),
        enabled: true,
    };
    
    let watchdog = ConnectivityWatchdog::new(config.clone());
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();
    
    // Health check that always fails
    let health_check = move || {
        let count = call_count_clone.clone();
        async move {
            count.fetch_add(1, Ordering::SeqCst);
            false // Simulate bootstrap failure
        }
    };
    
    // Act: Start monitoring
    let handle = watchdog.clone().start_monitoring(health_check);
    
    // Initially should be online
    assert!(!watchdog.is_local_only_mode(), "Should start in online mode");
    
    // Wait past detection threshold
    sleep(Duration::from_millis(300)).await;
    
    // Assert: Should be in local-only mode now
    assert!(
        watchdog.is_local_only_mode(),
        "Should enter local-only mode after bootstrap failures exceed threshold"
    );
    
    // Verify health checks were called multiple times
    let count = call_count.load(Ordering::SeqCst);
    assert!(
        count >= 4,
        "Health check should be called at least 4 times, got {}",
        count
    );
    
    handle.abort();
}

/// Test that watchdog exits local-only mode when bootstrap succeeds
#[tokio::test]
async fn test_watchdog_exits_local_only_on_bootstrap_success() {
    // Arrange: Create watchdog
    let config = WatchdogConfig {
        check_interval: Duration::from_millis(50),
        detection_threshold: Duration::from_millis(150),
        recovery_check_interval: Duration::from_millis(100),
        enabled: true,
    };
    
    let watchdog = ConnectivityWatchdog::new(config);
    let attempt_count = Arc::new(AtomicUsize::new(0));
    let attempt_count_clone = attempt_count.clone();
    
    // Health check that fails first 3 times, then succeeds
    let health_check = move || {
        let count = attempt_count_clone.clone();
        async move {
            let current = count.fetch_add(1, Ordering::SeqCst);
            current >= 5 // Fail first 5, then succeed
        }
    };
    
    // Act: Start monitoring
    let handle = watchdog.clone().start_monitoring(health_check);
    
    // Wait for failures to trigger local-only
    sleep(Duration::from_millis(250)).await;
    assert!(
        watchdog.is_local_only_mode(),
        "Should be in local-only mode after failures"
    );
    
    // Wait for recovery
    sleep(Duration::from_millis(200)).await;
    
    // Assert: Should exit local-only mode
    assert!(
        !watchdog.is_local_only_mode(),
        "Should exit local-only mode after bootstrap succeeds"
    );
    
    handle.abort();
}

/// Test that watchdog can be manually controlled for testing
#[tokio::test]
async fn test_watchdog_manual_control() {
    let watchdog = ConnectivityWatchdog::default();
    
    // Start in online mode
    assert!(!watchdog.is_local_only_mode());
    
    // Manually enter local-only
    watchdog.force_local_only();
    assert!(watchdog.is_local_only_mode());
    
    // Manually exit local-only
    watchdog.force_online();
    assert!(!watchdog.is_local_only_mode());
}

/// Test that time_since_last_success tracks correctly
#[tokio::test]
async fn test_watchdog_tracks_time_since_last_success() {
    let watchdog = ConnectivityWatchdog::default();
    
    // Initially no success recorded
    assert!(watchdog.time_since_last_success().await.is_none());
    
    // Record success
    watchdog.record_success().await;
    
    // Wait a bit
    sleep(Duration::from_millis(100)).await;
    
    // Should have elapsed time
    let elapsed = watchdog.time_since_last_success().await.unwrap();
    assert!(
        elapsed >= Duration::from_millis(100),
        "Elapsed time should be at least 100ms, got {:?}",
        elapsed
    );
    assert!(
        elapsed < Duration::from_millis(500),
        "Elapsed time should be less than 500ms, got {:?}",
        elapsed
    );
}

/// Test that disabled watchdog doesn't monitor
#[tokio::test]
async fn test_watchdog_disabled() {
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
            false
        }
    };
    
    let handle = watchdog.clone().start_monitoring(health_check);
    
    // Wait a bit
    sleep(Duration::from_millis(200)).await;
    
    // Should never enter local-only mode when disabled
    assert!(!watchdog.is_local_only_mode());
    
    // Health check should not be called when disabled
    let count = call_count.load(Ordering::SeqCst);
    assert_eq!(count, 0, "Health check should not be called when disabled");
    
    handle.abort();
}

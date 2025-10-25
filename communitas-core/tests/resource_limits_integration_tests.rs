// Copyright (c) 2025 Saorsa Labs Limited
//
// Integration tests for resource limits enforcement
//
// Tests MESH_CAPABILITIES.md §8.3: Resource management

use communitas_core::{ResourceLimitError, ResourceLimits};
use std::time::Duration;

/// Test that ResourceLimits enforces peer connection limit
#[test]
fn test_resource_limits_enforce_peer_limit() {
    // Arrange: Create limits with max 50 peers
    let limits = ResourceLimits::default();
    assert_eq!(limits.max_peer_connections, 50);

    // Act & Assert: Should allow connections below limit
    assert!(limits.enforce_peer_limit(49).is_ok());

    // Should reject at limit
    let result = limits.enforce_peer_limit(50);
    assert!(result.is_err());

    match result {
        Err(ResourceLimitError::PeerLimitExceeded { current, limit }) => {
            assert_eq!(current, 50);
            assert_eq!(limit, 50);
        }
        _ => panic!("Expected PeerLimitExceeded error"),
    }

    // Should reject above limit
    assert!(limits.enforce_peer_limit(51).is_err());
}

/// Test that ResourceLimits enforces relay connection limit
#[test]
fn test_resource_limits_enforce_relay_limit() {
    // Arrange
    let limits = ResourceLimits::default();
    assert_eq!(limits.max_relay_connections, 3);

    // Act & Assert
    assert!(limits.enforce_relay_limit(2).is_ok());
    assert!(limits.enforce_relay_limit(3).is_err());
    assert!(limits.enforce_relay_limit(4).is_err());
}

/// Test that ResourceLimits enforces document size limit
#[test]
fn test_resource_limits_enforce_document_limit() {
    // Arrange: Default is 50 MB
    let limits = ResourceLimits::default();
    assert_eq!(limits.crdt_document_limit_mb, 50);

    // Act & Assert: Should allow documents below limit
    assert!(limits.enforce_document_limit(25).is_ok());
    assert!(limits.enforce_document_limit(50).is_ok());

    // Should reject documents exceeding limit
    let result = limits.enforce_document_limit(51);
    assert!(result.is_err());

    match result {
        Err(ResourceLimitError::DocumentTooLarge { size_mb, limit_mb }) => {
            assert_eq!(size_mb, 51);
            assert_eq!(limit_mb, 50);
        }
        _ => panic!("Expected DocumentTooLarge error"),
    }
}

/// Test that ResourceLimits checks memory usage
#[test]
fn test_resource_limits_check_memory_usage() {
    // Arrange: Default is 2048 MB
    let limits = ResourceLimits::default();
    assert_eq!(limits.max_memory_mb, 2048);

    // Act & Assert
    assert!(limits.check_memory_usage(1024).is_ok());
    assert!(limits.check_memory_usage(2048).is_ok());
    assert!(limits.check_memory_usage(2049).is_err());
}

/// Test low-resource preset
#[test]
fn test_low_resource_preset() {
    // Arrange & Act
    let limits = ResourceLimits::low_resource();

    // Assert: Should have conservative limits
    assert_eq!(limits.max_memory_mb, 512);
    assert_eq!(limits.max_peer_connections, 20);
    assert_eq!(limits.max_relay_connections, 1);
    assert_eq!(limits.crdt_document_limit_mb, 10);
    assert_eq!(limits.upload_rate_limit_mbps, Some(5));
    assert_eq!(limits.download_rate_limit_mbps, Some(20));

    // Should validate
    assert!(limits.validate().is_ok());
}

/// Test high-performance preset
#[test]
fn test_high_performance_preset() {
    // Arrange & Act
    let limits = ResourceLimits::high_performance();

    // Assert: Should have generous limits
    assert_eq!(limits.max_memory_mb, 8192);
    assert_eq!(limits.max_peer_connections, 200);
    assert_eq!(limits.max_relay_connections, 10);
    assert_eq!(limits.crdt_document_limit_mb, 200);
    assert_eq!(limits.upload_rate_limit_mbps, None);
    assert_eq!(limits.download_rate_limit_mbps, None);

    // Should validate
    assert!(limits.validate().is_ok());
}

/// Test bandwidth conversion
#[test]
fn test_bandwidth_conversion() {
    // Arrange
    let limits = ResourceLimits {
        upload_rate_limit_mbps: Some(10),   // 10 Mbps
        download_rate_limit_mbps: Some(50), // 50 Mbps
        ..Default::default()
    };

    // Act & Assert: 10 Mbps = 1,250,000 bytes/sec
    assert_eq!(limits.upload_rate_bytes_per_sec(), Some(1_250_000));

    // 50 Mbps = 6,250,000 bytes/sec
    assert_eq!(limits.download_rate_bytes_per_sec(), Some(6_250_000));
}

/// Test validation catches invalid configurations
#[test]
fn test_validation_catches_invalid_configs() {
    // Zero peers is invalid
    let mut limits = ResourceLimits::default();
    limits.max_peer_connections = 0;
    assert!(limits.validate().is_err());

    // Zero memory is invalid
    limits = ResourceLimits::default();
    limits.max_memory_mb = 0;
    assert!(limits.validate().is_err());

    // Document limit exceeding memory is invalid
    limits = ResourceLimits::default();
    limits.crdt_document_limit_mb = 3000; // Greater than max_memory_mb (2048)
    let result = limits.validate();
    assert!(result.is_err());
}

/// Test that default configuration is valid
#[test]
fn test_default_config_is_valid() {
    let limits = ResourceLimits::default();
    assert!(limits.validate().is_ok());
}

/// Test connection timeout is reasonable
#[test]
fn test_connection_timeout() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.connection_timeout, Duration::from_secs(30));

    let low_res = ResourceLimits::low_resource();
    assert_eq!(low_res.connection_timeout, Duration::from_secs(15));

    let high_perf = ResourceLimits::high_performance();
    assert_eq!(high_perf.connection_timeout, Duration::from_secs(60));
}

/// Test anti-entropy interval limits
#[test]
fn test_anti_entropy_interval() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.anti_entropy_max_interval, Duration::from_secs(300)); // 5 min

    let low_res = ResourceLimits::low_resource();
    assert_eq!(low_res.anti_entropy_max_interval, Duration::from_secs(600)); // 10 min

    let high_perf = ResourceLimits::high_performance();
    assert_eq!(high_perf.anti_entropy_max_interval, Duration::from_secs(60)); // 1 min
}

/// Test that unlimited bandwidth works
#[test]
fn test_unlimited_bandwidth() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.upload_rate_limit_mbps, None);
    assert_eq!(limits.download_rate_limit_mbps, None);
    assert_eq!(limits.upload_rate_bytes_per_sec(), None);
    assert_eq!(limits.download_rate_bytes_per_sec(), None);
}

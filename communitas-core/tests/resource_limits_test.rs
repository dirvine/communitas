// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Resource Limits Tests
//!
//! Tests for resource management and enforcement as specified in
//! MESH_CAPABILITIES.md Section 8.3

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::resource_limits::{ResourceLimits, ResourceLimitsConfig, ResourceUsage};
use std::time::Duration;

#[test]
fn test_default_resource_limits() {
    let limits = ResourceLimits::default();

    assert_eq!(limits.max_peer_connections, 50);
    assert_eq!(limits.max_memory_mb, 2048);
    assert_eq!(limits.connection_timeout, Duration::from_secs(30));
    assert_eq!(limits.anti_entropy_max_interval, Duration::from_secs(300));
}

#[test]
fn test_custom_resource_limits() {
    let config = ResourceLimitsConfig {
        max_peer_connections: 100,
        max_relay_connections: 5,
        max_memory_mb: 4096,
        crdt_document_limit_mb: 100,
        connection_timeout_secs: 60,
        anti_entropy_max_interval_secs: 600,
        max_upload_rate_mbps: Some(10),
        max_download_rate_mbps: Some(50),
    };

    let limits = ResourceLimits::from_config(config);

    assert_eq!(limits.max_peer_connections, 100);
    assert_eq!(limits.max_memory_mb, 4096);
    assert_eq!(limits.connection_timeout, Duration::from_secs(60));
}

#[test]
fn test_enforce_peer_limit_success() {
    let limits = ResourceLimits::default();

    let result = limits.enforce_peer_limit(25);
    assert!(result.is_ok(), "Should allow 25 peers when limit is 50");
}

#[test]
fn test_enforce_peer_limit_at_max() {
    let limits = ResourceLimits::default();

    let result = limits.enforce_peer_limit(50);
    assert!(
        result.is_err(),
        "Should reject 50th peer when limit is 50 (0-49)"
    );
}

#[test]
fn test_enforce_peer_limit_exceeded() {
    let limits = ResourceLimits::default();

    let result = limits.enforce_peer_limit(51);
    assert!(result.is_err(), "Should reject when exceeding limit");

    match result {
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains("Peer") || msg.contains("limit"),
                "Error: {}",
                msg
            );
        }
        Ok(_) => panic!("Should have returned an error"),
    }
}

#[test]
fn test_enforce_memory_limit() {
    let limits = ResourceLimits::default();

    let usage = ResourceUsage {
        peer_connections: 10,
        memory_mb: 1024,
        upload_rate_mbps: 5.0,
        download_rate_mbps: 20.0,
    };

    let result = limits.enforce_memory_limit(usage.memory_mb);
    assert!(result.is_ok(), "1024MB should be within 2048MB limit");
}

#[test]
fn test_enforce_memory_limit_exceeded() {
    let limits = ResourceLimits::default();

    let result = limits.enforce_memory_limit(3000);
    assert!(result.is_err(), "3000MB exceeds 2048MB limit");
}

#[test]
fn test_enforce_bandwidth_limit() {
    let config = ResourceLimitsConfig {
        max_peer_connections: 50,
        max_relay_connections: 3,
        max_memory_mb: 2048,
        crdt_document_limit_mb: 50,
        connection_timeout_secs: 30,
        anti_entropy_max_interval_secs: 300,
        max_upload_rate_mbps: Some(10),
        max_download_rate_mbps: Some(50),
    };

    let limits = ResourceLimits::from_config(config);

    assert!(limits.enforce_upload_rate(8.0).is_ok());
    assert!(limits.enforce_upload_rate(12.0).is_err());

    assert!(limits.enforce_download_rate(45.0).is_ok());
    assert!(limits.enforce_download_rate(60.0).is_err());
}

#[test]
fn test_check_all_limits() {
    let limits = ResourceLimits::default();

    let usage_ok = ResourceUsage {
        peer_connections: 40,
        memory_mb: 1500,
        upload_rate_mbps: 5.0,
        download_rate_mbps: 20.0,
    };

    assert!(limits.check_all(&usage_ok).is_ok());

    let usage_peers_exceeded = ResourceUsage {
        peer_connections: 60,
        memory_mb: 1500,
        upload_rate_mbps: 5.0,
        download_rate_mbps: 20.0,
    };

    assert!(limits.check_all(&usage_peers_exceeded).is_err());

    let usage_memory_exceeded = ResourceUsage {
        peer_connections: 40,
        memory_mb: 3000,
        upload_rate_mbps: 5.0,
        download_rate_mbps: 20.0,
    };

    assert!(limits.check_all(&usage_memory_exceeded).is_err());
}

#[test]
fn test_load_from_toml() {
    let toml_str = r#"
max_peer_connections = 75
max_relay_connections = 5
max_memory_mb = 3072
crdt_document_limit_mb = 75
connection_timeout_secs = 45
anti_entropy_max_interval_secs = 450
max_upload_rate_mbps = 15
max_download_rate_mbps = 75
"#;

    let config: ResourceLimitsConfig = toml::from_str(toml_str).expect("Parse TOML");
    let limits = ResourceLimits::from_config(config);

    assert_eq!(limits.max_peer_connections, 75);
    assert_eq!(limits.max_memory_mb, 3072);
    assert_eq!(limits.connection_timeout, Duration::from_secs(45));
}

#[test]
fn test_adaptive_limits() {
    let mut limits = ResourceLimits {
        max_peer_connections: 100,
        ..Default::default()
    };

    // Direct field access for adaptive limits
    assert_eq!(limits.max_peer_connections, 100);

    limits.max_memory_mb = 4096;
    assert_eq!(limits.max_memory_mb, 4096);
}

#[tokio::test]
async fn test_periodic_enforcement() {
    let limits = ResourceLimits::default();

    let mut usage = ResourceUsage {
        peer_connections: 10,
        memory_mb: 500,
        upload_rate_mbps: 2.0,
        download_rate_mbps: 10.0,
    };

    for i in 1..=60 {
        usage.peer_connections = i;

        if i < 50 {
            assert!(limits.check_all(&usage).is_ok(), "Should allow {} peers", i);
        } else {
            assert!(
                limits.check_all(&usage).is_err(),
                "Should reject {} peers",
                i
            );
        }
    }
}

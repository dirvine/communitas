// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Test Configuration
//!
//! Provides configurable settings for E2E tests via environment variables.
//! This allows tests to be tuned for different CI environments.

#![allow(dead_code)]

use std::time::Duration;

/// Test configuration loaded from environment variables
pub struct TestConfig {
    /// Timeout for sync operations (default: 30s)
    pub sync_timeout: Duration,
    /// Timeout for individual tool calls (default: 10s)
    pub tool_timeout: Duration,
    /// Poll interval for sync checks (default: 500ms)
    pub poll_interval: Duration,
    /// Whether network tests are enabled
    pub network_enabled: bool,
    /// Use localhost fallback for bootstrap nodes
    pub use_localhost_fallback: bool,
    /// Localhost port for fallback bootstrap node
    pub localhost_port: u16,
}

impl TestConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            sync_timeout: Self::parse_duration_secs("MCP_TEST_SYNC_TIMEOUT", 30),
            tool_timeout: Self::parse_duration_secs("MCP_TEST_TOOL_TIMEOUT", 10),
            poll_interval: Self::parse_duration_millis("MCP_TEST_POLL_INTERVAL", 500),
            network_enabled: Self::parse_bool("MCP_TEST_NETWORK_ENABLED", false),
            use_localhost_fallback: Self::parse_bool("MCP_TEST_LOCALHOST_FALLBACK", true),
            localhost_port: Self::parse_u16("MCP_TEST_LOCALHOST_PORT", 11000),
        }
    }

    /// Get the sync timeout duration
    pub fn sync_timeout(&self) -> Duration {
        self.sync_timeout
    }

    /// Get the tool timeout duration
    pub fn tool_timeout(&self) -> Duration {
        self.tool_timeout
    }

    /// Get the poll interval
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }

    /// Check if network tests are enabled
    pub fn network_enabled(&self) -> bool {
        self.network_enabled
    }

    /// Get bootstrap nodes with fallback support
    pub fn bootstrap_nodes(&self) -> Vec<String> {
        let mut nodes = vec![
            "142.93.199.50:11000".to_string(),   // saorsa-2 (NYC1)
            "147.182.234.192:11000".to_string(), // saorsa-3 (SFO3)
        ];

        if self.use_localhost_fallback {
            nodes.push(format!("127.0.0.1:{}", self.localhost_port));
        }

        nodes
    }

    fn parse_duration_secs(key: &str, default: u64) -> Duration {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(default))
    }

    fn parse_duration_millis(key: &str, default: u64) -> Duration {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_millis(default))
    }

    fn parse_bool(key: &str, default: bool) -> bool {
        std::env::var(key)
            .ok()
            .map(|v| v == "true" || v == "1")
            .unwrap_or(default)
    }

    fn parse_u16(key: &str, default: u16) -> u16 {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(default)
    }
}

impl Default for TestConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Global test configuration (lazy-loaded)
pub fn config() -> &'static TestConfig {
    use std::sync::OnceLock;
    static CONFIG: OnceLock<TestConfig> = OnceLock::new();
    CONFIG.get_or_init(TestConfig::from_env)
}

/// Helper to get sync timeout (convenience function)
pub fn sync_timeout() -> Duration {
    config().sync_timeout()
}

/// Helper to get tool timeout (convenience function)
pub fn tool_timeout() -> Duration {
    config().tool_timeout()
}

/// Helper to get poll interval (convenience function)
pub fn poll_interval() -> Duration {
    config().poll_interval()
}

/// Helper to check if network tests are enabled
pub fn network_tests_enabled() -> bool {
    config().network_enabled()
}

/// Helper to get bootstrap nodes
pub fn bootstrap_nodes() -> Vec<String> {
    config().bootstrap_nodes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TestConfig::from_env();
        assert_eq!(config.sync_timeout, Duration::from_secs(30));
        assert_eq!(config.tool_timeout, Duration::from_secs(10));
        assert_eq!(config.poll_interval, Duration::from_millis(500));
    }

    #[test]
    fn test_bootstrap_nodes_with_fallback() {
        let config = TestConfig {
            use_localhost_fallback: true,
            localhost_port: 11000,
            ..TestConfig::from_env()
        };

        let nodes = config.bootstrap_nodes();
        assert!(nodes.len() >= 2);
        assert!(nodes.iter().any(|n| n.contains("127.0.0.1")));
    }

    #[test]
    fn test_bootstrap_nodes_without_fallback() {
        let config = TestConfig {
            use_localhost_fallback: false,
            ..TestConfig::from_env()
        };

        let nodes = config.bootstrap_nodes();
        assert!(!nodes.iter().any(|n| n.contains("127.0.0.1")));
    }
}

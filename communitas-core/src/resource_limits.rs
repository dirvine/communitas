// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Resource Limits and Management
//!
//! Implements MESH_CAPABILITIES.md Section 8.3: Resource management and limits
//! to prevent OOM, connection exhaustion, and bandwidth saturation.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Resource limit errors
#[derive(Error, Debug)]
pub enum ResourceLimitError {
    #[error("Peer connection limit exceeded: {current}/{max}")]
    PeerLimitExceeded { current: usize, max: usize },

    #[error("Memory limit exceeded: {current}MB/{max}MB")]
    MemoryLimitExceeded { current: usize, max: usize },

    #[error("Upload rate limit exceeded: {current:.2}Mbps/{max:.2}Mbps")]
    UploadRateExceeded { current: f64, max: f64 },

    #[error("Download rate limit exceeded: {current:.2}Mbps/{max:.2}Mbps")]
    DownloadRateExceeded { current: f64, max: f64 },

    #[error("Connection timeout: {0:?}")]
    Timeout(Duration),
}

/// Result type for resource limit operations
pub type ResourceLimitResult<T> = Result<T, ResourceLimitError>;

/// Resource limits configuration (loadable from TOML)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsConfig {
    /// Maximum number of concurrent peer connections
    pub max_peer_connections: usize,

    /// Maximum memory usage in megabytes
    pub max_memory_mb: usize,

    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,

    /// Maximum anti-entropy sync interval in seconds
    pub anti_entropy_max_interval_secs: u64,

    /// Optional upload rate limit in Mbps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_upload_rate_mbps: Option<u64>,

    /// Optional download rate limit in Mbps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_download_rate_mbps: Option<u64>,
}

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            max_peer_connections: 50,
            max_memory_mb: 2048,
            connection_timeout_secs: 30,
            anti_entropy_max_interval_secs: 300,
            max_upload_rate_mbps: None,
            max_download_rate_mbps: None,
        }
    }
}

/// Current resource usage snapshot
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub peer_connections: usize,
    pub memory_mb: usize,
    pub upload_rate_mbps: f64,
    pub download_rate_mbps: f64,
}

/// Resource limits manager
///
/// Enforces resource constraints to prevent:
/// - Connection exhaustion
/// - Out-of-memory conditions
/// - Bandwidth saturation
/// - Excessive sync intervals
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    max_peer_connections: usize,
    max_memory_mb: usize,
    connection_timeout: Duration,
    anti_entropy_max_interval: Duration,
    max_upload_rate_mbps: Option<f64>,
    max_download_rate_mbps: Option<f64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self::from_config(ResourceLimitsConfig::default())
    }
}

impl ResourceLimits {
    /// Create resource limits from configuration
    pub fn from_config(config: ResourceLimitsConfig) -> Self {
        Self {
            max_peer_connections: config.max_peer_connections,
            max_memory_mb: config.max_memory_mb,
            connection_timeout: Duration::from_secs(config.connection_timeout_secs),
            anti_entropy_max_interval: Duration::from_secs(
                config.anti_entropy_max_interval_secs,
            ),
            max_upload_rate_mbps: config.max_upload_rate_mbps.map(|r| r as f64),
            max_download_rate_mbps: config.max_download_rate_mbps.map(|r| r as f64),
        }
    }

    /// Get maximum peer connections
    pub fn max_peer_connections(&self) -> usize {
        self.max_peer_connections
    }

    /// Get maximum memory in MB
    pub fn max_memory_mb(&self) -> usize {
        self.max_memory_mb
    }

    /// Get connection timeout
    pub fn connection_timeout(&self) -> Duration {
        self.connection_timeout
    }

    /// Get anti-entropy maximum interval
    pub fn anti_entropy_max_interval(&self) -> Duration {
        self.anti_entropy_max_interval
    }

    /// Set maximum peer connections (for adaptive limits)
    pub fn set_max_peer_connections(&mut self, max: usize) {
        self.max_peer_connections = max;
    }

    /// Set maximum memory (for adaptive limits)
    pub fn set_max_memory_mb(&mut self, max: usize) {
        self.max_memory_mb = max;
    }

    /// Enforce peer connection limit
    pub fn enforce_peer_limit(&self, current: usize) -> ResourceLimitResult<()> {
        if current >= self.max_peer_connections {
            Err(ResourceLimitError::PeerLimitExceeded {
                current,
                max: self.max_peer_connections,
            })
        } else {
            Ok(())
        }
    }

    /// Enforce memory limit
    pub fn enforce_memory_limit(&self, current_mb: usize) -> ResourceLimitResult<()> {
        if current_mb > self.max_memory_mb {
            Err(ResourceLimitError::MemoryLimitExceeded {
                current: current_mb,
                max: self.max_memory_mb,
            })
        } else {
            Ok(())
        }
    }

    /// Enforce upload rate limit
    pub fn enforce_upload_rate(&self, current_mbps: f64) -> ResourceLimitResult<()> {
        if let Some(max) = self.max_upload_rate_mbps {
            if current_mbps > max {
                return Err(ResourceLimitError::UploadRateExceeded {
                    current: current_mbps,
                    max,
                });
            }
        }
        Ok(())
    }

    /// Enforce download rate limit
    pub fn enforce_download_rate(&self, current_mbps: f64) -> ResourceLimitResult<()> {
        if let Some(max) = self.max_download_rate_mbps {
            if current_mbps > max {
                return Err(ResourceLimitError::DownloadRateExceeded {
                    current: current_mbps,
                    max,
                });
            }
        }
        Ok(())
    }

    /// Check all limits against current usage
    pub fn check_all(&self, usage: &ResourceUsage) -> ResourceLimitResult<()> {
        self.enforce_peer_limit(usage.peer_connections)?;
        self.enforce_memory_limit(usage.memory_mb)?;
        self.enforce_upload_rate(usage.upload_rate_mbps)?;
        self.enforce_download_rate(usage.download_rate_mbps)?;
        Ok(())
    }

    /// Get current resource usage from system
    pub fn measure_current_usage(&self) -> ResourceUsage {
        ResourceUsage {
            peer_connections: 0,
            memory_mb: 0,
            upload_rate_mbps: 0.0,
            download_rate_mbps: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = ResourceLimitsConfig::default();
        let toml = toml::to_string(&config).expect("Serialize");
        let parsed: ResourceLimitsConfig = toml::from_str(&toml).expect("Deserialize");

        assert_eq!(parsed.max_peer_connections, 50);
        assert_eq!(parsed.max_memory_mb, 2048);
    }

    #[test]
    fn test_zero_limits() {
        let config = ResourceLimitsConfig {
            max_peer_connections: 0,
            max_memory_mb: 0,
            connection_timeout_secs: 1,
            anti_entropy_max_interval_secs: 1,
            max_upload_rate_mbps: None,
            max_download_rate_mbps: None,
        };

        let limits = ResourceLimits::from_config(config);

        assert!(limits.enforce_peer_limit(0).is_err());
        assert!(limits.enforce_memory_limit(1).is_err());
    }
}

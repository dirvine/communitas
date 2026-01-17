// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Resource Limits and Management
//!
//! Implements MESH_CAPABILITIES.md Section 8.3: Resource management and limits
//! to prevent OOM, connection exhaustion, and bandwidth saturation.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use sysinfo::{System, get_current_pid};
use thiserror::Error;
use tracing::warn;

/// Resource limit errors
#[derive(Error, Debug)]
pub enum ResourceLimitError {
    #[error("Peer connection limit exceeded: {current}/{limit}")]
    PeerLimitExceeded { current: usize, limit: usize },

    #[error("Memory limit exceeded: {current}MB/{limit_mb}MB")]
    MemoryLimitExceeded { current: usize, limit_mb: usize },

    #[error("Document size too large: {size_mb}MB/{limit_mb}MB")]
    DocumentTooLarge { size_mb: usize, limit_mb: usize },

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

    /// Maximum number of relay connections
    pub max_relay_connections: usize,

    /// Maximum memory usage in megabytes
    pub max_memory_mb: usize,

    /// Maximum CRDT document size in megabytes
    pub crdt_document_limit_mb: usize,

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
            max_relay_connections: 3,
            max_memory_mb: 2048,
            crdt_document_limit_mb: 50,
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
    pub max_peer_connections: usize,
    pub max_relay_connections: usize,
    pub max_memory_mb: usize,
    pub crdt_document_limit_mb: usize,
    pub connection_timeout: Duration,
    pub anti_entropy_max_interval: Duration,
    pub upload_rate_limit_mbps: Option<u64>,
    pub download_rate_limit_mbps: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        // Direct construction to avoid recursion with from_config fallback
        let config = ResourceLimitsConfig::default();
        Self {
            max_peer_connections: config.max_peer_connections,
            max_relay_connections: config.max_relay_connections,
            max_memory_mb: config.max_memory_mb,
            crdt_document_limit_mb: config.crdt_document_limit_mb,
            connection_timeout: Duration::from_secs(config.connection_timeout_secs),
            anti_entropy_max_interval: Duration::from_secs(config.anti_entropy_max_interval_secs),
            upload_rate_limit_mbps: config.max_upload_rate_mbps,
            download_rate_limit_mbps: config.max_download_rate_mbps,
        }
    }
}

impl ResourceLimits {
    /// Create resource limits from configuration with validation.
    ///
    /// Returns `Err` if the configuration is invalid (e.g., zero limits,
    /// document size exceeds memory limit).
    pub fn try_from_config(config: ResourceLimitsConfig) -> ResourceLimitResult<Self> {
        // Validate: peer connections must be positive
        if config.max_peer_connections == 0 {
            return Err(ResourceLimitError::PeerLimitExceeded {
                current: 0,
                limit: 0,
            });
        }

        // Validate: memory must be positive
        if config.max_memory_mb == 0 {
            return Err(ResourceLimitError::MemoryLimitExceeded {
                current: 0,
                limit_mb: 0,
            });
        }

        // Validate: document limit should not exceed memory limit
        if config.crdt_document_limit_mb > config.max_memory_mb {
            return Err(ResourceLimitError::MemoryLimitExceeded {
                current: config.crdt_document_limit_mb,
                limit_mb: config.max_memory_mb,
            });
        }

        Ok(Self {
            max_peer_connections: config.max_peer_connections,
            max_relay_connections: config.max_relay_connections,
            max_memory_mb: config.max_memory_mb,
            crdt_document_limit_mb: config.crdt_document_limit_mb,
            connection_timeout: Duration::from_secs(config.connection_timeout_secs),
            anti_entropy_max_interval: Duration::from_secs(config.anti_entropy_max_interval_secs),
            upload_rate_limit_mbps: config.max_upload_rate_mbps,
            download_rate_limit_mbps: config.max_download_rate_mbps,
        })
    }

    /// Create resource limits from configuration.
    ///
    /// Logs a warning and uses safe defaults if validation fails.
    /// For stricter error handling, use [`try_from_config`].
    pub fn from_config(config: ResourceLimitsConfig) -> Self {
        match Self::try_from_config(config) {
            Ok(limits) => limits,
            Err(e) => {
                warn!(
                    "Invalid resource limits configuration: {}. Using safe defaults.",
                    e
                );
                Self::default()
            }
        }
    }

    /// Create low-resource preset for constrained devices
    pub fn low_resource() -> Self {
        Self::from_config(ResourceLimitsConfig {
            max_peer_connections: 20,
            max_relay_connections: 1,
            max_memory_mb: 512,
            crdt_document_limit_mb: 10,
            connection_timeout_secs: 15,
            anti_entropy_max_interval_secs: 600, // 10 min
            max_upload_rate_mbps: Some(5),
            max_download_rate_mbps: Some(20),
        })
    }

    /// Create high-performance preset for powerful devices
    pub fn high_performance() -> Self {
        Self::from_config(ResourceLimitsConfig {
            max_peer_connections: 200,
            max_relay_connections: 10,
            max_memory_mb: 8192,
            crdt_document_limit_mb: 200,
            connection_timeout_secs: 60,
            anti_entropy_max_interval_secs: 60, // 1 min
            max_upload_rate_mbps: None,
            max_download_rate_mbps: None,
        })
    }

    /// Enforce peer connection limit
    pub fn enforce_peer_limit(&self, current: usize) -> ResourceLimitResult<()> {
        if current >= self.max_peer_connections {
            Err(ResourceLimitError::PeerLimitExceeded {
                current,
                limit: self.max_peer_connections,
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
                limit_mb: self.max_memory_mb,
            })
        } else {
            Ok(())
        }
    }

    /// Enforce relay connection limit
    pub fn enforce_relay_limit(&self, current: usize) -> ResourceLimitResult<()> {
        if current >= self.max_relay_connections {
            Err(ResourceLimitError::PeerLimitExceeded {
                current,
                limit: self.max_relay_connections,
            })
        } else {
            Ok(())
        }
    }

    /// Enforce CRDT document size limit
    pub fn enforce_document_limit(&self, size_mb: usize) -> ResourceLimitResult<()> {
        if size_mb > self.crdt_document_limit_mb {
            Err(ResourceLimitError::DocumentTooLarge {
                size_mb,
                limit_mb: self.crdt_document_limit_mb,
            })
        } else {
            Ok(())
        }
    }

    /// Check memory usage against limit
    pub fn check_memory_usage(&self, current_mb: usize) -> ResourceLimitResult<()> {
        self.enforce_memory_limit(current_mb)
    }

    /// Enforce upload rate limit
    pub fn enforce_upload_rate(&self, current_mbps: f64) -> ResourceLimitResult<()> {
        if let Some(max) = self.upload_rate_limit_mbps {
            let max_f64 = max as f64;
            if current_mbps > max_f64 {
                return Err(ResourceLimitError::UploadRateExceeded {
                    current: current_mbps,
                    max: max_f64,
                });
            }
        }
        Ok(())
    }

    /// Enforce download rate limit
    pub fn enforce_download_rate(&self, current_mbps: f64) -> ResourceLimitResult<()> {
        if let Some(max) = self.download_rate_limit_mbps {
            let max_f64 = max as f64;
            if current_mbps > max_f64 {
                return Err(ResourceLimitError::DownloadRateExceeded {
                    current: current_mbps,
                    max: max_f64,
                });
            }
        }
        Ok(())
    }

    /// Convert upload rate limit to bytes per second
    pub fn upload_rate_bytes_per_sec(&self) -> Option<u64> {
        self.upload_rate_limit_mbps.map(|mbps| mbps * 125_000)
    }

    /// Convert download rate limit to bytes per second
    pub fn download_rate_bytes_per_sec(&self) -> Option<u64> {
        self.download_rate_limit_mbps.map(|mbps| mbps * 125_000)
    }

    /// Validate configuration for consistency
    pub fn validate(&self) -> ResourceLimitResult<()> {
        // Peer connections must be positive
        if self.max_peer_connections == 0 {
            return Err(ResourceLimitError::PeerLimitExceeded {
                current: 0,
                limit: 0,
            });
        }

        // Memory must be positive
        if self.max_memory_mb == 0 {
            return Err(ResourceLimitError::MemoryLimitExceeded {
                current: 0,
                limit_mb: 0,
            });
        }

        // Document limit should not exceed memory limit
        if self.crdt_document_limit_mb > self.max_memory_mb {
            return Err(ResourceLimitError::MemoryLimitExceeded {
                current: self.crdt_document_limit_mb,
                limit_mb: self.max_memory_mb,
            });
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

    /// Measure current usage with a provided peer count
    ///
    /// If memory monitoring is unavailable, uses max_memory_mb as a conservative
    /// fallback to prevent bypassing memory limits.
    pub fn measure_usage_with_peers(&self, peer_connections: usize) -> ResourceUsage {
        // Use max_memory_mb as conservative fallback if measurement fails
        // This prevents silently bypassing memory limits when monitoring unavailable
        let memory_mb = current_process_memory_mb().unwrap_or(self.max_memory_mb);
        ResourceUsage {
            peer_connections,
            memory_mb,
            upload_rate_mbps: 0.0,
            download_rate_mbps: 0.0,
        }
    }

    /// Get current resource usage from system
    pub fn measure_current_usage(&self) -> ResourceUsage {
        self.measure_usage_with_peers(0)
    }
}

fn current_process_memory_mb() -> Option<usize> {
    let pid = match get_current_pid() {
        Ok(pid) => pid,
        Err(e) => {
            warn!(
                "Failed to get current process ID for memory monitoring: {}",
                e
            );
            return None;
        }
    };
    let mut system = System::new();
    system.refresh_processes();
    match system.process(pid) {
        // process.memory() returns bytes, divide by 1024*1024 to get MB
        Some(process) => Some((process.memory() / (1024 * 1024)) as usize),
        None => {
            warn!("Failed to find current process in system info for memory monitoring");
            None
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
    fn test_try_from_config_valid() {
        let config = ResourceLimitsConfig::default();
        let result = ResourceLimits::try_from_config(config);
        assert!(result.is_ok());
        let limits = result.unwrap();
        assert_eq!(limits.max_peer_connections, 50);
        assert_eq!(limits.max_memory_mb, 2048);
    }

    #[test]
    fn test_try_from_config_zero_peer_connections() {
        let config = ResourceLimitsConfig {
            max_peer_connections: 0,
            ..ResourceLimitsConfig::default()
        };
        let result = ResourceLimits::try_from_config(config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResourceLimitError::PeerLimitExceeded { .. }
        ));
    }

    #[test]
    fn test_try_from_config_zero_memory() {
        let config = ResourceLimitsConfig {
            max_memory_mb: 0,
            ..ResourceLimitsConfig::default()
        };
        let result = ResourceLimits::try_from_config(config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResourceLimitError::MemoryLimitExceeded { .. }
        ));
    }

    #[test]
    fn test_try_from_config_document_exceeds_memory() {
        let config = ResourceLimitsConfig {
            max_memory_mb: 100,
            crdt_document_limit_mb: 200,
            ..ResourceLimitsConfig::default()
        };
        let result = ResourceLimits::try_from_config(config);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResourceLimitError::MemoryLimitExceeded { .. }
        ));
    }

    #[test]
    fn test_from_config_invalid_falls_back_to_defaults() {
        let config = ResourceLimitsConfig {
            max_peer_connections: 0,
            max_memory_mb: 0,
            ..ResourceLimitsConfig::default()
        };
        // from_config should fall back to defaults instead of panicking
        let limits = ResourceLimits::from_config(config);
        // Should have default values, not zeros
        assert_eq!(limits.max_peer_connections, 50);
        assert_eq!(limits.max_memory_mb, 2048);
    }

    #[test]
    fn test_from_config_valid_uses_provided_values() {
        let config = ResourceLimitsConfig {
            max_peer_connections: 100,
            max_memory_mb: 4096,
            ..ResourceLimitsConfig::default()
        };
        let limits = ResourceLimits::from_config(config);
        assert_eq!(limits.max_peer_connections, 100);
        assert_eq!(limits.max_memory_mb, 4096);
    }

    #[test]
    fn test_default_is_valid() {
        let limits = ResourceLimits::default();
        // Should be able to validate successfully
        assert!(limits.validate().is_ok());
        assert_eq!(limits.max_peer_connections, 50);
        assert_eq!(limits.max_memory_mb, 2048);
    }

    #[test]
    fn test_low_resource_is_valid() {
        let limits = ResourceLimits::low_resource();
        assert!(limits.validate().is_ok());
    }

    #[test]
    fn test_high_performance_is_valid() {
        let limits = ResourceLimits::high_performance();
        assert!(limits.validate().is_ok());
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Resource limits and enforcement for Communitas mesh networking
//
// Implements resource management as specified in MESH_CAPABILITIES.md §8.3
// to prevent OOM conditions, connection exhaustion, and bandwidth saturation.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Resource limits for mesh networking operations
///
/// These limits are enforced throughout the system to ensure stable operation
/// under resource constraints and prevent abuse.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    // Memory Management
    /// Maximum memory usage in MB (default: 2048 MB = 2 GB)
    pub max_memory_mb: usize,

    /// Maximum CRDT document size in MB (default: 50 MB)
    pub crdt_document_limit_mb: usize,

    /// Cache size limit in MB (default: 500 MB)
    pub cache_size_mb: usize,

    // Connection Management
    /// Maximum concurrent peer connections (default: 50)
    pub max_peer_connections: usize,

    /// Maximum relay connections (default: 3)
    pub max_relay_connections: usize,

    /// Connection timeout (default: 30 seconds)
    pub connection_timeout: Duration,

    // Bandwidth Management
    /// Upload rate limit in Mbps (None = unlimited)
    pub upload_rate_limit_mbps: Option<u64>,

    /// Download rate limit in Mbps (None = unlimited)
    pub download_rate_limit_mbps: Option<u64>,

    /// Burst allowance in MB (default: 10 MB)
    pub burst_allowance_mb: usize,

    // CPU Management
    /// Maximum worker threads (default: 4)
    pub max_worker_threads: usize,

    /// Crypto thread pool size (default: 2)
    pub crypto_thread_pool: usize,

    // Anti-Entropy Management
    /// Maximum anti-entropy sync interval (default: 300 seconds)
    pub anti_entropy_max_interval: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            // Memory defaults (as per MESH_CAPABILITIES.md §8.3)
            max_memory_mb: 2048,
            crdt_document_limit_mb: 50,
            cache_size_mb: 500,

            // Connection defaults
            max_peer_connections: 50,
            max_relay_connections: 3,
            connection_timeout: Duration::from_secs(30),

            // Bandwidth defaults (unlimited by default)
            upload_rate_limit_mbps: None,
            download_rate_limit_mbps: None,
            burst_allowance_mb: 10,

            // CPU defaults
            max_worker_threads: 4,
            crypto_thread_pool: 2,

            // Anti-entropy defaults
            anti_entropy_max_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl ResourceLimits {
    /// Create new resource limits with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create resource limits for low-resource environments (mobile, IoT)
    pub fn low_resource() -> Self {
        Self {
            max_memory_mb: 512,
            crdt_document_limit_mb: 10,
            cache_size_mb: 100,
            max_peer_connections: 20,
            max_relay_connections: 1,
            connection_timeout: Duration::from_secs(15),
            upload_rate_limit_mbps: Some(5),
            download_rate_limit_mbps: Some(20),
            burst_allowance_mb: 2,
            max_worker_threads: 2,
            crypto_thread_pool: 1,
            anti_entropy_max_interval: Duration::from_secs(600), // 10 minutes
        }
    }

    /// Create resource limits for high-performance environments (servers, desktop)
    pub fn high_performance() -> Self {
        Self {
            max_memory_mb: 8192,
            crdt_document_limit_mb: 200,
            cache_size_mb: 2048,
            max_peer_connections: 200,
            max_relay_connections: 10,
            connection_timeout: Duration::from_secs(60),
            upload_rate_limit_mbps: None,
            download_rate_limit_mbps: None,
            burst_allowance_mb: 50,
            max_worker_threads: 16,
            crypto_thread_pool: 4,
            anti_entropy_max_interval: Duration::from_secs(60), // 1 minute
        }
    }

    /// Enforce peer connection limit
    ///
    /// Returns an error if the current connection count exceeds the limit
    pub fn enforce_peer_limit(&self, current_connections: usize) -> Result<(), ResourceLimitError> {
        if current_connections >= self.max_peer_connections {
            Err(ResourceLimitError::PeerLimitExceeded {
                current: current_connections,
                limit: self.max_peer_connections,
            })
        } else {
            Ok(())
        }
    }

    /// Enforce relay connection limit
    pub fn enforce_relay_limit(&self, current_relays: usize) -> Result<(), ResourceLimitError> {
        if current_relays >= self.max_relay_connections {
            Err(ResourceLimitError::RelayLimitExceeded {
                current: current_relays,
                limit: self.max_relay_connections,
            })
        } else {
            Ok(())
        }
    }

    /// Enforce document size limit
    pub fn enforce_document_limit(
        &self,
        document_size_mb: usize,
    ) -> Result<(), ResourceLimitError> {
        if document_size_mb > self.crdt_document_limit_mb {
            Err(ResourceLimitError::DocumentTooLarge {
                size_mb: document_size_mb,
                limit_mb: self.crdt_document_limit_mb,
            })
        } else {
            Ok(())
        }
    }

    /// Check if memory usage is within limits
    pub fn check_memory_usage(&self, current_usage_mb: usize) -> Result<(), ResourceLimitError> {
        if current_usage_mb > self.max_memory_mb {
            Err(ResourceLimitError::MemoryLimitExceeded {
                current_mb: current_usage_mb,
                limit_mb: self.max_memory_mb,
            })
        } else {
            Ok(())
        }
    }

    /// Get upload rate limit in bytes per second (if set)
    pub fn upload_rate_bytes_per_sec(&self) -> Option<u64> {
        self.upload_rate_limit_mbps.map(|mbps| mbps * 1_000_000 / 8)
    }

    /// Get download rate limit in bytes per second (if set)
    pub fn download_rate_bytes_per_sec(&self) -> Option<u64> {
        self.download_rate_limit_mbps
            .map(|mbps| mbps * 1_000_000 / 8)
    }

    /// Validate limits for consistency
    pub fn validate(&self) -> Result<(), ResourceLimitError> {
        if self.max_peer_connections == 0 {
            return Err(ResourceLimitError::InvalidConfiguration(
                "max_peer_connections must be > 0".to_string(),
            ));
        }

        if self.max_memory_mb == 0 {
            return Err(ResourceLimitError::InvalidConfiguration(
                "max_memory_mb must be > 0".to_string(),
            ));
        }

        if self.crdt_document_limit_mb > self.max_memory_mb {
            return Err(ResourceLimitError::InvalidConfiguration(
                "crdt_document_limit_mb cannot exceed max_memory_mb".to_string(),
            ));
        }

        Ok(())
    }
}

/// Errors that can occur during resource limit enforcement
#[derive(Debug, Error)]
pub enum ResourceLimitError {
    #[error("Peer connection limit exceeded: {current} connections (limit: {limit})")]
    PeerLimitExceeded { current: usize, limit: usize },

    #[error("Relay connection limit exceeded: {current} relays (limit: {limit})")]
    RelayLimitExceeded { current: usize, limit: usize },

    #[error("Document size limit exceeded: {size_mb}MB (limit: {limit_mb}MB)")]
    DocumentTooLarge { size_mb: usize, limit_mb: usize },

    #[error("Memory limit exceeded: {current_mb}MB (limit: {limit_mb}MB)")]
    MemoryLimitExceeded { current_mb: usize, limit_mb: usize },

    #[error("Invalid resource limit configuration: {0}")]
    InvalidConfiguration(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_peer_connections, 50);
        assert_eq!(limits.max_memory_mb, 2048);
        assert!(limits.validate().is_ok());
    }

    #[test]
    fn test_low_resource_limits() {
        let limits = ResourceLimits::low_resource();
        assert_eq!(limits.max_peer_connections, 20);
        assert_eq!(limits.max_memory_mb, 512);
        assert!(limits.validate().is_ok());
    }

    #[test]
    fn test_high_performance_limits() {
        let limits = ResourceLimits::high_performance();
        assert_eq!(limits.max_peer_connections, 200);
        assert_eq!(limits.max_memory_mb, 8192);
        assert!(limits.validate().is_ok());
    }

    #[test]
    fn test_peer_limit_enforcement() {
        let limits = ResourceLimits::default();

        // Within limit
        assert!(limits.enforce_peer_limit(49).is_ok());

        // At limit
        assert!(limits.enforce_peer_limit(50).is_err());

        // Exceeds limit
        assert!(limits.enforce_peer_limit(51).is_err());
    }

    #[test]
    fn test_document_limit_enforcement() {
        let limits = ResourceLimits::default();

        // Within limit
        assert!(limits.enforce_document_limit(25).is_ok());

        // Exceeds limit
        assert!(limits.enforce_document_limit(51).is_err());
    }

    #[test]
    fn test_bandwidth_conversion() {
        let limits = ResourceLimits {
            upload_rate_limit_mbps: Some(10),
            download_rate_limit_mbps: Some(50),
            ..Default::default()
        };

        // 10 Mbps = 1,250,000 bytes/sec
        assert_eq!(limits.upload_rate_bytes_per_sec(), Some(1_250_000));

        // 50 Mbps = 6,250,000 bytes/sec
        assert_eq!(limits.download_rate_bytes_per_sec(), Some(6_250_000));
    }

    #[test]
    fn test_validation() {
        let mut limits = ResourceLimits::default();
        assert!(limits.validate().is_ok());

        // Invalid: zero peers
        limits.max_peer_connections = 0;
        assert!(limits.validate().is_err());
        limits.max_peer_connections = 50;

        // Invalid: document limit > memory limit
        limits.crdt_document_limit_mb = 3000;
        assert!(limits.validate().is_err());
    }
}

/**
 * Saorsa Storage System - Configuration Management
 * Implements configuration validation, persistence, and runtime updates
 */
use crate::saorsa_storage::errors::*;
use crate::saorsa_storage::{EncryptionMode, StoragePolicy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
// use std::time::Duration;  // Currently unused
use chrono::{DateTime, Utc};

/// Main storage system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage policies configuration
    pub policies: PolicyConfig,
    /// Encryption settings
    pub encryption: EncryptionConfig,
    /// Network configuration
    pub network: NetworkConfigSection,
    /// Cache configuration
    pub cache: CacheConfigSection,
    /// Performance tuning
    pub performance: PerformanceConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Logging and monitoring
    pub monitoring: MonitoringConfig,
    /// Feature flags
    pub features: FeatureFlags,
}

/// Storage policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Default storage policy for new content
    pub default_policy: StoragePolicy,
    /// Maximum content size per policy (in bytes)
    pub max_content_sizes: HashMap<String, u64>,
    /// Enable policy transitions
    pub allow_transitions: bool,
    /// Require audit trail for policy operations
    pub require_audit: bool,
    /// Default namespace for PrivateScoped policy
    pub default_namespace: String,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Default encryption mode
    pub default_mode: EncryptionMode,
    /// Key derivation iterations (for performance tuning)
    pub key_derivation_iterations: u32,
    /// Enable key rotation
    pub enable_key_rotation: bool,
    /// Key rotation interval in days
    pub key_rotation_interval_days: u32,
    /// Master key file path (encrypted)
    pub master_key_path: PathBuf,
    /// Enable hardware security module if available
    pub enable_hsm: bool,
}

/// Network configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfigSection {
    /// Operation timeout in seconds
    pub operation_timeout_secs: u64,
    /// Number of retry attempts
    pub retry_attempts: u32,
    /// Retry backoff in milliseconds
    pub retry_backoff_ms: u64,
    /// Maximum concurrent network operations
    pub max_concurrent_operations: usize,
    /// Enable geographic routing
    pub enable_geographic_routing: bool,
    /// Peer discovery interval in seconds
    pub peer_discovery_interval_secs: u64,
    /// DHT replication factor
    pub dht_replication_factor: u8,
}

/// Cache configuration section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfigSection {
    /// Maximum cache size in bytes
    pub max_size_bytes: usize,
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Default TTL in seconds
    pub default_ttl_secs: Option<u64>,
    /// Compression threshold in bytes
    pub compress_threshold: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Enable integrity checking
    pub enable_integrity_check: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Chunk size for content addressing (bytes)
    pub chunk_size: usize,
    /// Maximum chunks per content item
    pub max_chunks: u32,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level (1-9)
    pub compression_level: u8,
    /// Thread pool size for I/O operations
    pub io_thread_pool_size: usize,
    /// Buffer size for streaming operations
    pub stream_buffer_size: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable content integrity verification
    pub enable_integrity_verification: bool,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Maximum requests per minute per peer
    pub max_requests_per_minute: u32,
    /// Enable access logging
    pub enable_access_logging: bool,
    /// Minimum entropy for generated keys
    pub min_key_entropy: f64,
    /// Enable secure deletion
    pub enable_secure_deletion: bool,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_secs: u64,
    /// Log level (error, warn, info, debug, trace)
    pub log_level: String,
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Metrics export endpoint
    pub metrics_endpoint: Option<String>,
}

/// Feature flags for experimental features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enable Reed-Solomon error correction
    pub enable_reed_solomon: bool,
    /// Enable content deduplication
    pub enable_deduplication: bool,
    /// Enable geographic routing optimization
    pub enable_geo_optimization: bool,
    /// Enable experimental compression algorithms
    pub enable_experimental_compression: bool,
    /// Enable background maintenance tasks
    pub enable_background_maintenance: bool,
}

/// Configuration validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Configuration manager
pub struct ConfigManager {
    config: StorageConfig,
    config_path: Option<PathBuf>,
    last_loaded: Option<DateTime<Utc>>,
    watchers: Vec<Box<dyn ConfigWatcher>>,
}

/// Trait for configuration change notifications
pub trait ConfigWatcher: Send + Sync {
    fn on_config_changed(&self, config: &StorageConfig);
}

impl ConfigManager {
    /// Create configuration manager with default settings
    pub fn new() -> Self {
        Self {
            config: StorageConfig::default(),
            config_path: None,
            last_loaded: None,
            watchers: Vec::new(),
        }
    }

    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> StorageResult<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let content =
            std::fs::read_to_string(&path_buf).map_err(|e| StorageError::ConfigError {
                reason: format!("Failed to read config file: {}", e),
            })?;

        let config: StorageConfig =
            toml::from_str(&content).map_err(|e| StorageError::ConfigError {
                reason: format!("Failed to parse config: {}", e),
            })?;

        let manager = Self {
            config,
            config_path: Some(path_buf),
            last_loaded: Some(Utc::now()),
            watchers: Vec::new(),
        };

        // Validate configuration
        let validation = manager.validate()?;
        if !validation.is_valid {
            return Err(StorageError::ConfigError {
                reason: format!("Invalid configuration: {:?}", validation.errors),
            });
        }

        Ok(manager)
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> StorageResult<()> {
        let content =
            toml::to_string_pretty(&self.config).map_err(|e| StorageError::ConfigError {
                reason: format!("Failed to serialize config: {}", e),
            })?;

        std::fs::write(path, content).map_err(|e| StorageError::ConfigError {
            reason: format!("Failed to write config file: {}", e),
        })?;

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &StorageConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: StorageConfig) -> StorageResult<()> {
        // Validate new configuration
        let temp_manager = ConfigManager {
            config: new_config.clone(),
            config_path: None,
            last_loaded: None,
            watchers: Vec::new(),
        };

        let validation = temp_manager.validate()?;
        if !validation.is_valid {
            return Err(StorageError::ConfigError {
                reason: format!("Invalid configuration: {:?}", validation.errors),
            });
        }

        self.config = new_config;

        // Notify watchers
        for watcher in &self.watchers {
            watcher.on_config_changed(&self.config);
        }

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> StorageResult<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate policy configuration
        if self.config.policies.default_namespace.is_empty() {
            errors.push("Default namespace cannot be empty".to_string());
        }

        // Validate encryption configuration
        if self.config.encryption.key_derivation_iterations < 1000 {
            warnings.push("Key derivation iterations below 1000 may be insecure".to_string());
        }

        if self.config.encryption.key_rotation_interval_days == 0 {
            errors.push("Key rotation interval must be greater than 0".to_string());
        }

        // Validate network configuration
        if self.config.network.retry_attempts == 0 {
            errors.push("Retry attempts must be greater than 0".to_string());
        }

        if self.config.network.operation_timeout_secs == 0 {
            errors.push("Operation timeout must be greater than 0".to_string());
        }

        // Validate cache configuration
        if self.config.cache.max_size_bytes == 0 {
            errors.push("Cache max size must be greater than 0".to_string());
        }

        if self.config.cache.max_entries == 0 {
            errors.push("Cache max entries must be greater than 0".to_string());
        }

        // Validate performance configuration
        if self.config.performance.chunk_size == 0 {
            errors.push("Chunk size must be greater than 0".to_string());
        }

        if self.config.performance.max_chunks == 0 {
            errors.push("Max chunks must be greater than 0".to_string());
        }

        if self.config.performance.compression_level > 9 {
            errors.push("Compression level must be between 1 and 9".to_string());
        }

        // Validate security configuration
        if self.config.security.max_requests_per_minute == 0 {
            errors.push("Rate limit must be greater than 0".to_string());
        }

        if self.config.security.min_key_entropy < 0.0 || self.config.security.min_key_entropy > 8.0
        {
            errors.push("Key entropy must be between 0.0 and 8.0".to_string());
        }

        // Check for conflicting settings
        if !self.config.features.enable_reed_solomon
            && self.config.network.dht_replication_factor < 3
        {
            warnings.push(
                "Low replication factor without Reed-Solomon may reduce reliability".to_string(),
            );
        }

        let is_valid = errors.is_empty();
        Ok(ValidationResult {
            is_valid,
            errors,
            warnings,
        })
    }

    /// Get configuration value by path (dot notation)
    pub fn get_value(&self, path: &str) -> StorageResult<String> {
        // Simple implementation for common paths
        match path {
            "policies.default_namespace" => Ok(self.config.policies.default_namespace.clone()),
            "encryption.default_mode" => Ok(format!("{:?}", self.config.encryption.default_mode)),
            "cache.max_size_bytes" => Ok(self.config.cache.max_size_bytes.to_string()),
            "network.operation_timeout_secs" => {
                Ok(self.config.network.operation_timeout_secs.to_string())
            }
            _ => Err(StorageError::ConfigError {
                reason: format!("Unknown config path: {}", path),
            }),
        }
    }

    /// Set configuration value by path
    pub fn set_value(&mut self, path: &str, value: &str) -> StorageResult<()> {
        match path {
            "policies.default_namespace" => {
                self.config.policies.default_namespace = value.to_string();
            }
            "cache.max_size_bytes" => {
                self.config.cache.max_size_bytes =
                    value.parse().map_err(|_| StorageError::ConfigError {
                        reason: "Invalid number for cache.max_size_bytes".to_string(),
                    })?;
            }
            "network.operation_timeout_secs" => {
                self.config.network.operation_timeout_secs =
                    value.parse().map_err(|_| StorageError::ConfigError {
                        reason: "Invalid number for network.operation_timeout_secs".to_string(),
                    })?;
            }
            _ => {
                return Err(StorageError::ConfigError {
                    reason: format!("Unknown config path: {}", path),
                });
            }
        }

        // Validate after change
        let validation = self.validate()?;
        if !validation.is_valid {
            return Err(StorageError::ConfigError {
                reason: format!(
                    "Configuration invalid after change: {:?}",
                    validation.errors
                ),
            });
        }

        Ok(())
    }

    /// Add configuration change watcher
    pub fn add_watcher(&mut self, watcher: Box<dyn ConfigWatcher>) {
        self.watchers.push(watcher);
    }

    /// Reload configuration from file
    pub fn reload(&mut self) -> StorageResult<bool> {
        if let Some(config_path) = &self.config_path {
            let new_manager = Self::load_from_file(config_path)?;

            // Check if configuration actually changed
            let changed = !self.configs_equal(&self.config, &new_manager.config);

            if changed {
                self.config = new_manager.config;
                self.last_loaded = Some(Utc::now());

                // Notify watchers
                for watcher in &self.watchers {
                    watcher.on_config_changed(&self.config);
                }
            }

            Ok(changed)
        } else {
            Err(StorageError::ConfigError {
                reason: "No config file path available for reload".to_string(),
            })
        }
    }

    /// Get configuration as TOML string
    pub fn to_toml(&self) -> StorageResult<String> {
        toml::to_string_pretty(&self.config).map_err(|e| StorageError::ConfigError {
            reason: format!("Failed to serialize config: {}", e),
        })
    }

    // Private helper methods

    fn configs_equal(&self, a: &StorageConfig, b: &StorageConfig) -> bool {
        // Simple comparison by serializing both
        let a_str = toml::to_string(a).unwrap_or_default();
        let b_str = toml::to_string(b).unwrap_or_default();
        a_str == b_str
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            policies: PolicyConfig {
                default_policy: StoragePolicy::PrivateScoped {
                    namespace: "default".to_string(),
                },
                max_content_sizes: {
                    let mut sizes = HashMap::new();
                    sizes.insert("PrivateMax".to_string(), 100 * 1024 * 1024); // 100MB
                    sizes.insert("PrivateScoped".to_string(), 1024 * 1024 * 1024); // 1GB
                    sizes.insert("GroupScoped".to_string(), 5 * 1024 * 1024 * 1024); // 5GB
                    sizes.insert("PublicMarkdown".to_string(), 10 * 1024 * 1024); // 10MB
                    sizes
                },
                allow_transitions: true,
                require_audit: true,
                default_namespace: "default".to_string(),
            },
            encryption: EncryptionConfig {
                default_mode: EncryptionMode::ChaCha20Poly1305Derived,
                key_derivation_iterations: 10000,
                enable_key_rotation: true,
                key_rotation_interval_days: 90,
                master_key_path: PathBuf::from(".saorsa/master.key"),
                enable_hsm: false,
            },
            network: NetworkConfigSection {
                operation_timeout_secs: 30,
                retry_attempts: 3,
                retry_backoff_ms: 500,
                max_concurrent_operations: 10,
                enable_geographic_routing: true,
                peer_discovery_interval_secs: 60,
                dht_replication_factor: 8,
            },
            cache: CacheConfigSection {
                max_size_bytes: 100 * 1024 * 1024, // 100MB
                max_entries: 10000,
                default_ttl_secs: Some(3600), // 1 hour
                compress_threshold: 4096,     // 4KB
                cleanup_interval_secs: 300,   // 5 minutes
                enable_integrity_check: true,
            },
            performance: PerformanceConfig {
                chunk_size: 256 * 1024, // 256KB
                max_chunks: 40960,      // ~10GB max file
                enable_compression: true,
                compression_level: 6,
                io_thread_pool_size: 4,
                stream_buffer_size: 64 * 1024, // 64KB
            },
            security: SecurityConfig {
                enable_integrity_verification: true,
                enable_rate_limiting: true,
                max_requests_per_minute: 100,
                enable_access_logging: true,
                min_key_entropy: 7.0,
                enable_secure_deletion: true,
            },
            monitoring: MonitoringConfig {
                enable_metrics: true,
                metrics_interval_secs: 60,
                log_level: "info".to_string(),
                enable_profiling: false,
                metrics_endpoint: None,
            },
            features: FeatureFlags {
                enable_reed_solomon: true,
                enable_deduplication: true,
                enable_geo_optimization: true,
                enable_experimental_compression: false,
                enable_background_maintenance: true,
            },
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config_validation() {
        let manager = ConfigManager::new();
        let validation = manager.validate().unwrap();
        assert!(validation.is_valid);
    }

    #[test]
    fn test_config_serialization() {
        let manager = ConfigManager::new();
        let toml_str = manager.to_toml().unwrap();
        assert!(!toml_str.is_empty());

        // Should be able to parse it back
        let _config: StorageConfig = toml::from_str(&toml_str).unwrap();
    }

    #[test]
    fn test_config_file_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let manager = ConfigManager::new();

        // Save to file
        let save_result = manager.save_to_file(temp_file.path());
        assert!(save_result.is_ok());

        // Load from file
        let loaded_manager = ConfigManager::load_from_file(temp_file.path());
        assert!(loaded_manager.is_ok());
    }

    #[test]
    fn test_config_value_access() {
        let mut manager = ConfigManager::new();

        // Get value
        let namespace = manager.get_value("policies.default_namespace").unwrap();
        assert_eq!(namespace, "default");

        // Set value
        let set_result = manager.set_value("policies.default_namespace", "test_namespace");
        assert!(set_result.is_ok());

        let new_namespace = manager.get_value("policies.default_namespace").unwrap();
        assert_eq!(new_namespace, "test_namespace");
    }

    #[test]
    fn test_invalid_config_validation() {
        let mut config = StorageConfig::default();
        config.cache.max_size_bytes = 0; // Invalid

        let manager = ConfigManager {
            config,
            config_path: None,
            last_loaded: None,
            watchers: Vec::new(),
        };

        let validation = manager.validate().unwrap();
        assert!(!validation.is_valid);
        assert!(!validation.errors.is_empty());
    }
}

//! Post-Quantum Cryptography Configuration Module
//!
//! This module provides centralized configuration management for all PQC operations
//! across the Communitas platform, including ML-KEM-768 and ML-DSA-65 parameters,
//! security policies, performance tuning, and hybrid mode support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use thiserror::Error;

/// Comprehensive error types for PQC configuration operations
#[derive(Debug, Error, Clone)]
pub enum PqcConfigError {
    #[error("Invalid security level: {level}. Must be between 1 and 5")]
    InvalidSecurityLevel { level: u8 },

    #[error("Invalid ML-KEM parameter: {param} = {value}")]
    InvalidMlKemParameter { param: String, value: String },

    #[error("Invalid ML-DSA parameter: {param} = {value}")]
    InvalidMlDsaParameter { param: String, value: String },

    #[error("Configuration validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Hybrid mode configuration error: {details}")]
    HybridModeError { details: String },

    #[error("Performance profile '{profile}' not found")]
    ProfileNotFound { profile: String },

    #[error("Serialization error: {details}")]
    SerializationError { details: String },

    #[error("Platform compatibility error: {platform} does not support {feature}")]
    PlatformCompatibilityError { platform: String, feature: String },

    #[error("Thread safety error: {operation} failed due to lock contention")]
    ThreadSafetyError { operation: String },
}

/// Security levels for different use cases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Development and testing (Level 1)
    Development = 1,
    /// Standard security for most users (Level 3)
    Standard = 3,
    /// High security for sensitive data (Level 4)
    High = 4,
    /// Maximum security for critical operations (Level 5)
    Maximum = 5,
}

impl SecurityLevel {
    /// Convert from u8 with validation
    pub fn from_u8(level: u8) -> Result<Self, PqcConfigError> {
        match level {
            1 => Ok(Self::Development),
            3 => Ok(Self::Standard),
            4 => Ok(Self::High),
            5 => Ok(Self::Maximum),
            _ => Err(PqcConfigError::InvalidSecurityLevel { level }),
        }
    }
}

/// ML-KEM-768 specific configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlKemConfig {
    /// Key generation seed entropy bits (256-512)
    pub seed_entropy_bits: u16,
    /// Encapsulation randomness bits (256-512)
    pub encap_randomness_bits: u16,
    /// Key derivation iterations (1000-10000)
    pub key_derivation_iterations: u32,
    /// Enable constant-time operations
    pub constant_time_ops: bool,
    /// Memory optimization level (1-3)
    pub memory_optimization: u8,
}

impl MlKemConfig {
    /// Validate ML-KEM configuration parameters
    pub fn validate(&self) -> Result<(), PqcConfigError> {
        if !(256..=512).contains(&self.seed_entropy_bits) {
            return Err(PqcConfigError::InvalidMlKemParameter {
                param: "seed_entropy_bits".to_string(),
                value: self.seed_entropy_bits.to_string(),
            });
        }

        if !(256..=512).contains(&self.encap_randomness_bits) {
            return Err(PqcConfigError::InvalidMlKemParameter {
                param: "encap_randomness_bits".to_string(),
                value: self.encap_randomness_bits.to_string(),
            });
        }

        if !(1000..=10000).contains(&self.key_derivation_iterations) {
            return Err(PqcConfigError::InvalidMlKemParameter {
                param: "key_derivation_iterations".to_string(),
                value: self.key_derivation_iterations.to_string(),
            });
        }

        if !(1..=3).contains(&self.memory_optimization) {
            return Err(PqcConfigError::InvalidMlKemParameter {
                param: "memory_optimization".to_string(),
                value: self.memory_optimization.to_string(),
            });
        }

        Ok(())
    }
}

/// ML-DSA-65 specific configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlDsaConfig {
    /// Signature generation randomness bits (256-512)
    pub sig_randomness_bits: u16,
    /// Hash function iterations for security (1000-5000)
    pub hash_iterations: u32,
    /// Enable deterministic signatures
    pub deterministic_signatures: bool,
    /// Signature verification optimization level (1-3)
    pub verification_optimization: u8,
    /// Pre-computation tables for performance
    pub precompute_tables: bool,
}

impl MlDsaConfig {
    /// Validate ML-DSA configuration parameters
    pub fn validate(&self) -> Result<(), PqcConfigError> {
        if !(256..=512).contains(&self.sig_randomness_bits) {
            return Err(PqcConfigError::InvalidMlDsaParameter {
                param: "sig_randomness_bits".to_string(),
                value: self.sig_randomness_bits.to_string(),
            });
        }

        if !(1000..=5000).contains(&self.hash_iterations) {
            return Err(PqcConfigError::InvalidMlDsaParameter {
                param: "hash_iterations".to_string(),
                value: self.hash_iterations.to_string(),
            });
        }

        if !(1..=3).contains(&self.verification_optimization) {
            return Err(PqcConfigError::InvalidMlDsaParameter {
                param: "verification_optimization".to_string(),
                value: self.verification_optimization.to_string(),
            });
        }

        Ok(())
    }
}

/// Hybrid mode configuration for transition between classical and PQC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridModeConfig {
    /// Enable hybrid mode (PQC + classical)
    pub enabled: bool,
    /// Classical algorithm to combine with PQC (e.g., "ECDH-P256", "RSA-2048")
    pub classical_algorithm: String,
    /// Hybrid combination method ("concatenate", "xor", "kdf")
    pub combination_method: String,
    /// Transition phase (0.0 = full classical, 1.0 = full PQC)
    pub transition_phase: f64,
    /// Enable backward compatibility
    pub backward_compatibility: bool,
}

impl HybridModeConfig {
    /// Validate hybrid mode configuration
    pub fn validate(&self) -> Result<(), PqcConfigError> {
        if !(0.0..=1.0).contains(&self.transition_phase) {
            return Err(PqcConfigError::HybridModeError {
                details: format!(
                    "transition_phase must be between 0.0 and 1.0, got {}",
                    self.transition_phase
                ),
            });
        }

        let valid_algorithms = ["ECDH-P256", "ECDH-P384", "RSA-2048", "RSA-3072"];
        if !valid_algorithms.contains(&self.classical_algorithm.as_str()) {
            return Err(PqcConfigError::HybridModeError {
                details: format!(
                    "unsupported classical algorithm: {}",
                    self.classical_algorithm
                ),
            });
        }

        let valid_methods = ["concatenate", "xor", "kdf"];
        if !valid_methods.contains(&self.combination_method.as_str()) {
            return Err(PqcConfigError::HybridModeError {
                details: format!(
                    "unsupported combination method: {}",
                    self.combination_method
                ),
            });
        }

        Ok(())
    }
}

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target latency for operations (in milliseconds)
    pub target_latency_ms: u32,
    /// Memory usage limit (in MB)
    pub memory_limit_mb: u32,
    /// CPU utilization target (0.0-1.0)
    pub cpu_utilization_target: f64,
    /// Enable parallel processing
    pub parallel_processing: bool,
    /// Number of worker threads (0 = auto-detect)
    pub worker_threads: u32,
    /// Cache size for precomputed values (in MB)
    pub cache_size_mb: u32,
}

impl PerformanceConfig {
    /// Validate performance configuration
    pub fn validate(&self) -> Result<(), PqcConfigError> {
        if self.target_latency_ms == 0 || self.target_latency_ms > 10000 {
            return Err(PqcConfigError::ValidationFailed {
                reason: "target_latency_ms must be between 1 and 10000".to_string(),
            });
        }

        if self.memory_limit_mb < 10 || self.memory_limit_mb > 4096 {
            return Err(PqcConfigError::ValidationFailed {
                reason: "memory_limit_mb must be between 10 and 4096".to_string(),
            });
        }

        if !(0.1..=1.0).contains(&self.cpu_utilization_target) {
            return Err(PqcConfigError::ValidationFailed {
                reason: "cpu_utilization_target must be between 0.1 and 1.0".to_string(),
            });
        }

        if self.worker_threads > 64 {
            return Err(PqcConfigError::ValidationFailed {
                reason: "worker_threads must not exceed 64".to_string(),
            });
        }

        Ok(())
    }
}

/// Network security configuration for P2P connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityConfig {
    /// Enable PQC for DHT operations
    pub dht_pqc_enabled: bool,
    /// Key exchange timeout (in seconds)
    pub key_exchange_timeout_secs: u32,
    /// Maximum number of concurrent key exchanges
    pub max_concurrent_exchanges: u32,
    /// Enable forward secrecy
    pub forward_secrecy: bool,
    /// Rekeying interval (in minutes)
    pub rekey_interval_mins: u32,
}

/// Identity management configuration for PQC keypairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Key rotation interval (in days)
    pub key_rotation_days: u32,
    /// Enable key escrow for recovery
    pub key_escrow_enabled: bool,
    /// Backup key generation
    pub backup_keys: bool,
    /// Four-word address entropy bits
    pub address_entropy_bits: u16,
}

/// Storage security configuration with PQC enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSecurityConfig {
    /// Enable PQC for storage encryption
    pub storage_pqc_enabled: bool,
    /// Key derivation rounds for storage keys
    pub key_derivation_rounds: u32,
    /// Enable integrity verification
    pub integrity_verification: bool,
    /// Storage key rotation interval (in days)
    pub storage_key_rotation_days: u32,
}

/// Messaging security configuration for group and direct messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagingSecurityConfig {
    /// Enable PQC for message encryption
    pub message_pqc_enabled: bool,
    /// Perfect forward secrecy for messages
    pub message_forward_secrecy: bool,
    /// Group key rotation interval (in hours)
    pub group_key_rotation_hours: u32,
    /// Message authentication using ML-DSA
    pub message_authentication: bool,
}

/// Main PQC configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcConfig {
    /// Overall security level
    pub security_level: SecurityLevel,
    /// ML-KEM configuration
    pub ml_kem: MlKemConfig,
    /// ML-DSA configuration
    pub ml_dsa: MlDsaConfig,
    /// Hybrid mode configuration
    pub hybrid_mode: HybridModeConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Network security configuration
    pub network_security: NetworkSecurityConfig,
    /// Identity management configuration
    pub identity: IdentityConfig,
    /// Storage security configuration
    pub storage_security: StorageSecurityConfig,
    /// Messaging security configuration
    pub messaging_security: MessagingSecurityConfig,
    /// Configuration version for compatibility
    pub version: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl PqcConfig {
    /// Create development configuration with relaxed security
    pub fn development() -> Self {
        Self {
            security_level: SecurityLevel::Development,
            ml_kem: MlKemConfig {
                seed_entropy_bits: 256,
                encap_randomness_bits: 256,
                key_derivation_iterations: 1000,
                constant_time_ops: false,
                memory_optimization: 1,
            },
            ml_dsa: MlDsaConfig {
                sig_randomness_bits: 256,
                hash_iterations: 1000,
                deterministic_signatures: true,
                verification_optimization: 1,
                precompute_tables: false,
            },
            hybrid_mode: HybridModeConfig {
                enabled: true,
                classical_algorithm: "ECDH-P256".to_string(),
                combination_method: "concatenate".to_string(),
                transition_phase: 0.5,
                backward_compatibility: true,
            },
            performance: PerformanceConfig {
                target_latency_ms: 1000,
                memory_limit_mb: 256,
                cpu_utilization_target: 0.5,
                parallel_processing: false,
                worker_threads: 1,
                cache_size_mb: 16,
            },
            network_security: NetworkSecurityConfig {
                dht_pqc_enabled: false,
                key_exchange_timeout_secs: 30,
                max_concurrent_exchanges: 10,
                forward_secrecy: false,
                rekey_interval_mins: 60,
            },
            identity: IdentityConfig {
                key_rotation_days: 90,
                key_escrow_enabled: false,
                backup_keys: false,
                address_entropy_bits: 128,
            },
            storage_security: StorageSecurityConfig {
                storage_pqc_enabled: false,
                key_derivation_rounds: 1000,
                integrity_verification: true,
                storage_key_rotation_days: 30,
            },
            messaging_security: MessagingSecurityConfig {
                message_pqc_enabled: false,
                message_forward_secrecy: false,
                group_key_rotation_hours: 24,
                message_authentication: true,
            },
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create standard configuration for typical users
    pub fn standard() -> Self {
        Self {
            security_level: SecurityLevel::Standard,
            ml_kem: MlKemConfig {
                seed_entropy_bits: 384,
                encap_randomness_bits: 384,
                key_derivation_iterations: 5000,
                constant_time_ops: true,
                memory_optimization: 2,
            },
            ml_dsa: MlDsaConfig {
                sig_randomness_bits: 384,
                hash_iterations: 2500,
                deterministic_signatures: false,
                verification_optimization: 2,
                precompute_tables: true,
            },
            hybrid_mode: HybridModeConfig {
                enabled: true,
                classical_algorithm: "ECDH-P384".to_string(),
                combination_method: "kdf".to_string(),
                transition_phase: 0.8,
                backward_compatibility: true,
            },
            performance: PerformanceConfig {
                target_latency_ms: 500,
                memory_limit_mb: 512,
                cpu_utilization_target: 0.7,
                parallel_processing: true,
                worker_threads: 0, // auto-detect
                cache_size_mb: 64,
            },
            network_security: NetworkSecurityConfig {
                dht_pqc_enabled: true,
                key_exchange_timeout_secs: 15,
                max_concurrent_exchanges: 20,
                forward_secrecy: true,
                rekey_interval_mins: 30,
            },
            identity: IdentityConfig {
                key_rotation_days: 60,
                key_escrow_enabled: false,
                backup_keys: true,
                address_entropy_bits: 192,
            },
            storage_security: StorageSecurityConfig {
                storage_pqc_enabled: true,
                key_derivation_rounds: 5000,
                integrity_verification: true,
                storage_key_rotation_days: 14,
            },
            messaging_security: MessagingSecurityConfig {
                message_pqc_enabled: true,
                message_forward_secrecy: true,
                group_key_rotation_hours: 12,
                message_authentication: true,
            },
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create high security configuration for sensitive data
    pub fn high_security() -> Self {
        Self {
            security_level: SecurityLevel::High,
            ml_kem: MlKemConfig {
                seed_entropy_bits: 512,
                encap_randomness_bits: 512,
                key_derivation_iterations: 7500,
                constant_time_ops: true,
                memory_optimization: 3,
            },
            ml_dsa: MlDsaConfig {
                sig_randomness_bits: 512,
                hash_iterations: 3500,
                deterministic_signatures: false,
                verification_optimization: 3,
                precompute_tables: true,
            },
            hybrid_mode: HybridModeConfig {
                enabled: true,
                classical_algorithm: "RSA-3072".to_string(),
                combination_method: "kdf".to_string(),
                transition_phase: 0.9,
                backward_compatibility: false,
            },
            performance: PerformanceConfig {
                target_latency_ms: 200,
                memory_limit_mb: 1024,
                cpu_utilization_target: 0.8,
                parallel_processing: true,
                worker_threads: 0, // auto-detect
                cache_size_mb: 128,
            },
            network_security: NetworkSecurityConfig {
                dht_pqc_enabled: true,
                key_exchange_timeout_secs: 10,
                max_concurrent_exchanges: 30,
                forward_secrecy: true,
                rekey_interval_mins: 15,
            },
            identity: IdentityConfig {
                key_rotation_days: 30,
                key_escrow_enabled: true,
                backup_keys: true,
                address_entropy_bits: 256,
            },
            storage_security: StorageSecurityConfig {
                storage_pqc_enabled: true,
                key_derivation_rounds: 7500,
                integrity_verification: true,
                storage_key_rotation_days: 7,
            },
            messaging_security: MessagingSecurityConfig {
                message_pqc_enabled: true,
                message_forward_secrecy: true,
                group_key_rotation_hours: 6,
                message_authentication: true,
            },
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Create maximum security configuration for critical operations
    pub fn maximum_security() -> Self {
        Self {
            security_level: SecurityLevel::Maximum,
            ml_kem: MlKemConfig {
                seed_entropy_bits: 512,
                encap_randomness_bits: 512,
                key_derivation_iterations: 10000,
                constant_time_ops: true,
                memory_optimization: 3,
            },
            ml_dsa: MlDsaConfig {
                sig_randomness_bits: 512,
                hash_iterations: 5000,
                deterministic_signatures: false,
                verification_optimization: 3,
                precompute_tables: true,
            },
            hybrid_mode: HybridModeConfig {
                enabled: false, // Pure PQC mode
                classical_algorithm: "RSA-3072".to_string(),
                combination_method: "kdf".to_string(),
                transition_phase: 1.0,
                backward_compatibility: false,
            },
            performance: PerformanceConfig {
                target_latency_ms: 100,
                memory_limit_mb: 2048,
                cpu_utilization_target: 0.9,
                parallel_processing: true,
                worker_threads: 0, // auto-detect
                cache_size_mb: 256,
            },
            network_security: NetworkSecurityConfig {
                dht_pqc_enabled: true,
                key_exchange_timeout_secs: 5,
                max_concurrent_exchanges: 50,
                forward_secrecy: true,
                rekey_interval_mins: 5,
            },
            identity: IdentityConfig {
                key_rotation_days: 7,
                key_escrow_enabled: true,
                backup_keys: true,
                address_entropy_bits: 256,
            },
            storage_security: StorageSecurityConfig {
                storage_pqc_enabled: true,
                key_derivation_rounds: 10000,
                integrity_verification: true,
                storage_key_rotation_days: 1,
            },
            messaging_security: MessagingSecurityConfig {
                message_pqc_enabled: true,
                message_forward_secrecy: true,
                group_key_rotation_hours: 1,
                message_authentication: true,
            },
            version: "1.0.0".to_string(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Validate the entire configuration
    pub fn validate(&self) -> Result<(), PqcConfigError> {
        self.ml_kem.validate()?;
        self.ml_dsa.validate()?;
        self.hybrid_mode.validate()?;
        self.performance.validate()?;

        // Cross-validation checks
        if self.security_level == SecurityLevel::Development
            && self.hybrid_mode.transition_phase > 0.8
        {
            return Err(PqcConfigError::ValidationFailed {
                reason: "Development mode should not use high PQC transition phase".to_string(),
            });
        }

        if self.security_level == SecurityLevel::Maximum && self.hybrid_mode.enabled {
            return Err(PqcConfigError::ValidationFailed {
                reason: "Maximum security mode should use pure PQC, not hybrid".to_string(),
            });
        }

        Ok(())
    }

    /// Update configuration for specific security level
    pub fn update_security_level(&mut self, level: SecurityLevel) -> Result<(), PqcConfigError> {
        let new_config = match level {
            SecurityLevel::Development => Self::development(),
            SecurityLevel::Standard => Self::standard(),
            SecurityLevel::High => Self::high_security(),
            SecurityLevel::Maximum => Self::maximum_security(),
        };

        *self = new_config;
        self.validate()?;
        Ok(())
    }
}

/// Thread-safe configuration manager
pub struct PqcConfigManager {
    config: Arc<RwLock<PqcConfig>>,
    performance_profiles: Arc<RwLock<HashMap<String, PerformanceConfig>>>,
}

impl PqcConfigManager {
    /// Create new configuration manager with standard settings
    pub fn new() -> Self {
        let mut profiles = HashMap::new();

        // Add predefined performance profiles
        profiles.insert(
            "low_latency".to_string(),
            PerformanceConfig {
                target_latency_ms: 50,
                memory_limit_mb: 1024,
                cpu_utilization_target: 0.9,
                parallel_processing: true,
                worker_threads: 0,
                cache_size_mb: 256,
            },
        );

        profiles.insert(
            "balanced".to_string(),
            PerformanceConfig {
                target_latency_ms: 200,
                memory_limit_mb: 512,
                cpu_utilization_target: 0.7,
                parallel_processing: true,
                worker_threads: 0,
                cache_size_mb: 128,
            },
        );

        profiles.insert(
            "low_power".to_string(),
            PerformanceConfig {
                target_latency_ms: 1000,
                memory_limit_mb: 256,
                cpu_utilization_target: 0.3,
                parallel_processing: false,
                worker_threads: 1,
                cache_size_mb: 32,
            },
        );

        Self {
            config: Arc::new(RwLock::new(PqcConfig::standard())),
            performance_profiles: Arc::new(RwLock::new(profiles)),
        }
    }

    /// Get current configuration (read-only)
    pub fn get_config(&self) -> Result<PqcConfig, PqcConfigError> {
        self.config
            .read()
            .map_err(|_| PqcConfigError::ThreadSafetyError {
                operation: "read_config".to_string(),
            })
            .map(|guard| guard.clone())
    }

    /// Update configuration
    pub fn update_config(&self, new_config: PqcConfig) -> Result<(), PqcConfigError> {
        new_config.validate()?;

        let mut config = self
            .config
            .write()
            .map_err(|_| PqcConfigError::ThreadSafetyError {
                operation: "write_config".to_string(),
            })?;

        *config = new_config;
        Ok(())
    }

    /// Update security level
    pub fn update_security_level(&self, level: SecurityLevel) -> Result<(), PqcConfigError> {
        let mut config = self
            .config
            .write()
            .map_err(|_| PqcConfigError::ThreadSafetyError {
                operation: "update_security_level".to_string(),
            })?;

        config.update_security_level(level)?;
        Ok(())
    }

    /// Apply performance profile
    pub fn apply_performance_profile(&self, profile_name: &str) -> Result<(), PqcConfigError> {
        let profiles =
            self.performance_profiles
                .read()
                .map_err(|_| PqcConfigError::ThreadSafetyError {
                    operation: "read_profiles".to_string(),
                })?;

        let profile = profiles
            .get(profile_name)
            .ok_or_else(|| PqcConfigError::ProfileNotFound {
                profile: profile_name.to_string(),
            })?
            .clone();

        drop(profiles);

        let mut config = self
            .config
            .write()
            .map_err(|_| PqcConfigError::ThreadSafetyError {
                operation: "apply_profile".to_string(),
            })?;

        config.performance = profile;
        config.validate()?;
        Ok(())
    }

    /// Add custom performance profile
    pub fn add_performance_profile(
        &self,
        name: String,
        profile: PerformanceConfig,
    ) -> Result<(), PqcConfigError> {
        profile.validate()?;

        let mut profiles =
            self.performance_profiles
                .write()
                .map_err(|_| PqcConfigError::ThreadSafetyError {
                    operation: "add_profile".to_string(),
                })?;

        profiles.insert(name, profile);
        Ok(())
    }

    /// Serialize configuration to JSON
    pub fn serialize_config(&self) -> Result<String, PqcConfigError> {
        let config = self.get_config()?;
        serde_json::to_string_pretty(&config).map_err(|e| PqcConfigError::SerializationError {
            details: e.to_string(),
        })
    }

    /// Deserialize configuration from JSON
    pub fn deserialize_config(&self, json: &str) -> Result<(), PqcConfigError> {
        let config: PqcConfig =
            serde_json::from_str(json).map_err(|e| PqcConfigError::SerializationError {
                details: e.to_string(),
            })?;

        self.update_config(config)?;
        Ok(())
    }
}

impl Default for PqcConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global configuration manager instance
static CONFIG_MANAGER: std::sync::OnceLock<PqcConfigManager> = std::sync::OnceLock::new();

/// Get global configuration manager instance
pub fn get_config_manager() -> &'static PqcConfigManager {
    CONFIG_MANAGER.get_or_init(PqcConfigManager::new)
}

// Tauri command interface for frontend access
#[cfg(feature = "tauri")]
pub mod tauri_commands {
    use super::*;
    use tauri::command;

    #[command]
    pub async fn get_pqc_config() -> Result<PqcConfig, String> {
        get_config_manager().get_config().map_err(|e| e.to_string())
    }

    #[command]
    pub async fn update_pqc_security_level(level: u8) -> Result<(), String> {
        let security_level = SecurityLevel::from_u8(level).map_err(|e| e.to_string())?;

        get_config_manager()
            .update_security_level(security_level)
            .map_err(|e| e.to_string())
    }

    #[command]
    pub async fn apply_pqc_performance_profile(profile_name: String) -> Result<(), String> {
        get_config_manager()
            .apply_performance_profile(&profile_name)
            .map_err(|e| e.to_string())
    }

    #[command]
    pub async fn get_pqc_config_json() -> Result<String, String> {
        get_config_manager()
            .serialize_config()
            .map_err(|e| e.to_string())
    }

    #[command]
    pub async fn load_pqc_config_json(json: String) -> Result<(), String> {
        get_config_manager()
            .deserialize_config(&json)
            .map_err(|e| e.to_string())
    }

    #[command]
    pub async fn validate_pqc_config() -> Result<bool, String> {
        match get_config_manager().get_config() {
            Ok(config) => config.validate().map(|_| true).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_level_validation() {
        assert!(SecurityLevel::from_u8(1).is_ok());
        assert!(SecurityLevel::from_u8(3).is_ok());
        assert!(SecurityLevel::from_u8(4).is_ok());
        assert!(SecurityLevel::from_u8(5).is_ok());
        assert!(SecurityLevel::from_u8(2).is_err());
        assert!(SecurityLevel::from_u8(6).is_err());
    }

    #[test]
    fn test_ml_kem_config_validation() {
        let mut config = PqcConfig::development().ml_kem;
        assert!(config.validate().is_ok());

        config.seed_entropy_bits = 100; // Too low
        assert!(config.validate().is_err());

        config.seed_entropy_bits = 600; // Too high
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ml_dsa_config_validation() {
        let mut config = PqcConfig::development().ml_dsa;
        assert!(config.validate().is_ok());

        config.hash_iterations = 500; // Too low
        assert!(config.validate().is_err());

        config.hash_iterations = 6000; // Too high
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_hybrid_mode_validation() {
        let mut config = PqcConfig::development().hybrid_mode;
        assert!(config.validate().is_ok());

        config.transition_phase = -0.1; // Invalid range
        assert!(config.validate().is_err());

        config.transition_phase = 1.1; // Invalid range
        assert!(config.validate().is_err());

        config.transition_phase = 0.5;
        config.classical_algorithm = "INVALID".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_performance_config_validation() {
        let mut config = PqcConfig::development().performance;
        assert!(config.validate().is_ok());

        config.target_latency_ms = 0; // Invalid
        assert!(config.validate().is_err());

        config.target_latency_ms = 500;
        config.memory_limit_mb = 5; // Too low
        assert!(config.validate().is_err());

        config.memory_limit_mb = 512;
        config.cpu_utilization_target = 1.5; // Too high
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_presets() {
        let dev_config = PqcConfig::development();
        assert!(dev_config.validate().is_ok());
        assert_eq!(dev_config.security_level, SecurityLevel::Development);

        let std_config = PqcConfig::standard();
        assert!(std_config.validate().is_ok());
        assert_eq!(std_config.security_level, SecurityLevel::Standard);

        let high_config = PqcConfig::high_security();
        assert!(high_config.validate().is_ok());
        assert_eq!(high_config.security_level, SecurityLevel::High);

        let max_config = PqcConfig::maximum_security();
        assert!(max_config.validate().is_ok());
        assert_eq!(max_config.security_level, SecurityLevel::Maximum);
    }

    #[test]
    fn test_config_manager() {
        let manager = PqcConfigManager::new();

        // Test initial config
        let config = manager.get_config().expect("Failed to get config");
        assert_eq!(config.security_level, SecurityLevel::Standard);

        // Test security level update
        manager
            .update_security_level(SecurityLevel::High)
            .expect("Failed to update security level");

        let updated_config = manager.get_config().expect("Failed to get updated config");
        assert_eq!(updated_config.security_level, SecurityLevel::High);

        // Test performance profile
        manager
            .apply_performance_profile("low_latency")
            .expect("Failed to apply performance profile");

        let profile_config = manager.get_config().expect("Failed to get profile config");
        assert_eq!(profile_config.performance.target_latency_ms, 50);
    }

    #[test]
    fn test_serialization() {
        let manager = PqcConfigManager::new();

        // Test serialization
        let json = manager.serialize_config().expect("Failed to serialize");
        assert!(!json.is_empty());

        // Test deserialization
        manager
            .deserialize_config(&json)
            .expect("Failed to deserialize");

        // Verify config is still valid
        let config = manager.get_config().expect("Failed to get config");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cross_validation() {
        let mut config = PqcConfig::development();
        config.hybrid_mode.transition_phase = 0.9; // Too high for development
        assert!(config.validate().is_err());

        let mut max_config = PqcConfig::maximum_security();
        max_config.hybrid_mode.enabled = true; // Should be false for maximum security
        assert!(max_config.validate().is_err());
    }
}

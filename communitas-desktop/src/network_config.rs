// Copyright (c) 2025 Saorsa Labs Limited
//
// Network configuration loader for production deployment
//
// Reads and validates network configuration from TOML files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub network: NetworkSettings,
    pub bootstrap: BootstrapConfig,
    pub nat_traversal: NatTraversalConfig,
    pub transport: TransportConfig,
    pub discovery: DiscoveryConfig,
    pub gossip: GossipConfig,
    pub presence: PresenceConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    #[serde(default)]
    pub resource_limits: ResourceLimitsConfig,
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    pub enabled: bool,
    pub network_id: String,
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub nodes: Vec<String>,
    #[serde(default)]
    pub verification: BootstrapVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BootstrapVerification {
    pub require_tls: bool,
    pub certificate_fingerprints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    pub enabled: bool,
    #[serde(default)]
    pub hole_punching: HolePunchingConfig,
    #[serde(default)]
    pub relay: RelayConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolePunchingConfig {
    pub enabled: bool,
    pub max_retries: usize,
    pub timeout_seconds: u64,
}

impl Default for HolePunchingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 5,
            timeout_seconds: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfig {
    pub enabled: bool,
    pub max_relay_peers: usize,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_relay_peers: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub quic: QuicConfig,
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuicConfig {
    pub enable_0rtt: bool,
    pub connection_timeout_seconds: u64,
    pub keep_alive_interval_seconds: u64,
    pub max_concurrent_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    pub certificate_validation: String,
    pub tls_version: String,
    pub cipher_suites: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub foaf: FoafDiscoveryConfig,
    pub dns: DnsDiscoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoafDiscoveryConfig {
    pub enabled: bool,
    pub max_depth: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsDiscoveryConfig {
    pub enabled: bool,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    pub hyparview: HyParViewConfig,
    pub plumtree: PlumTreeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyParViewConfig {
    pub active_view_size: usize,
    pub passive_view_size: usize,
    pub shuffle_interval_seconds: u64,
    pub max_retry_attempts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlumTreeConfig {
    pub eager_push_peers: usize,
    pub lazy_push_peers: usize,
    pub message_ttl: usize,
    pub duplicate_cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceConfig {
    pub beacons: PresenceBeaconsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceBeaconsConfig {
    pub broadcast_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub include_location: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub post_quantum: PostQuantumConfig,
    pub encryption: EncryptionConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostQuantumConfig {
    pub enabled: bool,
    pub key_rotation_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub require_e2e_encryption: bool,
    pub min_key_strength_bits: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics: MetricsConfig,
    pub health_checks: HealthChecksConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub report_interval_seconds: u64,
    pub export_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthChecksConfig {
    pub enabled: bool,
    pub check_interval_seconds: u64,
    pub peer_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub bootstrap: Option<EnvironmentBootstrap>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBootstrap {
    pub nodes: Vec<String>,
}

/// Resource limits configuration (Phase 2 TDD - MESH_CAPABILITIES.md §8.3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimitsConfig {
    #[serde(default = "default_max_peer_connections")]
    pub max_peer_connections: usize,
    #[serde(default = "default_max_relay_connections")]
    pub max_relay_connections: usize,
    #[serde(default = "default_connection_timeout_secs")]
    pub connection_timeout_secs: u64,
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: usize,
    #[serde(default = "default_crdt_document_limit_mb")]
    pub crdt_document_limit_mb: usize,
    pub upload_rate_limit_mbps: Option<u64>,
    pub download_rate_limit_mbps: Option<u64>,
}

fn default_max_peer_connections() -> usize { 50 }
fn default_max_relay_connections() -> usize { 3 }
fn default_connection_timeout_secs() -> u64 { 30 }
fn default_max_memory_mb() -> usize { 2048 }
fn default_crdt_document_limit_mb() -> usize { 50 }

impl Default for ResourceLimitsConfig {
    fn default() -> Self {
        Self {
            max_peer_connections: default_max_peer_connections(),
            max_relay_connections: default_max_relay_connections(),
            connection_timeout_secs: default_connection_timeout_secs(),
            max_memory_mb: default_max_memory_mb(),
            crdt_document_limit_mb: default_crdt_document_limit_mb(),
            upload_rate_limit_mbps: None,
            download_rate_limit_mbps: None,
        }
    }
}

impl ResourceLimitsConfig {
    /// Convert to communitas_core::ResourceLimits
    pub fn to_core_limits(&self) -> communitas_core::ResourceLimits {
        communitas_core::ResourceLimits {
            max_memory_mb: self.max_memory_mb,
            crdt_document_limit_mb: self.crdt_document_limit_mb,
            cache_size_mb: 500, // Default from spec
            max_peer_connections: self.max_peer_connections,
            max_relay_connections: self.max_relay_connections,
            connection_timeout: Duration::from_secs(self.connection_timeout_secs),
            upload_rate_limit_mbps: self.upload_rate_limit_mbps,
            download_rate_limit_mbps: self.download_rate_limit_mbps,
            burst_allowance_mb: 10, // Default from spec
            max_worker_threads: 4, // Default from spec
            crypto_thread_pool: 2, // Default from spec
            anti_entropy_max_interval: Duration::from_secs(300), // Default from spec
        }
    }
}

impl NetworkConfig {
    /// Load network configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, NetworkConfigError> {
        let path = path.as_ref();
        info!("Loading network configuration from: {}", path.display());

        let contents = fs::read_to_string(path)
            .map_err(|e| NetworkConfigError::Io(e.to_string(), path.to_path_buf()))?;

        let mut config: NetworkConfig =
            toml::from_str(&contents).map_err(NetworkConfigError::Toml)?;

        // Apply environment-specific overrides
        if let Ok(env) = std::env::var("COMMUNITAS_ENV")
            && let Some(env_config) = config.environments.get(&env)
            && let Some(bootstrap) = &env_config.bootstrap
        {
            config.bootstrap.nodes = bootstrap.nodes.clone();
            info!(
                "Applied {} environment bootstrap nodes: {:?}",
                env, bootstrap.nodes
            );
        }

        // Validate configuration
        config.validate()?;

        info!("Network configuration loaded successfully");
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), NetworkConfigError> {
        // Validate network settings
        if self.network.network_id.is_empty() {
            return Err(NetworkConfigError::Validation(
                "network_id cannot be empty".to_string(),
            ));
        }

        // Validate bootstrap nodes
        if self.bootstrap.nodes.is_empty() {
            warn!("No bootstrap nodes configured - peer discovery may be limited");
        }

        for node in &self.bootstrap.nodes {
            if !node.contains(':') {
                return Err(NetworkConfigError::Validation(format!(
                    "Invalid bootstrap node format (expected host:port): {}",
                    node
                )));
            }
        }

        // Validate QUIC settings
        if self.transport.quic.max_concurrent_connections == 0 {
            return Err(NetworkConfigError::Validation(
                "max_concurrent_connections must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get bootstrap nodes for current environment
    pub fn get_bootstrap_nodes(&self) -> &[String] {
        &self.bootstrap.nodes
    }

    /// Check if network is enabled
    pub fn is_network_enabled(&self) -> bool {
        self.network.enabled
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NetworkConfigError {
    #[error("IO error reading {1}: {0}")]
    Io(String, std::path::PathBuf),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Configuration validation error: {0}")]
    Validation(String),

    #[error("Mutex lock error: {0}")]
    Mutex(String),
}

// Global network configuration instance
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

static NETWORK_CONFIG: Lazy<Arc<Mutex<Option<NetworkConfig>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Get the global network configuration
pub fn get_network_config() -> Result<NetworkConfig, NetworkConfigError> {
    let mut config_guard = NETWORK_CONFIG
        .lock()
        .map_err(|e| NetworkConfigError::Mutex(e.to_string()))?;

    if let Some(ref config) = *config_guard {
        return Ok(config.clone());
    }

    // Try to load from environment-specific config first
    let config_path = std::env::var("COMMUNITAS_NETWORK_CONFIG")
        .unwrap_or_else(|_| "config/production-network.toml".to_string());

    let config = NetworkConfig::load_from_file(&config_path)?;
    *config_guard = Some(config.clone());
    Ok(config)
}

// Tauri commands for network configuration
#[tauri::command]
pub fn network_config_get_bootstrap_nodes() -> Result<Vec<String>, String> {
    let config =
        get_network_config().map_err(|e| format!("Failed to load network config: {}", e))?;

    Ok(config.get_bootstrap_nodes().to_vec())
}

#[tauri::command]
pub fn network_config_is_network_enabled() -> Result<bool, String> {
    let config =
        get_network_config().map_err(|e| format!("Failed to load network config: {}", e))?;

    Ok(config.is_network_enabled())
}

#[tauri::command]
pub fn network_config_validate() -> Result<bool, String> {
    match get_network_config() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Network config validation failed: {}", e)),
    }
}

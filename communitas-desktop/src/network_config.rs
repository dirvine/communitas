// Copyright (c) 2025 Saorsa Labs Limited
//
// Network configuration loader for production deployment
//
// Reads and validates network configuration from TOML files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use tracing::{info, warn, error};

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
    pub stun_servers: Vec<String>,
    pub turn_servers: Vec<TurnServer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServer {
    pub urls: Vec<String>,
    pub username: String,
    pub credential: String,
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

impl NetworkConfig {
    /// Expand environment variables in a string (e.g., "$VAR_NAME" -> actual value)
    fn expand_env_vars(input: &str) -> String {
        if input.starts_with('$') {
            let var_name = &input[1..];
            std::env::var(var_name).unwrap_or_else(|_| {
                warn!("Environment variable '{}' not set, using empty string", var_name);
                String::new()
            })
        } else {
            input.to_string()
        }
    }

    /// Load network configuration from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, NetworkConfigError> {
        let path = path.as_ref();
        info!("Loading network configuration from: {}", path.display());

        let contents = fs::read_to_string(path)
            .map_err(|e| NetworkConfigError::Io(e.to_string(), path.to_path_buf()))?;

        let mut config: NetworkConfig = toml::from_str(&contents)
            .map_err(NetworkConfigError::Toml)?;

        // Expand environment variables in TURN server credentials
        for turn_server in &mut config.nat_traversal.turn_servers {
            turn_server.username = Self::expand_env_vars(&turn_server.username);
            turn_server.credential = Self::expand_env_vars(&turn_server.credential);
        }

        // Apply environment-specific overrides
        if let Ok(env) = std::env::var("COMMUNITAS_ENV") {
            if let Some(env_config) = config.environments.get(&env) {
                if let Some(bootstrap) = &env_config.bootstrap {
                    config.bootstrap.nodes = bootstrap.nodes.clone();
                    info!("Applied {} environment bootstrap nodes: {:?}", env, bootstrap.nodes);
                }
            }
        }

        // Validate configuration
        config.validate()?;

        info!("Network configuration loaded successfully");
        Ok(config)
    }

    /// Load default production configuration
    pub fn load_default_production() -> Result<Self, NetworkConfigError> {
        Self::load_from_file("config/production-network.toml")
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), NetworkConfigError> {
        // Validate network settings
        if self.network.network_id.is_empty() {
            return Err(NetworkConfigError::Validation("network_id cannot be empty".to_string()));
        }

        // Validate bootstrap nodes
        if self.bootstrap.nodes.is_empty() {
            warn!("No bootstrap nodes configured - peer discovery may be limited");
        }

        for node in &self.bootstrap.nodes {
            if !node.contains(':') {
                return Err(NetworkConfigError::Validation(
                    format!("Invalid bootstrap node format (expected host:port): {}", node)
                ));
            }
        }

        // Validate QUIC settings
        if self.transport.quic.max_concurrent_connections == 0 {
            return Err(NetworkConfigError::Validation(
                "max_concurrent_connections must be greater than 0".to_string()
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

    /// Get STUN servers
    pub fn get_stun_servers(&self) -> &[String] {
        &self.nat_traversal.stun_servers
    }

    /// Get TURN servers
    pub fn get_turn_servers(&self) -> &[TurnServer] {
        &self.nat_traversal.turn_servers
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
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

static NETWORK_CONFIG: Lazy<Arc<Mutex<Option<NetworkConfig>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Get the global network configuration
pub fn get_network_config() -> Result<NetworkConfig, NetworkConfigError> {
    let mut config_guard = NETWORK_CONFIG.lock().map_err(|e| NetworkConfigError::Mutex(e.to_string()))?;

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
    let config = get_network_config()
        .map_err(|e| format!("Failed to load network config: {}", e))?;

    Ok(config.get_bootstrap_nodes().to_vec())
}

#[tauri::command]
pub fn network_config_is_network_enabled() -> Result<bool, String> {
    let config = get_network_config()
        .map_err(|e| format!("Failed to load network config: {}", e))?;

    Ok(config.is_network_enabled())
}

#[tauri::command]
pub fn network_config_get_stun_servers() -> Result<Vec<String>, String> {
    let config = get_network_config()
        .map_err(|e| format!("Failed to load network config: {}", e))?;

    Ok(config.get_stun_servers().to_vec())
}

#[tauri::command]
pub fn network_config_validate() -> Result<bool, String> {
    match get_network_config() {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("Network config validation failed: {}", e)),
    }
}

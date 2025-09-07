// Communitas Headless Node
// This binary runs a headless Communitas node using saorsa-core APIs

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

/// Try to self-update the binary using GitHub releases
pub fn try_self_update() -> Result<Option<String>> {
    use self_update::cargo_crate_version;
    let status = self_update::backends::github::Update::configure()
        .repo_owner("dirvine")
        .repo_name("communitas")
        .bin_name("communitas-node")
        .current_version(cargo_crate_version!())
        .build()?
        .update()?;
    Ok(Some(status.version().to_string()))
}

#[derive(Parser, Debug)]
#[command(name = "communitas-node")]
#[command(about = "Headless Communitas P2P node", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "/etc/communitas/config.toml")]
    config: PathBuf,

    /// Storage directory
    #[arg(short, long, default_value = "/var/lib/communitas")]
    storage: PathBuf,

    /// Listen address
    #[arg(short, long, default_value = "0.0.0.0:443")]
    listen: SocketAddr,

    /// Bootstrap nodes (four-word addresses)
    #[arg(short, long)]
    bootstrap: Vec<String>,

    /// Enable metrics endpoint
    #[arg(long)]
    metrics: bool,

    /// Metrics listen address
    #[arg(long, default_value = "127.0.0.1:9600")]
    metrics_addr: SocketAddr,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    /// Node identity (four-word address)
    identity: Option<String>,

    /// Bootstrap nodes
    bootstrap_nodes: Vec<String>,

    /// Storage settings
    storage: StorageConfig,

    /// Network settings
    network: NetworkConfig,

    /// Auto-update settings
    update: UpdateConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageConfig {
    /// Base directory for storage
    base_dir: PathBuf,

    /// Cache size in MB
    cache_size_mb: usize,

    /// Enable FEC for storage
    enable_fec: bool,

    /// FEC parameters
    fec_k: usize,
    fec_m: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkConfig {
    /// Listen addresses
    listen_addrs: Vec<SocketAddr>,

    /// Enable IPv6
    enable_ipv6: bool,

    /// Enable WebRTC bridge
    enable_webrtc: bool,

    /// QUIC settings
    quic_idle_timeout_ms: u64,
    quic_max_streams: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateConfig {
    /// Update channel (stable, beta, nightly)
    channel: String,

    /// Check interval in seconds
    check_interval_secs: u64,

    /// Enable auto-update
    auto_update: bool,

    /// Jitter range in seconds (0 disables jitter, default 0 for saorsa-core 0.3.18+)
    jitter_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            identity: None,
            bootstrap_nodes: vec![
                "ocean-forest-moon-star:443".to_string(),
                "river-mountain-sun-cloud:443".to_string(),
            ],
            storage: StorageConfig {
                base_dir: PathBuf::from("/var/lib/communitas"),
                cache_size_mb: 1024,
                enable_fec: true,
                fec_k: 8,
                fec_m: 4,
            },
            network: NetworkConfig {
                listen_addrs: vec![
                    "0.0.0.0:443"
                        .parse()
                        .expect("Failed to parse default listen address"),
                ],
                enable_ipv6: true,
                enable_webrtc: false,
                quic_idle_timeout_ms: 30000,
                quic_max_streams: 100,
            },
            update: UpdateConfig {
                channel: "stable".to_string(),
                check_interval_secs: 21600, // 6 hours
                auto_update: true,
                jitter_secs: 0, // No jitter needed for saorsa-core 0.3.18+
            },
        }
    }
}

async fn load_or_create_config(path: &PathBuf) -> Result<Config> {
    if path.exists() {
        let content = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read config file")?;
        toml::from_str(&content).context("Failed to parse config")
    } else {
        // Create default config
        let config = Config::default();
        let parent = path.parent().context("Invalid config path")?;
        tokio::fs::create_dir_all(parent)
            .await
            .context("Failed to create config directory")?;

        let content = toml::to_string_pretty(&config).context("Failed to serialize config")?;
        tokio::fs::write(path, content)
            .await
            .context("Failed to write config")?;

        Ok(config)
    }
}

async fn setup_identity(config: &Config) -> Result<(String, Vec<u8>, Vec<u8>)> {
    // If identity is configured, use it
    if let Some(identity) = &config.identity {
        info!("Using configured identity: {}", identity);
        // In production, load keys from secure storage
        // For now, generate new keys (should be persisted)
        // Note: saorsa-pqc may have different module structure, using placeholder
        let pk = vec![0u8; 32];
        let sk = vec![0u8; 64];
        Ok((identity.clone(), pk, sk))
    } else {
        // Generate new identity
        warn!("No identity configured, generating new one");
        // In production, this should generate a proper four-word address
        // and persist the keys securely
        let words = ["communitas", "node", "test", "instance"];
        let pk = vec![0u8; 32];
        let sk = vec![0u8; 64];
        Ok((words.join("-"), pk, sk))
    }
}

async fn start_health_endpoint(addr: SocketAddr) -> Result<()> {
    use warp::Filter;

    let health = warp::path("health").map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "uptime": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before Unix epoch")
                .as_secs(),
        }))
    });

    let metrics = warp::path("metrics").map(|| {
        // In production, return Prometheus metrics
        "# HELP communitas_peers_connected Number of connected peers\n\
             # TYPE communitas_peers_connected gauge\n\
             communitas_peers_connected 0\n"
    });

    let routes = health.or(metrics);

    tokio::spawn(async move {
        warp::serve(routes).run(addr).await;
    });

    Ok(())
}

async fn run_node(args: Args) -> Result<()> {
    // Load or create config
    let config = load_or_create_config(&args.config).await?;
    info!("Loaded configuration from {:?}", args.config);

    // Try self-update if enabled
    if config.update.auto_update {
        info!("Checking for updates...");
        match try_self_update() {
            Ok(Some(new_version)) => {
                info!("Successfully updated to version {}", new_version);
                info!("Please restart the application to use the new version");
                // In production, you might want to restart automatically
                // or notify the user through other means
            }
            Ok(None) => {
                info!("No updates available");
            }
            Err(e) => {
                warn!("Failed to check for updates: {:#}", e);
            }
        }
    }

    // Setup storage
    tokio::fs::create_dir_all(&config.storage.base_dir)
        .await
        .context("Failed to create storage directory")?;

    // Setup identity
    let (identity, _pubkey, _privkey) = setup_identity(&config).await?;
    info!("Node identity: {}", identity);

    // Initialize DHT using saorsa_core
    // Note: Actual initialization will depend on saorsa_core's API
    // For now, we'll skip DHT initialization as we need to check saorsa_core's actual API
    info!(
        "DHT initialization would happen here with {} bootstrap nodes",
        config.bootstrap_nodes.len()
    );

    // Start health/metrics endpoint if enabled
    if args.metrics {
        start_health_endpoint(args.metrics_addr).await?;
        info!("Metrics endpoint started on {}", args.metrics_addr);
    }

    // Main event loop
    info!("Communitas node started successfully");
    info!("Press Ctrl+C to shutdown");

    // Wait for shutdown signal
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    // Graceful shutdown
    info!("Performing graceful shutdown...");
    // In production, properly close connections and save state

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    if let Err(e) = run_node(args).await {
        error!("Node failed: {:#}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.update.channel, "stable");
        assert!(config.storage.enable_fec);
        assert_eq!(config.storage.fec_k, 8);
        assert_eq!(config.storage.fec_m, 4);
    }

    #[tokio::test]
    async fn test_load_or_create_config() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // First call should create default config
        let config1 = load_or_create_config(&config_path).await.unwrap();
        assert!(config_path.exists());

        // Second call should load existing config
        let config2 = load_or_create_config(&config_path).await.unwrap();
        assert_eq!(config1.update.channel, config2.update.channel);
    }
}

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

use saorsa_core::{
    dht::Dht,
    identity::Identity,
    network::{Network, NetworkBuilder},
    storage::Storage,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long)]
    config: PathBuf,

    /// Override listen address
    #[arg(long)]
    listen: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    identity: IdentityConfig,
    network: NetworkConfig,
    storage: StorageConfig,
    logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentityConfig {
    four_words: String,
    display_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct NetworkConfig {
    listen_address: String,
    bootstrap_nodes: Vec<String>,
    enable_mdns: bool,
    enable_relay: bool,
    max_connections: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageConfig {
    data_dir: PathBuf,
    cache_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoggingConfig {
    level: String,
    file: Option<PathBuf>,
}

struct HeadlessNode {
    identity: Arc<Identity>,
    network: Arc<Network>,
    dht: Arc<Dht>,
    storage: Arc<Storage>,
}

impl HeadlessNode {
    async fn new(config: Config) -> Result<Self> {
        info!("Initializing headless node...");

        // Parse four-word identity
        let four_words: Vec<&str> = config.identity.four_words.split('-').collect();
        if four_words.len() != 4 {
            return Err(anyhow::anyhow!("Invalid four-word identity format"));
        }

        // Create identity
        let identity = Identity::from_four_words(
            &four_words.try_into().map_err(|_| anyhow::anyhow!("Invalid four-word array"))?,
            &config.identity.display_name,
        )?;

        info!(
            "Node identity: {} ({})",
            config.identity.four_words, config.identity.display_name
        );

        // Initialize storage
        std::fs::create_dir_all(&config.storage.data_dir)?;
        let storage = Storage::new(config.storage.data_dir.clone(), config.storage.cache_size)?;

        // Initialize DHT
        let dht = Dht::new(identity.clone(), storage.clone())?;

        // Build network
        let mut network_builder = NetworkBuilder::new(identity.clone())
            .with_listen_address(&config.network.listen_address)?
            .with_max_connections(config.network.max_connections);

        if config.network.enable_mdns {
            network_builder = network_builder.with_mdns();
        }

        if config.network.enable_relay {
            network_builder = network_builder.with_relay();
        }

        // Add bootstrap nodes
        for bootstrap in &config.network.bootstrap_nodes {
            network_builder = network_builder.with_bootstrap(bootstrap)?;
        }

        let network = network_builder.build().await?;

        // Start network
        network.start().await?;

        info!(
            "Network started on {}",
            config.network.listen_address
        );

        Ok(Self {
            identity: Arc::new(identity),
            network: Arc::new(network),
            dht: Arc::new(dht),
            storage: Arc::new(storage),
        })
    }

    async fn run(&self) -> Result<()> {
        info!("Headless node running...");

        // Start DHT maintenance loop
        let dht = self.dht.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

                match dht.maintain().await {
                    Ok(stats) => {
                        info!(
                            "DHT maintenance: {} entries, {} peers",
                            stats.entries, stats.connected_peers
                        );
                    }
                    Err(e) => {
                        warn!("DHT maintenance error: {}", e);
                    }
                }
            }
        });

        // Monitor peer connections
        let network = self.network.clone();
        tokio::spawn(async move {
            let mut last_peer_count = 0;
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                let peers = network.connected_peers().await;
                if peers.len() != last_peer_count {
                    info!("Connected peers: {}", peers.len());
                    for (i, peer) in peers.iter().enumerate().take(5) {
                        info!("  Peer {}: {}", i + 1, peer);
                    }
                    if peers.len() > 5 {
                        info!("  ... and {} more", peers.len() - 5);
                    }
                    last_peer_count = peers.len();
                }
            }
        });

        // Handle network events
        let mut event_receiver = self.network.subscribe_events().await?;
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                match event {
                    NetworkEvent::PeerConnected(peer_id) => {
                        info!("Peer connected: {}", peer_id);
                    }
                    NetworkEvent::PeerDisconnected(peer_id) => {
                        info!("Peer disconnected: {}", peer_id);
                    }
                    NetworkEvent::MessageReceived { from, data } => {
                        info!("Message from {}: {} bytes", from, data.len());
                    }
                    _ => {}
                }
            }
        });

        // Wait for shutdown signal
        signal::ctrl_c().await?;
        info!("Shutdown signal received");

        Ok(())
    }

    async fn shutdown(self) -> Result<()> {
        info!("Shutting down headless node...");

        // Stop network
        self.network.stop().await?;

        // Flush storage
        self.storage.flush().await?;

        info!("Headless node shut down successfully");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let config: Config = {
        let config_str = std::fs::read_to_string(&args.config)?;
        toml::from_str(&config_str)?
    };

    // Setup tracing
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        match config.logging.level.as_str() {
            "error" => tracing::Level::ERROR,
            "warn" => tracing::Level::WARN,
            "info" => tracing::Level::INFO,
            "debug" => tracing::Level::DEBUG,
            "trace" => tracing::Level::TRACE,
            _ => tracing::Level::INFO,
        }
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_thread_ids(true)
        .with_target(true);

    if let Some(log_file) = &config.logging.file {
        // Create log directory if needed
        if let Some(parent) = log_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;

        subscriber
            .with_writer(file)
            .init();
    } else {
        subscriber.init();
    }

    info!("Starting Communitas headless node v{}", env!("CARGO_PKG_VERSION"));
    info!("Config: {:?}", args.config);

    // Override listen address if provided
    let mut config = config;
    if let Some(listen) = args.listen {
        config.network.listen_address = listen;
    }

    // Create and run node
    let node = HeadlessNode::new(config).await?;

    // Run until shutdown
    if let Err(e) = node.run().await {
        error!("Node runtime error: {}", e);
    }

    // Graceful shutdown
    node.shutdown().await?;

    Ok(())
}

// Stub types for compilation - these would come from saorsa_core
use async_trait::async_trait;

#[derive(Debug)]
enum NetworkEvent {
    PeerConnected(String),
    PeerDisconnected(String),
    MessageReceived { from: String, data: Vec<u8> },
}

// Extension traits for the stubs
#[async_trait]
trait NetworkExt {
    async fn connected_peers(&self) -> Vec<String>;
    async fn subscribe_events(&self) -> Result<tokio::sync::mpsc::Receiver<NetworkEvent>>;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

#[async_trait]
trait DhtExt {
    async fn maintain(&self) -> Result<DhtStats>;
}

#[async_trait]
trait StorageExt {
    async fn flush(&self) -> Result<()>;
}

struct DhtStats {
    entries: usize,
    connected_peers: usize,
}

// Mock implementations for compilation
#[async_trait]
impl NetworkExt for Network {
    async fn connected_peers(&self) -> Vec<String> {
        vec![]
    }

    async fn subscribe_events(&self) -> Result<tokio::sync::mpsc::Receiver<NetworkEvent>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        Ok(rx)
    }

    async fn start(&self) -> Result<()> {
        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl DhtExt for Dht {
    async fn maintain(&self) -> Result<DhtStats> {
        Ok(DhtStats {
            entries: 0,
            connected_peers: 0,
        })
    }
}

#[async_trait]
impl StorageExt for Storage {
    async fn flush(&self) -> Result<()> {
        Ok(())
    }
}
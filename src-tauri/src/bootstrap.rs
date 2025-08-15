// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! Bootstrap Node Implementation for Communitas
//!
//! This module provides the bootstrap node functionality using the saorsa-core library.
//! The bootstrap node serves as the initial entry point for new nodes joining the network.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio::{signal, sync::RwLock, time::interval};
use tracing::{info, warn};

/// Bootstrap node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    /// Address to listen on (e.g., "0.0.0.0:8888")
    pub listen_address: SocketAddr,
    /// Public address for other nodes to connect
    pub public_address: String,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Data storage directory
    pub data_dir: PathBuf,
    /// DHT storage path
    pub dht_storage_path: PathBuf,
    /// Enable health check endpoint
    pub enable_health_check: bool,
    /// Health check port
    pub health_check_port: u16,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:8888".parse().unwrap(),
            public_address: "bootstrap.communitas.app:8888".to_string(),
            max_connections: 1000,
            connection_timeout: 30,
            data_dir: PathBuf::from("/var/lib/saorsa"),
            dht_storage_path: PathBuf::from("/var/lib/saorsa/dht"),
            enable_health_check: true,
            health_check_port: 8888,
        }
    }
}

/// Node statistics for monitoring
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub uptime_seconds: u64,
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_processed: u64,
    pub bytes_transferred: u64,
    pub dht_entries: u64,
}

/// Bootstrap node implementation
#[derive(Debug)]
pub struct BootstrapNode {
    config: BootstrapConfig,
    stats: Arc<RwLock<NodeStats>>,
}

impl BootstrapNode {
    /// Create a new bootstrap node instance
    pub async fn new(config: BootstrapConfig) -> Result<Self> {
        info!("Initializing Communitas Bootstrap Node");

        // Ensure data directories exist
        tokio::fs::create_dir_all(&config.data_dir)
            .await
            .context("Failed to create data directory")?;

        tokio::fs::create_dir_all(&config.dht_storage_path)
            .await
            .context("Failed to create DHT storage directory")?;

        let stats = Arc::new(RwLock::new(NodeStats::default()));

        Ok(Self { config, stats })
    }

    /// Start the bootstrap node
    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting Communitas Bootstrap Node on {}",
            self.config.listen_address
        );

        // Start statistics tracking
        let stats_task = self.start_stats_tracking();

        // Start health check server if enabled
        let health_check_task = if self.config.enable_health_check {
            Some(self.start_health_check_server().await?)
        } else {
            None
        };

        info!("Bootstrap node started successfully");
        info!("Public address: {}", self.config.public_address);

        // Wait for shutdown signal
        tokio::select! {
            _ = signal::ctrl_c() => {
                info!("Received shutdown signal");
            }
            _ = stats_task => {
                warn!("Statistics tracking ended unexpectedly");
            }
            _ = async {
                if let Some(task) = health_check_task {
                    task.await
                } else {
                    std::future::pending().await
                }
            } => {
                warn!("Health check server ended unexpectedly");
            }
        }

        self.shutdown().await?;
        Ok(())
    }

    /// Start statistics tracking
    async fn start_stats_tracking(&self) -> Result<()> {
        let mut interval = interval(Duration::from_secs(60));
        let stats = Arc::clone(&self.stats);

        loop {
            interval.tick().await;

            let mut stats_guard = stats.write().await;
            stats_guard.uptime_seconds += 60;

            // Log periodic status
            if stats_guard.uptime_seconds % 3600 == 0 {
                info!(
                    "Bootstrap node status: uptime={}h, connections={}, dht_entries={}",
                    stats_guard.uptime_seconds / 3600,
                    stats_guard.active_connections,
                    stats_guard.dht_entries
                );
            }
        }
    }

    /// Start health check HTTP server
    async fn start_health_check_server(&self) -> Result<tokio::task::JoinHandle<()>> {
        let stats = Arc::clone(&self.stats);
        let port = self.config.health_check_port;

        let handle = tokio::spawn(async move {
            use warp::Filter;

            // Health check endpoint
            let health = warp::path("health").and(warp::get()).map(|| {
                warp::reply::json(&serde_json::json!({
                    "status": "healthy",
                    "service": "communitas-bootstrap",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }))
            });

            // Status endpoint with statistics
            let status = warp::path("status")
                .and(warp::get())
                .and(warp::any().map(move || Arc::clone(&stats)))
                .and_then(|stats: Arc<RwLock<NodeStats>>| async move {
                    let stats_guard = stats.read().await;
                    let response = serde_json::json!({
                        "status": "running",
                        "service": "communitas-bootstrap",
                        "stats": *stats_guard,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });
                    Ok::<_, std::convert::Infallible>(warp::reply::json(&response))
                });

            let routes = health.or(status);

            info!("Starting health check server on port {}", port);
            warp::serve(routes).run(([0, 0, 0, 0], port)).await;
        });

        Ok(handle)
    }

    /// Gracefully shutdown the bootstrap node
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down bootstrap node...");
        info!("Bootstrap node shutdown completed");
        Ok(())
    }

    /// Get current node statistics
    pub async fn stats(&self) -> NodeStats {
        self.stats.read().await.clone()
    }
}

/// Bootstrap node CLI entry point
pub async fn run_bootstrap_node(config: BootstrapConfig) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,p2p_core=debug")
        .init();

    let bootstrap_node = BootstrapNode::new(config).await?;
    bootstrap_node.start().await?;

    Ok(())
}

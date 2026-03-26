//! Communitas Headless Node
//!
//! A thin wrapper around the x0xd daemon. Ensures x0xd is running,
//! then monitors its health periodically.

use anyhow::{Context, Result};
use clap::Parser;
use communitas_x0x_client::{DaemonManager, X0xClient};

/// Communitas headless node — ensures x0xd is running and monitors health.
#[derive(Parser, Debug)]
#[command(name = "communitas-headless", about = "Headless node wrapping x0xd")]
struct Cli {
    /// Health check interval in seconds.
    #[arg(long, default_value_t = 60)]
    interval: u64,

    /// Custom x0xd API base URL (default: http://127.0.0.1:12700).
    #[arg(long)]
    api_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialise tracing (respects RUST_LOG env var).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let client = match &cli.api_url {
        Some(url) => X0xClient::with_base_url(url),
        None => X0xClient::new(),
    };

    let dm = DaemonManager::with_client(client.clone());

    tracing::info!("ensuring x0xd daemon is running");
    dm.ensure_running()
        .await
        .context("failed to ensure x0xd is running")?;

    // Log identity once at startup.
    match client.agent().await {
        Ok(identity) => {
            tracing::info!(agent_id = %identity.agent_id, "headless node running");
        }
        Err(e) => {
            tracing::warn!("could not fetch agent identity: {e}");
        }
    }

    // Health monitoring loop.
    let interval = tokio::time::Duration::from_secs(cli.interval);
    loop {
        tokio::time::sleep(interval).await;
        match client.health().await {
            Ok(h) => {
                tracing::info!(
                    status = %h.status,
                    peers = h.peers,
                    uptime_secs = h.uptime_secs,
                    version = %h.version,
                    "health check OK",
                );
            }
            Err(e) => {
                tracing::warn!("health check failed: {e}");
            }
        }
    }
}

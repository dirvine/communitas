//! Communitas Bridge Server
//!
//! HTTP/SSE bridge that exposes Tauri commands as REST endpoints,
//! enabling browser-based testing with real P2P networking via Chrome DevTools MCP.

#![forbid(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod server;
mod handlers;
mod state;
mod error;

use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "communitas_bridge=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Communitas Bridge Server");

    // Initialize P2P networking layer
    let bridge_state = match state::BridgeState::new().await {
        Ok(state) => state,
        Err(e) => {
            error!("Failed to initialize bridge state: {}", e);
            return Err(e);
        }
    };

    // Start HTTP server (runs until shutdown signal)
    let _server_handle = tokio::spawn(async move {
        if let Err(e) = server::start(bridge_state).await {
            error!("Server error: {}", e);
        }
    });

    info!("Bridge server started successfully");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    info!("Shutting down bridge server");

    Ok(())
}

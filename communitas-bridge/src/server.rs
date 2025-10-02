//! HTTP server setup and routing

use crate::{handlers, state::BridgeState};
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// Start the HTTP server
pub async fn start(state: Arc<BridgeState>) -> Result<()> {
    let app = create_router(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3030".to_string())
        .parse::<u16>()
        .unwrap_or(3030);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Bridge server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the Axum router with all endpoints
fn create_router(state: Arc<BridgeState>) -> Router {
    // CORS configuration for browser access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check
        .route("/health", get(handlers::health))
        // Core initialization
        .route("/api/core/initialize", post(handlers::core_initialize))
        .route("/api/core/status", get(handlers::core_status))
        // Channel operations
        .route("/api/channels", post(handlers::create_channel))
        .route("/api/channels", get(handlers::list_channels))
        .route("/api/channels/:id/messages", get(handlers::get_channel_messages))
        .route("/api/channels/:id/messages", post(handlers::send_channel_message))
        // Member operations
        .route("/api/:entity_type/:id/members", get(handlers::get_members))
        .route("/api/:entity_type/:id/members", post(handlers::add_member))
        // Thread operations
        .route("/api/threads/create", post(handlers::create_thread))
        .route("/api/threads/:id/messages", get(handlers::get_thread_messages))
        // P2P network connections
        .route("/api/network/connection-info", get(handlers::get_network_connection_info))
        .route("/api/network/peers", get(handlers::get_connected_peers))
        .route("/api/network/connect", post(handlers::connect_to_peer))
        .route("/api/network/disconnect", post(handlers::disconnect_from_peer))
        // State
        .with_state(state)
        .layer(cors)
}

use super::{handlers::AppState, routes::build_router};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

/// HTTP control server
pub struct ControlServer {
    port: u16,
    state: AppState,
}

impl ControlServer {
    /// Create new control server
    pub fn new(port: u16, state: AppState) -> Self {
        Self { port, state }
    }

    /// Start the HTTP server
    pub async fn run(self) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        info!("Starting HTTP control API on {}", addr);

        // Build router with CORS
        let app = build_router(self.state).layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

        // Create TCP listener
        let listener = TcpListener::bind(&addr).await?;

        info!("HTTP control API listening on http://{}", addr);

        // Start serving
        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }
}

use super::handlers::*;
use axum::{
    Router,
    routing::{get, post},
};

/// Build the API router with all endpoints
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health))
        // Authentication endpoints
        .route("/api/auth/vault", post(create_vault))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        // Identity endpoints
        .route("/api/identity/current", get(get_identity))
        // Network endpoints
        .route("/api/network/status", get(get_network_status))
        // Entity endpoints
        .route("/api/entities", post(create_entity))
        .route("/api/entities", get(list_entities))
        // Message endpoints
        .route("/api/messages/send", post(send_message))
        .route(
            "/api/entities/:entity_id/messages",
            get(get_entity_messages),
        )
        .with_state(state)
}

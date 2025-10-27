//! Error types for the bridge server

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("P2P networking error: {0}")]
    #[allow(dead_code)]
    Network(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal server error: {0}")]
    #[allow(dead_code)]
    Internal(String),
}

impl IntoResponse for BridgeError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            BridgeError::Network(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            BridgeError::CommandFailed(msg) => (StatusCode::BAD_REQUEST, msg),
            BridgeError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            BridgeError::Serialization(err) => (
                StatusCode::BAD_REQUEST,
                format!("Serialization error: {}", err),
            ),
            BridgeError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

pub type BridgeResult<T> = Result<T, BridgeError>;

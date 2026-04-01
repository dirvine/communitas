// SPDX-License-Identifier: MIT OR Apache-2.0

/// Errors returned by the x0x client.
#[derive(Debug, thiserror::Error)]
pub enum X0xError {
    /// HTTP request failed (network error, timeout, etc.).
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// The daemon returned an error response (`{"ok": false, "error": "..."}`).
    #[error("daemon error: {0}")]
    Daemon(String),

    /// JSON serialization or deserialization failed.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// WebSocket error.
    #[error("websocket error: {0}")]
    WebSocket(#[from] Box<tokio_tungstenite::tungstenite::Error>),

    /// The daemon is not reachable.
    #[error("daemon not reachable at {0}")]
    NotReachable(String),

    /// An I/O error (e.g. running install/start commands).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Base64 decoding failed.
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// x0x daemon configuration discovery failed (missing api.port / api-token files).
    #[error("x0x config error: {0}")]
    Config(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, X0xError>;

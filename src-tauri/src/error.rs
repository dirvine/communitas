// Central application error type for backend commands and services

use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Hex decode error: {0}")]
    Hex(#[from] hex::FromHexError),

    #[error("Mutex poisoned")]
    MutexPoisoned,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Other: {0}")]
    Other(String),
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Other(e.to_string())
    }
}

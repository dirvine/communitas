// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Error types for the Communitas Iced application.

use thiserror::Error;

/// Application-level errors.
#[derive(Debug, Error, Clone)]
pub enum AppError {
    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Network/connection error.
    #[error("Network error: {0}")]
    Network(String),

    /// Entity not found.
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Storage error.
    #[error("Storage error: {0}")]
    Storage(String),

    /// WebRTC call error.
    #[error("Call error: {0}")]
    Call(String),

    /// Keyring/keychain error.
    #[error("Keyring error: {0}")]
    Keyring(String),

    /// CRDT sync error.
    #[error("Sync error: {0}")]
    Sync(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        let msg = err.to_string();
        if msg.contains("auth") || msg.contains("password") || msg.contains("login") {
            Self::Auth(msg)
        } else if msg.contains("network") || msg.contains("connection") {
            Self::Network(msg)
        } else if msg.contains("not found") {
            Self::EntityNotFound(msg)
        } else if msg.contains("permission") || msg.contains("denied") {
            Self::PermissionDenied(msg)
        } else {
            Self::Internal(msg)
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Storage(err.to_string())
    }
}

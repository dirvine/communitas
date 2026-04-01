// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Recovery module error types for ADR-016 vault recovery.

use thiserror::Error;

/// Errors that can occur during vault recovery operations.
#[derive(Debug, Error)]
pub enum RecoveryError {
    /// Failed to generate cryptographic entropy.
    #[error("entropy generation failed: {0}")]
    EntropyGenerationFailed(String),

    /// Failed to generate mnemonic from entropy.
    #[error("mnemonic generation failed: {0}")]
    MnemonicGenerationFailed(String),

    /// The provided mnemonic phrase is invalid.
    #[error("invalid mnemonic: {0}")]
    InvalidMnemonic(String),

    /// Mnemonic checksum verification failed.
    #[error("mnemonic checksum verification failed")]
    ChecksumFailed,

    /// Key derivation from mnemonic failed.
    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// A vault with this four-word identity already exists.
    #[error("vault already exists for identity: {four_words}")]
    VaultAlreadyExists {
        /// The four-word identity that already has a vault.
        four_words: String,
    },

    /// Storage operation failed during recovery.
    #[error("storage error: {0}")]
    StorageError(String),
}

/// Result type alias for recovery operations.
pub type RecoveryResult<T> = std::result::Result<T, RecoveryError>;

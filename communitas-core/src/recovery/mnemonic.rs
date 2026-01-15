// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Mnemonic phrase generation and validation for vault recovery (ADR-016).
//!
//! This module provides BIP-39 compatible mnemonic phrases for vault recovery.
//! The 24-word phrases provide 256-bit security and can be used to derive
//! all cryptographic keys for a vault.

use bip39::Language;

/// Configuration for recovery phrase generation.
#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    /// Number of mnemonic words (24 for 256-bit security).
    pub word_count: usize,
    /// BIP39 language for the mnemonic wordlist.
    pub language: Language,
    /// Whether to use an additional passphrase (25th word).
    pub use_passphrase: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            word_count: 24,
            language: Language::English,
            use_passphrase: false,
        }
    }
}

impl RecoveryConfig {
    /// Create a new recovery configuration with default 24-word English mnemonic.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of mnemonic words.
    ///
    /// Valid values are 12, 15, 18, 21, or 24 words.
    /// 24 words is recommended for maximum security (256-bit).
    #[must_use]
    pub fn with_word_count(mut self, count: usize) -> Self {
        self.word_count = count;
        self
    }

    /// Set the BIP39 language for the wordlist.
    #[must_use]
    pub fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// Enable or disable additional passphrase (25th word).
    #[must_use]
    pub fn with_passphrase(mut self, use_passphrase: bool) -> Self {
        self.use_passphrase = use_passphrase;
        self
    }
}

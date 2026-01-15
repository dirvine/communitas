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

use bip39::{Language, Mnemonic};
use rand::RngCore;
use zeroize::Zeroize;

use super::error::RecoveryError;

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

/// Calculate entropy bytes needed for given word count.
/// 12 words = 128 bits = 16 bytes
/// 15 words = 160 bits = 20 bytes  
/// 18 words = 192 bits = 24 bytes
/// 21 words = 224 bits = 28 bytes
/// 24 words = 256 bits = 32 bytes
fn entropy_bytes_for_word_count(word_count: usize) -> Result<usize, RecoveryError> {
    match word_count {
        12 => Ok(16),
        15 => Ok(20),
        18 => Ok(24),
        21 => Ok(28),
        24 => Ok(32),
        _ => Err(RecoveryError::MnemonicGenerationFailed(format!(
            "invalid word count {word_count}: must be 12, 15, 18, 21, or 24"
        ))),
    }
}

/// Generate a new BIP39 mnemonic for identity recovery.
///
/// Returns a mnemonic with the word count specified in the configuration.
/// Default is 24 words with 256-bit entropy.
///
/// # Errors
///
/// Returns `RecoveryError::MnemonicGenerationFailed` if entropy generation fails.
pub fn generate_recovery_mnemonic(config: &RecoveryConfig) -> Result<Mnemonic, RecoveryError> {
    let entropy_len = entropy_bytes_for_word_count(config.word_count)?;
    let mut entropy = vec![0u8; entropy_len];
    rand::thread_rng().fill_bytes(&mut entropy);

    let result = Mnemonic::from_entropy_in(config.language, &entropy)
        .map_err(|e| RecoveryError::MnemonicGenerationFailed(e.to_string()));

    // Zeroize entropy immediately after mnemonic creation
    entropy.zeroize();

    result
}

/// Validate a mnemonic string and verify its checksum.
///
/// # Errors
///
/// Returns `RecoveryError::InvalidMnemonic` if the mnemonic is malformed
/// or has an invalid checksum.
pub fn validate_mnemonic(
    mnemonic_words: &str,
    language: Language,
) -> Result<Mnemonic, RecoveryError> {
    Mnemonic::parse_in(language, mnemonic_words)
        .map_err(|e| RecoveryError::InvalidMnemonic(e.to_string()))
}

/// Convert mnemonic to word list for display.
#[must_use]
pub fn mnemonic_to_words(mnemonic: &Mnemonic) -> Vec<String> {
    mnemonic.words().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_generation() {
        let config = RecoveryConfig::default();
        let mnemonic = generate_recovery_mnemonic(&config).unwrap();
        let words = mnemonic_to_words(&mnemonic);
        assert_eq!(words.len(), 24, "Expected 24 words for default config");
    }

    #[test]
    fn test_mnemonic_generation_12_words() {
        let config = RecoveryConfig::new().with_word_count(12);
        let mnemonic = generate_recovery_mnemonic(&config).unwrap();
        let words = mnemonic_to_words(&mnemonic);
        assert_eq!(words.len(), 12, "Expected 12 words");
    }

    #[test]
    fn test_mnemonic_validation() {
        // Generate a valid mnemonic first
        let config = RecoveryConfig::default();
        let mnemonic = generate_recovery_mnemonic(&config).unwrap();
        let mnemonic_str = mnemonic.to_string();

        // Validate it
        let validated = validate_mnemonic(&mnemonic_str, Language::English).unwrap();
        assert_eq!(validated.to_string(), mnemonic_str);
    }

    #[test]
    fn test_invalid_mnemonic() {
        // Using a word that doesn't exist in BIP39 wordlist
        let invalid_word = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";
        let result = validate_mnemonic(invalid_word, Language::English);
        assert!(
            result.is_err(),
            "Expected error for mnemonic with invalid word"
        );
    }

    #[test]
    fn test_mnemonic_to_words() {
        let config = RecoveryConfig::default();
        let mnemonic = generate_recovery_mnemonic(&config).unwrap();
        let words = mnemonic_to_words(&mnemonic);

        // Verify all words are non-empty
        for word in &words {
            assert!(!word.is_empty(), "Word should not be empty");
        }
    }
}

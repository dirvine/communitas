// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Deterministic key derivation from BIP39 mnemonic (ADR-016).
//!
//! This module provides deterministic generation of identity keys from a BIP39
//! mnemonic. With x0x integration, PQC key management is handled by the x0x
//! daemon. This module retains BIP39 mnemonic generation and basic seed
//! derivation for identity recovery workflows.

use bip39::Mnemonic;
use zeroize::Zeroize;

use super::error::RecoveryResult;

/// Domain separation constants for key derivation
mod derivation {
    /// Master key derivation context
    pub const MASTER_KEY: &str = "communitas:identity:master:v1";
    /// Signing key derivation context
    pub const SIGNING: &str = "communitas:signing:v1";
    /// Encryption key derivation context
    pub const ENCRYPTION: &str = "communitas:encryption:v1";
}

/// Identity keys derived from a BIP39 mnemonic.
///
/// With x0x integration, PQC key generation is handled by the daemon.
/// This struct stores the deterministic seed material that can be used
/// to reproduce keys via x0x's key import API.
#[derive(Clone)]
pub struct IdentityKeys {
    /// Four-word identity derived from the seed (legacy, kept for compatibility)
    pub four_words: String,

    /// Signing seed bytes (32 bytes, can be used to reproduce signing keys)
    signing_key_bytes: Vec<u8>,

    /// Verifying key bytes (derived from signing seed)
    verifying_key_bytes: Vec<u8>,

    /// Decapsulation seed bytes (32 bytes)
    decapsulation_key_bytes: Vec<u8>,

    /// Encapsulation key bytes (derived from decapsulation seed)
    encapsulation_key_bytes: Vec<u8>,
}

impl IdentityKeys {
    /// Get the signing key seed bytes.
    #[must_use]
    pub fn signing_key_bytes(&self) -> &[u8] {
        &self.signing_key_bytes
    }

    /// Get the verifying key bytes.
    #[must_use]
    pub fn verifying_key_bytes(&self) -> &[u8] {
        &self.verifying_key_bytes
    }

    /// Get the decapsulation key seed bytes.
    #[must_use]
    pub fn decapsulation_key_bytes(&self) -> &[u8] {
        &self.decapsulation_key_bytes
    }

    /// Get the encapsulation key bytes.
    #[must_use]
    pub fn encapsulation_key_bytes(&self) -> &[u8] {
        &self.encapsulation_key_bytes
    }
}

impl Drop for IdentityKeys {
    fn drop(&mut self) {
        // Securely zeroize private key material
        self.signing_key_bytes.zeroize();
        self.decapsulation_key_bytes.zeroize();
    }
}

impl std::fmt::Debug for IdentityKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityKeys")
            .field("four_words", &self.four_words)
            .field("signing_key_bytes", &"[REDACTED]")
            .field(
                "verifying_key_bytes",
                &format!("[{} bytes]", self.verifying_key_bytes.len()),
            )
            .field("decapsulation_key_bytes", &"[REDACTED]")
            .field(
                "encapsulation_key_bytes",
                &format!("[{} bytes]", self.encapsulation_key_bytes.len()),
            )
            .finish()
    }
}

/// Derive all identity keys from a BIP39 mnemonic.
///
/// This function deterministically derives key seeds from a BIP39 mnemonic
/// phrase. The same mnemonic and passphrase always produce identical seeds.
///
/// With x0x integration, these seeds can be imported into the x0x daemon
/// to reproduce the same PQC keys.
///
/// # Key Derivation Chain
///
/// ```text
/// Mnemonic -> PBKDF2-HMAC-SHA512 (BIP39) -> 64-byte seed
///     |
///     +---> BLAKE3-KDF("communitas:identity:master:v1") -> 32-byte master key
///          |
///          +---> BLAKE3-KDF("communitas:signing:v1") -> 32-byte signing seed
///          |
///          +---> BLAKE3-KDF("communitas:encryption:v1") -> 32-byte encryption seed
/// ```
pub fn derive_identity_keys(
    mnemonic: &Mnemonic,
    passphrase: Option<&str>,
) -> RecoveryResult<IdentityKeys> {
    // BIP39 seed derivation: PBKDF2-HMAC-SHA512, 2048 iterations
    let mut seed = mnemonic.to_seed(passphrase.unwrap_or(""));

    // Derive master key using BLAKE3 with domain separation
    let mut master_key = blake3::derive_key(derivation::MASTER_KEY, &seed);

    // Zeroize BIP39 seed immediately after deriving master key
    seed.zeroize();

    // Derive signing seed
    let signing_seed = blake3::derive_key(derivation::SIGNING, &master_key);

    // Derive encryption seed
    let encryption_seed = blake3::derive_key(derivation::ENCRYPTION, &master_key);

    // Zeroize master key after deriving all child keys
    master_key.zeroize();

    // Use signing seed to derive a deterministic "verifying key" (public component)
    let verifying_key_bytes = blake3::hash(&signing_seed).as_bytes().to_vec();

    // Use encryption seed to derive a deterministic "encapsulation key" (public component)
    let encapsulation_key_bytes = blake3::hash(&encryption_seed).as_bytes().to_vec();

    // Derive four-word identity from verifying key
    let four_words = derive_four_words_from_seed(&verifying_key_bytes);

    Ok(IdentityKeys {
        four_words,
        signing_key_bytes: signing_seed.to_vec(),
        verifying_key_bytes,
        decapsulation_key_bytes: encryption_seed.to_vec(),
        encapsulation_key_bytes,
    })
}

/// Derive a four-word identity from key bytes.
///
/// Uses BLAKE3 hash of the key bytes to produce 4 deterministic words.
fn derive_four_words_from_seed(key_bytes: &[u8]) -> String {
    let hash = blake3::hash(key_bytes);
    let hash_bytes = hash.as_bytes();

    // Use a simple but deterministic word generation from hash bytes
    let words: Vec<String> = (0..4)
        .map(|i| {
            let start = i * 4;
            let val = u32::from_le_bytes([
                hash_bytes[start],
                hash_bytes[start + 1],
                hash_bytes[start + 2],
                hash_bytes[start + 3],
            ]);
            // Generate a 4-8 character lowercase word deterministically
            let mut word = String::new();
            let mut v = val;
            let len = 4 + (v % 5) as usize;
            for _ in 0..len {
                word.push((b'a' + (v % 26) as u8) as char);
                v /= 26;
            }
            word
        })
        .collect();

    words.join("-")
}

/// Create a new identity with a fresh BIP39 mnemonic.
///
/// **IMPORTANT**: The mnemonic must be shown to the user exactly once for backup.
/// It should NEVER be stored by the application.
pub fn create_new_identity(
    config: &super::mnemonic::RecoveryConfig,
    passphrase: Option<&str>,
) -> RecoveryResult<(bip39::Mnemonic, IdentityKeys)> {
    let mnemonic = super::mnemonic::generate_recovery_mnemonic(config)?;
    let keys = derive_identity_keys(&mnemonic, passphrase)?;
    Ok((mnemonic, keys))
}

/// Recover an identity from an existing BIP39 mnemonic phrase.
///
/// This function validates the mnemonic and derives the same identity keys
/// that were originally created.
pub fn recover_identity(
    mnemonic_words: &str,
    language: bip39::Language,
    passphrase: Option<&str>,
) -> RecoveryResult<IdentityKeys> {
    let mnemonic = super::mnemonic::validate_mnemonic(mnemonic_words, language)?;
    derive_identity_keys(&mnemonic, passphrase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bip39::Language;

    /// Test mnemonic for deterministic tests (DO NOT USE IN PRODUCTION)
    const TEST_MNEMONIC: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn get_test_mnemonic() -> Mnemonic {
        Mnemonic::parse_in(Language::English, TEST_MNEMONIC).unwrap()
    }

    #[test]
    fn test_derive_identity_keys_deterministic() {
        let mnemonic = get_test_mnemonic();

        let keys1 = derive_identity_keys(&mnemonic, None).unwrap();
        let keys2 = derive_identity_keys(&mnemonic, None).unwrap();

        assert_eq!(keys1.four_words, keys2.four_words);
        assert_eq!(keys1.signing_key_bytes, keys2.signing_key_bytes);
        assert_eq!(keys1.verifying_key_bytes, keys2.verifying_key_bytes);
        assert_eq!(keys1.decapsulation_key_bytes, keys2.decapsulation_key_bytes);
        assert_eq!(keys1.encapsulation_key_bytes, keys2.encapsulation_key_bytes);
    }

    #[test]
    fn test_derive_identity_keys_different_passphrase() {
        let mnemonic = get_test_mnemonic();

        let keys_no_pass = derive_identity_keys(&mnemonic, None).unwrap();
        let keys_with_pass = derive_identity_keys(&mnemonic, Some("secret")).unwrap();

        assert_ne!(keys_no_pass.four_words, keys_with_pass.four_words);
        assert_ne!(
            keys_no_pass.signing_key_bytes,
            keys_with_pass.signing_key_bytes
        );
    }

    #[test]
    fn test_identity_keys_key_sizes() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Seeds are 32 bytes
        assert_eq!(keys.signing_key_bytes.len(), 32);
        assert_eq!(keys.verifying_key_bytes.len(), 32);
        assert_eq!(keys.decapsulation_key_bytes.len(), 32);
        assert_eq!(keys.encapsulation_key_bytes.len(), 32);
    }

    #[test]
    fn test_four_words_format() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        let parts: Vec<&str> = keys.four_words.split('-').collect();
        assert_eq!(parts.len(), 4, "Should have exactly 4 words");

        for word in parts {
            assert!(!word.is_empty(), "Each word should be non-empty");
            assert!(
                word.chars().all(|c| c.is_ascii_lowercase()),
                "Words should be lowercase ASCII"
            );
        }
    }

    #[test]
    fn test_different_mnemonics_produce_different_keys() {
        let mnemonic1 = get_test_mnemonic();
        let mnemonic2 = Mnemonic::parse_in(
            Language::English,
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
        )
        .unwrap();

        let keys1 = derive_identity_keys(&mnemonic1, None).unwrap();
        let keys2 = derive_identity_keys(&mnemonic2, None).unwrap();

        assert_ne!(keys1.four_words, keys2.four_words);
        assert_ne!(keys1.signing_key_bytes, keys2.signing_key_bytes);
    }

    #[test]
    fn test_passphrase_deterministic() {
        let mnemonic = get_test_mnemonic();

        let keys1 = derive_identity_keys(&mnemonic, Some("my-passphrase")).unwrap();
        let keys2 = derive_identity_keys(&mnemonic, Some("my-passphrase")).unwrap();

        assert_eq!(keys1.four_words, keys2.four_words);
        assert_eq!(keys1.signing_key_bytes, keys2.signing_key_bytes);
    }

    #[test]
    fn test_debug_redacts_private_keys() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        let debug_output = format!("{:?}", keys);

        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains(&hex::encode(&keys.signing_key_bytes)));
        assert!(!debug_output.contains(&hex::encode(&keys.decapsulation_key_bytes)));
    }

    #[test]
    fn test_24_word_mnemonic() {
        let mnemonic = Mnemonic::parse_in(
            Language::English,
            "abandon abandon abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon abandon art",
        )
        .unwrap();

        let keys = derive_identity_keys(&mnemonic, None).unwrap();
        assert_eq!(keys.signing_key_bytes.len(), 32);
    }

    #[test]
    fn test_create_new_identity() {
        use super::super::mnemonic::RecoveryConfig;

        let config = RecoveryConfig::default();
        let (mnemonic, keys) = create_new_identity(&config, None).unwrap();

        assert_eq!(mnemonic.word_count(), 24);
        assert_eq!(keys.signing_key_bytes.len(), 32);

        let parts: Vec<&str> = keys.four_words.split('-').collect();
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn test_create_new_identity_with_passphrase() {
        use super::super::mnemonic::RecoveryConfig;

        let config = RecoveryConfig::default();
        let (mnemonic, keys_with_pass) = create_new_identity(&config, Some("secret")).unwrap();

        let keys_no_pass = derive_identity_keys(&mnemonic, None).unwrap();

        assert_ne!(keys_with_pass.four_words, keys_no_pass.four_words);
        assert_ne!(
            keys_with_pass.signing_key_bytes,
            keys_no_pass.signing_key_bytes
        );
    }

    #[test]
    fn test_recover_identity_valid() {
        let keys = recover_identity(TEST_MNEMONIC, Language::English, None).unwrap();
        assert_eq!(keys.signing_key_bytes.len(), 32);

        let parts: Vec<&str> = keys.four_words.split('-').collect();
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn test_recover_identity_matches_derive() {
        let mnemonic = get_test_mnemonic();

        let keys_derived = derive_identity_keys(&mnemonic, None).unwrap();
        let keys_recovered = recover_identity(TEST_MNEMONIC, Language::English, None).unwrap();

        assert_eq!(keys_derived.four_words, keys_recovered.four_words);
        assert_eq!(
            keys_derived.signing_key_bytes,
            keys_recovered.signing_key_bytes
        );
        assert_eq!(
            keys_derived.verifying_key_bytes,
            keys_recovered.verifying_key_bytes
        );
    }

    #[test]
    fn test_recover_identity_with_passphrase() {
        let keys_with_pass =
            recover_identity(TEST_MNEMONIC, Language::English, Some("secret")).unwrap();
        let keys_no_pass = recover_identity(TEST_MNEMONIC, Language::English, None).unwrap();

        assert_ne!(keys_with_pass.four_words, keys_no_pass.four_words);
    }

    #[test]
    fn test_recover_identity_invalid_mnemonic() {
        let result = recover_identity(
            "invalid words that are not in bip39 dictionary at all",
            Language::English,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_recover_identity_bad_checksum() {
        let result = recover_identity(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon",
            Language::English,
            None,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_and_recover_roundtrip() {
        use super::super::mnemonic::RecoveryConfig;

        let config = RecoveryConfig::default();
        let (mnemonic, original_keys) = create_new_identity(&config, Some("passphrase")).unwrap();

        let mnemonic_words: Vec<&str> = mnemonic.words().collect();
        let mnemonic_string = mnemonic_words.join(" ");

        let recovered_keys =
            recover_identity(&mnemonic_string, Language::English, Some("passphrase")).unwrap();

        assert_eq!(original_keys.four_words, recovered_keys.four_words);
        assert_eq!(
            original_keys.signing_key_bytes,
            recovered_keys.signing_key_bytes
        );
        assert_eq!(
            original_keys.verifying_key_bytes,
            recovered_keys.verifying_key_bytes
        );
    }
}

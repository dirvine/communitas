// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Deterministic key derivation from BIP39 mnemonic (ADR-016).
//!
//! This module provides deterministic generation of ML-DSA signing keys,
//! ML-KEM encapsulation keys, and four-word identities from a BIP39 mnemonic.
//! The same mnemonic always produces the same cryptographic keys.

use bip39::Mnemonic;
use fips203::ml_kem_768;
use fips203::traits::{KeyGen as KemKeyGen, SerDes as KemSerDes};
use fips204::ml_dsa_65;
use fips204::traits::{KeyGen as DsaKeyGen, SerDes as DsaSerDes};
use four_word_networking::FourWordAdaptiveEncoder;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use zeroize::Zeroize;

use super::error::{RecoveryError, RecoveryResult};

/// Domain separation constants for key derivation
mod derivation {
    /// Master key derivation context
    pub const MASTER_KEY: &str = "communitas:identity:master:v1";
    /// ML-DSA-65 signing key derivation context
    pub const MLDSA65: &str = "communitas:mldsa65:v1";
    /// ML-KEM-768 encryption key derivation context
    pub const MLKEM768: &str = "communitas:mlkem768:v1";
}

/// ML-DSA-65 key sizes
mod mldsa65_sizes {
    /// Public key size in bytes
    pub const PUBLIC_KEY: usize = 1952;
    /// Private key size in bytes
    pub const PRIVATE_KEY: usize = 4032;
}

/// ML-KEM-768 key sizes
mod mlkem768_sizes {
    /// Encapsulation (public) key size in bytes
    pub const ENCAPSULATION_KEY: usize = 1184;
    /// Decapsulation (private) key size in bytes
    pub const DECAPSULATION_KEY: usize = 2400;
}

/// Identity keys derived from a BIP39 mnemonic.
///
/// Contains both ML-DSA-65 signing keys for identity/authentication
/// and ML-KEM-768 keys for key encapsulation/encryption.
#[derive(Clone)]
pub struct IdentityKeys {
    /// Four-word identity derived from the ML-DSA public key (e.g., "ocean-forest-moon-star")
    pub four_words: String,

    /// ML-DSA-65 signing key (private key, 4032 bytes)
    signing_key_bytes: Vec<u8>,

    /// ML-DSA-65 verifying key (public key, 1952 bytes)
    verifying_key_bytes: Vec<u8>,

    /// ML-KEM-768 decapsulation key (private key, 2400 bytes)
    decapsulation_key_bytes: Vec<u8>,

    /// ML-KEM-768 encapsulation key (public key, 1184 bytes)
    encapsulation_key_bytes: Vec<u8>,
}

impl IdentityKeys {
    /// Get the ML-DSA-65 signing key bytes (private key).
    #[must_use]
    pub fn signing_key_bytes(&self) -> &[u8] {
        &self.signing_key_bytes
    }

    /// Get the ML-DSA-65 verifying key bytes (public key).
    #[must_use]
    pub fn verifying_key_bytes(&self) -> &[u8] {
        &self.verifying_key_bytes
    }

    /// Get the ML-KEM-768 decapsulation key bytes (private key).
    #[must_use]
    pub fn decapsulation_key_bytes(&self) -> &[u8] {
        &self.decapsulation_key_bytes
    }

    /// Get the ML-KEM-768 encapsulation key bytes (public key).
    #[must_use]
    pub fn encapsulation_key_bytes(&self) -> &[u8] {
        &self.encapsulation_key_bytes
    }

    /// Parse the ML-DSA-65 signing key from stored bytes.
    ///
    /// # Errors
    /// Returns error if the stored bytes are invalid.
    pub fn signing_key(&self) -> RecoveryResult<ml_dsa_65::PrivateKey> {
        let bytes: [u8; mldsa65_sizes::PRIVATE_KEY] =
            self.signing_key_bytes.as_slice().try_into().map_err(|_| {
                RecoveryError::KeyDerivationFailed(format!(
                    "Invalid ML-DSA-65 private key length: expected {}, got {}",
                    mldsa65_sizes::PRIVATE_KEY,
                    self.signing_key_bytes.len()
                ))
            })?;

        ml_dsa_65::PrivateKey::try_from_bytes(bytes)
            .map_err(|e| RecoveryError::KeyDerivationFailed(format!("Invalid ML-DSA-65 key: {e}")))
    }

    /// Parse the ML-DSA-65 verifying key from stored bytes.
    ///
    /// # Errors
    /// Returns error if the stored bytes are invalid.
    pub fn verifying_key(&self) -> RecoveryResult<ml_dsa_65::PublicKey> {
        let bytes: [u8; mldsa65_sizes::PUBLIC_KEY] = self
            .verifying_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| {
                RecoveryError::KeyDerivationFailed(format!(
                    "Invalid ML-DSA-65 public key length: expected {}, got {}",
                    mldsa65_sizes::PUBLIC_KEY,
                    self.verifying_key_bytes.len()
                ))
            })?;

        ml_dsa_65::PublicKey::try_from_bytes(bytes)
            .map_err(|e| RecoveryError::KeyDerivationFailed(format!("Invalid ML-DSA-65 key: {e}")))
    }

    /// Parse the ML-KEM-768 decapsulation key from stored bytes.
    ///
    /// # Errors
    /// Returns error if the stored bytes are invalid.
    pub fn decapsulation_key(&self) -> RecoveryResult<ml_kem_768::DecapsKey> {
        let bytes: [u8; mlkem768_sizes::DECAPSULATION_KEY] = self
            .decapsulation_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| {
                RecoveryError::KeyDerivationFailed(format!(
                    "Invalid ML-KEM-768 decapsulation key length: expected {}, got {}",
                    mlkem768_sizes::DECAPSULATION_KEY,
                    self.decapsulation_key_bytes.len()
                ))
            })?;

        ml_kem_768::DecapsKey::try_from_bytes(bytes).map_err(|e| {
            RecoveryError::KeyDerivationFailed(format!("Invalid ML-KEM-768 decapsulation key: {e}"))
        })
    }

    /// Parse the ML-KEM-768 encapsulation key from stored bytes.
    ///
    /// # Errors
    /// Returns error if the stored bytes are invalid.
    pub fn encapsulation_key(&self) -> RecoveryResult<ml_kem_768::EncapsKey> {
        let bytes: [u8; mlkem768_sizes::ENCAPSULATION_KEY] = self
            .encapsulation_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| {
                RecoveryError::KeyDerivationFailed(format!(
                    "Invalid ML-KEM-768 encapsulation key length: expected {}, got {}",
                    mlkem768_sizes::ENCAPSULATION_KEY,
                    self.encapsulation_key_bytes.len()
                ))
            })?;

        ml_kem_768::EncapsKey::try_from_bytes(bytes).map_err(|e| {
            RecoveryError::KeyDerivationFailed(format!("Invalid ML-KEM-768 encapsulation key: {e}"))
        })
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
/// This function deterministically derives ML-DSA-65 signing keys and ML-KEM-768
/// encapsulation keys from a BIP39 mnemonic phrase. The same mnemonic and passphrase
/// always produce identical keys.
///
/// # Key Derivation Chain
///
/// ```text
/// Mnemonic → PBKDF2-HMAC-SHA512 (BIP39) → 64-byte seed
///     │
///     └──► BLAKE3-KDF("communitas:identity:master:v1") → 32-byte master key
///          │
///          ├──► BLAKE3-KDF("communitas:mldsa65:v1") → ML-DSA-65 seed
///          │    └──► ChaCha20Rng → ML-DSA-65 keypair
///          │
///          └──► BLAKE3-KDF("communitas:mlkem768:v1") → ML-KEM-768 seed
///               └──► ChaCha20Rng → ML-KEM-768 keypair
/// ```
///
/// # Arguments
///
/// * `mnemonic` - A valid BIP39 mnemonic phrase (12-24 words)
/// * `passphrase` - Optional passphrase (BIP39 "25th word") for additional security
///
/// # Returns
///
/// Returns `IdentityKeys` containing:
/// - ML-DSA-65 signing and verifying keys
/// - ML-KEM-768 encapsulation and decapsulation keys
/// - Four-word identity derived from the public signing key
///
/// # Errors
///
/// Returns `RecoveryError::KeyDerivationFailed` if:
/// - ML-DSA-65 key generation fails
/// - ML-KEM-768 key generation fails
/// - Four-word identity generation fails
///
/// # Example
///
/// ```no_run
/// use bip39::{Language, Mnemonic};
/// use communitas_core::recovery::derive_identity_keys;
///
/// let mnemonic = Mnemonic::parse_in(
///     Language::English,
///     "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
/// ).unwrap();
///
/// let keys = derive_identity_keys(&mnemonic, None).unwrap();
/// println!("Identity: {}", keys.four_words);
/// ```
pub fn derive_identity_keys(
    mnemonic: &Mnemonic,
    passphrase: Option<&str>,
) -> RecoveryResult<IdentityKeys> {
    // BIP39 seed derivation: PBKDF2-HMAC-SHA512, 2048 iterations
    // Salt: "mnemonic" + passphrase
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));

    // Derive master key using BLAKE3 with domain separation
    let master_key = blake3::derive_key(derivation::MASTER_KEY, &seed);

    // Derive ML-DSA-65 signing keypair
    let mldsa_seed = blake3::derive_key(derivation::MLDSA65, &master_key);
    let mut mldsa_rng = ChaCha20Rng::from_seed(mldsa_seed);

    let (verifying_key, signing_key) = ml_dsa_65::KG::try_keygen_with_rng(&mut mldsa_rng)
        .map_err(|e| RecoveryError::KeyDerivationFailed(format!("ML-DSA-65 keygen failed: {e}")))?;

    let signing_key_bytes = signing_key.into_bytes().to_vec();
    let verifying_key_bytes = verifying_key.clone().into_bytes().to_vec();

    // Derive ML-KEM-768 encryption keypair
    let mlkem_seed = blake3::derive_key(derivation::MLKEM768, &master_key);
    let mut mlkem_rng = ChaCha20Rng::from_seed(mlkem_seed);

    let (encapsulation_key, decapsulation_key) =
        ml_kem_768::KG::try_keygen_with_rng(&mut mlkem_rng).map_err(|e| {
            RecoveryError::KeyDerivationFailed(format!("ML-KEM-768 keygen failed: {e}"))
        })?;

    let decapsulation_key_bytes = decapsulation_key.into_bytes().to_vec();
    let encapsulation_key_bytes = encapsulation_key.into_bytes().to_vec();

    // Derive four-word identity from ML-DSA public key
    let four_words = derive_four_words_from_pubkey(&verifying_key_bytes)?;

    Ok(IdentityKeys {
        four_words,
        signing_key_bytes,
        verifying_key_bytes,
        decapsulation_key_bytes,
        encapsulation_key_bytes,
    })
}

/// Derive a four-word identity from an ML-DSA-65 public key.
///
/// The four words are derived by:
/// 1. Hashing the public key with BLAKE3 to get 32 bytes
/// 2. Constructing an IPv4 address from the first 4 bytes
/// 3. Encoding the address using four-word-networking
///
/// # Arguments
///
/// * `pubkey_bytes` - The ML-DSA-65 public key bytes (1952 bytes)
///
/// # Returns
///
/// A four-word identity string in the format "word1-word2-word3-word4"
fn derive_four_words_from_pubkey(pubkey_bytes: &[u8]) -> RecoveryResult<String> {
    // Hash the public key to get a deterministic 32-byte value
    let hash = blake3::hash(pubkey_bytes);
    let hash_bytes = hash.as_bytes();

    // Initialize the four-word encoder
    let encoder = FourWordAdaptiveEncoder::new().map_err(|e| {
        RecoveryError::KeyDerivationFailed(format!("Failed to initialize word encoder: {e}"))
    })?;

    // Construct an IPv4 address from the first 4 bytes of the hash
    // This gives us a deterministic address that encodes to 4 words
    let ip_addr = format!(
        "{}.{}.{}.{}:0",
        hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]
    );

    // Encode the address to words
    let words_str = encoder.encode(&ip_addr).map_err(|e| {
        RecoveryError::KeyDerivationFailed(format!("Failed to encode identity words: {e}"))
    })?;

    // The encoder returns space-separated words, convert to dash-separated
    // and take only the first 4 words (port encoding adds more)
    let words: Vec<&str> = words_str.split_whitespace().take(4).collect();

    if words.len() < 4 {
        return Err(RecoveryError::KeyDerivationFailed(format!(
            "Insufficient words generated: expected 4, got {}",
            words.len()
        )));
    }

    Ok(words.join("-"))
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

        // Derive keys twice
        let keys1 = derive_identity_keys(&mnemonic, None).unwrap();
        let keys2 = derive_identity_keys(&mnemonic, None).unwrap();

        // Same mnemonic should produce same keys
        assert_eq!(keys1.four_words, keys2.four_words);
        assert_eq!(keys1.signing_key_bytes, keys2.signing_key_bytes);
        assert_eq!(keys1.verifying_key_bytes, keys2.verifying_key_bytes);
        assert_eq!(keys1.decapsulation_key_bytes, keys2.decapsulation_key_bytes);
        assert_eq!(keys1.encapsulation_key_bytes, keys2.encapsulation_key_bytes);
    }

    #[test]
    fn test_derive_identity_keys_different_passphrase() {
        let mnemonic = get_test_mnemonic();

        // Derive keys with different passphrases
        let keys_no_pass = derive_identity_keys(&mnemonic, None).unwrap();
        let keys_with_pass = derive_identity_keys(&mnemonic, Some("secret")).unwrap();

        // Different passphrases should produce different keys
        assert_ne!(keys_no_pass.four_words, keys_with_pass.four_words);
        assert_ne!(
            keys_no_pass.signing_key_bytes,
            keys_with_pass.signing_key_bytes
        );
        assert_ne!(
            keys_no_pass.verifying_key_bytes,
            keys_with_pass.verifying_key_bytes
        );
    }

    #[test]
    fn test_identity_keys_key_sizes() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Verify ML-DSA-65 key sizes
        assert_eq!(keys.signing_key_bytes.len(), mldsa65_sizes::PRIVATE_KEY);
        assert_eq!(keys.verifying_key_bytes.len(), mldsa65_sizes::PUBLIC_KEY);

        // Verify ML-KEM-768 key sizes
        assert_eq!(
            keys.decapsulation_key_bytes.len(),
            mlkem768_sizes::DECAPSULATION_KEY
        );
        assert_eq!(
            keys.encapsulation_key_bytes.len(),
            mlkem768_sizes::ENCAPSULATION_KEY
        );
    }

    #[test]
    fn test_four_words_format() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Four words should be in format "word-word-word-word"
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
    fn test_signing_key_roundtrip() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Should be able to parse signing key
        let signing_key = keys.signing_key().unwrap();

        // Verify round-trip
        let bytes = signing_key.into_bytes();
        assert_eq!(bytes.as_slice(), keys.signing_key_bytes.as_slice());
    }

    #[test]
    fn test_verifying_key_roundtrip() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Should be able to parse verifying key
        let verifying_key = keys.verifying_key().unwrap();

        // Verify round-trip
        let bytes = verifying_key.into_bytes();
        assert_eq!(bytes.as_slice(), keys.verifying_key_bytes.as_slice());
    }

    #[test]
    fn test_encapsulation_key_roundtrip() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Should be able to parse encapsulation key
        let encaps_key = keys.encapsulation_key().unwrap();

        // Verify round-trip
        let bytes = encaps_key.into_bytes();
        assert_eq!(bytes.as_slice(), keys.encapsulation_key_bytes.as_slice());
    }

    #[test]
    fn test_decapsulation_key_roundtrip() {
        let mnemonic = get_test_mnemonic();
        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Should be able to parse decapsulation key
        let decaps_key = keys.decapsulation_key().unwrap();

        // Verify round-trip
        let bytes = decaps_key.into_bytes();
        assert_eq!(bytes.as_slice(), keys.decapsulation_key_bytes.as_slice());
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

        // Different mnemonics should produce different keys
        assert_ne!(keys1.four_words, keys2.four_words);
        assert_ne!(keys1.signing_key_bytes, keys2.signing_key_bytes);
    }

    #[test]
    fn test_passphrase_deterministic() {
        let mnemonic = get_test_mnemonic();

        // Same passphrase should produce same keys
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

        // Private key material should be redacted
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains(&hex::encode(&keys.signing_key_bytes)));
        assert!(!debug_output.contains(&hex::encode(&keys.decapsulation_key_bytes)));
    }

    #[test]
    fn test_24_word_mnemonic() {
        // Test with a full 24-word mnemonic
        let mnemonic = Mnemonic::parse_in(
            Language::English,
            "abandon abandon abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon abandon abandon \
             abandon abandon abandon abandon abandon abandon abandon art",
        )
        .unwrap();

        let keys = derive_identity_keys(&mnemonic, None).unwrap();

        // Should work correctly
        assert_eq!(keys.signing_key_bytes.len(), mldsa65_sizes::PRIVATE_KEY);
        assert_eq!(keys.verifying_key_bytes.len(), mldsa65_sizes::PUBLIC_KEY);
    }
}

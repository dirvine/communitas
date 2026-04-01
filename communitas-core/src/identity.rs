// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Identity helpers
//!
//! Legacy four-word identity functions are stubbed out since x0x handles
//! identity natively. Connection word encoding is no longer needed because
//! x0x uses agent IDs (64-char hex strings) for peer addressing.

use std::net::SocketAddr;
use thiserror::Error;

/// Identity encoding errors
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Failed to encode identity: {0}")]
    EncodingFailed(String),

    #[error("Failed to decode identity: {0}")]
    DecodingFailed(String),

    #[error("Invalid four-word format: {0}")]
    InvalidFormat(String),

    #[error("Four-word encoder initialization failed: {0}")]
    EncoderInitFailed(String),
}

pub type IdentityResult<T> = Result<T, IdentityError>;

/// Generate a random four-word string (legacy identity helper)
///
/// **Deprecated**: x0x uses agent IDs for identity. This now generates
/// a random dash-separated string using blake3 hashing for backward
/// compatibility with callers that expect this format.
pub fn generate_id_words() -> IdentityResult<String> {
    let mut buf = [0u8; 16];
    getrandom::getrandom(&mut buf)
        .map_err(|e| IdentityError::EncodingFailed(format!("RNG failure: {e}")))?;
    let hash = blake3::hash(&buf);
    let bytes = hash.as_bytes();
    // Produce 4 short lowercase words from hash bytes
    let words: Vec<String> = (0..4)
        .map(|i| {
            let start = i * 4;
            let val = u32::from_le_bytes([
                bytes[start],
                bytes[start + 1],
                bytes[start + 2],
                bytes[start + 3],
            ]);
            // Use a simple word generation scheme: 4-8 lowercase letters
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
    Ok(words.join("-"))
}

/// Derive a deterministic seed from a four-word string (legacy)
///
/// Takes a four-word identity and produces a deterministic 32-byte seed
/// that can be used to generate cryptographic keys.
pub fn identity_to_seed(identity: &str) -> IdentityResult<[u8; 32]> {
    if !validate_identity_format(identity) {
        return Err(IdentityError::InvalidFormat(format!(
            "Invalid four-word format: expected word-word-word-word, got: {}",
            identity
        )));
    }
    let hash = blake3::hash(identity.as_bytes());
    Ok(*hash.as_bytes())
}

/// Validate a four-word string (legacy identity validation)
///
/// **Deprecated**: With x0x, identity validation uses agent IDs.
/// This stub always returns true for correctly formatted strings.
pub fn validate_id_words(identity: &str) -> bool {
    validate_identity_format(identity)
}

/// Convert a SocketAddr to a connection identity string
///
/// **Deprecated**: x0x uses agent IDs for addressing.
/// This stub returns the address as a formatted string.
pub fn conn_words(addr: &SocketAddr) -> IdentityResult<String> {
    Ok(addr.to_string())
}

/// Parse a connection identity back to a SocketAddr
///
/// **Deprecated**: x0x uses agent IDs for addressing.
/// This stub parses the address string directly.
pub fn conn_from_words(words: &str) -> IdentityResult<SocketAddr> {
    let normalized = words.replace('.', "-");

    if !normalized.contains(' ') && !normalized.contains('-') && !words.contains(':') {
        return Err(IdentityError::InvalidFormat(
            "Connection identity must contain word separators or be a valid address".to_string(),
        ));
    }

    // Try direct parse as SocketAddr
    let addr: SocketAddr = words
        .parse()
        .map_err(|e| IdentityError::DecodingFailed(format!("Failed to parse address: {e}")))?;

    Ok(addr)
}

/// Validate a four-word identity format
///
/// Checks if a string has the correct four-word format (word-word-word-word).
pub fn validate_identity_format(words: &str) -> bool {
    let parts: Vec<&str> = words.split('-').collect();
    parts.len() == 4 && parts.iter().all(|part| !part.is_empty())
}

/// Validate a connection identity format
///
/// Checks if a string has a valid connection identity format (at least 4 words).
pub fn validate_connection_format(words: &str) -> bool {
    let parts: Vec<&str> = words.split('-').collect();
    parts.len() >= 4 && parts.iter().all(|part| !part.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id_words() {
        let identity = generate_id_words().unwrap();
        let parts: Vec<&str> = identity.split('-').collect();
        assert_eq!(parts.len(), 4);
        for part in parts {
            assert!(!part.is_empty());
        }
    }

    #[test]
    fn test_identity_to_seed_deterministic() {
        let seed1 = identity_to_seed("ocean-forest-moon-star").unwrap();
        let seed2 = identity_to_seed("ocean-forest-moon-star").unwrap();
        assert_eq!(seed1, seed2);
        assert_eq!(seed1.len(), 32);
    }

    #[test]
    fn test_identity_to_seed_different_identities() {
        let seed1 = identity_to_seed("ocean-forest-moon-star").unwrap();
        let seed2 = identity_to_seed("river-mountain-sun-cloud").unwrap();
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_validate_id_words() {
        assert!(validate_id_words("ocean-forest-moon-star"));
        assert!(!validate_id_words("only-three-words"));
        assert!(!validate_id_words("too-many-words-here-now"));
        assert!(!validate_id_words(""));
    }

    #[test]
    fn test_conn_words_deterministic() {
        let addr: SocketAddr = "10.0.0.1:5000".parse().unwrap();
        let words1 = conn_words(&addr).unwrap();
        let words2 = conn_words(&addr).unwrap();
        assert_eq!(words1, words2);
    }

    #[test]
    fn test_validate_identity_format() {
        assert!(validate_identity_format("ocean-forest-moon-star"));
        assert!(validate_identity_format("river-mountain-sun-cloud"));
        assert!(!validate_identity_format("only-three-words"));
        assert!(!validate_identity_format("too-many-words-here-now"));
        assert!(!validate_identity_format("no spaces allowed"));
        assert!(!validate_identity_format(""));
    }

    #[test]
    fn test_validate_connection_format() {
        assert!(validate_connection_format("ocean-forest-moon-star"));
        assert!(validate_connection_format("ocean-forest-moon-star-extra"));
        assert!(!validate_connection_format("only-three"));
        assert!(!validate_connection_format("no spaces"));
        assert!(!validate_connection_format(""));
    }
}

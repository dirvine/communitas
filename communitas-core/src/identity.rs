// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Identity and connection encoding using four-word-networking
//!
//! This module provides helpers for:
//! - Connection words (IP:port encoding) for friend-to-friend dialing
//! - Legacy four-word identity helpers kept for backward compatibility

use four_word_networking::FourWordAdaptiveEncoder;
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
/// Generates 4 valid random words from the four-word-networking dictionary.
/// This is retained for backward compatibility and is not used for identity.
///
/// # Returns
/// Four-word identity string with valid dictionary words
///
/// # Example
/// ```
/// use communitas_core::identity::generate_id_words;
///
/// let identity = generate_id_words()?;
/// assert_eq!(identity.split('-').count(), 4);
/// # Ok::<(), communitas_core::identity::IdentityError>(())
/// ```
pub fn generate_id_words() -> IdentityResult<String> {
    // Initialize encoder
    let encoder = FourWordAdaptiveEncoder::new().map_err(|e| {
        IdentityError::EncoderInitFailed(format!("Failed to initialize encoder: {}", e))
    })?;

    // Generate 4 random valid dictionary words
    let words = encoder.get_random_words(4);
    if words.len() != 4 {
        return Err(IdentityError::EncodingFailed(
            "Failed to generate 4 dictionary words".to_string(),
        ));
    }

    Ok(words.join("-"))
}

/// Derive a deterministic seed from a four-word string (legacy)
///
/// Takes a four-word identity and produces a deterministic 32-byte seed
/// that can be used to generate cryptographic keys. Same identity always
/// produces the same seed.
///
/// **Note**: This function only validates the format (4 words separated by dashes),
/// not whether the words are in the four-word-networking dictionary. This allows
/// backward compatibility with identities created using different word lists.
///
/// # Arguments
/// * `identity` - Four-word identity string (e.g., "ocean-forest-moon-star")
///
/// # Returns
/// 32-byte deterministic seed derived from the identity
///
/// # Example
/// ```
/// use communitas_core::identity::identity_to_seed;
///
/// let seed = identity_to_seed("ocean-forest-moon-star")?;
/// assert_eq!(seed.len(), 32);
/// # Ok::<(), communitas_core::identity::IdentityError>(())
/// ```
pub fn identity_to_seed(identity: &str) -> IdentityResult<[u8; 32]> {
    // Validate the format only (not dictionary words)
    // This allows backward compatibility with identities from different word lists
    if !validate_identity_format(identity) {
        return Err(IdentityError::InvalidFormat(format!(
            "Invalid four-word format: expected word-word-word-word, got: {}",
            identity
        )));
    }

    // Hash the identity string to get deterministic 32 bytes
    let hash = blake3::hash(identity.as_bytes());
    Ok(*hash.as_bytes())
}

/// Validate a four-word string (legacy identity validation)
///
/// Checks that all 4 words are valid dictionary words from four-word-networking.
///
/// # Arguments
/// * `identity` - Four-word identity string (e.g., "ocean-forest-moon-star")
///
/// # Returns
/// true if all 4 words are valid, false otherwise
pub fn validate_id_words(identity: &str) -> bool {
    // Check format first
    if !validate_identity_format(identity) {
        return false;
    }

    // Initialize encoder
    let encoder = match FourWordAdaptiveEncoder::new() {
        Ok(e) => e,
        Err(_) => return false,
    };

    // Validate each word is in the dictionary
    for word in identity.split('-') {
        if !encoder.is_valid_word(word) {
            return false;
        }
    }

    true
}

/// Convert a SocketAddr to a four-word connection identity
///
/// Encodes both the IP address and port into a human-readable format.
/// IPv4 addresses produce 4 words, IPv6 addresses produce more words.
///
/// # Arguments
/// * `addr` - Socket address (IP + port)
///
/// # Returns
/// Four-word (or more for IPv6) connection identity string
///
/// # Example
/// ```
/// use communitas_core::identity::conn_words;
/// use std::net::SocketAddr;
///
/// let addr: SocketAddr = "127.0.0.1:8080".parse()?;
/// let conn_id = conn_words(&addr)?;
/// assert!(conn_id.contains(' '));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn conn_words(addr: &SocketAddr) -> IdentityResult<String> {
    // Initialize encoder
    let encoder = FourWordAdaptiveEncoder::new().map_err(|e| {
        IdentityError::EncoderInitFailed(format!("Failed to initialize encoder: {}", e))
    })?;

    // Encode the socket address to words
    let words = encoder.encode(&addr.to_string()).map_err(|e| {
        IdentityError::EncodingFailed(format!("Failed to encode connection address: {}", e))
    })?;

    Ok(words)
}

/// Parse a four-word connection identity back to a SocketAddr
///
/// Decodes a human-readable connection identity string back to its original
/// IP address and port.
///
/// # Arguments
/// * `words` - Four-word (or more) connection identity string
///
/// # Returns
/// Socket address (IP + port)
///
/// # Example
/// ```
/// use communitas_core::identity::{conn_words, conn_from_words};
/// use std::net::SocketAddr;
///
/// let original: SocketAddr = "192.168.1.100:9000".parse()?;
/// let words = conn_words(&original)?;
/// let decoded = conn_from_words(&words)?;
/// assert_eq!(original, decoded);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn conn_from_words(words: &str) -> IdentityResult<SocketAddr> {
    let normalized = words.replace('.', "-");

    // Validate format (should contain separators for FourWordAdaptiveEncoder)
    if !normalized.contains(' ') && !normalized.contains('-') {
        return Err(IdentityError::InvalidFormat(
            "Connection identity must contain word separators".to_string(),
        ));
    }

    // Initialize encoder
    let encoder = FourWordAdaptiveEncoder::new().map_err(|e| {
        IdentityError::EncoderInitFailed(format!("Failed to initialize encoder: {}", e))
    })?;

    // Decode the words back to address string
    let addr_str = encoder.decode(&normalized).map_err(|e| {
        IdentityError::DecodingFailed(format!("Failed to decode connection address: {}", e))
    })?;

    // Parse the address string to SocketAddr
    let addr: SocketAddr = addr_str.parse().map_err(|e| {
        IdentityError::DecodingFailed(format!("Failed to parse decoded address: {}", e))
    })?;

    Ok(addr)
}

/// Validate a four-word identity format
///
/// Checks if a string has the correct four-word format (word-word-word-word).
///
/// # Arguments
/// * `words` - String to validate
///
/// # Returns
/// true if valid format, false otherwise
pub fn validate_identity_format(words: &str) -> bool {
    let parts: Vec<&str> = words.split('-').collect();
    parts.len() == 4 && parts.iter().all(|part| !part.is_empty())
}

/// Validate a connection identity format
///
/// Checks if a string has a valid connection identity format (at least 4 words).
///
/// # Arguments
/// * `words` - String to validate
///
/// # Returns
/// true if valid format, false otherwise
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

        // Should have exactly 4 words separated by dashes
        let parts: Vec<&str> = identity.split('-').collect();
        assert_eq!(parts.len(), 4);

        // Each part should be non-empty
        for part in parts {
            assert!(!part.is_empty());
        }

        // Should be valid according to four-word-networking
        assert!(validate_id_words(&identity));
    }

    #[test]
    fn test_identity_to_seed_deterministic() {
        // Use a generated valid identity
        let identity = generate_id_words().unwrap();

        let seed1 = identity_to_seed(&identity).unwrap();
        let seed2 = identity_to_seed(&identity).unwrap();

        // Same identity should always produce same seed
        assert_eq!(seed1, seed2);
        assert_eq!(seed1.len(), 32);
    }

    #[test]
    fn test_identity_to_seed_different_identities() {
        // Generate two different valid identities
        let identity1 = generate_id_words().unwrap();
        let identity2 = generate_id_words().unwrap();

        // They should be different (extremely high probability)
        if identity1 == identity2 {
            // Skip this test if we got the same identity by chance
            return;
        }

        let seed1 = identity_to_seed(&identity1).unwrap();
        let seed2 = identity_to_seed(&identity2).unwrap();

        // Different identities should produce different seeds
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_validate_id_words() {
        // Valid identity - generated from our function
        let valid_identity = generate_id_words().unwrap();
        assert!(validate_id_words(&valid_identity));

        // Invalid format
        assert!(!validate_id_words("only-three-words"));
        assert!(!validate_id_words("too-many-words-here-now"));
        assert!(!validate_id_words(""));
    }

    #[test]
    fn test_conn_words_ipv4() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn_id = conn_words(&addr).unwrap();

        // Should contain spaces (four-word-networking uses spaces for IPs)
        assert!(conn_id.contains(' '));

        // IPv4 should produce at least 4 words
        let parts: Vec<&str> = conn_id.split(' ').collect();
        assert!(parts.len() >= 4);
    }

    #[test]
    fn test_conn_words_ipv6() {
        let addr: SocketAddr = "[::1]:8080".parse().unwrap();
        let conn_id = conn_words(&addr).unwrap();

        // Should contain spaces (four-word-networking uses spaces for IPs)
        assert!(conn_id.contains(' '));

        // IPv6 should produce multiple words (more than 4)
        let parts: Vec<&str> = conn_id.split(' ').collect();
        assert!(parts.len() >= 4);
    }

    #[test]
    fn test_conn_roundtrip_ipv4() {
        let original: SocketAddr = "192.168.1.100:9000".parse().unwrap();

        let words = conn_words(&original).unwrap();
        let decoded = conn_from_words(&words).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_conn_roundtrip_ipv6() {
        let original: SocketAddr = "[2001:db8::1]:9000".parse().unwrap();

        let words = conn_words(&original).unwrap();
        let decoded = conn_from_words(&words).unwrap();

        assert_eq!(original, decoded);
    }

    #[test]
    fn test_conn_from_words_invalid_format() {
        let result = conn_from_words("invalid");
        assert!(result.is_err());
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
        assert!(validate_connection_format(
            "ocean-forest-moon-star-extra-more"
        ));

        assert!(!validate_connection_format("only-three"));
        assert!(!validate_connection_format("no spaces"));
        assert!(!validate_connection_format(""));
    }

    #[test]
    fn test_conn_words_deterministic() {
        let addr: SocketAddr = "10.0.0.1:5000".parse().unwrap();

        let words1 = conn_words(&addr).unwrap();
        let words2 = conn_words(&addr).unwrap();

        assert_eq!(words1, words2);
    }

    #[test]
    fn test_different_ports_different_words() {
        let addr1: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8081".parse().unwrap();

        let words1 = conn_words(&addr1).unwrap();
        let words2 = conn_words(&addr2).unwrap();

        // Different ports should produce different connection identities
        assert_ne!(words1, words2);
    }
}

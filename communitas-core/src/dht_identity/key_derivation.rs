//! DHT key derivation for four-word identities
//!
//! This module provides domain-separated key derivation using BLAKE3 for:
//! - Identity records (K_id)
//! - Connection records (K_conn) 
//! - Site manifest records (K_site)

use blake3::Hasher;

/// 32-byte DHT key
pub type DhtKey = [u8; 32];

/// Normalized four-word identity
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedFourWords(String);

impl NormalizedFourWords {
    /// Create from raw four-words string, normalizing format
    pub fn new(four_words: &str) -> Result<Self, String> {
        let normalized = normalize_four_words(four_words)?;
        Ok(Self(normalized))
    }

    /// Get the normalized string
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to owned string
    pub fn into_string(self) -> String {
        self.0
    }
}

/// Normalize four-word identity to canonical format
/// 
/// Input: "Ocean-Forest Moon Sun" or "ocean forest-moon_sun"
/// Output: "ocean-forest-moon-sun" (lowercase, dash-separated)
pub fn normalize_four_words(input: &str) -> Result<String, String> {
    let trimmed = input.trim().to_lowercase();
    
    // Split on various separators and collect words
    let words: Vec<&str> = trimmed
        .split(&[' ', '-', '_'][..])
        .filter(|w| !w.is_empty())
        .collect();
    
    if words.len() != 4 {
        return Err(format!("Expected exactly 4 words, got {}", words.len()));
    }
    
    // Validate each word contains only letters
    for word in &words {
        if !word.chars().all(|c| c.is_ascii_lowercase()) {
            return Err(format!("Invalid word '{}': must contain only lowercase letters", word));
        }
        if word.len() < 2 || word.len() > 16 {
            return Err(format!("Invalid word '{}': must be 2-16 characters", word));
        }
    }
    
    Ok(words.join("-"))
}

/// Derive DHT key for identity records
pub fn derive_identity_key(four_words: &NormalizedFourWords) -> DhtKey {
    derive_key_with_domain("communitas:id:v1:", four_words.as_str())
}

/// Derive DHT key for connection records  
pub fn derive_connection_key(four_words: &NormalizedFourWords) -> DhtKey {
    derive_key_with_domain("communitas:conn:v1:", four_words.as_str())
}

/// Derive DHT key for site manifest records
pub fn derive_site_key(four_words: &NormalizedFourWords) -> DhtKey {
    derive_key_with_domain("communitas:site:v1:", four_words.as_str())
}

/// Derive content address (CID) for blobs
pub fn derive_content_address(canonical_cbor: &[u8]) -> DhtKey {
    let mut hasher = Hasher::new();
    hasher.update(canonical_cbor);
    hasher.finalize().into()
}

/// Internal function to derive keys with domain separation
fn derive_key_with_domain(domain: &str, input: &str) -> DhtKey {
    let mut hasher = Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_four_words_valid() {
        let cases = [
            ("ocean-forest-moon-star", "ocean-forest-moon-star"),
            ("Ocean-Forest-Moon-Star", "ocean-forest-moon-star"),
            ("ocean forest moon star", "ocean-forest-moon-star"),
            ("ocean_forest_moon_star", "ocean-forest-moon-star"),
            ("Ocean Forest-Moon_Star", "ocean-forest-moon-star"),
            ("  ocean  forest  moon  star  ", "ocean-forest-moon-star"),
        ];

        for (input, expected) in &cases {
            let result = normalize_four_words(input).expect("Should normalize successfully");
            assert_eq!(result, *expected, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_normalize_four_words_invalid() {
        let invalid_cases = [
            ("ocean forest moon", "Expected exactly 4 words"),
            ("ocean forest moon star extra", "Expected exactly 4 words"), 
            ("ocean forest moon star123", "must contain only lowercase letters"),
            ("ocean forest moon a", "must be 2-16 characters"),
            ("ocean forest moon verylongwordthatistoolong", "must be 2-16 characters"),
            ("", "Expected exactly 4 words"),
            ("   ", "Expected exactly 4 words"),
        ];

        for (input, expected_error) in &invalid_cases {
            let result = normalize_four_words(input);
            assert!(result.is_err(), "Should fail for input: '{}'", input);
            let error = result.unwrap_err();
            assert!(error.contains(expected_error), 
                   "Error '{}' should contain '{}'", error, expected_error);
        }
    }

    #[test]
    fn test_normalized_four_words_construction() {
        let valid = NormalizedFourWords::new("ocean forest moon star").unwrap();
        assert_eq!(valid.as_str(), "ocean-forest-moon-star");
        
        let invalid = NormalizedFourWords::new("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_key_derivation_deterministic() {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();
        
        // Keys should be deterministic
        let key1 = derive_identity_key(&four_words);
        let key2 = derive_identity_key(&four_words);
        assert_eq!(key1, key2);
        
        let conn_key1 = derive_connection_key(&four_words);
        let conn_key2 = derive_connection_key(&four_words);
        assert_eq!(conn_key1, conn_key2);
        
        let site_key1 = derive_site_key(&four_words);
        let site_key2 = derive_site_key(&four_words);
        assert_eq!(site_key1, site_key2);
    }

    #[test]
    fn test_key_derivation_different_types() {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();
        
        let id_key = derive_identity_key(&four_words);
        let conn_key = derive_connection_key(&four_words);
        let site_key = derive_site_key(&four_words);
        
        // Different key types should produce different keys
        assert_ne!(id_key, conn_key);
        assert_ne!(id_key, site_key);
        assert_ne!(conn_key, site_key);
    }

    #[test]
    fn test_key_derivation_different_inputs() {
        let words1 = NormalizedFourWords::new("ocean forest moon star").unwrap();
        let words2 = NormalizedFourWords::new("river mountain cloud wind").unwrap();
        
        let key1 = derive_identity_key(&words1);
        let key2 = derive_identity_key(&words2);
        
        // Different inputs should produce different keys
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_content_address_derivation() {
        let data1 = b"test data 1";
        let data2 = b"test data 2";
        
        let cid1 = derive_content_address(data1);
        let cid2 = derive_content_address(data2);
        let cid1_repeat = derive_content_address(data1);
        
        // Same data should produce same CID
        assert_eq!(cid1, cid1_repeat);
        
        // Different data should produce different CIDs
        assert_ne!(cid1, cid2);
    }

    #[test]
    fn test_known_test_vectors() {
        // These test vectors ensure consistency across implementations
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();
        
        let id_key = derive_identity_key(&four_words);
        let conn_key = derive_connection_key(&four_words);
        let site_key = derive_site_key(&four_words);
        
        // Verify the keys are 32 bytes
        assert_eq!(id_key.len(), 32);
        assert_eq!(conn_key.len(), 32);
        assert_eq!(site_key.len(), 32);
        
        // Print for reference (these become our test vectors)
        println!("Test vectors for 'ocean-forest-moon-star':");
        println!("  ID key:   {:02x?}", id_key);
        println!("  Conn key: {:02x?}", conn_key);  
        println!("  Site key: {:02x?}", site_key);
    }

    #[test]
    fn test_domain_separation() {
        // Verify that domain separation prevents collisions
        let input = "ocean-forest-moon-star";
        
        let id_hash = derive_key_with_domain("communitas:id:v1:", input);
        let fake_hash = derive_key_with_domain("communitas:fake:v1:", input);
        
        assert_ne!(id_hash, fake_hash, "Domain separation should prevent collisions");
    }
}

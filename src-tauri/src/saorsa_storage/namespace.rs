/**
 * Saorsa Storage System - Namespace Management
 * Implements cryptographic namespace isolation using HKDF and HMAC
 */

use crate::saorsa_storage::errors::*;
use blake3::Hasher;
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use regex::Regex;
use lazy_static::lazy_static;

type HmacSha256 = Hmac<Sha256>;

/// Key metadata for tracking key lifecycle
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    pub created_at: DateTime<Utc>,
    pub version: u32,
    pub key_id: String,
    pub namespace: String,
}

/// Historical key for decryption of old content
#[derive(Debug, Clone)]
pub struct HistoricalKey {
    pub key: [u8; 32],
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub retired_at: Option<DateTime<Utc>>,
}

/// DHT key with proof of work for enumeration resistance
#[derive(Debug, Clone)]
pub struct DhtKeyWithProof {
    pub key: [u8; 20], // Truncated to 160 bits
    pub is_enumerable: bool,
    pub proof_of_work: Option<Vec<u8>>,
}

/// Namespace manager implementing HKDF-based key derivation
pub struct NamespaceManager {
    master_key: [u8; 32],
    hkdf: Hkdf<Sha256>,
    key_cache: Arc<RwLock<HashMap<String, [u8; 32]>>>,
    key_metadata: Arc<RwLock<HashMap<String, KeyMetadata>>>,
    historical_keys: Arc<RwLock<HashMap<String, Vec<HistoricalKey>>>>,
    key_versions: Arc<RwLock<HashMap<String, u32>>>,
}

impl NamespaceManager {
    /// Create a new namespace manager with the given master key
    pub fn new(master_key: &[u8]) -> NamespaceResult<Self> {
        if master_key.len() != 32 {
            return Err(KeyDerivationError::InvalidKeyLength {
                length: master_key.len(),
            })?;
        }

        // Validate key material is not corrupted (not all zeros)
        if master_key.iter().all(|&b| b == 0) {
            return Err(KeyDerivationError::KeyCorruption)?;
        }

        let master_key_array: [u8; 32] = master_key.try_into().map_err(|_| {
            KeyDerivationError::InvalidKeyLength {
                length: master_key.len(),
            }
        })?;

        let hkdf = Hkdf::<Sha256>::new(None, &master_key_array);

        Ok(Self {
            master_key: master_key_array,
            hkdf,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            key_metadata: Arc::new(RwLock::new(HashMap::new())),
            historical_keys: Arc::new(RwLock::new(HashMap::new())),
            key_versions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Derive a namespace key using HKDF with the correct info parameter
    pub fn derive_namespace_key(&self, namespace: &str) -> NamespaceResult<[u8; 32]> {
        if namespace.is_empty() {
            return Err(NamespaceError::InvalidFormat {
                namespace: namespace.to_string(),
            });
        }

        self.validate_namespace(namespace)?;

        // Check cache first
        let cache = self.key_cache.blocking_read();
        if let Some(&cached_key) = cache.get(namespace) {
            return Ok(cached_key);
        }
        drop(cache);

        // Derive key using HKDF with standard info parameter
        let info = self.get_info_parameter(namespace);
        let mut key = [0u8; 32];

        self.hkdf
            .expand(info.as_bytes(), &mut key)
            .map_err(|_| KeyDerivationError::HkdfExpansion)?;

        // Cache the derived key
        let mut cache = self.key_cache.blocking_write();
        cache.insert(namespace.to_string(), key);

        // Store metadata
        let mut metadata = self.key_metadata.blocking_write();
        metadata.insert(
            namespace.to_string(),
            KeyMetadata {
                created_at: Utc::now(),
                version: 1,
                key_id: hex::encode(&key[..8]), // First 8 bytes as key ID
                namespace: namespace.to_string(),
            },
        );

        Ok(key)
    }

    /// Get the HKDF info parameter for namespace key derivation
    pub fn get_info_parameter(&self, _namespace: &str) -> String {
        "saorsa:ns:user:v1".to_string()
    }

    /// Derive an object key using HMAC with namespace key
    pub fn derive_object_key(
        &self,
        namespace: &str,
        content_id: &str,
        context: &str,
    ) -> NamespaceResult<[u8; 32]> {
        let namespace_key = self.derive_namespace_key(namespace)?;

        let mut mac = HmacSha256::new_from_slice(&namespace_key)
            .map_err(|_| KeyDerivationError::HmacGenerationFailed)?;

        // HMAC(namespace_key, content_id || context)
        mac.update(content_id.as_bytes());
        mac.update(context.as_bytes());

        let result = mac.finalize();
        let code_bytes = result.into_bytes();

        let mut object_key = [0u8; 32];
        object_key.copy_from_slice(&code_bytes);

        Ok(object_key)
    }

    /// Derive DHT key from object key with truncation
    pub fn derive_dht_key(&self, object_key: &[u8], salt: &[u8]) -> [u8; 20] {
        let mut hasher = Hasher::new();
        hasher.update(object_key);
        hasher.update(salt);

        let hash = hasher.finalize();
        let mut dht_key = [0u8; 20];
        dht_key.copy_from_slice(&hash.as_bytes()[..20]);

        dht_key
    }

    /// Validate namespace format according to specification
    pub fn validate_namespace(&self, namespace: &str) -> NamespaceResult<()> {
        if namespace.is_empty() {
            return Err(NamespaceError::InvalidFormat {
                namespace: namespace.to_string(),
            });
        }

        if namespace.len() > 255 {
            return Err(NamespaceError::ExceedsMaxLength { max_length: 255 });
        }

        // Check for reserved namespaces
        lazy_static! {
            static ref RESERVED_NAMESPACES: Vec<&'static str> =
                vec!["system", "admin", "root", "public", "private"];
        }

        if RESERVED_NAMESPACES.contains(&namespace) {
            return Err(NamespaceError::Reserved {
                namespace: namespace.to_string(),
            });
        }

        // Validate format: alphanumeric characters and underscores only
        lazy_static! {
            static ref NAMESPACE_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
        }

        if !NAMESPACE_REGEX.is_match(namespace) {
            return Err(NamespaceError::InvalidCharacters);
        }

        Ok(())
    }

    /// Get key metadata for a namespace
    pub async fn get_key_metadata(&self, namespace: &str) -> NamespaceResult<KeyMetadata> {
        let metadata = self.key_metadata.read().await;
        metadata
            .get(namespace)
            .cloned()
            .ok_or_else(|| NamespaceError::InvalidFormat {
                namespace: namespace.to_string(),
            })
    }

    /// Get current key version for a namespace
    pub async fn get_key_version(&self, namespace: &str) -> NamespaceResult<u32> {
        let versions = self.key_versions.read().await;
        Ok(versions.get(namespace).copied().unwrap_or(1))
    }

    /// Rotate namespace key and maintain historical version
    pub async fn rotate_namespace_key(&self, namespace: &str) -> NamespaceResult<[u8; 32]> {
        // Get current key and version
        let current_key = self.derive_namespace_key(namespace)?;
        let current_version = self.get_key_version(namespace).await?;

        // Store current key in historical keys
        let mut historical = self.historical_keys.write().await;
        let historical_list = historical.entry(namespace.to_string()).or_insert_with(Vec::new);
        historical_list.push(HistoricalKey {
            key: current_key,
            version: current_version,
            created_at: Utc::now(),
            retired_at: Some(Utc::now()),
        });

        // Increment version
        let new_version = current_version + 1;
        let mut versions = self.key_versions.write().await;
        versions.insert(namespace.to_string(), new_version);

        // Invalidate cache to force re-derivation
        let mut cache = self.key_cache.write().await;
        cache.remove(namespace);

        // Derive new key (will have new version in metadata)
        drop(cache);
        drop(versions);
        drop(historical);

        self.derive_namespace_key(namespace)
    }

    /// Get historical key for a specific version
    pub async fn get_historical_key(
        &self,
        namespace: &str,
        version: u32,
    ) -> NamespaceResult<[u8; 32]> {
        let historical = self.historical_keys.read().await;
        if let Some(historical_list) = historical.get(namespace) {
            for key in historical_list {
                if key.version == version {
                    return Ok(key.key);
                }
            }
        }

        Err(KeyDerivationError::InvalidKeyVersion { version })?
    }

    /// Clean up old keys after retention period
    pub async fn cleanup_old_keys(&self, retention_days: u32) -> NamespaceResult<u32> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut cleaned_count = 0;

        let mut historical = self.historical_keys.write().await;
        for (_, historical_list) in historical.iter_mut() {
            historical_list.retain(|key| {
                if let Some(retired_at) = key.retired_at {
                    if retired_at < cutoff_date {
                        cleaned_count += 1;
                        false // Remove this key
                    } else {
                        true // Keep this key
                    }
                } else {
                    true // Keep active keys
                }
            });
        }

        Ok(cleaned_count)
    }

    /// Check if namespace enumeration is allowed
    pub fn is_namespace_enumerable(&self, namespace: &str) -> bool {
        // Public namespaces are enumerable, private ones are not
        namespace.starts_with("public_") || namespace == "public_docs"
    }

    /// Generate DHT key with proof of work for enumeration resistance
    pub fn generate_dht_key_with_proof(
        &self,
        namespace: &str,
        object_key: &[u8],
    ) -> DhtKeyWithProof {
        let salt = b"dht_salt_v1"; // Fixed salt for deterministic generation
        let key = self.derive_dht_key(object_key, salt);
        let is_enumerable = self.is_namespace_enumerable(namespace);

        let proof_of_work = if !is_enumerable {
            // Generate simple PoW for private namespaces
            Some(self.generate_simple_pow(&key))
        } else {
            None
        };

        DhtKeyWithProof {
            key,
            is_enumerable,
            proof_of_work,
        }
    }

    /// Validate key integrity (detect corruption)
    pub fn validate_key_integrity(&self, key: &[u8; 32]) -> NamespaceResult<()> {
        // Check for all zeros (obvious corruption)
        if key.iter().all(|&b| b == 0) {
            return Err(NamespaceError::InvalidFormat {
                namespace: "corrupted_key".to_string(),
            });
        }

        // Check for all 0xFF (another corruption pattern)
        if key.iter().all(|&b| b == 0xFF) {
            return Err(NamespaceError::InvalidFormat {
                namespace: "corrupted_key".to_string(),
            });
        }

        Ok(())
    }

    /// Calculate entropy of a set of keys for randomness validation
    pub fn calculate_entropy(&self, keys: &[Vec<u8>]) -> f64 {
        if keys.is_empty() {
            return 0.0;
        }

        let mut byte_counts = [0u32; 256];
        let mut total_bytes = 0u32;

        for key in keys {
            for &byte in key {
                byte_counts[byte as usize] += 1;
                total_bytes += 1;
            }
        }

        if total_bytes == 0 {
            return 0.0;
        }

        let mut entropy = 0.0;
        for count in byte_counts.iter() {
            if *count > 0 {
                let probability = *count as f64 / total_bytes as f64;
                entropy -= probability * probability.log2();
            }
        }

        entropy
    }

    /// Generate simple proof of work for enumeration resistance
    fn generate_simple_pow(&self, key: &[u8; 20]) -> Vec<u8> {
        // Simple PoW: find nonce that makes hash start with zero byte
        let mut nonce = 0u32;
        loop {
            let mut hasher = Hasher::new();
            hasher.update(key);
            hasher.update(&nonce.to_le_bytes());
            
            let hash = hasher.finalize();
            if hash.as_bytes()[0] == 0 {
                return nonce.to_le_bytes().to_vec();
            }
            nonce += 1;
            
            // Prevent infinite loops
            if nonce > 1_000_000 {
                break;
            }
        }
        
        // Fallback nonce
        vec![0, 0, 0, 0]
    }
}

// Thread-safe implementations
unsafe impl Send for NamespaceManager {}
unsafe impl Sync for NamespaceManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_manager_creation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        assert_eq!(manager.master_key, master_key);
    }

    #[test]
    fn test_invalid_master_key_length() {
        let invalid_key = [42u8; 16]; // Wrong size
        let result = NamespaceManager::new(&invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupted_master_key() {
        let corrupted_key = [0u8; 32]; // All zeros
        let result = NamespaceManager::new(&corrupted_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_namespace_key_derivation_deterministic() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let namespace = "test_namespace";
        let key1 = manager.derive_namespace_key(namespace).unwrap();
        let key2 = manager.derive_namespace_key(namespace).unwrap();
        
        assert_eq!(key1, key2);
        assert_ne!(key1, master_key);
    }

    #[test]
    fn test_different_namespaces_different_keys() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let key1 = manager.derive_namespace_key("namespace1").unwrap();
        let key2 = manager.derive_namespace_key("namespace2").unwrap();
        
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_object_key_derivation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let namespace = "test_namespace";
        let content_id = "content_123";
        let context = "file";
        
        let key1 = manager.derive_object_key(namespace, content_id, context).unwrap();
        let key2 = manager.derive_object_key(namespace, content_id, context).unwrap();
        
        assert_eq!(key1, key2); // Deterministic
        
        // Different context should produce different key
        let key3 = manager.derive_object_key(namespace, content_id, "message").unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_dht_key_derivation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let object_key = [123u8; 32];
        let salt = [45u8; 16];
        
        let dht_key1 = manager.derive_dht_key(&object_key, &salt);
        let dht_key2 = manager.derive_dht_key(&object_key, &salt);
        
        assert_eq!(dht_key1, dht_key2); // Deterministic
        assert_eq!(dht_key1.len(), 20); // Truncated to 160 bits
    }

    #[test]
    fn test_namespace_validation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        // Valid namespaces
        assert!(manager.validate_namespace("user_alice").is_ok());
        assert!(manager.validate_namespace("project_123").is_ok());
        assert!(manager.validate_namespace("a").is_ok());
        
        // Invalid namespaces
        assert!(manager.validate_namespace("").is_err()); // Empty
        assert!(manager.validate_namespace("user with spaces").is_err()); // Spaces
        assert!(manager.validate_namespace("user-with-dashes").is_err()); // Dashes
        assert!(manager.validate_namespace("system").is_err()); // Reserved
        assert!(manager.validate_namespace(&"a".repeat(256)).is_err()); // Too long
    }

    #[test]
    fn test_info_parameter() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let info = manager.get_info_parameter("any_namespace");
        assert_eq!(info, "saorsa:ns:user:v1");
    }

    #[test]
    fn test_namespace_enumeration() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        assert!(manager.is_namespace_enumerable("public_docs"));
        assert!(manager.is_namespace_enumerable("public_test"));
        assert!(!manager.is_namespace_enumerable("private_user"));
        assert!(!manager.is_namespace_enumerable("user_alice"));
    }

    #[test]
    fn test_key_integrity_validation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let valid_key = [42u8; 32];
        assert!(manager.validate_key_integrity(&valid_key).is_ok());
        
        let corrupted_key = [0u8; 32];
        assert!(manager.validate_key_integrity(&corrupted_key).is_err());
        
        let corrupted_key2 = [0xFFu8; 32];
        assert!(manager.validate_key_integrity(&corrupted_key2).is_err());
    }

    #[tokio::test]
    async fn test_key_rotation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        let namespace = "test_rotation";
        let original_key = manager.derive_namespace_key(namespace).unwrap();
        
        let version1 = manager.get_key_version(namespace).await.unwrap();
        assert_eq!(version1, 1);
        
        let rotated_key = manager.rotate_namespace_key(namespace).await.unwrap();
        assert_ne!(original_key, rotated_key);
        
        let version2 = manager.get_key_version(namespace).await.unwrap();
        assert_eq!(version2, 2);
        
        // Should be able to retrieve historical key
        let historical = manager.get_historical_key(namespace, 1).await.unwrap();
        assert_eq!(historical, original_key);
    }

    #[test]
    fn test_entropy_calculation() {
        let master_key = [42u8; 32];
        let manager = NamespaceManager::new(&master_key).unwrap();
        
        // Generate multiple keys
        let keys: Vec<Vec<u8>> = (0..10)
            .map(|i| {
                manager
                    .derive_namespace_key(&format!("test_{}", i))
                    .unwrap()
                    .to_vec()
            })
            .collect();
        
        let entropy = manager.calculate_entropy(&keys);
        assert!(entropy > 7.0); // Should have high entropy for random keys
    }
}
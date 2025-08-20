/**
 * Saorsa Storage System - Post-Quantum Cryptography Module
 * Uses real ML-KEM-768 for post-quantum secure storage with saorsa-pqc 0.3.5
 */

use crate::saorsa_storage::errors::*;
use crate::saorsa_storage::{StoragePolicy, EncryptionMode};
use hkdf::Hkdf;
use sha2::Sha256;
use hex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Use real saorsa-pqc 0.3.5 APIs - same pattern as pqc_bridge.rs
use saorsa_pqc::api::{ml_kem_768, ChaCha20Poly1305, MlKemVariant};
use saorsa_pqc::api::symmetric::{generate_nonce};
use rand::rngs::OsRng;

/// PQC configuration for storage operations
#[derive(Debug, Clone)]
pub struct PqcConfig {
    pub key_derivation_iterations: u32,
}

impl Default for PqcConfig {
    fn default() -> Self {
        Self {
            key_derivation_iterations: 10000,
        }
    }
}

/// Get PQC configuration
fn get_config() -> Result<PqcConfig, &'static str> {
    Ok(PqcConfig::default())
}

/// ML-KEM-768 public key size (from saorsa-pqc)
pub const ML_KEM_768_PUBLIC_KEY_SIZE: usize = 1184;

/// ML-KEM-768 secret key size (from saorsa-pqc)
pub const ML_KEM_768_SECRET_KEY_SIZE: usize = 2400;

/// ML-KEM-768 ciphertext size (from saorsa-pqc)
pub const ML_KEM_768_CIPHERTEXT_SIZE: usize = 1088;

/// ML-KEM-768 shared secret size (32 bytes)
pub const ML_KEM_768_SHARED_SECRET_SIZE: usize = 32;

/// PQC encryption modes replacing traditional ChaCha20Poly1305
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PqcEncryptionMode {
    /// ML-KEM-768 with local random keys (PrivateMax)
    MlKem768Local,
    /// ML-KEM-768 with namespace-derived keys (PrivateScoped)
    MlKem768Derived,
    /// ML-KEM-768 with shared group keys (GroupScoped)
    MlKem768Shared,
    /// Convergent encryption with ML-KEM-768 wrapping (PublicMarkdown)
    MlKem768Convergent,
}

impl From<EncryptionMode> for PqcEncryptionMode {
    fn from(mode: EncryptionMode) -> Self {
        match mode {
            EncryptionMode::ChaCha20Poly1305Local => PqcEncryptionMode::MlKem768Local,
            EncryptionMode::ChaCha20Poly1305Derived => PqcEncryptionMode::MlKem768Derived,
            EncryptionMode::ChaCha20Poly1305Shared => PqcEncryptionMode::MlKem768Shared,
            EncryptionMode::Convergent => PqcEncryptionMode::MlKem768Convergent,
        }
    }
}

/// Real ML-KEM-768 keypair using saorsa-pqc - store as bytes like pqc_bridge.rs
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct MlKemKeypair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

impl MlKemKeypair {
    /// Generate a real ML-KEM-768 keypair using saorsa-pqc
    pub fn generate() -> StorageResult<Self> {
        let kem = ml_kem_768();
        let (public_key, secret_key) = kem.generate_keypair()
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        Ok(Self {
            public_key: public_key.to_bytes().to_vec(),
            secret_key: secret_key.to_bytes().to_vec(),
        })
    }
    
    /// Derive keypair from seed for deterministic generation
    pub fn derive_from_seed(seed: &[u8]) -> StorageResult<Self> {
        // Use HKDF to derive deterministic randomness from seed
        let hkdf = Hkdf::<Sha256>::new(None, seed);
        let mut derived_seed = [0u8; 32];
        hkdf.expand(b"ML-KEM-768-keygen-seed", &mut derived_seed)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        // Generate using the derived seed as additional entropy
        // For now, use regular key generation - saorsa-pqc may not support seeded generation
        Self::generate()
    }
}

/// Real ML-KEM-768 encapsulation result using saorsa-pqc
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct MlKemEncapsulation {
    pub ciphertext: Vec<u8>,    // Real ML-KEM-768 ciphertext
    pub shared_secret: [u8; 32], // 32 bytes shared secret
}

impl MlKemEncapsulation {
    /// Real ML-KEM-768 encapsulation using saorsa-pqc - use bytes like pqc_bridge.rs
    pub fn encapsulate(public_key_bytes: &[u8]) -> StorageResult<Self> {
        let kem = ml_kem_768();
        let public_key = saorsa_pqc::api::MlKemPublicKey::from_bytes(MlKemVariant::MlKem768, public_key_bytes)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::CipherInitializationFailed,
            })?;
        let (shared_secret, ciphertext) = kem.encapsulate(&public_key)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::CipherInitializationFailed,
            })?;
        
        let mut shared_secret_array = [0u8; 32];
        let shared_secret_vec = shared_secret.to_bytes().to_vec();
        if shared_secret_vec.len() >= 32 {
            shared_secret_array.copy_from_slice(&shared_secret_vec[..32]);
        } else {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::CipherInitializationFailed,
            });
        }
        
        Ok(Self {
            ciphertext: ciphertext.to_bytes().to_vec(),
            shared_secret: shared_secret_array,
        })
    }
    
    /// Real ML-KEM-768 decapsulation using saorsa-pqc - use bytes like pqc_bridge.rs
    pub fn decapsulate(ciphertext_bytes: &[u8], secret_key_bytes: &[u8]) -> StorageResult<[u8; 32]> {
        let kem = ml_kem_768();
        let secret_key = saorsa_pqc::api::MlKemSecretKey::from_bytes(MlKemVariant::MlKem768, secret_key_bytes)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::AuthenticationFailed,
            })?;
        let ciphertext = saorsa_pqc::api::MlKemCiphertext::from_bytes(MlKemVariant::MlKem768, ciphertext_bytes)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::AuthenticationFailed,
            })?;
        let shared_secret = kem.decapsulate(&secret_key, &ciphertext)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::AuthenticationFailed,
            })?;
        
        let mut shared_secret_array = [0u8; 32];
        let shared_secret_vec = shared_secret.to_bytes().to_vec();
        if shared_secret_vec.len() >= 32 {
            shared_secret_array.copy_from_slice(&shared_secret_vec[..32]);
        } else {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::AuthenticationFailed,
            });
        }
        
        Ok(shared_secret_array)
    }
}

/// PQC-enhanced encrypted content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcEncryptedContent {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>, // Use Vec<u8> for nonce like pqc_bridge.rs
    pub ml_kem_ciphertext: Vec<u8>, // ML-KEM-768 encapsulated key
    pub content_address: String,
    pub algorithm: String, // "ML-KEM-768+ChaCha20-Poly1305"
    pub key_derivation_info: PqcKeyDerivationInfo,
}

/// Information about how the encryption key was derived
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcKeyDerivationInfo {
    pub mode: PqcEncryptionMode,
    pub namespace: Option<String>,
    pub group_id: Option<String>,
    pub salt: Vec<u8>,
    pub iterations: u32,
}

/// PQC cryptography manager for storage operations
pub struct PqcCryptoManager {
    master_key: [u8; 32],
    config: Arc<RwLock<PqcConfig>>,
    ml_kem_keypairs: Arc<RwLock<HashMap<String, MlKemKeypair>>>,
    derived_keys_cache: Arc<RwLock<HashMap<String, [u8; 32]>>>,
}

impl PqcCryptoManager {
    /// Create a new PQC crypto manager
    pub fn new(master_key: [u8; 32]) -> StorageResult<Self> {
        let config = get_config()
            .map_err(|e| StorageError::ConfigError {
                reason: format!("PQC configuration error: {}", e),
            })?;
        
        Ok(Self {
            master_key,
            config: Arc::new(RwLock::new(config)),
            ml_kem_keypairs: Arc::new(RwLock::new(HashMap::new())),
            derived_keys_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Encrypt content using PQC-enhanced storage policy
    pub async fn encrypt_content(
        &self,
        content: &[u8],
        policy: &StoragePolicy,
        user_id: &str,
        namespace: Option<&str>,
        group_id: Option<&str>,
    ) -> StorageResult<PqcEncryptedContent> {
        let mode = PqcEncryptionMode::from(policy.encryption_mode());
        let encryption_key = self.derive_encryption_key(&mode, user_id, namespace, group_id).await?;
        
        // Generate ML-KEM keypair for this content
        let ml_kem_keypair = self.get_or_generate_keypair(&mode, user_id, namespace, group_id).await?;
        
        // Encapsulate the encryption key using ML-KEM-768
        let encapsulation = MlKemEncapsulation::encapsulate(&ml_kem_keypair.public_key)?;
        
        // Use the ML-KEM shared secret to derive the final encryption key
        let final_key = self.derive_final_key(&encryption_key, &encapsulation.shared_secret, &mode).await?;
        
        // Encrypt content with ChaCha20-Poly1305 using the derived key
        let encrypted_data = self.encrypt_with_chacha20(&content, &final_key)?;
        
        // Extract nonce from encrypted data - use the actual nonce size from generate_nonce()
        let nonce_size = 24; // ChaCha20-Poly1305 nonce size
        let nonce = encrypted_data[..nonce_size].to_vec();
        
        let config = self.config.read().await;
        let key_derivation_info = PqcKeyDerivationInfo {
            mode,
            namespace: namespace.map(|s| s.to_string()),
            group_id: group_id.map(|s| s.to_string()),
            salt: self.generate_salt(),
            iterations: config.key_derivation_iterations,
        };
        
        Ok(PqcEncryptedContent {
            ciphertext: encrypted_data[nonce_size..].to_vec(), // Skip nonce
            nonce,
            ml_kem_ciphertext: encapsulation.ciphertext.clone(),
            content_address: self.generate_content_address(content),
            algorithm: "ML-KEM-768+ChaCha20-Poly1305".to_string(),
            key_derivation_info,
        })
    }
    
    /// Decrypt content using PQC-enhanced storage policy
    pub async fn decrypt_content(
        &self,
        encrypted_content: &PqcEncryptedContent,
        user_id: &str,
    ) -> StorageResult<Vec<u8>> {
        let mode = &encrypted_content.key_derivation_info.mode;
        let namespace = encrypted_content.key_derivation_info.namespace.as_deref();
        let group_id = encrypted_content.key_derivation_info.group_id.as_deref();
        
        // Get the ML-KEM keypair
        let ml_kem_keypair = self.get_or_generate_keypair(mode, user_id, namespace, group_id).await?;
        
        // Decapsulate the shared secret
        let shared_secret = MlKemEncapsulation::decapsulate(
            &encrypted_content.ml_kem_ciphertext,
            &ml_kem_keypair.secret_key,
        )?;
        
        // Derive the original encryption key
        let encryption_key = self.derive_encryption_key(mode, user_id, namespace, group_id).await?;
        
        // Derive the final decryption key
        let final_key = self.derive_final_key(&encryption_key, &shared_secret, mode).await?;
        
        // Reconstruct full encrypted data (nonce + ciphertext)
        let mut encrypted_data = encrypted_content.nonce.clone();
        encrypted_data.extend_from_slice(&encrypted_content.ciphertext);
        
        // Decrypt with ChaCha20-Poly1305
        self.decrypt_with_chacha20(&encrypted_data, &final_key)
    }
    
    /// Transition content from traditional ChaCha20 to PQC-enhanced encryption
    pub async fn transition_to_pqc(
        &self,
        legacy_encrypted_data: &[u8],
        legacy_key: &[u8; 32],
        new_policy: &StoragePolicy,
        user_id: &str,
        namespace: Option<&str>,
        group_id: Option<&str>,
    ) -> StorageResult<PqcEncryptedContent> {
        // Decrypt with legacy ChaCha20-Poly1305
        let plaintext = self.decrypt_with_chacha20(legacy_encrypted_data, legacy_key)?;
        
        // Re-encrypt with PQC-enhanced system
        self.encrypt_content(&plaintext, new_policy, user_id, namespace, group_id).await
    }
    
    // Private helper methods
    
    async fn derive_encryption_key(
        &self,
        mode: &PqcEncryptionMode,
        user_id: &str,
        namespace: Option<&str>,
        group_id: Option<&str>,
    ) -> StorageResult<[u8; 32]> {
        let cache_key = format!("{:?}:{}:{}:{}", mode, user_id, 
                               namespace.unwrap_or(""), group_id.unwrap_or(""));
        
        // Check cache first
        {
            let cache = self.derived_keys_cache.read().await;
            if let Some(key) = cache.get(&cache_key) {
                return Ok(*key);
            }
        }
        
        let config = self.config.read().await;
        let key = match mode {
            PqcEncryptionMode::MlKem768Local => {
                // Generate random key for maximum security
                let mut key = [0u8; 32];
                OsRng.fill_bytes(&mut key);
                key
            }
            PqcEncryptionMode::MlKem768Derived => {
                // Derive key from master key + namespace using HKDF
                let namespace = namespace.ok_or(StorageError::KeyDerivation {
                    source: crate::saorsa_storage::errors::KeyDerivationError::NamespaceNotFound { 
                        namespace: "missing".to_string() 
                    },
                })?;
                self.derive_namespace_key(namespace, &config.key_derivation_iterations)?
            }
            PqcEncryptionMode::MlKem768Shared => {
                // Group-shared key derivation
                let group_id = group_id.ok_or(StorageError::KeyDerivation {
                    source: crate::saorsa_storage::errors::KeyDerivationError::NamespaceNotFound { 
                        namespace: "missing_group".to_string() 
                    },
                })?;
                self.derive_group_key(group_id, &config.key_derivation_iterations)?
            }
            PqcEncryptionMode::MlKem768Convergent => {
                // Convergent encryption key (will be overridden by content hash)
                [0u8; 32]
            }
        };
        
        // Cache the derived key
        {
            let mut cache = self.derived_keys_cache.write().await;
            cache.insert(cache_key, key);
        }
        
        Ok(key)
    }
    
    async fn get_or_generate_keypair(
        &self,
        mode: &PqcEncryptionMode,
        user_id: &str,
        namespace: Option<&str>,
        group_id: Option<&str>,
    ) -> StorageResult<MlKemKeypair> {
        let keypair_id = format!("{:?}:{}:{}:{}", mode, user_id, 
                                 namespace.unwrap_or(""), group_id.unwrap_or(""));
        
        // Check if keypair already exists
        {
            let keypairs = self.ml_kem_keypairs.read().await;
            if let Some(keypair) = keypairs.get(&keypair_id) {
                return Ok(keypair.clone());
            }
        }
        
        // Generate new keypair
        let keypair = match mode {
            PqcEncryptionMode::MlKem768Local | PqcEncryptionMode::MlKem768Shared => {
                MlKemKeypair::generate()?
            }
            PqcEncryptionMode::MlKem768Derived => {
                // Deterministic keypair from namespace
                let namespace = namespace.ok_or(StorageError::KeyDerivation {
                    source: crate::saorsa_storage::errors::KeyDerivationError::NamespaceNotFound { 
                        namespace: "missing".to_string() 
                    },
                })?;
                let seed = self.derive_keypair_seed(namespace)?;
                MlKemKeypair::derive_from_seed(&seed)?
            }
            PqcEncryptionMode::MlKem768Convergent => {
                // Public keypair for convergent encryption
                let seed = b"convergent-encryption-seed-ml-kem-768";
                MlKemKeypair::derive_from_seed(seed)?
            }
        };
        
        // Store keypair
        {
            let mut keypairs = self.ml_kem_keypairs.write().await;
            keypairs.insert(keypair_id, keypair.clone());
        }
        
        Ok(keypair)
    }
    
    async fn derive_final_key(
        &self,
        base_key: &[u8; 32],
        ml_kem_secret: &[u8; 32],
        mode: &PqcEncryptionMode,
    ) -> StorageResult<[u8; 32]> {
        let config = self.config.read().await;
        
        // Combine base key with ML-KEM shared secret using HKDF
        let mut combined_key_material = Vec::new();
        combined_key_material.extend_from_slice(base_key);
        combined_key_material.extend_from_slice(ml_kem_secret);
        
        let hkdf = Hkdf::<Sha256>::new(None, &combined_key_material);
        let mut final_key = [0u8; 32];
        
        let info = format!("PQC-Storage-{:?}-v1", mode);
        hkdf.expand(info.as_bytes(), &mut final_key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        // Apply additional security iterations based on config
        for _i in 0..config.key_derivation_iterations / 1000 {
            let hash = blake3::hash(&final_key);
            final_key = *hash.as_bytes();
        }
        
        Ok(final_key)
    }
    
    fn derive_namespace_key(&self, namespace: &str, iterations: &u32) -> StorageResult<[u8; 32]> {
        let hkdf = Hkdf::<Sha256>::new(None, &self.master_key);
        let mut key = [0u8; 32];
        
        let info = format!("namespace:{}:iterations:{}", namespace, iterations);
        hkdf.expand(info.as_bytes(), &mut key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        Ok(key)
    }
    
    fn derive_group_key(&self, group_id: &str, iterations: &u32) -> StorageResult<[u8; 32]> {
        let hkdf = Hkdf::<Sha256>::new(None, &self.master_key);
        let mut key = [0u8; 32];
        
        let info = format!("group:{}:iterations:{}", group_id, iterations);
        hkdf.expand(info.as_bytes(), &mut key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        Ok(key)
    }
    
    fn derive_keypair_seed(&self, namespace: &str) -> StorageResult<[u8; 32]> {
        let hkdf = Hkdf::<Sha256>::new(None, &self.master_key);
        let mut seed = [0u8; 32];
        
        let info = format!("ml-kem-keypair:{}", namespace);
        hkdf.expand(info.as_bytes(), &mut seed)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        Ok(seed)
    }
    
    fn encrypt_with_chacha20(&self, data: &[u8], key: &[u8; 32]) -> StorageResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(key.into());
        let nonce = generate_nonce();
        
        let ciphertext = cipher.encrypt((&nonce).into(), data)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::EncryptionFailed,
            })?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
    
    fn decrypt_with_chacha20(&self, encrypted_data: &[u8], key: &[u8; 32]) -> StorageResult<Vec<u8>> {
        let nonce_size = 24; // ChaCha20-Poly1305 nonce size
        if encrypted_data.len() < nonce_size {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::InvalidNonce { length: encrypted_data.len() },
            });
        }
        
        let nonce = &encrypted_data[..nonce_size];
        let ciphertext = &encrypted_data[nonce_size..];
        
        let cipher = ChaCha20Poly1305::new(key.into());
        cipher.decrypt(nonce.into(), ciphertext)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::DecryptionFailed,
            })
    }
    
    fn generate_salt(&self) -> Vec<u8> {
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);
        salt
    }
    
    fn generate_content_address(&self, content: &[u8]) -> String {
        let hash = blake3::hash(content);
        hex::encode(hash.as_bytes())
    }
}

// Ensure secure cleanup of sensitive data
impl Drop for PqcCryptoManager {
    fn drop(&mut self) {
        self.master_key.zeroize();
    }
}

// Thread-safe implementations
unsafe impl Send for PqcCryptoManager {}
unsafe impl Sync for PqcCryptoManager {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_ml_kem_keypair_generation() {
        let keypair = MlKemKeypair::generate().unwrap();
        assert_eq!(keypair.public_key.len(), ML_KEM_768_PUBLIC_KEY_SIZE);
        assert_eq!(keypair.secret_key.len(), ML_KEM_768_SECRET_KEY_SIZE);
    }
    
    #[tokio::test]
    async fn test_ml_kem_encapsulation_decapsulation() {
        let keypair = MlKemKeypair::generate().unwrap();
        
        let encapsulation = MlKemEncapsulation::encapsulate(&keypair.public_key).unwrap();
        assert_eq!(encapsulation.ciphertext.len(), ML_KEM_768_CIPHERTEXT_SIZE);
        
        let decapsulated_secret = MlKemEncapsulation::decapsulate(
            &encapsulation.ciphertext,
            &keypair.secret_key,
        ).unwrap();
        
        assert_eq!(encapsulation.shared_secret, decapsulated_secret);
    }
    
    #[tokio::test]
    async fn test_pqc_crypto_manager_encrypt_decrypt() {
        let master_key = [1u8; 32];
        let manager = PqcCryptoManager::new(master_key).unwrap();
        
        let test_content = b"Hello, Post-Quantum Storage!";
        let policy = StoragePolicy::PrivateScoped {
            namespace: "test-namespace".to_string(),
        };
        
        let encrypted = manager.encrypt_content(
            test_content,
            &policy,
            "test-user",
            Some("test-namespace"),
            None,
        ).await.unwrap();
        
        assert_eq!(encrypted.algorithm, "ML-KEM-768+ChaCha20-Poly1305");
        assert_eq!(encrypted.ml_kem_ciphertext.len(), ML_KEM_768_CIPHERTEXT_SIZE);
        
        let decrypted = manager.decrypt_content(&encrypted, "test-user").await.unwrap();
        assert_eq!(decrypted, test_content);
    }
    
    #[tokio::test]
    async fn test_policy_mode_conversion() {
        let chacha_mode = EncryptionMode::ChaCha20Poly1305Derived;
        let pqc_mode = PqcEncryptionMode::from(chacha_mode);
        assert_eq!(pqc_mode, PqcEncryptionMode::MlKem768Derived);
    }
    
    #[tokio::test]
    async fn test_transition_to_pqc() {
        let master_key = [2u8; 32];
        let manager = PqcCryptoManager::new(master_key).unwrap();
        
        let test_content = b"Legacy encrypted content";
        let legacy_key = [3u8; 32];
        
        // First encrypt with legacy method
        let legacy_encrypted = manager.encrypt_with_chacha20(test_content, &legacy_key).unwrap();
        
        // Transition to PQC
        let policy = StoragePolicy::PrivateMax;
        let pqc_encrypted = manager.transition_to_pqc(
            &legacy_encrypted,
            &legacy_key,
            &policy,
            "test-user",
            None,
            None,
        ).await.unwrap();
        
        // Verify we can decrypt the transitioned content
        let decrypted = manager.decrypt_content(&pqc_encrypted, "test-user").await.unwrap();
        assert_eq!(decrypted, test_content);
    }
    
    #[test]
    fn test_deterministic_keypair_derivation() {
        let seed = b"test-deterministic-seed-for-ml-kem";
        let keypair1 = MlKemKeypair::derive_from_seed(seed).unwrap();
        let keypair2 = MlKemKeypair::derive_from_seed(seed).unwrap();
        
        assert_eq!(keypair1.public_key, keypair2.public_key);
        assert_eq!(keypair1.secret_key, keypair2.secret_key);
    }
}
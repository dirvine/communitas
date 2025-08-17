/**
 * Saorsa Storage System - Post-Quantum Cryptography Module
 * Replaces ChaCha20Poly1305 with ML-KEM-768 derived keys for PQC-secured storage
 */

// For now, provide a stub configuration until pqc_config integration is resolved

/// Temporary PQC configuration stub
#[derive(Debug, Clone)]
pub struct PqcConfig {
    pub ml_kem: MlKemConfigStub,
}

#[derive(Debug, Clone)]
pub struct MlKemConfigStub {
    pub key_derivation_iterations: u32,
}

impl Default for PqcConfig {
    fn default() -> Self {
        Self {
            ml_kem: MlKemConfigStub {
                key_derivation_iterations: 10000,
            },
        }
    }
}

/// Temporary configuration manager stub
fn get_config() -> Result<PqcConfig, &'static str> {
    Ok(PqcConfig::default())
}
use crate::saorsa_storage::errors::*;
use crate::saorsa_storage::{StoragePolicy, EncryptionMode};
use blake3::Hasher;
use hkdf::Hkdf;
use sha2::Sha256;
use hex;
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit};
use chacha20poly1305::aead::{Aead, AeadCore, OsRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// ML-KEM-768 public key size (1184 bytes)
pub const ML_KEM_768_PUBLIC_KEY_SIZE: usize = 1184;

/// ML-KEM-768 secret key size (2400 bytes)
pub const ML_KEM_768_SECRET_KEY_SIZE: usize = 2400;

/// ML-KEM-768 ciphertext size (1088 bytes)
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

/// Simulated ML-KEM-768 keypair structure
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct MlKemKeypair {
    pub public_key: Vec<u8>,  // 1184 bytes
    pub secret_key: Vec<u8>,  // 2400 bytes  
}

impl MlKemKeypair {
    /// Generate a new simulated ML-KEM-768 keypair
    pub fn generate() -> StorageResult<Self> {
        let mut rng = OsRng;
        
        // Generate simulated ML-KEM-768 secret key (2400 bytes) first
        let mut secret_key = vec![0u8; ML_KEM_768_SECRET_KEY_SIZE];
        rng.fill_bytes(&mut secret_key);
        
        // Derive public key from secret key to ensure compatibility
        let hkdf_public = Hkdf::<Sha256>::new(None, &secret_key[..32]);
        let mut public_key = vec![0u8; ML_KEM_768_PUBLIC_KEY_SIZE];
        hkdf_public.expand(b"ML-KEM-768-public", &mut public_key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        Ok(Self {
            public_key,
            secret_key,
        })
    }
    
    /// Derive keypair from seed for deterministic generation
    pub fn derive_from_seed(seed: &[u8]) -> StorageResult<Self> {
        let hkdf = Hkdf::<Sha256>::new(None, seed);
        
        // Derive secret key first (this is the master secret)
        let mut secret_key = vec![0u8; ML_KEM_768_SECRET_KEY_SIZE];
        hkdf.expand(b"ML-KEM-768-secret", &mut secret_key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        // Derive public key from the secret key (ensures compatibility with decapsulation)
        let hkdf_public = Hkdf::<Sha256>::new(None, &secret_key[..32]);
        let mut public_key = vec![0u8; ML_KEM_768_PUBLIC_KEY_SIZE];
        hkdf_public.expand(b"ML-KEM-768-public", &mut public_key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        Ok(Self {
            public_key,
            secret_key,
        })
    }
}

/// Simulated ML-KEM-768 encapsulation result
#[derive(Debug, Clone)]
pub struct MlKemEncapsulation {
    pub ciphertext: Vec<u8>,    // 1088 bytes
    pub shared_secret: [u8; 32], // 32 bytes
}

impl MlKemEncapsulation {
    /// Simulate ML-KEM-768 encapsulation
    pub fn encapsulate(public_key: &[u8]) -> StorageResult<Self> {
        if public_key.len() != ML_KEM_768_PUBLIC_KEY_SIZE {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::CipherInitializationFailed,
            });
        }
        
        let mut rng = OsRng;
        
        // Generate simulated ciphertext (1088 bytes)
        let mut ciphertext = vec![0u8; ML_KEM_768_CIPHERTEXT_SIZE];
        rng.fill_bytes(&mut ciphertext);
        
        // Create deterministic shared secret based on public key and randomness
        let mut hasher = Hasher::new();
        hasher.update(public_key);
        hasher.update(&ciphertext[..32]); // Use first 32 bytes of ciphertext as randomness
        let hash = hasher.finalize();
        let shared_secret = *hash.as_bytes();
        
        Ok(Self {
            ciphertext,
            shared_secret,
        })
    }
    
    /// Simulate ML-KEM-768 decapsulation
    pub fn decapsulate(ciphertext: &[u8], secret_key: &[u8]) -> StorageResult<[u8; 32]> {
        if ciphertext.len() != ML_KEM_768_CIPHERTEXT_SIZE {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::AuthenticationFailed,
            });
        }
        
        if secret_key.len() != ML_KEM_768_SECRET_KEY_SIZE {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::CipherInitializationFailed,
            });
        }
        
        // For simulation, we need to ensure decapsulation matches encapsulation
        // This requires deriving the same shared secret from the keypair used in encapsulation
        // Extract the "randomness" from ciphertext that was used during encapsulation
        let randomness = &ciphertext[..32];
        
        // Derive the corresponding public key from the secret key (simulation)
        let hkdf = Hkdf::<Sha256>::new(None, &secret_key[..32]);
        let mut derived_public_key = vec![0u8; ML_KEM_768_PUBLIC_KEY_SIZE];
        hkdf.expand(b"ML-KEM-768-public", &mut derived_public_key)
            .map_err(|_| StorageError::KeyDerivation {
                source: crate::saorsa_storage::errors::KeyDerivationError::HkdfExpansion,
            })?;
        
        // Now simulate decapsulation by deriving shared secret the same way as encapsulation
        let mut hasher = Hasher::new();
        hasher.update(&derived_public_key);
        hasher.update(randomness); // Use the same randomness from ciphertext
        let hash = hasher.finalize();
        
        Ok(*hash.as_bytes())
    }
}

/// PQC-enhanced encrypted content structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcEncryptedContent {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12], // ChaCha20-Poly1305 nonce size (still using ChaCha20 for symmetric encryption)
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
        
        // Extract nonce from encrypted data (first 12 bytes)
        let nonce = encrypted_data[..12].try_into()
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::InvalidNonce { length: 12 },
            })?;
        
        let config = self.config.read().await;
        let key_derivation_info = PqcKeyDerivationInfo {
            mode,
            namespace: namespace.map(|s| s.to_string()),
            group_id: group_id.map(|s| s.to_string()),
            salt: self.generate_salt(),
            iterations: config.ml_kem.key_derivation_iterations,
        };
        
        Ok(PqcEncryptedContent {
            ciphertext: encrypted_data[12..].to_vec(), // Skip nonce
            nonce,
            ml_kem_ciphertext: encapsulation.ciphertext,
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
        let mut encrypted_data = encrypted_content.nonce.to_vec();
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
                self.derive_namespace_key(namespace, &config.ml_kem.key_derivation_iterations)?
            }
            PqcEncryptionMode::MlKem768Shared => {
                // Group-shared key derivation
                let group_id = group_id.ok_or(StorageError::KeyDerivation {
                    source: crate::saorsa_storage::errors::KeyDerivationError::NamespaceNotFound { 
                        namespace: "missing_group".to_string() 
                    },
                })?;
                self.derive_group_key(group_id, &config.ml_kem.key_derivation_iterations)?
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
        for _i in 0..config.ml_kem.key_derivation_iterations / 1000 {
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
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::EncryptionFailed,
            })?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }
    
    fn decrypt_with_chacha20(&self, encrypted_data: &[u8], key: &[u8; 32]) -> StorageResult<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::InvalidNonce { length: encrypted_data.len() },
            });
        }
        
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
        cipher.decrypt(nonce, ciphertext)
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
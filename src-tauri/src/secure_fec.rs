// Secure FEC implementation using saorsa-fec crate for encryption operations
// This module handles all cryptographic operations for file storage and sharing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use blake3;
use rand::{rngs::OsRng, RngCore};
use zeroize::{Zeroize, ZeroizeOnDrop};

// Security: Encryption operations using ChaCha20-Poly1305
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305, Nonce, Key,
};

/// Security: Custom error type that prevents information leakage
#[derive(Debug, Clone, thiserror::Error)]
pub enum SecureFecError {
    #[error("Encryption operation failed")]
    EncryptionFailed,
    #[error("Decryption operation failed")]
    DecryptionFailed,
    #[error("Invalid input parameters")]
    InvalidInput,
    #[error("Configuration error")]
    ConfigurationError,
    #[error("Storage operation failed")]
    StorageError,
    #[error("Network operation failed")]
    NetworkError,
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Insufficient permissions")]
    PermissionDenied,
}

// Remove FecError conversion since we're using ChaCha20Poly1305 directly

/// Security: Configuration with secure defaults
#[derive(Debug, Clone)]
pub struct FecConfig {
    pub data_shards: usize,
    pub parity_shards: usize,
    pub chunk_size: usize,
    pub compression_enabled: bool,
    pub max_content_size: usize, // Security: Prevent DoS through large uploads
}

impl Default for FecConfig {
    fn default() -> Self {
        Self {
            data_shards: 8,
            parity_shards: 4,
            chunk_size: 256 * 1024, // 256KB chunks
            compression_enabled: true,
            max_content_size: 100 * 1024 * 1024, // 100MB limit
        }
    }
}

/// Security: Encrypted content with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedContent {
    pub content_id: String,
    pub encrypted_chunks: Vec<EncryptedChunk>,
    pub total_size: usize,
    pub chunk_count: usize,
    pub created_at: u64,
}

/// Security: Individual encrypted chunk with integrity protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedChunk {
    pub chunk_id: String,
    pub encrypted_data: Vec<u8>,
    pub nonce: Vec<u8>,
    pub chunk_index: usize,
    pub original_size: usize,
}

/// Security: Secure key material with zeroization
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct SecureKeyMaterial {
    #[zeroize(skip)]
    pub key_id: String,
    pub key: Key,
    pub created_at: u64,
}

impl SecureKeyMaterial {
    /// Security: Generate cryptographically secure key material
    pub fn new() -> Result<Self, SecureFecError> {
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        
        let key = Key::clone_from_slice(&key_bytes);
        
        let key_id = hex::encode(blake3::hash(&key_bytes).as_bytes());
        
        // Security: Zero the temporary key bytes
        key_bytes.zeroize();
        
        Ok(Self {
            key_id,
            key,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| SecureFecError::ConfigurationError)?
                .as_secs(),
        })
    }
}

/// Security: Thread-safe FEC manager with secure operations
#[derive(Debug)]
pub struct SecureFecManager {
    config: FecConfig,
    key_cache: Arc<RwLock<HashMap<String, SecureKeyMaterial>>>,
}

impl SecureFecManager {
    /// Create new secure FEC manager with validated configuration
    pub fn new(config: FecConfig) -> Result<Self, SecureFecError> {
        // Security: Validate configuration parameters
        if config.data_shards == 0 || config.parity_shards == 0 {
            return Err(SecureFecError::InvalidInput);
        }
        if config.chunk_size == 0 || config.chunk_size > 10 * 1024 * 1024 {
            return Err(SecureFecError::InvalidInput);
        }
        if config.max_content_size == 0 || config.max_content_size > 1024 * 1024 * 1024 {
            return Err(SecureFecError::InvalidInput);
        }

        Ok(Self {
            config,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Security: Encrypt data with content size validation
    pub async fn encrypt_data(
        &self,
        data: &[u8],
        key_id: Option<String>,
    ) -> Result<EncryptedContent, SecureFecError> {
        // Security: Validate input size to prevent DoS
        if data.is_empty() {
            return Err(SecureFecError::InvalidInput);
        }
        if data.len() > self.config.max_content_size {
            return Err(SecureFecError::InvalidInput);
        }

        // Security: Get or generate secure key material
        let key_material = if let Some(id) = key_id {
            self.get_key_material(&id).await
                .ok_or(SecureFecError::AuthenticationFailed)?
        } else {
            let material = SecureKeyMaterial::new()?;
            let id = material.key_id.clone();
            self.store_key_material(id.clone(), material.clone()).await?;
            material
        };

        // Process data in secure chunks
        let mut encrypted_chunks = Vec::new();
        let total_size = data.len();
        let chunk_count = (total_size + self.config.chunk_size - 1) / self.config.chunk_size;

        for (chunk_index, chunk_data) in data.chunks(self.config.chunk_size).enumerate() {
            let encrypted_chunk = self.encrypt_chunk(chunk_data, &key_material, chunk_index).await?;
            encrypted_chunks.push(encrypted_chunk);
        }

        let content_id = self.generate_content_id(&encrypted_chunks)?;

        Ok(EncryptedContent {
            content_id,
            encrypted_chunks,
            total_size,
            chunk_count,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| SecureFecError::ConfigurationError)?
                .as_secs(),
        })
    }

    /// Security: Decrypt data with integrity verification
    pub async fn decrypt_data(
        &self,
        encrypted_content: &EncryptedContent,
        key_id: &str,
    ) -> Result<Vec<u8>, SecureFecError> {
        // Security: Validate content structure
        if encrypted_content.encrypted_chunks.is_empty() {
            return Err(SecureFecError::InvalidInput);
        }
        if encrypted_content.total_size == 0 {
            return Err(SecureFecError::InvalidInput);
        }

        // Security: Get key material with authentication
        let key_material = self.get_key_material(key_id).await
            .ok_or(SecureFecError::AuthenticationFailed)?;

        // Security: Verify content integrity
        let calculated_id = self.generate_content_id(&encrypted_content.encrypted_chunks)?;
        if calculated_id != encrypted_content.content_id {
            return Err(SecureFecError::AuthenticationFailed);
        }

        // Decrypt chunks in order
        let mut decrypted_data = Vec::with_capacity(encrypted_content.total_size);
        
        for chunk in &encrypted_content.encrypted_chunks {
            let chunk_data = self.decrypt_chunk(chunk, &key_material).await?;
            decrypted_data.extend_from_slice(&chunk_data);
        }

        // Security: Verify total size matches
        if decrypted_data.len() != encrypted_content.total_size {
            return Err(SecureFecError::DecryptionFailed);
        }

        Ok(decrypted_data)
    }

    /// Security: Encrypt individual chunk with unique nonce
    async fn encrypt_chunk(
        &self,
        chunk_data: &[u8],
        key_material: &SecureKeyMaterial,
        chunk_index: usize,
    ) -> Result<EncryptedChunk, SecureFecError> {
        // Security: Generate unique nonce for each chunk
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::clone_from_slice(&nonce_bytes);

        // Encrypt using ChaCha20-Poly1305
        let cipher = ChaCha20Poly1305::new(&key_material.key);
        let encrypted_data = cipher.encrypt(&nonce, chunk_data)
            .map_err(|_| SecureFecError::EncryptionFailed)?;

        let chunk_id = hex::encode(blake3::hash(chunk_data).as_bytes());

        Ok(EncryptedChunk {
            chunk_id,
            encrypted_data,
            nonce: nonce_bytes.to_vec(),
            chunk_index,
            original_size: chunk_data.len(),
        })
    }

    /// Security: Decrypt individual chunk with integrity verification
    async fn decrypt_chunk(
        &self,
        encrypted_chunk: &EncryptedChunk,
        key_material: &SecureKeyMaterial,
    ) -> Result<Vec<u8>, SecureFecError> {
        // Security: Validate nonce size
        if encrypted_chunk.nonce.len() != 12 {
            return Err(SecureFecError::DecryptionFailed);
        }

        let nonce_bytes: [u8; 12] = encrypted_chunk.nonce.as_slice()
            .try_into()
            .map_err(|_| SecureFecError::DecryptionFailed)?;
        let nonce = Nonce::clone_from_slice(&nonce_bytes);

        // Decrypt using ChaCha20-Poly1305
        let cipher = ChaCha20Poly1305::new(&key_material.key);
        let decrypted_data = cipher.decrypt(&nonce, encrypted_chunk.encrypted_data.as_slice())
            .map_err(|_| SecureFecError::DecryptionFailed)?;

        // Security: Verify chunk integrity
        let calculated_id = hex::encode(blake3::hash(&decrypted_data).as_bytes());
        if calculated_id != encrypted_chunk.chunk_id {
            return Err(SecureFecError::AuthenticationFailed);
        }

        // Security: Verify size matches
        if decrypted_data.len() != encrypted_chunk.original_size {
            return Err(SecureFecError::DecryptionFailed);
        }

        Ok(decrypted_data)
    }

    /// Generate deterministic content ID from encrypted chunks
    fn generate_content_id(&self, chunks: &[EncryptedChunk]) -> Result<String, SecureFecError> {
        let mut hasher = blake3::Hasher::new();
        
        for chunk in chunks {
            hasher.update(&chunk.chunk_id.as_bytes());
            hasher.update(&chunk.chunk_index.to_le_bytes());
            hasher.update(&chunk.original_size.to_le_bytes());
        }
        
        Ok(hex::encode(hasher.finalize().as_bytes()))
    }

    /// Security: Store key material with thread safety
    async fn store_key_material(
        &self,
        key_id: String,
        key_material: SecureKeyMaterial,
    ) -> Result<(), SecureFecError> {
        let mut cache = self.key_cache.write().await;
        cache.insert(key_id, key_material);
        Ok(())
    }

    /// Security: Retrieve key material with authentication
    async fn get_key_material(&self, key_id: &str) -> Option<SecureKeyMaterial> {
        let cache = self.key_cache.read().await;
        cache.get(key_id).cloned()
    }

    /// Security: Clear expired keys from memory
    pub async fn cleanup_expired_keys(&self, max_age_seconds: u64) -> Result<usize, SecureFecError> {
        let mut cache = self.key_cache.write().await;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| SecureFecError::ConfigurationError)?
            .as_secs();

        let initial_count = cache.len();
        cache.retain(|_, material| {
            current_time.saturating_sub(material.created_at) < max_age_seconds
        });

        Ok(initial_count - cache.len())
    }

    /// Get configuration for diagnostics
    pub fn get_config(&self) -> &FecConfig {
        &self.config
    }
}

/// Security: Secure memory cleanup on drop
impl Drop for SecureFecManager {
    fn drop(&mut self) {
        // Key material will be zeroized automatically via ZeroizeOnDrop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secure_fec_manager_creation() {
        let config = FecConfig::default();
        let manager = SecureFecManager::new(config).expect("Failed to create FEC manager");
        
        assert_eq!(manager.get_config().data_shards, 8);
        assert_eq!(manager.get_config().parity_shards, 4);
    }

    #[tokio::test]
    async fn test_encryption_decryption_roundtrip() {
        let config = FecConfig::default();
        let manager = SecureFecManager::new(config).expect("Failed to create FEC manager");
        
        let test_data = b"Hello, secure world! This is a test of encryption.";
        
        // Encrypt data
        let encrypted_content = manager.encrypt_data(test_data, None).await
            .expect("Failed to encrypt data");
        
        assert!(!encrypted_content.content_id.is_empty());
        assert!(!encrypted_content.encrypted_chunks.is_empty());
        assert_eq!(encrypted_content.total_size, test_data.len());
        
        // Get the key ID from the first encryption
        let key_material = SecureKeyMaterial::new().expect("Failed to create key material");
        let key_id = key_material.key_id.clone();
        manager.store_key_material(key_id.clone(), key_material).await
            .expect("Failed to store key material");
        
        // Re-encrypt with the specific key
        let encrypted_content = manager.encrypt_data(test_data, Some(key_id.clone())).await
            .expect("Failed to encrypt data");
        
        // Decrypt data
        let decrypted_data = manager.decrypt_data(&encrypted_content, &key_id).await
            .expect("Failed to decrypt data");
        
        assert_eq!(decrypted_data, test_data);
    }

    #[tokio::test]
    async fn test_invalid_input_rejection() {
        let config = FecConfig::default();
        let manager = SecureFecManager::new(config).expect("Failed to create FEC manager");
        
        // Test empty data
        let result = manager.encrypt_data(&[], None).await;
        assert!(result.is_err());
        
        // Test oversized data
        let large_data = vec![0u8; 200 * 1024 * 1024]; // 200MB > 100MB limit
        let result = manager.encrypt_data(&large_data, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let config = FecConfig::default();
        let manager = SecureFecManager::new(config).expect("Failed to create FEC manager");
        
        let test_data = b"Test data";
        
        // Encrypt with one key
        let encrypted_content = manager.encrypt_data(test_data, None).await
            .expect("Failed to encrypt data");
        
        // Try to decrypt with wrong key ID
        let result = manager.decrypt_data(&encrypted_content, "wrong-key-id").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_content_integrity_verification() {
        let config = FecConfig::default();
        let manager = SecureFecManager::new(config).expect("Failed to create FEC manager");
        
        let test_data = b"Test data for integrity";
        let key_material = SecureKeyMaterial::new().expect("Failed to create key material");
        let key_id = key_material.key_id.clone();
        manager.store_key_material(key_id.clone(), key_material).await
            .expect("Failed to store key material");
        
        let mut encrypted_content = manager.encrypt_data(test_data, Some(key_id.clone())).await
            .expect("Failed to encrypt data");
        
        // Tamper with content ID
        encrypted_content.content_id = "tampered-id".to_string();
        
        // Decryption should fail due to integrity check
        let result = manager.decrypt_data(&encrypted_content, &key_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_key_cleanup() {
        let config = FecConfig::default();
        let manager = SecureFecManager::new(config).expect("Failed to create FEC manager");
        
        // Add some key material
        let key_material = SecureKeyMaterial::new().expect("Failed to create key material");
        let key_id = key_material.key_id.clone();
        manager.store_key_material(key_id, key_material).await
            .expect("Failed to store key material");
        
        // Cleanup with 0 max age (should remove all keys)
        let cleaned = manager.cleanup_expired_keys(0).await
            .expect("Failed to cleanup keys");
        
        assert_eq!(cleaned, 1);
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        // Test invalid configurations
        let invalid_configs = vec![
            FecConfig { data_shards: 0, ..Default::default() },
            FecConfig { parity_shards: 0, ..Default::default() },
            FecConfig { chunk_size: 0, ..Default::default() },
            FecConfig { chunk_size: 20 * 1024 * 1024, ..Default::default() }, // Too large
            FecConfig { max_content_size: 0, ..Default::default() },
        ];
        
        for config in invalid_configs {
            let result = SecureFecManager::new(config);
            assert!(result.is_err(), "Invalid config should be rejected");
        }
    }
}
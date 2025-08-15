// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! Secure cross-platform storage for cryptographic keys
//! 
//! This module provides a unified interface for storing cryptographic keys
//! securely across different platforms using OS-native keychains and secure storage.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a securely stored key entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEntry {
    /// Unique identifier for this key
    pub key_id: String,
    /// The encrypted key data
    pub encrypted_data: Vec<u8>,
    /// Metadata about the key (non-sensitive)
    pub metadata: KeyMetadata,
    /// Timestamp when the key was stored
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Timestamp when the key was last accessed
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Metadata about stored keys (non-sensitive information)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Human-readable description
    pub description: String,
    /// Key type (e.g., "identity", "signing", "encryption")
    pub key_type: String,
    /// Algorithm used (e.g., "Ed25519", "AES-256")
    pub algorithm: String,
    /// Four-word address if applicable
    pub four_word_address: Option<String>,
}

/// Unified interface for secure key storage across platforms
#[async_trait::async_trait]
pub trait SecureStorage: Send + Sync {
    /// Store a key securely
    async fn store_key(&self, key_id: &str, key_data: &[u8], metadata: KeyMetadata) -> Result<()>;
    
    /// Retrieve a key by ID
    async fn retrieve_key(&self, key_id: &str) -> Result<Option<Vec<u8>>>;
    
    /// Delete a key by ID  
    async fn delete_key(&self, key_id: &str) -> Result<bool>;
    
    /// List all stored key IDs with their metadata
    async fn list_keys(&self) -> Result<HashMap<String, KeyMetadata>>;
    
    /// Check if a key exists
    async fn key_exists(&self, key_id: &str) -> Result<bool>;
    
    /// Update key metadata (without changing the key data)
    async fn update_metadata(&self, key_id: &str, metadata: KeyMetadata) -> Result<()>;
    
    /// Get storage statistics
    async fn get_storage_info(&self) -> Result<StorageInfo>;
}

/// Information about the storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Type of storage backend
    pub backend_type: String,
    /// Number of keys stored
    pub key_count: usize,
    /// Whether the backend is available
    pub is_available: bool,
    /// Last successful operation timestamp
    pub last_operation: Option<chrono::DateTime<chrono::Utc>>,
}

/// Factory for creating platform-appropriate storage backends
pub struct SecureStorageFactory;

impl SecureStorageFactory {
    /// Create the best available storage backend for the current platform
    pub async fn create_default() -> Result<Box<dyn SecureStorage>> {
        #[cfg(target_os = "macos")]
        {
            match MacOSKeychain::new().await {
                Ok(keychain) => return Ok(Box::new(keychain)),
                Err(e) => {
                    tracing::warn!("Failed to initialize macOS Keychain: {}. Falling back to encrypted file storage.", e);
                }
            }
        }
        
        #[cfg(target_os = "windows")]
        {
            match WindowsCredentialManager::new().await {
                Ok(cred_mgr) => return Ok(Box::new(cred_mgr)),
                Err(e) => {
                    tracing::warn!("Failed to initialize Windows Credential Manager: {}. Falling back to encrypted file storage.", e);
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            match LinuxSecretService::new().await {
                Ok(secret_svc) => return Ok(Box::new(secret_svc)),
                Err(e) => {
                    tracing::warn!("Failed to initialize Linux Secret Service: {}. Falling back to encrypted file storage.", e);
                }
            }
        }
        
        // Fallback to encrypted file storage for all platforms
        let file_storage = EncryptedFileStorage::new().await
            .context("Failed to initialize encrypted file storage")?;
        Ok(Box::new(file_storage))
    }
    
    /// Create a specific storage backend by name
    pub async fn create_by_name(backend_name: &str) -> Result<Box<dyn SecureStorage>> {
        match backend_name.to_lowercase().as_str() {
            #[cfg(target_os = "macos")]
            "macos_keychain" | "keychain" => {
                Ok(Box::new(MacOSKeychain::new().await?))
            }
            #[cfg(target_os = "windows")]
            "windows_credential_manager" | "credmgr" => {
                Ok(Box::new(WindowsCredentialManager::new().await?))
            }
            #[cfg(target_os = "linux")]
            "linux_secret_service" | "secretservice" => {
                Ok(Box::new(LinuxSecretService::new().await?))
            }
            "encrypted_file" | "file" => {
                Ok(Box::new(EncryptedFileStorage::new().await?))
            }
            _ => {
                anyhow::bail!("Unknown storage backend: {}", backend_name)
            }
        }
    }
}

// Platform-specific implementations

#[cfg(target_os = "macos")]
use super::macos_keychain::MacOSKeychain;

#[cfg(target_os = "windows")]
use super::windows_credential_manager::WindowsCredentialManager;

#[cfg(target_os = "linux")]
use super::linux_secret_service::LinuxSecretService;

use super::encrypted_file_storage::EncryptedFileStorage;

/// Utility functions for secure storage
pub mod utils {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    /// Generate a unique key ID based on content and timestamp
    pub fn generate_key_id(prefix: &str, content_hash: &[u8]) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let hash_short = hex::encode(&content_hash[..8]);
        format!("{}_{}_t{}", prefix, hash_short, timestamp)
    }
    
    /// Validate key ID format
    pub fn validate_key_id(key_id: &str) -> Result<()> {
        if key_id.is_empty() {
            anyhow::bail!("Key ID cannot be empty");
        }
        if key_id.len() > 255 {
            anyhow::bail!("Key ID too long (max 255 characters)");
        }
        if !key_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            anyhow::bail!("Key ID contains invalid characters");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_factory_creates_storage() {
        let storage = SecureStorageFactory::create_default().await.unwrap();
        let info = storage.get_storage_info().await.unwrap();
        assert!(!info.backend_type.is_empty());
    }
    
    #[test]
    fn test_key_id_validation() {
        assert!(utils::validate_key_id("valid_key_id").is_ok());
        assert!(utils::validate_key_id("valid-key-123").is_ok());
        assert!(utils::validate_key_id("").is_err());
        assert!(utils::validate_key_id("invalid key with spaces").is_err());
    }
}

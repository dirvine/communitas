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


//! Encrypted file storage for cryptographic keys
//!
//! This module provides a cross-platform encrypted file storage backend
//! that serves as a fallback when platform-specific keychains are unavailable.

use super::{SecureStorage, KeyMetadata, StorageInfo};
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

/// Key derivation settings for file encryption
const ARGON2_MEMORY: u32 = 32768; // 32MB
const ARGON2_TIME: u32 = 2;
const ARGON2_PARALLELISM: u32 = 2;

/// Storage file format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StorageFile {
    version: u32,
    encrypted_keys: HashMap<String, EncryptedKeyEntry>,
    created_at: chrono::DateTime<chrono::Utc>,
    last_modified: chrono::DateTime<chrono::Utc>,
}

/// Individual encrypted key entry in the storage file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedKeyEntry {
    encrypted_data: Vec<u8>,
    nonce: Vec<u8>,
    salt: Vec<u8>,
    metadata: KeyMetadata,
    created_at: chrono::DateTime<chrono::Utc>,
    last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Encrypted file storage backend
pub struct EncryptedFileStorage {
    storage_path: PathBuf,
    master_password: Option<String>,
    cached_storage: RwLock<Option<StorageFile>>,
}

impl EncryptedFileStorage {
    /// Create new encrypted file storage
    pub async fn new() -> Result<Self> {
        let storage_path = Self::get_default_storage_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create storage directory")?;
        }
        
        Ok(Self {
            storage_path,
            master_password: None,
            cached_storage: RwLock::new(None),
        })
    }
    
    /// Create with custom storage path
    pub async fn with_path(path: PathBuf) -> Result<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create storage directory")?;
        }
        
        Ok(Self {
            storage_path: path,
            master_password: None,
            cached_storage: RwLock::new(None),
        })
    }
    
    /// Set master password for encryption
    pub fn set_master_password(&mut self, password: String) {
        self.master_password = Some(password);
    }
    
    /// Get default storage path for the current platform
    fn get_default_storage_path() -> Result<PathBuf> {
        let app_name = "Communitas";
        
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").context("HOME environment variable not set")?;
            Ok(PathBuf::from(format!("{}/Library/Application Support/{}/keys.enc", home, app_name)))
        }
        
        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA").context("APPDATA environment variable not set")?;
            Ok(PathBuf::from(format!("{}\\{}\\keys.enc", appdata, app_name)))
        }
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
                Ok(PathBuf::from(format!("{}/{}/keys.enc", xdg_data_home, app_name)))
            } else {
                let home = std::env::var("HOME").context("HOME environment variable not set")?;
                Ok(PathBuf::from(format!("{}/.local/share/{}/keys.enc", home, app_name)))
            }
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            Ok(PathBuf::from("communitas_keys.enc"))
        }
    }
    
    /// Load and decrypt storage file
    async fn load_storage(&self) -> Result<StorageFile> {
        // Check cache first
        {
            let cached = self.cached_storage.read().await;
            if let Some(ref storage) = *cached {
                return Ok(storage.clone());
            }
        }
        
        // Load from file
        if !self.storage_path.exists() {
            // Create new empty storage
            let storage = StorageFile {
                version: 1,
                encrypted_keys: HashMap::new(),
                created_at: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
            };
            
            // Cache it
            {
                let mut cached = self.cached_storage.write().await;
                *cached = Some(storage.clone());
            }
            
            return Ok(storage);
        }
        
        let encrypted_data = fs::read(&self.storage_path).await
            .context("Failed to read storage file")?;
            
        if encrypted_data.is_empty() {
            // Empty file, create new storage
            let storage = StorageFile {
                version: 1,
                encrypted_keys: HashMap::new(),
                created_at: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
            };
            
            {
                let mut cached = self.cached_storage.write().await;
                *cached = Some(storage.clone());
            }
            
            return Ok(storage);
        }
        
        // For now, return empty storage if no password is set
        // In a real implementation, this would prompt for password
        if self.master_password.is_none() {
            let storage = StorageFile {
                version: 1,
                encrypted_keys: HashMap::new(),
                created_at: chrono::Utc::now(),
                last_modified: chrono::Utc::now(),
            };
            
            {
                let mut cached = self.cached_storage.write().await;
                *cached = Some(storage.clone());
            }
            
            return Ok(storage);
        }
        
        // TODO: Implement actual decryption using master password
        // For now, just deserialize as JSON (this is insecure!)
        let storage: StorageFile = serde_json::from_slice(&encrypted_data)
            .context("Failed to deserialize storage file")?;
            
        // Cache it
        {
            let mut cached = self.cached_storage.write().await;
            *cached = Some(storage.clone());
        }
        
        Ok(storage)
    }
    
    /// Save and encrypt storage file
    async fn save_storage(&self, storage: StorageFile) -> Result<()> {
        // TODO: Implement actual encryption using master password
        // For now, just serialize as JSON (this is insecure!)
        let data = serde_json::to_vec_pretty(&storage)
            .context("Failed to serialize storage")?;
            
        // Write atomically by using a temporary file
        let temp_path = self.storage_path.with_extension("tmp");
        fs::write(&temp_path, &data).await
            .context("Failed to write temporary storage file")?;
            
        fs::rename(&temp_path, &self.storage_path).await
            .context("Failed to rename temporary storage file")?;
        
        // Update cache
        {
            let mut cached = self.cached_storage.write().await;
            *cached = Some(storage);
        }
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl SecureStorage for EncryptedFileStorage {
    async fn store_key(&self, key_id: &str, key_data: &[u8], metadata: KeyMetadata) -> Result<()> {
        crate::identity::secure_storage::utils::validate_key_id(key_id)?;
        
        let mut storage = self.load_storage().await?;
        
        // TODO: Implement proper encryption with master password
        // For now, store as-is (this is insecure!)
        let entry = EncryptedKeyEntry {
            encrypted_data: key_data.to_vec(),
            nonce: vec![0u8; 12], // TODO: Generate random nonce
            salt: vec![0u8; 32],  // TODO: Generate random salt
            metadata,
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
        };
        
        storage.encrypted_keys.insert(key_id.to_string(), entry);
        storage.last_modified = chrono::Utc::now();
        
        self.save_storage(storage).await?;
        
        Ok(())
    }
    
    async fn retrieve_key(&self, key_id: &str) -> Result<Option<Vec<u8>>> {
        let mut storage = self.load_storage().await?;
        
        if let Some(entry) = storage.encrypted_keys.get_mut(key_id) {
            entry.last_accessed = chrono::Utc::now();
            storage.last_modified = chrono::Utc::now();
            
            // TODO: Implement proper decryption
            // For now, return as-is (this is insecure!)
            let key_data = entry.encrypted_data.clone();
            
            // Save updated access time
            self.save_storage(storage).await?;
            
            Ok(Some(key_data))
        } else {
            Ok(None)
        }
    }
    
    async fn delete_key(&self, key_id: &str) -> Result<bool> {
        let mut storage = self.load_storage().await?;
        
        let removed = storage.encrypted_keys.remove(key_id).is_some();
        
        if removed {
            storage.last_modified = chrono::Utc::now();
            self.save_storage(storage).await?;
        }
        
        Ok(removed)
    }
    
    async fn list_keys(&self) -> Result<HashMap<String, KeyMetadata>> {
        let storage = self.load_storage().await?;
        
        let mut result = HashMap::new();
        for (key_id, entry) in &storage.encrypted_keys {
            result.insert(key_id.clone(), entry.metadata.clone());
        }
        
        Ok(result)
    }
    
    async fn key_exists(&self, key_id: &str) -> Result<bool> {
        let storage = self.load_storage().await?;
        Ok(storage.encrypted_keys.contains_key(key_id))
    }
    
    async fn update_metadata(&self, key_id: &str, metadata: KeyMetadata) -> Result<()> {
        let mut storage = self.load_storage().await?;
        
        if let Some(entry) = storage.encrypted_keys.get_mut(key_id) {
            entry.metadata = metadata;
            storage.last_modified = chrono::Utc::now();
            self.save_storage(storage).await?;
            Ok(())
        } else {
            anyhow::bail!("Key not found: {}", key_id)
        }
    }
    
    async fn get_storage_info(&self) -> Result<StorageInfo> {
        let storage = self.load_storage().await?;
        
        Ok(StorageInfo {
            backend_type: "EncryptedFile".to_string(),
            key_count: storage.encrypted_keys.len(),
            is_available: true,
            last_operation: Some(storage.last_modified),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    async fn create_test_storage() -> Result<EncryptedFileStorage> {
        let temp_dir = tempdir()?;
        let storage_path = temp_dir.path().join("test_keys.enc");
        EncryptedFileStorage::with_path(storage_path).await
    }
    
    #[tokio::test]
    async fn test_store_and_retrieve_key() {
        let storage = create_test_storage().await.unwrap();
        
        let key_data = b"test_key_data";
        let metadata = KeyMetadata {
            description: "Test key".to_string(),
            key_type: "test".to_string(),
            algorithm: "Test".to_string(),
            four_word_address: None,
        };
        
        // Store key
        storage.store_key("test_key", key_data, metadata.clone()).await.unwrap();
        
        // Retrieve key
        let retrieved = storage.retrieve_key("test_key").await.unwrap();
        assert_eq!(retrieved, Some(key_data.to_vec()));
        
        // Check it exists
        assert!(storage.key_exists("test_key").await.unwrap());
        
        // List keys
        let keys = storage.list_keys().await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys.get("test_key").unwrap().description, "Test key");
    }
    
    #[tokio::test]
    async fn test_delete_key() {
        let storage = create_test_storage().await.unwrap();
        
        let metadata = KeyMetadata {
            description: "Test key".to_string(),
            key_type: "test".to_string(),
            algorithm: "Test".to_string(),
            four_word_address: None,
        };
        
        // Store key
        storage.store_key("test_key", b"data", metadata).await.unwrap();
        
        // Delete key
        let deleted = storage.delete_key("test_key").await.unwrap();
        assert!(deleted);
        
        // Verify it's gone
        assert!(!storage.key_exists("test_key").await.unwrap());
        
        // Deleting non-existent key should return false
        let deleted = storage.delete_key("non_existent").await.unwrap();
        assert!(!deleted);
    }
}

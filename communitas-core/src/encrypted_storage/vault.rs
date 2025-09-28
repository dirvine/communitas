//! Encrypted Vault Implementation
//!
//! Each vault represents a single four-word identity's encrypted storage.
//! Vaults support multiple data types: identity data, local files, cached content,
//! and collaborative data with Forward Error Correction.

use crate::encrypted_storage::{KeyManager, StorageConfig, fec_storage::FecStorage};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

/// Represents an encrypted vault for a four-word identity
pub struct EncryptedVault {
    pub four_words: String,
    pub display_name: String,
    metadata: VaultMetadata,
    encryption_key: Zeroizing<Vec<u8>>,
    data_store: RwLock<HashMap<String, EncryptedEntry>>,
    vault_path: PathBuf,
    key_manager: KeyManager,
    fec_storage: Option<FecStorage>,
}

/// Vault metadata stored unencrypted for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub version: u32,
    pub created_at: u64,
    pub last_accessed: u64,
    pub salt: Vec<u8>,
    pub pbkdf2_iterations: u32,
    pub total_size: u64,
    pub entry_count: usize,
    pub checksum: Vec<u8>, // BLAKE3 hash of vault contents
}

/// Individual encrypted entry in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedEntry {
    key: String,
    encrypted_data: Vec<u8>,
    metadata: EntryMetadata,
}

/// Metadata for each encrypted entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntryMetadata {
    created_at: u64,
    modified_at: u64,
    size: usize,
    content_type: ContentType,
    compression: Option<CompressionType>,
    fec_shards: Option<Vec<PathBuf>>, // FEC shard locations if used
}

/// Type of content stored
#[derive(Debug, Clone, Serialize, Deserialize)]
enum ContentType {
    Identity,        // Core identity data
    LocalFile,       // User files stored locally
    CachedContent,   // Cached collaborative content
    Configuration,   // App configuration
    Session,         // Session data
}

/// Compression type for stored data
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CompressionType {
    None,
    // Future: Zstd, Lz4 (when dependencies are added)
}

impl EncryptedVault {
    /// Create a new vault
    pub async fn create(
        four_words: String,
        display_name: String,
        encryption_key: Zeroizing<Vec<u8>>,
        salt: Vec<u8>,
        config: &StorageConfig,
    ) -> Result<Self> {
        let vault_path = config.vault_dir.join(&four_words);

        // Create vault directory
        fs::create_dir_all(&vault_path)
            .await
            .context("Failed to create vault directory")?;

        let metadata = VaultMetadata {
            version: 1,
            created_at: current_timestamp(),
            last_accessed: current_timestamp(),
            salt,
            pbkdf2_iterations: config.pbkdf2_iterations,
            total_size: 0,
            entry_count: 0,
            checksum: vec![],
        };

        // Save metadata
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = serde_json::to_vec(&metadata)?;
        fs::write(&metadata_path, metadata_json).await?;

        let key_manager = KeyManager::new(config.pbkdf2_iterations, config.use_keyring).await?;

        let fec_storage = if config.enable_fec {
            Some(FecStorage::new(&vault_path, config.fec_redundancy).await?)
        } else {
            None
        };

        Ok(Self {
            four_words,
            display_name,
            metadata,
            encryption_key,
            data_store: RwLock::new(HashMap::new()),
            vault_path,
            key_manager,
            fec_storage,
        })
    }

    /// Load an existing vault
    pub async fn load(
        four_words: &str,
        password: &str,
        config: &StorageConfig,
    ) -> Result<Self> {
        let vault_path = config.vault_dir.join(four_words);

        // Load metadata
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = fs::read(&metadata_path)
            .await
            .context("Failed to read vault metadata")?;
        let metadata: VaultMetadata = serde_json::from_slice(&metadata_json)?;

        // Derive encryption key
        let key_manager = KeyManager::new(metadata.pbkdf2_iterations, config.use_keyring).await?;
        let encryption_key = key_manager.derive_key(password, &metadata.salt).await?;

        // Load encrypted entries index
        let index_path = vault_path.join("index.enc");
        let data_store = if index_path.exists() {
            let encrypted_index = fs::read(&index_path).await?;
            let decrypted_index = key_manager.decrypt(&encryption_key, &encrypted_index)?;
            let entries: HashMap<String, EncryptedEntry> = serde_json::from_slice(&decrypted_index)?;
            RwLock::new(entries)
        } else {
            RwLock::new(HashMap::new())
        };

        // Load display name from encrypted identity data
        let display_name = Self::load_display_name(&vault_path, &encryption_key, &key_manager).await
            .unwrap_or_else(|_| "Unknown".to_string());

        let fec_storage = if config.enable_fec {
            Some(FecStorage::new(&vault_path, config.fec_redundancy).await?)
        } else {
            None
        };

        Ok(Self {
            four_words: four_words.to_string(),
            display_name,
            metadata,
            encryption_key,
            data_store,
            vault_path,
            key_manager,
            fec_storage,
        })
    }

    /// Store encrypted data
    pub async fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        self.store_internal(key, data, ContentType::LocalFile, None).await
    }

    /// Store with Forward Error Correction
    pub async fn store_with_fec(&self, key: &str, data: &[u8], redundancy: f32) -> Result<()> {
        // Encrypt data first
        let encrypted = self.key_manager.encrypt(&self.encryption_key, data)?;

        // Store with FEC if available
        if let Some(fec) = &self.fec_storage {
            let shard_paths = fec.store_with_fec(key, &encrypted, redundancy).await?;

            // Update entry metadata
            let entry = EncryptedEntry {
                key: key.to_string(),
                encrypted_data: vec![], // Data is in FEC shards
                metadata: EntryMetadata {
                    created_at: current_timestamp(),
                    modified_at: current_timestamp(),
                    size: data.len(),
                    content_type: ContentType::LocalFile,
                    compression: None,
                    fec_shards: Some(shard_paths),
                },
            };

            let mut store = self.data_store.write().await;
            store.insert(key.to_string(), entry);

            self.save_index().await?;
        } else {
            // Fall back to normal storage
            self.store(key, data).await?;
        }

        Ok(())
    }

    /// Retrieve encrypted data
    pub async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
        let store = self.data_store.read().await;
        let entry = store.get(key)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key))?;

        // Check if data is in FEC shards
        if let Some(shard_paths) = &entry.metadata.fec_shards {
            if let Some(fec) = &self.fec_storage {
                let encrypted = fec.retrieve_from_fec(shard_paths).await?;
                let decrypted = self.key_manager.decrypt(&self.encryption_key, &encrypted)?;
                return Ok(decrypted.to_vec());
            }
        }

        // Regular encrypted data
        let decrypted = self.key_manager.decrypt(&self.encryption_key, &entry.encrypted_data)?;

        // Decompress if needed
        let data = match entry.metadata.compression {
            Some(CompressionType::None) => {
                decrypted.to_vec()
            }
            _ => decrypted.to_vec(),
        };

        Ok(data)
    }

    /// Delete an entry
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut store = self.data_store.write().await;

        if let Some(entry) = store.remove(key) {
            // Delete FEC shards if present
            if let Some(shard_paths) = entry.metadata.fec_shards {
                for path in shard_paths {
                    let _ = fs::remove_file(path).await;
                }
            }

            // Delete regular file
            let file_path = self.vault_path.join(format!("{}.enc", key));
            let _ = fs::remove_file(file_path).await;

            self.save_index().await?;
        }

        Ok(())
    }

    /// List all keys in the vault
    pub async fn list_keys(&self) -> Vec<String> {
        let store = self.data_store.read().await;
        store.keys().cloned().collect()
    }

    /// Get vault statistics
    pub async fn get_stats(&self) -> VaultStats {
        let store = self.data_store.read().await;

        let mut total_size = 0;
        let mut file_count = 0;
        let mut fec_count = 0;

        for entry in store.values() {
            total_size += entry.metadata.size;
            if entry.metadata.fec_shards.is_some() {
                fec_count += 1;
            } else {
                file_count += 1;
            }
        }

        VaultStats {
            total_size,
            entry_count: store.len(),
            file_count,
            fec_count,
            created_at: self.metadata.created_at,
            last_accessed: self.metadata.last_accessed,
        }
    }

    /// Export vault for backup
    pub async fn export(&self, include_data: bool) -> Result<Vec<u8>> {
        let mut export = VaultExport {
            four_words: self.four_words.clone(),
            display_name: self.display_name.clone(),
            metadata: self.metadata.clone(),
            entries: if include_data {
                let store = self.data_store.read().await;
                Some(store.clone().into_iter().collect())
            } else {
                None
            },
        };

        // Update checksum
        export.metadata.checksum = self.calculate_checksum().await?;

        Ok(serde_json::to_vec(&export)?)
    }

    /// Import vault from backup
    pub async fn import(backup_data: &[u8], password: &str, config: &StorageConfig) -> Result<Self> {
        let export: VaultExport = serde_json::from_slice(backup_data)?;

        // Create vault with imported metadata
        let key_manager = KeyManager::new(export.metadata.pbkdf2_iterations, config.use_keyring).await?;
        let encryption_key = key_manager.derive_key(password, &export.metadata.salt).await?;

        let vault_path = config.vault_dir.join(&export.four_words);
        fs::create_dir_all(&vault_path).await?;

        // Save metadata
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = serde_json::to_vec(&export.metadata)?;
        fs::write(&metadata_path, metadata_json).await?;

        let fec_storage = if config.enable_fec {
            Some(FecStorage::new(&vault_path, config.fec_redundancy).await?)
        } else {
            None
        };

        let vault = Self {
            four_words: export.four_words,
            display_name: export.display_name,
            metadata: export.metadata,
            encryption_key,
            data_store: RwLock::new(export.entries.unwrap_or_default().into_iter().collect()),
            vault_path,
            key_manager,
            fec_storage,
        };

        vault.save_index().await?;

        Ok(vault)
    }

    // Private helper methods

    async fn store_internal(
        &self,
        key: &str,
        data: &[u8],
        content_type: ContentType,
        compression: Option<CompressionType>,
    ) -> Result<()> {
        // Compress if requested (currently no compression)
        let compressed_data = data.to_vec();

        // Encrypt
        let encrypted = self.key_manager.encrypt(&self.encryption_key, &compressed_data)?;

        // Create entry
        let entry = EncryptedEntry {
            key: key.to_string(),
            encrypted_data: encrypted.clone(),
            metadata: EntryMetadata {
                created_at: current_timestamp(),
                modified_at: current_timestamp(),
                size: data.len(),
                content_type,
                compression,
                fec_shards: None,
            },
        };

        // Store in memory
        let mut store = self.data_store.write().await;
        store.insert(key.to_string(), entry);

        // Store on disk
        let file_path = self.vault_path.join(format!("{}.enc", key));
        fs::write(file_path, encrypted).await?;

        // Update index
        self.save_index().await?;

        Ok(())
    }

    async fn save_index(&self) -> Result<()> {
        let store = self.data_store.read().await;
        let index_json = serde_json::to_vec(&*store)?;
        let encrypted_index = self.key_manager.encrypt(&self.encryption_key, &index_json)?;

        let index_path = self.vault_path.join("index.enc");
        fs::write(index_path, encrypted_index).await?;

        Ok(())
    }

    async fn calculate_checksum(&self) -> Result<Vec<u8>> {
        let store = self.data_store.read().await;
        let mut hasher = blake3::Hasher::new();

        for (key, entry) in store.iter() {
            hasher.update(key.as_bytes());
            hasher.update(&entry.encrypted_data);
        }

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    async fn load_display_name(
        vault_path: &PathBuf,
        encryption_key: &[u8],
        key_manager: &KeyManager,
    ) -> Result<String> {
        let identity_path = vault_path.join("identity.enc");
        if identity_path.exists() {
            let encrypted = fs::read(identity_path).await?;
            let decrypted = key_manager.decrypt(encryption_key, &encrypted)?;
            let identity: IdentityData = serde_json::from_slice(&decrypted)?;
            Ok(identity.display_name)
        } else {
            Err(anyhow::anyhow!("Identity data not found"))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultExport {
    four_words: String,
    display_name: String,
    metadata: VaultMetadata,
    entries: Option<Vec<(String, EncryptedEntry)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentityData {
    display_name: String,
    created_at: u64,
}

#[derive(Debug, Clone)]
pub struct VaultStats {
    pub total_size: usize,
    pub entry_count: usize,
    pub file_count: usize,
    pub fec_count: usize,
    pub created_at: u64,
    pub last_accessed: u64,
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_vault_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            vault_dir: temp_dir.path().to_path_buf(),
            use_keyring: false,
            ..Default::default()
        };

        let key = Zeroizing::new(vec![0u8; 32]);
        let salt = vec![1u8; 32];

        let vault = EncryptedVault::create(
            "test-vault".to_string(),
            "Test User".to_string(),
            key,
            salt,
            &config,
        ).await.unwrap();

        // Store and retrieve data
        let test_data = b"Test data for vault";
        vault.store("test_key", test_data).await.unwrap();

        let retrieved = vault.retrieve("test_key").await.unwrap();
        assert_eq!(retrieved, test_data);

        // List keys
        let keys = vault.list_keys().await;
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"test_key".to_string()));

        // Delete entry
        vault.delete("test_key").await.unwrap();
        assert!(vault.retrieve("test_key").await.is_err());
    }
}
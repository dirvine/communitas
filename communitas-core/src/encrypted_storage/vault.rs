// SPDX-License-Identifier: MIT OR Apache-2.0

//! Local Vault Implementation
//!
//! Each vault represents a single four-word identity's local storage.
//! Vaults support multiple data types: identity data, local files, cached content,
//! and collaborative data with Forward Error Correction.

use crate::encrypted_storage::{StorageConfig, fec_storage::FecStorage};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;

const INDEX_FILE: &str = "index.json";
const IDENTITY_FILE: &str = "identity.json";

/// Represents a local vault for a four-word identity
pub struct EncryptedVault {
    pub four_words: String,
    pub display_name: String,
    metadata: VaultMetadata,
    data_store: RwLock<HashMap<String, VaultEntry>>,
    vault_path: PathBuf,
    fec_storage: Option<FecStorage>,
}

/// Vault metadata stored unencrypted for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub version: u32,
    pub created_at: u64,
    pub last_accessed: u64,
    #[serde(default)]
    pub salt: Vec<u8>,
    #[serde(default)]
    pub pbkdf2_iterations: u32,
    pub total_size: u64,
    pub entry_count: usize,
    pub checksum: Vec<u8>, // BLAKE3 hash of vault contents
    /// Display name stored unencrypted for vault listing without needing to decrypt
    #[serde(default)]
    pub display_name: String,
    /// Whether the vault contents are encrypted (legacy). Defaults to true for backward compatibility.
    #[serde(default = "default_metadata_encrypted")]
    pub encrypted: bool,
}

fn default_metadata_encrypted() -> bool {
    true
}

/// Individual entry in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultEntry {
    key: String,
    data: Vec<u8>,
    metadata: EntryMetadata,
}

/// Metadata for each entry
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
    Identity,      // Core identity data
    LocalFile,     // User files stored locally
    CachedContent, // Cached collaborative content
    Configuration, // App configuration
    Session,       // Session data
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
        config: &StorageConfig,
    ) -> Result<Self> {
        let vault_path = config.vault_dir.join(&four_words);

        // Create vault directory
        fs::create_dir_all(&vault_path)
            .await
            .context("Failed to create vault directory")?;

        let metadata = VaultMetadata {
            version: 2,
            created_at: current_timestamp(),
            last_accessed: current_timestamp(),
            salt: Vec::new(),
            pbkdf2_iterations: 0,
            total_size: 0,
            entry_count: 0,
            checksum: vec![],
            display_name: display_name.clone(),
            encrypted: false,
        };

        // Save metadata
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = serde_json::to_vec(&metadata)?;
        fs::write(&metadata_path, metadata_json).await?;

        // Store identity data (display name) for loading
        let identity_data = IdentityData {
            display_name: display_name.clone(),
            created_at: current_timestamp(),
        };
        let identity_json = serde_json::to_vec(&identity_data)?;
        let identity_path = vault_path.join(IDENTITY_FILE);
        fs::write(&identity_path, identity_json).await?;

        let fec_storage = if config.enable_fec {
            Some(FecStorage::new(&vault_path, config.fec_redundancy).await?)
        } else {
            None
        };

        Ok(Self {
            four_words,
            display_name,
            metadata,
            data_store: RwLock::new(HashMap::new()),
            vault_path,
            fec_storage,
        })
    }

    /// Load an existing vault
    pub async fn load(four_words: &str, config: &StorageConfig) -> Result<Self> {
        let vault_path = config.vault_dir.join(four_words);

        // Load metadata
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = fs::read(&metadata_path)
            .await
            .context("Failed to read vault metadata")?;
        let metadata: VaultMetadata = serde_json::from_slice(&metadata_json)?;

        if metadata.encrypted {
            return Err(anyhow::anyhow!(
                "Encrypted vault detected; migrate or recreate vault to continue"
            ));
        }

        let index_path = vault_path.join(INDEX_FILE);
        let data_store = if index_path.exists() {
            let index_bytes = fs::read(&index_path).await?;
            let entries: HashMap<String, VaultEntry> =
                serde_json::from_slice(&index_bytes).context("Invalid vault index")?;
            RwLock::new(entries)
        } else if vault_path.join("index.enc").exists() {
            return Err(anyhow::anyhow!(
                "Encrypted vault index detected; migrate or recreate vault to continue"
            ));
        } else {
            RwLock::new(HashMap::new())
        };

        let display_name = Self::load_display_name(&vault_path)
            .await
            .unwrap_or_else(|_| {
                if !metadata.display_name.is_empty() {
                    metadata.display_name.clone()
                } else {
                    "Unknown".to_string()
                }
            });

        let fec_storage = if config.enable_fec {
            Some(FecStorage::new(&vault_path, config.fec_redundancy).await?)
        } else {
            None
        };

        Ok(Self {
            four_words: four_words.to_string(),
            display_name,
            metadata,
            data_store,
            vault_path,
            fec_storage,
        })
    }

    /// Store data
    pub async fn store(&self, key: &str, data: &[u8]) -> Result<()> {
        self.store_internal(key, data, ContentType::LocalFile, None)
            .await
    }

    /// Store with Forward Error Correction
    pub async fn store_with_fec(&self, key: &str, data: &[u8], redundancy: f32) -> Result<()> {
        // Store with FEC if available
        if let Some(fec) = &self.fec_storage {
            let shard_paths = fec.store_with_fec(key, data, redundancy).await?;

            // Update entry metadata
            let entry = VaultEntry {
                key: key.to_string(),
                data: vec![], // Data is in FEC shards
                metadata: EntryMetadata {
                    created_at: current_timestamp(),
                    modified_at: current_timestamp(),
                    size: data.len(),
                    content_type: ContentType::LocalFile,
                    compression: None,
                    fec_shards: Some(shard_paths),
                },
            };

            {
                let mut store = self.data_store.write().await;
                store.insert(key.to_string(), entry);
            }

            self.save_index().await?;
        } else {
            // Fall back to normal storage
            self.store(key, data).await?;
        }

        Ok(())
    }

    /// Retrieve data
    pub async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
        let store = self.data_store.read().await;
        let entry = store
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key))?;

        // Check if data is in FEC shards
        if let Some(shard_paths) = &entry.metadata.fec_shards
            && let Some(fec) = &self.fec_storage
        {
            let data = fec.retrieve_from_fec(shard_paths).await?;
            return Ok(data);
        }

        // Regular data
        let data = match entry.metadata.compression {
            Some(CompressionType::None) | None => entry.data.clone(),
        };

        Ok(data)
    }

    /// Delete an entry
    pub async fn delete(&self, key: &str) -> Result<()> {
        let entry = {
            let mut store = self.data_store.write().await;
            store.remove(key)
        };

        if let Some(entry) = entry {
            // Delete FEC shards if present
            if let Some(shard_paths) = entry.metadata.fec_shards {
                for path in shard_paths {
                    let _ = fs::remove_file(path).await;
                }
            }

            // Delete regular file
            let file_path = self.vault_path.join(format!("{}.bin", key));
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
    pub async fn import(backup_data: &[u8], config: &StorageConfig) -> Result<Self> {
        let mut export: VaultExport = serde_json::from_slice(backup_data)?;

        export.metadata.encrypted = false;
        export.metadata.pbkdf2_iterations = 0;
        export.metadata.salt = Vec::new();

        let vault_path = config.vault_dir.join(&export.four_words);
        fs::create_dir_all(&vault_path).await?;

        // Save metadata
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = serde_json::to_vec(&export.metadata)?;
        fs::write(&metadata_path, metadata_json).await?;

        // Save identity data (display name)
        let identity_data = IdentityData {
            display_name: export.display_name.clone(),
            created_at: current_timestamp(),
        };
        let identity_json = serde_json::to_vec(&identity_data)?;
        let identity_path = vault_path.join(IDENTITY_FILE);
        fs::write(&identity_path, identity_json).await?;

        let fec_storage = if config.enable_fec {
            Some(FecStorage::new(&vault_path, config.fec_redundancy).await?)
        } else {
            None
        };

        let vault = Self {
            four_words: export.four_words,
            display_name: export.display_name,
            metadata: export.metadata,
            data_store: RwLock::new(export.entries.unwrap_or_default().into_iter().collect()),
            vault_path: vault_path.clone(),
            fec_storage,
        };

        vault.save_index().await?;
        vault.persist_entries().await?;

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

        // Create entry
        let entry = VaultEntry {
            key: key.to_string(),
            data: compressed_data.clone(),
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
        {
            let mut store = self.data_store.write().await;
            store.insert(key.to_string(), entry);
        }

        // Store on disk
        let file_path = self.vault_path.join(format!("{}.bin", key));
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.ok();
        }
        fs::write(file_path, compressed_data).await?;

        // Update index
        self.save_index().await?;

        Ok(())
    }

    async fn save_index(&self) -> Result<()> {
        let store = self.data_store.read().await;
        let index_json = serde_json::to_vec(&*store)?;

        let index_path = self.vault_path.join(INDEX_FILE);
        fs::write(index_path, index_json).await?;

        Ok(())
    }

    async fn calculate_checksum(&self) -> Result<Vec<u8>> {
        let store = self.data_store.read().await;
        let mut hasher = blake3::Hasher::new();

        for (key, entry) in store.iter() {
            hasher.update(key.as_bytes());
            hasher.update(&entry.data);
        }

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    async fn load_display_name(vault_path: &Path) -> Result<String> {
        let identity_path = vault_path.join(IDENTITY_FILE);
        if identity_path.exists() {
            let raw = fs::read(identity_path).await?;
            let identity: IdentityData = serde_json::from_slice(&raw)?;
            Ok(identity.display_name)
        } else if vault_path.join("identity.enc").exists() {
            Err(anyhow::anyhow!(
                "Encrypted identity data detected; migrate or recreate vault to continue"
            ))
        } else {
            Err(anyhow::anyhow!("Identity data not found"))
        }
    }

    async fn persist_entries(&self) -> Result<()> {
        let store = self.data_store.read().await;
        for entry in store.values() {
            if entry.metadata.fec_shards.is_some() || entry.data.is_empty() {
                continue;
            }
            let file_path = self.vault_path.join(format!("{}.bin", entry.key));
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await.ok();
            }
            fs::write(file_path, &entry.data).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultExport {
    four_words: String,
    display_name: String,
    metadata: VaultMetadata,
    entries: Option<Vec<(String, VaultEntry)>>,
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
        .unwrap_or_default()
        .as_secs()
}

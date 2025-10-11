// Copyright 2024 Saorsa Labs
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Local storage management with DHT integration

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use super::reed_solomon_manager::Shard;

/// Local storage directory structure
#[derive(Debug, Clone)]
pub struct LocalStorageStructure {
    pub root: PathBuf,
    pub personal: PathBuf,     // Personal data (local copy)
    pub group_shards: PathBuf, // Reed Solomon shards from groups
    pub dht_cache: PathBuf,    // DHT data cached locally
    pub metadata: PathBuf,     // Storage metadata and indices
    pub temp: PathBuf,         // Temporary files
    pub web: PathBuf,          // Web directory for markdown content (home.md as index)
}

impl LocalStorageStructure {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            personal: root.join("personal"),
            group_shards: root.join("group_shards"),
            dht_cache: root.join("dht_cache"),
            metadata: root.join("metadata"),
            temp: root.join("temp"),
            web: root.join("web"),
            root,
        }
    }

    pub async fn create_directories(&self) -> Result<()> {
        let directories = [
            &self.root,
            &self.personal,
            &self.group_shards,
            &self.dht_cache,
            &self.metadata,
            &self.temp,
            &self.web,
        ];

        for dir in directories.iter() {
            tokio::fs::create_dir_all(dir)
                .await
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
        }

        // Initialize home.md in the web directory if it doesn't exist
        self.initialize_home_markdown().await?;

        debug!(
            "Created local storage directory structure at {}",
            self.root.display()
        );
        Ok(())
    }

    /// Initialize the home.md file in the web directory
    pub async fn initialize_home_markdown(&self) -> Result<()> {
        let home_file = self.web.join("home.md");

        // Only create if it doesn't exist (don't overwrite existing content)
        if !home_file.exists() {
            let default_content = r#"# Welcome to Your Digital Space

This is your personal web space on the markdown internet. 

## About This Space
- Your four-word address serves as both your identity and web address
- Write in markdown, link to other four-word addresses
- Content is stored distributedly across your trusted peers
- Everything is cryptographically signed and encrypted

## Getting Started
- Edit this file using the markdown editor
- Add images, videos, and other media to your web directory
- Link to other identities using their four-word addresses
- Create subdirectories to organize your content

## Your Identity
This space belongs to your unique four-word identity. All content here is:
- Cryptographically signed by your private key
- Distributed across trusted peers using Reed-Solomon encoding
- Accessible to others through your four-word address
- Self-hosted without requiring traditional servers

Start editing to make this space your own!
"#;

            tokio::fs::write(&home_file, default_content)
                .await
                .with_context(|| {
                    format!("Failed to create home.md file: {}", home_file.display())
                })?;

            debug!("Created default home.md file at {}", home_file.display());
        }

        Ok(())
    }
}

/// Metadata for stored items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub item_id: String,
    pub item_type: StorageItemType,
    pub file_path: PathBuf,
    pub size: usize,
    pub hash: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub encryption_info: Option<EncryptionInfo>,
    pub compression_info: Option<CompressionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageItemType {
    PersonalData {
        user_id: String,
    },
    GroupShard {
        group_id: String,
        shard_index: usize,
    },
    DHTData {
        key: String,
        owner: String,
    },
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInfo {
    pub algorithm: String,
    pub key_id: String,
    pub nonce_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    pub algorithm: String,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
}

/// Local storage manager with DHT integration
#[derive(Debug)]
pub struct LocalStorageManager {
    structure: LocalStorageStructure,
    metadata_index: RwLock<HashMap<String, StorageMetadata>>,
    usage_stats: RwLock<LocalStorageStats>,
    capacity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalStorageStats {
    pub personal_data_size: usize,
    pub group_shards_size: usize,
    pub dht_cache_size: usize,
    pub total_files: usize,
    pub last_cleanup: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct TestAllocation {
    pub personal_storage: usize,
    pub dht_backup_storage: usize,
    pub public_dht_storage: usize,
}

impl LocalStorageManager {
    pub async fn new<P: AsRef<Path>>(root: P, capacity: usize) -> Result<Self> {
        let structure = LocalStorageStructure::new(root);
        structure.create_directories().await?;

        let manager = Self {
            structure,
            metadata_index: RwLock::new(HashMap::new()),
            usage_stats: RwLock::new(LocalStorageStats {
                personal_data_size: 0,
                group_shards_size: 0,
                dht_cache_size: 0,
                total_files: 0,
                last_cleanup: chrono::Utc::now(),
            }),
            capacity,
        };

        // Load existing metadata index
        manager.load_metadata_index().await?;
        manager.calculate_usage_stats().await?;

        info!(
            "Local storage manager initialized at {}",
            manager.structure.root.display()
        );
        Ok(manager)
    }

    /// Store personal data locally
    pub async fn store_personal(&self, user_id: &str, data_id: &str, data: &[u8]) -> Result<()> {
        let item_id = format!("personal:{}:{}", user_id, data_id);
        let file_path = self
            .structure
            .personal
            .join(user_id)
            .join(format!("{}.data", data_id));

        // Create user directory if needed
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Write data to file
        tokio::fs::write(&file_path, data)
            .await
            .context("Failed to write personal data to file")?;

        // Create metadata
        let metadata = StorageMetadata {
            item_id: item_id.clone(),
            item_type: StorageItemType::PersonalData {
                user_id: user_id.to_string(),
            },
            file_path: file_path.clone(),
            size: data.len(),
            hash: blake3::hash(data).to_string(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            encryption_info: None,  // Encryption handled at higher level
            compression_info: None, // TODO: Add compression
        };

        // Update index
        let previous_metadata = {
            let mut index = self.metadata_index.write().await;
            index.insert(item_id, metadata.clone())
        };

        // Update stats
        {
            let mut stats = self.usage_stats.write().await;
            if let Some(previous) = previous_metadata {
                stats.personal_data_size =
                    stats.personal_data_size.saturating_sub(previous.size) + data.len();
            } else {
                stats.personal_data_size += data.len();
                stats.total_files += 1;
            }
        }

        // Persist metadata
        self.save_metadata_index().await?;

        debug!(
            "Stored personal data for {} ({}): {} bytes",
            user_id,
            data_id,
            data.len()
        );
        Ok(())
    }

    /// Retrieve personal data
    pub async fn retrieve_personal(&self, user_id: &str, data_id: &str) -> Result<Vec<u8>> {
        let item_id = format!("personal:{}:{}", user_id, data_id);

        // Get metadata
        let metadata = {
            let index = self.metadata_index.read().await;
            index
                .get(&item_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Personal data not found: {}", item_id))?
        };

        // Read file
        let data = tokio::fs::read(&metadata.file_path)
            .await
            .context("Failed to read personal data file")?;

        // Verify integrity
        let current_hash = blake3::hash(&data).to_string();
        if current_hash != metadata.hash {
            error!("Data integrity check failed for {}", item_id);
            bail!("Data integrity check failed");
        }

        // Update access time
        {
            let mut index = self.metadata_index.write().await;
            if let Some(meta) = index.get_mut(&item_id) {
                meta.last_accessed = chrono::Utc::now();
            }
        }

        debug!(
            "Retrieved personal data for {} ({}): {} bytes",
            user_id,
            data_id,
            data.len()
        );
        Ok(data)
    }

    /// Store group shard locally
    pub async fn store_group_shard(&self, group_id: &str, shard: &Shard) -> Result<()> {
        let item_id = format!("shard:{}:{}", group_id, shard.index);
        let file_path = self
            .structure
            .group_shards
            .join(group_id)
            .join(format!("shard_{}.data", shard.index));

        // Create group directory if needed
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Serialize shard for storage (includes metadata)
        let shard_data = bincode::serialize(shard).context("Failed to serialize shard")?;

        // Write to file
        tokio::fs::write(&file_path, &shard_data)
            .await
            .context("Failed to write shard to file")?;

        // Create metadata
        let metadata = StorageMetadata {
            item_id: item_id.clone(),
            item_type: StorageItemType::GroupShard {
                group_id: group_id.to_string(),
                shard_index: shard.index,
            },
            file_path: file_path.clone(),
            size: shard_data.len(),
            hash: blake3::hash(&shard_data).to_string(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            encryption_info: None,
            compression_info: None,
        };

        // Update index
        let previous_metadata = {
            let mut index = self.metadata_index.write().await;
            index.insert(item_id, metadata.clone())
        };

        // Update stats
        {
            let mut stats = self.usage_stats.write().await;
            if let Some(previous) = previous_metadata {
                stats.group_shards_size = stats
                    .group_shards_size
                    .saturating_sub(previous.size)
                    + shard_data.len();
            } else {
                stats.group_shards_size += shard_data.len();
                stats.total_files += 1;
            }
        }

        self.save_metadata_index().await?;

        debug!(
            "Stored group shard {} for group {}: {} bytes",
            shard.index,
            group_id,
            shard_data.len()
        );
        Ok(())
    }

    /// Retrieve group shard
    pub async fn retrieve_group_shard(&self, group_id: &str, shard_index: usize) -> Result<Shard> {
        let item_id = format!("shard:{}:{}", group_id, shard_index);

        // Get metadata
        let metadata = {
            let index = self.metadata_index.read().await;
            index
                .get(&item_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Group shard not found: {}", item_id))?
        };

        // Read file
        let shard_data = tokio::fs::read(&metadata.file_path)
            .await
            .context("Failed to read shard file")?;

        // Verify integrity
        let current_hash = blake3::hash(&shard_data).to_string();
        if current_hash != metadata.hash {
            error!("Shard integrity check failed for {}", item_id);
            bail!("Shard integrity check failed");
        }

        // Deserialize shard
        let shard: Shard =
            bincode::deserialize(&shard_data).context("Failed to deserialize shard")?;

        // Update access time
        {
            let mut index = self.metadata_index.write().await;
            if let Some(meta) = index.get_mut(&item_id) {
                meta.last_accessed = chrono::Utc::now();
            }
        }

        debug!(
            "Retrieved group shard {} for group {}: {} bytes",
            shard_index,
            group_id,
            shard_data.len()
        );
        Ok(shard)
    }

    /// Store DHT data (from other nodes)
    pub async fn store_dht_data(&self, key: &str, data: &[u8]) -> Result<()> {
        let item_id = format!("dht:{}", key);
        let safe_filename = key
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();
        let file_path = self
            .structure
            .dht_cache
            .join(format!("{}.data", safe_filename));

        // Write data to file
        tokio::fs::write(&file_path, data)
            .await
            .context("Failed to write DHT data to file")?;

        // Create metadata
        let metadata = StorageMetadata {
            item_id: item_id.clone(),
            item_type: StorageItemType::DHTData {
                key: key.to_string(),
                owner: "unknown".to_string(), // TODO: Extract from DHT metadata
            },
            file_path: file_path.clone(),
            size: data.len(),
            hash: blake3::hash(data).to_string(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            encryption_info: None,
            compression_info: None,
        };

        // Update index
        let previous_metadata = {
            let mut index = self.metadata_index.write().await;
            index.insert(item_id, metadata.clone())
        };

        // Update stats
        {
            let mut stats = self.usage_stats.write().await;
            if let Some(previous) = previous_metadata {
                stats.dht_cache_size = stats
                    .dht_cache_size
                    .saturating_sub(previous.size)
                    + data.len();
            } else {
                stats.dht_cache_size += data.len();
                stats.total_files += 1;
            }
        }

        self.save_metadata_index().await?;

        debug!("Stored DHT data for key {}: {} bytes", key, data.len());
        Ok(())
    }

    /// Retrieve DHT data
    pub async fn retrieve_dht_data(&self, key: &str) -> Result<Vec<u8>> {
        let item_id = format!("dht:{}", key);

        // Get metadata
        let metadata = {
            let index = self.metadata_index.read().await;
            index
                .get(&item_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("DHT data not found: {}", item_id))?
        };

        // Read file
        let data = tokio::fs::read(&metadata.file_path)
            .await
            .context("Failed to read DHT data file")?;

        // Verify integrity
        let current_hash = blake3::hash(&data).to_string();
        if current_hash != metadata.hash {
            error!("DHT data integrity check failed for {}", item_id);
            bail!("DHT data integrity check failed");
        }

        // Update access time
        {
            let mut index = self.metadata_index.write().await;
            if let Some(meta) = index.get_mut(&item_id) {
                meta.last_accessed = chrono::Utc::now();
            }
        }

        debug!("Retrieved DHT data for key {}: {} bytes", key, data.len());
        Ok(data)
    }

    /// Get all shards for a group
    pub async fn get_group_shards(&self, group_id: &str) -> Result<Vec<Shard>> {
        let mut shards = Vec::new();

        // Collect shard indices first to avoid holding lock during async operations
        let shard_indices = {
            let index = self.metadata_index.read().await;
            index
                .values()
                .filter_map(|metadata| {
                    if let StorageItemType::GroupShard {
                        group_id: meta_group_id,
                        shard_index,
                    } = &metadata.item_type
                    {
                        if meta_group_id == group_id {
                            Some(*shard_index)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        // Now fetch each shard
        for shard_index in shard_indices {
            let shard = self.retrieve_group_shard(group_id, shard_index).await?;
            shards.push(shard);
        }

        debug!("Found {} shards for group {}", shards.len(), group_id);
        Ok(shards)
    }

    // Wrapper methods to match test interface
    pub async fn store_personal_data(&self, data_id: &str, data: &[u8]) -> Result<()> {
        self.store_personal("default_user", data_id, data).await
    }

    pub async fn retrieve_personal_data(&self, data_id: &str) -> Result<Vec<u8>> {
        self.retrieve_personal("default_user", data_id).await
    }

    pub async fn store_group_shard_test(
        &self,
        group_id: &str,
        shard_id: &str,
        data: &[u8],
    ) -> Result<()> {
        // Create a dummy shard for testing
        let shard = Shard {
            index: shard_id.parse().unwrap_or(0),
            shard_type: super::reed_solomon_manager::ShardType::Data,
            data: data.to_vec(),
            group_id: group_id.to_string(),
            data_id: "test".to_string(),
            integrity_hash: blake3::hash(data).to_string(),
            created_at: chrono::Utc::now(),
            size: data.len(),
        };
        self.store_group_shard(group_id, &shard).await
    }

    pub async fn retrieve_group_shard_test(
        &self,
        group_id: &str,
        shard_id: &str,
    ) -> Result<Vec<u8>> {
        let shard_index = shard_id.parse().unwrap_or(0);
        let shard = self.retrieve_group_shard(group_id, shard_index).await?;
        Ok(shard.data)
    }

    pub async fn store_dht_data_by_hash(&self, key: &blake3::Hash, data: &[u8]) -> Result<()> {
        self.store_dht_data(&key.to_string(), data).await
    }

    pub async fn retrieve_dht_data_by_hash(&self, key: &blake3::Hash) -> Result<Vec<u8>> {
        self.retrieve_dht_data(&key.to_string()).await
    }

    /// Get storage allocation information
    pub fn get_allocation(&self) -> TestAllocation {
        TestAllocation {
            personal_storage: self.capacity / 4,   // 1/4 of total capacity
            dht_backup_storage: self.capacity / 4, // 1/4 of total capacity
            public_dht_storage: self.capacity / 2, // 1/2 of total capacity
        }
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> LocalStorageStats {
        let stats = self.usage_stats.read().await;
        stats.clone()
    }

    /// Cleanup old or unused data
    pub async fn cleanup_storage(
        &self,
        max_age_days: u32,
        max_cache_size: usize,
    ) -> Result<CleanupReport> {
        info!(
            "Starting storage cleanup (max age: {} days, max cache: {} bytes)",
            max_age_days, max_cache_size
        );

        let mut report = CleanupReport {
            files_removed: 0,
            bytes_freed: 0,
            errors: vec![],
        };

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(max_age_days as i64);
        let mut items_to_remove = Vec::new();

        // Find items to clean up
        {
            let index = self.metadata_index.read().await;
            for (item_id, metadata) in index.iter() {
                let should_remove = match &metadata.item_type {
                    StorageItemType::DHTData { .. } => {
                        // Remove old DHT cache data
                        metadata.last_accessed < cutoff_date
                    }
                    StorageItemType::PersonalData { .. } => {
                        // Don't auto-remove personal data
                        false
                    }
                    StorageItemType::GroupShard { .. } => {
                        // Don't auto-remove group shards
                        false
                    }
                    StorageItemType::Metadata => false,
                };

                if should_remove {
                    items_to_remove.push(item_id.clone());
                }
            }
        }

        // Remove identified items
        for item_id in items_to_remove {
            match self.remove_item(&item_id).await {
                Ok(size) => {
                    report.files_removed += 1;
                    report.bytes_freed += size;
                    debug!("Cleaned up item {}: {} bytes", item_id, size);
                }
                Err(e) => {
                    error!("Failed to clean up item {}: {}", item_id, e);
                    report
                        .errors
                        .push(format!("Failed to remove {}: {}", item_id, e));
                }
            }
        }

        // Update cleanup time
        {
            let mut stats = self.usage_stats.write().await;
            stats.last_cleanup = chrono::Utc::now();
        }

        self.save_metadata_index().await?;

        info!(
            "Storage cleanup completed: {} files removed, {} bytes freed",
            report.files_removed, report.bytes_freed
        );

        Ok(report)
    }

    // Private helper methods

    async fn load_metadata_index(&self) -> Result<()> {
        let index_file = self.structure.metadata.join("storage_index.json");

        if !index_file.exists() {
            debug!("No existing metadata index found, starting fresh");
            return Ok(());
        }

        let index_data = tokio::fs::read_to_string(&index_file)
            .await
            .context("Failed to read metadata index")?;

        let stored_index: HashMap<String, StorageMetadata> =
            serde_json::from_str(&index_data).context("Failed to parse metadata index")?;

        let item_count = stored_index.len();
        {
            let mut index = self.metadata_index.write().await;
            *index = stored_index;
        }

        info!("Loaded {} items from metadata index", item_count);
        Ok(())
    }

    async fn save_metadata_index(&self) -> Result<()> {
        let index_file = self.structure.metadata.join("storage_index.json");
        let temp_file = self.structure.temp.join("storage_index.tmp");

        let index_data = {
            let index = self.metadata_index.read().await;
            serde_json::to_string_pretty(&*index).context("Failed to serialize metadata index")?
        };

        // Write to temp file first, then atomically move
        tokio::fs::write(&temp_file, &index_data)
            .await
            .context("Failed to write temp metadata index")?;

        tokio::fs::rename(&temp_file, &index_file)
            .await
            .context("Failed to move metadata index to final location")?;

        debug!("Saved metadata index to {}", index_file.display());
        Ok(())
    }

    async fn calculate_usage_stats(&self) -> Result<()> {
        let mut stats = LocalStorageStats {
            personal_data_size: 0,
            group_shards_size: 0,
            dht_cache_size: 0,
            total_files: 0,
            last_cleanup: chrono::Utc::now(),
        };

        let index = self.metadata_index.read().await;
        for metadata in index.values() {
            stats.total_files += 1;

            match &metadata.item_type {
                StorageItemType::PersonalData { .. } => {
                    stats.personal_data_size += metadata.size;
                }
                StorageItemType::GroupShard { .. } => {
                    stats.group_shards_size += metadata.size;
                }
                StorageItemType::DHTData { .. } => {
                    stats.dht_cache_size += metadata.size;
                }
                StorageItemType::Metadata => {}
            }
        }

        debug!(
            "Calculated storage stats: {} files, {} personal, {} shards, {} DHT cache",
            stats.total_files,
            stats.personal_data_size,
            stats.group_shards_size,
            stats.dht_cache_size
        );

        {
            let mut current_stats = self.usage_stats.write().await;
            *current_stats = stats;
        }

        Ok(())
    }

    async fn remove_item(&self, item_id: &str) -> Result<usize> {
        let metadata = {
            let mut index = self.metadata_index.write().await;
            index
                .remove(item_id)
                .ok_or_else(|| anyhow::anyhow!("Item not found in index: {}", item_id))?
        };

        // Remove file
        if metadata.file_path.exists() {
            tokio::fs::remove_file(&metadata.file_path)
                .await
                .context("Failed to remove file")?;
        }

        // Update stats
        {
            let mut stats = self.usage_stats.write().await;
            match &metadata.item_type {
                StorageItemType::PersonalData { .. } => {
                    stats.personal_data_size =
                        stats.personal_data_size.saturating_sub(metadata.size);
                }
                StorageItemType::GroupShard { .. } => {
                    stats.group_shards_size = stats.group_shards_size.saturating_sub(metadata.size);
                }
                StorageItemType::DHTData { .. } => {
                    stats.dht_cache_size = stats.dht_cache_size.saturating_sub(metadata.size);
                }
                StorageItemType::Metadata => {}
            }
            stats.total_files = stats.total_files.saturating_sub(1);
        }

        Ok(metadata.size)
    }
}

#[derive(Debug)]
pub struct CleanupReport {
    pub files_removed: usize,
    pub bytes_freed: usize,
    pub errors: Vec<String>,
}

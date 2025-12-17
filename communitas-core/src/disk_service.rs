// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Entity Disk Service - Per-entity virtual disk management
//!
//! This module provides virtual disk functionality for entities (users, organizations,
//! groups, channels, projects). Each entity has three disk types:
//!
//! - **Private**: Encrypted, local-only storage (owner access only)
//! - **Public**: Content-addressed, distributed storage (world-readable)
//! - **Shared**: Group-accessible with shared encryption (members only)

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Type of virtual disk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiskType {
    /// Private: Encrypted, local-only storage (owner access only)
    Private,
    /// Public: Content-addressed, distributed storage (world-readable)
    Public,
    /// Shared: Group-accessible with shared encryption (members only)
    Shared,
}

impl DiskType {
    /// Get the directory name for this disk type
    pub fn as_dir_name(&self) -> &'static str {
        match self {
            DiskType::Private => "private",
            DiskType::Public => "public",
            DiskType::Shared => "shared",
        }
    }
}

impl std::fmt::Display for DiskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiskType::Private => write!(f, "private"),
            DiskType::Public => write!(f, "public"),
            DiskType::Shared => write!(f, "shared"),
        }
    }
}

/// Information about a file or directory in a virtual disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Full path within the disk (e.g., "/docs/readme.md")
    pub path: String,
    /// File or directory name
    pub name: String,
    /// True if this is a directory
    pub is_directory: bool,
    /// Size in bytes (0 for directories)
    pub size_bytes: u64,
    /// Last modified timestamp (Unix epoch seconds)
    pub modified_at: i64,
    /// BLAKE3 hash of contents (empty for directories)
    pub content_hash: String,
}

/// Storage statistics for a virtual disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStats {
    /// Entity ID this disk belongs to
    pub entity_id: String,
    /// Type of disk
    pub disk_type: DiskType,
    /// Total bytes used
    pub used_bytes: u64,
    /// Total number of files
    pub file_count: u32,
    /// Total number of directories
    pub dir_count: u32,
    /// Last modification timestamp
    pub last_modified: i64,
}

/// Internal metadata for tracking files
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiskFileMetadata {
    pub entity_id: String,
    pub disk_type: DiskType,
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub modified_at: i64,
    pub content_hash: String,
    pub file_path: PathBuf, // Actual filesystem path
}

/// Entity Disk Service - manages per-entity virtual disks
#[derive(Debug)]
pub struct EntityDiskService {
    /// Root directory for all disk storage
    root: PathBuf,
    /// Metadata index: (entity_id, disk_type, path) -> metadata
    index: RwLock<HashMap<String, DiskFileMetadata>>,
}

impl EntityDiskService {
    /// Create a new EntityDiskService with the given root directory
    pub async fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();

        // Create root directory if it doesn't exist
        tokio::fs::create_dir_all(&root)
            .await
            .with_context(|| format!("Failed to create disk root: {}", root.display()))?;

        let service = Self {
            root,
            index: RwLock::new(HashMap::new()),
        };

        // Load existing metadata
        service.load_index().await?;

        info!(
            "EntityDiskService initialized at {}",
            service.root.display()
        );
        Ok(service)
    }

    /// Get the filesystem path for an entity's disk directory
    fn get_entity_disk_path(&self, entity_id: &str, disk_type: DiskType) -> PathBuf {
        self.root
            .join("entities")
            .join(entity_id)
            .join(disk_type.as_dir_name())
    }

    /// Get the filesystem path for a file within an entity's disk
    fn get_file_path(&self, entity_id: &str, disk_type: DiskType, path: &str) -> PathBuf {
        let disk_path = self.get_entity_disk_path(entity_id, disk_type);
        // Normalize path: remove leading slash and any .. components
        let clean_path = path.trim_start_matches('/').replace("..", "");
        disk_path.join(clean_path)
    }

    /// Generate the index key for a file
    fn index_key(entity_id: &str, disk_type: DiskType, path: &str) -> String {
        format!("{}:{}:{}", entity_id, disk_type, path)
    }

    /// Write a file to an entity's virtual disk
    pub async fn write_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        data: &[u8],
    ) -> Result<FileInfo> {
        // Validate path
        if path.is_empty() || path == "/" {
            bail!("Invalid file path: cannot write to root");
        }

        let file_path = self.get_file_path(entity_id, disk_type, path);

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create parent directories for {}",
                    file_path.display()
                )
            })?;
        }

        // Write the file
        tokio::fs::write(&file_path, data)
            .await
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        // Calculate hash
        let content_hash = blake3::hash(data).to_string();

        // Get file name
        let name = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let now = chrono::Utc::now().timestamp();

        // Create metadata
        let metadata = DiskFileMetadata {
            entity_id: entity_id.to_string(),
            disk_type,
            path: path.to_string(),
            name: name.clone(),
            is_directory: false,
            size_bytes: data.len() as u64,
            modified_at: now,
            content_hash: content_hash.clone(),
            file_path: file_path.clone(),
        };

        // Update index
        {
            let key = Self::index_key(entity_id, disk_type, path);
            let mut index = self.index.write().await;
            index.insert(key, metadata);
        }

        // Persist index
        self.save_index().await?;

        debug!(
            "Wrote file {}:{}{} ({} bytes)",
            entity_id,
            disk_type,
            path,
            data.len()
        );

        Ok(FileInfo {
            path: path.to_string(),
            name,
            is_directory: false,
            size_bytes: data.len() as u64,
            modified_at: now,
            content_hash,
        })
    }

    /// Read a file from an entity's virtual disk
    pub async fn read_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<u8>> {
        let key = Self::index_key(entity_id, disk_type, path);

        // Get metadata
        let metadata = {
            let index = self.index.read().await;
            index.get(&key).cloned()
        };

        let file_path = match metadata {
            Some(meta) => {
                if meta.is_directory {
                    bail!("Cannot read directory as file: {}", path);
                }
                meta.file_path
            }
            None => {
                // Try direct path lookup
                let fp = self.get_file_path(entity_id, disk_type, path);
                if !fp.exists() {
                    bail!("File not found: {}:{}{}", entity_id, disk_type, path);
                }
                fp
            }
        };

        // Read the file
        let data = tokio::fs::read(&file_path)
            .await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        debug!(
            "Read file {}:{}{} ({} bytes)",
            entity_id,
            disk_type,
            path,
            data.len()
        );

        Ok(data)
    }

    /// List files in a directory within an entity's virtual disk
    pub async fn list_files(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<FileInfo>> {
        let dir_path = if path.is_empty() || path == "/" {
            self.get_entity_disk_path(entity_id, disk_type)
        } else {
            self.get_file_path(entity_id, disk_type, path)
        };

        // Create directory if it doesn't exist
        if !dir_path.exists() {
            tokio::fs::create_dir_all(&dir_path)
                .await
                .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;
        }

        if !dir_path.is_dir() {
            bail!("Path is not a directory: {}", path);
        }

        let mut entries = Vec::new();
        let mut read_dir = tokio::fs::read_dir(&dir_path)
            .await
            .with_context(|| format!("Failed to read directory: {}", dir_path.display()))?;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await?;

            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and index files
            if name.starts_with('.') || name == "disk_index.json" {
                continue;
            }

            let entry_relative_path = if path.is_empty() || path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path.trim_end_matches('/'), name)
            };

            let (content_hash, size_bytes) = if metadata.is_file() {
                let data = tokio::fs::read(&entry_path).await?;
                (blake3::hash(&data).to_string(), data.len() as u64)
            } else {
                (String::new(), 0)
            };

            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            entries.push(FileInfo {
                path: entry_relative_path,
                name,
                is_directory: metadata.is_dir(),
                size_bytes,
                modified_at,
                content_hash,
            });
        }

        // Sort by name
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        debug!(
            "Listed {}:{}{} - {} entries",
            entity_id,
            disk_type,
            path,
            entries.len()
        );

        Ok(entries)
    }

    /// Delete a file from an entity's virtual disk
    pub async fn delete_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<()> {
        if path.is_empty() || path == "/" {
            bail!("Cannot delete root directory");
        }

        let file_path = self.get_file_path(entity_id, disk_type, path);

        if !file_path.exists() {
            bail!("File not found: {}:{}{}", entity_id, disk_type, path);
        }

        if file_path.is_dir() {
            tokio::fs::remove_dir_all(&file_path)
                .await
                .with_context(|| format!("Failed to delete directory: {}", file_path.display()))?;
        } else {
            tokio::fs::remove_file(&file_path)
                .await
                .with_context(|| format!("Failed to delete file: {}", file_path.display()))?;
        }

        // Remove from index
        {
            let key = Self::index_key(entity_id, disk_type, path);
            let mut index = self.index.write().await;
            index.remove(&key);
        }

        // Persist index
        self.save_index().await?;

        debug!("Deleted {}:{}{}", entity_id, disk_type, path);

        Ok(())
    }

    /// Get storage statistics for an entity's disk
    pub async fn get_stats(&self, entity_id: &str, disk_type: DiskType) -> Result<DiskStats> {
        let disk_path = self.get_entity_disk_path(entity_id, disk_type);

        let mut used_bytes: u64 = 0;
        let mut file_count: u32 = 0;
        let mut dir_count: u32 = 0;
        let mut last_modified: i64 = 0;

        if disk_path.exists() {
            self.calculate_stats_recursive(
                &disk_path,
                &mut used_bytes,
                &mut file_count,
                &mut dir_count,
                &mut last_modified,
            )
            .await?;
        }

        Ok(DiskStats {
            entity_id: entity_id.to_string(),
            disk_type,
            used_bytes,
            file_count,
            dir_count,
            last_modified,
        })
    }

    /// Recursively calculate storage statistics
    async fn calculate_stats_recursive(
        &self,
        path: &Path,
        used_bytes: &mut u64,
        file_count: &mut u32,
        dir_count: &mut u32,
        last_modified: &mut i64,
    ) -> Result<()> {
        let mut read_dir = tokio::fs::read_dir(path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_path = entry.path();
            let metadata = entry.metadata().await?;

            // Skip hidden files
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "disk_index.json" {
                continue;
            }

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            if modified > *last_modified {
                *last_modified = modified;
            }

            if metadata.is_dir() {
                *dir_count += 1;
                Box::pin(self.calculate_stats_recursive(
                    &entry_path,
                    used_bytes,
                    file_count,
                    dir_count,
                    last_modified,
                ))
                .await?;
            } else {
                *file_count += 1;
                *used_bytes += metadata.len();
            }
        }

        Ok(())
    }

    /// Create a directory in an entity's virtual disk
    pub async fn create_directory(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<FileInfo> {
        if path.is_empty() || path == "/" {
            bail!("Cannot create root directory");
        }

        let dir_path = self.get_file_path(entity_id, disk_type, path);

        tokio::fs::create_dir_all(&dir_path)
            .await
            .with_context(|| format!("Failed to create directory: {}", dir_path.display()))?;

        let name = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let now = chrono::Utc::now().timestamp();

        // Create metadata
        let metadata = DiskFileMetadata {
            entity_id: entity_id.to_string(),
            disk_type,
            path: path.to_string(),
            name: name.clone(),
            is_directory: true,
            size_bytes: 0,
            modified_at: now,
            content_hash: String::new(),
            file_path: dir_path,
        };

        // Update index
        {
            let key = Self::index_key(entity_id, disk_type, path);
            let mut index = self.index.write().await;
            index.insert(key, metadata);
        }

        // Persist index
        self.save_index().await?;

        debug!("Created directory {}:{}{}", entity_id, disk_type, path);

        Ok(FileInfo {
            path: path.to_string(),
            name,
            is_directory: true,
            size_bytes: 0,
            modified_at: now,
            content_hash: String::new(),
        })
    }

    /// Check if a file exists
    pub async fn file_exists(&self, entity_id: &str, disk_type: DiskType, path: &str) -> bool {
        let file_path = self.get_file_path(entity_id, disk_type, path);
        file_path.exists()
    }

    /// Get file info without reading the contents
    pub async fn get_file_info(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<FileInfo> {
        let file_path = self.get_file_path(entity_id, disk_type, path);

        if !file_path.exists() {
            bail!("File not found: {}:{}{}", entity_id, disk_type, path);
        }

        let metadata = tokio::fs::metadata(&file_path).await?;
        let is_directory = metadata.is_dir();

        let name = Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let (size_bytes, content_hash) = if is_directory {
            (0, String::new())
        } else {
            let data = tokio::fs::read(&file_path).await?;
            (data.len() as u64, blake3::hash(&data).to_string())
        };

        Ok(FileInfo {
            path: path.to_string(),
            name,
            is_directory,
            size_bytes,
            modified_at,
            content_hash,
        })
    }

    /// Load the metadata index from disk
    async fn load_index(&self) -> Result<()> {
        let index_path = self.root.join("disk_index.json");

        if !index_path.exists() {
            debug!("No existing disk index found, starting fresh");
            return Ok(());
        }

        match tokio::fs::read_to_string(&index_path).await {
            Ok(data) => match serde_json::from_str::<HashMap<String, DiskFileMetadata>>(&data) {
                Ok(stored_index) => {
                    let count = stored_index.len();
                    let mut index = self.index.write().await;
                    *index = stored_index;
                    info!("Loaded {} entries from disk index", count);
                }
                Err(e) => {
                    warn!("Failed to parse disk index, starting fresh: {}", e);
                }
            },
            Err(e) => {
                warn!("Failed to read disk index, starting fresh: {}", e);
            }
        }

        Ok(())
    }

    /// Save the metadata index to disk
    async fn save_index(&self) -> Result<()> {
        let index_path = self.root.join("disk_index.json");
        let temp_path = self.root.join(".disk_index.tmp");

        let data = {
            let index = self.index.read().await;
            serde_json::to_string_pretty(&*index).context("Failed to serialize disk index")?
        };

        // Write to temp file first
        tokio::fs::write(&temp_path, &data)
            .await
            .context("Failed to write temp index file")?;

        // Atomically move to final location
        tokio::fs::rename(&temp_path, &index_path)
            .await
            .context("Failed to move index file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_write_and_read_file() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let data = b"Hello, World!";

        // Write file
        let info = service
            .write_file(entity_id, DiskType::Private, "/docs/test.txt", data)
            .await
            .unwrap();

        assert_eq!(info.name, "test.txt");
        assert_eq!(info.size_bytes, 13);
        assert!(!info.is_directory);

        // Read file
        let read_data = service
            .read_file(entity_id, DiskType::Private, "/docs/test.txt")
            .await
            .unwrap();

        assert_eq!(read_data, data);
    }

    #[tokio::test]
    async fn test_list_files() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";

        // Write some files
        service
            .write_file(entity_id, DiskType::Public, "/file1.txt", b"one")
            .await
            .unwrap();
        service
            .write_file(entity_id, DiskType::Public, "/file2.txt", b"two")
            .await
            .unwrap();

        // List files
        let files = service
            .list_files(entity_id, DiskType::Public, "/")
            .await
            .unwrap();

        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_file() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";

        // Write file
        service
            .write_file(entity_id, DiskType::Shared, "/to_delete.txt", b"delete me")
            .await
            .unwrap();

        // Verify it exists
        assert!(
            service
                .file_exists(entity_id, DiskType::Shared, "/to_delete.txt")
                .await
        );

        // Delete it
        service
            .delete_file(entity_id, DiskType::Shared, "/to_delete.txt")
            .await
            .unwrap();

        // Verify it's gone
        assert!(
            !service
                .file_exists(entity_id, DiskType::Shared, "/to_delete.txt")
                .await
        );
    }

    #[tokio::test]
    async fn test_get_stats() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";

        // Write some files
        service
            .write_file(entity_id, DiskType::Private, "/file1.txt", b"hello")
            .await
            .unwrap();
        service
            .write_file(entity_id, DiskType::Private, "/dir/file2.txt", b"world")
            .await
            .unwrap();

        // Get stats
        let stats = service
            .get_stats(entity_id, DiskType::Private)
            .await
            .unwrap();

        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.used_bytes, 10); // "hello" + "world"
        assert_eq!(stats.dir_count, 1); // "dir"
    }

    #[tokio::test]
    async fn test_create_directory() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";

        // Create directory
        let info = service
            .create_directory(entity_id, DiskType::Public, "/my-folder")
            .await
            .unwrap();

        assert_eq!(info.name, "my-folder");
        assert!(info.is_directory);

        // Verify it exists
        assert!(
            service
                .file_exists(entity_id, DiskType::Public, "/my-folder")
                .await
        );
    }

    #[tokio::test]
    async fn test_disk_types() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";

        // Write to each disk type
        service
            .write_file(entity_id, DiskType::Private, "/private.txt", b"private")
            .await
            .unwrap();
        service
            .write_file(entity_id, DiskType::Public, "/public.txt", b"public")
            .await
            .unwrap();
        service
            .write_file(entity_id, DiskType::Shared, "/shared.txt", b"shared")
            .await
            .unwrap();

        // Verify isolation
        assert!(
            service
                .file_exists(entity_id, DiskType::Private, "/private.txt")
                .await
        );
        assert!(
            !service
                .file_exists(entity_id, DiskType::Public, "/private.txt")
                .await
        );
        assert!(
            !service
                .file_exists(entity_id, DiskType::Shared, "/private.txt")
                .await
        );
    }
}

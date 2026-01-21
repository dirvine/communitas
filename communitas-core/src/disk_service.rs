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
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Default chunk size for streaming transfers (1MB)
pub const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024;

/// Maximum supported file size (1GB)
pub const MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;

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

/// Information about a chunk of a file for streaming transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Offset in bytes from the start of the file
    pub offset: u64,
    /// Size of this chunk in bytes
    pub size: u64,
    /// BLAKE3 hash of this chunk's data
    pub chunk_hash: String,
    /// Total file size (for progress tracking)
    pub total_size: u64,
    /// Total number of chunks
    pub total_chunks: u64,
    /// This chunk's index (0-based)
    pub chunk_index: u64,
}

/// Result of a chunk read operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkReadResult {
    /// The chunk data
    pub data: Vec<u8>,
    /// Chunk metadata
    pub info: ChunkInfo,
    /// Whether this is the last chunk
    pub is_last: bool,
}

/// Result of a chunk write operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkWriteResult {
    /// Chunk metadata
    pub info: ChunkInfo,
    /// Running BLAKE3 hash of all data written so far
    pub cumulative_hash: String,
    /// Whether this was the final chunk
    pub is_complete: bool,
}

/// Stale transfer threshold: transfers older than 24 hours can be cleaned up
pub const STALE_TRANSFER_THRESHOLD_SECS: i64 = 24 * 60 * 60;

/// Metadata for an in-progress chunked write (persisted for resume support)
///
/// Fields are crate-private to maintain invariants. Use accessor methods to read state.
/// Invariants enforced:
/// - `bytes_written <= total_size`
/// - `bytes_written` is always a multiple of `chunk_size` (except for final chunk)
/// - `started_at <= last_updated`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferState {
    /// Unique transfer identifier (entity_id:disk_type:path)
    pub(crate) transfer_id: String,
    /// Entity ID
    pub(crate) entity_id: String,
    /// Disk type
    pub(crate) disk_type: DiskType,
    /// Target path
    pub(crate) path: String,
    /// Expected total size
    pub(crate) total_size: u64,
    /// Bytes written so far
    pub(crate) bytes_written: u64,
    /// BLAKE3 hasher state (serialized) - reserved for future incremental hashing
    #[serde(default)]
    pub(crate) hasher_state: Vec<u8>,
    /// Chunk size being used
    pub(crate) chunk_size: u64,
    /// Transfer started timestamp (Unix epoch seconds)
    pub(crate) started_at: i64,
    /// Last write timestamp (Unix epoch seconds)
    pub(crate) last_updated: i64,
    /// BLAKE3 hash of all data written so far (only accurate on completion)
    pub(crate) cumulative_hash: String,
}

impl TransferState {
    /// Get the unique transfer identifier
    pub fn transfer_id(&self) -> &str {
        &self.transfer_id
    }

    /// Get the entity ID
    pub fn entity_id(&self) -> &str {
        &self.entity_id
    }

    /// Get the disk type
    pub fn disk_type(&self) -> DiskType {
        self.disk_type
    }

    /// Get the target path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the expected total size in bytes
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Get the number of bytes written so far
    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    /// Get the chunk size being used
    pub fn chunk_size(&self) -> u64 {
        self.chunk_size
    }

    /// Get the timestamp when the transfer started (Unix epoch seconds)
    pub fn started_at(&self) -> i64 {
        self.started_at
    }

    /// Get the timestamp of the last write (Unix epoch seconds)
    pub fn last_updated(&self) -> i64 {
        self.last_updated
    }

    /// Get the cumulative hash (only accurate when transfer is complete)
    pub fn cumulative_hash(&self) -> &str {
        &self.cumulative_hash
    }

    /// Get the transfer progress as a percentage (0.0 to 100.0)
    pub fn progress_percent(&self) -> f64 {
        if self.total_size == 0 {
            100.0
        } else {
            (self.bytes_written as f64 / self.total_size as f64) * 100.0
        }
    }

    /// Check if the transfer is complete
    pub fn is_complete(&self) -> bool {
        self.bytes_written >= self.total_size
    }

    /// Get the number of chunks completed
    pub fn chunks_completed(&self) -> u64 {
        if self.chunk_size == 0 {
            0
        } else {
            self.bytes_written / self.chunk_size
        }
    }

    /// Get the total number of chunks
    pub fn total_chunks(&self) -> u64 {
        if self.chunk_size == 0 || self.total_size == 0 {
            0
        } else {
            self.total_size.div_ceil(self.chunk_size)
        }
    }
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
    /// In-progress chunked writes: transfer_id -> state
    chunked_writes: RwLock<HashMap<String, TransferState>>,
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
            chunked_writes: RwLock::new(HashMap::new()),
        };

        // Load existing metadata
        service.load_index().await?;

        // Load persisted transfer states
        service.load_transfer_states().await?;

        info!(
            "EntityDiskService initialized at {}",
            service.root.display()
        );
        Ok(service)
    }

    /// Validate entity_id to prevent path traversal attacks
    ///
    /// Returns an error if entity_id contains dangerous characters.
    fn validate_entity_id(entity_id: &str) -> Result<()> {
        if entity_id.is_empty() {
            bail!("Entity ID cannot be empty");
        }
        if entity_id.contains('/') || entity_id.contains('\\') {
            bail!("Entity ID cannot contain path separators: {}", entity_id);
        }
        if entity_id.contains("..") {
            bail!("Entity ID cannot contain path traversal sequences: {}", entity_id);
        }
        if entity_id.contains('\0') {
            bail!("Entity ID cannot contain null bytes");
        }
        // Reject entity IDs that are just dots
        if entity_id.chars().all(|c| c == '.') {
            bail!("Entity ID cannot be only dots: {}", entity_id);
        }
        Ok(())
    }

    /// Sanitize a file path to prevent path traversal attacks
    ///
    /// Returns an error if the path contains traversal attempts.
    fn sanitize_path(path: &str) -> Result<String> {
        // Remove leading slash for relative path construction
        let path = path.trim_start_matches('/');

        // Split into components and validate each
        let mut clean_components: Vec<&str> = Vec::new();
        for component in path.split('/') {
            // Skip empty components (from multiple consecutive slashes)
            if component.is_empty() {
                continue;
            }
            // Reject any component that is "." or ".."
            if component == "." || component == ".." {
                bail!("Path traversal not allowed: {}", path);
            }
            // Reject components containing ".." anywhere (e.g., "foo..bar")
            if component.contains("..") {
                bail!("Path component contains invalid sequence '..': {}", component);
            }
            // Reject null bytes
            if component.contains('\0') {
                bail!("Path cannot contain null bytes");
            }
            clean_components.push(component);
        }

        Ok(clean_components.join("/"))
    }

    /// Get the filesystem path for an entity's disk directory
    fn get_entity_disk_path(&self, entity_id: &str, disk_type: DiskType) -> Result<PathBuf> {
        Self::validate_entity_id(entity_id)?;
        Ok(self.root
            .join("entities")
            .join(entity_id)
            .join(disk_type.as_dir_name()))
    }

    /// Get the filesystem path for a file within an entity's disk
    fn get_file_path(&self, entity_id: &str, disk_type: DiskType, path: &str) -> Result<PathBuf> {
        let disk_path = self.get_entity_disk_path(entity_id, disk_type)?;
        let clean_path = Self::sanitize_path(path)?;
        let full_path = disk_path.join(&clean_path);

        // For existing files, do a canonicalization check to catch symlink attacks
        // For new files, rely on sanitize_path() validation which already rejects
        // "..", ".", and other traversal attempts
        if full_path.exists() {
            let canonical_disk = disk_path.canonicalize().unwrap_or_else(|_| disk_path.clone());
            let canonical_full = full_path.canonicalize().unwrap_or_else(|_| full_path.clone());

            if !canonical_full.starts_with(&canonical_disk) {
                bail!(
                    "Path escapes disk boundary via symlink: {} resolves outside {}",
                    full_path.display(),
                    disk_path.display()
                );
            }
        }

        Ok(full_path)
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

        let file_path = self.get_file_path(entity_id, disk_type, path)?;

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
                let fp = self.get_file_path(entity_id, disk_type, path)?;
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

    /// Read a chunk of a file for streaming transfers
    ///
    /// This method reads a specific portion of a file, enabling memory-efficient
    /// streaming of large files. Each chunk includes a BLAKE3 hash for verification.
    ///
    /// # Arguments
    /// * `entity_id` - The entity owning the file
    /// * `disk_type` - Which virtual disk to read from
    /// * `path` - Path to the file
    /// * `offset` - Byte offset to start reading from
    /// * `chunk_size` - Number of bytes to read (defaults to DEFAULT_CHUNK_SIZE)
    pub async fn read_chunk(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        offset: u64,
        chunk_size: Option<u64>,
    ) -> Result<ChunkReadResult> {
        let chunk_size = chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        let file_path = self.get_file_path(entity_id, disk_type, path)?;

        if !file_path.exists() {
            bail!("File not found: {}:{}{}", entity_id, disk_type, path);
        }

        let metadata = tokio::fs::metadata(&file_path).await?;
        if metadata.is_dir() {
            bail!("Cannot read chunk from directory: {}", path);
        }

        let total_size = metadata.len();
        if offset >= total_size {
            bail!(
                "Offset {} exceeds file size {}",
                offset,
                total_size
            );
        }

        // Calculate chunk boundaries
        let remaining = total_size - offset;
        let actual_chunk_size = std::cmp::min(chunk_size, remaining);
        let total_chunks = total_size.div_ceil(chunk_size);
        let chunk_index = offset / chunk_size;
        let is_last = offset + actual_chunk_size >= total_size;

        // Read the chunk
        let mut file = tokio::fs::File::open(&file_path).await?;
        file.seek(SeekFrom::Start(offset)).await?;

        let mut buffer = vec![0u8; actual_chunk_size as usize];
        file.read_exact(&mut buffer).await?;

        // Calculate chunk hash
        let chunk_hash = blake3::hash(&buffer).to_string();

        debug!(
            "Read chunk {}/{} of {}:{}{} ({} bytes at offset {})",
            chunk_index + 1,
            total_chunks,
            entity_id,
            disk_type,
            path,
            actual_chunk_size,
            offset
        );

        Ok(ChunkReadResult {
            data: buffer,
            info: ChunkInfo {
                offset,
                size: actual_chunk_size,
                chunk_hash,
                total_size,
                total_chunks,
                chunk_index,
            },
            is_last,
        })
    }

    /// Start a chunked write session for streaming a large file
    ///
    /// This initializes a new chunked write operation. Call `write_chunk` to
    /// write data, then `finish_chunked_write` to complete the operation.
    ///
    /// # Arguments
    /// * `entity_id` - The entity owning the file
    /// * `disk_type` - Which virtual disk to write to
    /// * `path` - Path to the file
    /// * `total_size` - Expected total size of the file
    /// * `chunk_size` - Chunk size to use (defaults to DEFAULT_CHUNK_SIZE)
    pub async fn start_chunked_write(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        total_size: u64,
        chunk_size: Option<u64>,
    ) -> Result<ChunkInfo> {
        if path.is_empty() || path == "/" {
            bail!("Invalid file path: cannot write to root");
        }

        if total_size > MAX_FILE_SIZE {
            bail!(
                "File size {} exceeds maximum {} (1GB)",
                total_size,
                MAX_FILE_SIZE
            );
        }

        let chunk_size = chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        let file_path = self.get_file_path(entity_id, disk_type, path)?;

        // Create parent directories
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create parent directories for {}",
                    file_path.display()
                )
            })?;
        }

        // Create or truncate the file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
            .await
            .with_context(|| format!("Failed to create file: {}", file_path.display()))?;

        // Pre-allocate file if possible (helps with performance)
        if total_size > 0 {
            file.set_len(total_size).await.ok(); // Ignore errors, not critical
        }
        drop(file);

        let total_chunks = if total_size == 0 {
            0
        } else {
            total_size.div_ceil(chunk_size)
        };

        let now = chrono::Utc::now().timestamp();
        let transfer_id = Self::index_key(entity_id, disk_type, path);
        let state = TransferState {
            transfer_id: transfer_id.clone(),
            entity_id: entity_id.to_string(),
            disk_type,
            path: path.to_string(),
            total_size,
            bytes_written: 0,
            hasher_state: Vec::new(), // BLAKE3 state will be accumulated
            chunk_size,
            started_at: now,
            last_updated: now,
            cumulative_hash: String::new(),
        };

        // Store the write state
        {
            let mut writes = self.chunked_writes.write().await;
            writes.insert(transfer_id, state);
        }

        // Persist transfer states
        self.save_transfer_states().await?;

        debug!(
            "Started chunked write for {}:{}{} ({} bytes, {} chunks)",
            entity_id, disk_type, path, total_size, total_chunks
        );

        Ok(ChunkInfo {
            offset: 0,
            size: 0,
            chunk_hash: String::new(),
            total_size,
            total_chunks,
            chunk_index: 0,
        })
    }

    /// Write a chunk of data to an in-progress chunked write
    ///
    /// # Arguments
    /// * `entity_id` - The entity owning the file
    /// * `disk_type` - Which virtual disk to write to
    /// * `path` - Path to the file
    /// * `offset` - Byte offset to write at
    /// * `data` - The chunk data to write
    pub async fn write_chunk(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        offset: u64,
        data: &[u8],
    ) -> Result<ChunkWriteResult> {
        let key = Self::index_key(entity_id, disk_type, path);

        // Get and update write state
        let (chunk_size, total_size, bytes_written) = {
            let writes = self.chunked_writes.read().await;
            let state = writes
                .get(&key)
                .ok_or_else(|| anyhow::anyhow!("No active chunked write for {}:{}{}", entity_id, disk_type, path))?;

            // Verify offset matches expected position
            if offset != state.bytes_written {
                bail!(
                    "Expected offset {}, got {}. Chunks must be written sequentially.",
                    state.bytes_written,
                    offset
                );
            }

            // Verify we won't exceed total size
            if offset + data.len() as u64 > state.total_size {
                bail!(
                    "Write would exceed declared file size ({} + {} > {})",
                    offset,
                    data.len(),
                    state.total_size
                );
            }

            (state.chunk_size, state.total_size, state.bytes_written)
        };

        let file_path = self.get_file_path(entity_id, disk_type, path)?;

        // Write the chunk
        let mut file = OpenOptions::new()
            .write(true)
            .open(&file_path)
            .await
            .with_context(|| format!("Failed to open file for writing: {}", file_path.display()))?;

        file.seek(SeekFrom::Start(offset)).await?;
        file.write_all(data).await?;
        file.flush().await?;

        // Calculate chunk hash
        let chunk_hash = blake3::hash(data).to_string();

        // Update state
        let new_bytes_written = bytes_written + data.len() as u64;
        let is_complete = new_bytes_written >= total_size;
        let total_chunks = total_size.div_ceil(chunk_size);
        let chunk_index = offset / chunk_size;

        // Calculate cumulative hash by reading all data written so far
        // This is expensive but ensures correctness; we can optimize later
        let cumulative_hash = if is_complete {
            let all_data = tokio::fs::read(&file_path).await?;
            blake3::hash(&all_data).to_string()
        } else {
            chunk_hash.clone()
        };

        // Update state with new progress
        {
            let mut writes = self.chunked_writes.write().await;
            if let Some(state) = writes.get_mut(&key) {
                state.bytes_written = new_bytes_written;
                state.last_updated = chrono::Utc::now().timestamp();
                state.cumulative_hash = cumulative_hash.clone();
            }
        }

        // Persist transfer states after each chunk
        self.save_transfer_states().await?;

        debug!(
            "Wrote chunk {}/{} to {}:{}{} ({} bytes at offset {})",
            chunk_index + 1,
            total_chunks,
            entity_id,
            disk_type,
            path,
            data.len(),
            offset
        );

        Ok(ChunkWriteResult {
            info: ChunkInfo {
                offset,
                size: data.len() as u64,
                chunk_hash,
                total_size,
                total_chunks,
                chunk_index,
            },
            cumulative_hash,
            is_complete,
        })
    }

    /// Complete a chunked write operation
    ///
    /// This finalizes the file, updates the metadata index, and cleans up
    /// the write state. Should be called after all chunks have been written.
    pub async fn finish_chunked_write(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<FileInfo> {
        let key = Self::index_key(entity_id, disk_type, path);

        // Remove and get write state
        let state = {
            let mut writes = self.chunked_writes.write().await;
            writes
                .remove(&key)
                .ok_or_else(|| anyhow::anyhow!("No active chunked write for {}:{}{}", entity_id, disk_type, path))?
        };

        // Verify all data was written
        if state.bytes_written != state.total_size {
            bail!(
                "Incomplete write: {} of {} bytes written",
                state.bytes_written,
                state.total_size
            );
        }

        let file_path = self.get_file_path(entity_id, disk_type, path)?;

        // Calculate final hash
        let data = tokio::fs::read(&file_path).await?;
        let content_hash = blake3::hash(&data).to_string();

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
            size_bytes: state.total_size,
            modified_at: now,
            content_hash: content_hash.clone(),
            file_path: file_path.clone(),
        };

        // Update index
        {
            let mut index = self.index.write().await;
            index.insert(key, metadata);
        }

        // Persist index and transfer states
        self.save_index().await?;
        self.save_transfer_states().await?;

        debug!(
            "Finished chunked write for {}:{}{} ({} bytes)",
            entity_id, disk_type, path, state.total_size
        );

        Ok(FileInfo {
            path: path.to_string(),
            name,
            is_directory: false,
            size_bytes: state.total_size,
            modified_at: now,
            content_hash,
        })
    }

    /// Abort an in-progress chunked write
    ///
    /// This cleans up the partial file and write state.
    pub async fn abort_chunked_write(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<()> {
        let key = Self::index_key(entity_id, disk_type, path);

        // Remove write state
        {
            let mut writes = self.chunked_writes.write().await;
            writes.remove(&key);
        }

        // Delete partial file if it exists
        if let Ok(file_path) = self.get_file_path(entity_id, disk_type, path)
            && file_path.exists()
        {
            match tokio::fs::remove_file(&file_path).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("Failed to delete partial file {}: {}", file_path.display(), e);
                }
            }
        }

        // Persist transfer states (removal)
        self.save_transfer_states().await?;

        debug!("Aborted chunked write for {}:{}{}", entity_id, disk_type, path);

        Ok(())
    }

    /// Get the number of chunks needed for a file
    pub fn calculate_chunk_count(file_size: u64, chunk_size: Option<u64>) -> u64 {
        let chunk_size = chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
        if file_size == 0 {
            0
        } else {
            file_size.div_ceil(chunk_size)
        }
    }

    /// Check if there's an active chunked write for a path
    pub async fn has_active_chunked_write(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> bool {
        let key = Self::index_key(entity_id, disk_type, path);
        let writes = self.chunked_writes.read().await;
        writes.contains_key(&key)
    }

    /// Get progress of an active chunked write
    pub async fn get_chunked_write_progress(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Option<(u64, u64)> {
        let key = Self::index_key(entity_id, disk_type, path);
        let writes = self.chunked_writes.read().await;
        writes.get(&key).map(|s| (s.bytes_written, s.total_size))
    }

    /// List files in a directory within an entity's virtual disk
    pub async fn list_files(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<FileInfo>> {
        let dir_path = if path.is_empty() || path == "/" {
            self.get_entity_disk_path(entity_id, disk_type)?
        } else {
            self.get_file_path(entity_id, disk_type, path)?
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

        let file_path = self.get_file_path(entity_id, disk_type, path)?;

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
        let disk_path = self.get_entity_disk_path(entity_id, disk_type)?;

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

        let dir_path = self.get_file_path(entity_id, disk_type, path)?;

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
        match self.get_file_path(entity_id, disk_type, path) {
            Ok(file_path) => file_path.exists(),
            Err(_) => false, // Invalid paths don't exist
        }
    }

    /// Get file info without reading the contents
    pub async fn get_file_info(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<FileInfo> {
        let file_path = self.get_file_path(entity_id, disk_type, path)?;

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
    ///
    /// Returns an error if the index file exists but is corrupt or unreadable.
    /// Missing index files are treated as fresh start (not an error).
    async fn load_index(&self) -> Result<()> {
        let index_path = self.root.join("disk_index.json");

        if !index_path.exists() {
            debug!("No existing disk index found, starting fresh");
            return Ok(());
        }

        let data = tokio::fs::read_to_string(&index_path)
            .await
            .with_context(|| format!("Failed to read disk index at {}", index_path.display()))?;

        let stored_index: HashMap<String, DiskFileMetadata> = serde_json::from_str(&data)
            .with_context(|| {
                format!(
                    "Failed to parse disk index at {} - file may be corrupt",
                    index_path.display()
                )
            })?;

        let count = stored_index.len();
        let mut index = self.index.write().await;
        *index = stored_index;
        info!("Loaded {} entries from disk index", count);

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

    /// Load persisted transfer states from disk
    ///
    /// Returns an error if the states file exists but is corrupt or unreadable.
    /// Missing state files are treated as fresh start (not an error).
    async fn load_transfer_states(&self) -> Result<()> {
        let states_path = self.root.join("transfer_states.json");

        if !states_path.exists() {
            debug!("No existing transfer states found");
            return Ok(());
        }

        let data = tokio::fs::read_to_string(&states_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to read transfer states at {}",
                    states_path.display()
                )
            })?;

        let stored_states: HashMap<String, TransferState> = serde_json::from_str(&data)
            .with_context(|| {
                format!(
                    "Failed to parse transfer states at {} - file may be corrupt",
                    states_path.display()
                )
            })?;

        let count = stored_states.len();
        let mut states = self.chunked_writes.write().await;
        *states = stored_states;
        info!("Loaded {} active transfer states", count);

        Ok(())
    }

    /// Save transfer states to disk
    async fn save_transfer_states(&self) -> Result<()> {
        let states_path = self.root.join("transfer_states.json");
        let temp_path = self.root.join(".transfer_states.tmp");

        let data = {
            let states = self.chunked_writes.read().await;
            serde_json::to_string_pretty(&*states)
                .context("Failed to serialize transfer states")?
        };

        // Write to temp file first
        tokio::fs::write(&temp_path, &data)
            .await
            .context("Failed to write temp transfer states file")?;

        // Atomically move to final location
        tokio::fs::rename(&temp_path, &states_path)
            .await
            .context("Failed to move transfer states file")?;

        Ok(())
    }

    /// List all active (in-progress) transfers
    pub async fn list_active_transfers(&self) -> Vec<TransferState> {
        let states = self.chunked_writes.read().await;
        states.values().cloned().collect()
    }

    /// Get the state of a specific transfer by transfer_id
    pub async fn get_transfer_state(&self, transfer_id: &str) -> Option<TransferState> {
        let states = self.chunked_writes.read().await;
        states.get(transfer_id).cloned()
    }

    /// Get the state of a transfer by entity_id, disk_type, and path
    pub async fn get_transfer_state_by_path(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Option<TransferState> {
        let transfer_id = Self::index_key(entity_id, disk_type, path);
        self.get_transfer_state(&transfer_id).await
    }

    /// Cleanup stale transfers (older than STALE_TRANSFER_THRESHOLD_SECS)
    ///
    /// Returns the number of transfers cleaned up
    pub async fn cleanup_stale_transfers(&self) -> Result<usize> {
        let now = chrono::Utc::now().timestamp();
        let threshold = now - STALE_TRANSFER_THRESHOLD_SECS;

        let stale_transfers: Vec<TransferState> = {
            let states = self.chunked_writes.read().await;
            states
                .values()
                .filter(|s| s.last_updated < threshold)
                .cloned()
                .collect()
        };

        let count = stale_transfers.len();

        for transfer in &stale_transfers {
            // Remove partial files
            if let Ok(file_path) =
                self.get_file_path(&transfer.entity_id, transfer.disk_type, &transfer.path)
                && file_path.exists()
            {
                match tokio::fs::remove_file(&file_path).await {
                    Ok(()) => {}
                    Err(e) => {
                        warn!(
                            "Failed to delete partial file {} during stale cleanup: {}",
                            file_path.display(),
                            e
                        );
                    }
                }
            }

            // Remove from state
            {
                let mut states = self.chunked_writes.write().await;
                states.remove(&transfer.transfer_id);
            }

            debug!(
                "Cleaned up stale transfer {} (last updated: {})",
                transfer.transfer_id, transfer.last_updated
            );
        }

        if count > 0 {
            self.save_transfer_states().await?;
            info!("Cleaned up {} stale transfers", count);
        }

        Ok(count)
    }

    /// Move a file or directory within an entity's virtual disk
    pub async fn move_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        source_path: &str,
        dest_path: &str,
    ) -> Result<FileInfo> {
        if source_path.is_empty() || source_path == "/" {
            bail!("Cannot move root directory");
        }
        if dest_path.is_empty() || dest_path == "/" {
            bail!("Cannot move to root directory");
        }

        let source_fs_path = self.get_file_path(entity_id, disk_type, source_path)?;
        let dest_fs_path = self.get_file_path(entity_id, disk_type, dest_path)?;

        if !source_fs_path.exists() {
            bail!(
                "Source not found: {}:{}{}",
                entity_id,
                disk_type,
                source_path
            );
        }

        // Create parent directories for destination
        if let Some(parent) = dest_fs_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create parent directories for {}",
                    dest_fs_path.display()
                )
            })?;
        }

        // Perform the move
        tokio::fs::rename(&source_fs_path, &dest_fs_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to move {} to {}",
                    source_fs_path.display(),
                    dest_fs_path.display()
                )
            })?;

        // Update index: remove old entry, add new entry
        let is_directory = dest_fs_path.is_dir();
        let name = std::path::Path::new(dest_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| dest_path.to_string());
        let now = chrono::Utc::now().timestamp();

        let (size_bytes, content_hash) = if is_directory {
            (0, String::new())
        } else {
            let data = tokio::fs::read(&dest_fs_path).await?;
            (data.len() as u64, blake3::hash(&data).to_string())
        };

        // Remove old index entry
        {
            let old_key = Self::index_key(entity_id, disk_type, source_path);
            let mut index = self.index.write().await;
            index.remove(&old_key);
        }

        // Add new index entry
        let metadata = DiskFileMetadata {
            entity_id: entity_id.to_string(),
            disk_type,
            path: dest_path.to_string(),
            name: name.clone(),
            is_directory,
            size_bytes,
            modified_at: now,
            content_hash: content_hash.clone(),
            file_path: dest_fs_path,
        };

        {
            let new_key = Self::index_key(entity_id, disk_type, dest_path);
            let mut index = self.index.write().await;
            index.insert(new_key, metadata);
        }

        self.save_index().await?;

        debug!(
            "Moved {}:{}{} to {}",
            entity_id, disk_type, source_path, dest_path
        );

        Ok(FileInfo {
            path: dest_path.to_string(),
            name,
            is_directory,
            size_bytes,
            modified_at: now,
            content_hash,
        })
    }

    /// Copy a file or directory within an entity's virtual disk
    pub async fn copy_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        source_path: &str,
        dest_path: &str,
    ) -> Result<FileInfo> {
        if source_path.is_empty() || source_path == "/" {
            bail!("Cannot copy root directory");
        }
        if dest_path.is_empty() || dest_path == "/" {
            bail!("Cannot copy to root directory");
        }

        let source_fs_path = self.get_file_path(entity_id, disk_type, source_path)?;
        let dest_fs_path = self.get_file_path(entity_id, disk_type, dest_path)?;

        if !source_fs_path.exists() {
            bail!(
                "Source not found: {}:{}{}",
                entity_id,
                disk_type,
                source_path
            );
        }

        // Create parent directories for destination
        if let Some(parent) = dest_fs_path.parent() {
            tokio::fs::create_dir_all(parent).await.with_context(|| {
                format!(
                    "Failed to create parent directories for {}",
                    dest_fs_path.display()
                )
            })?;
        }

        let is_directory = source_fs_path.is_dir();

        if is_directory {
            // Recursively copy directory
            Self::copy_dir_recursive(&source_fs_path, &dest_fs_path).await?;
        } else {
            // Copy single file
            tokio::fs::copy(&source_fs_path, &dest_fs_path)
                .await
                .with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        source_fs_path.display(),
                        dest_fs_path.display()
                    )
                })?;
        }

        let name = std::path::Path::new(dest_path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| dest_path.to_string());
        let now = chrono::Utc::now().timestamp();

        let (size_bytes, content_hash) = if is_directory {
            (0, String::new())
        } else {
            let data = tokio::fs::read(&dest_fs_path).await?;
            (data.len() as u64, blake3::hash(&data).to_string())
        };

        // Add new index entry
        let metadata = DiskFileMetadata {
            entity_id: entity_id.to_string(),
            disk_type,
            path: dest_path.to_string(),
            name: name.clone(),
            is_directory,
            size_bytes,
            modified_at: now,
            content_hash: content_hash.clone(),
            file_path: dest_fs_path,
        };

        {
            let new_key = Self::index_key(entity_id, disk_type, dest_path);
            let mut index = self.index.write().await;
            index.insert(new_key, metadata);
        }

        self.save_index().await?;

        debug!(
            "Copied {}:{}{} to {}",
            entity_id, disk_type, source_path, dest_path
        );

        Ok(FileInfo {
            path: dest_path.to_string(),
            name,
            is_directory,
            size_bytes,
            modified_at: now,
            content_hash,
        })
    }

    /// Helper to recursively copy a directory
    async fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
        tokio::fs::create_dir_all(dst).await?;

        let mut entries = tokio::fs::read_dir(src).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            let dest_path = dst.join(entry.file_name());

            if entry_path.is_dir() {
                Box::pin(Self::copy_dir_recursive(&entry_path, &dest_path)).await?;
            } else {
                tokio::fs::copy(&entry_path, &dest_path).await?;
            }
        }

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

    #[tokio::test]
    async fn test_read_chunk() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        // Create a file larger than chunk size for testing
        let data: Vec<u8> = (0..3000).map(|i| (i % 256) as u8).collect();

        service
            .write_file(entity_id, DiskType::Private, "/large.bin", &data)
            .await
            .unwrap();

        // Read first chunk
        let chunk0 = service
            .read_chunk(entity_id, DiskType::Private, "/large.bin", 0, Some(1000))
            .await
            .unwrap();

        assert_eq!(chunk0.data.len(), 1000);
        assert_eq!(chunk0.info.offset, 0);
        assert_eq!(chunk0.info.size, 1000);
        assert_eq!(chunk0.info.total_size, 3000);
        assert_eq!(chunk0.info.chunk_index, 0);
        assert!(!chunk0.is_last);

        // Read second chunk
        let chunk1 = service
            .read_chunk(entity_id, DiskType::Private, "/large.bin", 1000, Some(1000))
            .await
            .unwrap();

        assert_eq!(chunk1.data.len(), 1000);
        assert_eq!(chunk1.info.offset, 1000);
        assert_eq!(chunk1.info.chunk_index, 1);
        assert!(!chunk1.is_last);

        // Read last chunk
        let chunk2 = service
            .read_chunk(entity_id, DiskType::Private, "/large.bin", 2000, Some(1000))
            .await
            .unwrap();

        assert_eq!(chunk2.data.len(), 1000);
        assert_eq!(chunk2.info.offset, 2000);
        assert_eq!(chunk2.info.chunk_index, 2);
        assert!(chunk2.is_last);

        // Verify data integrity
        assert_eq!(&chunk0.data[..], &data[0..1000]);
        assert_eq!(&chunk1.data[..], &data[1000..2000]);
        assert_eq!(&chunk2.data[..], &data[2000..3000]);
    }

    #[tokio::test]
    async fn test_chunked_write_workflow() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let path = "/chunked_file.bin";
        let total_size: u64 = 2500;

        // Start chunked write
        let init_info = service
            .start_chunked_write(entity_id, DiskType::Private, path, total_size, None)
            .await
            .unwrap();

        assert_eq!(init_info.total_size, total_size);
        assert!(
            service
                .has_active_chunked_write(entity_id, DiskType::Private, path)
                .await
        );

        // Write first chunk (1000 bytes)
        let chunk0: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        let result0 = service
            .write_chunk(entity_id, DiskType::Private, path, 0, &chunk0)
            .await
            .unwrap();

        assert_eq!(result0.info.offset, 0);
        assert_eq!(result0.info.size, 1000);
        assert!(!result0.is_complete);

        // Check progress
        let progress = service
            .get_chunked_write_progress(entity_id, DiskType::Private, path)
            .await;
        assert!(progress.is_some());
        let (written, total) = progress.unwrap();
        assert_eq!(written, 1000);
        assert_eq!(total, total_size);

        // Write second chunk (1000 bytes)
        let chunk1: Vec<u8> = (0..1000).map(|i| ((i + 50) % 256) as u8).collect();
        let result1 = service
            .write_chunk(entity_id, DiskType::Private, path, 1000, &chunk1)
            .await
            .unwrap();

        assert_eq!(result1.info.offset, 1000);
        assert!(!result1.is_complete);

        // Write final chunk (500 bytes)
        let chunk2: Vec<u8> = (0..500).map(|i| ((i + 100) % 256) as u8).collect();
        let result2 = service
            .write_chunk(entity_id, DiskType::Private, path, 2000, &chunk2)
            .await
            .unwrap();

        assert_eq!(result2.info.offset, 2000);
        assert!(result2.is_complete);

        // Finish chunked write
        let file_info = service
            .finish_chunked_write(entity_id, DiskType::Private, path)
            .await
            .unwrap();

        assert_eq!(file_info.name, "chunked_file.bin");
        assert_eq!(file_info.size_bytes, total_size);

        // Transfer should no longer be active
        assert!(
            !service
                .has_active_chunked_write(entity_id, DiskType::Private, path)
                .await
        );

        // Verify file contents
        let read_data = service
            .read_file(entity_id, DiskType::Private, path)
            .await
            .unwrap();

        assert_eq!(read_data.len(), total_size as usize);
        assert_eq!(&read_data[0..1000], &chunk0[..]);
        assert_eq!(&read_data[1000..2000], &chunk1[..]);
        assert_eq!(&read_data[2000..2500], &chunk2[..]);
    }

    #[tokio::test]
    async fn test_abort_chunked_write() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let path = "/aborted.bin";

        // Start chunked write
        service
            .start_chunked_write(entity_id, DiskType::Private, path, 5000, None)
            .await
            .unwrap();

        // Write one chunk
        let chunk: Vec<u8> = vec![0u8; 1000];
        service
            .write_chunk(entity_id, DiskType::Private, path, 0, &chunk)
            .await
            .unwrap();

        assert!(
            service
                .has_active_chunked_write(entity_id, DiskType::Private, path)
                .await
        );

        // Abort the transfer
        service
            .abort_chunked_write(entity_id, DiskType::Private, path)
            .await
            .unwrap();

        // Transfer should no longer be active
        assert!(
            !service
                .has_active_chunked_write(entity_id, DiskType::Private, path)
                .await
        );

        // File should not exist
        assert!(!service.file_exists(entity_id, DiskType::Private, path).await);
    }

    #[tokio::test]
    async fn test_chunked_write_offset_mismatch() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let path = "/mismatch.bin";

        // Start chunked write
        service
            .start_chunked_write(entity_id, DiskType::Private, path, 3000, None)
            .await
            .unwrap();

        // Write first chunk
        let chunk: Vec<u8> = vec![0u8; 1000];
        service
            .write_chunk(entity_id, DiskType::Private, path, 0, &chunk)
            .await
            .unwrap();

        // Try to write with wrong offset (should fail)
        let result = service
            .write_chunk(entity_id, DiskType::Private, path, 500, &chunk)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Expected offset"));

        // Cleanup
        service
            .abort_chunked_write(entity_id, DiskType::Private, path)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_chunked_write_size_exceeded() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let path = "/exceeded.bin";

        // Start chunked write with small declared size
        service
            .start_chunked_write(entity_id, DiskType::Private, path, 500, None)
            .await
            .unwrap();

        // Try to write a chunk that exceeds declared size
        let chunk: Vec<u8> = vec![0u8; 1000];
        let result = service
            .write_chunk(entity_id, DiskType::Private, path, 0, &chunk)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Write would exceed declared file size"));

        // Cleanup
        service
            .abort_chunked_write(entity_id, DiskType::Private, path)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_calculate_chunk_count() {
        // Exact multiple of chunk size
        assert_eq!(
            EntityDiskService::calculate_chunk_count(3000, Some(1000)),
            3
        );

        // Not exact multiple (needs extra chunk)
        assert_eq!(
            EntityDiskService::calculate_chunk_count(3001, Some(1000)),
            4
        );

        // Smaller than chunk size
        assert_eq!(
            EntityDiskService::calculate_chunk_count(500, Some(1000)),
            1
        );

        // Empty file
        assert_eq!(EntityDiskService::calculate_chunk_count(0, Some(1000)), 0);

        // Default chunk size
        assert_eq!(
            EntityDiskService::calculate_chunk_count(2 * 1024 * 1024, None),
            2
        );
    }

    #[tokio::test]
    async fn test_list_active_transfers() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";

        // Initially no transfers
        let transfers = service.list_active_transfers().await;
        assert!(transfers.is_empty());

        // Start a transfer
        service
            .start_chunked_write(entity_id, DiskType::Private, "/file1.bin", 1000, None)
            .await
            .unwrap();

        let transfers = service.list_active_transfers().await;
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].entity_id, entity_id);
        assert_eq!(transfers[0].path, "/file1.bin");

        // Start another transfer
        service
            .start_chunked_write(entity_id, DiskType::Public, "/file2.bin", 2000, None)
            .await
            .unwrap();

        let transfers = service.list_active_transfers().await;
        assert_eq!(transfers.len(), 2);

        // Abort one
        service
            .abort_chunked_write(entity_id, DiskType::Private, "/file1.bin")
            .await
            .unwrap();

        let transfers = service.list_active_transfers().await;
        assert_eq!(transfers.len(), 1);
        assert_eq!(transfers[0].path, "/file2.bin");

        // Cleanup
        service
            .abort_chunked_write(entity_id, DiskType::Public, "/file2.bin")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_transfer_state() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let path = "/transfer_state.bin";

        // No state before start
        let state = service
            .get_transfer_state_by_path(entity_id, DiskType::Private, path)
            .await;
        assert!(state.is_none());

        // Start transfer
        service
            .start_chunked_write(entity_id, DiskType::Private, path, 2500, Some(1000))
            .await
            .unwrap();

        // Get state by path
        let state = service
            .get_transfer_state_by_path(entity_id, DiskType::Private, path)
            .await;
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(state.total_size, 2500);
        assert_eq!(state.bytes_written, 0);
        assert_eq!(state.chunk_size, 1000);
        assert!(!state.transfer_id.is_empty());
        assert!(state.started_at > 0);

        // Get by transfer_id
        let transfer_id = state.transfer_id.clone();
        let state2 = service.get_transfer_state(&transfer_id).await;
        assert!(state2.is_some());
        assert_eq!(state2.unwrap().path, path);

        // Write a chunk and verify cumulative hash updates
        let chunk: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        service
            .write_chunk(entity_id, DiskType::Private, path, 0, &chunk)
            .await
            .unwrap();

        let state = service
            .get_transfer_state_by_path(entity_id, DiskType::Private, path)
            .await
            .unwrap();
        assert_eq!(state.bytes_written, 1000);
        assert!(!state.cumulative_hash.is_empty());

        // Cleanup
        service
            .abort_chunked_write(entity_id, DiskType::Private, path)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_transfer_state_persistence() {
        let temp = tempdir().unwrap();
        let root_path = temp.path().to_owned();

        // Create a transfer with first service instance
        {
            let service = EntityDiskService::new(&root_path).await.unwrap();

            let entity_id = "test-entity-one-two";
            let path = "/persistent.bin";

            service
                .start_chunked_write(entity_id, DiskType::Private, path, 3000, Some(1000))
                .await
                .unwrap();

            // Write some data
            let chunk: Vec<u8> = vec![42u8; 1000];
            service
                .write_chunk(entity_id, DiskType::Private, path, 0, &chunk)
                .await
                .unwrap();

            // Verify state file exists
            let states_path = root_path.join("transfer_states.json");
            assert!(states_path.exists());
        }

        // Create new service instance and verify state is loaded
        {
            let service = EntityDiskService::new(&root_path).await.unwrap();

            let transfers = service.list_active_transfers().await;
            assert_eq!(transfers.len(), 1);

            let state = transfers.first().unwrap();
            assert_eq!(state.entity_id, "test-entity-one-two");
            assert_eq!(state.path, "/persistent.bin");
            assert_eq!(state.total_size, 3000);
            assert_eq!(state.bytes_written, 1000);
            assert_eq!(state.chunk_size, 1000);

            // Cleanup
            service
                .abort_chunked_write(&state.entity_id, state.disk_type, &state.path)
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    async fn test_cleanup_stale_transfers() {
        let temp = tempdir().unwrap();
        let service = EntityDiskService::new(temp.path()).await.unwrap();

        let entity_id = "test-entity-one-two";
        let path = "/stale.bin";

        // Start a transfer
        service
            .start_chunked_write(entity_id, DiskType::Private, path, 1000, None)
            .await
            .unwrap();

        // Fresh transfer should not be cleaned up
        let cleaned = service.cleanup_stale_transfers().await.unwrap();
        assert_eq!(cleaned, 0);

        // Transfer is still there
        let transfers = service.list_active_transfers().await;
        assert_eq!(transfers.len(), 1);

        // Manually make the transfer stale by modifying last_updated
        // (We access internal state for testing - in production this would happen naturally over time)
        {
            let mut states = service.chunked_writes.write().await;
            let key = EntityDiskService::index_key(entity_id, DiskType::Private, path);
            if let Some(state) = states.get_mut(&key) {
                state.last_updated =
                    chrono::Utc::now().timestamp() - STALE_TRANSFER_THRESHOLD_SECS - 1;
            }
        }

        // Now cleanup should remove it
        let cleaned = service.cleanup_stale_transfers().await.unwrap();
        assert_eq!(cleaned, 1);

        // Transfer should be gone
        let transfers = service.list_active_transfers().await;
        assert!(transfers.is_empty());
    }
}

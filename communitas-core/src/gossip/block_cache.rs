// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Persistent Block Cache with LRU Eviction and Pinning
//!
//! This module provides disk-backed storage for Sites content blocks
//! with the following features:
//! - LRU eviction to manage disk space
//! - Pinning for owned/published content
//! - TTL expiration for fetched content
//! - BLAKE3 hash verification on load
//!
//! ## Storage Layout
//! ```
//! <storage_dir>/blocks/
//!   ├── metadata.json (hash → metadata, pin status, access time)
//!   └── blobs/
//!       ├── 01/23/0123456789abcdef... (first 2 bytes for sharding)
//!       └── ab/cd/abcdefg...
//! ```

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use super::sites::Block;

/// Default cache size limit (1GB)
const DEFAULT_MAX_CACHE_SIZE: u64 = 1024 * 1024 * 1024;

/// Default TTL for unpinned blocks (7 days)
const DEFAULT_BLOCK_TTL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Block metadata for cache management
#[derive(Debug, Clone)]
struct BlockMetadata {
    /// BLAKE3 hash
    hash: [u8; 32],

    /// Size in bytes
    size: u64,

    /// Last access timestamp (Unix milliseconds)
    last_access: u64,

    /// Is this block pinned? (never evict)
    pinned: bool,

    /// Expiration timestamp (Unix milliseconds, 0 = never)
    expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlockMetadataRecord {
    size: u64,
    last_access: u64,
    pinned: bool,
    expires_at: u64,
}

/// Persistent block cache with LRU eviction
pub struct BlockCache {
    /// Storage directory
    storage_dir: PathBuf,

    /// In-memory metadata index (hash → metadata)
    index: Arc<RwLock<std::collections::HashMap<[u8; 32], BlockMetadata>>>,

    /// Current cache size in bytes
    current_size: Arc<RwLock<u64>>,

    /// Maximum cache size
    max_size: u64,

    /// TTL for unpinned blocks
    block_ttl: Duration,
}

impl BlockCache {
    /// Create a new block cache
    ///
    /// # Arguments
    /// * `storage_dir` - Directory for block storage
    /// * `max_size` - Maximum cache size in bytes (default: 1GB)
    pub async fn new(storage_dir: PathBuf, max_size: Option<u64>) -> Result<Self> {
        let max_size = max_size.unwrap_or(DEFAULT_MAX_CACHE_SIZE);

        // Create directory structure
        let blocks_dir = storage_dir.join("blocks");
        let blobs_dir = blocks_dir.join("blobs");
        fs::create_dir_all(&blobs_dir).await?;

        let cache = Self {
            storage_dir,
            index: Arc::new(RwLock::new(std::collections::HashMap::new())),
            current_size: Arc::new(RwLock::new(0)),
            max_size,
            block_ttl: DEFAULT_BLOCK_TTL,
        };

        // Load existing blocks from disk
        cache.load_index().await?;

        Ok(cache)
    }

    fn metadata_path(&self) -> PathBuf {
        self.storage_dir.join("blocks").join("metadata.json")
    }

    async fn load_metadata_records(&self) -> Result<HashMap<[u8; 32], BlockMetadataRecord>> {
        let path = self.metadata_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let data = fs::read_to_string(&path).await?;
        let raw: HashMap<String, BlockMetadataRecord> = serde_json::from_str(&data)?;
        let mut records = HashMap::new();

        for (hash_hex, record) in raw {
            if let Ok(bytes) = hex::decode(&hash_hex)
                && let Ok(hash) = bytes.as_slice().try_into()
            {
                records.insert(hash, record);
            }
        }

        Ok(records)
    }

    async fn persist_metadata(&self) -> Result<()> {
        let index = self.index.read().await;
        let mut raw: HashMap<String, BlockMetadataRecord> = HashMap::new();

        for (hash, meta) in index.iter() {
            raw.insert(
                hex::encode(hash),
                BlockMetadataRecord {
                    size: meta.size,
                    last_access: meta.last_access,
                    pinned: meta.pinned,
                    expires_at: meta.expires_at,
                },
            );
        }

        let data = serde_json::to_string_pretty(&raw)?;
        let path = self.metadata_path();
        fs::write(path, data).await?;
        Ok(())
    }

    /// Load block index from disk
    async fn load_index(&self) -> Result<()> {
        let blobs_dir = self.storage_dir.join("blocks").join("blobs");
        let persisted = match self.load_metadata_records().await {
            Ok(records) => records,
            Err(e) => {
                warn!("Failed to load block metadata: {}", e);
                HashMap::new()
            }
        };

        let mut total_size = 0u64;
        let mut loaded_count = 0;

        // Scan blob directory
        if !blobs_dir.exists() {
            return Ok(());
        }

        // Walk directory tree (two-level sharding: aa/bb/aabb...)
        let mut read_dir = fs::read_dir(&blobs_dir).await?;
        while let Some(l1_entry) = read_dir.next_entry().await? {
            if !l1_entry.file_type().await?.is_dir() {
                continue;
            }

            let mut l2_read_dir = fs::read_dir(l1_entry.path()).await?;
            while let Some(l2_entry) = l2_read_dir.next_entry().await? {
                if !l2_entry.file_type().await?.is_dir() {
                    continue;
                }

                let mut blob_read_dir = fs::read_dir(l2_entry.path()).await?;
                while let Some(blob_entry) = blob_read_dir.next_entry().await? {
                    let filename_os = blob_entry.file_name();
                    let filename = match filename_os.to_str() {
                        Some(name) => name.to_string(),
                        None => continue, // Skip non-UTF8 filenames
                    };

                    if let Some(hash) = self.hash_from_filename(&filename) {
                        let metadata = blob_entry.metadata().await?;
                        let size = metadata.len();
                        let modified = metadata.modified()?;
                        let last_access = modified
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or(std::time::Duration::from_secs(0))
                            .as_millis() as u64;

                        let (pinned, recorded_last_access, recorded_expires) = persisted
                            .get(&hash)
                            .map(|record| (record.pinned, record.last_access, record.expires_at))
                            .unwrap_or((
                                false,
                                last_access,
                                last_access + self.block_ttl.as_millis() as u64,
                            ));

                        let block_meta = BlockMetadata {
                            hash,
                            size,
                            last_access: recorded_last_access,
                            pinned,
                            expires_at: if pinned { 0 } else { recorded_expires },
                        };

                        let mut index = self.index.write().await;
                        index.insert(hash, block_meta);
                        total_size += size;
                        loaded_count += 1;
                    }
                }
            }
        }

        *self.current_size.write().await = total_size;
        debug!(
            "Loaded {} blocks ({} bytes) from cache",
            loaded_count, total_size
        );

        self.persist_metadata().await?;

        Ok(())
    }

    /// Convert filename to hash
    fn hash_from_filename(&self, filename: &str) -> Option<[u8; 32]> {
        hex::decode(filename).ok()?.as_slice().try_into().ok()
    }

    /// Get block file path
    fn block_path(&self, hash: &[u8; 32]) -> PathBuf {
        let hex = hex::encode(hash);
        let l1 = &hex[0..2];
        let l2 = &hex[2..4];
        self.storage_dir
            .join("blocks")
            .join("blobs")
            .join(l1)
            .join(l2)
            .join(&hex)
    }

    /// Store a block in cache
    ///
    /// # Arguments
    /// * `block` - Block to store
    /// * `pinned` - If true, block will never be evicted
    pub async fn store(&self, block: Block, pinned: bool) -> Result<()> {
        // Verify block hash
        if !block.verify() {
            anyhow::bail!("Block hash verification failed");
        }

        let hash = block.hash;
        let size = block.content.len() as u64;

        // Check if we need to evict blocks (unless pinned)
        if !pinned {
            self.maybe_evict(size).await?;
        }

        // Write block to disk
        let path = self.block_path(&hash);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&path, &block.content).await?;

        // Update index
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let metadata = BlockMetadata {
            hash,
            size,
            last_access: now,
            pinned,
            expires_at: if pinned {
                0
            } else {
                now + self.block_ttl.as_millis() as u64
            },
        };

        let mut index = self.index.write().await;
        index.insert(hash, metadata);

        *self.current_size.write().await += size;
        drop(index);

        self.persist_metadata().await?;

        debug!(
            "Stored block {} ({} bytes, pinned: {})",
            hex::encode(hash),
            size,
            pinned
        );

        Ok(())
    }

    /// Get a block from cache
    pub async fn get(&self, hash: &[u8; 32]) -> Result<Option<Block>> {
        // Check index
        let mut index = self.index.write().await;
        let metadata = match index.get_mut(hash) {
            Some(m) => m,
            None => return Ok(None),
        };

        // Check expiration
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        if metadata.expires_at > 0 && metadata.expires_at < now {
            // Expired, remove it
            drop(index);
            self.remove(hash).await?;
            return Ok(None);
        }

        // Update last access time
        metadata.last_access = now;
        drop(index);

        // Read from disk
        let path = self.block_path(hash);
        let content = match fs::read(&path).await {
            Ok(c) => c,
            Err(_) => {
                // File missing, remove from index
                self.remove(hash).await?;
                return Ok(None);
            }
        };

        let block = Block::new(content);

        // Verify hash matches
        if block.hash != *hash {
            warn!("Block hash mismatch, removing from cache");
            self.remove(hash).await?;
            return Ok(None);
        }

        self.persist_metadata().await?;

        Ok(Some(block))
    }

    /// Pin a block (prevents eviction)
    pub async fn pin(&self, hash: &[u8; 32]) -> Result<()> {
        let mut index = self.index.write().await;
        if let Some(metadata) = index.get_mut(hash) {
            metadata.pinned = true;
            metadata.expires_at = 0; // Never expires
        }
        drop(index);
        self.persist_metadata().await?;
        Ok(())
    }

    /// Unpin a block
    pub async fn unpin(&self, hash: &[u8; 32]) -> Result<()> {
        let mut index = self.index.write().await;
        if let Some(metadata) = index.get_mut(hash) {
            metadata.pinned = false;
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
            metadata.expires_at = now + self.block_ttl.as_millis() as u64;
        }
        drop(index);
        self.persist_metadata().await?;
        Ok(())
    }

    /// Remove a block from cache
    async fn remove(&self, hash: &[u8; 32]) -> Result<()> {
        let mut index = self.index.write().await;
        if let Some(metadata) = index.remove(hash) {
            *self.current_size.write().await -= metadata.size;

            // Delete file
            let path = self.block_path(hash);
            if path.exists() {
                fs::remove_file(&path).await.ok();
            }
        }
        drop(index);
        self.persist_metadata().await?;
        Ok(())
    }

    /// Evict blocks to make space
    ///
    /// Uses LRU policy (oldest accessed first).
    /// Pinned blocks are never evicted.
    async fn maybe_evict(&self, needed_size: u64) -> Result<()> {
        let current = *self.current_size.read().await;

        // Check if we need to evict
        if current + needed_size <= self.max_size {
            return Ok(());
        }

        let to_free = (current + needed_size) - self.max_size;
        let mut freed = 0u64;

        // Get all unpinned blocks sorted by access time
        let index = self.index.read().await;
        let mut candidates: Vec<_> = index.values().filter(|m| !m.pinned).cloned().collect();
        drop(index);

        // Sort by last_access (oldest first)
        candidates.sort_by_key(|m| m.last_access);

        // Evict until we've freed enough space
        for metadata in candidates {
            if freed >= to_free {
                break;
            }

            self.remove(&metadata.hash).await?;
            freed += metadata.size;
            debug!(
                "Evicted block {} ({} bytes)",
                hex::encode(metadata.hash),
                metadata.size
            );
        }

        if freed < to_free {
            anyhow::bail!(
                "Could not free enough space (needed {}, freed {})",
                to_free,
                freed
            );
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let index = self.index.read().await;
        let current_size = *self.current_size.read().await;

        let pinned_count = index.values().filter(|m| m.pinned).count();
        let total_count = index.len();

        CacheStats {
            total_blocks: total_count,
            pinned_blocks: pinned_count,
            total_size_bytes: current_size,
            max_size_bytes: self.max_size,
        }
    }

    /// Expire old blocks
    pub async fn expire_old_blocks(&self) -> Result<usize> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;

        let index = self.index.read().await;
        let expired: Vec<[u8; 32]> = index
            .values()
            .filter(|m| !m.pinned && m.expires_at > 0 && m.expires_at < now)
            .map(|m| m.hash)
            .collect();
        drop(index);

        let count = expired.len();
        for hash in expired {
            self.remove(&hash).await?;
        }

        if count > 0 {
            debug!("Expired {} old blocks", count);
        }

        Ok(count)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_blocks: usize,
    pub pinned_blocks: usize,
    pub total_size_bytes: u64,
    pub max_size_bytes: u64,
}

impl CacheStats {
    /// Get usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.max_size_bytes == 0 {
            0.0
        } else {
            (self.total_size_bytes as f64 / self.max_size_bytes as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_cache() -> BlockCache {
        let temp_dir = tempfile::tempdir().unwrap();
        BlockCache::new(temp_dir.path().to_path_buf(), Some(1024 * 1024)) // 1MB for tests
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = create_test_cache().await;
        let stats = cache.stats().await;
        assert_eq!(stats.total_blocks, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }

    #[tokio::test]
    async fn test_store_and_get_block() {
        let cache = create_test_cache().await;

        let content = b"Test content".to_vec();
        let block = Block::new(content.clone());
        let hash = block.hash;

        // Store block
        cache.store(block.clone(), false).await.unwrap();

        // Retrieve block
        let retrieved = cache.get(&hash).await.unwrap().expect("Block should exist");
        assert_eq!(retrieved.content, content);
        assert_eq!(retrieved.hash, hash);
    }

    #[tokio::test]
    async fn test_pinned_block_not_evicted() {
        let cache = create_test_cache().await;

        // Store pinned block
        let pinned_content = b"Important content".to_vec();
        let pinned_block = Block::new(pinned_content.clone());
        let pinned_hash = pinned_block.hash;

        cache.store(pinned_block, true).await.unwrap();

        // Fill cache to trigger eviction
        for i in 0..100 {
            let _content = format!("Filler content {}", i).into_bytes();
            let block = Block::new(vec![0u8; 20_000]); // 20KB each
            cache.store(block, false).await.unwrap();
        }

        // Pinned block should still exist
        let retrieved = cache.get(&pinned_hash).await.unwrap();
        assert!(retrieved.is_some(), "Pinned block should not be evicted");
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = create_test_cache().await;

        // Store blocks
        let block1 = Block::new(vec![1u8; 600_000]); // 600KB
        let block2 = Block::new(vec![2u8; 600_000]); // 600KB

        cache.store(block1.clone(), false).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        cache.store(block2.clone(), false).await.unwrap();

        // Cache is 1.2MB, max is 1MB, so should evict block1
        let stats = cache.stats().await;
        assert!(stats.total_size_bytes <= cache.max_size);

        // block1 should be evicted (older)
        let retrieved1 = cache.get(&block1.hash).await.unwrap();
        assert!(retrieved1.is_none(), "Older block should be evicted");

        // block2 should still exist
        let retrieved2 = cache.get(&block2.hash).await.unwrap();
        assert!(retrieved2.is_some(), "Newer block should exist");
    }

    #[tokio::test]
    async fn test_pin_and_unpin() {
        let cache = create_test_cache().await;

        let block = Block::new(b"Test".to_vec());
        let hash = block.hash;

        cache.store(block, false).await.unwrap();

        // Pin it
        cache.pin(&hash).await.unwrap();

        let index = cache.index.read().await;
        assert!(index.get(&hash).unwrap().pinned);
        drop(index);

        // Unpin it
        cache.unpin(&hash).await.unwrap();

        let index = cache.index.read().await;
        assert!(!index.get(&hash).unwrap().pinned);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = create_test_cache().await;

        let block = Block::new(vec![0u8; 500_000]); // 500KB
        cache.store(block, true).await.unwrap();

        let stats = cache.stats().await;
        assert_eq!(stats.total_blocks, 1);
        assert_eq!(stats.pinned_blocks, 1);
        assert_eq!(stats.total_size_bytes, 500_000);
        assert!(stats.usage_percent() > 0.0);
    }
}

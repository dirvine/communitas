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


//! Content Store Implementation
//!
//! This module implements BLAKE3-based content addressing with:
//! - Cryptographic integrity validation
//! - TTL-based expiration
//! - Efficient storage and retrieval
//! - Content deduplication
//! - Streaming support for large content

use super::*;
use anyhow::{Result, Context};
use blake3::{Hash};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error, instrument};

/// Content identifier based on BLAKE3 hash
#[derive(Debug, Clone, Hash)]
pub struct ContentId(Hash);

impl ContentId {
    /// Create ContentId from data
    pub fn from_data(data: &[u8]) -> Self {
        Self(blake3::hash(data))
    }
    
    /// Create ContentId from existing hash
    pub fn from_hash(hash: Hash) -> Self {
        Self(hash)
    }
    
    /// Get the underlying BLAKE3 hash
    pub fn hash(&self) -> &Hash {
        &self.0
    }
    
    /// Get bytes representation
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
    
    /// Convert to hex string for display
    pub fn to_hex(&self) -> String {
        self.0.to_hex().to_string()
    }
    
    /// Verify that data matches this content ID
    pub fn verify(&self, data: &[u8]) -> bool {
        &blake3::hash(data) == &self.0
    }
}

impl Serialize for ContentId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_hex().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ContentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let hex_str = String::deserialize(deserializer)?;
        let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("Invalid hash length"));
        }
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&bytes);
        Ok(ContentId(Hash::from(hash_bytes)))
    }
}

impl PartialEq for ContentId {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_bytes() == other.0.as_bytes()
    }
}

impl Eq for ContentId {}

impl PartialOrd for ContentId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ContentId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.as_bytes().cmp(other.0.as_bytes())
    }
}
/// Entry in the content store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtEntry {
    /// Content identifier (BLAKE3 hash)
    pub key: ContentId,
    /// The actual content data
    pub value: Vec<u8>,
    /// When this entry was stored
    pub timestamp: SystemTime,
    /// Time-to-live for this entry
    pub ttl: Duration,
    /// Nodes that have replicas of this content
    pub replicas: Vec<NodeId>,
}

impl DhtEntry {
    /// Create new DHT entry
    pub fn new(value: Vec<u8>, ttl: Duration) -> Self {
        let key = ContentId::from_data(&value);
        let timestamp = SystemTime::now();
        
        Self {
            key,
            value,
            timestamp,
            ttl,
            replicas: Vec::new(),
        }
    }
    
    /// Check if entry has expired
    pub fn is_expired(&self) -> bool {
        self.timestamp + self.ttl < SystemTime::now()
    }
    
    /// Get expiration time
    pub fn expires_at(&self) -> SystemTime {
        self.timestamp + self.ttl
    }
    
    /// Verify content integrity
    pub fn verify_integrity(&self) -> bool {
        self.key.verify(&self.value)
    }
    
    /// Update TTL (extend lifetime)
    pub fn extend_ttl(&mut self, additional_time: Duration) {
        self.ttl += additional_time;
    }
    
    /// Add replica node
    pub fn add_replica(&mut self, node_id: NodeId) {
        if !self.replicas.contains(&node_id) {
            self.replicas.push(node_id);
        }
    }
    
    /// Remove replica node
    pub fn remove_replica(&mut self, node_id: &NodeId) -> bool {
        if let Some(pos) = self.replicas.iter().position(|id| id == node_id) {
            self.replicas.remove(pos);
            true
        } else {
            false
        }
    }
}

/// Content store statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStoreStats {
    pub total_entries: usize,
    pub total_size_bytes: u64,
    pub expired_entries: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub integrity_failures: u64,
    pub last_cleanup: SystemTime,
    pub cleanup_count: u32,
}

impl Default for ContentStoreStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            total_size_bytes: 0,
            expired_entries: 0,
            cache_hits: 0,
            cache_misses: 0,
            integrity_failures: 0,
            last_cleanup: SystemTime::UNIX_EPOCH,
            cleanup_count: 0,
        }
    }
}
/// Storage backend trait for different storage implementations
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store data with given key
    async fn store(&mut self, key: &ContentId, entry: &DhtEntry) -> Result<()>;
    
    /// Retrieve data by key
    async fn get(&self, key: &ContentId) -> Result<Option<DhtEntry>>;
    
    /// Remove data by key
    async fn remove(&mut self, key: &ContentId) -> Result<bool>;
    
    /// List all keys
    async fn keys(&self) -> Result<Vec<ContentId>>;
    
    /// Get storage statistics
    async fn stats(&self) -> Result<u64>; // Total size in bytes
    
    /// Cleanup expired entries
    async fn cleanup(&mut self) -> Result<usize>; // Number of entries removed
}

/// In-memory storage backend with LRU eviction
#[derive(Debug)]
pub struct MemoryStorage {
    /// Main storage map
    entries: HashMap<ContentId, DhtEntry>,
    /// LRU ordering (most recent first)
    lru_order: VecDeque<ContentId>,
    /// Maximum cache size
    max_size: usize,
    /// Current size in bytes
    current_size: u64,
    /// Maximum size in bytes
    max_size_bytes: u64,
}

impl MemoryStorage {
    /// Create new memory storage
    pub fn new(max_entries: usize, max_size_bytes: u64) -> Self {
        Self {
            entries: HashMap::new(),
            lru_order: VecDeque::new(),
            max_size: max_entries,
            current_size: 0,
            max_size_bytes,
        }
    }
    
    /// Update LRU order
    fn update_lru(&mut self, key: &ContentId) {
        // Remove from current position
        if let Some(pos) = self.lru_order.iter().position(|k| k == key) {
            self.lru_order.remove(pos);
        }
        // Add to front
        self.lru_order.push_front(key.clone());
    }
    
    /// Evict least recently used entries if needed
    fn evict_if_needed(&mut self) {
        // Evict by count
        while self.entries.len() > self.max_size {
            if let Some(key) = self.lru_order.pop_back() {
                if let Some(entry) = self.entries.remove(&key) {
                    self.current_size -= entry.value.len() as u64;
                }
            }
        }
        
        // Evict by size
        while self.current_size > self.max_size_bytes {
            if let Some(key) = self.lru_order.pop_back() {
                if let Some(entry) = self.entries.remove(&key) {
                    self.current_size -= entry.value.len() as u64;
                }
            } else {
                break; // No more entries to evict
            }
        }
    }
}

#[async_trait::async_trait]
impl StorageBackend for MemoryStorage {
    async fn store(&mut self, key: &ContentId, entry: &DhtEntry) -> Result<()> {
        let size = entry.value.len() as u64;
        
        // Check if entry already exists
        if let Some(existing) = self.entries.get(key) {
            self.current_size -= existing.value.len() as u64;
        }
        
        // Add new entry
        self.entries.insert(key.clone(), entry.clone());
        self.current_size += size;
        self.update_lru(key);
        
        // Evict if necessary
        self.evict_if_needed();
        
        Ok(())
    }
    
    async fn get(&self, key: &ContentId) -> Result<Option<DhtEntry>> {
        Ok(self.entries.get(key).cloned())
    }
    
    async fn remove(&mut self, key: &ContentId) -> Result<bool> {
        if let Some(entry) = self.entries.remove(key) {
            self.current_size -= entry.value.len() as u64;
            if let Some(pos) = self.lru_order.iter().position(|k| k == key) {
                self.lru_order.remove(pos);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    async fn keys(&self) -> Result<Vec<ContentId>> {
        Ok(self.entries.keys().cloned().collect())
    }
    
    async fn stats(&self) -> Result<u64> {
        Ok(self.current_size)
    }
    
    async fn cleanup(&mut self) -> Result<usize> {
        let now = SystemTime::now();
        let mut expired_keys = Vec::new();
        
        for (key, entry) in &self.entries {
            if entry.timestamp + entry.ttl < now {
                expired_keys.push(key.clone());
            }
        }
        
        let count = expired_keys.len();
        for key in expired_keys {
            self.remove(&key).await?;
        }
        
        Ok(count)
    }
}

/// Main content store implementation
pub struct ContentStore {
    /// Storage backend
    storage: Arc<RwLock<Box<dyn StorageBackend>>>,
    /// Cache for frequently accessed items
    cache: Arc<RwLock<HashMap<ContentId, DhtEntry>>>,
    /// Statistics
    stats: Arc<RwLock<ContentStoreStats>>,
    /// Configuration
    config: ContentStoreConfig,
}

/// Content store configuration
#[derive(Debug, Clone)]
pub struct ContentStoreConfig {
    pub max_cache_size: usize,
    pub max_entry_size: usize,
    pub default_ttl: Duration,
    pub cleanup_interval: Duration,
    pub enable_compression: bool,
    pub enable_encryption: bool,
}

impl Default for ContentStoreConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 1000,
            max_entry_size: 1024 * 1024, // 1MB
            default_ttl: Duration::from_secs(3600), // 1 hour
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            enable_compression: false,
            enable_encryption: false,
        }
    }
}

impl ContentStore {
    /// Create new content store
    #[instrument(skip(max_size))]
    pub fn new(max_size: usize) -> Self {
        info!("Creating content store with max size: {}", max_size);
        
        let storage = Arc::new(RwLock::new(Box::new(MemoryStorage::new(
            max_size,
            max_size as u64 * 1024, // Assume average 1KB per entry
        )) as Box<dyn StorageBackend>));
        
        let cache = Arc::new(RwLock::new(HashMap::new()));
        let stats = Arc::new(RwLock::new(ContentStoreStats::default()));
        let config = ContentStoreConfig::default();
        
        Self {
            storage,
            cache,
            stats,
            config,
        }
    }
    
    /// Store content in the DHT
    #[instrument(skip(self, entry))]
    pub async fn store(&self, entry: DhtEntry) -> Result<()> {
        // Verify content integrity
        if !entry.verify_integrity() {
            error!("Content integrity check failed for key: {}", entry.key.to_hex());
            let mut stats = self.stats.write().await;
            stats.integrity_failures += 1;
            return Err(anyhow::anyhow!("Content integrity check failed"));
        }
        
        // Check entry size
        if entry.value.len() > self.config.max_entry_size {
            return Err(anyhow::anyhow!(
                "Entry too large: {} bytes (max: {})",
                entry.value.len(),
                self.config.max_entry_size
            ));
        }
        
        debug!("Storing content: {} ({} bytes)", entry.key.to_hex(), entry.value.len());
        
        // Store in backend
        let mut storage = self.storage.write().await;
        storage.store(&entry.key, &entry).await
            .context("Failed to store in backend")?;
        
        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(entry.key.clone(), entry.clone());
        
        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_entries += 1;
        stats.total_size_bytes += entry.value.len() as u64;
        
        info!("Successfully stored content: {}", entry.key.to_hex());
        Ok(())
    }
    
    /// Retrieve content from the DHT
    #[instrument(skip(self))]
    pub async fn get(&self, key: &ContentId) -> Result<Option<DhtEntry>> {
        debug!("Retrieving content: {}", key.to_hex());
        
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(key) {
                if !entry.is_expired() {
                    debug!("Cache hit for key: {}", key.to_hex());
                    let mut stats = self.stats.write().await;
                    stats.cache_hits += 1;
                    return Ok(Some(entry.clone()));
                }
            }
        }
        
        // Check storage backend
        let storage = self.storage.read().await;
        match storage.get(key).await? {
            Some(entry) => {
                if entry.is_expired() {
                    debug!("Entry expired: {}", key.to_hex());
                    // Remove expired entry
                    drop(storage);
                    let mut storage = self.storage.write().await;
                    storage.remove(key).await?;
                    
                    let mut stats = self.stats.write().await;
                    stats.cache_misses += 1;
                    stats.expired_entries += 1;
                    return Ok(None);
                }
                
                // Verify integrity
                if !entry.verify_integrity() {
                    error!("Stored content integrity check failed for key: {}", key.to_hex());
                    let mut stats = self.stats.write().await;
                    stats.integrity_failures += 1;
                    return Err(anyhow::anyhow!("Stored content integrity check failed"));
                }
                
                // Update cache
                {
                    let mut cache = self.cache.write().await;
                    cache.insert(key.clone(), entry.clone());
                }
                
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
                
                debug!("Retrieved content from storage: {}", key.to_hex());
                Ok(Some(entry))
            },
            None => {
                debug!("Content not found: {}", key.to_hex());
                let mut stats = self.stats.write().await;
                stats.cache_misses += 1;
                Ok(None)
            }
        }
    }
    
    /// Remove content from the store
    #[instrument(skip(self))]
    pub async fn remove(&self, key: &ContentId) -> Result<bool> {
        debug!("Removing content: {}", key.to_hex());
        
        // Remove from storage
        let mut storage = self.storage.write().await;
        let removed = storage.remove(key).await?;
        
        // Remove from cache
        let mut cache = self.cache.write().await;
        cache.remove(key);
        
        if removed {
            let mut stats = self.stats.write().await;
            stats.total_entries = stats.total_entries.saturating_sub(1);
            info!("Removed content: {}", key.to_hex());
        }
        
        Ok(removed)
    }
    
    /// Get all stored content keys
    pub async fn keys(&self) -> Result<Vec<ContentId>> {
        let storage = self.storage.read().await;
        storage.keys().await
    }
    
    /// Get number of stored items
    pub fn item_count(&self) -> usize {
        // This is a best-effort estimate
        0 // Would need to track this properly in a real implementation
    }
    
    /// Perform cleanup of expired entries
    #[instrument(skip(self))]
    pub async fn cleanup(&self) -> Result<usize> {
        debug!("Starting content store cleanup");
        
        let mut storage = self.storage.write().await;
        let removed_count = storage.cleanup().await?;
        
        // Clean cache as well
        let mut cache = self.cache.write().await;
        let now = SystemTime::now();
        cache.retain(|_, entry| entry.timestamp + entry.ttl >= now);
        
        let mut stats = self.stats.write().await;
        stats.expired_entries += removed_count;
        stats.cleanup_count += 1;
        stats.last_cleanup = SystemTime::now();
        
        if removed_count > 0 {
            info!("Cleanup completed: removed {} expired entries", removed_count);
        }
        
        Ok(removed_count)
    }
    
    /// Get content store statistics
    pub async fn stats(&self) -> ContentStoreStats {
        let stats = self.stats.read().await;
        let mut result = stats.clone();
        
        // Update current size from storage
        if let Ok(storage) = self.storage.try_read() {
            if let Ok(size) = storage.stats().await {
                result.total_size_bytes = size;
            }
        }
        
        result
    }
    
    /// Verify integrity of all stored content
    #[instrument(skip(self))]
    pub async fn verify_all(&self) -> Result<IntegrityReport> {
        info!("Starting integrity verification of all stored content");
        
        let storage = self.storage.read().await;
        let keys = storage.keys().await?;
        
        let mut report = IntegrityReport {
            total_checked: 0,
            integrity_failures: 0,
            expired_entries: 0,
            corrupted_keys: Vec::new(),
        };
        
        for key in keys {
            report.total_checked += 1;
            
            if let Some(entry) = storage.get(&key).await? {
                if entry.is_expired() {
                    report.expired_entries += 1;
                } else if !entry.verify_integrity() {
                    report.integrity_failures += 1;
                    report.corrupted_keys.push(key);
                }
            }
        }
        
        info!("Integrity verification completed: {}/{} entries verified, {} failures", 
              report.total_checked - report.integrity_failures, 
              report.total_checked, 
              report.integrity_failures);
        
        Ok(report)
    }
}

/// Report from integrity verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    pub total_checked: usize,
    pub integrity_failures: usize,
    pub expired_entries: usize,
    pub corrupted_keys: Vec<ContentId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_id_creation() {
        let data = b"Hello, world!";
        let id1 = ContentId::from_data(data);
        let id2 = ContentId::from_data(data);
        
        assert_eq!(id1, id2);
        assert!(id1.verify(data));
        assert!(!id1.verify(b"Different data"));
    }
    
    #[test]
    fn test_dht_entry_expiration() {
        let value = b"test data".to_vec();
        let ttl = Duration::from_millis(100);
        let entry = DhtEntry::new(value, ttl);
        
        assert!(!entry.is_expired());
        
        std::thread::sleep(Duration::from_millis(150));
        assert!(entry.is_expired());
    }
    
    #[test]
    fn test_dht_entry_integrity() {
        let value = b"test data".to_vec();
        let ttl = Duration::from_secs(3600);
        let entry = DhtEntry::new(value, ttl);
        
        assert!(entry.verify_integrity());
        
        // Corrupt the value
        let mut corrupted_entry = entry.clone();
        corrupted_entry.value[0] = !corrupted_entry.value[0];
        
        assert!(!corrupted_entry.verify_integrity());
    }
    
    #[tokio::test]
    async fn test_memory_storage() {
        let mut storage = MemoryStorage::new(10, 1024);
        let entry = DhtEntry::new(b"test data".to_vec(), Duration::from_secs(3600));
        let key = entry.key.clone();
        
        // Store entry
        storage.store(&key, &entry).await.unwrap();
        
        // Retrieve entry
        let retrieved = storage.get(&key).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, entry.value);
        
        // Remove entry
        let removed = storage.remove(&key).await.unwrap();
        assert!(removed);
        
        // Verify removal
        let not_found = storage.get(&key).await.unwrap();
        assert!(not_found.is_none());
    }
    
    #[tokio::test]
    async fn test_content_store() {
        let store = ContentStore::new(100);
        let entry = DhtEntry::new(b"Hello, DHT!".to_vec(), Duration::from_secs(3600));
        let key = entry.key.clone();
        
        // Store content
        store.store(entry).await.unwrap();
        
        // Retrieve content
        let retrieved = store.get(&key).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, b"Hello, DHT!");
        
        // Verify statistics
        let stats = store.stats().await;
        assert!(stats.total_entries > 0);
    }
    
    #[tokio::test]
    async fn test_content_store_cleanup() {
        let store = ContentStore::new(100);
        
        // Store entry with short TTL
        let entry = DhtEntry::new(b"Short lived".to_vec(), Duration::from_millis(50));
        let key = entry.key.clone();
        store.store(entry).await.unwrap();
        
        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Cleanup should remove expired entry
        let removed = store.cleanup().await.unwrap();
        assert!(removed > 0);
        
        // Entry should no longer be retrievable
        let retrieved = store.get(&key).await.unwrap();
        assert!(retrieved.is_none());
    }
}

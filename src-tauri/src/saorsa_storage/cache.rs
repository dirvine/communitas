/**
 * Saorsa Storage System - Local Caching Layer
 * Implements LRU caching with compression and integrity verification
 */

use crate::saorsa_storage::errors::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub data: Vec<u8>,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub size: usize,
    pub ttl: Option<Duration>,
    pub is_compressed: bool,
    pub checksum: String,
}

impl CacheEntry {
    /// Check if cache entry has expired
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            self.created_at.elapsed() > ttl
        } else {
            false
        }
    }

    /// Update access metadata
    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
        self.access_count += 1;
    }

    /// Calculate entry score for LRU eviction
    pub fn calculate_score(&self) -> f64 {
        let age_factor = self.last_accessed.elapsed().as_secs_f64();
        let frequency_factor = 1.0 / (self.access_count as f64 + 1.0);
        let size_factor = self.size as f64 / 1024.0; // Size in KB
        
        // Higher score = more likely to be evicted
        age_factor * frequency_factor * (1.0 + size_factor / 1024.0)
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub compression_ratio: f32,
    pub avg_access_time_ms: f64,
    pub last_updated: DateTime<Utc>,
}

impl CacheStats {
    /// Calculate hit ratio
    pub fn hit_ratio(&self) -> f64 {
        if self.hit_count + self.miss_count == 0 {
            0.0
        } else {
            self.hit_count as f64 / (self.hit_count + self.miss_count) as f64
        }
    }

    /// Calculate average entry size
    pub fn avg_entry_size(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            self.total_size_bytes as f64 / self.total_entries as f64
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size_bytes: usize,
    pub max_entries: usize,
    pub default_ttl: Option<Duration>,
    pub compress_threshold: usize,
    pub cleanup_interval: Duration,
    pub enable_integrity_check: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: 100 * 1024 * 1024, // 100MB
            max_entries: 10000,
            default_ttl: Some(Duration::from_secs(3600)), // 1 hour
            compress_threshold: 4096, // 4KB
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            enable_integrity_check: true,
        }
    }
}

/// Local storage cache with LRU eviction
pub struct StorageCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    lru_order: Arc<RwLock<VecDeque<String>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl StorageCache {
    /// Create a new storage cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new storage cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            lru_order: Arc::new(RwLock::new(VecDeque::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats {
                total_entries: 0,
                total_size_bytes: 0,
                hit_count: 0,
                miss_count: 0,
                eviction_count: 0,
                compression_ratio: 1.0,
                avg_access_time_ms: 0.0,
                last_updated: Utc::now(),
            })),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Store data in cache
    pub async fn put(
        &self,
        key: &str,
        data: Vec<u8>,
        ttl: Option<Duration>,
    ) -> CacheResult<()> {
        let start_time = Instant::now();
        
        if key.is_empty() {
            return Err(CacheError::InvalidEntry);
        }

        // Store original size before potential move
        let original_size = data.len();

        // Compress data if it exceeds threshold
        let (final_data, is_compressed) = if data.len() > self.config.compress_threshold {
            (self.compress_data(&data)?, true)
        } else {
            (data, false)
        };

        // Calculate checksum for integrity
        let checksum = if self.config.enable_integrity_check {
            self.calculate_checksum(&final_data)
        } else {
            String::new()
        };

        let entry = CacheEntry {
            key: key.to_string(),
            data: final_data,
            created_at: start_time,
            last_accessed: start_time,
            access_count: 0,
            size: original_size, // Store original size
            ttl: ttl.or(self.config.default_ttl),
            is_compressed,
            checksum,
        };

        // Check if we need to make space
        self.ensure_capacity(entry.size).await?;

        // Insert entry
        let mut entries = self.entries.write().await;
        let mut lru_order = self.lru_order.write().await;
        
        // Remove existing entry if present
        if entries.contains_key(key) {
            self.remove_from_lru_order(&mut lru_order, key);
        }

        entries.insert(key.to_string(), entry);
        lru_order.push_front(key.to_string());

        // Update statistics
        self.update_stats_after_put(original_size, is_compressed).await;

        // Cleanup if needed
        self.cleanup_if_needed().await?;

        Ok(())
    }

    /// Retrieve data from cache
    pub async fn get(&self, key: &str) -> CacheResult<Vec<u8>> {
        let start_time = Instant::now();

        let mut entries = self.entries.write().await;
        let mut lru_order = self.lru_order.write().await;

        if let Some(entry) = entries.get_mut(key) {
            // Check if entry has expired
            if entry.is_expired() {
                self.remove_from_lru_order(&mut lru_order, key);
                entries.remove(key);
                drop(entries);
                drop(lru_order);
                
                self.update_stats_miss().await;
                return Err(CacheError::CacheMiss {
                    key: key.to_string(),
                });
            }

            // Verify integrity if enabled
            if self.config.enable_integrity_check && !entry.checksum.is_empty() {
                let current_checksum = self.calculate_checksum(&entry.data);
                if current_checksum != entry.checksum {
                    // Data corruption detected
                    self.remove_from_lru_order(&mut lru_order, key);
                    entries.remove(key);
                    drop(entries);
                    drop(lru_order);
                    
                    return Err(CacheError::CacheCorruption);
                }
            }

            // Update access information
            entry.touch();
            
            // Move to front of LRU
            self.remove_from_lru_order(&mut lru_order, key);
            lru_order.push_front(key.to_string());

            // Decompress if needed
            let data = if entry.is_compressed {
                self.decompress_data(&entry.data)?
            } else {
                entry.data.clone()
            };

            drop(entries);
            drop(lru_order);

            // Update hit statistics
            self.update_stats_hit(start_time.elapsed()).await;

            Ok(data)
        } else {
            drop(entries);
            drop(lru_order);
            
            self.update_stats_miss().await;
            Err(CacheError::CacheMiss {
                key: key.to_string(),
            })
        }
    }

    /// Remove entry from cache
    pub async fn remove(&self, key: &str) -> CacheResult<bool> {
        let mut entries = self.entries.write().await;
        let mut lru_order = self.lru_order.write().await;

        if let Some(entry) = entries.remove(key) {
            self.remove_from_lru_order(&mut lru_order, key);
            
            // Update stats
            let mut stats = self.stats.write().await;
            stats.total_entries = stats.total_entries.saturating_sub(1);
            stats.total_size_bytes = stats.total_size_bytes.saturating_sub(entry.size);
            stats.last_updated = Utc::now();
            
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if key exists in cache
    pub async fn contains(&self, key: &str) -> bool {
        let entries = self.entries.read().await;
        if let Some(entry) = entries.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) -> CacheResult<()> {
        let mut entries = self.entries.write().await;
        let mut lru_order = self.lru_order.write().await;
        
        entries.clear();
        lru_order.clear();

        // Reset stats
        let mut stats = self.stats.write().await;
        stats.total_entries = 0;
        stats.total_size_bytes = 0;
        stats.last_updated = Utc::now();

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Get cache utilization percentage
    pub async fn get_utilization(&self) -> f64 {
        let stats = self.stats.read().await;
        (stats.total_size_bytes as f64 / self.config.max_size_bytes as f64) * 100.0
    }

    /// Perform manual cleanup of expired entries
    pub async fn cleanup(&self) -> CacheResult<usize> {
        let mut entries = self.entries.write().await;
        let mut lru_order = self.lru_order.write().await;
        let mut removed_count = 0;

        // Collect expired keys
        let expired_keys: Vec<String> = entries
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        // Remove expired entries
        for key in expired_keys {
            if entries.remove(&key).is_some() {
                self.remove_from_lru_order(&mut lru_order, &key);
                removed_count += 1;
            }
        }

        // Update cleanup timestamp
        let mut last_cleanup = self.last_cleanup.write().await;
        *last_cleanup = Instant::now();

        Ok(removed_count)
    }

    /// Preload cache with data
    pub async fn preload(&self, data: HashMap<String, Vec<u8>>) -> CacheResult<usize> {
        let mut loaded_count = 0;

        for (key, value) in data {
            if self.put(&key, value, None).await.is_ok() {
                loaded_count += 1;
            }
        }

        Ok(loaded_count)
    }

    /// Get keys matching pattern
    pub async fn get_keys_matching(&self, pattern: &str) -> Vec<String> {
        let entries = self.entries.read().await;
        
        entries
            .keys()
            .filter(|key| key.contains(pattern))
            .cloned()
            .collect()
    }

    // Private helper methods

    async fn ensure_capacity(&self, required_size: usize) -> CacheResult<()> {
        let stats = self.stats.read().await;
        let current_size = stats.total_size_bytes;
        let current_entries = stats.total_entries;
        drop(stats);

        // Check if we exceed size or entry limits
        if current_size + required_size > self.config.max_size_bytes
            || current_entries >= self.config.max_entries
        {
            self.evict_entries(required_size).await?;
        }

        Ok(())
    }

    async fn evict_entries(&self, required_size: usize) -> CacheResult<()> {
        let mut entries = self.entries.write().await;
        let mut lru_order = self.lru_order.write().await;
        let mut freed_size = 0;
        let mut evicted_count = 0;

        // Calculate scores for all entries and sort by eviction priority
        let mut scored_entries: Vec<(String, f64)> = entries
            .iter()
            .map(|(key, entry)| (key.clone(), entry.calculate_score()))
            .collect();

        // Sort by score (highest first = most likely to evict)
        scored_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Evict entries until we have enough space
        for (key, _) in scored_entries {
            if let Some(entry) = entries.remove(&key) {
                self.remove_from_lru_order(&mut lru_order, &key);
                freed_size += entry.size;
                evicted_count += 1;

                // Stop if we've freed enough space
                if freed_size >= required_size {
                    break;
                }
            }
        }

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_size_bytes = stats.total_size_bytes.saturating_sub(freed_size);
        stats.total_entries = stats.total_entries.saturating_sub(evicted_count);
        stats.eviction_count += evicted_count as u64;
        stats.last_updated = Utc::now();

        if freed_size < required_size {
            return Err(CacheError::CacheFull);
        }

        Ok(())
    }

    fn remove_from_lru_order(&self, lru_order: &mut VecDeque<String>, key: &str) {
        if let Some(pos) = lru_order.iter().position(|k| k == key) {
            lru_order.remove(pos);
        }
    }

    fn compress_data(&self, data: &[u8]) -> CacheResult<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| CacheError::WriteFailed {
                reason: format!("Compression failed: {}", e),
            })?;

        encoder.finish()
            .map_err(|e| CacheError::WriteFailed {
                reason: format!("Compression finalization failed: {}", e),
            })
    }

    fn decompress_data(&self, data: &[u8]) -> CacheResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| CacheError::ReadFailed {
                reason: format!("Decompression failed: {}", e),
            })?;

        Ok(decompressed)
    }

    fn calculate_checksum(&self, data: &[u8]) -> String {
        use blake3::hash;
        hex::encode(hash(data).as_bytes())
    }

    async fn cleanup_if_needed(&self) -> CacheResult<()> {
        let last_cleanup = self.last_cleanup.read().await;
        if last_cleanup.elapsed() >= self.config.cleanup_interval {
            drop(last_cleanup);
            self.cleanup().await?;
        }
        Ok(())
    }

    async fn update_stats_after_put(&self, size: usize, is_compressed: bool) {
        let mut stats = self.stats.write().await;
        stats.total_entries += 1;
        stats.total_size_bytes += size;
        
        if is_compressed {
            // Update compression ratio estimate
            let compressed_count = (stats.compression_ratio * (stats.total_entries - 1) as f32) + 1.0;
            stats.compression_ratio = compressed_count / stats.total_entries as f32;
        }
        
        stats.last_updated = Utc::now();
    }

    async fn update_stats_hit(&self, access_time: Duration) {
        let mut stats = self.stats.write().await;
        stats.hit_count += 1;
        
        // Update moving average of access time
        let total_requests = stats.hit_count + stats.miss_count;
        let new_time_ms = access_time.as_secs_f64() * 1000.0;
        stats.avg_access_time_ms = ((stats.avg_access_time_ms * (total_requests - 1) as f64) + new_time_ms) / total_requests as f64;
        
        stats.last_updated = Utc::now();
    }

    async fn update_stats_miss(&self) {
        let mut stats = self.stats.write().await;
        stats.miss_count += 1;
        stats.last_updated = Utc::now();
    }
}

impl Default for StorageCache {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-safe implementations
unsafe impl Send for StorageCache {}
unsafe impl Sync for StorageCache {}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = StorageCache::new();
        let data = b"test data".to_vec();

        let result = cache.put("key1", data.clone(), None).await;
        assert!(result.is_ok());

        let retrieved = cache.get("key1").await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = StorageCache::new();
        
        let result = cache.get("nonexistent").await;
        assert!(result.is_err());
        
        if let Err(CacheError::CacheMiss { key }) = result {
            assert_eq!(key, "nonexistent");
        } else {
            panic!("Expected CacheMiss error");
        }
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = StorageCache::new();
        let data = b"test data".to_vec();
        let short_ttl = Duration::from_millis(50);

        cache.put("key1", data, Some(short_ttl)).await.unwrap();
        
        // Should be available immediately
        assert!(cache.get("key1").await.is_ok());
        
        // Wait for expiration
        sleep(Duration::from_millis(100)).await;
        
        // Should be expired now
        assert!(cache.get("key1").await.is_err());
    }

    #[tokio::test]
    async fn test_cache_compression() {
        let mut config = CacheConfig::default();
        config.compress_threshold = 10; // Very low threshold for testing
        
        let cache = StorageCache::with_config(config);
        let large_data = vec![42u8; 1000]; // 1KB of data

        cache.put("large_key", large_data.clone(), None).await.unwrap();
        let retrieved = cache.get("large_key").await.unwrap();
        
        assert_eq!(retrieved, large_data);
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let mut config = CacheConfig::default();
        config.max_entries = 2; // Very small cache
        
        let cache = StorageCache::with_config(config);
        
        cache.put("key1", b"data1".to_vec(), None).await.unwrap();
        cache.put("key2", b"data2".to_vec(), None).await.unwrap();
        cache.put("key3", b"data3".to_vec(), None).await.unwrap();
        
        // key1 should be evicted
        assert!(cache.get("key1").await.is_err());
        assert!(cache.get("key2").await.is_ok());
        assert!(cache.get("key3").await.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = StorageCache::new();
        
        cache.put("key1", b"data1".to_vec(), None).await.unwrap();
        cache.get("key1").await.unwrap(); // Hit
        cache.get("nonexistent").await.ok(); // Miss
        
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_ratio(), 0.5);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = StorageCache::new();
        
        cache.put("key1", b"data1".to_vec(), None).await.unwrap();
        cache.put("key2", b"data2".to_vec(), None).await.unwrap();
        
        let stats_before = cache.get_stats().await;
        assert_eq!(stats_before.total_entries, 2);
        
        cache.clear().await.unwrap();
        
        let stats_after = cache.get_stats().await;
        assert_eq!(stats_after.total_entries, 0);
        assert_eq!(stats_after.total_size_bytes, 0);
    }

    #[tokio::test]
    async fn test_cache_contains() {
        let cache = StorageCache::new();
        
        assert!(!cache.contains("key1").await);
        
        cache.put("key1", b"data1".to_vec(), None).await.unwrap();
        assert!(cache.contains("key1").await);
        
        cache.remove("key1").await.unwrap();
        assert!(!cache.contains("key1").await);
    }
}
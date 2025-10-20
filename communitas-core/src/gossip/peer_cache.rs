// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Persistent Peer Cache for Boot Performance
//!
//! Per SPEC2.md §6: Persists successful peer connections to enable fast boot.
//! Tracks: (peer_id, addr_hints, nat_class, roles, last_success, success_count)
//!
//! **Lock-Free File-Based Storage:**
//! - Uses atomic file operations (write to temp, then rename)
//! - JSON format for human readability and cross-platform compatibility
//! - Eventual consistency (last write wins) - acceptable for peer cache
//! - Shared across multiple process instances

use anyhow::{Context, Result};
use saorsa_gossip_types::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// NAT classification for connection strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NatClass {
    /// No NAT, publicly accessible
    Public,
    /// Full cone NAT
    FullCone,
    /// Restricted cone NAT
    RestrictedCone,
    /// Port-restricted cone NAT
    PortRestrictedCone,
    /// Symmetric NAT (hardest to traverse)
    Symmetric,
    /// Unknown/unclassified
    Unknown,
}

impl NatClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            NatClass::Public => "public",
            NatClass::FullCone => "full_cone",
            NatClass::RestrictedCone => "restricted_cone",
            NatClass::PortRestrictedCone => "port_restricted_cone",
            NatClass::Symmetric => "symmetric",
            NatClass::Unknown => "unknown",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "public" => NatClass::Public,
            "full_cone" => NatClass::FullCone,
            "restricted_cone" => NatClass::RestrictedCone,
            "port_restricted_cone" => NatClass::PortRestrictedCone,
            "symmetric" => NatClass::Symmetric,
            _ => NatClass::Unknown,
        }
    }
}

/// Cached peer entry with connection history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerCacheEntry {
    #[serde(
        serialize_with = "serialize_peer_id",
        deserialize_with = "deserialize_peer_id"
    )]
    pub peer_id: PeerId,
    pub addr_hints: Vec<SocketAddr>,
    pub nat_class: NatClass,
    pub roles: Vec<String>,
    #[serde(
        serialize_with = "serialize_time",
        deserialize_with = "deserialize_time"
    )]
    pub last_success: SystemTime,
    pub success_count: u32,
    pub failure_count: u32,
    #[serde(default)]
    pub is_bootstrap: bool,
}

// Serde helpers for PeerId
fn serialize_peer_id<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&hex::encode(peer_id.as_bytes()))
}

fn deserialize_peer_id<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let hex_str = String::deserialize(deserializer)?;
    let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
    let array: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| serde::de::Error::custom("Invalid PeerId length"))?;
    Ok(PeerId::new(array))
}

// Serde helpers for SystemTime
fn serialize_time<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let secs = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(serde::ser::Error::custom)?
        .as_secs();
    serializer.serialize_u64(secs)
}

fn deserialize_time<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

impl PeerCacheEntry {
    /// Calculate peer score for connection priority
    /// Score components:
    /// - Success rate: 0.0 (all failures) to 1.0 (all successes)
    /// - Recency bonus: 0.0 (old) to 1.0 (recent)
    /// - NAT penalty: 0.0 (public) to 0.5 (symmetric)
    /// - Role bonus: 0.0 (none) to 0.3 (coordinator)
    pub fn score(&self) -> f64 {
        let total_attempts = self.success_count + self.failure_count;
        if total_attempts == 0 {
            return 0.0;
        }

        // Success rate (0-1)
        let success_rate = self.success_count as f64 / total_attempts as f64;

        // Recency bonus: decays exponentially with time
        // Recent connections get higher scores
        let now = SystemTime::now();
        let age_secs = now
            .duration_since(self.last_success)
            .map(|d| d.as_secs())
            .unwrap_or(u64::MAX);

        const ONE_DAY: u64 = 24 * 60 * 60;
        let recency_bonus = (-(age_secs as f64 / ONE_DAY as f64)).exp();

        // NAT penalty: harder NAT types get lower scores
        let nat_penalty = match self.nat_class {
            NatClass::Public => 0.0,
            NatClass::FullCone => 0.1,
            NatClass::RestrictedCone => 0.2,
            NatClass::PortRestrictedCone => 0.3,
            NatClass::Symmetric => 0.5,
            NatClass::Unknown => 0.25, // Middle ground for unknown
        };

        // Role bonuses: coordinators and relays are valuable
        let role_bonus = if self.roles.contains(&"coordinator".to_string()) {
            0.3
        } else if self.roles.contains(&"relay".to_string()) {
            0.2
        } else {
            0.0
        };

        // Final score: success_rate (0-1) + recency_bonus (0-1) - nat_penalty (0-0.5) + role_bonus (0-0.3)
        (success_rate + recency_bonus + role_bonus - nat_penalty).max(0.0)
    }
}

/// File format for peer cache
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerCacheFile {
    version: u32,
    peers: HashMap<String, PeerCacheEntry>,
}

impl PeerCacheFile {
    fn new() -> Self {
        Self {
            version: 1,
            peers: HashMap::new(),
        }
    }
}

/// Lock-free persistent peer cache using atomic file operations
pub struct PeerCache {
    cache_path: PathBuf,
}

impl PeerCache {
    /// Maximum number of peers to cache (FIFO eviction when exceeded)
    const MAX_CACHE_SIZE: usize = 1000;

    /// Get system-wide peer cache path
    /// Returns: ~/.local/share/communitas/peer_cache.json (Linux/macOS)
    /// or %APPDATA%\communitas\peer_cache.json (Windows)
    pub fn default_cache_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get system data directory"))?
            .join("communitas");

        // Ensure directory exists
        std::fs::create_dir_all(&data_dir).context("Failed to create communitas data directory")?;

        Ok(data_dir.join("peer_cache.json"))
    }

    /// Load existing cache or create new one
    pub async fn load(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        // Create empty cache file if it doesn't exist
        if !path.exists() {
            let empty_cache = PeerCacheFile::new();
            let json = serde_json::to_string_pretty(&empty_cache)
                .context("Failed to serialize empty cache")?;
            std::fs::write(path, json)
                .with_context(|| format!("Failed to write cache file: {:?}", path))?;
            info!("Created new peer cache at {:?}", path);
        } else {
            info!("Loaded peer cache from {:?}", path);
        }

        Ok(Self {
            cache_path: path.to_path_buf(),
        })
    }

    /// Read cache from file
    fn read_cache(&self) -> Result<PeerCacheFile> {
        if !self.cache_path.exists() {
            return Ok(PeerCacheFile::new());
        }

        let contents = std::fs::read_to_string(&self.cache_path)
            .with_context(|| format!("Failed to read cache: {:?}", self.cache_path))?;

        serde_json::from_str(&contents)
            .context("Failed to parse peer cache JSON")
            .or_else(|e| {
                warn!("Corrupted peer cache, creating new: {}", e);
                Ok(PeerCacheFile::new())
            })
    }

    /// Write cache to file atomically
    /// Uses temp file + atomic rename for lock-free concurrent updates
    fn write_cache(&self, cache: &PeerCacheFile) -> Result<()> {
        let json = serde_json::to_string_pretty(cache).context("Failed to serialize peer cache")?;

        // Write to temp file in same directory
        let temp_path = self.cache_path.with_extension("tmp");
        std::fs::write(&temp_path, json)
            .with_context(|| format!("Failed to write temp cache: {:?}", temp_path))?;

        // Atomic rename (last write wins)
        std::fs::rename(&temp_path, &self.cache_path)
            .with_context(|| format!("Failed to rename cache: {:?}", self.cache_path))?;

        Ok(())
    }

    /// Update peer on successful connection
    pub async fn update_success(&mut self, peer_id: PeerId, addr: SocketAddr) -> Result<()> {
        let mut cache = self.read_cache()?;
        let peer_id_str = hex::encode(peer_id.as_bytes());
        let now = SystemTime::now();

        if let Some(entry) = cache.peers.get_mut(&peer_id_str) {
            // Update existing entry
            if !entry.addr_hints.contains(&addr) {
                entry.addr_hints.push(addr);
            }
            entry.last_success = now;
            entry.success_count += 1;
            debug!("Updated peer {} on success", peer_id_str);
        } else {
            // Insert new entry
            cache.peers.insert(
                peer_id_str.clone(),
                PeerCacheEntry {
                    peer_id,
                    addr_hints: vec![addr],
                    nat_class: NatClass::Unknown,
                    roles: Vec::new(),
                    last_success: now,
                    success_count: 1,
                    failure_count: 0,
                    is_bootstrap: false,
                },
            );
            debug!("Added new peer {} to cache", peer_id_str);
        }

        self.write_cache(&cache)?;
        self.enforce_max_size().await?;

        Ok(())
    }

    /// Update peer on connection failure
    pub async fn update_failure(&mut self, peer_id: PeerId) -> Result<()> {
        let mut cache = self.read_cache()?;
        let peer_id_str = hex::encode(peer_id.as_bytes());

        if let Some(entry) = cache.peers.get_mut(&peer_id_str) {
            entry.failure_count += 1;
            self.write_cache(&cache)?;
            debug!("Incremented failure count for peer {}", peer_id_str);
        }

        Ok(())
    }

    /// Get top N peers sorted by score
    pub fn get_top_peers(&self, limit: usize) -> Vec<PeerCacheEntry> {
        let cache = match self.read_cache() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to read peer cache: {}", e);
                return Vec::new();
            }
        };

        let mut entries: Vec<PeerCacheEntry> = cache.peers.into_values().collect();

        // Sort by score (descending)
        entries.sort_by(|a, b| {
            b.score()
                .partial_cmp(&a.score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top N
        entries.into_iter().take(limit).collect()
    }

    /// Prune peers with high failure rates
    pub async fn prune_failed(&mut self, threshold_ratio: f64) -> Result<()> {
        let mut cache = self.read_cache()?;
        let initial_count = cache.peers.len();

        // Remove peers where failure_count / (success_count + failure_count) > threshold
        // and have at least 5 total attempts
        cache.peers.retain(|_, entry| {
            let total_attempts = entry.success_count + entry.failure_count;
            if total_attempts <= 5 {
                return true; // Keep peers with few attempts
            }
            let failure_ratio = entry.failure_count as f64 / total_attempts as f64;
            failure_ratio <= threshold_ratio
        });

        let removed = initial_count - cache.peers.len();
        if removed > 0 {
            self.write_cache(&cache)?;
            warn!("Pruned {} failed peers from cache", removed);
        }

        Ok(())
    }

    /// Get count of cached peers
    pub fn len(&self) -> usize {
        self.read_cache().map(|c| c.peers.len()).unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Add bootstrap node using four-word address
    /// Bootstrap nodes are marked specially and given priority in connections
    pub async fn add_bootstrap_node(&mut self, four_words: &str) -> Result<()> {
        let mut cache = self.read_cache()?;

        // Parse four-word address to get socket address
        // For now, we use a placeholder peer_id derived from four-words
        // In production, this would resolve the four-word address to actual peer_id
        let peer_id_bytes = blake3::hash(four_words.as_bytes());
        let peer_id = PeerId::new(*peer_id_bytes.as_bytes());
        let peer_id_str = hex::encode(peer_id.as_bytes());

        if !cache.peers.contains_key(&peer_id_str) {
            let now = SystemTime::now();
            cache.peers.insert(
                peer_id_str.clone(),
                PeerCacheEntry {
                    peer_id,
                    addr_hints: Vec::new(), // No address hints yet, will be resolved
                    nat_class: NatClass::Public, // Assume bootstrap nodes are public
                    roles: vec!["bootstrap".to_string()],
                    last_success: now,
                    success_count: 100, // High initial success count for priority
                    failure_count: 0,
                    is_bootstrap: true,
                },
            );
            self.write_cache(&cache)?;
            info!("Added bootstrap node: {}", four_words);
        }

        Ok(())
    }

    /// Seed peer cache with bootstrap nodes from config
    /// Returns number of nodes seeded
    pub async fn seed_bootstrap_nodes(&mut self, bootstrap_nodes: &[String]) -> Result<usize> {
        let mut seeded = 0;
        for node in bootstrap_nodes {
            match self.add_bootstrap_node(node).await {
                Ok(_) => seeded += 1,
                Err(e) => warn!("Failed to seed bootstrap node {}: {}", node, e),
            }
        }
        info!("Seeded {} bootstrap nodes into peer cache", seeded);
        Ok(seeded)
    }

    /// Enforce maximum cache size using FIFO eviction
    /// Removes oldest peers (by last_success) when cache exceeds MAX_CACHE_SIZE
    /// Bootstrap nodes are never evicted
    async fn enforce_max_size(&mut self) -> Result<()> {
        let mut cache = self.read_cache()?;

        if cache.peers.len() <= Self::MAX_CACHE_SIZE {
            return Ok(());
        }

        // Collect non-bootstrap peers sorted by last_success
        let mut non_bootstrap: Vec<(String, SystemTime)> = cache
            .peers
            .iter()
            .filter(|(_, entry)| !entry.is_bootstrap)
            .map(|(id, entry)| (id.clone(), entry.last_success))
            .collect();

        non_bootstrap.sort_by_key(|(_, last_success)| *last_success);

        // Remove oldest peers to get back under limit
        let to_remove = cache.peers.len() - Self::MAX_CACHE_SIZE;
        let mut removed = 0;

        for (peer_id, _) in non_bootstrap.iter().take(to_remove) {
            cache.peers.remove(peer_id);
            removed += 1;
        }

        if removed > 0 {
            self.write_cache(&cache)?;
            info!(
                "Evicted {} oldest peers (FIFO) to maintain cache size <= {}",
                removed,
                Self::MAX_CACHE_SIZE
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_peer_id(seed: u8) -> PeerId {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        PeerId::new(bytes)
    }

    #[tokio::test]
    async fn test_peer_cache_creation() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let cache = PeerCache::load(&cache_path)
            .await
            .expect("Should create new cache");

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_update_success() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let mut cache = PeerCache::load(&cache_path).await.expect("create cache");
        let peer_id = create_test_peer_id(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("parse addr");

        cache
            .update_success(peer_id, addr)
            .await
            .expect("update success");

        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let top_peers = cache.get_top_peers(10);
        assert_eq!(top_peers.len(), 1);
        assert_eq!(top_peers[0].success_count, 1);
        assert_eq!(top_peers[0].failure_count, 0);
    }

    #[tokio::test]
    async fn test_update_failure() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let mut cache = PeerCache::load(&cache_path).await.expect("create cache");
        let peer_id = create_test_peer_id(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("parse addr");

        // Add peer first
        cache
            .update_success(peer_id, addr)
            .await
            .expect("update success");

        // Record failure
        cache.update_failure(peer_id).await.expect("update failure");

        let top_peers = cache.get_top_peers(10);
        assert_eq!(top_peers.len(), 1);
        assert_eq!(top_peers[0].failure_count, 1);
    }

    #[tokio::test]
    async fn test_top_peers_sorting() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let mut cache = PeerCache::load(&cache_path).await.expect("create cache");

        // Add peers with different success counts
        for i in 1..=5 {
            let peer_id = create_test_peer_id(i);
            let addr: SocketAddr = format!("127.0.0.1:808{}", i).parse().expect("parse addr");

            for _ in 0..i {
                cache
                    .update_success(peer_id, addr)
                    .await
                    .expect("update success");
            }
        }

        let top_peers = cache.get_top_peers(3);
        assert_eq!(top_peers.len(), 3);

        // Peers should be sorted by score (descending)
        assert!(top_peers[0].score() >= top_peers[1].score());
        assert!(top_peers[1].score() >= top_peers[2].score());

        // Verify all 5 peers were added
        let all_peers = cache.get_top_peers(10);
        assert_eq!(all_peers.len(), 5);
    }

    #[tokio::test]
    async fn test_bootstrap_nodes() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let mut cache = PeerCache::load(&cache_path).await.expect("create cache");

        cache
            .add_bootstrap_node("ocean-forest-moon-star")
            .await
            .expect("add bootstrap");

        assert_eq!(cache.len(), 1);

        let top_peers = cache.get_top_peers(10);
        assert_eq!(top_peers.len(), 1);
        assert!(top_peers[0].is_bootstrap);
        assert_eq!(top_peers[0].success_count, 100); // High initial count
    }

    #[tokio::test]
    async fn test_prune_failed() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let mut cache = PeerCache::load(&cache_path).await.expect("create cache");
        let peer_id = create_test_peer_id(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("parse addr");

        // Add peer with failures
        cache
            .update_success(peer_id, addr)
            .await
            .expect("update success");

        for _ in 0..10 {
            cache.update_failure(peer_id).await.expect("update failure");
        }

        assert_eq!(cache.len(), 1);

        // Prune peers with >50% failure rate
        cache.prune_failed(0.5).await.expect("prune failed");

        // Peer should be removed (1 success, 10 failures = 90% failure rate)
        assert_eq!(cache.len(), 0);
    }

    #[tokio::test]
    async fn test_max_cache_size() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        let mut cache = PeerCache::load(&cache_path).await.expect("create cache");

        // Add more than MAX_CACHE_SIZE peers
        for i in 0..1100 {
            let peer_id = create_test_peer_id((i % 256) as u8);
            let addr: SocketAddr = format!("127.0.0.1:{}", 8000 + i)
                .parse()
                .expect("parse addr");

            cache
                .update_success(peer_id, addr)
                .await
                .expect("update success");
        }

        // Cache should be limited to MAX_CACHE_SIZE
        assert!(cache.len() <= PeerCache::MAX_CACHE_SIZE);
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.json");

        // Create two cache instances pointing to same file
        let mut cache1 = PeerCache::load(&cache_path).await.expect("create cache1");
        let mut cache2 = PeerCache::load(&cache_path).await.expect("create cache2");

        let peer1 = create_test_peer_id(1);
        let peer2 = create_test_peer_id(2);
        let addr1: SocketAddr = "127.0.0.1:8081".parse().expect("parse addr1");
        let addr2: SocketAddr = "127.0.0.1:8082".parse().expect("parse addr2");

        // Concurrent writes (last write wins)
        cache1.update_success(peer1, addr1).await.expect("write 1");
        cache2.update_success(peer2, addr2).await.expect("write 2");

        // Read from fresh instance
        let cache3 = PeerCache::load(&cache_path).await.expect("create cache3");

        // At least one peer should be present (last write wins due to atomic rename)
        assert!(!cache3.is_empty());
    }
}

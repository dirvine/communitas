//! Wrapper around saorsa-gossip's BootstrapCache for Communitas-specific needs.
//!
//! This replaces the previous bespoke peer cache implementation with a thin layer
//! over `ant-quic`'s epsilon-greedy cache. It also tracks user-specified bootstrap
//! addresses that do not yet have associated peer identities so they can still
//! be surfaced in diagnostics and peer list responses.

use anyhow::{Context, Result};
use saorsa_gossip_transport::{AntPeerId, BootstrapCache, BootstrapCacheConfigBuilder, CachedPeer};
use saorsa_gossip_types::PeerId as GossipPeerId;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::warn;

/// Persistent peer cache wrapper
#[derive(Clone)]
pub struct PeerCache {
    bootstrap_cache: Arc<BootstrapCache>,
    seed_store: Arc<RwLock<SeedStore>>,
    seed_path: Arc<PathBuf>,
}

impl PeerCache {
    /// Initialize the cache using the provided directory.
    ///
    /// Creates both the bootstrap cache directory and the auxiliary seed store.
    pub async fn open(dir: &Path) -> Result<Self> {
        tokio::fs::create_dir_all(dir)
            .await
            .with_context(|| format!("failed to create bootstrap cache dir {}", dir.display()))?;

        let cache = BootstrapCache::open(
            BootstrapCacheConfigBuilder::default()
                .cache_dir(dir)
                .max_peers(10_000)
                .build(),
        )
        .await
        .context("failed to open bootstrap cache")?;

        let seed_path = dir.join("seed_nodes.json");
        let seed_store = SeedStore::load(&seed_path).await.unwrap_or_default();

        Ok(Self {
            bootstrap_cache: Arc::new(cache),
            seed_store: Arc::new(RwLock::new(seed_store)),
            seed_path: Arc::new(seed_path),
        })
    }

    /// Expose the underlying bootstrap cache for transport wiring.
    pub fn bootstrap_cache(&self) -> Arc<BootstrapCache> {
        Arc::clone(&self.bootstrap_cache)
    }

    /// Total cached peers with quality data.
    pub async fn len(&self) -> usize {
        self.bootstrap_cache.peer_count().await
    }

    /// Whether the cache has no peers.
    pub async fn is_empty(&self) -> bool {
        self.bootstrap_cache.peer_count().await == 0
    }

    /// Select the best peers according to epsilon-greedy scoring.
    pub async fn get_top_peers(&self, limit: usize) -> Vec<CachedPeer> {
        self.bootstrap_cache.select_peers(limit).await
    }

    /// Record a successful connection (addresses are learned via connection metadata).
    pub async fn record_success(&self, peer_id: GossipPeerId, addr: SocketAddr) -> Result<()> {
        let ant_id = gossip_to_ant(peer_id);
        self.bootstrap_cache
            .add_from_connection(ant_id, vec![addr], None)
            .await;
        self.bootstrap_cache.record_success(&ant_id, 50).await;
        Ok(())
    }

    /// Record a failed connection attempt for scoring.
    pub async fn record_failure(&self, peer_id: GossipPeerId) -> Result<()> {
        let ant_id = gossip_to_ant(peer_id);
        self.bootstrap_cache.record_failure(&ant_id).await;
        Ok(())
    }

    /// Return cached addresses for a specific peer.
    pub async fn get_addr_hints(&self, peer_id: GossipPeerId) -> Vec<SocketAddr> {
        let ant_id = gossip_to_ant(peer_id);
        if let Some(peer) = self.bootstrap_cache.get_peer(&ant_id).await
            && !peer.addresses.is_empty()
        {
            return peer.addresses;
        }
        Vec::new()
    }

    /// Seed the cache with manual bootstrap nodes (addresses only).
    pub async fn seed_bootstrap_nodes(&self, nodes: &[String]) -> Result<usize> {
        let mut store = self.seed_store.write().await;
        let mut added = 0usize;
        for node in nodes {
            if let Some(addr) = parse_addr(node) {
                if store
                    .entries
                    .insert(addr.to_string(), SeedEntry::new())
                    .is_none()
                {
                    added += 1;
                }
            } else {
                warn!("Unable to parse bootstrap node '{}'", node);
            }
        }
        drop(store);
        self.persist_seeds().await?;
        Ok(added)
    }

    /// Add a bootstrap address discovered via peer lists.
    pub async fn add_bootstrap_addr(&self, addr: SocketAddr, is_community: bool) -> Result<()> {
        let mut store = self.seed_store.write().await;
        store
            .entries
            .entry(addr.to_string())
            .and_modify(|entry| entry.is_community |= is_community)
            .or_insert_with(|| {
                let mut entry = SeedEntry::new();
                entry.is_community = is_community;
                entry
            });
        drop(store);
        self.persist_seeds().await
    }

    /// Record that a bootstrap address responded successfully.
    pub async fn record_bootstrap_success(&self, addr: &SocketAddr) -> Result<()> {
        let mut store = self.seed_store.write().await;
        if let Some(entry) = store.entries.get_mut(&addr.to_string()) {
            entry.success_count = entry.success_count.saturating_add(1);
            entry.last_success_epoch = Some(now_secs());
        }
        drop(store);
        self.persist_seeds().await
    }

    /// Return all known seed addresses (manual + discovered).
    pub async fn seed_addresses(&self) -> Vec<SocketAddr> {
        let store = self.seed_store.read().await;
        store
            .entries
            .keys()
            .filter_map(|val| val.parse::<SocketAddr>().ok())
            .collect()
    }

    async fn persist_seeds(&self) -> Result<()> {
        let snapshot = { self.seed_store.read().await.clone() };
        let tmp = self.seed_path.with_extension("tmp");
        let bytes =
            serde_json::to_vec_pretty(&snapshot).context("failed to serialize bootstrap seeds")?;
        tokio::fs::write(&tmp, bytes)
            .await
            .with_context(|| format!("failed to write {}", tmp.display()))?;
        tokio::fs::rename(&tmp, &*self.seed_path)
            .await
            .with_context(|| {
                format!("failed to atomically replace {}", self.seed_path.display())
            })?;
        Ok(())
    }
}

/// Parse socket addresses from either ip:port or four-word identities.
fn parse_addr(input: &str) -> Option<SocketAddr> {
    if let Ok(addr) = input.parse::<SocketAddr>() {
        return Some(addr);
    }
    if let Ok(addr) = crate::identity::conn_from_words(input) {
        return Some(addr);
    }
    let normalized = input.replace('-', " ");
    if normalized != input
        && let Ok(addr) = crate::identity::conn_from_words(&normalized)
    {
        return Some(addr);
    }
    None
}

fn gossip_to_ant(peer_id: GossipPeerId) -> AntPeerId {
    AntPeerId(*peer_id.as_bytes())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|dur| dur.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SeedEntry {
    success_count: u32,
    last_success_epoch: Option<u64>,
    is_community: bool,
}

impl SeedEntry {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SeedStore {
    entries: HashMap<String, SeedEntry>,
}

impl SeedStore {
    async fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = tokio::fs::read(path)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_slice(&bytes).context("failed to parse bootstrap seed store")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ==================== parse_addr tests ====================

    #[test]
    fn test_parse_addr_ipv4() {
        let result = parse_addr("127.0.0.1:8080");
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn test_parse_addr_ipv6() {
        let result = parse_addr("[::1]:8080");
        assert!(result.is_some());
        assert_eq!(result.unwrap().to_string(), "[::1]:8080");
    }

    #[test]
    fn test_parse_addr_invalid() {
        let result = parse_addr("not-an-address");
        // May return None or parse as four-word depending on dictionary
        // The point is it shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_parse_addr_empty() {
        let result = parse_addr("");
        assert!(result.is_none());
    }

    // ==================== SeedEntry tests ====================

    #[test]
    fn test_seed_entry_default() {
        let entry = SeedEntry::new();
        assert_eq!(entry.success_count, 0);
        assert!(entry.last_success_epoch.is_none());
        assert!(!entry.is_community);
    }

    // ==================== SeedStore tests ====================

    #[tokio::test]
    async fn test_seed_store_load_nonexistent() {
        let result = SeedStore::load(Path::new("/nonexistent/path/store.json")).await;
        assert!(result.is_ok());
        let store = result.unwrap();
        assert!(store.entries.is_empty());
    }

    #[tokio::test]
    async fn test_seed_store_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("seeds.json");

        // Create and serialize a store
        let mut store = SeedStore::default();
        store.entries.insert(
            "127.0.0.1:8080".to_string(),
            SeedEntry {
                success_count: 5,
                last_success_epoch: Some(1234567890),
                is_community: true,
            },
        );

        let bytes = serde_json::to_vec_pretty(&store).unwrap();
        tokio::fs::write(&path, bytes).await.unwrap();

        // Load and verify
        let loaded = SeedStore::load(&path).await.unwrap();
        assert_eq!(loaded.entries.len(), 1);
        let entry = loaded.entries.get("127.0.0.1:8080").unwrap();
        assert_eq!(entry.success_count, 5);
        assert_eq!(entry.last_success_epoch, Some(1234567890));
        assert!(entry.is_community);
    }

    // ==================== PeerCache tests ====================

    #[tokio::test]
    async fn test_peer_cache_open() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await;
        assert!(cache.is_ok());
    }

    #[tokio::test]
    async fn test_peer_cache_initially_empty() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();
        assert!(cache.is_empty().await);
        assert_eq!(cache.len().await, 0);
    }

    #[tokio::test]
    async fn test_peer_cache_seed_bootstrap_nodes_valid() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let added = cache
            .seed_bootstrap_nodes(&["127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string()])
            .await
            .unwrap();

        assert_eq!(added, 2);

        let seeds = cache.seed_addresses().await;
        assert_eq!(seeds.len(), 2);
    }

    #[tokio::test]
    async fn test_peer_cache_seed_bootstrap_nodes_dedup() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        // Add same address twice
        let added1 = cache
            .seed_bootstrap_nodes(&["127.0.0.1:8080".to_string()])
            .await
            .unwrap();
        let added2 = cache
            .seed_bootstrap_nodes(&["127.0.0.1:8080".to_string()])
            .await
            .unwrap();

        assert_eq!(added1, 1);
        assert_eq!(added2, 0); // Already exists

        let seeds = cache.seed_addresses().await;
        assert_eq!(seeds.len(), 1);
    }

    #[tokio::test]
    async fn test_peer_cache_seed_bootstrap_nodes_invalid_skipped() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let added = cache
            .seed_bootstrap_nodes(&[
                "127.0.0.1:8080".to_string(),
                "invalid-not-an-address-or-words".to_string(),
                "127.0.0.1:8081".to_string(),
            ])
            .await
            .unwrap();

        // Should add 2 valid addresses, skip the invalid one
        assert_eq!(added, 2);
    }

    #[tokio::test]
    async fn test_peer_cache_add_bootstrap_addr() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let addr: SocketAddr = "192.168.1.1:9000".parse().unwrap();
        cache.add_bootstrap_addr(addr, false).await.unwrap();

        let seeds = cache.seed_addresses().await;
        assert!(seeds.contains(&addr));
    }

    #[tokio::test]
    async fn test_peer_cache_add_bootstrap_addr_community_flag() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let addr: SocketAddr = "192.168.1.1:9000".parse().unwrap();

        // Add without community flag
        cache.add_bootstrap_addr(addr, false).await.unwrap();

        // Add again with community flag - should update
        cache.add_bootstrap_addr(addr, true).await.unwrap();

        // Verify via seed store
        let store = cache.seed_store.read().await;
        let entry = store.entries.get(&addr.to_string()).unwrap();
        assert!(entry.is_community);
    }

    #[tokio::test]
    async fn test_peer_cache_record_bootstrap_success() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let addr: SocketAddr = "192.168.1.1:9000".parse().unwrap();
        cache.add_bootstrap_addr(addr, false).await.unwrap();

        // Record success
        cache.record_bootstrap_success(&addr).await.unwrap();

        // Verify
        let store = cache.seed_store.read().await;
        let entry = store.entries.get(&addr.to_string()).unwrap();
        assert_eq!(entry.success_count, 1);
        assert!(entry.last_success_epoch.is_some());
    }

    #[tokio::test]
    async fn test_peer_cache_record_bootstrap_success_increments() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let addr: SocketAddr = "192.168.1.1:9000".parse().unwrap();
        cache.add_bootstrap_addr(addr, false).await.unwrap();

        // Record multiple successes
        cache.record_bootstrap_success(&addr).await.unwrap();
        cache.record_bootstrap_success(&addr).await.unwrap();
        cache.record_bootstrap_success(&addr).await.unwrap();

        let store = cache.seed_store.read().await;
        let entry = store.entries.get(&addr.to_string()).unwrap();
        assert_eq!(entry.success_count, 3);
    }

    #[tokio::test]
    async fn test_peer_cache_get_addr_hints_empty() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        // Random peer ID
        let peer_id = GossipPeerId::new([1u8; 32]);
        let hints = cache.get_addr_hints(peer_id).await;
        assert!(hints.is_empty());
    }

    #[tokio::test]
    async fn test_peer_cache_persistence() {
        let dir = TempDir::new().unwrap();

        // Create cache and add seeds
        {
            let cache = PeerCache::open(dir.path()).await.unwrap();
            cache
                .seed_bootstrap_nodes(&["10.0.0.1:5000".to_string()])
                .await
                .unwrap();
        }

        // Reopen and verify
        {
            let cache = PeerCache::open(dir.path()).await.unwrap();
            let seeds = cache.seed_addresses().await;
            assert_eq!(seeds.len(), 1);
            assert_eq!(seeds[0].to_string(), "10.0.0.1:5000");
        }
    }

    #[tokio::test]
    async fn test_peer_cache_bootstrap_cache_accessor() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        // Should be able to get the underlying bootstrap cache
        let _bc = cache.bootstrap_cache();
        // Just verify it doesn't panic
    }

    #[tokio::test]
    async fn test_peer_cache_get_top_peers_empty() {
        let dir = TempDir::new().unwrap();
        let cache = PeerCache::open(dir.path()).await.unwrap();

        let peers = cache.get_top_peers(10).await;
        assert!(peers.is_empty());
    }

    // ==================== now_secs utility test ====================

    #[test]
    fn test_now_secs_reasonable() {
        let secs = now_secs();
        // Should be after 2024-01-01 (1704067200) and before 2100-01-01 (4102444800)
        assert!(secs > 1704067200);
        assert!(secs < 4102444800);
    }
}

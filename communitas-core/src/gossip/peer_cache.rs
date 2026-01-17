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

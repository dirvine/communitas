//! Peer cache for storing and managing known peers
//!
//! This module provides persistent storage of successfully connected peers
//! to enable resilient network bootstrapping across restarts.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Information about a cached peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Four-word identity (if known)
    pub identity: Option<String>,
    /// Socket address for connection
    pub address: SocketAddr,
    /// Public key (hex encoded)
    pub public_key: Option<String>,
    /// Last successful connection timestamp
    pub last_connected: i64,
    /// Number of successful connections
    pub connection_count: u32,
    /// Average connection quality (0-100)
    pub quality_score: u8,
}

/// Peer cache that stores known peers persistently
pub struct PeerCache {
    /// Storage path for the cache file
    cache_file: PathBuf,
    /// In-memory cache of peers
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Maximum number of peers to cache
    max_peers: usize,
}

impl PeerCache {
    /// Create a new peer cache
    pub fn new(storage_dir: PathBuf, max_peers: usize) -> Result<Self> {
        let cache_file = storage_dir.join("peer_cache.json");
        let cache = Self {
            cache_file,
            peers: Arc::new(RwLock::new(HashMap::new())),
            max_peers,
        };

        // Load existing cache if available
        tokio::spawn({
            let cache_clone = cache.clone();
            async move {
                if let Err(e) = cache_clone.load().await {
                    warn!("Failed to load peer cache: {}", e);
                }
            }
        });

        Ok(cache)
    }

    /// Load peer cache from disk
    pub async fn load(&self) -> Result<()> {
        if !self.cache_file.exists() {
            debug!("No existing peer cache found at {:?}", self.cache_file);
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.cache_file).await?;
        let loaded_peers: HashMap<String, PeerInfo> = serde_json::from_str(&content)?;

        let mut peers = self.peers.write().await;
        *peers = loaded_peers;

        info!("Loaded {} peers from cache", peers.len());
        Ok(())
    }

    /// Save peer cache to disk
    pub async fn save(&self) -> Result<()> {
        let peers = self.peers.read().await;
        let content = serde_json::to_string_pretty(&*peers)?;

        // Ensure directory exists
        if let Some(parent) = self.cache_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&self.cache_file, content).await?;
        debug!("Saved {} peers to cache", peers.len());
        Ok(())
    }

    /// Add or update a peer in the cache
    pub async fn add_peer(&self, peer_id: String, info: PeerInfo) -> Result<()> {
        let mut peers = self.peers.write().await;

        // If cache is full, remove the oldest peer
        if peers.len() >= self.max_peers
            && !peers.contains_key(&peer_id)
            && let Some(oldest_key) = peers
                .iter()
                .min_by_key(|(_, info)| info.last_connected)
                .map(|(k, _)| k.clone())
        {
            peers.remove(&oldest_key);
            debug!("Evicted oldest peer {} from cache", oldest_key);
        }

        peers.insert(peer_id.clone(), info);
        drop(peers);

        // Save to disk asynchronously
        let cache_clone = self.clone();
        tokio::spawn(async move {
            if let Err(e) = cache_clone.save().await {
                error!("Failed to save peer cache: {}", e);
            }
        });

        Ok(())
    }

    /// Get a peer from the cache
    #[allow(dead_code)]
    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerInfo> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    /// Get all cached peers
    #[allow(dead_code)]
    pub async fn get_all_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }

    /// Get bootstrap candidates from cache
    pub async fn get_bootstrap_candidates(&self, count: usize) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        let mut candidates: Vec<_> = peers.values().cloned().collect();

        // Sort by quality score and recent connection time
        candidates.sort_by(|a, b| {
            b.quality_score
                .cmp(&a.quality_score)
                .then(b.last_connected.cmp(&a.last_connected))
        });

        candidates.truncate(count);
        candidates
    }

    /// Update connection statistics for a peer
    #[allow(dead_code)]
    pub async fn update_peer_stats(
        &self,
        peer_id: &str,
        connected: bool,
        quality: Option<u8>,
    ) -> Result<()> {
        let mut peers = self.peers.write().await;

        if let Some(peer) = peers.get_mut(peer_id)
            && connected
        {
            peer.last_connected = chrono::Utc::now().timestamp();
            peer.connection_count += 1;

            // Update quality score with moving average
            if let Some(q) = quality {
                peer.quality_score = ((peer.quality_score as u32 + q as u32) / 2) as u8;
            }
        }

        Ok(())
    }

    /// Get the number of cached peers
    #[allow(dead_code)]
    pub async fn peer_count(&self) -> usize {
        let peers = self.peers.read().await;
        peers.len()
    }
}

impl Clone for PeerCache {
    fn clone(&self) -> Self {
        Self {
            cache_file: self.cache_file.clone(),
            peers: self.peers.clone(),
            max_peers: self.max_peers,
        }
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Persistent Peer Cache for Boot Performance
//!
//! Per SPEC2.md §6: Persists successful peer connections to enable fast boot.
//! Tracks: (peer_id, addr_hints, nat_class, roles, last_success, success_count)

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use saorsa_gossip_types::PeerId;
use std::net::SocketAddr;
use std::path::Path;
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// NAT classification for connection strategy
#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn from_str(s: &str) -> Self {
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

/// Cached peer entry with connection metadata
#[derive(Debug, Clone)]
pub struct PeerCacheEntry {
    pub peer_id: PeerId,
    pub addr_hints: Vec<SocketAddr>,
    pub nat_class: NatClass,
    pub roles: Vec<String>, // e.g., ["coordinator", "relay", "rendezvous"]
    pub last_success: SystemTime,
    pub success_count: u32,
    pub failure_count: u32,
}

impl PeerCacheEntry {
    /// Calculate connection score for prioritization
    /// Higher score = better candidate for connection attempt
    pub fn score(&self) -> f64 {
        let success_rate = if self.success_count + self.failure_count > 0 {
            self.success_count as f64 / (self.success_count + self.failure_count) as f64
        } else {
            0.5 // Neutral for unknown peers
        };

        // Recency bonus: prefer recently successful peers
        let recency_bonus = match self.last_success.elapsed() {
            Ok(elapsed) if elapsed.as_secs() < 300 => 1.0, // <5min: full bonus
            Ok(elapsed) if elapsed.as_secs() < 3600 => 0.7, // <1hr: partial bonus
            Ok(elapsed) if elapsed.as_secs() < 86400 => 0.3, // <1day: small bonus
            _ => 0.0,
        };

        // NAT traversal difficulty penalty
        let nat_penalty = match self.nat_class {
            NatClass::Public => 0.0,
            NatClass::FullCone => 0.1,
            NatClass::RestrictedCone => 0.2,
            NatClass::PortRestrictedCone => 0.3,
            NatClass::Symmetric => 0.5,
            NatClass::Unknown => 0.2,
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

/// Persistent peer cache using SQLite
pub struct PeerCache {
    conn: Connection,
}

impl PeerCache {
    /// Load existing cache or create new one
    pub async fn load(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open peer cache database")?;

        // Create table if not exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS peers (
                peer_id TEXT PRIMARY KEY,
                addr_hints TEXT NOT NULL,
                nat_class TEXT NOT NULL,
                roles TEXT NOT NULL,
                last_success INTEGER NOT NULL,
                success_count INTEGER NOT NULL,
                failure_count INTEGER NOT NULL
            )",
            [],
        )?;

        info!("Loaded peer cache from {:?}", path);
        Ok(Self { conn })
    }

    /// Update peer on successful connection
    pub async fn update_success(&mut self, peer_id: PeerId, addr: SocketAddr) -> Result<()> {
        let peer_id_str = hex::encode(peer_id.as_bytes());
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .context("Time went backwards")?
            .as_secs() as i64;

        // Check if peer exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM peers WHERE peer_id = ?1",
                params![peer_id_str],
                |_| Ok(true),
            )
            .unwrap_or(false);

        if exists {
            // Update existing entry
            self.conn.execute(
                "UPDATE peers SET
                    addr_hints = ?2,
                    last_success = ?3,
                    success_count = success_count + 1
                WHERE peer_id = ?1",
                params![peer_id_str, addr.to_string(), now],
            )?;
            debug!("Updated peer {} on success", peer_id_str);
        } else {
            // Insert new entry
            self.conn.execute(
                "INSERT INTO peers (peer_id, addr_hints, nat_class, roles, last_success, success_count, failure_count)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    peer_id_str,
                    addr.to_string(),
                    NatClass::Unknown.as_str(),
                    "[]", // Empty roles initially
                    now,
                    1, // First success
                    0  // No failures yet
                ],
            )?;
            debug!("Added new peer {} to cache", peer_id_str);
        }

        Ok(())
    }

    /// Update peer on connection failure
    pub async fn update_failure(&mut self, peer_id: PeerId) -> Result<()> {
        let peer_id_str = hex::encode(peer_id.as_bytes());

        self.conn.execute(
            "UPDATE peers SET failure_count = failure_count + 1 WHERE peer_id = ?1",
            params![peer_id_str],
        )?;

        debug!("Incremented failure count for peer {}", peer_id_str);
        Ok(())
    }

    /// Get top N peers sorted by score
    pub fn get_top_peers(&self, limit: usize) -> Vec<PeerCacheEntry> {
        let mut stmt = self
            .conn
            .prepare("SELECT peer_id, addr_hints, nat_class, roles, last_success, success_count, failure_count FROM peers")
            .ok();

        if stmt.is_none() {
            return Vec::new();
        }

        let rows = stmt.as_mut().and_then(|s| {
            s.query_map([], |row| {
                let peer_id_hex: String = row.get(0)?;
                let peer_id_bytes = hex::decode(&peer_id_hex)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let peer_id_array: [u8; 32] = peer_id_bytes
                    .as_slice()
                    .try_into()
                    .map_err(|_| rusqlite::Error::InvalidQuery)?;
                let peer_id = PeerId::new(peer_id_array);

                let addr_hints_str: String = row.get(1)?;
                let addr_hints: Vec<SocketAddr> = addr_hints_str
                    .split(',')
                    .filter_map(|s| s.parse().ok())
                    .collect();

                let nat_class = NatClass::from_str(&row.get::<_, String>(2)?);
                let roles_str: String = row.get(3)?;
                let roles: Vec<String> = serde_json::from_str(&roles_str).unwrap_or_default();
                let last_success = SystemTime::UNIX_EPOCH
                    + std::time::Duration::from_secs(row.get::<_, i64>(4)? as u64);
                let success_count: u32 = row.get(5)?;
                let failure_count: u32 = row.get(6)?;

                Ok(PeerCacheEntry {
                    peer_id,
                    addr_hints,
                    nat_class,
                    roles,
                    last_success,
                    success_count,
                    failure_count,
                })
            })
            .ok()
        });

        if rows.is_none() {
            return Vec::new();
        }

        let mut entries: Vec<PeerCacheEntry> = rows.unwrap().filter_map(Result::ok).collect();

        // Sort by score (descending)
        entries.sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());

        // Take top N
        entries.into_iter().take(limit).collect()
    }

    /// Prune peers with high failure rates
    pub async fn prune_failed(&mut self, threshold_ratio: f64) -> Result<()> {
        // Remove peers where failure_count / (success_count + failure_count) > threshold
        let count = self.conn.execute(
            "DELETE FROM peers WHERE
                (failure_count * 1.0) / (success_count + failure_count) > ?1
                AND (success_count + failure_count) > 5",
            params![threshold_ratio],
        )?;

        if count > 0 {
            warn!("Pruned {} failed peers from cache", count);
        }

        Ok(())
    }

    /// Get count of cached peers
    pub fn len(&self) -> usize {
        self.conn
            .query_row("SELECT COUNT(*) FROM peers", [], |row| row.get(0))
            .unwrap_or(0)
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        let cache_path = temp_dir.path().join("peers.db");

        let cache = PeerCache::load(&cache_path)
            .await
            .expect("Should create new cache");

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[tokio::test]
    async fn test_update_success_new_peer() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        let peer_id = create_test_peer_id(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("addr");

        cache.update_success(peer_id, addr).await.expect("update");

        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());
    }

    #[tokio::test]
    async fn test_update_success_existing_peer() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        let peer_id = create_test_peer_id(1);
        let addr1: SocketAddr = "127.0.0.1:8080".parse().expect("addr");
        let addr2: SocketAddr = "127.0.0.1:8081".parse().expect("addr");

        // First success
        cache
            .update_success(peer_id, addr1)
            .await
            .expect("update 1");
        assert_eq!(cache.len(), 1);

        // Second success (should update, not create new)
        cache
            .update_success(peer_id, addr2)
            .await
            .expect("update 2");
        assert_eq!(cache.len(), 1);

        // Verify success count incremented
        let peers = cache.get_top_peers(10);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].success_count, 2);
    }

    #[tokio::test]
    async fn test_update_failure() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        let peer_id = create_test_peer_id(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("addr");

        // Create peer first
        cache.update_success(peer_id, addr).await.expect("success");

        // Record failures
        cache.update_failure(peer_id).await.expect("failure 1");
        cache.update_failure(peer_id).await.expect("failure 2");

        let peers = cache.get_top_peers(10);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].success_count, 1);
        assert_eq!(peers[0].failure_count, 2);
    }

    #[tokio::test]
    async fn test_peer_scoring_success_rate() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        // Peer A: 10 successes, 0 failures (perfect score)
        let peer_a = create_test_peer_id(1);
        let addr_a: SocketAddr = "127.0.0.1:8080".parse().expect("addr");
        for _ in 0..10 {
            cache.update_success(peer_a, addr_a).await.expect("success");
        }

        // Peer B: 5 successes, 5 failures (50% success rate)
        let peer_b = create_test_peer_id(2);
        let addr_b: SocketAddr = "127.0.0.1:8081".parse().expect("addr");
        for _ in 0..5 {
            cache.update_success(peer_b, addr_b).await.expect("success");
            cache.update_failure(peer_b).await.expect("failure");
        }

        let peers = cache.get_top_peers(10);
        assert_eq!(peers.len(), 2);

        // Peer A should be ranked higher (better success rate)
        assert_eq!(peers[0].peer_id, peer_a);
        assert!(peers[0].score() > peers[1].score());
    }

    #[tokio::test]
    async fn test_peer_scoring_nat_class() {
        // Test that NAT class affects scoring
        let entry_public = PeerCacheEntry {
            peer_id: create_test_peer_id(1),
            addr_hints: vec!["127.0.0.1:8080".parse().expect("addr")],
            nat_class: NatClass::Public,
            roles: vec![],
            last_success: SystemTime::now(),
            success_count: 10,
            failure_count: 0,
        };

        let entry_symmetric = PeerCacheEntry {
            peer_id: create_test_peer_id(2),
            addr_hints: vec!["127.0.0.1:8081".parse().expect("addr")],
            nat_class: NatClass::Symmetric,
            roles: vec![],
            last_success: SystemTime::now(),
            success_count: 10,
            failure_count: 0,
        };

        // Public NAT should score higher (no penalty)
        assert!(entry_public.score() > entry_symmetric.score());
    }

    #[tokio::test]
    async fn test_peer_scoring_roles() {
        // Test that coordinator role gives bonus
        let entry_coordinator = PeerCacheEntry {
            peer_id: create_test_peer_id(1),
            addr_hints: vec!["127.0.0.1:8080".parse().expect("addr")],
            nat_class: NatClass::Unknown,
            roles: vec!["coordinator".to_string()],
            last_success: SystemTime::now(),
            success_count: 10,
            failure_count: 0,
        };

        let entry_regular = PeerCacheEntry {
            peer_id: create_test_peer_id(2),
            addr_hints: vec!["127.0.0.1:8081".parse().expect("addr")],
            nat_class: NatClass::Unknown,
            roles: vec![],
            last_success: SystemTime::now(),
            success_count: 10,
            failure_count: 0,
        };

        // Coordinator should score higher (role bonus)
        assert!(entry_coordinator.score() > entry_regular.score());
    }

    #[tokio::test]
    async fn test_prune_failed_peers() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        // Good peer: 10 successes, 1 failure (90% success rate)
        let peer_good = create_test_peer_id(1);
        let addr_good: SocketAddr = "127.0.0.1:8080".parse().expect("addr");
        for _ in 0..10 {
            cache
                .update_success(peer_good, addr_good)
                .await
                .expect("success");
        }
        cache.update_failure(peer_good).await.expect("failure");

        // Bad peer: 2 successes, 8 failures (20% success rate)
        let peer_bad = create_test_peer_id(2);
        let addr_bad: SocketAddr = "127.0.0.1:8081".parse().expect("addr");
        for _ in 0..2 {
            cache
                .update_success(peer_bad, addr_bad)
                .await
                .expect("success");
        }
        for _ in 0..8 {
            cache.update_failure(peer_bad).await.expect("failure");
        }

        assert_eq!(cache.len(), 2);

        // Prune peers with >50% failure rate
        cache.prune_failed(0.5).await.expect("prune");

        // Only good peer should remain
        assert_eq!(cache.len(), 1);
        let peers = cache.get_top_peers(10);
        assert_eq!(peers[0].peer_id, peer_good);
    }

    #[tokio::test]
    async fn test_get_top_peers_limit() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        // Create 10 peers
        for i in 0..10 {
            let peer_id = create_test_peer_id(i);
            let addr: SocketAddr = format!("127.0.0.1:80{:02}", i).parse().expect("addr");
            cache.update_success(peer_id, addr).await.expect("success");
        }

        assert_eq!(cache.len(), 10);

        // Get top 5
        let top_5 = cache.get_top_peers(5);
        assert_eq!(top_5.len(), 5);

        // Get top 20 (should return all 10)
        let top_20 = cache.get_top_peers(20);
        assert_eq!(top_20.len(), 10);
    }

    #[tokio::test]
    async fn test_persistence_across_restarts() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let peer_id = create_test_peer_id(1);
        let addr: SocketAddr = "127.0.0.1:8080".parse().expect("addr");

        // Create cache and add peer
        {
            let mut cache = PeerCache::load(&cache_path).await.expect("cache 1");
            cache.update_success(peer_id, addr).await.expect("success");
            assert_eq!(cache.len(), 1);
        }

        // Load cache again (simulates restart)
        {
            let cache = PeerCache::load(&cache_path).await.expect("cache 2");
            assert_eq!(cache.len(), 1);

            let peers = cache.get_top_peers(10);
            assert_eq!(peers[0].peer_id, peer_id);
            assert_eq!(peers[0].success_count, 1);
        }
    }

    #[tokio::test]
    async fn test_nat_class_serialization() {
        let all_classes = vec![
            NatClass::Public,
            NatClass::FullCone,
            NatClass::RestrictedCone,
            NatClass::PortRestrictedCone,
            NatClass::Symmetric,
            NatClass::Unknown,
        ];

        for nat_class in all_classes {
            let s = nat_class.as_str();
            let deserialized = NatClass::from_str(s);
            assert_eq!(nat_class, deserialized);
        }

        // Test unknown string
        assert_eq!(NatClass::from_str("invalid"), NatClass::Unknown);
    }

    #[tokio::test]
    async fn test_empty_cache_operations() {
        let temp_dir = TempDir::new().expect("temp dir");
        let cache_path = temp_dir.path().join("peers.db");

        let mut cache = PeerCache::load(&cache_path).await.expect("cache");

        // Prune on empty cache
        cache.prune_failed(0.5).await.expect("prune");
        assert_eq!(cache.len(), 0);

        // Get top peers on empty cache
        let peers = cache.get_top_peers(10);
        assert_eq!(peers.len(), 0);
    }

    #[tokio::test]
    async fn test_peer_entry_score_recency() {
        use std::time::Duration;

        // Recent peer (just now)
        let entry_recent = PeerCacheEntry {
            peer_id: create_test_peer_id(1),
            addr_hints: vec!["127.0.0.1:8080".parse().expect("addr")],
            nat_class: NatClass::Unknown,
            roles: vec![],
            last_success: SystemTime::now(),
            success_count: 10,
            failure_count: 0,
        };

        // Old peer (1 day ago)
        let entry_old = PeerCacheEntry {
            peer_id: create_test_peer_id(2),
            addr_hints: vec!["127.0.0.1:8081".parse().expect("addr")],
            nat_class: NatClass::Unknown,
            roles: vec![],
            last_success: SystemTime::now() - Duration::from_secs(86400),
            success_count: 10,
            failure_count: 0,
        };

        // Recent peer should score higher (recency bonus)
        assert!(entry_recent.score() > entry_old.score());
    }
}

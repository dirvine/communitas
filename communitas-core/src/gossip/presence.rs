// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Presence Management
//!
//! Implements SPEC.md §5: Presence model
//!
//! - "Online" means a valid beacon seen in at least one shared group within TTL
//! - No global presence
//! - UI shows group-scoped presence and last-seen

use chrono::{DateTime, Utc};
use saorsa_gossip_types::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Presence status for a peer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresenceStatus {
    /// Valid beacon seen within TTL
    Online,
    /// No recent beacon
    Offline,
    /// Unknown (never seen or no shared groups)
    Unknown,
}

/// Presence information for a peer
#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub peer_id: PeerId,
    pub status: PresenceStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub shared_groups: Vec<String>, // Group IDs where we've seen this peer
}

/// Presence manager wrapping saorsa-gossip-presence::PresenceManager
pub struct PresenceWrapper {
    #[allow(dead_code)] // Will be used for FOAF discovery integration
    presence_service: Arc<RwLock<saorsa_gossip_presence::PresenceManager>>,
    cache: Arc<RwLock<HashMap<PeerId, PresenceInfo>>>,
}

impl PresenceWrapper {
    /// Create a new presence manager
    pub fn new(presence_service: Arc<RwLock<saorsa_gossip_presence::PresenceManager>>) -> Self {
        Self {
            presence_service,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get presence status for a peer
    pub async fn get_status(&self, peer_id: PeerId) -> PresenceStatus {
        let cache = self.cache.read().await;
        cache
            .get(&peer_id)
            .map(|info| info.status)
            .unwrap_or(PresenceStatus::Unknown)
    }

    /// Get full presence info for a peer
    pub async fn get_info(&self, peer_id: PeerId) -> Option<PresenceInfo> {
        let cache = self.cache.read().await;
        cache.get(&peer_id).cloned()
    }

    /// Get all online peers in a specific group
    pub async fn get_online_in_group(&self, group_id: &str) -> Vec<PeerId> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|info| {
                info.status == PresenceStatus::Online
                    && info.shared_groups.contains(&group_id.to_string())
            })
            .map(|info| info.peer_id)
            .collect()
    }

    /// Find a contact by four-word address using presence beacons
    ///
    /// Per SPEC.md §3: Use Presence::find instead of DHT lookup
    ///
    /// TODO: This requires extending saorsa-gossip-presence::PresenceManager with:
    /// - `get_groups() -> Vec<TopicId>` - List all joined topics
    /// - `get_group_presence(topic) -> HashMap<PeerId, PresenceRecord>` - Get presence records for a topic
    /// - `PresenceRecord` needs to include four_words metadata
    ///
    /// For now, this is a placeholder that returns None
    pub async fn find(&self, _four_words: &str) -> Option<PeerId> {
        // TODO: Implement once PresenceManager API is extended
        // See FOAF_DISCOVERY_IMPLEMENTATION.md for details
        None
    }

    /// Update presence cache from beacon
    #[allow(dead_code)] // Will be called from FOAF discovery protocol
    async fn update_from_beacon(&self, peer_id: PeerId, group_id: String) {
        let mut cache = self.cache.write().await;

        cache
            .entry(peer_id)
            .and_modify(|info| {
                info.status = PresenceStatus::Online;
                info.last_seen = Some(Utc::now());
                if !info.shared_groups.contains(&group_id) {
                    info.shared_groups.push(group_id.clone());
                }
            })
            .or_insert(PresenceInfo {
                peer_id,
                status: PresenceStatus::Online,
                last_seen: Some(Utc::now()),
                shared_groups: vec![group_id],
            });
    }

    /// Clean up expired presence entries (TTL: 10-15 minutes)
    pub async fn cleanup_expired(&self) {
        const TTL_SECONDS: i64 = 15 * 60; // 15 minutes

        let mut cache = self.cache.write().await;
        let now = Utc::now();

        for info in cache.values_mut() {
            if let Some(last_seen) = info.last_seen {
                let age = (now - last_seen).num_seconds();
                if age > TTL_SECONDS {
                    info.status = PresenceStatus::Offline;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_presence_ttl() {
        // Test that presence expires after TTL
        // This is a placeholder - full implementation requires saorsa-gossip integration
    }
}

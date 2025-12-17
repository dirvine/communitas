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
    pub four_words: Option<String>, // Four-word identity if known
    pub last_endpoint: Option<std::net::SocketAddr>, // Last known endpoint for direct dial
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
    /// Searches all joined groups for a peer broadcasting the specified
    /// four-word identity in their presence beacon.
    pub async fn find(&self, four_words: &str) -> Option<PeerId> {
        // First check our local cache
        if let Some(info) = self.get_by_four_words(four_words).await
            && info.status == PresenceStatus::Online
        {
            return Some(info.peer_id);
        }

        // Query the underlying presence service for presence records across all groups
        let presence_guard = self.presence_service.read().await;

        // Get all joined groups
        let groups = presence_guard.get_groups().await;

        // Search for four_words in presence records across all groups
        for topic_id in groups {
            let presence_records = presence_guard.get_group_presence(topic_id).await;

            // Check each presence record for matching four_words
            for (peer_id, record) in presence_records {
                // Skip expired beacons
                if record.is_expired() {
                    continue;
                }

                if let Some(fw) = &record.four_words
                    && fw == four_words
                {
                    // Update our cache for faster future lookups
                    drop(presence_guard); // Release read lock before acquiring write lock
                    self.update_from_beacon_with_endpoint(
                        peer_id,
                        hex::encode(topic_id.as_bytes()),
                        Some(four_words.to_string()),
                        None, // We don't have the endpoint here
                    )
                    .await;
                    return Some(peer_id);
                }
            }
        }

        None
    }

    /// Update presence cache from beacon
    #[allow(dead_code)] // Will be called from FOAF discovery protocol
    async fn update_from_beacon(&self, peer_id: PeerId, group_id: String) {
        self.update_from_beacon_with_endpoint(peer_id, group_id, None, None)
            .await;
    }

    /// Update presence cache from beacon with endpoint tracking
    ///
    /// Call this when receiving a presence beacon to track both presence status
    /// and the peer's endpoint for direct reconnection.
    pub async fn update_from_beacon_with_endpoint(
        &self,
        peer_id: PeerId,
        group_id: String,
        four_words: Option<String>,
        endpoint: Option<std::net::SocketAddr>,
    ) {
        let mut cache = self.cache.write().await;

        cache
            .entry(peer_id)
            .and_modify(|info| {
                info.status = PresenceStatus::Online;
                info.last_seen = Some(Utc::now());
                if !info.shared_groups.contains(&group_id) {
                    info.shared_groups.push(group_id.clone());
                }
                // Update four_words if provided
                if four_words.is_some() {
                    info.four_words = four_words.clone();
                }
                // Update endpoint if provided
                if endpoint.is_some() {
                    info.last_endpoint = endpoint;
                }
            })
            .or_insert(PresenceInfo {
                peer_id,
                status: PresenceStatus::Online,
                last_seen: Some(Utc::now()),
                shared_groups: vec![group_id],
                four_words,
                last_endpoint: endpoint,
            });
    }

    /// Get the last known endpoint for a peer
    pub async fn get_peer_endpoint(&self, peer_id: PeerId) -> Option<std::net::SocketAddr> {
        let cache = self.cache.read().await;
        cache.get(&peer_id).and_then(|info| info.last_endpoint)
    }

    /// Get presence info for peers by their four-word identity
    pub async fn get_by_four_words(&self, four_words: &str) -> Option<PresenceInfo> {
        let cache = self.cache.read().await;
        cache
            .values()
            .find(|info| info.four_words.as_deref() == Some(four_words))
            .cloned()
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

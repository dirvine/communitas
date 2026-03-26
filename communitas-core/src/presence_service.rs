//! Presence Service for Group-Scoped Presence Beacons
//!
//! Implements SPEC.md S5: Presence model
//!
//! With x0x integration, presence is managed by the x0x daemon.
//! This module provides the types and a stub service that can be
//! connected to x0x presence APIs.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Beacon time-to-live (TTL) in seconds (15 minutes per SPEC.md)
pub const BEACON_TTL_SECONDS: i64 = 15 * 60;

/// Beacon broadcast interval (5 minutes)
pub const BEACON_BROADCAST_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Agent identifier for presence tracking
pub type AgentId = String;

/// Topic identifier for group-scoped presence
pub type TopicId = [u8; 32];

/// Presence beacon structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceBeacon {
    /// Agent ID broadcasting the beacon
    pub agent_id: AgentId,

    /// Display name
    pub display_name: String,

    /// Timestamp when beacon was created
    pub timestamp: DateTime<Utc>,

    /// Topic ID (group/channel) where beacon is broadcast
    pub topic_id: TopicId,

    /// Rotating nonce for replay protection
    pub nonce: [u8; 12],
}

impl PresenceBeacon {
    /// Create a new presence beacon
    pub fn new(
        agent_id: AgentId,
        display_name: String,
        topic_id: TopicId,
    ) -> Result<Self, getrandom::Error> {
        let mut nonce = [0u8; 12];
        getrandom::getrandom(&mut nonce)?;

        Ok(Self {
            agent_id,
            display_name,
            timestamp: Utc::now(),
            topic_id,
            nonce,
        })
    }

    /// Check if beacon is still valid (within TTL)
    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        let age = (now - self.timestamp).num_seconds();
        age < BEACON_TTL_SECONDS
    }
}

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
    /// Agent identifier
    pub agent_id: AgentId,
    /// Display name
    pub display_name: String,
    /// Current status
    pub status: PresenceStatus,
    /// When last seen
    pub last_seen: Option<DateTime<Utc>>,
    /// Groups where we've seen this peer
    pub shared_groups: Vec<TopicId>,
}

/// Presence Service
///
/// Manages presence beacons. With x0x integration, this service
/// is backed by the x0x daemon's presence API.
pub struct PresenceService {
    /// Our agent ID
    #[allow(dead_code)]
    agent_id: AgentId,

    /// Our display name
    #[allow(dead_code)]
    display_name: String,

    /// Presence cache (agent_id -> presence info)
    cache: Arc<RwLock<HashMap<AgentId, PresenceInfo>>>,

    /// Topics we're broadcasting presence to
    active_topics: Arc<RwLock<Vec<TopicId>>>,
}

impl PresenceService {
    /// Create a new presence service
    pub fn new(agent_id: AgentId, display_name: String) -> Self {
        Self {
            agent_id,
            display_name,
            cache: Arc::new(RwLock::new(HashMap::new())),
            active_topics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start broadcasting presence to a topic
    pub async fn start_broadcasting(&self, topic_id: TopicId) -> Result<()> {
        let mut active = self.active_topics.write().await;
        if !active.contains(&topic_id) {
            active.push(topic_id);
            info!("Started broadcasting presence to topic {:?}", topic_id);
        }
        Ok(())
    }

    /// Stop broadcasting presence to a topic
    pub async fn stop_broadcasting(&self, topic_id: TopicId) -> Result<()> {
        let mut active = self.active_topics.write().await;
        active.retain(|&id| id != topic_id);
        info!("Stopped broadcasting presence to topic {:?}", topic_id);
        Ok(())
    }

    /// Update presence cache from received beacon
    #[allow(dead_code)]
    async fn update_presence(&self, beacon: PresenceBeacon) {
        let mut cache = self.cache.write().await;

        cache
            .entry(beacon.agent_id.clone())
            .and_modify(|info| {
                info.status = PresenceStatus::Online;
                info.last_seen = Some(beacon.timestamp);
                info.display_name = beacon.display_name.clone();
                if !info.shared_groups.contains(&beacon.topic_id) {
                    info.shared_groups.push(beacon.topic_id);
                }
            })
            .or_insert(PresenceInfo {
                agent_id: beacon.agent_id,
                display_name: beacon.display_name,
                status: PresenceStatus::Online,
                last_seen: Some(beacon.timestamp),
                shared_groups: vec![beacon.topic_id],
            });

        debug!("Updated presence");
    }

    /// Get presence status for a peer
    pub async fn get_status(&self, agent_id: &str) -> PresenceStatus {
        let cache = self.cache.read().await;
        cache
            .get(agent_id)
            .map(|info| info.status)
            .unwrap_or(PresenceStatus::Unknown)
    }

    /// Get full presence info for a peer
    pub async fn get_info(&self, agent_id: &str) -> Option<PresenceInfo> {
        let cache = self.cache.read().await;
        cache.get(agent_id).cloned()
    }

    /// Get all online peers in a specific group
    pub async fn get_online_in_group(&self, topic_id: TopicId) -> Vec<PresenceInfo> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|info| {
                info.status == PresenceStatus::Online && info.shared_groups.contains(&topic_id)
            })
            .cloned()
            .collect()
    }

    /// Get all online peers across all groups
    pub async fn get_all_online(&self) -> Vec<PresenceInfo> {
        let cache = self.cache.read().await;
        cache
            .values()
            .filter(|info| info.status == PresenceStatus::Online)
            .cloned()
            .collect()
    }

    /// Clean up expired presence entries
    #[allow(dead_code)]
    async fn cleanup_expired(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let now = Utc::now();

        for info in cache.values_mut() {
            if let Some(last_seen) = info.last_seen {
                let age = (now - last_seen).num_seconds();
                if age > BEACON_TTL_SECONDS {
                    info.status = PresenceStatus::Offline;
                    debug!(
                        "Marked agent {:?} as offline (age: {}s)",
                        info.agent_id, age
                    );
                }
            }
        }

        Ok(())
    }

    /// Find a peer by agent ID in any shared group
    pub async fn find_by_agent_id(&self, agent_id: &str) -> Option<PresenceInfo> {
        let cache = self.cache.read().await;
        cache
            .values()
            .find(|info| info.agent_id == agent_id && info.status == PresenceStatus::Online)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_agent_id() -> String {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).expect("rng");
        hex::encode(bytes)
    }

    fn random_topic_id() -> TopicId {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).expect("rng");
        bytes
    }

    #[test]
    fn test_beacon_creation() {
        let agent_id = random_agent_id();
        let topic_id = random_topic_id();

        let beacon =
            PresenceBeacon::new(agent_id.clone(), "Test Peer".to_string(), topic_id).unwrap();

        assert_eq!(beacon.agent_id, agent_id);
        assert_eq!(beacon.display_name, "Test Peer");
        assert!(beacon.is_valid());
    }

    #[tokio::test]
    async fn test_presence_service_creation() {
        let agent_id = random_agent_id();

        let service = PresenceService::new(agent_id.clone(), "Test Peer".to_string());

        assert_eq!(service.agent_id, agent_id);
        assert_eq!(service.display_name, "Test Peer");
    }

    #[tokio::test]
    async fn test_presence_status_updates() {
        let agent_id = random_agent_id();

        let service = PresenceService::new(agent_id.clone(), "Test Peer".to_string());

        // Initially unknown
        let status = service.get_status(&agent_id).await;
        assert_eq!(status, PresenceStatus::Unknown);

        // Update with beacon
        let topic_id = random_topic_id();
        let beacon =
            PresenceBeacon::new(agent_id.clone(), "Test Peer".to_string(), topic_id).unwrap();

        service.update_presence(beacon).await;

        // Should be online now
        let status = service.get_status(&agent_id).await;
        assert_eq!(status, PresenceStatus::Online);

        let info = service.get_info(&agent_id).await.unwrap();
        assert_eq!(info.display_name, "Test Peer");
        assert!(info.shared_groups.contains(&topic_id));
    }

    #[tokio::test]
    async fn test_get_online_in_group() {
        let our_id = random_agent_id();
        let service = PresenceService::new(our_id, "Us".to_string());

        let topic1 = random_topic_id();
        let topic2 = random_topic_id();

        let peer1 = random_agent_id();
        let beacon1 =
            PresenceBeacon::new(peer1.clone(), "Peer 1".to_string(), topic1).unwrap();
        service.update_presence(beacon1).await;

        let peer2 = random_agent_id();
        let beacon2 =
            PresenceBeacon::new(peer2.clone(), "Peer 2".to_string(), topic2).unwrap();
        service.update_presence(beacon2).await;

        let online_topic1 = service.get_online_in_group(topic1).await;
        assert_eq!(online_topic1.len(), 1);
        assert_eq!(online_topic1[0].agent_id, peer1);

        let online_topic2 = service.get_online_in_group(topic2).await;
        assert_eq!(online_topic2.len(), 1);
        assert_eq!(online_topic2[0].agent_id, peer2);
    }

    #[tokio::test]
    async fn test_find_by_agent_id() {
        let our_id = random_agent_id();
        let service = PresenceService::new(our_id, "Us".to_string());

        let topic_id = random_topic_id();
        let test_agent = random_agent_id();

        let beacon =
            PresenceBeacon::new(test_agent.clone(), "Alice".to_string(), topic_id).unwrap();

        service.update_presence(beacon).await;

        let found = service.find_by_agent_id(&test_agent).await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().agent_id, test_agent);

        let not_found = service.find_by_agent_id("nonexistent").await;
        assert!(not_found.is_none());
    }
}

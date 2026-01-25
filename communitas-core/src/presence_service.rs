//! Presence Service for Group-Scoped Presence Beacons
//!
//! Implements SPEC.md §5: Presence model
//!
//! - "Online" means a valid beacon seen in at least one shared group within TTL
//! - No global presence
//! - MLS-encrypted beacons using ChaCha20Poly1305
//! - Broadcast via gossip pubsub (Plumtree)

use anyhow::{Context, Result};
use bytes::Bytes;
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};
use chrono::{DateTime, Utc};
use saorsa_gossip_pubsub::PubSub;
use saorsa_gossip_types::{PeerId, TopicId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, info, warn};

/// Beacon time-to-live (TTL) in seconds (15 minutes per SPEC.md)
pub const BEACON_TTL_SECONDS: i64 = 15 * 60;

/// Beacon broadcast interval (5 minutes)
pub const BEACON_BROADCAST_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// Presence beacon structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceBeacon {
    /// Peer ID broadcasting the beacon
    pub peer_id: PeerId,

    /// Four-word identity
    pub four_words: String,

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
        peer_id: PeerId,
        four_words: String,
        display_name: String,
        topic_id: TopicId,
    ) -> Result<Self, getrandom::Error> {
        // Generate random nonce for this beacon
        let mut nonce = [0u8; 12];
        getrandom::getrandom(&mut nonce)?;

        Ok(Self {
            peer_id,
            four_words,
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

    /// Encrypt beacon using ChaCha20Poly1305
    pub fn encrypt(&self, key: &[u8; 32]) -> Result<Vec<u8>> {
        let cipher = ChaCha20Poly1305::new(key.into());

        // Serialize beacon
        let plaintext = bincode::serialize(self).context("Failed to serialize beacon")?;

        // Encrypt with nonce
        let nonce = Nonce::from(self.nonce);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        Ok(ciphertext)
    }

    /// Decrypt beacon using ChaCha20Poly1305
    pub fn decrypt(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Self> {
        let cipher = ChaCha20Poly1305::new(key.into());

        // Decrypt
        let nonce_obj = Nonce::from(*nonce);
        let plaintext = cipher
            .decrypt(&nonce_obj, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        // Deserialize
        let beacon: PresenceBeacon =
            bincode::deserialize(&plaintext).context("Failed to deserialize beacon")?;

        Ok(beacon)
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
    pub peer_id: PeerId,
    pub four_words: String,
    pub display_name: String,
    pub status: PresenceStatus,
    pub last_seen: Option<DateTime<Utc>>,
    pub shared_groups: Vec<TopicId>, // Groups where we've seen this peer
}

/// Presence Service
///
/// Manages presence beacons:
/// - Broadcasts our presence to joined groups
/// - Receives and validates presence beacons from peers
/// - Maintains group-scoped presence cache
pub struct PresenceService {
    /// Our peer ID
    peer_id: PeerId,

    /// Our four-word identity
    four_words: String,

    /// Our display name
    display_name: String,

    /// Pub/sub layer for beacon broadcast
    pubsub: Arc<RwLock<Box<dyn PubSub>>>,

    /// Encryption keys per topic (group-specific keys)
    topic_keys: Arc<RwLock<HashMap<TopicId, [u8; 32]>>>,

    /// Presence cache (peer_id → presence info)
    cache: Arc<RwLock<HashMap<PeerId, PresenceInfo>>>,

    /// Topics we're broadcasting presence to
    active_topics: Arc<RwLock<Vec<TopicId>>>,
}

impl PresenceService {
    /// Create a new presence service
    pub fn new(
        peer_id: PeerId,
        four_words: String,
        display_name: String,
        pubsub: Arc<RwLock<Box<dyn PubSub>>>,
    ) -> Self {
        Self {
            peer_id,
            four_words,
            display_name,
            pubsub,
            topic_keys: Arc::new(RwLock::new(HashMap::new())),
            cache: Arc::new(RwLock::new(HashMap::new())),
            active_topics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Set encryption key for a topic (group)
    pub async fn set_topic_key(&self, topic_id: TopicId, key: [u8; 32]) {
        let mut keys = self.topic_keys.write().await;
        keys.insert(topic_id, key);
        info!("Set encryption key for topic {:?}", topic_id);
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

    /// Broadcast a presence beacon to a specific topic
    pub async fn broadcast_beacon(&self, topic_id: TopicId) -> Result<()> {
        // Get encryption key for this topic
        let key = {
            let keys = self.topic_keys.read().await;
            keys.get(&topic_id)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("No encryption key for topic {:?}", topic_id))?
        };

        // Create beacon
        let beacon = PresenceBeacon::new(
            self.peer_id,
            self.four_words.clone(),
            self.display_name.clone(),
            topic_id,
        )?;

        // Encrypt beacon
        let encrypted = beacon.encrypt(&key)?;

        // Broadcast via pubsub
        let pubsub = self.pubsub.read().await;
        pubsub.publish(topic_id, Bytes::from(encrypted)).await?;

        debug!("Broadcast presence beacon to topic {:?}", topic_id);
        Ok(())
    }

    /// Start periodic beacon broadcasting
    ///
    /// Broadcasts beacons to all active topics every BEACON_BROADCAST_INTERVAL
    pub fn start_beacon_loop(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut timer = interval(BEACON_BROADCAST_INTERVAL);

            loop {
                timer.tick().await;

                // Get active topics
                let topics = self.active_topics.read().await.clone();

                // Broadcast to each topic
                for topic_id in topics {
                    if let Err(e) = self.broadcast_beacon(topic_id).await {
                        warn!("Failed to broadcast beacon to topic {:?}: {}", topic_id, e);
                    }
                }
            }
        })
    }

    /// Start periodic presence cleanup
    ///
    /// Removes expired presence entries every minute
    pub fn start_cleanup_loop(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut timer = interval(Duration::from_secs(60));

            loop {
                timer.tick().await;

                if let Err(e) = self.cleanup_expired().await {
                    warn!("Failed to cleanup expired presence: {}", e);
                }
            }
        })
    }

    /// Handle incoming presence beacon
    pub async fn handle_beacon(&self, topic_id: TopicId, encrypted_data: &[u8]) -> Result<()> {
        // Get decryption key for this topic
        let key = {
            let keys = self.topic_keys.read().await;
            keys.get(&topic_id)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("No decryption key for topic {:?}", topic_id))?
        };

        // Extract nonce (first 12 bytes) and ciphertext
        if encrypted_data.len() < 12 {
            return Err(anyhow::anyhow!("Encrypted data too short"));
        }

        let nonce: [u8; 12] = encrypted_data[..12].try_into()?;
        let ciphertext = &encrypted_data[12..];

        // Decrypt beacon
        let beacon = PresenceBeacon::decrypt(ciphertext, &key, &nonce)?;

        // Validate beacon
        if !beacon.is_valid() {
            debug!("Received expired beacon from {:?}", beacon.peer_id);
            return Ok(());
        }

        // Update presence cache
        self.update_presence(beacon).await;

        Ok(())
    }

    /// Update presence cache from received beacon
    async fn update_presence(&self, beacon: PresenceBeacon) {
        let mut cache = self.cache.write().await;

        cache
            .entry(beacon.peer_id)
            .and_modify(|info| {
                info.status = PresenceStatus::Online;
                info.last_seen = Some(beacon.timestamp);
                info.four_words = beacon.four_words.clone();
                info.display_name = beacon.display_name.clone();
                if !info.shared_groups.contains(&beacon.topic_id) {
                    info.shared_groups.push(beacon.topic_id);
                }
            })
            .or_insert(PresenceInfo {
                peer_id: beacon.peer_id,
                four_words: beacon.four_words,
                display_name: beacon.display_name,
                status: PresenceStatus::Online,
                last_seen: Some(beacon.timestamp),
                shared_groups: vec![beacon.topic_id],
            });

        debug!("Updated presence for peer {:?}", beacon.peer_id);
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
    async fn cleanup_expired(&self) -> Result<()> {
        let mut cache = self.cache.write().await;
        let now = Utc::now();

        for info in cache.values_mut() {
            if let Some(last_seen) = info.last_seen {
                let age = (now - last_seen).num_seconds();
                if age > BEACON_TTL_SECONDS {
                    info.status = PresenceStatus::Offline;
                    debug!("Marked peer {:?} as offline (age: {}s)", info.peer_id, age);
                }
            }
        }

        Ok(())
    }

    /// Find a peer by four-word address in any shared group
    pub async fn find_by_four_words(&self, four_words: &str) -> Option<PresenceInfo> {
        let cache = self.cache.read().await;
        cache
            .values()
            .find(|info| info.four_words == four_words && info.status == PresenceStatus::Online)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_gossip_identity::MlDsaKeyPair;
    use saorsa_gossip_transport::UdpTransportAdapter;
    use saorsa_gossip_types::TopicId;
    use std::net::SocketAddr;

    /// Helper to generate random PeerId
    fn random_peer_id() -> PeerId {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).expect("rng");
        PeerId::new(bytes)
    }

    /// Helper to generate random TopicId
    fn random_topic_id() -> TopicId {
        let mut bytes = [0u8; 32];
        getrandom::getrandom(&mut bytes).expect("rng");
        TopicId::new(bytes)
    }

    #[test]
    fn test_beacon_creation() {
        let peer_id = random_peer_id();
        let topic_id = random_topic_id();

        let beacon = PresenceBeacon::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            topic_id,
        )
        .unwrap(); // OK in tests

        assert_eq!(beacon.peer_id, peer_id);
        assert_eq!(beacon.four_words, "test-peer-one-two");
        assert_eq!(beacon.display_name, "Test Peer");
        assert!(beacon.is_valid());
    }

    #[test]
    fn test_beacon_encryption_roundtrip() {
        let peer_id = random_peer_id();
        let topic_id = random_topic_id();

        let beacon = PresenceBeacon::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            topic_id,
        )
        .unwrap(); // OK in tests

        // Generate encryption key
        let key = [42u8; 32];

        // Encrypt
        let encrypted = beacon.encrypt(&key).expect("encryption should succeed");

        // Decrypt
        let decrypted = PresenceBeacon::decrypt(&encrypted, &key, &beacon.nonce)
            .expect("decryption should succeed");

        assert_eq!(decrypted.peer_id, beacon.peer_id);
        assert_eq!(decrypted.four_words, beacon.four_words);
        assert_eq!(decrypted.display_name, beacon.display_name);
        assert_eq!(decrypted.topic_id, beacon.topic_id);
    }

    #[test]
    fn test_beacon_wrong_key_fails() {
        let peer_id = random_peer_id();
        let topic_id = random_topic_id();

        let beacon = PresenceBeacon::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            topic_id,
        )
        .unwrap(); // OK in tests

        let key1 = [42u8; 32];
        let key2 = [43u8; 32];

        let encrypted = beacon.encrypt(&key1).expect("encryption should succeed");

        // Try to decrypt with wrong key
        let result = PresenceBeacon::decrypt(&encrypted, &key2, &beacon.nonce);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_presence_service_creation() {
        let peer_id = random_peer_id();
        let pubsub = build_test_pubsub(peer_id).await;

        let service = PresenceService::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            pubsub,
        );

        assert_eq!(service.peer_id, peer_id);
        assert_eq!(service.four_words, "test-peer-one-two");
        assert_eq!(service.display_name, "Test Peer");
    }

    #[tokio::test]
    async fn test_presence_status_updates() {
        let peer_id = random_peer_id();
        let pubsub = build_test_pubsub(peer_id).await;

        let service = PresenceService::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            pubsub,
        );

        // Initially unknown
        let status = service.get_status(peer_id).await;
        assert_eq!(status, PresenceStatus::Unknown);

        // Update with beacon
        let topic_id = random_topic_id();
        let beacon = PresenceBeacon::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            topic_id,
        )
        .unwrap(); // OK in tests

        service.update_presence(beacon).await;

        // Should be online now
        let status = service.get_status(peer_id).await;
        assert_eq!(status, PresenceStatus::Online);

        let info = service.get_info(peer_id).await.unwrap();
        assert_eq!(info.four_words, "test-peer-one-two");
        assert_eq!(info.display_name, "Test Peer");
        assert!(info.shared_groups.contains(&topic_id));
    }

    #[tokio::test]
    async fn test_get_online_in_group() {
        let peer_id = random_peer_id();
        let pubsub = build_test_pubsub(peer_id).await;

        let service = PresenceService::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            pubsub,
        );

        let topic1 = random_topic_id();
        let topic2 = random_topic_id();

        // Add peer1 to topic1
        let peer1 = random_peer_id();
        let beacon1 = PresenceBeacon::new(
            peer1,
            "peer-one-alpha-beta".to_string(),
            "Peer 1".to_string(),
            topic1,
        )
        .unwrap(); // OK in tests
        service.update_presence(beacon1).await;

        // Add peer2 to topic2
        let peer2 = random_peer_id();
        let beacon2 = PresenceBeacon::new(
            peer2,
            "peer-two-gamma-delta".to_string(),
            "Peer 2".to_string(),
            topic2,
        )
        .unwrap(); // OK in tests
        service.update_presence(beacon2).await;

        // Check topic1 has only peer1
        let online_topic1 = service.get_online_in_group(topic1).await;
        assert_eq!(online_topic1.len(), 1);
        assert_eq!(online_topic1[0].peer_id, peer1);

        // Check topic2 has only peer2
        let online_topic2 = service.get_online_in_group(topic2).await;
        assert_eq!(online_topic2.len(), 1);
        assert_eq!(online_topic2[0].peer_id, peer2);
    }

    #[tokio::test]
    async fn test_find_by_four_words() {
        let peer_id = random_peer_id();
        let pubsub = build_test_pubsub(peer_id).await;

        let service = PresenceService::new(
            peer_id,
            "test-peer-one-two".to_string(),
            "Test Peer".to_string(),
            pubsub,
        );

        let topic_id = random_topic_id();
        let test_peer_id = random_peer_id();

        let beacon = PresenceBeacon::new(
            test_peer_id,
            "alice-bob-carol-dave".to_string(),
            "Alice".to_string(),
            topic_id,
        )
        .unwrap(); // OK in tests

        service.update_presence(beacon).await;

        // Should find by four-words
        let found = service.find_by_four_words("alice-bob-carol-dave").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().peer_id, test_peer_id);

        // Should not find with wrong four-words
        let not_found = service.find_by_four_words("wrong-four-word-address").await;
        assert!(not_found.is_none());
    }

    async fn build_test_pubsub(peer_id: PeerId) -> Arc<RwLock<Box<dyn PubSub>>> {
        let bind: SocketAddr = "127.0.0.1:0".parse().expect("valid addr");
        let transport = Arc::new(
            UdpTransportAdapter::new(bind, vec![])
                .await
                .expect("transport"),
        );
        let signing = MlDsaKeyPair::generate().expect("keypair");
        Arc::new(RwLock::new(Box::new(
            saorsa_gossip_pubsub::PlumtreePubSub::new(peer_id, transport, signing),
        )))
    }
}

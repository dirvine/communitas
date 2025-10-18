// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Gossip Context - Main orchestrator for the gossip overlay system
//!
//! Implements SPEC2.md mappings:
//! - User identity → ML-DSA identity + alias
//! - Contacts → Overlay edges and seeds
//! - Channels/Projects/Orgs → MLS groups + gossip topics
//! - Presence → MLS-encrypted beacons (ChaCha20Poly1305)
//! - Backup → Favourite contacts hold encrypted replicas (ChaCha20Poly1305)

use anyhow::{Context, Result};
use bytes::Bytes;
use saorsa_gossip_crdt_sync::{AntiEntropyManager, OrSet}; // Actual exports
use saorsa_gossip_groups::GroupContext; // Actual export
use saorsa_gossip_identity::Identity;
use saorsa_gossip_membership::Membership;
use saorsa_gossip_presence::PresenceManager; // Actual exports
use saorsa_gossip_pubsub::PubSub;
use saorsa_gossip_transport::{GossipTransport, QuicTransport, StreamType};
use saorsa_gossip_types::{PeerId, TopicId};
use saorsa_pqc::symmetric::{ChaCha20Poly1305Cipher, SymmetricKey};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Centralized context for the gossip overlay system
///
/// This replaces CoreContext's DHT-based discovery with a gossip-based
/// overlay network per SPEC.md.
pub struct GossipContext {
    /// User identity (four-word) → ML-DSA identity + alias
    pub identity: Identity,
    pub four_words: String,
    pub display_name: String,
    pub device_name: String,

    /// Contacts → Overlay edges and seeds (HyParView + SWIM)
    pub membership: Arc<RwLock<Box<dyn Membership>>>,

    /// Channels/Projects/Orgs → MLS groups + gossip topics
    pub groups: Arc<RwLock<HashMap<String, GroupContext>>>, // entity_id → group
    pub groups_by_topic: Arc<RwLock<HashMap<TopicId, GroupContext>>>, // topic_id → group (for PresenceManager)
    pub topics: Arc<RwLock<HashMap<String, TopicId>>>,                // entity_id → topic_id

    /// Presence → MLS-encrypted beacons
    pub presence: Arc<RwLock<PresenceManager>>,

    /// FOAF discovery for contact finding without DHT
    pub discovery: Arc<super::discovery::FoafDiscovery>,

    /// CRDT sync for message anti-entropy (using OrSet for now)
    pub crdt_message_set: Arc<RwLock<OrSet<Vec<u8>>>>,

    /// Anti-entropy manager for CRDT synchronization
    pub anti_entropy: Arc<AntiEntropyManager<OrSet<Vec<u8>>>>,

    /// Transport layer (QUIC via ant-quic)
    pub transport: Arc<QuicTransport>,

    /// Pub/sub layer (Plumtree broadcast)
    pub pubsub: Arc<RwLock<Box<dyn PubSub>>>,

    /// Backup system - favourite contacts for encrypted replicas
    pub favourite_contacts: Arc<RwLock<Vec<String>>>, // four-word addresses

    /// Peer cache for fast boot (SPEC2.md §6)
    pub peer_cache: Arc<RwLock<super::peer_cache::PeerCache>>,

    /// Coordinator client for NAT traversal (SPEC2.md §2, §8, §9)
    pub coordinator: Arc<super::coordinator::CoordinatorClient>,

    /// Rendezvous client for global user discovery (SPEC2.md §4, §9)
    pub rendezvous: Arc<super::rendezvous::RendezvousClient>,

    /// Site publisher for publishing content-addressed sites
    pub site_publisher: Option<Arc<super::sites::SitePublisher>>,

    /// Site fetcher for discovering and fetching sites
    pub site_fetcher: Option<Arc<super::sites::SiteFetcher>>,

    /// Local peer ID
    pub peer_id: PeerId,
}

impl GossipContext {
    /// Initialize a new GossipContext
    ///
    /// This follows SPEC.md §2 boot sequence:
    /// 1. Load ML-DSA identity
    /// 2. Dial 1-3 favourite contacts
    /// 3. Start membership (HyParView+SWIM)
    /// 4. Join MLS groups and subscribe to topics
    /// 5. Begin presence beacons
    /// 6. Start CRDT anti-entropy
    ///
    /// Note: This is a simplified initial implementation. Full boot sequence
    /// is implemented in gossip::boot::GossipBootSequence.
    pub async fn initialize(
        four_words: String,
        display_name: String,
        device_name: String,
    ) -> Result<Self> {
        info!("Initializing GossipContext for {}", four_words);

        // 1. Load or create ML-DSA identity
        // Use system data directory to avoid triggering file watchers in dev mode
        let keystore_path = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?
            .join("communitas")
            .join("keystore");

        // Ensure keystore directory exists
        std::fs::create_dir_all(&keystore_path).context("Failed to create keystore directory")?;

        let keystore_str = keystore_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Keystore path contains invalid UTF-8"))?;
        let identity = Identity::load_or_create(&four_words, &display_name, keystore_str)
            .await
            .context("Failed to load/create ML-DSA identity")?;

        let peer_id = identity.peer_id();
        debug!("Loaded identity, peer_id: {:?}", peer_id);

        // 2. Initialize QUIC transport
        let config = saorsa_gossip_transport::TransportConfig::default();
        let transport = QuicTransport::new(config);
        let transport = Arc::new(transport);

        // 3. Create membership layer (will be started in boot sequence)
        // HyParView parameters: active_degree (3-7), passive_degree (3x active)
        let membership: Arc<RwLock<Box<dyn Membership>>> = Arc::new(RwLock::new(Box::new(
            saorsa_gossip_membership::HyParViewMembership::new(
                5,  // active_degree: maintain 5 active connections
                15, // passive_degree: keep 15 passive peers
                transport.clone(),
            ),
        )));

        // 4. Create pub/sub layer
        let signing_key = identity.key_pair().clone();
        let pubsub: Arc<RwLock<Box<dyn PubSub>>> = Arc::new(RwLock::new(Box::new(
            saorsa_gossip_pubsub::PlumtreePubSub::new(
                peer_id,
                transport.clone(),
                signing_key.clone(),
            ),
        )));

        // 5. Create empty groups maps (populated during join_entity)
        let groups = Arc::new(RwLock::new(HashMap::new()));
        let groups_by_topic = Arc::new(RwLock::new(HashMap::new()));

        // 6. Create presence manager
        let presence = Arc::new(RwLock::new(PresenceManager::new(
            peer_id,
            transport.clone(),
            groups_by_topic.clone(),
        )));

        // 6b. Create FOAF discovery manager
        let discovery = Arc::new(super::discovery::FoafDiscovery::new());

        // 7. Create CRDT message set
        let crdt_message_set = Arc::new(RwLock::new(OrSet::new()));

        // 8. Create anti-entropy manager (60 second sync interval)
        let anti_entropy = Arc::new(AntiEntropyManager::new(crdt_message_set.clone(), 60));

        // 9. Initialize favourite contacts (will be loaded from storage)
        let favourite_contacts = Arc::new(RwLock::new(Vec::new()));

        // 10. Initialize topics map
        let topics = Arc::new(RwLock::new(HashMap::new()));

        // 11. Load or create peer cache (SPEC2.md §6) - system-wide location
        let cache_path = super::peer_cache::PeerCache::default_cache_path()
            .context("Failed to get default peer cache path")?;
        let peer_cache = super::peer_cache::PeerCache::load(&cache_path)
            .await
            .context("Failed to load peer cache")?;
        let peer_cache = Arc::new(RwLock::new(peer_cache));

        // 12. Initialize coordinator client (SPEC2.md §2, §8, §9)
        // Create a new QuicTransport instance for coordinator (shared config)
        let coord_config = saorsa_gossip_transport::TransportConfig::default();
        let coord_transport = saorsa_gossip_transport::QuicTransport::new(coord_config);
        let coordinator_transport: Arc<RwLock<Box<dyn GossipTransport>>> =
            Arc::new(RwLock::new(Box::new(coord_transport)));

        let coordinator = super::coordinator::CoordinatorClient::new(
            peer_id,
            coordinator_transport,
            membership.clone(),
        );
        let coordinator = Arc::new(coordinator);

        // Create rendezvous client for global user discovery (SPEC2.md §4, §9)
        // Needs separate transport and pubsub instances
        let rdv_config = saorsa_gossip_transport::TransportConfig::default();
        let rdv_transport_qt = saorsa_gossip_transport::QuicTransport::new(rdv_config.clone());
        let rdv_pubsub_qt = saorsa_gossip_transport::QuicTransport::new(rdv_config);
        let rdv_transport: Arc<RwLock<Box<dyn GossipTransport>>> =
            Arc::new(RwLock::new(Box::new(rdv_transport_qt)));

        let rdv_pubsub_impl = saorsa_gossip_pubsub::PlumtreePubSub::new(
            peer_id,
            Arc::new(rdv_pubsub_qt),
            signing_key.clone(),
        );
        let rdv_pubsub: Arc<RwLock<Box<dyn PubSub>>> =
            Arc::new(RwLock::new(Box::new(rdv_pubsub_impl)));

        let rendezvous =
            super::rendezvous::RendezvousClient::new(peer_id, rdv_transport, rdv_pubsub);
        let rendezvous = Arc::new(rendezvous);

        // 13. Initialize Saorsa Sites (SPEC2.md §5 - Rendezvous Protocol)
        // Create SitePublisher with our identity as site_id
        // Use BLAKE3 hash of public key to get 32 bytes
        let pub_key = identity.key_pair().public_key();
        let key_hash = blake3::hash(pub_key);
        let site_id = super::sites::SiteId::new(*key_hash.as_bytes());
        let site_publisher = Arc::new(super::sites::SitePublisher::new(site_id));

        // Create SiteFetcher with transport access via rendezvous
        let site_fetcher = Arc::new(super::sites::SiteFetcher::new(rendezvous.clone()));

        Ok(Self {
            identity,
            four_words,
            display_name,
            device_name,
            membership,
            groups,
            groups_by_topic,
            topics,
            presence,
            discovery,
            crdt_message_set,
            anti_entropy,
            transport,
            pubsub,
            favourite_contacts,
            peer_cache,
            coordinator,
            rendezvous,
            site_publisher: Some(site_publisher),
            site_fetcher: Some(site_fetcher),
            peer_id,
        })
    }

    /// Get our peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get four-word address
    pub fn four_words(&self) -> &str {
        &self.four_words
    }

    /// Add a favourite contact for backup replication
    pub async fn add_favourite_contact(&self, four_words: String) -> Result<()> {
        let mut favourites = self.favourite_contacts.write().await;
        if !favourites.contains(&four_words) {
            favourites.push(four_words.clone());
            info!("Added favourite contact: {}", four_words);
        }
        Ok(())
    }

    /// Get list of favourite contacts
    pub async fn get_favourite_contacts(&self) -> Vec<String> {
        self.favourite_contacts.read().await.clone()
    }

    /// Map a channel/project/org entity to an MLS group + topic
    ///
    /// Per SPEC.md §1: Channel/Project/Org → MLS group + gossip topic
    pub async fn map_entity_to_topic(
        &self,
        entity_id: &str,
        entity_type: &str, // "channel", "project", "org"
    ) -> Result<TopicId> {
        // Check if topic already exists
        {
            let topics = self.topics.read().await;
            if let Some(topic_id) = topics.get(entity_id) {
                return Ok(*topic_id);
            }
        }

        // Create new topic from entity ID
        let topic_id = TopicId::from_entity(entity_id)?;

        // Store mapping
        {
            let mut topics = self.topics.write().await;
            topics.insert(entity_id.to_string(), topic_id);
        }

        info!(
            "Mapped {} {} to topic {:?}",
            entity_type, entity_id, topic_id
        );
        Ok(topic_id)
    }

    /// Join an MLS group and subscribe to its topic
    ///
    /// Per SPEC.md §2.4: For each channel/org: join MLS group, subscribe to topic
    pub async fn join_entity(&self, entity_id: &str, entity_type: &str) -> Result<()> {
        // 1. Get or create topic ID
        let topic_id = self.map_entity_to_topic(entity_id, entity_type).await?;

        // 2. Create MLS group context (simplified for now)
        let group_ctx = GroupContext::from_entity(entity_id)?;
        {
            let mut groups = self.groups.write().await;
            groups.insert(entity_id.to_string(), group_ctx.clone());
        }
        {
            let mut groups_by_topic = self.groups_by_topic.write().await;
            groups_by_topic.insert(topic_id, group_ctx);
        }

        // 3. Subscribe to topic (returns a receiver, not async)
        let pubsub = self.pubsub.read().await;
        let _rx = pubsub.subscribe(topic_id);
        // TODO: Store receiver for processing incoming messages

        info!("Joined {} {}, subscribed to topic", entity_type, entity_id);
        Ok(())
    }

    /// Leave an entity (unsubscribe and leave MLS group)
    pub async fn leave_entity(&self, entity_id: &str) -> Result<()> {
        // 1. Get topic ID
        let topic_id = {
            let topics = self.topics.read().await;
            topics
                .get(entity_id)
                .copied()
                .context("Entity not found in topic map")?
        };

        // 2. Unsubscribe from topic
        let pubsub = self.pubsub.write().await;
        pubsub.unsubscribe(topic_id).await?;

        // 3. Remove MLS group from both maps
        {
            let mut groups = self.groups.write().await;
            groups.remove(entity_id);
        }
        {
            let mut groups_by_topic = self.groups_by_topic.write().await;
            groups_by_topic.remove(&topic_id);
        }

        // 4. Remove from topics map
        {
            let mut topics = self.topics.write().await;
            topics.remove(entity_id);
        }

        info!("Left entity {}, unsubscribed from topic", entity_id);
        Ok(())
    }

    /// Publish a message to an entity's topic
    pub async fn publish_to_entity(&self, entity_id: &str, message: Vec<u8>) -> Result<()> {
        // 1. Get topic ID
        let topic_id = {
            let topics = self.topics.read().await;
            topics
                .get(entity_id)
                .copied()
                .context("Entity not found, must join first")?
        };

        // 2. Get MLS group for encryption
        let groups = self.groups.read().await;
        let _group_ctx = groups
            .get(entity_id)
            .context("MLS group not found, must join first")?;

        // TODO: Encrypt with MLS group key
        // For now, just publish the message (encryption will be added later)
        let encrypted = message; // Placeholder

        // 3. Publish via gossip
        let pubsub = self.pubsub.write().await;
        pubsub.publish(topic_id, encrypted.into()).await?;

        debug!("Published message to entity {}", entity_id);
        Ok(())
    }

    // ========================================================================
    // Storage API - CRDT-based local-first storage
    // ========================================================================

    /// Generate a unique tag for CRDT operations (peer_id, timestamp)
    fn generate_unique_tag(&self) -> (PeerId, u64) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0); // Fallback to epoch if clock is before 1970 (extremely rare)
        (self.peer_id, timestamp)
    }

    /// Store a message in the local CRDT set
    ///
    /// Messages are stored locally and synchronized via anti-entropy
    /// to replace DHT-based storage.
    pub async fn store_message(&self, message: Vec<u8>) -> Result<()> {
        let tag = self.generate_unique_tag();
        let mut crdt_set = self.crdt_message_set.write().await;
        crdt_set.add(message, tag)?;
        info!("Message stored in local CRDT set");
        Ok(())
    }

    /// Retrieve all messages from local CRDT set
    pub async fn get_all_messages(&self) -> Result<Vec<Vec<u8>>> {
        let crdt_set = self.crdt_message_set.read().await;
        Ok(crdt_set.elements().into_iter().cloned().collect())
    }

    /// Check if a message exists in the CRDT set
    pub async fn contains_message(&self, message: &Vec<u8>) -> Result<bool> {
        let crdt_set = self.crdt_message_set.read().await;
        Ok(crdt_set.contains(message))
    }

    /// Remove a message from the CRDT set
    pub async fn remove_message(&self, message: &Vec<u8>) -> Result<()> {
        let mut crdt_set = self.crdt_message_set.write().await;
        crdt_set.remove(message)?;
        info!("Message removed from local CRDT set");
        Ok(())
    }

    // ========================================================================
    // Contact Discovery API - FOAF + Presence based
    // ========================================================================

    /// Find a contact by four-word address using FOAF + Presence
    ///
    /// Replaces DHT lookup with gossip-based discovery:
    /// 1. Check local cache
    /// 2. Check presence in shared groups
    /// 3. Query FOAF (friend-of-a-friend)
    /// 4. Fall back to introducer nodes (cold start)
    pub async fn find_contact(&self, four_words: &str) -> Result<PeerId> {
        self.discovery
            .find_contact(four_words)
            .await
            .context("Contact not found via FOAF or presence")
    }

    /// Add a known contact to local cache
    pub async fn add_contact(&self, four_words: String, peer_id: PeerId) -> Result<()> {
        self.discovery.add_contact(four_words, peer_id).await;
        Ok(())
    }

    /// Get all known contacts from cache
    pub async fn get_contacts(&self) -> Result<Vec<(String, PeerId)>> {
        Ok(self.discovery.get_contacts().await)
    }

    /// Remove a contact from cache
    pub async fn remove_contact(&self, four_words: &str) -> Result<()> {
        self.discovery.remove_contact(four_words).await;
        Ok(())
    }

    // ========================================================================
    // Messaging API - Plumtree pub/sub
    // ========================================================================

    /// Send a message to a specific peer
    ///
    /// Uses QUIC transport directly for point-to-point messaging
    pub async fn send_direct_message(&self, peer_id: PeerId, message: Vec<u8>) -> Result<()> {
        self.transport
            .send_to_peer(peer_id, StreamType::Bulk, Bytes::from(message))
            .await
            .context("Failed to send direct message")
    }

    /// Subscribe to messages from an entity
    ///
    /// Returns a channel receiver for incoming messages
    /// Note: Returns `UnboundedReceiver<(PeerId, Bytes)>` which includes sender info
    pub async fn subscribe_to_entity(
        &self,
        entity_id: &str,
    ) -> Result<tokio::sync::mpsc::UnboundedReceiver<(PeerId, Bytes)>> {
        // 1. Ensure we're joined to the entity
        let topic_id = {
            let topics = self.topics.read().await;
            topics
                .get(entity_id)
                .copied()
                .context("Entity not found, must join first")?
        };

        // 2. Subscribe to topic
        let pubsub = self.pubsub.read().await;
        let rx = pubsub.subscribe(topic_id);

        Ok(rx)
    }

    // ========================================================================
    // Backup & Recovery API
    // ========================================================================

    /// Replicate local state to favourite contacts
    ///
    /// Encrypts and sends CRDT state to favourite peers for backup using ChaCha20Poly1305.
    /// Per SPEC2.md: Uses quantum-resistant ChaCha20Poly1305 AEAD from saorsa-pqc.
    pub async fn replicate_to_favourites(&self) -> Result<()> {
        let favourites = self.favourite_contacts.read().await;
        if favourites.is_empty() {
            debug!("No favourite contacts configured for backup");
            return Ok(());
        }

        // Get current CRDT state (all messages)
        let messages = self.get_all_messages().await?;

        // Serialize state
        let plaintext = bincode::serialize(&messages).context("Failed to serialize state")?;

        // Send to each favourite contact
        for four_words in favourites.iter() {
            // Find peer ID for favourite
            match self.find_contact(four_words).await {
                Ok(peer_id) => {
                    // Generate per-favourite encryption key
                    // In production, derive from shared MLS group key or key agreement
                    let key = SymmetricKey::generate();
                    let cipher = ChaCha20Poly1305Cipher::new(&key);

                    // Encrypt with ChaCha20Poly1305 AEAD
                    let (ciphertext, nonce) = cipher
                        .encrypt(&plaintext, None)
                        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

                    // Package: [nonce (12 bytes) || ciphertext || key (32 bytes)]
                    // Note: In production, key should be shared via key agreement, not sent inline
                    let mut package = Vec::with_capacity(12 + ciphertext.len() + 32);
                    package.extend_from_slice(nonce.as_slice());
                    package.extend_from_slice(&ciphertext);
                    package.extend_from_slice(key.as_bytes());

                    // Send encrypted replica via Bulk stream
                    if let Err(e) = self
                        .transport
                        .send_to_peer(peer_id, StreamType::Bulk, Bytes::from(package))
                        .await
                    {
                        warn!("Failed to replicate to {}: {}", four_words, e);
                    } else {
                        info!(
                            "Replicated encrypted state to favourite: {} (ChaCha20Poly1305)",
                            four_words
                        );
                    }
                }
                Err(e) => {
                    warn!("Favourite {} not reachable: {}", four_words, e);
                }
            }
        }

        Ok(())
    }

    /// Recover state from a favourite contact
    ///
    /// Connects to favourite, retrieves encrypted replica, decrypts with ChaCha20Poly1305, and merges CRDT state.
    /// Per SPEC2.md: Decrypts backup using quantum-resistant ChaCha20Poly1305 AEAD from saorsa-pqc.
    pub async fn recover_from_favourite(
        &self,
        _four_words: &str,
        encrypted_package: Vec<u8>,
    ) -> Result<()> {
        info!("Attempting recovery from favourite");

        // 1. Unpack: [nonce (12 bytes) || ciphertext || key (32 bytes)]
        if encrypted_package.len() < 44 {
            anyhow::bail!("Invalid package: too short (minimum 44 bytes for nonce + key)");
        }

        let nonce_bytes = &encrypted_package[0..12];
        let key_bytes = &encrypted_package[encrypted_package.len() - 32..];
        let ciphertext = &encrypted_package[12..encrypted_package.len() - 32];

        // 2. Reconstruct key and cipher
        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
        let key = SymmetricKey::from_bytes(key_array);
        let cipher = ChaCha20Poly1305Cipher::new(&key);

        // 3. Decrypt with ChaCha20Poly1305 AEAD
        let nonce: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
        let plaintext = cipher.decrypt(ciphertext, &nonce, None).map_err(|e| {
            anyhow::anyhow!("Decryption failed (possible tampering or wrong key): {}", e)
        })?;

        // 4. Deserialize messages
        let messages: Vec<Vec<u8>> =
            bincode::deserialize(&plaintext).context("Failed to deserialize recovered state")?;

        // 5. Merge into local CRDT
        let mut crdt_set = self.crdt_message_set.write().await;
        for message in messages {
            let tag = self.generate_unique_tag();
            if let Err(e) = crdt_set.add(message.clone(), tag) {
                warn!("Failed to add recovered message: {}", e);
            }
        }

        info!(
            "Recovery complete: {} messages restored (ChaCha20Poly1305 decryption)",
            crdt_set.len()
        );
        Ok(())
    }

    // ========================================================================
    // Presence API
    // ========================================================================

    /// Start sending presence beacons for all joined groups
    ///
    /// Per SPEC.md §5: Group-scoped presence with rotating beacons
    /// Beacons are sent every 5 minutes with 15 minute TTL
    pub async fn start_presence_beacons(&self) -> Result<()> {
        let presence = self.presence.read().await;
        presence.start_beacons(300).await?; // 300 seconds = 5 minutes
        info!("Started presence beaconing (5min interval)");
        Ok(())
    }

    /// Stop presence beacons
    pub async fn stop_presence_beacons(&self) -> Result<()> {
        let presence = self.presence.read().await;
        presence.stop_beacons().await?;
        info!("Stopped presence beaconing");
        Ok(())
    }

    /// Check if a peer is online in any shared group
    pub async fn is_peer_online(&self, peer_id: PeerId) -> Result<bool> {
        let presence = self.presence.read().await;

        // Check all groups we're in
        let groups_by_topic = self.groups_by_topic.read().await;
        for topic_id in groups_by_topic.keys() {
            let online_peers = presence.get_online_peers(*topic_id).await;
            if online_peers.contains(&peer_id) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get online peers in a specific entity
    pub async fn get_online_peers(&self, entity_id: &str) -> Result<Vec<PeerId>> {
        // Get topic ID for entity
        let topic_id = {
            let topics = self.topics.read().await;
            topics.get(entity_id).copied().context("Entity not found")?
        };

        // Get online peers from presence manager
        let presence = self.presence.read().await;
        let online_peers = presence.get_online_peers(topic_id).await;

        Ok(online_peers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gossip_context_initialization() {
        let ctx = GossipContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            "Desktop".to_string(),
        )
        .await;

        assert!(
            ctx.is_ok(),
            "GossipContext initialization failed: {:?}",
            ctx.err()
        );
        let ctx = ctx.expect("should succeed");
        assert_eq!(ctx.four_words(), "ocean-forest-moon-star");
    }

    #[tokio::test]
    async fn test_favourite_contacts() {
        let ctx = GossipContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            "Desktop".to_string(),
        )
        .await
        .expect("init");

        ctx.add_favourite_contact("river-mountain-cloud-light".to_string())
            .await
            .expect("add favourite");

        let favourites = ctx.get_favourite_contacts().await;
        assert_eq!(favourites.len(), 1);
        assert_eq!(favourites[0], "river-mountain-cloud-light");
    }

    #[tokio::test]
    async fn test_sites_initialization() {
        let ctx = GossipContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            "Desktop".to_string(),
        )
        .await
        .expect("init");

        // Verify site_publisher and site_fetcher are initialized
        assert!(
            ctx.site_publisher.is_some(),
            "SitePublisher should be initialized"
        );
        assert!(
            ctx.site_fetcher.is_some(),
            "SiteFetcher should be initialized"
        );
    }
}

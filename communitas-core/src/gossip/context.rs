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

use super::CoordinatorRoles;
use anyhow::{Context, Result};
use bytes::Bytes;
use saorsa_gossip_coordinator::{
    CoordinatorPublisher, NatClass, PeriodicPublisher, coordinator_topic,
};
use saorsa_gossip_crdt_sync::{AntiEntropyManager, OrSet}; // Actual exports
use saorsa_gossip_groups::GroupContext; // Actual export
use saorsa_gossip_identity::Identity;
use saorsa_gossip_membership::Membership;
use saorsa_gossip_presence::PresenceManager; // Actual exports
use saorsa_gossip_pubsub::PubSub;
use saorsa_gossip_transport::{
    GossipStreamType, GossipTransport, TransportAdapter, UdpTransportAdapter,
    UdpTransportAdapterConfig,
};
use saorsa_gossip_types::{PeerId, TopicId};
use saorsa_pqc::symmetric::{ChaCha20Poly1305Cipher, SymmetricKey};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use super::self_filtering_transport::SelfFilteringTransport;

// Phase 2 TDD: Import resilience modules
use crate::{ConnectivityWatchdog, ResourceLimits, WatchdogConfig};

// Contact storage for endpoint tracking
use super::contact_storage::{ContactRecord, ContactStore};

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct DirectEntityEnvelope {
    pub entity_id: String,
    pub payload: Vec<u8>,
}

fn direct_entity_broadcast_enabled() -> bool {
    matches!(
        std::env::var("COMMUNITAS_DIRECT_ENTITY_BROADCAST")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes"
    )
}

/// Type alias for entity message handler callback
/// Called with (entity_id, sender_peer_id, message_bytes)
pub type EntityMessageHandler = Arc<dyn Fn(String, PeerId, Bytes) + Send + Sync>;

/// Centralized context for the gossip overlay system
///
/// This replaces CoreContext's DHT-based discovery with a gossip-based
/// overlay network per SPEC.md.
///
/// ## Lock Ordering (MUST follow to prevent deadlock)
///
/// When acquiring multiple locks, always acquire in this order:
///
/// 1. `pubsub` (highest priority)
/// 2. `membership`
/// 3. `groups` / `groups_by_topic` (groups first)
/// 4. `topics`
/// 5. `entity_types`
/// 6. `pending_sync_entities`
/// 7. `entity_receiver_tasks`
/// 8. `presence`
/// 9. `crdt_message_set`
/// 10. `favourite_contacts`
/// 11. `entity_message_handler` (lowest priority)
///
/// **Rule**: Never hold a lower-priority lock while acquiring a higher-priority one.
/// **Rule**: Prefer dropping locks before acquiring new ones (clone-and-drop pattern).
/// **Rule**: Keep lock hold durations as short as possible.
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
    /// Entity types for joined entities (entity_id → entity_type)
    entity_types: Arc<RwLock<HashMap<String, String>>>,
    /// Entities that joined before peers connected and still need initial sync
    pending_sync_entities: Arc<RwLock<HashSet<String>>>,

    /// Presence → MLS-encrypted beacons
    pub presence: Arc<RwLock<PresenceManager>>,

    /// FOAF discovery for contact finding without DHT
    pub discovery: Arc<super::discovery::FoafDiscovery>,

    /// CRDT sync for message anti-entropy (using OrSet for now)
    pub crdt_message_set: Arc<RwLock<OrSet<Vec<u8>>>>,

    /// Anti-entropy manager for CRDT synchronization
    pub anti_entropy: Arc<AntiEntropyManager<OrSet<Vec<u8>>>>,

    /// Transport layer (QUIC via ant-quic), with self-send filtering
    pub transport: Arc<SelfFilteringTransport<UdpTransportAdapter>>,

    /// Pub/sub layer (Plumtree broadcast)
    pub pubsub: Arc<RwLock<Box<dyn PubSub>>>,

    /// Backup system - favourite contacts for encrypted replicas
    pub favourite_contacts: Arc<RwLock<Vec<String>>>, // four-word addresses

    /// Peer cache for fast boot (SPEC2.md §6)
    pub peer_cache: Arc<super::peer_cache::PeerCache>,

    /// Coordinator client for NAT traversal (SPEC2.md §2, §8, §9)
    pub coordinator: Arc<super::CoordinatorClient>,

    /// Rendezvous client for global user discovery (SPEC2.md §4, §9)
    pub rendezvous: Arc<super::RendezvousClient>,

    /// Site publisher for publishing content-addressed sites
    pub site_publisher: Option<Arc<super::sites::SitePublisher>>,

    /// Site fetcher for discovering and fetching sites
    pub site_fetcher: Option<Arc<super::sites::SiteFetcher>>,

    /// Sites dispatcher coordinating shared transport routing
    pub sites_dispatcher: Option<Arc<super::sites_dispatcher::SitesDispatcher>>,

    /// Sites protocol listener (routes incoming requests to publisher)
    pub sites_listener: Option<Arc<super::sites_listener::SitesListener>>,

    /// Sites listener task handle
    #[allow(dead_code)]
    sites_listener_handle: Option<tokio::task::JoinHandle<()>>,

    /// Name registry for DNS-free name resolution (four-words → SiteId)
    pub name_registry: Option<Arc<super::name_record::NameRegistry>>,

    /// Local peer ID
    pub peer_id: PeerId,

    /// Connectivity watchdog for internet collapse detection (Phase 2 TDD)
    pub watchdog: Arc<ConnectivityWatchdog>,

    /// Resource limits for connection/memory management (Phase 2 TDD)
    pub resource_limits: Arc<ResourceLimits>,

    /// Contact store for endpoint tracking (per SPEC2.md contact management)
    pub contact_store: ContactStore,

    /// Entity message handler callback for incoming gossip messages
    /// Set via set_entity_message_handler() to receive entity messages
    entity_message_handler: Arc<RwLock<Option<EntityMessageHandler>>>,

    /// Background tasks for entity subscription receivers
    /// entity_id → JoinHandle for the receiver task
    entity_receiver_tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,

    /// Listen port for connection identity encoding
    /// Stored during initialization for use by get_my_connection_words()
    listen_port: Option<u16>,
}

impl GossipContext {
    fn join_timeout() -> std::time::Duration {
        std::env::var("COMMUNITAS_JOIN_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .map(std::time::Duration::from_secs)
            .unwrap_or_else(|| std::time::Duration::from_secs(10))
    }
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
        listen_port: Option<u16>,
    ) -> Result<Self> {
        info!(
            "Initializing GossipContext for {} (port: {:?})",
            four_words, listen_port
        );

        // 1. Load or create ML-DSA identity
        // Prefer COMMUNITAS_DATA_DIR when provided (e.g., local cluster isolation),
        // otherwise fall back to the system data directory.
        let data_root = if let Ok(dir) = std::env::var("COMMUNITAS_DATA_DIR") {
            std::path::PathBuf::from(dir)
        } else {
            dirs::data_local_dir()
                .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?
                .join("communitas")
        };
        std::fs::create_dir_all(&data_root).context("Failed to create data directory")?;

        let keystore_path = data_root.join("keystore");
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

        // 2. Load or create peer cache (SPEC2.md §6) - system-wide location
        let cache_dir = data_root.join("bootstrap-cache");
        let peer_cache = Arc::new(
            super::peer_cache::PeerCache::open(&cache_dir)
                .await
                .context("Failed to open bootstrap cache")?,
        );

        // 3. Initialize QUIC transport (UdpTransportAdapter)
        // Use listen_port if provided, otherwise bind to port 0 (OS-assigned)
        // Use Bootstrap role to allow starting without upstream bootstrap nodes
        let bind_port = listen_port.unwrap_or(0);
        let bind_ip = std::env::var("COMMUNITAS_LOCAL_IP")
            .ok()
            .and_then(|ip| ip.parse::<std::net::IpAddr>().ok())
            .unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
        let bind_addr = std::net::SocketAddr::new(bind_ip, bind_port);

        // Extract keypair bytes from identity to ensure transport uses the same peer ID
        let keypair = identity.key_pair();
        let pub_key_bytes = keypair.public_key().to_vec();
        let sec_key_bytes = keypair.secret_key().to_vec();

        let mut transport_config = UdpTransportAdapterConfig::new(bind_addr, vec![])
            .with_keypair(pub_key_bytes.clone(), sec_key_bytes.clone());

        let max_inflight = std::env::var("COMMUNITAS_TRANSPORT_MAX_INFLIGHT")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(16);
        transport_config = transport_config.with_max_inflight_sends(max_inflight);

        let send_timeout_secs = std::env::var("COMMUNITAS_TRANSPORT_SEND_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(30);
        transport_config =
            transport_config.with_send_timeout(Duration::from_secs(send_timeout_secs));

        let transport =
            UdpTransportAdapter::with_config(transport_config, Some(peer_cache.bootstrap_cache()))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create UdpTransportAdapter: {}", e))?;
        let transport = Arc::new(transport);
        let transport = Arc::new(SelfFilteringTransport::new(transport));

        // No need to call listen() for UdpTransportAdapter as it binds on creation.

        // 4. Create membership layer (will be started in boot sequence)
        // HyParView parameters: active_degree (3-7), passive_degree (3x active)
        let membership: Arc<RwLock<Box<dyn Membership>>> = Arc::new(RwLock::new(Box::new(
            saorsa_gossip_membership::HyParViewMembership::new(
                peer_id,
                saorsa_gossip_membership::MembershipConfig {
                    active_degree: 5,
                    max_active_degree: 10,
                    max_passive_degree: 64,
                    ..saorsa_gossip_membership::MembershipConfig::default()
                },
                transport.clone(),
            ),
        )));

        // 5. Create pub/sub layer
        let signing_key = identity.key_pair().clone();
        let pubsub: Arc<RwLock<Box<dyn PubSub>>> = Arc::new(RwLock::new(Box::new(
            saorsa_gossip_pubsub::PlumtreePubSub::new(
                peer_id,
                transport.clone(),
                signing_key.clone(),
            ),
        )));

        // 6. Create empty groups maps (populated during join_entity)
        let groups = Arc::new(RwLock::new(HashMap::new()));
        let groups_by_topic = Arc::new(RwLock::new(HashMap::new()));

        // 7. Create presence manager
        let presence = Arc::new(RwLock::new(PresenceManager::new_with_four_words(
            peer_id,
            transport.clone(),
            groups_by_topic.clone(),
            Some(four_words.clone()),
        )));
        {
            let presence_guard = presence.write().await;
            presence_guard
                .set_addr_hints(vec![bind_addr.to_string()])
                .await;
        }

        // 7b. Create FOAF discovery manager with presence integration
        let discovery = Arc::new(super::discovery::FoafDiscovery::with_config(
            Some(presence.clone()),
            None,
            peer_id,
            2,
        ));

        // 8. Create CRDT message set
        let crdt_message_set = Arc::new(RwLock::new(OrSet::new()));

        // 9. Create anti-entropy manager (60 second sync interval)
        let anti_entropy = Arc::new(AntiEntropyManager::new(crdt_message_set.clone(), 60));

        // 10. Initialize favourite contacts (will be loaded from storage)
        let favourite_contacts = Arc::new(RwLock::new(Vec::new()));

        // 11. Initialize topics map
        let topics = Arc::new(RwLock::new(HashMap::new()));

        // 12. Initialize coordinator client (SPEC2.md §2, §8, §9)
        // Coordinator gets a dedicated transport to avoid competing with the main gossip receiver.
        let coordinator_transport = {
            let coord_bind = std::net::SocketAddr::new(bind_ip, 0);
            let coord_config = UdpTransportAdapterConfig::new(coord_bind, vec![]);

            match UdpTransportAdapter::with_config(coord_config, Some(peer_cache.bootstrap_cache()))
                .await
            {
                Ok(adapter) => Arc::new(SelfFilteringTransport::new(Arc::new(adapter))),
                Err(err) => {
                    warn!(
                        "Failed to create dedicated coordinator transport ({}). Falling back to main transport.",
                        err
                    );
                    transport.clone()
                }
            }
        };

        let coordinator_transport: Arc<RwLock<Box<dyn GossipTransport>>> =
            Arc::new(RwLock::new(Box::new(coordinator_transport)));

        let coordinator =
            super::CoordinatorClient::new(peer_id, coordinator_transport, membership.clone());
        let coordinator = Arc::new(coordinator);

        // Rendezvous client shares the same transport stack
        let rdv_transport: Arc<RwLock<Box<dyn GossipTransport>>> =
            Arc::new(RwLock::new(Box::new(transport.clone())));

        let rdv_pubsub_impl = saorsa_gossip_pubsub::PlumtreePubSub::new(
            peer_id,
            transport.clone(),
            signing_key.clone(),
        );
        let rdv_pubsub: Arc<RwLock<Box<dyn PubSub>>> =
            Arc::new(RwLock::new(Box::new(rdv_pubsub_impl)));

        let rendezvous = super::RendezvousClient::new(peer_id, rdv_transport, rdv_pubsub);
        let rendezvous = Arc::new(rendezvous);

        // 13. Initialize Saorsa Sites (SPEC2.md §5 - Rendezvous Protocol)
        // Create SitePublisher with our identity as site_id
        // Use BLAKE3 hash of public key to get 32 bytes
        let pub_key = identity.key_pair().public_key();
        let key_hash = blake3::hash(pub_key);
        let site_id = super::sites::SiteId::new(*key_hash.as_bytes());
        let site_publisher = Arc::new(super::sites::SitePublisher::new(site_id));

        // Create SiteFetcher with transport access via rendezvous
        // 13. Sites protocol uses DEDICATED transport (separate from main gossip)
        // CRITICAL: Main transport is shared by Membership, PubSub, Presence.
        // Sites needs its own transport to avoid conflicts.
        //
        // BOTH SitesListener AND SiteFetcher will share this dedicated Sites transport.
        let (sites_listener, site_fetcher, sites_dispatcher) = {
            let sites_bind = if let Some(main_port) = listen_port {
                let sites_port = main_port + 1;
                std::net::SocketAddr::new(bind_ip, sites_port)
            } else {
                "0.0.0.0:0"
                    .parse()
                    .context("Failed to parse default sites bind address")?
            };

            let sites_transport: Arc<UdpTransportAdapter> = match UdpTransportAdapter::with_config(
                UdpTransportAdapterConfig::new(sites_bind, vec![]),
                None,
            )
            .await
            {
                Ok(transport) => Arc::new(transport),
                Err(err) => {
                    warn!(
                        "Failed to create dedicated sites transport ({}). Falling back to main transport.",
                        err
                    );
                    transport.inner().clone()
                }
            };

            // Convert to SharedTransport for clean sharing
            let sites_shared: super::transport_types::SharedTransport = sites_transport.clone();

            // Create SitesListener with Sites transport
            let listener = Arc::new(super::sites_listener::SitesListener::new(
                sites_shared.clone(),
                Some(site_publisher.clone()),
            ));

            // Create SiteFetcher with SAME Sites transport
            let mut fetcher = super::sites::SiteFetcher::new_with_shared_transport(
                rendezvous.clone(),
                sites_shared.clone(),
            );

            // Create SitesDispatcher to coordinate listener and fetcher
            let dispatcher = Arc::new(super::sites_dispatcher::SitesDispatcher::new(
                sites_transport.clone(),
                listener.clone(),
            ));

            // Wire fetcher to use dispatcher for response routing
            fetcher.set_dispatcher(dispatcher.clone());
            let fetcher = Arc::new(fetcher);

            // Start dispatcher's receive loop (single loop, no race conditions)
            let handle = dispatcher.clone().start();

            tracing::info!("Sites dispatcher started with listener and fetcher");

            ((listener, handle), fetcher, dispatcher)
        };

        // 14. Initialize connectivity watchdog (Phase 2 TDD - MESH_CAPABILITIES.md §3.2)
        let watchdog_config = WatchdogConfig::default();
        let watchdog = Arc::new(ConnectivityWatchdog::new(watchdog_config));

        // 15. Initialize resource limits (Phase 2 TDD - MESH_CAPABILITIES.md §8.3)
        let resource_limits = Arc::new(ResourceLimits::default());

        Ok(Self {
            identity,
            four_words,
            display_name,
            device_name,
            membership,
            groups,
            groups_by_topic,
            topics,
            entity_types: Arc::new(RwLock::new(HashMap::new())),
            pending_sync_entities: Arc::new(RwLock::new(HashSet::new())),
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
            sites_dispatcher: Some(sites_dispatcher),
            sites_listener: Some(sites_listener.0),
            sites_listener_handle: Some(sites_listener.1),
            name_registry: Some(Arc::new(super::name_record::NameRegistry::new())),
            peer_id,
            watchdog,
            resource_limits,
            contact_store: ContactStore::new(),
            entity_message_handler: Arc::new(RwLock::new(None)),
            entity_receiver_tasks: Arc::new(RwLock::new(HashMap::new())),
            listen_port,
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

    /// Get the listen port (if explicitly set during initialization)
    pub fn listen_port(&self) -> Option<u16> {
        self.listen_port
    }

    /// Shutdown background networking tasks (Sites dispatcher/listener, etc.)
    pub fn shutdown(&self) {
        if let Some(dispatcher) = &self.sites_dispatcher {
            dispatcher.stop();
        }
        if let Some(listener) = &self.sites_listener {
            listener.stop();
        }
        if let Some(handle) = &self.sites_listener_handle {
            handle.abort();
        }
    }

    /// Get connection words for this node's listen address
    ///
    /// Returns the four-word encoding of this node's local listen address,
    /// suitable for sharing with peers so they can connect to us.
    ///
    /// # Returns
    /// Four-word connection identity string (e.g., "apple bear coral dawn" for IPv4)
    ///
    /// # Errors
    /// Returns error if:
    /// - No listen port was configured (port was 0/OS-assigned)
    /// - Failed to determine local IP address
    /// - Four-word encoding failed
    ///
    /// # Example
    /// ```ignore
    /// let ctx = GossipContext::initialize(..., Some(11000)).await?;
    /// let words = ctx.get_my_connection_words()?;
    /// println!("Share this to connect: {}", words);
    /// ```
    pub fn get_my_connection_words(&self) -> crate::IdentityResult<String> {
        // Get listen port (must have been explicitly set)
        let port = self.listen_port.ok_or_else(|| {
            crate::identity::IdentityError::EncodingFailed(
                "No listen port configured (port was OS-assigned)".to_string(),
            )
        })?;

        // Get local IP address (prefer explicit COMMUNITAS_LOCAL_IP when set)
        let local_ip = match std::env::var("COMMUNITAS_LOCAL_IP") {
            Ok(value) => match value.parse::<std::net::IpAddr>() {
                Ok(ip) if !ip.is_unspecified() => ip,
                _ => local_ip_address::local_ip().map_err(|e| {
                    crate::identity::IdentityError::EncodingFailed(format!(
                        "Failed to get local IP address: {}",
                        e
                    ))
                })?,
            },
            Err(_) => local_ip_address::local_ip().map_err(|e| {
                crate::identity::IdentityError::EncodingFailed(format!(
                    "Failed to get local IP address: {}",
                    e
                ))
            })?,
        };

        // Construct SocketAddr
        let addr = std::net::SocketAddr::new(local_ip, port);

        // Encode to four words
        crate::conn_words(&addr)
    }

    /// Enforce configured resource limits based on current usage.
    pub async fn enforce_resource_limits(&self) -> Result<(), crate::ResourceLimitError> {
        let peer_connections = self.transport.connected_peers().await.len();
        let usage = self
            .resource_limits
            .measure_usage_with_peers(peer_connections);
        self.resource_limits.check_all(&usage)
    }

    /// Get ML-DSA-65 keypair for Sites protocol (signing manifests and name records)
    ///
    /// Converts from saorsa_gossip_identity types to saorsa_pqc::ml_dsa_65 types
    /// required by SiteManifest.sign() and NameRecord.sign().
    ///
    /// # Returns
    /// (PublicKey, PrivateKey) compatible with Sites signing operations
    pub fn get_sites_signing_keys(
        &self,
    ) -> Result<(
        saorsa_pqc::ml_dsa_65::PublicKey,
        saorsa_pqc::ml_dsa_65::PrivateKey,
    )> {
        use saorsa_pqc::dsa_traits::SerDes;

        let kp = self.identity.key_pair();

        // Get public key bytes (ML-DSA-65 public key is 1952 bytes)
        let pub_bytes = kp.public_key();
        let pk_array: [u8; 1952] = pub_bytes.try_into().map_err(|_| {
            anyhow::anyhow!(
                "Public key should be 1952 bytes for ML-DSA-65, got {}",
                pub_bytes.len()
            )
        })?;
        let public_key = saorsa_pqc::ml_dsa_65::PublicKey::try_from_bytes(pk_array)
            .map_err(|e| anyhow::anyhow!("Failed to parse public key: {}", e))?;

        // Get secret key using typed method
        let secret_key_typed = kp
            .get_secret_key_typed()
            .map_err(|e| anyhow::anyhow!("Failed to get typed secret key: {}", e))?;

        // The secret_key_typed is saorsa_pqc::pqc::types::MlDsaSecretKey
        // We need saorsa_pqc::ml_dsa_65::PrivateKey
        // Get bytes and convert (ML-DSA-65 secret key is 4032 bytes)
        let sk_bytes = secret_key_typed.as_bytes();
        let sk_array: [u8; 4032] = sk_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Secret key should be 4032 bytes for ML-DSA-65"))?;
        let private_key = saorsa_pqc::ml_dsa_65::PrivateKey::try_from_bytes(sk_array)
            .map_err(|e| anyhow::anyhow!("Failed to parse private key: {}", e))?;

        Ok((public_key, private_key))
    }

    /// Get the raw identity MlDsaKeyPair for ProviderSummary signing
    pub fn get_identity_keypair(&self) -> &saorsa_gossip_identity::MlDsaKeyPair {
        self.identity.key_pair()
    }

    /// Get the identity reference for direct access
    pub fn identity(&self) -> &Identity {
        &self.identity
    }

    /// Check if WAN (wide-area network) operations should be attempted
    ///
    /// Returns false when in local-only mode (bootstrap nodes unreachable)
    /// to prevent wasting resources on doomed WAN dial attempts.
    ///
    /// Phase 3 TDD: Respects connectivity watchdog state (MESH_CAPABILITIES.md §3.2)
    pub fn should_attempt_wan_operations(&self) -> bool {
        !self.watchdog.is_local_only_mode()
    }

    /// Check if system is currently in local-only mode
    pub fn is_local_only_mode(&self) -> bool {
        self.watchdog.is_local_only_mode()
    }

    /// Override local-only mode (used for test harnesses and manual control)
    pub fn set_local_only_mode(&self, enabled: bool) {
        if enabled {
            self.watchdog.force_local_only();
        } else {
            self.watchdog.force_online();
        }
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

    /// Remove a favourite contact
    pub async fn remove_favourite_contact(&self, four_words: &str) -> Result<()> {
        let mut favourites = self.favourite_contacts.write().await;
        favourites.retain(|fw| fw != four_words);
        Ok(())
    }

    /// Get list of favourite contacts
    pub async fn get_favourite_contacts(&self) -> Vec<String> {
        self.favourite_contacts.read().await.clone()
    }

    /// Set the handler for incoming entity messages
    ///
    /// The handler is called with (entity_id, sender_peer_id, message_bytes)
    /// whenever a message is received on a subscribed entity topic.
    pub async fn set_entity_message_handler(&self, handler: EntityMessageHandler) {
        let mut guard = self.entity_message_handler.write().await;
        *guard = Some(handler);
        info!("Entity message handler registered");
    }

    pub(super) fn entity_message_handler(&self) -> Arc<RwLock<Option<EntityMessageHandler>>> {
        Arc::clone(&self.entity_message_handler)
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
        let topic_id = TopicId::from_entity(entity_id);

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
        let log_transport = matches!(
            std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        // Lock hierarchy: Holding multiple read locks simultaneously is safe (no deadlock risk).
        // topics (#4) and entity_receiver_tasks (#7) - correct order maintained.
        {
            let topics = self.topics.read().await;
            let tasks = self.entity_receiver_tasks.read().await;
            if topics.contains_key(entity_id) && tasks.contains_key(entity_id) {
                if log_transport {
                    info!(
                        "Join entity {} ({}): already joined, skipping rejoin",
                        entity_id, entity_type
                    );
                }
                return Ok(());
            }
        }
        {
            let mut entity_types = self.entity_types.write().await;
            entity_types.insert(entity_id.to_string(), entity_type.to_string());
        }
        // 1. Get or create topic ID
        let topic_id = self.map_entity_to_topic(entity_id, entity_type).await?;
        if log_transport {
            info!(
                "Join entity {} ({}): topic {:?}",
                entity_id, entity_type, topic_id
            );
        }

        // 2. Create MLS group context (simplified for now)
        let group_ctx = GroupContext::from_entity(entity_id);
        {
            let mut groups = self.groups.write().await;
            groups.insert(entity_id.to_string(), group_ctx.clone());
        }
        {
            let mut groups_by_topic = self.groups_by_topic.write().await;
            groups_by_topic.insert(topic_id, group_ctx);
        }

        // 3. Subscribe to topic and spawn receiver task
        let mut rx = {
            let pubsub = self.pubsub.read().await;
            pubsub.subscribe(topic_id)
        };

        // 3.5. Initialize topic peers from connected transport peers
        // This populates the eager_peers set so messages are actually broadcast
        let connected_peers: Vec<_> = self
            .transport
            .connected_peers()
            .await
            .into_iter()
            .map(|(peer_id, _addr)| peer_id)
            .filter(|peer_id| *peer_id != self.peer_id)
            .collect();

        if log_transport {
            info!(
                "Join entity {}: initializing {} connected peers",
                entity_id,
                connected_peers.len()
            );
        }

        if !connected_peers.is_empty() {
            // Initialize topic peers (read lock released after this block)
            {
                let pubsub = self.pubsub.read().await;
                (**pubsub)
                    .initialize_topic_peers(topic_id, connected_peers.clone())
                    .await;
            } // Read lock released here

            debug!(
                "Initialized topic {:?} with {} connected peers",
                topic_id,
                connected_peers.len()
            );

            let sync_sent = self.send_sync_requests(entity_id, entity_type).await;
            let mut pending = self.pending_sync_entities.write().await;
            if sync_sent {
                pending.remove(entity_id);
            } else {
                pending.insert(entity_id.to_string());
            }
        } else {
            debug!(
                "No connected peers yet for topic {:?}, messages will buffer",
                topic_id
            );
            let mut pending = self.pending_sync_entities.write().await;
            pending.insert(entity_id.to_string());
        }

        // 4. Spawn background task to process incoming messages
        let entity_id_clone = entity_id.to_string();
        let handler_ref = self.entity_message_handler.clone();

        let handle = tokio::spawn(async move {
            debug!(
                "Started entity message receiver task for {}",
                entity_id_clone
            );

            while let Some((sender_peer_id, message_bytes)) = rx.recv().await {
                debug!(
                    "Received message for entity {} from peer {:?}",
                    entity_id_clone, sender_peer_id
                );

                // Get handler and invoke if set
                if let Some(handler) = handler_ref.read().await.as_ref() {
                    handler(entity_id_clone.clone(), sender_peer_id, message_bytes);
                } else {
                    warn!(
                        "No entity message handler set, dropping message for {}",
                        entity_id_clone
                    );
                }
            }

            debug!("Entity message receiver task ended for {}", entity_id_clone);
        });

        // Store the task handle
        {
            let mut tasks = self.entity_receiver_tasks.write().await;
            tasks.insert(entity_id.to_string(), handle);
        }

        info!("Joined {} {}, subscribed to topic", entity_type, entity_id);
        Ok(())
    }

    async fn send_sync_requests(&self, entity_id: &str, entity_type: &str) -> bool {
        let sync_request = crate::crdt::SyncRequest {
            entity_id: entity_id.to_string(),
            entity_type: Self::parse_entity_type(entity_type),
            requester_peer_id: self.four_words.clone(),
            vector_clock: crate::crdt::VectorClock::new(), // Empty = request all messages
            missing_message_ids: None,
            request_id: chrono::Utc::now().timestamp_millis() as u64,
        };

        let gossip_msg = crate::crdt::GossipMessageType::SyncRequest(sync_request);
        let mut sync_sent = false;
        match serde_json::to_vec(&gossip_msg) {
            Ok(bytes) => {
                let publish_result = tokio::time::timeout(
                    Self::join_timeout(),
                    self.publish_to_entity(entity_id, bytes.clone()),
                )
                .await;
                let direct_result = tokio::time::timeout(
                    Self::join_timeout(),
                    self.direct_publish_membership_message(entity_id, bytes),
                )
                .await;

                match publish_result {
                    Ok(Ok(())) => {
                        sync_sent = true;
                    }
                    Ok(Err(e)) => {
                        warn!("Failed to send sync request for {}: {}", entity_id, e);
                    }
                    Err(_) => {
                        warn!("Timed out sending sync request for {}", entity_id);
                    }
                }

                match direct_result {
                    Ok(Ok(())) => {
                        sync_sent = true;
                    }
                    Ok(Err(e)) => {
                        warn!("Direct sync request failed for {}: {}", entity_id, e);
                    }
                    Err(_) => {
                        warn!("Timed out direct sync request for {}", entity_id);
                    }
                }

                if sync_sent {
                    info!("Sent sync request for entity {}", entity_id);
                }
            }
            Err(e) => {
                warn!("Failed to serialize sync request: {}", e);
            }
        }

        // Request membership snapshot to hydrate roles on join.
        match tokio::time::timeout(
            Self::join_timeout(),
            self.request_member_sync(entity_id, entity_type),
        )
        .await
        {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                warn!("Failed to request member sync for {}: {}", entity_id, e);
            }
            Err(_) => {
                warn!("Timed out requesting member sync for {}", entity_id);
            }
        }

        sync_sent
    }

    pub async fn request_pending_entity_syncs(&self) {
        let pending: Vec<String> = {
            let pending_guard = self.pending_sync_entities.read().await;
            pending_guard.iter().cloned().collect()
        };

        if pending.is_empty() {
            return;
        }

        let entity_types = { self.entity_types.read().await.clone() };
        let mut completed = Vec::new();

        for entity_id in pending {
            match entity_types.get(&entity_id) {
                Some(entity_type) => {
                    if self.send_sync_requests(&entity_id, entity_type).await {
                        completed.push(entity_id);
                    }
                }
                None => {
                    warn!(
                        "Missing entity type for pending sync on {}, clearing pending flag",
                        entity_id
                    );
                    completed.push(entity_id);
                }
            }
        }

        if !completed.is_empty() {
            let mut pending_guard = self.pending_sync_entities.write().await;
            for entity_id in completed {
                pending_guard.remove(&entity_id);
            }
        }
    }

    pub async fn request_entity_syncs_for_all(&self) {
        let entity_types = { self.entity_types.read().await.clone() };
        if entity_types.is_empty() {
            return;
        }

        let mut completed = Vec::new();
        for (entity_id, entity_type) in &entity_types {
            if self.send_sync_requests(entity_id, entity_type).await {
                completed.push(entity_id.clone());
            }
        }

        if !completed.is_empty() {
            let mut pending_guard = self.pending_sync_entities.write().await;
            for entity_id in completed {
                pending_guard.remove(&entity_id);
            }
        }
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
        // Lock hierarchy: pubsub is highest priority, must be dropped before acquiring other locks
        {
            let pubsub = self.pubsub.write().await;
            pubsub.unsubscribe(topic_id).await?;
        }
        // pubsub lock dropped here - safe to acquire lower-priority locks

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
        {
            let mut entity_types = self.entity_types.write().await;
            entity_types.remove(entity_id);
        }
        {
            let mut pending = self.pending_sync_entities.write().await;
            pending.remove(entity_id);
        }

        info!("Left entity {}, unsubscribed from topic", entity_id);
        Ok(())
    }

    /// Publish a message to an entity's topic
    pub async fn publish_to_entity(&self, entity_id: &str, message: Vec<u8>) -> Result<()> {
        let log_transport = matches!(
            std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        // 1. Get topic ID
        let topic_id = {
            let topics = self.topics.read().await;
            topics
                .get(entity_id)
                .copied()
                .context("Entity not found, must join first")?
        };

        // 1.5. Refresh eager peers for this topic from current transport connections.
        // This reduces "first message lost" scenarios when peers connect after initial join.
        let connected_peers: Vec<_> = self
            .transport
            .connected_peers()
            .await
            .into_iter()
            .filter(|(peer_id, _addr)| *peer_id != self.peer_id)
            .collect();
        let connected_peer_ids_raw: Vec<_> = connected_peers
            .iter()
            .map(|(peer_id, _)| *peer_id)
            .collect();
        let mut connected_peer_ids = self
            .prime_peer_connections(&connected_peers, Duration::from_secs(2))
            .await;
        if connected_peer_ids.is_empty() && !connected_peer_ids_raw.is_empty() {
            connected_peer_ids = connected_peer_ids_raw.clone();
        }

        if log_transport {
            info!(
                "Publish entity {} on topic {:?}: {} connected peers {:?}",
                entity_id,
                topic_id,
                connected_peer_ids.len(),
                connected_peer_ids
            );
        }

        let had_peers = !connected_peer_ids.is_empty();
        if had_peers {
            let pubsub = self.pubsub.read().await;
            (**pubsub)
                .initialize_topic_peers(topic_id, connected_peer_ids)
                .await;
        }

        // 2. Get MLS group for encryption
        // Lock hierarchy: Scope groups lock to drop before acquiring pubsub (#1 > #3)
        {
            let groups = self.groups.read().await;
            let _group_ctx = groups
                .get(entity_id)
                .context("MLS group not found, must join first")?;
        }
        // groups lock dropped here - safe to acquire pubsub

        // Messages are protected by the QUIC transport. MLS group encryption
        // will wrap this payload once the MLS keying layer is integrated.
        let payload = message.clone();

        // 3. Publish via gossip
        let pubsub = self.pubsub.read().await;
        pubsub.publish(topic_id, payload.clone().into()).await?;

        if direct_entity_broadcast_enabled() {
            let transport = self.transport.clone();
            let local_peer = self.peer_id;
            let entity_id = entity_id.to_string();
            let entity_id_for_log = entity_id.clone();
            let payload = message;
            tokio::spawn(async move {
                if let Err(err) = Self::direct_publish_to_peers_inner(
                    transport,
                    local_peer,
                    entity_id,
                    payload,
                    GossipStreamType::Bulk,
                    log_transport,
                )
                .await
                {
                    warn!(
                        "Direct entity broadcast failed for {}: {}",
                        entity_id_for_log, err
                    );
                }
            });
        }

        if !had_peers {
            let transport = self.transport.clone();
            let pubsub = self.pubsub.clone();
            let local_peer = self.peer_id;
            let entity_id = entity_id.to_string();
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(2000)).await;
                let connected_peers: Vec<_> = transport
                    .connected_peers()
                    .await
                    .into_iter()
                    .filter(|(peer_id, _addr)| *peer_id != local_peer)
                    .collect();
                if connected_peers.is_empty() {
                    return;
                }
                let peer_ids: Vec<_> = connected_peers
                    .iter()
                    .map(|(peer_id, _addr)| *peer_id)
                    .collect();
                let pubsub = pubsub.read().await;
                (**pubsub).initialize_topic_peers(topic_id, peer_ids).await;
                if let Err(err) = pubsub.publish(topic_id, payload.into()).await {
                    warn!("Deferred publish for {} failed: {}", entity_id, err);
                }
            });
        }

        debug!("Published message to entity {}", entity_id);
        Ok(())
    }

    /// Request a membership snapshot for an entity.
    pub async fn request_member_sync(&self, entity_id: &str, entity_type: &str) -> Result<()> {
        let request = crate::crdt::MemberSyncRequest {
            entity_id: entity_id.to_string(),
            entity_type: Self::parse_entity_type(entity_type),
            requester_peer_id: self.four_words.clone(),
        };
        let gossip_msg = crate::crdt::GossipMessageType::MemberSyncRequest(request);
        let bytes =
            serde_json::to_vec(&gossip_msg).context("Failed to serialize member sync request")?;
        let direct_enabled = direct_entity_broadcast_enabled();
        let mut direct_ok = false;
        if direct_enabled {
            match tokio::time::timeout(
                Self::join_timeout(),
                self.direct_publish_membership_message(entity_id, bytes.clone()),
            )
            .await
            {
                Ok(Ok(())) => {
                    direct_ok = true;
                }
                Ok(Err(err)) => {
                    warn!(
                        "Direct member sync request failed for {}: {}",
                        entity_id, err
                    );
                }
                Err(_) => {
                    warn!("Direct member sync request timed out for {}", entity_id);
                }
            }
        }

        match self.publish_member_sync_request(entity_id, bytes).await {
            Ok(()) => Ok(()),
            Err(err) => {
                if direct_ok {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }

    async fn publish_member_sync_request(&self, entity_id: &str, message: Vec<u8>) -> Result<()> {
        let topic_id = {
            let topics = self.topics.read().await;
            topics
                .get(entity_id)
                .copied()
                .context("Entity not found, must join first")?
        };

        // Lock hierarchy: Scope groups lock to drop before acquiring pubsub (#1 > #3)
        {
            let groups = self.groups.read().await;
            let _group_ctx = groups
                .get(entity_id)
                .context("MLS group not found, must join first")?;
        }
        // groups lock dropped here - safe to acquire pubsub

        let pubsub = self.pubsub.read().await;
        pubsub.publish(topic_id, message.into()).await?;
        Ok(())
    }

    async fn direct_publish_to_peers(
        &self,
        entity_id: &str,
        payload: Vec<u8>,
        stream_type: GossipStreamType,
    ) -> Result<()> {
        let log_transport = matches!(
            std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        Self::direct_publish_to_peers_inner(
            self.transport.clone(),
            self.peer_id,
            entity_id.to_string(),
            payload,
            stream_type,
            log_transport,
        )
        .await
    }

    async fn direct_publish_to_peers_inner(
        transport: Arc<SelfFilteringTransport<UdpTransportAdapter>>,
        local_peer: PeerId,
        entity_id: String,
        payload: Vec<u8>,
        stream_type: GossipStreamType,
        log_transport: bool,
    ) -> Result<()> {
        let peers: Vec<(PeerId, std::net::SocketAddr)> = transport
            .connected_peers()
            .await
            .into_iter()
            .filter(|(peer_id, _addr)| *peer_id != local_peer)
            .collect();
        if peers.is_empty() {
            return Ok(());
        }

        let resolved_peers: HashSet<PeerId> = peers.iter().map(|(peer_id, _)| *peer_id).collect();

        if resolved_peers.is_empty() {
            return Ok(());
        }

        info!(
            "Direct entity broadcast for {} to {} peers",
            entity_id,
            resolved_peers.len()
        );

        let envelope = DirectEntityEnvelope {
            entity_id: entity_id.clone(),
            payload,
        };
        let bytes =
            serde_json::to_vec(&envelope).context("Failed to serialize direct entity envelope")?;

        let mut send_set = tokio::task::JoinSet::new();
        for peer_id in resolved_peers {
            let transport = transport.clone();
            let entity_id = entity_id.clone();
            let bytes = bytes.clone();
            send_set.spawn(async move {
                if log_transport {
                    info!(
                        "Direct entity broadcast for {} to peer {:?} ({} bytes)",
                        entity_id,
                        peer_id,
                        bytes.len()
                    );
                }
                match transport
                    .send_to_peer(peer_id, stream_type, Bytes::from(bytes))
                    .await
                {
                    Ok(()) => {}
                    Err(err) => {
                        warn!(
                            "Direct entity broadcast for {} to peer {:?} failed: {}",
                            entity_id, peer_id, err
                        );
                    }
                }
            });
        }
        while let Some(result) = send_set.join_next().await {
            if let Err(err) = result {
                warn!("Direct entity broadcast task join failed: {err}");
            }
        }

        Ok(())
    }

    pub async fn direct_publish_entity_to_peer(
        &self,
        entity_id: &str,
        peer_id: PeerId,
        payload: Vec<u8>,
        stream_type: GossipStreamType,
    ) -> Result<()> {
        let log_transport = matches!(
            std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        let envelope = DirectEntityEnvelope {
            entity_id: entity_id.to_string(),
            payload,
        };
        let bytes =
            serde_json::to_vec(&envelope).context("Failed to serialize direct entity envelope")?;
        if log_transport {
            info!(
                "Direct entity send for {} to peer {:?} ({} bytes)",
                entity_id,
                peer_id,
                bytes.len()
            );
        }
        self.transport
            .send_to_peer(peer_id, stream_type, Bytes::from(bytes))
            .await?;
        Ok(())
    }

    async fn prime_peer_connections(
        &self,
        peers: &[(PeerId, std::net::SocketAddr)],
        timeout: Duration,
    ) -> Vec<PeerId> {
        if peers.is_empty() {
            return Vec::new();
        }

        let mut dial_set = tokio::task::JoinSet::new();
        for (_peer_id, addr) in peers {
            let transport = self.transport.clone();
            let addr = *addr;
            dial_set.spawn(async move {
                match tokio::time::timeout(timeout, TransportAdapter::dial(&transport, addr)).await
                {
                    Ok(Ok(peer_id)) => Some(peer_id),
                    Ok(Err(err)) => {
                        warn!("Dial to {} failed while priming: {}", addr, err);
                        None
                    }
                    Err(_) => {
                        warn!("Dial to {} timed out while priming", addr);
                        None
                    }
                }
            });
        }

        let mut resolved = HashSet::new();
        while let Some(result) = dial_set.join_next().await {
            match result {
                Ok(Some(peer_id)) => {
                    resolved.insert(peer_id);
                }
                Ok(None) => {}
                Err(err) => {
                    warn!("Dial task join failed: {err}");
                }
            }
        }

        resolved.into_iter().collect()
    }

    /// Fallback path for membership messages when PubSub delivery is unreliable.
    pub async fn direct_publish_membership_message(
        &self,
        entity_id: &str,
        payload: Vec<u8>,
    ) -> Result<()> {
        let mut last_err: Option<anyhow::Error> = None;
        let mut success = false;

        for stream_type in [GossipStreamType::Membership, GossipStreamType::Bulk] {
            match self
                .direct_publish_to_peers(entity_id, payload.clone(), stream_type)
                .await
            {
                Ok(()) => success = true,
                Err(err) => last_err = Some(err),
            }
        }

        if success {
            Ok(())
        } else {
            Err(last_err.unwrap_or_else(|| anyhow::anyhow!("direct publish failed")))
        }
    }

    /// Fallback path for member updates when PubSub delivery is unreliable.
    pub async fn direct_publish_member_update(
        &self,
        entity_id: &str,
        payload: Vec<u8>,
    ) -> Result<()> {
        self.direct_publish_membership_message(entity_id, payload)
            .await
    }

    /// Send a membership message directly to a specific peer.
    pub async fn direct_send_membership_to_peer(
        &self,
        entity_id: &str,
        peer_id: PeerId,
        payload: Vec<u8>,
    ) -> Result<()> {
        let envelope = DirectEntityEnvelope {
            entity_id: entity_id.to_string(),
            payload,
        };
        let bytes =
            serde_json::to_vec(&envelope).context("Failed to serialize direct entity envelope")?;
        let mut last_err: Option<anyhow::Error> = None;
        let mut success = false;

        for stream_type in [GossipStreamType::Membership, GossipStreamType::Bulk] {
            match self
                .transport
                .send_to_peer(peer_id, stream_type, Bytes::from(bytes.clone()))
                .await
            {
                Ok(()) => success = true,
                Err(err) => last_err = Some(err),
            }
        }

        if success {
            Ok(())
        } else {
            Err(last_err.unwrap_or_else(|| anyhow::anyhow!("direct send failed")))
        }
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

    /// Find a contact by four-word address and return address hints.
    pub async fn find_contact_with_hints(
        &self,
        four_words: &str,
    ) -> Result<crate::gossip::discovery::ContactDiscoveryResult> {
        let mut result = self
            .discovery
            .find_contact_with_hints(four_words)
            .await
            .context("Contact not found via FOAF or presence")?;

        if result.addr_hints.is_empty()
            && let Some(addr) = self.get_contact_endpoint(four_words).await
        {
            result.addr_hints.push(addr.to_string());
        }

        Ok(result)
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
    // Contact Linking API - Local-Only / Network-Linked
    // ========================================================================

    /// Create a local-only contact (not linked to network identity)
    pub async fn create_local_contact(
        &self,
        display_name: String,
    ) -> Result<crate::gossip::contact_storage::ContactRecord> {
        let contact = crate::gossip::contact_storage::ContactRecord::new_local(display_name);
        self.contact_store
            .add(contact.clone())
            .await
            .context("Failed to add local contact")?;
        Ok(contact)
    }

    /// Link a local-only contact to a network identity
    pub async fn link_contact(
        &self,
        contact_id: &str,
        four_words: &str,
    ) -> Result<crate::gossip::contact_storage::ContactRecord> {
        let contact = self
            .contact_store
            .link_contact(contact_id, four_words)
            .await
            .context("Failed to link contact")?;
        Ok(contact)
    }

    /// Get all local-only contacts
    pub async fn get_local_only_contacts(
        &self,
    ) -> Vec<crate::gossip::contact_storage::ContactRecord> {
        self.contact_store.local_only().await
    }

    /// Get all network-linked contacts
    pub async fn get_linked_contacts(&self) -> Vec<crate::gossip::contact_storage::ContactRecord> {
        self.contact_store.network_linked().await
    }

    // ========================================================================
    // Messaging API - Plumtree pub/sub
    // ========================================================================

    /// Send a message to a specific peer
    ///
    /// Uses QUIC transport directly for point-to-point messaging
    pub async fn send_direct_message(&self, peer_id: PeerId, message: Vec<u8>) -> Result<()> {
        self.transport
            .send_to_peer(peer_id, GossipStreamType::Bulk, Bytes::from(message))
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
        let plaintext = postcard::to_stdvec(&messages).context("Failed to serialize state")?;

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
                        .send_to_peer(peer_id, GossipStreamType::Bulk, Bytes::from(package))
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
            postcard::from_bytes(&plaintext).context("Failed to deserialize recovered state")?;

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
        // Lock hierarchy: Acquire groups_by_topic (#3) before presence (#8)
        let topic_ids: Vec<TopicId> = {
            let groups_by_topic = self.groups_by_topic.read().await;
            groups_by_topic.keys().copied().collect()
        };
        // groups_by_topic lock dropped here

        let presence = self.presence.read().await;
        for topic_id in topic_ids {
            let online_peers = presence.get_online_peers(topic_id).await;
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

    /// Add a peer to the eager_peers set for an entity's topic
    ///
    /// This ensures that when we publish messages to this entity,
    /// the peer will receive them. Called when we receive a sync request
    /// from a peer, indicating they're interested in this entity's messages.
    pub async fn add_peer_to_entity_topic(&self, entity_id: &str, peer_id: PeerId) -> Result<()> {
        if peer_id == self.peer_id {
            debug!(
                "Skipping local peer when adding to topic for entity {}",
                entity_id
            );
            return Ok(());
        }

        // Get topic ID for entity
        let topic_id = {
            let topics = self.topics.read().await;
            match topics.get(entity_id) {
                Some(id) => *id,
                None => {
                    debug!(
                        "Entity {} not found when adding peer to topic, skipping",
                        entity_id
                    );
                    return Ok(());
                }
            }
        };

        // Add peer to eager_peers via initialize_topic_peers
        let pubsub = self.pubsub.read().await;
        (**pubsub)
            .initialize_topic_peers(topic_id, vec![peer_id])
            .await;

        debug!(
            "Added peer {:?} to eager_peers for entity {}",
            peer_id, entity_id
        );
        Ok(())
    }

    /// Establish peer routing for Sites transport
    ///
    /// The Sites protocol uses a dedicated transport that needs its own peer routing.
    /// This method primes the Sites transport's peer routing table by sending a dummy message.
    pub async fn establish_sites_peer_routing(&self, peer_id: PeerId) -> Result<()> {
        if let Some(ref listener) = self.sites_listener {
            // Send empty ping to establish route in Sites transport
            listener
                .transport()
                .send_to_peer(
                    peer_id,
                    saorsa_gossip_transport::GossipStreamType::Bulk,
                    bytes::Bytes::from_static(b""),
                )
                .await
                .context("Failed to establish Sites peer routing")?;
            debug!("Established Sites peer routing to {:?}", peer_id);
        } else {
            warn!("Sites listener not available - cannot establish peer routing");
        }
        Ok(())
    }

    /// Establish a connection to a specific address (bootstrap)
    pub async fn dial_address(&self, addr: std::net::SocketAddr) -> Result<()> {
        use saorsa_gossip_transport::GossipTransport;

        if let Err(err) = self.enforce_resource_limits().await {
            warn!(
                "Resource limits prevent dialing bootstrap {}: {}",
                addr, err
            );
            return Err(anyhow::anyhow!(err));
        }

        // Use transport layer to establish actual QUIC connection
        info!("Dialing bootstrap node at {} via transport", addr);
        match self.transport.dial_bootstrap(addr).await {
            Ok(peer_id) => {
                info!(
                    "Successfully connected to bootstrap {} (peer_id: {:?})",
                    addr, peer_id
                );

                // Register peer for anti-entropy sync
                self.anti_entropy.add_peer(peer_id).await;

                // Also notify membership layer about this peer
                let seed = addr.to_string();
                let membership = self.membership.read().await;
                if let Err(e) = membership.join(vec![seed]).await {
                    warn!("Membership join after dial failed (non-fatal): {}", e);
                }

                // Ensure the new peer is added to all existing entity topics
                let entity_ids: Vec<String> = {
                    let topics = self.topics.read().await;
                    topics.keys().cloned().collect()
                };
                for entity_id in entity_ids {
                    if let Err(e) = self.add_peer_to_entity_topic(&entity_id, peer_id).await {
                        warn!(
                            "Failed to add peer {:?} to topic for {}: {}",
                            peer_id, entity_id, e
                        );
                    }
                }

                Ok(())
            }
            Err(e) => {
                warn!("Failed to dial bootstrap {}: {}", addr, e);
                Err(anyhow::anyhow!("Failed to dial {}: {}", addr, e))
            }
        }
    }

    // ========================================================================
    // Contact Store API - Endpoint tracking for reconnection
    // ========================================================================

    /// Publish a direct message to a recipient's DM topic
    ///
    /// Creates a topic based on the recipient's four-word address and
    /// publishes the message via the gossip pubsub layer.
    pub async fn publish_dm(&self, recipient_four_words: &str, message: Vec<u8>) -> Result<()> {
        // Create a DM topic ID from the recipient's four-word address
        // DM topics use format: "dm:{four_words}"
        let dm_topic_str = format!("dm:{}", recipient_four_words);
        let topic_id = TopicId::from_entity(&dm_topic_str);

        // Publish via pubsub
        let pubsub = self.pubsub.read().await;
        pubsub.publish(topic_id, message.into()).await?;

        debug!("Published DM to {}", recipient_four_words);
        Ok(())
    }

    /// Get all contact records from the contact store
    pub async fn get_all_contact_records(&self) -> Vec<ContactRecord> {
        self.contact_store.all().await
    }

    /// Get the valid endpoint for a contact
    ///
    /// Returns the endpoint if it exists and is valid (not stale, not too many failures)
    pub async fn get_contact_endpoint(&self, four_words: &str) -> Option<std::net::SocketAddr> {
        let contact = self.contact_store.get(four_words).await?;
        contact.get_valid_endpoint()
    }

    /// Update the endpoint for a contact
    ///
    /// Creates the contact if it doesn't exist.
    pub async fn update_contact_endpoint(
        &self,
        four_words: &str,
        addr: &std::net::SocketAddr,
    ) -> Result<()> {
        // Check if contact exists
        if self.contact_store.exists(four_words).await {
            self.contact_store
                .update_endpoint(four_words, addr)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to update endpoint: {}", e))?;
        } else {
            // Create new contact with endpoint
            let mut contact = ContactRecord::new(four_words.to_string());
            contact
                .update_endpoint(addr)
                .map_err(|e| anyhow::anyhow!("Failed to set endpoint: {}", e))?;
            self.contact_store
                .add(contact)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to add contact: {}", e))?;
        }

        info!("Updated endpoint for contact {} to {}", four_words, addr);
        Ok(())
    }

    /// Record a successful connection to a contact
    ///
    /// Updates the endpoint and resets failure count.
    /// Creates the contact if it doesn't exist.
    pub async fn record_contact_success(
        &self,
        four_words: &str,
        addr: std::net::SocketAddr,
    ) -> Result<()> {
        if self.contact_store.exists(four_words).await {
            self.contact_store
                .record_success(four_words, addr)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to record success: {}", e))?;
        } else {
            // Create new contact with successful connection
            let mut contact = ContactRecord::new(four_words.to_string());
            contact.record_success(addr);
            self.contact_store
                .add(contact)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to add contact: {}", e))?;
        }

        debug!(
            "Recorded successful connection to {} at {}",
            four_words, addr
        );
        Ok(())
    }

    /// Record a connection failure to a contact
    ///
    /// Increments the failure count. After too many failures, the endpoint
    /// will be skipped in get_contact_endpoint().
    pub async fn record_contact_failure(&self, four_words: &str) -> Result<()> {
        if !self.contact_store.exists(four_words).await {
            return Err(anyhow::anyhow!("Contact not found: {}", four_words));
        }

        self.contact_store
            .record_failure(four_words)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to record failure: {}", e))?;

        debug!("Recorded connection failure to {}", four_words);
        Ok(())
    }

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Parse entity type string into EntityType enum
    fn parse_entity_type(entity_type: &str) -> crate::crdt::EntityType {
        match entity_type.to_lowercase().as_str() {
            "person" | "contact" => crate::crdt::EntityType::Person,
            "group" => crate::crdt::EntityType::Group,
            "project" => crate::crdt::EntityType::Project,
            "channel" => crate::crdt::EntityType::Channel,
            "organisation" | "organization" | "org" => crate::crdt::EntityType::Organisation,
            _ => {
                warn!(
                    "Unknown entity type '{}', defaulting to Channel",
                    entity_type
                );
                crate::crdt::EntityType::Channel
            }
        }
    }

    // ========================================================================
    // Coordinator Mode (for Bootstrap/Seed Nodes)
    // ========================================================================

    /// Start coordinator mode for this node
    ///
    /// This enables the node to act as a bootstrap/coordinator node that:
    /// - Helps new peers join the network (bootstrap role)
    /// - Provides address reflection for NAT detection (reflector role)
    /// - Coordinates peer introductions for hole punching (rendezvous role)
    /// - Forwards messages for peers behind symmetric NATs (relay role)
    ///
    /// Call this after `initialize()` for headless/bootstrap nodes.
    ///
    /// # Arguments
    /// * `external_addrs` - List of external addresses this node is reachable at
    /// * `publish_interval_secs` - How often to publish coordinator adverts (default: 300)
    ///
    /// # Returns
    /// A JoinHandle for the background publishing task
    pub async fn start_coordinator_mode(
        &self,
        external_addrs: Vec<std::net::SocketAddr>,
        publish_interval_secs: Option<u64>,
    ) -> Result<JoinHandle<()>> {
        let interval = publish_interval_secs.unwrap_or(300); // 5 minutes default

        // Get our peer ID
        let peer_id = self.peer_id;

        // Configure all coordinator roles (full bootstrap node)
        let roles = CoordinatorRoles {
            coordinator: true, // Help new peers join
            reflector: true,   // Provide address observation
            rendezvous: true,  // Coordinate peer introductions
            relay: true,       // Forward messages for symmetric NATs
        };

        // Create the coordinator publisher
        let publisher = CoordinatorPublisher::new(
            peer_id,
            roles,
            external_addrs.clone(),
            NatClass::Eim, // Bootstrap nodes typically have public IPs (Endpoint-Independent Mapping)
        );

        // Get the signing key from identity
        let signing_key = self
            .identity
            .key_pair()
            .get_secret_key_typed()
            .map_err(|e| anyhow::anyhow!("Failed to get signing key: {}", e))?;
        publisher.set_signing_key(signing_key).await;

        info!(
            "Starting coordinator mode with {} external addresses, publishing every {}s",
            external_addrs.len(),
            interval
        );
        info!("Coordinator roles: bootstrap, reflector, rendezvous, relay");

        // Create periodic publisher
        let periodic = PeriodicPublisher::new(publisher, interval);
        let mut advert_rx = periodic.start().await;

        // Get pubsub for publishing
        let pubsub = self.pubsub.clone();
        let topic = coordinator_topic();

        // Spawn background task to publish adverts
        let handle = tokio::spawn(async move {
            while let Some(advert_bytes) = advert_rx.recv().await {
                // Publish to coordinator topic
                let pubsub_guard = pubsub.read().await;
                if let Err(e) = pubsub_guard.publish(topic, advert_bytes).await {
                    warn!("Failed to publish coordinator advert: {}", e);
                } else {
                    debug!("Published coordinator advert to gossip overlay");
                }
            }
            info!("Coordinator publisher stopped");
        });

        info!("Coordinator mode started successfully");
        Ok(handle)
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
            None,
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
            None,
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
            None,
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

    #[tokio::test]
    async fn test_get_my_connection_words_with_port() {
        // Initialize with explicit port
        let ctx = GossipContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            "Desktop".to_string(),
            Some(11000),
        )
        .await
        .expect("init");

        // Should have listen_port stored
        assert_eq!(ctx.listen_port(), Some(11000));

        // get_my_connection_words should succeed
        let words = ctx.get_my_connection_words();
        assert!(words.is_ok(), "get_my_connection_words failed: {:?}", words);

        let words = words.unwrap();
        // Should contain spaces (four-word-networking uses spaces for IPs)
        assert!(
            words.contains(' '),
            "Connection words should contain spaces"
        );

        // Words should be decodable back to an address
        let decoded = crate::conn_from_words(&words);
        assert!(decoded.is_ok(), "Should decode back to address");
        assert_eq!(decoded.unwrap().port(), 11000);
    }

    #[tokio::test]
    async fn test_get_my_connection_words_without_port() {
        // Initialize without explicit port (None = OS-assigned)
        let ctx = GossipContext::initialize(
            "river-mountain-cloud-light".to_string(),
            "Bob".to_string(),
            "Laptop".to_string(),
            None,
        )
        .await
        .expect("init");

        // Should not have listen_port stored
        assert_eq!(ctx.listen_port(), None);

        // get_my_connection_words should fail (port unknown)
        let words = ctx.get_my_connection_words();
        assert!(
            words.is_err(),
            "get_my_connection_words should fail without explicit port"
        );
    }

    #[tokio::test]
    async fn test_connection_words_roundtrip() {
        // Test that connection words can roundtrip through encode/decode
        use std::net::SocketAddr;

        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();
        let words = crate::conn_words(&addr).unwrap();
        let decoded = crate::conn_from_words(&words).unwrap();
        assert_eq!(addr, decoded);
    }
}

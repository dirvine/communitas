// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Core Context - Centralized application state for Communitas (RC1b architecture)
//!
//! This module provides the main application context that coordinates:
//! - User identity and profiles
//! - Gossip networking layer
//! - Message synchronization
//! - Storage and persistence
//!
//! **Architecture Note**: This is the RC1b implementation that replaces the old
//! saorsa_core-based CoreContext with a simpler saorsa-gossip + four-word-networking
//! based architecture.

use crate::disk_service::EntityDiskService;
use crate::keystore::Keystore;
use crate::message_sync::MessageSyncService;
use crate::types::{DeviceType, UserProfile};
use crate::webrtc::CommunitasWebRtcService;
use blake3;
use communitas_kanban::KanbanService;
use fips204::traits::{SerDes, Signer, Verifier};
use rand::rngs::OsRng;
use saorsa_pqc::ml_dsa_87::{PrivateKey, PublicKey, try_keygen_with_rng};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Centralized context for the Communitas application
///
/// This replaces the old saorsa_core-based CoreContext with a simpler architecture
/// based on saorsa-gossip for networking and four-word-networking for identities.
///
/// **Lifecycle**:
/// 1. Initialize with user profile (four-word ID, display name, device)
/// 2. Start gossip networking (optional, can run in local mode)
/// 3. Initialize message sync service
/// 4. Ready for operations
pub struct CoreContext {
    /// User profile with identity and device info
    pub profile: UserProfile,

    /// ML-DSA-87 post-quantum signing key (kept in memory for session)
    pub signing_key: PrivateKey,

    /// ML-DSA-87 post-quantum public key
    pub public_key: PublicKey,

    /// Four-word user identity (derived from public key)
    pub four_words: String,

    /// Display name for this user
    pub display_name: String,

    /// Device name for this instance
    pub device_name: String,

    /// CRDT manager for persistent document storage
    pub crdt_manager: Arc<crate::CrdtManager>,

    /// Entity service for managing groups, channels, and members
    pub entity_service: Arc<crate::EntityService>,

    /// Message service for unified messaging operations
    pub message_service: Arc<crate::MessageService>,

    /// Message synchronization service (CRDT-based) - legacy, use message_service instead
    pub message_sync: Arc<MessageSyncService>,

    /// Document replicator for collaborative editing (CRDT-based, Yrs)
    /// Handles dual-storage: Files (encrypted) + Web (public)
    pub doc_replicator: Arc<crate::doc_replicator::DocReplicator>,

    /// Current listen address (if networking is active)
    pub listen_address: Option<SocketAddr>,

    /// Connection identity (four-word encoded listen address)
    pub connection_identity: Option<String>,

    /// External/public address (NAT-reflected address for WAN connectivity)
    /// This is the address that other peers on the internet see us at,
    /// obtained via address reflection from a coordinator/bootstrap node
    pub external_address: Option<SocketAddr>,

    /// Gossip overlay system (replaces DHT-based networking)
    /// Handles P2P networking, membership, pubsub, presence, and discovery
    pub gossip: Option<Arc<crate::gossip::GossipContext>>,

    /// Group keys for channels/projects/orgs (MLS-based)
    /// Maps group ID to group keypair
    pub group_keys: HashMap<String, GroupKeyPairPlaceholder>,

    /// Per-entity virtual disk service (Private, Public, Shared disks)
    pub disk_service: Arc<EntityDiskService>,

    /// WebRTC service for voice, video, and screen sharing
    /// Initialized when networking starts (requires gossip context)
    pub webrtc: Option<Arc<CommunitasWebRtcService>>,

    /// Kanban service for project management boards
    /// CRDT-based, offline-first collaborative Kanban system
    pub kanban_service: Arc<KanbanService>,

    /// Invite service for cross-organization collaboration
    /// Handles four-word invite creation, acceptance, rejection, and revocation
    pub invite_service: Arc<crate::invite_service::InviteService>,
}

impl std::fmt::Debug for CoreContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoreContext")
            .field("profile", &self.profile)
            .field("four_words", &self.four_words)
            .field("display_name", &self.display_name)
            .field("device_name", &self.device_name)
            .field("crdt_manager", &"<active>")
            .field("entity_service", &"<active>")
            .field("message_service", &"<active>")
            .field("listen_address", &self.listen_address)
            .field("connection_identity", &self.connection_identity)
            .field("external_address", &self.external_address)
            .field("signing_key", &"<redacted>")
            .field("public_key", &"<key_bytes>")
            .field("doc_replicator", &"<active>")
            .field("gossip", &self.gossip.as_ref().map(|_| "<active>"))
            .field("group_keys", &self.group_keys)
            .field("disk_service", &"<active>")
            .field("webrtc", &self.webrtc.as_ref().map(|_| "<active>"))
            .field("kanban_service", &"<active>")
            .field("invite_service", &"<active>")
            .finish()
    }
}

/// Placeholder for group keypairs (will use MLS in future)
#[derive(Debug, Clone)]
pub struct GroupKeyPairPlaceholder {
    pub group_id: String,
    // Will be replaced with actual MLS group key material
}

impl CoreContext {
    /// Initialize a new CoreContext from a four-word identity
    ///
    /// This creates a new profile with:
    /// - Generated Ed25519 keypair
    /// - Four-word user identity derived from public key
    /// - Local storage directory
    ///
    /// **Note**: This does NOT start networking. Call `start_networking()` separately.
    ///
    /// # Arguments
    /// * `four_words` - Four-word user identity (e.g., "ocean-forest-moon-star")
    /// * `display_name` - Human-readable display name
    /// * `device_name` - Device identifier for this instance
    /// * `device_type` - Device type classification
    /// * `storage_dir` - Directory for profile storage
    ///
    /// # Returns
    /// New CoreContext instance
    ///
    /// # Errors
    /// Returns error if:
    /// - Four-word format is invalid
    /// - Storage directory cannot be created
    /// - Message sync initialization fails
    pub async fn initialize(
        four_words: String,
        display_name: String,
        device_name: String,
        device_type: DeviceType,
        storage_dir: PathBuf,
    ) -> Result<Self, String> {
        // Validate four-word format
        let words: Vec<&str> = four_words.split('-').collect();
        if words.len() != 4 {
            return Err(format!(
                "Invalid four-word format: expected 4 words, got {}",
                words.len()
            ));
        }

        // Initialize keystore for secure key management
        let keystore = Keystore::new();

        // Use four-word identity as hex identifier for keystore lookups
        let id_hex = blake3::hash(four_words.as_bytes()).to_hex().to_string();

        // Try to load existing ML-DSA keys from secure keystore
        let (public_key, signing_key) = match keystore.load_mldsa_keys(&id_hex) {
            Ok((pk_bytes, sk_bytes)) => {
                info!(
                    "Loaded existing ML-DSA-87 keypair for identity '{}' from keystore",
                    four_words
                );

                // Deserialize keys from stored bytes
                let public_key = PublicKey::try_from_bytes(
                    pk_bytes
                        .as_slice()
                        .try_into()
                        .map_err(|_| "Invalid public key length in keystore".to_string())?,
                )
                .map_err(|e| format!("Failed to deserialize public key: {}", e))?;

                let signing_key = PrivateKey::try_from_bytes(
                    sk_bytes
                        .as_slice()
                        .try_into()
                        .map_err(|_| "Invalid signing key length in keystore".to_string())?,
                )
                .map_err(|e| format!("Failed to deserialize signing key: {}", e))?;

                (public_key, signing_key)
            }
            Err(_) => {
                // No existing keys - generate new ones using cryptographically secure RNG
                info!(
                    "Generating new ML-DSA-87 keypair for identity '{}' using CSPRNG",
                    four_words
                );

                let mut rng = OsRng;
                let (public_key, signing_key) = try_keygen_with_rng(&mut rng)
                    .map_err(|e| format!("Failed to generate ML-DSA-87 keypair: {}", e))?;

                // Store keys securely in platform keychain
                let pk_bytes = public_key.clone().into_bytes();
                let sk_bytes = signing_key.clone().into_bytes();

                keystore
                    .save_mldsa_keys(&id_hex, &pk_bytes, &sk_bytes)
                    .map_err(|e| {
                        warn!(
                            "Failed to save keys to keystore: {}. Keys will not persist.",
                            e
                        );
                        e
                    })?;

                info!("Saved ML-DSA-87 keypair to secure keystore (Level 5 PQC security)");

                (public_key, signing_key)
            }
        };

        // Get public key bytes for UserProfile
        let pubkey_bytes = public_key.clone().into_bytes();

        // Create storage directory if it doesn't exist
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir).map_err(|e| {
                format!(
                    "Failed to create storage directory {:?}: {}",
                    storage_dir, e
                )
            })?;
        }

        // For UserProfile, we need a fixed-size array. Use first 32 bytes of public key
        let pubkey_array: [u8; 32] = pubkey_bytes[..32]
            .try_into()
            .map_err(|_| "Public key too short".to_string())?;

        // Create user profile
        let profile = UserProfile::new(
            four_words.clone(),
            display_name.clone(),
            pubkey_array,
            device_type,
            storage_dir.clone(),
        );

        // Initialize CRDT manager for persistent storage
        let crdt_manager = Arc::new(
            crate::CrdtManager::new(&storage_dir.join("crdt.db"))
                .await
                .map_err(|e| format!("Failed to initialize CrdtManager: {}", e))?,
        );

        // Initialize entity service for managing groups, channels, and members
        let entity_service = Arc::new(crate::EntityService::new(crdt_manager.clone()));

        // Initialize unified message service
        let message_service = Arc::new(crate::MessageService::new(four_words.clone()));

        // Initialize legacy message sync service (for backward compatibility)
        let message_sync = Arc::new(MessageSyncService::new(four_words.clone()));

        // Initialize document replicator with dual storage enabled (Sprint 3.2)
        let doc_config = crate::doc_replicator::DocReplicatorConfig {
            files_storage_enabled: true,
            web_storage_enabled: true,
        };
        let doc_replicator = Arc::new(
            crate::doc_replicator::DocReplicator::new(doc_config)
                .await
                .map_err(|e| format!("Failed to initialize DocReplicator: {}", e))?,
        );

        // Initialize per-entity virtual disk service
        let disk_root = storage_dir.join("disks");
        let disk_service = Arc::new(
            EntityDiskService::new(&disk_root)
                .await
                .map_err(|e| format!("Failed to initialize EntityDiskService: {}", e))?,
        );

        // Initialize Kanban service for project management boards
        let kanban_service = Arc::new(KanbanService::new(four_words.clone()));

        // Initialize Invite service for cross-organization collaboration
        let invite_service = Arc::new(crate::invite_service::InviteService::new(
            crdt_manager.clone(),
            entity_service.clone(),
        ));

        info!(
            "CoreContext initialized for user '{}' ({}) with EntityService, MessageService, DocReplicator, DiskService, KanbanService, and InviteService",
            display_name, four_words
        );

        Ok(Self {
            profile,
            signing_key,
            public_key,
            four_words,
            display_name,
            device_name,
            crdt_manager,
            entity_service,
            message_service,
            message_sync,
            doc_replicator,
            listen_address: None,
            connection_identity: None,
            external_address: None,
            gossip: None,
            group_keys: HashMap::new(),
            disk_service,
            webrtc: None, // Initialized when networking starts
            kanban_service,
            invite_service,
        })
    }

    /// Start networking with gossip overlay system
    ///
    /// Initializes the saorsa-gossip based P2P networking layer:
    /// - Creates QUIC transport on random high port (49152-65535)
    /// - Initializes HyParView membership and Plumtree pubsub
    /// - Sets up peer cache for fast boot
    /// - Connects to coordinator and rendezvous services
    /// - Generates connection identity (four-word encoded address)
    ///
    /// # Arguments
    /// * `_port` - Optional specific port (currently ignored, transport auto-selects)
    ///
    /// # Returns
    /// Connection identity (four-word encoded address)
    pub async fn start_networking(
        &mut self,
        preferred_port: Option<u16>,
    ) -> Result<String, String> {
        info!("Starting gossip networking for {}", self.four_words);

        // Allocate UDP port using PortManager
        let mut port_manager = if let Some(port) = preferred_port {
            crate::gossip::PortManager::with_preferred_port(port)
        } else {
            crate::gossip::PortManager::new()
        };

        let listen_port = port_manager
            .allocate_port()
            .map_err(|e| format!("Failed to allocate port: {}", e))?;

        info!("Allocated port {} for QUIC transport", listen_port);

        // Initialize gossip context with allocated port
        let gossip_ctx = crate::gossip::GossipContext::initialize(
            self.four_words.clone(),
            self.display_name.clone(),
            self.device_name.clone(),
            Some(listen_port),
        )
        .await
        .map_err(|e| format!("Failed to initialize gossip: {}", e))?;

        // Execute gossip boot sequence (SPEC.md §2)
        // This enables: membership, topic subscriptions, presence, and CRDT anti-entropy
        info!("Executing gossip boot sequence (5 steps)");
        let mut boot_sequence = crate::gossip::GossipBootSequence::new(gossip_ctx);
        boot_sequence
            .execute()
            .await
            .map_err(|e| format!("Failed to execute boot sequence: {}", e))?;

        // Extract the gossip context after boot
        let gossip = boot_sequence.into_context();
        info!("Gossip boot sequence completed successfully");

        // Build listen address
        let local_ip =
            local_ip_address::local_ip().map_err(|e| format!("Failed to get local IP: {}", e))?;

        let listen_addr = std::net::SocketAddr::new(local_ip, listen_port);

        // Generate connection identity using four-word encoding
        let connection_identity = crate::conn_words(&listen_addr)
            .map_err(|e| format!("Failed to encode connection address: {}", e))?;

        info!(
            "Gossip networking started on {} ({})",
            listen_addr, connection_identity
        );

        self.listen_address = Some(listen_addr);
        self.connection_identity = Some(connection_identity.clone());
        let gossip_arc = Arc::new(gossip);
        self.gossip = Some(gossip_arc.clone());

        // Initialize WebRTC service (requires gossip context)
        match CommunitasWebRtcService::new(gossip_arc).await {
            Ok(webrtc) => {
                info!("WebRTC service initialized successfully");
                self.webrtc = Some(Arc::new(webrtc));
            }
            Err(e) => {
                warn!(
                    "Failed to initialize WebRTC service: {}. Voice/video calls will be unavailable.",
                    e
                );
                // Don't fail networking start - WebRTC is optional
            }
        }

        // Auto-request external address after a brief delay (allow time for peer connections)
        // This is non-blocking and best-effort - failure is logged but doesn't affect networking
        info!("Scheduling automatic external address detection...");

        Ok(connection_identity)
    }

    /// Automatically request external address with retry logic
    ///
    /// This is called after networking starts to discover our public IP address.
    /// It retries a few times with delays to allow peer connections to establish.
    ///
    /// # Returns
    /// Ok if external address was successfully determined, Err otherwise
    pub async fn auto_request_external_address(&mut self) -> Result<(), String> {
        // Retry up to 3 times with 2 second delays to allow peer connections
        for attempt in 1..=3 {
            info!("Auto-detecting external address (attempt {}/3)...", attempt);

            // Wait a bit for peers to connect (first attempt waits longer)
            let delay_ms = if attempt == 1 { 2000 } else { 1000 };
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;

            match self.request_external_address().await {
                Ok(()) => {
                    if let Some(addr) = self.external_address {
                        info!("Auto-detected external address: {}", addr);
                        return Ok(());
                    }
                }
                Err(e) => {
                    if attempt < 3 {
                        warn!(
                            "External address detection attempt {} failed: {}. Retrying...",
                            attempt, e
                        );
                    } else {
                        warn!("External address detection failed after 3 attempts: {}", e);
                    }
                }
            }
        }

        Err("Failed to auto-detect external address after 3 attempts".to_string())
    }

    /// Request external/public address via NAT reflection from a bootstrap node
    ///
    /// Get our external IP address and port as seen from the internet.
    ///
    /// This uses the native QUIC OBSERVED_ADDRESS frame mechanism (draft-ietf-quic-address-discovery)
    /// which is automatically exchanged during QUIC connection establishment.
    ///
    /// Should be called after networking is active and we have connections to bootstrap nodes.
    ///
    /// # Returns
    /// The external address if successfully obtained, or an error message
    pub async fn request_external_address(&mut self) -> Result<(), String> {
        let gossip = self
            .gossip
            .as_ref()
            .ok_or("Networking not started. Call start_networking() first")?;

        info!("Requesting external address via native QUIC address discovery");

        // Use the native QUIC OBSERVED_ADDRESS mechanism through the transport layer
        // This is the standard way to discover external address via QUIC connections
        if let Some(external_addr) = gossip.transport.get_external_address() {
            info!(
                "Got external address via QUIC OBSERVED_ADDRESS: {}",
                external_addr
            );
            self.external_address = Some(external_addr);
            return Ok(());
        }

        // If native discovery didn't work, check if we have any connected peers at all
        let cache = gossip.peer_cache.read().await;
        let peers = cache.get_top_peers(5);
        drop(cache);

        if peers.is_empty() {
            return Err("No connected peers - external address not yet available. \
                 The address will be discovered automatically when connections are established."
                .to_string());
        }

        // We have peers but no observed address yet - this can happen if:
        // 1. The connection is still being established
        // 2. The remote peer doesn't support OBSERVED_ADDRESS frames
        // 3. We're behind a very restrictive NAT
        warn!(
            "Connected to {} peers but no OBSERVED_ADDRESS received yet",
            peers.len()
        );
        Err(
            "External address not yet available - waiting for OBSERVED_ADDRESS frame from peers"
                .to_string(),
        )
    }

    /// Stop networking gracefully
    ///
    /// Shuts down gossip services, presence beacons, and transport
    pub async fn stop_networking(&mut self) -> Result<(), String> {
        if let Some(_gossip) = self.gossip.take() {
            info!("Stopping gossip networking for {}", self.four_words);
            // Clear WebRTC service (depends on gossip)
            self.webrtc = None;
            // Note: Graceful shutdown will be implemented when needed
            // The Arc drop will clean up resources
            self.listen_address = None;
            self.connection_identity = None;
            self.external_address = None;
        }
        Ok(())
    }

    /// Connect to a peer using their four-word identity
    ///
    /// This adds the peer to favourite contacts and initiates FOAF discovery.
    /// The gossip overlay will find and connect to the peer automatically.
    ///
    /// # Arguments
    /// * `peer_four_words` - The peer's four-word identity (e.g., "ocean-forest-moon-star")
    ///
    /// # Returns
    /// Success if peer added to favourites
    pub async fn connect_to_peer(&self, peer_four_words: &str) -> Result<(), String> {
        let gossip = self
            .gossip
            .as_ref()
            .ok_or("Networking not started. Call start_networking() first")?;

        info!("Adding peer {} to favourites", peer_four_words);

        // Add peer to favourite contacts
        gossip
            .add_favourite_contact(peer_four_words.to_string())
            .await
            .map_err(|e| format!("Failed to add favourite contact: {}", e))?;

        // Try to decode as connection address and dial immediately (for bootstrap)
        if let Ok(addr) = crate::identity::conn_from_words(peer_four_words) {
            info!("Dialing peer at {} ({})", addr, peer_four_words);
            // Use internal transport to dial
            // We need to convert SocketAddr to the transport's format or use a dial method on GossipContext if available.
            // GossipContext doesn't have a high-level dial.
            // But we can use the transport directly if we can access it.
            // gossip.transport is public.
            // QuicTransport::connect_to_peer requires PeerId, which we don't have yet if we only have address.
            // But QuicTransport usually has a connect() method that takes SocketAddr?
            // Checking saorsa_gossip_transport::QuicTransport...
            // It usually handles connection by PeerId lookup OR direct dial.

            // Actually, HyParView membership handles "join" via address?
            // gossip.membership.write().await.join(addr)...?

            // Let's use a simpler approach: if we have the address, we can try to "introduce" via membership.
            // Or we can use `gossip.transport.dial(addr)` if it exists.

            // Since I cannot see saorsa_gossip_transport source easily, I'll assume `membership` is the way.
            // `membership` is `Box<dyn Membership>`.
            // `Membership` usually has `join(addr)`.

            // Let's try to access gossip.dial_address(addr) if I add it to GossipContext.
            // Or just leave it as is and implement `dial_address` in `GossipContext`.
            if let Err(e) = gossip.dial_address(addr).await {
                warn!("Failed to dial peer {}: {}", addr, e);
            } else {
                info!("Successfully dialed peer {}", addr);
            }
        }

        // The gossip overlay will automatically discover and connect via FOAF
        info!(
            "Peer {} added. FOAF discovery will locate and connect automatically",
            peer_four_words
        );

        Ok(())
    }

    /// Add a group key for a channel/project/org
    ///
    /// # Arguments
    /// * `group_id` - Group identifier
    /// * `key` - Group key material (placeholder for now)
    pub fn add_group_key(&mut self, group_id: String, key: GroupKeyPairPlaceholder) {
        self.group_keys.insert(group_id, key);
    }

    /// Get the ML-DSA-87 public key for this identity
    pub fn get_public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the public key bytes
    /// ML-DSA-87 public keys are 2592 bytes
    pub fn public_key_bytes(&self) -> [u8; 2592] {
        self.public_key.clone().into_bytes()
    }

    /// Sign a message with ML-DSA-87 post-quantum signature
    ///
    /// # Arguments
    /// * `message` - Message bytes to sign
    ///
    /// # Returns
    /// ML-DSA-87 signature (4627 bytes for ML-DSA-87)
    ///
    /// # Errors
    /// Returns error if signing fails
    pub fn sign(&self, message: &[u8]) -> Result<[u8; 4627], String> {
        self.signing_key
            .try_sign(message, &[]) // Empty context
            .map_err(|e| format!("ML-DSA-87 signing failed: {}", e))
    }

    /// Verify an ML-DSA-87 signature
    ///
    /// # Arguments
    /// * `message` - Message bytes that were signed
    /// * `signature` - ML-DSA-87 signature (4627 bytes)
    ///
    /// # Returns
    /// true if signature is valid, false otherwise
    pub fn verify(&self, message: &[u8], signature: &[u8; 4627]) -> bool {
        self.public_key.verify(message, signature, &[]) // Empty context, returns bool directly
    }

    /// Get the storage directory for this profile
    pub fn storage_dir(&self) -> &PathBuf {
        &self.profile.storage_dir
    }

    /// Check if networking is active
    pub fn is_networking_active(&self) -> bool {
        self.gossip.is_some() && self.listen_address.is_some()
    }

    /// Get connection identity (if networking is active)
    pub fn connection_identity(&self) -> Option<&str> {
        self.connection_identity.as_deref()
    }

    /// Update display name
    pub fn set_display_name(&mut self, display_name: String) {
        self.display_name = display_name.clone();
        self.profile.display_name = display_name;
    }

    /// Get device type
    pub fn device_type(&self) -> DeviceType {
        self.profile.device_type
    }

    /// Check if passkey is registered for this profile
    pub fn has_passkey(&self) -> bool {
        self.profile.has_passkey()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_core_context_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_path_buf();

        let context = CoreContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            DeviceType::Desktop,
            storage_dir.clone(),
        )
        .await;

        assert!(context.is_ok());
        let ctx = context.unwrap();

        assert_eq!(ctx.display_name, "Test User");
        assert_eq!(ctx.device_name, "Test Device");
        assert_eq!(ctx.profile.device_type, DeviceType::Desktop);
        assert_eq!(ctx.four_words, "ocean-forest-moon-star");
        assert!(!ctx.is_networking_active());
        assert!(storage_dir.exists());
    }

    #[tokio::test]
    async fn test_invalid_four_word_format() {
        let temp_dir = TempDir::new().unwrap();

        let context = CoreContext::initialize(
            "only-three-words".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            DeviceType::Desktop,
            temp_dir.path().to_path_buf(),
        )
        .await;

        assert!(context.is_err());
        assert!(context.unwrap_err().contains("Invalid four-word format"));
    }

    #[tokio::test]
    async fn test_display_name_update() {
        let temp_dir = TempDir::new().unwrap();

        let mut context = CoreContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Old Name".to_string(),
            "Test Device".to_string(),
            DeviceType::Desktop,
            temp_dir.path().to_path_buf(),
        )
        .await
        .unwrap();

        context.set_display_name("New Name".to_string());

        assert_eq!(context.display_name, "New Name");
        assert_eq!(context.profile.display_name, "New Name");
    }

    #[tokio::test]
    async fn test_signing() {
        let temp_dir = TempDir::new().unwrap();

        let context = CoreContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            DeviceType::Desktop,
            temp_dir.path().to_path_buf(),
        )
        .await
        .unwrap();

        let message = b"test message";
        let signature = context.sign(message).unwrap();

        // Verify ML-DSA-87 signature
        assert!(context.verify(message, &signature));

        // Verify signature fails with wrong message
        let wrong_message = b"wrong message";
        assert!(!context.verify(wrong_message, &signature));
    }

    #[tokio::test]
    async fn test_group_key_management() {
        let temp_dir = TempDir::new().unwrap();

        let mut context = CoreContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            DeviceType::Desktop,
            temp_dir.path().to_path_buf(),
        )
        .await
        .unwrap();

        let group_id = "test-group".to_string();
        let key = GroupKeyPairPlaceholder {
            group_id: group_id.clone(),
        };

        context.add_group_key(group_id.clone(), key);

        assert!(context.group_keys.contains_key(&group_id));
    }

    #[tokio::test]
    async fn test_networking_not_active_by_default() {
        let temp_dir = TempDir::new().unwrap();

        let context = CoreContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            DeviceType::Desktop,
            temp_dir.path().to_path_buf(),
        )
        .await
        .unwrap();

        assert!(!context.is_networking_active());
        assert!(context.connection_identity().is_none());
    }
}

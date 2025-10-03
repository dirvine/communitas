use crate::bootstrap_integration::EnhancedBootstrapManager;
use crate::encrypted_storage::{EncryptedStorageManager, StorageConfig};
use saorsa_core::address::NetworkAddress;
use saorsa_core::chat::ChatManager;
use saorsa_core::identity::IdentityManager;
use saorsa_core::identity::enhanced::{DeviceType, EnhancedIdentity, EnhancedIdentityManager};
use saorsa_core::identity::manager::IdentityManagerConfig;
use saorsa_core::messaging::DhtClient;
use saorsa_core::messaging::service::MessagingService;
use saorsa_core::network::P2PNode;
use saorsa_core::storage::StorageManager;
use saorsa_core::{
    dht::core_engine::{DhtCoreEngine, NodeId},
    identity::FourWordAddress,
};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tracing::{info, warn};

// Group key storage for membership updates
use saorsa_core::api::GroupKeyPair;

/// Centralized context that wires Communitas to saorsa-core components.
/// This avoids re-implementations in this repo and delegates to saorsa-core.
pub struct CoreContext {
    pub four_words: String,
    pub identity: EnhancedIdentity,
    pub display_name: String, // Store display name separately
    pub storage: StorageManager,
    pub chat: ChatManager,
    pub messaging: MessagingService,
    pub group_keys: HashMap<String, GroupKeyPair>,
    pub bootstrap_manager: Option<Arc<EnhancedBootstrapManager>>,
    pub encrypted_storage: Option<Arc<EncryptedStorageManager>>,
    pub device_name: String,
    pub local_endpoint: Option<NetworkAddress>, // Local network endpoint
    pub dht_client: DhtClient,                  // For direct DHT access
    pub p2p_node: Option<Arc<TokioRwLock<P2PNode>>>, // P2P networking node
}

impl CoreContext {
    /// Build a new CoreContext from a four-word identity and display/device info.
    pub async fn initialize(
        four_words: String,
        display_name: String,
        device_name: String,
        device_type: DeviceType,
    ) -> Result<Self, String> {
        // Basic validation of four-word address (delegate to saorsa-core when possible)
        let words: Vec<&str> = four_words.split('-').collect();
        if words.len() != 4 {
            return Err(
                "Four-word address must contain exactly 4 words separated by hyphens".to_string(),
            );
        }
        let word_array = [
            words[0].to_string(),
            words[1].to_string(),
            words[2].to_string(),
            words[3].to_string(),
        ];
        if !saorsa_core::fwid::fw_check(word_array) {
            return Err("Invalid four-word address format".to_string());
        }

        // Identity manager and base identity
        let id_mgr = IdentityManager::new(IdentityManagerConfig::default());
        let base = id_mgr
            .create_identity(display_name.clone(), four_words.clone(), None, None)
            .await
            .map_err(|e| format!("Failed to create identity: {}", e))?;

        // Enhanced identity (PQC + threshold-ready)
        let mut enhanced_mgr = EnhancedIdentityManager::new(id_mgr);
        let enhanced_identity = enhanced_mgr
            .create_enhanced_identity(base, device_name.clone(), device_type)
            .await
            .map_err(|e| format!("Failed to create enhanced identity: {}", e))?;

        // Messaging DHT client (single-node engine for now)
        let dht_client = DhtClient::new().map_err(|e| format!("DHT init failed: {}", e))?;

        // Storage manager requires a DHT engine instance
        let dht_engine = DhtCoreEngine::new(NodeId::from_bytes([42u8; 32]))
            .map_err(|e| format!("DHT engine creation failed: {}", e))?;
        let storage = StorageManager::new(dht_engine, &enhanced_identity)
            .map_err(|e| format!("Storage init failed: {}", e))?;

        // Chat manager backed by storage and identity
        // Note: StorageManager doesn't implement Clone, so we create a new instance
        let dht_engine_chat = DhtCoreEngine::new(NodeId::from_bytes([43u8; 32]))
            .map_err(|e| format!("DHT engine creation for chat failed: {}", e))?;
        let storage_chat = StorageManager::new(dht_engine_chat, &enhanced_identity)
            .map_err(|e| format!("Storage init for chat failed: {}", e))?;
        let chat = ChatManager::new(storage_chat, enhanced_identity.clone());

        // Messaging service
        let messaging =
            MessagingService::new(FourWordAddress(four_words.clone()), dht_client.clone())
                .await
                .map_err(|e| format!("Messaging init failed: {}", e))?;

        // Bootstrap manager for network connectivity
        let bootstrap_config = crate::bootstrap_integration::BootstrapConfig::default();
        let bootstrap_manager = match EnhancedBootstrapManager::new(bootstrap_config).await {
            Ok(manager) => {
                // Start background tasks for bootstrap management
                let _ = manager.start_background_tasks().await;
                Some(Arc::new(manager))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to create bootstrap manager: {}. Continuing without it.",
                    e
                );
                None
            }
        };

        // Initialize encrypted storage manager
        let storage_config = StorageConfig::default();
        let encrypted_storage = match EncryptedStorageManager::new(storage_config).await {
            Ok(manager) => Some(Arc::new(manager)),
            Err(e) => {
                tracing::warn!(
                    "Failed to create encrypted storage manager: {}. Continuing without it.",
                    e
                );
                None
            }
        };

        // NOTE: P2P networking is now handled by MessagingService internally
        // We don't create a separate P2P node to avoid port conflicts
        // The messaging service already starts ant-quic networking
        let local_endpoint = None; // Will be populated from messaging service if needed
        let p2p_node = None; // Messaging service handles P2P internally

        let ctx = Self {
            four_words,
            identity: enhanced_identity,
            display_name,
            storage,
            chat,
            messaging,
            group_keys: HashMap::new(),
            bootstrap_manager,
            encrypted_storage,
            device_name,
            local_endpoint,
            dht_client: dht_client.clone(),
            p2p_node,
        };

        // Auto-register PeerInfo in DHT after P2P initialization
        if let Some(socket_addr) = ctx.get_local_endpoint_socket().await {
            info!("Auto-registering PeerInfo in DHT: {}", socket_addr);
            if let Err(e) = ctx.publish_peer_info_to_dht().await {
                warn!("Failed to auto-register PeerInfo: {}", e);
            } else {
                info!("✅ PeerInfo registered in DHT");
            }
        }

        Ok(ctx)
    }

    /// Initialize CoreContext with a shared DHT client (for testing only)
    /// This allows multiple CoreContext instances to share KEM public keys via the same DHT
    pub async fn initialize_with_shared_dht(
        four_words: String,
        display_name: String,
        device_name: String,
        device_type: DeviceType,
        shared_dht: DhtClient,
    ) -> Result<Self, String> {
        // Basic validation of four-word address
        let words: Vec<&str> = four_words.split('-').collect();
        if words.len() != 4 {
            return Err(
                "Four-word address must contain exactly 4 words separated by hyphens".to_string(),
            );
        }
        let word_array = [
            words[0].to_string(),
            words[1].to_string(),
            words[2].to_string(),
            words[3].to_string(),
        ];
        if !saorsa_core::fwid::fw_check(word_array) {
            return Err("Invalid four-word address format".to_string());
        }

        // Identity manager and base identity
        let id_mgr = IdentityManager::new(IdentityManagerConfig::default());
        let base = id_mgr
            .create_identity(display_name.clone(), four_words.clone(), None, None)
            .await
            .map_err(|e| format!("Failed to create identity: {}", e))?;

        // Enhanced identity (PQC + threshold-ready)
        let mut enhanced_mgr = EnhancedIdentityManager::new(id_mgr);
        let enhanced_identity = enhanced_mgr
            .create_enhanced_identity(base, device_name.clone(), device_type)
            .await
            .map_err(|e| format!("Failed to create enhanced identity: {}", e))?;

        // Storage manager - create separate DHT engines for storage
        let dht_engine = DhtCoreEngine::new(NodeId::from_bytes([42u8; 32]))
            .map_err(|e| format!("DHT engine creation failed: {}", e))?;
        let storage = StorageManager::new(dht_engine, &enhanced_identity)
            .map_err(|e| format!("Storage init failed: {}", e))?;

        // Chat manager - create separate DHT engine for chat
        let dht_engine_chat = DhtCoreEngine::new(NodeId::from_bytes([43u8; 32]))
            .map_err(|e| format!("DHT engine creation for chat failed: {}", e))?;
        let storage_chat = StorageManager::new(dht_engine_chat, &enhanced_identity)
            .map_err(|e| format!("Storage init for chat failed: {}", e))?;
        let chat = ChatManager::new(storage_chat, enhanced_identity.clone());

        // Messaging service - USE SHARED DHT for KEM key sharing
        let messaging =
            MessagingService::new(FourWordAddress(four_words.clone()), shared_dht.clone())
                .await
                .map_err(|e| format!("Messaging init failed: {}", e))?;

        // Bootstrap manager
        let bootstrap_config = crate::bootstrap_integration::BootstrapConfig::default();
        let bootstrap_manager = match EnhancedBootstrapManager::new(bootstrap_config).await {
            Ok(manager) => {
                let _ = manager.start_background_tasks().await;
                Some(Arc::new(manager))
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to create bootstrap manager: {}. Continuing without it.",
                    e
                );
                None
            }
        };

        // Encrypted storage manager
        let storage_config = StorageConfig::default();
        let encrypted_storage = match EncryptedStorageManager::new(storage_config).await {
            Ok(manager) => Some(Arc::new(manager)),
            Err(e) => {
                tracing::warn!(
                    "Failed to create encrypted storage manager: {}. Continuing without it.",
                    e
                );
                None
            }
        };

        let local_endpoint = None;
        let p2p_node = None;

        let ctx = Self {
            four_words,
            identity: enhanced_identity,
            display_name,
            storage,
            chat,
            messaging,
            group_keys: HashMap::new(),
            bootstrap_manager,
            encrypted_storage,
            device_name,
            local_endpoint,
            dht_client: shared_dht.clone(), // Use the shared DHT client
            p2p_node,
        };

        // Auto-register PeerInfo in DHT after P2P initialization
        if let Some(socket_addr) = ctx.get_local_endpoint_socket().await {
            info!("Auto-registering PeerInfo in shared DHT: {}", socket_addr);
            if let Err(e) = ctx.publish_peer_info_to_dht().await {
                warn!("Failed to auto-register PeerInfo: {}", e);
            } else {
                info!("✅ PeerInfo registered in shared DHT");
            }
        }

        Ok(ctx)
    }

    /// Publish this node's peer information to the DHT for discovery
    /// This enables other peers to find our network address via DHT lookup
    pub async fn publish_peer_info_to_dht(&self) -> Result<(), String> {
        use chrono::Utc;
        use serde_json::json;
        use std::net::{IpAddr, Ipv4Addr};

        // Get our socket address
        let mut socket = self
            .get_local_endpoint_socket()
            .await
            .ok_or_else(|| "No local endpoint available".to_string())?;

        // Replace unspecified address (0.0.0.0) with localhost for local testing
        if socket.ip().is_unspecified() {
            socket.set_ip(IpAddr::V4(Ipv4Addr::LOCALHOST));
        }

        // Create PeerInfo JSON structure
        let peer_info = json!({
            "addresses": vec![socket.to_string()],
            "public_key": Vec::<u8>::new(),
            "capabilities": vec!["p2p", "messaging"],
            "last_seen": Utc::now(),
        });

        // Publish to DHT under key "peer:{identity}"
        let key = format!("peer:{}", self.four_words);
        let serialized = serde_json::to_vec(&peer_info)
            .map_err(|e| format!("Failed to serialize PeerInfo: {}", e))?;

        self.dht_client
            .put(key, serialized)
            .await
            .map_err(|e| format!("Failed to publish PeerInfo to DHT: {}", e))?;

        Ok(())
    }

    /// Get the encrypted storage manager, creating it if necessary
    pub async fn get_storage_manager(&self) -> Result<Arc<EncryptedStorageManager>, String> {
        if let Some(manager) = &self.encrypted_storage {
            Ok(manager.clone())
        } else {
            // Create on-demand if not initialized
            let storage_config = StorageConfig::default();
            let manager = EncryptedStorageManager::new(storage_config)
                .await
                .map_err(|e| format!("Failed to create encrypted storage: {}", e))?;
            Ok(Arc::new(manager))
        }
    }

    /// Get current identity information
    pub async fn get_current_identity(&self) -> Option<IdentityInfo> {
        Some(IdentityInfo {
            four_words: self.four_words.clone(),
            display_name: self.display_name.clone(),
        })
    }

    /// Get the user ID for this identity
    /// Returns the UserId from the base identity
    pub fn get_user_id(&self) -> String {
        // UserId is a String in saorsa_core::chat
        // Access it from the base_identity field
        self.identity.base_identity.user_id.clone()
    }

    /// Get the four-word network address for this identity
    /// This is the address used for P2P messaging
    pub fn get_four_words(&self) -> String {
        self.four_words.clone()
    }

    /// Mark a peer as online and establish session for messaging
    /// This should be called after connect_to_peer to prepare for message exchange
    pub async fn mark_peer_online(&self, four_words: &str) -> Result<(), String> {
        use saorsa_core::identity::FourWordAddress;

        let peer_addr = FourWordAddress::parse_str(four_words)
            .map_err(|e| format!("Failed to parse four-word address: {}", e))?;

        tracing::info!("Marking peer online: {}", four_words);

        self.messaging
            .mark_user_online(peer_addr)
            .await
            .map_err(|e| format!("Failed to mark user online: {}", e))?;

        tracing::info!("✅ Peer marked online: {}", four_words);
        Ok(())
    }

    /// Add a member to a channel
    pub async fn add_channel_member(
        &mut self,
        channel_id: &str,
        user_id: String,
        role: saorsa_core::chat::ChannelRole,
    ) -> Result<(), String> {
        use saorsa_core::chat::ChannelId;

        // ChannelId is just a String wrapper
        let channel_id_obj = ChannelId(channel_id.to_string());

        // Add member via ChatManager
        self.chat
            .add_member(&channel_id_obj, user_id, role)
            .await
            .map_err(|e| format!("Failed to add member to channel: {}", e))?;

        tracing::info!("Added member to channel: {}", channel_id);
        Ok(())
    }

    /// Get members of a channel
    pub async fn get_channel_members(&self, channel_id: &str) -> Result<Vec<String>, String> {
        use saorsa_core::chat::ChannelId;

        // ChannelId is just a String wrapper
        let channel_id_obj = ChannelId(channel_id.to_string());

        // Get channel info which includes members
        let channel = self
            .chat
            .get_channel(&channel_id_obj)
            .await
            .map_err(|e| format!("Failed to get channel: {}", e))?;

        Ok(channel.members.into_iter().map(|m| m.user_id).collect())
    }

    /// Send a message to a channel via P2P MessagingService
    /// Returns the message ID
    pub async fn send_channel_message(
        &mut self,
        channel_id: &str,
        content: &str,
    ) -> Result<String, String> {
        use saorsa_core::chat::ChannelId as ChatChannelId;
        use saorsa_core::identity::FourWordAddress;
        use saorsa_core::messaging::{ChannelId as MsgChannelId, MessageContent, SendOptions};
        use uuid::Uuid;

        // Parse channel ID
        let channel_uuid = Uuid::parse_str(channel_id)
            .map_err(|e| format!("Invalid channel ID '{}': {}", channel_id, e))?;

        // Get channel from ChatManager to access members
        let chat_channel_id = ChatChannelId(channel_id.to_string());
        let channel = self
            .chat
            .get_channel(&chat_channel_id)
            .await
            .map_err(|e| format!("Failed to get channel: {}", e))?;

        // Convert channel members to FourWordAddress recipients
        // Each member's user_id is a four-word address string (e.g., "ocean-forest-moon-star")
        let recipients: Vec<FourWordAddress> = channel
            .members
            .iter()
            .filter_map(|member| {
                // user_id is the four-word address string
                match FourWordAddress::parse_str(&member.user_id) {
                    Ok(addr) => Some(addr),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to parse member {} as FourWordAddress: {}",
                            member.user_id,
                            e
                        );
                        None
                    }
                }
            })
            .collect();

        if recipients.is_empty() {
            return Err(format!(
                "No valid recipients found for channel {}",
                channel_id
            ));
        }

        tracing::info!(
            "Sending P2P message to {} recipients in channel {}",
            recipients.len(),
            channel_id
        );

        // Create message content for P2P delivery
        let message_content = MessageContent::Text(content.to_string());

        // Create send options (default: non-ephemeral, no expiry)
        let options = SendOptions {
            ephemeral: false,
            expiry_seconds: None,
            reply_to: None,
            thread_id: None,
            attachments: vec![],
        };

        // Create MessagingService channel ID
        let msg_channel_id = MsgChannelId(channel_uuid);

        // Send via MessagingService with retry logic for PQC key exchange
        // First attempt may fail with "No session key established" which triggers
        // background key exchange via initiate_exchange(). Retry after waiting.
        let (message_id, delivery_receipt) = {
            let mut last_error: Option<anyhow::Error> = None;
            let mut result = None;

            for attempt in 1..=3 {
                match self
                    .messaging
                    .send_message(
                        recipients.clone(),
                        message_content.clone(),
                        msg_channel_id,
                        options.clone(),
                    )
                    .await
                {
                    Ok(send_result) => {
                        if attempt > 1 {
                            tracing::info!("✅ Message sent successfully on attempt {}", attempt);
                        }
                        result = Some(send_result);
                        break;
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        if err_str.contains("No session key established")
                            || err_str.contains("No established PQC session")
                        {
                            tracing::warn!(
                                "Attempt {}/3: PQC session not ready, waiting for key exchange...",
                                attempt
                            );
                            tracing::debug!("Key exchange error: {}", err_str);
                            last_error = Some(e);

                            if attempt < 3 {
                                // Wait progressively longer for key exchange to complete
                                let wait_ms = 500 * attempt as u64;
                                tracing::debug!("Waiting {}ms before retry...", wait_ms);
                                tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms))
                                    .await;
                            }
                        } else {
                            // Different error - fail immediately
                            tracing::error!("Non-retryable error during send_message: {}", err_str);
                            return Err(format!("Failed to send P2P message: {}", err_str));
                        }
                    }
                }
            }

            result.ok_or_else(|| {
                let err_msg = last_error
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "Unknown error".to_string());
                format!("Failed to send P2P message after 3 retries: {}", err_msg)
            })?
        };

        let msg_id_str = message_id.0.to_string();

        // Count successful deliveries
        use saorsa_core::messaging::DeliveryStatus;
        let successful_count = delivery_receipt
            .delivery_status
            .iter()
            .filter(|(_, status)| matches!(status, DeliveryStatus::Delivered(_)))
            .count();

        tracing::info!(
            "✅ Sent P2P message {} to channel {} - Delivered to {}/{} recipients",
            msg_id_str,
            channel_id,
            successful_count,
            delivery_receipt.delivery_status.len()
        );

        Ok(msg_id_str)
    }

    /// Get recent messages from a channel
    pub async fn get_channel_messages(
        &self,
        channel_id: &str,
        limit: usize,
    ) -> Result<Vec<saorsa_core::messaging::RichMessage>, String> {
        use saorsa_core::messaging::ChannelId;
        use uuid::Uuid;

        // Parse UUID from string - messaging::ChannelId expects a Uuid
        let uuid = Uuid::parse_str(channel_id)
            .map_err(|e| format!("Invalid channel ID '{}': {}", channel_id, e))?;
        let channel_id_obj = ChannelId(uuid);

        // Get messages via MessagingService
        self.messaging
            .get_channel_messages(channel_id_obj, limit, None)
            .await
            .map_err(|e| format!("Failed to get channel messages: {}", e))
    }

    /// Detect local network endpoint for P2P connectivity
    /// Prefers LAN addresses for local testing, falls back to localhost
    /// Uses port 0 to let the OS choose a random available port
    #[allow(dead_code)]
    async fn detect_local_endpoint() -> Option<NetworkAddress> {
        use local_ip_address::local_ip;

        // Use port 0 to let the OS choose a random available port
        let port = 0u16;

        // Try to get local IP address (LAN)
        if let Ok(ip) = local_ip() {
            match ip {
                IpAddr::V4(ipv4) => {
                    let addr = NetworkAddress::from_ipv4(ipv4, port);
                    tracing::debug!("Detected IPv4 endpoint (port 0 - OS will choose): {}", addr);
                    return Some(addr);
                }
                IpAddr::V6(ipv6) => {
                    // Convert to SocketAddr and then to NetworkAddress
                    let socket_addr = SocketAddr::new(IpAddr::V6(ipv6), port);
                    let addr = NetworkAddress::try_from(socket_addr).ok()?;
                    tracing::debug!("Detected IPv6 endpoint (port 0 - OS will choose): {}", addr);
                    return Some(addr);
                }
            }
        }

        // Fallback to localhost with port 0
        tracing::warn!("Could not detect LAN IP, using localhost (port 0 - OS will choose)");
        let localhost = NetworkAddress::from_ipv4(Ipv4Addr::LOCALHOST, 0);
        Some(localhost)
    }

    /// Get the local endpoint four-word address
    pub async fn get_local_endpoint_four_words(&self) -> Option<String> {
        // Get listen addresses from MessagingService
        let listen_addrs = self.messaging.listen_addrs().await;

        if let Some(addr) = listen_addrs.first() {
            // Convert SocketAddr to NetworkAddress
            let network_addr = NetworkAddress::try_from(*addr).ok()?;
            return network_addr.four_words().map(|s| s.to_string());
        }

        // Fallback to stored endpoint
        self.local_endpoint
            .as_ref()
            .and_then(|addr| addr.four_words().map(|words| words.to_string()))
    }

    /// Get the local endpoint as a socket address
    pub async fn get_local_endpoint_socket(&self) -> Option<SocketAddr> {
        // Get listen addresses from MessagingService
        let listen_addrs = self.messaging.listen_addrs().await;

        if let Some(addr) = listen_addrs.first() {
            return Some(*addr);
        }

        // Fallback to stored endpoint
        self.local_endpoint
            .as_ref()
            .and_then(|addr| addr.to_string().parse().ok())
    }

    /// Parse an address string into NetworkAddress
    #[allow(dead_code)]
    fn parse_address(addr_str: &str) -> Result<NetworkAddress, String> {
        // Address format could be "1.2.3.4:port" or "[::1]:port" or similar
        let socket_addr: SocketAddr = addr_str
            .parse()
            .map_err(|e| format!("Failed to parse address '{}': {}", addr_str, e))?;

        Ok(NetworkAddress::try_from(socket_addr)
            .map_err(|e| format!("Failed to convert to NetworkAddress: {}", e))?)
    }

    /// Create and start a P2P node
    /// Returns the node and the actual bound address (with OS-assigned port)
    #[allow(dead_code)]
    async fn create_p2p_node(
        endpoint: &NetworkAddress,
    ) -> Result<(P2PNode, NetworkAddress), String> {
        tracing::debug!("Creating P2P node with endpoint: {}", endpoint);

        // Convert NetworkAddress to string for listen_on
        let addr_str = endpoint.to_string();

        // Build P2P node with the detected endpoint
        let node = P2PNode::builder()
            .listen_on(&addr_str)
            .build()
            .await
            .map_err(|e| format!("Failed to build P2P node: {}", e))?;

        // Start the node in the background
        node.start()
            .await
            .map_err(|e| format!("Failed to start P2P node: {}", e))?;

        // Get the actual bound address from the node
        let actual_addr_str = node
            .local_addr()
            .ok_or_else(|| "Node has no local address".to_string())?;

        tracing::info!("P2P node bound to actual address: {}", actual_addr_str);

        // Parse the address string into NetworkAddress
        let actual_addr = Self::parse_address(&actual_addr_str)?;

        Ok((node, actual_addr))
    }

    /// Connect to a peer via four-word address
    pub async fn connect_to_peer(&self, four_words: &str) -> Result<(), String> {
        use four_word_networking::FourWordAdaptiveEncoder;

        // Decode four-word address directly to socket address string
        let encoder = FourWordAdaptiveEncoder::new()
            .map_err(|e| format!("Failed to create encoder: {}", e))?;

        // Convert hyphens to spaces - the decoder expects space-separated words
        let space_separated = four_words.replace('-', " ");

        let socket_addr_str = encoder.decode(&space_separated).map_err(|e| {
            format!(
                "Failed to decode four-word address '{}': {}",
                space_separated, e
            )
        })?;

        tracing::info!("Connecting to peer: {} ({})", four_words, socket_addr_str);

        // Connect via MessagingService with plain socket address string
        self.messaging
            .connect_peer(&socket_addr_str)
            .await
            .map_err(|e| format!("Failed to connect to peer: {}", e))?;

        tracing::info!("Successfully connected to peer: {}", four_words);
        Ok(())
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Vec<String> {
        self.messaging.connected_peers().await
    }

    /// Get number of connected peers
    pub async fn get_peer_count(&self) -> usize {
        self.messaging.peer_count().await
    }

    /// Check if P2P node is running
    pub async fn is_p2p_running(&self) -> bool {
        self.messaging.is_running().await
    }

    /// Decode four-word address to NetworkAddress
    fn decode_four_words(four_words: &str) -> Result<NetworkAddress, String> {
        use four_word_networking::FourWordAdaptiveEncoder;

        let encoder = FourWordAdaptiveEncoder::new()
            .map_err(|e| format!("Failed to create encoder: {}", e))?;

        // Convert hyphens to spaces - the decoder expects space-separated words
        let space_separated = four_words.replace('-', " ");

        let decoded = encoder.decode(&space_separated).map_err(|e| {
            format!(
                "Failed to decode four-word address '{}': {}",
                space_separated, e
            )
        })?;

        // Parse the decoded string as NetworkAddress
        // The decoded format should be "ip:port"
        decoded
            .parse::<NetworkAddress>()
            .map_err(|e| format!("Failed to parse network address '{}': {}", decoded, e))
    }
}

/// Simple identity information structure
pub struct IdentityInfo {
    pub four_words: String,
    pub display_name: String,
}
// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

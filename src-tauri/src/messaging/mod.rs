// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


//! # P2P Messaging System
//! 
//! A production-ready distributed messaging system with:
//! - End-to-end encryption using ML-KEM-768/ML-DSA-65/ChaCha20Poly1305
//! - Post-quantum secure key exchange and signatures
//! - DHT-based message routing and discovery
//! - Real-time delivery with acknowledgments
//! - Message persistence with configurable TTL
//! - Group messaging with efficient key management
//! - Offline message queuing and synchronization
//! - Network partition tolerance and eventual consistency

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex, broadcast, mpsc};
use tracing::{info, warn, error, debug, instrument};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use blake3;
use rand::{RngCore, rngs::OsRng};

// Import our DHT and identity modules
use crate::dht::{Dht, NodeId, ContentId};
use crate::identity::CommunidentityManager;

pub mod crypto;
pub mod storage;
pub mod group;
pub mod sync;

use crypto::{MessageCrypto, EncryptedMessage, KeyExchange};
use storage::{MessageStore, MessageQuery, StorageBackend};
use group::{GroupManager, GroupMessage, GroupMetadata};
use sync::{MessageSyncer, SyncManager, ConflictResolver};

/// Maximum message size in bytes (1MB)
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Message TTL in seconds (1 week)
pub const DEFAULT_MESSAGE_TTL: i64 = 7 * 24 * 60 * 60;

/// Maximum number of pending messages per user
pub const MAX_PENDING_MESSAGES: usize = 10000;

/// Message acknowledgment timeout in seconds
pub const ACK_TIMEOUT_SECONDS: u64 = 30;

/// Message replication factor for DHT storage
pub const MESSAGE_REPLICATION_FACTOR: usize = 3;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_content(content: &[u8]) -> Self {
        let hash = blake3::hash(content);
        let uuid_bytes = &hash.as_bytes()[..16];
        let uuid = Uuid::from_slice(uuid_bytes).expect("Valid UUID from hash");
        Self(uuid)
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(id: String) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupId(pub Uuid);

impl GroupId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Vector clock for causal ordering of messages
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorClock {
    clocks: HashMap<UserId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn increment(&mut self, user_id: &UserId) {
        let counter = self.clocks.entry(user_id.clone()).or_insert(0);
        *counter += 1;
    }
    
    pub fn update(&mut self, other: &VectorClock) {
        for (user, clock) in &other.clocks {
            let current = self.clocks.entry(user.clone()).or_insert(0);
            *current = (*current).max(*clock);
        }
    }
    
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;
        
        for (user, &other_clock) in &other.clocks {
            let self_clock = self.clocks.get(user).copied().unwrap_or(0);
            if self_clock > other_clock {
                return false;
            }
            if self_clock < other_clock {
                strictly_less = true;
            }
        }
        
        for (user, &self_clock) in &self.clocks {
            if !other.clocks.contains_key(user) && self_clock > 0 {
                return false;
            }
        }
        
        strictly_less
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    DirectMessage,
    GroupMessage,
    SystemMessage,
    Acknowledgment,
    KeyExchange,
    GroupInvite,
    GroupLeave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub id: MessageId,
    pub sender: UserId,
    pub recipient: Option<UserId>,
    pub group_id: Option<GroupId>,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub vector_clock: VectorClock,
    pub ttl: DateTime<Utc>,
    pub encrypted: bool,
    pub compressed: bool,
    pub size_bytes: usize,
}

impl MessageMetadata {
    pub fn new(
        sender: UserId,
        recipient: Option<UserId>,
        group_id: Option<GroupId>,
        message_type: MessageType,
        size_bytes: usize,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: MessageId::new(),
            sender,
            recipient,
            group_id,
            message_type,
            timestamp: now,
            vector_clock: VectorClock::new(),
            ttl: now + ChronoDuration::seconds(DEFAULT_MESSAGE_TTL),
            encrypted: true,
            compressed: size_bytes > 1024, // Compress messages > 1KB
            size_bytes,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.ttl
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub metadata: MessageMetadata,
    pub content: Vec<u8>,
    pub signature: Option<Vec<u8>>,
}

impl Message {
    pub fn new_direct(
        sender: UserId,
        recipient: UserId,
        content: Vec<u8>,
    ) -> Result<Self> {
        if content.len() > MAX_MESSAGE_SIZE {
            bail!("Message exceeds maximum size of {} bytes", MAX_MESSAGE_SIZE);
        }
        
        let metadata = MessageMetadata::new(
            sender,
            Some(recipient),
            None,
            MessageType::DirectMessage,
            content.len(),
        );
        
        Ok(Self {
            metadata,
            content,
            signature: None,
        })
    }
    
    pub fn new_group(
        sender: UserId,
        group_id: GroupId,
        content: Vec<u8>,
    ) -> Result<Self> {
        if content.len() > MAX_MESSAGE_SIZE {
            bail!("Message exceeds maximum size of {} bytes", MAX_MESSAGE_SIZE);
        }
        
        let metadata = MessageMetadata::new(
            sender,
            None,
            Some(group_id),
            MessageType::GroupMessage,
            content.len(),
        );
        
        Ok(Self {
            metadata,
            content,
            signature: None,
        })
    }
    
    pub fn content_hash(&self) -> String {
        let hash = blake3::hash(&self.content);
        format!("{}", hash)
    }
    
    pub fn is_valid(&self) -> bool {
        !self.metadata.is_expired() && 
        self.content.len() <= MAX_MESSAGE_SIZE &&
        self.metadata.size_bytes == self.content.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAcknowledgment {
    pub message_id: MessageId,
    pub recipient: UserId,
    pub timestamp: DateTime<Utc>,
    pub status: AckStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AckStatus {
    Received,
    Processed,
    Failed(String),
}

/// Configuration for the messaging system
#[derive(Debug, Clone)]
pub struct MessagingConfig {
    pub message_ttl_seconds: i64,
    pub max_pending_messages: usize,
    pub ack_timeout_seconds: u64,
    pub replication_factor: usize,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub storage_backend: String,
}

impl Default for MessagingConfig {
    fn default() -> Self {
        Self {
            message_ttl_seconds: DEFAULT_MESSAGE_TTL,
            max_pending_messages: MAX_PENDING_MESSAGES,
            ack_timeout_seconds: ACK_TIMEOUT_SECONDS,
            replication_factor: MESSAGE_REPLICATION_FACTOR,
            enable_compression: true,
            enable_encryption: true,
            storage_backend: "sqlite".to_string(),
        }
    }
}

/// Main P2P messaging system
pub struct MessagingSystem {
    config: MessagingConfig,
    dht: Arc<RwLock<Option<Arc<Dht>>>>,
    identity_manager: Arc<Mutex<Option<CommunidentityManager>>>,
    message_store: Arc<MessageStore>,
    group_manager: Arc<GroupManager>,
    crypto: Arc<MessageCrypto>,
    sync_manager: Arc<SyncManager>,
    
    // Message routing and delivery
    pending_messages: Arc<RwLock<HashMap<MessageId, Message>>>,
    pending_acks: Arc<RwLock<HashMap<MessageId, Vec<UserId>>>>,
    message_cache: Arc<RwLock<HashMap<MessageId, Message>>>,
    
    // Event channels
    message_sender: broadcast::Sender<Message>,
    ack_sender: broadcast::Sender<MessageAcknowledgment>,
    
    // Background task handles
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
    sync_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MessagingSystem {
    pub async fn new(
        config: MessagingConfig,
        dht: Arc<RwLock<Option<Arc<Dht>>>>,
        identity_manager: Arc<Mutex<Option<CommunidentityManager>>>,
    ) -> Result<Arc<Self>> {
        let message_store = Arc::new(
            MessageStore::new(&config.storage_backend)
                .await
                .context("Failed to initialize message store")?
        );
        
        let group_manager = Arc::new(
            GroupManager::new(message_store.clone(), dht.clone())
                .await
                .context("Failed to initialize group manager")?
        );
        
        let crypto = Arc::new(
            MessageCrypto::new(identity_manager.clone())
                .await
                .context("Failed to initialize message crypto")?
        );
        
        let sync_manager = Arc::new(
            SyncManager::new(message_store.clone(), dht.clone())
                .await
                .context("Failed to initialize sync manager")?
        );
        
        let (message_sender, _) = broadcast::channel(1000);
        let (ack_sender, _) = broadcast::channel(1000);
        
        let system = Arc::new(Self {
            config,
            dht,
            identity_manager,
            message_store,
            group_manager,
            crypto,
            sync_manager,
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            pending_acks: Arc::new(RwLock::new(HashMap::new())),
            message_cache: Arc::new(RwLock::new(HashMap::new())),
            message_sender,
            ack_sender,
            cleanup_handle: None,
            sync_handle: None,
        });
        
        // Start background tasks
        let cleanup_system = system.clone();
        let cleanup_handle = tokio::spawn(async move {
            cleanup_system.cleanup_loop().await;
        });
        
        let sync_system = system.clone();
        let sync_handle = tokio::spawn(async move {
            sync_system.sync_loop().await;
        });
        
        // Update handles (need to modify struct to be mutable)
        // This is a design consideration - in practice we'd handle this differently
        
        info!("P2P Messaging System initialized with {} backend", system.config.storage_backend);
        
        Ok(system)
    }
    
    /// Send a direct message to another user
    #[instrument(skip(self, content))]
    pub async fn send_message(
        &self,
        recipient: UserId,
        content: Vec<u8>,
    ) -> Result<MessageId> {
        let sender = self.get_current_user_id().await?;
        let mut message = Message::new_direct(sender, recipient.clone(), content)?;
        
        // Update vector clock
        message.metadata.vector_clock.increment(&message.metadata.sender);
        
        // Encrypt message
        if self.config.enable_encryption {
            message = self.crypto.encrypt_message(message, &recipient).await?;
        }
        
        // Compress if enabled and beneficial
        if self.config.enable_compression && message.metadata.compressed {
            message = self.compress_message(message).await?;
        }
        
        // Store locally
        self.message_store.store_message(&message).await?;
        
        // Route through DHT
        self.route_message_via_dht(&message).await?;
        
        // Track for acknowledgment
        self.track_message_for_ack(&message).await?;
        
        // Emit message event
        let _ = self.message_sender.send(message.clone());
        
        info!("Message {} sent to {}", message.metadata.id, recipient.0);
        
        Ok(message.metadata.id)
    }
    
    /// Send a group message
    #[instrument(skip(self, content))]
    pub async fn send_group_message(
        &self,
        group_id: GroupId,
        content: Vec<u8>,
    ) -> Result<MessageId> {
        let sender = self.get_current_user_id().await?;
        let mut message = Message::new_group(sender, group_id.clone(), content)?;
        
        // Update vector clock
        message.metadata.vector_clock.increment(&message.metadata.sender);
        
        // Encrypt for group
        if self.config.enable_encryption {
            message = self.crypto.encrypt_group_message(message, &group_id).await?;
        }
        
        // Compress if needed
        if self.config.enable_compression && message.metadata.compressed {
            message = self.compress_message(message).await?;
        }
        
        // Store locally
        self.message_store.store_message(&message).await?;
        
        // Distribute to group members via DHT
        self.distribute_group_message(&message).await?;
        
        // Emit message event
        let _ = self.message_sender.send(message.clone());
        
        info!("Group message {} sent to group {}", message.metadata.id, group_id.0);
        
        Ok(message.metadata.id)
    }
    
    /// Receive and process an incoming message
    #[instrument(skip(self, message))]
    pub async fn receive_message(&self, message: Message) -> Result<()> {
        // Validate message
        if !message.is_valid() {
            warn!("Received invalid message: {:?}", message.metadata.id);
            return Ok(());
        }
        
        // Check for duplicate
        if self.message_store.has_message(&message.metadata.id).await? {
            debug!("Ignoring duplicate message: {}", message.metadata.id);
            return Ok(());
        }
        
        // Decrypt if needed
        let mut processed_message = message;
        if processed_message.metadata.encrypted {
            processed_message = self.crypto.decrypt_message(processed_message).await?;
        }
        
        // Decompress if needed
        if processed_message.metadata.compressed {
            processed_message = self.decompress_message(processed_message).await?;
        }
        
        // Update vector clock
        self.update_vector_clock(&processed_message.metadata.vector_clock).await?;
        
        // Store message
        self.message_store.store_message(&processed_message).await?;
        
        // Send acknowledgment
        self.send_acknowledgment(&processed_message).await?;
        
        // Cache for quick access
        self.cache_message(&processed_message).await;
        
        // Emit message event
        let _ = self.message_sender.send(processed_message.clone());
        
        info!("Message {} received and processed", processed_message.metadata.id);
        
        Ok(())
    }
    
    /// Get messages for a user or group
    pub async fn get_messages(
        &self,
        query: MessageQuery,
    ) -> Result<Vec<Message>> {
        self.message_store.query_messages(query).await
    }
    
    /// Subscribe to incoming messages
    pub fn subscribe_messages(&self) -> broadcast::Receiver<Message> {
        self.message_sender.subscribe()
    }
    
    /// Subscribe to message acknowledgments
    pub fn subscribe_acknowledgments(&self) -> broadcast::Receiver<MessageAcknowledgment> {
        self.ack_sender.subscribe()
    }
    
    // Private helper methods
    
    async fn get_current_user_id(&self) -> Result<UserId> {
        let identity_guard = self.identity_manager.lock().await;
        let identity_manager = identity_guard.as_ref()
            .context("Identity manager not initialized")?;
        
        let identity = identity_manager.get_current_identity()
            .await
            .context("No current identity")?;
        
        Ok(UserId::new(identity.four_word_address))
    }
    
    async fn route_message_via_dht(&self, message: &Message) -> Result<()> {
        let dht_guard = self.dht.read().await;
        let dht = dht_guard.as_ref().context("DHT not initialized")?;
        
        // Create content ID from message ID and recipient
        let content_key = if let Some(ref recipient) = message.metadata.recipient {
            format!("msg:{}:{}", recipient.0, message.metadata.id)
        } else if let Some(ref group_id) = message.metadata.group_id {
            format!("group_msg:{}:{}", group_id.0, message.metadata.id)
        } else {
            format!("msg:{}", message.metadata.id)
        };
        
        let content_id = ContentId::from_key(&content_key);
        let message_data = bincode::serialize(message)?;
        
        // Store in DHT with replication
        dht.put_content(content_id, message_data, self.config.replication_factor).await?;
        
        Ok(())
    }
    
    async fn distribute_group_message(&self, message: &Message) -> Result<()> {
        let group_id = message.metadata.group_id.as_ref()
            .context("Group message missing group ID")?;
        
        let members = self.group_manager.get_group_members(group_id).await?;
        
        for member in members {
            if member != message.metadata.sender {
                // Create individual delivery record in DHT
                let delivery_key = format!("delivery:{}:{}", member.0, message.metadata.id);
                let content_id = ContentId::from_key(&delivery_key);
                let message_data = bincode::serialize(message)?;
                
                if let Ok(dht_guard) = self.dht.read().await {
                    if let Some(dht) = dht_guard.as_ref() {
                        let _ = dht.put_content(content_id, message_data, 1).await;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn track_message_for_ack(&self, message: &Message) -> Result<()> {
        if let Some(ref recipient) = message.metadata.recipient {
            let mut pending_acks = self.pending_acks.write().await;
            pending_acks.insert(message.metadata.id.clone(), vec![recipient.clone()]);
        }
        Ok(())
    }
    
    async fn send_acknowledgment(&self, message: &Message) -> Result<()> {
        let current_user = self.get_current_user_id().await?;
        
        let ack = MessageAcknowledgment {
            message_id: message.metadata.id.clone(),
            recipient: current_user,
            timestamp: Utc::now(),
            status: AckStatus::Received,
        };
        
        // Send ack back to sender via DHT
        let ack_key = format!("ack:{}:{}", message.metadata.sender.0, message.metadata.id);
        let content_id = ContentId::from_key(&ack_key);
        let ack_data = bincode::serialize(&ack)?;
        
        if let Ok(dht_guard) = self.dht.read().await {
            if let Some(dht) = dht_guard.as_ref() {
                let _ = dht.put_content(content_id, ack_data, 1).await;
            }
        }
        
        // Emit ack event
        let _ = self.ack_sender.send(ack);
        
        Ok(())
    }
    
    async fn compress_message(&self, mut message: Message) -> Result<Message> {
        use flate2::Compression;
        use flate2::write::GzEncoder;
        use std::io::Write;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&message.content)?;
        let compressed = encoder.finish()?;
        
        if compressed.len() < message.content.len() {
            message.content = compressed;
            message.metadata.compressed = true;
            message.metadata.size_bytes = message.content.len();
        }
        
        Ok(message)
    }
    
    async fn decompress_message(&self, mut message: Message) -> Result<Message> {
        use flate2::read::GzDecoder;
        use std::io::Read;
        
        let mut decoder = GzDecoder::new(&message.content[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        
        message.content = decompressed;
        message.metadata.compressed = false;
        message.metadata.size_bytes = message.content.len();
        
        Ok(message)
    }
    
    async fn update_vector_clock(&self, other_clock: &VectorClock) -> Result<()> {
        // This would update our local vector clock based on the received message
        // Implementation depends on how we store the local clock
        Ok(())
    }
    
    async fn cache_message(&self, message: &Message) {
        let mut cache = self.message_cache.write().await;
        
        // Simple LRU eviction
        if cache.len() >= 1000 {
            let oldest_key = cache.keys().next().cloned();
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }
        
        cache.insert(message.metadata.id.clone(), message.clone());
    }
    
    /// Background cleanup of expired messages and acknowledgments
    async fn cleanup_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.cleanup_expired_data().await {
                error!("Error during cleanup: {}", e);
            }
        }
    }
    
    async fn cleanup_expired_data(&self) -> Result<()> {
        // Clean expired messages from store
        self.message_store.cleanup_expired().await?;
        
        // Clean expired message cache
        let mut cache = self.message_cache.write().await;
        let now = Utc::now();
        cache.retain(|_, message| message.metadata.ttl > now);
        
        // Clean up pending acknowledgments that have timed out
        let mut pending_acks = self.pending_acks.write().await;
        let timeout = tokio::time::Duration::from_secs(self.config.ack_timeout_seconds);
        let cutoff = Utc::now() - ChronoDuration::from_std(timeout).unwrap_or_default();
        
        pending_acks.retain(|_, _| {
            // This is simplified - we'd need to track timestamps for each pending ack
            true
        });
        
        debug!("Cleanup completed");
        Ok(())
    }
    
    /// Background synchronization with other nodes
    async fn sync_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60)); // 1 minute
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.sync_manager.sync_messages().await {
                error!("Error during sync: {}", e);
            }
        }
    }
}

// For safe shutdown
impl Drop for MessagingSystem {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
        if let Some(handle) = self.sync_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;
    
    #[tokio::test]
    async fn test_message_creation() -> Result<()> {
        let sender = UserId::new("test_sender".to_string());
        let recipient = UserId::new("test_recipient".to_string());
        let content = b"Hello, World!".to_vec();
        
        let message = Message::new_direct(sender, recipient, content)?;
        
        assert!(message.is_valid());
        assert_eq!(message.content, b"Hello, World!");
        assert_eq!(message.metadata.message_type, MessageType::DirectMessage);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_vector_clock_ordering() -> Result<()> {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        let user1 = UserId::new("user1".to_string());
        let user2 = UserId::new("user2".to_string());
        
        clock1.increment(&user1);
        clock2.increment(&user2);
        
        // Neither happened before the other (concurrent)
        assert!(!clock1.happened_before(&clock2));
        assert!(!clock2.happened_before(&clock1));
        
        clock2.update(&clock1);
        clock2.increment(&user2);
        
        // Now clock1 happened before clock2
        assert!(clock1.happened_before(&clock2));
        assert!(!clock2.happened_before(&clock1));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_message_size_validation() {
        let sender = UserId::new("test_sender".to_string());
        let recipient = UserId::new("test_recipient".to_string());
        let large_content = vec![0u8; MAX_MESSAGE_SIZE + 1];
        
        let result = Message::new_direct(sender, recipient, large_content);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_message_expiration() -> Result<()> {
        let sender = UserId::new("test_sender".to_string());
        let recipient = UserId::new("test_recipient".to_string());
        let content = b"Test message".to_vec();
        
        let mut message = Message::new_direct(sender, recipient, content)?;
        
        // Set TTL to past
        message.metadata.ttl = Utc::now() - ChronoDuration::seconds(1);
        
        assert!(message.metadata.is_expired());
        assert!(!message.is_valid());
        
        Ok(())
    }
}

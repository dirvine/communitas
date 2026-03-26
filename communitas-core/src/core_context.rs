// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Core Context - Centralized application state for Communitas
//!
//! This module provides the main application context that coordinates:
//! - User identity and profiles (via x0x daemon)
//! - P2P networking (via x0x daemon)
//! - Message synchronization
//! - Storage and persistence
//!
//! **Architecture Note**: Networking and identity are delegated to the x0x daemon,
//! accessed via `communitas-x0x-client`.

use crate::disk_service::EntityDiskService;
use crate::encrypted_storage::{
    identity_keys_exist, load_identity_keys, store_identity_keys, vault_dir_from_root,
};
use crate::keystore::Keystore;
use crate::message_sync::MessageSyncService;
use crate::types::{DeviceType, UserProfile};
use blake3;
use communitas_kanban::KanbanService;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Centralized context for the Communitas application
///
/// Networking and cryptographic identity are delegated to the x0x daemon,
/// accessed through `communitas_x0x_client::X0xClient`.
///
/// **Lifecycle**:
/// 1. Initialize with user profile (four-word ID, display name, device)
/// 2. Start networking via x0x daemon (optional, can run in local mode)
/// 3. Initialize message sync service
/// 4. Ready for operations
pub struct CoreContext {
    /// User profile with identity and device info
    pub profile: UserProfile,

    /// Four-word user identity
    pub four_words: String,

    /// x0x agent identity (set once networking starts)
    pub agent_id: Option<String>,

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

    /// x0x client for daemon communication (networking, identity, crypto)
    pub x0x: Arc<communitas_x0x_client::X0xClient>,

    /// Per-entity virtual disk service (Private, Public, Shared disks)
    pub disk_service: Arc<EntityDiskService>,

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
            .field("agent_id", &self.agent_id)
            .field("display_name", &self.display_name)
            .field("device_name", &self.device_name)
            .field("crdt_manager", &"<active>")
            .field("entity_service", &"<active>")
            .field("message_service", &"<active>")
            .field("doc_replicator", &"<active>")
            .field("x0x", &"<client>")
            .field("disk_service", &"<active>")
            .field("kanban_service", &"<active>")
            .field("invite_service", &"<active>")
            .finish()
    }
}

impl CoreContext {
    /// Initialize a new CoreContext from a four-word identity
    ///
    /// This creates a new profile with local storage for CRDT, entities, messages,
    /// and documents. Cryptographic identity is managed by the x0x daemon.
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
        info!("CoreContext::initialize: START for {}", four_words);

        // Validate four-word format
        let words: Vec<&str> = four_words.split('-').collect();
        if words.len() != 4 {
            return Err(format!(
                "Invalid four-word format: expected 4 words, got {}",
                words.len()
            ));
        }
        info!("CoreContext::initialize: four-word format validated");

        let vault_dir = vault_dir_from_root(&storage_dir);

        // Try to load existing key material from vault (for local vault encryption)
        if identity_keys_exist(&vault_dir, &four_words).await {
            match load_identity_keys(&vault_dir, &four_words).await {
                Ok(_keys) => {
                    info!("Loaded identity keys from vault for '{}'", four_words);
                }
                Err(err) => {
                    warn!(
                        "Failed to load identity keys from vault for '{}': {}",
                        four_words, err
                    );
                }
            }
        } else {
            // Try legacy keyring for migration
            info!("CoreContext::initialize: checking keystore for legacy keys");
            let keystore = Keystore::new();
            let id_hex = blake3::hash(four_words.as_bytes()).to_hex().to_string();

            if let Ok((pk_bytes, sk_bytes)) = keystore.load_mldsa_keys(&id_hex) {
                info!(
                    "Migrating legacy ML-DSA keys for identity '{}' to vault",
                    four_words
                );
                if let Err(err) = store_identity_keys(
                    &vault_dir,
                    &four_words,
                    &display_name,
                    &pk_bytes,
                    &sk_bytes,
                )
                .await
                {
                    warn!("Failed to migrate identity keys to vault: {err}");
                }
            }
        }

        // Create storage directory if it doesn't exist
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir).map_err(|e| {
                format!(
                    "Failed to create storage directory {:?}: {}",
                    storage_dir, e
                )
            })?;
        }

        // Derive a deterministic pubkey placeholder for UserProfile from four_words
        let pubkey_hash = blake3::hash(four_words.as_bytes());
        let pubkey_array: [u8; 32] = *pubkey_hash.as_bytes();

        // Create user profile
        info!("CoreContext::initialize: creating UserProfile");
        let profile = UserProfile::new(
            four_words.clone(),
            display_name.clone(),
            pubkey_array,
            device_type,
            storage_dir.clone(),
        );
        info!("CoreContext::initialize: UserProfile created");

        // Initialize CRDT manager for persistent storage
        info!("CoreContext::initialize: creating CrdtManager");
        let crdt_manager = Arc::new(
            crate::CrdtManager::new(&storage_dir.join("crdt.db"))
                .await
                .map_err(|e| format!("Failed to initialize CrdtManager: {}", e))?,
        );
        info!("CoreContext::initialize: CrdtManager created");

        // Start background compaction task to prevent unbounded disk growth from tombstones
        crdt_manager.start_compaction_task().await;
        info!("CoreContext::initialize: CRDT compaction task started");

        // Initialize entity service for managing groups, channels, and members
        info!("CoreContext::initialize: creating EntityService");
        let entity_service = Arc::new(crate::EntityService::new(crdt_manager.clone()));

        // Initialize unified message service
        info!("CoreContext::initialize: creating MessageService");
        let message_service = Arc::new(crate::MessageService::new(four_words.clone()));

        // Initialize legacy message sync service (for backward compatibility)
        info!("CoreContext::initialize: creating MessageSyncService");
        let message_sync = Arc::new(MessageSyncService::new(four_words.clone()));

        // Initialize document replicator with dual storage enabled (Sprint 3.2)
        info!("CoreContext::initialize: creating DocReplicator");
        let doc_config = crate::doc_replicator::DocReplicatorConfig {
            files_storage_enabled: true,
            web_storage_enabled: true,
        };
        let doc_replicator = Arc::new(
            crate::doc_replicator::DocReplicator::new(doc_config)
                .await
                .map_err(|e| format!("Failed to initialize DocReplicator: {}", e))?,
        );
        info!("CoreContext::initialize: DocReplicator created");

        // Initialize per-entity virtual disk service
        info!("CoreContext::initialize: creating EntityDiskService");
        let disk_root = storage_dir.join("disks");
        let disk_service = Arc::new(
            EntityDiskService::new(&disk_root)
                .await
                .map_err(|e| format!("Failed to initialize EntityDiskService: {}", e))?,
        );
        info!("CoreContext::initialize: EntityDiskService created");

        // Initialize Kanban service for project management boards
        info!("CoreContext::initialize: creating KanbanService");
        let kanban_service = Arc::new(KanbanService::new(four_words.clone()));

        // Initialize Invite service for cross-organization collaboration
        info!("CoreContext::initialize: creating InviteService");
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
            four_words,
            agent_id: None,
            display_name,
            device_name,
            crdt_manager,
            entity_service,
            message_service,
            message_sync,
            doc_replicator,
            x0x: Arc::new(communitas_x0x_client::X0xClient::new()),
            disk_service,
            kanban_service,
            invite_service,
        })
    }

    /// Start networking via the x0x daemon
    ///
    /// Ensures the x0x daemon is running and retrieves the agent identity.
    ///
    /// # Arguments
    /// * `_preferred_port` - Ignored; the x0x daemon manages its own ports
    ///
    /// # Returns
    /// The x0x agent ID for this node
    pub async fn start_networking(
        &mut self,
        _preferred_port: Option<u16>,
    ) -> Result<String, String> {
        info!("Starting x0x networking for {}", self.four_words);

        let dm = communitas_x0x_client::DaemonManager::new();
        dm.ensure_running()
            .await
            .map_err(|e| format!("Failed to start x0x daemon: {}", e))?;

        let identity = self
            .x0x
            .agent()
            .await
            .map_err(|e| format!("Failed to get x0x agent identity: {}", e))?;

        info!("x0x networking active: agent_id={}", identity.agent_id);

        self.agent_id = Some(identity.agent_id.clone());

        Ok(identity.agent_id)
    }

    /// Stop networking gracefully
    ///
    /// The x0x daemon runs independently so there is nothing to tear down here.
    pub async fn stop_networking(&mut self) -> Result<(), String> {
        // x0x daemon runs independently - nothing to stop
        Ok(())
    }

    /// Connect to a peer using their agent ID via x0x daemon
    ///
    /// # Arguments
    /// * `agent_id` - The peer's x0x agent ID
    ///
    /// # Returns
    /// Success if the connection request was accepted by the daemon
    pub async fn connect_to_peer(&self, agent_id: &str) -> Result<(), String> {
        info!("Connecting to peer {} via x0x daemon", agent_id);
        self.x0x
            .connect_agent(agent_id)
            .await
            .map_err(|e| format!("Failed to connect to peer {}: {}", agent_id, e))
    }

    /// Send a channel message and publish it via x0x if networking is active
    ///
    /// This method:
    /// 1. Stores the message locally via CRDT
    /// 2. If x0x networking is active, publishes to the channel topic
    ///
    /// # Arguments
    /// * `channel_id` - The channel to send to
    /// * `content_text` - The message text
    /// * `reply_to_id` - Optional parent message for threading
    ///
    /// # Returns
    /// The message ID on success
    pub async fn send_and_publish_channel_message(
        &self,
        channel_id: String,
        content_text: String,
        reply_to_id: Option<String>,
    ) -> Result<String, String> {
        use crate::crdt::{EntityType, MessageContent};

        // Create message content
        let content = MessageContent {
            text: content_text,
            author: self.four_words.clone(),
            attachments: None,
        };

        // Store locally via message_service
        let message = if let Some(reply_to) = reply_to_id {
            self.message_service
                .send_message(
                    channel_id.clone(),
                    EntityType::Channel,
                    content,
                    Some(reply_to),
                )
                .await
                .map_err(|e| format!("Failed to send message: {}", e))?
        } else {
            self.message_service
                .send_message(channel_id.clone(), EntityType::Channel, content, None)
                .await
                .map_err(|e| format!("Failed to send message: {}", e))?
        };

        let message_id = message.metadata.id.clone();

        // If x0x networking is active, publish to the network
        if self.agent_id.is_some() {
            // Serialize message to JSON bytes
            let message_bytes = serde_json::to_vec(&message)
                .map_err(|e| format!("Failed to serialize message: {}", e))?;

            let topic = format!("entity:{}", channel_id);
            if let Err(e) = self.x0x.publish(&topic, &message_bytes).await {
                warn!("Failed to publish message via x0x: {}", e);
                // Don't fail - message is stored locally, sync will catch up
            } else {
                info!(
                    "Message {} published via x0x for channel {}",
                    message_id, channel_id
                );
            }
        } else {
            info!(
                "x0x not active - message {} stored locally only",
                message_id
            );
        }

        Ok(message_id)
    }

    /// Request a message sync for an entity via x0x.
    ///
    /// Currently a no-op since sync is handled by the x0x daemon.
    pub async fn request_entity_message_sync(&self, _entity_id: &str) -> Result<(), String> {
        // Message sync is handled by x0x daemon subscriptions
        Ok(())
    }

    /// Get the storage directory for this profile
    pub fn storage_dir(&self) -> &PathBuf {
        &self.profile.storage_dir
    }

    /// Check if networking is active
    pub fn is_networking_active(&self) -> bool {
        self.agent_id.is_some()
    }

    /// Get the x0x agent ID (if networking is active)
    pub fn get_agent_id(&self) -> Option<&str> {
        self.agent_id.as_deref()
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
        assert!(context.get_agent_id().is_none());
    }
}

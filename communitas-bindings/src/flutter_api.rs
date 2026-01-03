//! Flutter Rust Bridge API
//!
//! This module provides the Flutter/Dart bindings using flutter_rust_bridge.
//! It exposes core functionality via the Command/Query/Event architecture.
//!
//! Build with: `cargo build -p communitas-bindings --features flutter-bindings`
//!
//! # Architecture
//!
//! This module wraps `CommunitasApp` (the headless core) and provides a clean FFI
//! surface for Flutter. All mutations go through Commands, all reads through Queries.
//!
//! ```text
//! Flutter UI -> flutter_api.rs -> CommunitasApp -> CoreContext
//! ```

#![allow(dead_code)] // API surface may not be used by tests
#![allow(unexpected_cfgs)] // Allow flutter_rust_bridge cfg checks

use flutter_rust_bridge::frb;

use communitas_core::app::CommunitasApp;
use communitas_core::auth_service::{AuthService, SessionInfo};
use communitas_core::command::{Command, EntityResponse, Event, Query, QueryResponse};
use communitas_core::crdt::EntityType;
use communitas_core::disk_service::DiskType;
use communitas_core::encrypted_storage::VaultInfo;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

// ============================================================================
// Tokio Runtime (Global)
// ============================================================================

/// Global tokio runtime for async operations
#[allow(clippy::expect_used)]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
});

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}

// ============================================================================
// Error Handling
// ============================================================================

// Return values directly - errors become Dart exceptions via panic
// FRB v2 converts panics to Dart exceptions automatically
fn to_dart_error<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    match result {
        Ok(v) => v,
        Err(e) => panic!("API Error: {}", e),
    }
}

// ============================================================================
// Data Types (Flutter-specific versions)
// ============================================================================

/// User profile information
#[derive(Debug, Clone)]
pub struct FlutterUserProfile {
    pub four_words: String,
    pub display_name: String,
    pub device_name: String,
    pub device_type: String,
}

/// Entity type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlutterEntityType {
    Group,
    Channel,
    Project,
    Organisation,
    Person,
}

impl From<FlutterEntityType> for EntityType {
    fn from(t: FlutterEntityType) -> Self {
        match t {
            FlutterEntityType::Group => EntityType::Group,
            FlutterEntityType::Channel => EntityType::Channel,
            FlutterEntityType::Project => EntityType::Project,
            FlutterEntityType::Organisation => EntityType::Organisation,
            FlutterEntityType::Person => EntityType::Person,
        }
    }
}

impl From<EntityType> for FlutterEntityType {
    fn from(t: EntityType) -> Self {
        match t {
            EntityType::Group => FlutterEntityType::Group,
            EntityType::Channel => FlutterEntityType::Channel,
            EntityType::Project => FlutterEntityType::Project,
            EntityType::Organisation => FlutterEntityType::Organisation,
            EntityType::Person => FlutterEntityType::Person,
        }
    }
}

/// Entity information
#[derive(Debug, Clone)]
pub struct FlutterEntity {
    pub id: String,
    pub name: String,
    pub entity_type: FlutterEntityType,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub member_count: usize,
    pub parent_org_id: Option<String>,
    pub network_four_words: Option<String>,
    pub is_local_only: bool,
}

impl From<&EntityResponse> for FlutterEntity {
    fn from(e: &EntityResponse) -> Self {
        Self {
            id: e.id.clone(),
            name: e.name.clone(),
            entity_type: e.entity_type.into(),
            description: e.description.clone(),
            created_by: e.created_by.clone(),
            created_at: e.created_at,
            member_count: e.member_count,
            parent_org_id: e.parent_org_id.clone(),
            network_four_words: e.network_four_words.clone(),
            is_local_only: e.is_local_only,
        }
    }
}

/// Vault information
#[derive(Debug, Clone)]
pub struct FlutterVaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

impl From<&VaultInfo> for FlutterVaultInfo {
    fn from(v: &VaultInfo) -> Self {
        Self {
            four_words: v.four_words.clone(),
            display_name: v.display_name.clone(),
            created_at: v.created_at,
            last_accessed: v.last_accessed,
            size_bytes: v.size_bytes,
        }
    }
}

impl From<VaultInfo> for FlutterVaultInfo {
    fn from(v: VaultInfo) -> Self {
        FlutterVaultInfo::from(&v)
    }
}

/// Session information
#[derive(Debug, Clone)]
pub struct FlutterSessionInfo {
    pub session_id: String,
    pub four_words: String,
    pub display_name: String,
}

impl From<&SessionInfo> for FlutterSessionInfo {
    fn from(s: &SessionInfo) -> Self {
        Self {
            session_id: s.session_id.clone(),
            four_words: s.four_words.clone(),
            display_name: s.display_name.clone(),
        }
    }
}

impl From<SessionInfo> for FlutterSessionInfo {
    fn from(s: SessionInfo) -> Self {
        FlutterSessionInfo::from(&s)
    }
}

/// Network status information
#[derive(Debug, Clone)]
pub struct FlutterNetworkInfo {
    pub is_active: bool,
    pub bound_port: Option<u16>,
    pub peer_count: u32,
    pub external_address: Option<String>,
    pub bootstrap_connected: bool,
}

/// Disk type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlutterDiskType {
    Private,
    Public,
    Shared,
}

impl From<FlutterDiskType> for DiskType {
    fn from(t: FlutterDiskType) -> Self {
        match t {
            FlutterDiskType::Private => DiskType::Private,
            FlutterDiskType::Public => DiskType::Public,
            FlutterDiskType::Shared => DiskType::Shared,
        }
    }
}

/// Event wrapper for Flutter
#[derive(Debug, Clone)]
pub enum FlutterEvent {
    NetworkingStarted { address: String },
    NetworkingStopped,
    PeerConnected { peer_id: String },
    PeerDisconnected { peer_id: String },
    EntityCreated { entity_id: String },
    EntityUpdated { entity_id: String },
    MessageSent { message_id: String, entity_id: String },
    MessageReceived { message_id: String, entity_id: String },
    InviteCreated { invite_id: String },
    InviteAccepted { invite_id: String },
    FileWritten { entity_id: String, path: String },
    FileDeleted { entity_id: String, path: String },
    Error { code: String, message: String },
}

impl From<&Event> for FlutterEvent {
    fn from(e: &Event) -> Self {
        match e {
            Event::NetworkingStarted {
                listen_address,
                connection_identity: _,
            } => FlutterEvent::NetworkingStarted {
                address: listen_address.clone(),
            },
            Event::NetworkingStopped => FlutterEvent::NetworkingStopped,
            Event::PeerConnected { peer_four_words } => FlutterEvent::PeerConnected {
                peer_id: peer_four_words.clone(),
            },
            Event::EntityCreated { entity_id, .. } => FlutterEvent::EntityCreated {
                entity_id: entity_id.clone(),
            },
            Event::EntityLinkedToNetwork { entity_id, .. } => FlutterEvent::EntityUpdated {
                entity_id: entity_id.clone(),
            },
            Event::EntitySynced { entity_id } => FlutterEvent::EntityUpdated {
                entity_id: entity_id.clone(),
            },
            Event::MessageSent {
                message_id,
                entity_id,
                ..
            } => FlutterEvent::MessageSent {
                message_id: message_id.clone(),
                entity_id: entity_id.clone(),
            },
            Event::InviteCreated { invite_id, .. } => FlutterEvent::InviteCreated {
                invite_id: invite_id.clone(),
            },
            Event::InviteAccepted { invite_id, .. } => FlutterEvent::InviteAccepted {
                invite_id: invite_id.clone(),
            },
            Event::FileWritten { entity_id, path, .. } => FlutterEvent::FileWritten {
                entity_id: entity_id.clone(),
                path: path.clone(),
            },
            Event::FileDeleted { entity_id, path, .. } => FlutterEvent::FileDeleted {
                entity_id: entity_id.clone(),
                path: path.clone(),
            },
            // Map other events that don't have direct Flutter equivalents
            _ => FlutterEvent::Error {
                code: "UNHANDLED_EVENT".to_string(),
                message: format!("Event type: {:?}", std::mem::discriminant(e)),
            },
        }
    }
}

// ============================================================================
// Standalone Functions
// ============================================================================

/// Generate a random four-word identity
pub fn generate_id_words() -> String {
    to_dart_error(communitas_core::identity::generate_id_words())
}

// ============================================================================
// CommunitasApi - Main API for Flutter (wraps CommunitasApp)
// ============================================================================

/// Main API struct for Flutter bindings
///
/// This wraps CommunitasApp and provides the Command/Query interface for Flutter.
/// All mutations go through execute(), all reads through query().
#[frb(opaque)]
pub struct CommunitasApi {
    /// The headless core application
    app: Arc<CommunitasApp>,
    /// Auth service (managed separately for vault operations)
    auth_service: Arc<RwLock<Option<AuthService>>>,
    /// Storage path for vault operations
    storage_path: PathBuf,
}

impl CommunitasApi {
    // ========================================================================
    // Initialization
    // ========================================================================

    /// Initialize the API with the given identity and storage path
    ///
    /// This creates a new CommunitasApp instance and returns the API wrapper.
    /// Returns an error string if initialization fails, or the API on success.
    pub fn create(
        four_words: String,
        display_name: String,
        device_name: String,
        storage_path: String,
    ) -> Result<Self, String> {
        let path = PathBuf::from(&storage_path);

        let app = block_on(async {
            CommunitasApp::new(four_words, display_name, device_name, storage_path).await
        })
        .map_err(|e| e.to_string())?;

        Ok(Self {
            app: Arc::new(app),
            auth_service: Arc::new(RwLock::new(None)),
            storage_path: path,
        })
    }

    /// Helper to get or initialize auth service
    async fn get_or_init_auth<'a>(
        &self,
        auth_lock: &'a mut Option<AuthService>,
    ) -> &'a mut AuthService {
        if auth_lock.is_none() {
            let storage_config = communitas_core::encrypted_storage::StorageConfig {
                vault_dir: self.storage_path.join("vaults"),
                use_keyring: true,
                ..Default::default()
            };
            let storage = to_dart_error(
                communitas_core::encrypted_storage::EncryptedStorageManager::new(storage_config)
                    .await
            );
            *auth_lock = Some(AuthService::new(storage));
        }
        auth_lock
            .as_mut()
            .expect("Auth service should be initialized")
    }

    // ========================================================================
    // Command Execution (All mutations)
    // ========================================================================

    /// Execute a command and return resulting events
    ///
    /// This is the primary way to mutate application state.
    /// All commands produce events that describe what changed.
    fn execute_command(&self, command: Command) -> Vec<FlutterEvent> {
        block_on(async {
            let events = to_dart_error(self.app.execute(command).await);
            events.iter().map(FlutterEvent::from).collect()
        })
    }

    // ========================================================================
    // Query Execution (All reads)
    // ========================================================================

    /// Execute a query and return the response
    fn execute_query(&self, query: Query) -> QueryResponse {
        block_on(async {
            to_dart_error(self.app.query(query).await)
        })
    }

    // ========================================================================
    // Profile & Identity
    // ========================================================================

    /// Get the current user profile
    pub fn get_profile(&self) -> FlutterUserProfile {
        let response = self.execute_query(Query::GetProfile);
        match response {
            QueryResponse::Profile {
                four_words,
                display_name,
                device_name,
                device_type,
            } => FlutterUserProfile {
                four_words,
                display_name,
                device_name,
                device_type,
            },
            _ => panic!("Unexpected response type for GetProfile"),
        }
    }

    /// Update display name
    pub fn update_display_name(&self, display_name: String) -> Vec<FlutterEvent> {
        self.execute_command(Command::UpdateDisplayName { display_name })
    }

    // ========================================================================
    // Authentication (Vault operations - separate from core app)
    // ========================================================================

    /// Create a new vault with the given identity
    pub fn auth_create_vault(
        &self,
        four_words: String,
        display_name: String,
        password: String,
    ) -> String {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await;
            to_dart_error(
                auth.create_vault(&four_words, &password, &display_name).await
            )
        })
    }

    /// Login to an existing vault
    pub fn auth_login(
        &self,
        four_words: String,
        password: String,
    ) -> FlutterSessionInfo {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await;
            let session = to_dart_error(
                auth.login(&four_words, &password, None).await
            );
            FlutterSessionInfo::from(session)
        })
    }

    /// Logout from the current session
    pub fn auth_logout(&self) {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            if let Some(auth) = auth_lock.as_mut() {
                to_dart_error(auth.logout().await);
            }
        })
    }

    /// Get the current session info
    pub fn auth_get_current_session(&self) -> Option<FlutterSessionInfo> {
        block_on(async {
            let auth_lock = self.auth_service.read().await;
            auth_lock
                .as_ref()
                .and_then(|auth| auth.get_current_session())
                .map(FlutterSessionInfo::from)
        })
    }

    /// List all available vaults
    pub fn auth_list_vaults(&self) -> Vec<FlutterVaultInfo> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await;
            let vaults = to_dart_error(auth.list_vaults().await);
            vaults.into_iter().map(FlutterVaultInfo::from).collect()
        })
    }

    /// Check if a vault exists
    pub fn auth_vault_exists(&self, four_words: String) -> bool {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await;
            to_dart_error(auth.vault_exists(&four_words).await)
        })
    }

    /// Delete a vault
    pub fn auth_delete_vault(&self, four_words: String, password: String) {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await;
            to_dart_error(auth.delete_vault(&four_words, &password).await)
        })
    }

    // ========================================================================
    // Entity Management (via Commands/Queries)
    // ========================================================================

    /// Create a new entity
    pub fn entity_create(
        &self,
        name: String,
        entity_type: FlutterEntityType,
        description: Option<String>,
        _parent_org_id: Option<String>,
    ) -> Vec<FlutterEvent> {
        let core_type: EntityType = entity_type.into();
        self.execute_command(Command::CreateEntity {
            name,
            entity_type: core_type,
            description,
            initial_members: vec![],
        })
    }

    /// Get an entity by ID
    pub fn entity_get(&self, entity_id: String) -> FlutterEntity {
        let response = self.execute_query(Query::GetEntity { entity_id });
        match response {
            QueryResponse::Entity(entity) => FlutterEntity::from(&entity),
            _ => panic!("Unexpected response type for GetEntity"),
        }
    }

    /// List all entities
    pub fn entity_list(&self) -> Vec<FlutterEntity> {
        let response = self.execute_query(Query::ListEntities);
        match response {
            QueryResponse::EntityList(entities) => {
                entities.iter().map(FlutterEntity::from).collect()
            }
            _ => panic!("Unexpected response type for ListEntities"),
        }
    }

    /// List entities by type
    pub fn entity_list_by_type(
        &self,
        entity_type: FlutterEntityType,
    ) -> Vec<FlutterEntity> {
        let core_type: EntityType = entity_type.into();
        let response = self.execute_query(Query::ListEntitiesByType {
            entity_type: core_type,
        });
        match response {
            QueryResponse::EntityList(entities) => {
                entities.iter().map(FlutterEntity::from).collect()
            }
            _ => panic!("Unexpected response type for ListEntitiesByType"),
        }
    }

    /// Add a member to an entity
    pub fn entity_add_member(
        &self,
        entity_type: FlutterEntityType,
        entity_id: String,
        member_id: String,
        role: String,
    ) -> Vec<FlutterEvent> {
        self.execute_command(Command::AddMember {
            entity_type: entity_type.into(),
            entity_id,
            member_id,
            role,
        })
    }

    /// Remove a member from an entity
    pub fn entity_remove_member(
        &self,
        entity_type: FlutterEntityType,
        entity_id: String,
        member_id: String,
    ) -> Vec<FlutterEvent> {
        self.execute_command(Command::RemoveMember {
            entity_type: entity_type.into(),
            entity_id,
            member_id,
        })
    }

    // ========================================================================
    // P2P Networking
    // ========================================================================

    /// Start the gossip network
    pub fn gossip_start(&self, port: Option<u16>) -> Vec<FlutterEvent> {
        self.execute_command(Command::StartNetworking {
            preferred_port: port,
        })
    }

    /// Stop the gossip network
    pub fn gossip_stop(&self) -> Vec<FlutterEvent> {
        self.execute_command(Command::StopNetworking)
    }

    /// Connect to a peer by four words
    pub fn gossip_connect_to_peer(&self, four_words: String) -> Vec<FlutterEvent> {
        self.execute_command(Command::ConnectToPeer {
            peer_four_words: four_words,
        })
    }

    /// Get network information
    pub fn gossip_get_network_info(&self) -> FlutterNetworkInfo {
        // Get networking active status
        let is_active_response = self.execute_query(Query::IsNetworkingActive);
        let is_active = match is_active_response {
            QueryResponse::Bool(active) => active,
            _ => false,
        };

        // Get connection identity (four words)
        let identity_response = self.execute_query(Query::GetConnectionIdentity);
        let connection_identity = match identity_response {
            QueryResponse::OptionalString(id) => id,
            _ => None,
        };

        // Get external address
        let external_response = self.execute_query(Query::GetExternalAddress);
        let external_address = match external_response {
            QueryResponse::OptionalString(addr) => addr,
            _ => None,
        };

        FlutterNetworkInfo {
            is_active,
            bound_port: None, // Not directly available via query
            peer_count: 0,    // Would need separate query for peer list
            external_address,
            bootstrap_connected: connection_identity.is_some(),
        }
    }

    // ========================================================================
    // Messaging
    // ========================================================================

    /// Send a message to an entity
    pub fn message_send(
        &self,
        entity_id: String,
        entity_type: FlutterEntityType,
        text: String,
        reply_to_id: Option<String>,
    ) -> Vec<FlutterEvent> {
        // Get current user's four words for author field
        let profile = self.get_profile();
        self.execute_command(Command::SendMessage {
            entity_id,
            entity_type: entity_type.into(),
            text,
            author: profile.four_words,
            reply_to_id,
            attachments: None,
        })
    }

    // ========================================================================
    // Invites
    // ========================================================================

    /// Create an invite
    pub fn invite_create(
        &self,
        recipient_id: String,
        entity_type: FlutterEntityType,
        entity_id: String,
        role: String,
        message: Option<String>,
    ) -> Vec<FlutterEvent> {
        self.execute_command(Command::CreateInvite {
            recipient_id,
            entity_type: entity_type.into(),
            entity_id,
            role,
            message,
            expires_in_hours: Some(72), // Default 3 days expiry
        })
    }

    /// Accept an invite
    pub fn invite_accept(&self, invite_id: String) -> Vec<FlutterEvent> {
        self.execute_command(Command::AcceptInvite { invite_id })
    }

    /// Reject an invite
    pub fn invite_reject(&self, invite_id: String) -> Vec<FlutterEvent> {
        self.execute_command(Command::RejectInvite { invite_id })
    }

    /// Revoke an invite (sender only)
    pub fn invite_revoke(&self, invite_id: String) -> Vec<FlutterEvent> {
        self.execute_command(Command::RevokeInvite { invite_id })
    }
}

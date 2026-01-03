// UniFFI scaffolding (Swift bindings) - enabled by default
#[cfg(feature = "uniffi-bindings")]
uniffi::setup_scaffolding!();

// Flutter API module (flutter_rust_bridge bindings)
// Build with: cargo build --no-default-features --features flutter-bindings
#[cfg(feature = "flutter-bindings")]
mod frb_generated;
#[cfg(feature = "flutter-bindings")]
pub mod flutter_api;

use communitas_core::auth_service::{AuthService, SessionInfo};
use communitas_core::crdt::EntityType;
use communitas_core::crdt::MessageContent;
use communitas_core::disk_service::{DiskStats, DiskType, FileInfo as DiskFileInfo};
use communitas_core::doc_replicator::StorageMode;
use communitas_core::encrypted_storage::{
    EncryptedStorageManager, PasskeyInfo, RecentIdentity, StorageConfig, VaultInfo,
};
use communitas_core::types::DeviceType;
use communitas_core::types::UserProfile;
use communitas_core::CoreContext;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

// ============================================================================
// Tokio Runtime (Global)
// ============================================================================

/// Global tokio runtime for async operations.
/// UniFFI async functions need a runtime to execute Rust futures that use tokio.
/// This runtime is created lazily on first use and stays alive for the app lifetime.
///
/// Note: `expect` is used here because if the tokio runtime cannot be created,
/// there is no way to recover - the entire application cannot function without it.
#[allow(clippy::expect_used)]
static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
});

/// Helper to run async code on the global runtime
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ClientError {
    #[error("Core initialization failed: {0}")]
    InitError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Entity error: {0}")]
    EntityError(String),
    #[error("Message error: {0}")]
    MessageError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Document error: {0}")]
    DocumentError(String),
    #[error("Presence error: {0}")]
    PresenceError(String),
    #[error("IO error: {0}")]
    IoError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("WebRTC error: {0}")]
    WebRtcError(String),
    #[error("Invite error: {0}")]
    InviteError(String),
}

impl From<String> for ClientError {
    fn from(s: String) -> Self {
        ClientError::IoError(s)
    }
}

impl From<anyhow::Error> for ClientError {
    fn from(e: anyhow::Error) -> Self {
        ClientError::IoError(e.to_string())
    }
}

// ============================================================================
// Standalone Functions (UniFFI Exported)
// ============================================================================

/// Generate a random four-word identity from the dictionary
///
/// Generates 4 random valid words from the four-word-networking dictionary
/// to create a new user identity like "ocean-forest-moon-star".
///
/// This should be used when creating new test identities instead of
/// hardcoded or repeated words.
#[uniffi::export]
pub fn generate_id_words() -> Result<String, ClientError> {
    communitas_core::identity::generate_id_words()
        .map_err(|e| ClientError::AuthError(e.to_string()))
}

// ============================================================================
// Data Types (Mirrored for UniFFI)
// ============================================================================

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftUserProfile {
    pub four_words: String,
    pub display_name: String,
    pub device_name: String,
    pub device_type: String,
}

impl From<&UserProfile> for SwiftUserProfile {
    fn from(p: &UserProfile) -> Self {
        Self {
            four_words: p.id_fw.clone(),
            display_name: p.display_name.clone(),
            device_name: format!("{:?}", p.device_type),
            device_type: format!("{:?}", p.device_type),
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum SwiftEntityType {
    Group,
    Channel,
    Project,
    Organisation,
    Person,
}

impl From<SwiftEntityType> for EntityType {
    fn from(t: SwiftEntityType) -> Self {
        match t {
            SwiftEntityType::Group => EntityType::Group,
            SwiftEntityType::Channel => EntityType::Channel,
            SwiftEntityType::Project => EntityType::Project,
            SwiftEntityType::Organisation => EntityType::Organisation,
            SwiftEntityType::Person => EntityType::Person,
        }
    }
}

impl From<EntityType> for SwiftEntityType {
    fn from(t: EntityType) -> Self {
        match t {
            EntityType::Group => SwiftEntityType::Group,
            EntityType::Channel => SwiftEntityType::Channel,
            EntityType::Project => SwiftEntityType::Project,
            EntityType::Organisation => SwiftEntityType::Organisation,
            EntityType::Person => SwiftEntityType::Person,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftEntity {
    pub id: String,
    pub name: String,
    pub entity_type: SwiftEntityType,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub members: Vec<String>,
    pub parent_org_id: Option<String>,
    /// Network four-word identity if this entity is linked to the P2P network
    pub network_four_words: Option<String>,
    /// True if this is a local-only placeholder (no network identity yet)
    pub is_local_only: bool,
    /// Timestamp when entity was linked to a network identity (milliseconds)
    pub linked_at: Option<i64>,
    /// Timestamp of last successful sync with network peer (milliseconds)
    pub last_sync_at: Option<i64>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftMemberInfo {
    pub four_words: String,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftMessage {
    pub id: String,
    pub text: String,
    pub author: String,
    pub created_at: i64,
    pub reply_to_id: Option<String>,
    pub entity_id: String,
    pub reactions: Vec<SwiftReaction>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftReaction {
    pub emoji: String,
    pub count: u32,
    pub users: Vec<String>,
}

// Auth types
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftSessionInfo {
    pub session_id: String,
    pub four_words: String,
    pub display_name: String,
}

impl From<SessionInfo> for SwiftSessionInfo {
    fn from(s: SessionInfo) -> Self {
        Self {
            session_id: s.session_id,
            four_words: s.four_words,
            display_name: s.display_name,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftVaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

impl From<VaultInfo> for SwiftVaultInfo {
    fn from(v: VaultInfo) -> Self {
        Self {
            four_words: v.four_words,
            display_name: v.display_name,
            created_at: v.created_at,
            last_accessed: v.last_accessed,
            size_bytes: v.size_bytes,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftRecentIdentity {
    pub four_words: String,
    pub display_name: String,
    pub last_used: u64,
    pub has_passkey: bool,
}

impl From<RecentIdentity> for SwiftRecentIdentity {
    fn from(r: RecentIdentity) -> Self {
        Self {
            four_words: r.four_words,
            display_name: r.display_name,
            last_used: r.last_used,
            has_passkey: r.has_passkey,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftPasskeyInfo {
    pub credential_id: String,
    pub device_name: String,
    pub created_at: u64,
    pub last_used: Option<u64>,
}

impl From<PasskeyInfo> for SwiftPasskeyInfo {
    fn from(p: PasskeyInfo) -> Self {
        Self {
            credential_id: p.four_words.clone(), // Use four_words as credential_id
            device_name: p.device_name,
            created_at: p.registered_at,
            last_used: p.last_used,
        }
    }
}

// ============================================================================
// Invite Types (Cross-Organization Collaboration)
// ============================================================================

/// Invite status for cross-organization collaboration
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SwiftInviteStatus {
    /// Invite is pending a response
    Pending,
    /// Invite was accepted by the recipient
    Accepted,
    /// Invite was rejected by the recipient
    Rejected,
    /// Invite expired before any action was taken
    Expired,
    /// Invite was revoked by the creator or an admin
    Revoked,
}

impl From<communitas_core::InviteStatus> for SwiftInviteStatus {
    fn from(s: communitas_core::InviteStatus) -> Self {
        use communitas_core::InviteStatus;
        match s {
            InviteStatus::Pending => SwiftInviteStatus::Pending,
            InviteStatus::Accepted => SwiftInviteStatus::Accepted,
            InviteStatus::Rejected => SwiftInviteStatus::Rejected,
            InviteStatus::Expired => SwiftInviteStatus::Expired,
            InviteStatus::Revoked => SwiftInviteStatus::Revoked,
        }
    }
}

impl From<SwiftInviteStatus> for communitas_core::InviteStatus {
    fn from(s: SwiftInviteStatus) -> Self {
        use communitas_core::InviteStatus;
        match s {
            SwiftInviteStatus::Pending => InviteStatus::Pending,
            SwiftInviteStatus::Accepted => InviteStatus::Accepted,
            SwiftInviteStatus::Rejected => InviteStatus::Rejected,
            SwiftInviteStatus::Expired => InviteStatus::Expired,
            SwiftInviteStatus::Revoked => InviteStatus::Revoked,
        }
    }
}

/// Invite for cross-organization collaboration
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftInvite {
    /// Unique invite ID
    pub id: String,
    /// Type of entity this invite grants access to
    pub entity_type: SwiftEntityType,
    /// ID of the entity being joined
    pub entity_id: String,
    /// Four-word ID of the invite creator
    pub creator_id: String,
    /// Four-word ID of the intended recipient
    pub recipient_id: String,
    /// Role being offered (e.g., "member", "admin")
    pub role: String,
    /// Current status of the invite
    pub status: SwiftInviteStatus,
    /// When the invite was created (milliseconds since epoch)
    pub created_at: i64,
    /// When the invite expires (None = never expires)
    pub expires_at: Option<i64>,
    /// Optional message from the creator
    pub message: Option<String>,
    /// Four-word ID of who resolved the invite (if resolved)
    pub resolved_by: Option<String>,
    /// When the invite was resolved (if resolved)
    pub resolved_at: Option<i64>,
}

impl From<communitas_core::Invite> for SwiftInvite {
    fn from(i: communitas_core::Invite) -> Self {
        Self {
            id: i.id,
            entity_type: i.entity_type.into(),
            entity_id: i.entity_id,
            creator_id: i.creator_id,
            recipient_id: i.recipient_id,
            role: i.role,
            status: i.status.into(),
            created_at: i.created_at,
            expires_at: i.expires_at,
            message: i.message,
            resolved_by: i.resolved_by,
            resolved_at: i.resolved_at,
        }
    }
}

// Network/Presence types
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SwiftPresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftPresenceInfo {
    pub four_words: String,
    pub status: SwiftPresenceStatus,
    pub last_seen: i64,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftContactInfo {
    /// Unique identifier (UUID) for this contact
    pub id: String,
    /// Four-word network identity (optional for local-only contacts)
    pub four_words: Option<String>,
    pub display_name: Option<String>,
    pub is_favourite: bool,
    pub online: bool,
    /// True if this is a local-only placeholder (no network identity yet)
    pub is_local_only: bool,
    /// Timestamp when linked to network identity (milliseconds)
    pub linked_at: Option<i64>,
    /// Timestamp of last successful sync (milliseconds)
    pub last_sync_at: Option<i64>,
    // Endpoint tracking for direct reconnection
    pub last_seen_endpoint: Option<String>,
    pub endpoint_updated_at: Option<i64>,
    pub endpoint_success_count: u32,
    pub endpoint_failure_count: u32,
}

// Document types
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SwiftStorageMode {
    FilesOnly,
    WebOnly,
    Both,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftDocumentInfo {
    pub id: String,
    pub name: String,
    pub entity_id: String,
    pub created_at: i64,
    pub modified_at: i64,
    pub storage_mode: SwiftStorageMode,
}

// Storage types
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftFileInfo {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub modified_at: i64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftStorageStats {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub file_count: u32,
}

// Disk types (per-entity virtual disks)
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SwiftDiskType {
    /// Private: Encrypted, local-only storage (owner access only)
    Private,
    /// Public: Content-addressed, distributed storage (world-readable)
    Public,
    /// Shared: Group-accessible with shared encryption (members only)
    Shared,
}

impl From<SwiftDiskType> for DiskType {
    fn from(t: SwiftDiskType) -> Self {
        match t {
            SwiftDiskType::Private => DiskType::Private,
            SwiftDiskType::Public => DiskType::Public,
            SwiftDiskType::Shared => DiskType::Shared,
        }
    }
}

impl From<DiskType> for SwiftDiskType {
    fn from(t: DiskType) -> Self {
        match t {
            DiskType::Private => SwiftDiskType::Private,
            DiskType::Public => SwiftDiskType::Public,
            DiskType::Shared => SwiftDiskType::Shared,
        }
    }
}

/// File information for virtual disk operations (includes content hash)
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftDiskFileInfo {
    /// Full path within the disk (e.g., "/docs/readme.md")
    pub path: String,
    /// File or directory name
    pub name: String,
    /// True if this is a directory
    pub is_directory: bool,
    /// Size in bytes (0 for directories)
    pub size_bytes: u64,
    /// Last modified timestamp (Unix epoch seconds)
    pub modified_at: i64,
    /// BLAKE3 hash of contents (empty for directories)
    pub content_hash: String,
}

impl From<DiskFileInfo> for SwiftDiskFileInfo {
    fn from(f: DiskFileInfo) -> Self {
        Self {
            path: f.path,
            name: f.name,
            is_directory: f.is_directory,
            size_bytes: f.size_bytes,
            modified_at: f.modified_at,
            content_hash: f.content_hash,
        }
    }
}

/// Storage statistics for a virtual disk
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftDiskStats {
    /// Entity ID this disk belongs to
    pub entity_id: String,
    /// Type of disk
    pub disk_type: SwiftDiskType,
    /// Total bytes used
    pub used_bytes: u64,
    /// Total number of files
    pub file_count: u32,
    /// Total number of directories
    pub dir_count: u32,
    /// Last modification timestamp
    pub last_modified: i64,
}

impl From<DiskStats> for SwiftDiskStats {
    fn from(s: DiskStats) -> Self {
        Self {
            entity_id: s.entity_id,
            disk_type: s.disk_type.into(),
            used_bytes: s.used_bytes,
            file_count: s.file_count,
            dir_count: s.dir_count,
            last_modified: s.last_modified,
        }
    }
}

// Sync types
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftSyncState {
    pub entity_id: String,
    pub message_count: u32,
    pub last_sync_time: i64,
    pub is_syncing: bool,
}

// ============================================================================
// Kanban Types
// ============================================================================

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum SwiftKanbanCardState {
    Open,
    Closed,
    Postponed,
    Archived,
}

impl From<communitas_kanban::CardState> for SwiftKanbanCardState {
    fn from(s: communitas_kanban::CardState) -> Self {
        match s {
            communitas_kanban::CardState::Open => SwiftKanbanCardState::Open,
            communitas_kanban::CardState::Closed => SwiftKanbanCardState::Closed,
            communitas_kanban::CardState::Postponed => SwiftKanbanCardState::Postponed,
            communitas_kanban::CardState::Archived => SwiftKanbanCardState::Archived,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftKanbanBoard {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub project_id: String,
    pub created_by: String,
    pub created_at: i64,
    pub columns: Vec<SwiftKanbanColumn>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftKanbanColumn {
    pub id: String,
    pub board_id: String,
    pub name: String,
    pub position: u32,
    pub color: Option<String>,
    pub wip_limit: Option<u32>,
}

impl From<&communitas_kanban::Column> for SwiftKanbanColumn {
    fn from(c: &communitas_kanban::Column) -> Self {
        Self {
            id: c.id.clone(),
            board_id: c.board_id.clone(),
            name: c.name.clone(),
            position: c.position,
            color: c.color.clone(),
            wip_limit: c.wip_limit,
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftKanbanCard {
    pub id: String,
    pub board_id: String,
    pub column_id: String,
    pub title: String,
    pub description: String,
    pub position: u32,
    pub state: SwiftKanbanCardState,
    pub created_by: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub due_date: Option<i64>,
    pub assignee_ids: Vec<String>,
    pub tag_ids: Vec<String>,
    pub comment_count: u32,
}

impl SwiftKanbanCard {
    /// Create a SwiftKanbanCard from a kanban Card with the comment count
    pub fn from_with_count(c: &communitas_kanban::Card, comment_count: u32) -> Self {
        Self {
            id: c.id.clone(),
            board_id: c.board_id.clone(),
            column_id: c.column_id.clone(),
            title: c.title.clone(),
            description: c.description.clone(),
            position: c.position,
            state: c.state.into(),
            created_by: c.created_by.clone(),
            created_at: c.created_at,
            updated_at: c.updated_at,
            due_date: c.due_date,
            assignee_ids: c.assignee_ids.clone(),
            tag_ids: c.tag_ids.clone(),
            comment_count,
        }
    }
}

impl From<&communitas_kanban::Card> for SwiftKanbanCard {
    fn from(c: &communitas_kanban::Card) -> Self {
        Self::from_with_count(c, 0)
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftKanbanComment {
    pub id: String,
    pub card_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: i64,
    pub reply_to_id: Option<String>,
}

impl From<&communitas_kanban::Comment> for SwiftKanbanComment {
    fn from(c: &communitas_kanban::Comment) -> Self {
        Self {
            id: c.id.clone(),
            card_id: c.card_id.clone(),
            author_id: c.author_id.clone(),
            content: c.content.clone(),
            created_at: c.created_at,
            reply_to_id: c.reply_to_id.clone(),
        }
    }
}

// ============================================================================
// Permission Types (Phase 3: Granular Per-Resource Permissions)
// ============================================================================

/// Access level for a resource type
///
/// Ordered from most restrictive to least restrictive:
/// NotVisible < ReadOnly < Edit
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SwiftAccessLevel {
    /// Resource is hidden from the member
    NotVisible,
    /// Member can view but not modify
    ReadOnly,
    /// Member has full read/write access
    Edit,
}

impl From<communitas_core::permissions::AccessLevel> for SwiftAccessLevel {
    fn from(level: communitas_core::permissions::AccessLevel) -> Self {
        match level {
            communitas_core::permissions::AccessLevel::NotVisible => SwiftAccessLevel::NotVisible,
            communitas_core::permissions::AccessLevel::ReadOnly => SwiftAccessLevel::ReadOnly,
            communitas_core::permissions::AccessLevel::Edit => SwiftAccessLevel::Edit,
        }
    }
}

impl From<SwiftAccessLevel> for communitas_core::permissions::AccessLevel {
    fn from(level: SwiftAccessLevel) -> Self {
        match level {
            SwiftAccessLevel::NotVisible => communitas_core::permissions::AccessLevel::NotVisible,
            SwiftAccessLevel::ReadOnly => communitas_core::permissions::AccessLevel::ReadOnly,
            SwiftAccessLevel::Edit => communitas_core::permissions::AccessLevel::Edit,
        }
    }
}

/// Types of resources that can have permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum SwiftResourceType {
    /// Chat messages, threads, reactions
    Messages,
    /// Collaborative documents (CRDT)
    Documents,
    /// Kanban boards (project-only)
    KanbanBoards,
    /// Files in entity storage
    Files,
    /// Member list and roles
    Members,
    /// Entity settings and configuration
    Settings,
}

impl From<communitas_core::permissions::ResourceType> for SwiftResourceType {
    fn from(rt: communitas_core::permissions::ResourceType) -> Self {
        match rt {
            communitas_core::permissions::ResourceType::Messages => SwiftResourceType::Messages,
            communitas_core::permissions::ResourceType::Documents => SwiftResourceType::Documents,
            communitas_core::permissions::ResourceType::KanbanBoards => {
                SwiftResourceType::KanbanBoards
            }
            communitas_core::permissions::ResourceType::Files => SwiftResourceType::Files,
            communitas_core::permissions::ResourceType::Members => SwiftResourceType::Members,
            communitas_core::permissions::ResourceType::Settings => SwiftResourceType::Settings,
        }
    }
}

impl From<SwiftResourceType> for communitas_core::permissions::ResourceType {
    fn from(rt: SwiftResourceType) -> Self {
        match rt {
            SwiftResourceType::Messages => communitas_core::permissions::ResourceType::Messages,
            SwiftResourceType::Documents => communitas_core::permissions::ResourceType::Documents,
            SwiftResourceType::KanbanBoards => {
                communitas_core::permissions::ResourceType::KanbanBoards
            }
            SwiftResourceType::Files => communitas_core::permissions::ResourceType::Files,
            SwiftResourceType::Members => communitas_core::permissions::ResourceType::Members,
            SwiftResourceType::Settings => communitas_core::permissions::ResourceType::Settings,
        }
    }
}

/// A single permission entry (resource type + access level)
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftMemberPermission {
    pub resource_type: SwiftResourceType,
    pub access_level: SwiftAccessLevel,
}

// ============================================================================
// WebRTC Types
// ============================================================================

/// Call state for active WebRTC calls
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftCallState {
    /// Unique call identifier
    pub call_id: String,
    /// Target peer's four-word address
    pub target_four_words: String,
    /// Whether video is currently enabled
    pub is_video_enabled: bool,
    /// Whether audio is currently enabled
    pub is_audio_enabled: bool,
    /// Whether screen sharing is active
    pub is_screen_sharing: bool,
    /// Call connection state: "initiating", "ringing", "connected", "ended"
    pub state: String,
}

/// Media constraints for call initiation
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftMediaConstraints {
    /// Enable audio in this call
    pub has_audio: bool,
    /// Enable video in this call
    pub has_video: bool,
}

/// Media device information
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftMediaDevice {
    /// Device identifier
    pub device_id: String,
    /// Human-readable device label
    pub label: String,
    /// Device kind: "audioinput", "audiooutput", "videoinput"
    pub kind: String,
}

/// Events emitted during calls
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SwiftCallEvent {
    /// Incoming call from another peer
    IncomingCall {
        call_id: String,
        from_four_words: String,
        has_video: bool,
    },
    /// Call has been initiated (outgoing)
    CallInitiated {
        call_id: String,
        to_four_words: String,
    },
    /// Call connection established
    CallConnected { call_id: String },
    /// Call was rejected by the remote peer
    CallRejected { call_id: String },
    /// Call has ended
    CallEnded { call_id: String },
    /// Remote peer toggled their video
    RemoteVideoChanged { call_id: String, enabled: bool },
    /// Remote peer toggled their audio
    RemoteAudioChanged { call_id: String, enabled: bool },
    /// Remote peer started/stopped screen share
    RemoteScreenShareChanged { call_id: String, active: bool },
}

// Network info types
#[derive(Debug, Clone, uniffi::Record)]
pub struct SwiftNetworkInfo {
    /// Whether networking is currently active
    pub is_active: bool,
    /// Four-word encoded connection identity (e.g., "ocean-forest-moon-star")
    pub connection_identity: Option<String>,
    /// Listen address as string (e.g., "192.168.1.100:49152")
    pub listen_address: Option<String>,
    /// External/public address as seen from internet (NAT-reflected)
    /// This is the address other peers should use to connect to us
    pub external_address: Option<String>,
    /// External address encoded as four-word format (for sharing)
    pub external_address_words: Option<String>,
    /// Port number if networking is active
    pub port: Option<u16>,
    /// Our four-word user identity
    pub four_words: String,
    /// Whether we're in local-only mode (no WAN connectivity)
    pub is_local_only_mode: bool,
}

// ============================================================================
// Main Client (Facade)
// ============================================================================

/// Main Communitas client with sub-clients for each domain
#[derive(uniffi::Object)]
pub struct CommunitasClient {
    inner: Arc<RwLock<CoreContext>>,
    auth_service: Arc<RwLock<Option<AuthService>>>,
    storage_path: PathBuf,
}

#[uniffi::export]
impl CommunitasClient {
    /// Initialize the client with identity and storage
    #[uniffi::constructor]
    pub fn new(
        four_words: String,
        display_name: String,
        device_name: String,
        storage_path: String,
    ) -> Result<Arc<Self>, ClientError> {
        let path = PathBuf::from(&storage_path);

        let device_type = DeviceType::Mobile;

        let context = block_on(async {
            CoreContext::initialize(
                four_words,
                display_name,
                device_name,
                device_type,
                path.clone(),
            )
            .await
        })
        .map_err(ClientError::InitError)?;

        Ok(Arc::new(Self {
            inner: Arc::new(RwLock::new(context)),
            auth_service: Arc::new(RwLock::new(None)),
            storage_path: path,
        }))
    }

    /// Get current user profile
    pub fn get_profile(&self) -> SwiftUserProfile {
        block_on(async {
            let ctx = self.inner.read().await;
            SwiftUserProfile {
                four_words: ctx.profile.id_fw.clone(),
                display_name: ctx.profile.display_name.clone(),
                device_name: ctx.device_name.clone(),
                device_type: format!("{:?}", ctx.profile.device_type),
            }
        })
    }

    /// Check if networking is active
    pub fn is_networking_active(&self) -> bool {
        block_on(async {
            let ctx = self.inner.read().await;
            ctx.is_networking_active()
        })
    }

    /// Get connection identity (four-word encoded address)
    pub fn get_connection_identity(&self) -> Option<String> {
        block_on(async {
            let ctx = self.inner.read().await;
            ctx.connection_identity().map(|s| s.to_string())
        })
    }

    // ========================================================================
    // Auth Sub-Client Methods
    // ========================================================================

    /// Create a new vault for a four-word identity
    pub fn auth_create_vault(
        &self,
        four_words: String,
        password: String,
        display_name: String,
    ) -> Result<String, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            auth.create_vault(&four_words, &password, &display_name)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))
        })
    }

    /// Login with four-word identity and password
    pub fn auth_login(
        &self,
        four_words: String,
        password: String,
    ) -> Result<SwiftSessionInfo, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            let session = auth
                .login(&four_words, &password, None)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;
            Ok(SwiftSessionInfo::from(session))
        })
    }

    /// Logout current session
    pub fn auth_logout(&self) -> Result<(), ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            if let Some(auth) = auth_lock.as_mut() {
                auth.logout()
                    .await
                    .map_err(|e| ClientError::AuthError(e.to_string()))?;
            }
            Ok(())
        })
    }

    /// Get current active session
    pub fn auth_get_current_session(&self) -> Option<SwiftSessionInfo> {
        block_on(async {
            let auth_lock = self.auth_service.read().await;
            auth_lock
                .as_ref()
                .and_then(|auth| auth.get_current_session())
                .map(SwiftSessionInfo::from)
        })
    }

    /// List all available vaults
    pub fn auth_list_vaults(&self) -> Result<Vec<SwiftVaultInfo>, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            let vaults = auth
                .list_vaults()
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;
            Ok(vaults.into_iter().map(SwiftVaultInfo::from).collect())
        })
    }

    /// Check if vault exists for four-word identity
    pub fn auth_vault_exists(&self, four_words: String) -> Result<bool, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            auth.vault_exists(&four_words)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))
        })
    }

    /// Delete a vault (requires password confirmation)
    pub fn auth_delete_vault(
        &self,
        four_words: String,
        password: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            auth.delete_vault(&four_words, &password)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))
        })
    }

    /// Register passkey for biometric authentication
    pub fn auth_register_passkey(
        &self,
        four_words: String,
        device_name: String,
    ) -> Result<SwiftPasskeyInfo, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            let info = auth
                .passkey_register(&four_words, &device_name)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;
            Ok(SwiftPasskeyInfo::from(info))
        })
    }

    /// Authenticate using passkey/biometric
    pub fn auth_authenticate_with_passkey(
        &self,
        four_words: String,
    ) -> Result<SwiftSessionInfo, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            let session = auth
                .passkey_authenticate(&four_words)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;
            Ok(SwiftSessionInfo::from(session))
        })
    }

    /// Check if identity has a registered passkey
    pub fn auth_has_passkey(&self, four_words: String) -> Result<bool, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            auth.passkey_has_passkey(&four_words)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))
        })
    }

    /// Delete passkey for an identity
    pub fn auth_delete_passkey(&self, four_words: String) -> Result<(), ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            auth.passkey_delete(&four_words)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))
        })
    }

    /// Get recent identities
    pub fn auth_get_recent_identities(&self) -> Result<Vec<SwiftRecentIdentity>, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            let recent = auth
                .get_recent_identities()
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;
            Ok(recent.into_iter().map(SwiftRecentIdentity::from).collect())
        })
    }

    /// Attempt auto-login using last-used identity
    pub fn auth_try_auto_login(&self) -> Result<Option<SwiftSessionInfo>, ClientError> {
        block_on(async {
            let mut auth_lock = self.auth_service.write().await;
            let auth = self.get_or_init_auth(&mut auth_lock).await?;
            let result = auth
                .try_auto_login()
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;
            Ok(result.map(SwiftSessionInfo::from))
        })
    }

    // ========================================================================
    // Entity Sub-Client Methods
    // ========================================================================

    /// Create a new entity (Group, Channel, Project, etc.)
    pub fn entity_create(
        &self,
        name: String,
        entity_type: SwiftEntityType,
        description: Option<String>,
        parent_org_id: Option<String>,
    ) -> Result<SwiftEntity, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let user_id = ctx.four_words.clone();

            let entity = ctx
                .entity_service
                .create_entity(name, entity_type.into(), description, user_id, vec![])
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Set parent org if provided
            if let Some(org_id) = parent_org_id.as_ref() {
                ctx.entity_service
                    .set_parent_organization(&entity.id, org_id)
                    .await
                    .map_err(|e| ClientError::EntityError(e.to_string()))?;
            }

            Ok(SwiftEntity {
                id: entity.id,
                name: entity.name,
                entity_type: entity.entity_type.into(),
                description: entity.description,
                created_by: entity.created_by,
                created_at: entity.created_at,
                members: entity.members,
                parent_org_id,
                network_four_words: entity.network_four_words,
                is_local_only: entity.is_local_only,
                linked_at: entity.linked_at,
                last_sync_at: entity.last_sync_at,
            })
        })
    }

    /// Get entity by ID
    pub fn entity_get(&self, entity_id: String) -> Result<SwiftEntity, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(SwiftEntity {
                id: entity.id,
                name: entity.name,
                entity_type: entity.entity_type.into(),
                description: entity.description,
                created_by: entity.created_by,
                created_at: entity.created_at,
                members: entity.members,
                parent_org_id: entity.parent_org_id,
                network_four_words: entity.network_four_words,
                is_local_only: entity.is_local_only,
                linked_at: entity.linked_at,
                last_sync_at: entity.last_sync_at,
            })
        })
    }

    /// List all entities
    pub fn entity_list(&self) -> Result<Vec<SwiftEntity>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let entities = ctx
                .entity_service
                .list_entities()
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(entities
                .into_iter()
                .map(|e| SwiftEntity {
                    id: e.id,
                    name: e.name,
                    entity_type: e.entity_type.into(),
                    description: e.description,
                    created_by: e.created_by,
                    created_at: e.created_at,
                    members: e.members,
                    parent_org_id: e.parent_org_id,
                    network_four_words: e.network_four_words,
                    is_local_only: e.is_local_only,
                    linked_at: e.linked_at,
                    last_sync_at: e.last_sync_at,
                })
                .collect())
        })
    }

    /// List entities by type
    pub fn entity_list_by_type(
        &self,
        entity_type: SwiftEntityType,
    ) -> Result<Vec<SwiftEntity>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let entities = ctx
                .entity_service
                .list_entities()
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            let target_type: EntityType = entity_type.into();
            Ok(entities
                .into_iter()
                .filter(|e| e.entity_type == target_type)
                .map(|e| SwiftEntity {
                    id: e.id,
                    name: e.name,
                    entity_type: e.entity_type.into(),
                    description: e.description,
                    created_by: e.created_by,
                    created_at: e.created_at,
                    members: e.members,
                    parent_org_id: e.parent_org_id,
                    network_four_words: e.network_four_words,
                    is_local_only: e.is_local_only,
                    linked_at: e.linked_at,
                    last_sync_at: e.last_sync_at,
                })
                .collect())
        })
    }

    /// Add member to entity
    pub fn entity_add_member(
        &self,
        entity_id: String,
        member_four_words: String,
        role: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get entity to determine type
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            ctx.entity_service
                .add_member(entity.entity_type, &entity_id, &member_four_words, &role)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))
        })
    }

    /// Remove member from entity
    pub fn entity_remove_member(
        &self,
        entity_id: String,
        member_four_words: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get entity to determine type
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            ctx.entity_service
                .remove_member(
                    entity.entity_type,
                    &entity_id,
                    &member_four_words,
                    &ctx.four_words,
                )
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))
        })
    }

    /// List members of an entity
    pub fn entity_list_members(
        &self,
        entity_id: String,
    ) -> Result<Vec<SwiftMemberInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get entity to determine type
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(members
                .into_iter()
                .map(|m| SwiftMemberInfo {
                    four_words: m.member_id.clone(),
                    display_name: Some(m.member_id), // Use member_id as display_name
                    role: m.role,
                    joined_at: m.joined_at,
                })
                .collect())
        })
    }

    /// Set parent organization for entity
    pub fn entity_set_parent_org(
        &self,
        entity_id: String,
        org_id: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            ctx.entity_service
                .set_parent_organization(&entity_id, &org_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))
        })
    }

    // ========================================================================
    // Entity Linking Sub-Client Methods (Local-Only / Network-Linked)
    // ========================================================================

    /// Create a local-only entity (not linked to network identity)
    pub fn entity_create_local(
        &self,
        name: String,
        entity_type: SwiftEntityType,
        description: Option<String>,
    ) -> Result<SwiftEntity, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let rust_type: EntityType = entity_type.into();

            let entity = ctx
                .entity_service
                .create_local_entity(name, rust_type, description, ctx.four_words.clone())
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(SwiftEntity {
                id: entity.id,
                name: entity.name,
                entity_type: entity.entity_type.into(),
                description: entity.description,
                created_by: entity.created_by,
                created_at: entity.created_at,
                members: entity.members,
                parent_org_id: entity.parent_org_id,
                network_four_words: entity.network_four_words,
                is_local_only: entity.is_local_only,
                linked_at: entity.linked_at,
                last_sync_at: entity.last_sync_at,
            })
        })
    }

    /// Link a local-only entity to a network identity
    pub fn entity_link_to_network(
        &self,
        entity_id: String,
        four_words: String,
    ) -> Result<SwiftEntity, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let entity = ctx
                .entity_service
                .link_entity_to_network(&entity_id, &four_words)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(SwiftEntity {
                id: entity.id,
                name: entity.name,
                entity_type: entity.entity_type.into(),
                description: entity.description,
                created_by: entity.created_by,
                created_at: entity.created_at,
                members: entity.members,
                parent_org_id: entity.parent_org_id,
                network_four_words: entity.network_four_words,
                is_local_only: entity.is_local_only,
                linked_at: entity.linked_at,
                last_sync_at: entity.last_sync_at,
            })
        })
    }

    /// Get all local-only entities
    pub fn entity_list_local_only(&self) -> Result<Vec<SwiftEntity>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let entities = ctx
                .entity_service
                .list_entities()
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(entities
                .into_iter()
                .filter(|e| e.is_local_only)
                .map(|e| SwiftEntity {
                    id: e.id,
                    name: e.name,
                    entity_type: e.entity_type.into(),
                    description: e.description,
                    created_by: e.created_by,
                    created_at: e.created_at,
                    members: e.members,
                    parent_org_id: e.parent_org_id,
                    network_four_words: e.network_four_words,
                    is_local_only: e.is_local_only,
                    linked_at: e.linked_at,
                    last_sync_at: e.last_sync_at,
                })
                .collect())
        })
    }

    /// Get all network-linked entities
    pub fn entity_list_linked(&self) -> Result<Vec<SwiftEntity>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let entities = ctx
                .entity_service
                .list_entities()
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            Ok(entities
                .into_iter()
                .filter(|e| e.is_linked())
                .map(|e| SwiftEntity {
                    id: e.id,
                    name: e.name,
                    entity_type: e.entity_type.into(),
                    description: e.description,
                    created_by: e.created_by,
                    created_at: e.created_at,
                    members: e.members,
                    parent_org_id: e.parent_org_id,
                    network_four_words: e.network_four_words,
                    is_local_only: e.is_local_only,
                    linked_at: e.linked_at,
                    last_sync_at: e.last_sync_at,
                })
                .collect())
        })
    }

    // ========================================================================
    // Messaging Sub-Client Methods
    // ========================================================================

    /// Send a message to an entity
    pub fn message_send(
        &self,
        entity_id: String,
        text: String,
        reply_to_id: Option<String>,
    ) -> Result<String, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            let content = MessageContent {
                text,
                author: ctx.four_words.clone(),
                attachments: None,
            };

            let msg = ctx
                .message_service
                .send_message(entity_id, entity.entity_type, content, reply_to_id)
                .await
                .map_err(|e| ClientError::MessageError(e.to_string()))?;

            Ok(msg.metadata.id)
        })
    }

    /// Send a direct message to a peer
    ///
    /// This stores the message locally AND publishes it via gossip pubsub
    /// to the recipient's DM inbox topic.
    pub fn message_send_direct(
        &self,
        recipient_four_words: String,
        text: String,
    ) -> Result<String, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let content = MessageContent {
                text,
                author: ctx.four_words.clone(),
                attachments: None,
            };

            // Create entity ID for DM conversation
            let entity_id = format!("dm:{}", recipient_four_words);

            // Store the message locally using send_message (returns full CRDTMessage)
            let message = ctx
                .message_service
                .send_message(entity_id, EntityType::Person, content, None)
                .await
                .map_err(|e| ClientError::MessageError(e.to_string()))?;

            let message_id = message.metadata.id.clone();

            // Publish message via gossip pubsub to recipient's DM inbox
            if let Some(gossip) = &ctx.gossip {
                // Serialize the message to JSON for transmission
                let message_bytes = serde_json::to_vec(&message).map_err(|e| {
                    ClientError::MessageError(format!("Serialization error: {}", e))
                })?;

                // Publish to recipient's DM topic
                if let Err(e) = gossip
                    .publish_dm(&recipient_four_words, message_bytes)
                    .await
                {
                    // Log the error but don't fail the operation - message is stored locally
                    tracing::warn!(
                        "Failed to publish DM to {}: {} (message stored locally)",
                        recipient_four_words,
                        e
                    );
                } else {
                    tracing::info!(
                        "Published DM to {} via pubsub (msg_id: {})",
                        recipient_four_words,
                        message_id
                    );
                }
            } else {
                tracing::debug!("Gossip not active, message stored locally only");
            }

            Ok(message_id)
        })
    }

    /// Get messages for an entity
    pub fn message_get_for_entity(
        &self,
        entity_id: String,
        limit: Option<u32>,
        before_id: Option<String>,
    ) -> Result<Vec<SwiftMessage>, ClientError> {
        let _ = (limit, before_id); // TODO: Implement pagination
        block_on(async {
            let ctx = self.inner.read().await;

            let response = ctx
                .message_service
                .get_entity_messages(entity_id.clone())
                .await
                .map_err(|e| ClientError::MessageError(e.to_string()))?;

            Ok(response
                .messages
                .into_iter()
                .map(|m| SwiftMessage {
                    id: m.metadata.id,
                    text: m.content.text,
                    author: m.content.author,
                    created_at: m.metadata.timestamp as i64,
                    reply_to_id: m.metadata.reply_to_id,
                    entity_id: entity_id.clone(),
                    reactions: vec![], // TODO: Implement reactions
                })
                .collect())
        })
    }

    /// Get messages in a thread
    pub fn message_get_thread(
        &self,
        entity_id: String,
        parent_message_id: String,
    ) -> Result<Vec<SwiftMessage>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let messages = ctx
                .message_service
                .get_thread_messages(entity_id.clone(), parent_message_id)
                .await
                .map_err(|e| ClientError::MessageError(e.to_string()))?;

            Ok(messages
                .into_iter()
                .map(|m| SwiftMessage {
                    id: m.metadata.id,
                    text: m.content.text,
                    author: m.content.author,
                    created_at: m.metadata.timestamp as i64,
                    reply_to_id: m.metadata.reply_to_id,
                    entity_id: entity_id.clone(),
                    reactions: vec![],
                })
                .collect())
        })
    }

    /// Get direct messages with a peer
    pub fn message_get_direct(
        &self,
        peer_four_words: String,
    ) -> Result<Vec<SwiftMessage>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let response = ctx
                .message_service
                .get_direct_messages(peer_four_words.clone())
                .await
                .map_err(|e| ClientError::MessageError(e.to_string()))?;

            Ok(response
                .messages
                .into_iter()
                .map(|m| SwiftMessage {
                    id: m.metadata.id,
                    text: m.content.text,
                    author: m.content.author,
                    created_at: m.metadata.timestamp as i64,
                    reply_to_id: m.metadata.reply_to_id,
                    entity_id: format!("dm:{}", peer_four_words),
                    reactions: vec![],
                })
                .collect())
        })
    }

    /// Get sync state for an entity
    pub fn message_get_sync_state(&self, entity_id: String) -> Result<SwiftSyncState, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            let sync_state = ctx
                .message_service
                .get_entity_sync_state(entity_id.clone(), entity.entity_type)
                .await
                .map_err(|e| ClientError::MessageError(e.to_string()))?;

            Ok(SwiftSyncState {
                entity_id,
                message_count: sync_state.message_count as u32,
                last_sync_time: sync_state.last_sync_time as i64,
                is_syncing: false,
            })
        })
    }

    // ========================================================================
    // Document Sub-Client Methods
    // ========================================================================

    /// Create a new document
    pub fn document_create(
        &self,
        entity_id: String,
        name: String,
        storage_mode: SwiftStorageMode,
    ) -> Result<String, ClientError> {
        let _ = entity_id; // TODO: Associate with entity
        block_on(async {
            let ctx = self.inner.read().await;

            let mode = match storage_mode {
                SwiftStorageMode::FilesOnly => StorageMode::Files,
                SwiftStorageMode::WebOnly => StorageMode::Web,
                SwiftStorageMode::Both => StorageMode::Both,
            };

            let doc_id = ctx
                .doc_replicator
                .create_document(&name, mode)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))?;

            Ok(doc_id)
        })
    }

    /// Get document info
    pub fn document_get_info(&self, doc_id: String) -> Result<SwiftDocumentInfo, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Check if document exists
            let exists = ctx
                .doc_replicator
                .get_document(&doc_id)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))?;

            if exists.is_none() {
                return Err(ClientError::NotFound(format!(
                    "Document {} not found",
                    doc_id
                )));
            }

            Ok(SwiftDocumentInfo {
                id: doc_id.clone(),
                name: doc_id, // TODO: Store name separately
                entity_id: String::new(),
                created_at: 0,
                modified_at: 0,
                storage_mode: SwiftStorageMode::Both,
            })
        })
    }

    /// List documents (for an entity or all)
    pub fn document_list(
        &self,
        entity_id: Option<String>,
    ) -> Result<Vec<SwiftDocumentInfo>, ClientError> {
        let _ = entity_id; // TODO: Filter by entity
        block_on(async {
            let ctx = self.inner.read().await;

            let doc_ids = ctx
                .doc_replicator
                .list_documents()
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))?;

            Ok(doc_ids
                .into_iter()
                .map(|id| SwiftDocumentInfo {
                    id: id.clone(),
                    name: id,
                    entity_id: String::new(),
                    created_at: 0,
                    modified_at: 0,
                    storage_mode: SwiftStorageMode::Both,
                })
                .collect())
        })
    }

    /// Delete a document
    pub fn document_delete(&self, doc_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.doc_replicator
                .delete_document(&doc_id)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))
        })
    }

    /// Insert text into document at position
    pub fn document_insert_text(
        &self,
        doc_id: String,
        position: u32,
        text: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.doc_replicator
                .insert_text(&doc_id, position as usize, &text)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))
        })
    }

    /// Delete text from document
    pub fn document_delete_text(
        &self,
        doc_id: String,
        position: u32,
        length: u32,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.doc_replicator
                .delete_text(&doc_id, position as usize, length as usize)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))
        })
    }

    /// Get document text
    pub fn document_get_text(&self, doc_id: String) -> Result<String, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.doc_replicator
                .get_text(&doc_id)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))
        })
    }

    /// Get CRDT update for synchronization
    pub fn document_get_crdt_update(&self, doc_id: String) -> Result<Vec<u8>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.doc_replicator
                .get_crdt_update(&doc_id)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))
        })
    }

    /// Apply CRDT update from peer
    pub fn document_apply_crdt_update(
        &self,
        doc_id: String,
        update: Vec<u8>,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.doc_replicator
                .apply_crdt_update(&doc_id, &update)
                .await
                .map_err(|e| ClientError::DocumentError(e.to_string()))
        })
    }

    // ========================================================================
    // Gossip/Network Sub-Client Methods
    // ========================================================================

    /// Start networking with gossip overlay
    pub fn gossip_start(&self, port: Option<u16>) -> Result<String, ClientError> {
        block_on(async {
            let mut ctx = self.inner.write().await;

            let connection_identity = ctx
                .start_networking(port)
                .await
                .map_err(ClientError::NetworkError)?;

            Ok(connection_identity)
        })
    }

    /// Stop networking
    pub fn gossip_stop(&self) -> Result<(), ClientError> {
        block_on(async {
            let mut ctx = self.inner.write().await;

            ctx.stop_networking()
                .await
                .map_err(ClientError::NetworkError)
        })
    }

    /// Connect to a peer by four-word address
    pub fn gossip_connect_to_peer(&self, four_words: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.connect_to_peer(&four_words)
                .await
                .map_err(ClientError::NetworkError)
        })
    }

    /// Find a contact by four-word address
    pub fn gossip_find_contact(&self, four_words: String) -> Result<String, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let peer_id = gossip
                .find_contact(&four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))?;

            Ok(format!("{:?}", peer_id))
        })
    }

    /// Add a contact
    pub fn gossip_add_contact(&self, four_words: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // For now, just add to favourites - full contact requires peer discovery
            gossip
                .add_favourite_contact(four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Get all contacts
    pub fn gossip_get_contacts(&self) -> Result<Vec<SwiftContactInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let contacts = gossip
                .get_contacts()
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))?;

            let favourites = gossip.get_favourite_contacts().await;

            // Get endpoint info from contact store
            let contact_records = gossip.get_all_contact_records().await;

            Ok(contacts
                .into_iter()
                .map(|(fw, _peer_id)| {
                    // Look up endpoint info from contact store
                    let record = contact_records
                        .iter()
                        .find(|r| r.four_words.as_ref() == Some(&fw));
                    SwiftContactInfo {
                        id: record
                            .map(|r| r.id.clone())
                            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                        four_words: Some(fw.clone()),
                        display_name: record.and_then(|r| r.display_name.clone()),
                        is_favourite: favourites.contains(&fw),
                        online: false, // TODO: Check presence
                        is_local_only: record.map(|r| r.is_local_only).unwrap_or(false),
                        linked_at: record.and_then(|r| r.linked_at.map(|t| t as i64)),
                        last_sync_at: record.and_then(|r| r.last_sync_at.map(|t| t as i64)),
                        last_seen_endpoint: record.and_then(|r| r.last_seen_endpoint.clone()),
                        endpoint_updated_at: record
                            .and_then(|r| r.endpoint_updated_at.map(|t| t as i64)),
                        endpoint_success_count: record
                            .map(|r| r.endpoint_success_count)
                            .unwrap_or(0),
                        endpoint_failure_count: record
                            .map(|r| r.endpoint_failure_count)
                            .unwrap_or(0),
                    }
                })
                .collect())
        })
    }

    /// Remove a contact
    pub fn gossip_remove_contact(&self, four_words: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .remove_contact(&four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Add favourite contact
    pub fn gossip_add_favourite_contact(&self, four_words: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .add_favourite_contact(four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Get favourite contacts
    pub fn gossip_get_favourite_contacts(&self) -> Result<Vec<String>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            Ok(gossip.get_favourite_contacts().await)
        })
    }

    /// Get the cached endpoint for a contact (four-word encoded)
    ///
    /// Returns the last-seen endpoint if valid (within TTL and not too many failures).
    /// Returns None if no valid endpoint is cached.
    pub fn gossip_contact_get_endpoint(&self, four_words: String) -> Option<String> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx.gossip.as_ref()?;

            // Get endpoint as SocketAddr
            let addr = gossip.get_contact_endpoint(&four_words).await?;

            // Encode as four words using conn_words
            communitas_core::identity::conn_words(&addr).ok()
        })
    }

    /// Update the cached endpoint for a contact
    ///
    /// The endpoint should be a four-word encoded IP:port string (space-separated).
    pub fn gossip_contact_update_endpoint(
        &self,
        four_words: String,
        endpoint: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // Decode the four-word endpoint to SocketAddr
            let addr = communitas_core::identity::conn_from_words(&endpoint).map_err(|e| {
                ClientError::NetworkError(format!("Invalid endpoint encoding: {}", e))
            })?;

            gossip
                .update_contact_endpoint(&four_words, &addr)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Record a successful connection to a contact
    ///
    /// Updates the endpoint, resets failure count, and increments success count.
    /// The endpoint should be a four-word encoded IP:port string (space-separated).
    pub fn gossip_contact_record_success(
        &self,
        four_words: String,
        endpoint: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // Decode the four-word endpoint to SocketAddr
            let addr = communitas_core::identity::conn_from_words(&endpoint).map_err(|e| {
                ClientError::NetworkError(format!("Invalid endpoint encoding: {}", e))
            })?;

            gossip
                .record_contact_success(&four_words, addr)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Record a connection failure to a contact
    ///
    /// Increments the failure count. After 3 consecutive failures, the cached
    /// endpoint will be skipped in favor of FOAF discovery.
    pub fn gossip_contact_record_failure(&self, four_words: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .record_contact_failure(&four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    // ========================================================================
    // Contact Linking Sub-Client Methods (Local-Only / Network-Linked)
    // ========================================================================

    /// Create a local-only contact (not linked to network identity)
    pub fn contact_create_local(
        &self,
        display_name: String,
    ) -> Result<SwiftContactInfo, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let contact = gossip
                .create_local_contact(display_name)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))?;

            Ok(SwiftContactInfo {
                id: contact.id,
                four_words: contact.four_words,
                display_name: contact.display_name,
                is_favourite: contact.is_favourite,
                online: false,
                is_local_only: contact.is_local_only,
                linked_at: contact.linked_at.map(|t| t as i64),
                last_sync_at: contact.last_sync_at.map(|t| t as i64),
                last_seen_endpoint: contact.last_seen_endpoint,
                endpoint_updated_at: contact.endpoint_updated_at.map(|t| t as i64),
                endpoint_success_count: contact.endpoint_success_count,
                endpoint_failure_count: contact.endpoint_failure_count,
            })
        })
    }

    /// Link a local-only contact to a network identity
    pub fn contact_link_to_network(
        &self,
        contact_id: String,
        four_words: String,
    ) -> Result<SwiftContactInfo, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let contact = gossip
                .link_contact(&contact_id, &four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))?;

            Ok(SwiftContactInfo {
                id: contact.id,
                four_words: contact.four_words,
                display_name: contact.display_name,
                is_favourite: contact.is_favourite,
                online: false,
                is_local_only: contact.is_local_only,
                linked_at: contact.linked_at.map(|t| t as i64),
                last_sync_at: contact.last_sync_at.map(|t| t as i64),
                last_seen_endpoint: contact.last_seen_endpoint,
                endpoint_updated_at: contact.endpoint_updated_at.map(|t| t as i64),
                endpoint_success_count: contact.endpoint_success_count,
                endpoint_failure_count: contact.endpoint_failure_count,
            })
        })
    }

    /// Get all contacts including local-only ones
    pub fn contact_get_all(&self) -> Result<Vec<SwiftContactInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let contacts = gossip.get_all_contact_records().await;

            Ok(contacts
                .into_iter()
                .map(|c| SwiftContactInfo {
                    id: c.id,
                    four_words: c.four_words,
                    display_name: c.display_name,
                    is_favourite: c.is_favourite,
                    online: false, // TODO: Check presence
                    is_local_only: c.is_local_only,
                    linked_at: c.linked_at.map(|t| t as i64),
                    last_sync_at: c.last_sync_at.map(|t| t as i64),
                    last_seen_endpoint: c.last_seen_endpoint,
                    endpoint_updated_at: c.endpoint_updated_at.map(|t| t as i64),
                    endpoint_success_count: c.endpoint_success_count,
                    endpoint_failure_count: c.endpoint_failure_count,
                })
                .collect())
        })
    }

    /// Get all local-only contacts
    pub fn contact_get_local_only(&self) -> Result<Vec<SwiftContactInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let contacts = gossip.get_local_only_contacts().await;

            Ok(contacts
                .into_iter()
                .map(|c| SwiftContactInfo {
                    id: c.id,
                    four_words: c.four_words,
                    display_name: c.display_name,
                    is_favourite: c.is_favourite,
                    online: false,
                    is_local_only: c.is_local_only,
                    linked_at: c.linked_at.map(|t| t as i64),
                    last_sync_at: c.last_sync_at.map(|t| t as i64),
                    last_seen_endpoint: c.last_seen_endpoint,
                    endpoint_updated_at: c.endpoint_updated_at.map(|t| t as i64),
                    endpoint_success_count: c.endpoint_success_count,
                    endpoint_failure_count: c.endpoint_failure_count,
                })
                .collect())
        })
    }

    /// Get all network-linked contacts
    pub fn contact_get_linked(&self) -> Result<Vec<SwiftContactInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let contacts = gossip.get_linked_contacts().await;

            Ok(contacts
                .into_iter()
                .map(|c| SwiftContactInfo {
                    id: c.id,
                    four_words: c.four_words,
                    display_name: c.display_name,
                    is_favourite: c.is_favourite,
                    online: false,
                    is_local_only: c.is_local_only,
                    linked_at: c.linked_at.map(|t| t as i64),
                    last_sync_at: c.last_sync_at.map(|t| t as i64),
                    last_seen_endpoint: c.last_seen_endpoint,
                    endpoint_updated_at: c.endpoint_updated_at.map(|t| t as i64),
                    endpoint_success_count: c.endpoint_success_count,
                    endpoint_failure_count: c.endpoint_failure_count,
                })
                .collect())
        })
    }

    /// Join an entity (subscribe to topic)
    pub fn gossip_join_entity(
        &self,
        entity_id: String,
        entity_type: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .join_entity(&entity_id, &entity_type)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Leave an entity (unsubscribe from topic)
    pub fn gossip_leave_entity(&self, entity_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .leave_entity(&entity_id)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Publish message to entity topic
    pub fn gossip_publish_to_entity(
        &self,
        entity_id: String,
        message: Vec<u8>,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .publish_to_entity(&entity_id, message)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Send direct P2P message
    pub fn gossip_send_p2p_message(
        &self,
        peer_four_words: String,
        message: Vec<u8>,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // Find peer ID first
            let peer_id = gossip
                .find_contact(&peer_four_words)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))?;

            gossip
                .send_direct_message(peer_id, message)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Dial a specific socket address to establish connection
    ///
    /// This is useful for connecting to a peer when you know their IP:port
    /// (e.g., from mDNS discovery or manual entry).
    ///
    /// # Arguments
    /// * `address` - Socket address as string (e.g., "127.0.0.1:49152" or "192.168.1.100:4433")
    pub fn gossip_dial_address(&self, address: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // Parse the address string to SocketAddr
            let addr: std::net::SocketAddr = address.parse().map_err(|e| {
                ClientError::NetworkError(format!("Invalid address '{}': {}", address, e))
            })?;

            gossip
                .dial_address(addr)
                .await
                .map_err(|e| ClientError::NetworkError(e.to_string()))
        })
    }

    /// Get the bound port for this node's networking
    ///
    /// Returns the UDP port that this node is listening on, or None if networking is not active.
    pub fn gossip_get_bound_port(&self) -> Option<u16> {
        block_on(async {
            let ctx = self.inner.read().await;
            ctx.listen_address.map(|addr| addr.port())
        })
    }

    /// Get comprehensive network information
    ///
    /// Returns information about the current networking state including:
    /// - Whether networking is active
    /// - Connection identity (four-word encoded address)
    /// - Listen address and port
    /// - External address (IP:port and four-word encoded)
    /// - Local-only mode status
    pub fn gossip_get_network_info(&self) -> SwiftNetworkInfo {
        block_on(async {
            let ctx = self.inner.read().await;

            let is_local_only = ctx
                .gossip
                .as_ref()
                .map(|g| g.is_local_only_mode())
                .unwrap_or(true);

            // Encode external address to four-word format if available
            let external_address_words = ctx
                .external_address
                .and_then(|addr| communitas_core::conn_words(&addr).ok());

            SwiftNetworkInfo {
                is_active: ctx.is_networking_active(),
                connection_identity: ctx.connection_identity.clone(),
                listen_address: ctx.listen_address.map(|addr| addr.to_string()),
                external_address: ctx.external_address.map(|addr| addr.to_string()),
                external_address_words,
                port: ctx.listen_address.map(|addr| addr.port()),
                four_words: ctx.four_words.clone(),
                is_local_only_mode: is_local_only,
            }
        })
    }

    /// Request external/public address via NAT reflection
    ///
    /// Queries a connected bootstrap node to determine our external IP address
    /// as seen from the internet. Should be called after networking is active.
    pub fn gossip_request_external_address(&self) -> Result<(), ClientError> {
        block_on(async {
            let mut ctx = self.inner.write().await;
            ctx.request_external_address()
                .await
                .map_err(ClientError::NetworkError)
        })
    }

    /// Automatically request external address with retry logic
    ///
    /// This is designed to be called after networking starts. It retries
    /// a few times with delays to allow peer connections to establish.
    /// Non-blocking - failures are logged but don't prevent networking.
    pub fn gossip_auto_request_external_address(&self) -> Result<(), ClientError> {
        block_on(async {
            let mut ctx = self.inner.write().await;
            ctx.auto_request_external_address()
                .await
                .map_err(ClientError::NetworkError)
        })
    }

    // ========================================================================
    // Presence Sub-Client Methods
    // ========================================================================

    /// Start sending presence beacons
    pub fn presence_start_beacons(&self) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .start_presence_beacons()
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))
        })
    }

    /// Stop presence beacons
    pub fn presence_stop_beacons(&self) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            gossip
                .stop_presence_beacons()
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))
        })
    }

    /// Get presence status for a peer
    pub fn presence_get_status(
        &self,
        four_words: String,
    ) -> Result<SwiftPresenceStatus, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // Find peer ID
            let peer_id = gossip
                .find_contact(&four_words)
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))?;

            let is_online = gossip
                .is_peer_online(peer_id)
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))?;

            Ok(if is_online {
                SwiftPresenceStatus::Online
            } else {
                SwiftPresenceStatus::Offline
            })
        })
    }

    /// Check if a peer is online
    pub fn presence_is_online(&self, four_words: String) -> Result<bool, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            // Find peer ID
            let peer_id = gossip
                .find_contact(&four_words)
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))?;

            gossip
                .is_peer_online(peer_id)
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))
        })
    }

    /// Get online peers in an entity
    pub fn presence_get_online_in_entity(
        &self,
        entity_id: String,
    ) -> Result<Vec<SwiftPresenceInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let gossip = ctx
                .gossip
                .as_ref()
                .ok_or_else(|| ClientError::NetworkError("Networking not started".into()))?;

            let peer_ids = gossip
                .get_online_peers(&entity_id)
                .await
                .map_err(|e| ClientError::PresenceError(e.to_string()))?;

            // Convert peer IDs to presence info
            Ok(peer_ids
                .into_iter()
                .map(|_peer_id| SwiftPresenceInfo {
                    four_words: String::new(), // TODO: Reverse lookup
                    status: SwiftPresenceStatus::Online,
                    last_seen: chrono::Utc::now().timestamp(),
                    device_name: None,
                })
                .collect())
        })
    }

    // ========================================================================
    // Kanban Sub-Client Methods
    // ========================================================================

    /// Helper: Check permission for kanban operations on a project
    fn check_kanban_permission(
        &self,
        project_id: &str,
        required: SwiftAccessLevel,
    ) -> Result<(), ClientError> {
        let can_access = self.permission_can_access(
            project_id.to_string(),
            SwiftResourceType::KanbanBoards,
            required,
        )?;

        if !can_access {
            let level_name = match required {
                SwiftAccessLevel::NotVisible => "access",
                SwiftAccessLevel::ReadOnly => "view",
                SwiftAccessLevel::Edit => "edit",
            };
            return Err(ClientError::AuthError(format!(
                "Insufficient permission to {} Kanban boards in this project",
                level_name
            )));
        }

        Ok(())
    }

    /// Create a new Kanban board for a project
    pub fn kanban_create_board(
        &self,
        project_id: String,
        name: String,
        _description: Option<String>,
    ) -> Result<String, ClientError> {
        // Check permission: Edit required to create boards
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            let settings = communitas_kanban::BoardSettings::all_features();
            let board = ctx
                .kanban_service
                .create_board(&project_id, name, Some(settings))
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(board.id)
        })
    }

    /// Get a Kanban board by ID
    pub fn kanban_get_board(&self, board_id: String) -> Result<SwiftKanbanBoard, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            // Check permission: ReadOnly required to view boards
            drop(ctx); // Release read lock before permission check
            self.check_kanban_permission(&board.project_id, SwiftAccessLevel::ReadOnly)?;

            let ctx = self.inner.read().await;
            let columns = ctx
                .kanban_service
                .list_columns(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(SwiftKanbanBoard {
                id: board.id,
                name: board.name,
                description: board.description,
                project_id: board.project_id,
                created_by: board.created_by,
                created_at: board.created_at,
                columns: columns.iter().map(SwiftKanbanColumn::from).collect(),
            })
        })
    }

    /// Add a column to a board
    pub fn kanban_add_column(
        &self,
        board_id: String,
        name: String,
        _color: Option<String>,
        _wip_limit: Option<u32>,
    ) -> Result<String, ClientError> {
        // First, get the board to find its project_id
        let project_id = block_on(async {
            let ctx = self.inner.read().await;
            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;
            Ok::<_, ClientError>(board.project_id)
        })?;

        // Check permission: Edit required to add columns
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            // Note: color and wip_limit can be set via update_column method
            let column = ctx
                .kanban_service
                .add_column(&board_id, name, None)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(column.id)
        })
    }

    /// Create a card in a column
    pub fn kanban_create_card(
        &self,
        board_id: String,
        column_id: String,
        title: String,
        description: Option<String>,
    ) -> Result<String, ClientError> {
        // First, get the board to find its project_id
        let project_id = block_on(async {
            let ctx = self.inner.read().await;
            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;
            Ok::<_, ClientError>(board.project_id)
        })?;

        // Check permission: Edit required to create cards
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            let card = ctx
                .kanban_service
                .create_card(&board_id, &column_id, title, description)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(card.id)
        })
    }

    /// Move a card to a different column or position
    pub fn kanban_move_card(
        &self,
        board_id: String,
        card_id: String,
        to_column_id: String,
        position: u32,
    ) -> Result<(), ClientError> {
        // First, get the board to find its project_id
        let project_id = block_on(async {
            let ctx = self.inner.read().await;
            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;
            Ok::<_, ClientError>(board.project_id)
        })?;

        // Check permission: Edit required to move cards
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            ctx.kanban_service
                .move_card(&board_id, &card_id, &to_column_id, position)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(())
        })
    }

    /// Update a card's title and/or description
    pub fn kanban_update_card(
        &self,
        board_id: String,
        card_id: String,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<(), ClientError> {
        // First, get the board to find its project_id
        let project_id = block_on(async {
            let ctx = self.inner.read().await;
            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;
            Ok::<_, ClientError>(board.project_id)
        })?;

        // Check permission: Edit required to update cards
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            let update = communitas_kanban::CardUpdate {
                title,
                description,
                ..Default::default()
            };

            ctx.kanban_service
                .update_card(&board_id, &card_id, update)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(())
        })
    }

    /// Delete a card
    pub fn kanban_delete_card(
        &self,
        board_id: String,
        card_id: String,
    ) -> Result<(), ClientError> {
        // First, get the board to find its project_id
        let project_id = block_on(async {
            let ctx = self.inner.read().await;
            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;
            Ok::<_, ClientError>(board.project_id)
        })?;

        // Check permission: Edit required to delete cards
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            ctx.kanban_service
                .delete_card(&board_id, &card_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(())
        })
    }

    /// Get a card by ID
    pub fn kanban_get_card(
        &self,
        board_id: String,
        card_id: String,
    ) -> Result<SwiftKanbanCard, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let card = ctx
                .kanban_service
                .get_card(&board_id, &card_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            // Get comment count for this card
            let comment_count = ctx
                .kanban_service
                .list_comments(&board_id, &card_id)
                .map(|comments| comments.len() as u32)
                .unwrap_or(0);

            Ok(SwiftKanbanCard::from_with_count(&card, comment_count))
        })
    }

    /// List all cards in a board (optionally filter by column)
    pub fn kanban_list_cards(
        &self,
        board_id: String,
        column_id: Option<String>,
    ) -> Result<Vec<SwiftKanbanCard>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let cards = if let Some(col_id) = column_id {
                ctx.kanban_service
                    .list_cards_in_column(&board_id, &col_id)
                    .map_err(|e| ClientError::StorageError(e.to_string()))?
            } else {
                // List all cards by iterating columns
                let columns = ctx
                    .kanban_service
                    .list_columns(&board_id)
                    .map_err(|e| ClientError::StorageError(e.to_string()))?;

                let mut all_cards = Vec::new();
                for col in &columns {
                    if let Ok(cards) = ctx.kanban_service.list_cards_in_column(&board_id, &col.id) {
                        all_cards.extend(cards);
                    }
                }
                all_cards
            };

            // Map cards with comment counts
            let swift_cards: Vec<SwiftKanbanCard> = cards
                .iter()
                .map(|card| {
                    let comment_count = ctx
                        .kanban_service
                        .list_comments(&board_id, &card.id)
                        .map(|comments| comments.len() as u32)
                        .unwrap_or(0);
                    SwiftKanbanCard::from_with_count(card, comment_count)
                })
                .collect();

            Ok(swift_cards)
        })
    }

    /// Add a comment to a card
    pub fn kanban_add_comment(
        &self,
        board_id: String,
        card_id: String,
        content: String,
        reply_to_id: Option<String>,
    ) -> Result<String, ClientError> {
        // First, get the board to find its project_id
        let project_id = block_on(async {
            let ctx = self.inner.read().await;
            let board = ctx
                .kanban_service
                .get_board(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;
            Ok::<_, ClientError>(board.project_id)
        })?;

        // Check permission: Edit required to add comments
        self.check_kanban_permission(&project_id, SwiftAccessLevel::Edit)?;

        block_on(async {
            let ctx = self.inner.read().await;

            let comment = ctx
                .kanban_service
                .add_comment(&board_id, &card_id, content, reply_to_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(comment.id)
        })
    }

    /// List all comments on a card
    pub fn kanban_list_comments(
        &self,
        board_id: String,
        card_id: String,
    ) -> Result<Vec<SwiftKanbanComment>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let comments = ctx
                .kanban_service
                .list_comments(&board_id, &card_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(comments.iter().map(SwiftKanbanComment::from).collect())
        })
    }

    /// Get CRDT sync update for a board
    pub fn kanban_get_sync_update(&self, board_id: String) -> Result<Vec<u8>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let update = ctx
                .kanban_service
                .get_full_update(&board_id)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(update)
        })
    }

    /// Apply a CRDT sync update to a board
    pub fn kanban_apply_sync_update(
        &self,
        board_id: String,
        update: Vec<u8>,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.kanban_service
                .apply_update(&board_id, &update)
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(())
        })
    }

    // ========================================================================
    // Invite Sub-Client Methods (Cross-Organization Collaboration)
    // ========================================================================

    /// Create an invite for cross-organization collaboration
    ///
    /// Creates a new invite that allows a recipient to join an entity
    /// (organisation, group, channel, or project) with a specified role.
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity to join
    /// * `entity_id` - ID of the entity to join
    /// * `recipient_id` - Four-word ID of the intended recipient
    /// * `role` - Role to grant (e.g., "member", "admin", "owner")
    /// * `message` - Optional message to include with the invite
    /// * `expires_in_hours` - Optional hours until expiration (None = never expires)
    ///
    /// # Returns
    /// The created invite
    pub fn invite_create(
        &self,
        entity_type: SwiftEntityType,
        entity_id: String,
        recipient_id: String,
        role: String,
        message: Option<String>,
        expires_in_hours: Option<u32>,
    ) -> Result<SwiftInvite, ClientError> {
        use communitas_core::InviteRequest;

        block_on(async {
            let ctx = self.inner.read().await;
            let creator_id = ctx.four_words.clone();

            // Build the invite request
            // InviteRequest::new(recipient_id, entity_type, entity_id, role)
            let mut request = InviteRequest::new(
                &recipient_id,
                entity_type.into(),
                &entity_id,
                &role,
            );

            if let Some(msg) = message {
                request = request.with_message(msg);
            }

            if let Some(hours) = expires_in_hours {
                request = request.with_expiration(hours);
            }

            let invite = ctx
                .invite_service
                .create_invite(&creator_id, request)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(invite.into())
        })
    }

    /// Accept an invite to join an entity
    ///
    /// The current user accepts the invite and joins the entity.
    /// Only the intended recipient can accept an invite.
    ///
    /// # Arguments
    /// * `invite_id` - ID of the invite to accept
    ///
    /// # Returns
    /// Ok if successfully accepted
    pub fn invite_accept(&self, invite_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let recipient_id = ctx.four_words.clone();

            ctx.invite_service
                .accept_invite(&recipient_id, &invite_id)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(())
        })
    }

    /// Reject an invite
    ///
    /// The recipient rejects the invite to join an entity.
    /// Only the intended recipient can reject an invite.
    ///
    /// # Arguments
    /// * `invite_id` - ID of the invite to reject
    ///
    /// # Returns
    /// Ok if successfully rejected
    pub fn invite_reject(&self, invite_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let recipient_id = ctx.four_words.clone();

            ctx.invite_service
                .reject_invite(&recipient_id, &invite_id)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(())
        })
    }

    /// Revoke an invite
    ///
    /// The creator or an admin can revoke a pending invite.
    ///
    /// # Arguments
    /// * `invite_id` - ID of the invite to revoke
    ///
    /// # Returns
    /// Ok if successfully revoked
    pub fn invite_revoke(&self, invite_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let revoker_id = ctx.four_words.clone();

            ctx.invite_service
                .revoke_invite(&revoker_id, &invite_id)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(())
        })
    }

    /// Get an invite by ID
    ///
    /// # Arguments
    /// * `invite_id` - ID of the invite to retrieve
    ///
    /// # Returns
    /// The invite if found
    pub fn invite_get(&self, invite_id: String) -> Result<SwiftInvite, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let invite = ctx
                .invite_service
                .get_invite(&invite_id)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(invite.into())
        })
    }

    /// List all pending invites for the current user
    ///
    /// Returns invites where the current user is the recipient
    /// and the status is Pending.
    ///
    /// # Returns
    /// List of pending invites for this user
    pub fn invite_list_pending(&self) -> Result<Vec<SwiftInvite>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let recipient_id = ctx.four_words.clone();

            let invites = ctx
                .invite_service
                .list_pending_invites(&recipient_id)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(invites.into_iter().map(|i| i.into()).collect())
        })
    }

    /// List all invites for an entity
    ///
    /// Returns all invites (pending, accepted, rejected, etc.) for the specified entity.
    /// Useful for admins to see invite history.
    ///
    /// # Arguments
    /// * `entity_type` - Type of entity
    /// * `entity_id` - ID of the entity
    ///
    /// # Returns
    /// List of all invites for this entity
    pub fn invite_list_for_entity(
        &self,
        entity_type: SwiftEntityType,
        entity_id: String,
    ) -> Result<Vec<SwiftInvite>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;
            let requester_id = ctx.four_words.clone();

            let invites = ctx
                .invite_service
                .list_entity_invites(&requester_id, entity_type.into(), &entity_id)
                .await
                .map_err(|e| ClientError::InviteError(e.to_string()))?;

            Ok(invites.into_iter().map(|i| i.into()).collect())
        })
    }

    // ========================================================================
    // Disk Sub-Client Methods (Per-Entity Virtual Disks)
    // ========================================================================

    /// Write a file to an entity's virtual disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID (four-word format or hex)
    /// * `disk_type` - The type of disk (Private, Public, Shared)
    /// * `path` - Path within the disk (e.g., "/docs/readme.md")
    /// * `data` - File contents as bytes
    pub fn disk_write_file(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
        data: Vec<u8>,
    ) -> Result<SwiftDiskFileInfo, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let info = ctx
                .disk_service
                .write_file(&entity_id, disk_type.into(), &path, &data)
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(SwiftDiskFileInfo::from(info))
        })
    }

    /// Read a file from an entity's virtual disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    /// * `path` - Path within the disk
    ///
    /// # Returns
    /// File contents as bytes
    pub fn disk_read_file(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
    ) -> Result<Vec<u8>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.disk_service
                .read_file(&entity_id, disk_type.into(), &path)
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))
        })
    }

    /// List files in a directory within an entity's virtual disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    /// * `path` - Directory path (use "/" for root)
    pub fn disk_list_files(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
    ) -> Result<Vec<SwiftDiskFileInfo>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let files = ctx
                .disk_service
                .list_files(&entity_id, disk_type.into(), &path)
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(files.into_iter().map(SwiftDiskFileInfo::from).collect())
        })
    }

    /// Delete a file from an entity's virtual disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    /// * `path` - Path to the file to delete
    pub fn disk_delete_file(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            ctx.disk_service
                .delete_file(&entity_id, disk_type.into(), &path)
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))
        })
    }

    /// Get storage statistics for an entity's disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    pub fn disk_get_stats(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
    ) -> Result<SwiftDiskStats, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let stats = ctx
                .disk_service
                .get_stats(&entity_id, disk_type.into())
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(SwiftDiskStats::from(stats))
        })
    }

    /// Create a directory in an entity's virtual disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    /// * `path` - Directory path to create
    pub fn disk_create_directory(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
    ) -> Result<SwiftDiskFileInfo, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let info = ctx
                .disk_service
                .create_directory(&entity_id, disk_type.into(), &path)
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(SwiftDiskFileInfo::from(info))
        })
    }

    /// Check if a file exists in an entity's virtual disk
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    /// * `path` - Path to check
    pub fn disk_file_exists(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
    ) -> bool {
        block_on(async {
            let ctx = self.inner.read().await;
            ctx.disk_service
                .file_exists(&entity_id, disk_type.into(), &path)
                .await
        })
    }

    /// Get file info without reading the contents
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID
    /// * `disk_type` - The type of disk
    /// * `path` - Path to the file
    pub fn disk_get_file_info(
        &self,
        entity_id: String,
        disk_type: SwiftDiskType,
        path: String,
    ) -> Result<SwiftDiskFileInfo, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let info = ctx
                .disk_service
                .get_file_info(&entity_id, disk_type.into(), &path)
                .await
                .map_err(|e| ClientError::StorageError(e.to_string()))?;

            Ok(SwiftDiskFileInfo::from(info))
        })
    }

    // ========================================================================
    // Permission Sub-Client Methods (Phase 3: Granular Per-Resource Permissions)
    // ========================================================================

    /// Get effective permission level for current user on a resource in an entity
    ///
    /// Combines role-based defaults with any member-specific overrides.
    pub fn permission_get_effective(
        &self,
        entity_id: String,
        resource_type: SwiftResourceType,
    ) -> Result<SwiftAccessLevel, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the current user's four_words
            let user_fw = &ctx.profile.id_fw;

            // Get the entity first
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info (including roles)
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Find the member's role
            let role = members
                .iter()
                .find(|m| m.member_id == *user_fw)
                .map(|m| m.role.as_str())
                .unwrap_or("guest");

            // Get role defaults
            let defaults = communitas_core::permissions::role_defaults(role);
            let rt: communitas_core::permissions::ResourceType = resource_type.into();

            // Return the permission from defaults (no overrides stored yet)
            let access = defaults
                .get(&rt)
                .copied()
                .unwrap_or(communitas_core::permissions::AccessLevel::NotVisible);

            Ok(access.into())
        })
    }

    /// Check if current user can perform an action requiring a specific access level
    ///
    /// Returns true if the user's effective permission is >= the required level.
    pub fn permission_can_access(
        &self,
        entity_id: String,
        resource_type: SwiftResourceType,
        required_level: SwiftAccessLevel,
    ) -> Result<bool, ClientError> {
        let effective = self.permission_get_effective(entity_id, resource_type)?;

        // Compare access levels using the Ord implementation
        let effective_core: communitas_core::permissions::AccessLevel = effective.into();
        let required_core: communitas_core::permissions::AccessLevel = required_level.into();

        Ok(effective_core.allows(required_core))
    }

    /// Check if current user can view a resource (requires ReadOnly or Edit)
    pub fn permission_can_view(
        &self,
        entity_id: String,
        resource_type: SwiftResourceType,
    ) -> Result<bool, ClientError> {
        self.permission_can_access(entity_id, resource_type, SwiftAccessLevel::ReadOnly)
    }

    /// Check if current user can edit a resource (requires Edit level)
    pub fn permission_can_edit(
        &self,
        entity_id: String,
        resource_type: SwiftResourceType,
    ) -> Result<bool, ClientError> {
        self.permission_can_access(entity_id, resource_type, SwiftAccessLevel::Edit)
    }

    /// Set permission override for a member (requires admin/owner role)
    ///
    /// This overrides the member's role-based default for the specific resource.
    pub fn permission_set_member_override(
        &self,
        entity_id: String,
        member_four_words: String,
        resource_type: SwiftResourceType,
        level: SwiftAccessLevel,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the entity first
            let user_fw = &ctx.profile.id_fw;
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info (including roles)
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Find the current user's role
            let user_role = members
                .iter()
                .find(|m| m.member_id == *user_fw)
                .map(|m| m.role.as_str())
                .unwrap_or("guest");

            // Only owner and admin can modify permissions
            if user_role != "owner" && user_role != "admin" {
                return Err(ClientError::AuthError(
                    "Only owners and admins can modify member permissions".into(),
                ));
            }

            // Verify target member exists
            if !members.iter().any(|m| m.member_id == member_four_words) {
                return Err(ClientError::NotFound(format!(
                    "Member {} not found in entity",
                    member_four_words
                )));
            }

            // Store override in CRDT entity metadata
            let core_resource: communitas_core::permissions::ResourceType = resource_type.into();
            let core_level: communitas_core::permissions::AccessLevel = level.into();

            ctx.entity_service
                .set_permission_override(
                    entity.entity_type,
                    &entity_id,
                    &member_four_words,
                    core_resource.as_str(),
                    core_level.as_str(),
                )
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            tracing::info!(
                entity_id = %entity_id,
                member = %member_four_words,
                resource = ?resource_type,
                level = ?level,
                "Permission override set in CRDT"
            );

            Ok(())
        })
    }

    /// Remove a permission override for a member, reverting to role default
    pub fn permission_remove_member_override(
        &self,
        entity_id: String,
        member_four_words: String,
        resource_type: SwiftResourceType,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the entity first
            let user_fw = &ctx.profile.id_fw;
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info (including roles)
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Find the current user's role
            let user_role = members
                .iter()
                .find(|m| m.member_id == *user_fw)
                .map(|m| m.role.as_str())
                .unwrap_or("guest");

            if user_role != "owner" && user_role != "admin" {
                return Err(ClientError::AuthError(
                    "Only owners and admins can modify member permissions".into(),
                ));
            }

            // Remove override from CRDT entity metadata
            let core_resource: communitas_core::permissions::ResourceType = resource_type.into();

            ctx.entity_service
                .remove_permission_override(
                    entity.entity_type,
                    &entity_id,
                    &member_four_words,
                    core_resource.as_str(),
                )
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            tracing::info!(
                entity_id = %entity_id,
                member = %member_four_words,
                resource = ?resource_type,
                "Permission override removed from CRDT"
            );

            Ok(())
        })
    }

    /// Get all permission overrides for a specific member
    pub fn permission_get_member_overrides(
        &self,
        entity_id: String,
        member_four_words: String,
    ) -> Result<Vec<SwiftMemberPermission>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the entity first
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Verify member exists
            if !members.iter().any(|m| m.member_id == member_four_words) {
                return Err(ClientError::NotFound(format!(
                    "Member {} not found in entity",
                    member_four_words
                )));
            }

            // Load overrides from CRDT entity metadata
            let overrides = ctx
                .entity_service
                .get_permission_overrides(entity.entity_type, &entity_id, &member_four_words)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Convert to SwiftMemberPermission list
            let permissions: Vec<SwiftMemberPermission> = overrides
                .into_iter()
                .filter_map(|(resource_str, access_str)| {
                    let resource: communitas_core::permissions::ResourceType =
                        resource_str.parse().ok()?;
                    let access: communitas_core::permissions::AccessLevel =
                        access_str.parse().ok()?;
                    Some(SwiftMemberPermission {
                        resource_type: resource.into(),
                        access_level: access.into(),
                    })
                })
                .collect();

            Ok(permissions)
        })
    }

    /// Get the role of a member in an entity
    pub fn permission_get_member_role(
        &self,
        entity_id: String,
        member_four_words: String,
    ) -> Result<String, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the entity first
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info (including roles)
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            let role = members
                .iter()
                .find(|m| m.member_id == member_four_words)
                .map(|m| m.role.clone())
                .ok_or_else(|| {
                    ClientError::NotFound(format!(
                        "Member {} not found in entity",
                        member_four_words
                    ))
                })?;

            Ok(role)
        })
    }

    /// Set the role of a member in an entity (requires admin/owner)
    pub fn permission_set_member_role(
        &self,
        entity_id: String,
        member_four_words: String,
        role: String,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the entity first
            let user_fw = &ctx.profile.id_fw;
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info (including roles)
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Find the current user's role
            let user_role = members
                .iter()
                .find(|m| m.member_id == *user_fw)
                .map(|m| m.role.as_str())
                .unwrap_or("guest");

            if user_role != "owner" && user_role != "admin" {
                return Err(ClientError::AuthError(
                    "Only owners and admins can change member roles".into(),
                ));
            }

            // Validate role is a known standard role
            if !communitas_core::permissions::is_standard_role(&role) {
                return Err(ClientError::EntityError(format!(
                    "Unknown role '{}'. Valid roles: {:?}",
                    role,
                    communitas_core::permissions::standard_roles()
                )));
            }

            // Update role in CRDT entity
            ctx.entity_service
                .set_member_role(entity.entity_type, &entity_id, &member_four_words, &role)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            tracing::info!(
                entity_id = %entity_id,
                member = %member_four_words,
                new_role = %role,
                "Member role changed in CRDT"
            );

            Ok(())
        })
    }

    /// Get all permissions (defaults + overrides) for a member as a list
    pub fn permission_get_all_for_member(
        &self,
        entity_id: String,
        member_four_words: String,
    ) -> Result<Vec<SwiftMemberPermission>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            // Get the entity first
            let entity = ctx
                .entity_service
                .get_entity(&entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Get the member list with full info (including roles)
            let members = ctx
                .entity_service
                .list_members(entity.entity_type, &entity_id)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            let role = members
                .iter()
                .find(|m| m.member_id == member_four_words)
                .map(|m| m.role.as_str())
                .ok_or_else(|| {
                    ClientError::NotFound(format!(
                        "Member {} not found in entity",
                        member_four_words
                    ))
                })?;

            // Get defaults for the role
            let mut permissions_map = communitas_core::permissions::role_defaults(role);

            // Load overrides from CRDT storage and merge
            let overrides = ctx
                .entity_service
                .get_permission_overrides(entity.entity_type, &entity_id, &member_four_words)
                .await
                .map_err(|e| ClientError::EntityError(e.to_string()))?;

            // Apply overrides to defaults
            for (resource_str, access_str) in overrides {
                if let Ok(resource) = resource_str.parse::<communitas_core::permissions::ResourceType>()
                {
                    if let Ok(access) = access_str.parse::<communitas_core::permissions::AccessLevel>()
                    {
                        permissions_map.insert(resource, access);
                    }
                }
            }

            // Convert to SwiftMemberPermission list
            let permissions: Vec<SwiftMemberPermission> = permissions_map
                .into_iter()
                .map(|(rt, level)| SwiftMemberPermission {
                    resource_type: rt.into(),
                    access_level: level.into(),
                })
                .collect();

            Ok(permissions)
        })
    }

    // ========================================================================
    // WebRTC Sub-Client Methods (Voice, Video, Screen Share)
    // ========================================================================

    /// Check if WebRTC service is available
    ///
    /// WebRTC requires active networking (gossip overlay must be running).
    pub fn webrtc_is_available(&self) -> bool {
        block_on(async {
            let ctx = self.inner.read().await;
            ctx.webrtc.is_some()
        })
    }

    /// Initiate a call to another peer
    ///
    /// # Arguments
    /// * `target_four_words` - Four-word address of the peer to call
    /// * `constraints` - Media constraints (audio/video settings)
    ///
    /// # Returns
    /// The call ID for tracking the call
    pub fn webrtc_initiate_call(
        &self,
        target_four_words: String,
        constraints: SwiftMediaConstraints,
    ) -> Result<String, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx.webrtc.as_ref().ok_or_else(|| {
                ClientError::WebRtcError("WebRTC not available (networking not started)".into())
            })?;

            // Convert Swift constraints to core constraints
            let core_constraints = if constraints.has_video {
                communitas_core::webrtc::MediaConstraints::video_call()
            } else {
                communitas_core::webrtc::MediaConstraints::audio_only()
            };

            let call_id = webrtc
                .initiate_call(&target_four_words, core_constraints)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))?;

            Ok(call_id.to_string())
        })
    }

    /// Accept an incoming call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call to accept
    /// * `constraints` - Media constraints for local media
    pub fn webrtc_accept_call(
        &self,
        call_id: String,
        constraints: SwiftMediaConstraints,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            let core_constraints = if constraints.has_video {
                communitas_core::webrtc::MediaConstraints::video_call()
            } else {
                communitas_core::webrtc::MediaConstraints::audio_only()
            };

            webrtc
                .accept_call(core_call_id, core_constraints)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// Reject an incoming call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call to reject
    pub fn webrtc_reject_call(&self, call_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            webrtc
                .reject_call(core_call_id)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// End an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call to end
    pub fn webrtc_end_call(&self, call_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            webrtc
                .end_call(core_call_id)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// Enable or disable video in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    /// * `enabled` - Whether to enable video
    pub fn webrtc_set_video_enabled(
        &self,
        call_id: String,
        enabled: bool,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            webrtc
                .set_video_enabled(core_call_id, enabled)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// Enable or disable audio in an active call (mute/unmute)
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    /// * `enabled` - Whether to enable audio
    pub fn webrtc_set_audio_enabled(
        &self,
        call_id: String,
        enabled: bool,
    ) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            webrtc
                .set_audio_enabled(core_call_id, enabled)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// Start screen sharing in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    ///
    /// Note: Actual screen capture is handled by the Swift layer using ScreenCaptureKit.
    /// This method signals intent and updates state.
    pub fn webrtc_start_screen_share(&self, call_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            webrtc
                .start_screen_share(core_call_id)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// Stop screen sharing in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    pub fn webrtc_stop_screen_share(&self, call_id: String) -> Result<(), ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let uuid = uuid::Uuid::parse_str(&call_id)
                .map_err(|_| ClientError::WebRtcError("Invalid call ID".into()))?;
            let core_call_id = communitas_core::webrtc::CallId(uuid);

            webrtc
                .stop_screen_share(core_call_id)
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))
        })
    }

    /// Get available media devices
    ///
    /// Note: Device enumeration is typically done on the client side using AVFoundation.
    /// This returns an empty list - use Swift's AVCaptureDevice APIs instead.
    pub fn webrtc_get_media_devices(&self) -> Result<Vec<SwiftMediaDevice>, ClientError> {
        block_on(async {
            let ctx = self.inner.read().await;

            let webrtc = ctx
                .webrtc
                .as_ref()
                .ok_or_else(|| ClientError::WebRtcError("WebRTC not available".into()))?;

            let devices = webrtc
                .get_media_devices()
                .await
                .map_err(|e| ClientError::WebRtcError(e.to_string()))?;

            Ok(devices
                .into_iter()
                .map(|d| SwiftMediaDevice {
                    device_id: d.device_id,
                    label: d.label,
                    kind: d.kind,
                })
                .collect())
        })
    }
}

// Private helper methods
impl CommunitasClient {
    async fn get_or_init_auth<'a>(
        &self,
        auth_lock: &'a mut Option<AuthService>,
    ) -> Result<&'a mut AuthService, ClientError> {
        if auth_lock.is_none() {
            let config = StorageConfig {
                vault_dir: self.storage_path.join("vaults"),
                use_keyring: true,
                ..Default::default()
            };

            let storage_manager = EncryptedStorageManager::new(config)
                .await
                .map_err(|e| ClientError::AuthError(e.to_string()))?;

            *auth_lock = Some(AuthService::new(storage_manager));
        }

        auth_lock
            .as_mut()
            .ok_or_else(|| ClientError::AuthError("Auth service not initialized".to_string()))
    }
}

// Legacy compatibility exports
pub use SwiftEntity as Entity;
pub use SwiftMessage as Message;

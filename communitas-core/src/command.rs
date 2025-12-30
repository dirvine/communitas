// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Command/Event/Query Architecture for Headless Core
//!
//! This module implements the "Headless Core with Multi-Adapter" pattern that enables:
//! - GUI adapters (Iced, Swift) to control the application through execute/query/subscribe
//! - AI agents to control the application via MCP server
//! - CLI tools to automate workflows
//! - Test harnesses to verify behavior
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//! │   Iced GUI  │  │  Swift GUI  │  │ MCP Server  │  │     CLI     │
//! │  (Adapter)  │  │  (Adapter)  │  │  (Adapter)  │  │  (Adapter)  │
//! └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
//!        │                │                │                │
//!        └────────────────┼────────────────┼────────────────┘
//!                         │                │
//!                         ▼                ▼
//!               ┌─────────────────────────────────────┐
//!               │        CommunitasApp (Core)         │
//!               │  execute(cmd) / query(q) / sub(s)   │
//!               └─────────────────────────────────────┘
//! ```
//!
//! # Key Principles
//!
//! 1. **All mutations via Commands** - Every state change MUST go through a command.
//!    No adapter should directly mutate state.
//!
//! 2. **Events record what happened** - After a command executes, it produces events
//!    that describe what changed. These events can be replayed for debugging/testing.
//!
//! 3. **Queries for reads** - All state reads go through the Query enum. This enables
//!    caching, access control, and consistent responses across adapters.
//!
//! 4. **Subscriptions for reactivity** - Adapters subscribe to event streams to
//!    update their UI when state changes.

use crate::crdt::EntityType;
use crate::invite::InviteStatus;
use serde::{Deserialize, Serialize};

// ============================================================================
// Commands - All mutations to application state
// ============================================================================

/// Commands represent all possible mutations to the application state.
///
/// Every action that modifies state MUST go through a command. This ensures:
/// - Consistent validation across all adapters
/// - Event sourcing capability (commands produce events)
/// - MCP server can expose all application functionality
/// - Test harnesses can verify behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    // ========================================================================
    // Profile & Identity Commands
    // ========================================================================
    /// Initialize the application with a new or existing identity
    Initialize {
        four_words: String,
        display_name: String,
        device_name: String,
        storage_dir: String,
    },

    /// Update the user's display name
    UpdateDisplayName { display_name: String },

    // ========================================================================
    // Networking Commands
    // ========================================================================
    /// Start P2P networking with gossip overlay
    StartNetworking { preferred_port: Option<u16> },

    /// Stop P2P networking gracefully
    StopNetworking,

    /// Connect to a peer by their four-word identity
    ConnectToPeer { peer_four_words: String },

    /// Request external address discovery via NAT reflection
    RequestExternalAddress,

    // ========================================================================
    // Entity Management Commands
    // ========================================================================
    /// Create a new entity (organization, group, channel, project)
    CreateEntity {
        name: String,
        entity_type: EntityType,
        description: Option<String>,
        initial_members: Vec<String>,
    },

    /// Create a local-only entity (not yet linked to network)
    CreateLocalEntity {
        name: String,
        entity_type: EntityType,
        description: Option<String>,
    },

    /// Link a local entity to a network identity
    LinkEntityToNetwork {
        entity_id: String,
        four_words: String,
    },

    /// Mark an entity as synced with network
    MarkEntitySynced { entity_id: String },

    /// Set parent organization for a child entity
    SetParentOrganization {
        entity_id: String,
        parent_org_id: String,
    },

    // ========================================================================
    // Member Management Commands
    // ========================================================================
    /// Add a member to an entity
    AddMember {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        role: String,
    },

    /// Remove a member from an entity
    RemoveMember {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
    },

    /// Remove a member from organization and all child entities (cascade)
    RemoveOrganizationMember { org_id: String, member_id: String },

    /// Set a member's role
    SetMemberRole {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        new_role: String,
    },

    // ========================================================================
    // Permission Commands
    // ========================================================================
    /// Set a permission override for a member
    SetPermissionOverride {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        resource_type: String,
        access_level: String,
    },

    /// Remove a permission override for a member
    RemovePermissionOverride {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        resource_type: String,
    },

    // ========================================================================
    // Messaging Commands
    // ========================================================================
    /// Send a message to an entity
    SendMessage {
        entity_id: String,
        entity_type: EntityType,
        text: String,
        author: String,
        reply_to_id: Option<String>,
        attachments: Option<Vec<String>>,
    },

    /// Send a direct message to one or more recipients
    SendDirectMessage {
        recipients: Vec<String>,
        text: String,
        author: String,
    },

    // ========================================================================
    // Invite Commands
    // ========================================================================
    /// Create an invite for someone to join an entity
    CreateInvite {
        recipient_id: String,
        entity_type: EntityType,
        entity_id: String,
        role: String,
        message: Option<String>,
        expires_in_hours: Option<u32>,
    },

    /// Accept an invite
    AcceptInvite { invite_id: String },

    /// Reject an invite
    RejectInvite { invite_id: String },

    /// Revoke an invite
    RevokeInvite { invite_id: String },

    // ========================================================================
    // Virtual Disk Commands
    // ========================================================================
    /// Write a file to an entity's virtual disk
    WriteFile {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
        data: Vec<u8>,
    },

    /// Delete a file from an entity's virtual disk
    DeleteFile {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
    },

    /// Create a directory in an entity's virtual disk
    CreateDirectory {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
    },

    // ========================================================================
    // Kanban Commands
    // ========================================================================
    /// Create a new Kanban board
    CreateKanbanBoard {
        entity_id: String,
        board_name: String,
        description: Option<String>,
    },

    /// Create a column in a Kanban board
    CreateKanbanColumn {
        board_id: String,
        column_name: String,
        position: Option<u32>,
    },

    /// Create a card in a Kanban column
    CreateKanbanCard {
        board_id: String,
        column_id: String,
        title: String,
        description: Option<String>,
        assignee: Option<String>,
    },

    /// Move a Kanban card to a different column
    MoveKanbanCard {
        board_id: String,
        card_id: String,
        target_column_id: String,
        position: Option<u32>,
    },

    /// Update a Kanban card
    UpdateKanbanCard {
        board_id: String,
        card_id: String,
        title: Option<String>,
        description: Option<String>,
        assignee: Option<String>,
    },

    /// Delete a Kanban card
    DeleteKanbanCard { board_id: String, card_id: String },

    // ========================================================================
    // WebRTC Commands (Voice/Video/Screen)
    // ========================================================================
    /// Start a voice/video call
    StartCall {
        entity_id: String,
        video_enabled: bool,
    },

    /// Join an existing call
    JoinCall { call_id: String },

    /// Leave a call
    LeaveCall { call_id: String },

    /// Toggle video in a call
    ToggleVideo { call_id: String, enabled: bool },

    /// Toggle audio in a call
    ToggleAudio { call_id: String, enabled: bool },

    /// Start screen sharing
    StartScreenShare { call_id: String },

    /// Stop screen sharing
    StopScreenShare { call_id: String },
}

/// Disk type argument for commands (serializable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskTypeArg {
    Private,
    Public,
    Shared,
}

// ============================================================================
// Events - What happened as a result of commands
// ============================================================================

/// Events describe what changed as a result of a command.
///
/// Events are:
/// - Immutable records of what happened
/// - Used to update UI reactively
/// - Can be replayed for debugging/testing
/// - Broadcast to subscribers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    // ========================================================================
    // Profile & Identity Events
    // ========================================================================
    /// Application was initialized
    Initialized {
        four_words: String,
        display_name: String,
        device_name: String,
    },

    /// Display name was updated
    DisplayNameUpdated { old_name: String, new_name: String },

    // ========================================================================
    // Networking Events
    // ========================================================================
    /// Networking started
    NetworkingStarted {
        listen_address: String,
        connection_identity: String,
    },

    /// Networking stopped
    NetworkingStopped,

    /// Connected to a peer
    PeerConnected { peer_four_words: String },

    /// External address discovered
    ExternalAddressDiscovered { address: String },

    /// Connection failed
    ConnectionFailed {
        peer_four_words: String,
        reason: String,
    },

    // ========================================================================
    // Entity Events
    // ========================================================================
    /// Entity was created
    EntityCreated {
        entity_id: String,
        name: String,
        entity_type: EntityType,
        created_by: String,
    },

    /// Entity was linked to network
    EntityLinkedToNetwork {
        entity_id: String,
        four_words: String,
    },

    /// Entity was synced with network
    EntitySynced { entity_id: String },

    /// Parent organization was set
    ParentOrganizationSet {
        entity_id: String,
        parent_org_id: String,
    },

    // ========================================================================
    // Member Events
    // ========================================================================
    /// Member was added to an entity
    MemberAdded {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        role: String,
    },

    /// Member was removed from an entity
    MemberRemoved {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
    },

    /// Member's role was changed
    MemberRoleChanged {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        old_role: String,
        new_role: String,
    },

    /// Member was removed from organization and child entities
    OrganizationMemberRemoved {
        org_id: String,
        member_id: String,
        removed_from: Vec<(EntityType, String)>,
    },

    // ========================================================================
    // Permission Events
    // ========================================================================
    /// Permission override was set
    PermissionOverrideSet {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        resource_type: String,
        access_level: String,
    },

    /// Permission override was removed
    PermissionOverrideRemoved {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
        resource_type: String,
    },

    // ========================================================================
    // Message Events
    // ========================================================================
    /// Message was sent
    MessageSent {
        message_id: String,
        entity_id: String,
        entity_type: EntityType,
        author: String,
        text: String,
    },

    /// Message was received
    MessageReceived {
        message_id: String,
        entity_id: String,
        entity_type: EntityType,
        author: String,
        text: String,
    },

    /// Direct message was sent
    DirectMessageSent {
        message_ids: Vec<String>,
        recipients: Vec<String>,
    },

    // ========================================================================
    // Invite Events
    // ========================================================================
    /// Invite was created
    InviteCreated {
        invite_id: String,
        recipient_id: String,
        entity_type: EntityType,
        entity_id: String,
        role: String,
    },

    /// Invite was accepted
    InviteAccepted {
        invite_id: String,
        recipient_id: String,
        entity_id: String,
    },

    /// Invite was rejected
    InviteRejected { invite_id: String },

    /// Invite was revoked
    InviteRevoked { invite_id: String },

    // ========================================================================
    // Virtual Disk Events
    // ========================================================================
    /// File was written
    FileWritten {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
        size_bytes: u64,
    },

    /// File was deleted
    FileDeleted {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
    },

    /// Directory was created
    DirectoryCreated {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
    },

    // ========================================================================
    // Kanban Events
    // ========================================================================
    /// Kanban board was created
    KanbanBoardCreated {
        board_id: String,
        entity_id: String,
        board_name: String,
    },

    /// Kanban column was created
    KanbanColumnCreated {
        column_id: String,
        board_id: String,
        column_name: String,
    },

    /// Kanban card was created
    KanbanCardCreated {
        card_id: String,
        column_id: String,
        title: String,
    },

    /// Kanban card was moved
    KanbanCardMoved {
        card_id: String,
        from_column_id: String,
        to_column_id: String,
    },

    /// Kanban card was updated
    KanbanCardUpdated { card_id: String },

    /// Kanban card was deleted
    KanbanCardDeleted { card_id: String },

    // ========================================================================
    // WebRTC Events
    // ========================================================================
    /// Call started
    CallStarted { call_id: String, entity_id: String },

    /// Joined a call
    CallJoined { call_id: String },

    /// Left a call
    CallLeft { call_id: String },

    /// Video toggled
    VideoToggled { call_id: String, enabled: bool },

    /// Audio toggled
    AudioToggled { call_id: String, enabled: bool },

    /// Screen sharing started
    ScreenShareStarted { call_id: String },

    /// Screen sharing stopped
    ScreenShareStopped { call_id: String },

    // ========================================================================
    // Error Events
    // ========================================================================
    /// An error occurred while processing a command
    CommandFailed { command_type: String, error: String },
}

// ============================================================================
// Queries - All reads from application state
// ============================================================================

/// Queries represent all possible reads from the application state.
///
/// All state reads should go through queries to enable:
/// - Consistent responses across adapters
/// - Access control enforcement
/// - Caching strategies
/// - MCP server exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Query {
    // ========================================================================
    // Profile & Identity Queries
    // ========================================================================
    /// Get the current user profile
    GetProfile,

    /// Check if networking is active
    IsNetworkingActive,

    /// Get connection identity
    GetConnectionIdentity,

    /// Get external address
    GetExternalAddress,

    // ========================================================================
    // Entity Queries
    // ========================================================================
    /// Get an entity by ID
    GetEntity { entity_id: String },

    /// List all entities
    ListEntities,

    /// List entities by type
    ListEntitiesByType { entity_type: EntityType },

    /// List child entities of an organization
    ListChildEntities { org_id: String },

    // ========================================================================
    // Member Queries
    // ========================================================================
    /// List members of an entity
    ListMembers {
        entity_type: EntityType,
        entity_id: String,
    },

    /// Get a member's role
    GetMemberRole {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
    },

    /// Get permission overrides for a member
    GetPermissionOverrides {
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
    },

    // ========================================================================
    // Message Queries
    // ========================================================================
    /// Get messages for an entity
    GetEntityMessages { entity_id: String },

    /// Get thread messages (replies to a parent message)
    GetThreadMessages {
        entity_id: String,
        parent_message_id: String,
    },

    /// Get direct messages with a peer
    GetDirectMessages { other_peer_id: String },

    /// Get entity sync state
    GetEntitySyncState {
        entity_id: String,
        entity_type: EntityType,
    },

    // ========================================================================
    // Invite Queries
    // ========================================================================
    /// Get an invite by ID
    GetInvite { invite_id: String },

    /// List pending invites for current user
    ListPendingInvites,

    /// List invites sent by current user
    ListSentInvites,

    // ========================================================================
    // Virtual Disk Queries
    // ========================================================================
    /// Read a file from an entity's virtual disk
    ReadFile {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
    },

    /// List files in a directory
    ListFiles {
        entity_id: String,
        disk_type: DiskTypeArg,
        path: String,
    },

    /// Get disk statistics
    GetDiskStats {
        entity_id: String,
        disk_type: DiskTypeArg,
    },

    // ========================================================================
    // Kanban Queries
    // ========================================================================
    /// Get a Kanban board
    GetKanbanBoard { board_id: String },

    /// List Kanban boards for an entity
    ListKanbanBoards { entity_id: String },

    /// Get a Kanban card
    GetKanbanCard { board_id: String, card_id: String },

    // ========================================================================
    // Presence Queries
    // ========================================================================
    /// Get presence info for a peer
    GetPresence { peer_id: String },

    /// List online peers
    ListOnlinePeers,

    // ========================================================================
    // WebRTC Queries
    // ========================================================================
    /// List active calls
    ListActiveCalls,

    /// Get call participants
    GetCallParticipants { call_id: String },
}

// ============================================================================
// Query Responses
// ============================================================================

/// Response types for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryResponse {
    /// Profile information
    Profile {
        four_words: String,
        display_name: String,
        device_name: String,
        device_type: String,
    },

    /// Boolean response
    Bool(bool),

    /// Optional string response
    OptionalString(Option<String>),

    /// Entity information
    Entity(EntityResponse),

    /// List of entities
    EntityList(Vec<EntityResponse>),

    /// List of members
    MemberList(Vec<MemberResponse>),

    /// Member role
    MemberRole(String),

    /// Permission overrides
    PermissionOverrides(Vec<(String, String)>),

    /// Messages
    Messages(Vec<MessageResponse>),

    /// Sync state
    SyncState(SyncStateResponse),

    /// Invite information
    Invite(InviteResponse),

    /// List of invites
    InviteList(Vec<InviteResponse>),

    /// File contents
    FileContents(Vec<u8>),

    /// File list
    FileList(Vec<FileInfoResponse>),

    /// Disk statistics
    DiskStats(DiskStatsResponse),

    /// Kanban board
    KanbanBoard(KanbanBoardResponse),

    /// List of Kanban boards
    KanbanBoardList(Vec<KanbanBoardResponse>),

    /// Kanban card
    KanbanCard(KanbanCardResponse),

    /// Presence information
    Presence(PresenceResponse),

    /// List of peer IDs
    PeerList(Vec<String>),

    /// Call information
    CallList(Vec<CallResponse>),

    /// Call participants
    CallParticipants(Vec<String>),
}

// ============================================================================
// Response Types
// ============================================================================

/// Entity response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub member_count: usize,
    pub parent_org_id: Option<String>,
    pub network_four_words: Option<String>,
    pub is_local_only: bool,
}

/// Member response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberResponse {
    pub member_id: String,
    pub role: String,
    pub joined_at: i64,
}

/// Message response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub entity_id: String,
    pub author: String,
    pub text: String,
    pub timestamp: i64,
    pub reply_to_id: Option<String>,
}

/// Sync state response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStateResponse {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub message_count: usize,
    pub last_sync_time: u64,
}

/// Invite response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteResponse {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub role: String,
    pub status: InviteStatus,
    pub message: Option<String>,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

/// File info response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoResponse {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub modified_at: i64,
}

/// Disk stats response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStatsResponse {
    pub entity_id: String,
    pub disk_type: DiskTypeArg,
    pub used_bytes: u64,
    pub file_count: u32,
    pub dir_count: u32,
}

/// Kanban board response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanBoardResponse {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub description: Option<String>,
    pub column_count: usize,
}

/// Kanban card response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanCardResponse {
    pub id: String,
    pub column_id: String,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub position: u32,
}

/// Presence response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceResponse {
    pub peer_id: String,
    pub status: String,
    pub last_seen: i64,
}

/// Call response data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResponse {
    pub id: String,
    pub entity_id: String,
    pub participant_count: usize,
    pub started_at: i64,
}

// ============================================================================
// Subscriptions - For reactive updates
// ============================================================================

/// Subscription types for reactive updates
///
/// Adapters can subscribe to specific event streams to receive
/// real-time updates when state changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Subscription {
    /// Subscribe to all events
    AllEvents,

    /// Subscribe to events for a specific entity
    EntityEvents { entity_id: String },

    /// Subscribe to all message events
    MessageEvents,

    /// Subscribe to messages for a specific entity
    EntityMessages { entity_id: String },

    /// Subscribe to presence updates
    PresenceUpdates,

    /// Subscribe to invite events
    InviteEvents,

    /// Subscribe to networking events
    NetworkingEvents,

    /// Subscribe to Kanban events for an entity
    KanbanEvents { entity_id: String },

    /// Subscribe to call events
    CallEvents,
}

// ============================================================================
// Result types
// ============================================================================

/// Result of executing a command
pub type CommandResult = Result<Vec<Event>, CommandError>;

/// Result of running a query
pub type QueryResult = Result<QueryResponse, QueryError>;

/// Error type for command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandError {
    pub command_type: String,
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.command_type, self.message, self.code)
    }
}

impl std::error::Error for CommandError {}

/// Error type for query execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryError {
    pub query_type: String,
    pub message: String,
    pub code: String,
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.query_type, self.message, self.code)
    }
}

impl std::error::Error for QueryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = Command::CreateEntity {
            name: "Test Org".to_string(),
            entity_type: EntityType::Organisation,
            description: Some("A test organization".to_string()),
            initial_members: vec!["alice-bob-charlie-dave".to_string()],
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: Command = serde_json::from_str(&json).unwrap();

        match deserialized {
            Command::CreateEntity {
                name, entity_type, ..
            } => {
                assert_eq!(name, "Test Org");
                assert_eq!(entity_type, EntityType::Organisation);
            }
            _ => panic!("Wrong command type"),
        }
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::EntityCreated {
            entity_id: "abc123".to_string(),
            name: "Test Org".to_string(),
            entity_type: EntityType::Organisation,
            created_by: "alice-bob-charlie-dave".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        match deserialized {
            Event::EntityCreated {
                entity_id, name, ..
            } => {
                assert_eq!(entity_id, "abc123");
                assert_eq!(name, "Test Org");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_query_serialization() {
        let query = Query::GetEntity {
            entity_id: "abc123".to_string(),
        };

        let json = serde_json::to_string(&query).unwrap();
        let deserialized: Query = serde_json::from_str(&json).unwrap();

        match deserialized {
            Query::GetEntity { entity_id } => {
                assert_eq!(entity_id, "abc123");
            }
            _ => panic!("Wrong query type"),
        }
    }

    #[test]
    fn test_subscription_serialization() {
        let sub = Subscription::EntityMessages {
            entity_id: "abc123".to_string(),
        };

        let json = serde_json::to_string(&sub).unwrap();
        let deserialized: Subscription = serde_json::from_str(&json).unwrap();

        match deserialized {
            Subscription::EntityMessages { entity_id } => {
                assert_eq!(entity_id, "abc123");
            }
            _ => panic!("Wrong subscription type"),
        }
    }
}

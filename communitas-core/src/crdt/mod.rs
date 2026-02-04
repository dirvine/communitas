// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Pure CRDT Infrastructure for Communitas
//!
//! This module provides a complete offline-first, CRDT-based architecture
//! for distributed collaboration without central servers.
//!
//! ## Architecture
//!
//! - **Documents**: Entity structures (Member, Channel, Org, etc.) stored as Yrs documents
//! - **Operations**: CRDT primitives (LWW, Counter, OR-Set, Tombstone)
//! - **Conflict Resolution**: Automatic and manual conflict handling strategies
//! - **Offline Queue**: Operation queue for offline-first functionality
//!
//! ## Key Concepts
//!
//! ### Last-Write-Wins (LWW)
//! Used for metadata fields that should use the most recent value:
//! - Channel names, descriptions
//! - User display names, avatars
//! - Organization settings
//!
//! ### CRDT Set (OR-Set)
//! Used for collections where add/remove operations can happen concurrently:
//! - Channel members
//! - Organization members
//! - Group participants
//!
//! ### Counter
//! Used for aggregates that should never lose increments:
//! - Reply counts on threads
//! - View counts
//! - Reaction counts
//!
//! ### Tombstones
//! Used for soft deletes that replicate across peers:
//! - Deleted messages
//! - Deleted files
//! - Deactivated entities
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use communitas_core::crdt::{
//!     documents::{CrdtDocument, MemberDocument},
//!     operations::LamportTimestamp,
//!     offline_queue::{OfflineQueue, OperationBuilder},
//! };
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create a new member document
//! let member = MemberDocument {
//!     id: "member-123".to_string(),
//!     pubkey_hex: "abcd1234".to_string(),
//!     display_name: "Alice".to_string(),
//!     email: Some("alice@example.com".to_string()),
//!     bio: None,
//!     avatar_url: None,
//!     created_at: 1234567890,
//!     personal_disk_id: "disk-456".to_string(),
//!     website_root: None,
//! };
//!
//! // Create Yrs document
//! let doc = MemberDocument::create_document(&member.id)?;
//! member.update_document(&doc)?;
//!
//! // Queue an update operation for offline processing
//! let mut queue = OfflineQueue::new();
//! let op = OperationBuilder::new("member", "member-123", "peer-id")
//!     .update("display_name".to_string(), serde_json::json!("Alice Smith"));
//! queue.enqueue(op);
//! # Ok(())
//! # }
//! ```

pub mod conflict_resolution;
pub mod documents;
pub mod offline_queue;
pub mod operations;

// Re-export key types for convenience
pub use conflict_resolution::{
    ConflictDetector, ConflictNotification, ConflictResolution, ConflictResolver, StateMachine,
    call_state_machine, issue_state_machine,
};
pub use documents::{ChannelDocument, CrdtDocument, MemberDocument, OrganizationDocument};
pub use offline_queue::{OfflineQueue, OperationBuilder, OperationType, QueuedOperation};
pub use operations::{Counter, LWWRegister, LamportTimestamp, ORSet, Tombstone};

// Re-export legacy CRDT types for backwards compatibility during migration
pub use crate::legacy_crdt::{
    CRDTMessage, CanvasCursorUpdate, CanvasOperation, CanvasOperationType, CanvasStateRequest,
    CanvasStateResponse, ClockOrdering, EntitySyncState, EntityType, GossipMessageType,
    LocalMessageState, MemberSyncRequest, MemberSyncResponse, MemberUpdate, MemberUpdateAction,
    MessageContent, MessageMetadata, MessageStatus, MissingRange, PeerInfo, PeerListRequest,
    PeerListResponse, Reaction, SyncRequest, SyncResponse, VectorClock, sort_messages_causally,
};

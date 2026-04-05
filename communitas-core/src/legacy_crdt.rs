// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! CRDT-based Message Synchronization
//!
//! Implements Conflict-free Replicated Data Types for distributed messaging:
//! - Vector clocks for causal ordering
//! - Lamport timestamps for total ordering
//! - Out-of-order message detection
//! - Missing message synchronization
//! - Causal consistency enforcement
//!
//! All entities (contacts, groups, projects, orgs, channels) use the same CRDT pattern.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Vector Clock - Tracks logical time for each peer
/// Maps peer ID (four-word address) -> logical timestamp
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VectorClock(pub BTreeMap<String, u64>);

impl VectorClock {
    /// Create a new empty vector clock
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    /// Increment the clock for a specific peer
    pub fn increment(&mut self, peer_id: &str) {
        let counter = self.0.entry(peer_id.to_string()).or_insert(0);
        *counter += 1;
    }

    /// Merge two vector clocks (take max for each peer)
    pub fn merge(&mut self, other: &VectorClock) {
        for (peer, timestamp) in &other.0 {
            let entry = self.0.entry(peer.clone()).or_insert(0);
            *entry = (*entry).max(*timestamp);
        }
    }

    /// Compare two vector clocks for causal ordering
    pub fn compare(&self, other: &VectorClock) -> ClockOrdering {
        let mut self_less = false;
        let mut other_less = false;

        // Get all peers from both clocks
        let mut all_peers: std::collections::HashSet<&String> = self.0.keys().collect();
        all_peers.extend(other.0.keys());

        for peer in all_peers {
            let self_val = self.0.get(peer).copied().unwrap_or(0);
            let other_val = other.0.get(peer).copied().unwrap_or(0);

            match self_val.cmp(&other_val) {
                Ordering::Less => other_less = true,
                Ordering::Greater => self_less = true,
                Ordering::Equal => {}
            }
        }

        match (self_less, other_less) {
            (true, true) => ClockOrdering::Concurrent, // Neither happened before
            (true, false) => ClockOrdering::After,     // self is after other
            (false, true) => ClockOrdering::Before,    // self is before other
            (false, false) => ClockOrdering::Equal,    // Same logical time
        }
    }

    /// Check if we have all causal dependencies for a message with this clock
    pub fn has_dependencies(&self, message_clock: &VectorClock) -> bool {
        for (peer, timestamp) in &message_clock.0 {
            let our_timestamp = self.0.get(peer).copied().unwrap_or(0);

            // If message has timestamp N for peer P, we must have seen 0..N-1
            if our_timestamp < timestamp.saturating_sub(1) {
                return false; // Missing events from this peer
            }
        }
        true
    }

    /// Get ranges of missing messages by comparing with another clock
    pub fn get_missing_ranges(&self, remote: &VectorClock) -> Vec<MissingRange> {
        let mut missing = Vec::new();

        for (peer_id, remote_ts) in &remote.0 {
            let local_ts = self.0.get(peer_id).copied().unwrap_or(0);

            if *remote_ts > local_ts {
                missing.push(MissingRange {
                    peer_id: peer_id.clone(),
                    from_timestamp: local_ts + 1,
                    to_timestamp: *remote_ts,
                });
            }
        }

        missing
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Clock comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockOrdering {
    Before,     // First clock happened before second
    After,      // First clock happened after second
    Concurrent, // Clocks are concurrent (conflict)
    Equal,      // Clocks are identical
}

/// Range of missing messages from a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingRange {
    pub peer_id: String,
    pub from_timestamp: u64,
    pub to_timestamp: u64,
}

/// Message metadata for CRDT synchronization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageMetadata {
    pub id: String,                          // Unique message ID
    pub entity_id: String,                   // Entity this message belongs to
    pub entity_type: EntityType,             // Type of entity
    pub author_peer_id: String,              // Four-word address of sender
    pub vector_clock: VectorClock,           // Causal ordering
    pub lamport_clock: u64,                  // Total ordering fallback
    pub timestamp: u64,                      // Unix timestamp (wallclock, reference only)
    pub previous_message_id: Option<String>, // Causal parent in thread
    pub reply_to_id: Option<String>,         // If replying to specific message
}

// EntityType has been extracted to crate::entity_type
pub use crate::entity_type::EntityType;

/// Complete message with CRDT metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CRDTMessage {
    /// Core content
    pub content: MessageContent,

    /// CRDT synchronization metadata
    pub metadata: MessageMetadata,

    /// Local UI state (not synced across peers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_state: Option<LocalMessageState>,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageContent {
    pub text: String,
    pub author: String, // Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<Attachment>>,
}

/// File attachment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Attachment {
    pub attachment_type: AttachmentType,
    pub url: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    File,
    Image,
    Video,
}

/// Local message state (UI only, not synced)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalMessageState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MessageStatus>,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_reply_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Sent,
    Delivered,
    Read,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reaction {
    pub emoji: String,
    pub count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_reacted: Option<bool>,
    pub peer_ids: Vec<String>,
}

/// Sync request - ask a peer for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub requester_peer_id: String,
    pub vector_clock: VectorClock,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_message_ids: Option<Vec<String>>,
    #[serde(default)]
    pub request_id: u64,
}

/// Sync response - reply with messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub messages: Vec<CRDTMessage>,
    pub vector_clock: VectorClock,
}

/// Member update message - add or remove a member for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberUpdate {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub member_id: String,
    pub role: Option<String>,
    pub updated_by: String,
    pub action: MemberUpdateAction,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MemberUpdateAction {
    Add,
    Remove,
}

/// Request a membership snapshot for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSyncRequest {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub requester_peer_id: String,
}

/// Response with member updates representing the current membership state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberSyncResponse {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub responder_peer_id: String,
    pub updates: Vec<MemberUpdate>,
}

/// Entity synchronization state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySyncState {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub vector_clock: VectorClock,
    pub last_sync_time: u64,
    pub message_count: usize,
    pub missing_messages: Vec<String>,
    pub out_of_order_messages: Vec<String>,
}

// ============================================================================
// Peer Discovery Messages (Phase 2: Bootstrap Node Enhancement)
// ============================================================================

/// Information about a peer for sharing in peer lists
///
/// Contains the essential information needed to connect to and evaluate a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Socket address (IP:port) for connecting to this peer
    pub addr: String,

    /// Quality score (0.0-1.0) based on connection success rate and latency
    /// Higher is better. Peers with scores below 0.3 should be avoided.
    pub score: f64,

    /// NAT type classification (if known)
    /// - "open" = directly reachable
    /// - "symmetric" = requires relay
    /// - "unknown" = not yet determined
    #[serde(default)]
    pub nat_class: Option<String>,

    /// Peer roles/capabilities (if known)
    /// - "bootstrap" = can serve as introducer node
    /// - "relay" = can relay for NAT-restricted peers
    #[serde(default)]
    pub roles: Vec<String>,
}

impl PeerInfo {
    /// Create a new PeerInfo with just an address
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            score: 0.5, // Default neutral score
            nat_class: None,
            roles: Vec::new(),
        }
    }

    /// Create with full information
    pub fn with_details(
        addr: impl Into<String>,
        score: f64,
        nat_class: Option<String>,
        roles: Vec<String>,
    ) -> Self {
        Self {
            addr: addr.into(),
            score: score.clamp(0.0, 1.0),
            nat_class,
            roles,
        }
    }
}

/// Request for a list of known peers
///
/// Sent by new nodes to bootstrap nodes to discover additional peers.
/// Bootstrap nodes respond with their best-quality cached peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerListRequest {
    /// Maximum number of peers to return (typically 10-20)
    pub max_peers: u8,

    /// Optional: request only peers with specific roles
    #[serde(default)]
    pub required_roles: Vec<String>,

    /// Optional: requester's NAT class to filter compatible peers
    #[serde(default)]
    pub requester_nat_class: Option<String>,
}

impl PeerListRequest {
    /// Create a simple request for N peers
    pub fn new(max_peers: u8) -> Self {
        Self {
            max_peers,
            required_roles: Vec::new(),
            requester_nat_class: None,
        }
    }

    /// Create a request filtering for specific roles
    pub fn with_roles(max_peers: u8, roles: Vec<String>) -> Self {
        Self {
            max_peers,
            required_roles: roles,
            requester_nat_class: None,
        }
    }
}

/// Response containing a list of known peers
///
/// Returned by bootstrap nodes in response to PeerListRequest.
/// Peers are sorted by quality score (best first).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerListResponse {
    /// List of peers, sorted by quality score (best first)
    pub peers: Vec<PeerInfo>,

    /// Total number of peers known by the responder (for diagnostics)
    pub total_known_peers: usize,
}

impl PeerListResponse {
    /// Create a response with the given peers
    pub fn new(peers: Vec<PeerInfo>, total_known_peers: usize) -> Self {
        Self {
            peers,
            total_known_peers,
        }
    }

    /// Create an empty response
    pub fn empty() -> Self {
        Self {
            peers: Vec::new(),
            total_known_peers: 0,
        }
    }
}

// ============================================================================
// Canvas Gossip Message Types
// ============================================================================

/// Canvas operation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CanvasOperationType {
    /// Add a new element to the canvas
    Add,
    /// Update an existing element
    Update,
    /// Remove an element from the canvas
    Remove,
}

/// Canvas element operation for real-time synchronization
///
/// Represents a single operation on a canvas element (add, update, or remove).
/// Operations include vector clock metadata for CRDT conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasOperation {
    /// Unique identifier for this operation
    pub operation_id: String,

    /// Canvas entity ID this operation belongs to
    pub canvas_id: String,

    /// Element ID being operated on
    pub element_id: String,

    /// Type of operation (add, update, remove)
    pub operation_type: CanvasOperationType,

    /// Element data (JSON-serialized for flexibility)
    /// None for remove operations
    pub element_data: Option<serde_json::Value>,

    /// Vector clock at time of operation
    pub vector_clock: VectorClock,

    /// Lamport timestamp for total ordering
    pub lamport_clock: u64,

    /// Peer ID that originated this operation
    pub origin_peer: String,

    /// Unix timestamp (milliseconds)
    pub timestamp_ms: u64,
}

impl CanvasOperation {
    /// Create a new add operation
    pub fn add(
        canvas_id: String,
        element_id: String,
        element_data: serde_json::Value,
        vector_clock: VectorClock,
        lamport_clock: u64,
        origin_peer: String,
    ) -> Self {
        Self {
            operation_id: format!("{}-{}-add", canvas_id, element_id),
            canvas_id,
            element_id,
            operation_type: CanvasOperationType::Add,
            element_data: Some(element_data),
            vector_clock,
            lamport_clock,
            origin_peer,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create a new update operation
    pub fn update(
        canvas_id: String,
        element_id: String,
        element_data: serde_json::Value,
        vector_clock: VectorClock,
        lamport_clock: u64,
        origin_peer: String,
    ) -> Self {
        Self {
            operation_id: format!("{}-{}-update-{}", canvas_id, element_id, lamport_clock),
            canvas_id,
            element_id,
            operation_type: CanvasOperationType::Update,
            element_data: Some(element_data),
            vector_clock,
            lamport_clock,
            origin_peer,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Create a new remove operation
    pub fn remove(
        canvas_id: String,
        element_id: String,
        vector_clock: VectorClock,
        lamport_clock: u64,
        origin_peer: String,
    ) -> Self {
        Self {
            operation_id: format!("{}-{}-remove", canvas_id, element_id),
            canvas_id,
            element_id,
            operation_type: CanvasOperationType::Remove,
            element_data: None,
            vector_clock,
            lamport_clock,
            origin_peer,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }
}

/// Canvas cursor position update for collaborative awareness
///
/// Sent periodically to show where other users are working on the canvas.
/// These are ephemeral and not persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasCursorUpdate {
    /// Canvas entity ID
    pub canvas_id: String,

    /// Peer ID of the cursor owner
    pub peer_id: String,

    /// Display name of the user (for rendering)
    pub display_name: String,

    /// X coordinate on canvas
    pub x: f64,

    /// Y coordinate on canvas
    pub y: f64,

    /// Optional element ID if cursor is over an element
    pub hovered_element_id: Option<String>,

    /// Optional selection state (element IDs currently selected)
    pub selected_elements: Vec<String>,

    /// Currently active tool (e.g., "pen", "select", "eraser")
    pub tool: Option<String>,

    /// User's assigned color for cursor rendering
    pub color: Option<String>,

    /// Unix timestamp (milliseconds) - for expiring stale cursors
    pub timestamp_ms: u64,
}

impl CanvasCursorUpdate {
    /// Create a new cursor update
    pub fn new(canvas_id: String, peer_id: String, display_name: String, x: f64, y: f64) -> Self {
        Self {
            canvas_id,
            peer_id,
            display_name,
            x,
            y,
            hovered_element_id: None,
            selected_elements: Vec::new(),
            tool: None,
            color: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Set hovered element
    pub fn with_hovered_element(mut self, element_id: String) -> Self {
        self.hovered_element_id = Some(element_id);
        self
    }

    /// Set selected elements
    pub fn with_selection(mut self, elements: Vec<String>) -> Self {
        self.selected_elements = elements;
        self
    }

    /// Set the current tool
    pub fn with_tool(mut self, tool: String) -> Self {
        self.tool = Some(tool);
        self
    }

    /// Set the user's cursor color
    pub fn with_color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }
}

/// Request for full canvas state
///
/// Sent when a peer joins a canvas session and needs the current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasStateRequest {
    /// Canvas entity ID to request
    pub canvas_id: String,

    /// Requesting peer ID
    pub requester_peer_id: String,

    /// Optional: request only elements modified after this vector clock
    /// If None, request full state
    pub since_vector_clock: Option<VectorClock>,
}

impl CanvasStateRequest {
    /// Create a request for full canvas state
    pub fn full(canvas_id: String, requester_peer_id: String) -> Self {
        Self {
            canvas_id,
            requester_peer_id,
            since_vector_clock: None,
        }
    }

    /// Create a request for incremental state since a vector clock
    pub fn incremental(canvas_id: String, requester_peer_id: String, since: VectorClock) -> Self {
        Self {
            canvas_id,
            requester_peer_id,
            since_vector_clock: Some(since),
        }
    }
}

/// Response with canvas state snapshot
///
/// Contains all elements and the current vector clock for the canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasStateResponse {
    /// Canvas entity ID
    pub canvas_id: String,

    /// Responding peer ID
    pub responder_peer_id: String,

    /// All canvas elements (JSON-serialized)
    pub elements: Vec<serde_json::Value>,

    /// Current vector clock for the canvas
    pub vector_clock: VectorClock,

    /// Whether this is a partial (incremental) or full response
    pub is_incremental: bool,

    /// Total element count (for progress indication)
    pub total_element_count: usize,
}

impl CanvasStateResponse {
    /// Create a full state response
    pub fn full(
        canvas_id: String,
        responder_peer_id: String,
        elements: Vec<serde_json::Value>,
        vector_clock: VectorClock,
    ) -> Self {
        let total = elements.len();
        Self {
            canvas_id,
            responder_peer_id,
            elements,
            vector_clock,
            is_incremental: false,
            total_element_count: total,
        }
    }

    /// Create an incremental state response
    pub fn incremental(
        canvas_id: String,
        responder_peer_id: String,
        elements: Vec<serde_json::Value>,
        vector_clock: VectorClock,
        total_element_count: usize,
    ) -> Self {
        Self {
            canvas_id,
            responder_peer_id,
            elements,
            vector_clock,
            is_incremental: true,
            total_element_count,
        }
    }
}

/// Gossip message type wrapper
///
/// Wraps different message types sent over the gossip network:
/// - Chat messages (regular CRDTMessage)
/// - Sync requests (when a peer needs historical messages)
/// - Sync responses (reply with historical messages)
/// - Peer list requests (when a node needs to discover peers)
/// - Peer list responses (reply with known healthy peers)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum GossipMessageType {
    /// Regular chat/entity message
    Chat(CRDTMessage),

    /// Request for historical messages (sent when joining a topic)
    SyncRequest(SyncRequest),

    /// Response with historical messages
    SyncResponse(SyncResponse),

    /// Member add/remove updates for entity membership
    MemberUpdate(MemberUpdate),

    /// Request a membership snapshot for an entity
    MemberSyncRequest(MemberSyncRequest),

    /// Response with membership snapshot updates
    MemberSyncResponse(MemberSyncResponse),

    /// Request for a list of known peers (bootstrap node enhancement)
    PeerListRequest(PeerListRequest),

    /// Response with list of healthy peers sorted by quality score
    PeerListResponse(PeerListResponse),

    /// Canvas element operation (add, update, remove)
    CanvasOperation(CanvasOperation),

    /// Canvas cursor position update for collaborative awareness
    CanvasCursorUpdate(CanvasCursorUpdate),

    /// Request for canvas state (when joining a canvas session)
    CanvasStateRequest(CanvasStateRequest),

    /// Response with canvas state snapshot
    CanvasStateResponse(CanvasStateResponse),
}

/// Sort messages in causal order
pub fn sort_messages_causally(messages: &mut [CRDTMessage]) {
    messages.sort_by(|a, b| {
        // First try vector clock comparison
        match a.metadata.vector_clock.compare(&b.metadata.vector_clock) {
            ClockOrdering::Before => Ordering::Less,
            ClockOrdering::After => Ordering::Greater,
            ClockOrdering::Equal | ClockOrdering::Concurrent => {
                // Fallback to Lamport clock
                match a.metadata.lamport_clock.cmp(&b.metadata.lamport_clock) {
                    Ordering::Equal => {
                        // Final tiebreaker: message ID
                        a.metadata.id.cmp(&b.metadata.id)
                    }
                    other => other,
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_increment() {
        let mut clock = VectorClock::new();
        clock.increment("alice");
        clock.increment("alice");
        clock.increment("bob");

        assert_eq!(clock.0.get("alice"), Some(&2));
        assert_eq!(clock.0.get("bob"), Some(&1));
    }

    #[test]
    fn test_vector_clock_comparison() {
        let mut clock1 = VectorClock::new();
        clock1.increment("alice");
        clock1.increment("alice");

        let mut clock2 = VectorClock::new();
        clock2.increment("alice");

        assert_eq!(clock1.compare(&clock2), ClockOrdering::After);
        assert_eq!(clock2.compare(&clock1), ClockOrdering::Before);

        let mut clock3 = VectorClock::new();
        clock3.increment("bob");

        assert_eq!(clock1.compare(&clock3), ClockOrdering::Concurrent);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut clock1 = VectorClock::new();
        clock1.increment("alice");
        clock1.increment("alice");

        let mut clock2 = VectorClock::new();
        clock2.increment("alice");
        clock2.increment("bob");

        clock1.merge(&clock2);

        assert_eq!(clock1.0.get("alice"), Some(&2));
        assert_eq!(clock1.0.get("bob"), Some(&1));
    }

    #[test]
    fn test_has_dependencies() {
        let mut local = VectorClock::new();
        local.increment("alice");
        local.increment("alice");

        let mut message_clock = VectorClock::new();
        message_clock.increment("alice");
        message_clock.increment("alice");
        message_clock.increment("alice");

        assert!(local.has_dependencies(&message_clock));

        message_clock.increment("alice");
        message_clock.increment("alice");

        assert!(!local.has_dependencies(&message_clock));
    }

    // ========================================================================
    // Canvas Gossip Message Tests
    // ========================================================================

    #[test]
    fn test_canvas_operation_add_serialization() {
        let mut clock = VectorClock::new();
        clock.increment("peer-1");

        let op = CanvasOperation::add(
            "canvas-123".to_string(),
            "element-456".to_string(),
            serde_json::json!({"type": "rectangle", "x": 100, "y": 200}),
            clock,
            1,
            "peer-1".to_string(),
        );

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: CanvasOperation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.canvas_id, "canvas-123");
        assert_eq!(deserialized.element_id, "element-456");
        assert_eq!(deserialized.operation_type, CanvasOperationType::Add);
        assert!(deserialized.element_data.is_some());
    }

    #[test]
    fn test_canvas_operation_remove_serialization() {
        let clock = VectorClock::new();

        let op = CanvasOperation::remove(
            "canvas-123".to_string(),
            "element-456".to_string(),
            clock,
            5,
            "peer-2".to_string(),
        );

        let json = serde_json::to_string(&op).unwrap();
        let deserialized: CanvasOperation = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.operation_type, CanvasOperationType::Remove);
        assert!(deserialized.element_data.is_none());
    }

    #[test]
    fn test_canvas_cursor_update_serialization() {
        let cursor = CanvasCursorUpdate::new(
            "canvas-123".to_string(),
            "peer-1".to_string(),
            "Alice".to_string(),
            150.5,
            200.75,
        )
        .with_hovered_element("elem-1".to_string())
        .with_selection(vec!["elem-2".to_string(), "elem-3".to_string()]);

        let json = serde_json::to_string(&cursor).unwrap();
        let deserialized: CanvasCursorUpdate = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.canvas_id, "canvas-123");
        assert_eq!(deserialized.peer_id, "peer-1");
        assert_eq!(deserialized.display_name, "Alice");
        assert!((deserialized.x - 150.5).abs() < f64::EPSILON);
        assert!((deserialized.y - 200.75).abs() < f64::EPSILON);
        assert_eq!(deserialized.hovered_element_id, Some("elem-1".to_string()));
        assert_eq!(deserialized.selected_elements.len(), 2);
    }

    #[test]
    fn test_canvas_state_request_serialization() {
        let request = CanvasStateRequest::full("canvas-123".to_string(), "peer-1".to_string());

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CanvasStateRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.canvas_id, "canvas-123");
        assert_eq!(deserialized.requester_peer_id, "peer-1");
        assert!(deserialized.since_vector_clock.is_none());

        // Test incremental request
        let mut clock = VectorClock::new();
        clock.increment("peer-1");
        let incremental =
            CanvasStateRequest::incremental("canvas-456".to_string(), "peer-2".to_string(), clock);

        let json = serde_json::to_string(&incremental).unwrap();
        let deserialized: CanvasStateRequest = serde_json::from_str(&json).unwrap();

        assert!(deserialized.since_vector_clock.is_some());
    }

    #[test]
    fn test_canvas_state_response_serialization() {
        let mut clock = VectorClock::new();
        clock.increment("peer-1");
        clock.increment("peer-2");

        let elements = vec![
            serde_json::json!({"id": "elem-1", "type": "rectangle"}),
            serde_json::json!({"id": "elem-2", "type": "circle"}),
        ];

        let response = CanvasStateResponse::full(
            "canvas-123".to_string(),
            "peer-1".to_string(),
            elements,
            clock,
        );

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: CanvasStateResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.canvas_id, "canvas-123");
        assert_eq!(deserialized.responder_peer_id, "peer-1");
        assert_eq!(deserialized.elements.len(), 2);
        assert_eq!(deserialized.total_element_count, 2);
        assert!(!deserialized.is_incremental);
    }

    #[test]
    fn test_gossip_message_canvas_operation_roundtrip() {
        let mut clock = VectorClock::new();
        clock.increment("peer-1");

        let op = CanvasOperation::update(
            "canvas-123".to_string(),
            "element-456".to_string(),
            serde_json::json!({"x": 300, "y": 400}),
            clock,
            10,
            "peer-1".to_string(),
        );

        let msg = GossipMessageType::CanvasOperation(op);
        let json = serde_json::to_string(&msg).unwrap();

        // Verify the tag is correct
        assert!(json.contains("\"type\":\"canvas_operation\""));

        let deserialized: GossipMessageType = serde_json::from_str(&json).unwrap();
        match deserialized {
            GossipMessageType::CanvasOperation(op) => {
                assert_eq!(op.canvas_id, "canvas-123");
                assert_eq!(op.operation_type, CanvasOperationType::Update);
            }
            _ => panic!("Expected CanvasOperation variant"),
        }
    }

    #[test]
    fn test_gossip_message_canvas_cursor_roundtrip() {
        let cursor = CanvasCursorUpdate::new(
            "canvas-123".to_string(),
            "peer-1".to_string(),
            "Bob".to_string(),
            50.0,
            75.0,
        );

        let msg = GossipMessageType::CanvasCursorUpdate(cursor);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"canvas_cursor_update\""));

        let deserialized: GossipMessageType = serde_json::from_str(&json).unwrap();
        match deserialized {
            GossipMessageType::CanvasCursorUpdate(c) => {
                assert_eq!(c.display_name, "Bob");
            }
            _ => panic!("Expected CanvasCursorUpdate variant"),
        }
    }

    #[test]
    fn test_gossip_message_canvas_state_request_roundtrip() {
        let request = CanvasStateRequest::full("canvas-789".to_string(), "peer-3".to_string());

        let msg = GossipMessageType::CanvasStateRequest(request);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"canvas_state_request\""));

        let deserialized: GossipMessageType = serde_json::from_str(&json).unwrap();
        match deserialized {
            GossipMessageType::CanvasStateRequest(r) => {
                assert_eq!(r.canvas_id, "canvas-789");
            }
            _ => panic!("Expected CanvasStateRequest variant"),
        }
    }

    #[test]
    fn test_gossip_message_canvas_state_response_roundtrip() {
        let clock = VectorClock::new();
        let response = CanvasStateResponse::incremental(
            "canvas-123".to_string(),
            "peer-1".to_string(),
            vec![serde_json::json!({"id": "new-elem"})],
            clock,
            100,
        );

        let msg = GossipMessageType::CanvasStateResponse(response);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains("\"type\":\"canvas_state_response\""));

        let deserialized: GossipMessageType = serde_json::from_str(&json).unwrap();
        match deserialized {
            GossipMessageType::CanvasStateResponse(r) => {
                assert!(r.is_incremental);
                assert_eq!(r.total_element_count, 100);
            }
            _ => panic!("Expected CanvasStateResponse variant"),
        }
    }
}

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

/// Entity type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Person,
    Group,
    Project,
    Channel,
    Organisation,
}

impl EntityType {
    /// Get the string representation for document IDs
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Person => "person",
            EntityType::Group => "group",
            EntityType::Project => "project",
            EntityType::Channel => "channel",
            EntityType::Organisation => "organisation",
        }
    }
}

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
}

/// Sync response - reply with messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub entity_id: String,
    pub entity_type: EntityType,
    pub messages: Vec<CRDTMessage>,
    pub vector_clock: VectorClock,
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

    /// Request for a list of known peers (bootstrap node enhancement)
    PeerListRequest(PeerListRequest),

    /// Response with list of healthy peers sorted by quality score
    PeerListResponse(PeerListResponse),
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
}

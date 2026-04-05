// SPDX-License-Identifier: MIT OR Apache-2.0

//! Request and response types for the x0xd REST API.
//!
//! All types mirror the JSON shapes returned by x0xd on `127.0.0.1:12700`.
//! Payloads are base64-encoded bytes in the wire format; these types expose
//! them as `Vec<u8>` with serde helpers for the encoding.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Generic envelope ────────────────────────────────────────────────────────

/// Raw API response envelope. Consumers should not use this directly;
/// the client methods unwrap it into domain types or [`crate::X0xError`].
#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse<T> {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(flatten)]
    pub data: Option<T>,
}

// ── System & Identity ───────────────────────────────────────────────────────

/// Response from `GET /health`.
#[derive(Debug, Clone, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub peers: u32,
    pub uptime_secs: u64,
}

/// Response from `GET /status`.
#[derive(Debug, Clone, Deserialize)]
pub struct DaemonStatus {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub api_address: String,
    #[serde(default)]
    pub external_addrs: Vec<String>,
    pub agent_id: String,
    pub peers: u32,
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Response from `GET /agent`.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub machine_id: String,
    pub user_id: Option<String>,
}

/// Response from `GET /agent/user-id`.
#[derive(Debug, Clone, Deserialize)]
pub struct UserIdStatus {
    #[serde(default)]
    pub user_id: Option<String>,
}

/// A connected gossip peer from `GET /peers`.
#[derive(Debug, Clone, Deserialize)]
pub struct PeerInfo {
    pub id: String,
}

/// Response from `GET /peers`.
#[derive(Debug, Clone, Deserialize)]
pub struct PeerList {
    pub peers: Vec<PeerInfo>,
}

// ── Discovery ───────────────────────────────────────────────────────────────

/// A single discovered agent from `GET /agents/discovered`.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveredAgent {
    pub agent_id: String,
    pub machine_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub addresses: Vec<String>,
    #[serde(default)]
    pub last_seen: Option<u64>,
}

/// Response wrapper for `GET /agents/discovered`.
#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveredAgentList {
    pub agents: Vec<DiscoveredAgent>,
}

/// Internal wrapper for `GET /agents/discovered/:agent_id`.
///
/// The daemon returns `{"ok": true, "agent": {...}}` with the agent nested
/// under the `"agent"` key rather than flattened at the root level.
#[derive(Debug, Deserialize)]
pub(crate) struct DiscoveredAgentWrapper {
    pub agent: DiscoveredAgent,
}

/// Response wrapper for `GET /presence`.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceList {
    pub agents: Vec<String>,
}

/// Response from `GET /network/status`.
#[derive(Debug, Clone, Deserialize)]
pub struct NetworkStatus {
    #[serde(default)]
    pub avg_rtt_ms: Option<u64>,
    #[serde(default)]
    pub can_receive_direct: bool,
    #[serde(default)]
    pub connected_peers: u32,
    #[serde(default)]
    pub coordination_sessions: u32,
    #[serde(default)]
    pub direct_connections: u32,
    #[serde(default)]
    pub external_addrs: Vec<String>,
    #[serde(default)]
    pub has_public_ip: bool,
    #[serde(default)]
    pub hole_punch_success_rate: Option<f64>,
    #[serde(default)]
    pub is_coordinating: bool,
    #[serde(default)]
    pub is_relaying: bool,
    #[serde(default)]
    pub local_addr: Option<String>,
    #[serde(default)]
    pub nat_type: Option<String>,
    #[serde(default)]
    pub relay_sessions: u32,
    #[serde(default)]
    pub relayed_connections: u32,
    #[serde(default)]
    pub uptime_secs: Option<u64>,
}

/// Response from `GET /network/bootstrap-cache`.
#[derive(Debug, Clone, Deserialize)]
pub struct BootstrapCacheStatus {
    #[serde(default)]
    pub connected_peers: Vec<String>,
    #[serde(default)]
    pub connection_count: u32,
}

/// Request body for `POST /announce`.
#[derive(Debug, Default, Serialize)]
pub struct AnnounceRequest {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub include_user_identity: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub human_consent: bool,
}

// ── Gossip (pub/sub) ────────────────────────────────────────────────────────

/// Request body for `POST /publish`.
#[derive(Debug, Serialize)]
pub struct PublishRequest {
    pub topic: String,
    pub payload: String, // base64
}

/// Request body for `POST /subscribe`.
#[derive(Debug, Serialize)]
pub struct SubscribeRequest {
    pub topic: String,
}

/// A group reference embedded in an agent card.
#[derive(Debug, Clone, Deserialize)]
pub struct CardGroup {
    pub name: String,
    pub invite_link: String,
}

/// A store reference embedded in an agent card.
#[derive(Debug, Clone, Deserialize)]
pub struct CardStore {
    pub name: String,
    pub topic: String,
}

/// A shareable identity card from `GET /agent/card`.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentCard {
    pub display_name: String,
    pub agent_id: String,
    pub machine_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub addresses: Vec<String>,
    #[serde(default)]
    pub groups: Vec<CardGroup>,
    #[serde(default)]
    pub stores: Vec<CardStore>,
    pub created_at: u64,
}

/// Response from `GET /agent/card`.
#[derive(Debug, Clone, Deserialize)]
pub struct AgentCardResponse {
    pub card: AgentCard,
    pub link: String,
}

/// Request body for `POST /agent/card/import`.
#[derive(Debug, Serialize)]
pub struct ImportCardRequest {
    pub card: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<TrustLevel>,
}

/// Response from `POST /agent/card/import`.
#[derive(Debug, Clone, Deserialize)]
pub struct ImportCardResponse {
    pub agent_id: String,
    pub display_name: String,
    pub trust_level: String,
    pub groups: usize,
    pub stores: usize,
}

/// Response from `POST /subscribe`.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeResponse {
    pub subscription_id: String,
}

/// A daemon gossip message payload shape.
///
/// This type matches the payload shape used by daemon streaming surfaces, even
/// though this crate is intentionally frozen to REST + WebSocket only.
#[derive(Debug, Clone, Deserialize)]
pub struct GossipMessage {
    pub topic: String,
    pub payload: String, // base64
    #[serde(default)]
    pub origin: Option<String>,
}

// ── Direct messaging ────────────────────────────────────────────────────────

/// Request body for `POST /agents/connect`.
#[derive(Debug, Serialize)]
pub struct ConnectRequest {
    pub agent_id: String,
}

/// Request body for `POST /direct/send`.
#[derive(Debug, Serialize)]
pub struct DirectSendRequest {
    pub agent_id: String,
    pub payload: String, // base64
}

/// A direct connection from `GET /direct/connections`.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectConnection {
    pub agent_id: String,
    #[serde(default)]
    pub machine_id: Option<String>,
    #[serde(default)]
    pub connected_at: Option<u64>,
}

/// Response from `GET /direct/connections`.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectConnectionList {
    pub connections: Vec<DirectConnection>,
}

/// A daemon direct-message payload shape.
///
/// This type matches the payload shape used by daemon streaming surfaces, even
/// though this crate is intentionally frozen to REST + WebSocket only.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectMessage {
    pub sender: String,
    pub machine_id: String,
    pub payload: String, // base64
    #[serde(default)]
    pub received_at: Option<u64>,
}

// ── Contacts & trust ────────────────────────────────────────────────────────

/// Trust level for a contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    Blocked,
    Unknown,
    Known,
    Trusted,
}

/// Request body for `POST /contacts`.
#[derive(Debug, Serialize)]
pub struct AddContactRequest {
    pub agent_id: String,
    pub trust_level: TrustLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Request body for `POST /contacts/trust`.
#[derive(Debug, Serialize)]
pub struct SetTrustRequest {
    pub agent_id: String,
    pub level: TrustLevel,
}

/// Request body for `PATCH /contacts/:agent_id`.
///
/// The daemon accepts `trust_level` and `identity_type`; it does not support
/// a `label` field on this endpoint.
#[derive(Debug, Serialize)]
pub struct UpdateContactRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<TrustLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_type: Option<String>,
}

/// A contact from `GET /contacts`.
#[derive(Debug, Clone, Deserialize)]
pub struct Contact {
    pub agent_id: String,
    pub trust_level: TrustLevel,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub added_at: Option<u64>,
    #[serde(default)]
    pub last_seen: Option<u64>,
}

/// Response wrapper for `GET /contacts`.
#[derive(Debug, Clone, Deserialize)]
pub struct ContactList {
    pub contacts: Vec<Contact>,
}

/// A machine record for a contact.
#[derive(Debug, Clone, Deserialize)]
pub struct MachineRecord {
    pub machine_id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub first_seen: Option<u64>,
    #[serde(default)]
    pub last_seen: Option<u64>,
    #[serde(default)]
    pub pinned: bool,
}

/// Response wrapper for `GET /contacts/:id/machines`.
#[derive(Debug, Clone, Deserialize)]
pub struct MachineList {
    pub machines: Vec<MachineRecord>,
}

/// Response wrapper for `GET /contacts/:agent_id/revocations`.
#[derive(Debug, Clone, Deserialize)]
pub struct RevocationList {
    #[serde(default)]
    pub revocations: Vec<serde_json::Value>,
}

/// Request body for `POST /contacts/:id/machines`.
#[derive(Debug, Serialize)]
pub struct AddMachineRequest {
    pub machine_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

/// Request body for `POST /trust/evaluate`.
#[derive(Debug, Serialize)]
pub struct EvaluateTrustRequest {
    pub agent_id: String,
    pub machine_id: String,
}

/// Response from `POST /trust/evaluate`.
#[derive(Debug, Clone, Deserialize)]
pub struct TrustEvaluation {
    pub decision: String,
}

// ── MLS groups ──────────────────────────────────────────────────────────────

/// Request body for `POST /mls/groups`.
#[derive(Debug, Serialize)]
pub struct CreateMlsGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
}

/// Response from MLS group creation / listing.
#[derive(Debug, Clone, Deserialize)]
pub struct MlsGroup {
    pub group_id: String,
    pub epoch: u64,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub member_count: Option<u32>,
}

/// Response wrapper for `GET /mls/groups`.
#[derive(Debug, Clone, Deserialize)]
pub struct MlsGroupList {
    pub groups: Vec<MlsGroup>,
}

/// Request body for `POST /mls/groups/:id/members`.
#[derive(Debug, Serialize)]
pub struct AddMlsMemberRequest {
    pub agent_id: String,
}

/// Response from adding a member to an MLS group.
#[derive(Debug, Clone, Deserialize)]
pub struct AddMlsMemberResponse {
    pub epoch: u64,
    pub member_count: u32,
}

/// Request body for `POST /mls/groups/:id/welcome`.
#[derive(Debug, Serialize)]
pub struct CreateWelcomeRequest {
    pub agent_id: String,
}

/// Response from `POST /mls/groups/:id/welcome`.
#[derive(Debug, Clone, Deserialize)]
pub struct WelcomeResponse {
    pub welcome: String,
    pub group_id: String,
    pub epoch: u64,
}

/// Request body for `POST /mls/groups/:id/encrypt`.
#[derive(Debug, Serialize)]
pub struct EncryptRequest {
    pub payload: String, // base64
}

/// Response from encrypting with an MLS group.
#[derive(Debug, Clone, Deserialize)]
pub struct EncryptResponse {
    pub ciphertext: String, // base64
    pub epoch: u64,
}

/// Request body for `POST /mls/groups/:id/decrypt`.
#[derive(Debug, Serialize)]
pub struct DecryptRequest {
    pub ciphertext: String, // base64
    pub epoch: u64,
}

/// Response from decrypting with an MLS group.
#[derive(Debug, Clone, Deserialize)]
pub struct DecryptResponse {
    pub payload: String, // base64
}

// ── Named groups (high-level) ───────────────────────────────────────────────

/// Request body for `POST /groups`.
#[derive(Debug, Serialize)]
pub struct CreateGroupRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Response from creating a named group.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatedGroup {
    pub group_id: String,
    pub name: String,
    #[serde(default)]
    pub chat_topic: Option<String>,
}

/// A named group summary from `GET /groups`.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupSummary {
    pub group_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub created_at: Option<u64>,
    #[serde(default)]
    pub member_count: Option<u32>,
}

/// Response wrapper for `GET /groups`.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupList {
    pub groups: Vec<GroupSummary>,
}

/// Full group info from `GET /groups/:id`.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub creator: Option<String>,
    #[serde(default)]
    pub created_at: Option<u64>,
    #[serde(default)]
    pub member_count: Option<u32>,
    #[serde(default)]
    pub chat_topic: Option<String>,
    #[serde(default)]
    pub metadata_topic: Option<String>,
    #[serde(default)]
    pub members: Vec<GroupMember>,
}

/// A member within a named group.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupMember {
    pub agent_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Request body for `POST /groups/:id/invite`.
#[derive(Debug, Serialize)]
pub struct InviteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_secs: Option<u64>,
}

/// Response from creating an invite.
#[derive(Debug, Clone, Deserialize)]
pub struct InviteResponse {
    pub invite_link: String,
    pub group_id: String,
    pub group_name: String,
    #[serde(default)]
    pub expires_at: Option<u64>,
}

/// Request body for `POST /groups/join`.
#[derive(Debug, Serialize)]
pub struct JoinGroupRequest {
    pub invite: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Response from joining a group.
#[derive(Debug, Clone, Deserialize)]
pub struct JoinGroupResponse {
    pub group_id: String,
    pub group_name: String,
    #[serde(default)]
    pub chat_topic: Option<String>,
}

/// Request body for `PUT /groups/:id/display-name`.
#[derive(Debug, Serialize)]
pub struct SetDisplayNameRequest {
    pub name: String,
}

// ── Task lists (CRDTs) ─────────────────────────────────────────────────────

/// Request body for `POST /task-lists`.
#[derive(Debug, Serialize)]
pub struct CreateTaskListRequest {
    pub name: String,
    pub topic: String,
}

/// Response from creating a task list.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatedTaskList {
    pub id: String,
}

/// A task list summary from `GET /task-lists`.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskListSummary {
    pub id: String,
    #[serde(default)]
    pub topic: Option<String>,
}

/// Response wrapper for `GET /task-lists`.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskListIndex {
    pub task_lists: Vec<TaskListSummary>,
}

/// Request body for `POST /task-lists/:id/tasks`.
#[derive(Debug, Serialize)]
pub struct AddTaskRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request body for `PATCH /task-lists/:id/tasks/:tid`.
#[derive(Debug, Serialize)]
pub struct UpdateTaskRequest {
    pub action: String, // "claim" or "complete"
}

/// A task from `GET /task-lists/:id/tasks`.
#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub priority: Option<u32>,
}

/// Response wrapper for `GET /task-lists/:id/tasks`.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskIndex {
    pub tasks: Vec<Task>,
}

// ── Key-value stores ────────────────────────────────────────────────────────

/// Request body for `POST /stores`.
#[derive(Debug, Serialize)]
pub struct CreateStoreRequest {
    pub name: String,
    pub topic: String,
}

/// Response from creating a store.
#[derive(Debug, Clone, Deserialize)]
pub struct CreatedStore {
    pub id: String,
}

/// A store summary from `GET /stores`.
#[derive(Debug, Clone, Deserialize)]
pub struct StoreSummary {
    pub id: String,
    #[serde(default)]
    pub topic: Option<String>,
}

/// Response wrapper for `GET /stores`.
#[derive(Debug, Clone, Deserialize)]
pub struct StoreIndex {
    pub stores: Vec<StoreSummary>,
}

/// Request body for `PUT /stores/:id/:key`.
#[derive(Debug, Serialize)]
pub struct PutValueRequest {
    pub value: String, // base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Response from `GET /stores/:id/:key`.
#[derive(Debug, Clone, Deserialize)]
pub struct StoreValue {
    pub key: String,
    pub value: String, // base64
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub created_at: Option<u64>,
    #[serde(default)]
    pub updated_at: Option<u64>,
}

/// A key entry from `GET /stores/:id/keys`.
#[derive(Debug, Clone, Deserialize)]
pub struct StoreKeyEntry {
    pub key: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub updated_at: Option<u64>,
}

/// Response wrapper for `GET /stores/:id/keys`.
#[derive(Debug, Clone, Deserialize)]
pub struct StoreKeyIndex {
    pub keys: Vec<StoreKeyEntry>,
}

// ── File transfer ───────────────────────────────────────────────────────────

/// Request body for `POST /files/send`.
///
/// The daemon requires both `agent_id` and `sha256` to be non-empty.
/// `path` is an optional local source path that x0xd uses when reading file
/// content to send; omit it if x0xd will receive the data over the QUIC stream.
#[derive(Debug, Serialize)]
pub struct SendFileRequest {
    pub agent_id: String,
    pub filename: String,
    pub size: u64,
    /// SHA-256 hex digest of the file content. Required by the daemon.
    pub sha256: String,
    /// Optional local filesystem path for x0xd to read the file from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Response from initiating a file send.
#[derive(Debug, Clone, Deserialize)]
pub struct SendFileResponse {
    pub transfer_id: String,
}

/// Transfer direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TransferDirection {
    Sending,
    Receiving,
}

/// Transfer status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TransferStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
    Rejected,
}

/// A file transfer record from `GET /files/transfers`.
#[derive(Debug, Clone, Deserialize)]
pub struct FileTransfer {
    pub transfer_id: String,
    pub direction: TransferDirection,
    pub remote_agent_id: String,
    pub filename: String,
    pub total_size: u64,
    #[serde(default)]
    pub bytes_transferred: u64,
    pub status: TransferStatus,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub started_at: Option<u64>,
}

/// Response wrapper for `GET /files/transfers`.
#[derive(Debug, Clone, Deserialize)]
pub struct TransferList {
    pub transfers: Vec<FileTransfer>,
}

/// Internal wrapper for `GET /files/transfers/:id`.
///
/// The daemon returns `{"ok": true, "transfer": {...}}` with the transfer nested
/// under the `"transfer"` key rather than flattened at the root level.
#[derive(Debug, Deserialize)]
pub(crate) struct FileTransferWrapper {
    pub transfer: FileTransfer,
}

/// Request body for `POST /files/reject/:id`.
#[derive(Debug, Serialize)]
pub struct RejectFileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A WebSocket session from `GET /ws/sessions`.
#[derive(Debug, Clone, Deserialize)]
pub struct WsSessionInfo {
    pub session_id: String,
    #[serde(default)]
    pub receives_direct: bool,
    #[serde(default)]
    pub subscribed_topics: Vec<String>,
}

// ── Constitution ────────────────────────────────────────────────────────────

/// Structured response from `GET /constitution/json`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConstitutionInfo {
    /// Semantic version of the constitution document.
    pub version: String,
    /// Drafting status (e.g. "Draft", "Ratified").
    pub status: String,
    /// Full markdown text of the constitution.
    pub content: String,
}

// ── WebSocket sessions ──────────────────────────────────────────────────────

/// Response wrapper from `GET /ws/sessions`.
#[derive(Debug, Clone, Deserialize)]
pub struct WsSessionList {
    #[serde(default)]
    pub sessions: Vec<WsSessionInfo>,
    #[serde(default)]
    pub shared_subscriptions: HashMap<String, u32>,
}

// ── Presence (extended) ────────────────────────────────────────────────────

/// Response from `GET /presence/online`.
///
/// Same shape as discovered-agents list, but filtered to exclude blocked agents.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceOnlineList {
    pub agents: Vec<DiscoveredAgent>,
}

/// Response from `GET /presence/foaf`.
///
/// FOAF random-walk discovery returns trusted/known agents nearby.
#[derive(Debug, Clone, Deserialize)]
pub struct FoafDiscoveryList {
    pub agents: Vec<DiscoveredAgent>,
}

/// Response from `GET /presence/status/:id`.
///
/// Local cache lookup — no network I/O.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceStatusResponse {
    pub online: bool,
    #[serde(default)]
    pub agent: Option<DiscoveredAgent>,
}

/// Response from `GET /presence/find/:id`.
///
/// FOAF walk targeting a specific agent.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceFindResponse {
    #[serde(default)]
    pub agent: Option<DiscoveredAgent>,
}

// ── Agent discovery (extended) ─────────────────────────────────────────────

/// Response from `GET /agents/reachability/:agent_id`.
///
/// NAT traversal heuristics for a specific agent.
#[derive(Debug, Clone, Deserialize)]
pub struct ReachabilityInfo {
    pub likely_direct: bool,
    pub needs_coordination: bool,
    pub is_relay: bool,
    pub is_coordinator: bool,
    #[serde(default)]
    pub addresses: Vec<String>,
}

/// Response from `POST /agents/find/:agent_id`.
///
/// Active 3-stage search: cache → shard → rendezvous.
#[derive(Debug, Clone, Deserialize)]
pub struct FindAgentResponse {
    pub found: bool,
    #[serde(default)]
    pub addresses: Vec<String>,
}

/// Response from `GET /users/:user_id/agents`.
#[derive(Debug, Clone, Deserialize)]
pub struct UserAgentsList {
    pub user_id: String,
    pub agents: Vec<DiscoveredAgent>,
}

// ── WebSocket frames ────────────────────────────────────────────────────────

/// Messages sent by the client to x0xd over WebSocket.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsOutbound {
    Ping,
    Subscribe { topics: Vec<String> },
    Unsubscribe { topics: Vec<String> },
    Publish { topic: String, payload: String },
    SendDirect { agent_id: String, payload: String },
}

/// Messages received from x0xd over WebSocket.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsInbound {
    Connected {
        session_id: String,
        agent_id: String,
    },
    Message {
        topic: String,
        payload: String,
        #[serde(default)]
        origin: Option<String>,
    },
    DirectMessage {
        sender: String,
        machine_id: String,
        payload: String,
        #[serde(default)]
        received_at: Option<u64>,
    },
    Subscribed {
        topics: Vec<String>,
    },
    Unsubscribed {
        topics: Vec<String>,
    },
    Pong,
    Error {
        message: String,
    },
}

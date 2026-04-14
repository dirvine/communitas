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
    pub announced_at: Option<u64>,
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

/// A trust-scoped service entry from `GET /introduction`.
#[derive(Debug, Clone, Deserialize)]
pub struct IntroductionService {
    pub name: String,
    pub description: String,
    pub min_trust: String,
}

/// Trust-gated introduction card from `GET /introduction`.
#[derive(Debug, Clone, Deserialize)]
pub struct IntroductionCard {
    pub agent_id: String,
    #[serde(default)]
    pub machine_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub certificate: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    pub identity_words: String,
    #[serde(default)]
    pub services: Vec<IntroductionService>,
    #[serde(default)]
    pub signature: Option<String>,
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

/// Envelope carried by `/events` SSE frames.
#[derive(Debug, Clone, Deserialize)]
pub struct SseEnvelope {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Payload for `/events` frames where `type == "message"`.
#[derive(Debug, Clone, Deserialize)]
pub struct SseMessageData {
    pub subscription_id: String,
    pub topic: String,
    pub payload: String,
    #[serde(default)]
    pub sender: Option<String>,
    pub verified: bool,
    #[serde(default)]
    pub trust_level: Option<String>,
}

/// Payload for `/events` frames where `type == "file:offer"`.
#[derive(Debug, Clone, Deserialize)]
pub struct SseFileOfferData {
    pub transfer_id: String,
    pub filename: String,
    pub size: u64,
    pub sender: String,
}

/// Payload for `/events` frames where `type == "file:complete"`.
#[derive(Debug, Clone, Deserialize)]
pub struct SseFileCompleteData {
    pub transfer_id: String,
    pub filename: String,
    pub sha256: String,
    pub path: String,
}

/// Payload for `/presence/events` frames.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceStreamEvent {
    pub event: String,
    pub agent_id: String,
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
    #[serde(default)]
    pub identity_type: Option<String>,
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
    /// Full five-axis policy returned by `GET /groups/:id`. May be
    /// absent on legacy daemon responses; callers should treat
    /// `None` as "assume MlsEncrypted defaults".
    #[serde(default)]
    pub policy: Option<GroupPolicy>,
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

/// Structured response from `GET /upgrade`.
#[derive(Debug, Clone, Deserialize)]
pub struct UpgradeStatus {
    #[serde(default)]
    pub update_available: Option<bool>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub current_version: Option<String>,
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

// ─── Named-groups extended surface ─────────────────────────────────────────
// Mirrors x0x::groups::{policy, member, request, state_commit, directory,
// public_message, discovery}. Keep field names and snake_case serde forms
// aligned with those source types — drift breaks the wire format.

/// Controls whether a group is visible to non-members.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupDiscoverability {
    #[default]
    Hidden,
    ListedToContacts,
    PublicDirectory,
}

/// Controls how new members are admitted.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupAdmission {
    #[default]
    InviteOnly,
    RequestAccess,
    OpenJoin,
}

/// Controls how group content is cryptographically protected.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupConfidentiality {
    #[default]
    MlsEncrypted,
    SignedPublic,
}

/// Controls who can read group content.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupReadAccess {
    #[default]
    MembersOnly,
    Public,
}

/// Controls who can write content to the group.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupWriteAccess {
    #[default]
    MembersOnly,
    ModeratedPublic,
    AdminOnly,
}

/// Full five-axis group policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupPolicy {
    pub discoverability: GroupDiscoverability,
    pub admission: GroupAdmission,
    pub confidentiality: GroupConfidentiality,
    pub read_access: GroupReadAccess,
    pub write_access: GroupWriteAccess,
}

/// Named preset bundle for common policy shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupPolicyPreset {
    PrivateSecure,
    PublicRequestSecure,
    PublicOpen,
    PublicAnnounce,
}

impl GroupPolicyPreset {
    /// Concrete policy axes for a preset.
    #[must_use]
    pub fn to_policy(self) -> GroupPolicy {
        match self {
            Self::PrivateSecure => GroupPolicy {
                discoverability: GroupDiscoverability::Hidden,
                admission: GroupAdmission::InviteOnly,
                confidentiality: GroupConfidentiality::MlsEncrypted,
                read_access: GroupReadAccess::MembersOnly,
                write_access: GroupWriteAccess::MembersOnly,
            },
            Self::PublicRequestSecure => GroupPolicy {
                discoverability: GroupDiscoverability::PublicDirectory,
                admission: GroupAdmission::RequestAccess,
                confidentiality: GroupConfidentiality::MlsEncrypted,
                read_access: GroupReadAccess::MembersOnly,
                write_access: GroupWriteAccess::MembersOnly,
            },
            Self::PublicOpen => GroupPolicy {
                discoverability: GroupDiscoverability::PublicDirectory,
                admission: GroupAdmission::OpenJoin,
                confidentiality: GroupConfidentiality::SignedPublic,
                read_access: GroupReadAccess::Public,
                write_access: GroupWriteAccess::MembersOnly,
            },
            Self::PublicAnnounce => GroupPolicy {
                discoverability: GroupDiscoverability::PublicDirectory,
                admission: GroupAdmission::OpenJoin,
                confidentiality: GroupConfidentiality::SignedPublic,
                read_access: GroupReadAccess::Public,
                write_access: GroupWriteAccess::AdminOnly,
            },
        }
    }

    /// Parse the snake_case/kebab-case name accepted by the daemon.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().replace('-', "_").as_str() {
            "private_secure" => Some(Self::PrivateSecure),
            "public_request_secure" => Some(Self::PublicRequestSecure),
            "public_open" => Some(Self::PublicOpen),
            "public_announce" => Some(Self::PublicAnnounce),
            _ => None,
        }
    }
}

/// Summarised policy carried in `GroupCard` and `PATCH /groups/:id/policy`
/// bodies. Structurally identical to `GroupPolicy` — separate for parity
/// with the x0xd wire type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupPolicySummary {
    pub discoverability: GroupDiscoverability,
    pub admission: GroupAdmission,
    pub confidentiality: GroupConfidentiality,
    #[serde(default)]
    pub read_access: GroupReadAccess,
    #[serde(default)]
    pub write_access: GroupWriteAccess,
}

/// Role of a member within a group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupRole {
    Owner,
    Admin,
    Moderator,
    Member,
    Guest,
}

/// Membership lifecycle state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupMemberState {
    #[default]
    Active,
    Pending,
    Removed,
    Banned,
}

/// Full roster entry. Mirrors `x0x::groups::GroupMember`. The existing
/// [`GroupMember`] defined for the minimal `GET /groups/:id` shape is
/// kept for backward compatibility; new code should prefer this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedGroupMember {
    pub agent_id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    pub role: GroupRole,
    pub state: GroupMemberState,
    #[serde(default)]
    pub display_name: Option<String>,
    pub joined_at: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub added_by: Option<String>,
    #[serde(default)]
    pub removed_by: Option<String>,
    #[serde(default)]
    pub kem_public_key_b64: Option<String>,
}

/// Response from `GET /groups/:id/members`.
#[derive(Debug, Clone, Deserialize)]
pub struct NamedGroupMembersResponse {
    #[serde(default)]
    pub members: Vec<NamedGroupMember>,
}

/// Status of a join request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JoinRequestStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

/// A join request submitted against a `RequestAccess` group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinRequest {
    pub request_id: String,
    pub group_id: String,
    pub requester_agent_id: String,
    #[serde(default)]
    pub requester_user_id: Option<String>,
    pub requested_role: GroupRole,
    #[serde(default)]
    pub message: Option<String>,
    pub created_at: u64,
    #[serde(default)]
    pub reviewed_at: Option<u64>,
    #[serde(default)]
    pub reviewed_by: Option<String>,
    pub status: JoinRequestStatus,
}

/// Response from `GET /groups/:id/requests`.
#[derive(Debug, Clone, Deserialize)]
pub struct JoinRequestListResponse {
    #[serde(default)]
    pub requests: Vec<JoinRequest>,
}

/// Kind of a signed public message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GroupPublicMessageKind {
    Chat,
    Announcement,
}

/// Signed message published to a SignedPublic group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupPublicMessage {
    pub group_id: String,
    pub state_hash_at_send: String,
    pub revision_at_send: u64,
    pub author_agent_id: String,
    pub author_public_key: String,
    #[serde(default)]
    pub author_user_id: Option<String>,
    #[serde(flatten)]
    pub kind: GroupPublicMessageKind,
    pub body: String,
    pub timestamp: u64,
    pub signature: String,
}

/// Request body for `POST /groups/:id/send`.
#[derive(Debug, Clone, Serialize)]
pub struct SendGroupMessageRequest {
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Response from `GET /groups/:id/messages`.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupMessagesResponse {
    #[serde(default)]
    pub messages: Vec<GroupPublicMessage>,
}

/// Immutable genesis record bound to a group's stable id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupGenesis {
    pub group_id: String,
    pub creator_agent_id: String,
    pub created_at: u64,
    pub creation_nonce: String,
}

/// Response from `GET /groups/:id/state` — the daemon's rendered view
/// of the current state-commit chain. All hash fields are hex strings.
/// `genesis` may be absent on legacy groups that predate Phase D.3.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GroupStateResponse {
    pub group_id: String,
    #[serde(default)]
    pub mls_group_id: Option<String>,
    #[serde(default)]
    pub genesis: Option<GroupGenesis>,
    pub state_revision: u64,
    pub state_hash: String,
    #[serde(default)]
    pub prev_state_hash: Option<String>,
    #[serde(default)]
    pub security_binding: Option<String>,
    #[serde(default)]
    pub withdrawn: bool,
    pub roster_root: String,
    pub policy_hash: String,
    pub public_meta_hash: String,
}

/// Signed commitment to the current valid group state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupStateCommit {
    pub group_id: String,
    pub revision: u64,
    #[serde(default)]
    pub prev_state_hash: Option<String>,
    pub roster_root: String,
    pub policy_hash: String,
    pub public_meta_hash: String,
    #[serde(default)]
    pub security_binding: Option<String>,
    pub state_hash: String,
    #[serde(default)]
    pub withdrawn: bool,
    pub committed_by: String,
    pub committed_at: u64,
    pub signer_public_key: String,
    pub signature: String,
}

/// Public-facing signed card for a discoverable group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupCard {
    pub group_id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub banner_url: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub policy_summary: GroupPolicySummary,
    pub owner_agent_id: String,
    pub admin_count: u32,
    pub member_count: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub request_access_enabled: bool,
    #[serde(default)]
    pub metadata_topic: Option<String>,
    #[serde(default)]
    pub revision: u64,
    #[serde(default)]
    pub state_hash: String,
    #[serde(default)]
    pub prev_state_hash: Option<String>,
    #[serde(default)]
    pub issued_at: u64,
    #[serde(default)]
    pub expires_at: u64,
    #[serde(default)]
    pub authority_agent_id: String,
    #[serde(default)]
    pub authority_public_key: String,
    #[serde(default)]
    pub withdrawn: bool,
    #[serde(default)]
    pub signature: String,
}

/// Response from `GET /groups/discover` and `GET /groups/discover/nearby`.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupCardList {
    #[serde(default)]
    pub groups: Vec<GroupCard>,
}

/// Which dimension a directory shard indexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShardKind {
    Tag,
    Name,
    Id,
}

/// A single active directory-shard subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRecord {
    pub kind: ShardKind,
    pub shard: u32,
    #[serde(default)]
    pub key: Option<String>,
    pub subscribed_at: u64,
}

/// Response from `GET /groups/discover/subscriptions`.
#[derive(Debug, Clone, Deserialize)]
pub struct ShardSubscriptionsResponse {
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    pub subscriptions: Vec<SubscriptionRecord>,
}

/// Request body for `POST /groups/discover/subscribe`. Either `key` or
/// `shard` must be set; `key` is normalised by x0xd before being hashed
/// into a shard id.
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeShardRequest {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<u32>,
}

/// Response from `POST /groups/discover/subscribe`.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeShardResponse {
    pub newly_added: bool,
    pub kind: ShardKind,
    pub shard: u32,
    pub topic: String,
}

/// Request body for `PATCH /groups/:id`.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateNamedGroupRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Request body for `PATCH /groups/:id/policy`. Either `preset` alone
/// or individual axes may be supplied; the daemon applies the preset first
/// and then overlays any explicit axes.
#[derive(Debug, Default, Clone, Serialize)]
pub struct UpdateGroupPolicyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverability: Option<GroupDiscoverability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admission: Option<GroupAdmission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidentiality: Option<GroupConfidentiality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_access: Option<GroupReadAccess>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_access: Option<GroupWriteAccess>,
}

/// Request body for `PATCH /groups/:id/members/:agent_id/role`.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateMemberRoleRequest {
    pub role: GroupRole,
}

/// Request body for `POST /groups/:id/requests`.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CreateJoinRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Request body for `POST /groups/:id/secure/encrypt`.
#[derive(Debug, Clone, Serialize)]
pub struct SecureEncryptRequest {
    pub payload_b64: String,
}

/// Response from `POST /groups/:id/secure/encrypt`.
#[derive(Debug, Clone, Deserialize)]
pub struct SecureEncryptResponse {
    pub ciphertext_b64: String,
    pub nonce_b64: String,
    pub secret_epoch: u64,
}

/// Request body for `POST /groups/:id/secure/decrypt`.
#[derive(Debug, Clone, Serialize)]
pub struct SecureDecryptRequest {
    pub ciphertext_b64: String,
    pub nonce_b64: String,
    pub secret_epoch: u64,
}

/// Response from `POST /groups/:id/secure/decrypt`.
#[derive(Debug, Clone, Deserialize)]
pub struct SecureDecryptResponse {
    pub plaintext_b64: String,
}

/// Request body for `POST /groups/:id/secure/reseal`.
#[derive(Debug, Clone, Serialize)]
pub struct SecureResealRequest {
    pub recipient: String,
}

/// Envelope produced by `POST /groups/:id/secure/reseal`. Also accepted by
/// `POST /groups/secure/open-envelope` (adversarial test).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureShareEnvelope {
    pub group_id: String,
    pub recipient: String,
    pub secret_epoch: u64,
    pub kem_ciphertext_b64: String,
    pub aead_nonce_b64: String,
    pub aead_ciphertext_b64: String,
}

/// Extended info from `POST /groups`. Mirrors the daemon response for
/// newly-created groups with policy presets applied.
#[derive(Debug, Clone, Serialize)]
pub struct CreateNamedGroupRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
}

//! Request and response types for the x0xd REST API.
//!
//! All types mirror the JSON shapes returned by x0xd on `127.0.0.1:12700`.
//! Payloads are base64-encoded bytes in the wire format; these types expose
//! them as `Vec<u8>` with serde helpers for the encoding.

use serde::{Deserialize, Serialize};

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

/// Response from `GET /peers`.
#[derive(Debug, Clone, Deserialize)]
pub struct PeerList {
    pub peers: Vec<String>,
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

/// A presence beacon from `GET /presence`.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceBeacon {
    pub agent_id: String,
    #[serde(default)]
    pub machine_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub last_seen: Option<u64>,
}

/// Response wrapper for `GET /presence`.
#[derive(Debug, Clone, Deserialize)]
pub struct PresenceList {
    pub agents: Vec<PresenceBeacon>,
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

/// Response from `POST /subscribe`.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribeResponse {
    pub id: String,
}

/// A gossip message received via SSE or WebSocket.
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
    pub machine_id: String,
    #[serde(default)]
    pub connected_at: Option<u64>,
}

/// Response from `GET /direct/connections`.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectConnectionList {
    pub connections: Vec<DirectConnection>,
}

/// A direct message received via SSE or WebSocket.
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
#[derive(Debug, Serialize)]
pub struct UpdateContactRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trust_level: Option<TrustLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
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

/// Request body for `POST /contacts/:id/machines`.
#[derive(Debug, Serialize)]
pub struct AddMachineRequest {
    pub machine_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
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
#[derive(Debug, Serialize)]
pub struct SendFileRequest {
    pub agent_id: String,
    pub filename: String,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
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

/// Request body for `POST /files/reject/:id`.
#[derive(Debug, Serialize)]
pub struct RejectFileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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

import Foundation

// MARK: - API Envelope

/// The x0x API uses **flattened** responses: `{"ok": true, "field": "value", ...}`.
/// There is no universal `data` wrapper. Each response type includes `ok` directly.
/// For decoding, we first check `ok` and extract the error if present,
/// then decode the full payload as the target type (which also contains `ok`).

/// A minimal probe used to check the `ok` field and extract errors.
public struct ApiEnvelope: Decodable {
    public let ok: Bool
    public let error: String?
}

/// An empty data payload for endpoints that return no body beyond `{"ok": true}`.
public struct Empty: Codable {
    public let ok: Bool?

    public init() {
        self.ok = nil
    }
}

// MARK: - Health & Status

/// Response from `GET /health`.
/// ```json
/// {"ok":true,"status":"healthy","version":"0.10.0","peers":4,"uptime_secs":300}
/// ```
public struct HealthStatus: Codable, Sendable {
    public let ok: Bool?
    public let status: String
    public let version: String?
    public let peers: UInt64?
    public let uptimeSecs: UInt64?

    enum CodingKeys: String, CodingKey {
        case ok, status, version, peers
        case uptimeSecs = "uptime_secs"
    }
}

/// Response from `GET /status`.
/// ```json
/// {"ok":true,"status":"connected","version":"0.10.0","uptime_secs":300,
///  "api_address":"127.0.0.1:12700","external_addrs":["..."],
///  "agent_id":"hex64","peers":4,"warnings":[]}
/// ```
public struct DaemonStatus: Codable, Sendable {
    public let ok: Bool?
    public let status: String?
    public let version: String?
    public let uptimeSecs: UInt64?
    public let apiAddress: String?
    public let externalAddrs: [String]?
    public let agentId: String?
    public let peers: UInt64?
    public let warnings: [String]?

    enum CodingKeys: String, CodingKey {
        case ok, status, version, peers, warnings
        case uptimeSecs = "uptime_secs"
        case apiAddress = "api_address"
        case externalAddrs = "external_addrs"
        case agentId = "agent_id"
    }
}

/// Response from `GET /agent`.
/// ```json
/// {"ok":true,"agent_id":"hex64","machine_id":"hex64","user_id":null}
/// ```
public struct AgentIdentity: Codable, Sendable, Identifiable {
    public var id: String { agentId }
    public let ok: Bool?
    public let agentId: String
    public let machineId: String?
    public let userId: String?

    enum CodingKeys: String, CodingKey {
        case ok
        case agentId = "agent_id"
        case machineId = "machine_id"
        case userId = "user_id"
    }
}

// MARK: - Pub/Sub

public struct PublishRequest: Codable, Sendable {
    public let topic: String
    public let payload: String // base64

    public init(topic: String, payload: String) {
        self.topic = topic
        self.payload = payload
    }
}

public struct SubscribeResponse: Codable, Sendable {
    public let ok: Bool?
    public let subscriptionId: String

    enum CodingKeys: String, CodingKey {
        case ok
        case subscriptionId = "subscription_id"
    }
}

// MARK: - Direct Messaging

public struct DirectMessageRequest: Codable, Sendable {
    public let agentId: String
    public let payload: String // base64

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case payload
    }

    public init(agentId: String, payload: String) {
        self.agentId = agentId
        self.payload = payload
    }
}

// MARK: - Gossip

public struct GossipMessage: Codable, Sendable, Identifiable {
    public var id: String { messageId }
    public let messageId: String
    public let topic: String
    public let sender: String
    public let payload: String // base64
    public let timestamp: UInt64

    enum CodingKeys: String, CodingKey {
        case messageId = "message_id"
        case topic, sender, payload, timestamp
    }
}

public struct DirectMessage: Codable, Sendable, Identifiable {
    public var id: String { messageId }
    public let messageId: String
    public let sender: String
    public let payload: String // base64
    public let timestamp: UInt64

    enum CodingKeys: String, CodingKey {
        case messageId = "message_id"
        case sender, payload, timestamp
    }
}

// MARK: - Tasks

/// Wrapper for `GET /task-lists/:id/tasks` response: `{"ok":true,"tasks":[...]}`.
public struct TaskList: Codable, Sendable {
    public let ok: Bool?
    public let tasks: [TaskItem]
}

/// A task item returned by `GET /task-lists/:id/tasks`.
/// Daemon returns: `{ id, title, description, state, assignee?, priority }`.
public struct TaskItem: Codable, Sendable, Identifiable {
    public let id: String
    public let title: String?
    public let description: String
    public let state: String
    public let assignee: String?
    public let priority: UInt8?
}

// MARK: - Task List Requests

public struct CreateTaskListRequest: Codable, Sendable {
    public let name: String
    public let topic: String

    public init(name: String, topic: String) {
        self.name = name
        self.topic = topic
    }
}

/// Response from creating a task list: `{"ok": true, "id": "..."}`.
public struct CreateTaskListResponse: Codable, Sendable {
    public let ok: Bool?
    public let id: String
}

public struct AddTaskRequest: Codable, Sendable {
    public let title: String

    public init(title: String) {
        self.title = title
    }
}

public struct AddTaskResponse: Codable, Sendable {
    public let ok: Bool?
    public let taskId: String

    enum CodingKeys: String, CodingKey {
        case ok
        case taskId = "task_id"
    }
}

// MARK: - KV Store (now /stores/:id/:key)

/// Request body for creating a store.
public struct CreateStoreRequest: Codable, Sendable {
    public let name: String
    public let topic: String

    public init(name: String, topic: String) {
        self.name = name
        self.topic = topic
    }
}

/// Response after creating a store: `{"ok": true, "id": "..."}`.
public struct CreateStoreResponse: Codable, Sendable {
    public let ok: Bool?
    public let id: String
}

/// A store summary from `GET /stores`: `{"id": "...", "topic": "..."}`.
public struct StoreSummary: Codable, Sendable, Identifiable {
    public let id: String
    public let topic: String?
}

/// Wrapper for `GET /stores` response.
public struct StoreListResponse: Codable, Sendable {
    public let ok: Bool?
    public let stores: [StoreSummary]
}

/// Request body for `PUT /stores/:id/:key`.
public struct StorePutRequest: Codable, Sendable {
    public let value: String // base64
    public let contentType: String?

    enum CodingKeys: String, CodingKey {
        case value
        case contentType = "content_type"
    }

    public init(value: String, contentType: String? = nil) {
        self.value = value
        self.contentType = contentType
    }
}

/// Response from `GET /stores/:id/:key`.
public struct StoreGetResponse: Codable, Sendable {
    public let ok: Bool?
    public let value: String
    public let contentType: String?

    enum CodingKeys: String, CodingKey {
        case ok, value
        case contentType = "content_type"
    }
}

// MARK: - Network

/// Response from `GET /network/status`.
/// ```json
/// {"ok":true,"avg_rtt_ms":76,"can_receive_direct":true,"connected_peers":4,
///  "direct_connections":11,"external_addrs":["..."],"hole_punch_success_rate":0.0,
///  "nat_type":"FullCone",...}
/// ```
public struct NetworkStatus: Codable, Sendable {
    public let ok: Bool?
    public let avgRttMs: Double?
    public let canReceiveDirect: Bool?
    public let connectedPeers: UInt64?
    public let directConnections: UInt64?
    public let externalAddrs: [String]?
    public let holePunchSuccessRate: Double?
    public let natType: String?

    enum CodingKeys: String, CodingKey {
        case ok
        case avgRttMs = "avg_rtt_ms"
        case canReceiveDirect = "can_receive_direct"
        case connectedPeers = "connected_peers"
        case directConnections = "direct_connections"
        case externalAddrs = "external_addrs"
        case holePunchSuccessRate = "hole_punch_success_rate"
        case natType = "nat_type"
    }
}

/// Response from `GET /peers` - wrapped: `{"ok":true,"peers":[{"id":"hex"}]}`.
public struct PeerListResponse: Codable, Sendable {
    public let ok: Bool?
    public let peers: [PeerInfo]

    /// Helper accessor for symmetry with other list responses.
    public var peerInfos: [PeerInfo] {
        peers
    }
}

/// A peer entry from `GET /peers`. The daemon returns `{"id":"hex"}` objects.
public struct PeerInfo: Codable, Sendable, Identifiable {
    public var id: String { peerId }
    public let peerId: String

    enum CodingKeys: String, CodingKey {
        case peerId = "id"
    }

    public init(peerId: String) {
        self.peerId = peerId
    }
}

/// Response from `GET /agents/discovered` - wrapped: `{"ok":true,"agents":[...]}`.
public struct DiscoveredAgentsResponse: Codable, Sendable {
    public let ok: Bool?
    public let agents: [DiscoveredAgent]
}

/// A discovered agent entry from `GET /agents/discovered`.
/// Daemon returns: `{ agent_id, machine_id, user_id?, addresses: [String], announced_at, last_seen }`.
public struct DiscoveredAgent: Codable, Sendable, Identifiable {
    public var id: String { agentId }
    public let agentId: String
    public let machineId: String
    public let userId: String?
    public let addresses: [String]
    public let announcedAt: UInt64?
    public let lastSeen: UInt64?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case machineId = "machine_id"
        case userId = "user_id"
        case addresses
        case announcedAt = "announced_at"
        case lastSeen = "last_seen"
    }

    public init(agentId: String, machineId: String, userId: String?, addresses: [String], announcedAt: UInt64?, lastSeen: UInt64?) {
        self.agentId = agentId
        self.machineId = machineId
        self.userId = userId
        self.addresses = addresses
        self.announcedAt = announcedAt
        self.lastSeen = lastSeen
    }
}

/// Response from `GET /presence` - wrapped: `{"ok":true,"agents":["hex1","hex2"]}`.
public struct PresenceResponse: Codable, Sendable {
    public let ok: Bool?
    public let agents: [String]
}

// MARK: - List Wrapper Responses

/// Response from `GET /contacts` - wrapped: `{"ok":true,"contacts":[...]}`.
public struct ContactListResponse: Codable, Sendable {
    public let ok: Bool?
    public let contacts: [Contact]
}

/// Response from `GET /groups` - wrapped: `{"ok":true,"groups":[...]}`.
public struct GroupListResponse: Codable, Sendable {
    public let ok: Bool?
    public let groups: [GroupSummary]
}

/// Request body for `PUT /groups/:id/display-name`.
/// The daemon expects `{"name": "..."}` — NOT `"display_name"`.
public struct SetGroupDisplayNameRequest: Codable, Sendable {
    public let name: String

    enum CodingKeys: String, CodingKey {
        case name
    }

    public init(displayName: String) {
        self.name = displayName
    }
}

// MARK: - Identity & Agent Card

/// Response from `GET /agent/user-id`.
/// ```json
/// {"ok":true,"user_id":"hex64_or_null"}
/// ```
public struct UserIdStatus: Codable, Sendable {
    public let ok: Bool?
    public let userId: String?

    enum CodingKeys: String, CodingKey {
        case ok
        case userId = "user_id"
    }
}

/// A group entry inside an agent card.
public struct CardGroup: Codable, Sendable {
    public let name: String
    public let inviteLink: String

    enum CodingKeys: String, CodingKey {
        case name
        case inviteLink = "invite_link"
    }
}

/// A store entry inside an agent card.
public struct CardStore: Codable, Sendable {
    public let name: String
    public let topic: String
}

/// An agent card containing shareable identity and membership info.
public struct AgentCard: Codable, Sendable {
    public let displayName: String
    public let agentId: String
    public let machineId: String
    public let userId: String?
    public let externalAddresses: [String]?
    public let groups: [CardGroup]?
    public let stores: [CardStore]?
    public let createdAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case displayName = "display_name"
        case agentId = "agent_id"
        case machineId = "machine_id"
        case userId = "user_id"
        case externalAddresses = "external_addresses"
        case groups, stores
        case createdAt = "created_at"
    }
}

/// Response from `GET /agent/card`.
/// ```json
/// {"ok":true,"card":{...},"link":"x0x://agent/..."}
/// ```
public struct AgentCardResponse: Codable, Sendable {
    public let ok: Bool?
    public let card: AgentCard
    public let link: String
}

/// Request body for `POST /agent/card/import`.
public struct ImportCardRequest: Codable, Sendable {
    public let card: String
    public let trustLevel: String?

    enum CodingKeys: String, CodingKey {
        case card
        case trustLevel = "trust_level"
    }

    public init(card: String, trustLevel: TrustLevel?) {
        self.card = card
        self.trustLevel = trustLevel?.rawValue
    }
}

/// Response from `POST /agent/card/import`.
public struct ImportCardResponse: Codable, Sendable {
    public let ok: Bool?
    public let agentId: String?
    public let displayName: String?
    public let trustLevel: String?
    public let groups: Int?
    public let stores: Int?

    enum CodingKeys: String, CodingKey {
        case ok
        case agentId = "agent_id"
        case displayName = "display_name"
        case trustLevel = "trust_level"
        case groups, stores
    }
}

/// Request body for `POST /announce`.
public struct AnnounceRequest: Codable, Sendable {
    public let includeUserIdentity: Bool
    public let humanConsent: Bool

    enum CodingKeys: String, CodingKey {
        case includeUserIdentity = "include_user_identity"
        case humanConsent = "human_consent"
    }

    public init(includeUserIdentity: Bool, humanConsent: Bool) {
        self.includeUserIdentity = includeUserIdentity
        self.humanConsent = humanConsent
    }
}

// MARK: - Direct Connection

/// A direct (QUIC) connection to another agent.
/// Returned by `GET /direct/connections`.
public struct DirectConnection: Codable, Sendable, Identifiable {
    public var id: String { agentId }
    public let agentId: String
    public let machineId: String?
    public let connectedAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case machineId = "machine_id"
        case connectedAt = "connected_at"
    }
}

/// Wrapper for `GET /direct/connections` response.
public struct DirectConnectionList: Codable, Sendable {
    public let ok: Bool?
    public let connections: [DirectConnection]
}

// MARK: - Contact Management (Extended)

/// Request body for `POST /contacts/trust`.
public struct SetTrustRequest: Codable, Sendable {
    public let agentId: String
    public let level: String

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case level
    }

    public init(agentId: String, level: TrustLevel) {
        self.agentId = agentId
        self.level = level.rawValue
    }
}

/// Request body for `PATCH /contacts/:agent_id`.
public struct UpdateContactRequest: Codable, Sendable {
    public let trustLevel: String?
    public let identityType: String?

    enum CodingKeys: String, CodingKey {
        case trustLevel = "trust_level"
        case identityType = "identity_type"
    }

    public init(trustLevel: TrustLevel?, identityType: String?) {
        self.trustLevel = trustLevel?.rawValue
        self.identityType = identityType
    }
}

/// A revocation record from `GET /contacts/:agent_id/revocations`.
public struct Revocation: Codable, Sendable {
    public let agentId: String?
    public let reason: String?
    public let timestamp: UInt64?
    public let revokerId: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case reason, timestamp
        case revokerId = "revoker_id"
    }
}

/// Wrapper for `GET /contacts/:agent_id/revocations` response.
public struct RevocationList: Codable, Sendable {
    public let ok: Bool?
    public let revocations: [Revocation]
}

/// Request body for `POST /contacts/:agent_id/machines`.
public struct AddMachineRequest: Codable, Sendable {
    public let machineId: String
    public let label: String?
    public let pinned: Bool?

    enum CodingKeys: String, CodingKey {
        case machineId = "machine_id"
        case label, pinned
    }

    public init(machineId: String, label: String?, pinned: Bool?) {
        self.machineId = machineId
        self.label = label
        self.pinned = pinned
    }
}

// MARK: - Trust Evaluation

/// Request body for `POST /trust/evaluate`.
public struct EvaluateTrustRequest: Codable, Sendable {
    public let agentId: String
    public let machineId: String

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case machineId = "machine_id"
    }

    public init(agentId: String, machineId: String) {
        self.agentId = agentId
        self.machineId = machineId
    }
}

/// Response from `POST /trust/evaluate`.
public struct TrustEvaluation: Codable, Sendable {
    public let ok: Bool?
    public let decision: String
}

// MARK: - Bootstrap Cache

/// Response from `GET /network/bootstrap-cache`.
public struct BootstrapCacheStatus: Codable, Sendable {
    public let ok: Bool?
    public let connectedPeers: [String]?
    public let connectionCount: UInt32?

    enum CodingKeys: String, CodingKey {
        case ok
        case connectedPeers = "connected_peers"
        case connectionCount = "connection_count"
    }
}

// MARK: - File Transfer (Extended)

/// Request body for `POST /files/reject/:id` with optional reason.
public struct RejectFileRequest: Codable, Sendable {
    public let reason: String?

    public init(reason: String?) {
        self.reason = reason
    }
}

/// Wrapper for `GET /files/transfers/:id` response (single transfer).
public struct FileTransferWrapper: Codable, Sendable {
    public let ok: Bool?
    public let transferId: String?
    public let direction: TransferDirection?
    public let remoteAgentId: String?
    public let filename: String?
    public let totalSize: UInt64?
    public let bytesTransferred: UInt64?
    public let status: TransferStatus?
    public let sha256: String?
    public let error: String?
    public let startedAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case ok
        case transferId = "transfer_id"
        case direction
        case remoteAgentId = "remote_agent_id"
        case filename
        case totalSize = "total_size"
        case bytesTransferred = "bytes_transferred"
        case status, sha256, error
        case startedAt = "started_at"
    }
}

// MARK: - WebSocket Sessions

/// A WebSocket session info entry.
public struct WsSessionInfo: Codable, Sendable {
    public let sessionId: String
    public let subscribedTopics: [String]
    public let receivesDirect: Bool

    enum CodingKeys: String, CodingKey {
        case sessionId = "session_id"
        case subscribedTopics = "subscribed_topics"
        case receivesDirect = "receives_direct"
    }
}

/// Response from `GET /ws/sessions`.
public struct WsSessionList: Codable, Sendable {
    public let ok: Bool?
    public let sessions: [WsSessionInfo]
    public let sharedSubscriptions: [String: Int]?

    enum CodingKeys: String, CodingKey {
        case ok, sessions
        case sharedSubscriptions = "shared_subscriptions"
    }
}

// MARK: - Task Lists (Extended)

/// A task list summary from `GET /task-lists`.
public struct TaskListSummary: Codable, Sendable, Identifiable {
    public let id: String
    public let topic: String?
}

/// Wrapper for `GET /task-lists` response.
public struct TaskListIndex: Codable, Sendable {
    public let ok: Bool?
    public let taskLists: [TaskListSummary]

    enum CodingKeys: String, CodingKey {
        case ok
        case taskLists = "task_lists"
    }
}

// MARK: - Upgrade

/// Response from `GET /upgrade`.
public struct UpgradeStatus: Codable, Sendable {
    public let ok: Bool?
    public let updateAvailable: Bool?
    public let version: String?
    public let currentVersion: String?

    enum CodingKeys: String, CodingKey {
        case ok
        case updateAvailable = "update_available"
        case version
        case currentVersion = "current_version"
    }
}

// MARK: - Discovered Agent (Single)

/// Wrapper for `GET /agents/discovered/:agent_id` response (flattened).
public struct DiscoveredAgentWrapper: Codable, Sendable {
    public let ok: Bool?
    public let agentId: String
    public let machineId: String
    public let userId: String?
    public let addresses: [String]
    public let announcedAt: UInt64?
    public let lastSeen: UInt64?

    enum CodingKeys: String, CodingKey {
        case ok
        case agentId = "agent_id"
        case machineId = "machine_id"
        case userId = "user_id"
        case addresses
        case announcedAt = "announced_at"
        case lastSeen = "last_seen"
    }

    /// Convert to a `DiscoveredAgent` value.
    public func toDiscoveredAgent() -> DiscoveredAgent {
        DiscoveredAgent(
            agentId: agentId,
            machineId: machineId,
            userId: userId,
            addresses: addresses,
            announcedAt: announcedAt,
            lastSeen: lastSeen
        )
    }
}

// MARK: - Daemon State

public enum DaemonState: String, Sendable {
    case notInstalled
    case notRunning
    case starting
    case running
    case error
}

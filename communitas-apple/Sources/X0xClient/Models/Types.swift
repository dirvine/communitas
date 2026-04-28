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
    /// Opt-in: wait up to this many ms for an ant-quic `probe_peer` ACK
    /// after send. Omit for legacy fire-and-forget. (x0xd ≥ 0.19.6.)
    public let requireAckMs: UInt64?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case payload
        case requireAckMs = "require_ack_ms"
    }

    public init(agentId: String, payload: String, requireAckMs: UInt64? = nil) {
        self.agentId = agentId
        self.payload = payload
        self.requireAckMs = requireAckMs
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
    public let description: String?

    public init(title: String, description: String? = nil) {
        self.title = title
        self.description = description
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
    public let key: String?
    public let value: String
    public let contentType: String?
    public let contentHash: String?
    public let createdAt: UInt64?
    public let updatedAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case ok, key, value
        case contentType = "content_type"
        case contentHash = "content_hash"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }
}

/// A key entry returned by `GET /stores/:id/keys`.
public struct StoreKeyEntry: Codable, Sendable {
    public let key: String
    public let contentType: String?
    public let contentHash: String?
    public let size: UInt64?
    public let updatedAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case key
        case contentType = "content_type"
        case contentHash = "content_hash"
        case size
        case updatedAt = "updated_at"
    }
}

/// Wrapper for `GET /stores/:id/keys`.
public struct StoreKeysResponse: Codable, Sendable {
    public let ok: Bool?
    public let keys: [StoreKeyEntry]
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
    public let coordinationSessions: UInt64?
    public let directConnections: UInt64?
    public let externalAddrs: [String]?
    public let hasPublicIP: Bool?
    public let holePunchSuccessRate: Double?
    public let isCoordinating: Bool?
    public let isRelaying: Bool?
    public let localAddr: String?
    public let natType: String?
    public let relaySessions: UInt64?
    public let relayedConnections: UInt64?
    public let uptimeSecs: UInt64?

    enum CodingKeys: String, CodingKey {
        case ok
        case avgRttMs = "avg_rtt_ms"
        case canReceiveDirect = "can_receive_direct"
        case connectedPeers = "connected_peers"
        case coordinationSessions = "coordination_sessions"
        case directConnections = "direct_connections"
        case externalAddrs = "external_addrs"
        case hasPublicIP = "has_public_ip"
        case holePunchSuccessRate = "hole_punch_success_rate"
        case isCoordinating = "is_coordinating"
        case isRelaying = "is_relaying"
        case localAddr = "local_addr"
        case natType = "nat_type"
        case relaySessions = "relay_sessions"
        case relayedConnections = "relayed_connections"
        case uptimeSecs = "uptime_secs"
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

/// PubSub drop-detection counters from `GET /diagnostics/gossip`.
public struct GossipStats: Codable, Sendable {
    public let publishTotal: UInt64
    public let publishFailed: UInt64
    public let incomingTotal: UInt64
    public let incomingDecoded: UInt64
    public let incomingDecodeFailed: UInt64
    public let deliveredToSubscriber: UInt64
    public let subscriberChannelClosed: UInt64
    public let inFlightDecode: Int64
    public let decodeToDeliveryDrops: Int64

    enum CodingKeys: String, CodingKey {
        case publishTotal = "publish_total"
        case publishFailed = "publish_failed"
        case incomingTotal = "incoming_total"
        case incomingDecoded = "incoming_decoded"
        case incomingDecodeFailed = "incoming_decode_failed"
        case deliveredToSubscriber = "delivered_to_subscriber"
        case subscriberChannelClosed = "subscriber_channel_closed"
        case inFlightDecode = "in_flight_decode"
        case decodeToDeliveryDrops = "decode_to_delivery_drops"
    }
}

/// Response wrapper from `GET /diagnostics/gossip`.
public struct GossipStatsResponse: Codable, Sendable {
    public let ok: Bool?
    public let stats: GossipStats
}

/// Active-liveness probe result from `POST /peers/:peer_id/probe`.
public struct ProbePeerResult: Codable, Sendable {
    public let ok: Bool?
    public let rttMs: UInt64?
    public let rttUs: UInt64?
    public let timeoutMs: UInt64?
    public let error: String?

    enum CodingKeys: String, CodingKey {
        case ok, error
        case rttMs = "rtt_ms"
        case rttUs = "rtt_us"
        case timeoutMs = "timeout_ms"
    }
}

/// Connection health snapshot from `GET /peers/:peer_id/health`.
///
/// `health` is the legacy Debug-rendered string; new code should prefer
/// `snapshot` (structured fields, x0xd ≥ 0.19.7). Both are populated when
/// the daemon supports it; older daemons return `health` only.
public struct PeerHealth: Codable, Sendable {
    public let ok: Bool?
    public let peerId: String?
    public let health: String?
    public let snapshot: PeerHealthSnapshot?
    public let error: String?

    enum CodingKeys: String, CodingKey {
        case ok, health, snapshot, error
        case peerId = "peer_id"
    }
}

/// Structured peer connection-health snapshot (x0xd ≥ 0.19.7).
///
/// `Instant`-typed ant-quic fields are converted to elapsed-millisecond
/// deltas so the wire format stays calendar-agnostic.
public struct PeerHealthSnapshot: Codable, Sendable {
    public let connected: Bool
    public let generation: UInt64?
    public let readerTaskActive: Bool?
    public let lastReceivedMsAgo: UInt64?
    public let lastSentMsAgo: UInt64?
    public let idleMs: UInt64?
    /// Most-recent close reason as a Debug string (no canonical Codable
    /// upstream yet).
    public let closeReason: String?

    enum CodingKeys: String, CodingKey {
        case connected, generation
        case readerTaskActive = "reader_task_active"
        case lastReceivedMsAgo = "last_received_ms_ago"
        case lastSentMsAgo = "last_sent_ms_ago"
        case idleMs = "idle_ms"
        case closeReason = "close_reason"
    }
}

/// Response from `POST /direct/send` (x0xd ≥ 0.19.6).
///
/// When the request omits `require_ack_ms`, the daemon does not include
/// the `requireAck` block — `requireAck` will be `nil` here for legacy
/// fire-and-forget behaviour.
public struct DirectSendResponse: Codable, Sendable {
    public let ok: Bool
    public let path: String?
    public let requestId: String?
    public let retriesUsed: UInt64?
    public let requireAck: RequireAckResult?
    public let error: String?

    enum CodingKeys: String, CodingKey {
        case ok, path, error
        case requestId = "request_id"
        case retriesUsed = "retries_used"
        case requireAck = "require_ack"
    }
}

/// Active-liveness ACK probe carried in `DirectSendResponse.requireAck`.
///
/// Mirrors `ProbePeerResult` but is emitted as part of the send response
/// rather than as a standalone probe call.
public struct RequireAckResult: Codable, Sendable {
    public let ok: Bool
    public let rttMs: UInt64?
    public let rttUs: UInt64?
    public let error: String?
    public let reason: String?

    enum CodingKeys: String, CodingKey {
        case ok, error, reason
        case rttMs = "rtt_ms"
        case rttUs = "rtt_us"
    }
}

/// One frame on `GET /peers/events` (x0xd ≥ 0.19.6).
///
/// The wire shape is `{"peer_id":"...","event":"Established { generation: 5 }",
/// "at_ms":1777370802198}`. `event` is a Debug-rendered enum string —
/// substring-match on `"Established"`, `"Replaced"`, `"Closing"`,
/// `"Closed"`, `"ReaderExited"` rather than reaching for a typed enum.
public struct PeerLifecycleEvent: Codable, Sendable {
    public let peerId: String
    public let event: String
    public let atMs: UInt64

    enum CodingKeys: String, CodingKey {
        case event
        case peerId = "peer_id"
        case atMs = "at_ms"
    }
}

/// Full ant-quic connectivity snapshot from `GET /diagnostics/connectivity`.
public struct ConnectivityDiagnostics: Codable, Sendable {
    public let ok: Bool?
    public let peerId: String
    public let localAddr: String
    public let externalAddrs: [String]
    public let natType: String
    public let canReceiveDirect: Bool
    public let directReachabilityScope: String
    public let hasGlobalAddress: Bool
    public let portMapping: PortMappingDiagnostics
    public let mdns: MdnsDiagnostics
    public let services: ServiceDiagnostics
    public let connections: ConnectionDiagnostics
    public let relay: RelayDiagnostics
    public let coordinator: CoordinatorDiagnostics
    public let avgRttMs: UInt64
    public let uptimeS: UInt64

    enum CodingKeys: String, CodingKey {
        case ok
        case peerId = "peer_id"
        case localAddr = "local_addr"
        case externalAddrs = "external_addrs"
        case natType = "nat_type"
        case canReceiveDirect = "can_receive_direct"
        case directReachabilityScope = "direct_reachability_scope"
        case hasGlobalAddress = "has_global_address"
        case portMapping = "port_mapping"
        case mdns, services, connections, relay, coordinator
        case avgRttMs = "avg_rtt_ms"
        case uptimeS = "uptime_s"
    }
}

public struct PortMappingDiagnostics: Codable, Sendable {
    public let active: Bool
    public let externalAddr: String?

    enum CodingKeys: String, CodingKey {
        case active
        case externalAddr = "external_addr"
    }
}

public struct MdnsDiagnostics: Codable, Sendable {
    public let browsing: Bool
    public let advertising: Bool
    public let discoveredPeers: UInt64

    enum CodingKeys: String, CodingKey {
        case browsing, advertising
        case discoveredPeers = "discovered_peers"
    }
}

public struct ServiceDiagnostics: Codable, Sendable {
    public let relayEnabled: Bool
    public let coordinatorEnabled: Bool
    public let bootstrapEnabled: Bool

    enum CodingKeys: String, CodingKey {
        case relayEnabled = "relay_enabled"
        case coordinatorEnabled = "coordinator_enabled"
        case bootstrapEnabled = "bootstrap_enabled"
    }
}

public struct ConnectionDiagnostics: Codable, Sendable {
    public let connectedPeers: UInt64
    public let active: UInt64
    public let direct: UInt64
    public let relayed: UInt64
    public let holePunchSuccessRate: Double

    enum CodingKeys: String, CodingKey {
        case connectedPeers = "connected_peers"
        case active, direct, relayed
        case holePunchSuccessRate = "hole_punch_success_rate"
    }
}

public struct RelayDiagnostics: Codable, Sendable {
    public let isRelaying: Bool
    public let sessions: UInt64
    public let bytesForwarded: UInt64

    enum CodingKeys: String, CodingKey {
        case isRelaying = "is_relaying"
        case sessions
        case bytesForwarded = "bytes_forwarded"
    }
}

public struct CoordinatorDiagnostics: Codable, Sendable {
    public let isCoordinating: Bool
    public let sessions: UInt64

    enum CodingKeys: String, CodingKey {
        case isCoordinating = "is_coordinating"
        case sessions
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

/// Response from `GET /machines/discovered` - wrapped: `{"ok":true,"machines":[...]}`.
public struct DiscoveredMachinesResponse: Codable, Sendable {
    public let ok: Bool?
    public let machines: [DiscoveredMachine]
}

/// A discovered machine endpoint entry from x0x machine announcements.
public struct DiscoveredMachine: Codable, Sendable, Identifiable {
    public var id: String { machineId }
    public let machineId: String
    public let addresses: [String]
    public let announcedAt: UInt64
    public let lastSeen: UInt64
    public let natType: String?
    public let canReceiveDirect: Bool?
    public let isRelay: Bool?
    public let isCoordinator: Bool?
    public let agentIds: [String]
    public let userIds: [String]

    enum CodingKeys: String, CodingKey {
        case machineId = "machine_id"
        case addresses
        case announcedAt = "announced_at"
        case lastSeen = "last_seen"
        case natType = "nat_type"
        case canReceiveDirect = "can_receive_direct"
        case isRelay = "is_relay"
        case isCoordinator = "is_coordinator"
        case agentIds = "agent_ids"
        case userIds = "user_ids"
    }
}

/// Wrapper for `GET /machines/discovered/:machine_id`.
public struct DiscoveredMachineWrapper: Codable, Sendable {
    public let ok: Bool?
    public let machine: DiscoveredMachine
}

/// Response from `GET /agents/:agent_id/machine`.
public struct AgentMachine: Codable, Sendable {
    public let ok: Bool?
    public let agentId: String
    public let machine: DiscoveredMachine

    enum CodingKeys: String, CodingKey {
        case ok
        case agentId = "agent_id"
        case machine
    }
}

/// Response from `GET /users/:user_id/machines`.
public struct UserMachineList: Codable, Sendable {
    public let ok: Bool?
    public let userId: String
    public let machines: [DiscoveredMachine]

    enum CodingKeys: String, CodingKey {
        case ok
        case userId = "user_id"
        case machines
    }
}

/// Request body for `POST /machines/connect`.
public struct ConnectMachineRequest: Codable, Sendable {
    public let machineId: String

    enum CodingKeys: String, CodingKey {
        case machineId = "machine_id"
    }

    public init(machineId: String) {
        self.machineId = machineId
    }
}

/// Response from `POST /machines/connect`.
public struct ConnectMachineResponse: Codable, Sendable {
    public let ok: Bool?
    public let outcome: String
    public let addr: String?

    public init(ok: Bool? = nil, outcome: String, addr: String? = nil) {
        self.ok = ok
        self.outcome = outcome
        self.addr = addr
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
    public let addresses: [String]?
    public let groups: [CardGroup]?
    public let stores: [CardStore]?
    public let createdAt: UInt64?

    /// Backward-compatible alias for older call sites that still refer to the
    /// previous `externalAddresses` name.
    public var externalAddresses: [String]? {
        addresses
    }

    enum CodingKeys: String, CodingKey {
        case displayName = "display_name"
        case agentId = "agent_id"
        case machineId = "machine_id"
        case userId = "user_id"
        case addresses
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

/// A trust-scoped service entry from `GET /introduction`.
public struct IntroductionService: Codable, Sendable {
    public let name: String
    public let description: String
    public let minTrust: String

    enum CodingKeys: String, CodingKey {
        case name, description
        case minTrust = "min_trust"
    }
}

/// Trust-gated introduction card returned by `GET /introduction`.
public struct IntroductionCard: Codable, Sendable {
    public let ok: Bool?
    public let agentId: String
    public let machineId: String?
    public let userId: String?
    public let certificate: String?
    public let displayName: String?
    public let identityWords: String
    public let services: [IntroductionService]
    public let signature: String?

    enum CodingKeys: String, CodingKey {
        case ok
        case agentId = "agent_id"
        case machineId = "machine_id"
        case userId = "user_id"
        case certificate
        case displayName = "display_name"
        case identityWords = "identity_words"
        case services
        case signature
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
    public let transfer: FileTransfer
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

/// Response from `GET /constitution/json`.
public struct ConstitutionInfo: Codable, Sendable {
    public let version: String
    public let status: String
    public let content: String
}

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

/// Wrapper for `GET /agents/discovered/:agent_id` response.
public struct DiscoveredAgentWrapper: Codable, Sendable {
    public let ok: Bool?
    public let agent: DiscoveredAgent
}

// MARK: - Presence (Extended)

/// Response from `GET /presence/status/:id`.
public struct PresenceStatusResponse: Codable, Sendable {
    public let ok: Bool?
    public let online: Bool
    public let agent: DiscoveredAgent?
}

/// Response wrapper for `GET /presence/find/:id` — optional agent.
public struct PresenceFindResponse: Codable, Sendable {
    public let ok: Bool?
    public let agent: DiscoveredAgent?
}

// MARK: - Agent Discovery (Extended)

/// Response from `GET /agents/reachability/:agent_id`.
public struct ReachabilityInfo: Codable, Sendable {
    public let likelyDirect: Bool
    public let needsCoordination: Bool
    public let isRelay: Bool
    public let isCoordinator: Bool
    public let addresses: [String]

    enum CodingKeys: String, CodingKey {
        case likelyDirect = "likely_direct"
        case needsCoordination = "needs_coordination"
        case isRelay = "is_relay"
        case isCoordinator = "is_coordinator"
        case addresses
    }
}

/// Response from `POST /agents/find/:agent_id`.
public struct FindAgentResponse: Codable, Sendable {
    public let ok: Bool?
    public let found: Bool
    public let addresses: [String]?
}

/// Response from `GET /users/:user_id/agents`.
public struct UserAgentsResponse: Codable, Sendable {
    public let ok: Bool?
    public let userId: String
    public let agents: [DiscoveredAgent]

    enum CodingKeys: String, CodingKey {
        case ok
        case userId = "user_id"
        case agents
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

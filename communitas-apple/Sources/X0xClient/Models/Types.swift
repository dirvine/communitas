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

public struct TaskList: Codable, Sendable {
    public let tasks: [TaskItem]
}

public struct TaskItem: Codable, Sendable, Identifiable {
    public var id: String { taskId }
    public let taskId: String
    public let description: String
    public let status: String

    enum CodingKeys: String, CodingKey {
        case taskId = "task_id"
        case description, status
    }
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

public struct CreateTaskListResponse: Codable, Sendable {
    public let ok: Bool?
    public let listId: String

    enum CodingKeys: String, CodingKey {
        case ok
        case listId = "list_id"
    }
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

/// Response after creating a store.
public struct CreateStoreResponse: Codable, Sendable {
    public let ok: Bool?
    public let storeId: String

    enum CodingKeys: String, CodingKey {
        case ok
        case storeId = "store_id"
    }
}

/// A store summary from `GET /stores`.
public struct StoreSummary: Codable, Sendable, Identifiable {
    public var id: String { storeId }
    public let storeId: String
    public let name: String
    public let topic: String?

    enum CodingKeys: String, CodingKey {
        case storeId = "store_id"
        case name, topic
    }
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

/// Response from `GET /peers` - wrapped: `{"ok":true,"peers":["peer1","peer2"]}`.
public struct PeerListResponse: Codable, Sendable {
    public let ok: Bool?
    public let peers: [String]

    /// Helper to create PeerInfo models from the raw peer ID strings.
    public var peerInfos: [PeerInfo] {
        peers.map { PeerInfo(peerId: $0) }
    }
}

/// A peer for display. The `/peers` endpoint returns string IDs;
/// we wrap them for UI convenience.
public struct PeerInfo: Codable, Sendable, Identifiable {
    public var id: String { peerId }
    public let peerId: String

    enum CodingKeys: String, CodingKey {
        case peerId = "peer_id"
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

public struct DiscoveredAgent: Codable, Sendable, Identifiable {
    public var id: String { agentId }
    public let agentId: String
    public let displayName: String?
    public let lastSeen: UInt64?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case displayName = "display_name"
        case lastSeen = "last_seen"
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

public struct SetGroupDisplayNameRequest: Codable, Sendable {
    public let displayName: String

    enum CodingKeys: String, CodingKey {
        case displayName = "display_name"
    }

    public init(displayName: String) {
        self.displayName = displayName
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

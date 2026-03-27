import Foundation

// MARK: - API Envelope

/// The standard x0x API response envelope: `{"ok": true, "data": {...}}`.
public struct ApiResponse<T: Decodable>: Decodable {
    public let ok: Bool
    public let data: T?
    public let error: String?
}

/// An empty data payload for endpoints that return no body.
public struct Empty: Codable {
    public init() {}
}

// MARK: - Health & Status

public struct HealthStatus: Codable, Sendable {
    public let status: String
    public let version: String?
    public let uptime: UInt64?
}

public struct DaemonStatus: Codable, Sendable {
    public let running: Bool
    public let agentId: String?
    public let peerCount: UInt64?
    public let version: String?

    enum CodingKeys: String, CodingKey {
        case running
        case agentId = "agent_id"
        case peerCount = "peer_count"
        case version
    }
}

public struct AgentIdentity: Codable, Sendable, Identifiable {
    public var id: String { agentId }
    public let agentId: String
    public let publicKey: String?
    public let fourWords: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case publicKey = "public_key"
        case fourWords = "four_words"
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
    public let subscriptionId: String

    enum CodingKeys: String, CodingKey {
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
    public let listId: String

    enum CodingKeys: String, CodingKey {
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
    public let taskId: String

    enum CodingKeys: String, CodingKey {
        case taskId = "task_id"
    }
}

// MARK: - KV Store

public struct KvStore: Codable, Sendable {
    public let key: String
    public let value: String
}

public struct StoreValue: Codable, Sendable {
    public let value: String
}

// MARK: - Network

public struct NetworkStatus: Codable, Sendable {
    public let connected: Bool
    public let peerCount: UInt64?
    public let listenAddresses: [String]?

    enum CodingKeys: String, CodingKey {
        case connected
        case peerCount = "peer_count"
        case listenAddresses = "listen_addresses"
    }
}

public struct PeerInfo: Codable, Sendable, Identifiable {
    public var id: String { peerId }
    public let peerId: String
    public let address: String?
    public let latency: UInt64?

    enum CodingKeys: String, CodingKey {
        case peerId = "peer_id"
        case address, latency
    }
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

import Foundation

/// Trust level assigned to a contact.
public enum TrustLevel: String, Codable, Sendable, CaseIterable, Hashable {
    case untrusted
    case known
    case trusted
    case verified
}

/// A contact stored in the x0x daemon.
/// ```json
/// {"agent_id":"hex","trust_level":"known","label":"Alice","added_at":1234,"last_seen":null}
/// ```
public struct Contact: Codable, Sendable, Identifiable, Hashable {
    public var id: String { agentId }
    public let agentId: String
    public let label: String?
    public let trustLevel: TrustLevel
    public let addedAt: UInt64?
    public let lastSeen: UInt64?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case label
        case trustLevel = "trust_level"
        case addedAt = "added_at"
        case lastSeen = "last_seen"
    }

    public init(agentId: String, label: String?, trustLevel: TrustLevel, addedAt: UInt64?, lastSeen: UInt64? = nil) {
        self.agentId = agentId
        self.label = label
        self.trustLevel = trustLevel
        self.addedAt = addedAt
        self.lastSeen = lastSeen
    }
}

/// Request body for adding a contact.
public struct AddContactRequest: Codable, Sendable {
    public let agentId: String
    public let trustLevel: TrustLevel
    public let label: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case trustLevel = "trust_level"
        case label
    }

    public init(agentId: String, trustLevel: TrustLevel, label: String?) {
        self.agentId = agentId
        self.trustLevel = trustLevel
        self.label = label
    }
}

/// A machine record associated with an agent.
public struct MachineRecord: Codable, Sendable, Identifiable {
    public var id: String { machineId }
    public let machineId: String
    public let agentId: String
    public let address: String?
    public let lastSeen: UInt64?

    enum CodingKeys: String, CodingKey {
        case machineId = "machine_id"
        case agentId = "agent_id"
        case address
        case lastSeen = "last_seen"
    }
}

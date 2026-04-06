import Foundation

/// Trust level assigned to a contact.
/// Matches the x0x daemon values: `blocked`, `unknown`, `known`, `trusted`.
public enum TrustLevel: String, Codable, Sendable, CaseIterable, Hashable {
    case blocked
    case unknown
    case known
    case trusted
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
    public let identityType: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case label
        case trustLevel = "trust_level"
        case addedAt = "added_at"
        case lastSeen = "last_seen"
        case identityType = "identity_type"
    }

    public init(agentId: String, label: String?, trustLevel: TrustLevel, addedAt: UInt64?, lastSeen: UInt64? = nil, identityType: String? = nil) {
        self.agentId = agentId
        self.label = label
        self.trustLevel = trustLevel
        self.addedAt = addedAt
        self.lastSeen = lastSeen
        self.identityType = identityType
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
/// Daemon returns: `{ machine_id, label?, first_seen, last_seen, pinned }`.
public struct MachineRecord: Codable, Sendable, Identifiable {
    public var id: String { machineId }
    public let machineId: String
    public let label: String?
    public let firstSeen: UInt64?
    public let lastSeen: UInt64?
    public let pinned: Bool?

    enum CodingKeys: String, CodingKey {
        case machineId = "machine_id"
        case label
        case firstSeen = "first_seen"
        case lastSeen = "last_seen"
        case pinned
    }
}

/// Wrapper for `GET /contacts/:agent_id/machines` response.
public struct MachineListResponse: Codable, Sendable {
    public let ok: Bool?
    public let machines: [MachineRecord]
}

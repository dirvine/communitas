import Foundation

// MARK: - MLS Groups

/// An MLS (Message Layer Security) encrypted group.
/// Returned by `POST /mls/groups`, `GET /mls/groups`, `GET /mls/groups/:id`.
public struct MlsGroup: Codable, Sendable, Identifiable {
    public var id: String { groupId }
    public let ok: Bool?
    public let groupId: String
    public let epoch: UInt64
    public let members: [String]?
    public let memberCount: Int?

    enum CodingKeys: String, CodingKey {
        case ok
        case groupId = "group_id"
        case epoch, members
        case memberCount = "member_count"
    }
}

/// Wrapper for `GET /mls/groups` response.
public struct MlsGroupList: Codable, Sendable {
    public let ok: Bool?
    public let groups: [MlsGroup]
}

/// Request body for `POST /mls/groups`.
public struct CreateMlsGroupRequest: Codable, Sendable {
    public let groupId: String?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
    }

    public init(groupId: String?) {
        self.groupId = groupId
    }
}

/// Request body for `POST /mls/groups/:id/members`.
public struct AddMlsMemberRequest: Codable, Sendable {
    public let agentId: String

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
    }

    public init(agentId: String) {
        self.agentId = agentId
    }
}

/// Response from `POST /mls/groups/:id/members`.
public struct AddMlsMemberResponse: Codable, Sendable {
    public let ok: Bool?
    public let epoch: UInt64
    public let members: [String]?
}

/// Request body for `POST /mls/groups/:id/welcome`.
public struct CreateWelcomeRequest: Codable, Sendable {
    public let agentId: String

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
    }

    public init(agentId: String) {
        self.agentId = agentId
    }
}

/// Response from `POST /mls/groups/:id/welcome`.
public struct WelcomeResponse: Codable, Sendable {
    public let ok: Bool?
    public let welcome: String
    public let groupId: String
    public let epoch: UInt64

    enum CodingKeys: String, CodingKey {
        case ok, welcome
        case groupId = "group_id"
        case epoch
    }
}

/// Request body for `POST /mls/groups/:id/encrypt`.
public struct EncryptRequest: Codable, Sendable {
    public let payload: String // base64

    public init(payload: String) {
        self.payload = payload
    }
}

/// Response from `POST /mls/groups/:id/encrypt`.
public struct EncryptResponse: Codable, Sendable {
    public let ok: Bool?
    public let ciphertext: String // base64
    public let epoch: UInt64
}

/// Request body for `POST /mls/groups/:id/decrypt`.
public struct DecryptRequest: Codable, Sendable {
    public let ciphertext: String // base64
    public let epoch: UInt64

    public init(ciphertext: String, epoch: UInt64) {
        self.ciphertext = ciphertext
        self.epoch = epoch
    }
}

/// Response from `POST /mls/groups/:id/decrypt`.
public struct DecryptResponse: Codable, Sendable {
    public let ok: Bool?
    public let payload: String // base64
}

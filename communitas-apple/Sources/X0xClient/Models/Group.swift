import Foundation

/// Request body for creating a group.
public struct CreateGroupRequest: Codable, Sendable {
    public let name: String
    public let description: String?
    public let displayName: String?

    enum CodingKeys: String, CodingKey {
        case name
        case description
        case displayName = "display_name"
    }

    public init(name: String, description: String?, displayName: String?) {
        self.name = name
        self.description = description
        self.displayName = displayName
    }
}

/// Response after creating a group.
public struct CreatedGroup: Codable, Sendable {
    public let groupId: String
    public let name: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name
    }
}

/// Summary of a group the agent is a member of.
/// ```json
/// {"group_id":"hex","name":"...","description":"","creator":"hex","created_at":1234,"member_count":1}
/// ```
public struct GroupSummary: Codable, Sendable, Identifiable, Hashable {
    public var id: String { groupId }
    public let groupId: String
    public let name: String
    public let description: String?
    public let creator: String?
    public let createdAt: UInt64?
    public let memberCount: UInt64?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name, description, creator
        case createdAt = "created_at"
        case memberCount = "member_count"
    }
}

/// Detailed group information.
public struct GroupInfo: Codable, Sendable {
    public let groupId: String
    public let name: String
    public let members: [String]?
    public let description: String?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name, members, description
    }
}

/// Request to generate an invite.
public struct InviteRequest: Codable, Sendable {
    public let groupId: String
    public let expirySecs: UInt64?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case expirySecs = "expiry_secs"
    }

    public init(groupId: String, expirySecs: UInt64?) {
        self.groupId = groupId
        self.expirySecs = expirySecs
    }
}

/// Response containing the invite token.
public struct InviteResponse: Codable, Sendable {
    public let invite: String
}

/// Request to join a group via invite token.
public struct JoinGroupRequest: Codable, Sendable {
    public let invite: String
    public let displayName: String?

    enum CodingKeys: String, CodingKey {
        case invite
        case displayName = "display_name"
    }

    public init(invite: String, displayName: String?) {
        self.invite = invite
        self.displayName = displayName
    }
}

/// Response after joining a group.
public struct JoinGroupResponse: Codable, Sendable {
    public let groupId: String
    public let name: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name
    }
}

/// An MLS group the agent is part of.
public struct MlsGroup: Codable, Sendable, Identifiable {
    public var id: String { groupId }
    public let groupId: String
    public let name: String?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name
    }
}

/// A named group with additional metadata.
public struct NamedGroup: Codable, Sendable, Identifiable {
    public var id: String { groupId }
    public let groupId: String
    public let name: String
    public let description: String?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name, description
    }
}

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
    public let chatTopic: String?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name
        case chatTopic = "chat_topic"
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

/// A member entry within detailed group information.
/// Daemon returns `{ "agent_id": "hex", "display_name": "Alice" }` per member.
public struct GroupMember: Codable, Sendable {
    public let agentId: String
    public let displayName: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case displayName = "display_name"
    }
}

/// Detailed group information.
public struct GroupInfo: Codable, Sendable {
    public let groupId: String
    public let name: String
    public let description: String?
    public let creator: String?
    public let createdAt: UInt64?
    public let memberCount: UInt64?
    public let chatTopic: String?
    public let metadataTopic: String?
    public let members: [GroupMember]?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name, description, creator, members
        case createdAt = "created_at"
        case memberCount = "member_count"
        case chatTopic = "chat_topic"
        case metadataTopic = "metadata_topic"
    }
}

/// Request to generate an invite.
public struct InviteRequest: Codable, Sendable {
    public let expirySecs: UInt64?

    enum CodingKeys: String, CodingKey {
        case expirySecs = "expiry_secs"
    }

    public init(expirySecs: UInt64?) {
        self.expirySecs = expirySecs
    }
}

/// Response containing the invite link.
/// Daemon returns `{ "ok": true, "invite_link": "...", "group_id": "...", "group_name": "...", "expires_at": N }`.
public struct InviteResponse: Codable, Sendable {
    public let ok: Bool?
    public let inviteLink: String
    public let groupId: String?
    public let groupName: String?
    public let expiresAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case ok
        case inviteLink = "invite_link"
        case groupId = "group_id"
        case groupName = "group_name"
        case expiresAt = "expires_at"
    }
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
/// Daemon returns `{ "ok": true, "group_id": "...", "group_name": "...", "chat_topic": "..." }`.
public struct JoinGroupResponse: Codable, Sendable {
    public let ok: Bool?
    public let groupId: String
    public let groupName: String
    public let chatTopic: String?

    enum CodingKeys: String, CodingKey {
        case ok
        case groupId = "group_id"
        case groupName = "group_name"
        case chatTopic = "chat_topic"
    }
}


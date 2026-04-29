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
    /// Full five-axis policy returned by `GET /groups/:id`. May be
    /// absent on legacy daemon responses; callers should treat
    /// `nil` as "assume MlsEncrypted defaults".
    public let policy: GroupPolicy?

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name, description, creator, members, policy
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

// MARK: - Extended named-groups surface
//
// Mirrors `x0x::groups::{policy, member, request, state_commit, directory,
// public_message, discovery}` verbatim. snake_case wire format. Types
// emitted by the daemon are `Decodable`; types posted to the daemon
// are `Encodable`. Structures that round-trip both ways are `Codable`.

/// Named preset bundling the five policy axes.
public enum GroupPolicyPreset: String, Codable, Sendable, CaseIterable {
    case privateSecure = "private_secure"
    case publicRequestSecure = "public_request_secure"
    case publicOpen = "public_open"
    case publicAnnounce = "public_announce"

    /// Concise label suitable for a preset tile.
    public var label: String {
        switch self {
        case .privateSecure: return "Private"
        case .publicRequestSecure: return "Public · request access"
        case .publicOpen: return "Public community"
        case .publicAnnounce: return "Announcement"
        }
    }

    /// One-line description shown beneath the label.
    public var summary: String {
        switch self {
        case .privateSecure:
            return "Hidden, invite-only, end-to-end encrypted."
        case .publicRequestSecure:
            return "Discoverable; content stays encrypted until admins approve."
        case .publicOpen:
            return "Open join, public read, members post."
        case .publicAnnounce:
            return "Open read; only admins may post."
        }
    }
}

/// Who can see the group on the discovery plane.
public enum GroupDiscoverability: String, Codable, Sendable, CaseIterable {
    case hidden
    case listedToContacts = "listed_to_contacts"
    case publicDirectory = "public_directory"
}

/// How new members are admitted.
public enum GroupAdmission: String, Codable, Sendable, CaseIterable {
    case inviteOnly = "invite_only"
    case requestAccess = "request_access"
    case openJoin = "open_join"
}

/// How content is protected.
public enum GroupConfidentiality: String, Codable, Sendable, CaseIterable {
    case mlsEncrypted = "mls_encrypted"
    case signedPublic = "signed_public"
}

/// Who can read the group's content.
public enum GroupReadAccess: String, Codable, Sendable, CaseIterable {
    case membersOnly = "members_only"
    case `public` = "public"
}

/// Who can write to the group.
public enum GroupWriteAccess: String, Codable, Sendable, CaseIterable {
    case membersOnly = "members_only"
    case moderatedPublic = "moderated_public"
    case adminOnly = "admin_only"
}

/// Five-axis group policy.
public struct GroupPolicy: Codable, Sendable, Equatable {
    public let discoverability: GroupDiscoverability
    public let admission: GroupAdmission
    public let confidentiality: GroupConfidentiality
    public let readAccess: GroupReadAccess
    public let writeAccess: GroupWriteAccess

    enum CodingKeys: String, CodingKey {
        case discoverability, admission, confidentiality
        case readAccess = "read_access"
        case writeAccess = "write_access"
    }

    public init(
        discoverability: GroupDiscoverability,
        admission: GroupAdmission,
        confidentiality: GroupConfidentiality,
        readAccess: GroupReadAccess,
        writeAccess: GroupWriteAccess
    ) {
        self.discoverability = discoverability
        self.admission = admission
        self.confidentiality = confidentiality
        self.readAccess = readAccess
        self.writeAccess = writeAccess
    }
}

/// Same shape as `GroupPolicy`; carried inside `GroupCard` for parity
/// with the x0xd wire type (see `x0x::groups::GroupPolicySummary`).
public struct GroupPolicySummary: Codable, Sendable, Equatable, Hashable {
    public let discoverability: GroupDiscoverability
    public let admission: GroupAdmission
    public let confidentiality: GroupConfidentiality
    public let readAccess: GroupReadAccess
    public let writeAccess: GroupWriteAccess

    enum CodingKeys: String, CodingKey {
        case discoverability, admission, confidentiality
        case readAccess = "read_access"
        case writeAccess = "write_access"
    }
}

/// Role of a member within a group.
public enum GroupRole: String, Codable, Sendable, CaseIterable {
    case owner, admin, moderator, member, guest
}

/// Lifecycle state of a member.
public enum GroupMemberState: String, Codable, Sendable, CaseIterable {
    case active, pending, removed, banned
}

/// Full roster entry. Distinct from the minimal ``GroupMember`` used
/// inside ``GroupInfo`` — prefer this one for `GET /groups/:id/members`.
///
/// `updated_at`, `removed_by`, and `kem_public_key_b64` are only emitted
/// by the daemon's all-members admin path (`/groups/:id/members?all=true`
/// in newer x0xd builds). They're optional in Swift so the public
/// roster endpoint, which omits them today, decodes cleanly.
public struct NamedGroupMember: Codable, Sendable, Identifiable, Hashable {
    public var id: String { agentId }
    public let agentId: String
    public let userId: String?
    public let role: GroupRole
    public let state: GroupMemberState
    public let displayName: String?
    public let joinedAt: UInt64
    public let updatedAt: UInt64?
    public let addedBy: String?
    public let removedBy: String?
    public let kemPublicKeyB64: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case userId = "user_id"
        case role, state
        case displayName = "display_name"
        case joinedAt = "joined_at"
        case updatedAt = "updated_at"
        case addedBy = "added_by"
        case removedBy = "removed_by"
        case kemPublicKeyB64 = "kem_public_key_b64"
    }
}

/// Response wrapper for `GET /groups/:id/members`.
public struct NamedGroupMembersResponse: Codable, Sendable {
    public let ok: Bool?
    public let groupId: String?
    public let memberCount: UInt64?
    public let members: [NamedGroupMember]

    enum CodingKeys: String, CodingKey {
        case ok
        case groupId = "group_id"
        case memberCount = "member_count"
        case members
    }
}

/// Status of a join request.
public enum JoinRequestStatus: String, Codable, Sendable {
    case pending, approved, rejected, cancelled
}

/// A join request submitted against a `RequestAccess` group.
public struct JoinRequest: Codable, Sendable, Identifiable, Hashable {
    public var id: String { requestId }
    public let requestId: String
    public let groupId: String
    public let requesterAgentId: String
    public let requesterUserId: String?
    public let requestedRole: GroupRole
    public let message: String?
    public let createdAt: UInt64
    public let reviewedAt: UInt64?
    public let reviewedBy: String?
    public let status: JoinRequestStatus

    enum CodingKeys: String, CodingKey {
        case requestId = "request_id"
        case groupId = "group_id"
        case requesterAgentId = "requester_agent_id"
        case requesterUserId = "requester_user_id"
        case requestedRole = "requested_role"
        case message
        case createdAt = "created_at"
        case reviewedAt = "reviewed_at"
        case reviewedBy = "reviewed_by"
        case status
    }
}

/// Response wrapper for `GET /groups/:id/requests`.
public struct JoinRequestListResponse: Codable, Sendable {
    public let ok: Bool?
    public let requests: [JoinRequest]
}

/// Request body for `POST /groups/:id/requests`.
public struct CreateJoinRequestBody: Codable, Sendable {
    public let message: String?

    public init(message: String?) { self.message = message }
}

/// Kind of a signed public-group message.
public enum GroupPublicMessageKind: String, Codable, Sendable {
    case chat, announcement
}

/// Signed message published to a SignedPublic group.
public struct GroupPublicMessage: Codable, Sendable, Identifiable, Hashable {
    public var id: String { "\(groupId)/\(timestamp)/\(signature.prefix(12))" }
    public let groupId: String
    public let stateHashAtSend: String
    public let revisionAtSend: UInt64
    public let authorAgentId: String
    public let authorPublicKey: String
    public let authorUserId: String?
    public let kind: GroupPublicMessageKind
    public let body: String
    public let timestamp: UInt64
    public let signature: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case stateHashAtSend = "state_hash_at_send"
        case revisionAtSend = "revision_at_send"
        case authorAgentId = "author_agent_id"
        case authorPublicKey = "author_public_key"
        case authorUserId = "author_user_id"
        case kind, body, timestamp, signature
    }
}

/// Request body for `POST /groups/:id/send`.
public struct SendGroupMessageRequest: Codable, Sendable {
    public let body: String
    public let kind: String?

    public init(body: String, kind: String?) {
        self.body = body
        self.kind = kind
    }
}

/// Response wrapper for `GET /groups/:id/messages`.
public struct GroupMessagesResponse: Codable, Sendable {
    public let ok: Bool?
    public let messages: [GroupPublicMessage]
}

/// Immutable genesis record.
public struct GroupGenesis: Codable, Sendable, Hashable {
    public let groupId: String
    public let creatorAgentId: String
    public let createdAt: UInt64
    public let creationNonce: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case creatorAgentId = "creator_agent_id"
        case createdAt = "created_at"
        case creationNonce = "creation_nonce"
    }
}

/// Signed state-commit (Phase D.3 chain link).
public struct GroupStateCommit: Codable, Sendable, Hashable {
    public let groupId: String
    public let revision: UInt64
    public let prevStateHash: String?
    public let rosterRoot: String
    public let policyHash: String
    public let publicMetaHash: String
    public let securityBinding: String?
    public let stateHash: String
    public let withdrawn: Bool
    public let committedBy: String
    public let committedAt: UInt64
    public let signerPublicKey: String
    public let signature: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case revision
        case prevStateHash = "prev_state_hash"
        case rosterRoot = "roster_root"
        case policyHash = "policy_hash"
        case publicMetaHash = "public_meta_hash"
        case securityBinding = "security_binding"
        case stateHash = "state_hash"
        case withdrawn
        case committedBy = "committed_by"
        case committedAt = "committed_at"
        case signerPublicKey = "signer_public_key"
        case signature
    }
}

/// Response from `GET /groups/:id/state`.
public struct GroupStateResponse: Codable, Sendable, Hashable {
    public let ok: Bool?
    public let groupId: String
    public let mlsGroupId: String?
    public let genesis: GroupGenesis?
    public let stateRevision: UInt64
    public let stateHash: String
    public let prevStateHash: String?
    public let securityBinding: String?
    public let withdrawn: Bool
    public let rosterRoot: String
    public let policyHash: String
    public let publicMetaHash: String

    enum CodingKeys: String, CodingKey {
        case ok
        case groupId = "group_id"
        case mlsGroupId = "mls_group_id"
        case genesis
        case stateRevision = "state_revision"
        case stateHash = "state_hash"
        case prevStateHash = "prev_state_hash"
        case securityBinding = "security_binding"
        case withdrawn
        case rosterRoot = "roster_root"
        case policyHash = "policy_hash"
        case publicMetaHash = "public_meta_hash"
    }
}

/// Signed discoverable card for a group.
public struct GroupCard: Codable, Sendable, Identifiable, Hashable {
    public var id: String { groupId }
    public let groupId: String
    public let name: String
    public let description: String
    public let avatarUrl: String?
    public let bannerUrl: String?
    public let tags: [String]
    public let policySummary: GroupPolicySummary
    public let ownerAgentId: String
    public let adminCount: UInt32
    public let memberCount: UInt32
    public let createdAt: UInt64
    public let updatedAt: UInt64
    public let requestAccessEnabled: Bool
    public let metadataTopic: String?
    public let revision: UInt64
    public let stateHash: String
    public let prevStateHash: String?
    public let issuedAt: UInt64
    public let expiresAt: UInt64
    public let authorityAgentId: String
    public let authorityPublicKey: String
    public let withdrawn: Bool
    public let signature: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case name, description
        case avatarUrl = "avatar_url"
        case bannerUrl = "banner_url"
        case tags
        case policySummary = "policy_summary"
        case ownerAgentId = "owner_agent_id"
        case adminCount = "admin_count"
        case memberCount = "member_count"
        case createdAt = "created_at"
        case updatedAt = "updated_at"
        case requestAccessEnabled = "request_access_enabled"
        case metadataTopic = "metadata_topic"
        case revision
        case stateHash = "state_hash"
        case prevStateHash = "prev_state_hash"
        case issuedAt = "issued_at"
        case expiresAt = "expires_at"
        case authorityAgentId = "authority_agent_id"
        case authorityPublicKey = "authority_public_key"
        case withdrawn, signature
    }
}

/// Response wrapper for `GET /groups/discover` and `.../nearby`.
public struct GroupCardListResponse: Codable, Sendable {
    public let ok: Bool?
    public let groups: [GroupCard]
}

/// Which dimension a directory shard indexes.
public enum ShardKind: String, Codable, Sendable {
    case tag, name, id
}

/// A single active directory-shard subscription.
public struct SubscriptionRecord: Codable, Sendable, Hashable, Identifiable {
    public var id: String { "\(kind.rawValue)/\(shard)" }
    public let kind: ShardKind
    public let shard: UInt32
    public let key: String?
    public let subscribedAt: UInt64

    enum CodingKeys: String, CodingKey {
        case kind, shard, key
        case subscribedAt = "subscribed_at"
    }
}

/// Response wrapper for `GET /groups/discover/subscriptions`.
public struct ShardSubscriptionsResponse: Codable, Sendable {
    public let ok: Bool?
    public let count: Int
    public let subscriptions: [SubscriptionRecord]
}

/// Request body for `POST /groups/discover/subscribe`.
public struct SubscribeShardRequest: Codable, Sendable {
    public let kind: String
    public let key: String?
    public let shard: UInt32?

    public init(kind: String, key: String?, shard: UInt32?) {
        self.kind = kind
        self.key = key
        self.shard = shard
    }
}

/// Response from `POST /groups/discover/subscribe`.
public struct SubscribeShardResponse: Codable, Sendable {
    public let ok: Bool?
    public let newlyAdded: Bool
    public let kind: ShardKind
    public let shard: UInt32
    public let topic: String

    enum CodingKeys: String, CodingKey {
        case ok
        case newlyAdded = "newly_added"
        case kind, shard, topic
    }
}

/// Request body for `PATCH /groups/:id`.
public struct UpdateNamedGroupRequest: Codable, Sendable {
    public let name: String?
    public let description: String?

    public init(name: String? = nil, description: String? = nil) {
        self.name = name
        self.description = description
    }
}

/// Request body for `PATCH /groups/:id/policy`. Set `preset` alone or
/// any subset of axes; the daemon applies preset first then overlays
/// explicit axes. `nil` fields are omitted from the wire payload.
public struct UpdateGroupPolicyRequest: Codable, Sendable {
    public let preset: String?
    public let discoverability: GroupDiscoverability?
    public let admission: GroupAdmission?
    public let confidentiality: GroupConfidentiality?
    public let readAccess: GroupReadAccess?
    public let writeAccess: GroupWriteAccess?

    enum CodingKeys: String, CodingKey {
        case preset, discoverability, admission, confidentiality
        case readAccess = "read_access"
        case writeAccess = "write_access"
    }

    public init(
        preset: String? = nil,
        discoverability: GroupDiscoverability? = nil,
        admission: GroupAdmission? = nil,
        confidentiality: GroupConfidentiality? = nil,
        readAccess: GroupReadAccess? = nil,
        writeAccess: GroupWriteAccess? = nil
    ) {
        self.preset = preset
        self.discoverability = discoverability
        self.admission = admission
        self.confidentiality = confidentiality
        self.readAccess = readAccess
        self.writeAccess = writeAccess
    }
}

/// Request body for `PATCH /groups/:id/members/:agent_id/role`.
public struct UpdateMemberRoleRequest: Codable, Sendable {
    public let role: GroupRole

    public init(role: GroupRole) { self.role = role }
}

/// Request body for `POST /groups/:id/members` (add-member).
public struct AddNamedGroupMemberRequest: Codable, Sendable {
    public let agentId: String
    public let displayName: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case displayName = "display_name"
    }

    public init(agentId: String, displayName: String? = nil) {
        self.agentId = agentId
        self.displayName = displayName
    }
}

/// Request body for `POST /groups` with an optional policy preset.
public struct CreateNamedGroupRequest: Codable, Sendable {
    public let name: String
    public let description: String?
    public let displayName: String?
    public let preset: String?

    enum CodingKeys: String, CodingKey {
        case name, description, preset
        case displayName = "display_name"
    }

    public init(name: String, description: String?, displayName: String?, preset: String?) {
        self.name = name
        self.description = description
        self.displayName = displayName
        self.preset = preset
    }
}

/// Request body for `POST /groups/:id/secure/encrypt`.
public struct SecureEncryptRequest: Codable, Sendable {
    public let payloadB64: String

    enum CodingKeys: String, CodingKey {
        case payloadB64 = "payload_b64"
    }

    public init(payloadB64: String) { self.payloadB64 = payloadB64 }
}

/// Response from `POST /groups/:id/secure/encrypt`.
public struct SecureEncryptResponse: Codable, Sendable {
    public let ok: Bool?
    public let ciphertextB64: String
    public let nonceB64: String
    public let secretEpoch: UInt64

    enum CodingKeys: String, CodingKey {
        case ok
        case ciphertextB64 = "ciphertext_b64"
        case nonceB64 = "nonce_b64"
        case secretEpoch = "secret_epoch"
    }
}

/// Request body for `POST /groups/:id/secure/decrypt`.
public struct SecureDecryptRequest: Codable, Sendable {
    public let ciphertextB64: String
    public let nonceB64: String
    public let secretEpoch: UInt64

    enum CodingKeys: String, CodingKey {
        case ciphertextB64 = "ciphertext_b64"
        case nonceB64 = "nonce_b64"
        case secretEpoch = "secret_epoch"
    }

    public init(ciphertextB64: String, nonceB64: String, secretEpoch: UInt64) {
        self.ciphertextB64 = ciphertextB64
        self.nonceB64 = nonceB64
        self.secretEpoch = secretEpoch
    }
}

/// Response from `POST /groups/:id/secure/decrypt`.
public struct SecureDecryptResponse: Codable, Sendable {
    public let ok: Bool?
    public let plaintextB64: String

    enum CodingKeys: String, CodingKey {
        case ok
        case plaintextB64 = "plaintext_b64"
    }
}

/// Request body for `POST /groups/:id/secure/reseal`.
public struct SecureResealRequest: Codable, Sendable {
    public let recipient: String

    public init(recipient: String) { self.recipient = recipient }
}

/// Envelope produced by `POST /groups/:id/secure/reseal`. Also accepted
/// by `POST /groups/secure/open-envelope`.
public struct SecureShareEnvelope: Codable, Sendable, Hashable {
    public let groupId: String
    public let recipient: String
    public let secretEpoch: UInt64
    public let kemCiphertextB64: String
    public let aeadNonceB64: String
    public let aeadCiphertextB64: String

    enum CodingKeys: String, CodingKey {
        case groupId = "group_id"
        case recipient
        case secretEpoch = "secret_epoch"
        case kemCiphertextB64 = "kem_ciphertext_b64"
        case aeadNonceB64 = "aead_nonce_b64"
        case aeadCiphertextB64 = "aead_ciphertext_b64"
    }

    public init(
        groupId: String,
        recipient: String,
        secretEpoch: UInt64,
        kemCiphertextB64: String,
        aeadNonceB64: String,
        aeadCiphertextB64: String
    ) {
        self.groupId = groupId
        self.recipient = recipient
        self.secretEpoch = secretEpoch
        self.kemCiphertextB64 = kemCiphertextB64
        self.aeadNonceB64 = aeadNonceB64
        self.aeadCiphertextB64 = aeadCiphertextB64
    }
}


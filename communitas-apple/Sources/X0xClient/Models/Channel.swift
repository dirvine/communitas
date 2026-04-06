import Foundation

/// The type of a gossip chat payload.
public enum ChatPayloadType: String, Codable, Sendable {
    case message
    case edit
    case delete
}

/// A chat message transmitted over gossip topics for channel/thread communication.
public struct ChannelChatMessage: Codable, Identifiable, Sendable, Equatable {
    public let id: String
    public var text: String
    public var senderName: String
    public var senderId: String
    public var timestamp: Int64
    public var channel: String
    public var threadRoot: String?
    public var broadcast: Bool
    public var replyCount: Int
    /// For edit events: the original message ID being edited, and the new text (stored in `text`).
    public var messageId: String?
    /// For edit/delete events: the event type. Defaults to `.message` for regular chat messages.
    public var type: ChatPayloadType
    /// If the message has been edited, the Unix-ms timestamp of the edit.
    public var editedAt: Int64?
    /// Whether the message has been deleted.
    public var isDeleted: Bool
    /// Emoji reactions keyed by emoji string, value is the total count.
    /// Serialized and persisted alongside the message for offline-first support.
    public var reactions: [String: Int]
    /// The ID of the message this message is replying to (inline quote / reply preview).
    public var replyToId: String?

    enum CodingKeys: String, CodingKey {
        case id, text, channel, broadcast, timestamp, type, reactions
        case msgType = "msg_type"
        case senderName = "sender_name"
        case senderId = "sender_id"
        case threadRoot = "thread_root"
        case replyCount = "reply_count"
        case messageId = "message_id"
        case msgId = "msg_id"
        case editedAt = "edited_at"
        case isDeleted = "is_deleted"
        case replyToId = "reply_to_id"
        case quoteId = "quote_id"
    }

    public init(
        id: String,
        text: String,
        senderName: String,
        senderId: String,
        timestamp: Int64,
        channel: String,
        threadRoot: String? = nil,
        broadcast: Bool = false,
        replyCount: Int = 0,
        messageId: String? = nil,
        type: ChatPayloadType = .message,
        editedAt: Int64? = nil,
        isDeleted: Bool = false,
        reactions: [String: Int] = [:],
        replyToId: String? = nil
    ) {
        self.id = id
        self.text = text
        self.senderName = senderName
        self.senderId = senderId
        self.timestamp = timestamp
        self.channel = channel
        self.threadRoot = threadRoot
        self.broadcast = broadcast
        self.replyCount = replyCount
        self.messageId = messageId
        self.type = type
        self.editedAt = editedAt
        self.isDeleted = isDeleted
        self.reactions = reactions
        self.replyToId = replyToId
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeIfPresent(String.self, forKey: .id) ?? ""
        text = try container.decodeIfPresent(String.self, forKey: .text) ?? ""
        senderName = try container.decodeIfPresent(String.self, forKey: .senderName) ?? ""
        senderId = try container.decodeIfPresent(String.self, forKey: .senderId) ?? ""
        timestamp = try container.decodeIfPresent(Int64.self, forKey: .timestamp) ?? 0
        channel = try container.decodeIfPresent(String.self, forKey: .channel) ?? ""
        threadRoot = try container.decodeIfPresent(String.self, forKey: .threadRoot)
        broadcast = try container.decodeIfPresent(Bool.self, forKey: .broadcast) ?? false
        replyCount = try container.decodeIfPresent(Int.self, forKey: .replyCount) ?? 0
        messageId = try container.decodeIfPresent(String.self, forKey: .msgId)
            ?? container.decodeIfPresent(String.self, forKey: .messageId)
        let rawType = try container.decodeIfPresent(String.self, forKey: .type)
            ?? container.decodeIfPresent(String.self, forKey: .msgType)
            ?? ChatPayloadType.message.rawValue
        type = ChatPayloadType(rawValue: rawType) ?? .message
        editedAt = try container.decodeIfPresent(Int64.self, forKey: .editedAt)
        isDeleted = try container.decodeIfPresent(Bool.self, forKey: .isDeleted) ?? false
        reactions = try container.decodeIfPresent([String: Int].self, forKey: .reactions) ?? [:]
        replyToId = try container.decodeIfPresent(String.self, forKey: .replyToId)
            ?? container.decodeIfPresent(String.self, forKey: .quoteId)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(text, forKey: .text)
        try container.encode(senderName, forKey: .senderName)
        try container.encode(senderId, forKey: .senderId)
        try container.encode(timestamp, forKey: .timestamp)
        try container.encode(channel, forKey: .channel)
        if let threadRoot, !threadRoot.isEmpty {
            try container.encode(threadRoot, forKey: .threadRoot)
        }
        if broadcast {
            try container.encode(true, forKey: .broadcast)
        }
        if replyCount > 0 {
            try container.encode(replyCount, forKey: .replyCount)
        }
        if let messageId, !messageId.isEmpty {
            try container.encode(messageId, forKey: .msgId)
        }
        if type != .message {
            try container.encode(type.rawValue, forKey: .type)
        }
        if let editedAt {
            try container.encode(editedAt, forKey: .editedAt)
        }
        if isDeleted {
            try container.encode(true, forKey: .isDeleted)
        }
        if !reactions.isEmpty {
            try container.encode(reactions, forKey: .reactions)
        }
        if let replyToId, !replyToId.isEmpty {
            try container.encode(replyToId, forKey: .replyToId)
        }
    }

    /// The message timestamp as a `Date`.
    public var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
    }

    /// The edited-at timestamp as a `Date`, if present.
    public var editedDate: Date? {
        guard let editedAt else { return nil }
        return Date(timeIntervalSince1970: TimeInterval(editedAt) / 1000.0)
    }
}

/// The action component of a reaction event.
public enum ReactionAction: String, Codable, Sendable {
    case add
    case remove
}

/// A gossip payload for emoji reactions. Published on the same channel topic as chat messages.
/// JSON layout matches the x0x GUI conventions.
public struct ReactionEvent: Codable, Sendable {
    public let id: String
    /// Always `"reaction"` — used to distinguish from `ChannelChatMessage` payloads.
    public let type: String
    public let messageId: String
    public let emoji: String
    public let action: ReactionAction
    public let senderId: String
    public let senderName: String
    public let timestamp: Int64

    enum CodingKeys: String, CodingKey {
        case id, type, emoji, action, timestamp
        case messageId = "msg_id"
        case legacyMessageId = "messageId"
        case senderId = "sender_id"
        case legacySenderId = "senderId"
        case senderName = "sender_name"
        case legacySenderName = "senderName"
    }

    public init(
        id: String = UUID().uuidString,
        messageId: String,
        emoji: String,
        action: ReactionAction,
        senderId: String,
        senderName: String,
        timestamp: Int64 = Int64(Date().timeIntervalSince1970 * 1000)
    ) {
        self.id = id
        self.type = "reaction"
        self.messageId = messageId
        self.emoji = emoji
        self.action = action
        self.senderId = senderId
        self.senderName = senderName
        self.timestamp = timestamp
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeIfPresent(String.self, forKey: .id) ?? UUID().uuidString
        type = try container.decodeIfPresent(String.self, forKey: .type) ?? "reaction"
        messageId = try container.decodeIfPresent(String.self, forKey: .messageId)
            ?? container.decode(String.self, forKey: .legacyMessageId)
        emoji = try container.decode(String.self, forKey: .emoji)
        action = try container.decode(ReactionAction.self, forKey: .action)
        senderId = try container.decodeIfPresent(String.self, forKey: .senderId)
            ?? container.decode(String.self, forKey: .legacySenderId)
        senderName = try container.decodeIfPresent(String.self, forKey: .senderName)
            ?? container.decodeIfPresent(String.self, forKey: .legacySenderName)
            ?? ""
        timestamp = try container.decodeIfPresent(Int64.self, forKey: .timestamp)
            ?? Int64(Date().timeIntervalSince1970 * 1000)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(type, forKey: .type)
        try container.encode(messageId, forKey: .messageId)
        try container.encode(emoji, forKey: .emoji)
        try container.encode(action, forKey: .action)
        try container.encode(senderId, forKey: .senderId)
        try container.encode(senderName, forKey: .senderName)
        try container.encode(timestamp, forKey: .timestamp)
    }
}

/// Canonical channel metadata stored in `channels_index`.
/// Matches the Dioxus `ChannelMeta` schema exactly.
public struct ChannelMeta: Codable, Identifiable, Sendable, Equatable {
    public var id: String { name }
    public var name: String
    public var description: String
    public var creator: String
    public var createdAt: UInt64
    public var topic: String

    enum CodingKeys: String, CodingKey {
        case name, description, creator, topic
        case createdAt = "created_at"
    }

    public init(
        name: String,
        description: String,
        creator: String,
        createdAt: UInt64,
        topic: String
    ) {
        self.name = name
        self.description = description
        self.creator = creator
        self.createdAt = createdAt
        self.topic = topic
    }
}

/// The `channels_index` key stores a JSON array of `ChannelMeta` (matching Dioxus).
/// This typealias makes intent clear at call sites.
public typealias ChannelIndex = [ChannelMeta]

/// An ephemeral typing event published on the channel topic to show who is typing.
/// Payload format matches the x0x GUI conventions.
public struct TypingEvent: Codable, Sendable {
    public let id: String
    /// Always `"typing"` — used to distinguish from other payload types.
    public let type: String
    public let senderId: String
    public let senderName: String
    public let timestamp: Int64

    enum CodingKeys: String, CodingKey {
        case id, type, timestamp
        case senderId = "sender_id"
        case legacySenderId = "senderId"
        case senderName = "sender_name"
        case legacySenderName = "senderName"
    }

    public init(id: String = UUID().uuidString, senderId: String, senderName: String,
                timestamp: Int64 = Int64(Date().timeIntervalSince1970 * 1000)) {
        self.id = id
        self.type = "typing"
        self.senderId = senderId
        self.senderName = senderName
        self.timestamp = timestamp
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        id = try container.decodeIfPresent(String.self, forKey: .id) ?? UUID().uuidString
        type = try container.decodeIfPresent(String.self, forKey: .type) ?? "typing"
        senderId = try container.decodeIfPresent(String.self, forKey: .senderId)
            ?? container.decode(String.self, forKey: .legacySenderId)
        senderName = try container.decodeIfPresent(String.self, forKey: .senderName)
            ?? container.decodeIfPresent(String.self, forKey: .legacySenderName)
            ?? ""
        timestamp = try container.decodeIfPresent(Int64.self, forKey: .timestamp)
            ?? Int64(Date().timeIntervalSince1970 * 1000)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(type, forKey: .type)
        try container.encode(senderId, forKey: .senderId)
        try container.encode(senderName, forKey: .senderName)
        try container.encode(timestamp, forKey: .timestamp)
    }
}

/// A pin/unpin event for channel messages, matching the x0x GUI convention.
public struct PinEvent: Codable, Sendable {
    public let type: String
    public let messageId: String
    public let action: String
    public let senderId: String
    public let timestamp: Int64

    enum CodingKeys: String, CodingKey {
        case type, action, timestamp
        case messageId = "msg_id"
        case legacyMessageId = "messageId"
        case senderId = "sender_id"
        case legacySenderId = "senderId"
    }

    public init(messageId: String, action: String, senderId: String,
                timestamp: Int64 = Int64(Date().timeIntervalSince1970 * 1000)) {
        self.type = "pin"
        self.messageId = messageId
        self.action = action
        self.senderId = senderId
        self.timestamp = timestamp
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        type = try container.decodeIfPresent(String.self, forKey: .type) ?? "pin"
        messageId = try container.decodeIfPresent(String.self, forKey: .messageId)
            ?? container.decode(String.self, forKey: .legacyMessageId)
        action = try container.decode(String.self, forKey: .action)
        senderId = try container.decodeIfPresent(String.self, forKey: .senderId)
            ?? container.decodeIfPresent(String.self, forKey: .legacySenderId)
            ?? ""
        timestamp = try container.decodeIfPresent(Int64.self, forKey: .timestamp)
            ?? Int64(Date().timeIntervalSince1970 * 1000)
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(type, forKey: .type)
        try container.encode(messageId, forKey: .messageId)
        try container.encode(action, forKey: .action)
        try container.encode(senderId, forKey: .senderId)
        try container.encode(timestamp, forKey: .timestamp)
    }
}

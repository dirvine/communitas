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
        case senderName = "sender_name"
        case senderId = "sender_id"
        case threadRoot = "thread_root"
        case replyCount = "reply_count"
        case messageId = "message_id"
        case editedAt = "edited_at"
        case isDeleted = "is_deleted"
        case replyToId = "reply_to_id"
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
/// JSON layout matches the Dioxus interop contract.
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
        case messageId = "messageId"
        case senderId = "senderId"
        case senderName = "senderName"
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

/// Legacy pre-freeze schema kept for one-time compatibility migration.
/// The old format stored `{"channels": ["general"], "categories": {"General": ["general"]}}`.
public struct LegacyChannelIndex: Codable, Sendable {
    public var channels: [String]
    public var categories: [String: [String]]

    public init(channels: [String] = [], categories: [String: [String]] = [:]) {
        self.channels = channels
        self.categories = categories
    }
}

/// An ephemeral typing event published on the channel topic to show who is typing.
/// Payload format matches the typing indicator spec.
public struct TypingEvent: Codable, Sendable {
    public let id: String
    /// Always `"typing"` — used to distinguish from other payload types.
    public let type: String
    public let senderId: String
    public let senderName: String
    public let timestamp: Int64

    enum CodingKeys: String, CodingKey {
        case id, type, timestamp
        case senderId = "senderId"
        case senderName = "senderName"
    }

    public init(id: String = UUID().uuidString, senderId: String, senderName: String,
                timestamp: Int64 = Int64(Date().timeIntervalSince1970 * 1000)) {
        self.id = id
        self.type = "typing"
        self.senderId = senderId
        self.senderName = senderName
        self.timestamp = timestamp
    }
}

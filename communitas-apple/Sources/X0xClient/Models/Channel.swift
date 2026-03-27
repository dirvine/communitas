import Foundation

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

    enum CodingKeys: String, CodingKey {
        case id, text, channel, broadcast, timestamp
        case senderName = "sender_name"
        case senderId = "sender_id"
        case threadRoot = "thread_root"
        case replyCount = "reply_count"
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
        replyCount: Int = 0
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
    }

    /// The message timestamp as a `Date`.
    public var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
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

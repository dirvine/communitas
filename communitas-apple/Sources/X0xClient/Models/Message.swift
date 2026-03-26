import Foundation

/// A chat message for display in the UI.
public struct ChatMessage: Identifiable, Sendable {
    public let id: String
    public let sender: String
    public let content: String
    public let timestamp: Date
    public let isOutgoing: Bool

    public init(id: String, sender: String, content: String, timestamp: Date, isOutgoing: Bool) {
        self.id = id
        self.sender = sender
        self.content = content
        self.timestamp = timestamp
        self.isOutgoing = isOutgoing
    }

    /// Create from a base64-encoded gossip message payload.
    public static func fromGossip(_ msg: GossipMessage, myAgentId: String) -> ChatMessage? {
        guard let data = Data(base64Encoded: msg.payload),
              let text = String(data: data, encoding: .utf8) else {
            return nil
        }
        let date = Date(timeIntervalSince1970: TimeInterval(msg.timestamp))
        return ChatMessage(
            id: msg.messageId,
            sender: msg.sender,
            content: text,
            timestamp: date,
            isOutgoing: msg.sender == myAgentId
        )
    }

    /// Create from a base64-encoded direct message payload.
    public static func fromDirect(_ msg: DirectMessage, myAgentId: String) -> ChatMessage? {
        guard let data = Data(base64Encoded: msg.payload),
              let text = String(data: data, encoding: .utf8) else {
            return nil
        }
        let date = Date(timeIntervalSince1970: TimeInterval(msg.timestamp))
        return ChatMessage(
            id: msg.messageId,
            sender: msg.sender,
            content: text,
            timestamp: date,
            isOutgoing: msg.sender == myAgentId
        )
    }
}

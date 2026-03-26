import Foundation

/// Direction of a file transfer.
public enum TransferDirection: String, Codable, Sendable {
    case upload
    case download
}

/// Current status of a file transfer.
public enum TransferStatus: String, Codable, Sendable {
    case pending
    case inProgress = "in_progress"
    case completed
    case failed
    case cancelled
}

/// A file transfer record.
public struct FileTransfer: Codable, Sendable, Identifiable {
    public var id: String { transferId }
    public let transferId: String
    public let filename: String
    public let size: UInt64
    public let direction: TransferDirection
    public let status: TransferStatus
    public let peerAgentId: String
    public let progress: Double?

    enum CodingKeys: String, CodingKey {
        case transferId = "transfer_id"
        case filename, size, direction, status
        case peerAgentId = "peer_agent_id"
        case progress
    }
}

/// Request to initiate a file send.
public struct SendFileRequest: Codable, Sendable {
    public let agentId: String
    public let filename: String
    public let size: UInt64

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case filename, size
    }

    public init(agentId: String, filename: String, size: UInt64) {
        self.agentId = agentId
        self.filename = filename
        self.size = size
    }
}

/// Response after initiating a file send.
public struct SendFileResponse: Codable, Sendable {
    public let transferId: String

    enum CodingKeys: String, CodingKey {
        case transferId = "transfer_id"
    }
}

import Foundation

/// Direction of a file transfer.
/// Daemon values: `"Sending"` / `"Receiving"`.
public enum TransferDirection: String, Codable, Sendable {
    case sending = "Sending"
    case receiving = "Receiving"
}

/// Current status of a file transfer.
/// Daemon values: `"Pending"`, `"InProgress"`, `"Complete"`, `"Failed"`, `"Rejected"`.
public enum TransferStatus: String, Codable, Sendable {
    case pending = "Pending"
    case inProgress = "InProgress"
    case complete = "Complete"
    case failed = "Failed"
    case rejected = "Rejected"
}

/// A file transfer record returned by `GET /files/transfers`.
/// Daemon serialises the `TransferState` struct directly.
public struct FileTransfer: Codable, Sendable, Identifiable {
    public var id: String { transferId }
    public let transferId: String
    public let direction: TransferDirection
    public let remoteAgentId: String
    public let filename: String
    public let totalSize: UInt64
    public let bytesTransferred: UInt64
    public let status: TransferStatus
    public let sha256: String?
    public let error: String?
    public let startedAt: UInt64?

    public init(transferId: String, direction: TransferDirection, remoteAgentId: String, filename: String, totalSize: UInt64, bytesTransferred: UInt64, status: TransferStatus, sha256: String?, error: String?, startedAt: UInt64?) {
        self.transferId = transferId
        self.direction = direction
        self.remoteAgentId = remoteAgentId
        self.filename = filename
        self.totalSize = totalSize
        self.bytesTransferred = bytesTransferred
        self.status = status
        self.sha256 = sha256
        self.error = error
        self.startedAt = startedAt
    }

    /// Computed transfer progress (0.0–1.0).
    public var progress: Double {
        guard totalSize > 0 else { return 0 }
        return Double(bytesTransferred) / Double(totalSize)
    }

    enum CodingKeys: String, CodingKey {
        case transferId = "transfer_id"
        case direction
        case remoteAgentId = "remote_agent_id"
        case filename
        case totalSize = "total_size"
        case bytesTransferred = "bytes_transferred"
        case status
        case sha256
        case error
        case startedAt = "started_at"
    }
}

/// Wrapper for `GET /files/transfers` response: `{"ok":true,"transfers":[...]}`.
public struct FileTransferListResponse: Codable, Sendable {
    public let ok: Bool?
    public let transfers: [FileTransfer]
}

/// Request to initiate a file send.
/// Daemon requires `agent_id`, `filename`, `size`, and `sha256`.
public struct SendFileRequest: Codable, Sendable {
    public let agentId: String
    public let filename: String
    public let size: UInt64
    public let sha256: String
    public let path: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case filename, size, sha256, path
    }

    public init(agentId: String, filename: String, size: UInt64, sha256: String, path: String? = nil) {
        self.agentId = agentId
        self.filename = filename
        self.size = size
        self.sha256 = sha256
        self.path = path
    }
}

/// Response after initiating a file send.
public struct SendFileResponse: Codable, Sendable {
    public let ok: Bool?
    public let transferId: String

    enum CodingKeys: String, CodingKey {
        case ok
        case transferId = "transfer_id"
    }
}

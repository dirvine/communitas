import Foundation

// MARK: - Generic JSON

/// Lossless JSON value used for x0xd diagnostics snapshots whose schema evolves
/// faster than the app UI needs to interpret every counter.
public enum JSONValue: Codable, Sendable, Equatable {
    case string(String)
    case number(Double)
    case bool(Bool)
    case object([String: JSONValue])
    case array([JSONValue])
    case null

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Double.self) {
            self = .number(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? container.decode([JSONValue].self) {
            self = .array(value)
        } else {
            self = .object(try container.decode([String: JSONValue].self))
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        switch self {
        case let .string(value): try container.encode(value)
        case let .number(value): try container.encode(value)
        case let .bool(value): try container.encode(value)
        case let .object(value): try container.encode(value)
        case let .array(value): try container.encode(value)
        case .null: try container.encodeNil()
        }
    }
}

public typealias DiagnosticsSnapshot = JSONValue
public typealias ExecSessionsSnapshot = JSONValue

// MARK: - Agent signing

public struct AgentSignRequest: Codable, Sendable {
    public let payloadB64: String

    enum CodingKeys: String, CodingKey {
        case payloadB64 = "payload_b64"
    }

    public init(payloadB64: String) {
        self.payloadB64 = payloadB64
    }
}

public struct AgentSignResponse: Codable, Sendable {
    public let ok: Bool?
    public let agentId: String
    public let publicKeyB64: String
    public let signatureB64: String
    public let algorithm: String

    enum CodingKeys: String, CodingKey {
        case ok, algorithm
        case agentId = "agent_id"
        case publicKeyB64 = "public_key_b64"
        case signatureB64 = "signature_b64"
    }
}

// MARK: - Remote exec

public struct ExecRunRequest: Codable, Sendable {
    public let agentId: String
    public let argv: [String]
    public let stdinB64: String?
    public let timeoutMs: UInt32?
    public let cwd: String?

    enum CodingKeys: String, CodingKey {
        case agentId = "agent_id"
        case argv
        case stdinB64 = "stdin_b64"
        case timeoutMs = "timeout_ms"
        case cwd
    }

    public init(agentId: String, argv: [String], stdinB64: String? = nil, timeoutMs: UInt32? = nil, cwd: String? = nil) {
        self.agentId = agentId
        self.argv = argv
        self.stdinB64 = stdinB64
        self.timeoutMs = timeoutMs
        self.cwd = cwd
    }
}

public struct ExecRunResponse: Codable, Sendable {
    public let ok: Bool?
    public let requestId: String
    public let code: Int32?
    public let signal: Int32?
    public let durationMs: UInt64
    public let stdoutB64: String
    public let stderrB64: String
    public let stdoutBytesTotal: UInt64
    public let stderrBytesTotal: UInt64
    public let truncated: Bool
    public let denialReason: String?
    public let warnings: [String]

    enum CodingKeys: String, CodingKey {
        case ok, code, signal, truncated, warnings
        case requestId = "request_id"
        case durationMs = "duration_ms"
        case stdoutB64 = "stdout_b64"
        case stderrB64 = "stderr_b64"
        case stdoutBytesTotal = "stdout_bytes_total"
        case stderrBytesTotal = "stderr_bytes_total"
        case denialReason = "denial_reason"
    }
}

public struct ExecCancelRequest: Codable, Sendable {
    public let requestId: String
    public let agentId: String?

    enum CodingKeys: String, CodingKey {
        case requestId = "request_id"
        case agentId = "agent_id"
    }

    public init(requestId: String, agentId: String? = nil) {
        self.requestId = requestId
        self.agentId = agentId
    }
}

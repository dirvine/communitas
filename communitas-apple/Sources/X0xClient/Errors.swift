import Foundation

/// Errors from the x0x daemon client.
public enum X0xError: Error, LocalizedError {
    /// The daemon is not reachable at the configured URL.
    case daemonUnreachable

    /// The daemon returned a non-success HTTP status.
    case httpError(statusCode: Int, body: String)

    /// The API responded with `ok: false`.
    case apiError(message: String)

    /// Failed to decode the response body.
    case decodingError(underlying: Error)

    /// Failed to encode the request body.
    case encodingError(underlying: Error)

    /// The daemon binary was not found at the expected path.
    case daemonNotInstalled

    /// The daemon failed to start.
    case daemonStartFailed(reason: String)

    /// A WebSocket connection error.
    case webSocketError(reason: String)

    /// An invalid URL was constructed.
    case invalidURL(path: String)

    /// A generic / unexpected error.
    case unexpected(message: String)

    public var errorDescription: String? {
        switch self {
        case .daemonUnreachable:
            return "Cannot reach x0xd at the configured address."
        case .httpError(let code, let body):
            return "HTTP \(code): \(body)"
        case .apiError(let message):
            return "API error: \(message)"
        case .decodingError(let err):
            return "Decoding error: \(err.localizedDescription)"
        case .encodingError(let err):
            return "Encoding error: \(err.localizedDescription)"
        case .daemonNotInstalled:
            return "x0xd binary not found. Please install it first."
        case .daemonStartFailed(let reason):
            return "Failed to start x0xd: \(reason)"
        case .webSocketError(let reason):
            return "WebSocket error: \(reason)"
        case .invalidURL(let path):
            return "Invalid URL path: \(path)"
        case .unexpected(let message):
            return message
        }
    }
}

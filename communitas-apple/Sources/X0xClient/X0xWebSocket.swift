import Foundation

/// A WebSocket connection to the x0x daemon for real-time event streaming.
public final class X0xWebSocket: NSObject, Sendable {
    private let url: URL
    private let task: URLSessionWebSocketTask

    /// Creates a WebSocket connection to the daemon.
    /// - Parameters:
    ///   - baseURL: The base WebSocket URL (default: `ws://127.0.0.1:12700`).
    ///   - path: The endpoint path. Use `/ws` for the general-purpose WebSocket
    ///           or `/ws/direct` for direct-message WebSocket.
    ///   - token: Optional bearer token. When provided it is appended as `?token=<token>`
    ///            so the daemon can authenticate the connection before the upgrade completes.
    public init(baseURL: URL = URL(string: "ws://127.0.0.1:12700")!, path: String = "/ws", token: String? = nil) {
        // Append the auth token as a query parameter when present.
        let rawPath: String
        if let token, !token.isEmpty {
            let separator = path.contains("?") ? "&" : "?"
            rawPath = "\(path)\(separator)token=\(token)"
        } else {
            rawPath = path
        }

        guard let wsURL = URL(string: rawPath, relativeTo: baseURL)?.absoluteURL else {
            // Fall back to the base URL so we never crash; the connection will simply fail.
            self.url = baseURL
            self.task = URLSession.shared.webSocketTask(with: baseURL)
            super.init()
            return
        }
        self.url = wsURL
        self.task = URLSession.shared.webSocketTask(with: wsURL)
        super.init()
    }

    /// Convenience initialiser that derives baseURL and token from a ``X0xConfig``.
    /// - Parameters:
    ///   - config: The discovered daemon configuration.
    ///   - path: The endpoint path (default `/ws`).
    public convenience init(config: X0xConfig, path: String = "/ws") {
        let base = config.baseWSURL ?? URL(string: "ws://127.0.0.1:12700")!
        self.init(baseURL: base, path: path, token: config.token)
    }

    /// Connect and begin receiving messages.
    public func connect() {
        task.resume()
    }

    /// Send a JSON message to the server.
    public func send(_ text: String) async throws {
        try await task.send(.string(text))
    }

    /// Receive the next message as a string.
    public func receive() async throws -> String {
        let message = try await task.receive()
        switch message {
        case .string(let text):
            return text
        case .data(let data):
            guard let text = String(data: data, encoding: .utf8) else {
                throw X0xError.webSocketError(reason: "Received non-UTF8 binary data")
            }
            return text
        @unknown default:
            throw X0xError.webSocketError(reason: "Unknown WebSocket message type")
        }
    }

    /// Disconnect the WebSocket.
    public func disconnect() {
        task.cancel(with: .goingAway, reason: nil)
    }
}

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
    public init(baseURL: URL = URL(string: "ws://127.0.0.1:12700")!, path: String = "/ws") {
        guard let wsURL = URL(string: path, relativeTo: baseURL) else {
            fatalError("Invalid WebSocket URL: \(baseURL)\(path)")
        }
        self.url = wsURL
        self.task = URLSession.shared.webSocketTask(with: wsURL)
        super.init()
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

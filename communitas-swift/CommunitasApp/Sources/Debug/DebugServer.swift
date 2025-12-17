import Foundation
import Network

/// Lightweight HTTP debug server for programmatic app control
/// Only available in DEBUG builds - binds to localhost:9999
@MainActor
public final class DebugServer {
    public static let shared = DebugServer()

    private var listener: NWListener?
    private var connections: [NWConnection] = []
    private(set) var port: UInt16 = 9999
    private let queue = DispatchQueue(label: "com.communitas.debug-server")

    /// Registered action handlers
    private var handlers: [String: @MainActor (Data?) async throws -> Data] = [:]

    private init() {}

    /// Testing initializer that allows creating isolated instances for unit tests
    internal init(forTesting: Bool) {
        // Don't start the listener in test mode
    }

    /// Start the debug server (only in DEBUG builds)
    /// - Parameter customPort: Optional custom port (default: 9999, or DEBUG_PORT env var)
    public func start(customPort: UInt16? = nil) {
        #if DEBUG
        // Use custom port, environment variable, or default
        if let customPort = customPort {
            port = customPort
        } else if let envPort = ProcessInfo.processInfo.environment["DEBUG_PORT"],
                  let envPortNum = UInt16(envPort) {
            port = envPortNum
        }

        // Capture port for use in closure
        let serverPort = port

        do {
            let params = NWParameters.tcp
            params.allowLocalEndpointReuse = true

            listener = try NWListener(using: params, on: NWEndpoint.Port(rawValue: port)!)

            listener?.stateUpdateHandler = { state in
                switch state {
                case .ready:
                    print("[DebugServer] Listening on localhost:\(serverPort)")
                case .failed(let error):
                    print("[DebugServer] Failed to start: \(error)")
                case .cancelled:
                    print("[DebugServer] Cancelled")
                default:
                    break
                }
            }

            listener?.newConnectionHandler = { [weak self] connection in
                Task { @MainActor in
                    self?.handleConnection(connection)
                }
            }

            listener?.start(queue: queue)
            print("[DebugServer] Starting on port \(port)...")
        } catch {
            print("[DebugServer] Failed to create listener: \(error)")
        }
        #else
        print("[DebugServer] Not available in release builds")
        #endif
    }

    /// Stop the debug server
    func stop() {
        #if DEBUG
        listener?.cancel()
        listener = nil
        connections.forEach { $0.cancel() }
        connections.removeAll()
        print("[DebugServer] Stopped")
        #endif
    }

    /// Register an action handler
    func registerHandler(_ action: String, handler: @escaping @MainActor (Data?) async throws -> Data) {
        handlers[action] = handler
    }

    // MARK: - Connection Handling

    private func handleConnection(_ connection: NWConnection) {
        connections.append(connection)

        connection.stateUpdateHandler = { [weak self] state in
            Task { @MainActor in
                switch state {
                case .ready:
                    self?.receiveRequest(on: connection)
                case .failed, .cancelled:
                    self?.removeConnection(connection)
                default:
                    break
                }
            }
        }

        connection.start(queue: queue)
    }

    private func removeConnection(_ connection: NWConnection) {
        connections.removeAll { $0 === connection }
    }

    private func receiveRequest(on connection: NWConnection) {
        connection.receive(minimumIncompleteLength: 1, maximumLength: 65536) { [weak self] data, _, isComplete, error in
            Task { @MainActor in
                guard let self = self else { return }

                if let error = error {
                    print("[DebugServer] Receive error: \(error)")
                    connection.cancel()
                    return
                }

                if let data = data, !data.isEmpty {
                    self.processRequest(data: data, connection: connection)
                }

                if isComplete {
                    connection.cancel()
                }
            }
        }
    }

    private func processRequest(data: Data, connection: NWConnection) {
        // Parse HTTP request using binary search for header/body separator
        guard let requestString = String(data: data, encoding: .utf8) else {
            sendErrorResponse(connection: connection, status: 400, message: "Invalid request encoding")
            return
        }

        // Find the header/body separator (\r\n\r\n)
        let separator = "\r\n\r\n"
        guard let separatorRange = requestString.range(of: separator) else {
            // Try with just \n\n as fallback
            let altSeparator = "\n\n"
            guard let altRange = requestString.range(of: altSeparator) else {
                sendErrorResponse(connection: connection, status: 400, message: "No header/body separator found")
                return
            }
            // Use alternative separator
            processRequestWithSeparator(requestString: requestString, separatorRange: altRange, connection: connection)
            return
        }

        processRequestWithSeparator(requestString: requestString, separatorRange: separatorRange, connection: connection)
    }

    private func processRequestWithSeparator(requestString: String, separatorRange: Range<String.Index>, connection: NWConnection) {
        let headerSection = String(requestString[..<separatorRange.lowerBound])
        let bodySection = String(requestString[separatorRange.upperBound...])

        // Parse request line
        let lines = headerSection.components(separatedBy: .newlines).filter { !$0.isEmpty }
        guard let requestLine = lines.first else {
            sendErrorResponse(connection: connection, status: 400, message: "Empty request")
            return
        }

        let requestParts = requestLine.components(separatedBy: " ")
        guard requestParts.count >= 2 else {
            sendErrorResponse(connection: connection, status: 400, message: "Malformed request line")
            return
        }

        let method = requestParts[0]
        let path = requestParts[1]

        // Extract body as Data
        var body: Data?
        if !bodySection.isEmpty {
            body = bodySection.data(using: .utf8)
        }

        // Route the request
        Task { @MainActor in
            await self.routeRequest(method: method, path: path, body: body, connection: connection)
        }
    }

    private func routeRequest(method: String, path: String, body: Data?, connection: NWConnection) async {
        // Handle CORS preflight
        if method == "OPTIONS" {
            sendCORSResponse(connection: connection)
            return
        }

        // Parse path: /debug/{action}
        let pathComponents = path.split(separator: "/").map(String.init)

        // Health check
        if path == "/health" || path == "/debug/health" {
            let response: [String: Any] = [
                "status": "ok",
                "server": "DebugServer",
                "port": port,
                "handlers": Array(handlers.keys)
            ]
            sendJSONResponse(connection: connection, data: response)
            return
        }

        // Extract action from path
        guard pathComponents.count >= 2,
              pathComponents[0] == "debug" else {
            sendErrorResponse(connection: connection, status: 404, message: "Not found. Use /debug/{action}")
            return
        }

        let action = pathComponents[1]

        // Find handler
        guard let handler = handlers[action] else {
            sendErrorResponse(connection: connection, status: 404, message: "Unknown action: \(action). Available: \(Array(handlers.keys))")
            return
        }

        // Execute handler
        do {
            let resultData = try await handler(body)
            sendResponse(connection: connection, status: 200, body: resultData, contentType: "application/json")
        } catch {
            sendErrorResponse(connection: connection, status: 500, message: "Handler error: \(error.localizedDescription)")
        }
    }

    // MARK: - Response Helpers

    private func sendJSONResponse(connection: NWConnection, data: Any) {
        do {
            let jsonData = try JSONSerialization.data(withJSONObject: data, options: [.prettyPrinted])
            sendResponse(connection: connection, status: 200, body: jsonData, contentType: "application/json")
        } catch {
            sendErrorResponse(connection: connection, status: 500, message: "JSON serialization error")
        }
    }

    private func sendResponse(connection: NWConnection, status: Int, body: Data, contentType: String) {
        let statusText = httpStatusText(status)
        var response = "HTTP/1.1 \(status) \(statusText)\r\n"
        response += "Content-Type: \(contentType)\r\n"
        response += "Content-Length: \(body.count)\r\n"
        response += "Access-Control-Allow-Origin: *\r\n"
        response += "Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n"
        response += "Access-Control-Allow-Headers: Content-Type\r\n"
        response += "Connection: close\r\n"
        response += "\r\n"

        var responseData = response.data(using: .utf8)!
        responseData.append(body)

        connection.send(content: responseData, completion: .contentProcessed { error in
            if let error = error {
                print("[DebugServer] Send error: \(error)")
            }
            connection.cancel()
        })
    }

    private func sendErrorResponse(connection: NWConnection, status: Int, message: String) {
        let errorJSON: [String: Any] = [
            "error": true,
            "status": status,
            "message": message
        ]
        do {
            let jsonData = try JSONSerialization.data(withJSONObject: errorJSON, options: [])
            sendResponse(connection: connection, status: status, body: jsonData, contentType: "application/json")
        } catch {
            let fallback = "{\"error\":true,\"message\":\"Internal error\"}"
            sendResponse(connection: connection, status: 500, body: fallback.data(using: .utf8)!, contentType: "application/json")
        }
    }

    private func sendCORSResponse(connection: NWConnection) {
        var response = "HTTP/1.1 204 No Content\r\n"
        response += "Access-Control-Allow-Origin: *\r\n"
        response += "Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n"
        response += "Access-Control-Allow-Headers: Content-Type\r\n"
        response += "Access-Control-Max-Age: 86400\r\n"
        response += "Connection: close\r\n"
        response += "\r\n"

        connection.send(content: response.data(using: .utf8), completion: .contentProcessed { _ in
            connection.cancel()
        })
    }

    internal func httpStatusText(_ status: Int) -> String {
        switch status {
        case 200: return "OK"
        case 204: return "No Content"
        case 400: return "Bad Request"
        case 404: return "Not Found"
        case 500: return "Internal Server Error"
        default: return "Unknown"
        }
    }
}

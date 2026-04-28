import Foundation

// MARK: - SseFrame

/// One parsed Server-Sent Event frame from x0xd.
///
/// SSE frames carry one or more `event:`/`id:`/`data:` lines terminated by
/// a blank line. Multi-line `data:` values are joined with `\n`.
public struct SseFrame: Sendable, Equatable {
    /// Optional `event:` field — e.g. `"peer-lifecycle"`, `"presence"`.
    public let event: String?
    /// Optional `id:` field for client-side replay.
    public let id: String?
    /// Concatenated `data:` payload.
    public let data: String

    public init(event: String? = nil, id: String? = nil, data: String) {
        self.event = event
        self.id = id
        self.data = data
    }

    /// Decode the `data:` payload as JSON of the requested type.
    public func json<T: Decodable>(as _: T.Type = T.self) throws -> T {
        guard let bytes = data.data(using: .utf8) else {
            throw X0xError.unexpected(message: "non-UTF8 SSE data: \(data)")
        }
        do {
            return try JSONDecoder().decode(T.self, from: bytes)
        } catch {
            throw X0xError.decodingError(underlying: error)
        }
    }
}

// MARK: - X0xSseStream

/// Async server-sent-event consumer for the x0x daemon's streaming routes.
///
/// Mirrors the Rust `X0xSseStream` API in
/// `communitas-x0x-client/src/sse.rs`. Frames are exposed as an
/// `AsyncThrowingStream<SseFrame, Error>` so callers iterate with
/// `for try await frame in stream.frames`.
///
/// The four canonical endpoints have convenience constructors:
/// - ``connect()``: `/events`
/// - ``connectDirect()``: `/direct/events`
/// - ``connectPresence()``: `/presence/events`
/// - ``connectPeerEvents()``: `/peers/events` (x0xd ≥ 0.19.6)
public final class X0xSseStream: Sendable {
    /// Bytes-level URLSession task driving the stream.
    private let task: URLSessionDataTask

    /// Parsed frames as an async throwing stream.
    public let frames: AsyncThrowingStream<SseFrame, Error>

    private init(
        task: URLSessionDataTask,
        frames: AsyncThrowingStream<SseFrame, Error>
    ) {
        self.task = task
        self.frames = frames
    }

    /// Cancel the underlying connection. Idempotent.
    public func cancel() {
        task.cancel()
    }

    // MARK: Convenience constructors

    /// Connect to `/events` using a discovered ``X0xConfig``.
    public static func connect() async throws -> X0xSseStream {
        try await connectStream(path: "/events")
    }

    /// Connect to `/direct/events` using a discovered ``X0xConfig``.
    public static func connectDirect() async throws -> X0xSseStream {
        try await connectStream(path: "/direct/events")
    }

    /// Connect to `/presence/events` using a discovered ``X0xConfig``.
    public static func connectPresence() async throws -> X0xSseStream {
        try await connectStream(path: "/presence/events")
    }

    /// Connect to `/peers/events` (peer-lifecycle, x0xd ≥ 0.19.6) using a
    /// discovered ``X0xConfig``.
    public static func connectPeerEvents() async throws -> X0xSseStream {
        try await connectStream(path: "/peers/events")
    }

    // MARK: Explicit-config constructors

    /// Connect with an explicit config + path. Useful for tests pointing
    /// at a temporary daemon.
    public static func connect(config: X0xConfig, path: String) async throws -> X0xSseStream {
        guard let base = config.baseHTTPURL else {
            throw X0xError.invalidURL(path: "http://\(config.address)")
        }
        let url = base.appendingPathComponent(path.trimmingCharacters(in: CharacterSet(charactersIn: "/")))
        return try await connect(url: url, token: config.token)
    }

    /// Connect to a fully-qualified URL with an optional bearer token.
    public static func connect(url: URL, token: String?) async throws -> X0xSseStream {
        var request = URLRequest(url: url)
        request.setValue("text/event-stream", forHTTPHeaderField: "Accept")
        if let token = token, !token.isEmpty {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }

        let session = URLSession(configuration: .default)
        let (bytes, response) = try await session.bytes(for: request)
        guard let http = response as? HTTPURLResponse else {
            throw X0xError.unexpected(message: "non-HTTP response on SSE \(url)")
        }
        guard (200..<300).contains(http.statusCode) else {
            throw X0xError.httpError(statusCode: http.statusCode, body: "")
        }

        // The URLSession.bytes(for:) call doesn't expose the underlying
        // task directly, so we keep a placeholder to honour cancel().
        // session.invalidateAndCancel() inside cancel() tears down the
        // implicit task that owns `bytes`.
        let placeholder = session.dataTask(with: request)
        let stream = AsyncThrowingStream<SseFrame, Error> { continuation in
            let task = Task {
                do {
                    var pending = SseFrameAccumulator()
                    for try await line in bytes.lines {
                        if let frame = pending.feed(line: line) {
                            continuation.yield(frame)
                        }
                    }
                    if let frame = pending.flush() {
                        continuation.yield(frame)
                    }
                    continuation.finish()
                } catch {
                    continuation.finish(throwing: error)
                }
            }
            continuation.onTermination = { _ in
                task.cancel()
                session.invalidateAndCancel()
            }
        }

        return X0xSseStream(task: placeholder, frames: stream)
    }

    // MARK: Private helpers

    private static func connectStream(path: String) async throws -> X0xSseStream {
        guard let config = X0xConfig.discover() else {
            throw X0xError.daemonStartFailed(reason: "x0xd config not discoverable")
        }
        return try await connect(config: config, path: path)
    }
}

// MARK: - Frame accumulator

/// Stateful SSE frame parser. Per the SSE spec a frame is terminated by a
/// blank line; lines starting with `:` are comments. Multi-line `data:`
/// fields are joined with `\n`.
private struct SseFrameAccumulator {
    private var event: String?
    private var id: String?
    private var data: [String] = []

    mutating func feed(line: String) -> SseFrame? {
        if line.isEmpty {
            return finalise()
        }
        if line.hasPrefix(":") {
            return nil // comment
        }
        guard let colonIdx = line.firstIndex(of: ":") else {
            // Field with no value — treat the whole line as the field name.
            return nil
        }
        let field = String(line[..<colonIdx])
        var value = String(line[line.index(after: colonIdx)...])
        if value.hasPrefix(" ") {
            value.removeFirst()
        }
        switch field {
        case "event":
            event = value
        case "id":
            id = value
        case "data":
            data.append(value)
        default:
            break
        }
        return nil
    }

    /// Force-emit any accumulated state on stream close. Returns nil if
    /// nothing meaningful was buffered.
    mutating func flush() -> SseFrame? {
        finalise()
    }

    private mutating func finalise() -> SseFrame? {
        defer {
            event = nil
            id = nil
            data = []
        }
        guard !data.isEmpty || event != nil || id != nil else {
            return nil
        }
        let payload = data.joined(separator: "\n")
        return SseFrame(event: event, id: id, data: payload)
    }
}

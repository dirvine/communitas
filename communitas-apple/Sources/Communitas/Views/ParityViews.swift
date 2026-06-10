import SwiftUI
import X0xClient

// MARK: - LiveFeedView (Pub/Sub WebSocket live feed)

/// Pub/sub live feed driven by `X0xWebSocket` (`/ws`).
///
/// Closes the Apple-column 🟡 in `Messaging — pub/sub / WebSocket live feed`
/// of `x0x/docs/parity-matrix.md`. Subscribes to a topic via the REST
/// API, opens a WebSocket, and renders incoming `gossip` frames as
/// they arrive. Includes an inline publisher for round-trip XCUITest.
struct LiveFeedView: View {
    @EnvironmentObject var appState: AppState
    @StateObject private var feed = LiveFeedModel()

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 8) {
                TextField("Topic", text: $feed.topic)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("pubsub-topic")
                Button(feed.connected ? "Disconnect" : "Subscribe") {
                    Task { await feed.toggle(client: appState.client, baseURL: appState.client.baseURL, token: appState.client.token) }
                }
                .buttonStyle(.borderedProminent)
                .disabled(feed.topic.trimmingCharacters(in: .whitespaces).isEmpty
                    || appState.daemonState != .running)
                .accessibilityIdentifier("pubsub-subscribe")
            }

            HStack(spacing: 8) {
                TextEditor(text: $feed.draft)
                    .frame(minHeight: 80, maxHeight: 120)
                    .border(Color.secondary.opacity(0.2))
                    .accessibilityIdentifier("pubsub-payload")
                Button("Publish") {
                    Task { await feed.publish(client: appState.client) }
                }
                .buttonStyle(.bordered)
                .disabled(feed.draft.isEmpty
                    || feed.topic.trimmingCharacters(in: .whitespaces).isEmpty
                    || appState.daemonState != .running)
                .accessibilityIdentifier("pubsub-publish")
            }

            if let last = feed.lastReceived {
                Text(last)
                    .font(.system(.body, design: .monospaced))
                    .padding(8)
                    .background(Color.secondary.opacity(0.1))
                    .cornerRadius(6)
                    .accessibilityIdentifier("pubsub-last-received")
            }

            if let err = feed.errorMessage {
                Text(err).foregroundStyle(.red).font(.caption)
            }

            List(feed.frames.reversed(), id: \.self) { line in
                Text(line)
                    .font(.system(.caption, design: .monospaced))
                    .lineLimit(2)
            }
            .accessibilityIdentifier("live-feed-frames")
            .frame(maxHeight: .infinity)
        }
        .padding(20)
        .navigationTitle("Live Feed")
        .onDisappear {
            feed.disconnect()
        }
    }
}

/// Backing model for ``LiveFeedView``.
@MainActor
final class LiveFeedModel: ObservableObject {
    @Published var topic: String = ""
    @Published var draft: String = ""
    @Published var frames: [String] = []
    @Published var lastReceived: String?
    @Published var connected: Bool = false
    @Published var errorMessage: String?

    private var ws: X0xWebSocket?
    private var receiveTask: Task<Void, Never>?
    private var subscriptionId: String?

    func toggle(client: X0xClient, baseURL: URL, token: String?) async {
        if connected {
            disconnect()
        } else {
            await connect(client: client, baseURL: baseURL, token: token)
        }
    }

    func connect(client: X0xClient, baseURL: URL, token: String?) async {
        let trimmed = topic.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        do {
            subscriptionId = try await client.subscribe(topic: trimmed)
        } catch {
            errorMessage = "subscribe failed: \(error.localizedDescription)"
            return
        }

        let wsBase: URL
        if var components = URLComponents(url: baseURL, resolvingAgainstBaseURL: false) {
            components.scheme = components.scheme == "https" ? "wss" : "ws"
            wsBase = components.url ?? baseURL
        } else {
            wsBase = baseURL
        }
        let socket = X0xWebSocket(baseURL: wsBase, path: "/ws", token: token)
        socket.connect()
        do {
            try await socket.send(#"{"type":"subscribe","topic":"\#(trimmed)"}"#)
        } catch {
            errorMessage = "ws subscribe failed: \(error.localizedDescription)"
            return
        }
        ws = socket
        connected = true
        errorMessage = nil

        receiveTask = Task { [weak self] in
            await self?.receiveLoop(socket: socket)
        }
    }

    private func receiveLoop(socket: X0xWebSocket) async {
        while !Task.isCancelled {
            do {
                let msg = try await socket.receive()
                await MainActor.run {
                    self.frames.append(msg)
                    if self.frames.count > 200 { self.frames.removeFirst() }
                    if let payload = Self.decodePayload(msg) {
                        self.lastReceived = payload
                    } else {
                        self.lastReceived = msg
                    }
                }
            } catch {
                await MainActor.run {
                    if self.connected {
                        self.errorMessage = "ws read failed: \(error.localizedDescription)"
                        self.connected = false
                    }
                }
                return
            }
        }
    }

    /// Decode a base64 `payload` field out of a JSON gossip frame and
    /// return it as a UTF-8 string. Returns nil for non-JSON frames or
    /// frames that don't carry a base64 payload — callers fall back to
    /// the raw frame so probes are observable either way.
    private static func decodePayload(_ json: String) -> String? {
        guard let data = json.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let payload = obj["payload"] as? String,
              let bytes = Data(base64Encoded: payload),
              let str = String(data: bytes, encoding: .utf8)
        else { return nil }
        return str
    }

    func publish(client: X0xClient) async {
        let trimmed = topic.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        let payload = Data(draft.utf8).base64EncodedString()
        do {
            try await client.publish(topic: trimmed, payload: payload)
        } catch {
            errorMessage = "publish failed: \(error.localizedDescription)"
        }
    }

    func disconnect() {
        receiveTask?.cancel()
        receiveTask = nil
        ws?.disconnect()
        ws = nil
        if let subId = subscriptionId {
            // Best-effort unsubscribe; ignore failures during teardown.
            Task.detached { [subId] in
                _ = try? await X0xClient.fromDiscovery()?.unsubscribe(subscriptionId: subId)
            }
        }
        subscriptionId = nil
        connected = false
    }
}

// MARK: - KvStoresView

/// Top-level KV-store view — exposes `POST /stores`, `PUT/GET/DELETE
/// /stores/:id/:key` so XCUITest can drive the access-policy + CRUD
/// round-trip used by the parity matrix Apple column.
struct KvStoresView: View {
    @EnvironmentObject var appState: AppState
    @State private var stores: [StoreSummary] = []
    @State private var newStoreName: String = ""
    @State private var selectedStoreId: String?
    @State private var key: String = ""
    @State private var value: String = ""
    @State private var lastReadValue: String = ""
    @State private var statusMessage: String?
    @State private var showCreateSheet = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Button("New Store") { showCreateSheet = true }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier("kv-new-store")
                if let id = selectedStoreId {
                    Text("Selected: \(id.prefix(12))…")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                Spacer()
            }

            Picker("Store", selection: $selectedStoreId) {
                Text("(none)").tag(Optional<String>.none)
                ForEach(stores) { store in
                    Text(store.id.prefix(12) + "…").tag(Optional(store.id))
                }
            }
            .pickerStyle(.menu)
            .accessibilityIdentifier("kv-store-picker")

            HStack(spacing: 8) {
                TextField("Key", text: $key)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("kv-key")
                TextField("Value", text: $value)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("kv-value")
            }

            HStack(spacing: 8) {
                Button("PUT") {
                    Task { await put() }
                }
                .buttonStyle(.borderedProminent)
                .disabled(selectedStoreId == nil || key.isEmpty)
                .accessibilityIdentifier("kv-put")

                Button("GET") {
                    Task { await get() }
                }
                .buttonStyle(.bordered)
                .disabled(selectedStoreId == nil || key.isEmpty)
                .accessibilityIdentifier("kv-get")

                Button("DELETE") {
                    Task { await delete() }
                }
                .buttonStyle(.bordered)
                .tint(.red)
                .disabled(selectedStoreId == nil || key.isEmpty)
                .accessibilityIdentifier("kv-delete")
            }

            if !lastReadValue.isEmpty {
                Text(lastReadValue)
                    .font(.system(.body, design: .monospaced))
                    .padding(8)
                    .background(Color.secondary.opacity(0.1))
                    .cornerRadius(6)
                    .accessibilityIdentifier("kv-last-read-value")
            }

            if let s = statusMessage {
                Text(s).font(.caption).foregroundStyle(.secondary)
            }

            Spacer()
        }
        .padding(20)
        .navigationTitle("KV Stores")
        .task { await reload() }
        .sheet(isPresented: $showCreateSheet) {
            VStack(spacing: 12) {
                Text("Create Store").font(.headline)
                TextField("Store name", text: $newStoreName)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("kv-store-name")
                HStack {
                    Button("Cancel") { showCreateSheet = false }
                    Spacer()
                    Button("Create") {
                        Task { await createStore() }
                    }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier("kv-store-create")
                }
            }
            .padding(20)
            .frame(width: 360)
        }
    }

    private func createStore() async {
        let trimmed = newStoreName.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        do {
            let id = try await appState.client.createStore(
                name: trimmed,
                topic: "kv-\(UUID().uuidString.prefix(8))"
            )
            selectedStoreId = id
            newStoreName = ""
            showCreateSheet = false
            statusMessage = "Created store \(id.prefix(12))…"
            await reload()
        } catch {
            statusMessage = "Create failed: \(error.localizedDescription)"
        }
    }

    private func reload() async {
        do {
            stores = try await appState.client.listStores()
        } catch {
            stores = []
        }
    }

    private func put() async {
        guard let id = selectedStoreId else { return }
        do {
            try await appState.client.storePut(storeId: id, key: key, value: value)
            statusMessage = "PUT ok"
        } catch {
            statusMessage = "PUT failed: \(error.localizedDescription)"
        }
    }

    private func get() async {
        guard let id = selectedStoreId else { return }
        do {
            lastReadValue = try await appState.client.storeGet(storeId: id, key: key)
            statusMessage = "GET ok"
        } catch {
            lastReadValue = ""
            statusMessage = "GET failed: \(error.localizedDescription)"
        }
    }

    private func delete() async {
        guard let id = selectedStoreId else { return }
        do {
            try await appState.client.storeDelete(storeId: id, key: key)
            statusMessage = "DELETE ok"
        } catch {
            statusMessage = "DELETE failed: \(error.localizedDescription)"
        }
    }
}

// MARK: - FourWordBootstrapView

/// Surface for the `four_word_networking`-backed location bootstrap.
///
/// Decoding 4 words → IP:port lives in the Rust `four_word_networking`
/// crate; Communitas calls the daemon's CLI binary as a subprocess to
/// reuse that logic without duplicating the dictionary into Swift. This
/// matches the CLI surface (`x0x connect <words…>`) and keeps the
/// matrix's "Four-word network bootstrap" capability reachable from
/// Apple.
struct FourWordBootstrapView: View {
    @EnvironmentObject var appState: AppState

    @State private var w1: String = ""
    @State private var w2: String = ""
    @State private var w3: String = ""
    @State private var w4: String = ""
    @State private var output: String = ""
    @State private var running: Bool = false

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Connect to a peer by their four-word location address.")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            HStack(spacing: 8) {
                TextField("word 1", text: $w1).accessibilityIdentifier("four-word-w1")
                TextField("word 2", text: $w2).accessibilityIdentifier("four-word-w2")
                TextField("word 3", text: $w3).accessibilityIdentifier("four-word-w3")
                TextField("word 4", text: $w4).accessibilityIdentifier("four-word-w4")
            }
            .textFieldStyle(.roundedBorder)
            .accessibilityIdentifier("four-word-input")

            HStack {
                Button(running ? "Connecting…" : "Connect") {
                    Task { await connect() }
                }
                .buttonStyle(.borderedProminent)
                .disabled(running || !FourWordResolver.binaryAvailable())
                .accessibilityIdentifier("four-word-connect-button")

                Spacer()
            }

            if !FourWordResolver.binaryAvailable() {
                Text("`x0x` CLI not found — install the daemon binary to enable four-word lookups.")
                    .font(.caption)
                    .foregroundStyle(.orange)
            }

            if !output.isEmpty {
                ScrollView {
                    Text(output)
                        .font(.system(.caption, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .textSelection(.enabled)
                        .padding(8)
                }
                .frame(maxHeight: 240)
                .background(Color.secondary.opacity(0.08))
                .cornerRadius(6)
                .accessibilityIdentifier("four-word-output")
            }

            Spacer()
        }
        .padding(20)
        .navigationTitle("Four-Word Bootstrap")
    }

    private func connect() async {
        running = true
        defer { running = false }
        let words = [w1, w2, w3, w4].map { $0.trimmingCharacters(in: .whitespaces) }
        do {
            output = try await FourWordResolver.connect(words: words)
        } catch {
            output = "Failed: \(error.localizedDescription)"
        }
    }
}

/// Small helper that fronts the `x0x connect <words…>` CLI invocation.
///
/// Keeping it static makes the Swift round-trip test independent of
/// `AppState` and easier to gate behind `X0X_LIVE_TESTS=1`.
enum FourWordResolver {
    /// Locations probed by ``locateBinary()``. Matches ``DaemonManager``
    /// plus the workspace-relative debug/release outputs so XCUITest in
    /// a Swift Package layout still finds it.
    static let searchPaths: [String] = [
        "/usr/local/bin/x0x",
        "/opt/homebrew/bin/x0x",
        "/opt/zerobrew/bin/x0x",
        "\(NSHomeDirectory())/.local/bin/x0x",
        "\(NSHomeDirectory())/.cargo/bin/x0x",
        "\(NSHomeDirectory())/.x0x/bin/x0x",
    ]

    static func binaryAvailable() -> Bool {
        locateBinary() != nil
    }

    static func locateBinary() -> URL? {
        if let override = ProcessInfo.processInfo.environment["X0X_BIN"], !override.isEmpty {
            let url = URL(fileURLWithPath: override)
            if FileManager.default.isExecutableFile(atPath: url.path) {
                return url
            }
        }
        for p in searchPaths {
            if FileManager.default.isExecutableFile(atPath: p) {
                return URL(fileURLWithPath: p)
            }
        }
        // Also check target/{release,debug}/x0x in both a nested checkout
        // (`communitas/x0x`) and the normal sibling workspace (`../x0x`).
        let communitasRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // Views
            .deletingLastPathComponent() // Communitas
            .deletingLastPathComponent() // Sources
            .deletingLastPathComponent() // communitas-apple
            .deletingLastPathComponent() // communitas
        let workspaceRoot = communitasRoot.deletingLastPathComponent()
        let candidates = [
            communitasRoot.appendingPathComponent("x0x/target/release/x0x"),
            communitasRoot.appendingPathComponent("x0x/target/debug/x0x"),
            workspaceRoot.appendingPathComponent("x0x/target/release/x0x"),
            workspaceRoot.appendingPathComponent("x0x/target/debug/x0x"),
        ]
        for c in candidates where FileManager.default.isExecutableFile(atPath: c.path) {
            return c
        }
        return nil
    }

    enum FourWordError: Error, CustomStringConvertible {
        case binaryMissing
        case invalidWords(String)
        case nonZeroExit(Int32, String)
        case timeout(TimeInterval)

        var description: String {
            switch self {
            case .binaryMissing: return "x0x CLI binary not found"
            case .invalidWords(let s): return "invalid four-word phrase: \(s)"
            case .nonZeroExit(let code, let out): return "x0x connect exited \(code): \(out)"
            case .timeout(let secs): return "x0x connect timed out after \(Int(secs))s"
            }
        }
    }

    /// Run `x0x connect <w1> <w2> <w3> <w4>` and return the combined
    /// stdout+stderr output. Bubbles non-zero exits as ``nonZeroExit``
    /// so callers can distinguish "decode succeeded but peer not yet
    /// announced" from "decode failed".
    static func connect(words: [String]) async throws -> String {
        let trimmed = words.map { $0.trimmingCharacters(in: .whitespaces) }
        guard trimmed.count == 4, trimmed.allSatisfy({ !$0.isEmpty }) else {
            throw FourWordError.invalidWords(words.joined(separator: " "))
        }
        return try await runCli(args: ["connect"] + trimmed, timeoutSecs: 15)
    }

    /// Best-effort version probe — used by the round-trip test to
    /// confirm the binary is reachable without actually trying a
    /// connection.
    static func version() async throws -> String {
        try await runCli(args: ["--version"], timeoutSecs: 5)
    }

    private static func runCli(args: [String], timeoutSecs: TimeInterval) async throws -> String {
        guard let bin = locateBinary() else {
            throw FourWordError.binaryMissing
        }

        let process = Process()
        process.executableURL = bin
        process.arguments = args
        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe
        try process.run()

        let deadline = Date().addingTimeInterval(timeoutSecs)
        while process.isRunning && Date() < deadline {
            try await Task.sleep(nanoseconds: 100_000_000)
        }
        if process.isRunning {
            process.terminate()
            throw FourWordError.timeout(timeoutSecs)
        }

        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        let text = String(data: data, encoding: .utf8) ?? ""
        if process.terminationStatus != 0 {
            throw FourWordError.nonZeroExit(process.terminationStatus, text)
        }
        return text
    }
}

// MARK: - PresenceToast

/// Wraps `X0xSseStream.connectPresence` and pushes the most recent
/// event into a published string so a transient toast can render it.
@MainActor
final class PresenceToastModel: ObservableObject {
    @Published var lastEvent: String?
    @Published var lastEventAt: Date?

    private var task: Task<Void, Never>?

    func start(config: X0xConfig) {
        guard task == nil else { return }
        task = Task { [weak self] in
            do {
                let stream = try await X0xSseStream.connect(config: config, path: "/presence/events")
                for try await frame in stream.frames {
                    guard !Task.isCancelled else { return }
                    await self?.handle(frame: frame)
                }
            } catch {
                // SSE is best-effort — silent on connection failure
                // so the rest of the app stays usable when the daemon
                // is restarting.
            }
        }
    }

    func stop() {
        task?.cancel()
        task = nil
    }

    private func handle(frame: SseFrame) async {
        let summary: String
        if let event = frame.event {
            summary = "\(event): \(frame.data.prefix(160))"
        } else {
            summary = String(frame.data.prefix(160))
        }
        await MainActor.run {
            self.lastEvent = summary
            self.lastEventAt = Date()
        }
    }
}

/// Transient banner that surfaces the most-recent presence SSE frame.
struct PresenceToastView: View {
    let event: String
    var body: some View {
        HStack(spacing: 8) {
            Image(systemName: "bell.badge")
            Text(event)
                .font(.caption)
                .lineLimit(2)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.regularMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .shadow(radius: 4)
        .accessibilityIdentifier("presence-event-toast")
    }
}

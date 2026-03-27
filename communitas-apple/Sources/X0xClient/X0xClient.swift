import Foundation

// MARK: - X0xConfig

/// Configuration discovered from the x0x daemon's runtime files.
///
/// The x0x daemon writes two files into `~/Library/Application Support/x0x/` when running:
/// - `api.port` — the host:port the API listens on (e.g., `127.0.0.1:12700`)
/// - `api-token` — a 64-character hex bearer token required for authenticated endpoints
public struct X0xConfig: Sendable {
    /// The HTTP/WebSocket address of the daemon (e.g., `"127.0.0.1:12700"`).
    public let address: String
    /// The bearer token for authenticating API requests.
    public let token: String

    /// The base HTTP URL derived from the discovered address.
    public var baseHTTPURL: URL? {
        URL(string: "http://\(address)")
    }

    /// The base WebSocket URL derived from the discovered address.
    public var baseWSURL: URL? {
        URL(string: "ws://\(address)")
    }

    public init(address: String, token: String) {
        self.address = address
        self.token = token
    }

    /// Attempt to discover the running daemon's configuration from the filesystem.
    ///
    /// Reads `~/Library/Application Support/x0x/api.port` and `api-token`.
    /// Returns `nil` if either file is missing or cannot be read (daemon not running).
    public static func discover() -> X0xConfig? {
        let fm = FileManager.default
        guard let appSupport = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first else {
            return nil
        }
        let x0xDir = appSupport.appendingPathComponent("x0x")
        let portFile = x0xDir.appendingPathComponent("api.port")
        let tokenFile = x0xDir.appendingPathComponent("api-token")

        guard let addressData = try? Data(contentsOf: portFile),
              let tokenData = try? Data(contentsOf: tokenFile) else {
            return nil
        }

        let address = String(decoding: addressData, as: UTF8.self)
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let token = String(decoding: tokenData, as: UTF8.self)
            .trimmingCharacters(in: .whitespacesAndNewlines)

        guard !address.isEmpty, !token.isEmpty else {
            return nil
        }

        return X0xConfig(address: address, token: token)
    }
}

// MARK: - X0xClient

/// HTTP client for the x0x daemon REST API at `http://127.0.0.1:12700`.
///
/// The x0x API uses **flattened** JSON responses: `{"ok": true, "field": "value", ...}`.
/// There is no universal `data` wrapper. Each response struct includes `ok` at the top level.
///
/// Authenticated endpoints require an `Authorization: Bearer <token>` header.
/// Use ``X0xConfig/discover()`` to obtain the token from the daemon's runtime files, then
/// initialise the client with ``init(config:)`` or ``fromDiscovery()``.
public final class X0xClient: Sendable {
    /// Base URL of the x0x daemon.
    public let baseURL: URL

    /// Bearer token for API authentication. `nil` means no auth header is sent.
    public let token: String?

    private let session: URLSession
    private let decoder: JSONDecoder

    /// Initialise with an explicit base URL and optional bearer token.
    /// - Parameters:
    ///   - baseURL: HTTP base URL of the daemon. Defaults to `http://127.0.0.1:12700`.
    ///   - token: Bearer token for authenticated endpoints. Pass `nil` to omit auth header.
    public init(baseURL: URL = URL(string: "http://127.0.0.1:12700")!, token: String? = nil) {
        self.baseURL = baseURL
        self.token = token
        self.session = URLSession.shared
        self.decoder = JSONDecoder()
    }

    /// Initialise from a discovered ``X0xConfig``.
    public convenience init(config: X0xConfig) {
        let url = config.baseHTTPURL ?? URL(string: "http://127.0.0.1:12700")!
        self.init(baseURL: url, token: config.token)
    }

    /// Attempt to discover the daemon config and build a client.
    /// Returns `nil` when the daemon is not running (config files absent).
    public static func fromDiscovery() -> X0xClient? {
        guard let config = X0xConfig.discover() else { return nil }
        return X0xClient(config: config)
    }

    /// A WebSocket base URL derived from ``baseURL`` by substituting the `http` scheme for `ws`.
    /// Falls back to `ws://127.0.0.1:12700` if the conversion fails.
    public var webSocketBaseURL: URL {
        guard var components = URLComponents(url: baseURL, resolvingAgainstBaseURL: false) else {
            return URL(string: "ws://127.0.0.1:12700")!
        }
        components.scheme = components.scheme == "https" ? "wss" : "ws"
        return components.url ?? URL(string: "ws://127.0.0.1:12700")!
    }

    // MARK: - Health & Status

    /// Check daemon health. `GET /health`
    public func health() async throws -> HealthStatus {
        try await get("/health")
    }

    /// Get daemon status. `GET /status`
    public func status() async throws -> DaemonStatus {
        try await get("/status")
    }

    /// Get the agent identity. `GET /agent`
    public func agent() async throws -> AgentIdentity {
        try await get("/agent")
    }

    // MARK: - Pub/Sub (Gossip Messaging)

    /// Publish a message to a topic. `POST /publish`
    public func publish(topic: String, payload: String) async throws {
        let body = PublishRequest(topic: topic, payload: payload)
        let _: Empty = try await post("/publish", body: body)
    }

    /// Subscribe to a topic. Returns the subscription ID. `POST /subscribe`
    public func subscribe(topic: String) async throws -> String {
        let body = ["topic": topic]
        let resp: SubscribeResponse = try await post("/subscribe", body: body)
        return resp.subscriptionId
    }

    // MARK: - Direct Messaging

    /// Send a direct message to an agent. `POST /direct/send`
    public func sendDirect(agentId: String, payload: String) async throws {
        let body = DirectMessageRequest(agentId: agentId, payload: payload)
        let _: Empty = try await post("/direct/send", body: body)
    }

    // MARK: - Contacts

    /// List all contacts. `GET /contacts`
    /// Returns wrapped: `{"ok":true,"contacts":[...]}`
    public func listContacts() async throws -> [Contact] {
        let resp: ContactListResponse = try await get("/contacts")
        return resp.contacts
    }

    /// Add a contact. `POST /contacts`
    public func addContact(agentId: String, trustLevel: TrustLevel, label: String?) async throws {
        let body = AddContactRequest(agentId: agentId, trustLevel: trustLevel, label: label)
        let _: Empty = try await post("/contacts", body: body)
    }

    /// Remove a contact. `DELETE /contacts/:agent_id`
    public func removeContact(agentId: String) async throws {
        let _: Empty = try await delete("/contacts/\(agentId)")
    }

    // MARK: - Groups (Named Groups)

    /// Create a new group. `POST /groups`
    public func createGroup(name: String, description: String?, displayName: String?) async throws -> CreatedGroup {
        let body = CreateGroupRequest(name: name, description: description, displayName: displayName)
        return try await post("/groups", body: body)
    }

    /// List all groups. `GET /groups`
    /// Returns wrapped: `{"ok":true,"groups":[...]}`
    public func listGroups() async throws -> [GroupSummary] {
        let resp: GroupListResponse = try await get("/groups")
        return resp.groups
    }

    /// Get detailed information about a group. `GET /groups/:id`
    public func groupInfo(groupId: String) async throws -> GroupInfo {
        try await get("/groups/\(groupId)")
    }

    /// Generate an invite for a group. `POST /groups/:id/invite`
    public func invite(groupId: String, expirySecs: UInt64? = nil) async throws -> InviteResponse {
        let body = InviteRequest(groupId: groupId, expirySecs: expirySecs)
        return try await post("/groups/\(groupId)/invite", body: body)
    }

    /// Join a group via invite token. `POST /groups/join`
    public func joinGroup(invite: String, displayName: String?) async throws -> JoinGroupResponse {
        let body = JoinGroupRequest(invite: invite, displayName: displayName)
        return try await post("/groups/join", body: body)
    }

    /// Leave a group. `DELETE /groups/:id`
    public func leaveGroup(groupId: String) async throws {
        let _: Empty = try await delete("/groups/\(groupId)")
    }

    /// Set the display name for the current agent in a group. `PUT /groups/:id/display-name`
    public func setGroupDisplayName(groupId: String, displayName: String) async throws {
        let body = SetGroupDisplayNameRequest(displayName: displayName)
        let _: Empty = try await put("/groups/\(groupId)/display-name", body: body)
    }

    // MARK: - Network

    /// Get network status information. `GET /network/status`
    public func networkStatus() async throws -> NetworkStatus {
        try await get("/network/status")
    }

    /// List connected peers. `GET /peers`
    /// Returns wrapped: `{"ok":true,"peers":["peer1","peer2"]}`
    public func peers() async throws -> [PeerInfo] {
        let resp: PeerListResponse = try await get("/peers")
        return resp.peerInfos
    }

    /// List discovered agents on the network. `GET /agents/discovered`
    /// Returns wrapped: `{"ok":true,"agents":[...]}`
    public func discoveredAgents() async throws -> [DiscoveredAgent] {
        let resp: DiscoveredAgentsResponse = try await get("/agents/discovered")
        return resp.agents
    }

    /// Fetch online presence. `GET /presence`
    /// Returns wrapped: `{"ok":true,"agents":["agentId1","agentId2"]}`
    public func presence() async throws -> [String] {
        let resp: PresenceResponse = try await get("/presence")
        return resp.agents
    }

    /// List machines associated with a contact. `GET /contacts/:agent_id/machines`
    public func listMachines(agentId: String) async throws -> [MachineRecord] {
        try await get("/contacts/\(agentId)/machines")
    }

    // MARK: - File Transfer

    /// Initiate a file send. `POST /files/send`
    public func sendFile(agentId: String, filename: String, size: UInt64) async throws -> String {
        let body = SendFileRequest(agentId: agentId, filename: filename, size: size)
        let resp: SendFileResponse = try await post("/files/send", body: body)
        return resp.transferId
    }

    /// List active file transfers. `GET /files/transfers`
    public func listTransfers() async throws -> [FileTransfer] {
        try await get("/files/transfers")
    }

    /// Accept an incoming file transfer. `POST /files/accept/:id`
    public func acceptFile(transferId: String) async throws {
        let _: Empty = try await post("/files/accept/\(transferId)", body: Empty())
    }

    /// Reject an incoming file transfer. `POST /files/reject/:id`
    public func rejectFile(transferId: String) async throws {
        let _: Empty = try await post("/files/reject/\(transferId)", body: Empty())
    }

    // MARK: - Task Lists

    /// Create a new task list. `POST /task-lists`
    public func createTaskList(name: String, topic: String) async throws -> String {
        let body = CreateTaskListRequest(name: name, topic: topic)
        let resp: CreateTaskListResponse = try await post("/task-lists", body: body)
        return resp.id
    }

    /// List tasks in a task list. `GET /task-lists/:id/tasks`
    public func listTasks(listId: String) async throws -> [TaskItem] {
        let resp: TaskList = try await get("/task-lists/\(listId)/tasks")
        return resp.tasks
    }

    /// Add a task to a task list. `POST /task-lists/:id/tasks`
    public func addTask(listId: String, title: String) async throws -> String {
        let body = AddTaskRequest(title: title)
        let resp: AddTaskResponse = try await post("/task-lists/\(listId)/tasks", body: body)
        return resp.taskId
    }

    /// Claim a task. `PATCH /task-lists/:id/tasks/:tid`
    public func claimTask(listId: String, taskId: String) async throws {
        let body = ["action": "claim"]
        let _: Empty = try await patch("/task-lists/\(listId)/tasks/\(taskId)", body: body)
    }

    /// Complete a task. `PATCH /task-lists/:id/tasks/:tid`
    public func completeTask(listId: String, taskId: String) async throws {
        let body = ["action": "complete"]
        let _: Empty = try await patch("/task-lists/\(listId)/tasks/\(taskId)", body: body)
    }

    // MARK: - Agent Connection

    /// Connect to another agent by ID. `POST /agents/connect`
    public func connectAgent(agentId: String) async throws {
        let body = ["agent_id": agentId]
        let _: Empty = try await post("/agents/connect", body: body)
    }

    // MARK: - Stores (KV)

    /// Create a new store. `POST /stores`
    public func createStore(name: String, topic: String) async throws -> String {
        let body = CreateStoreRequest(name: name, topic: topic)
        let resp: CreateStoreResponse = try await post("/stores", body: body)
        return resp.id
    }

    /// List stores. `GET /stores`
    public func listStores() async throws -> [StoreSummary] {
        let resp: StoreListResponse = try await get("/stores")
        return resp.stores
    }

    /// Join a store. `POST /stores/:id/join`
    public func joinStore(storeId: String, topic: String) async throws {
        let body = ["topic": topic]
        let _: Empty = try await post("/stores/\(storeId)/join", body: body)
    }

    /// List keys in a store. `GET /stores/:id/keys`
    public func storeKeys(storeId: String) async throws -> [String] {
        let resp: StoreKeysResponse = try await get("/stores/\(storeId)/keys")
        return resp.keys.map(\.key)
    }

    /// Get a value from a store. `GET /stores/:id/:key`
    /// The x0x store API returns base64-encoded values; this method decodes them.
    public func storeGet(storeId: String, key: String) async throws -> String {
        let resp: StoreGetResponse = try await get("/stores/\(storeId)/\(key)")
        guard let data = Data(base64Encoded: resp.value) else {
            throw X0xError.decodingError(underlying: DecodingError.dataCorrupted(
                DecodingError.Context(codingPath: [], debugDescription: "Invalid base64 in store value")
            ))
        }
        guard let decoded = String(data: data, encoding: .utf8) else {
            throw X0xError.decodingError(underlying: DecodingError.dataCorrupted(
                DecodingError.Context(codingPath: [], debugDescription: "Store value is not valid UTF-8")
            ))
        }
        return decoded
    }

    /// Put a value in a store. `PUT /stores/:id/:key`
    /// The x0x store API expects base64-encoded values; this method encodes them.
    public func storePut(storeId: String, key: String, value: String, contentType: String? = nil) async throws {
        let base64Value = Data(value.utf8).base64EncodedString()
        let body = StorePutRequest(value: base64Value, contentType: contentType ?? "application/json")
        let _: Empty = try await put("/stores/\(storeId)/\(key)", body: body)
    }

    /// Delete a key from a store. `DELETE /stores/:id/:key`
    public func storeDelete(storeId: String, key: String) async throws {
        let _: Empty = try await delete("/stores/\(storeId)/\(key)")
    }

    // MARK: - Private Helpers

    private func get<T: Decodable>(_ path: String) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        let (data, response) = try await performRequest(URLRequest(url: url))
        return try decodeFlatResponse(data: data, response: response)
    }

    private func post<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        do {
            request.httpBody = try JSONEncoder().encode(body)
        } catch {
            throw X0xError.encodingError(underlying: error)
        }

        let (data, response) = try await performRequest(request)
        return try decodeFlatResponse(data: data, response: response)
    }

    private func put<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "PUT"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        do {
            request.httpBody = try JSONEncoder().encode(body)
        } catch {
            throw X0xError.encodingError(underlying: error)
        }

        let (data, response) = try await performRequest(request)
        return try decodeFlatResponse(data: data, response: response)
    }

    private func patch<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "PATCH"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        do {
            request.httpBody = try JSONEncoder().encode(body)
        } catch {
            throw X0xError.encodingError(underlying: error)
        }

        let (data, response) = try await performRequest(request)
        return try decodeFlatResponse(data: data, response: response)
    }

    private func delete<T: Decodable>(_ path: String) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"

        let (data, response) = try await performRequest(request)
        return try decodeFlatResponse(data: data, response: response)
    }

    private func performRequest(_ request: URLRequest) async throws -> (Data, URLResponse) {
        var authenticatedRequest = request
        if let token {
            authenticatedRequest.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        do {
            return try await session.data(for: authenticatedRequest)
        } catch {
            throw X0xError.daemonUnreachable
        }
    }

    /// Decode a **flattened** x0x API response.
    ///
    /// The x0x API does NOT use a universal `{"ok":true,"data":{...}}` envelope.
    /// Instead, responses are flat: `{"ok":true,"status":"healthy","version":"0.10.0",...}`.
    /// We first check the HTTP status, then probe `ok`, then decode the full payload as T.
    private func decodeFlatResponse<T: Decodable>(data: Data, response: URLResponse) throws -> T {
        if let httpResponse = response as? HTTPURLResponse,
           !(200 ... 299).contains(httpResponse.statusCode) {
            let body = String(data: data, encoding: .utf8) ?? "<binary>"
            throw X0xError.httpError(statusCode: httpResponse.statusCode, body: body)
        }

        // Check `ok` field for API-level errors
        if let envelope = try? decoder.decode(ApiEnvelope.self, from: data) {
            if !envelope.ok {
                throw X0xError.apiError(message: envelope.error ?? "Unknown error")
            }
        }

        // For Empty responses, a simple `{"ok": true}` is sufficient
        if T.self == Empty.self {
            return Empty() as! T // swiftlint:disable:this force_cast
        }

        // Decode the full flat response as T
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            throw X0xError.decodingError(underlying: error)
        }
    }
}

// MARK: - Store Keys Response

struct StoreKeyEntry: Codable {
    let key: String
    let contentType: String?
    let contentHash: String?
    let size: UInt64?
    let updatedAt: UInt64?

    enum CodingKeys: String, CodingKey {
        case key
        case contentType = "content_type"
        case contentHash = "content_hash"
        case size
        case updatedAt = "updated_at"
    }
}

struct StoreKeysResponse: Codable {
    let ok: Bool?
    let keys: [StoreKeyEntry]
}

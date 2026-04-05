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

    /// Gracefully shut down the daemon. `POST /shutdown`
    public func shutdown() async throws {
        let _: Empty = try await post("/shutdown", body: Empty())
    }

    /// Get the agent identity. `GET /agent`
    public func agent() async throws -> AgentIdentity {
        try await get("/agent")
    }

    /// Get the user identity binding, if configured. `GET /agent/user-id`
    public func agentUserId() async throws -> String? {
        let resp: UserIdStatus = try await get("/agent/user-id")
        return resp.userId
    }

    /// Generate a shareable agent card. `GET /agent/card`
    /// - Parameters:
    ///   - displayName: Display name to include in the card.
    ///   - includeGroups: Whether to include group invites in the card.
    public func agentCard(displayName: String? = nil, includeGroups: Bool? = nil) async throws -> AgentCardResponse {
        var queryItems: [URLQueryItem] = []
        if let displayName {
            queryItems.append(URLQueryItem(name: "display_name", value: displayName))
        }
        if let includeGroups {
            queryItems.append(URLQueryItem(name: "include_groups", value: includeGroups ? "true" : "false"))
        }
        return try await get("/agent/card", queryItems: queryItems)
    }

    /// Import an agent card to contacts. `POST /agent/card/import`
    public func importAgentCard(card: String, trustLevel: TrustLevel? = nil) async throws -> ImportCardResponse {
        let body = ImportCardRequest(card: card, trustLevel: trustLevel)
        return try await post("/agent/card/import", body: body)
    }

    /// Re-announce identity to the network with default flags. `POST /announce`
    public func announce() async throws {
        let _: Empty = try await post("/announce", body: Empty())
    }

    /// Re-announce identity with explicit consent flags. `POST /announce`
    public func announceWithOptions(includeUserIdentity: Bool, humanConsent: Bool) async throws {
        let body = AnnounceRequest(includeUserIdentity: includeUserIdentity, humanConsent: humanConsent)
        let _: Empty = try await post("/announce", body: body)
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

    /// Unsubscribe from a topic by subscription ID. `DELETE /subscribe/:id`
    public func unsubscribe(subscriptionId: String) async throws {
        let _: Empty = try await delete("/subscribe/\(subscriptionId)")
    }

    // MARK: - Direct Messaging

    /// Send a direct message to an agent. `POST /direct/send`
    public func sendDirect(agentId: String, payload: String) async throws {
        let body = DirectMessageRequest(agentId: agentId, payload: payload)
        let _: Empty = try await post("/direct/send", body: body)
    }

    /// List active direct connections. `GET /direct/connections`
    public func directConnections() async throws -> [DirectConnection] {
        let resp: DirectConnectionList = try await get("/direct/connections")
        return resp.connections
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

    /// Quick trust-level update. `POST /contacts/trust`
    public func setTrust(agentId: String, level: TrustLevel) async throws {
        let body = SetTrustRequest(agentId: agentId, level: level)
        let _: Empty = try await post("/contacts/trust", body: body)
    }

    /// Update a contact's trust level and/or identity type. `PATCH /contacts/:agent_id`
    public func updateContact(agentId: String, trustLevel: TrustLevel? = nil, identityType: String? = nil) async throws {
        let body = UpdateContactRequest(trustLevel: trustLevel, identityType: identityType)
        let _: Empty = try await patch("/contacts/\(agentId)", body: body)
    }

    /// Remove a contact. `DELETE /contacts/:agent_id`
    public func removeContact(agentId: String) async throws {
        let _: Empty = try await delete("/contacts/\(agentId)")
    }

    /// Revoke a contact relationship. `POST /contacts/:agent_id/revoke`
    public func revokeContact(agentId: String, reason: String) async throws {
        let body = ["reason": reason]
        let _: Empty = try await post("/contacts/\(agentId)/revoke", body: body)
    }

    /// List revocations for a contact. `GET /contacts/:agent_id/revocations`
    public func revocations(agentId: String) async throws -> [Revocation] {
        let resp: RevocationList = try await get("/contacts/\(agentId)/revocations")
        return resp.revocations
    }

    // MARK: - Machine Management

    /// Add a machine record to a contact. `POST /contacts/:agent_id/machines`
    public func addMachine(agentId: String, machineId: String, label: String? = nil, pinned: Bool? = nil) async throws {
        let body = AddMachineRequest(machineId: machineId, label: label, pinned: pinned)
        let _: Empty = try await post("/contacts/\(agentId)/machines", body: body)
    }

    /// Remove a machine record from a contact. `DELETE /contacts/:agent_id/machines/:machine_id`
    public func removeMachine(agentId: String, machineId: String) async throws {
        let _: Empty = try await delete("/contacts/\(agentId)/machines/\(machineId)")
    }

    /// Pin a contact to a specific machine. `POST /contacts/:agent_id/machines/:machine_id/pin`
    public func pinMachine(agentId: String, machineId: String) async throws {
        let _: Empty = try await post("/contacts/\(agentId)/machines/\(machineId)/pin", body: Empty())
    }

    /// Remove a machine pin from a contact. `DELETE /contacts/:agent_id/machines/:machine_id/pin`
    public func unpinMachine(agentId: String, machineId: String) async throws {
        let _: Empty = try await delete("/contacts/\(agentId)/machines/\(machineId)/pin")
    }

    // MARK: - Trust Evaluation

    /// Evaluate trust for an agent/machine pair. `POST /trust/evaluate`
    public func evaluateTrust(agentId: String, machineId: String) async throws -> TrustEvaluation {
        let body = EvaluateTrustRequest(agentId: agentId, machineId: machineId)
        return try await post("/trust/evaluate", body: body)
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

    /// Get details for a specific discovered agent. `GET /agents/discovered/:agent_id`
    public func discoveredAgent(agentId: String) async throws -> DiscoveredAgent {
        let resp: DiscoveredAgentWrapper = try await get("/agents/discovered/\(agentId)")
        return resp.toDiscoveredAgent()
    }

    /// Fetch online presence. `GET /presence`
    /// Returns wrapped: `{"ok":true,"agents":["agentId1","agentId2"]}`
    public func presence() async throws -> [String] {
        let resp: PresenceResponse = try await get("/presence")
        return resp.agents
    }

    /// Get bootstrap peer cache status. `GET /network/bootstrap-cache`
    public func bootstrapCache() async throws -> BootstrapCacheStatus {
        try await get("/network/bootstrap-cache")
    }

    /// List machines associated with a contact. `GET /contacts/:agent_id/machines`
    /// Returns wrapped: `{"ok":true,"machines":[...]}`
    public func listMachines(agentId: String) async throws -> [MachineRecord] {
        let resp: MachineListResponse = try await get("/contacts/\(agentId)/machines")
        return resp.machines
    }

    // MARK: - File Transfer

    /// Initiate a file send. `POST /files/send`
    /// - Parameters:
    ///   - agentId: Destination agent ID (hex).
    ///   - filename: Original filename.
    ///   - size: File size in bytes.
    ///   - sha256: SHA-256 hash of the file (required by daemon).
    ///   - path: Optional local file path for the daemon to read from.
    public func sendFile(agentId: String, filename: String, size: UInt64, sha256: String, path: String? = nil) async throws -> String {
        let body = SendFileRequest(agentId: agentId, filename: filename, size: size, sha256: sha256, path: path)
        let resp: SendFileResponse = try await post("/files/send", body: body)
        return resp.transferId
    }

    /// List active file transfers. `GET /files/transfers`
    /// Returns wrapped: `{"ok":true,"transfers":[...]}`
    public func listTransfers() async throws -> [FileTransfer] {
        let resp: FileTransferListResponse = try await get("/files/transfers")
        return resp.transfers
    }

    /// Get status of a specific file transfer. `GET /files/transfers/:id`
    public func transferStatus(transferId: String) async throws -> FileTransfer {
        let resp: FileTransferWrapper = try await get("/files/transfers/\(transferId)")
        return FileTransfer(
            transferId: resp.transferId ?? transferId,
            direction: resp.direction ?? .sending,
            remoteAgentId: resp.remoteAgentId ?? "",
            filename: resp.filename ?? "",
            totalSize: resp.totalSize ?? 0,
            bytesTransferred: resp.bytesTransferred ?? 0,
            status: resp.status ?? .pending,
            sha256: resp.sha256,
            error: resp.error,
            startedAt: resp.startedAt
        )
    }

    /// Accept an incoming file transfer. `POST /files/accept/:id`
    public func acceptFile(transferId: String) async throws {
        let _: Empty = try await post("/files/accept/\(transferId)", body: Empty())
    }

    /// Reject an incoming file transfer. `POST /files/reject/:id`
    /// - Parameter reason: Optional reason for rejection.
    public func rejectFile(transferId: String, reason: String? = nil) async throws {
        if let reason {
            let body = RejectFileRequest(reason: reason)
            let _: Empty = try await post("/files/reject/\(transferId)", body: body)
        } else {
            let _: Empty = try await post("/files/reject/\(transferId)", body: Empty())
        }
    }

    // MARK: - Task Lists

    /// List all task lists. `GET /task-lists`
    public func listTaskLists() async throws -> [TaskListSummary] {
        let resp: TaskListIndex = try await get("/task-lists")
        return resp.taskLists
    }

    /// Create a new task list. `POST /task-lists`
    public func createTaskList(name: String, topic: String) async throws -> String {
        let body = CreateTaskListRequest(name: name, topic: topic)
        let resp: CreateTaskListResponse = try await post("/task-lists", body: body)
        return resp.id
    }

    /// List tasks in a task list. `GET /task-lists/:id/tasks`
    /// Returns wrapped: `{"ok":true,"tasks":[...]}`
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
    public func joinStore(storeId: String) async throws {
        let _: Empty = try await post("/stores/\(storeId)/join", body: Empty())
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

    // MARK: - MLS Groups (Encrypted)

    /// Create an MLS encrypted group. `POST /mls/groups`
    /// - Parameter groupId: Optional hex group ID. Random if omitted.
    public func createMlsGroup(groupId: String? = nil) async throws -> MlsGroup {
        let body = CreateMlsGroupRequest(groupId: groupId)
        return try await post("/mls/groups", body: body)
    }

    /// List all MLS groups. `GET /mls/groups`
    public func listMlsGroups() async throws -> [MlsGroup] {
        let resp: MlsGroupList = try await get("/mls/groups")
        return resp.groups
    }

    /// Get details for an MLS group. `GET /mls/groups/:id`
    public func getMlsGroup(groupId: String) async throws -> MlsGroup {
        try await get("/mls/groups/\(groupId)")
    }

    /// Add a member to an MLS group. `POST /mls/groups/:id/members`
    public func addMlsMember(groupId: String, agentId: String) async throws -> AddMlsMemberResponse {
        let body = AddMlsMemberRequest(agentId: agentId)
        return try await post("/mls/groups/\(groupId)/members", body: body)
    }

    /// Remove a member from an MLS group. `DELETE /mls/groups/:id/members/:agent_id`
    public func removeMlsMember(groupId: String, agentId: String) async throws {
        let _: Empty = try await delete("/mls/groups/\(groupId)/members/\(agentId)")
    }

    /// Encrypt a payload with an MLS group's current key. `POST /mls/groups/:id/encrypt`
    /// - Parameters:
    ///   - groupId: The MLS group ID.
    ///   - payload: Raw data to encrypt.
    /// - Returns: Encrypted ciphertext (base64) and the epoch used.
    public func encrypt(groupId: String, payload: Data) async throws -> EncryptResponse {
        let body = EncryptRequest(payload: payload.base64EncodedString())
        return try await post("/mls/groups/\(groupId)/encrypt", body: body)
    }

    /// Decrypt ciphertext from an MLS group. `POST /mls/groups/:id/decrypt`
    /// - Parameters:
    ///   - groupId: The MLS group ID.
    ///   - ciphertext: Base64-encoded ciphertext.
    ///   - epoch: The epoch the ciphertext was encrypted at.
    /// - Returns: Decrypted raw data.
    public func decrypt(groupId: String, ciphertext: String, epoch: UInt64) async throws -> Data {
        let body = DecryptRequest(ciphertext: ciphertext, epoch: epoch)
        let resp: DecryptResponse = try await post("/mls/groups/\(groupId)/decrypt", body: body)
        guard let data = Data(base64Encoded: resp.payload) else {
            throw X0xError.decodingError(underlying: DecodingError.dataCorrupted(
                DecodingError.Context(codingPath: [], debugDescription: "Invalid base64 in decrypted payload")
            ))
        }
        return data
    }

    /// Create a welcome message for a prospective MLS group member. `POST /mls/groups/:id/welcome`
    public func createMlsWelcome(groupId: String, agentId: String) async throws -> WelcomeResponse {
        let body = CreateWelcomeRequest(agentId: agentId)
        return try await post("/mls/groups/\(groupId)/welcome", body: body)
    }

    // MARK: - Constitution

    /// Fetch the x0x constitution with version metadata. `GET /constitution/json`
    ///
    /// This endpoint is auth-exempt — no Bearer token required.
    public func constitutionJSON() async throws -> ConstitutionInfo {
        try await get("/constitution/json")
    }

    // MARK: - Upgrade

    /// Check for daemon updates. `GET /upgrade`
    public func checkUpgrade() async throws -> UpgradeStatus {
        try await get("/upgrade")
    }

    // MARK: - WebSocket Sessions

    /// List active WebSocket sessions. `GET /ws/sessions`
    public func wsSessions() async throws -> WsSessionList {
        try await get("/ws/sessions")
    }

    // MARK: - Presence (Extended)

    /// List all currently online agents (network view, non-blocked). `GET /presence/online`
    public func presenceOnline() async throws -> [DiscoveredAgent] {
        let resp: DiscoveredAgentsResponse = try await get("/presence/online")
        return resp.agents
    }

    /// FOAF random-walk discovery of nearby agents (social view). `GET /presence/foaf`
    public func presenceFoaf(ttl: UInt32? = nil, timeoutMs: UInt64? = nil) async throws -> [DiscoveredAgent] {
        var items: [URLQueryItem] = []
        if let t = ttl { items.append(URLQueryItem(name: "ttl", value: "\(t)")) }
        if let ms = timeoutMs { items.append(URLQueryItem(name: "timeout_ms", value: "\(ms)")) }
        let resp: DiscoveredAgentsResponse = try await get("/presence/foaf", queryItems: items)
        return resp.agents
    }

    /// Find a specific agent by hex ID via FOAF random walk. `GET /presence/find/:id`
    public func presenceFind(agentId: String, ttl: UInt32? = nil, timeoutMs: UInt64? = nil) async throws -> DiscoveredAgent? {
        var items: [URLQueryItem] = []
        if let t = ttl { items.append(URLQueryItem(name: "ttl", value: "\(t)")) }
        if let ms = timeoutMs { items.append(URLQueryItem(name: "timeout_ms", value: "\(ms)")) }
        let resp: PresenceFindResponse = try await get("/presence/find/\(agentId)", queryItems: items)
        return resp.agent
    }

    /// Get local cache presence status for a specific agent. `GET /presence/status/:id`
    public func presenceStatus(agentId: String) async throws -> PresenceStatusResponse {
        try await get("/presence/status/\(agentId)")
    }

    // MARK: - Agent Discovery (Extended)

    /// Check NAT traversal reachability for an agent. `GET /agents/reachability/:agent_id`
    public func agentReachability(agentId: String) async throws -> ReachabilityInfo {
        try await get("/agents/reachability/\(agentId)")
    }

    /// Actively search for an agent (3-stage: cache → shard → rendezvous). `POST /agents/find/:agent_id`
    public func findAgent(agentId: String) async throws -> FindAgentResponse {
        try await postEmpty("/agents/find/\(agentId)")
    }

    /// Look up all agents belonging to a user ID. `GET /users/:user_id/agents`
    public func userAgents(userId: String) async throws -> [DiscoveredAgent] {
        let resp: UserAgentsResponse = try await get("/users/\(userId)/agents")
        return resp.agents
    }

    // MARK: - Private Helpers

    private func get<T: Decodable>(_ path: String) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        let (data, response) = try await performRequest(URLRequest(url: url))
        return try decodeFlatResponse(data: data, response: response)
    }

    private func get<T: Decodable>(_ path: String, queryItems: [URLQueryItem]) async throws -> T {
        guard var components = URLComponents(string: path) else {
            throw X0xError.invalidURL(path: path)
        }
        if !queryItems.isEmpty {
            components.queryItems = queryItems
        }
        guard let relative = components.string,
              let url = URL(string: relative, relativeTo: baseURL) else {
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

    private func postEmpty<T: Decodable>(_ path: String) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "POST"

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

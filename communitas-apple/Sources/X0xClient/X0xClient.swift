import Foundation

/// HTTP client for the x0x daemon REST API at `http://127.0.0.1:12700`.
public final class X0xClient: Sendable {
    /// Base URL of the x0x daemon.
    public let baseURL: URL

    private let session: URLSession
    private let decoder: JSONDecoder

    public init(baseURL: URL = URL(string: "http://127.0.0.1:12700")!) {
        self.baseURL = baseURL
        self.session = URLSession.shared
        self.decoder = JSONDecoder()
    }

    // MARK: - Health & Status

    /// Check daemon health.
    public func health() async throws -> HealthStatus {
        try await get("/health")
    }

    /// Get daemon status.
    public func status() async throws -> DaemonStatus {
        try await get("/status")
    }

    /// Get the agent identity.
    public func agent() async throws -> AgentIdentity {
        try await get("/agent")
    }

    // MARK: - Pub/Sub

    /// Publish a message to a topic. The payload should be base64-encoded.
    public func publish(topic: String, payload: String) async throws {
        let body = PublishRequest(topic: topic, payload: payload)
        let _: Empty = try await post("/pubsub/publish", body: body)
    }

    /// Subscribe to a topic. Returns the subscription ID.
    public func subscribe(topic: String) async throws -> String {
        let body = ["topic": topic]
        let resp: SubscribeResponse = try await post("/pubsub/subscribe", body: body)
        return resp.subscriptionId
    }

    // MARK: - Direct Messaging

    /// Send a direct message to an agent. Payload should be base64-encoded.
    public func sendDirect(agentId: String, payload: String) async throws {
        let body = DirectMessageRequest(agentId: agentId, payload: payload)
        let _: Empty = try await post("/direct/send", body: body)
    }

    // MARK: - Contacts

    /// List all contacts.
    public func listContacts() async throws -> [Contact] {
        try await get("/contacts")
    }

    /// Add a contact.
    public func addContact(agentId: String, trustLevel: TrustLevel, label: String?) async throws {
        let body = AddContactRequest(agentId: agentId, trustLevel: trustLevel, label: label)
        let _: Empty = try await post("/contacts", body: body)
    }

    /// Remove a contact.
    public func removeContact(agentId: String) async throws {
        let _: Empty = try await delete("/contacts/\(agentId)")
    }

    // MARK: - Groups

    /// Create a new group.
    public func createGroup(name: String, description: String?, displayName: String?) async throws -> CreatedGroup {
        let body = CreateGroupRequest(name: name, description: description, displayName: displayName)
        return try await post("/groups", body: body)
    }

    /// List all groups.
    public func listGroups() async throws -> [GroupSummary] {
        try await get("/groups")
    }

    /// Get detailed information about a group.
    public func groupInfo(groupId: String) async throws -> GroupInfo {
        try await get("/groups/\(groupId)")
    }

    /// Generate an invite for a group.
    public func invite(groupId: String, expirySecs: UInt64? = nil) async throws -> InviteResponse {
        let body = InviteRequest(groupId: groupId, expirySecs: expirySecs)
        return try await post("/groups/invite", body: body)
    }

    /// Join a group via invite token.
    public func joinGroup(invite: String, displayName: String?) async throws -> JoinGroupResponse {
        let body = JoinGroupRequest(invite: invite, displayName: displayName)
        return try await post("/groups/join", body: body)
    }

    /// Leave a group.
    public func leaveGroup(groupId: String) async throws {
        let _: Empty = try await post("/groups/\(groupId)/leave", body: Empty())
    }

    // MARK: - File Transfer

    /// Initiate a file send.
    public func sendFile(agentId: String, filename: String, size: UInt64) async throws -> String {
        let body = SendFileRequest(agentId: agentId, filename: filename, size: size)
        let resp: SendFileResponse = try await post("/files/send", body: body)
        return resp.transferId
    }

    /// List active file transfers.
    public func listTransfers() async throws -> [FileTransfer] {
        try await get("/files/transfers")
    }

    // MARK: - KV Store

    /// Get a value from the KV store.
    public func kvGet(key: String) async throws -> String {
        let resp: StoreValue = try await get("/kv/\(key)")
        return resp.value
    }

    /// Set a value in the KV store.
    public func kvSet(key: String, value: String) async throws {
        let body = KvStore(key: key, value: value)
        let _: Empty = try await post("/kv", body: body)
    }

    // MARK: - Private Helpers

    private func get<T: Decodable>(_ path: String) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        let (data, response) = try await performRequest(URLRequest(url: url))
        return try decodeEnvelope(data: data, response: response)
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
        return try decodeEnvelope(data: data, response: response)
    }

    private func delete<T: Decodable>(_ path: String) async throws -> T {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw X0xError.invalidURL(path: path)
        }

        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"

        let (data, response) = try await performRequest(request)
        return try decodeEnvelope(data: data, response: response)
    }

    private func performRequest(_ request: URLRequest) async throws -> (Data, URLResponse) {
        do {
            return try await session.data(for: request)
        } catch {
            throw X0xError.daemonUnreachable
        }
    }

    private func decodeEnvelope<T: Decodable>(data: Data, response: URLResponse) throws -> T {
        if let httpResponse = response as? HTTPURLResponse,
           !(200 ... 299).contains(httpResponse.statusCode) {
            let body = String(data: data, encoding: .utf8) ?? "<binary>"
            throw X0xError.httpError(statusCode: httpResponse.statusCode, body: body)
        }

        // Try envelope first: {"ok": true, "data": {...}}
        do {
            let envelope = try decoder.decode(ApiResponse<T>.self, from: data)
            if !envelope.ok {
                throw X0xError.apiError(message: envelope.error ?? "Unknown error")
            }
            if let payload = envelope.data {
                return payload
            }
            // For Empty responses the data field may be null
            if T.self == Empty.self {
                return Empty() as! T // swiftlint:disable:this force_cast
            }
            throw X0xError.apiError(message: "Missing data in response")
        } catch let error as X0xError {
            throw error
        } catch {
            // Fallback: try decoding directly (some endpoints may not use the envelope)
            do {
                return try decoder.decode(T.self, from: data)
            } catch let fallbackError {
                throw X0xError.decodingError(underlying: fallbackError)
            }
        }
    }
}

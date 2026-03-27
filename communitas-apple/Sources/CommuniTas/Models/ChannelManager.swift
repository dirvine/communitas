import Foundation
import X0xClient

/// Manages channels and messages for a specific group (Space).
@MainActor
final class ChannelManager: ObservableObject {
    let client: X0xClient
    let groupId: String
    let groupName: String
    private let agentId: String
    private let displayName: String

    @Published var channels: [ChannelMeta] = []
    @Published var currentChannel: String = "general"
    @Published var messages: [ChannelChatMessage] = []
    @Published var threadMessages: [String: [ChannelChatMessage]] = [:]
    @Published var unreadCounts: [String: Int] = [:]
    @Published var isLoading = false
    @Published var errorMessage: String?

    private var webSocket: X0xWebSocket?
    private var listeningTask: Task<Void, Never>?

    init(client: X0xClient, groupId: String, groupName: String, agentId: String, displayName: String = "Me") {
        self.client = client
        self.groupId = groupId
        self.groupName = groupName
        self.agentId = agentId
        self.displayName = displayName
    }

    deinit {
        listeningTask?.cancel()
        webSocket?.disconnect()
    }

    // MARK: - Topic Helpers

    /// First 16 characters of the group ID, matching the Dioxus x0x_contract convention.
    private var groupPrefix: String {
        String(groupId.prefix(16))
    }

    /// Channel store ID: `x0x-channels-{groupPrefix}`.
    private var channelStoreId: String {
        "x0x-channels-\(groupPrefix)"
    }

    private func channelTopic(name: String) -> String {
        "x0x.group.\(groupPrefix).chat/\(name)"
    }

    private func threadTopic(parentMessageId: String) -> String {
        "x0x.group.\(groupPrefix).thread/\(parentMessageId)"
    }

    /// Flat key inside the channel store (matches Dioxus `CHANNELS_INDEX_KEY`).
    private var channelIndexKey: String {
        "channels_index"
    }

    /// Per-channel metadata key inside the channel store.
    private func channelMetaKey(name: String) -> String {
        "channel:\(name)"
    }

    // MARK: - Channel Management

    /// Load the channel index and metadata from KV store.
    func loadChannels() async {
        isLoading = true
        defer { isLoading = false }

        do {
            let indexJson = try await client.kvGet(key: channelIndexKey)
            if let data = indexJson.data(using: .utf8) {
                let index = try JSONDecoder().decode(ChannelIndex.self, from: data)
                var loaded: [ChannelMeta] = []
                for channelName in index.channels {
                    if let meta = await loadChannelMeta(name: channelName) {
                        loaded.append(meta)
                    }
                }
                channels = loaded
            }
        } catch {
            // No index yet -- create default "general" channel
            await ensureDefaultChannel()
        }
    }

    private func loadChannelMeta(name: String) async -> ChannelMeta? {
        do {
            let json = try await client.kvGet(key: channelMetaKey(name: name))
            guard let data = json.data(using: .utf8) else { return nil }
            return try JSONDecoder().decode(ChannelMeta.self, from: data)
        } catch {
            return nil
        }
    }

    /// Ensure the "general" channel exists for the group.
    private func ensureDefaultChannel() async {
        let general = ChannelMeta(
            name: "general",
            description: "General discussion",
            creator: agentId,
            createdAt: Int64(Date().timeIntervalSince1970 * 1000),
            topic: channelTopic(name: "general")
        )
        do {
            try await saveChannelMeta(general)
            let index = ChannelIndex(channels: ["general"], categories: ["General": ["general"]])
            try await saveChannelIndex(index)
            channels = [general]
        } catch {
            errorMessage = "Failed to create default channel: \(error.localizedDescription)"
        }
    }

    /// Create a new channel in this group.
    func createChannel(name: String, description: String, category: String? = nil, isPrivate: Bool = false) async throws {
        let sanitized = name.lowercased().replacingOccurrences(of: " ", with: "-")

        let meta = ChannelMeta(
            name: sanitized,
            description: description,
            creator: agentId,
            createdAt: Int64(Date().timeIntervalSince1970 * 1000),
            topic: channelTopic(name: sanitized),
            isPrivate: isPrivate
        )

        try await saveChannelMeta(meta)

        // Update index
        var index = await loadChannelIndex()
        if !index.channels.contains(sanitized) {
            index.channels.append(sanitized)
        }
        let cat = category ?? "General"
        var catList = index.categories[cat] ?? []
        if !catList.contains(sanitized) {
            catList.append(sanitized)
            index.categories[cat] = catList
        }
        try await saveChannelIndex(index)

        channels.append(meta)

        // Subscribe to the new channel topic
        _ = try await client.subscribe(topic: channelTopic(name: sanitized))
    }

    private func saveChannelMeta(_ meta: ChannelMeta) async throws {
        let data = try JSONEncoder().encode(meta)
        guard let json = String(data: data, encoding: .utf8) else { return }
        try await client.kvSet(key: channelMetaKey(name: meta.name), value: json)
    }

    private func loadChannelIndex() async -> ChannelIndex {
        do {
            let json = try await client.kvGet(key: channelIndexKey)
            guard let data = json.data(using: .utf8) else { return ChannelIndex() }
            return try JSONDecoder().decode(ChannelIndex.self, from: data)
        } catch {
            return ChannelIndex()
        }
    }

    private func saveChannelIndex(_ index: ChannelIndex) async throws {
        let data = try JSONEncoder().encode(index)
        guard let json = String(data: data, encoding: .utf8) else { return }
        try await client.kvSet(key: channelIndexKey, value: json)
    }

    // MARK: - Channel Subscription

    /// Subscribe to the current channel and start listening for messages.
    func subscribeToChannel(name: String) async {
        currentChannel = name
        messages = []

        // Cancel previous listener
        listeningTask?.cancel()
        webSocket?.disconnect()

        let topic = channelTopic(name: name)
        do {
            _ = try await client.subscribe(topic: topic)
        } catch {
            errorMessage = "Failed to subscribe: \(error.localizedDescription)"
        }

        // Start WebSocket listener
        startListening()
    }

    private func startListening() {
        let ws = X0xWebSocket(path: "/ws")
        self.webSocket = ws
        ws.connect()

        listeningTask = Task { [weak self] in
            while !Task.isCancelled {
                do {
                    let text = try await ws.receive()
                    await self?.handleWebSocketMessage(text)
                } catch {
                    if !Task.isCancelled {
                        try? await Task.sleep(nanoseconds: 1_000_000_000)
                    }
                }
            }
        }
    }

    private func handleWebSocketMessage(_ text: String) async {
        guard let data = text.data(using: .utf8) else { return }

        // Try to parse as a gossip event
        struct GossipEvent: Codable {
            let event: String?
            let topic: String?
            let payload: String?
            let sender: String?
        }

        guard let event = try? JSONDecoder().decode(GossipEvent.self, from: data),
              let payload = event.payload,
              let payloadData = Data(base64Encoded: payload) else {
            return
        }

        guard let chatMsg = try? JSONDecoder().decode(ChannelChatMessage.self, from: payloadData) else {
            return
        }

        let expectedTopic = channelTopic(name: currentChannel)
        if event.topic == expectedTopic {
            if !messages.contains(where: { $0.id == chatMsg.id }) {
                messages.append(chatMsg)
                messages.sort { $0.timestamp < $1.timestamp }
            }
        }

        // Track thread replies
        if let threadRoot = chatMsg.threadRoot {
            var thread = threadMessages[threadRoot] ?? []
            if !thread.contains(where: { $0.id == chatMsg.id }) {
                thread.append(chatMsg)
                thread.sort { $0.timestamp < $1.timestamp }
                threadMessages[threadRoot] = thread
            }

            // Increment reply count on parent if present
            if let idx = messages.firstIndex(where: { $0.id == threadRoot }) {
                messages[idx].replyCount = (threadMessages[threadRoot]?.count ?? 0)
            }
        }

        // Track unread for other channels
        if let topic = event.topic, topic != channelTopic(name: currentChannel) {
            // Extract channel name from topic using group prefix
            let topicPrefix = "x0x.group.\(groupPrefix).chat/"
            if topic.hasPrefix(topicPrefix) {
                let channelName = String(topic.dropFirst(topicPrefix.count))
                unreadCounts[channelName, default: 0] += 1
            }
        }
    }

    // MARK: - Sending Messages

    /// Send a message to the current channel.
    func sendMessage(text: String) async throws {
        let msg = ChannelChatMessage(
            id: UUID().uuidString,
            text: text,
            senderName: displayName,
            senderId: agentId,
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            channel: currentChannel
        )

        let payload = try encodeMessagePayload(msg)
        try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)

        // Optimistically add to local list
        messages.append(msg)
    }

    /// Start a thread on a parent message (subscribe to thread topic).
    func startThread(parentMessageId: String) async {
        let topic = threadTopic(parentMessageId: parentMessageId)
        do {
            _ = try await client.subscribe(topic: topic)
        } catch {
            errorMessage = "Failed to subscribe to thread: \(error.localizedDescription)"
        }
    }

    /// Reply in a thread.
    func replyInThread(threadRoot: String, text: String, broadcast: Bool = false) async throws {
        let msg = ChannelChatMessage(
            id: UUID().uuidString,
            text: text,
            senderName: displayName,
            senderId: agentId,
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            channel: currentChannel,
            threadRoot: threadRoot,
            broadcast: broadcast
        )

        // Publish to thread topic
        let payload = try encodeMessagePayload(msg)
        try await client.publish(topic: threadTopic(parentMessageId: threadRoot), payload: payload)

        // If broadcast, also publish to channel topic
        if broadcast {
            try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
        }

        // Optimistically add to thread messages
        var thread = threadMessages[threadRoot] ?? []
        thread.append(msg)
        threadMessages[threadRoot] = thread

        // Update reply count on parent
        if let idx = messages.firstIndex(where: { $0.id == threadRoot }) {
            messages[idx].replyCount = thread.count
        }
    }

    /// Load thread messages for a parent message.
    func loadThread(parentMessageId: String) async -> [ChannelChatMessage] {
        await startThread(parentMessageId: parentMessageId)
        return threadMessages[parentMessageId] ?? []
    }

    // MARK: - Payload Encoding

    private func encodeMessagePayload(_ msg: ChannelChatMessage) throws -> String {
        let data = try JSONEncoder().encode(msg)
        return data.base64EncodedString()
    }

    // MARK: - Cleanup

    func disconnect() {
        listeningTask?.cancel()
        webSocket?.disconnect()
    }
}

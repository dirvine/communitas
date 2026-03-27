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

    init(client: X0xClient, groupId: String, groupName: String, agentId: String, displayName: String) {
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

    // MARK: - Store Management

    /// Ensure the channel store exists, creating it if necessary.
    private func ensureStore() async {
        do {
            let stores = try await client.listStores()
            if !stores.contains(where: { $0.storeId == channelStoreId || $0.name == channelStoreId }) {
                _ = try await client.createStore(name: channelStoreId, topic: "x0x.group.\(groupPrefix).meta")
            }
        } catch { /* store may already exist */ }
    }

    // MARK: - Channel Management

    /// Load the channel index from the channel store.
    /// The index is a JSON array of `ChannelMeta` (matching the Dioxus schema).
    func loadChannels() async {
        isLoading = true
        defer { isLoading = false }

        await ensureStore()

        do {
            let indexJson = try await client.storeGet(storeId: channelStoreId, key: channelIndexKey)
            if let data = indexJson.data(using: .utf8) {
                var loaded = try JSONDecoder().decode(ChannelIndex.self, from: data)
                // Ensure "general" exists
                if !loaded.contains(where: { $0.name == "general" }) {
                    let general = makeGeneralChannel()
                    loaded.insert(general, at: 0)
                    try? await saveChannelIndex(loaded)
                }
                channels = loaded
            }
        } catch {
            // No index yet -- create default "general" channel
            await ensureDefaultChannel()
        }
    }

    private func makeGeneralChannel() -> ChannelMeta {
        ChannelMeta(
            name: "general",
            description: "General discussion",
            creator: agentId,
            createdAt: UInt64(Date().timeIntervalSince1970 * 1000),
            topic: channelTopic(name: "general")
        )
    }

    /// Ensure the "general" channel exists for the group.
    private func ensureDefaultChannel() async {
        let general = makeGeneralChannel()
        do {
            let index: ChannelIndex = [general]
            try await saveChannelIndex(index)
            channels = index
        } catch {
            errorMessage = "Failed to create default channel: \(error.localizedDescription)"
        }
    }

    /// Create a new channel in this group.
    func createChannel(name: String, description: String, category: String? = nil) async throws {
        let sanitized = name.lowercased().replacingOccurrences(of: " ", with: "-")

        let meta = ChannelMeta(
            name: sanitized,
            description: description,
            creator: agentId,
            createdAt: UInt64(Date().timeIntervalSince1970 * 1000),
            topic: channelTopic(name: sanitized)
        )

        // Update the channel index (array of ChannelMeta)
        var index = await loadChannelIndex()
        if !index.contains(where: { $0.name == sanitized }) {
            index.append(meta)
        }
        try await saveChannelIndex(index)

        channels.append(meta)

        // Subscribe to the new channel topic
        _ = try await client.subscribe(topic: channelTopic(name: sanitized))
    }

    private func loadChannelIndex() async -> ChannelIndex {
        do {
            let json = try await client.storeGet(storeId: channelStoreId, key: channelIndexKey)
            guard let data = json.data(using: .utf8) else { return [] }
            return try JSONDecoder().decode(ChannelIndex.self, from: data)
        } catch {
            return []
        }
    }

    private func saveChannelIndex(_ index: ChannelIndex) async throws {
        let data = try JSONEncoder().encode(index)
        guard let json = String(data: data, encoding: .utf8) else { return }
        try await client.storePut(storeId: channelStoreId, key: channelIndexKey, value: json)
    }

    // MARK: - History Persistence

    private func historyKey(channel: String) -> String {
        "channel_history_\(groupPrefix)_\(channel)"
    }

    private func threadHistoryKey(threadRoot: String) -> String {
        "thread_history_\(groupPrefix)_\(threadRoot)"
    }

    func loadHistory(channel: String) -> [ChannelChatMessage] {
        guard let data = UserDefaults.standard.data(forKey: historyKey(channel: channel)),
              let messages = try? JSONDecoder().decode([ChannelChatMessage].self, from: data) else { return [] }
        return messages
    }

    func saveHistory(channel: String, messages: [ChannelChatMessage]) {
        let trimmed = Array(messages.suffix(200))
        if let data = try? JSONEncoder().encode(trimmed) {
            UserDefaults.standard.set(data, forKey: historyKey(channel: channel))
        }
    }

    private func loadThreadHistory(threadRoot: String) -> [ChannelChatMessage] {
        guard let data = UserDefaults.standard.data(forKey: threadHistoryKey(threadRoot: threadRoot)),
              let msgs = try? JSONDecoder().decode([ChannelChatMessage].self, from: data) else { return [] }
        return msgs
    }

    private func saveThreadHistory(threadRoot: String, messages: [ChannelChatMessage]) {
        let trimmed = Array(messages.suffix(200))
        if let data = try? JSONEncoder().encode(trimmed) {
            UserDefaults.standard.set(data, forKey: threadHistoryKey(threadRoot: threadRoot))
        }
    }

    // MARK: - Channel Subscription

    /// Subscribe to the current channel and start listening for messages.
    func subscribeToChannel(name: String) async {
        currentChannel = name
        messages = loadHistory(channel: name)

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
                saveHistory(channel: currentChannel, messages: messages)
            }
        }

        // Track thread replies
        if let threadRoot = chatMsg.threadRoot {
            var thread = threadMessages[threadRoot] ?? []
            if !thread.contains(where: { $0.id == chatMsg.id }) {
                thread.append(chatMsg)
                thread.sort { $0.timestamp < $1.timestamp }
                threadMessages[threadRoot] = thread
                saveThreadHistory(threadRoot: threadRoot, messages: thread)
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
        saveHistory(channel: currentChannel, messages: messages)
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
        saveThreadHistory(threadRoot: threadRoot, messages: thread)

        // Update reply count on parent
        if let idx = messages.firstIndex(where: { $0.id == threadRoot }) {
            messages[idx].replyCount = thread.count
        }
    }

    /// Load thread messages for a parent message.
    func loadThread(parentMessageId: String) async -> [ChannelChatMessage] {
        // Load persisted thread history if not already in memory
        if threadMessages[parentMessageId] == nil || threadMessages[parentMessageId]?.isEmpty == true {
            let persisted = loadThreadHistory(threadRoot: parentMessageId)
            if !persisted.isEmpty {
                threadMessages[parentMessageId] = persisted
            }
        }
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

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

    /// Pinned message IDs for the current channel (loaded from KV store).
    @Published var pinnedMessageIds: [String] = []

    /// Maximum number of pinned messages per channel.
    private let maxPins = 25

    /// Typing users: keyed by senderId, value is (name, lastSeen).
    @Published var typingUsers: [String: (name: String, lastSeen: Date)] = [:]

    /// Tracks (messageId:emoji:senderId) triples we have seen to prevent duplicate reactions.
    private var seenReactions: Set<String> = []

    private var webSocket: X0xWebSocket?
    private var listeningTask: Task<Void, Never>?

    /// Last time a typing event was sent (for throttling to max 1 per 2 seconds).
    private var lastTypingSent: Date?

    /// Task for expiring stale typing users (3-second polling loop).
    /// Using a Task instead of Timer avoids non-Sendable RunLoop timer threading concerns.
    private var typingCleanupTask: Task<Void, Never>?

    /// Confidentiality of the underlying group, fetched lazily from
    /// `GET /groups/:id`. SignedPublic groups are routed through
    /// `POST /groups/:id/send` so the daemon ML-DSA-signs the body
    /// and binds it to the current state-hash. MlsEncrypted (or
    /// unknown / legacy daemon) keeps the existing gossip path.
    @Published private(set) var confidentiality: GroupConfidentiality = .mlsEncrypted

    init(client: X0xClient, groupId: String, groupName: String, agentId: String, displayName: String) {
        self.client = client
        self.groupId = groupId
        self.groupName = groupName
        self.agentId = agentId
        self.displayName = displayName
        Task { [weak self] in
            await self?.refreshConfidentiality()
        }
    }

    /// Refresh the cached `confidentiality` from the daemon. Idempotent
    /// and safe to call repeatedly. Errors and missing-policy
    /// responses leave the field at its current value.
    func refreshConfidentiality() async {
        if let info = try? await client.groupInfo(groupId: groupId),
           let policy = info.policy {
            self.confidentiality = policy.confidentiality
        }
    }

    deinit {
        listeningTask?.cancel()
        typingCleanupTask?.cancel()
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
            if !stores.contains(where: { $0.id == channelStoreId }) {
                _ = try await client.createStore(name: channelStoreId, topic: channelStoreId)
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
    func createChannel(name: String, description: String) async throws {
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

    // MARK: - Pinned Messages

    private func pinnedMessagesKey(channel: String) -> String {
        "pinned_messages_\(channel)"
    }

    /// Load pinned message IDs for the given channel from the KV store.
    func loadPinnedMessages(channel: String) async {
        do {
            let json = try await client.storeGet(storeId: channelStoreId, key: pinnedMessagesKey(channel: channel))
            guard let data = json.data(using: .utf8),
                  let ids = try? JSONDecoder().decode([String].self, from: data) else { return }
            pinnedMessageIds = ids
        } catch {
            pinnedMessageIds = []
        }
    }

    private func savePinnedMessages(channel: String, ids: [String]) async throws {
        let data = try JSONEncoder().encode(ids)
        guard let json = String(data: data, encoding: .utf8) else { return }
        try await client.storePut(storeId: channelStoreId, key: pinnedMessagesKey(channel: channel), value: json)
    }

    /// Pin a message in the current channel. Enforces 25-pin limit.
    func pinMessage(messageId: String) async {
        guard !pinnedMessageIds.contains(messageId) else { return }
        guard pinnedMessageIds.count < maxPins else {
            errorMessage = "Cannot pin more than \(maxPins) messages per channel."
            return
        }
        var updated = pinnedMessageIds
        updated.append(messageId)
        do {
            try await savePinnedMessages(channel: currentChannel, ids: updated)
            pinnedMessageIds = updated
            let event = PinEvent(messageId: messageId, action: "pin", senderId: agentId)
            let payload = try JSONEncoder().encode(event).base64EncodedString()
            try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
        } catch {
            errorMessage = "Failed to pin message: \(error.localizedDescription)"
        }
    }

    /// Unpin a message in the current channel.
    func unpinMessage(messageId: String) async {
        var updated = pinnedMessageIds
        updated.removeAll { $0 == messageId }
        do {
            try await savePinnedMessages(channel: currentChannel, ids: updated)
            pinnedMessageIds = updated
            let event = PinEvent(messageId: messageId, action: "unpin", senderId: agentId)
            let payload = try JSONEncoder().encode(event).base64EncodedString()
            try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
        } catch {
            errorMessage = "Failed to unpin message: \(error.localizedDescription)"
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

        // Load pinned messages for the new channel
        await loadPinnedMessages(channel: name)

        // Start WebSocket listener
        startListening()
    }

    private func startListening() {
        let ws = X0xWebSocket(baseURL: client.webSocketBaseURL, path: "/ws", token: client.token)
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

        // Peek at the `type` field to distinguish reaction/typing events from chat messages
        struct TypePeek: Codable { let type: String? }
        let peeked = try? JSONDecoder().decode(TypePeek.self, from: payloadData)
        if peeked?.type == "reaction" {
            if let reaction = try? JSONDecoder().decode(ReactionEvent.self, from: payloadData) {
                applyReactionEvent(reaction)
            }
            return
        }
        if peeked?.type == "typing" {
            if let typingEvent = try? JSONDecoder().decode(TypingEvent.self, from: payloadData) {
                // Ignore own typing events
                guard typingEvent.senderId != agentId else { return }
                typingUsers[typingEvent.senderId] = (name: typingEvent.senderName, lastSeen: Date())
                scheduleTypingCleanup()
            }
            return
        }
        if peeked?.type == "pin" {
            if let pinEvent = try? JSONDecoder().decode(PinEvent.self, from: payloadData) {
                applyPinEvent(pinEvent)
            }
            return
        }

        guard let chatMsg = try? JSONDecoder().decode(ChannelChatMessage.self, from: payloadData) else {
            return
        }

        // Handle edit/delete events
        switch chatMsg.type {
        case .edit:
            if let targetId = chatMsg.messageId {
                applyEdit(messageId: targetId, newText: chatMsg.text, editedAt: chatMsg.timestamp)
            }
            return
        case .delete:
            if let targetId = chatMsg.messageId {
                applyDelete(messageId: targetId)
            }
            return
        case .message:
            break
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

    // MARK: - Edit / Delete helpers

    private func applyEdit(messageId: String, newText: String, editedAt: Int64) {
        if let idx = messages.firstIndex(where: { $0.id == messageId }) {
            messages[idx].text = newText
            messages[idx].editedAt = editedAt
            saveHistory(channel: currentChannel, messages: messages)
        }
    }

    private func applyDelete(messageId: String) {
        if let idx = messages.firstIndex(where: { $0.id == messageId }) {
            messages[idx].isDeleted = true
            saveHistory(channel: currentChannel, messages: messages)
        }
    }

    private func applyPinEvent(_ pin: PinEvent) {
        if pin.action == "pin" {
            guard !pinnedMessageIds.contains(pin.messageId) else { return }
            if pinnedMessageIds.count < maxPins {
                pinnedMessageIds.append(pin.messageId)
            }
        } else {
            pinnedMessageIds.removeAll { $0 == pin.messageId }
        }
        Task { [weak self] in
            guard let self else { return }
            try? await savePinnedMessages(channel: currentChannel, ids: pinnedMessageIds)
        }
    }

    // MARK: - Reactions

    /// Returns `true` if the given agent has already reacted with this emoji on this message.
    func hasReacted(emoji: String, messageId: String, agentId: String) -> Bool {
        let dedupKey = "\(messageId):\(emoji):\(agentId)"
        return seenReactions.contains(dedupKey)
    }

    /// Publish an emoji reaction (add or remove) and optimistically update local state.
    func sendReaction(emoji: String, messageId: String, action: ReactionAction) async throws {
        let event = ReactionEvent(
            messageId: messageId,
            emoji: emoji,
            action: action,
            senderId: agentId,
            senderName: displayName
        )
        let data = try JSONEncoder().encode(event)
        let payload = data.base64EncodedString()
        try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
        // Optimistic local update using same dedup logic
        applyReactionEvent(event)
    }

    /// Apply a reaction event to local message state with per-sender deduplication.
    private func applyReactionEvent(_ reaction: ReactionEvent) {
        let dedupKey = "\(reaction.messageId):\(reaction.emoji):\(reaction.senderId)"
        if reaction.action == .add {
            guard !seenReactions.contains(dedupKey) else { return }
            seenReactions.insert(dedupKey)
            if let idx = messages.firstIndex(where: { $0.id == reaction.messageId }) {
                messages[idx].reactions[reaction.emoji, default: 0] += 1
                saveHistory(channel: currentChannel, messages: messages)
            }
        } else {
            guard seenReactions.contains(dedupKey) else { return }
            seenReactions.remove(dedupKey)
            if let idx = messages.firstIndex(where: { $0.id == reaction.messageId }) {
                let current = messages[idx].reactions[reaction.emoji, default: 0]
                if current <= 1 {
                    messages[idx].reactions.removeValue(forKey: reaction.emoji)
                } else {
                    messages[idx].reactions[reaction.emoji] = current - 1
                }
                saveHistory(channel: currentChannel, messages: messages)
            }
        }
    }

    // MARK: - Sending Messages

    /// Send a message to the current channel, optionally quoting/replying to another message.
    ///
    /// Routes based on the group's confidentiality (Phase E):
    /// - `SignedPublic` → `POST /groups/:id/send` so the daemon
    ///   ML-DSA-signs the body and binds it to the current state-hash.
    ///   The daemon publishes the signed envelope on its own public
    ///   topic; we do not also publish to the gossip channel topic
    ///   for these groups.
    /// - `MlsEncrypted` (default) → existing gossip path, unchanged.
    func sendMessage(text: String, replyToId: String? = nil) async throws {
        let msg = ChannelChatMessage(
            id: UUID().uuidString,
            text: text,
            senderName: displayName,
            senderId: agentId,
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            channel: currentChannel,
            replyToId: replyToId
        )

        switch confidentiality {
        case .signedPublic:
            _ = try await client.sendGroupPublicMessage(
                groupId: groupId,
                body: text,
                kind: "chat"
            )
        case .mlsEncrypted:
            let payload = try encodeMessagePayload(msg)
            try await client.publish(
                topic: channelTopic(name: currentChannel),
                payload: payload
            )
        }

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

    /// Publish an edit event for a message the current user sent.
    func editMessage(messageId: String, newText: String) async throws {
        let now = Int64(Date().timeIntervalSince1970 * 1000)
        let event = ChannelChatMessage(
            id: UUID().uuidString,
            text: newText,
            senderName: displayName,
            senderId: agentId,
            timestamp: now,
            channel: currentChannel,
            messageId: messageId,
            type: .edit,
            editedAt: now
        )
        let payload = try encodeMessagePayload(event)
        try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
        // Apply locally
        applyEdit(messageId: messageId, newText: newText, editedAt: now)
    }

    /// Publish a delete event for a message the current user sent.
    func deleteMessage(messageId: String) async throws {
        let now = Int64(Date().timeIntervalSince1970 * 1000)
        let event = ChannelChatMessage(
            id: UUID().uuidString,
            text: "",
            senderName: displayName,
            senderId: agentId,
            timestamp: now,
            channel: currentChannel,
            messageId: messageId,
            type: .delete
        )
        let payload = try encodeMessagePayload(event)
        try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
        // Apply locally
        applyDelete(messageId: messageId)
    }

    // MARK: - Payload Encoding

    private func encodeMessagePayload(_ msg: ChannelChatMessage) throws -> String {
        let data = try JSONEncoder().encode(msg)
        return data.base64EncodedString()
    }

    // MARK: - Typing Indicators

    /// Publish a typing event to the current channel (throttled to max 1 per 2 seconds).
    func sendTypingEvent() {
        let now = Date()
        if let last = lastTypingSent, now.timeIntervalSince(last) < 2.0 {
            return
        }
        lastTypingSent = now

        let event = TypingEvent(
            id: UUID().uuidString,
            senderId: agentId,
            senderName: displayName,
            timestamp: Int64(now.timeIntervalSince1970 * 1000)
        )

        Task { [weak self] in
            guard let self else { return }
            do {
                let data = try JSONEncoder().encode(event)
                let payload = data.base64EncodedString()
                try await client.publish(topic: channelTopic(name: currentChannel), payload: payload)
            } catch {
                // Typing events are ephemeral — silently ignore publish failures
            }
        }
    }

    /// Start (or restart) a Task-based cleanup loop that expires stale typing users.
    /// Using a Task instead of Timer avoids RunLoop thread-safety concerns with NSTimer.
    private func scheduleTypingCleanup() {
        // Only start a new cleanup loop if one isn't already running.
        guard typingCleanupTask == nil else { return }
        typingCleanupTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 3_000_000_000)
                guard !Task.isCancelled, let self else { break }
                let cutoff = Date().addingTimeInterval(-3.0)
                self.typingUsers = self.typingUsers.filter { $0.value.lastSeen > cutoff }
                if self.typingUsers.isEmpty {
                    self.typingCleanupTask = nil
                    break
                }
            }
        }
    }

    // MARK: - Cleanup

    func disconnect() {
        listeningTask?.cancel()
        typingCleanupTask?.cancel()
        typingCleanupTask = nil
        webSocket?.disconnect()
    }
}

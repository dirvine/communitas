import SwiftUI
import X0xClient

/// Full channel chat view with header, message list, and composer.
struct MessagingView: View {
    @EnvironmentObject var appState: AppState
    @State private var draft = ""
    @State private var isSending = false
    @State private var selectedThreadMessage: ChannelChatMessage?
    @State private var replyingTo: ChannelChatMessage?
    @State private var showPinnedPanel = false

    // MARK: - Search State (Phase 2.7)
    @State private var isSearching = false
    @State private var searchQuery = ""
    @State private var searchDebounceTask: Task<Void, Never>?
    @State private var searchResults: [ChannelChatMessage] = []
    @State private var scrollToMessageId: String?

    var body: some View {
        Group {
            if let group = appState.selectedGroup,
               let channelName = appState.selectedChannel {
                let manager = appState.channelManager(for: group)
                channelChatView(manager: manager, channelName: channelName, group: group)
            } else {
                noChannelSelected
            }
        }
    }

    @ViewBuilder
    private func channelChatView(manager: ChannelManager, channelName: String, group: GroupSummary) -> some View {
        VStack(spacing: 0) {
            // Channel header
            channelHeader(channelName: channelName, manager: manager, group: group)

            Divider()

            // Search bar (shown when search is active)
            if isSearching {
                searchBar(manager: manager)
                Divider()
            }

            // Pinned messages banner
            if !manager.pinnedMessageIds.isEmpty {
                pinnedBanner(manager: manager)
                Divider()
            }

            // Messages
            messageList(manager: manager)

            // Typing indicator bar
            if !manager.typingUsers.isEmpty {
                typingIndicatorBar(manager: manager)
            }

            Divider()

            // Reply preview bar
            if let replying = replyingTo {
                replyPreviewBar(message: replying)
                Divider()
            }

            // Composer (with mention autocomplete overlay)
            composerWithMentions(manager: manager)
        }
        .sheet(item: $selectedThreadMessage) { message in
            ThreadView(
                parentMessage: message,
                manager: manager
            )
            .frame(minWidth: 400, minHeight: 500)
        }
        .sheet(isPresented: $showPinnedPanel) {
            PinnedMessagesPanel(manager: manager)
                .frame(minWidth: 400, minHeight: 400)
        }
        .onChange(of: isSearching) { _, searching in
            if !searching {
                searchQuery = ""
                searchResults = []
                searchDebounceTask?.cancel()
            }
        }
    }

    private func channelHeader(channelName: String, manager: ChannelManager, group: GroupSummary) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "number")
                .foregroundStyle(.secondary)
            VStack(alignment: .leading, spacing: 2) {
                Text(channelName)
                    .font(.headline)
                if let channel = manager.channels.first(where: { $0.name == channelName }) {
                    Text(channel.description)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
            Spacer()
            // Search toggle button
            Button {
                withAnimation(.easeInOut(duration: 0.2)) {
                    isSearching.toggle()
                }
            } label: {
                Image(systemName: isSearching ? "xmark.circle.fill" : "magnifyingglass")
                    .foregroundStyle(isSearching ? Color.accentColor : .secondary)
            }
            .buttonStyle(.plain)
            .help(isSearching ? "Close search" : "Search messages")

            Text(group.name)
                .font(.caption)
                .foregroundStyle(.tertiary)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(Color.secondary.opacity(0.1), in: Capsule())
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Search Bar (Phase 2.7)

    @ViewBuilder
    private func searchBar(manager: ChannelManager) -> some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                Image(systemName: "magnifyingglass")
                    .foregroundStyle(.secondary)
                    .font(.body)

                TextField("Search messages…", text: $searchQuery)
                    .textFieldStyle(.plain)
                    .onChange(of: searchQuery) { _, newValue in
                        searchDebounceTask?.cancel()
                        searchDebounceTask = Task {
                            try? await Task.sleep(nanoseconds: 300_000_000)
                            guard !Task.isCancelled else { return }
                            await performSearch(query: newValue, manager: manager)
                        }
                    }

                if !searchQuery.isEmpty {
                    Button {
                        searchQuery = ""
                        searchResults = []
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundStyle(.secondary)
                    }
                    .buttonStyle(.plain)
                }
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 8)

            // Search results overlay
            if !searchQuery.isEmpty {
                Divider()
                searchResultsList(manager: manager)
            }
        }
        .background(.bar)
    }

    @ViewBuilder
    private func searchResultsList(manager: ChannelManager) -> some View {
        if searchResults.isEmpty {
            HStack {
                Spacer()
                Text("No results")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 12)
                Spacer()
            }
        } else {
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 0) {
                    ForEach(searchResults) { message in
                        Button {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                isSearching = false
                            }
                            scrollToMessageId = message.id
                        } label: {
                            SearchResultRow(message: message, query: searchQuery)
                        }
                        .buttonStyle(.plain)
                        Divider()
                            .padding(.leading, 48)
                    }
                }
            }
            .frame(maxHeight: 220)
        }
    }

    private func performSearch(query: String, manager: ChannelManager) async {
        let trimmed = query.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else {
            searchResults = []
            return
        }
        let lower = trimmed.lowercased()
        let results = manager.messages
            .filter { msg in
                !msg.isDeleted &&
                (msg.text.lowercased().contains(lower) ||
                 msg.senderName.lowercased().contains(lower))
            }
            .sorted { $0.timestamp > $1.timestamp }
            .prefix(20)
        searchResults = Array(results)
    }

    // MARK: - Pinned Banner

    private func pinnedBanner(manager: ChannelManager) -> some View {
        Button {
            showPinnedPanel = true
        } label: {
            HStack(spacing: 6) {
                Image(systemName: "pin.fill")
                    .font(.caption)
                    .foregroundStyle(Color.accentColor)
                Text("\(manager.pinnedMessageIds.count) pinned \(manager.pinnedMessageIds.count == 1 ? "message" : "messages")")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundStyle(Color.accentColor)
                Spacer()
                Image(systemName: "chevron.right")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 7)
            .background(Color.accentColor.opacity(0.06))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }

    // MARK: - Message List

    private func messageList(manager: ChannelManager) -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 2) {
                    ForEach(manager.messages) { message in
                        MessageRow(
                            message: message,
                            allMessages: manager.messages,
                            isOwnMessage: message.senderId == (appState.agentIdentity?.agentId ?? ""),
                            currentAgentId: appState.agentIdentity?.agentId ?? "",
                            manager: manager,
                            onThreadTap: {
                                selectedThreadMessage = message
                            },
                            onReply: {
                                replyingTo = message
                            }
                        )
                        .id(message.id)
                    }
                }
                .padding(.vertical, 8)
            }
            .onChange(of: manager.messages.count) {
                if let last = manager.messages.last {
                    withAnimation(.easeOut(duration: 0.2)) {
                        proxy.scrollTo(last.id, anchor: .bottom)
                    }
                }
            }
            .onChange(of: scrollToMessageId) { _, messageId in
                if let messageId {
                    withAnimation(.easeOut(duration: 0.3)) {
                        proxy.scrollTo(messageId, anchor: .center)
                    }
                    scrollToMessageId = nil
                }
            }
        }
    }

    // MARK: - Typing Indicator Bar (Phase 2.8)

    private func typingIndicatorBar(manager: ChannelManager) -> some View {
        HStack(spacing: 6) {
            TypingDotsView()
            Text(typingIndicatorText(manager: manager))
                .font(.caption)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 4)
        .animation(.easeInOut(duration: 0.2), value: manager.typingUsers.count)
    }

    private func typingIndicatorText(manager: ChannelManager) -> String {
        let names = manager.typingUsers.values.map { $0.name }
        switch names.count {
        case 1:
            return "\(names[0]) is typing…"
        case 2:
            return "\(names[0]) and \(names[1]) are typing…"
        default:
            let first = names[0]
            let others = names.count - 1
            return "\(first) and \(others) others are typing…"
        }
    }

    // MARK: - Reply Preview Bar

    private func replyPreviewBar(message: ChannelChatMessage) -> some View {
        HStack(spacing: 8) {
            Image(systemName: "arrowshape.turn.up.left.fill")
                .font(.caption)
                .foregroundStyle(Color.accentColor)
            VStack(alignment: .leading, spacing: 1) {
                Text("Replying to \(message.senderName)")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundStyle(Color.accentColor)
                Text(message.text)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer()
            Button {
                replyingTo = nil
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 7)
        .background(Color.secondary.opacity(0.06))
    }

    // MARK: - Composer with Mention Autocomplete

    @ViewBuilder
    private func composerWithMentions(manager: ChannelManager) -> some View {
        let candidates = mentionCandidates()
        let activeQuery = activeMentionQuery()

        ZStack(alignment: .bottomLeading) {
            messageComposer(manager: manager)

            if let query = activeQuery, !candidates.isEmpty {
                VStack {
                    Spacer()
                    HStack(alignment: .bottom) {
                        MentionAutocomplete(
                            candidates: candidates,
                            query: query,
                            onSelect: { candidate in
                                insertMention(candidate: candidate)
                            }
                        )
                        .padding(.leading, 12)
                        .padding(.bottom, 64)
                        Spacer()
                    }
                }
            }
        }
    }

    private func messageComposer(manager: ChannelManager) -> some View {
        HStack(spacing: 8) {
            TextField("Message #\(appState.selectedChannel ?? "channel")...", text: $draft)
                .textFieldStyle(.plain)
                .padding(10)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
                .onSubmit { sendMessage(manager: manager) }
                .onChange(of: draft) { _, newValue in
                    // Publish typing event when draft changes (throttled inside manager)
                    if !newValue.isEmpty {
                        manager.sendTypingEvent()
                    }
                    appState.recordInteraction()
                }

            Button {
                sendMessage(manager: manager)
            } label: {
                Image(systemName: "paperplane.fill")
                    .font(.body)
                    .frame(width: 32, height: 32)
            }
            .buttonStyle(.borderedProminent)
            .clipShape(Circle())
            .disabled(draft.trimmingCharacters(in: .whitespaces).isEmpty || isSending)
            .keyboardShortcut(.return, modifiers: .command)
        }
        .padding(12)
    }

    // MARK: - Mention Helpers

    /// Returns the mention query (text after the last bare `@`) if the cursor is in an active mention.
    private func activeMentionQuery() -> String? {
        // Find the last `@` in the draft that doesn't look like a completed mention (no space before next @)
        guard let atRange = draft.range(of: "@", options: .backwards) else { return nil }
        let afterAt = String(draft[atRange.upperBound...])
        // If there's a space after the last @, the mention is complete
        if afterAt.contains(" ") { return nil }
        return afterAt
    }

    /// Returns all contacts as mention candidates.
    private func mentionCandidates() -> [MentionCandidate] {
        appState.contacts.map { contact in
            MentionCandidate(
                id: contact.agentId,
                displayName: contact.label ?? String(contact.agentId.prefix(8))
            )
        }
    }

    /// Replaces the active `@query` in the draft with `@displayName `.
    private func insertMention(candidate: MentionCandidate) {
        guard let atRange = draft.range(of: "@", options: .backwards) else { return }
        let replacement = "@\(candidate.displayName) "
        draft = String(draft[draft.startIndex..<atRange.lowerBound]) + replacement
    }

    // MARK: - Send

    private func sendMessage(manager: ChannelManager) {
        let text = draft.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }

        // Check for @mention of own display name in incoming messages (handled in ChannelManager).
        // Here we capture the replyToId before clearing state.
        let currentReplyToId = replyingTo?.id

        draft = ""
        replyingTo = nil
        isSending = true
        appState.recordInteraction()

        // Mention notification: if we send a message mentioning someone, no local notification needed.
        // Notification for receiving a mention is handled in handleWebSocketMessage.

        Task {
            defer { isSending = false }
            do {
                try await manager.sendMessage(text: text, replyToId: currentReplyToId)
            } catch {
                appState.errorMessage = error.localizedDescription
            }
        }
    }

    private var noChannelSelected: some View {
        VStack(spacing: 12) {
            Image(systemName: "bubble.left.and.text.bubble.right")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("No Channel Selected")
                .font(.title2)
            Text("Select a channel from the sidebar to start chatting.")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Message Row

struct MessageRow: View {
    let message: ChannelChatMessage
    let allMessages: [ChannelChatMessage]
    let isOwnMessage: Bool
    let currentAgentId: String
    let manager: ChannelManager
    let onThreadTap: () -> Void
    let onReply: () -> Void

    @State private var isEditing = false
    @State private var editDraft = ""
    @State private var showDeleteConfirm = false
    @State private var showQuickReactions = false
    @State private var showFullEmojiPicker = false
    @EnvironmentObject private var appState: AppState

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            // Avatar
            senderAvatar

            VStack(alignment: .leading, spacing: 4) {
                // Sender + timestamp row
                HStack(spacing: 6) {
                    Text(message.senderName)
                        .font(.subheadline)
                        .fontWeight(.semibold)

                    if isOwnMessage {
                        Text("(you)")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }

                    Text(message.date, style: .time)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)

                    if message.editedAt != nil && !message.isDeleted {
                        Text("(edited)")
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }

                    // Pinned indicator
                    if manager.pinnedMessageIds.contains(message.id) {
                        Image(systemName: "pin.fill")
                            .font(.caption2)
                            .foregroundStyle(Color.accentColor.opacity(0.8))
                    }
                }

                // Inline quote block (reply preview)
                if let replyId = message.replyToId {
                    inlineQuoteView(replyId: replyId)
                }

                // Message body
                if message.isDeleted {
                    Text("This message was deleted")
                        .font(.body)
                        .foregroundStyle(.tertiary)
                        .italic()
                } else if isEditing {
                    inlineEditView
                } else {
                    MarkdownMessageView(text: message.text)
                }

                // Reaction chips
                if !message.isDeleted && !message.reactions.isEmpty {
                    reactionChips
                }

                // Thread indicator
                if !message.isDeleted {
                    if message.replyCount > 0 {
                        Button {
                            onThreadTap()
                        } label: {
                            HStack(spacing: 4) {
                                Image(systemName: "bubble.left.and.bubble.right")
                                    .font(.caption2)
                                Text("\(message.replyCount) \(message.replyCount == 1 ? "reply" : "replies")")
                                    .font(.caption)
                            }
                            .foregroundStyle(Color.accentColor)
                            .padding(.top, 2)
                        }
                        .buttonStyle(.plain)
                    }

                    // "Start thread" affordance for messages without threads
                    if message.replyCount == 0 && message.threadRoot == nil {
                        Button {
                            onThreadTap()
                        } label: {
                            HStack(spacing: 4) {
                                Image(systemName: "arrowshape.turn.up.left")
                                    .font(.caption2)
                                Text("Reply in thread")
                                    .font(.caption)
                            }
                            .foregroundStyle(.secondary)
                            .padding(.top, 2)
                        }
                        .buttonStyle(.plain)
                        .opacity(0)
                        .onHover { hovering in
                            _ = hovering
                        }
                    }
                }
            }

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 4)
        .contentShape(Rectangle())
        .contextMenu {
            // Reply (inline quote)
            if !message.isDeleted {
                Button {
                    onReply()
                } label: {
                    Label("Reply", systemImage: "arrowshape.turn.up.left")
                }

                Divider()
            }

            // Reaction options in context menu
            if !message.isDeleted {
                Menu("React") {
                    ForEach(EmojiData.quickReactions, id: \.self) { emoji in
                        Button(emoji) {
                            sendReaction(emoji: emoji)
                        }
                    }
                    Divider()
                    Button("More Emojis…") {
                        showFullEmojiPicker = true
                    }
                }
                Divider()
            }

            // Pin / Unpin
            if !message.isDeleted {
                let isPinned = manager.pinnedMessageIds.contains(message.id)
                Button {
                    Task {
                        if isPinned {
                            await manager.unpinMessage(messageId: message.id)
                        } else {
                            await manager.pinMessage(messageId: message.id)
                        }
                    }
                } label: {
                    Label(
                        isPinned ? "Unpin Message" : "Pin Message",
                        systemImage: isPinned ? "pin.slash" : "pin"
                    )
                }
                Divider()
            }

            if isOwnMessage && !message.isDeleted {
                Button("Edit Message") {
                    editDraft = message.text
                    isEditing = true
                }
                Button("Delete Message", role: .destructive) {
                    showDeleteConfirm = true
                }
            }
        }
        .alert("Delete Message", isPresented: $showDeleteConfirm) {
            Button("Delete", role: .destructive) {
                Task {
                    do {
                        try await manager.deleteMessage(messageId: message.id)
                    } catch {
                        appState.errorMessage = error.localizedDescription
                    }
                }
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Are you sure you want to delete this message? This action cannot be undone.")
        }
        .popover(isPresented: $showFullEmojiPicker) {
            EmojiPicker { emoji in
                showFullEmojiPicker = false
                sendReaction(emoji: emoji)
            }
        }
    }

    // MARK: - Inline Quote View

    @ViewBuilder
    private func inlineQuoteView(replyId: String) -> some View {
        let original = allMessages.first(where: { $0.id == replyId })
        HStack(alignment: .top, spacing: 0) {
            // Left accent border
            RoundedRectangle(cornerRadius: 2)
                .fill(Color.accentColor.opacity(0.5))
                .frame(width: 3)
                .padding(.vertical, 1)

            VStack(alignment: .leading, spacing: 2) {
                if let orig = original {
                    Text(orig.senderName)
                        .font(.caption)
                        .fontWeight(.semibold)
                        .foregroundStyle(Color.accentColor)
                    Text(orig.isDeleted ? "This message was deleted" : orig.text)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .italic(orig.isDeleted)
                } else {
                    Text("Replying to a message")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .italic()
                }
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
        }
        .background(Color.secondary.opacity(0.05), in: RoundedRectangle(cornerRadius: 4))
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .strokeBorder(Color.secondary.opacity(0.1), lineWidth: 1)
        )
    }

    // MARK: - Reaction Chips

    /// Sorted reaction chips displayed below the message body.
    private var reactionChips: some View {
        let sorted = message.reactions
            .sorted { $0.value > $1.value }
        return FlowLayout(spacing: 4) {
            ForEach(sorted, id: \.key) { emoji, count in
                ReactionChip(
                    emoji: emoji,
                    count: count,
                    isOwnReaction: isOwnReaction(emoji: emoji)
                ) {
                    toggleReaction(emoji: emoji)
                }
            }
            // "+" button to add another reaction
            Button {
                showFullEmojiPicker = true
            } label: {
                Text("+")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .frame(width: 28, height: 22)
                    .background(Color.secondary.opacity(0.1), in: RoundedRectangle(cornerRadius: 11))
            }
            .buttonStyle(.plain)
        }
    }

    // MARK: - Inline Edit

    @ViewBuilder
    private var inlineEditView: some View {
        VStack(alignment: .leading, spacing: 6) {
            TextEditor(text: $editDraft)
                .font(.body)
                .frame(minHeight: 40, maxHeight: 120)
                .padding(6)
                .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 6))
                .overlay(
                    RoundedRectangle(cornerRadius: 6)
                        .strokeBorder(Color.accentColor.opacity(0.4), lineWidth: 1)
                )

            HStack(spacing: 8) {
                Button("Save") {
                    let trimmed = editDraft.trimmingCharacters(in: .whitespacesAndNewlines)
                    guard !trimmed.isEmpty else { return }
                    isEditing = false
                    Task {
                        do {
                            try await manager.editMessage(messageId: message.id, newText: trimmed)
                        } catch {
                            appState.errorMessage = error.localizedDescription
                        }
                    }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(editDraft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                Button("Cancel") {
                    isEditing = false
                    editDraft = ""
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
    }

    // MARK: - Helpers

    private var senderAvatar: some View {
        ZStack {
            Circle()
                .fill(avatarColor)
                .frame(width: 32, height: 32)
            Text(String(message.senderName.prefix(1)).uppercased())
                .font(.caption)
                .fontWeight(.semibold)
                .foregroundStyle(.white)
        }
        .padding(.top, 2)
    }

    private var avatarColor: Color {
        let hash = message.senderId.hashValue
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(hash) % colors.count
        return colors[index]
    }

    /// Whether the current agent has already reacted with this emoji.
    /// Since we store aggregate counts (not per-sender lists), we track via `seenReactions` in the manager.
    /// Here we rely on manager exposing a check helper, or approximate via local state.
    private func isOwnReaction(emoji: String) -> Bool {
        manager.hasReacted(emoji: emoji, messageId: message.id, agentId: currentAgentId)
    }

    private func sendReaction(emoji: String) {
        let alreadyReacted = isOwnReaction(emoji: emoji)
        let action: ReactionAction = alreadyReacted ? .remove : .add
        Task {
            do {
                try await manager.sendReaction(emoji: emoji, messageId: message.id, action: action)
            } catch {
                appState.errorMessage = error.localizedDescription
            }
        }
    }

    private func toggleReaction(emoji: String) {
        sendReaction(emoji: emoji)
    }
}

// MARK: - Reaction Chip

/// A pill-shaped reaction chip showing emoji + count.
struct ReactionChip: View {
    let emoji: String
    let count: Int
    let isOwnReaction: Bool
    let onTap: () -> Void

    var body: some View {
        Button(action: onTap) {
            HStack(spacing: 3) {
                Text(emoji)
                    .font(.system(size: 14))
                Text("\(count)")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundStyle(isOwnReaction ? Color.accentColor : .primary)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(
                isOwnReaction
                    ? Color.accentColor.opacity(0.15)
                    : Color.secondary.opacity(0.1),
                in: RoundedRectangle(cornerRadius: 11)
            )
            .overlay(
                RoundedRectangle(cornerRadius: 11)
                    .strokeBorder(
                        isOwnReaction ? Color.accentColor.opacity(0.5) : Color.clear,
                        lineWidth: 1
                    )
            )
        }
        .buttonStyle(.plain)
        .help(isOwnReaction ? "Remove your \(emoji) reaction" : "React with \(emoji)")
    }
}

// MARK: - Pinned Messages Panel

/// Sheet listing all pinned messages in the current channel.
struct PinnedMessagesPanel: View {
    @ObservedObject var manager: ChannelManager
    @Environment(\.dismiss) private var dismiss

    var pinnedMessages: [ChannelChatMessage] {
        manager.pinnedMessageIds.compactMap { id in
            manager.messages.first(where: { $0.id == id })
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Image(systemName: "pin.fill")
                    .foregroundStyle(Color.accentColor)
                Text("Pinned Messages")
                    .font(.headline)
                Spacer()
                Button("Done") { dismiss() }
                    .buttonStyle(.bordered)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)

            Divider()

            if pinnedMessages.isEmpty {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "pin.slash")
                        .font(.system(size: 36))
                        .foregroundStyle(.secondary)
                    Text("No pinned messages")
                        .foregroundStyle(.secondary)
                }
                Spacer()
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(pinnedMessages) { message in
                            PinnedMessageRow(message: message, manager: manager)
                            Divider()
                        }
                    }
                }
            }
        }
    }
}

// MARK: - Pinned Message Row

private struct PinnedMessageRow: View {
    let message: ChannelChatMessage
    @ObservedObject var manager: ChannelManager

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            // Avatar
            ZStack {
                Circle()
                    .fill(avatarColor)
                    .frame(width: 28, height: 28)
                Text(String(message.senderName.prefix(1)).uppercased())
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(message.senderName)
                        .font(.caption)
                        .fontWeight(.semibold)
                    Text(message.date, style: .date)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                Text(message.isDeleted ? "This message was deleted" : message.text)
                    .font(.body)
                    .lineLimit(3)
                    .foregroundStyle(message.isDeleted ? AnyShapeStyle(.tertiary) : AnyShapeStyle(.primary))
                    .italic(message.isDeleted)
            }

            Spacer()

            Button("Unpin") {
                Task {
                    await manager.unpinMessage(messageId: message.id)
                }
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .contentShape(Rectangle())
    }

    private var avatarColor: Color {
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        return colors[abs(message.senderId.hashValue) % colors.count]
    }
}

// MARK: - Search Result Row (Phase 2.7)

struct SearchResultRow: View {
    let message: ChannelChatMessage
    let query: String

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            ZStack {
                Circle()
                    .fill(avatarColor)
                    .frame(width: 28, height: 28)
                Text(String(message.senderName.prefix(1)).uppercased())
                    .font(.caption2)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(message.senderName)
                        .font(.caption)
                        .fontWeight(.semibold)
                    Text(message.date, style: .date)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    Text(message.date, style: .time)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                Text(truncatedPreview(message.text))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .contentShape(Rectangle())
    }

    private func truncatedPreview(_ text: String) -> String {
        let lower = text.lowercased()
        let queryLower = query.lowercased()
        guard let range = lower.range(of: queryLower) else {
            return String(text.prefix(120))
        }
        let start = text.distance(from: text.startIndex, to: range.lowerBound)
        let contextStart = max(0, start - 20)
        let prefix = contextStart > 0 ? "…" : ""
        return prefix + String(text.dropFirst(contextStart).prefix(120))
    }

    private var avatarColor: Color {
        let hash = message.senderId.hashValue
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        return colors[abs(hash) % colors.count]
    }
}

// MARK: - Typing Dots View (Phase 2.8)

struct TypingDotsView: View {
    @State private var animationPhase = 0
    @State private var timer: Timer?

    var body: some View {
        HStack(spacing: 3) {
            ForEach(0..<3, id: \.self) { index in
                Circle()
                    .fill(Color.secondary)
                    .frame(width: 5, height: 5)
                    .opacity(animationPhase == index ? 1.0 : 0.3)
            }
        }
        .onAppear { startAnimation() }
        .onDisappear { timer?.invalidate() }
    }

    private func startAnimation() {
        timer = Timer.scheduledTimer(withTimeInterval: 0.4, repeats: true) { _ in
            withAnimation(.easeInOut(duration: 0.3)) {
                animationPhase = (animationPhase + 1) % 3
            }
        }
    }
}

// MARK: - Presence Dot (Phase 2.9)

/// A small colored circle indicating online/offline presence.
struct PresenceDot: View {
    let isOnline: Bool
    var size: CGFloat = 8

    var body: some View {
        Circle()
            .fill(isOnline ? Color.green : Color.gray.opacity(0.5))
            .frame(width: size, height: size)
            .overlay(
                Circle()
                    .strokeBorder(Color.white.opacity(0.6), lineWidth: isOnline ? 1 : 0)
            )
            .help(isOnline ? "Online" : "Offline")
    }
}

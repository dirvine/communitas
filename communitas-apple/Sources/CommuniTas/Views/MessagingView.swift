import SwiftUI
import X0xClient

/// Full channel chat view with header, message list, and composer.
struct MessagingView: View {
    @EnvironmentObject var appState: AppState
    @State private var draft = ""
    @State private var isSending = false
    @State private var selectedThreadMessage: ChannelChatMessage?

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

            // Messages
            messageList(manager: manager)

            Divider()

            // Composer
            messageComposer(manager: manager)
        }
        .sheet(item: $selectedThreadMessage) { message in
            ThreadView(
                parentMessage: message,
                manager: manager
            )
            .frame(minWidth: 400, minHeight: 500)
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

    private func messageList(manager: ChannelManager) -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 2) {
                    ForEach(manager.messages) { message in
                        MessageRow(
                            message: message,
                            isOwnMessage: message.senderId == (appState.agentIdentity?.agentId ?? ""),
                            onThreadTap: {
                                selectedThreadMessage = message
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
        }
    }

    private func messageComposer(manager: ChannelManager) -> some View {
        HStack(spacing: 8) {
            TextField("Message #\(appState.selectedChannel ?? "channel")...", text: $draft)
                .textFieldStyle(.plain)
                .padding(10)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
                .onSubmit { sendMessage(manager: manager) }

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

    private func sendMessage(manager: ChannelManager) {
        let text = draft.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        draft = ""
        isSending = true

        Task {
            defer { isSending = false }
            do {
                try await manager.sendMessage(text: text)
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
    let isOwnMessage: Bool
    let onThreadTap: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            // Avatar
            senderAvatar

            VStack(alignment: .leading, spacing: 4) {
                // Sender + timestamp
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
                }

                // Message text
                Text(message.text)
                    .font(.body)
                    .textSelection(.enabled)

                // Thread indicator
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
                        // Handled by parent hover
                        _ = hovering
                    }
                }
            }

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 4)
        .contentShape(Rectangle())
    }

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
        // Deterministic color from sender ID
        let hash = message.senderId.hashValue
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(hash) % colors.count
        return colors[index]
    }
}

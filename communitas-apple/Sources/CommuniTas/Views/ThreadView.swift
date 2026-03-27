import SwiftUI
import X0xClient

/// Thread view showing a parent message and its replies.
struct ThreadView: View {
    let parentMessage: ChannelChatMessage
    @ObservedObject var manager: ChannelManager
    @Environment(\.dismiss) private var dismiss

    @State private var replies: [ChannelChatMessage] = []
    @State private var draft = ""
    @State private var alsoSendToChannel = false
    @State private var isSending = false
    @State private var isLoading = true

    var body: some View {
        VStack(spacing: 0) {
            threadHeader
            Divider()
            threadContent
            Divider()
            replyComposer
        }
        .task {
            replies = await manager.loadThread(parentMessageId: parentMessage.id)
            isLoading = false
        }
        .onChange(of: manager.threadMessages[parentMessage.id]?.count) {
            replies = manager.threadMessages[parentMessage.id] ?? []
        }
    }

    private var threadHeader: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text("Thread")
                    .font(.headline)
                Text("#\(parentMessage.channel)")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button {
                dismiss()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.title3)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    private var threadContent: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    // Parent message
                    parentMessageView
                        .padding(.bottom, 8)

                    Divider()
                        .padding(.horizontal, 16)

                    if replies.isEmpty && !isLoading {
                        noRepliesView
                    } else {
                        // Thread replies
                        LazyVStack(alignment: .leading, spacing: 2) {
                            ForEach(replies) { reply in
                                threadReplyRow(reply)
                                    .id(reply.id)
                            }
                        }
                    }

                    if isLoading {
                        HStack {
                            Spacer()
                            ProgressView()
                                .padding()
                            Spacer()
                        }
                    }
                }
                .padding(.vertical, 8)
            }
            .onChange(of: replies.count) {
                if let last = replies.last {
                    withAnimation(.easeOut(duration: 0.2)) {
                        proxy.scrollTo(last.id, anchor: .bottom)
                    }
                }
            }
        }
    }

    private var parentMessageView: some View {
        HStack(alignment: .top, spacing: 10) {
            // Avatar
            ZStack {
                Circle()
                    .fill(avatarColor(for: parentMessage.senderId))
                    .frame(width: 36, height: 36)
                Text(String(parentMessage.senderName.prefix(1)).uppercased())
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(parentMessage.senderName)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                    Text(parentMessage.date, style: .date)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    Text(parentMessage.date, style: .time)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                MarkdownMessageView(text: parentMessage.text)

                Text("\(replies.count) \(replies.count == 1 ? "reply" : "replies")")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.top, 4)
            }

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }

    private func threadReplyRow(_ reply: ChannelChatMessage) -> some View {
        HStack(alignment: .top, spacing: 10) {
            ZStack {
                Circle()
                    .fill(avatarColor(for: reply.senderId))
                    .frame(width: 28, height: 28)
                Text(String(reply.senderName.prefix(1)).uppercased())
                    .font(.caption2)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
            }
            .padding(.top, 2)

            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 6) {
                    Text(reply.senderName)
                        .font(.caption)
                        .fontWeight(.semibold)
                    Text(reply.date, style: .time)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    if reply.broadcast {
                        Label("Also sent to channel", systemImage: "megaphone")
                            .font(.caption2)
                            .foregroundStyle(.orange)
                    }
                }
                MarkdownMessageView(text: reply.text)
            }

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 4)
    }

    private var replyComposer: some View {
        VStack(spacing: 8) {
            Toggle(isOn: $alsoSendToChannel) {
                Label("Also send to #\(parentMessage.channel)", systemImage: "megaphone")
                    .font(.caption)
            }
            .toggleStyle(.checkbox)
            .padding(.horizontal, 12)

            HStack(spacing: 8) {
                TextField("Reply...", text: $draft)
                    .textFieldStyle(.plain)
                    .padding(10)
                    .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
                    .onSubmit { sendReply() }

                Button {
                    sendReply()
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
            .padding(.horizontal, 12)
            .padding(.bottom, 12)
        }
    }

    private var noRepliesView: some View {
        VStack(spacing: 8) {
            Spacer()
            Text("No replies yet")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            Text("Be the first to reply to this thread.")
                .font(.caption)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 24)
    }

    private func sendReply() {
        let text = draft.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        draft = ""
        isSending = true

        Task {
            defer { isSending = false }
            do {
                try await manager.replyInThread(
                    threadRoot: parentMessage.id,
                    text: text,
                    broadcast: alsoSendToChannel
                )
            } catch {
                manager.errorMessage = error.localizedDescription
            }
        }
    }

    private func avatarColor(for senderId: String) -> Color {
        let hash = senderId.hashValue
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(hash) % colors.count
        return colors[index]
    }
}

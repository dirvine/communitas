import SwiftUI
import X0xClient

/// A social feed post.
struct FeedPost: Codable, Identifiable {
    let id: String
    let text: String
    let authorName: String
    let authorId: String
    let timestamp: Int64

    enum CodingKeys: String, CodingKey {
        case id, text, timestamp
        case authorName = "author_name"
        case authorId = "author_id"
    }

    var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
    }
}

/// Social posts feed for a Space.
struct FeedView: View {
    let groupId: String
    @EnvironmentObject var appState: AppState

    @State private var posts: [FeedPost] = []
    @State private var draft = ""
    @State private var isPosting = false
    @State private var webSocket: X0xWebSocket?
    @State private var listeningTask: Task<Void, Never>?

    private var prefix: String {
        appState.groupPrefix(for: groupId)
    }

    private var feedTopic: String {
        "x0x.group.\(prefix).feed"
    }

    var body: some View {
        VStack(spacing: 0) {
            feedHeader
            Divider()
            postComposer
            Divider()
            postList
        }
        .task {
            await subscribeToFeed()
            startListening()
        }
        .onDisappear {
            listeningTask?.cancel()
            webSocket?.disconnect()
        }
    }

    private var feedHeader: some View {
        HStack {
            Image(systemName: "text.bubble")
                .foregroundStyle(.secondary)
            Text("Feed")
                .font(.headline)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Post Composer

    private var postComposer: some View {
        VStack(spacing: 8) {
            TextEditor(text: $draft)
                .font(.body)
                .frame(height: 60)
                .scrollContentBackground(.hidden)
                .padding(6)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))

            HStack {
                Spacer()
                Button {
                    Task { await publishPost() }
                } label: {
                    HStack(spacing: 4) {
                        Image(systemName: "paperplane.fill")
                        Text("Post")
                    }
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(draft.trimmingCharacters(in: .whitespaces).isEmpty || isPosting)
            }
        }
        .padding(12)
    }

    // MARK: - Post List

    private var postList: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 8) {
                if posts.isEmpty {
                    emptyFeedView
                } else {
                    ForEach(posts.sorted(by: { $0.timestamp > $1.timestamp })) { post in
                        postRow(post)
                    }
                }
            }
            .padding(12)
        }
    }

    private var emptyFeedView: some View {
        VStack(spacing: 8) {
            Image(systemName: "text.bubble")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("No posts yet")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            Text("Be the first to post something.")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 40)
    }

    private func postRow(_ post: FeedPost) -> some View {
        HStack(alignment: .top, spacing: 10) {
            // Avatar
            ZStack {
                Circle()
                    .fill(avatarColor(for: post.authorId))
                    .frame(width: 36, height: 36)
                Text(String(post.authorName.prefix(1)).uppercased())
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(post.authorName)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                    Text(post.date, style: .relative)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                Text(post.text)
                    .font(.body)
                    .textSelection(.enabled)
            }

            Spacer()
        }
        .padding(10)
        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }

    // MARK: - Actions

    private func publishPost() async {
        let text = draft.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        isPosting = true
        defer { isPosting = false }

        let post = FeedPost(
            id: UUID().uuidString,
            text: text,
            authorName: appState.displayName,
            authorId: appState.agentIdentity?.agentId ?? "unknown",
            timestamp: Int64(Date().timeIntervalSince1970 * 1000)
        )

        do {
            let data = try JSONEncoder().encode(post)
            let payload = data.base64EncodedString()
            try await appState.client.publish(topic: feedTopic, payload: payload)
            posts.append(post)
            draft = ""
        } catch {
            appState.errorMessage = "Failed to post: \(error.localizedDescription)"
        }
    }

    private func subscribeToFeed() async {
        do {
            _ = try await appState.client.subscribe(topic: feedTopic)
        } catch {
            // Silently ignore
        }
    }

    private func startListening() {
        let ws = X0xWebSocket()
        self.webSocket = ws
        ws.connect()

        listeningTask = Task {
            while !Task.isCancelled {
                do {
                    let text = try await ws.receive()
                    await handleWebSocketMessage(text)
                } catch {
                    if !Task.isCancelled {
                        try? await Task.sleep(nanoseconds: 1_000_000_000)
                    }
                }
            }
        }
    }

    @MainActor
    private func handleWebSocketMessage(_ text: String) async {
        guard let data = text.data(using: .utf8) else { return }

        struct GossipEvent: Codable {
            let event: String?
            let topic: String?
            let payload: String?
            let sender: String?
        }

        guard let gossip = try? JSONDecoder().decode(GossipEvent.self, from: data),
              gossip.topic == feedTopic,
              let payload = gossip.payload,
              let payloadData = Data(base64Encoded: payload) else {
            return
        }

        if let post = try? JSONDecoder().decode(FeedPost.self, from: payloadData) {
            if !posts.contains(where: { $0.id == post.id }) {
                posts.append(post)
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

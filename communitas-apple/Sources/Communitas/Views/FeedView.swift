import AppKit
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
    @State private var isPosting = false
    @State private var hasStartedFeed = false
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
        .onAppear {
            scheduleFeedStartup()
        }
        .onDisappear {
            hasStartedFeed = false
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
        FeedPostComposerPanel(isPosting: isPosting) { text in
            publishPost(text: text)
        }
        .frame(height: 118)
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

    private func scheduleFeedStartup() {
        guard !hasStartedFeed else { return }
        hasStartedFeed = true
        DispatchQueue.main.async {
            Task { @MainActor in
                await subscribeToFeed()
                startListening()
            }
        }
    }

    private func publishPost(text rawText: String) -> Bool {
        let text = rawText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty, !isPosting else { return false }
        isPosting = true

        let post = FeedPost(
            id: UUID().uuidString,
            text: text,
            authorName: appState.displayName,
            authorId: appState.agentIdentity?.agentId ?? "unknown",
            timestamp: Int64(Date().timeIntervalSince1970 * 1000)
        )

        let payload: String
        do {
            let data = try JSONEncoder().encode(post)
            payload = data.base64EncodedString()
        } catch {
            isPosting = false
            appState.errorMessage = "Failed to post: \(error.localizedDescription)"
            return false
        }

        let client = appState.client
        let topic = feedTopic
        posts.append(post)

        Task {
            do {
                try await client.publish(topic: topic, payload: payload)
            } catch {
                await MainActor.run {
                    appState.errorMessage = "Failed to post: \(error.localizedDescription)"
                }
            }
            await MainActor.run {
                isPosting = false
            }
        }

        return true
    }

    private func subscribeToFeed() async {
        do {
            _ = try await appState.client.subscribe(topic: feedTopic)
        } catch {
            // Silently ignore
        }
    }

    private func startListening() {
        listeningTask?.cancel()
        webSocket?.disconnect()
        listeningTask = Task { @MainActor in
            var retryDelay: UInt64 = 1_000_000_000
            while !Task.isCancelled {
                await subscribeToFeed()
                let ws = X0xWebSocket(baseURL: appState.client.webSocketBaseURL, path: "/ws", token: appState.client.token)
                self.webSocket = ws
                ws.connect()
                do {
                    retryDelay = 1_000_000_000
                    while !Task.isCancelled {
                        let text = try await ws.receive()
                        await handleWebSocketMessage(text)
                    }
                } catch {
                    if webSocket === ws {
                        webSocket = nil
                    }
                    ws.disconnect()
                    if !Task.isCancelled {
                        try? await Task.sleep(nanoseconds: retryDelay)
                        retryDelay = min(retryDelay * 2, 30_000_000_000)
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

private struct FeedPostComposerPanel: NSViewRepresentable {
    let isPosting: Bool
    let onSubmit: (String) -> Bool

    func makeCoordinator() -> Coordinator {
        Coordinator(onSubmit: onSubmit)
    }

    func makeNSView(context: Context) -> NSStackView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .width
        stack.spacing = 8
        stack.translatesAutoresizingMaskIntoConstraints = false

        let textView = NSTextView()
        textView.isRichText = false
        textView.allowsUndo = true
        textView.font = .systemFont(ofSize: NSFont.systemFontSize)
        textView.drawsBackground = true
        textView.backgroundColor = NSColor.controlBackgroundColor.withAlphaComponent(0.45)
        textView.textContainerInset = NSSize(width: 8, height: 8)
        textView.isHorizontallyResizable = false
        textView.isVerticallyResizable = true
        textView.autoresizingMask = [.width]
        textView.textContainer?.widthTracksTextView = true
        textView.setAccessibilityIdentifier("feed-post-body")
        textView.delegate = context.coordinator

        let textScroll = NSScrollView()
        textScroll.hasVerticalScroller = true
        textScroll.borderType = .bezelBorder
        textScroll.documentView = textView
        textScroll.translatesAutoresizingMaskIntoConstraints = false
        textScroll.setAccessibilityIdentifier("feed-post-body-scroll")
        textScroll.heightAnchor.constraint(equalToConstant: 74).isActive = true

        let button = NSButton(
            title: "Post",
            target: context.coordinator,
            action: #selector(Coordinator.submit(_:))
        )
        button.bezelStyle = .rounded
        button.controlSize = .regular
        button.image = NSImage(systemSymbolName: "paperplane.fill", accessibilityDescription: "Post")
        button.imagePosition = .imageLeading
        button.setAccessibilityIdentifier("feed-post-button")

        let spacer = NSView()
        spacer.setContentHuggingPriority(.defaultLow, for: .horizontal)

        let buttonRow = NSStackView(views: [spacer, button])
        buttonRow.orientation = .horizontal
        buttonRow.alignment = .centerY
        buttonRow.distribution = .fill

        stack.addArrangedSubview(textScroll)
        stack.addArrangedSubview(buttonRow)

        context.coordinator.textView = textView
        context.coordinator.button = button
        context.coordinator.refreshButtonState()

        return stack
    }

    func updateNSView(_ stack: NSStackView, context: Context) {
        context.coordinator.onSubmit = onSubmit
        context.coordinator.isPosting = isPosting
        context.coordinator.refreshButtonState()
    }

    final class Coordinator: NSObject, NSTextViewDelegate {
        weak var textView: NSTextView?
        weak var button: NSButton?
        var isPosting = false
        var onSubmit: (String) -> Bool

        init(onSubmit: @escaping (String) -> Bool) {
            self.onSubmit = onSubmit
        }

        func textDidChange(_ notification: Notification) {
            refreshButtonState()
        }

        func refreshButtonState() {
            let hasText = !(textView?.string.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ?? true)
            button?.isEnabled = !isPosting && hasText
        }

        @objc func submit(_ sender: NSButton) {
            guard let textView else { return }
            if onSubmit(textView.string) {
                textView.string = ""
                refreshButtonState()
            }
        }
    }
}

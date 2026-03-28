import SwiftUI
import X0xClient

/// A direct message for local storage and display.
struct DMMessage: Codable, Identifiable {
    let id: String
    let text: String
    let senderId: String
    let senderName: String
    let timestamp: Int64
    let isOutgoing: Bool

    var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
    }
}

/// Direct message conversation with a contact.
struct DirectMessageView: View {
    @EnvironmentObject var appState: AppState
    @State private var messages: [DMMessage] = []
    @State private var draft = ""
    @State private var isSending = false
    @State private var webSocket: X0xWebSocket?
    @State private var listeningTask: Task<Void, Never>?
    @State private var hasConnected = false

    var body: some View {
        Group {
            if let contact = appState.selectedDMContact {
                conversationView(contact: contact)
            } else {
                contactPicker
            }
        }
    }

    // MARK: - Contact Picker (when no contact selected)

    private var contactPicker: some View {
        VStack(spacing: 12) {
            Image(systemName: "envelope")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("Direct Messages")
                .font(.title2)
            Text("Select a contact to start a conversation.")
                .foregroundStyle(.secondary)

            if appState.contacts.isEmpty {
                Text("You have no contacts yet.")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            } else {
                List(appState.contacts) { contact in
                    Button {
                        appState.selectedDMContact = contact
                    } label: {
                        HStack {
                            ZStack {
                                Circle()
                                    .fill(avatarColor(for: contact.agentId))
                                    .frame(width: 32, height: 32)
                                Text(String((contact.label ?? contact.agentId).prefix(1)).uppercased())
                                    .font(.caption)
                                    .fontWeight(.semibold)
                                    .foregroundStyle(.white)
                            }
                            VStack(alignment: .leading, spacing: 2) {
                                Text(contact.label ?? truncatedId(contact.agentId))
                                    .font(.body)
                                Text(truncatedId(contact.agentId))
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            Spacer()
                            trustBadge(contact.trustLevel)
                        }
                    }
                    .buttonStyle(.plain)
                }
                .frame(maxWidth: 400, maxHeight: 300)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - Conversation View

    private func conversationView(contact: Contact) -> some View {
        VStack(spacing: 0) {
            conversationHeader(contact: contact)
            Divider()
            messageList(contact: contact)
            Divider()
            messageComposer(contact: contact)
        }
        .task {
            loadHistory(agentId: contact.agentId)
            await connectToAgent(contact: contact)
            startListening(contact: contact)
        }
        .onDisappear {
            listeningTask?.cancel()
            webSocket?.disconnect()
        }
    }

    private func conversationHeader(contact: Contact) -> some View {
        HStack(spacing: 10) {
            ZStack {
                Circle()
                    .fill(avatarColor(for: contact.agentId))
                    .frame(width: 32, height: 32)
                Text(String((contact.label ?? contact.agentId).prefix(1)).uppercased())
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(contact.label ?? "Unknown")
                        .font(.headline)
                    trustBadge(contact.trustLevel)
                }
                Text(truncatedId(contact.agentId))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .fontDesign(.monospaced)
            }

            Spacer()

            Button {
                appState.selectedDMContact = nil
            } label: {
                Image(systemName: "chevron.left")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .help("Back to contact list")
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    private func messageList(contact: Contact) -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 4) {
                    ForEach(messages) { msg in
                        messageBubble(msg)
                            .id(msg.id)
                    }
                }
                .padding(.vertical, 8)
            }
            .onChange(of: messages.count) {
                if let last = messages.last {
                    withAnimation(.easeOut(duration: 0.2)) {
                        proxy.scrollTo(last.id, anchor: .bottom)
                    }
                }
            }
        }
    }

    private func messageBubble(_ msg: DMMessage) -> some View {
        HStack {
            if msg.isOutgoing { Spacer(minLength: 60) }

            VStack(alignment: msg.isOutgoing ? .trailing : .leading, spacing: 2) {
                Text(msg.text)
                    .font(.body)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    .background(
                        msg.isOutgoing
                            ? Color.accentColor.opacity(0.2)
                            : Color.secondary.opacity(0.08),
                        in: RoundedRectangle(cornerRadius: 12)
                    )
                    .textSelection(.enabled)

                Text(msg.date, style: .time)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            if !msg.isOutgoing { Spacer(minLength: 60) }
        }
        .padding(.horizontal, 16)
    }

    private func messageComposer(contact: Contact) -> some View {
        HStack(spacing: 8) {
            TextField("Message \(contact.label ?? "contact")...", text: $draft)
                .textFieldStyle(.plain)
                .padding(10)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))
                .onSubmit { sendMessage(to: contact) }

            Button {
                sendMessage(to: contact)
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

    // MARK: - Actions

    private func sendMessage(to contact: Contact) {
        let text = draft.trimmingCharacters(in: .whitespaces)
        guard !text.isEmpty else { return }
        draft = ""
        isSending = true

        let msg = DMMessage(
            id: UUID().uuidString,
            text: text,
            senderId: appState.agentIdentity?.agentId ?? "me",
            senderName: appState.displayName,
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            isOutgoing: true
        )

        Task {
            defer { isSending = false }
            do {
                let payload = Data(text.utf8).base64EncodedString()
                try await appState.client.sendDirect(agentId: contact.agentId, payload: payload)
                messages.append(msg)
                saveHistory(agentId: contact.agentId)
            } catch {
                appState.errorMessage = "Failed to send: \(error.localizedDescription)"
            }
        }
    }

    private func connectToAgent(contact: Contact) async {
        guard !hasConnected else { return }
        do {
            try await appState.client.connectAgent(agentId: contact.agentId)
            hasConnected = true
        } catch {
            // Silently ignore - agent may already be connected
        }
    }

    private func startListening(contact: Contact) {
        let ws = X0xWebSocket(baseURL: appState.client.webSocketBaseURL, path: "/ws/direct", token: appState.client.token)
        self.webSocket = ws
        ws.connect()

        listeningTask = Task {
            while !Task.isCancelled {
                do {
                    let text = try await ws.receive()
                    await handleDirectMessage(text, from: contact)
                } catch {
                    if !Task.isCancelled {
                        try? await Task.sleep(nanoseconds: 1_000_000_000)
                    }
                }
            }
        }
    }

    @MainActor
    private func handleDirectMessage(_ text: String, from contact: Contact) async {
        guard let data = text.data(using: .utf8) else { return }

        struct DirectEvent: Codable {
            let event: String?
            let sender: String?
            let payload: String?
            let timestamp: UInt64?
        }

        guard let event = try? JSONDecoder().decode(DirectEvent.self, from: data),
              event.sender == contact.agentId,
              let payload = event.payload,
              let payloadData = Data(base64Encoded: payload),
              let messageText = String(data: payloadData, encoding: .utf8) else {
            return
        }

        let msg = DMMessage(
            id: UUID().uuidString,
            text: messageText,
            senderId: contact.agentId,
            senderName: contact.label ?? truncatedId(contact.agentId),
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            isOutgoing: false
        )

        if !messages.contains(where: { $0.id == msg.id }) {
            messages.append(msg)
            saveHistory(agentId: contact.agentId)
        }
    }

    // MARK: - History Persistence

    private func historyKey(agentId: String) -> String {
        "dm_history_\(agentId)"
    }

    private func loadHistory(agentId: String) {
        guard let data = UserDefaults.standard.data(forKey: historyKey(agentId: agentId)),
              let loaded = try? JSONDecoder().decode([DMMessage].self, from: data) else {
            messages = []
            return
        }
        messages = loaded
    }

    private func saveHistory(agentId: String) {
        if let data = try? JSONEncoder().encode(messages) {
            UserDefaults.standard.set(data, forKey: historyKey(agentId: agentId))
        }
    }

    // MARK: - Helpers

    private func avatarColor(for senderId: String) -> Color {
        let hash = senderId.hashValue
        let colors: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(hash) % colors.count
        return colors[index]
    }

    private func truncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
    }

    private func trustBadge(_ level: TrustLevel) -> some View {
        Text(level.rawValue.capitalized)
            .font(.caption2)
            .padding(.horizontal, 8)
            .padding(.vertical, 2)
            .background(trustColor(level).opacity(0.15))
            .foregroundStyle(trustColor(level))
            .clipShape(Capsule())
    }

    private func trustColor(_ level: TrustLevel) -> Color {
        switch level {
        case .blocked: return .red
        case .unknown: return .orange
        case .known: return .blue
        case .trusted: return .green
        }
    }
}

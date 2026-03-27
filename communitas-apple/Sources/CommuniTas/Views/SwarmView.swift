import SwiftUI
import X0xClient

/// Swarm event type.
enum SwarmEventType: String, Codable {
    case posted
    case claimed
    case completed
}

/// A swarm event for task delegation.
struct SwarmEvent: Codable, Identifiable {
    let id: String
    let type: SwarmEventType
    let taskId: String
    let description: String
    let capabilities: String?
    let agentId: String?
    let agentName: String?
    let timestamp: Int64
    let result: String?

    enum CodingKeys: String, CodingKey {
        case id, type, description, capabilities, result, timestamp
        case taskId = "task_id"
        case agentId = "agent_id"
        case agentName = "agent_name"
    }

    var date: Date {
        Date(timeIntervalSince1970: TimeInterval(timestamp) / 1000.0)
    }
}

/// Agent task delegation via swarm topics.
struct SwarmView: View {
    let groupId: String
    @EnvironmentObject var appState: AppState

    @State private var events: [SwarmEvent] = []
    @State private var taskDescription = ""
    @State private var taskCapabilities = ""
    @State private var isPosting = false
    @State private var webSocket: X0xWebSocket?
    @State private var listeningTask: Task<Void, Never>?

    private var prefix: String {
        appState.groupPrefix(for: groupId)
    }

    private var tasksTopic: String {
        "x0x.group.\(prefix).swarm/tasks"
    }

    private var resultsTopic: String {
        "x0x.group.\(prefix).swarm/results"
    }

    var body: some View {
        VStack(spacing: 0) {
            swarmHeader
            Divider()

            HSplitView {
                // Left: submission + agent roster
                VStack(alignment: .leading, spacing: 16) {
                    taskSubmission
                    agentRoster
                    Spacer()
                }
                .frame(minWidth: 250, maxWidth: 320)
                .padding(12)

                // Right: event feed
                eventFeed
            }
        }
        .task {
            await subscribeToTopics()
            startListening()
        }
        .onDisappear {
            listeningTask?.cancel()
            webSocket?.disconnect()
        }
    }

    private var swarmHeader: some View {
        HStack {
            Image(systemName: "ant")
                .foregroundStyle(.secondary)
            Text("Swarm")
                .font(.headline)
            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Task Submission

    private var taskSubmission: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Post Task", systemImage: "plus.circle")
                .font(.subheadline)
                .fontWeight(.semibold)

            TextEditor(text: $taskDescription)
                .font(.body)
                .frame(height: 80)
                .scrollContentBackground(.hidden)
                .padding(6)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 8))

            TextField("Required capabilities (optional)", text: $taskCapabilities)
                .textFieldStyle(.plain)
                .padding(8)
                .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 6))

            Button {
                Task { await postTask() }
            } label: {
                HStack {
                    Image(systemName: "paperplane.fill")
                    Text("Post Task")
                }
            }
            .buttonStyle(.borderedProminent)
            .disabled(taskDescription.trimmingCharacters(in: .whitespaces).isEmpty || isPosting)
        }
    }

    // MARK: - Agent Roster

    private var agentRoster: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Active Agents", systemImage: "person.3")
                .font(.subheadline)
                .fontWeight(.semibold)

            let agents = uniqueAgents()
            if agents.isEmpty {
                Text("No active agents yet.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                FlowLayout(spacing: 6) {
                    ForEach(agents, id: \.self) { name in
                        Text(name)
                            .font(.caption)
                            .padding(.horizontal, 10)
                            .padding(.vertical, 4)
                            .background(Color.accentColor.opacity(0.12), in: Capsule())
                            .foregroundStyle(Color.accentColor)
                    }
                }
            }
        }
    }

    // MARK: - Event Feed

    private var eventFeed: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 6) {
                if events.isEmpty {
                    VStack(spacing: 8) {
                        Image(systemName: "ant")
                            .font(.system(size: 36))
                            .foregroundStyle(.secondary)
                        Text("No swarm events yet")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                        Text("Post a task to get started.")
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 40)
                } else {
                    ForEach(events.reversed()) { event in
                        eventRow(event)
                    }
                }
            }
            .padding(12)
        }
    }

    private func eventRow(_ event: SwarmEvent) -> some View {
        HStack(alignment: .top, spacing: 0) {
            // Color-coded left border
            Rectangle()
                .fill(eventColor(event.type))
                .frame(width: 4)

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(eventLabel(event.type))
                        .font(.caption)
                        .fontWeight(.semibold)
                        .foregroundStyle(eventColor(event.type))

                    Spacer()

                    Text(event.date, style: .time)
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }

                Text(event.description)
                    .font(.subheadline)
                    .lineLimit(3)

                if let agentName = event.agentName {
                    Label(agentName, systemImage: "person.circle")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if let result = event.result {
                    Text(result)
                        .font(.caption)
                        .foregroundStyle(.green)
                        .padding(.top, 2)
                }
            }
            .padding(8)
        }
        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }

    // MARK: - Helpers

    private func eventColor(_ type: SwarmEventType) -> Color {
        switch type {
        case .posted: return .cyan
        case .claimed: return .orange
        case .completed: return .green
        }
    }

    private func eventLabel(_ type: SwarmEventType) -> String {
        switch type {
        case .posted: return "POSTED"
        case .claimed: return "CLAIMED"
        case .completed: return "COMPLETED"
        }
    }

    private func uniqueAgents() -> [String] {
        var seen = Set<String>()
        var result: [String] = []
        for event in events {
            if let name = event.agentName, !seen.contains(name) {
                seen.insert(name)
                result.append(name)
            }
        }
        return result
    }

    // MARK: - Actions

    private func postTask() async {
        let desc = taskDescription.trimmingCharacters(in: .whitespaces)
        guard !desc.isEmpty else { return }
        isPosting = true
        defer { isPosting = false }

        let event = SwarmEvent(
            id: UUID().uuidString,
            type: .posted,
            taskId: UUID().uuidString,
            description: desc,
            capabilities: taskCapabilities.trimmingCharacters(in: .whitespaces).isEmpty
                ? nil : taskCapabilities.trimmingCharacters(in: .whitespaces),
            agentId: appState.agentIdentity?.agentId,
            agentName: appState.displayName,
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            result: nil
        )

        do {
            let data = try JSONEncoder().encode(event)
            let payload = data.base64EncodedString()
            try await appState.client.publish(topic: tasksTopic, payload: payload)
            events.append(event)
            taskDescription = ""
            taskCapabilities = ""
        } catch {
            appState.errorMessage = "Failed to post task: \(error.localizedDescription)"
        }
    }

    private func subscribeToTopics() async {
        do {
            _ = try await appState.client.subscribe(topic: tasksTopic)
            _ = try await appState.client.subscribe(topic: resultsTopic)
        } catch {
            // Silently ignore subscription errors
        }
    }

    private func startListening() {
        let ws = X0xWebSocket(baseURL: appState.client.webSocketBaseURL, path: "/ws", token: appState.client.token)
        self.webSocket = ws
        ws.connect()

        listeningTask = Task { [weak appState] in
            _ = appState
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
              let payload = gossip.payload,
              let payloadData = Data(base64Encoded: payload) else {
            return
        }

        if gossip.topic == tasksTopic || gossip.topic == resultsTopic {
            if let swarmEvent = try? JSONDecoder().decode(SwarmEvent.self, from: payloadData) {
                if !events.contains(where: { $0.id == swarmEvent.id }) {
                    events.append(swarmEvent)
                }
            }
        }
    }
}

// MARK: - Flow Layout

/// Simple flow layout for pill badges.
struct FlowLayout: Layout {
    let spacing: CGFloat

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let maxWidth = proposal.width ?? .infinity
        var currentX: CGFloat = 0
        var currentY: CGFloat = 0
        var lineHeight: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if currentX + size.width > maxWidth && currentX > 0 {
                currentX = 0
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
        }

        return CGSize(width: maxWidth, height: currentY + lineHeight)
    }

    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        var currentX = bounds.minX
        var currentY = bounds.minY
        var lineHeight: CGFloat = 0

        for subview in subviews {
            let size = subview.sizeThatFits(.unspecified)
            if currentX + size.width > bounds.maxX && currentX > bounds.minX {
                currentX = bounds.minX
                currentY += lineHeight + spacing
                lineHeight = 0
            }
            subview.place(at: CGPoint(x: currentX, y: currentY), proposal: .unspecified)
            lineHeight = max(lineHeight, size.height)
            currentX += size.width + spacing
        }
    }
}

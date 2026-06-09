import AppKit
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
    @State private var isPosting = false
    @State private var hasStartedSwarm = false
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
        .onAppear {
            scheduleSwarmStartup()
        }
        .onDisappear {
            hasStartedSwarm = false
            listeningTask?.cancel()
            webSocket?.disconnect()
        }
    }

    private var swarmHeader: some View {
        HStack {
            Image(systemName: "cpu.fill")
                .foregroundStyle(Color.accentColor)
            Text("Agent Swarm")
                .font(.headline)
                .fontWeight(.semibold)
            Spacer()
            Text("Mission Control")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Task Submission

    private var taskSubmission: some View {
        VStack(alignment: .leading, spacing: 8) {
            SwarmTaskSubmissionPanel(isPosting: isPosting) { description, capabilities in
                postTask(description: description, capabilities: capabilities)
            }
            .frame(height: 164)
        }
    }

    // MARK: - Agent Roster

    private var agentRoster: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Active Agents", systemImage: "cpu")
                .font(.subheadline)
                .fontWeight(.semibold)

            let swarmAgents = uniqueAgents()
            let networkAgents = appState.discoveredAgents

            if swarmAgents.isEmpty && networkAgents.isEmpty {
                Text("No active agents yet.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            } else {
                // Show swarm-observed agents first
                if !swarmAgents.isEmpty {
                    Text("In Swarm")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    FlowLayout(spacing: 6) {
                        ForEach(swarmAgents, id: \.self) { name in
                            HStack(spacing: 4) {
                                Circle()
                                    .fill(Color.green)
                                    .frame(width: 6, height: 6)
                                Text(name)
                                    .font(.caption)
                            }
                            .padding(.horizontal, 10)
                            .padding(.vertical, 4)
                            .background(Color.accentColor.opacity(0.12), in: Capsule())
                            .foregroundStyle(Color.accentColor)
                        }
                    }
                }

                // Show all discovered agents on the network
                if !networkAgents.isEmpty {
                    Text("On Network")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                        .padding(.top, swarmAgents.isEmpty ? 0 : 4)
                    FlowLayout(spacing: 6) {
                        ForEach(networkAgents) { agent in
                            HStack(spacing: 4) {
                                Image(systemName: "cpu")
                                    .font(.system(size: 8))
                                    .foregroundStyle(.blue)
                                Text(String(agent.agentId.prefix(8)))
                                    .font(.caption)
                            }
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(Color.blue.opacity(0.10), in: Capsule())
                            .foregroundStyle(.blue)
                        }
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

    private func scheduleSwarmStartup() {
        guard !hasStartedSwarm else { return }
        hasStartedSwarm = true
        DispatchQueue.main.async {
            Task { @MainActor in
                await subscribeToTopics()
                startListening()
            }
        }
    }

    private func postTask(description rawDescription: String, capabilities rawCapabilities: String) -> Bool {
        let desc = rawDescription.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !desc.isEmpty, !isPosting else { return false }
        isPosting = true

        let event = SwarmEvent(
            id: UUID().uuidString,
            type: .posted,
            taskId: UUID().uuidString,
            description: desc,
            capabilities: rawCapabilities.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                ? nil : rawCapabilities.trimmingCharacters(in: .whitespacesAndNewlines),
            agentId: appState.agentIdentity?.agentId,
            agentName: appState.displayName,
            timestamp: Int64(Date().timeIntervalSince1970 * 1000),
            result: nil
        )

        let payload: String
        do {
            let data = try JSONEncoder().encode(event)
            payload = data.base64EncodedString()
        } catch {
            isPosting = false
            appState.errorMessage = "Failed to post task: \(error.localizedDescription)"
            return false
        }

        let client = appState.client
        let topic = tasksTopic
        events.append(event)

        Task {
            do {
                try await client.publish(topic: topic, payload: payload)
            } catch {
                await MainActor.run {
                    appState.errorMessage = "Failed to post task: \(error.localizedDescription)"
                }
            }
            await MainActor.run {
                isPosting = false
            }
        }

        return true
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
        listeningTask?.cancel()
        webSocket?.disconnect()
        listeningTask = Task { @MainActor in
            var retryDelay: UInt64 = 1_000_000_000
            while !Task.isCancelled {
                await subscribeToTopics()
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

private struct SwarmTaskSubmissionPanel: NSViewRepresentable {
    let isPosting: Bool
    let onSubmit: (String, String) -> Bool

    func makeCoordinator() -> Coordinator {
        Coordinator(onSubmit: onSubmit)
    }

    func makeNSView(context: Context) -> NSStackView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.alignment = .leading
        stack.spacing = 8
        stack.translatesAutoresizingMaskIntoConstraints = false

        let label = NSTextField(labelWithString: "Post Task")
        label.font = .systemFont(ofSize: NSFont.systemFontSize, weight: .semibold)
        label.setAccessibilityIdentifier("swarm-post-task-label")

        let textView = NSTextView()
        textView.isRichText = false
        textView.allowsUndo = true
        textView.font = .systemFont(ofSize: NSFont.systemFontSize)
        textView.drawsBackground = true
        textView.backgroundColor = NSColor.controlBackgroundColor.withAlphaComponent(0.45)
        textView.textContainerInset = NSSize(width: 6, height: 6)
        textView.setAccessibilityIdentifier("swarm-task-description")

        let textScroll = NSScrollView()
        textScroll.hasVerticalScroller = true
        textScroll.borderType = .bezelBorder
        textScroll.documentView = textView
        textScroll.translatesAutoresizingMaskIntoConstraints = false
        textScroll.setAccessibilityIdentifier("swarm-task-description-scroll")
        NSLayoutConstraint.activate([
            textScroll.widthAnchor.constraint(equalToConstant: 308),
            textScroll.heightAnchor.constraint(equalToConstant: 80)
        ])

        let capabilitiesField = NSTextField()
        capabilitiesField.placeholderString = "Required capabilities (optional)"
        capabilitiesField.bezelStyle = .roundedBezel
        capabilitiesField.translatesAutoresizingMaskIntoConstraints = false
        capabilitiesField.setAccessibilityIdentifier("swarm-task-capabilities")
        capabilitiesField.widthAnchor.constraint(equalToConstant: 304).isActive = true

        let button = NSButton(
            title: "Post Task",
            target: context.coordinator,
            action: #selector(Coordinator.submit(_:))
        )
        button.bezelStyle = .rounded
        button.controlSize = .regular
        button.image = NSImage(systemSymbolName: "paperplane.fill", accessibilityDescription: "Post Task")
        button.imagePosition = .imageLeading
        button.setAccessibilityIdentifier("swarm-post-task-button")

        stack.addArrangedSubview(label)
        stack.addArrangedSubview(textScroll)
        stack.addArrangedSubview(capabilitiesField)
        stack.addArrangedSubview(button)

        context.coordinator.textView = textView
        context.coordinator.capabilitiesField = capabilitiesField
        context.coordinator.button = button

        return stack
    }

    func updateNSView(_ stack: NSStackView, context: Context) {
        context.coordinator.onSubmit = onSubmit
        context.coordinator.button?.isEnabled = !isPosting
    }

    final class Coordinator: NSObject {
        weak var textView: NSTextView?
        weak var capabilitiesField: NSTextField?
        weak var button: NSButton?
        var onSubmit: (String, String) -> Bool

        init(onSubmit: @escaping (String, String) -> Bool) {
            self.onSubmit = onSubmit
        }

        @objc func submit(_ sender: NSButton) {
            let description = textView?.string ?? ""
            let capabilities = capabilitiesField?.stringValue ?? ""
            if onSubmit(description, capabilities) {
                textView?.string = ""
                capabilitiesField?.stringValue = ""
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

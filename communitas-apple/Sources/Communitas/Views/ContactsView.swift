import SwiftUI
import X0xClient

struct ContactsView: View {
    @EnvironmentObject var appState: AppState
    @State private var showAddSheet = false
    @State private var showComposeSheet = false
    @State private var selectedContact: Contact?
    @State private var discoveredAgents: [DiscoveredAgent] = []
    @State private var isLoadingAgents = false
    @State private var discoveryReport: String?
    @State private var showingDiscoveryReport = false
    @State private var foafResults: [DiscoveredAgent] = []
    @State private var foafStatus: String?
    @State private var connectStatus: [String: String] = [:]
    @State private var directConnectionsCount: Int = 0
    @State private var presenceStatusText: String = ""

    var body: some View {
        Group {
            if appState.daemonState != .running {
                notConnectedPlaceholder
            } else {
                combinedList
            }
        }
        .navigationTitle("People")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    showComposeSheet = true
                } label: {
                    Label("Compose Direct Message", systemImage: "square.and.pencil")
                }
                .disabled(appState.daemonState != .running)
                .accessibilityIdentifier("compose-direct-message")
            }
            ToolbarItem(placement: .automatic) {
                Button {
                    showAddSheet = true
                } label: {
                    Label("Add Contact", systemImage: "plus")
                }
                .disabled(appState.daemonState != .running)
            }
            ToolbarItem(placement: .automatic) {
                Button {
                    Task {
                        await appState.refresh()
                        await loadDiscoveredAgents()
                        await refreshDirectConnectionsCount()
                    }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
            }
        }
        .sheet(isPresented: $showAddSheet) {
            AddContactSheet()
                .environmentObject(appState)
        }
        .sheet(isPresented: $showComposeSheet) {
            ComposeDirectMessageSheet()
                .environmentObject(appState)
        }
        .task {
            await loadDiscoveredAgents()
            await refreshDirectConnectionsCount()
        }
        .alert("Agent Diagnostics", isPresented: $showingDiscoveryReport, actions: {
            Button("OK", role: .cancel) {}
        }, message: {
            Text(discoveryReport ?? "No diagnostics available")
        })
    }

    /// Combined list showing contacts and discovered agents.
    private var combinedList: some View {
        List {
            // Contacts section
            Section {
                if appState.contacts.isEmpty {
                    HStack {
                        Image(systemName: "person.2.slash")
                            .foregroundStyle(.secondary)
                        Text("No contacts yet. Add a contact to start messaging.")
                            .foregroundStyle(.secondary)
                            .font(.callout)
                    }
                    .padding(.vertical, 4)
                } else {
                    ForEach(appState.contacts) { contact in
                        contactRow(contact)
                    }
                }
            } header: {
                Text("Contacts")
            }

            // Discovered Agents section
            Section {
                if isLoadingAgents {
                    HStack {
                        ProgressView()
                            .scaleEffect(0.8)
                        Text("Scanning network...")
                            .foregroundStyle(.secondary)
                            .font(.callout)
                    }
                    .padding(.vertical, 4)
                } else if discoveredAgents.isEmpty {
                    HStack {
                        Image(systemName: "antenna.radiowaves.left.and.right.slash")
                            .foregroundStyle(.secondary)
                        Text("No agents discovered on the network")
                            .foregroundStyle(.secondary)
                            .font(.callout)
                    }
                    .padding(.vertical, 4)
                } else {
                    ForEach(discoveredAgents) { agent in
                        agentRow(agent)
                    }
                }
            } header: {
                HStack {
                    Text("Discovered Agents")
                    if !discoveredAgents.isEmpty {
                        Text("(\(discoveredAgents.count))")
                            .foregroundStyle(.secondary)
                            .accessibilityIdentifier("discovered-agents-count")
                    }
                }
            }
            .accessibilityIdentifier("discovered-agents-list")

            // FOAF walk section
            Section {
                HStack(spacing: 8) {
                    Button {
                        Task { await runFoaf() }
                    } label: {
                        Label("Run FOAF Walk", systemImage: "arrow.triangle.branch")
                    }
                    .buttonStyle(.bordered)
                    .disabled(appState.daemonState != .running)
                    .accessibilityIdentifier("presence-foaf-button")
                    if let s = foafStatus {
                        Text(s).font(.caption).foregroundStyle(.secondary)
                            .accessibilityIdentifier("presence-foaf-status")
                    }
                    Spacer()
                    Text("Direct connections: \(directConnectionsCount)")
                        .font(.caption)
                        .accessibilityIdentifier("direct-connections-count")
                }
                if !foafResults.isEmpty {
                    ForEach(foafResults) { agent in
                        Text("FOAF: \(agent.agentId.prefix(16))…")
                            .font(.system(.caption, design: .monospaced))
                            .accessibilityIdentifier("presence-foaf-result-\(agent.agentId)")
                    }
                }
                if !presenceStatusText.isEmpty {
                    Text(presenceStatusText)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .accessibilityIdentifier("presence-status-text")
                }
            } header: {
                Text("Presence Discovery")
            }
        }
    }

    private func contactRow(_ contact: Contact) -> some View {
        HStack {
            // Presence dot
            PresenceDot(isOnline: appState.onlineAgents.contains(contact.agentId))

            VStack(alignment: .leading, spacing: 4) {
                Text(contact.label ?? truncatedId(contact.agentId))
                    .font(.headline)
                Text(truncatedId(contact.agentId))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            Spacer()
            trustBadge(contact.trustLevel)
        }
        .padding(.vertical, 4)
        .contextMenu {
            Button("Copy Agent ID") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(contact.agentId, forType: .string)
            }

            Divider()

            Menu("Set Trust Level") {
                ForEach(TrustLevel.allCases, id: \.self) { level in
                    Button {
                        Task {
                            try? await appState.client.setTrust(agentId: contact.agentId, level: level)
                            await appState.refresh()
                        }
                    } label: {
                        HStack {
                            Text(level.rawValue.capitalized)
                            if contact.trustLevel == level {
                                Image(systemName: "checkmark")
                            }
                        }
                    }
                }
            }

            Button("View Machines") {
                appState.selectedInspectorItem = .agent(contact)
                appState.showInspector = true
            }

            Divider()

            Button("Remove", role: .destructive) {
                Task {
                    try? await appState.client.removeContact(agentId: contact.agentId)
                    await appState.refresh()
                }
            }
        }
    }

    private func agentRow(_ agent: DiscoveredAgent) -> some View {
        HStack(spacing: 10) {
            // Robot icon
            Image(systemName: "cpu")
                .font(.system(size: 18))
                .foregroundStyle(.blue)
                .frame(width: 24)

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(truncatedId(agent.agentId))
                        .font(.headline)
                    // AI / Agent label badge
                    Text("Agent")
                        .font(.caption2)
                        .fontWeight(.medium)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.blue.opacity(0.12))
                        .foregroundStyle(Color.blue)
                        .clipShape(Capsule())
                }
                Text(truncatedId(agent.agentId))
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .fontDesign(.monospaced)
                if let lastSeen = agent.lastSeen {
                    Text("Last seen \(relativeTime(lastSeen))")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
                if let status = connectStatus[agent.agentId] {
                    Text(status)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .accessibilityIdentifier("connect-status-\(agent.agentId)")
                }
            }
            Spacer()
            Button {
                Task { await connect(agent) }
            } label: {
                Label("Connect", systemImage: "link")
                    .labelStyle(.iconOnly)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("connect-agent-button-\(agent.agentId)")

            Button {
                Task { await inspectDiscoveredAgent(agent) }
            } label: {
                Label("Inspect", systemImage: "info.circle")
                    .labelStyle(.iconOnly)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier("inspect-agent-button-\(agent.agentId)")
        }
        .padding(.vertical, 4)
        .accessibilityIdentifier("discovered-agent-row-\(agent.agentId)")
        .contextMenu {
            Button("Add as Contact") {
                Task {
                    try? await appState.client.addContact(
                        agentId: agent.agentId,
                        trustLevel: .known,
                        label: nil
                    )
                    await appState.refresh()
                }
            }

            Button("Inspect Reachability") {
                Task {
                    await inspectDiscoveredAgent(agent)
                }
            }

            Button("Copy Agent ID") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(agent.agentId, forType: .string)
            }
        }
    }

    private func connect(_ agent: DiscoveredAgent) async {
        connectStatus[agent.agentId] = "Connecting…"
        do {
            try await appState.client.connectAgent(agentId: agent.agentId)
            await refreshDirectConnectionsCount()
            connectStatus[agent.agentId] = "Connected"
        } catch {
            connectStatus[agent.agentId] = "Failed: \(error.localizedDescription)"
        }
    }

    private func refreshDirectConnectionsCount() async {
        do {
            let conns = try await appState.client.directConnections()
            directConnectionsCount = conns.count
        } catch {
            directConnectionsCount = 0
        }
    }

    private func runFoaf() async {
        foafStatus = "Walking…"
        do {
            let agents = try await appState.client.presenceFoaf(ttl: 2, timeoutMs: 4_000)
            foafResults = agents
            foafStatus = "FOAF returned \(agents.count) agents"
        } catch {
            foafResults = []
            foafStatus = "FOAF failed: \(error.localizedDescription)"
        }
    }

    private func loadDiscoveredAgents() async {
        guard appState.daemonState == .running else { return }
        isLoadingAgents = true
        defer { isLoadingAgents = false }
        do {
            discoveredAgents = try await appState.client.discoveredAgents()
        } catch {
            // Best-effort — silently ignore failures
            discoveredAgents = []
        }
    }

    private func inspectDiscoveredAgent(_ agent: DiscoveredAgent) async {
        var lines: [String] = []

        if let status = try? await appState.client.presenceStatus(agentId: agent.agentId) {
            lines.append(status.online ? "Presence: online" : "Presence: offline / unknown")
        }

        if let reachability = try? await appState.client.agentReachability(agentId: agent.agentId) {
            if reachability.likelyDirect {
                lines.append("Reachability: likely direct")
            } else if reachability.needsCoordination {
                lines.append("Reachability: needs coordination")
            } else {
                lines.append("Reachability: unknown")
            }
            if !reachability.addresses.isEmpty {
                lines.append("Addresses: \(reachability.addresses.joined(separator: ", "))")
            }
        }

        if let found = try? await appState.client.findAgent(agentId: agent.agentId), found.found {
            lines.append("Active find: found")
        }

        let report = lines.isEmpty ? "No diagnostics available for this agent yet." : lines.joined(separator: " • ")
        discoveryReport = report
        presenceStatusText = report
        showingDiscoveryReport = true
    }

    private func relativeTime(_ unixSecs: UInt64) -> String {
        let now = UInt64(Date().timeIntervalSince1970)
        let diff = now > unixSecs ? now - unixSecs : 0
        if diff < 60 { return "just now" }
        if diff < 3600 { return "\(diff / 60)m ago" }
        if diff < 86400 { return "\(diff / 3600)h ago" }
        return "\(diff / 86400)d ago"
    }

    private var notConnectedPlaceholder: some View {
        VStack(spacing: 12) {
            Image(systemName: "wifi.slash")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("Daemon Not Connected")
                .font(.title2)
            Text("Start the x0x daemon to manage contacts.")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
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

    private func truncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
    }
}

// MARK: - Add Contact Sheet

struct AddContactSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss
    @State private var agentId = ""
    @State private var label = ""
    @State private var trustLevel: TrustLevel = .known
    @State private var isAdding = false
    @State private var error: String?

    var body: some View {
        VStack(spacing: 16) {
            Text("Add Contact")
                .font(.title2)

            Form {
                TextField("Agent ID or x0x card link", text: $agentId)
                    .font(.system(.body, design: .monospaced))
                TextField("Label (optional; agent ID only)", text: $label)
                Picker("Trust Level", selection: $trustLevel) {
                    ForEach(TrustLevel.allCases, id: \.self) { level in
                        Text(level.rawValue.capitalized).tag(level)
                    }
                }
            }
            .formStyle(.grouped)

            if let error {
                Text(error)
                    .foregroundStyle(.red)
                    .font(.caption)
            }

            HStack {
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("Add") { addContact() }
                    .buttonStyle(.borderedProminent)
                    .disabled(agentId.trimmingCharacters(in: .whitespaces).isEmpty || isAdding)
                    .keyboardShortcut(.defaultAction)
            }
        }
        .padding(20)
        .frame(width: 420)
    }

    private func addContact() {
        isAdding = true
        error = nil
        let trimmedLabel = label.trimmingCharacters(in: .whitespaces)

        Task {
            defer { isAdding = false }
            do {
                let raw = agentId.trimmingCharacters(in: .whitespaces)
                let looksLikeCard = raw.contains("x0x://agent/")
                    || raw.contains("\"agent_id\"")
                    || (raw.count > 96 && raw.count != 64)
                if looksLikeCard {
                    _ = try await appState.client.importAgentCard(card: raw, trustLevel: trustLevel)
                } else {
                    try await appState.client.addContact(
                        agentId: raw,
                        trustLevel: trustLevel,
                        label: trimmedLabel.isEmpty ? nil : trimmedLabel
                    )
                }
                await appState.refresh()
                dismiss()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}

// MARK: - Compose Direct Message Sheet

/// Lightweight DM composer used by the parity XCUITest. Sends via
/// `POST /direct/send` with a self-addressed agent_id so the test
/// does not require a peer.
struct ComposeDirectMessageSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss
    @State private var recipient: String = ""
    @State private var messageText: String = ""
    @State private var sending: Bool = false
    @State private var sentConfirmation: Bool = false
    @State private var sendError: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Compose Direct Message").font(.title2).fontWeight(.semibold)
            TextField("Recipient agent ID", text: $recipient)
                .textFieldStyle(.roundedBorder)
                .accessibilityIdentifier("dm-recipient-agent-id")
            TextEditor(text: $messageText)
                .frame(minHeight: 100)
                .border(Color.secondary.opacity(0.2))
                .accessibilityIdentifier("dm-body")

            if sentConfirmation {
                Text("Sent")
                    .font(.caption)
                    .foregroundStyle(.green)
                    .accessibilityIdentifier("dm-sent-confirmation")
            }
            if let sendError {
                Text(sendError)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .accessibilityIdentifier("dm-send-error")
            }

            HStack {
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.cancelAction)
                Spacer()
                Button("Send") {
                    Task { await send() }
                }
                .buttonStyle(.borderedProminent)
                .disabled(sending
                    || recipient.trimmingCharacters(in: .whitespaces).isEmpty
                    || messageText.isEmpty)
                .accessibilityIdentifier("dm-send")
            }
        }
        .padding(20)
        .frame(width: 480, height: 280)
    }

    private func send() async {
        sending = true
        defer { sending = false }

        var rcpt = recipient.trimmingCharacters(in: .whitespaces)
        if rcpt == "self", let agent = appState.agentIdentity?.agentId {
            rcpt = agent
        }
        let payload = Data(messageText.utf8).base64EncodedString()
        sendError = nil
        sentConfirmation = false
        do {
            try await appState.client.sendDirect(agentId: rcpt, payload: payload)
            sentConfirmation = true
        } catch {
            sendError = "Send failed: \(error.localizedDescription)"
        }
    }
}

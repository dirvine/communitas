import SwiftUI
import X0xClient

struct ContactsView: View {
    @EnvironmentObject var appState: AppState
    @State private var showAddSheet = false
    @State private var selectedContact: Contact?
    @State private var discoveredAgents: [DiscoveredAgent] = []
    @State private var isLoadingAgents = false

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
        .task {
            await loadDiscoveredAgents()
        }
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
                    }
                }
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
                    Text(agent.displayName ?? truncatedId(agent.agentId))
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
            }
            Spacer()
        }
        .padding(.vertical, 4)
        .contextMenu {
            Button("Copy Agent ID") {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(agent.agentId, forType: .string)
            }
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
        case .untrusted: return .red
        case .known: return .orange
        case .trusted: return .blue
        case .verified: return .green
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
                TextField("Agent ID", text: $agentId)
                    .font(.system(.body, design: .monospaced))
                TextField("Label (optional)", text: $label)
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
                try await appState.client.addContact(
                    agentId: agentId.trimmingCharacters(in: .whitespaces),
                    trustLevel: trustLevel,
                    label: trimmedLabel.isEmpty ? nil : trimmedLabel
                )
                await appState.refresh()
                dismiss()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}

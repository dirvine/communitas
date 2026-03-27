import SwiftUI
import X0xClient

struct ContactsView: View {
    @EnvironmentObject var appState: AppState
    @State private var showAddSheet = false
    @State private var selectedContact: Contact?

    var body: some View {
        Group {
            if appState.daemonState != .running {
                notConnectedPlaceholder
            } else if appState.contacts.isEmpty {
                emptyState
            } else {
                contactList
            }
        }
        .navigationTitle("Contacts")
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
                    Task { await appState.refresh() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
            }
        }
        .sheet(isPresented: $showAddSheet) {
            AddContactSheet()
                .environmentObject(appState)
        }
    }

    private var contactList: some View {
        List(appState.contacts, selection: $selectedContact) { contact in
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
            .tag(contact)
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
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Image(systemName: "person.2.slash")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("No Contacts")
                .font(.title2)
            Text("Add a contact to start messaging.")
                .foregroundStyle(.secondary)
            Button("Add Contact") { showAddSheet = true }
                .buttonStyle(.borderedProminent)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
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

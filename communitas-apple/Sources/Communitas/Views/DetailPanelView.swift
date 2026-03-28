import SwiftUI
import X0xClient

/// Inspector panel showing agent profile or space info depending on selection.
struct DetailPanelView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        Group {
            switch appState.selectedInspectorItem {
            case .agent(let contact):
                AgentProfilePanel(contact: contact)
            case .space(let group):
                SpaceInfoPanel(group: group)
            case nil:
                emptyPanel
            }
        }
        .frame(minWidth: 280)
        .background(DeepSpace.surface1)
    }

    private var emptyPanel: some View {
        VStack(spacing: 12) {
            Image(systemName: "sidebar.right")
                .font(.system(size: 32))
                .foregroundStyle(DeepSpace.textMuted)
            Text("Select an item to inspect")
                .font(.caption)
                .foregroundStyle(DeepSpace.textMuted)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Agent Profile Panel

struct AgentProfilePanel: View {
    @EnvironmentObject var appState: AppState
    let contact: Contact

    @State private var machines: [MachineRecord] = []
    @State private var isLoading = false

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                // Avatar
                ZStack {
                    Circle()
                        .fill(avatarColor)
                        .frame(width: 64, height: 64)
                    Text(initials)
                        .font(.title2)
                        .fontWeight(.semibold)
                        .foregroundStyle(.white)
                }
                .padding(.top, 20)

                // Name
                Text(contact.label ?? "Unknown Agent")
                    .font(.headline)
                    .foregroundStyle(DeepSpace.textPrimary)

                // Agent ID (copyable)
                GroupBox {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Agent ID")
                            .font(.caption2)
                            .foregroundStyle(DeepSpace.textMuted)
                        HStack {
                            Text(contact.agentId)
                                .font(.system(.caption2, design: .monospaced))
                                .foregroundStyle(DeepSpace.textSecondary)
                                .textSelection(.enabled)
                                .lineLimit(2)
                            Spacer()
                            Button {
                                NSPasteboard.general.clearContents()
                                NSPasteboard.general.setString(contact.agentId, forType: .string)
                            } label: {
                                Image(systemName: "doc.on.doc")
                                    .font(.caption2)
                            }
                            .buttonStyle(.plain)
                        }
                    }
                    .padding(4)
                }
                .backgroundStyle(DeepSpace.surface2)

                // Trust Level
                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Trust Level")
                            .font(.caption2)
                            .foregroundStyle(DeepSpace.textMuted)
                        HStack(spacing: 6) {
                            trustButton("Blocked", level: .blocked, color: DeepSpace.red)
                            trustButton("Unknown", level: .unknown, color: DeepSpace.amber)
                            trustButton("Known", level: .known, color: DeepSpace.cyan)
                            trustButton("Trusted", level: .trusted, color: DeepSpace.green)
                        }
                    }
                    .padding(4)
                }
                .backgroundStyle(DeepSpace.surface2)

                // Machine Records
                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Machines")
                            .font(.caption2)
                            .foregroundStyle(DeepSpace.textMuted)

                        if isLoading {
                            ProgressView()
                                .controlSize(.small)
                        } else if machines.isEmpty {
                            Text("No machines found")
                                .font(.caption)
                                .foregroundStyle(DeepSpace.textMuted)
                        } else {
                            ForEach(machines) { machine in
                                HStack {
                                    Image(systemName: "desktopcomputer")
                                        .font(.caption)
                                        .foregroundStyle(DeepSpace.textMuted)
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(truncatedId(machine.machineId))
                                            .font(.system(.caption2, design: .monospaced))
                                            .foregroundStyle(DeepSpace.textPrimary)
                                        if let label = machine.label {
                                            Text(label)
                                                .font(.caption2)
                                                .foregroundStyle(DeepSpace.textSecondary)
                                        }
                                    }
                                    Spacer()
                                }
                                .padding(.vertical, 2)
                            }
                        }
                    }
                    .padding(4)
                }
                .backgroundStyle(DeepSpace.surface2)

                // Remove Contact
                Button(role: .destructive) {
                    Task {
                        try? await appState.client.removeContact(agentId: contact.agentId)
                        appState.selectedInspectorItem = nil
                        await appState.refresh()
                    }
                } label: {
                    Label("Remove Contact", systemImage: "person.badge.minus")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .tint(DeepSpace.red)
                .padding(.top, 8)

                Spacer()
            }
            .padding(.horizontal, 16)
        }
        .task {
            await loadMachines()
        }
    }

    private func trustButton(_ label: String, level: TrustLevel, color: Color) -> some View {
        Button {
            Task {
                try? await appState.client.addContact(
                    agentId: contact.agentId,
                    trustLevel: level,
                    label: contact.label
                )
                await appState.refresh()
            }
        } label: {
            Text(label)
                .font(.caption2)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
        }
        .buttonStyle(.plain)
        .background(
            contact.trustLevel == level ? color.opacity(0.2) : DeepSpace.surface3,
            in: Capsule()
        )
        .foregroundStyle(contact.trustLevel == level ? color : DeepSpace.textMuted)
        .overlay(
            Capsule()
                .stroke(contact.trustLevel == level ? color.opacity(0.5) : DeepSpace.border, lineWidth: 1)
        )
    }

    private func loadMachines() async {
        isLoading = true
        defer { isLoading = false }
        do {
            machines = try await appState.client.listMachines(agentId: contact.agentId)
        } catch {
            machines = []
        }
    }

    private var initials: String {
        let name = contact.label ?? contact.agentId
        let parts = name.split(separator: " ")
        if parts.count >= 2 {
            return String(parts[0].prefix(1) + parts[1].prefix(1)).uppercased()
        }
        return String(name.prefix(2)).uppercased()
    }

    private var avatarColor: Color {
        let hash = contact.agentId.hashValue
        let colors: [Color] = [
            DeepSpace.cyan, DeepSpace.violet, DeepSpace.green,
            DeepSpace.amber, DeepSpace.lavender, DeepSpace.red
        ]
        let index = abs(hash) % colors.count
        return colors[index]
    }

    private func truncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
    }
}

// MARK: - Space Info Panel

struct SpaceInfoPanel: View {
    @EnvironmentObject var appState: AppState
    let group: GroupSummary

    @State private var inviteLink: String?
    @State private var displayNameInput = ""
    @State private var groupInfo: GroupInfo?
    @State private var isGeneratingInvite = false

    var body: some View {
        ScrollView {
            VStack(spacing: 16) {
                // Space Icon
                ZStack {
                    RoundedRectangle(cornerRadius: 16)
                        .fill(DeepSpace.surface3)
                        .frame(width: 64, height: 64)
                    Image(systemName: "building.2")
                        .font(.title2)
                        .foregroundStyle(DeepSpace.cyan)
                }
                .padding(.top, 20)

                // Space Name
                Text(group.name)
                    .font(.headline)
                    .foregroundStyle(DeepSpace.textPrimary)

                if let description = group.description {
                    Text(description)
                        .font(.caption)
                        .foregroundStyle(DeepSpace.textSecondary)
                        .multilineTextAlignment(.center)
                }

                // Stats
                HStack(spacing: 16) {
                    VStack {
                        Text("\(group.memberCount ?? 0)")
                            .font(.title3)
                            .fontWeight(.semibold)
                            .foregroundStyle(DeepSpace.textPrimary)
                        Text("Members")
                            .font(.caption2)
                            .foregroundStyle(DeepSpace.textMuted)
                    }
                    if let info = groupInfo, let members = info.members {
                        VStack {
                            Text("\(members.count)")
                                .font(.title3)
                                .fontWeight(.semibold)
                                .foregroundStyle(DeepSpace.textPrimary)
                            Text("Known")
                                .font(.caption2)
                                .foregroundStyle(DeepSpace.textMuted)
                        }
                    }
                }

                // Generate Invite
                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Button {
                            generateInvite()
                        } label: {
                            Label(
                                isGeneratingInvite ? "Generating..." : "Generate Invite Link",
                                systemImage: "link.badge.plus"
                            )
                            .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                        .tint(DeepSpace.cyan)
                        .disabled(isGeneratingInvite)

                        if let inviteLink {
                            HStack {
                                Text(inviteLink)
                                    .font(.system(.caption2, design: .monospaced))
                                    .foregroundStyle(DeepSpace.textSecondary)
                                    .textSelection(.enabled)
                                    .lineLimit(2)
                                Spacer()
                                Button {
                                    NSPasteboard.general.clearContents()
                                    NSPasteboard.general.setString(inviteLink, forType: .string)
                                } label: {
                                    Image(systemName: "doc.on.doc")
                                        .font(.caption2)
                                }
                                .buttonStyle(.plain)
                            }
                        }
                    }
                    .padding(4)
                }
                .backgroundStyle(DeepSpace.surface2)

                // Display Name
                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Display Name")
                            .font(.caption2)
                            .foregroundStyle(DeepSpace.textMuted)
                        HStack(spacing: 8) {
                            TextField("Your name in this space", text: $displayNameInput)
                                .textFieldStyle(.roundedBorder)
                                .font(.caption)
                            Button("Set") {
                                Task {
                                    try? await appState.client.setGroupDisplayName(
                                        groupId: group.groupId,
                                        displayName: displayNameInput
                                    )
                                }
                            }
                            .buttonStyle(.bordered)
                            .disabled(displayNameInput.trimmingCharacters(in: .whitespaces).isEmpty)
                        }
                    }
                    .padding(4)
                }
                .backgroundStyle(DeepSpace.surface2)

                // Leave Space
                Button(role: .destructive) {
                    Task {
                        try? await appState.client.leaveGroup(groupId: group.groupId)
                        appState.selectedInspectorItem = nil
                        await appState.refresh()
                    }
                } label: {
                    Label("Leave Space", systemImage: "rectangle.portrait.and.arrow.right")
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.bordered)
                .tint(DeepSpace.red)
                .padding(.top, 8)

                Spacer()
            }
            .padding(.horizontal, 16)
        }
        .task {
            do {
                groupInfo = try await appState.client.groupInfo(groupId: group.groupId)
            } catch {
                groupInfo = nil
            }
        }
    }

    private func generateInvite() {
        isGeneratingInvite = true
        Task {
            defer { isGeneratingInvite = false }
            do {
                let response = try await appState.client.invite(groupId: group.groupId)
                inviteLink = response.inviteLink
            } catch {
                appState.errorMessage = error.localizedDescription
            }
        }
    }
}

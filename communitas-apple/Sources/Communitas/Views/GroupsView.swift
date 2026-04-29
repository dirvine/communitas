import SwiftUI
import X0xClient

struct GroupsView: View {
    @EnvironmentObject var appState: AppState
    @State private var showCreateSheet = false
    @State private var showJoinSheet = false
    @State private var showDiscoverSheet = false
    @State private var selectedGroup: GroupSummary?
    @State private var manageGroup: GroupSummary?

    var body: some View {
        Group {
            if appState.daemonState != .running {
                notConnectedPlaceholder
            } else if appState.groups.isEmpty {
                emptyState
            } else {
                groupList
            }
        }
        .navigationTitle("Groups")
        .toolbar {
            ToolbarItemGroup(placement: .automatic) {
                Button {
                    showDiscoverSheet = true
                } label: {
                    Label("Discover", systemImage: "magnifyingglass")
                }
                .disabled(appState.daemonState != .running)
                .accessibilityIdentifier("group-discover-toggle")

                Button {
                    showJoinSheet = true
                } label: {
                    Label("Join Group", systemImage: "link")
                }
                .disabled(appState.daemonState != .running)

                Button {
                    showCreateSheet = true
                } label: {
                    Label("Create Group", systemImage: "plus")
                }
                .disabled(appState.daemonState != .running)
                .accessibilityIdentifier("group-create-new")

                Button {
                    Task { await appState.refresh() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
            }
        }
        .sheet(isPresented: $showCreateSheet) {
            CreateGroupSheet()
                .environmentObject(appState)
        }
        .sheet(isPresented: $showJoinSheet) {
            JoinGroupSheet()
                .environmentObject(appState)
        }
        .sheet(isPresented: $showDiscoverSheet) {
            GroupDiscoveryView()
                .environmentObject(appState)
        }
        .sheet(item: $manageGroup) { group in
            ManageGroupSheet(group: group)
                .environmentObject(appState)
        }
    }

    private var groupList: some View {
        List(appState.groups, selection: $selectedGroup) { group in
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(group.name)
                        .font(.headline)
                        .accessibilityIdentifier("group-row-title")
                    if let description = group.description {
                        Text(description)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }
                }
                Spacer()
                if let count = group.memberCount {
                    Label("\(count)", systemImage: "person.2")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .padding(.vertical, 4)
            .tag(group)
            .accessibilityIdentifier("group-row-\(group.groupId)")
            .contextMenu {
                Button("Manage…") {
                    manageGroup = group
                }
                Button("Copy Group ID") {
                    NSPasteboard.general.clearContents()
                    NSPasteboard.general.setString(group.groupId, forType: .string)
                }
                Button("Generate Invite") {
                    Task {
                        do {
                            let response = try await appState.client.invite(groupId: group.groupId)
                            NSPasteboard.general.clearContents()
                            NSPasteboard.general.setString(response.inviteLink, forType: .string)
                        } catch {
                            appState.errorMessage = error.localizedDescription
                        }
                    }
                }
                Divider()
                Button("Leave Group", role: .destructive) {
                    Task {
                        try? await appState.client.leaveGroup(groupId: group.groupId)
                        await appState.refresh()
                    }
                }
            }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Image(systemName: "bubble.left.and.bubble.right")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("No Groups")
                .font(.title2)
            Text("Create a group or join one with an invite.")
                .foregroundStyle(.secondary)
            HStack(spacing: 12) {
                Button("Create Group") { showCreateSheet = true }
                    .buttonStyle(.borderedProminent)
                Button("Join Group") { showJoinSheet = true }
                    .buttonStyle(.bordered)
            }
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
            Text("Start the x0x daemon to manage groups.")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Create Group Sheet

struct CreateGroupSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss
    @State private var name = ""
    @State private var description = ""
    @State private var displayName = ""
    @State private var preset: GroupPolicyPreset = .privateSecure
    @State private var isCreating = false
    @State private var error: String?

    var body: some View {
        VStack(spacing: 16) {
            Text("Create Group")
                .font(.title2)

            Form {
                Section {
                    TextField("Group Name", text: $name)
                        .accessibilityIdentifier("group-name")
                    TextField("Description (optional)", text: $description)
                    TextField("Your Display Name (optional)", text: $displayName)
                }
                Section("Visibility") {
                    Picker("Preset", selection: $preset) {
                        ForEach(GroupPolicyPreset.allCases, id: \.self) { p in
                            VStack(alignment: .leading, spacing: 2) {
                                Text(p.label).font(.body)
                                Text(p.summary)
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                            .tag(p)
                        }
                    }
                    .pickerStyle(.inline)
                    .labelsHidden()
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
                Button("Create") { createGroup() }
                    .buttonStyle(.borderedProminent)
                    .disabled(name.trimmingCharacters(in: .whitespaces).isEmpty || isCreating)
                    .keyboardShortcut(.defaultAction)
                    .accessibilityIdentifier("group-create-confirm")
            }
        }
        .padding(20)
        .frame(width: 440, height: 520)
    }

    private func createGroup() {
        isCreating = true
        error = nil
        let trimmedDesc = description.trimmingCharacters(in: .whitespaces)
        let trimmedDisplay = displayName.trimmingCharacters(in: .whitespaces)

        Task {
            defer { isCreating = false }
            do {
                _ = try await appState.client.createGroupWithPreset(
                    name: name.trimmingCharacters(in: .whitespaces),
                    description: trimmedDesc.isEmpty ? nil : trimmedDesc,
                    displayName: trimmedDisplay.isEmpty ? nil : trimmedDisplay,
                    preset: preset
                )
                await appState.refresh()
                dismiss()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}

// MARK: - Join Group Sheet

struct JoinGroupSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss
    @State private var inviteToken = ""
    @State private var displayName = ""
    @State private var isJoining = false
    @State private var error: String?

    var body: some View {
        VStack(spacing: 16) {
            Text("Join Group")
                .font(.title2)

            Form {
                TextField("Invite Token", text: $inviteToken)
                    .font(.system(.body, design: .monospaced))
                TextField("Your Display Name (optional)", text: $displayName)
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
                Button("Join") { joinGroup() }
                    .buttonStyle(.borderedProminent)
                    .disabled(inviteToken.trimmingCharacters(in: .whitespaces).isEmpty || isJoining)
                    .keyboardShortcut(.defaultAction)
            }
        }
        .padding(20)
        .frame(width: 420)
    }

    private func joinGroup() {
        isJoining = true
        error = nil
        let trimmedDisplay = displayName.trimmingCharacters(in: .whitespaces)

        Task {
            defer { isJoining = false }
            do {
                _ = try await appState.client.joinGroup(
                    invite: inviteToken.trimmingCharacters(in: .whitespaces),
                    displayName: trimmedDisplay.isEmpty ? nil : trimmedDisplay
                )
                await appState.refresh()
                dismiss()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}

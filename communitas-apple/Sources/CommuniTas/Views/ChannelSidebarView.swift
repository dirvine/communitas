import SwiftUI
import X0xClient

/// Sidebar showing spaces (groups) and their channels.
struct ChannelSidebarView: View {
    @EnvironmentObject var appState: AppState
    @State private var showCreateChannel = false
    @State private var expandedGroups: Set<String> = []

    var body: some View {
        List {
            if appState.daemonState != .running {
                notConnectedSection
            } else if appState.groups.isEmpty {
                noSpacesSection
            } else {
                ForEach(appState.groups) { group in
                    spaceSection(group)
                }
            }
        }
        .listStyle(.sidebar)
        .navigationTitle("Channels")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    Task { await appState.refresh() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
            }
        }
        .sheet(isPresented: $showCreateChannel) {
            if let group = appState.selectedGroup {
                CreateChannelSheet(group: group)
                    .environmentObject(appState)
            }
        }
        .task {
            // Auto-expand all groups initially
            for group in appState.groups {
                expandedGroups.insert(group.groupId)
            }
            // Load channels for all groups
            for group in appState.groups {
                let manager = appState.channelManager(for: group)
                await manager.loadChannels()
            }
        }
    }

    @ViewBuilder
    private func spaceSection(_ group: GroupSummary) -> some View {
        Section(isExpanded: Binding(
            get: { expandedGroups.contains(group.groupId) },
            set: { expanded in
                if expanded {
                    expandedGroups.insert(group.groupId)
                } else {
                    expandedGroups.remove(group.groupId)
                }
            }
        )) {
            let manager = appState.channelManager(for: group)

            if manager.channels.isEmpty {
                Text("No channels")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
                    .padding(.leading, 8)
            } else {
                ForEach(manager.channels) { channel in
                    channelRow(channel, group: group, manager: manager)
                }
            }

            Button {
                appState.selectedGroup = group
                showCreateChannel = true
            } label: {
                Label("Add Channel", systemImage: "plus.circle")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .padding(.leading, 4)
        } header: {
            HStack {
                Image(systemName: "building.2")
                    .foregroundStyle(.secondary)
                    .font(.caption)
                Text(group.name.uppercased())
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundStyle(.secondary)
                Spacer()
                if let count = group.memberCount {
                    Text("\(count)")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
            }
        }
    }

    private func channelRow(_ channel: ChannelMeta, group: GroupSummary, manager: ChannelManager) -> some View {
        let isSelected = appState.selectedGroup?.groupId == group.groupId
            && appState.selectedChannel == channel.name

        return Button {
            Task {
                await appState.selectGroupAndChannel(group: group, channel: channel.name)
                appState.selectedNavigation = .messaging
                manager.unreadCounts[channel.name] = 0
            }
        } label: {
            HStack(spacing: 6) {
                Image(systemName: "number")
                    .font(.caption)
                    .foregroundStyle(isSelected ? .primary : .secondary)
                    .frame(width: 16)

                Text(channel.name)
                    .font(.body)
                    .foregroundStyle(isSelected ? .primary : .secondary)
                    .fontWeight(hasUnread(channel: channel.name, manager: manager) ? .semibold : .regular)

                Spacer()

                if let count = manager.unreadCounts[channel.name], count > 0 {
                    Text("\(count)")
                        .font(.caption2)
                        .fontWeight(.bold)
                        .foregroundStyle(.white)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.accentColor, in: Capsule())
                }
            }
            .padding(.vertical, 2)
            .padding(.leading, 4)
        }
        .buttonStyle(.plain)
        .background(
            isSelected
                ? Color.accentColor.opacity(0.15)
                : Color.clear,
            in: RoundedRectangle(cornerRadius: 6)
        )
    }

    private func hasUnread(channel: String, manager: ChannelManager) -> Bool {
        (manager.unreadCounts[channel] ?? 0) > 0
    }

    private var notConnectedSection: some View {
        Section {
            HStack {
                Image(systemName: "wifi.slash")
                    .foregroundStyle(.secondary)
                Text("Daemon not connected")
                    .foregroundStyle(.secondary)
                    .font(.caption)
            }
        }
    }

    private var noSpacesSection: some View {
        Section {
            VStack(alignment: .leading, spacing: 4) {
                Text("No spaces yet")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text("Create a group in the Groups tab first.")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
    }
}

// MARK: - Create Channel Sheet

struct CreateChannelSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss
    let group: GroupSummary

    @State private var name = ""
    @State private var description = ""
    // Category removed - not part of canonical channel schema
    @State private var isCreating = false
    @State private var error: String?

    var body: some View {
        VStack(spacing: 16) {
            Text("Create Channel")
                .font(.title2)
            Text("in \(group.name)")
                .font(.caption)
                .foregroundStyle(.secondary)

            Form {
                TextField("Channel Name", text: $name)
                    .help("Lowercase, hyphens for spaces")
                TextField("Description", text: $description)
                // Category field removed - not part of canonical schema
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
                Button("Create") { createChannel() }
                    .buttonStyle(.borderedProminent)
                    .disabled(name.trimmingCharacters(in: .whitespaces).isEmpty || isCreating)
                    .keyboardShortcut(.defaultAction)
            }
        }
        .padding(20)
        .frame(width: 420)
    }

    private func createChannel() {
        isCreating = true
        error = nil

        Task {
            defer { isCreating = false }
            do {
                let manager = appState.channelManager(for: group)
                try await manager.createChannel(
                    name: name.trimmingCharacters(in: .whitespaces),
                    description: description.trimmingCharacters(in: .whitespaces)
                )
                dismiss()
            } catch {
                self.error = error.localizedDescription
            }
        }
    }
}

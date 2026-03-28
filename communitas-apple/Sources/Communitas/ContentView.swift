import SwiftUI
import X0xClient

// MARK: - ContentView

struct ContentView: View {
    @EnvironmentObject var appState: AppState

    /// Which space is currently expanded in the accordion.
    @State private var expandedSpaceId: String?
    /// Sheet: create channel inside a space.
    @State private var showCreateChannel = false
    /// Which group the create-channel sheet targets.
    @State private var createChannelGroup: GroupSummary?

    var body: some View {
        NavigationSplitView {
            sidebar
                .navigationSplitViewColumnWidth(min: 200, ideal: 230, max: 280)
        } detail: {
            detailView
                .inspector(isPresented: $appState.showInspector) {
                    DetailPanelView()
                        .inspectorColumnWidth(min: 280, ideal: 320, max: 400)
                        .environmentObject(appState)
                }
        }
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    appState.showInspector.toggle()
                } label: {
                    Label("Toggle Inspector", systemImage: "sidebar.right")
                }
                .keyboardShortcut("i", modifiers: [.command, .option])
            }
        }
        .sheet(isPresented: $appState.showCreateSpace) {
            CreateSpaceSheet()
                .environmentObject(appState)
        }
        .sheet(isPresented: $showCreateChannel) {
            if let group = createChannelGroup {
                CreateChannelSheet(group: group)
                    .environmentObject(appState)
            }
        }
        .task {
            // Load channels for all groups concurrently on launch.
            await withTaskGroup(of: Void.self) { taskGroup in
                for group in appState.groups {
                    let manager = appState.channelManager(for: group)
                    // Skip groups whose channels are already populated.
                    guard manager.channels.isEmpty else { continue }
                    taskGroup.addTask {
                        await manager.loadChannels()
                    }
                }
            }
            // Auto-expand the space containing the selected group
            if let selected = appState.selectedGroup {
                expandedSpaceId = selected.groupId
            } else if let first = appState.groups.first {
                expandedSpaceId = first.groupId
            }
        }
        .onChange(of: appState.groups) { _, newGroups in
            // When groups load, ensure we have an expanded space
            if expandedSpaceId == nil, let first = newGroups.first {
                expandedSpaceId = first.groupId
            }
        }
    }

    // MARK: - Sidebar

    private var sidebar: some View {
        List {
            profileSection
            spacesSection
            dmSection
            systemSection
        }
        .listStyle(.sidebar)
    }

    // MARK: - Profile Header

    private var profileSection: some View {
        Section {
            VStack(alignment: .leading, spacing: 4) {
                Text(profileDisplayName)
                    .font(.headline)
                    .fontWeight(.semibold)
                if let agentId = appState.agentIdentity?.agentId {
                    Text(truncatedId(agentId))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .fontDesign(.monospaced)
                }
                if let agentId = appState.agentIdentity?.agentId {
                    Text("agent:\(String(agentId.prefix(8))) \u{00B7} machine")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                } else {
                    Text("Agent \u{00B7} Machine")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                }
            }
            .padding(.vertical, 4)
        }
    }

    private var profileDisplayName: String {
        if !appState.displayName.isEmpty {
            return appState.displayName
        }
        if let agentId = appState.agentIdentity?.agentId {
            return String(agentId.prefix(8))
        }
        return "Communitas"
    }

    /// Truncate any ID for display (first 12 chars + ellipsis).
    private func truncatedId(_ id: String) -> String {
        guard id.count > 16 else { return id }
        return String(id.prefix(12)) + "..."
    }

    // MARK: - Spaces Section

    private var spacesSection: some View {
        Section {
            if appState.daemonState != .running {
                Label("Not connected", systemImage: "wifi.slash")
                    .foregroundStyle(.secondary)
                    .font(.caption)
            } else if appState.groups.isEmpty {
                Text("No spaces yet")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            } else {
                ForEach(appState.groups) { group in
                    spaceRows(group)
                }
            }
        } header: {
            HStack {
                Text("SPACES")
                Spacer()
                Button {
                    appState.showCreateSpace = true
                } label: {
                    Image(systemName: "plus")
                        .font(.caption)
                }
                .buttonStyle(.plain)
            }
        }
    }

    @ViewBuilder
    private func spaceRows(_ group: GroupSummary) -> some View {
        let isExpanded = expandedSpaceId == group.groupId

        // Space header row (acts as toggle)
        Button {
            if isExpanded {
                expandedSpaceId = nil
            } else {
                expandedSpaceId = group.groupId
                // Load channels when expanding. Use weak appState to avoid
                // retaining AppState across the async boundary unnecessarily.
                Task { [weak appState] in
                    guard let appState else { return }
                    let manager = appState.channelManager(for: group)
                    await manager.loadChannels()
                }
            }
        } label: {
            HStack(spacing: 6) {
                Circle()
                    .fill(spaceColor(for: group.groupId))
                    .frame(width: 8, height: 8)
                Text(group.name)
                    .font(.body)
                    .fontWeight(.medium)
                    .foregroundStyle(.primary)
                Spacer()
                Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            .padding(.vertical, 2)
        }
        .buttonStyle(.plain)

        // Expanded content: channels + app shortcuts
        if isExpanded {
            let manager = appState.channelManager(for: group)

            // Channels
            ForEach(manager.channels) { channel in
                channelRow(channel, group: group, manager: manager)
                    .padding(.leading, 16)
            }

            // Add channel
            Button {
                createChannelGroup = group
                showCreateChannel = true
            } label: {
                Label("Add channel", systemImage: "plus.circle")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .padding(.leading, 20)
            .padding(.vertical, 1)

            // App shortcuts (non-chat tabs)
            ForEach([SpaceTab.board, .files, .swarm, .feed, .wiki, .web], id: \.self) { tab in
                appShortcutRow(tab: tab, group: group)
                    .padding(.leading, 16)
            }
        }
    }

    private func channelRow(_ channel: ChannelMeta, group: GroupSummary, manager: ChannelManager) -> some View {
        let isSelected = appState.selectedGroup?.groupId == group.groupId
            && appState.selectedChannel == channel.name
            && appState.selectedSpaceTab == .chat
            && appState.selectedSystemPage == nil
            && appState.selectedDMContact == nil

        // Capture groupId and channelName as value types to avoid retaining
        // a ChannelManager reference that may be replaced by AppState.refresh().
        // The manager is looked up fresh inside the Task to prevent use-after-free.
        let groupId = group.groupId
        let channelName = channel.name

        return Button {
            Task { [weak appState] in
                guard let appState else { return }
                await appState.selectGroupAndChannel(group: group, channel: channelName)
                appState.selectedSpaceTab = .chat
                appState.selectedSystemPage = nil
                appState.selectedDMContact = nil
                // Look up manager fresh — avoids capturing a potentially stale reference.
                appState.channelManagers[groupId]?.unreadCounts[channelName] = 0
            }
        } label: {
            HStack(spacing: 6) {
                Image(systemName: "number")
                    .font(.caption)
                    .foregroundStyle(isSelected ? .primary : .secondary)
                    .frame(width: 14)

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
                        .padding(.horizontal, 5)
                        .padding(.vertical, 2)
                        .background(Color.accentColor, in: Capsule())
                } else if isSelected {
                    Circle()
                        .fill(Color.accentColor)
                        .frame(width: 6, height: 6)
                }
            }
            .padding(.vertical, 2)
        }
        .buttonStyle(.plain)
        .listRowBackground(
            isSelected
                ? Color.accentColor.opacity(0.12)
                : Color.clear
        )
    }

    private func appShortcutRow(tab: SpaceTab, group: GroupSummary) -> some View {
        let isSelected = appState.selectedGroup?.groupId == group.groupId
            && appState.selectedSpaceTab == tab
            && appState.selectedSystemPage == nil
            && appState.selectedDMContact == nil

        return Button {
            appState.selectedGroup = group
            appState.selectedSpaceTab = tab
            appState.selectedSystemPage = nil
            appState.selectedDMContact = nil
        } label: {
            Label(tab.rawValue, systemImage: tab.systemImage)
                .font(.body)
                .foregroundStyle(isSelected ? Color.accentColor : .secondary)
        }
        .buttonStyle(.plain)
        .padding(.vertical, 1)
        .listRowBackground(
            isSelected
                ? Color.accentColor.opacity(0.12)
                : Color.clear
        )
    }

    private func hasUnread(channel: String, manager: ChannelManager) -> Bool {
        (manager.unreadCounts[channel] ?? 0) > 0
    }

    /// Shared color-from-id helper used by spaces and DM avatars.
    private func colorForId(_ id: String) -> Color {
        let palette: [Color] = [.blue, .purple, .orange, .green, .pink, .teal, .indigo, .mint]
        let index = abs(id.hashValue) % palette.count
        return palette[index]
    }

    private func spaceColor(for groupId: String) -> Color {
        colorForId(groupId)
    }

    // MARK: - Direct Messages Section

    private var dmSection: some View {
        Section("DIRECT MESSAGES") {
            if appState.contacts.isEmpty {
                Text("No contacts")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            } else {
                ForEach(appState.contacts) { contact in
                    dmRow(contact)
                }
            }
        }
    }

    private func dmRow(_ contact: Contact) -> some View {
        let isSelected = appState.selectedDMContact?.agentId == contact.agentId
            && appState.selectedSystemPage == nil
        let isOnline = appState.onlineAgents.contains(contact.agentId)
        let displayLabel = contact.label ?? truncatedId(contact.agentId)

        return Button {
            appState.selectedDMContact = contact
            appState.selectedSystemPage = nil
        } label: {
            HStack(spacing: 8) {
                // Presence dot
                Circle()
                    .fill(isOnline ? Color.green : Color.secondary.opacity(0.4))
                    .frame(width: 8, height: 8)

                Text(displayLabel)
                    .font(.body)
                    .foregroundStyle(isSelected ? .primary : .secondary)

                Spacer()
            }
            .padding(.vertical, 2)
        }
        .buttonStyle(.plain)
        .listRowBackground(
            isSelected
                ? Color.accentColor.opacity(0.12)
                : Color.clear
        )
    }

    // MARK: - System Section

    private var systemSection: some View {
        Section("SYSTEM") {
            ForEach(SystemPage.allCases) { page in
                systemRow(page)
            }
        }
    }

    private func systemRow(_ page: SystemPage) -> some View {
        let isSelected = appState.selectedSystemPage == page
            && appState.selectedDMContact == nil

        return Button {
            appState.selectedSystemPage = page
            appState.selectedDMContact = nil
        } label: {
            Label(page.rawValue, systemImage: page.systemImage)
                .foregroundStyle(isSelected ? Color.accentColor : .primary)
        }
        .buttonStyle(.plain)
        .padding(.vertical, 1)
        .listRowBackground(
            isSelected
                ? Color.accentColor.opacity(0.12)
                : Color.clear
        )
    }

    // MARK: - Detail View

    @ViewBuilder
    private var detailView: some View {
        if let page = appState.selectedSystemPage {
            systemDetailView(page)
        } else if let contact = appState.selectedDMContact {
            DirectMessageView()
                .id(contact.agentId)
        } else if appState.selectedGroup != nil {
            SpaceView()
        } else {
            placeholderView
        }
    }

    @ViewBuilder
    private func systemDetailView(_ page: SystemPage) -> some View {
        switch page {
        case .people:
            ContactsView()
        case .network:
            NetworkView()
        case .settings:
            SettingsView()
        case .about:
            AboutView()
        }
    }

    private var placeholderView: some View {
        VStack(spacing: 12) {
            Image(systemName: "building.2")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("Welcome to Communitas")
                .font(.title2)
            Text("Select a space and channel from the sidebar.")
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - About View

struct AboutView: View {
    var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "building.2.crop.circle")
                .font(.system(size: 64))
                .foregroundStyle(.secondary)

            Text("Communitas")
                .font(.largeTitle)
                .fontWeight(.bold)

            Text("A local-first, privacy-focused collaboration platform.")
                .font(.body)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)

            Divider()
                .frame(width: 200)

            VStack(spacing: 8) {
                Link("saorsa-labs.com", destination: URL(string: "https://saorsa-labs.com") ?? URL(string: "https://example.com")!)
                    .font(.body)
                Link("GitHub", destination: URL(string: "https://github.com/saorsa-labs") ?? URL(string: "https://example.com")!)
                    .font(.body)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(40)
        .navigationTitle("About")
    }
}

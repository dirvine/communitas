import Foundation
import X0xClient

/// Represents the item shown in the inspector/detail panel.
enum InspectorItem: Equatable {
    case agent(Contact)
    case space(GroupSummary)

    static func == (lhs: InspectorItem, rhs: InspectorItem) -> Bool {
        switch (lhs, rhs) {
        case (.agent(let a), .agent(let b)): return a.agentId == b.agentId
        case (.space(let a), .space(let b)): return a.groupId == b.groupId
        default: return false
        }
    }
}

/// The active tab within a Space view.
enum SpaceTab: String, CaseIterable, Identifiable, Hashable {
    case chat = "Chat"
    case board = "Board"
    case files = "Files"
    case swarm = "Swarm"
    case feed = "Feed"
    case wiki = "Wiki"
    case web = "Web"

    var id: String { rawValue }

    var systemImage: String {
        switch self {
        case .chat: return "message"
        case .board: return "rectangle.3.group"
        case .files: return "doc.on.doc"
        case .swarm: return "ant"
        case .feed: return "text.bubble"
        case .wiki: return "book"
        case .web: return "globe"
        }
    }
}

/// Central application state shared across all views via `@EnvironmentObject`.
@MainActor
final class AppState: ObservableObject {
    let client = X0xClient()
    let daemon = DaemonManager()

    @Published var daemonState: DaemonState = .notRunning
    @Published var agentIdentity: AgentIdentity?
    @Published var contacts: [Contact] = []
    @Published var groups: [GroupSummary] = []
    @Published var errorMessage: String?
    @Published var selectedNavigation: NavigationItem? = .messaging

    /// Active channel managers keyed by group ID.
    @Published var channelManagers: [String: ChannelManager] = [:]

    /// The currently selected group for messaging.
    @Published var selectedGroup: GroupSummary?

    /// The currently selected channel name within the selected group.
    @Published var selectedChannel: String?

    /// The display name used for sending messages.
    /// Defaults to the first 8 chars of the agent ID (never "Me").
    @Published var displayName: String = ""

    /// The active space tab.
    @Published var selectedSpaceTab: SpaceTab = .chat

    /// The currently selected contact for direct messaging.
    @Published var selectedDMContact: Contact?

    /// The item currently shown in the inspector panel.
    @Published var selectedInspectorItem: InspectorItem?

    /// Whether to show the Create Space sheet.
    @Published var showCreateSpace = false

    /// Whether the inspector panel is visible.
    @Published var showInspector = false

    /// Unread message counts per group ID.
    @Published var unreadCounts: [String: Int] = [:]

    /// Refresh all state from the daemon.
    func refresh() async {
        daemonState = await daemon.state()

        guard daemonState == .running else { return }

        do {
            agentIdentity = try await client.agent()
        } catch {
            agentIdentity = nil
        }

        // Load display name from UserDefaults; fall back to first 8 chars of agent ID
        let stored = UserDefaults.standard.string(forKey: "displayName") ?? ""
        if !stored.isEmpty {
            displayName = stored
        } else if let agentId = agentIdentity?.agentId {
            displayName = String(agentId.prefix(8))
        }

        do {
            contacts = try await client.listContacts()
        } catch {
            contacts = []
        }

        do {
            groups = try await client.listGroups()
        } catch {
            groups = []
        }

        errorMessage = nil
    }

    /// Start the daemon and refresh state.
    func startDaemon() async {
        daemonState = .starting
        do {
            try await daemon.ensureRunning()
            await refresh()
        } catch {
            daemonState = .error
            errorMessage = error.localizedDescription
        }
    }

    /// Get or create a channel manager for a group.
    func channelManager(for group: GroupSummary) -> ChannelManager {
        if let existing = channelManagers[group.groupId] {
            return existing
        }
        let manager = ChannelManager(
            client: client,
            groupId: group.groupId,
            groupName: group.name,
            agentId: agentIdentity?.agentId ?? "unknown",
            displayName: displayName
        )
        channelManagers[group.groupId] = manager
        return manager
    }

    /// Select a group and channel for messaging.
    func selectGroupAndChannel(group: GroupSummary, channel: String) async {
        selectedGroup = group
        selectedChannel = channel
        let manager = channelManager(for: group)
        await manager.subscribeToChannel(name: channel)
    }

    /// Disconnect all channel managers.
    func disconnectAllChannels() {
        for manager in channelManagers.values {
            manager.disconnect()
        }
        channelManagers.removeAll()
    }

    /// Helper to get the group prefix (first 16 chars of group ID).
    func groupPrefix(for groupId: String) -> String {
        String(groupId.prefix(16))
    }
}

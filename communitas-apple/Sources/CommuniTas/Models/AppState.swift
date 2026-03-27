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
    /// The HTTP client used by all views. Rebuilt whenever the daemon config is (re)discovered.
    private(set) var client: X0xClient = X0xClient()
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

    /// Set of agent IDs currently online (from the /presence endpoint).
    @Published var onlineAgents: Set<String> = []

    /// Last time the user interacted with the app (for auto-away tracking).
    var lastInteractionTime: Date = Date()

    /// Whether the user is considered "away" (no interaction for 5 minutes).
    var isAway: Bool {
        Date().timeIntervalSince(lastInteractionTime) > 300
    }

    /// Background task for presence polling.
    private var presencePollingTask: Task<Void, Never>?

    /// Refresh all state from the daemon.
    func refresh() async {
        // Attempt to discover daemon config and build an authenticated client.
        // If the daemon isn't running yet the config files won't exist; we fall
        // back to an unauthenticated client so the health check can still run.
        if let discovered = X0xClient.fromDiscovery() {
            client = discovered
        }

        daemonState = await daemon.state(using: client)

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

        // Auto-select first group and its first channel if none selected
        if selectedGroup == nil, let first = groups.first {
            selectedGroup = first
            // Load actual channels; fall back to "general"
            let manager = channelManager(for: first)
            await manager.loadChannels()
            if let firstChannel = manager.channels.first {
                selectedChannel = firstChannel.name
            } else {
                selectedChannel = "general"
            }
        }

        errorMessage = nil
    }

    /// Start the daemon and refresh state.
    func startDaemon() async {
        daemonState = .starting
        do {
            try await daemon.ensureRunning()
            // Re-discover config now that the daemon has written its api.port / api-token files.
            if let discovered = X0xClient.fromDiscovery() {
                client = discovered
            }
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

    // MARK: - Presence (Phase 2.9)

    /// Fetch presence from the daemon and update `onlineAgents`.
    func refreshPresence() async {
        guard daemonState == .running else { return }
        do {
            let agents = try await client.presence()
            onlineAgents = Set(agents)
        } catch {
            // Presence is best-effort — silently ignore failures
        }
    }

    /// Start polling presence every 60 seconds. Safe to call multiple times.
    func startPresencePolling() {
        presencePollingTask?.cancel()
        presencePollingTask = Task { [weak self] in
            while !Task.isCancelled {
                await self?.refreshPresence()
                try? await Task.sleep(nanoseconds: 60_000_000_000)
            }
        }
    }

    /// Stop presence polling (call when app moves to background).
    func stopPresencePolling() {
        presencePollingTask?.cancel()
        presencePollingTask = nil
    }

    /// Record a user interaction (resets auto-away timer).
    func recordInteraction() {
        lastInteractionTime = Date()
    }
}

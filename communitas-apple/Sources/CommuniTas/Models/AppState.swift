import Foundation
import X0xClient

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
    @Published var selectedNavigation: NavigationItem? = .status

    /// Active channel managers keyed by group ID.
    @Published var channelManagers: [String: ChannelManager] = [:]

    /// The currently selected group for messaging.
    @Published var selectedGroup: GroupSummary?

    /// The currently selected channel name within the selected group.
    @Published var selectedChannel: String?

    /// The display name used for sending messages.
    @Published var displayName: String = "Me"

    /// Refresh all state from the daemon.
    func refresh() async {
        daemonState = await daemon.state()

        guard daemonState == .running else { return }

        do {
            agentIdentity = try await client.agent()
        } catch {
            agentIdentity = nil
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
}

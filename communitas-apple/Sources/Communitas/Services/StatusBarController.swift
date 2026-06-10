import AppKit
import X0xClient

/// Manages a persistent menu bar status item showing daemon health.
@MainActor
final class StatusBarController {
    private var statusItem: NSStatusItem?
    private var healthTimer: Timer?
    private weak var appState: AppState?

    init(appState: AppState) {
        self.appState = appState
        setupStatusItem()
        startHealthPolling()
    }

    deinit {
        healthTimer?.invalidate()
    }

    // MARK: - Setup

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        updateIcon(for: .notRunning)
        rebuildMenu()
    }

    // MARK: - Icon

    private func updateIcon(for state: DaemonState) {
        guard let button = statusItem?.button else { return }
        let symbolName: String
        switch state {
        case .running:
            symbolName = "circle.fill"
            button.contentTintColor = .systemGreen
        case .starting:
            symbolName = "circle.dotted"
            button.contentTintColor = .systemYellow
        case .notRunning:
            symbolName = "circle"
            button.contentTintColor = .systemGray
        case .notInstalled:
            symbolName = "xmark.circle"
            button.contentTintColor = .systemRed
        case .error:
            symbolName = "exclamationmark.circle"
            button.contentTintColor = .systemRed
        }
        button.image = NSImage(systemSymbolName: symbolName, accessibilityDescription: "Daemon status: \(state.rawValue)")
    }

    // MARK: - Menu

    func rebuildMenu() {
        let menu = NSMenu()

        // Status header
        let stateLabel = appState?.daemonState ?? .notRunning
        let statusItem = NSMenuItem(title: "x0xd: \(stateLabel.rawValue)", action: nil, keyEquivalent: "")
        statusItem.isEnabled = false
        menu.addItem(statusItem)

        if let identity = appState?.agentIdentity {
            let agentItem = NSMenuItem(title: "Agent: \(String(identity.agentId.prefix(12)))...", action: nil, keyEquivalent: "")
            agentItem.isEnabled = false
            menu.addItem(agentItem)
        }

        menu.addItem(.separator())

        // Quick actions
        let newSpace = NSMenuItem(title: "New Space", action: #selector(handleNewSpace), keyEquivalent: "")
        newSpace.target = self
        menu.addItem(newSpace)

        // Recent spaces
        if let groups = appState?.groups, !groups.isEmpty {
            let recentMenu = NSMenu()
            for group in groups.prefix(5) {
                let item = NSMenuItem(title: group.name, action: #selector(handleSelectSpace(_:)), keyEquivalent: "")
                item.target = self
                item.representedObject = group.groupId
                recentMenu.addItem(item)
            }
            let recentItem = NSMenuItem(title: "Recent Spaces", action: nil, keyEquivalent: "")
            recentItem.submenu = recentMenu
            menu.addItem(recentItem)
        }

        menu.addItem(.separator())

        // Show window
        let showWindow = NSMenuItem(title: "Show Communitas", action: #selector(handleShowWindow), keyEquivalent: "")
        showWindow.target = self
        menu.addItem(showWindow)

        menu.addItem(.separator())

        // Quit
        let quit = NSMenuItem(title: "Quit Communitas", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        menu.addItem(quit)

        self.statusItem?.menu = menu
    }

    // MARK: - Health Polling

    private func startHealthPolling() {
        healthTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            Task { @MainActor in
                self?.refreshStatus()
            }
        }
    }

    private func refreshStatus() {
        guard let appState else { return }
        updateIcon(for: appState.daemonState)
        rebuildMenu()
    }

    // MARK: - Actions

    @objc private func handleNewSpace() {
        NSApp.activate(ignoringOtherApps: true)
        appState?.showCreateSpace = true
    }

    @objc private func handleSelectSpace(_ sender: NSMenuItem) {
        guard let groupId = sender.representedObject as? String,
              let group = appState?.groups.first(where: { $0.groupId == groupId }) else { return }
        NSApp.activate(ignoringOtherApps: true)
        Task { @MainActor in
            await appState?.selectGroupAndChannel(group: group, channel: "general")
        }
    }

    @objc private func handleShowWindow() {
        NSApp.activate(ignoringOtherApps: true)
        if let window = NSApp.windows.first(where: { $0.isCommunitasContentWindow }) {
            window.makeKeyAndOrderFront(nil)
        }
    }
}

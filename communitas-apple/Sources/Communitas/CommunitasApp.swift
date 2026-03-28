import SwiftUI

/// App delegate for dock menu and lifecycle hooks.
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    weak var appState: AppState?
    var statusBarController: StatusBarController?

    func applicationDockMenu(_ sender: NSApplication) -> NSMenu? {
        let menu = NSMenu()

        let newSpace = NSMenuItem(title: "New Space", action: #selector(dockNewSpace), keyEquivalent: "")
        newSpace.target = self
        menu.addItem(newSpace)

        if let groups = appState?.groups, !groups.isEmpty {
            menu.addItem(.separator())
            let header = NSMenuItem(title: "Recent Spaces", action: nil, keyEquivalent: "")
            header.isEnabled = false
            menu.addItem(header)
            for group in groups.prefix(5) {
                let item = NSMenuItem(title: group.name, action: #selector(dockSelectSpace(_:)), keyEquivalent: "")
                item.target = self
                item.representedObject = group.groupId
                menu.addItem(item)
            }
        }

        menu.addItem(.separator())
        let network = NSMenuItem(title: "Network Status", action: #selector(dockNetworkStatus), keyEquivalent: "")
        network.target = self
        menu.addItem(network)

        return menu
    }

    @objc private func dockNewSpace() {
        NSApp.activate(ignoringOtherApps: true)
        appState?.showCreateSpace = true
    }

    @objc private func dockSelectSpace(_ sender: NSMenuItem) {
        guard let groupId = sender.representedObject as? String,
              let group = appState?.groups.first(where: { $0.groupId == groupId }) else { return }
        NSApp.activate(ignoringOtherApps: true)
        Task { @MainActor in
            await appState?.selectGroupAndChannel(group: group, channel: "general")
        }
    }

    @objc private func dockNetworkStatus() {
        NSApp.activate(ignoringOtherApps: true)
        appState?.selectedSystemPage = .network
        appState?.selectedDMContact = nil
    }
}

@main
struct CommunitasApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var appState = AppState()
    @StateObject private var updaterController = UpdaterController()

    var body: some Scene {
        WindowGroup {
            OnboardingView {
                ContentView()
                    .environmentObject(appState)
                    .environmentObject(updaterController)
                    .task {
                        NotificationService.shared.requestPermission()
                        await appState.refresh()
                        appState.startPresencePolling()

                        // Wire up app delegate and status bar
                        appDelegate.appState = appState
                        appDelegate.statusBarController = StatusBarController(appState: appState)
                    }
                    .onAppear {
                        NotificationService.shared.requestPermission()
                    }
            }
            .environmentObject(appState)
            .environmentObject(updaterController)
        }
        .windowStyle(.titleBar)
        .defaultSize(width: 1100, height: 750)
        .commands {
            CommandGroup(after: .appInfo) {
                Button("Check for Updates...") {
                    updaterController.checkForUpdates()
                }
            }

            CommandGroup(replacing: .newItem) {
                Button("New Space") {
                    appState.showCreateSpace = true
                }
                .keyboardShortcut("n", modifiers: .command)

                Button("Join Space") {
                    appState.showCreateSpace = true
                }
                .keyboardShortcut("j", modifiers: [.command, .shift])
            }

            CommandGroup(replacing: .toolbar) {
                Button("Toggle Sidebar") {
                    NSApp.keyWindow?.firstResponder?.tryToPerform(
                        #selector(NSSplitViewController.toggleSidebar(_:)),
                        with: nil
                    )
                }
                .keyboardShortcut("s", modifiers: [.command, .control])

                Button("Toggle Inspector") {
                    appState.showInspector.toggle()
                }
                .keyboardShortcut("i", modifiers: [.command, .option])
            }

            CommandMenu("Space") {
                Button("Invite to Space") {
                    if let group = appState.selectedGroup {
                        appState.selectedInspectorItem = .space(group)
                        appState.showInspector = true
                    }
                }
                .keyboardShortcut("i", modifiers: .command)
            }

            CommandMenu("Go") {
                Button("Quick Switcher") {
                    appState.showQuickSwitcher.toggle()
                }
                .keyboardShortcut("k", modifiers: .command)

                Divider()

                // Cmd+1..5 for first 5 spaces
                ForEach(Array(appState.groups.prefix(5).enumerated()), id: \.element.groupId) { index, group in
                    Button(group.name) {
                        Task {
                            await appState.selectGroupAndChannel(group: group, channel: "general")
                            appState.selectedSpaceTab = .chat
                            appState.selectedSystemPage = nil
                            appState.selectedDMContact = nil
                        }
                    }
                    .keyboardShortcut(KeyEquivalent(Character("\(index + 1)")), modifiers: .command)
                }
            }

            // Quick navigation shortcuts
            CommandGroup(after: .toolbar) {
                Divider()

                Button("Network") {
                    appState.selectedSystemPage = .network
                    appState.selectedDMContact = nil
                }
                .keyboardShortcut("3", modifiers: .command)

                Button("People") {
                    appState.selectedSystemPage = .people
                    appState.selectedDMContact = nil
                }
                .keyboardShortcut("5", modifiers: .command)

                Button("Settings") {
                    appState.selectedSystemPage = .settings
                    appState.selectedDMContact = nil
                }
                .keyboardShortcut(",", modifiers: .command)
            }
        }

        Settings {
            SettingsView()
                .environmentObject(appState)
        }
    }
}

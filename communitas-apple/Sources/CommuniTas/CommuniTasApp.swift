import SwiftUI

@main
struct CommuniTasApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(appState)
                .task {
                    NotificationService.shared.requestPermission()
                    await appState.refresh()
                    appState.startPresencePolling()
                }
                .onAppear {
                    // Request notification permission on launch
                    NotificationService.shared.requestPermission()
                }
        }
        .windowStyle(.titleBar)
        .defaultSize(width: 1100, height: 750)
        .commands {
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

            CommandMenu("Space") {
                Button("New Channel") {
                    // Navigate to messaging and trigger channel creation
                    appState.selectedNavigation = .messaging
                }
                .keyboardShortcut("n", modifiers: [.command, .shift])

                Divider()

                Button("Invite to Space") {
                    if let group = appState.selectedGroup {
                        appState.selectedInspectorItem = .space(group)
                        appState.showInspector = true
                    }
                }
                .keyboardShortcut("i", modifiers: .command)
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

            // Quick navigation shortcuts
            CommandGroup(after: .toolbar) {
                Divider()

                Button("Dashboard") {
                    appState.selectedNavigation = .dashboard
                }
                .keyboardShortcut("1", modifiers: .command)

                Button("Status") {
                    appState.selectedNavigation = .status
                }
                .keyboardShortcut("2", modifiers: .command)

                Button("Network") {
                    appState.selectedNavigation = .network
                }
                .keyboardShortcut("3", modifiers: .command)

                Button("Messaging") {
                    appState.selectedNavigation = .messaging
                }
                .keyboardShortcut("4", modifiers: .command)

                Button("Contacts") {
                    appState.selectedNavigation = .contacts
                }
                .keyboardShortcut("5", modifiers: .command)

                Button("Groups") {
                    appState.selectedNavigation = .groups
                }
                .keyboardShortcut("6", modifiers: .command)

                Button("Settings") {
                    appState.selectedNavigation = .settings
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

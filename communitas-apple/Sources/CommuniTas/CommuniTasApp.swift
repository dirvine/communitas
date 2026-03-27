import SwiftUI

@main
struct CommuniTasApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            OnboardingView {
                ContentView()
                    .environmentObject(appState)
                    .task {
                        NotificationService.shared.requestPermission()
                        await appState.refresh()
                        appState.startPresencePolling()
                    }
                    .onAppear {
                        NotificationService.shared.requestPermission()
                    }
            }
            .environmentObject(appState)
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

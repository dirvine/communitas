import SwiftUI
import AppKit

/// App delegate for dock menu and lifecycle hooks.
@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    weak var appState: AppState?
    var statusBarController: StatusBarController?
    private var fallbackWindow: NSWindow?
    private var fallbackAppState: AppState?
    private var fallbackUpdaterController: UpdaterController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
        logWindowDiagnostics("didFinishLaunching")
        if shouldForceMainWindow {
            ensureMainWindowExists(after: 0.75, createIfMissing: true)
        }
        positionMainWindowForAutomationIfRequested()
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        showMainWindow(createIfMissing: true)
        return true
    }

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

    func showMainWindow(createIfMissing: Bool = false) {
        if let window = mainWindow {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }
        guard createIfMissing || shouldForceMainWindow else {
            NSApp.activate(ignoringOtherApps: true)
            return
        }
        ensureMainWindowExists(after: 0, force: true, createIfMissing: true)
    }

    func ensureMainWindowExists(after delay: TimeInterval, force: Bool = false, createIfMissing: Bool = false) {
        DispatchQueue.main.asyncAfter(deadline: .now() + delay) { [weak self] in
            guard let self else { return }
            self.logWindowDiagnostics("ensureMainWindowExists before fallback")
            if let window = self.visibleMainWindow {
                if force || self.shouldForceMainWindow {
                    window.makeKeyAndOrderFront(nil)
                    NSApp.activate(ignoringOtherApps: true)
                    positionMainWindowForAutomationIfRequested()
                }
                return
            }
            let needsAutomationFallback = self.shouldForceMainWindow && self.fallbackWindow == nil
            guard force || createIfMissing || needsAutomationFallback else { return }
            self.createFallbackMainWindow()
        }
    }

    private var mainWindow: NSWindow? {
        NSApp.windows.first { window in
            window != fallbackWindow && window.isCommunitasContentWindow
        } ?? fallbackWindow
    }

    private var visibleMainWindow: NSWindow? {
        NSApp.windows.first { window in
            window != fallbackWindow && window.isCommunitasContentWindow && window.isVisible
        } ?? (fallbackWindow?.isVisible == true ? fallbackWindow : nil)
    }

    private func createFallbackMainWindow() {
        if let fallbackWindow {
            fallbackWindow.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }

        let appState = AppState()
        let updaterController = UpdaterController()
        fallbackAppState = appState
        fallbackUpdaterController = updaterController
        self.appState = appState

        let rootView = CommunitasRootView(
            appState: appState,
            updaterController: updaterController,
            appDelegate: self
        )
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1100, height: 750),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.identifier = NSUserInterfaceItemIdentifier("CommunitasMainWindow")
        let title = Bundle.main.object(forInfoDictionaryKey: "CFBundleDisplayName") as? String
            ?? Bundle.main.object(forInfoDictionaryKey: "CFBundleName") as? String
            ?? "Communitas"
        window.title = title
        window.setAccessibilityTitle(title)
        window.setAccessibilityIdentifier("CommunitasMainWindow")
        window.isReleasedWhenClosed = false
        window.level = .normal
        window.collectionBehavior = [.managed, .fullScreenPrimary]
        window.center()
        window.contentView = NSHostingView(rootView: rootView)
        fallbackWindow = window
        window.makeKeyAndOrderFront(nil)
        window.orderFrontRegardless()
        NSApp.activate(ignoringOtherApps: true)
        logWindowDiagnostics("created fallback window")
        positionMainWindowForAutomationIfRequested()
    }

    private func logWindowDiagnostics(_ context: String) {
        guard shouldForceMainWindow else { return }

        let windows = NSApp.windows.enumerated().map { index, window in
            let className = NSStringFromClass(type(of: window))
            return "#\(index): \(className) title=\(window.title) visible=\(window.isVisible) keyable=\(window.canBecomeKey) level=\(window.level.rawValue)"
        }.joined(separator: " | ")
        NSLog("Communitas window diagnostics \(context): count=\(NSApp.windows.count) \(windows)")
    }

    private var shouldForceMainWindow: Bool {
        let process = ProcessInfo.processInfo
        return process.environment["COMMUNITAS_FORCE_MAIN_WINDOW"] == "1"
            || process.arguments.contains("-CommunitasForceMainWindow")
    }
}

private struct CommunitasRootView: View {
    @ObservedObject var appState: AppState
    @ObservedObject var updaterController: UpdaterController
    let appDelegate: AppDelegate

    var body: some View {
        OnboardingView {
            ContentView()
                .environmentObject(appState)
                .environmentObject(updaterController)
                .task {
                    appDelegate.showMainWindow()
                    NotificationService.shared.requestPermission()
                    await appState.refresh()
                    appState.startPresencePolling()

                    appDelegate.appState = appState
                    if appDelegate.statusBarController == nil {
                        appDelegate.statusBarController = StatusBarController(appState: appState)
                    }
                }
                .onAppear {
                    appDelegate.showMainWindow()
                    NotificationService.shared.requestPermission()
                    positionMainWindowForAutomationIfRequested()
                }
        }
        .environmentObject(appState)
        .environmentObject(updaterController)
    }
}

@main
struct CommunitasApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var appState = AppState()
    @StateObject private var updaterController = UpdaterController()

    init() {
        NSApplication.shared.setActivationPolicy(.regular)
    }

    var body: some Scene {
        WindowGroup {
            CommunitasRootView(
                appState: appState,
                updaterController: updaterController,
                appDelegate: appDelegate
            )
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
                .environmentObject(updaterController)
        }
    }
}

@MainActor
private func positionMainWindowForAutomationIfRequested() {
    let process = ProcessInfo.processInfo
    let forceByEnvironment = process.environment["COMMUNITAS_FORCE_MAIN_WINDOW"] == "1"
    let forceByArgument = process.arguments.contains("-CommunitasForceMainWindow")
    guard forceByEnvironment || forceByArgument else { return }

    for attempt in 1...12 {
        DispatchQueue.main.asyncAfter(deadline: .now() + (Double(attempt) * 0.25)) {
            guard let screen = NSScreen.screens.first(where: { $0.localizedName.contains("Built-in") }) ?? NSScreen.main else {
                return
            }
            let visibleFrame = screen.visibleFrame
            let fallbackSize = NSSize(width: 1100, height: 750)

            for window in NSApp.windows where window.isCommunitasContentWindow {
                window.deminiaturize(nil)
                window.collectionBehavior.insert(.moveToActiveSpace)
                let currentSize = window.frame.size
                let width = min(max(currentSize.width, fallbackSize.width), visibleFrame.width)
                let height = min(max(currentSize.height, fallbackSize.height), visibleFrame.height)
                let frame = NSRect(
                    x: visibleFrame.midX - width / 2,
                    y: visibleFrame.midY - height / 2,
                    width: width,
                    height: height
                )
                window.setFrame(frame, display: true)
                window.makeKeyAndOrderFront(nil)
                window.orderFrontRegardless()
            }

            NSApp.activate(ignoringOtherApps: true)
        }
    }
}

extension NSWindow {
    var isCommunitasContentWindow: Bool {
        let className = NSStringFromClass(type(of: self))
        guard !className.contains("NSStatusBarWindow") else { return false }
        guard level == .normal else { return false }
        guard styleMask.contains(.titled) else { return false }
        return isVisible || canBecomeKey
    }
}

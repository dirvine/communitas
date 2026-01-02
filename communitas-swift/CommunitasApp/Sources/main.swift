import SwiftUI
import AppKit
import CommunitasAppLib

// MARK: - Root View
/// Switches between AuthenticationView and ContentView based on authentication state
struct RootView: View {
    @EnvironmentObject var appState: AppState

    var body: some View {
        Group {
            if appState.isAuthenticated {
                ContentView()
                    .transition(.opacity)
            } else {
                AuthenticationView()
                    .transition(.opacity)
            }
        }
        .animation(.easeInOut(duration: 0.3), value: appState.isAuthenticated)
    }
}

// MARK: - Window Info
/// Tracks window and its associated AppState
struct WindowInfo {
    let window: NSWindow
    let appState: AppState
}

class AppDelegate: NSObject, NSApplicationDelegate {
    var windows: [WindowInfo] = []
    private var windowCounter = 0

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Set activation policy to regular (foreground app)
        NSApp.setActivationPolicy(.regular)

        // Setup the menu bar
        setupMenuBar()

        // Create initial window
        createNewWindow()

        // Start debug server (only in DEBUG builds)
        #if DEBUG
        Task { @MainActor in
            // Start the debug server
            DebugServer.shared.start()

            // Register handlers with the first window's AppState
            if let firstWindow = windows.first {
                DebugHandlers.register(appState: firstWindow.appState)
            }
        }
        #endif
    }

    // MARK: - Menu Bar Setup

    func setupMenuBar() {
        let mainMenu = NSMenu()

        // App menu (Communitas)
        let appMenuItem = NSMenuItem()
        mainMenu.addItem(appMenuItem)
        let appMenu = NSMenu()
        appMenuItem.submenu = appMenu

        appMenu.addItem(NSMenuItem(title: "About Communitas", action: #selector(NSApplication.orderFrontStandardAboutPanel(_:)), keyEquivalent: ""))
        appMenu.addItem(NSMenuItem.separator())
        appMenu.addItem(NSMenuItem(title: "Preferences...", action: nil, keyEquivalent: ","))
        appMenu.addItem(NSMenuItem.separator())
        appMenu.addItem(NSMenuItem(title: "Hide Communitas", action: #selector(NSApplication.hide(_:)), keyEquivalent: "h"))

        let hideOthersItem = NSMenuItem(title: "Hide Others", action: #selector(NSApplication.hideOtherApplications(_:)), keyEquivalent: "h")
        hideOthersItem.keyEquivalentModifierMask = [.command, .option]
        appMenu.addItem(hideOthersItem)

        appMenu.addItem(NSMenuItem(title: "Show All", action: #selector(NSApplication.unhideAllApplications(_:)), keyEquivalent: ""))
        appMenu.addItem(NSMenuItem.separator())
        appMenu.addItem(NSMenuItem(title: "Quit Communitas", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))

        // File menu
        let fileMenuItem = NSMenuItem()
        mainMenu.addItem(fileMenuItem)
        let fileMenu = NSMenu(title: "File")
        fileMenuItem.submenu = fileMenu

        let newWindowItem = NSMenuItem(title: "New Window", action: #selector(newWindowMenuAction), keyEquivalent: "n")
        newWindowItem.target = self
        fileMenu.addItem(newWindowItem)

        fileMenu.addItem(NSMenuItem.separator())
        fileMenu.addItem(NSMenuItem(title: "Close Window", action: #selector(NSWindow.performClose(_:)), keyEquivalent: "w"))

        // Edit menu
        let editMenuItem = NSMenuItem()
        mainMenu.addItem(editMenuItem)
        let editMenu = NSMenu(title: "Edit")
        editMenuItem.submenu = editMenu

        editMenu.addItem(NSMenuItem(title: "Undo", action: Selector(("undo:")), keyEquivalent: "z"))
        editMenu.addItem(NSMenuItem(title: "Redo", action: Selector(("redo:")), keyEquivalent: "Z"))
        editMenu.addItem(NSMenuItem.separator())
        editMenu.addItem(NSMenuItem(title: "Cut", action: #selector(NSText.cut(_:)), keyEquivalent: "x"))
        editMenu.addItem(NSMenuItem(title: "Copy", action: #selector(NSText.copy(_:)), keyEquivalent: "c"))
        editMenu.addItem(NSMenuItem(title: "Paste", action: #selector(NSText.paste(_:)), keyEquivalent: "v"))
        editMenu.addItem(NSMenuItem(title: "Select All", action: #selector(NSText.selectAll(_:)), keyEquivalent: "a"))

        // Window menu
        let windowMenuItem = NSMenuItem()
        mainMenu.addItem(windowMenuItem)
        let windowMenu = NSMenu(title: "Window")
        windowMenuItem.submenu = windowMenu

        windowMenu.addItem(NSMenuItem(title: "Minimize", action: #selector(NSWindow.performMiniaturize(_:)), keyEquivalent: "m"))
        windowMenu.addItem(NSMenuItem(title: "Zoom", action: #selector(NSWindow.performZoom(_:)), keyEquivalent: ""))
        windowMenu.addItem(NSMenuItem.separator())
        windowMenu.addItem(NSMenuItem(title: "Bring All to Front", action: #selector(NSApplication.arrangeInFront(_:)), keyEquivalent: ""))

        NSApp.mainMenu = mainMenu
        NSApp.windowsMenu = windowMenu
    }

    // MARK: - Window Management

    @objc func newWindowMenuAction() {
        createNewWindow()
    }

    func createNewWindow() {
        windowCounter += 1

        // Each window gets its own AppState for independent identity
        let appState = AppState()

        // Create and configure the window with RootView
        let contentView = RootView()
            .environmentObject(appState)

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 900, height: 700),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )

        // Window title with number for multiple windows
        window.title = windowCounter == 1 ? "Communitas" : "Communitas \(windowCounter)"

        // Offset new windows slightly from existing ones
        if windowCounter > 1 {
            let offset = CGFloat((windowCounter - 1) * 30)
            window.setFrameOrigin(NSPoint(x: 100 + offset, y: 500 - offset))
        } else {
            window.center()
        }

        window.contentView = NSHostingView(rootView: contentView)
        window.makeKeyAndOrderFront(nil)

        // Store window info
        windows.append(WindowInfo(window: window, appState: appState))

        // Track window close to clean up
        NotificationCenter.default.addObserver(
            forName: NSWindow.willCloseNotification,
            object: window,
            queue: .main
        ) { [weak self] notification in
            if let closedWindow = notification.object as? NSWindow {
                self?.windows.removeAll { $0.window === closedWindow }
            }
        }

        // Activate the app
        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        return true
    }
}

// Main entry point
let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate

// Ensure app is initialized properly
app.setActivationPolicy(.regular)

// Call finishLaunching to trigger applicationDidFinishLaunching
app.finishLaunching()

// Now run the event loop
app.run()

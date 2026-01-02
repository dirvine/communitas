import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  override func applicationDidFinishLaunching(_ notification: Notification) {
    super.applicationDidFinishLaunching(notification)

    // Debug: Print window state
    print("DEBUG: MainFlutterWindow: \(String(describing: mainFlutterWindow))")
    print("DEBUG: Window frame: \(mainFlutterWindow?.frame ?? .zero)")
    print("DEBUG: Window isVisible: \(mainFlutterWindow?.isVisible ?? false)")
    print("DEBUG: Window contentView: \(String(describing: mainFlutterWindow?.contentView))")
    print("DEBUG: Window contentViewController: \(String(describing: mainFlutterWindow?.contentViewController))")

    // Force window to show
    if let window = mainFlutterWindow {
      window.setFrame(NSRect(x: 100, y: 100, width: 1200, height: 800), display: true)
      window.makeKeyAndOrderFront(nil)
      window.orderFrontRegardless()
      NSLog("Window made key and ordered front")
    } else {
      NSLog("ERROR: mainFlutterWindow is nil!")
    }

    NSApp.activate(ignoringOtherApps: true)
  }

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return true
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }
}

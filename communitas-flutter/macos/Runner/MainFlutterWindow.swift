import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    NSLog("MainFlutterWindow.awakeFromNib() - START")

    let flutterViewController = FlutterViewController()
    NSLog("MainFlutterWindow: Created FlutterViewController: \(flutterViewController)")

    let windowFrame = self.frame
    NSLog("MainFlutterWindow: Window frame before: \(windowFrame)")

    self.contentViewController = flutterViewController
    NSLog("MainFlutterWindow: Set contentViewController")

    self.setFrame(windowFrame, display: true)
    NSLog("MainFlutterWindow: Set frame")

    RegisterGeneratedPlugins(registry: flutterViewController)
    NSLog("MainFlutterWindow: Registered plugins")

    super.awakeFromNib()
    NSLog("MainFlutterWindow: super.awakeFromNib() called")

    // Explicitly make window visible
    self.makeKeyAndOrderFront(nil)
    self.orderFrontRegardless()
    NSLog("MainFlutterWindow: makeKeyAndOrderFront called")

    NSLog("MainFlutterWindow: isVisible = \(self.isVisible)")
    NSLog("MainFlutterWindow: frame = \(self.frame)")
    NSLog("MainFlutterWindow.awakeFromNib() - END")
  }
}

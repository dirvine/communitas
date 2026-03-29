import Sparkle

/// Provides the appcast feed URL to Sparkle when Info.plist doesn't contain SUFeedURL.
///
/// The CI build injects `SUFeedURL` into the bundled Info.plist, but local dev
/// builds don't have one. This delegate ensures Sparkle always has a feed URL.
private final class UpdaterDelegateHandler: NSObject, SPUUpdaterDelegate {
    /// Fallback feed URL used when Info.plist has no SUFeedURL.
    static let fallbackFeedURL = "https://github.com/saorsa-labs/communitas/releases/latest/download/appcast.xml"

    func feedURLString(for updater: SPUUpdater) -> String? {
        // If Info.plist already has SUFeedURL, Sparkle uses that and this
        // delegate method is not called. This only fires as a fallback.
        Self.fallbackFeedURL
    }
}

/// Wraps `SPUStandardUpdaterController` for use as a SwiftUI `ObservableObject`.
///
/// Create a single instance at the app level and share it via `@StateObject`.
/// Call `checkForUpdates()` to trigger an explicit user-initiated update check.
final class UpdaterController: ObservableObject {
    /// The underlying Sparkle updater.
    let updater: SPUUpdater

    private let controller: SPUStandardUpdaterController
    private let delegateHandler = UpdaterDelegateHandler()

    /// Whether the updater is ready to check for updates.
    @Published var canCheckForUpdates = false

    init() {
        let standardController = SPUStandardUpdaterController(
            startingUpdater: false,
            updaterDelegate: delegateHandler,
            userDriverDelegate: nil
        )
        self.controller = standardController
        self.updater = standardController.updater

        // Start the updater after configuring the delegate
        do {
            try updater.start()
            canCheckForUpdates = updater.canCheckForUpdates
        } catch {
            // Sparkle may fail in sandboxed or unsigned dev builds — not fatal
            print("Sparkle failed to start: \(error.localizedDescription)")
            canCheckForUpdates = false
        }
    }

    /// Triggers a user-initiated update check. Sparkle will present its standard UI.
    func checkForUpdates() {
        guard updater.canCheckForUpdates else { return }
        updater.checkForUpdates()
    }
}

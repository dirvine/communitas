import Sparkle

/// Wraps `SPUStandardUpdaterController` for use as a SwiftUI `ObservableObject`.
///
/// Create a single instance at the app level and share it via `@StateObject`.
/// Call `checkForUpdates()` to trigger an explicit user-initiated update check.
final class UpdaterController: ObservableObject {
    /// The underlying Sparkle updater.
    let updater: SPUUpdater

    private let controller: SPUStandardUpdaterController

    init() {
        let standardController = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: nil,
            userDriverDelegate: nil
        )
        self.controller = standardController
        self.updater = standardController.updater
    }

    /// Triggers a user-initiated update check. Sparkle will present its standard UI.
    func checkForUpdates() {
        updater.checkForUpdates()
    }
}

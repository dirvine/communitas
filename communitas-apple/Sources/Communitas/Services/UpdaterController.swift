import AppKit
import Foundation
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

private struct AppcastFallbackResult {
    let version: String
    let downloadURL: URL?
}

/// Wraps `SPUStandardUpdaterController` for use as a SwiftUI `ObservableObject`.
///
/// Create a single instance at the app level and share it via `@StateObject`.
/// Call `checkForUpdates()` to trigger an explicit user-initiated update check.
@MainActor
final class UpdaterController: ObservableObject {
    /// The underlying Sparkle updater.
    let updater: SPUUpdater

    private let controller: SPUStandardUpdaterController
    private let delegateHandler = UpdaterDelegateHandler()

    /// Whether the user can trigger an update check.
    ///
    /// This stays true even if Sparkle is unavailable because we can fall back to
    /// a manual appcast check instead of silently doing nothing.
    @Published var canCheckForUpdates = true

    init() {
        let standardController = SPUStandardUpdaterController(
            startingUpdater: false,
            updaterDelegate: delegateHandler,
            userDriverDelegate: nil
        )
        self.controller = standardController
        self.updater = standardController.updater

        // Disable automatic background checks — only check when user clicks
        // "Check for Updates..." in the menu. Prevents annoying "no updates"
        // dialog on launch before appcast.xml is published.
        updater.automaticallyChecksForUpdates = false

        // Start the updater after configuring the delegate.
        // If Sparkle is unavailable, keep manual checks enabled as a fallback.
        do {
            try updater.start()
        } catch {
            print("Sparkle failed to start, falling back to manual checks: \(error.localizedDescription)")
        }
    }

    /// Triggers a user-initiated update check.
    ///
    /// Prefer Sparkle when available. If Sparkle is not ready in this build,
    /// fall back to a manual appcast check so the action never becomes a silent no-op.
    func checkForUpdates() {
        if updater.canCheckForUpdates {
            updater.checkForUpdates()
            return
        }

        Task {
            await runManualFallbackCheck()
        }
    }

    private func runManualFallbackCheck() async {
        do {
            let latest = try await fetchLatestVersionFromAppcast()
            let current = Bundle.main.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
                ?? Bundle.main.object(forInfoDictionaryKey: "CFBundleVersion") as? String
                ?? "0.0.0"

            if isVersion(latest.version, newerThan: current) {
                let alert = NSAlert()
                alert.messageText = "Update Available"
                alert.informativeText = "Communitas \(latest.version) is available. You are running \(current)."
                alert.addButton(withTitle: "Download")
                alert.addButton(withTitle: "Later")
                let response = alert.runModal()
                if response == .alertFirstButtonReturn, let url = latest.downloadURL {
                    NSWorkspace.shared.open(url)
                }
            } else {
                let alert = NSAlert()
                alert.messageText = "You’re Up to Date"
                alert.informativeText = "Communitas \(current) is the latest available version."
                alert.addButton(withTitle: "OK")
                alert.runModal()
            }
        } catch {
            let alert = NSAlert()
            alert.messageText = "Update Check Failed"
            alert.informativeText = error.localizedDescription
            alert.addButton(withTitle: "OK")
            alert.runModal()
        }
    }

    private func fetchLatestVersionFromAppcast() async throws -> AppcastFallbackResult {
        guard let url = URL(string: UpdaterDelegateHandler.fallbackFeedURL) else {
            throw NSError(domain: "CommunitasUpdater", code: 1, userInfo: [
                NSLocalizedDescriptionKey: "Invalid appcast URL."
            ])
        }

        let request = URLRequest(url: url, cachePolicy: .reloadIgnoringLocalCacheData, timeoutInterval: 10)
        let (data, _) = try await URLSession.shared.data(for: request)
        guard let xml = String(data: data, encoding: .utf8) else {
            throw NSError(domain: "CommunitasUpdater", code: 2, userInfo: [
                NSLocalizedDescriptionKey: "The update feed was not valid UTF-8."
            ])
        }

        guard let version = firstMatch(in: xml, pattern: "<sparkle:shortVersionString>([^<]+)</sparkle:shortVersionString>")
            ?? firstMatch(in: xml, pattern: "<sparkle:version>([^<]+)</sparkle:version>") else {
            throw NSError(domain: "CommunitasUpdater", code: 3, userInfo: [
                NSLocalizedDescriptionKey: "The update feed did not contain a version."
            ])
        }

        let downloadURL = firstMatch(in: xml, pattern: "<enclosure[^>]*url=\"([^\"]+)\"")
            .flatMap(URL.init(string:))

        return AppcastFallbackResult(version: version, downloadURL: downloadURL)
    }

    private func firstMatch(in text: String, pattern: String) -> String? {
        guard let regex = try? NSRegularExpression(pattern: pattern) else {
            return nil
        }
        let range = NSRange(text.startIndex..<text.endIndex, in: text)
        guard let match = regex.firstMatch(in: text, range: range), match.numberOfRanges > 1,
              let captured = Range(match.range(at: 1), in: text) else {
            return nil
        }
        return String(text[captured])
    }

    private func isVersion(_ lhs: String, newerThan rhs: String) -> Bool {
        let left = lhs.split(separator: ".").map { Int($0) ?? 0 }
        let right = rhs.split(separator: ".").map { Int($0) ?? 0 }
        let count = max(left.count, right.count)
        for index in 0..<count {
            let l = index < left.count ? left[index] : 0
            let r = index < right.count ? right[index] : 0
            if l != r {
                return l > r
            }
        }
        return false
    }
}

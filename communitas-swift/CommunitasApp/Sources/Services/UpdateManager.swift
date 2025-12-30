// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

import Foundation
import os
#if os(macOS)
import Sparkle
import AppKit
#endif

/// Status of the update check and download process.
public enum UpdateStatus: Equatable {
    /// No update activity.
    case idle
    /// Checking for updates.
    case checking
    /// An update is available.
    case available(version: String)
    /// Downloading update.
    case downloading(progress: Double)
    /// Ready to install (restart required).
    case readyToInstall(version: String)
    /// Update failed.
    case failed(error: String)
}

/// Service for managing application updates using Sparkle.
///
/// This service handles:
/// - Automatic update checks on startup
/// - Manual update checks
/// - Silent background downloads
/// - User notification when restart is required
@MainActor
public final class UpdateManager: NSObject, ObservableObject {

    // MARK: - Published Properties

    @Published public private(set) var updateStatus: UpdateStatus = .idle
    @Published public private(set) var canCheckForUpdates: Bool = true
    @Published public private(set) var lastUpdateCheck: Date?

    // MARK: - Private Properties

    #if os(macOS)
    private var updaterController: SPUStandardUpdaterController?
    private let logger = Logger(subsystem: "com.saorsalabs.communitas", category: "UpdateManager")
    #endif

    // MARK: - Singleton

    public static let shared = UpdateManager()

    private override init() {
        super.init()

        #if os(macOS)
        setupSparkle()
        #endif
    }

    // MARK: - Setup

    #if os(macOS)
    private func setupSparkle() {
        // Create the updater controller with standard configuration
        // The controller handles creating the SPUUpdater and managing its lifecycle
        updaterController = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: self,
            userDriverDelegate: nil
        )

        // Configure automatic update behavior
        if let updater = updaterController?.updater {
            // Check for updates automatically
            updater.automaticallyChecksForUpdates = true

            // Check once per day
            updater.updateCheckInterval = 24 * 60 * 60 // 24 hours

            // Download updates automatically in the background
            updater.automaticallyDownloadsUpdates = true

            logger.info("Sparkle updater configured successfully")
        }
    }
    #endif

    // MARK: - Public Methods

    /// Manually check for updates.
    public func checkForUpdates() {
        #if os(macOS)
        guard canCheckForUpdates else {
            logger.warning("Update check already in progress")
            return
        }

        updateStatus = .checking
        updaterController?.checkForUpdates(nil)
        logger.info("Manual update check initiated")
        #endif
    }

    /// Check for updates in the background (no UI unless update available).
    public func checkForUpdatesInBackground() {
        #if os(macOS)
        updaterController?.updater.checkForUpdatesInBackground()
        logger.info("Background update check initiated")
        #endif
    }

    /// Cancel an in-progress update download.
    public func cancelUpdate() {
        #if os(macOS)
        // Sparkle doesn't expose a direct cancel method for downloads
        // Reset status instead
        updateStatus = .idle
        logger.info("Update cancelled by user")
        #endif
    }

    /// Dismiss the update notification.
    public func dismissUpdate() {
        updateStatus = .idle
    }

    /// Open the release notes URL.
    public func openReleaseNotes() {
        #if os(macOS)
        // Sparkle handles release notes in its own UI
        // If we need custom handling, we can fetch from GitHub releases
        if let url = URL(string: "https://github.com/saorsa-labs/communitas/releases") {
            NSWorkspace.shared.open(url)
        }
        #endif
    }
}

// MARK: - SPUUpdaterDelegate

#if os(macOS)
extension UpdateManager: SPUUpdaterDelegate {

    public nonisolated func updater(_ updater: SPUUpdater, didFinishLoading appcast: SUAppcast) {
        Task { @MainActor in
            logger.debug("Appcast loaded successfully")
        }
    }

    public nonisolated func updater(_ updater: SPUUpdater, didFindValidUpdate item: SUAppcastItem) {
        Task { @MainActor in
            let version = item.displayVersionString
            updateStatus = .available(version: version)
            logger.info("Update available: \(version)")
        }
    }

    public nonisolated func updaterDidNotFindUpdate(_ updater: SPUUpdater, error: any Error) {
        Task { @MainActor in
            updateStatus = .idle
            lastUpdateCheck = Date()
            logger.info("No update available: \(error.localizedDescription)")
        }
    }

    public nonisolated func updater(_ updater: SPUUpdater, failedToDownloadUpdate item: SUAppcastItem, error: any Error) {
        Task { @MainActor in
            updateStatus = .failed(error: error.localizedDescription)
            logger.error("Failed to download update: \(error.localizedDescription)")
        }
    }

    public nonisolated func updater(_ updater: SPUUpdater, willDownloadUpdate item: SUAppcastItem, with request: NSMutableURLRequest) {
        Task { @MainActor in
            updateStatus = .downloading(progress: 0)
            logger.info("Starting download for version \(item.displayVersionString)")
        }
    }

    public nonisolated func updater(_ updater: SPUUpdater, didDownloadUpdate item: SUAppcastItem) {
        Task { @MainActor in
            let version = item.displayVersionString
            updateStatus = .readyToInstall(version: version)
            logger.info("Update downloaded: \(version)")
        }
    }

    public nonisolated func updater(_ updater: SPUUpdater, willInstallUpdate item: SUAppcastItem) {
        Task { @MainActor in
            logger.info("Installing update: \(item.displayVersionString)")
        }
    }

    public nonisolated func updater(_ updater: SPUUpdater, didAbortWithError error: any Error) {
        Task { @MainActor in
            updateStatus = .failed(error: error.localizedDescription)
            logger.error("Update aborted: \(error.localizedDescription)")
        }
    }

    public nonisolated func updaterWillRelaunchApplication(_ updater: SPUUpdater) {
        Task { @MainActor in
            logger.info("Application will relaunch after update")
        }
    }

    // MARK: - Allowed Channels (for beta updates)

    public nonisolated func allowedChannels(for updater: SPUUpdater) -> Set<String> {
        // Return empty set for stable releases only
        // Add "beta" for beta channel access
        return Set()
    }
}
#endif

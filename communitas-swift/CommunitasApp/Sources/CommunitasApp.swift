import SwiftUI

// Note: This file is required by the Xcode project but the actual app entry point
// is in main.swift which uses AppDelegate pattern for macOS.
// This file contains placeholder/shared code for both iOS and macOS targets.

// MARK: - App Constants

enum AppConstants {
    static let appName = "Communitas"
    static let bundleIdentifier = "com.communitas.app"
    static let defaultBootstrapAddress = "138.197.29.195:4433"
}

// MARK: - Preview Provider for main content

#Preview("Main App") {
    ContentView()
        .environmentObject(AppState())
}

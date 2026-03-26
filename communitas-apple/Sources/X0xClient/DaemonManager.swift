import Foundation

/// Manages the lifecycle of the x0x daemon (x0xd) process.
public final class DaemonManager: Sendable {
    private let client: X0xClient

    /// Common install locations for the x0xd binary.
    private static let searchPaths: [String] = [
        "/usr/local/bin/x0xd",
        "/opt/homebrew/bin/x0xd",
        "/opt/zerobrew/bin/x0xd",
        "\(NSHomeDirectory())/.cargo/bin/x0xd",
        "\(NSHomeDirectory())/.x0x/bin/x0xd",
    ]

    public init(client: X0xClient = X0xClient()) {
        self.client = client
    }

    /// Probe the daemon to determine its current state.
    public func state() async -> DaemonState {
        do {
            let health = try await client.health()
            if health.status == "ok" || health.status == "healthy" {
                return .running
            }
            return .error
        } catch {
            if isInstalled() {
                return .notRunning
            }
            return .notInstalled
        }
    }

    /// Check whether the x0xd binary exists on disk.
    public func isInstalled() -> Bool {
        return binaryPath() != nil
    }

    /// Find the x0xd binary path.
    public func binaryPath() -> String? {
        for path in Self.searchPaths {
            if FileManager.default.isExecutableFile(atPath: path) {
                return path
            }
        }
        return nil
    }

    /// Start the daemon process.
    public func start() async throws {
        guard let path = binaryPath() else {
            throw X0xError.daemonNotInstalled
        }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: path)
        process.arguments = ["--daemon"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice

        do {
            try process.run()
        } catch {
            throw X0xError.daemonStartFailed(reason: error.localizedDescription)
        }

        // Wait a moment for the daemon to initialize, then verify.
        try await Task.sleep(nanoseconds: 2_000_000_000) // 2 seconds

        let currentState = await state()
        if currentState != .running {
            throw X0xError.daemonStartFailed(reason: "Daemon started but health check failed")
        }
    }

    /// Ensure the daemon is running. Starts it if not already running.
    public func ensureRunning() async throws {
        let currentState = await state()
        switch currentState {
        case .running:
            return
        case .notInstalled:
            throw X0xError.daemonNotInstalled
        case .notRunning, .starting, .error:
            try await start()
        }
    }
}

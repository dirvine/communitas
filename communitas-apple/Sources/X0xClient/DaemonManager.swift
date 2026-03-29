import Foundation

/// Manages the lifecycle of the x0x daemon process.
public final class DaemonManager: Sendable {
    private let client: X0xClient

    /// Common install locations for the x0x binary.
    private static let searchPaths: [String] = [
        "/usr/local/bin/x0x",
        "/opt/homebrew/bin/x0x",
        "/opt/zerobrew/bin/x0x",
        "\(NSHomeDirectory())/.cargo/bin/x0x",
        "\(NSHomeDirectory())/.x0x/bin/x0x",
    ]

    public init(client: X0xClient = X0xClient()) {
        self.client = client
    }

    /// Probe the daemon to determine its current state using the instance's default client.
    public func state() async -> DaemonState {
        await state(using: client)
    }

    /// Probe the daemon to determine its current state using a provided client.
    ///
    /// Use this overload when the caller holds an up-to-date authenticated ``X0xClient``
    /// (e.g., one built from a freshly discovered ``X0xConfig``).
    public func state(using overrideClient: X0xClient) async -> DaemonState {
        do {
            let health = try await overrideClient.health()
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

    /// Check whether the x0x binary exists on disk.
    public func isInstalled() -> Bool {
        return binaryPath() != nil
    }

    /// Find the x0x binary path.
    public func binaryPath() -> String? {
        for path in Self.searchPaths {
            if FileManager.default.isExecutableFile(atPath: path) {
                return path
            }
        }
        return nil
    }

    /// Start the daemon process using `x0x start`.
    public func start() async throws {
        guard let path = binaryPath() else {
            throw X0xError.daemonNotInstalled
        }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: path)
        process.arguments = ["start"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice

        do {
            try process.run()
        } catch {
            throw X0xError.daemonStartFailed(reason: error.localizedDescription)
        }

        // Wait a moment for the daemon to initialize, then verify.
        // Use a freshly discovered client so the health check goes to the correct port
        // and carries the bearer token written by the newly started daemon.
        try await Task.sleep(nanoseconds: 2_000_000_000) // 2 seconds

        let probeClient = X0xClient.fromDiscovery() ?? client
        let currentState = await state(using: probeClient)
        if currentState != .running {
            throw X0xError.daemonStartFailed(reason: "Daemon started but health check failed")
        }
    }

    /// Ensure the daemon is running. If not running, starts it once.
    /// Does not configure autostart -- the user must enable that from Settings.
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

    /// Configure x0x to start automatically on boot/login.
    ///
    /// Runs: `x0x autostart`
    public func ensureAutostart() async throws {
        guard let path = binaryPath() else {
            throw X0xError.daemonNotInstalled
        }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: path)
        process.arguments = ["autostart"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice

        do {
            try process.run()
            process.waitUntilExit()
            if process.terminationStatus != 0 {
                throw X0xError.daemonStartFailed(reason: "autostart exited with code \(process.terminationStatus)")
            }
        } catch let error as X0xError {
            throw error
        } catch {
            throw X0xError.daemonStartFailed(reason: "autostart failed: \(error.localizedDescription)")
        }
    }

    /// Stop the daemon process using `x0x stop`.
    public func stop() async throws {
        guard let path = binaryPath() else {
            throw X0xError.daemonNotInstalled
        }

        let process = Process()
        process.executableURL = URL(fileURLWithPath: path)
        process.arguments = ["stop"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice

        do {
            try process.run()
            process.waitUntilExit()
        } catch {
            throw X0xError.daemonStartFailed(reason: "stop failed: \(error.localizedDescription)")
        }
    }
}

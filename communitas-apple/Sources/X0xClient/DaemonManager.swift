import Foundation
import CryptoKit

/// Manages the lifecycle of the x0x daemon process.
public final class DaemonManager: Sendable {
    private let client: X0xClient

    /// Common install locations for the x0x binary.
    private static let searchPaths: [String] = [
        "/usr/local/bin/x0x",
        "/opt/homebrew/bin/x0x",
        "/opt/zerobrew/bin/x0x",
        "\(NSHomeDirectory())/.local/bin/x0x",
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

    /// Verify the installed `x0x` binary matches the SHA-256 published in the
    /// signed release manifest at
    /// `https://github.com/saorsa-labs/x0x/releases/latest/download/release-manifest.json`.
    ///
    /// Throws ``X0xError.daemonStartFailed`` with a descriptive reason when
    /// the manifest cannot be fetched, the platform is unknown, or the hash
    /// does not match. Mirrors the Rust-side `DaemonManager::verify_installed_binary`
    /// in `communitas-x0x-client`.
    public func verifyInstalledBinary() async throws {
        guard let path = binaryPath() else {
            throw X0xError.daemonNotInstalled
        }
        guard let target = Self.currentPlatformTarget() else {
            throw X0xError.daemonStartFailed(reason: "no platform mapping for current OS/arch")
        }
        let manifestURL = URL(string: "https://github.com/saorsa-labs/x0x/releases/latest/download/release-manifest.json")!
        let (data, response) = try await URLSession.shared.data(from: manifestURL)
        if let http = response as? HTTPURLResponse, http.statusCode != 200 {
            throw X0xError.daemonStartFailed(reason: "release manifest fetch returned HTTP \(http.statusCode)")
        }

        struct ManifestLite: Decodable {
            struct Asset: Decodable {
                let target: String
                let archive_sha256: String
            }
            let assets: [Asset]
        }
        let manifest: ManifestLite
        do {
            manifest = try JSONDecoder().decode(ManifestLite.self, from: data)
        } catch {
            throw X0xError.daemonStartFailed(reason: "manifest JSON parse failed: \(error.localizedDescription)")
        }
        guard let asset = manifest.assets.first(where: { $0.target == target }) else {
            throw X0xError.daemonStartFailed(reason: "manifest has no asset for target \(target)")
        }

        let expected = asset.archive_sha256.lowercased()
        let actual = try Self.sha256Hex(atPath: path)
        guard actual == expected else {
            throw X0xError.daemonStartFailed(
                reason: "x0x binary SHA-256 mismatch: expected \(expected), got \(actual) (target \(target))"
            )
        }
    }

    /// Map the current host to the release manifest's target-triple identifier.
    static func currentPlatformTarget() -> String? {
        #if os(macOS) && arch(arm64)
        return "aarch64-apple-darwin"
        #elseif os(macOS) && arch(x86_64)
        return "x86_64-apple-darwin"
        #elseif os(Linux) && arch(arm64)
        return "aarch64-unknown-linux-gnu"
        #elseif os(Linux) && arch(x86_64)
        return "x86_64-unknown-linux-gnu"
        #else
        return nil
        #endif
    }

    /// Stream a file through CryptoKit's SHA256 and return the hex digest.
    static func sha256Hex(atPath path: String) throws -> String {
        guard let handle = FileHandle(forReadingAtPath: path) else {
            throw X0xError.daemonStartFailed(reason: "cannot open \(path)")
        }
        defer { try? handle.close() }
        var hasher = SHA256()
        while true {
            let chunk = handle.readData(ofLength: 64 * 1024)
            if chunk.isEmpty { break }
            hasher.update(data: chunk)
        }
        return hasher.finalize().map { String(format: "%02x", $0) }.joined()
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

import Foundation
@testable import X0xClient

/// Per-test x0xd daemon fixture for Swift round-trip integration tests.
///
/// Mirrors the Rust harness at `x0x/tests/harness/src/daemon.rs`: launches a
/// fresh `x0xd` process bound to ephemeral ports, with an isolated temp data
/// dir, an instance-scoped identity dir, an empty bootstrap list and update
/// checks disabled. The fixture reads the daemon's `api.port` and
/// `api-token` runtime files and exposes them on the constructed
/// ``X0xClient`` so tests can talk to the live daemon without colliding
/// with any other instance on the host.
///
/// Tests that use this fixture should be guarded with the
/// ``DaemonFixture/liveTestsEnabled`` flag so `swift test` without an
/// explicit opt-in stays as the existing decoder-only smoke pass:
///
/// ```swift
/// guard DaemonFixture.liveTestsEnabled else { return }
/// let daemon = try await DaemonFixture.start(prefix: "identity")
/// defer { daemon.terminate() }
/// // … exercise daemon.client …
/// ```
final class DaemonFixture {

    /// HTTP base URL for the daemon, e.g. `http://127.0.0.1:54321`.
    let baseURL: URL
    /// Bearer token written by the daemon to `<data_dir>/api-token`.
    let token: String
    /// `X0xClient` pre-configured with `baseURL` and `token`.
    let client: X0xClient
    /// Temp data dir owned by this fixture (`api.port` and `api-token`
    /// land here; cleaned up on ``terminate()``).
    let dataDir: URL
    /// Per-instance identity dir under `~/.x0x-<name>`. Cleaned up on
    /// ``terminate()`` so back-to-back tests do not leak machine keys.
    let identityDir: URL

    private let process: Process
    private let instanceName: String
    private var terminated = false

    private init(
        process: Process,
        baseURL: URL,
        token: String,
        dataDir: URL,
        identityDir: URL,
        instanceName: String
    ) {
        self.process = process
        self.baseURL = baseURL
        self.token = token
        self.client = X0xClient(baseURL: baseURL, token: token)
        self.dataDir = dataDir
        self.identityDir = identityDir
        self.instanceName = instanceName
    }

    deinit {
        if !terminated {
            // Best-effort cleanup — defensive only; tests should call
            // ``terminate()`` explicitly so the process is reaped before
            // the suite tears down.
            process.terminate()
            try? FileManager.default.removeItem(at: dataDir)
            try? FileManager.default.removeItem(at: identityDir)
        }
    }

    // MARK: - Lifecycle

    /// Launches an `x0xd` process and waits for it to be ready.
    ///
    /// `prefix` distinguishes parallel fixtures — the final instance name
    /// is `<prefix>-<random>`. The launch fails fast if the binary cannot
    /// be located, the config cannot be written, the API port file does
    /// not appear within 30 s, or `/health` does not return 200 within
    /// 30 s of the port being known.
    static func start(prefix: String, extraConfig: String = "") async throws -> DaemonFixture {
        let binary = try locateBinary()
        let instanceName = "\(prefix)-\(UInt32.random(in: 1...UInt32.max))"

        let dataDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("x0x-fixture-\(instanceName)", isDirectory: true)
        try FileManager.default.createDirectory(at: dataDir, withIntermediateDirectories: true)

        let configPath = dataDir.appendingPathComponent("config.toml")
        let extraHasBootstrap = extraConfig
            .split(whereSeparator: { $0 == "\n" })
            .contains { $0.trimmingCharacters(in: .whitespaces).hasPrefix("bootstrap_peers") }
        let bootstrapLine = extraHasBootstrap ? "" : "bootstrap_peers = []\n"

        var config = """
        bind_address = "127.0.0.1:0"
        api_address = "127.0.0.1:0"
        data_dir = "\(dataDir.path)"
        log_level = "warn"
        \(bootstrapLine)instance_name = "\(instanceName)"

        """
        if !extraConfig.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            config += extraConfig
            if !extraConfig.hasSuffix("\n") {
                config += "\n"
            }
        }
        try config.write(to: configPath, atomically: true, encoding: .utf8)

        let identityDir = URL(fileURLWithPath: NSHomeDirectory())
            .appendingPathComponent(".x0x-\(instanceName)", isDirectory: true)

        let process = Process()
        process.executableURL = binary
        process.arguments = [
            "--config", configPath.path,
            "--skip-update-check",
        ]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice
        try process.run()

        let portFile = dataDir.appendingPathComponent("api.port")
        let tokenFile = dataDir.appendingPathComponent("api-token")

        do {
            let baseURL = try await waitForBaseURL(portFile: portFile, deadline: 30.0)
            try await waitForHealth(baseURL: baseURL, deadline: 30.0)
            let token = try await waitForToken(tokenFile: tokenFile, deadline: 5.0)

            return DaemonFixture(
                process: process,
                baseURL: baseURL,
                token: token,
                dataDir: dataDir,
                identityDir: identityDir,
                instanceName: instanceName
            )
        } catch {
            process.terminate()
            try? FileManager.default.removeItem(at: dataDir)
            try? FileManager.default.removeItem(at: identityDir)
            throw error
        }
    }

    /// Stops the daemon and removes all temp state. Idempotent.
    func terminate() {
        guard !terminated else { return }
        terminated = true
        process.terminate()
        // Give the process a brief moment to exit cleanly before tearing
        // down its data dir; if it lingers the data-dir removal will
        // still succeed because the open files don't hold the directory.
        process.waitUntilExit()
        try? FileManager.default.removeItem(at: dataDir)
        try? FileManager.default.removeItem(at: identityDir)
    }

    // MARK: - Test gating

    /// Live-daemon tests run only when `X0X_LIVE_TESTS=1` is set in the
    /// environment. This keeps `swift test` (which the package CI runs)
    /// as the existing decoder-only smoke pass.
    static var liveTestsEnabled: Bool {
        ProcessInfo.processInfo.environment["X0X_LIVE_TESTS"] == "1"
    }

    // MARK: - Internals

    /// Find an `x0xd` binary. Resolution order:
    /// 1. `X0XD_BIN` env var
    /// 2. `../x0x/target/release/x0xd` relative to the package root
    /// 3. `../x0x/target/debug/x0xd`
    /// 4. Anything `DaemonManager` already knows about
    private static func locateBinary() throws -> URL {
        if let override = ProcessInfo.processInfo.environment["X0XD_BIN"], !override.isEmpty {
            let url = URL(fileURLWithPath: override)
            if FileManager.default.isExecutableFile(atPath: url.path) {
                return url
            }
            throw FixtureError.binaryNotFound(
                "X0XD_BIN=\(override) is not an executable file"
            )
        }

        let communitasRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent() // Helpers
            .deletingLastPathComponent() // X0xClientTests
            .deletingLastPathComponent() // Tests
            .deletingLastPathComponent() // communitas-apple
            .deletingLastPathComponent() // communitas
        let workspaceRoot = communitasRoot.deletingLastPathComponent()
        let candidates = [
            communitasRoot.appendingPathComponent("x0x/target/release/x0xd"),
            communitasRoot.appendingPathComponent("x0x/target/debug/x0xd"),
            workspaceRoot.appendingPathComponent("x0x/target/release/x0xd"),
            workspaceRoot.appendingPathComponent("x0x/target/debug/x0xd"),
        ]
        for c in candidates {
            if FileManager.default.isExecutableFile(atPath: c.path) {
                return c
            }
        }
        if let fromManager = DaemonManager().binaryPath() {
            return URL(fileURLWithPath: fromManager)
        }

        throw FixtureError.binaryNotFound(
            "Could not locate x0xd. Build with `cargo build --release --bin x0xd` in ../x0x or set X0XD_BIN."
        )
    }

    private static func waitForBaseURL(portFile: URL, deadline: TimeInterval) async throws -> URL {
        let start = Date()
        while Date().timeIntervalSince(start) < deadline {
            if let raw = try? String(contentsOf: portFile, encoding: .utf8) {
                let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
                if !trimmed.isEmpty {
                    // Daemon writes either `host:port` or just `port`.
                    let address: String
                    if trimmed.contains(":") {
                        address = trimmed
                    } else if UInt16(trimmed) != nil {
                        address = "127.0.0.1:\(trimmed)"
                    } else {
                        address = ""
                    }
                    if !address.isEmpty,
                       let url = URL(string: "http://\(address)") {
                        return url
                    }
                }
            }
            try await Task.sleep(nanoseconds: 200_000_000)
        }
        throw FixtureError.startupTimeout("api.port file did not appear at \(portFile.path)")
    }

    private static func waitForHealth(baseURL: URL, deadline: TimeInterval) async throws {
        let start = Date()
        let healthURL = baseURL.appendingPathComponent("health")
        let session = URLSession(configuration: .ephemeral)
        while Date().timeIntervalSince(start) < deadline {
            do {
                let (_, response) = try await session.data(from: healthURL)
                if let http = response as? HTTPURLResponse, http.statusCode == 200 {
                    return
                }
            } catch {
                // Swallow until the deadline — daemon is still booting.
            }
            try await Task.sleep(nanoseconds: 500_000_000)
        }
        throw FixtureError.startupTimeout("/health did not return 200 within \(Int(deadline))s")
    }

    private static func waitForToken(tokenFile: URL, deadline: TimeInterval) async throws -> String {
        let start = Date()
        while Date().timeIntervalSince(start) < deadline {
            if let raw = try? String(contentsOf: tokenFile, encoding: .utf8) {
                let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
                if !trimmed.isEmpty {
                    return trimmed
                }
            }
            try await Task.sleep(nanoseconds: 100_000_000)
        }
        throw FixtureError.startupTimeout("api-token file did not appear at \(tokenFile.path)")
    }
}

/// Errors thrown by ``DaemonFixture/start(prefix:extraConfig:)``.
enum FixtureError: Error, CustomStringConvertible {
    case binaryNotFound(String)
    case startupTimeout(String)

    var description: String {
        switch self {
        case .binaryNotFound(let m): return "DaemonFixture: binary not found — \(m)"
        case .startupTimeout(let m): return "DaemonFixture: startup timeout — \(m)"
        }
    }
}

import SwiftUI
import X0xClient

// MARK: - State

enum OnboardingState {
    case checking
    case notInstalled
    case notRunning
    case installing
    case starting
    case failed(String)
    case cancelled
    case ready
}

// MARK: - ViewModel

@MainActor
final class OnboardingViewModel: ObservableObject {
    @Published var state: OnboardingState = .checking

    private let daemon = DaemonManager()

    init() {}

    func checkDaemon() async {
        state = .checking
        let daemonState = await daemon.state()
        switch daemonState {
        case .running:
            state = .ready
        case .notInstalled:
            state = .notInstalled
        case .notRunning, .starting, .error:
            state = .notRunning
        }
    }

    func install() async {
        state = .installing
        do {
            try await runShellCommand("/bin/sh", arguments: ["-c", "curl -sfL https://x0x.md | sh"])
            await startDaemon()
        } catch {
            state = .failed(error.localizedDescription)
        }
    }

    func startDaemon() async {
        state = .starting
        do {
            // If binary is already available, use DaemonManager.start()
            if daemon.isInstalled() {
                try await daemon.start()
                // Enable autostart so x0xd launches on login
                try? await daemon.ensureAutostart()
                // Poll until healthy with config re-discovery
                if await pollUntilHealthy(timeoutSecs: 30) {
                    state = .ready
                } else {
                    state = .failed("x0x started but did not become healthy within 30 seconds. Try restarting Communitas.")
                }
            } else {
                state = .failed("x0x binary not found after installation. Please try again.")
            }
        } catch {
            state = .failed(error.localizedDescription)
        }
    }

    /// Poll the daemon health endpoint, re-discovering config each time.
    private func pollUntilHealthy(timeoutSecs: Int) async -> Bool {
        let deadline = Date().addingTimeInterval(TimeInterval(timeoutSecs))
        while Date() < deadline {
            // Re-discover config on each poll (daemon may have just written api.port)
            if let client = X0xClient.fromDiscovery() {
                if let _ = try? await client.health() {
                    return true
                }
            }
            try? await Task.sleep(for: .milliseconds(500))
        }
        return false
    }

    func retryCheck() async {
        await checkDaemon()
    }

    // MARK: - Private helpers

    private func runShellCommand(_ executable: String, arguments: [String]) async throws {
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            let process = Process()
            process.executableURL = URL(fileURLWithPath: executable)
            process.arguments = arguments
            process.standardOutput = FileHandle.nullDevice
            process.standardError = FileHandle.nullDevice

            process.terminationHandler = { proc in
                if proc.terminationStatus == 0 {
                    continuation.resume()
                } else {
                    continuation.resume(throwing: OnboardingError.commandFailed(exitCode: proc.terminationStatus))
                }
            }

            do {
                try process.run()
            } catch {
                continuation.resume(throwing: error)
            }
        }
    }
}

// MARK: - Errors

enum OnboardingError: Error, LocalizedError {
    case commandFailed(exitCode: Int32)

    var errorDescription: String? {
        switch self {
        case .commandFailed(let code):
            return "Installation command failed with exit code \(code). Please try installing manually."
        }
    }
}

// MARK: - OnboardingView

struct OnboardingView<Content: View>: View {
    @StateObject private var viewModel = OnboardingViewModel()
    @ViewBuilder let content: () -> Content

    var body: some View {
        Group {
            switch viewModel.state {
            case .ready:
                content()
            default:
                OnboardingGateView(viewModel: viewModel)
            }
        }
        .task {
            await viewModel.checkDaemon()
        }
    }
}

// MARK: - OnboardingGateView

private struct OnboardingGateView: View {
    @ObservedObject var viewModel: OnboardingViewModel

    var body: some View {
        ZStack {
            Color(NSColor.windowBackgroundColor)
                .ignoresSafeArea()

            switch viewModel.state {
            case .checking:
                CheckingView()
            case .notInstalled:
                NotInstalledView(viewModel: viewModel)
            case .notRunning:
                NotRunningView(viewModel: viewModel)
            case .installing:
                ProgressGateView(message: "Installing x0x...")
            case .starting:
                ProgressGateView(message: "Starting x0x...")
            case .failed(let message):
                FailedView(message: message, viewModel: viewModel)
            case .cancelled:
                CancelledView(viewModel: viewModel)
            case .ready:
                EmptyView()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - Checking

private struct CheckingView: View {
    var body: some View {
        VStack(spacing: 20) {
            ProgressView()
                .scaleEffect(1.5)
            Text("Checking x0x status...")
                .font(.body)
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Not Installed

private struct NotInstalledView: View {
    @ObservedObject var viewModel: OnboardingViewModel

    var body: some View {
        VStack(spacing: 32) {
            Spacer()

            Image(systemName: "antenna.radiowaves.left.and.right")
                .font(.system(size: 60))
                .foregroundStyle(Color.accentColor)

            VStack(spacing: 12) {
                Text("Welcome to Communitas")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("Communitas needs x0x to connect to the\ndecentralized network.")
                    .font(.body)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }

            HStack(spacing: 16) {
                Button("Cancel") {
                    viewModel.state = .cancelled
                }
                .buttonStyle(.bordered)

                Button("Install x0x") {
                    Task { await viewModel.install() }
                }
                .buttonStyle(.borderedProminent)
            }

            Spacer()
        }
        .padding(48)
        .frame(maxWidth: 520)
    }
}

// MARK: - Not Running

private struct NotRunningView: View {
    @ObservedObject var viewModel: OnboardingViewModel

    var body: some View {
        VStack(spacing: 32) {
            Spacer()

            Image(systemName: "power")
                .font(.system(size: 60))
                .foregroundStyle(Color.accentColor)

            VStack(spacing: 12) {
                Text("Starting x0x...")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("x0x is installed but not currently running.")
                    .font(.body)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }

            HStack(spacing: 16) {
                Button("Quit") {
                    NSApplication.shared.terminate(nil)
                }
                .buttonStyle(.bordered)

                Button("Start x0x") {
                    Task { await viewModel.startDaemon() }
                }
                .buttonStyle(.borderedProminent)
            }

            Spacer()
        }
        .padding(48)
        .frame(maxWidth: 520)
    }
}

// MARK: - Progress (installing / starting)

private struct ProgressGateView: View {
    let message: String

    var body: some View {
        VStack(spacing: 20) {
            ProgressView()
                .scaleEffect(1.5)
            Text(message)
                .font(.body)
                .foregroundStyle(.secondary)
        }
    }
}

// MARK: - Failed

private struct FailedView: View {
    let message: String
    @ObservedObject var viewModel: OnboardingViewModel

    var body: some View {
        VStack(spacing: 32) {
            Spacer()

            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 60))
                .foregroundStyle(.red)

            VStack(spacing: 12) {
                Text("Something went wrong")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text(message)
                    .font(.body)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }

            HStack(spacing: 16) {
                Button("Quit") {
                    NSApplication.shared.terminate(nil)
                }
                .buttonStyle(.bordered)

                Button("Try Again") {
                    Task { await viewModel.retryCheck() }
                }
                .buttonStyle(.borderedProminent)
            }

            Spacer()
        }
        .padding(48)
        .frame(maxWidth: 520)
    }
}

// MARK: - Cancelled

private struct CancelledView: View {
    @ObservedObject var viewModel: OnboardingViewModel

    private let installCommand = "curl -sfL https://x0x.md | sh"

    var body: some View {
        VStack(spacing: 32) {
            Spacer()

            Image(systemName: "exclamationmark.circle")
                .font(.system(size: 60))
                .foregroundStyle(.orange)

            VStack(spacing: 12) {
                Text("x0x is required")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("Communitas requires x0x to function. Please install\nit and restart Communitas when you're ready.")
                    .font(.body)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }

            VStack(alignment: .leading, spacing: 8) {
                Text("Install manually:")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                TextField("", text: .constant(installCommand))
                    .font(.system(.body, design: .monospaced))
                    .textFieldStyle(.roundedBorder)
                    .textSelection(.enabled)
                    .frame(maxWidth: 400)
                    .disabled(true)
            }

            HStack(spacing: 16) {
                Button("Quit") {
                    NSApplication.shared.terminate(nil)
                }
                .buttonStyle(.bordered)

                Button("Try Again") {
                    Task { await viewModel.retryCheck() }
                }
                .buttonStyle(.borderedProminent)
            }

            Spacer()
        }
        .padding(48)
        .frame(maxWidth: 520)
    }
}

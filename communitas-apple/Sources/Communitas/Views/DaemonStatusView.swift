import SwiftUI
import X0xClient

struct DaemonStatusView: View {
    @EnvironmentObject var appState: AppState
    @State private var isRefreshing = false

    var body: some View {
        VStack(spacing: 24) {
            statusHeader
            agentInfo
            Spacer()
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    Task {
                        isRefreshing = true
                        await appState.refresh()
                        isRefreshing = false
                    }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(isRefreshing)
            }
        }
    }

    @ViewBuilder
    private var statusHeader: some View {
        GroupBox {
            VStack(spacing: 12) {
                HStack {
                    statusIndicator
                    Text(statusLabel)
                        .font(.headline)
                    Spacer()
                }

                if let errorMessage = appState.errorMessage {
                    Text(errorMessage)
                        .font(.caption)
                        .foregroundStyle(.red)
                }

                if appState.daemonState != .running {
                    HStack {
                        Button("Start Daemon") {
                            Task { await appState.startDaemon() }
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(appState.daemonState == .starting)

                        if appState.daemonState == .starting {
                            ProgressView()
                                .controlSize(.small)
                        }
                    }
                }
            }
            .padding(8)
        } label: {
            Label("x0x Daemon", systemImage: "server.rack")
        }
    }

    @ViewBuilder
    private var agentInfo: some View {
        if let identity = appState.agentIdentity {
            GroupBox {
                VStack(alignment: .leading, spacing: 8) {
                    LabeledContent("Agent ID") {
                        Text(identity.agentId)
                            .font(.system(.body, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }

                    if let machineId = identity.machineId {
                        LabeledContent("Machine ID") {
                            Text(machineId)
                                .font(.system(.body, design: .monospaced))
                                .textSelection(.enabled)
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                    }

                    LabeledContent("Contacts") {
                        Text("\(appState.contacts.count)")
                    }

                    LabeledContent("Groups") {
                        Text("\(appState.groups.count)")
                    }
                }
                .padding(8)
            } label: {
                Label("Identity", systemImage: "person.badge.key")
            }
        }
    }

    private var statusIndicator: some View {
        Circle()
            .fill(statusColor)
            .frame(width: 12, height: 12)
    }

    private var statusColor: Color {
        switch appState.daemonState {
        case .running: return .green
        case .starting: return .yellow
        case .notRunning: return .gray
        case .notInstalled: return .red
        case .error: return .red
        }
    }

    private var statusLabel: String {
        switch appState.daemonState {
        case .running: return "Running"
        case .starting: return "Starting..."
        case .notRunning: return "Not Running"
        case .notInstalled: return "Not Installed"
        case .error: return "Error"
        }
    }
}

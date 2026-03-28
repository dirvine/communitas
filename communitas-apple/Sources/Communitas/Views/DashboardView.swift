import SwiftUI
import X0xClient

/// Dashboard view with stats grid, identity card, quick actions, and discovered agents.
struct DashboardView: View {
    @EnvironmentObject var appState: AppState

    @State private var daemonStatus: DaemonStatus?
    @State private var healthStatus: HealthStatus?
    @State private var discoveredAgents: [DiscoveredAgent] = []
    @State private var isRefreshing = false

    private let columns = Array(repeating: GridItem(.flexible(), spacing: 12), count: 4)

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                statsGrid
                identityCard
                quickActions
                agentsList
            }
            .padding(24)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(DeepSpace.bg)
        .navigationTitle("Dashboard")
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    Task { await pollData() }
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(isRefreshing)
            }
        }
        .task {
            await pollData()
            startPolling()
        }
    }

    // MARK: - Stats Grid

    private var statsGrid: some View {
        LazyVGrid(columns: columns, spacing: 12) {
            StatCard(
                title: "Status",
                value: appState.daemonState == .running ? "Online" : "Offline",
                icon: "circle.fill",
                color: appState.daemonState == .running ? DeepSpace.green : DeepSpace.red
            )
            StatCard(
                title: "Version",
                value: healthStatus?.version ?? daemonStatus?.version ?? "--",
                icon: "tag",
                color: DeepSpace.cyan
            )
            StatCard(
                title: "Peers",
                value: daemonStatus?.peers.map { "\($0)" } ?? healthStatus?.peers.map { "\($0)" } ?? "0",
                icon: "person.2",
                color: DeepSpace.violet
            )
            StatCard(
                title: "Uptime",
                value: formatUptime(healthStatus?.uptimeSecs),
                icon: "clock",
                color: DeepSpace.amber
            )
        }
    }

    // MARK: - Identity Card

    private var identityCard: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                if let identity = appState.agentIdentity {
                    copyableRow(label: "Agent ID", value: identity.agentId)
                    if let machineId = identity.machineId {
                        copyableRow(label: "Machine ID", value: machineId)
                    }
                } else {
                    Text("Not connected to daemon")
                        .foregroundStyle(DeepSpace.textMuted)
                }
            }
            .padding(8)
        } label: {
            Label("Identity", systemImage: "person.badge.key")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    // MARK: - Quick Actions

    private var quickActions: some View {
        HStack(spacing: 12) {
            Button {
                appState.showCreateSpace = true
            } label: {
                Label("Create Space", systemImage: "plus.circle")
            }
            .buttonStyle(.borderedProminent)
            .tint(DeepSpace.cyan)
            .disabled(appState.daemonState != .running)

            Button {
                appState.selectedSystemPage = .network
                appState.selectedDMContact = nil
            } label: {
                Label("View Network", systemImage: "network")
            }
            .buttonStyle(.bordered)
            .disabled(appState.daemonState != .running)

            Spacer()
        }
    }

    // MARK: - Discovered Agents

    private var agentsList: some View {
        GroupBox {
            if discoveredAgents.isEmpty {
                HStack {
                    Spacer()
                    VStack(spacing: 8) {
                        Image(systemName: "antenna.radiowaves.left.and.right")
                            .font(.title2)
                            .foregroundStyle(DeepSpace.textMuted)
                        Text("No agents discovered yet")
                            .font(.caption)
                            .foregroundStyle(DeepSpace.textMuted)
                    }
                    .padding(.vertical, 16)
                    Spacer()
                }
            } else {
                LazyVStack(alignment: .leading, spacing: 6) {
                    ForEach(discoveredAgents) { agent in
                        HStack(spacing: 10) {
                            Circle()
                                .fill(DeepSpace.green)
                                .frame(width: 8, height: 8)
                            VStack(alignment: .leading, spacing: 2) {
                                Text(agent.displayName ?? truncatedId(agent.agentId))
                                    .font(.subheadline)
                                    .foregroundStyle(DeepSpace.textPrimary)
                                Text(truncatedId(agent.agentId))
                                    .font(.caption2)
                                    .foregroundStyle(DeepSpace.textMuted)
                                    .lineLimit(1)
                            }
                            Spacer()
                        }
                        .padding(.vertical, 4)
                        .padding(.horizontal, 8)
                    }
                }
            }
        } label: {
            Label("Discovered Agents", systemImage: "antenna.radiowaves.left.and.right")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    // MARK: - Helpers

    private func copyableRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundStyle(DeepSpace.textSecondary)
                .frame(width: 120, alignment: .leading)
            Text(value)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(DeepSpace.textPrimary)
                .textSelection(.enabled)
                .lineLimit(1)
                .truncationMode(.middle)
            Spacer()
            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(value, forType: .string)
            } label: {
                Image(systemName: "doc.on.doc")
                    .font(.caption2)
                    .foregroundStyle(DeepSpace.textMuted)
            }
            .buttonStyle(.plain)
        }
    }

    private func truncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
    }

    private func formatUptime(_ seconds: UInt64?) -> String {
        guard let seconds else { return "--" }
        if seconds < 60 { return "\(seconds)s" }
        if seconds < 3600 { return "\(seconds / 60)m" }
        if seconds < 86400 { return "\(seconds / 3600)h" }
        return "\(seconds / 86400)d"
    }

    private func pollData() async {
        isRefreshing = true
        defer { isRefreshing = false }

        await appState.refresh()

        guard appState.daemonState == .running else { return }

        do {
            daemonStatus = try await appState.client.status()
        } catch {
            daemonStatus = nil
        }

        do {
            healthStatus = try await appState.client.health()
        } catch {
            healthStatus = nil
        }

        do {
            discoveredAgents = try await appState.client.discoveredAgents()
        } catch {
            discoveredAgents = []
        }
    }

    private func startPolling() {
        Task {
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 8_000_000_000) // 8s
                await pollData()
            }
        }
    }
}

// MARK: - Stat Card

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Image(systemName: icon)
                    .font(.caption)
                    .foregroundStyle(color)
                Text(title)
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
            }
            Text(value)
                .font(.title3)
                .fontWeight(.semibold)
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(DeepSpace.surface2, in: RoundedRectangle(cornerRadius: 10))
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .stroke(DeepSpace.border, lineWidth: 1)
        )
    }
}

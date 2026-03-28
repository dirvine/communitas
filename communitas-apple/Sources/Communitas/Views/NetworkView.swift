import SwiftUI
import X0xClient

/// Network view showing connection stats, external addresses, and connected peers.
struct NetworkView: View {
    @EnvironmentObject var appState: AppState

    @State private var networkStatus: NetworkStatus?
    @State private var daemonStatus: DaemonStatus?
    @State private var healthStatus: HealthStatus?
    @State private var peers: [PeerInfo] = []
    @State private var isRefreshing = false

    private let statsColumns = Array(repeating: GridItem(.flexible(), spacing: 12), count: 3)

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                statsGrid
                nodeHealthSection
                externalAddressesSection
                networkStatsSection
                peersTable
            }
            .padding(24)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(DeepSpace.bg)
        .navigationTitle("Network")
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
        LazyVGrid(columns: statsColumns, spacing: 12) {
            StatCard(
                title: "Status",
                value: connectionStatusLabel,
                icon: "wifi",
                color: (networkStatus?.connectedPeers ?? 0) > 0 ? DeepSpace.green : DeepSpace.red
            )
            StatCard(
                title: "Peers",
                value: "\(networkStatus?.connectedPeers ?? 0)",
                icon: "person.2",
                color: DeepSpace.cyan
            )
            StatCard(
                title: "Addresses",
                value: "\(networkStatus?.externalAddrs?.count ?? 0)",
                icon: "globe",
                color: DeepSpace.amber
            )
        }
    }

    private var connectionStatusLabel: String {
        (networkStatus?.connectedPeers ?? 0) > 0 ? "Connected" : "Disconnected"
    }

    // MARK: - Node Health Section

    private var nodeHealthSection: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 10) {
                HStack(spacing: 10) {
                    Circle()
                        .fill(healthColor)
                        .frame(width: 10, height: 10)
                    Text(healthLabel)
                        .font(.subheadline)
                        .fontWeight(.medium)
                        .foregroundStyle(DeepSpace.textPrimary)
                    Spacer()
                }

                Divider().background(DeepSpace.border)

                if let agentId = appState.agentIdentity?.agentId {
                    copyableRow(label: "Agent ID", value: agentId)
                }

                if let version = healthStatus?.version ?? daemonStatus?.version {
                    HStack {
                        Text("Version")
                            .font(.caption)
                            .foregroundStyle(DeepSpace.textSecondary)
                            .frame(width: 100, alignment: .leading)
                        Text(version)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(DeepSpace.textPrimary)
                        Spacer()
                    }
                }

                if let uptime = healthStatus?.uptimeSecs ?? daemonStatus?.uptimeSecs {
                    HStack {
                        Text("Uptime")
                            .font(.caption)
                            .foregroundStyle(DeepSpace.textSecondary)
                            .frame(width: 100, alignment: .leading)
                        Text(formatUptime(uptime))
                            .font(.caption)
                            .foregroundStyle(DeepSpace.textPrimary)
                        Spacer()
                    }
                }
            }
            .padding(4)
        } label: {
            Label("Node Health", systemImage: "server.rack")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    private var healthColor: Color {
        switch appState.daemonState {
        case .running:
            if (networkStatus?.connectedPeers ?? 0) > 0 { return DeepSpace.green }
            return .yellow
        case .starting: return .yellow
        case .notRunning, .notInstalled, .error: return DeepSpace.red
        }
    }

    private var healthLabel: String {
        switch appState.daemonState {
        case .running:
            if (networkStatus?.connectedPeers ?? 0) > 0 { return "Online — Connected to network" }
            return "Running — No peers yet"
        case .starting: return "Starting..."
        case .notRunning: return "Not Running"
        case .notInstalled: return "Not Installed"
        case .error: return "Error"
        }
    }

    // MARK: - External Addresses Section

    private var externalAddressesSection: some View {
        GroupBox {
            if let addrs = networkStatus?.externalAddrs, !addrs.isEmpty {
                let ipv4 = addrs.filter { isIPv4($0) }
                let ipv6 = addrs.filter { !isIPv4($0) }

                VStack(alignment: .leading, spacing: 12) {
                    if !ipv4.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("IPv4")
                                .font(.caption2)
                                .fontWeight(.semibold)
                                .foregroundStyle(DeepSpace.textMuted)
                                .textCase(.uppercase)

                            ForEach(ipv4, id: \.self) { addr in
                                addressRow(addr)
                            }
                        }
                    }

                    if !ipv6.isEmpty {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("IPv6")
                                .font(.caption2)
                                .fontWeight(.semibold)
                                .foregroundStyle(DeepSpace.textMuted)
                                .textCase(.uppercase)

                            ForEach(ipv6, id: \.self) { addr in
                                addressRow(addr)
                            }
                        }
                    }
                }
                .padding(4)
            } else {
                emptyState(icon: "globe", message: "No external addresses detected")
            }
        } label: {
            Label("External Addresses", systemImage: "globe")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    private func addressRow(_ addr: String) -> some View {
        HStack {
            Text(addr)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(DeepSpace.textPrimary)
                .textSelection(.enabled)
            Spacer()
            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(addr, forType: .string)
            } label: {
                Image(systemName: "doc.on.doc")
                    .font(.caption2)
                    .foregroundStyle(DeepSpace.textMuted)
            }
            .buttonStyle(.plain)
        }
        .padding(.vertical, 2)
    }

    // MARK: - Network Statistics Section

    private var networkStatsSection: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 8) {
                if let rtt = networkStatus?.avgRttMs {
                    statRow(label: "Avg RTT", value: String(format: "%.0f ms", rtt))
                }

                if let direct = networkStatus?.directConnections {
                    statRow(label: "Direct Connections", value: "\(direct)")
                }

                if let canReceive = networkStatus?.canReceiveDirect {
                    statRow(label: "Can Receive Direct", value: canReceive ? "Yes" : "No")
                }

                if let holePunch = networkStatus?.holePunchSuccessRate {
                    statRow(label: "Hole Punch Rate", value: String(format: "%.0f%%", holePunch * 100))
                }

                if networkStatus?.avgRttMs == nil
                    && networkStatus?.directConnections == nil
                    && networkStatus?.canReceiveDirect == nil {
                    Text("Connect to peers to see statistics")
                        .font(.caption)
                        .foregroundStyle(DeepSpace.textMuted)
                        .padding(.vertical, 4)
                }
            }
            .padding(4)
        } label: {
            Label("Network Statistics", systemImage: "chart.bar")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    private func statRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundStyle(DeepSpace.textSecondary)
                .frame(minWidth: 160, alignment: .leading)
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(DeepSpace.textPrimary)
            Spacer()
        }
    }

    // MARK: - Peers Table

    private var peersTable: some View {
        GroupBox {
            if peers.isEmpty {
                emptyState(icon: "person.2.slash", message: "No connected peers")
            } else {
                VStack(spacing: 0) {
                    // Header
                    HStack {
                        Text("Peer ID")
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .font(.caption2)
                    .fontWeight(.semibold)
                    .foregroundStyle(DeepSpace.textMuted)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 6)

                    Divider()
                        .background(DeepSpace.border)

                    // Rows
                    ForEach(peers) { peer in
                        HStack(spacing: 8) {
                            Circle()
                                .fill(DeepSpace.green)
                                .frame(width: 6, height: 6)
                            Text(truncatedId(peer.peerId))
                                .font(.system(.caption, design: .monospaced))
                                .foregroundStyle(DeepSpace.textPrimary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .lineLimit(1)
                            Button {
                                NSPasteboard.general.clearContents()
                                NSPasteboard.general.setString(peer.peerId, forType: .string)
                            } label: {
                                Image(systemName: "doc.on.doc")
                                    .font(.caption2)
                                    .foregroundStyle(DeepSpace.textMuted)
                            }
                            .buttonStyle(.plain)
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 5)
                        .contextMenu {
                            Button("Copy Peer ID") {
                                NSPasteboard.general.clearContents()
                                NSPasteboard.general.setString(peer.peerId, forType: .string)
                            }
                        }
                    }
                }
            }
        } label: {
            Label("Connected Peers (\(peers.count))", systemImage: "person.2")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    // MARK: - Helpers

    private func emptyState(icon: String, message: String) -> some View {
        HStack {
            Spacer()
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title2)
                    .foregroundStyle(DeepSpace.textMuted)
                Text(message)
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
            }
            .padding(.vertical, 16)
            Spacer()
        }
    }

    private func copyableRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundStyle(DeepSpace.textSecondary)
                .frame(width: 100, alignment: .leading)
            Text(truncatedId(value))
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(DeepSpace.textPrimary)
                .textSelection(.enabled)
                .lineLimit(1)
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
        if id.count > 20 {
            return String(id.prefix(10)) + "..." + String(id.suffix(8))
        }
        return id
    }

    private func isIPv4(_ addr: String) -> Bool {
        // Quick heuristic: IPv6 addresses contain colons, IPv4 do not (before the port)
        let hostPart = addr.split(separator: ":").first.map(String.init) ?? addr
        return !hostPart.contains(":")
    }

    private func formatUptime(_ seconds: UInt64) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds < 3600 { return "\(seconds / 60)m \(seconds % 60)s" }
        if seconds < 86400 { return "\(seconds / 3600)h \((seconds % 3600) / 60)m" }
        return "\(seconds / 86400)d \((seconds % 86400) / 3600)h"
    }

    private func pollData() async {
        isRefreshing = true
        defer { isRefreshing = false }

        guard appState.daemonState == .running else { return }

        do {
            networkStatus = try await appState.client.networkStatus()
        } catch {
            networkStatus = nil
        }

        do {
            peers = try await appState.client.peers()
        } catch {
            peers = []
        }

        do {
            healthStatus = try await appState.client.health()
        } catch {
            healthStatus = nil
        }

        do {
            daemonStatus = try await appState.client.status()
        } catch {
            daemonStatus = nil
        }
    }

    private func startPolling() {
        Task {
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 5_000_000_000) // 5s
                await pollData()
            }
        }
    }
}

import AppKit
import SwiftUI
import X0xClient

private struct PeerDiagnostic {
    let health: String
    let probe: String
}

private struct NetworkMetric: Identifiable {
    let id: String
    let title: String
    let value: String
    let systemImage: String
    let color: Color
}

private struct NetworkInfoRow: Identifiable {
    let id: String
    let label: String
    let value: String
}

/// Network view showing connection stats, diagnostics, discovered machines, and peers.
///
/// The layout intentionally stays flat and bounded. A previous version used a
/// deeply nested GroupBox/table tree and crashed in SwiftUI constraint/body
/// evaluation on macOS while opening the page.
struct NetworkView: View {
    @EnvironmentObject var appState: AppState

    @State private var networkStatus: NetworkStatus?
    @State private var daemonStatus: DaemonStatus?
    @State private var healthStatus: HealthStatus?
    @State private var peers: [PeerInfo] = []
    @State private var directConnections: [DirectConnection] = []
    @State private var bootstrapCache: BootstrapCacheStatus?
    @State private var webSocketSessions: WsSessionList?
    @State private var upgradeStatus: UpgradeStatus?
    @State private var gossipStats: GossipStats?
    @State private var connectivityDiagnostics: ConnectivityDiagnostics?
    @State private var discoveredMachines: [DiscoveredMachine] = []
    @State private var agentMachine: AgentMachine?
    @State private var userMachines: UserMachineList?
    @State private var connectMachineResult: ConnectMachineResponse?
    @State private var peerDiagnostics: [String: PeerDiagnostic] = [:]
    @State private var isRefreshing = false
    @State private var lastUpgradeCheckAt: Date?

    private let upgradeCheckCooldown: TimeInterval = 6 * 60 * 60

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 18) {
                header
                metricGrid
                sectionPanel(title: "Node Health", systemImage: "server.rack") {
                    infoRows(nodeHealthRows)
                }
                sectionPanel(title: "Network Statistics", systemImage: "chart.bar") {
                    infoRows(networkRows, emptyMessage: "Connect to peers to see statistics")
                }
                sectionPanel(title: "External Addresses", systemImage: "globe") {
                    addressList
                }
                sectionPanel(title: "Daemon Diagnostics", systemImage: "stethoscope") {
                    infoRows(diagnosticRows, emptyMessage: "No extended diagnostics available")
                }
                sectionPanel(title: "Discovered Machines", systemImage: "network") {
                    discoveredMachineList
                }
                sectionPanel(title: "Connected Peers (\(peers.count))", systemImage: "person.2") {
                    peerList
                }
            }
            .padding(24)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(DeepSpace.bg)
        .navigationTitle("Network")
        .task {
            await pollData()
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 5_000_000_000)
                await pollData()
            }
        }
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("Network")
                    .font(.title2)
                    .fontWeight(.semibold)
                    .foregroundStyle(DeepSpace.textPrimary)
                Text(healthLabel)
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textSecondary)
            }
            Spacer()
            ProgressView()
                .controlSize(.small)
                .opacity(isRefreshing ? 1 : 0)
            AppKitInlineButton(
                title: "Refresh",
                systemSymbolName: "arrow.clockwise",
                accessibilityIdentifier: "network-refresh-button"
            ) {
                Task { await pollData() }
            }
            .frame(width: 96, height: 28)
        }
    }

    private var metricGrid: some View {
        VStack(spacing: 12) {
            HStack(spacing: 12) {
                ForEach(Array(metricCards.prefix(2))) { metric in
                    metricCard(metric)
                }
            }
            HStack(spacing: 12) {
                ForEach(Array(metricCards.dropFirst(2))) { metric in
                    metricCard(metric)
                }
            }
        }
    }

    private func metricCard(_ metric: NetworkMetric) -> some View {
        HStack(spacing: 12) {
            Image(systemName: metric.systemImage)
                .font(.title3)
                .foregroundStyle(metric.color)
                .frame(width: 28)
            VStack(alignment: .leading, spacing: 2) {
                Text(metric.title)
                    .font(.caption2)
                    .foregroundStyle(DeepSpace.textMuted)
                Text(metric.value)
                    .font(.headline)
                    .foregroundStyle(DeepSpace.textPrimary)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(14)
        .frame(maxWidth: .infinity, minHeight: 70)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(DeepSpace.surface1)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(DeepSpace.border, lineWidth: 1)
        )
    }

    private func sectionPanel<Content: View>(
        title: String,
        systemImage: String,
        @ViewBuilder content: () -> Content
    ) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Label(title, systemImage: systemImage)
                .font(.headline)
                .foregroundStyle(DeepSpace.textPrimary)
            content()
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(DeepSpace.surface1)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(DeepSpace.border, lineWidth: 1)
        )
    }

    private func infoRows(_ rows: [NetworkInfoRow], emptyMessage: String? = nil) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            if rows.isEmpty, let emptyMessage {
                Text(emptyMessage)
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
                    .padding(.vertical, 4)
            } else {
                ForEach(rows) { row in
                    HStack(alignment: .top, spacing: 12) {
                        Text(row.label)
                            .font(.caption)
                            .foregroundStyle(DeepSpace.textSecondary)
                            .frame(width: 160, alignment: .leading)
                        Text(row.value)
                            .font(.caption)
                            .fontWeight(.medium)
                            .foregroundStyle(DeepSpace.textPrimary)
                            .textSelection(.enabled)
                        Spacer(minLength: 0)
                    }
                }
            }
        }
    }

    private var addressList: some View {
        VStack(alignment: .leading, spacing: 8) {
            let addresses = networkStatus?.externalAddrs ?? []
            if addresses.isEmpty {
                Text("No external addresses detected")
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
                    .padding(.vertical, 4)
            } else {
                ForEach(addresses.prefix(12), id: \.self) { address in
                    HStack(alignment: .top, spacing: 10) {
                        Text(addressFamily(address))
                            .font(.caption2)
                            .fontWeight(.semibold)
                            .foregroundStyle(DeepSpace.textMuted)
                            .frame(width: 42, alignment: .leading)
                        Text(address)
                            .font(.system(.caption, design: .monospaced))
                            .foregroundStyle(DeepSpace.textPrimary)
                            .textSelection(.enabled)
                            .lineLimit(2)
                        Spacer(minLength: 0)
                    }
                }
            }
        }
    }

    private var discoveredMachineList: some View {
        VStack(alignment: .leading, spacing: 8) {
            if discoveredMachines.isEmpty {
                Text("No machine announcements discovered")
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
                    .padding(.vertical, 4)
            } else {
                if let agentMachine {
                    infoRows([
                        NetworkInfoRow(
                            id: "agent-machine",
                            label: "Current Agent Machine",
                            value: truncatedId(agentMachine.machine.machineId)
                        )
                    ])
                }
                if let userMachines {
                    infoRows([
                        NetworkInfoRow(
                            id: "user-machines",
                            label: "Machines for User",
                            value: "\(userMachines.machines.count)"
                        )
                    ])
                }
                if let connectMachineResult {
                    infoRows([
                        NetworkInfoRow(id: "last-connect", label: "Last Connect", value: connectMachineResult.outcome)
                    ])
                }
                ForEach(discoveredMachines.prefix(12)) { machine in
                    HStack(alignment: .center, spacing: 10) {
                        VStack(alignment: .leading, spacing: 3) {
                            Text(truncatedId(machine.machineId))
                                .font(.system(.caption, design: .monospaced))
                                .foregroundStyle(DeepSpace.textPrimary)
                                .textSelection(.enabled)
                            Text("\(machine.addresses.count) addresses")
                                .font(.caption2)
                                .foregroundStyle(DeepSpace.textMuted)
                        }
                        Spacer(minLength: 0)
                        AppKitInlineButton(
                            title: "Connect",
                            systemSymbolName: "link",
                            accessibilityIdentifier: "network-connect-\(machine.machineId)"
                        ) {
                            Task { await connect(machine: machine) }
                        }
                        .frame(width: 92, height: 26)
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private var peerList: some View {
        VStack(alignment: .leading, spacing: 8) {
            if peers.isEmpty {
                Text("No connected peers")
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
                    .padding(.vertical, 4)
            } else {
                ForEach(peers.prefix(25)) { peer in
                    HStack(alignment: .top, spacing: 10) {
                        Circle()
                            .fill(DeepSpace.green)
                            .frame(width: 7, height: 7)
                            .padding(.top, 5)
                        VStack(alignment: .leading, spacing: 3) {
                            Text(truncatedId(peer.peerId))
                                .font(.system(.caption, design: .monospaced))
                                .foregroundStyle(DeepSpace.textPrimary)
                                .textSelection(.enabled)
                            if let diagnostic = peerDiagnostics[peer.peerId] {
                                Text("Health: \(diagnostic.health)")
                                    .font(.caption2)
                                    .foregroundStyle(DeepSpace.textSecondary)
                                    .lineLimit(2)
                                Text("Probe: \(diagnostic.probe)")
                                    .font(.caption2)
                                    .foregroundStyle(DeepSpace.cyan)
                                    .lineLimit(1)
                            } else {
                                Text("Diagnostics not checked yet")
                                    .font(.caption2)
                                    .foregroundStyle(DeepSpace.textMuted)
                            }
                        }
                        Spacer(minLength: 0)
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }

    private var metricCards: [NetworkMetric] {
        [
            NetworkMetric(
                id: "status",
                title: "Status",
                value: connectionStatusLabel,
                systemImage: "wifi",
                color: (networkStatus?.connectedPeers ?? 0) > 0 ? DeepSpace.green : DeepSpace.red
            ),
            NetworkMetric(
                id: "peers",
                title: "Peers",
                value: "\(networkStatus?.connectedPeers ?? 0)",
                systemImage: "person.2",
                color: DeepSpace.cyan
            ),
            NetworkMetric(
                id: "addresses",
                title: "Addresses",
                value: "\(networkStatus?.externalAddrs?.count ?? 0)",
                systemImage: "globe",
                color: DeepSpace.amber
            ),
            NetworkMetric(
                id: "direct",
                title: "Direct",
                value: "\(directConnections.count)",
                systemImage: "bolt.horizontal.circle",
                color: DeepSpace.green
            )
        ]
    }

    private var nodeHealthRows: [NetworkInfoRow] {
        var rows = [
            NetworkInfoRow(id: "health", label: "Health", value: healthLabel)
        ]
        if let agentId = appState.agentIdentity?.agentId {
            rows.append(NetworkInfoRow(id: "agent", label: "Agent ID", value: agentId))
        }
        if let version = healthStatus?.version ?? daemonStatus?.version {
            rows.append(NetworkInfoRow(id: "version", label: "Version", value: version))
        }
        if let uptime = healthStatus?.uptimeSecs ?? daemonStatus?.uptimeSecs {
            rows.append(NetworkInfoRow(id: "uptime", label: "Uptime", value: formatUptime(uptime)))
        }
        return rows
    }

    private var networkRows: [NetworkInfoRow] {
        var rows: [NetworkInfoRow] = []
        if let rtt = networkStatus?.avgRttMs {
            rows.append(NetworkInfoRow(id: "rtt", label: "Avg RTT", value: String(format: "%.0f ms", rtt)))
        }
        if let direct = networkStatus?.directConnections {
            rows.append(NetworkInfoRow(id: "direct", label: "Direct Connections", value: "\(direct)"))
        }
        if let canReceive = networkStatus?.canReceiveDirect {
            rows.append(NetworkInfoRow(id: "can-receive", label: "Can Receive Direct", value: canReceive ? "Yes" : "No"))
        }
        if let holePunch = networkStatus?.holePunchSuccessRate {
            rows.append(NetworkInfoRow(id: "hole-punch", label: "Hole Punch Rate", value: String(format: "%.0f%%", holePunch * 100)))
        }
        return rows
    }

    private var diagnosticRows: [NetworkInfoRow] {
        var rows: [NetworkInfoRow] = []
        if let bootstrapCount = bootstrapCache?.connectionCount {
            rows.append(NetworkInfoRow(id: "bootstrap", label: "Bootstrap Cache", value: "\(bootstrapCount) connected"))
        }
        if let diagnostics = connectivityDiagnostics {
            rows.append(NetworkInfoRow(id: "conn-peer", label: "Connectivity Peer", value: truncatedId(diagnostics.peerId)))
            rows.append(NetworkInfoRow(id: "mdns", label: "mDNS Peers", value: "\(diagnostics.mdns.discoveredPeers)"))
            rows.append(NetworkInfoRow(id: "relay", label: "Relay Enabled", value: diagnostics.services.relayEnabled ? "Yes" : "No"))
            rows.append(NetworkInfoRow(id: "coordinator", label: "Coordinator Enabled", value: diagnostics.services.coordinatorEnabled ? "Yes" : "No"))
        }
        if let sessions = webSocketSessions?.sessions.count {
            rows.append(NetworkInfoRow(id: "ws", label: "WebSocket Sessions", value: "\(sessions)"))
        }
        if let shared = webSocketSessions?.sharedSubscriptions, !shared.isEmpty {
            rows.append(NetworkInfoRow(id: "shared-subs", label: "Shared Topic Subs", value: "\(shared.count)"))
        }
        if let stats = gossipStats {
            rows.append(NetworkInfoRow(id: "gossip-pub", label: "Gossip Published", value: "\(stats.publishTotal)"))
            rows.append(NetworkInfoRow(id: "gossip-in", label: "Gossip Incoming", value: "\(stats.incomingTotal)"))
            rows.append(NetworkInfoRow(id: "gossip-delivered", label: "Gossip Delivered", value: "\(stats.deliveredToSubscriber)"))
            rows.append(NetworkInfoRow(id: "gossip-drops", label: "Decode to Delivery Drops", value: "\(stats.decodeToDeliveryDrops)"))
        }
        if let available = upgradeStatus?.updateAvailable {
            let value = available
                ? "Update available: \(upgradeStatus?.version ?? "unknown")"
                : "Up to date (\(upgradeStatus?.currentVersion ?? upgradeStatus?.version ?? "unknown"))"
            rows.append(NetworkInfoRow(id: "upgrade", label: "Upgrade", value: value))
        }
        return rows
    }

    private var connectionStatusLabel: String {
        (networkStatus?.connectedPeers ?? 0) > 0 ? "Connected" : "Disconnected"
    }

    private var healthLabel: String {
        switch appState.daemonState {
        case .running:
            if (networkStatus?.connectedPeers ?? 0) > 0 { return "Online - connected to network" }
            return "Running - no peers yet"
        case .starting: return "Starting..."
        case .notRunning: return "Not running"
        case .notInstalled: return "Not installed"
        case .error: return "Error"
        }
    }

    private func truncatedId(_ id: String) -> String {
        if id.count > 20 {
            return String(id.prefix(10)) + "..." + String(id.suffix(8))
        }
        return id
    }

    private func addressFamily(_ address: String) -> String {
        let host: String
        if address.hasPrefix("[") {
            host = String(address.dropFirst().prefix { $0 != "]" })
        } else {
            host = String(address.split(separator: ":").first ?? "")
        }
        if host.contains(".") { return "IPv4" }
        if address.contains(":") { return "IPv6" }
        return "Addr"
    }

    private func formatUptime(_ seconds: UInt64) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds < 3_600 { return "\(seconds / 60)m \(seconds % 60)s" }
        if seconds < 86_400 { return "\(seconds / 3_600)h \((seconds % 3_600) / 60)m" }
        return "\(seconds / 86_400)d \((seconds % 86_400) / 3_600)h"
    }

    private func formatPeerHealth(_ health: PeerHealth) -> String {
        if let snapshot = health.health, !snapshot.isEmpty {
            return snapshot
        }
        if let error = health.error, !error.isEmpty {
            return error
        }
        return (health.ok ?? false) ? "Healthy" : "Unknown"
    }

    private func formatProbeResult(_ result: ProbePeerResult) -> String {
        if let ms = result.rttMs {
            return "\(ms) ms"
        }
        if let us = result.rttUs {
            return "\(us) us"
        }
        if let error = result.error, !error.isEmpty {
            return error
        }
        return (result.ok ?? false) ? "OK" : "No RTT"
    }

    @MainActor
    private func pollData() async {
        guard !isRefreshing else { return }
        isRefreshing = true
        defer { isRefreshing = false }

        guard appState.daemonState == .running else { return }

        networkStatus = try? await appState.client.networkStatus()
        connectivityDiagnostics = try? await appState.client.connectivityDiagnostics()

        if let machines = try? await appState.client.discoveredMachines() {
            discoveredMachines = machines
        } else {
            discoveredMachines = []
        }

        if let agentId = appState.agentIdentity?.agentId {
            agentMachine = try? await appState.client.machineForAgent(agentId: agentId)
        } else {
            agentMachine = nil
        }

        if let userId = appState.agentIdentity?.userId {
            userMachines = try? await appState.client.machinesByUser(userId: userId)
        } else {
            userMachines = nil
        }

        if let currentPeers = try? await appState.client.peers() {
            peers = currentPeers
            peerDiagnostics = await diagnostics(for: currentPeers)
        } else {
            peers = []
            peerDiagnostics = [:]
        }

        healthStatus = try? await appState.client.health()
        daemonStatus = try? await appState.client.status()
        directConnections = (try? await appState.client.directConnections()) ?? []
        bootstrapCache = try? await appState.client.bootstrapCache()
        webSocketSessions = try? await appState.client.wsSessions()
        gossipStats = try? await appState.client.gossipStats()
        if shouldCheckUpgradeStatus {
            lastUpgradeCheckAt = Date()
            upgradeStatus = try? await appState.client.checkUpgrade()
        }
    }

    private var shouldCheckUpgradeStatus: Bool {
        guard let lastUpgradeCheckAt else { return true }
        return Date().timeIntervalSince(lastUpgradeCheckAt) >= upgradeCheckCooldown
    }

    private func diagnostics(for currentPeers: [PeerInfo]) async -> [String: PeerDiagnostic] {
        var diagnostics: [String: PeerDiagnostic] = [:]
        for peer in currentPeers.prefix(12) {
            let health: String
            if let snapshot = try? await appState.client.peerHealth(peerId: peer.peerId) {
                health = formatPeerHealth(snapshot)
            } else {
                health = "Unavailable"
            }

            let probe: String
            if let result = try? await appState.client.probePeer(peerId: peer.peerId) {
                probe = formatProbeResult(result)
            } else {
                probe = "Probe failed"
            }

            diagnostics[peer.peerId] = PeerDiagnostic(health: health, probe: probe)
        }
        return diagnostics
    }

    @MainActor
    private func connect(machine: DiscoveredMachine) async {
        if let result = try? await appState.client.connectMachine(machineId: machine.machineId) {
            connectMachineResult = result
        } else {
            connectMachineResult = ConnectMachineResponse(ok: false, outcome: "Failed", addr: nil)
        }
    }
}

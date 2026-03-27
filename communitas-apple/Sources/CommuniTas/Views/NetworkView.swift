import SwiftUI
import X0xClient

/// Network view showing connection stats, external addresses, and connected peers.
struct NetworkView: View {
    @EnvironmentObject var appState: AppState

    @State private var networkStatus: NetworkStatus?
    @State private var peers: [PeerInfo] = []
    @State private var isRefreshing = false

    private let statsColumns = Array(repeating: GridItem(.flexible(), spacing: 12), count: 4)

    var body: some View {
        ScrollView {
            VStack(spacing: 20) {
                statsGrid
                listenAddresses
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
                value: networkStatus?.connected == true ? "Connected" : "Disconnected",
                icon: "wifi",
                color: networkStatus?.connected == true ? DeepSpace.green : DeepSpace.red
            )
            StatCard(
                title: "Peers",
                value: "\(networkStatus?.peerCount ?? 0)",
                icon: "person.2",
                color: DeepSpace.cyan
            )
            StatCard(
                title: "Connected",
                value: "\(peers.count)",
                icon: "link",
                color: DeepSpace.violet
            )
            StatCard(
                title: "Addresses",
                value: "\(networkStatus?.listenAddresses?.count ?? 0)",
                icon: "globe",
                color: DeepSpace.amber
            )
        }
    }

    // MARK: - Listen Addresses

    private var listenAddresses: some View {
        GroupBox {
            if let addrs = networkStatus?.listenAddresses, !addrs.isEmpty {
                VStack(alignment: .leading, spacing: 6) {
                    ForEach(addrs, id: \.self) { addr in
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
                    }
                }
                .padding(4)
            } else {
                Text("No listen addresses detected")
                    .font(.caption)
                    .foregroundStyle(DeepSpace.textMuted)
                    .padding(8)
            }
        } label: {
            Label("Listen Addresses", systemImage: "globe")
                .foregroundStyle(DeepSpace.textPrimary)
        }
        .backgroundStyle(DeepSpace.surface1)
    }

    // MARK: - Peers Table

    private var peersTable: some View {
        GroupBox {
            if peers.isEmpty {
                HStack {
                    Spacer()
                    VStack(spacing: 8) {
                        Image(systemName: "person.2.slash")
                            .font(.title2)
                            .foregroundStyle(DeepSpace.textMuted)
                        Text("No connected peers")
                            .font(.caption)
                            .foregroundStyle(DeepSpace.textMuted)
                    }
                    .padding(.vertical, 16)
                    Spacer()
                }
            } else {
                VStack(spacing: 0) {
                    // Header
                    HStack {
                        Text("Peer ID")
                            .frame(maxWidth: .infinity, alignment: .leading)
                        Text("Address")
                            .frame(width: 180, alignment: .leading)
                        Text("Latency")
                            .frame(width: 80, alignment: .trailing)
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
                        HStack {
                            Text(truncatedId(peer.peerId))
                                .font(.system(.caption, design: .monospaced))
                                .foregroundStyle(DeepSpace.textPrimary)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .lineLimit(1)
                            Text(peer.address ?? "--")
                                .font(.system(.caption2, design: .monospaced))
                                .foregroundStyle(DeepSpace.textSecondary)
                                .frame(width: 180, alignment: .leading)
                                .lineLimit(1)
                            Text(peer.latency.map { "\($0)ms" } ?? "--")
                                .font(.caption2)
                                .foregroundStyle(DeepSpace.textSecondary)
                                .frame(width: 80, alignment: .trailing)
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 5)
                        .contextMenu {
                            Button("Copy Peer ID") {
                                NSPasteboard.general.clearContents()
                                NSPasteboard.general.setString(peer.peerId, forType: .string)
                            }
                            if let address = peer.address {
                                Button("Copy Address") {
                                    NSPasteboard.general.clearContents()
                                    NSPasteboard.general.setString(address, forType: .string)
                                }
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

    private func truncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
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

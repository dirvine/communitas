import SwiftUI
import CommunitasKit

// MARK: - Network Panel View

/// Displays network status, connection information, and peer management
struct NetworkPanelView: View {
    @EnvironmentObject var state: AppState
    @State private var connectAddress: String = ""
    @State private var isConnecting: Bool = false
    @State private var errorMessage: String?
    @State private var showError: Bool = false

    // Production bootstrap nodes from config/production-network.toml
    private let bootstrapNodes: [(name: String, address: String)] = [
        ("Droplet 2064413", "167.71.188.131:50000"),
        ("communitas-bootstrap-1", "138.197.29.195:50000")
    ]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                // Header
                networkHeader

                Divider()

                // Network Status Section
                networkStatusSection

                Divider()

                // My Addresses Section
                myAddressesSection

                Divider()

                // Bootstrap Nodes Section
                bootstrapNodesSection

                Divider()

                // Connected Peers Section
                connectedPeersSection

                Divider()

                // Connect to Peer Section
                connectToPeerSection
            }
            .padding(24)
        }
        .background(Color(NSColor.windowBackgroundColor))
        .alert("Connection Error", isPresented: $showError) {
            Button("OK") {
                showError = false
                errorMessage = nil
            }
        } message: {
            Text(errorMessage ?? "Unknown error")
        }
    }

    // MARK: - Header

    private var networkHeader: some View {
        HStack {
            Image(systemName: "globe.americas.fill")
                .font(.title)
                .foregroundColor(.blue)

            VStack(alignment: .leading, spacing: 2) {
                Text("Network Status")
                    .font(.title2)
                    .fontWeight(.semibold)
                Text("View and manage your P2P network connections")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }

            Spacer()

            // Close button
            Button {
                state.toggleNetworkPanel()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .font(.title2)
                    .foregroundColor(.secondary)
            }
            .buttonStyle(.plain)
            .help("Close Network Panel")
        }
    }

    // MARK: - Network Status Section

    private var networkStatusSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("Status", systemImage: "antenna.radiowaves.left.and.right")
                .font(.headline)

            HStack(spacing: 16) {
                // Connection status indicator
                HStack(spacing: 8) {
                    Circle()
                        .fill(state.isNetworking ? Color.green : Color.orange)
                        .frame(width: 12, height: 12)
                    Text(state.isNetworking ? "Connected" : "Disconnected")
                        .font(.body)
                        .fontWeight(.medium)
                }

                Spacer()

                // Toggle networking button
                Button {
                    if state.isNetworking {
                        state.stopNetworking()
                    } else {
                        state.startNetworkingWithBootstrap()
                    }
                } label: {
                    Text(state.isNetworking ? "Disconnect" : "Connect")
                }
                .buttonStyle(.bordered)
            }
            .padding()
            .background(Color.gray.opacity(0.1))
            .cornerRadius(8)

            // Local-only mode indicator
            if let networkInfo = state.getNetworkInfo(), networkInfo.isLocalOnlyMode {
                HStack(spacing: 8) {
                    Image(systemName: "wifi.slash")
                        .foregroundColor(.orange)
                    Text("Local-only mode - No external connectivity")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
    }

    // MARK: - My Addresses Section

    private var myAddressesSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("My Addresses", systemImage: "person.badge.key")
                .font(.headline)

            // Identity four-word address
            AddressRow(
                label: "Identity",
                address: state.fourWords,
                icon: "person.fill"
            )

            // Listen address (if networking active)
            if let networkInfo = state.getNetworkInfo(), networkInfo.isActive {
                if let listenAddr = networkInfo.listenAddress {
                    AddressRow(
                        label: "Listen Address",
                        address: listenAddr,
                        icon: "network"
                    )
                }

                // External/public address (NAT-reflected) - shows both IP:port and four-word format
                if let externalAddr = networkInfo.externalAddress {
                    // Show IP:port format
                    AddressRow(
                        label: "External Address",
                        address: externalAddr,
                        icon: "globe"
                    )
                    // Show four-word encoded format (for sharing)
                    if let externalWords = networkInfo.externalAddressWords {
                        AddressRow(
                            label: "Share Address",
                            address: externalWords,
                            icon: "rectangle.and.text.magnifyingglass"
                        )
                    }
                } else {
                    // External address not yet detected or detection failed
                    HStack(spacing: 8) {
                        Image(systemName: "globe")
                            .foregroundColor(.secondary)
                        Text("External Address")
                            .foregroundColor(.secondary)
                        Spacer()
                        if state.externalAddressDetectionFailed {
                            // Detection failed - show "Not available"
                            Text("Not available")
                                .font(.caption)
                                .foregroundColor(.orange)
                        } else {
                            // Still detecting
                            HStack(spacing: 4) {
                                ProgressView()
                                    .scaleEffect(0.6)
                                Text("Detecting...")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                        }
                    }
                    .padding(.vertical, 4)
                }

                if let port = networkInfo.port {
                    AddressRow(
                        label: "Port",
                        address: String(port),
                        icon: "number"
                    )
                }

                if let connId = networkInfo.connectionIdentity {
                    AddressRow(
                        label: "Connection Identity",
                        address: connId,
                        icon: "link"
                    )
                }
            } else {
                Text("Start networking to see connection details")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .italic()
            }
        }
    }

    // MARK: - Bootstrap Nodes Section

    private var bootstrapNodesSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("Bootstrap Nodes", systemImage: "server.rack")
                .font(.headline)

            ForEach(bootstrapNodes, id: \.address) { node in
                BootstrapNodeRow(
                    name: node.name,
                    address: node.address,
                    isNetworkingActive: state.isNetworking
                )
            }
        }
    }

    // MARK: - Connected Peers Section

    private var connectedPeersSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Label("Connected Peers", systemImage: "person.2.fill")
                    .font(.headline)
                Spacer()
                Text("\(state.contacts.filter { $0.isOnline }.count) online")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Button {
                    state.loadContacts()
                } label: {
                    Image(systemName: "arrow.clockwise")
                        .font(.caption)
                }
                .buttonStyle(.plain)
            }

            if !state.isNetworking {
                Text("Start networking to see connected peers")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .italic()
                    .padding(.vertical, 8)
            } else if state.contacts.isEmpty {
                Text("No contacts added yet")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .italic()
                    .padding(.vertical, 8)
            } else {
                ForEach(state.contacts.prefix(10), id: \.id) { contact in
                    PeerRow(contact: contact)
                }

                if state.contacts.count > 10 {
                    Text("+ \(state.contacts.count - 10) more contacts")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
    }

    // MARK: - Connect to Peer Section

    private var connectToPeerSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Label("Connect to Peer", systemImage: "link.badge.plus")
                .font(.headline)

            Text("Enter a four-word address or IP:port to connect directly")
                .font(.caption)
                .foregroundColor(.secondary)

            HStack(spacing: 12) {
                TextField("e.g., ocean-forest-moon-star or 192.168.1.100:50000", text: $connectAddress)
                    .textFieldStyle(.roundedBorder)
                    .disabled(!state.isNetworking || isConnecting)

                Button {
                    connectToPeer()
                } label: {
                    if isConnecting {
                        ProgressView()
                            .scaleEffect(0.7)
                    } else {
                        Text("Connect")
                    }
                }
                .buttonStyle(.borderedProminent)
                .disabled(!state.isNetworking || connectAddress.isEmpty || isConnecting)
            }

            if !state.isNetworking {
                Text("Start networking first to connect to peers")
                    .font(.caption)
                    .foregroundColor(.orange)
            }
        }
    }

    // MARK: - Actions

    private func connectToPeer() {
        guard !connectAddress.isEmpty else { return }

        isConnecting = true
        let address = connectAddress.trimmingCharacters(in: .whitespacesAndNewlines)

        // Determine if it's a four-word address or IP:port
        if address.contains(":") && !address.contains("-") {
            // Looks like IP:port
            state.dialAddress(address)
        } else {
            // Assume it's a four-word address
            state.connectToPeer(fourWords: address)
        }

        // Clear input and reset state after brief delay
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            isConnecting = false
            connectAddress = ""
        }
    }
}

// MARK: - Address Row Component

struct AddressRow: View {
    let label: String
    let address: String
    let icon: String

    var body: some View {
        HStack {
            Image(systemName: icon)
                .foregroundColor(.secondary)
                .frame(width: 20)

            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
                .frame(width: 120, alignment: .leading)

            Text(address)
                .font(.system(.body, design: .monospaced))
                .lineLimit(1)
                .truncationMode(.middle)

            Spacer()

            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(address, forType: .string)
            } label: {
                Image(systemName: "doc.on.doc")
                    .font(.caption)
                    .foregroundColor(.blue)
            }
            .buttonStyle(.plain)
            .help("Copy to clipboard")
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color.gray.opacity(0.05))
        .cornerRadius(6)
    }
}

// MARK: - Bootstrap Node Row Component

struct BootstrapNodeRow: View {
    let name: String
    let address: String
    let isNetworkingActive: Bool

    var body: some View {
        HStack {
            Image(systemName: "server.rack")
                .foregroundColor(isNetworkingActive ? .blue : .secondary)
                .frame(width: 20)

            VStack(alignment: .leading, spacing: 2) {
                Text(name)
                    .font(.caption)
                    .fontWeight(.medium)
                Text(address)
                    .font(.system(.caption2, design: .monospaced))
                    .foregroundColor(.secondary)
            }

            Spacer()

            // Status indicator
            HStack(spacing: 4) {
                Circle()
                    .fill(isNetworkingActive ? Color.green : Color.gray)
                    .frame(width: 6, height: 6)
                Text(isNetworkingActive ? "Active" : "Idle")
                    .font(.caption2)
                    .foregroundColor(.secondary)
            }

            Button {
                NSPasteboard.general.clearContents()
                NSPasteboard.general.setString(address, forType: .string)
            } label: {
                Image(systemName: "doc.on.doc")
                    .font(.caption)
                    .foregroundColor(.blue)
            }
            .buttonStyle(.plain)
            .help("Copy address")
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color.gray.opacity(0.05))
        .cornerRadius(6)
    }
}

// MARK: - Peer Row Component

struct PeerRow: View {
    let contact: ContactItem

    var body: some View {
        HStack {
            // Online/Local-only indicator
            if contact.isLocalOnly {
                // Local-only: lock icon
                Image(systemName: "lock.fill")
                    .font(.system(size: 8))
                    .foregroundColor(.orange)
                    .frame(width: 8, height: 8)
            } else {
                // Online indicator
                Circle()
                    .fill(contact.isOnline ? Color.green : Color.gray.opacity(0.3))
                    .frame(width: 8, height: 8)
            }

            // Contact info
            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 4) {
                    Text(contact.effectiveName)
                        .font(.caption)
                        .fontWeight(.medium)

                    // Local badge
                    if contact.isLocalOnly {
                        Text("Local")
                            .font(.system(size: 8))
                            .foregroundColor(.orange)
                            .padding(.horizontal, 4)
                            .padding(.vertical, 1)
                            .background(Color.orange.opacity(0.15))
                            .cornerRadius(3)
                    }
                }

                if let fourWords = contact.fourWords {
                    Text(fourWords)
                        .font(.system(.caption2, design: .monospaced))
                        .foregroundColor(.secondary)
                } else {
                    Text("No network identity")
                        .font(.system(.caption2))
                        .foregroundColor(.secondary)
                        .italic()
                }
            }

            Spacer()

            // Last seen endpoint (if available and linked)
            if !contact.isLocalOnly, let endpoint = contact.lastSeenEndpoint {
                Text(endpoint)
                    .font(.system(.caption2, design: .monospaced))
                    .foregroundColor(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .frame(maxWidth: 150)
            }

            // Status
            if contact.isLocalOnly {
                Text("Local")
                    .font(.caption2)
                    .foregroundColor(.orange)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Color.orange.opacity(0.1))
                    .cornerRadius(4)
            } else {
                Text(contact.isOnline ? "Online" : "Offline")
                    .font(.caption2)
                    .foregroundColor(contact.isOnline ? .green : .secondary)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(contact.isOnline ? Color.green.opacity(0.1) : Color.gray.opacity(0.1))
                    .cornerRadius(4)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color.gray.opacity(0.03))
        .cornerRadius(6)
    }
}

#Preview {
    NetworkPanelView()
        .environmentObject(AppState())
        .frame(width: 600, height: 800)
}

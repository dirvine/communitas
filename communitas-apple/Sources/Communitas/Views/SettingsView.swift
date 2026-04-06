import SwiftUI
import X0xClient

struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @AppStorage("daemonURL") private var daemonURL = "http://127.0.0.1:12700"
    @AppStorage("displayName") private var displayName = ""
    @State private var agentCardLink: String?
    @State private var generatingCard = false

    var body: some View {
        Form {
            Section("Daemon Connection") {
                TextField("Daemon URL", text: $daemonURL)
                    .font(.system(.body, design: .monospaced))
                    .help("The HTTP URL where x0xd is listening.")

                HStack {
                    statusDot
                    Text(daemonStateLabel)
                        .foregroundStyle(.secondary)
                }
            }

            Section("Profile") {
                TextField("Display Name", text: $displayName)
                    .help("Your name as shown to other peers.")
                    .onSubmit {
                        let name = displayName.trimmingCharacters(in: .whitespaces)
                        appState.displayName = name.isEmpty
                            ? String((appState.agentIdentity?.agentId ?? "unknown").prefix(8))
                            : name
                        Task {
                            // Propagate to daemon so peers see the updated name
                            let cardName = name.isEmpty ? nil : name
                            _ = try? await appState.client.agentCard(
                                displayName: cardName,
                                includeGroups: false
                            )
                        }
                    }
            }

            Section("Identity") {
                if let identity = appState.agentIdentity {
                    LabeledContent("Agent ID") {
                        Text(identity.agentId)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                    if let machineId = identity.machineId {
                        LabeledContent("Machine ID") {
                            Text(machineId)
                                .font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled)
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                    }
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Button {
                                generateAgentCard(includeGroups: true)
                            } label: {
                                Label(generatingCard ? "Generating..." : "Generate Share Link", systemImage: "link")
                            }
                            .disabled(generatingCard || appState.daemonState != .running)

                            if let link = agentCardLink {
                                Button {
                                    NSPasteboard.general.clearContents()
                                    NSPasteboard.general.setString(link, forType: .string)
                                } label: {
                                    Label("Copy Link", systemImage: "doc.on.doc")
                                }
                            }
                        }
                        if let link = agentCardLink {
                            Text(link)
                                .font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled)
                                .lineLimit(3)
                        }
                    }
                    .padding(.top, 4)
                } else {
                    Text("Not connected to daemon.")
                        .foregroundStyle(.secondary)
                }
            }

            Section("About") {
                LabeledContent("App") {
                    Text("Communitas")
                }
                LabeledContent("Framework") {
                    Text("SwiftUI + x0x")
                }
                LabeledContent("Daemon Port") {
                    Text("12700")
                }
            }
        }
        .formStyle(.grouped)
        .navigationTitle("Settings")
        .task {
            guard agentCardLink == nil, appState.daemonState == .running else { return }
            await loadAgentCardLink()
        }
    }

    private func loadAgentCardLink() async {
        do {
            let resp = try await appState.client.agentCard(displayName: nil, includeGroups: true)
            agentCardLink = resp.link
        } catch {
            agentCardLink = nil
        }
    }

    private func generateAgentCard(includeGroups: Bool) {
        generatingCard = true
        Task {
            defer { generatingCard = false }
            do {
                let name = displayName.trimmingCharacters(in: .whitespaces)
                let resp = try await appState.client.agentCard(
                    displayName: name.isEmpty ? nil : name,
                    includeGroups: includeGroups
                )
                await MainActor.run {
                    agentCardLink = resp.link
                }
            } catch {
                await MainActor.run {
                    appState.errorMessage = error.localizedDescription
                }
            }
        }
    }

    private var daemonStateLabel: String {
        switch appState.daemonState {
        case .running: return "Running"
        case .starting: return "Starting..."
        case .notRunning: return "Not Running"
        case .notInstalled: return "Not Installed"
        case .error: return "Error"
        }
    }

    private var statusDot: some View {
        Circle()
            .fill(appState.daemonState == .running ? .green : .gray)
            .frame(width: 8, height: 8)
    }
}

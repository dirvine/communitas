import SwiftUI
import X0xClient

struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @AppStorage("daemonURL") private var daemonURL = "http://127.0.0.1:12700"
    @AppStorage("displayName") private var displayName = ""

    var body: some View {
        Form {
            Section("Daemon Connection") {
                TextField("Daemon URL", text: $daemonURL)
                    .font(.system(.body, design: .monospaced))
                    .help("The HTTP URL where x0xd is listening.")

                HStack {
                    statusDot
                    Text(appState.daemonState.rawValue.replacingOccurrences(of: "n", with: "N", options: [], range: nil))
                        .foregroundStyle(.secondary)
                }
            }

            Section("Profile") {
                TextField("Display Name", text: $displayName)
                    .help("Your name as shown to other peers.")
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
    }

    private var statusDot: some View {
        Circle()
            .fill(appState.daemonState == .running ? .green : .gray)
            .frame(width: 8, height: 8)
    }
}

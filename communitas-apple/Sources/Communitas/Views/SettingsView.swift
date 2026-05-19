import SwiftUI
import X0xClient

struct SettingsView: View {
    @EnvironmentObject var appState: AppState
    @EnvironmentObject var updaterController: UpdaterController
    @AppStorage("daemonURL") private var daemonURL = "http://127.0.0.1:12700"
    @AppStorage("displayName") private var displayName = ""
    @State private var agentCardLink: String?
    @State private var generatingCard = false
    @State private var exportStatus: String?
    @State private var importedAgentId: String?
    @State private var showImportSheet = false
    @State private var importBuffer: String = ""
    @State private var daemonUpdateStatus: String?
    @State private var daemonUpdateAvailableVersion: String?
    @State private var isCheckingDaemonUpdate = false
    @State private var isApplyingDaemonUpdate = false

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
                            .accessibilityIdentifier("settings-agent-id")
                    }
                    if let machineId = identity.machineId {
                        LabeledContent("Machine ID") {
                            Text(machineId)
                                .font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled)
                                .lineLimit(1)
                                .truncationMode(.middle)
                                .accessibilityIdentifier("settings-machine-id")
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
                        HStack {
                            Button {
                                exportKeypairBackup()
                            } label: {
                                Label("Export Identity Backup…", systemImage: "externaldrive.badge.plus")
                            }
                            .disabled(appState.daemonState != .running || appState.agentIdentity == nil)
                            .accessibilityIdentifier("export-keypair-button")

                            Button {
                                showImportSheet = true
                            } label: {
                                Label("Import Identity Card…", systemImage: "square.and.arrow.down")
                            }
                            .disabled(appState.daemonState != .running)
                            .accessibilityIdentifier("import-keypair-button")
                        }
                        if let link = agentCardLink {
                            Text(link)
                                .font(.system(.caption, design: .monospaced))
                                .textSelection(.enabled)
                                .lineLimit(3)
                                .accessibilityIdentifier("settings-agent-card-link")
                        }
                        if let s = exportStatus {
                            Text(s).font(.caption).foregroundStyle(.secondary)
                                .accessibilityIdentifier("export-keypair-status")
                        }
                        if let id = importedAgentId {
                            Text("Imported: \(id.prefix(12))…").font(.caption)
                                .accessibilityIdentifier("imported-agent-id")
                        }
                    }
                    .padding(.top, 4)
                } else {
                    Text("Not connected to daemon.")
                        .foregroundStyle(.secondary)
                }
            }

            Section("Software Updates") {
                Button {
                    updaterController.checkForUpdates()
                } label: {
                    Label("Check for Updates…", systemImage: "arrow.down.circle")
                }
                .disabled(!updaterController.canCheckForUpdates)
            }

            Section("x0xd Daemon Updates") {
                if let status = daemonUpdateStatus {
                    Text(status)
                        .font(.caption)
                        .foregroundStyle(daemonUpdateStatusColor)
                }

                HStack {
                    Button {
                        checkDaemonUpdate()
                    } label: {
                        Label(isCheckingDaemonUpdate ? "Checking..." : "Check x0xd for Updates", systemImage: "arrow.clockwise.circle")
                    }
                    .disabled(isCheckingDaemonUpdate || isApplyingDaemonUpdate || appState.daemonState != .running)

                    if let version = daemonUpdateAvailableVersion {
                        Button {
                            applyDaemonUpdate(version: version)
                        } label: {
                            Label(isApplyingDaemonUpdate ? "Applying..." : "Apply Update (\(version))", systemImage: "arrow.down.doc")
                        }
                        .disabled(isApplyingDaemonUpdate || appState.daemonState != .running)
                        .foregroundStyle(.green)
                    }
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
        .sheet(isPresented: $showImportSheet) {
            VStack(alignment: .leading, spacing: 12) {
                Text("Import Identity Card").font(.headline)
                Text("Paste an `x0x://agent/...` link or the JSON card contents.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                TextEditor(text: $importBuffer)
                    .frame(minHeight: 100)
                    .border(Color.secondary.opacity(0.2))
                    .accessibilityIdentifier("import-keypair-buffer")
                HStack {
                    Button("Cancel") { showImportSheet = false }
                    Spacer()
                    Button("Import") {
                        importKeypair()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(importBuffer.trimmingCharacters(in: .whitespaces).isEmpty)
                    .accessibilityIdentifier("import-keypair-confirm")
                }
            }
            .padding(20)
            .frame(width: 480)
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

    /// Export a consent-gated private identity backup to a JSON file.
    ///
    /// This writes the local x0x private key files (`agent.key`,
    /// `machine.key`, optional `user.key`, optional `agent.cert`, and
    /// optional `agent_kem.key`) into an explicit backup bundle. Agent
    /// cards remain available above as shareable public metadata; they
    /// are not key backups.
    private func exportKeypairBackup() {
        guard let identity = appState.agentIdentity else {
            exportStatus = "Export failed: daemon identity is not loaded."
            return
        }

        let panel = NSSavePanel()
        panel.canCreateDirectories = true
        panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "x0x-identity-backup-\(String(identity.agentId.prefix(8))).json"
        panel.title = "Export x0x Identity Backup"
        let response = panel.runModal()
        guard response == .OK, let url = panel.url else {
            exportStatus = "Export cancelled."
            return
        }

        // Prompt for passphrase using native NSAlert + NSSecureTextField
        let alert = NSAlert()
        alert.messageText = "Set Backup Passphrase"
        alert.informativeText = "Please enter a passphrase to encrypt your identity backup. You will need this passphrase to restore your identity in the future."
        alert.alertStyle = .warning

        let input = NSSecureTextField(frame: NSRect(x: 0, y: 0, width: 280, height: 24))
        input.placeholderString = "Passphrase"
        alert.accessoryView = input

        alert.addButton(withTitle: "OK")
        alert.addButton(withTitle: "Cancel")

        let alertResponse = alert.runModal()
        guard alertResponse == .alertFirstButtonReturn else {
            exportStatus = "Export cancelled: passphrase required."
            return
        }

        let passphrase = input.stringValue
        guard !passphrase.isEmpty else {
            exportStatus = "Export failed: passphrase cannot be empty."
            return
        }

        do {
            let bundle = try IdentityBackupExporter.exportBundle(
                agentId: identity.agentId,
                machineId: identity.machineId
            )
            try IdentityBackupExporter.writeBundle(bundle, to: url, with: passphrase)
            exportStatus = "Exported encrypted identity backup to \(url.lastPathComponent)"
        } catch {
            exportStatus = "Export failed: \(error.localizedDescription)"
        }
    }

    /// Import a shareable agent card so the daemon stores the contact and
    /// we can verify the `agent_id` matches the source card. This is not
    /// a private key restore path.
    private func importKeypair() {
        let trimmed = importBuffer.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        Task {
            do {
                let payload: String
                if trimmed.hasPrefix("x0x://agent/") {
                    payload = trimmed
                } else {
                    // Allow the wrapped JSON shape produced by Export.
                    if let data = trimmed.data(using: .utf8),
                       let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                       let link = obj["link"] as? String {
                        payload = link
                    } else {
                        payload = trimmed
                    }
                }
                let resp = try await appState.client.importAgentCard(card: payload, trustLevel: .known)
                importedAgentId = resp.agentId
                showImportSheet = false
                importBuffer = ""
                await appState.refresh()
            } catch {
                exportStatus = "Import failed: \(error.localizedDescription)"
            }
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

    private var daemonUpdateStatusColor: Color {
        if isApplyingDaemonUpdate || isCheckingDaemonUpdate {
            return .secondary
        }
        if daemonUpdateAvailableVersion != nil {
            return .blue
        }
        if let status = daemonUpdateStatus {
            if status.contains("applied") || status.contains("up to date") {
                return .green
            }
            if status.contains("failed") || status.contains("Error") {
                return .red
            }
        }
        return .primary
    }

    private func checkDaemonUpdate() {
        isCheckingDaemonUpdate = true
        daemonUpdateStatus = "Checking x0xd for updates..."
        daemonUpdateAvailableVersion = nil
        Task {
            do {
                let status = try await appState.client.checkUpgrade()
                await MainActor.run {
                    isCheckingDaemonUpdate = false
                    if status.updateAvailable == true {
                        let version = status.version ?? status.currentVersion ?? "new version"
                        daemonUpdateAvailableVersion = version
                        daemonUpdateStatus = "x0xd update \(version) is available."
                    } else {
                        daemonUpdateStatus = "x0xd reports that it is up to date."
                    }
                }
            } catch {
                await MainActor.run {
                    isCheckingDaemonUpdate = false
                    daemonUpdateStatus = "x0xd update check failed: \(error.localizedDescription)"
                }
            }
        }
    }

    private func applyDaemonUpdate(version: String) {
        isApplyingDaemonUpdate = true
        daemonUpdateStatus = "Applying x0xd update..."
        Task {
            do {
                let resp = try await appState.client.applyUpgrade()
                await MainActor.run {
                    isApplyingDaemonUpdate = false
                    daemonUpdateAvailableVersion = nil
                    if resp.applied {
                        daemonUpdateStatus = "x0xd update applied: \(resp.version ?? version)"
                    } else {
                        daemonUpdateStatus = "x0xd update not applied: \(resp.reason ?? "no upgrade required")"
                    }
                }
            } catch {
                await MainActor.run {
                    isApplyingDaemonUpdate = false
                    daemonUpdateStatus = "x0xd update application failed: \(error.localizedDescription)"
                }
            }
        }
    }
}

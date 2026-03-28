import SwiftUI
import X0xClient

/// File transfer view showing incoming files, active transfers, and send file UI.
struct FilesView: View {
    @EnvironmentObject var appState: AppState

    @State private var transfers: [FileTransfer] = []
    @State private var isLoading = true
    @State private var errorMessage: String?
    @State private var selectedAgentId = ""
    @State private var showFilePicker = false
    @State private var pollTask: Task<Void, Never>?

    var body: some View {
        VStack(spacing: 0) {
            filesHeader
            Divider()

            if isLoading {
                ProgressView("Loading transfers...")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    VStack(alignment: .leading, spacing: 20) {
                        incomingSection
                        activeTransfersSection
                        sendFileSection
                    }
                    .padding(16)
                }
            }
        }
        .task {
            await refreshTransfers()
            isLoading = false
            startPolling()
        }
        .onDisappear {
            pollTask?.cancel()
            pollTask = nil
        }
        .fileImporter(isPresented: $showFilePicker, allowedContentTypes: [.data]) { result in
            if case .success(let url) = result {
                Task { await sendFileAction(url: url) }
            }
        }
    }

    private var filesHeader: some View {
        HStack {
            Image(systemName: "doc.on.doc")
                .foregroundStyle(.secondary)
            Text("Files")
                .font(.headline)
            Spacer()
            Button {
                Task { await refreshTransfers() }
            } label: {
                Label("Refresh", systemImage: "arrow.clockwise")
                    .font(.caption)
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
        .background(.bar)
    }

    // MARK: - Incoming Files

    private var incomingSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Incoming", systemImage: "arrow.down.circle")
                .font(.subheadline)
                .fontWeight(.semibold)

            let incoming = transfers.filter { $0.direction == .download && $0.status == .pending }
            if incoming.isEmpty {
                Text("No pending incoming files.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                ForEach(incoming) { transfer in
                    incomingFileRow(transfer)
                }
            }
        }
    }

    private func incomingFileRow(_ transfer: FileTransfer) -> some View {
        HStack {
            Image(systemName: "doc.badge.arrow.down")
                .foregroundStyle(.blue)
            VStack(alignment: .leading, spacing: 2) {
                Text(transfer.filename)
                    .font(.subheadline)
                Text(formatSize(transfer.size))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            Button("Accept") {
                Task { await acceptAction(transfer) }
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.small)

            Button("Reject") {
                Task { await rejectAction(transfer) }
            }
            .buttonStyle(.bordered)
            .controlSize(.small)
            .tint(.red)
        }
        .padding(10)
        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }

    // MARK: - Active Transfers

    private var activeTransfersSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Active Transfers", systemImage: "arrow.left.arrow.right")
                .font(.subheadline)
                .fontWeight(.semibold)

            let active = transfers.filter { $0.status == .inProgress || $0.status == .completed }
            if active.isEmpty {
                Text("No active transfers.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.vertical, 8)
            } else {
                ForEach(active) { transfer in
                    transferRow(transfer)
                }
            }
        }
    }

    private func transferRow(_ transfer: FileTransfer) -> some View {
        HStack(spacing: 10) {
            Image(systemName: transfer.direction == .upload ? "arrow.up.doc" : "arrow.down.doc")
                .foregroundStyle(transfer.direction == .upload ? .orange : .blue)

            VStack(alignment: .leading, spacing: 2) {
                Text(transfer.filename)
                    .font(.subheadline)
                Text(formatSize(transfer.size))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            statusBadge(transfer.status)

            if let progress = transfer.progress {
                ProgressView(value: progress)
                    .frame(width: 80)
            }
        }
        .padding(10)
        .background(Color.secondary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
    }

    private func statusBadge(_ status: TransferStatus) -> some View {
        Text(status.rawValue.replacingOccurrences(of: "_", with: " ").capitalized)
            .font(.caption2)
            .padding(.horizontal, 8)
            .padding(.vertical, 2)
            .background(statusColor(status).opacity(0.15))
            .foregroundStyle(statusColor(status))
            .clipShape(Capsule())
    }

    private func statusColor(_ status: TransferStatus) -> Color {
        switch status {
        case .pending: return .gray
        case .inProgress: return .blue
        case .completed: return .green
        case .failed: return .red
        case .cancelled: return .orange
        }
    }

    // MARK: - Send File

    private var sendFileSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Label("Send File", systemImage: "arrow.up.doc")
                .font(.subheadline)
                .fontWeight(.semibold)

            HStack(spacing: 8) {
                Picker("To Agent", selection: $selectedAgentId) {
                    Text("Select contact...").tag("")
                    ForEach(appState.contacts) { contact in
                        Text(contact.label ?? truncatedId(contact.agentId))
                            .tag(contact.agentId)
                    }
                }
                .frame(maxWidth: 240)

                Button {
                    showFilePicker = true
                } label: {
                    Label("Choose File", systemImage: "folder")
                }
                .buttonStyle(.borderedProminent)
                .disabled(selectedAgentId.isEmpty)
            }
        }
    }

    // MARK: - Actions

    private func refreshTransfers() async {
        do {
            transfers = try await appState.client.listTransfers()
        } catch {
            // Silently ignore during polling
        }
    }

    private func acceptAction(_ transfer: FileTransfer) async {
        do {
            try await appState.client.acceptFile(transferId: transfer.transferId)
            await refreshTransfers()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func rejectAction(_ transfer: FileTransfer) async {
        do {
            try await appState.client.rejectFile(transferId: transfer.transferId)
            await refreshTransfers()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func sendFileAction(url: URL) async {
        guard !selectedAgentId.isEmpty else { return }
        let filename = url.lastPathComponent
        let size: UInt64
        do {
            let attributes = try FileManager.default.attributesOfItem(atPath: url.path)
            size = (attributes[.size] as? UInt64) ?? 0
        } catch {
            errorMessage = "Cannot read file: \(error.localizedDescription)"
            return
        }

        do {
            _ = try await appState.client.sendFile(agentId: selectedAgentId, filename: filename, size: size)
            await refreshTransfers()
        } catch {
            errorMessage = "Failed to send: \(error.localizedDescription)"
        }
    }

    private func startPolling() {
        pollTask?.cancel()
        pollTask = Task { @MainActor in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 5_000_000_000)
                guard !Task.isCancelled else { break }
                await refreshTransfers()
            }
        }
    }

    private func formatSize(_ bytes: UInt64) -> String {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter.string(fromByteCount: Int64(bytes))
    }

    private func truncatedId(_ id: String) -> String {
        if id.count > 16 {
            return String(id.prefix(8)) + "..." + String(id.suffix(6))
        }
        return id
    }
}

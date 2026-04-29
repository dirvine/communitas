import SwiftUI
import X0xClient

/// Named-group discovery surface.
///
/// Exposes the Phase C + C.2 distributed discovery index via the x0xd
/// REST API: merged local + shard-cache search (`GET /groups/discover`)
/// and the shard-only "Nearby" witness (`GET /groups/discover/nearby`).
/// Non-members can submit join requests inline for RequestAccess groups.
struct GroupDiscoveryView: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    enum Mode: String, CaseIterable {
        case all = "All"
        case nearby = "Nearby"
    }

    @State private var mode: Mode = .all
    @State private var query: String = ""
    @State private var cards: [GroupCard] = []
    @State private var lastError: String?
    @State private var requestStatus: [String: RequestState] = [:]
    @State private var refreshTask: Task<Void, Never>?

    var body: some View {
        VStack(spacing: 12) {
            HStack {
                Text("Discover Groups")
                    .font(.title2)
                    .fontWeight(.semibold)
                Spacer()
                Button("Done") { dismiss() }
                    .keyboardShortcut(.cancelAction)
            }

            HStack(spacing: 12) {
                TextField("Search by tag, name, or ID…", text: $query)
                    .textFieldStyle(.roundedBorder)
                    .disabled(mode == .nearby)
                    .accessibilityIdentifier("group-discover-query")
                    .onChange(of: query) { _, _ in
                        Task { await reload() }
                    }

                Picker("Mode", selection: $mode) {
                    ForEach(Mode.allCases, id: \.self) { m in
                        Text(m.rawValue).tag(m)
                    }
                }
                .pickerStyle(.segmented)
                .fixedSize()
                .accessibilityIdentifier("group-discover-mode-picker")
                .onChange(of: mode) { _, _ in
                    Task { await reload() }
                }
            }

            if let err = lastError {
                Text(err)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }

            if cards.isEmpty {
                emptyState
            } else {
                List {
                    ForEach(cards) { card in
                        DiscoveryCardRow(
                            card: card,
                            status: requestStatus[card.groupId] ?? .idle,
                            onRequest: { submitRequest(for: card) }
                        )
                        .accessibilityIdentifier("group-discover-row-\(card.groupId)")
                    }
                }
                .listStyle(.inset)
                .accessibilityIdentifier("group-discover-list")
            }
        }
        .padding(20)
        .frame(width: 640, height: 520)
        .task {
            await reload()
            refreshTask = Task {
                while !Task.isCancelled {
                    try? await Task.sleep(for: .seconds(10))
                    if Task.isCancelled { break }
                    await reload()
                }
            }
        }
        .onDisappear {
            refreshTask?.cancel()
        }
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Image(systemName: mode == .nearby ? "sparkles" : "magnifyingglass")
                .font(.system(size: 32))
                .foregroundStyle(.secondary)
            Text(
                mode == .nearby
                    ? "No PublicDirectory groups observed on the shard plane yet."
                    : query.trimmingCharacters(in: .whitespaces).isEmpty
                        ? "No discoverable groups observed yet."
                        : "No matches — try a different tag, name, or ID."
            )
            .font(.caption)
            .foregroundStyle(.secondary)
            .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func reload() async {
        do {
            let fetched: [GroupCard]
            switch mode {
            case .all:
                let trimmed = query.trimmingCharacters(in: .whitespaces)
                fetched = try await appState.client.discoverGroups(
                    query: trimmed.isEmpty ? nil : trimmed
                )
            case .nearby:
                fetched = try await appState.client.discoverGroupsNearby()
            }
            await MainActor.run {
                cards = fetched.sorted { lhs, rhs in
                    if lhs.revision != rhs.revision {
                        return lhs.revision > rhs.revision
                    }
                    return lhs.name.localizedCaseInsensitiveCompare(rhs.name) == .orderedAscending
                }
                lastError = nil
            }
        } catch {
            await MainActor.run {
                lastError = "Discovery failed: \(error.localizedDescription)"
            }
        }
    }

    private func submitRequest(for card: GroupCard) {
        let gid = card.groupId
        requestStatus[gid] = .pending
        Task {
            do {
                _ = try await appState.client.createJoinRequest(groupId: gid, message: nil)
                await MainActor.run { requestStatus[gid] = .submitted }
            } catch {
                await MainActor.run {
                    requestStatus[gid] = .failed(error.localizedDescription)
                }
            }
        }
    }
}

private enum RequestState: Equatable {
    case idle
    case pending
    case submitted
    case failed(String)
}

private struct DiscoveryCardRow: View {
    let card: GroupCard
    let status: RequestState
    let onRequest: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(card.name).font(.headline)
                    Text("rev \(card.revision) · \(card.memberCount) members")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                badges
            }

            if !card.description.isEmpty {
                Text(card.description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(3)
            }

            if !card.tags.isEmpty {
                HStack(spacing: 4) {
                    ForEach(card.tags, id: \.self) { tag in
                        Text("#\(tag)")
                            .font(.caption2)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.secondary.opacity(0.15))
                            .clipShape(Capsule())
                    }
                }
            }

            HStack {
                switch status {
                case .submitted:
                    Text("Request submitted — awaiting admin review.")
                        .font(.caption2)
                        .foregroundStyle(.green)
                case .failed(let msg):
                    Text("Request failed: \(msg)")
                        .font(.caption2)
                        .foregroundStyle(.red)
                default:
                    EmptyView()
                }

                Spacer()

                if canRequest {
                    Button(requestLabel) { onRequest() }
                        .buttonStyle(.borderedProminent)
                        .controlSize(.small)
                        .disabled(status == .pending || status == .submitted)
                }
            }
        }
        .padding(.vertical, 4)
    }

    private var canRequest: Bool {
        card.requestAccessEnabled
            && card.policySummary.admission == .requestAccess
            && !card.withdrawn
    }

    private var requestLabel: String {
        switch status {
        case .pending: return "Requesting…"
        case .submitted: return "Requested"
        default: return "Request access"
        }
    }

    @ViewBuilder
    private var badges: some View {
        HStack(spacing: 4) {
            Badge(text: discoverabilityLabel(card.policySummary.discoverability))
            Badge(text: admissionLabel(card.policySummary.admission))
            Badge(text: confidentialityLabel(card.policySummary.confidentiality))
        }
    }
}

private struct Badge: View {
    let text: String
    var body: some View {
        Text(text)
            .font(.caption2)
            .padding(.horizontal, 6)
            .padding(.vertical, 2)
            .background(Color.secondary.opacity(0.12))
            .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

private func discoverabilityLabel(_ v: GroupDiscoverability) -> String {
    switch v {
    case .hidden: return "hidden"
    case .listedToContacts: return "contacts"
    case .publicDirectory: return "public"
    }
}

private func admissionLabel(_ v: GroupAdmission) -> String {
    switch v {
    case .inviteOnly: return "invite only"
    case .requestAccess: return "request access"
    case .openJoin: return "open join"
    }
}

private func confidentialityLabel(_ v: GroupConfidentiality) -> String {
    switch v {
    case .mlsEncrypted: return "encrypted"
    case .signedPublic: return "signed public"
    }
}

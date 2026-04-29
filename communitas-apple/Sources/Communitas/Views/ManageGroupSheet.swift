import SwiftUI
import X0xClient

/// Admin panel for a named group.
///
/// Mirrors the Dioxus `SpaceAdminPanel`: policy, signed state-commit
/// chain, roster with role / ban controls, and join-request approvals.
/// Gated degrades gracefully — non-admins see a read-only view and get
/// clear errors when the daemon rejects admin actions with 403.
struct ManageGroupSheet: View {
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    let group: GroupSummary

    @State private var members: [NamedGroupMember] = []
    @State private var requests: [JoinRequest] = []
    @State private var state: GroupStateResponse?
    @State private var agentId: String?
    @State private var lastError: String?
    @State private var refreshTask: Task<Void, Never>?

    // Policy editor state.
    @State private var editPreset: GroupPolicyPreset? = nil
    @State private var editDiscoverability: GroupDiscoverability? = nil
    @State private var editAdmission: GroupAdmission? = nil
    @State private var editConfidentiality: GroupConfidentiality? = nil
    @State private var editReadAccess: GroupReadAccess? = nil
    @State private var editWriteAccess: GroupWriteAccess? = nil
    @State private var policyFeedback: String?

    // Per-member / per-request feedback.
    @State private var memberFeedback: [String: String] = [:]
    @State private var requestFeedback: [String: String] = [:]
    @State private var stateFeedback: String?

    private var callerRole: GroupRole? {
        guard let aid = agentId else { return nil }
        return members.first(where: { $0.agentId == aid })?.role
    }

    private var isOwner: Bool { callerRole == .owner }

    private var isAdminOrAbove: Bool {
        switch callerRole {
        case .owner, .admin: return true
        default: return false
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(group.name).font(.title2).fontWeight(.semibold)
                    Text(shorten(group.groupId))
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                }
                Spacer()
                Button("Done") { dismiss() }
                    .keyboardShortcut(.cancelAction)
            }
            .padding()

            Divider()

            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    if let err = lastError {
                        Text(err)
                            .font(.caption)
                            .foregroundStyle(.red)
                    }

                    policySection
                    stateSection
                    rosterSection
                    if isAdminOrAbove {
                        requestsSection
                    }
                }
                .padding()
            }
        }
        .frame(width: 720, height: 680)
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
        .onDisappear { refreshTask?.cancel() }
    }

    // MARK: - Policy section

    @ViewBuilder
    private var policySection: some View {
        SectionCard(title: "Policy", subtitle: "Five-axis access control (owner only).") {
            if !isOwner {
                Text("Only the owner can change policy. Contact an admin to request changes.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 8) {
                GridRow {
                    Text("Preset").frame(maxWidth: 140, alignment: .leading)
                    Picker("", selection: $editPreset) {
                        Text("(leave unchanged)").tag(Optional<GroupPolicyPreset>.none)
                        ForEach(GroupPolicyPreset.allCases, id: \.self) { p in
                            Text(p.rawValue).tag(Optional(p))
                        }
                    }
                    .labelsHidden()
                    .disabled(!isOwner)
                }
                GridRow {
                    Text("Discoverability")
                    Picker("", selection: $editDiscoverability) {
                        Text("(leave unchanged)").tag(Optional<GroupDiscoverability>.none)
                        ForEach(GroupDiscoverability.allCases, id: \.self) { v in
                            Text(v.rawValue).tag(Optional(v))
                        }
                    }
                    .labelsHidden()
                    .disabled(!isOwner)
                }
                GridRow {
                    Text("Admission")
                    Picker("", selection: $editAdmission) {
                        Text("(leave unchanged)").tag(Optional<GroupAdmission>.none)
                        ForEach(GroupAdmission.allCases, id: \.self) { v in
                            Text(v.rawValue).tag(Optional(v))
                        }
                    }
                    .labelsHidden()
                    .disabled(!isOwner)
                }
                GridRow {
                    Text("Confidentiality")
                    Picker("", selection: $editConfidentiality) {
                        Text("(leave unchanged)").tag(Optional<GroupConfidentiality>.none)
                        ForEach(GroupConfidentiality.allCases, id: \.self) { v in
                            Text(v.rawValue).tag(Optional(v))
                        }
                    }
                    .labelsHidden()
                    .disabled(!isOwner)
                }
                GridRow {
                    Text("Read access")
                    Picker("", selection: $editReadAccess) {
                        Text("(leave unchanged)").tag(Optional<GroupReadAccess>.none)
                        ForEach(GroupReadAccess.allCases, id: \.self) { v in
                            Text(v.rawValue).tag(Optional(v))
                        }
                    }
                    .labelsHidden()
                    .disabled(!isOwner)
                }
                GridRow {
                    Text("Write access")
                    Picker("", selection: $editWriteAccess) {
                        Text("(leave unchanged)").tag(Optional<GroupWriteAccess>.none)
                        ForEach(GroupWriteAccess.allCases, id: \.self) { v in
                            Text(v.rawValue).tag(Optional(v))
                        }
                    }
                    .labelsHidden()
                    .disabled(!isOwner)
                }
            }

            if let feedback = policyFeedback {
                Text(feedback).font(.caption).foregroundStyle(.secondary)
            }

            if isOwner {
                HStack {
                    Spacer()
                    Button("Apply policy change") { applyPolicy() }
                        .buttonStyle(.borderedProminent)
                        .disabled(!hasPolicyChange)
                        .accessibilityIdentifier("apply-policy-button")
                }
            }
        }
        .accessibilityIdentifier("group-policy-section")
    }

    private var hasPolicyChange: Bool {
        editPreset != nil
            || editDiscoverability != nil
            || editAdmission != nil
            || editConfidentiality != nil
            || editReadAccess != nil
            || editWriteAccess != nil
    }

    private func applyPolicy() {
        let patch = UpdateGroupPolicyRequest(
            preset: editPreset?.rawValue,
            discoverability: editDiscoverability,
            admission: editAdmission,
            confidentiality: editConfidentiality,
            readAccess: editReadAccess,
            writeAccess: editWriteAccess
        )
        Task {
            do {
                try await appState.client.updateGroupPolicy(
                    groupId: group.groupId,
                    patch: patch
                )
                await MainActor.run {
                    policyFeedback = "Policy updated."
                    editPreset = nil
                    editDiscoverability = nil
                    editAdmission = nil
                    editConfidentiality = nil
                    editReadAccess = nil
                    editWriteAccess = nil
                }
                await reload()
            } catch {
                await MainActor.run {
                    policyFeedback = "Failed: \(error.localizedDescription)"
                }
            }
        }
    }

    // MARK: - State chain section

    @ViewBuilder
    private var stateSection: some View {
        SectionCard(
            title: "State chain (Phase D.3)",
            subtitle: "Signed commit binds roster, policy, public metadata, and MLS epoch."
        ) {
            if let state = state {
                Grid(alignment: .leading, horizontalSpacing: 12, verticalSpacing: 4) {
                    GridRow {
                        Text("revision").foregroundStyle(.secondary)
                        Text("\(state.stateRevision)").font(.system(.caption, design: .monospaced))
                    }
                    GridRow {
                        Text("status").foregroundStyle(.secondary)
                        Text(state.withdrawn ? "withdrawn" : "active")
                            .foregroundStyle(state.withdrawn ? .red : .green)
                    }
                    GridRow {
                        Text("state_hash").foregroundStyle(.secondary)
                        Text(state.stateHash)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(2)
                    }
                    GridRow {
                        Text("prev_hash").foregroundStyle(.secondary)
                        Text(state.prevStateHash ?? "(genesis)")
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(2)
                    }
                    GridRow {
                        Text("roster_root").foregroundStyle(.secondary)
                        Text(state.rosterRoot)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(2)
                    }
                    GridRow {
                        Text("policy_hash").foregroundStyle(.secondary)
                        Text(state.policyHash)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(2)
                    }
                    GridRow {
                        Text("public_meta").foregroundStyle(.secondary)
                        Text(state.publicMetaHash)
                            .font(.system(.caption, design: .monospaced))
                            .textSelection(.enabled)
                            .lineLimit(2)
                    }
                    GridRow {
                        Text("security").foregroundStyle(.secondary)
                        Text(state.securityBinding ?? "(none)")
                            .font(.system(.caption, design: .monospaced))
                    }
                }
            } else {
                Text("Loading state chain…").font(.caption).foregroundStyle(.secondary)
            }

            if let feedback = stateFeedback {
                Text(feedback).font(.caption).foregroundStyle(.secondary)
            }

            if isOwner {
                HStack {
                    Button("Seal state") { sealState() }
                        .buttonStyle(.borderedProminent)
                    Button("Withdraw (hide publicly)", role: .destructive) { withdrawState() }
                }
            } else {
                Text("Only the owner can seal or withdraw the state chain.")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func sealState() {
        Task {
            do {
                try await appState.client.sealGroupState(groupId: group.groupId)
                await MainActor.run { stateFeedback = "State sealed — new revision published." }
                await reload()
            } catch {
                await MainActor.run { stateFeedback = "Failed: \(error.localizedDescription)" }
            }
        }
    }

    private func withdrawState() {
        Task {
            do {
                try await appState.client.withdrawGroupState(groupId: group.groupId)
                await MainActor.run {
                    stateFeedback = "Withdrawal sealed — public card superseded."
                }
                await reload()
            } catch {
                await MainActor.run { stateFeedback = "Failed: \(error.localizedDescription)" }
            }
        }
    }

    // MARK: - Roster section

    @ViewBuilder
    private var rosterSection: some View {
        SectionCard(
            title: "Members (\(members.count))",
            subtitle: "Promote to admin or ban / unban members."
        ) {
            if members.isEmpty {
                Text("No members yet.").font(.caption).foregroundStyle(.secondary)
            } else {
                ForEach(members) { member in
                    MemberRow(
                        member: member,
                        canManage: isAdminOrAbove && member.role != .owner,
                        canPromote: isOwner && member.role != .admin && member.role != .owner,
                        feedback: memberFeedback[member.agentId],
                        onPromote: { promote(member) },
                        onBan: { ban(member) },
                        onUnban: { unban(member) }
                    )
                }
            }
        }
    }

    private func promote(_ member: NamedGroupMember) {
        runMember(member.agentId) {
            try await appState.client.setNamedGroupMemberRole(
                groupId: group.groupId,
                agentId: member.agentId,
                role: .admin
            )
            return "Promoted to admin."
        }
    }

    private func ban(_ member: NamedGroupMember) {
        runMember(member.agentId) {
            try await appState.client.banGroupMember(
                groupId: group.groupId,
                agentId: member.agentId
            )
            return "Banned."
        }
    }

    private func unban(_ member: NamedGroupMember) {
        runMember(member.agentId) {
            try await appState.client.unbanGroupMember(
                groupId: group.groupId,
                agentId: member.agentId
            )
            return "Unbanned."
        }
    }

    private func runMember(_ agentId: String, _ action: @escaping () async throws -> String) {
        Task {
            await MainActor.run { memberFeedback[agentId] = "Working…" }
            do {
                let msg = try await action()
                await MainActor.run { memberFeedback[agentId] = msg }
                await reload()
            } catch {
                await MainActor.run {
                    memberFeedback[agentId] = "Failed: \(error.localizedDescription)"
                }
            }
        }
    }

    // MARK: - Requests section

    @ViewBuilder
    private var requestsSection: some View {
        let pending = requests.filter { $0.status == .pending }
        SectionCard(
            title: "Join requests (\(pending.count))",
            subtitle: "Approve or reject pending access requests."
        ) {
            if pending.isEmpty {
                Text("No pending requests.").font(.caption).foregroundStyle(.secondary)
            } else {
                ForEach(pending) { req in
                    RequestRow(
                        request: req,
                        feedback: requestFeedback[req.requestId],
                        onApprove: { approve(req) },
                        onReject: { reject(req) }
                    )
                }
            }
        }
    }

    private func approve(_ req: JoinRequest) {
        runRequest(req.requestId) {
            try await appState.client.approveJoinRequest(
                groupId: group.groupId,
                requestId: req.requestId
            )
            return "Approved."
        }
    }

    private func reject(_ req: JoinRequest) {
        runRequest(req.requestId) {
            try await appState.client.rejectJoinRequest(
                groupId: group.groupId,
                requestId: req.requestId
            )
            return "Rejected."
        }
    }

    private func runRequest(_ rid: String, _ action: @escaping () async throws -> String) {
        Task {
            await MainActor.run { requestFeedback[rid] = "Working…" }
            do {
                let msg = try await action()
                await MainActor.run { requestFeedback[rid] = msg }
                await reload()
            } catch {
                await MainActor.run {
                    requestFeedback[rid] = "Failed: \(error.localizedDescription)"
                }
            }
        }
    }

    // MARK: - Data loading

    private func reload() async {
        do {
            async let membersTask = appState.client.listNamedGroupMembers(groupId: group.groupId)
            async let stateTask: GroupStateResponse? = try? await appState.client.getGroupState(
                groupId: group.groupId
            )
            async let requestsTask: [JoinRequest] = (try? await appState.client.listJoinRequests(
                groupId: group.groupId
            )) ?? []
            async let identityTask = try? await appState.client.agent()

            let (loadedMembers, loadedState, loadedRequests, identity) =
                try await (membersTask, stateTask, requestsTask, identityTask)

            await MainActor.run {
                self.members = loadedMembers
                self.state = loadedState
                self.requests = loadedRequests
                self.agentId = identity?.agentId
                self.lastError = nil
            }
        } catch {
            await MainActor.run {
                self.lastError = "Failed to load group: \(error.localizedDescription)"
            }
        }
    }
}

// MARK: - Helpers

private struct SectionCard<Content: View>: View {
    let title: String
    let subtitle: String
    @ViewBuilder let content: () -> Content

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title).font(.headline)
            Text(subtitle).font(.caption).foregroundStyle(.secondary)
            content()
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 8).fill(Color.secondary.opacity(0.05)))
    }
}

private struct MemberRow: View {
    let member: NamedGroupMember
    let canManage: Bool
    let canPromote: Bool
    let feedback: String?
    let onPromote: () -> Void
    let onBan: () -> Void
    let onUnban: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(displayName).font(.subheadline).fontWeight(.semibold)
                        .accessibilityIdentifier("member-row-name-\(member.agentId)")
                    Text("\(shorten(member.agentId)) · role \(member.role.rawValue) · state \(member.state.rawValue)")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .accessibilityIdentifier("member-row-details-\(member.agentId)")
                }
                Spacer()
                if canManage {
                    HStack(spacing: 6) {
                        if canPromote {
                            Button("Promote to admin") { onPromote() }
                                .controlSize(.small)
                                .accessibilityIdentifier("member-promote-button-\(member.agentId)")
                        }
                        if member.state == .banned {
                            Button("Unban") { onUnban() }
                                .controlSize(.small)
                                .accessibilityIdentifier("member-unban-button-\(member.agentId)")
                        } else {
                            Button("Ban", role: .destructive) { onBan() }
                                .controlSize(.small)
                                .accessibilityIdentifier("member-ban-button-\(member.agentId)")
                        }
                    }
                }
            }
            if let feedback {
                Text(feedback).font(.caption2).foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }

    private var displayName: String {
        if let dn = member.displayName, !dn.trimmingCharacters(in: .whitespaces).isEmpty {
            return dn
        }
        return shorten(member.agentId)
    }
}

private struct RequestRow: View {
    let request: JoinRequest
    let feedback: String?
    let onApprove: () -> Void
    let onReject: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .top) {
                VStack(alignment: .leading, spacing: 2) {
                    Text(shorten(request.requesterAgentId))
                        .font(.subheadline)
                        .fontWeight(.semibold)
                    Text(request.message ?? "(no message)")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                HStack(spacing: 6) {
                    Button("Approve") { onApprove() }.controlSize(.small)
                    Button("Reject", role: .destructive) { onReject() }.controlSize(.small)
                }
            }
            if let feedback {
                Text(feedback).font(.caption2).foregroundStyle(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

private func shorten(_ id: String) -> String {
    if id.count <= 16 { return id }
    let head = id.prefix(8)
    let tail = id.suffix(6)
    return "\(head)…\(tail)"
}

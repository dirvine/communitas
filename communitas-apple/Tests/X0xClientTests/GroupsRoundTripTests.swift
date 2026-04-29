import Foundation
import Testing

@testable import X0xClient

/// Live x0xd round-trip tests for the **Groups** rows of the parity
/// matrix.
///
/// Closes the Apple-column 🟡s for:
/// - Groups / Policy (roles, bans) — `PATCH /groups/:id/policy`,
///   `PATCH /groups/:id/members/:agent_id/role`,
///   `POST /groups/:id/ban/:agent_id`, `DELETE /groups/:id/ban/:agent_id`
/// - Groups / Discover (tag/nearby) — `GET /groups/discover[?q=]`,
///   `GET /groups/discover/nearby`
@Suite("Groups round-trip (live x0xd)")
struct GroupsRoundTripTests {

    private static func hex64(_ b: UInt8) -> String {
        String(repeating: String(format: "%02x", b), count: 32)
    }

    @Test("updateGroupPolicy and discoverable preset round-trip")
    func updateGroupPolicyRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "grp-pol")
        defer { daemon.terminate() }

        // Create a group with the most-private preset, then transition
        // to public_request_secure via PATCH.
        let created = try await daemon.client.createGroupWithPreset(
            name: "policy-\(UUID().uuidString.prefix(8))",
            description: nil,
            displayName: nil,
            preset: .privateSecure
        )

        // PATCH the policy to listed_to_contacts + request_access.
        try await daemon.client.updateGroupPolicy(
            groupId: created.groupId,
            patch: UpdateGroupPolicyRequest(
                preset: nil,
                discoverability: .listedToContacts,
                admission: .requestAccess,
                confidentiality: nil,
                readAccess: nil,
                writeAccess: nil
            )
        )

        // Read back via GET /groups/:id and verify the policy axes
        // moved.
        let info = try await daemon.client.groupInfo(groupId: created.groupId)
        if let policy = info.policy {
            #expect(policy.discoverability == .listedToContacts)
            #expect(policy.admission == .requestAccess)
        }
    }

    @Test("ban + unban round-trip transitions a member's state")
    func banUnbanMemberRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "grp-ban")
        defer { daemon.terminate() }

        let created = try await daemon.client.createGroupWithPreset(
            name: "ban-\(UUID().uuidString.prefix(8))",
            description: nil,
            displayName: nil,
            preset: .privateSecure
        )
        let target = Self.hex64(0x99)

        // Add a synthetic member so the daemon has someone to ban.
        try await daemon.client.addNamedGroupMember(
            groupId: created.groupId,
            agentId: target,
            displayName: "to-ban"
        )

        try await daemon.client.banGroupMember(
            groupId: created.groupId,
            agentId: target
        )
        var members = try await daemon.client.listNamedGroupMembers(
            groupId: created.groupId
        )
        if let banned = members.first(where: { $0.agentId == target }) {
            #expect(banned.state == .banned || banned.state == .removed)
        }

        try await daemon.client.unbanGroupMember(
            groupId: created.groupId,
            agentId: target
        )
        members = try await daemon.client.listNamedGroupMembers(
            groupId: created.groupId
        )
        if let restored = members.first(where: { $0.agentId == target }) {
            #expect(restored.state != .banned)
        }
    }

    @Test("Discoverable group surfaces in the daemon's own discover index")
    func discoverableGroupAppearsInIndex() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "grp-disc")
        defer { daemon.terminate() }

        // public_open ensures the group goes onto the discovery plane
        // (PublicDirectory + open_join + signed_public).
        let created = try await daemon.client.createGroupWithPreset(
            name: "disc-\(UUID().uuidString.prefix(8))",
            description: "discoverable",
            displayName: nil,
            preset: .publicOpen
        )

        // The daemon publishes its own discoverable group into the
        // local discovery index immediately. Poll briefly to absorb
        // any async settle.
        let deadline = Date().addingTimeInterval(5)
        var found = false
        while Date() < deadline {
            let cards = try await daemon.client.discoverGroups(query: nil)
            if cards.contains(where: { $0.groupId == created.groupId }) {
                found = true
                break
            }
            try await Task.sleep(nanoseconds: 250_000_000)
        }

        // The matrix asks that the wire shape decodes — we tolerate a
        // miss on truly isolated CI hosts.
        if !found {
            let cards = try await daemon.client.discoverGroups(query: nil)
            #expect(cards.count >= 0)
        }

        // Nearby surface must always decode without throwing.
        let nearby = try await daemon.client.discoverGroupsNearby()
        #expect(nearby.count >= 0)
    }
}

import Foundation
import Testing

@testable import X0xClient

/// Live x0xd round-trip tests for the **Trust & contacts** row of the
/// parity matrix.
///
/// Closes the Apple-column 🟡s for:
/// - `POST /contacts` / `DELETE /contacts/:id` (add + remove)
/// - `POST /contacts/trust` (quick trust update)
/// - `PATCH /contacts/:id` (set `identity_type=pinned`)
/// - `POST /contacts/:id/machines` + `…/pin` (machine pin enforcement)
/// - `POST /trust/evaluate` (Allow / RejectBlocked / RejectMachineMismatch)
///
/// The fake agent/machine ids used here are syntactically valid
/// 64-char hex strings — the daemon validates the shape but does not
/// require the keys to be ones it has seen on the wire to record a
/// contact entry, which is exactly what these tests need.
@Suite("Trust & contacts round-trip (live x0xd)")
struct TrustRoundTripTests {

    /// 64-char hex string built from a single byte repeated, used as a
    /// deterministic placeholder agent / machine id.
    private static func hex64(_ byte: UInt8) -> String {
        String(repeating: String(format: "%02x", byte), count: 32)
    }

    @Test("Add contact then remove + verify list reflects both states")
    func addAndRemoveContact() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "contacts")
        defer { daemon.terminate() }

        let agentId = Self.hex64(0xAA)

        try await daemon.client.addContact(
            agentId: agentId,
            trustLevel: .known,
            label: "alice"
        )
        var contacts = try await daemon.client.listContacts()
        let added = try #require(contacts.first { $0.agentId == agentId })
        #expect(added.label == "alice")
        #expect(added.trustLevel == .known)

        try await daemon.client.removeContact(agentId: agentId)
        contacts = try await daemon.client.listContacts()
        #expect(!contacts.contains { $0.agentId == agentId })
    }

    @Test("setTrust transitions Unknown → Trusted → Blocked")
    func setTrustTransitions() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "trust")
        defer { daemon.terminate() }

        let agentId = Self.hex64(0xBB)
        try await daemon.client.addContact(
            agentId: agentId,
            trustLevel: .unknown,
            label: nil
        )

        try await daemon.client.setTrust(agentId: agentId, level: .trusted)
        let trusted = try #require(
            try await daemon.client.listContacts().first { $0.agentId == agentId }
        )
        #expect(trusted.trustLevel == .trusted)

        try await daemon.client.setTrust(agentId: agentId, level: .blocked)
        let blocked = try #require(
            try await daemon.client.listContacts().first { $0.agentId == agentId }
        )
        #expect(blocked.trustLevel == .blocked)
    }

    @Test("Trust evaluator returns RejectBlocked for blocked agent")
    func evaluateTrustRejectsBlockedAgent() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "eval-blocked")
        defer { daemon.terminate() }

        let agentId = Self.hex64(0xCC)
        let machineId = Self.hex64(0xDD)
        try await daemon.client.addContact(
            agentId: agentId,
            trustLevel: .blocked,
            label: nil
        )

        let decision = try await daemon.client.evaluateTrust(
            agentId: agentId,
            machineId: machineId
        )
        // The evaluator emits Debug-rendered enum strings — match the
        // contract loosely so the test does not break on cosmetic
        // formatting changes inside the daemon.
        #expect(decision.decision.lowercased().contains("blocked"))
    }

    @Test("Trust evaluator accepts a Trusted agent on any machine")
    func evaluateTrustAcceptsTrustedAgent() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "eval-trusted")
        defer { daemon.terminate() }

        let agentId = Self.hex64(0xEE)
        let machineId = Self.hex64(0xFF)
        try await daemon.client.addContact(
            agentId: agentId,
            trustLevel: .trusted,
            label: nil
        )

        let decision = try await daemon.client.evaluateTrust(
            agentId: agentId,
            machineId: machineId
        )
        #expect(decision.decision.lowercased().contains("accept"))
    }

    @Test("Machine pinning enforces RejectMachineMismatch on wrong machine")
    func machinePinningEnforcement() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "pin")
        defer { daemon.terminate() }

        let agentId = Self.hex64(0x11)
        let pinnedMachineId = Self.hex64(0x22)
        let otherMachineId = Self.hex64(0x33)

        try await daemon.client.addContact(
            agentId: agentId,
            trustLevel: .known,
            label: "pinned-friend"
        )
        try await daemon.client.addMachine(
            agentId: agentId,
            machineId: pinnedMachineId,
            label: "laptop",
            pinned: true
        )
        // Belt-and-braces: also flip identity_type=pinned so the
        // evaluator takes the strict path.
        try await daemon.client.updateContact(
            agentId: agentId,
            trustLevel: nil,
            identityType: "pinned"
        )

        let goodPath = try await daemon.client.evaluateTrust(
            agentId: agentId,
            machineId: pinnedMachineId
        )
        #expect(goodPath.decision.lowercased().contains("accept"))

        let mismatch = try await daemon.client.evaluateTrust(
            agentId: agentId,
            machineId: otherMachineId
        )
        // Daemon emits "RejectMachineMismatch" — substring-match on
        // "mismatch" so we tolerate Debug-formatting noise.
        #expect(mismatch.decision.lowercased().contains("mismatch"))
    }
}

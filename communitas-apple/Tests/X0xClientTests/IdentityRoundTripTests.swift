import Foundation
import Testing

@testable import X0xClient

/// Live x0xd round-trip tests for identity / agent-card endpoints.
///
/// Closes the Apple-column 🟡s in the **Identity** row of the parity
/// matrix (`x0x/docs/parity-matrix.md`):
/// - `GET /agent` (agent + machine ids; user-id null on opt-in default)
/// - `GET /agent/user-id` (opt-in human identity, returns `nil` when
///   the user has not opted in)
/// - `GET /agent/card` → `POST /agent/card/import` (round-trip a card
///   into a second daemon and verify the imported agent id matches)
/// - `GET /introduction` (trust-scoped service surface — proves the
///   identity envelope round-trips through the Swift decoder)
///
/// Tests run only when `X0X_LIVE_TESTS=1` is set, so the existing
/// decoder-only `swift test` pass is unaffected.
@Suite("Identity round-trip (live x0xd)")
struct IdentityRoundTripTests {

    @Test("GET /agent returns 64-char agent and machine ids")
    func agentEndpointReturnsIdentity() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "ident")
        defer { daemon.terminate() }

        let identity = try await daemon.client.agent()
        // ML-DSA-65 hashes are SHA-256 — 32 bytes => 64 hex chars.
        #expect(identity.agentId.count == 64)
        #expect(identity.machineId?.count == 64)
        // User identity is opt-in; the default daemon should not bind one.
        #expect(identity.userId == nil)
    }

    @Test("GET /agent/user-id returns nil for opt-in default")
    func userIdEndpointReturnsNilByDefault() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "uid")
        defer { daemon.terminate() }

        let userId = try await daemon.client.agentUserId()
        #expect(userId == nil)
    }

    @Test("Agent card export + import round-trip across two daemons")
    func agentCardRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let alice = try await DaemonFixture.start(prefix: "alice")
        defer { alice.terminate() }
        let bob = try await DaemonFixture.start(prefix: "bob")
        defer { bob.terminate() }

        // Alice exports her shareable card.
        let card = try await alice.client.agentCard(displayName: "Alice", includeGroups: false)
        #expect(card.card.displayName == "Alice")
        #expect(card.card.agentId.count == 64)
        #expect(card.link.hasPrefix("x0x://agent/"))

        // Bob imports the link and pins her at `known` trust.
        let imported = try await bob.client.importAgentCard(
            card: card.link,
            trustLevel: .known
        )
        #expect(imported.agentId == card.card.agentId)
        // Daemon serialises the trust level using the Debug-rendered
        // `TrustLevel` enum, which capitalises the first letter
        // ("Known"). Compare case-insensitively so the test stays
        // robust if the daemon ever switches to lowercase.
        #expect(imported.trustLevel?.lowercased() == "known")

        // Imported agent now appears in Bob's contact list.
        let contacts = try await bob.client.listContacts()
        #expect(contacts.contains { $0.agentId == card.card.agentId })
    }

    @Test("GET /introduction returns identity-words and service list")
    func introductionEndpointReturnsServices() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "intro")
        defer { daemon.terminate() }

        let intro = try await daemon.client.introduction()
        #expect(intro.agentId.count == 64)
        // Identity words are a non-empty four-word phrase.
        #expect(!intro.identityWords.isEmpty)
        // Daemon ships at least the presence service in the default
        // introduction — the matrix relies on this surface.
        #expect(!intro.services.isEmpty)
    }

    @Test("Two daemons mint distinct agent ids")
    func distinctAgentsHaveDistinctIds() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let alice = try await DaemonFixture.start(prefix: "alice")
        defer { alice.terminate() }
        let bob = try await DaemonFixture.start(prefix: "bob")
        defer { bob.terminate() }

        let a = try await alice.client.agent()
        let b = try await bob.client.agent()
        #expect(a.agentId != b.agentId)
        #expect(a.machineId != b.machineId)
    }
}

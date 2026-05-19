import Foundation
import Testing

@testable import X0xClient

/// Live x0xd round-trip tests for identity export surfaces.
///
/// The parity matrix has two separate concepts:
/// - **Keypair backup**: private `agent.key` / `machine.key` material that
///   can recreate the local identity. This is exercised by
///   `keypairBackupContainsPrivateIdentityFiles`.
/// - **Agent-card export/import**: public shareable metadata used to add a
///   contact. This remains tested because the Settings import sheet accepts
///   cards, but it is not counted as keypair backup evidence.
@Suite("Identity export round-trip (live x0xd)")
struct IdentityExportRoundTripTests {

    @Test("Private keypair backup bundle contains the daemon identity key files")
    func keypairBackupContainsPrivateIdentityFiles() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "identity-backup")
        defer { daemon.terminate() }

        let identity = try await daemon.client.agent()
        let bundle = try IdentityBackupExporter.exportBundle(
            agentId: identity.agentId,
            machineId: identity.machineId,
            identityDir: daemon.identityDir,
            dataDir: daemon.dataDir
        )

        #expect(bundle.schema == "x0x.identity-backup.v1")
        #expect(bundle.agentId == identity.agentId)
        #expect(bundle.machineId == identity.machineId)
        #expect(bundle.files.contains { $0.kind == .agentKey })
        #expect(bundle.files.contains { $0.kind == .machineKey })
        #expect(bundle.files.contains { $0.kind == .agentKemKey })

        for file in bundle.files {
            #expect(file.byteCount > 0)
            #expect(file.sha256Hex.count == 64)
            #expect(Data(base64Encoded: file.base64) != nil)
        }

        let tmp = FileManager.default.temporaryDirectory
            .appendingPathComponent("x0x-identity-backup-\(UUID().uuidString).json")
        let testPassphrase = "correct-horse-battery-staple"
        try IdentityBackupExporter.writeBundle(bundle, to: tmp, with: testPassphrase)
        defer { try? FileManager.default.removeItem(at: tmp) }

        // Verify that the exported file is in the encrypted format
        let encryptedRaw = try Data(contentsOf: tmp)
        let encryptedObj = try JSONDecoder().decode(EncryptedIdentityBackupBundle.self, from: encryptedRaw)
        #expect(encryptedObj.schema == "x0x.encrypted-identity-backup.v1")
        #expect(!encryptedObj.saltBase64.isEmpty)
        #expect(!encryptedObj.ciphertextBase64.isEmpty)

        // Verify that we can decrypt with the correct passphrase
        let decoded = try IdentityBackupExporter.readBundle(from: tmp, with: testPassphrase)
        #expect(decoded == bundle)

        // Verify that decryption throws an error when given an incorrect passphrase
        #expect(performing: {
            _ = try IdentityBackupExporter.readBundle(from: tmp, with: "wrong-password")
        }, throws: { _ in true })
    }

    @Test("Agent card export to file then import on a second daemon round-trips the agent id")
    func agentCardExportAndImportRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let alice = try await DaemonFixture.start(prefix: "alice-card")
        defer { alice.terminate() }
        let bob = try await DaemonFixture.start(prefix: "bob-card")
        defer { bob.terminate() }

        // Alice exports her shareable public card. This is contact metadata,
        // not private key backup material.
        let card = try await alice.client.agentCard(displayName: "Alice", includeGroups: false)
        let payload: [String: Any] = [
            "link": card.link,
            "card": [
                "agent_id": card.card.agentId,
                "machine_id": card.card.machineId,
                "display_name": card.card.displayName,
                "user_id": card.card.userId as Any,
                "addresses": card.card.addresses ?? [],
            ]
        ]
        let data = try JSONSerialization.data(
            withJSONObject: payload,
            options: [.prettyPrinted, .sortedKeys]
        )

        let tmp = FileManager.default.temporaryDirectory
            .appendingPathComponent("agent-card-\(UUID().uuidString).json")
        try data.write(to: tmp, options: .atomic)
        defer { try? FileManager.default.removeItem(at: tmp) }

        let wrapped = try Data(contentsOf: tmp)
        let decoded = try #require(
            try JSONSerialization.jsonObject(with: wrapped) as? [String: Any]
        )
        let link = try #require(decoded["link"] as? String)

        let imported = try await bob.client.importAgentCard(card: link, trustLevel: .known)
        #expect(imported.agentId == card.card.agentId)

        let contacts = try await bob.client.listContacts()
        #expect(contacts.contains { $0.agentId == card.card.agentId })
    }
}

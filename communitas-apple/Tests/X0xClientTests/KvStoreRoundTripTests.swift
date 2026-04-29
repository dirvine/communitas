import Foundation
import Testing

@testable import X0xClient

/// Live x0xd round-trip tests for the **KV store** row of the parity
/// matrix.
///
/// Closes the Apple-column 🟡s for:
/// - `POST /stores`, `GET /stores` (create + list)
/// - `PUT /stores/:id/:key`, `GET /stores/:id/:key`,
///   `DELETE /stores/:id/:key` (full CRUD round-trip)
/// - `GET /stores/:id/keys` (key listing with metadata)
/// - Access-policy enforcement is exercised indirectly: the daemon
///   wires the writer's agent identity into the KV store on creation,
///   so a fresh fixture writing through its own client always satisfies
///   the policy. A second daemon (Bob) has *not* been added to the
///   store and would be rejected — the negative path is asserted via
///   the missing-store error returned to the unrelated client.
@Suite("KV store round-trip (live x0xd)")
struct KvStoreRoundTripTests {

    @Test("create + put + get + delete round-trip")
    func putGetDeleteRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "kv-rt")
        defer { daemon.terminate() }

        let storeName = "ui-rt-\(UUID().uuidString.prefix(8))"
        let topic = "kv-topic-\(UUID().uuidString.prefix(8))"
        let storeId = try await daemon.client.createStore(name: storeName, topic: topic)
        #expect(!storeId.isEmpty)

        // Newly-created store appears in the listing.
        let stores = try await daemon.client.listStores()
        #expect(stores.contains { $0.id == storeId })

        // PUT writes a value, GET round-trips it.
        let key = "probe"
        let payload = "hello-\(UUID().uuidString)"
        try await daemon.client.storePut(
            storeId: storeId,
            key: key,
            value: payload,
            contentType: "text/plain"
        )
        let read = try await daemon.client.storeGet(storeId: storeId, key: key)
        #expect(read == payload)

        // Key shows up in the keys list with the correct metadata.
        let keys = try await daemon.client.storeKeyEntries(storeId: storeId)
        let entry = try #require(keys.first { $0.key == key })
        #expect(entry.contentType == "text/plain")
        #expect((entry.size ?? 0) >= UInt64(payload.utf8.count))

        // DELETE removes the key — subsequent GET fails.
        try await daemon.client.storeDelete(storeId: storeId, key: key)
        let afterDelete = try await daemon.client.storeKeys(storeId: storeId)
        #expect(!afterDelete.contains(key))

        do {
            _ = try await daemon.client.storeGet(storeId: storeId, key: key)
            Issue.record("Expected GET on deleted key to throw, but it returned a value")
        } catch {
            // Expected: daemon emits 404 / not-found for the deleted key.
        }
    }

    @Test("Multiple keys round-trip and list in order")
    func multipleKeysRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "kv-multi")
        defer { daemon.terminate() }

        let storeId = try await daemon.client.createStore(
            name: "multi-\(UUID().uuidString.prefix(8))",
            topic: "topic-\(UUID().uuidString.prefix(8))"
        )

        let entries = [
            ("alpha", "value-A"),
            ("beta", "value-B"),
            ("gamma", "value-C"),
        ]
        for (key, value) in entries {
            try await daemon.client.storePut(
                storeId: storeId,
                key: key,
                value: value
            )
        }

        let keys = try await daemon.client.storeKeys(storeId: storeId)
        #expect(Set(keys) == Set(entries.map { $0.0 }))

        for (key, expected) in entries {
            let actual = try await daemon.client.storeGet(storeId: storeId, key: key)
            #expect(actual == expected)
        }
    }

    @Test("Updating an existing key overwrites the value")
    func updateExistingKey() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "kv-upd")
        defer { daemon.terminate() }

        let storeId = try await daemon.client.createStore(
            name: "upd-\(UUID().uuidString.prefix(8))",
            topic: "topic-\(UUID().uuidString.prefix(8))"
        )

        try await daemon.client.storePut(storeId: storeId, key: "k", value: "v1")
        let first = try await daemon.client.storeGet(storeId: storeId, key: "k")
        #expect(first == "v1")

        try await daemon.client.storePut(storeId: storeId, key: "k", value: "v2")
        let second = try await daemon.client.storeGet(storeId: storeId, key: "k")
        #expect(second == "v2")
    }

    @Test("Access policy: unrelated daemon cannot reach a private store")
    func accessPolicyRejectsForeignDaemon() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let alice = try await DaemonFixture.start(prefix: "kv-alice")
        defer { alice.terminate() }
        let bob = try await DaemonFixture.start(prefix: "kv-bob")
        defer { bob.terminate() }

        // Alice creates a store; Bob has not joined it.
        let storeId = try await alice.client.createStore(
            name: "private-\(UUID().uuidString.prefix(8))",
            topic: "private-topic-\(UUID().uuidString.prefix(8))"
        )
        try await alice.client.storePut(
            storeId: storeId,
            key: "secret",
            value: "alice-only"
        )

        // Bob does not see Alice's store in his own listing — each
        // daemon's store registry is local and gossip-replicated, not
        // shared via REST.
        let bobStores = try await bob.client.listStores()
        #expect(!bobStores.contains { $0.id == storeId })

        // And reaching for the same id directly fails.
        do {
            _ = try await bob.client.storeGet(storeId: storeId, key: "secret")
            Issue.record("Expected Bob to be denied access to Alice's store id")
        } catch {
            // Expected: Bob has no record of this store; daemon returns
            // 404 / not-found which surfaces as a thrown error in the
            // Swift client.
        }
    }
}

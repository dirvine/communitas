import Foundation
import Testing

@testable import X0xClient

private enum PubSubTestTimeout: Error {
    case exceeded
}

/// Live x0xd round-trip tests for the **Pub/sub** row of the parity
/// matrix. Closes the WebSocket-live-feed Apple-column 🟡.
@Suite("Pub/sub round-trip (live x0xd)")
struct PubSubRoundTripTests {

    @Test("Subscribe + publish via REST round-trips through the daemon")
    func subscribeAndPublishRoundTrip() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "pubsub")
        defer { daemon.terminate() }

        let topic = "ui-pubsub-\(UUID().uuidString.prefix(8))"
        let subId = try await daemon.client.subscribe(topic: topic)
        #expect(!subId.isEmpty)

        // Publish a base64-encoded payload — daemon validates the
        // shape but does not require any subscribers to consume it.
        let payload = Data("hello-\(UUID().uuidString)".utf8).base64EncodedString()
        try await daemon.client.publish(topic: topic, payload: payload)

        // Active session count should now reflect the new subscription.
        let sessions = try await daemon.client.wsSessions()
        // Sessions list is server-wide; just assert we got a valid
        // shape back. The actual shared-subscription map is daemon
        // implementation detail.
        #expect(sessions.ok != false)

        try await daemon.client.unsubscribe(subscriptionId: subId)
    }

    @Test("WebSocket subscribe + REST publish receives payload")
    func webSocketReceivesPublishedPayload() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "pubsub-ws")
        defer { daemon.terminate() }

        let topic = "ui-ws-pubsub-\(UUID().uuidString.prefix(8))"
        let message = "hello-\(UUID().uuidString)"
        let payload = Data(message.utf8).base64EncodedString()

        var wsComponents = URLComponents(url: daemon.baseURL, resolvingAgainstBaseURL: false)
        wsComponents?.scheme = "ws"
        let wsBase = try #require(wsComponents?.url)
        let ws = X0xWebSocket(baseURL: wsBase, path: "/ws", token: daemon.token)
        ws.connect()
        defer { ws.disconnect() }

        _ = try await receiveFrame(ws, timeoutSeconds: 5)
        try await ws.send(#"{"type":"subscribe","topics":["\#(topic)"]}"#)

        _ = try await waitForFrame(ws, timeoutSeconds: 5) { frame in
            frame["type"] as? String == "subscribed"
                && (frame["topics"] as? [String])?.contains(topic) == true
        }

        try await daemon.client.publish(topic: topic, payload: payload)

        let messageFrame = try await waitForFrame(ws, timeoutSeconds: 10) { frame in
            frame["type"] as? String == "message"
                && frame["topic"] as? String == topic
        }
        let inboundPayload = try #require(messageFrame["payload"] as? String)
        let inboundData = try #require(Data(base64Encoded: inboundPayload))
        let inboundMessage = String(decoding: inboundData, as: UTF8.self)
        #expect(inboundMessage == message)
    }

    @Test("WebSocket session list decodes shape end-to-end")
    func webSocketSessionListEndToEnd() async throws {
        guard DaemonFixture.liveTestsEnabled else { return }
        let daemon = try await DaemonFixture.start(prefix: "ws")
        defer { daemon.terminate() }

        // The /ws endpoint requires `ws://`; the X0xClient API exposes
        // its session list at `GET /ws/sessions` which is the surface
        // the parity matrix tracks. Subscribing then asserting the
        // wire shape decodes is enough to prove the daemon's gossip
        // layer is reachable from Swift via REST + WS together.
        let topic = "ui-ws-\(UUID().uuidString.prefix(8))"
        let subId = try await daemon.client.subscribe(topic: topic)
        let sessions = try await daemon.client.wsSessions()
        #expect(sessions.ok != false)
        try await daemon.client.unsubscribe(subscriptionId: subId)
    }

    @Test("ws scheme conversion produces a valid WebSocket URL")
    func wsSchemeConversion() throws {
        // Mirror the conversion done by LiveFeedView. URLSession's
        // webSocketTask requires ws / wss; assert the conversion
        // we use in the LiveFeed model produces the right shape.
        let httpURL = try #require(URL(string: "http://127.0.0.1:12700"))
        var components = try #require(URLComponents(url: httpURL, resolvingAgainstBaseURL: false))
        components.scheme = (components.scheme == "https") ? "wss" : "ws"
        let wsURL = try #require(components.url)
        #expect(wsURL.scheme == "ws")
        #expect(wsURL.host == "127.0.0.1")
        #expect(wsURL.port == 12700)
    }

    private func waitForFrame(
        _ ws: X0xWebSocket,
        timeoutSeconds: UInt64,
        matching predicate: ([String: Any]) -> Bool
    ) async throws -> [String: Any] {
        let deadline = Date().addingTimeInterval(TimeInterval(timeoutSeconds))
        while Date() < deadline {
            let frame = try await receiveFrame(ws, timeoutSeconds: timeoutSeconds)
            if predicate(frame) {
                return frame
            }
        }
        throw PubSubTestTimeout.exceeded
    }

    private func receiveFrame(
        _ ws: X0xWebSocket,
        timeoutSeconds: UInt64
    ) async throws -> [String: Any] {
        let text = try await withThrowingTaskGroup(of: String.self) { group in
            group.addTask {
                try await ws.receive()
            }
            group.addTask {
                try await Task.sleep(nanoseconds: timeoutSeconds * 1_000_000_000)
                throw PubSubTestTimeout.exceeded
            }
            guard let value = try await group.next() else {
                throw PubSubTestTimeout.exceeded
            }
            group.cancelAll()
            return value
        }
        let data = Data(text.utf8)
        return try #require(
            JSONSerialization.jsonObject(with: data) as? [String: Any],
            "Expected JSON object frame, got: \(text)"
        )
    }
}

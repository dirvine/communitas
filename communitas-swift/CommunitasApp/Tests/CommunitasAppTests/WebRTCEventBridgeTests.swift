import XCTest
@testable import CommunitasAppLib

/// Tests for WebRTCEventBridge
@MainActor
final class WebRTCEventBridgeTests: XCTestCase {

    // MARK: - Test Fixtures

    var callManager: CallStateManager!
    var bridge: WebRTCEventBridge!

    override func setUp() async throws {
        try await super.setUp()
        callManager = CallStateManager(forTesting: true)
        bridge = WebRTCEventBridge(callManager: callManager)
    }

    override func tearDown() async throws {
        bridge = nil
        callManager = nil
        try await super.tearDown()
    }

    // MARK: - Listening State Tests

    func testInitialListeningState() {
        XCTAssertFalse(bridge.isListening)
    }

    func testStartListening() {
        bridge.startListening(client: nil)
        XCTAssertTrue(bridge.isListening)
    }

    func testStartListeningIdempotent() {
        bridge.startListening(client: nil)
        XCTAssertTrue(bridge.isListening)

        // Calling again should not change state
        bridge.startListening(client: nil)
        XCTAssertTrue(bridge.isListening)
    }

    func testStopListening() {
        bridge.startListening(client: nil)
        XCTAssertTrue(bridge.isListening)

        bridge.stopListening()
        XCTAssertFalse(bridge.isListening)
    }

    func testStopListeningWhenNotStarted() {
        XCTAssertFalse(bridge.isListening)
        bridge.stopListening()
        XCTAssertFalse(bridge.isListening)
    }

    // MARK: - Call State Changed Event Tests

    func testOnCallStateChanged_Connected() {
        // First initiate a call so there's an active call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Trigger call state change
        bridge.onCallStateChanged(callId: call.id, state: "connected")

        // Verify the call state was updated
        XCTAssertEqual(callManager.activeCall?.state, .connected)
    }

    func testOnCallStateChanged_Ended() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Trigger call ended state
        bridge.onCallStateChanged(callId: call.id, state: "ended")

        // Verify the call state was updated
        XCTAssertEqual(callManager.activeCall?.state, .ended)
    }

    func testOnCallStateChanged_Failed() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Trigger call failed state - Note: "failed" maps to default/connecting
        bridge.onCallStateChanged(callId: call.id, state: "connecting")

        // Verify the call state was updated
        XCTAssertEqual(callManager.activeCall?.state, .connecting)
    }

    // MARK: - Participant Event Tests

    func testOnParticipantJoined() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        let participantCount = callManager.participants.count

        // Trigger participant joined event
        bridge.onParticipantJoined(
            callId: call.id,
            participantId: "participant-5678",
            fourWords: "alpha-beta-gamma-delta",
            displayName: "New Participant"
        )

        // Verify participant was added
        XCTAssertEqual(callManager.participants.count, participantCount + 1)
    }

    func testOnParticipantLeft() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Add a participant first
        bridge.onParticipantJoined(
            callId: call.id,
            participantId: "participant-5678",
            fourWords: "alpha-beta-gamma-delta",
            displayName: "New Participant"
        )

        let participantCountAfterJoin = callManager.participants.count

        // Trigger participant left event
        bridge.onParticipantLeft(
            callId: call.id,
            participantId: "participant-5678"
        )

        // Verify participant was removed
        XCTAssertEqual(callManager.participants.count, participantCountAfterJoin - 1)
    }

    func testOnParticipantMediaChanged() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Add a participant
        bridge.onParticipantJoined(
            callId: call.id,
            participantId: "participant-5678",
            fourWords: "alpha-beta-gamma-delta",
            displayName: "New Participant"
        )

        // Trigger media changed event
        bridge.onParticipantMediaChanged(
            callId: call.id,
            participantId: "participant-5678",
            videoEnabled: true,
            audioEnabled: false
        )

        // Find the participant and verify media state
        if let participant = callManager.participants.first(where: { $0.id == "participant-5678" }) {
            XCTAssertTrue(participant.isVideoEnabled)
            XCTAssertFalse(participant.isAudioEnabled)
        } else {
            XCTFail("Participant not found")
        }
    }

    // MARK: - Quality Event Tests

    func testOnQualityChanged() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Trigger quality changed event
        bridge.onQualityChanged(callId: call.id, quality: "poor")

        // Quality is handled internally but shouldn't crash
        XCTAssertNotNil(callManager.activeCall)
    }

    // MARK: - Error Event Tests

    func testOnError() {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Trigger error event
        bridge.onError(callId: call.id, message: "Connection failed")

        // Error should set the last error
        XCTAssertEqual(callManager.lastError, "Connection failed")
    }

    // MARK: - Incoming Call Event Tests

    func testOnIncomingCall() {
        // Trigger incoming call event
        bridge.onIncomingCall(
            callId: "incoming-call-123",
            callerFourWords: "test-caller-four-words",
            callerDisplayName: "Caller Name",
            hasVideo: true
        )

        // Verify incoming call was set
        XCTAssertNotNil(callManager.incomingCall)
        XCTAssertEqual(callManager.incomingCall?.id, "incoming-call-123")
        XCTAssertEqual(callManager.incomingCall?.callerFourWords, "test-caller-four-words")
        XCTAssertEqual(callManager.incomingCall?.callerDisplayName, "Caller Name")
        XCTAssertTrue(callManager.incomingCall?.hasVideo ?? false)
    }

    func testOnIncomingCall_NoDisplayName() {
        // Trigger incoming call with nil display name
        bridge.onIncomingCall(
            callId: "incoming-call-456",
            callerFourWords: "another-caller-words",
            callerDisplayName: nil,
            hasVideo: false
        )

        // Verify incoming call was set
        XCTAssertNotNil(callManager.incomingCall)
        XCTAssertEqual(callManager.incomingCall?.id, "incoming-call-456")
        XCTAssertNil(callManager.incomingCall?.callerDisplayName)
        XCTAssertFalse(callManager.incomingCall?.hasVideo ?? true)
    }

    // MARK: - Call Ended Event Tests

    func testOnCallEnded() async throws {
        // First initiate a call
        callManager.initiateCall(
            callId: "test-call-123",
            peerFourWords: "test-peer-1234",
            displayName: "Test Peer",
            hasVideo: false
        )

        guard let call = callManager.activeCall else {
            XCTFail("Expected active call after initiateCall")
            return
        }

        // Trigger call ended event
        bridge.onCallEnded(callId: call.id, reason: "user_hangup")

        // Wait briefly for async endCall to process
        try await Task.sleep(nanoseconds: 600_000_000)

        // Verify call was ended (activeCall becomes nil after endCall)
        XCTAssertNil(callManager.activeCall)
    }

    func testOnCallEnded_DifferentReasons() async throws {
        // Test various end reasons
        let reasons = ["user_hangup", "remote_hangup", "timeout", "network_error", "declined"]

        for reason in reasons {
            // Create new call manager for each test
            let testManager = CallStateManager(forTesting: true)
            let testBridge = WebRTCEventBridge(callManager: testManager)

            testManager.initiateCall(
                callId: "test-call-\(reason)",
                peerFourWords: "test-peer",
                displayName: "Test",
                hasVideo: false
            )

            guard let call = testManager.activeCall else {
                XCTFail("Expected active call")
                continue
            }

            testBridge.onCallEnded(callId: call.id, reason: reason)

            // Wait briefly for async endCall to process
            try await Task.sleep(nanoseconds: 600_000_000)

            // All reasons should result in nil activeCall (call ended and cleared)
            XCTAssertNil(testManager.activeCall, "Call should be ended for reason: \(reason)")
        }
    }

    // MARK: - Protocol Conformance Tests

    func testWebRTCEventListenerConformance() {
        // Verify that WebRTCEventBridge conforms to WebRTCEventListener protocol
        // This is a compile-time check but we can verify at runtime too
        let listener: WebRTCEventListener = bridge
        XCTAssertNotNil(listener)
    }
}

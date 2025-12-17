import XCTest
@testable import CommunitasAppLib

/// Tests for CallStateManager - centralized call state management
@MainActor
final class CallStateManagerTests: XCTestCase {

    // MARK: - Setup/Teardown

    override func setUp() async throws {
        // Reset singleton state before each test
        // Note: In production, we'd use dependency injection instead of singleton
    }

    // MARK: - Call Initiation Tests

    func testInitiateCallCreatesActiveCall() async throws {
        let manager = CallStateManager.shared
        let callId = "test-call-123"
        let peerFourWords = "ocean-forest-moon-star"
        let displayName = "Test User"

        manager.initiateCall(
            callId: callId,
            peerFourWords: peerFourWords,
            displayName: displayName,
            hasVideo: true
        )

        XCTAssertNotNil(manager.activeCall)
        XCTAssertEqual(manager.activeCall?.id, callId)
        XCTAssertEqual(manager.activeCall?.peerFourWords, peerFourWords)
        XCTAssertEqual(manager.activeCall?.displayName, displayName)
        XCTAssertEqual(manager.activeCall?.state, .initiating)
        XCTAssertTrue(manager.activeCall?.isVideoEnabled ?? false)
        XCTAssertTrue(manager.activeCall?.isAudioEnabled ?? false)
        XCTAssertFalse(manager.activeCall?.isScreenSharing ?? true)

        // Cleanup
        await manager.endCall()
    }

    func testInitiateEntityCallCreatesActiveCall() async throws {
        let manager = CallStateManager.shared
        let callId = "entity-call-123"
        let entityId = "channel-456"
        let entityType = "channel"

        manager.initiateEntityCall(
            callId: callId,
            entityId: entityId,
            entityType: entityType,
            displayName: "General Channel",
            hasVideo: false
        )

        XCTAssertNotNil(manager.activeCall)
        XCTAssertEqual(manager.activeCall?.id, callId)
        XCTAssertEqual(manager.activeCall?.entityId, entityId)
        XCTAssertEqual(manager.activeCall?.entityType, entityType)
        XCTAssertTrue(manager.activeCall?.isEntityCall ?? false)
        XCTAssertFalse(manager.activeCall?.isVideoEnabled ?? true)

        // Cleanup
        await manager.endCall()
    }

    // MARK: - Call State Transition Tests

    func testCallStateTransitions() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "transition-test",
            peerFourWords: "test-four-words",
            displayName: nil,
            hasVideo: false
        )

        // Initial state
        XCTAssertEqual(manager.activeCall?.state, .initiating)

        // Transition to connecting
        manager.updateCallState(.connecting)
        XCTAssertEqual(manager.activeCall?.state, .connecting)

        // Transition to connected
        manager.updateCallState(.connected)
        XCTAssertEqual(manager.activeCall?.state, .connected)

        // Transition to reconnecting
        manager.updateCallState(.reconnecting)
        XCTAssertEqual(manager.activeCall?.state, .reconnecting)

        // Back to connected
        manager.updateCallState(.connected)
        XCTAssertEqual(manager.activeCall?.state, .connected)

        // Cleanup
        await manager.endCall()
    }

    // MARK: - Media Control Tests

    func testSetVideoEnabled() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "video-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: true
        )

        XCTAssertTrue(manager.activeCall?.isVideoEnabled ?? false)

        manager.setVideoEnabled(false)
        XCTAssertFalse(manager.activeCall?.isVideoEnabled ?? true)

        manager.setVideoEnabled(true)
        XCTAssertTrue(manager.activeCall?.isVideoEnabled ?? false)

        // Cleanup
        await manager.endCall()
    }

    func testSetAudioEnabled() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "audio-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: false
        )

        XCTAssertTrue(manager.activeCall?.isAudioEnabled ?? false)

        manager.setAudioEnabled(false)
        XCTAssertFalse(manager.activeCall?.isAudioEnabled ?? true)

        manager.setAudioEnabled(true)
        XCTAssertTrue(manager.activeCall?.isAudioEnabled ?? false)

        // Cleanup
        await manager.endCall()
    }

    func testSetScreenSharing() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "screen-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: true
        )

        XCTAssertFalse(manager.activeCall?.isScreenSharing ?? true)

        manager.setScreenSharing(true)
        XCTAssertTrue(manager.activeCall?.isScreenSharing ?? false)

        manager.setScreenSharing(false)
        XCTAssertFalse(manager.activeCall?.isScreenSharing ?? true)

        // Cleanup
        await manager.endCall()
    }

    // MARK: - Incoming Call Tests

    func testHandleIncomingCall() async throws {
        let manager = CallStateManager.shared

        manager.handleIncomingCall(
            callId: "incoming-123",
            callerFourWords: "caller-four-words",
            callerDisplayName: "Caller Name",
            hasVideo: true
        )

        XCTAssertNotNil(manager.incomingCall)
        XCTAssertEqual(manager.incomingCall?.id, "incoming-123")
        XCTAssertEqual(manager.incomingCall?.callerFourWords, "caller-four-words")
        XCTAssertEqual(manager.incomingCall?.callerDisplayName, "Caller Name")
        XCTAssertTrue(manager.incomingCall?.hasVideo ?? false)

        // Cleanup
        await manager.rejectIncomingCall()
    }

    func testAcceptIncomingCall() async throws {
        let manager = CallStateManager.shared

        manager.handleIncomingCall(
            callId: "accept-test",
            callerFourWords: "caller-words",
            callerDisplayName: nil,
            hasVideo: false
        )

        await manager.acceptIncomingCall(withVideo: false)

        XCTAssertNil(manager.incomingCall)
        XCTAssertNotNil(manager.activeCall)
        XCTAssertEqual(manager.activeCall?.id, "accept-test")
        XCTAssertEqual(manager.activeCall?.state, .connecting)

        // Cleanup
        await manager.endCall()
    }

    func testRejectIncomingCall() async throws {
        let manager = CallStateManager.shared

        manager.handleIncomingCall(
            callId: "reject-test",
            callerFourWords: "caller-words",
            callerDisplayName: nil,
            hasVideo: true
        )

        XCTAssertNotNil(manager.incomingCall)

        await manager.rejectIncomingCall()

        XCTAssertNil(manager.incomingCall)
        XCTAssertNil(manager.activeCall)
    }

    // MARK: - Participant Management Tests

    func testAddParticipant() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "participant-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: false
        )

        let participant = CallStateManager.CallParticipant(
            id: "participant-1",
            fourWords: "part-four-words",
            displayName: "Participant",
            isVideoEnabled: true,
            isAudioEnabled: true,
            isSpeaking: false,
            isScreenSharing: false
        )

        manager.addParticipant(participant)

        // Should have 2 participants: self + new one
        XCTAssertEqual(manager.participants.count, 2)
        XCTAssertTrue(manager.participants.contains { $0.id == "participant-1" })

        // Cleanup
        await manager.endCall()
    }

    func testRemoveParticipant() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "remove-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: false
        )

        let participant = CallStateManager.CallParticipant(
            id: "to-remove",
            fourWords: "remove-words",
            displayName: nil,
            isVideoEnabled: false,
            isAudioEnabled: true,
            isSpeaking: false,
            isScreenSharing: false
        )

        manager.addParticipant(participant)
        XCTAssertTrue(manager.participants.contains { $0.id == "to-remove" })

        manager.removeParticipant(id: "to-remove")
        XCTAssertFalse(manager.participants.contains { $0.id == "to-remove" })

        // Cleanup
        await manager.endCall()
    }

    // MARK: - End Call Tests

    func testEndCall() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "end-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: false
        )

        XCTAssertNotNil(manager.activeCall)
        XCTAssertTrue(manager.isInCall)

        await manager.endCall()

        XCTAssertNil(manager.activeCall)
        XCTAssertFalse(manager.isInCall)
        XCTAssertTrue(manager.participants.isEmpty)
    }

    // MARK: - Call Quality Tests

    func testUpdateCallQuality() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "quality-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: false
        )

        XCTAssertEqual(manager.callQuality, .unknown)

        manager.updateCallQuality(.excellent)
        XCTAssertEqual(manager.callQuality, .excellent)

        manager.updateCallQuality(.poor)
        XCTAssertEqual(manager.callQuality, .poor)

        // Cleanup
        await manager.endCall()
    }

    // MARK: - Error Handling Tests

    func testSetAndClearError() async throws {
        let manager = CallStateManager.shared

        XCTAssertNil(manager.lastError)

        manager.setError("Test error message")
        XCTAssertEqual(manager.lastError, "Test error message")

        manager.clearError()
        XCTAssertNil(manager.lastError)
    }

    // MARK: - Helper Tests

    func testCallDuration() async throws {
        let manager = CallStateManager.shared

        manager.initiateCall(
            callId: "duration-test",
            peerFourWords: "test-words",
            displayName: nil,
            hasVideo: false
        )

        // Duration should be greater than 0 after call starts
        try await Task.sleep(nanoseconds: 100_000_000) // 0.1 seconds
        XCTAssertGreaterThan(manager.callDuration, 0)

        // Cleanup
        await manager.endCall()
    }

    func testFormatDuration() {
        let manager = CallStateManager.shared

        XCTAssertEqual(manager.formatDuration(0), "00:00")
        XCTAssertEqual(manager.formatDuration(59), "00:59")
        XCTAssertEqual(manager.formatDuration(60), "01:00")
        XCTAssertEqual(manager.formatDuration(3599), "59:59")
        XCTAssertEqual(manager.formatDuration(3600), "1:00:00")
        XCTAssertEqual(manager.formatDuration(3661), "1:01:01")
    }
}

import Foundation
import CommunitasKit

// MARK: - WebRTC Extensions for CommunitasClient

/// Extension to add WebRTC call methods to the CommunitasClient
/// These are mock implementations for development until the real UniFFI WebRTC bindings are complete
extension CommunitasClient {

    // MARK: - Call Initiation

    /// Initiate a WebRTC call to one or more participants
    /// - Parameters:
    ///   - participants: Array of four-word addresses to call
    ///   - hasVideo: Whether to include video in the call
    /// - Returns: Call ID if successful
    func webrtcInitiateCall(participants: [String], hasVideo: Bool) async throws -> String? {
        print("[Communitas] Initiating call to \(participants) with video: \(hasVideo)")
        let callId = UUID().uuidString

        // Simulate connection after delay
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            WebRTCEventBridge.shared.onCallStateChanged(callId: callId, state: "connected")
        }

        return callId
    }

    /// Initiate a WebRTC call to an entity (channel, group, org)
    /// - Parameters:
    ///   - entityId: The entity ID to call
    ///   - entityType: Type of entity (channel, group, org)
    ///   - hasVideo: Whether to include video in the call
    /// - Returns: Call ID if successful
    func webrtcInitiateEntityCall(entityId: String, entityType: String, hasVideo: Bool) async throws -> String? {
        print("[Communitas] Initiating entity call to \(entityType):\(entityId) with video: \(hasVideo)")
        let callId = UUID().uuidString

        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            WebRTCEventBridge.shared.onCallStateChanged(callId: callId, state: "connected")
        }

        return callId
    }

    // MARK: - Call Control

    /// Accept an incoming call
    /// - Parameters:
    ///   - callId: The call ID to accept
    ///   - withVideo: Whether to accept with video enabled
    func webrtcAcceptCall(callId: String, withVideo: Bool) async throws {
        print("[Communitas] Accepting call \(callId) with video: \(withVideo)")

        await MainActor.run {
            WebRTCEventBridge.shared.onCallStateChanged(callId: callId, state: "connecting")
        }

        Task { @MainActor in
            try? await Task.sleep(nanoseconds: 1_000_000_000)
            WebRTCEventBridge.shared.onCallStateChanged(callId: callId, state: "connected")
        }
    }

    /// Reject an incoming call
    /// - Parameter callId: The call ID to reject
    func webrtcRejectCall(callId: String) async throws {
        print("[Communitas] Rejecting call \(callId)")
    }

    /// End an active call
    /// - Parameter callId: The call ID to end
    func webrtcEndCall(callId: String) async throws {
        print("[Communitas] Ending call \(callId)")
        await MainActor.run {
            WebRTCEventBridge.shared.onCallEnded(callId: callId, reason: "user_ended")
        }
    }

    // MARK: - Media Control

    /// Set audio enabled/disabled for a call
    /// - Parameters:
    ///   - callId: The call ID
    ///   - enabled: Whether audio should be enabled
    func webrtcSetAudioEnabled(callId: String, enabled: Bool) async throws {
        print("[Communitas] Setting audio enabled: \(enabled) for call \(callId)")
    }

    /// Set video enabled/disabled for a call
    /// - Parameters:
    ///   - callId: The call ID
    ///   - enabled: Whether video should be enabled
    func webrtcSetVideoEnabled(callId: String, enabled: Bool) async throws {
        print("[Communitas] Setting video enabled: \(enabled) for call \(callId)")
    }

    /// Start screen sharing for a call
    /// - Parameter callId: The call ID
    func webrtcStartScreenShare(callId: String) async throws {
        print("[Communitas] Starting screen share for call \(callId)")
    }

    /// Stop screen sharing for a call
    /// - Parameter callId: The call ID
    func webrtcStopScreenShare(callId: String) async throws {
        print("[Communitas] Stopping screen share for call \(callId)")
    }
}

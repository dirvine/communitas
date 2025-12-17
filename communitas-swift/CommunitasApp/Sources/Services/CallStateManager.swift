import Foundation
import Combine

/// Centralized manager for tracking and coordinating call states
/// Bridges the WebRTC layer events to SwiftUI views
@MainActor
final class CallStateManager: ObservableObject {

    // MARK: - Singleton

    static let shared = CallStateManager()

    // MARK: - Published Properties

    /// Currently active call, if any
    @Published private(set) var activeCall: ActiveCall?

    /// Incoming call waiting for user response
    @Published private(set) var incomingCall: IncomingCallInfo?

    /// All participants in the current call
    @Published private(set) var participants: [CallParticipant] = []

    /// Call quality metrics
    @Published private(set) var callQuality: CallQuality = .unknown

    /// Error message if call failed
    @Published private(set) var lastError: String?

    // MARK: - Types

    struct ActiveCall: Identifiable, Equatable {
        let id: String
        let peerFourWords: String
        let displayName: String?
        let entityId: String?
        let entityType: String?
        let startTime: Date
        var state: CallState
        var isVideoEnabled: Bool
        var isAudioEnabled: Bool
        var isScreenSharing: Bool

        var isEntityCall: Bool {
            entityId != nil
        }
    }

    enum CallState: String, Equatable {
        case initiating
        case ringing
        case connecting
        case connected
        case reconnecting
        case onHold
        case ending
        case ended
    }

    struct IncomingCallInfo: Identifiable, Equatable {
        let id: String
        let callerFourWords: String
        let callerDisplayName: String?
        let hasVideo: Bool
        let receivedAt: Date
    }

    struct CallParticipant: Identifiable, Equatable {
        let id: String
        let fourWords: String
        let displayName: String?
        var isVideoEnabled: Bool
        var isAudioEnabled: Bool
        var isSpeaking: Bool
        var isScreenSharing: Bool
    }

    enum CallQuality: String, Equatable {
        case unknown
        case excellent
        case good
        case fair
        case poor
    }

    // MARK: - Private Properties

    private var callStartTime: Date?
    private var callDurationTimer: Timer?
    private var cancellables = Set<AnyCancellable>()

    // MARK: - Initialization

    private init() {
        // Set up any necessary observers
    }

    /// Testing initializer that allows creating isolated instances
    internal init(forTesting: Bool) {
        // Don't set up observers in test mode
    }

    // MARK: - Call Lifecycle Management

    /// Initiate a new outgoing call
    func initiateCall(
        callId: String,
        peerFourWords: String,
        displayName: String?,
        hasVideo: Bool
    ) {
        activeCall = ActiveCall(
            id: callId,
            peerFourWords: peerFourWords,
            displayName: displayName,
            entityId: nil,
            entityType: nil,
            startTime: Date(),
            state: .initiating,
            isVideoEnabled: hasVideo,
            isAudioEnabled: true,
            isScreenSharing: false
        )

        // Add ourselves as participant
        participants = [
            CallParticipant(
                id: "self",
                fourWords: "self",  // Will be replaced with actual four words
                displayName: "You",
                isVideoEnabled: hasVideo,
                isAudioEnabled: true,
                isSpeaking: false,
                isScreenSharing: false
            )
        ]
    }

    /// Initiate an entity-based call (group, channel, org)
    func initiateEntityCall(
        callId: String,
        entityId: String,
        entityType: String,
        displayName: String?,
        hasVideo: Bool
    ) {
        activeCall = ActiveCall(
            id: callId,
            peerFourWords: "entity:\(entityId)",
            displayName: displayName,
            entityId: entityId,
            entityType: entityType,
            startTime: Date(),
            state: .initiating,
            isVideoEnabled: hasVideo,
            isAudioEnabled: true,
            isScreenSharing: false
        )

        participants = [
            CallParticipant(
                id: "self",
                fourWords: "self",
                displayName: "You",
                isVideoEnabled: hasVideo,
                isAudioEnabled: true,
                isSpeaking: false,
                isScreenSharing: false
            )
        ]
    }

    /// Handle an incoming call notification
    func handleIncomingCall(
        callId: String,
        callerFourWords: String,
        callerDisplayName: String?,
        hasVideo: Bool
    ) {
        incomingCall = IncomingCallInfo(
            id: callId,
            callerFourWords: callerFourWords,
            callerDisplayName: callerDisplayName,
            hasVideo: hasVideo,
            receivedAt: Date()
        )
    }

    /// Accept an incoming call
    func acceptIncomingCall(withVideo: Bool) async {
        guard let incoming = incomingCall else { return }

        activeCall = ActiveCall(
            id: incoming.id,
            peerFourWords: incoming.callerFourWords,
            displayName: incoming.callerDisplayName,
            entityId: nil,
            entityType: nil,
            startTime: Date(),
            state: .connecting,
            isVideoEnabled: withVideo,
            isAudioEnabled: true,
            isScreenSharing: false
        )

        incomingCall = nil
    }

    /// Reject an incoming call
    func rejectIncomingCall() async {
        incomingCall = nil
    }

    /// End the current call
    func endCall() async {
        guard var call = activeCall else { return }

        call.state = .ending
        activeCall = call

        // Cleanup
        stopCallDurationTimer()
        participants.removeAll()

        // Brief delay to show ending state
        try? await Task.sleep(nanoseconds: 500_000_000)

        activeCall = nil
    }

    // MARK: - Call State Updates

    /// Update call state from WebRTC events
    func updateCallState(_ newState: CallState) {
        guard var call = activeCall else { return }
        call.state = newState
        activeCall = call

        if newState == .connected {
            startCallDurationTimer()
        }
    }

    /// Update video enabled state
    func setVideoEnabled(_ enabled: Bool) {
        guard var call = activeCall else { return }
        call.isVideoEnabled = enabled
        activeCall = call

        // Update self participant
        if var selfParticipant = participants.first(where: { $0.id == "self" }) {
            selfParticipant.isVideoEnabled = enabled
            if let index = participants.firstIndex(where: { $0.id == "self" }) {
                participants[index] = selfParticipant
            }
        }
    }

    /// Update audio enabled state
    func setAudioEnabled(_ enabled: Bool) {
        guard var call = activeCall else { return }
        call.isAudioEnabled = enabled
        activeCall = call

        // Update self participant
        if var selfParticipant = participants.first(where: { $0.id == "self" }) {
            selfParticipant.isAudioEnabled = enabled
            if let index = participants.firstIndex(where: { $0.id == "self" }) {
                participants[index] = selfParticipant
            }
        }
    }

    /// Update screen sharing state
    func setScreenSharing(_ enabled: Bool) {
        guard var call = activeCall else { return }
        call.isScreenSharing = enabled
        activeCall = call

        // Update self participant
        if var selfParticipant = participants.first(where: { $0.id == "self" }) {
            selfParticipant.isScreenSharing = enabled
            if let index = participants.firstIndex(where: { $0.id == "self" }) {
                participants[index] = selfParticipant
            }
        }
    }

    // MARK: - Participant Management

    /// Add a participant to the call
    func addParticipant(_ participant: CallParticipant) {
        if !participants.contains(where: { $0.id == participant.id }) {
            participants.append(participant)
        }
    }

    /// Remove a participant from the call
    func removeParticipant(id: String) {
        participants.removeAll { $0.id == id }
    }

    /// Update participant state
    func updateParticipant(id: String, update: (inout CallParticipant) -> Void) {
        if let index = participants.firstIndex(where: { $0.id == id }) {
            var participant = participants[index]
            update(&participant)
            participants[index] = participant
        }
    }

    // MARK: - Quality Metrics

    /// Update call quality from WebRTC stats
    func updateCallQuality(_ quality: CallQuality) {
        callQuality = quality
    }

    // MARK: - Error Handling

    /// Set error state
    func setError(_ message: String) {
        lastError = message
    }

    /// Clear error state
    func clearError() {
        lastError = nil
    }

    // MARK: - Helpers

    /// Calculate call duration
    var callDuration: TimeInterval {
        guard let call = activeCall else { return 0 }
        return Date().timeIntervalSince(call.startTime)
    }

    /// Check if currently in a call
    var isInCall: Bool {
        activeCall != nil && activeCall?.state != .ended
    }

    /// Check if there's an incoming call
    var hasIncomingCall: Bool {
        incomingCall != nil
    }

    // MARK: - Private Helpers

    private func startCallDurationTimer() {
        callDurationTimer?.invalidate()
        callDurationTimer = Timer.scheduledTimer(withTimeInterval: 1.0, repeats: true) { [weak self] _ in
            Task { @MainActor in
                // Trigger UI update for duration display
                self?.objectWillChange.send()
            }
        }
    }

    private func stopCallDurationTimer() {
        callDurationTimer?.invalidate()
        callDurationTimer = nil
    }

    /// Format duration for display
    func formatDuration(_ duration: TimeInterval) -> String {
        let hours = Int(duration) / 3600
        let minutes = (Int(duration) % 3600) / 60
        let seconds = Int(duration) % 60

        if hours > 0 {
            return String(format: "%d:%02d:%02d", hours, minutes, seconds)
        }
        return String(format: "%02d:%02d", minutes, seconds)
    }
}

// MARK: - WebRTC Event Handler

extension CallStateManager {
    /// Handle events from the WebRTC layer
    /// This will be called by the Rust/UniFFI bridge when events occur
    func handleWebRTCEvent(_ event: WebRTCEvent) {
        switch event {
        case .callStateChanged(let callId, let state):
            if activeCall?.id == callId {
                updateCallState(mapWebRTCState(state))
            }

        case .participantJoined(let callId, let participantId, let fourWords, let displayName):
            if activeCall?.id == callId {
                addParticipant(CallParticipant(
                    id: participantId,
                    fourWords: fourWords,
                    displayName: displayName,
                    isVideoEnabled: false,
                    isAudioEnabled: true,
                    isSpeaking: false,
                    isScreenSharing: false
                ))
            }

        case .participantLeft(let callId, let participantId):
            if activeCall?.id == callId {
                removeParticipant(id: participantId)
            }

        case .participantMediaChanged(let callId, let participantId, let videoEnabled, let audioEnabled):
            if activeCall?.id == callId {
                updateParticipant(id: participantId) { participant in
                    participant.isVideoEnabled = videoEnabled
                    participant.isAudioEnabled = audioEnabled
                }
            }

        case .qualityChanged(let callId, let quality):
            if activeCall?.id == callId {
                updateCallQuality(mapWebRTCQuality(quality))
            }

        case .error(let callId, let message):
            if activeCall?.id == callId || callId.isEmpty {
                setError(message)
            }

        case .incomingCall(let callId, let callerFourWords, let callerDisplayName, let hasVideo):
            handleIncomingCall(
                callId: callId,
                callerFourWords: callerFourWords,
                callerDisplayName: callerDisplayName,
                hasVideo: hasVideo
            )

        case .callEnded(let callId, _):
            if activeCall?.id == callId {
                Task {
                    await endCall()
                }
            }
        }
    }

    private func mapWebRTCState(_ state: String) -> CallState {
        switch state.lowercased() {
        case "initiating": return .initiating
        case "ringing": return .ringing
        case "connecting": return .connecting
        case "connected": return .connected
        case "reconnecting": return .reconnecting
        case "on_hold", "onhold": return .onHold
        case "ending": return .ending
        case "ended": return .ended
        default: return .connecting
        }
    }

    private func mapWebRTCQuality(_ quality: String) -> CallQuality {
        switch quality.lowercased() {
        case "excellent": return .excellent
        case "good": return .good
        case "fair": return .fair
        case "poor": return .poor
        default: return .unknown
        }
    }
}

// MARK: - WebRTC Events (from Rust layer)

enum WebRTCEvent {
    case callStateChanged(callId: String, state: String)
    case participantJoined(callId: String, participantId: String, fourWords: String, displayName: String?)
    case participantLeft(callId: String, participantId: String)
    case participantMediaChanged(callId: String, participantId: String, videoEnabled: Bool, audioEnabled: Bool)
    case qualityChanged(callId: String, quality: String)
    case error(callId: String, message: String)
    case incomingCall(callId: String, callerFourWords: String, callerDisplayName: String?, hasVideo: Bool)
    case callEnded(callId: String, reason: String)
}

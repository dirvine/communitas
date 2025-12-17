import SwiftUI

/// Reusable media control buttons for initiating calls
/// Can be used in entity headers, contact views, and toolbars
struct MediaControlButtons: View {

    // MARK: - Properties

    /// Entity ID for entity-based calls (channels, groups, orgs)
    let entityId: String?

    /// Entity type for entity-based calls
    let entityType: String?

    /// Four-word address for direct 1:1 calls
    let peerFourWords: String?

    /// Display name for call UI
    let displayName: String?

    @EnvironmentObject var appState: AppState
    @StateObject private var mediaService = MediaDeviceService.shared

    @State private var isRequestingPermissions = false
    @State private var showPermissionDeniedAlert = false
    @State private var callError: String?
    @State private var showCallError = false

    // MARK: - Initializers

    /// Initialize for entity-based calls (channels, groups, orgs, projects)
    init(entityId: String, entityType: String, displayName: String? = nil) {
        self.entityId = entityId
        self.entityType = entityType
        self.peerFourWords = nil
        self.displayName = displayName
    }

    /// Initialize for direct 1:1 calls with a contact
    init(peerFourWords: String, displayName: String? = nil) {
        self.entityId = nil
        self.entityType = nil
        self.peerFourWords = peerFourWords
        self.displayName = displayName
    }

    // MARK: - Body

    var body: some View {
        HStack(spacing: 12) {
            // Audio call button
            Button {
                initiateCall(withVideo: false)
            } label: {
                Image(systemName: "phone.fill")
                    .font(.system(size: 16))
                    .foregroundColor(.green)
            }
            .buttonStyle(.plain)
            .help("Start audio call")
            .disabled(isRequestingPermissions)

            // Video call button
            Button {
                initiateCall(withVideo: true)
            } label: {
                Image(systemName: "video.fill")
                    .font(.system(size: 16))
                    .foregroundColor(.blue)
            }
            .buttonStyle(.plain)
            .help("Start video call")
            .disabled(isRequestingPermissions)

            // Screen share button (only for entity calls)
            if entityId != nil {
                Button {
                    startScreenShare()
                } label: {
                    Image(systemName: "rectangle.on.rectangle")
                        .font(.system(size: 16))
                        .foregroundColor(.purple)
                }
                .buttonStyle(.plain)
                .help("Share screen")
                .disabled(isRequestingPermissions)
            }
        }
        .alert("Permission Required", isPresented: $showPermissionDeniedAlert) {
            Button("Open Settings") {
                mediaService.openPrivacySettings()
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("Camera and microphone access are required for calls. Please enable them in System Settings.")
        }
        .alert("Call Error", isPresented: $showCallError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(callError ?? "Failed to initiate call")
        }
    }

    // MARK: - Actions

    private func initiateCall(withVideo: Bool) {
        isRequestingPermissions = true

        Task {
            // Request permissions
            let (cameraGranted, micGranted) = await mediaService.requestAllPermissions()

            // For video calls, we need both
            if withVideo && (!cameraGranted || !micGranted) {
                await MainActor.run {
                    isRequestingPermissions = false
                    showPermissionDeniedAlert = true
                }
                return
            }

            // For audio calls, we just need mic
            if !withVideo && !micGranted {
                await MainActor.run {
                    isRequestingPermissions = false
                    showPermissionDeniedAlert = true
                }
                return
            }

            // Initiate the call
            await MainActor.run {
                isRequestingPermissions = false

                if let peerFourWords = peerFourWords {
                    // Direct 1:1 call
                    initiateDirectCall(to: peerFourWords, withVideo: withVideo)
                } else if let entityId = entityId, let entityType = entityType {
                    // Entity-based call
                    initiateEntityCall(entityId: entityId, entityType: entityType, withVideo: withVideo)
                }
            }
        }
    }

    private func initiateDirectCall(to fourWords: String, withVideo: Bool) {
        // Navigate to call view
        appState.activeView = .call(peerFourWords: fourWords)

        // Call the Rust WebRTC layer via client
        Task {
            do {
                let callId = try await appState.client?.webrtcInitiateCall(
                    participants: [fourWords],
                    hasVideo: withVideo
                )
                print("[Communitas] Call initiated: \(callId ?? "unknown")")
            } catch {
                await MainActor.run {
                    callError = error.localizedDescription
                    showCallError = true
                    // Navigate back on error
                    appState.activeView = .home
                }
            }
        }
    }

    private func initiateEntityCall(entityId: String, entityType: String, withVideo: Bool) {
        // For entity calls, we use a different path
        // The peerFourWords will be the entity ID for routing
        appState.activeView = .call(peerFourWords: "entity:\(entityId)")

        Task {
            do {
                let callId = try await appState.client?.webrtcInitiateEntityCall(
                    entityId: entityId,
                    entityType: entityType,
                    hasVideo: withVideo
                )
                print("[Communitas] Entity call initiated: \(callId ?? "unknown")")
            } catch {
                await MainActor.run {
                    callError = error.localizedDescription
                    showCallError = true
                    appState.activeView = .home
                }
            }
        }
    }

    private func startScreenShare() {
        // Check screen capture permission first
        if mediaService.screenCapturePermission != .authorized {
            mediaService.openScreenRecordingSettings()
            return
        }

        // Navigate to call view for screen share
        if let entityId = entityId {
            appState.activeView = .call(peerFourWords: "screen:\(entityId)")

            Task {
                do {
                    // Start screen share call
                    let callId = try await appState.client?.webrtcInitiateEntityCall(
                        entityId: entityId,
                        entityType: entityType ?? "channel",
                        hasVideo: true
                    )
                    // Then enable screen share
                    if let callId = callId {
                        try await appState.client?.webrtcStartScreenShare(callId: callId)
                    }
                    print("[Communitas] Screen share started")
                } catch {
                    await MainActor.run {
                        callError = error.localizedDescription
                        showCallError = true
                        appState.activeView = .home
                    }
                }
            }
        }
    }
}

// MARK: - Compact Style Variant

/// A more compact version of MediaControlButtons for tight spaces
struct CompactMediaControlButtons: View {
    let entityId: String?
    let entityType: String?
    let peerFourWords: String?

    @EnvironmentObject var appState: AppState

    init(entityId: String, entityType: String) {
        self.entityId = entityId
        self.entityType = entityType
        self.peerFourWords = nil
    }

    init(peerFourWords: String) {
        self.entityId = nil
        self.entityType = nil
        self.peerFourWords = peerFourWords
    }

    var body: some View {
        HStack(spacing: 8) {
            if let peerFourWords = peerFourWords {
                MediaControlButtons(peerFourWords: peerFourWords)
                    .environmentObject(appState)
            } else if let entityId = entityId, let entityType = entityType {
                MediaControlButtons(entityId: entityId, entityType: entityType)
                    .environmentObject(appState)
            }
        }
        .scaleEffect(0.9)
    }
}

// MARK: - Preview

#Preview {
    VStack(spacing: 20) {
        // Contact call buttons
        HStack {
            Text("Contact:")
            Spacer()
            MediaControlButtons(peerFourWords: "test-user-four-words")
        }

        Divider()

        // Channel call buttons
        HStack {
            Text("Channel:")
            Spacer()
            MediaControlButtons(entityId: "channel-123", entityType: "channel")
        }
    }
    .padding()
    .environmentObject(AppState())
}

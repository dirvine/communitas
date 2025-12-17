import SwiftUI

/// Full-screen call view with video/audio controls and participant display
struct CallView: View {

    // MARK: - Properties

    let callId: String
    let peerFourWords: String
    let displayName: String?

    @EnvironmentObject var appState: AppState
    @StateObject private var mediaService = MediaDeviceService.shared
    @StateObject private var callManager = CallStateManager.shared

    @State private var showSettings = false

    // MARK: - Computed Properties

    private var isVideoEnabled: Bool {
        callManager.activeCall?.isVideoEnabled ?? true
    }

    private var isAudioEnabled: Bool {
        callManager.activeCall?.isAudioEnabled ?? true
    }

    private var isScreenSharing: Bool {
        callManager.activeCall?.isScreenSharing ?? false
    }

    private var callState: CallStateManager.CallState {
        callManager.activeCall?.state ?? .initiating
    }

    private var callDuration: TimeInterval {
        callManager.callDuration
    }

    // MARK: - Body

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                // Background
                Color.black
                    .ignoresSafeArea()

                // Main content
                VStack(spacing: 0) {
                    // Top bar with call info
                    topBar

                    // Main video area
                    Spacer()

                    videoArea(size: geometry.size)

                    Spacer()

                    // Call controls
                    callControls
                        .padding(.bottom, 40)
                }
            }
        }
        .onAppear {
            initializeCall()
        }
        .sheet(isPresented: $showSettings) {
            callSettingsSheet
        }
    }

    // MARK: - Top Bar

    private var topBar: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(displayName ?? formatFourWords(peerFourWords))
                    .font(.headline)
                    .foregroundColor(.white)

                HStack(spacing: 8) {
                    // Call state indicator
                    Circle()
                        .fill(callStateColor)
                        .frame(width: 8, height: 8)

                    Text(callStateText)
                        .font(.caption)
                        .foregroundColor(.white.opacity(0.7))

                    if callState == .connected {
                        Text(formatDuration(callDuration))
                            .font(.caption)
                            .foregroundColor(.white.opacity(0.7))
                    }
                }
            }

            Spacer()

            // Settings button
            Button {
                showSettings = true
            } label: {
                Image(systemName: "gear")
                    .font(.system(size: 20))
                    .foregroundColor(.white)
            }
            .buttonStyle(.plain)
        }
        .padding()
        .background(Color.black.opacity(0.3))
    }

    // MARK: - Video Area

    private func videoArea(size: CGSize) -> some View {
        ZStack {
            // Remote video placeholder (in real app, this would show actual video)
            if callState == .connected {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.gray.opacity(0.3))
                    .overlay {
                        VStack(spacing: 16) {
                            Image(systemName: "person.fill")
                                .font(.system(size: 80))
                                .foregroundColor(.white.opacity(0.5))

                            Text(displayName ?? formatFourWords(peerFourWords))
                                .font(.title2)
                                .foregroundColor(.white)
                        }
                    }
                    .padding()
            } else {
                // Connecting/ringing state
                VStack(spacing: 24) {
                    Image(systemName: "person.circle.fill")
                        .font(.system(size: 100))
                        .foregroundColor(.white.opacity(0.7))

                    Text(displayName ?? formatFourWords(peerFourWords))
                        .font(.title)
                        .foregroundColor(.white)

                    Text(callStateText)
                        .font(.headline)
                        .foregroundColor(.white.opacity(0.7))

                    if callState == .initiating || callState == .connecting || callState == .ringing {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            .scaleEffect(1.5)
                    }
                }
            }

            // Local video preview (picture-in-picture)
            if isVideoEnabled && callState == .connected {
                VStack {
                    Spacer()
                    HStack {
                        Spacer()
                        localVideoPreview
                            .frame(width: 120, height: 90)
                            .cornerRadius(8)
                            .shadow(radius: 4)
                            .padding()
                    }
                }
            }
        }
    }

    private var localVideoPreview: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 8)
                .fill(Color.gray.opacity(0.5))

            VStack {
                Image(systemName: "person.fill")
                    .font(.system(size: 24))
                    .foregroundColor(.white.opacity(0.7))
                Text("You")
                    .font(.caption2)
                    .foregroundColor(.white.opacity(0.7))
            }
        }
    }

    // MARK: - Call Controls

    private var callControls: some View {
        HStack(spacing: 24) {
            // Mute button
            CallControlButton(
                icon: isAudioEnabled ? "mic.fill" : "mic.slash.fill",
                isActive: !isAudioEnabled,
                action: toggleAudio
            )

            // Video toggle
            CallControlButton(
                icon: isVideoEnabled ? "video.fill" : "video.slash.fill",
                isActive: !isVideoEnabled,
                action: toggleVideo
            )

            // Screen share toggle
            CallControlButton(
                icon: "rectangle.on.rectangle",
                isActive: isScreenSharing,
                action: toggleScreenShare
            )

            // End call button (red, larger)
            Button {
                endCall()
            } label: {
                ZStack {
                    Circle()
                        .fill(Color.red)
                        .frame(width: 64, height: 64)

                    Image(systemName: "phone.down.fill")
                        .font(.system(size: 24))
                        .foregroundColor(.white)
                }
            }
            .buttonStyle(.plain)
        }
    }

    // MARK: - Settings Sheet

    private var callSettingsSheet: some View {
        NavigationView {
            Form {
                Section("Camera") {
                    Picker("Camera", selection: $mediaService.selectedCamera) {
                        ForEach(mediaService.availableCameras) { camera in
                            Text(camera.name).tag(camera as MediaDeviceService.MediaDevice?)
                        }
                    }
                }

                Section("Microphone") {
                    Picker("Microphone", selection: $mediaService.selectedMicrophone) {
                        ForEach(mediaService.availableMicrophones) { mic in
                            Text(mic.name).tag(mic as MediaDeviceService.MediaDevice?)
                        }
                    }
                }

                Section("Audio") {
                    Toggle("Noise Suppression", isOn: .constant(true))
                    Toggle("Echo Cancellation", isOn: .constant(true))
                }
            }
            .navigationTitle("Call Settings")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        showSettings = false
                    }
                }
            }
        }
        .frame(minWidth: 400, minHeight: 300)
    }

    // MARK: - Actions

    private func toggleAudio() {
        let newState = !isAudioEnabled
        callManager.setAudioEnabled(newState)

        Task {
            do {
                try await appState.client?.webrtcSetAudioEnabled(callId: callId, enabled: newState)
            } catch {
                print("[Communitas] Failed to toggle audio: \(error)")
                // Revert on error
                callManager.setAudioEnabled(!newState)
            }
        }
    }

    private func toggleVideo() {
        let newState = !isVideoEnabled
        callManager.setVideoEnabled(newState)

        Task {
            do {
                try await appState.client?.webrtcSetVideoEnabled(callId: callId, enabled: newState)
            } catch {
                print("[Communitas] Failed to toggle video: \(error)")
                // Revert on error
                callManager.setVideoEnabled(!newState)
            }
        }
    }

    private func toggleScreenShare() {
        Task {
            do {
                if isScreenSharing {
                    try await appState.client?.webrtcStopScreenShare(callId: callId)
                    callManager.setScreenSharing(false)
                } else {
                    // Check permission first
                    if mediaService.screenCapturePermission != .authorized {
                        mediaService.openScreenRecordingSettings()
                        return
                    }
                    try await appState.client?.webrtcStartScreenShare(callId: callId)
                    callManager.setScreenSharing(true)
                }
            } catch {
                print("[Communitas] Failed to toggle screen share: \(error)")
            }
        }
    }

    private func endCall() {
        Task {
            do {
                try await appState.client?.webrtcEndCall(callId: callId)
            } catch {
                print("[Communitas] Failed to end call: \(error)")
            }
            await callManager.endCall()
            await MainActor.run {
                appState.activeView = .home
            }
        }
    }

    private func initializeCall() {
        // Initialize call through CallStateManager
        if peerFourWords.hasPrefix("entity:") {
            let entityId = String(peerFourWords.dropFirst("entity:".count))
            callManager.initiateEntityCall(
                callId: callId,
                entityId: entityId,
                entityType: "channel",
                displayName: displayName,
                hasVideo: true
            )
        } else {
            callManager.initiateCall(
                callId: callId,
                peerFourWords: peerFourWords,
                displayName: displayName,
                hasVideo: true
            )
        }
    }

    // MARK: - Helpers

    private var callStateColor: Color {
        switch callState {
        case .initiating, .connecting, .ringing:
            return .yellow
        case .connected:
            return .green
        case .reconnecting, .onHold:
            return .orange
        case .ending, .ended:
            return .red
        }
    }

    private var callStateText: String {
        switch callState {
        case .initiating:
            return "Initiating..."
        case .connecting:
            return "Connecting..."
        case .ringing:
            return "Ringing..."
        case .connected:
            return "Connected"
        case .reconnecting:
            return "Reconnecting..."
        case .onHold:
            return "On Hold"
        case .ending:
            return "Ending..."
        case .ended:
            return "Ended"
        }
    }

    private func formatDuration(_ duration: TimeInterval) -> String {
        let minutes = Int(duration) / 60
        let seconds = Int(duration) % 60
        return String(format: "%02d:%02d", minutes, seconds)
    }

    private func formatFourWords(_ fourWords: String) -> String {
        // Handle special prefixes
        if fourWords.hasPrefix("entity:") || fourWords.hasPrefix("screen:") {
            let parts = fourWords.split(separator: ":")
            return parts.count > 1 ? String(parts[1]) : fourWords
        }
        return fourWords.replacingOccurrences(of: "-", with: " ").capitalized
    }
}

// MARK: - Call Control Button

struct CallControlButton: View {
    let icon: String
    let isActive: Bool
    var isDestructive: Bool = false
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            ZStack {
                Circle()
                    .fill(backgroundColor)
                    .frame(width: 52, height: 52)

                Image(systemName: icon)
                    .font(.system(size: 20))
                    .foregroundColor(iconColor)
            }
        }
        .buttonStyle(.plain)
    }

    private var backgroundColor: Color {
        if isDestructive {
            return .red
        } else if isActive {
            return .white
        } else {
            return .white.opacity(0.2)
        }
    }

    private var iconColor: Color {
        if isActive || isDestructive {
            return isDestructive ? .white : .black
        } else {
            return .white
        }
    }
}

// MARK: - Incoming Call View

struct IncomingCallView: View {
    let callerFourWords: String
    let callerDisplayName: String?
    let hasVideo: Bool
    let onAccept: () -> Void
    let onReject: () -> Void

    var body: some View {
        VStack(spacing: 32) {
            Spacer()

            // Caller info
            VStack(spacing: 16) {
                Image(systemName: "person.circle.fill")
                    .font(.system(size: 100))
                    .foregroundColor(.white.opacity(0.8))

                Text(callerDisplayName ?? callerFourWords.replacingOccurrences(of: "-", with: " ").capitalized)
                    .font(.title)
                    .foregroundColor(.white)

                Text("Incoming \(hasVideo ? "Video" : "Audio") Call")
                    .font(.headline)
                    .foregroundColor(.white.opacity(0.7))
            }

            Spacer()

            // Accept/Reject buttons
            HStack(spacing: 60) {
                // Reject
                Button(action: onReject) {
                    ZStack {
                        Circle()
                            .fill(Color.red)
                            .frame(width: 72, height: 72)

                        Image(systemName: "phone.down.fill")
                            .font(.system(size: 28))
                            .foregroundColor(.white)
                    }
                }
                .buttonStyle(.plain)

                // Accept
                Button(action: onAccept) {
                    ZStack {
                        Circle()
                            .fill(Color.green)
                            .frame(width: 72, height: 72)

                        Image(systemName: hasVideo ? "video.fill" : "phone.fill")
                            .font(.system(size: 28))
                            .foregroundColor(.white)
                    }
                }
                .buttonStyle(.plain)
            }
            .padding(.bottom, 60)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color.black.opacity(0.9))
    }
}

// MARK: - Preview

#Preview {
    CallView(
        callId: "test-call-123",
        peerFourWords: "ocean-forest-moon-star",
        displayName: "Test User"
    )
    .environmentObject(AppState())
}

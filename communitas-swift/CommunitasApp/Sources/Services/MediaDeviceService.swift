import AVFoundation
import Foundation
#if os(macOS)
import AppKit
#endif

/// Service for managing media devices (camera, microphone) and permissions
@MainActor
public final class MediaDeviceService: ObservableObject {

    // MARK: - Published Properties

    @Published public private(set) var cameraPermission: PermissionStatus = .notDetermined
    @Published public private(set) var microphonePermission: PermissionStatus = .notDetermined
    @Published public private(set) var screenCapturePermission: PermissionStatus = .notDetermined

    @Published public private(set) var availableCameras: [MediaDevice] = []
    @Published public private(set) var availableMicrophones: [MediaDevice] = []

    @Published public var selectedCamera: MediaDevice?
    @Published public var selectedMicrophone: MediaDevice?

    // MARK: - Types

    public enum PermissionStatus {
        case notDetermined
        case authorized
        case denied
        case restricted
    }

    public struct MediaDevice: Identifiable, Equatable, Hashable {
        public let id: String
        public let name: String
        public let deviceType: DeviceType
        public let isDefault: Bool

        public enum DeviceType: Hashable {
            case camera
            case microphone
        }

        public init(id: String, name: String, deviceType: DeviceType, isDefault: Bool) {
            self.id = id
            self.name = name
            self.deviceType = deviceType
            self.isDefault = isDefault
        }

        public func hash(into hasher: inout Hasher) {
            hasher.combine(id)
        }
    }

    // MARK: - Singleton

    public static let shared = MediaDeviceService()

    private init() {
        Task {
            await refreshPermissions()
            await enumerateDevices()
        }
    }

    /// Testing initializer that doesn't enumerate devices
    internal init(forTesting: Bool) {
        // Don't enumerate devices in test mode
    }

    // MARK: - Testing Helpers

    /// Set available cameras for testing
    internal func setAvailableCameras(_ cameras: [MediaDevice]) {
        availableCameras = cameras
    }

    /// Set available microphones for testing
    internal func setAvailableMicrophones(_ microphones: [MediaDevice]) {
        availableMicrophones = microphones
    }

    /// Set camera permission for testing
    internal func setCameraPermission(_ permission: PermissionStatus) {
        cameraPermission = permission
    }

    /// Set microphone permission for testing
    internal func setMicrophonePermission(_ permission: PermissionStatus) {
        microphonePermission = permission
    }

    // MARK: - Permission Management

    /// Refresh current permission status for all media types
    public func refreshPermissions() async {
        cameraPermission = mapAVAuthorizationStatus(AVCaptureDevice.authorizationStatus(for: .video))
        microphonePermission = mapAVAuthorizationStatus(AVCaptureDevice.authorizationStatus(for: .audio))

        // Screen capture permission is checked differently on macOS
        #if os(macOS)
        screenCapturePermission = checkScreenCapturePermission()
        #endif
    }

    /// Request camera permission
    public func requestCameraPermission() async -> Bool {
        let status = AVCaptureDevice.authorizationStatus(for: .video)

        switch status {
        case .authorized:
            cameraPermission = .authorized
            return true

        case .notDetermined:
            let granted = await AVCaptureDevice.requestAccess(for: .video)
            cameraPermission = granted ? .authorized : .denied
            return granted

        case .denied, .restricted:
            cameraPermission = status == .denied ? .denied : .restricted
            return false

        @unknown default:
            cameraPermission = .denied
            return false
        }
    }

    /// Request microphone permission
    public func requestMicrophonePermission() async -> Bool {
        let status = AVCaptureDevice.authorizationStatus(for: .audio)

        switch status {
        case .authorized:
            microphonePermission = .authorized
            return true

        case .notDetermined:
            let granted = await AVCaptureDevice.requestAccess(for: .audio)
            microphonePermission = granted ? .authorized : .denied
            return granted

        case .denied, .restricted:
            microphonePermission = status == .denied ? .denied : .restricted
            return false

        @unknown default:
            microphonePermission = .denied
            return false
        }
    }

    /// Request all media permissions needed for calls
    public func requestAllPermissions() async -> (camera: Bool, microphone: Bool) {
        async let camera = requestCameraPermission()
        async let microphone = requestMicrophonePermission()
        return await (camera, microphone)
    }

    /// Check if we have the minimum permissions for audio calls
    public var canMakeAudioCalls: Bool {
        microphonePermission == .authorized
    }

    /// Check if we have the minimum permissions for video calls
    public var canMakeVideoCalls: Bool {
        cameraPermission == .authorized && microphonePermission == .authorized
    }

    // MARK: - Device Enumeration

    /// Enumerate available cameras and microphones
    public func enumerateDevices() async {
        // Enumerate cameras
        let discoverySession = AVCaptureDevice.DiscoverySession(
            deviceTypes: [.builtInWideAngleCamera, .external],
            mediaType: .video,
            position: .unspecified
        )

        let cameras = discoverySession.devices.map { device in
            MediaDevice(
                id: device.uniqueID,
                name: device.localizedName,
                deviceType: .camera,
                isDefault: device.position == .front || discoverySession.devices.first?.uniqueID == device.uniqueID
            )
        }
        availableCameras = cameras

        // Select default camera if none selected
        if selectedCamera == nil {
            selectedCamera = cameras.first { $0.isDefault } ?? cameras.first
        }

        // Enumerate microphones
        let audioSession = AVCaptureDevice.DiscoverySession(
            deviceTypes: [.microphone, .external],
            mediaType: .audio,
            position: .unspecified
        )

        let microphones = audioSession.devices.map { device in
            MediaDevice(
                id: device.uniqueID,
                name: device.localizedName,
                deviceType: .microphone,
                isDefault: audioSession.devices.first?.uniqueID == device.uniqueID
            )
        }
        availableMicrophones = microphones

        // Select default microphone if none selected
        if selectedMicrophone == nil {
            selectedMicrophone = microphones.first { $0.isDefault } ?? microphones.first
        }
    }

    /// Select a camera by ID
    public func selectCamera(_ deviceId: String) {
        selectedCamera = availableCameras.first { $0.id == deviceId }
    }

    /// Select a microphone by ID
    public func selectMicrophone(_ deviceId: String) {
        selectedMicrophone = availableMicrophones.first { $0.id == deviceId }
    }

    // MARK: - Private Helpers

    private func mapAVAuthorizationStatus(_ status: AVAuthorizationStatus) -> PermissionStatus {
        switch status {
        case .notDetermined:
            return .notDetermined
        case .authorized:
            return .authorized
        case .denied:
            return .denied
        case .restricted:
            return .restricted
        @unknown default:
            return .denied
        }
    }

    #if os(macOS)
    private func checkScreenCapturePermission() -> PermissionStatus {
        // On macOS, screen capture permission is managed through System Preferences
        // We can check if we have permission by attempting to get the display list
        let displayCount = UInt32(16)
        var displays = [CGDirectDisplayID](repeating: 0, count: Int(displayCount))
        var actualCount: UInt32 = 0

        let result = CGGetActiveDisplayList(displayCount, &displays, &actualCount)

        if result == .success && actualCount > 0 {
            // Check if we can capture - this is a heuristic
            // True permission check requires attempting actual capture
            return .authorized
        }

        return .notDetermined
    }
    #endif

    // MARK: - Open System Preferences

    /// Open system preferences to the privacy settings for camera/microphone
    func openPrivacySettings() {
        #if os(macOS)
        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_Camera") {
            NSWorkspace.shared.open(url)
        }
        #endif
    }

    /// Open system preferences to the screen recording settings
    func openScreenRecordingSettings() {
        #if os(macOS)
        if let url = URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture") {
            NSWorkspace.shared.open(url)
        }
        #endif
    }
}

import XCTest
@testable import CommunitasAppLib

/// Tests for MediaDeviceService
@MainActor
final class MediaDeviceServiceTests: XCTestCase {

    // MARK: - Test Fixtures

    var service: MediaDeviceService!

    override func setUp() async throws {
        try await super.setUp()
        service = MediaDeviceService(forTesting: true)
    }

    override func tearDown() async throws {
        service = nil
        try await super.tearDown()
    }

    // MARK: - Permission Status Tests

    func testInitialPermissionStatus() {
        XCTAssertEqual(service.cameraPermission, .notDetermined)
        XCTAssertEqual(service.microphonePermission, .notDetermined)
    }

    func testSetCameraPermission() {
        service.setCameraPermission(.authorized)
        XCTAssertEqual(service.cameraPermission, .authorized)

        service.setCameraPermission(.denied)
        XCTAssertEqual(service.cameraPermission, .denied)

        service.setCameraPermission(.restricted)
        XCTAssertEqual(service.cameraPermission, .restricted)
    }

    func testSetMicrophonePermission() {
        service.setMicrophonePermission(.authorized)
        XCTAssertEqual(service.microphonePermission, .authorized)

        service.setMicrophonePermission(.denied)
        XCTAssertEqual(service.microphonePermission, .denied)

        service.setMicrophonePermission(.restricted)
        XCTAssertEqual(service.microphonePermission, .restricted)
    }

    // MARK: - Can Make Calls Tests

    func testCanMakeAudioCalls_WhenAuthorized() {
        service.setMicrophonePermission(.authorized)
        XCTAssertTrue(service.canMakeAudioCalls)
    }

    func testCanMakeAudioCalls_WhenNotAuthorized() {
        service.setMicrophonePermission(.notDetermined)
        XCTAssertFalse(service.canMakeAudioCalls)

        service.setMicrophonePermission(.denied)
        XCTAssertFalse(service.canMakeAudioCalls)

        service.setMicrophonePermission(.restricted)
        XCTAssertFalse(service.canMakeAudioCalls)
    }

    func testCanMakeVideoCalls_WhenBothAuthorized() {
        service.setCameraPermission(.authorized)
        service.setMicrophonePermission(.authorized)
        XCTAssertTrue(service.canMakeVideoCalls)
    }

    func testCanMakeVideoCalls_WhenOnlyCameraAuthorized() {
        service.setCameraPermission(.authorized)
        service.setMicrophonePermission(.denied)
        XCTAssertFalse(service.canMakeVideoCalls)
    }

    func testCanMakeVideoCalls_WhenOnlyMicrophoneAuthorized() {
        service.setCameraPermission(.denied)
        service.setMicrophonePermission(.authorized)
        XCTAssertFalse(service.canMakeVideoCalls)
    }

    func testCanMakeVideoCalls_WhenNeitherAuthorized() {
        service.setCameraPermission(.denied)
        service.setMicrophonePermission(.denied)
        XCTAssertFalse(service.canMakeVideoCalls)
    }

    // MARK: - Device List Tests

    func testInitialDeviceLists() {
        XCTAssertTrue(service.availableCameras.isEmpty)
        XCTAssertTrue(service.availableMicrophones.isEmpty)
        XCTAssertNil(service.selectedCamera)
        XCTAssertNil(service.selectedMicrophone)
    }

    func testSetAvailableCameras() {
        let cameras = [
            MediaDeviceService.MediaDevice(
                id: "camera-1",
                name: "FaceTime HD Camera",
                deviceType: .camera,
                isDefault: true
            ),
            MediaDeviceService.MediaDevice(
                id: "camera-2",
                name: "External Webcam",
                deviceType: .camera,
                isDefault: false
            )
        ]

        service.setAvailableCameras(cameras)
        XCTAssertEqual(service.availableCameras.count, 2)
        XCTAssertEqual(service.availableCameras[0].name, "FaceTime HD Camera")
        XCTAssertEqual(service.availableCameras[1].name, "External Webcam")
    }

    func testSetAvailableMicrophones() {
        let microphones = [
            MediaDeviceService.MediaDevice(
                id: "mic-1",
                name: "Built-in Microphone",
                deviceType: .microphone,
                isDefault: true
            ),
            MediaDeviceService.MediaDevice(
                id: "mic-2",
                name: "AirPods Pro",
                deviceType: .microphone,
                isDefault: false
            )
        ]

        service.setAvailableMicrophones(microphones)
        XCTAssertEqual(service.availableMicrophones.count, 2)
        XCTAssertEqual(service.availableMicrophones[0].name, "Built-in Microphone")
        XCTAssertEqual(service.availableMicrophones[1].name, "AirPods Pro")
    }

    // MARK: - Device Selection Tests

    func testSelectCamera() {
        let cameras = [
            MediaDeviceService.MediaDevice(
                id: "camera-1",
                name: "FaceTime HD Camera",
                deviceType: .camera,
                isDefault: true
            ),
            MediaDeviceService.MediaDevice(
                id: "camera-2",
                name: "External Webcam",
                deviceType: .camera,
                isDefault: false
            )
        ]

        service.setAvailableCameras(cameras)

        // Select first camera
        service.selectCamera("camera-1")
        XCTAssertEqual(service.selectedCamera?.id, "camera-1")
        XCTAssertEqual(service.selectedCamera?.name, "FaceTime HD Camera")

        // Select second camera
        service.selectCamera("camera-2")
        XCTAssertEqual(service.selectedCamera?.id, "camera-2")
        XCTAssertEqual(service.selectedCamera?.name, "External Webcam")
    }

    func testSelectCamera_NonExistentDevice() {
        let cameras = [
            MediaDeviceService.MediaDevice(
                id: "camera-1",
                name: "FaceTime HD Camera",
                deviceType: .camera,
                isDefault: true
            )
        ]

        service.setAvailableCameras(cameras)
        service.selectCamera("camera-1")
        XCTAssertNotNil(service.selectedCamera)

        // Try to select non-existent camera
        service.selectCamera("non-existent-camera")
        XCTAssertNil(service.selectedCamera)
    }

    func testSelectMicrophone() {
        let microphones = [
            MediaDeviceService.MediaDevice(
                id: "mic-1",
                name: "Built-in Microphone",
                deviceType: .microphone,
                isDefault: true
            ),
            MediaDeviceService.MediaDevice(
                id: "mic-2",
                name: "AirPods Pro",
                deviceType: .microphone,
                isDefault: false
            )
        ]

        service.setAvailableMicrophones(microphones)

        // Select first microphone
        service.selectMicrophone("mic-1")
        XCTAssertEqual(service.selectedMicrophone?.id, "mic-1")
        XCTAssertEqual(service.selectedMicrophone?.name, "Built-in Microphone")

        // Select second microphone
        service.selectMicrophone("mic-2")
        XCTAssertEqual(service.selectedMicrophone?.id, "mic-2")
        XCTAssertEqual(service.selectedMicrophone?.name, "AirPods Pro")
    }

    func testSelectMicrophone_NonExistentDevice() {
        let microphones = [
            MediaDeviceService.MediaDevice(
                id: "mic-1",
                name: "Built-in Microphone",
                deviceType: .microphone,
                isDefault: true
            )
        ]

        service.setAvailableMicrophones(microphones)
        service.selectMicrophone("mic-1")
        XCTAssertNotNil(service.selectedMicrophone)

        // Try to select non-existent microphone
        service.selectMicrophone("non-existent-mic")
        XCTAssertNil(service.selectedMicrophone)
    }

    // MARK: - MediaDevice Type Tests

    func testMediaDeviceEquality() {
        let device1 = MediaDeviceService.MediaDevice(
            id: "device-1",
            name: "Test Device",
            deviceType: .camera,
            isDefault: true
        )

        let device2 = MediaDeviceService.MediaDevice(
            id: "device-1",
            name: "Different Name",
            deviceType: .camera,
            isDefault: false
        )

        let device3 = MediaDeviceService.MediaDevice(
            id: "device-2",
            name: "Test Device",
            deviceType: .camera,
            isDefault: true
        )

        // Same ID should be equal (Equatable based on all properties)
        XCTAssertNotEqual(device1, device2) // Different name and isDefault
        XCTAssertNotEqual(device1, device3) // Different ID
    }

    func testMediaDeviceHashable() {
        let device1 = MediaDeviceService.MediaDevice(
            id: "device-1",
            name: "Test Device",
            deviceType: .camera,
            isDefault: true
        )

        let device2 = MediaDeviceService.MediaDevice(
            id: "device-1",
            name: "Different Name",
            deviceType: .microphone,
            isDefault: false
        )

        let device3 = MediaDeviceService.MediaDevice(
            id: "device-1",
            name: "Test Device",
            deviceType: .camera,
            isDefault: true
        )

        // Hash should be based on ID only
        XCTAssertEqual(device1.hashValue, device2.hashValue)

        // Set membership requires both same hash AND equality
        var set: Set<MediaDeviceService.MediaDevice> = []
        set.insert(device1)
        // device3 is equal to device1 (same properties), so it should be found
        XCTAssertTrue(set.contains(device3))
        // device2 has same hash but different properties, so equality fails
        XCTAssertFalse(set.contains(device2))
    }

    func testMediaDeviceIdentifiable() {
        let device = MediaDeviceService.MediaDevice(
            id: "test-id-123",
            name: "Test Device",
            deviceType: .camera,
            isDefault: true
        )

        XCTAssertEqual(device.id, "test-id-123")
    }

    // MARK: - PermissionStatus Tests

    func testPermissionStatusValues() {
        // Verify all enum cases exist and are distinct
        let statuses: [MediaDeviceService.PermissionStatus] = [
            .notDetermined,
            .authorized,
            .denied,
            .restricted
        ]

        XCTAssertEqual(statuses.count, 4)

        // Verify equality works
        XCTAssertEqual(MediaDeviceService.PermissionStatus.notDetermined, .notDetermined)
        XCTAssertNotEqual(MediaDeviceService.PermissionStatus.authorized, .denied)
    }

    // MARK: - DeviceType Tests

    func testDeviceTypeValues() {
        let cameraDevice = MediaDeviceService.MediaDevice(
            id: "cam",
            name: "Camera",
            deviceType: .camera,
            isDefault: true
        )

        let micDevice = MediaDeviceService.MediaDevice(
            id: "mic",
            name: "Microphone",
            deviceType: .microphone,
            isDefault: true
        )

        XCTAssertEqual(cameraDevice.deviceType, .camera)
        XCTAssertEqual(micDevice.deviceType, .microphone)
        XCTAssertNotEqual(cameraDevice.deviceType, micDevice.deviceType)
    }
}

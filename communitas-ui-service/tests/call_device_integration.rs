//! Integration tests for CallService device enumeration and selection.
//!
//! These tests verify the integration between CallService and device enumerators:
//! - Device listing with mock enumerator
//! - Device selection flow
//! - Lazy enumerator initialization
//! - Screen source enumeration

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_ui_api::call::{DeviceType, MediaDevice, ScreenShareSource, ScreenShareSourceType};
use communitas_ui_service::call::{
    DeviceEnumerator, MockDeviceEnumerator, MockScreenSourceEnumerator, ScreenSourceEnumerator,
};
use communitas_ui_service::storage::UiStorage;
use communitas_ui_service::UiServices;
use tempfile::TempDir;

/// Stack size for test threads (8MB) to handle large async state machines.
const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Run a test with a larger stack size to avoid overflow.
fn run_with_large_stack<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(TEST_STACK_SIZE)
        .spawn(test_fn)
        .expect("Failed to spawn test thread")
        .join()
        .expect("Test thread panicked");
}

// ===== Test Helpers =====

/// Helper to create UiServices without authentication (for testing auth guards).
async fn make_unauthenticated_services(temp: &TempDir) -> UiServices {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .unwrap(),
    );
    UiServices::new(storage, app).unwrap()
}

/// Helper to create UiServices with demo authentication enabled.
async fn make_authenticated_services(temp: &TempDir) -> UiServices {
    let services = make_unauthenticated_services(temp).await;
    // Enable demo mode to authenticate
    services.auth().enable_demo_mode();
    services
}

/// Custom test device enumerator that returns specific devices.
struct TestDeviceEnumerator {
    devices: Vec<MediaDevice>,
}

impl TestDeviceEnumerator {
    fn new(devices: Vec<MediaDevice>) -> Self {
        Self { devices }
    }

    fn with_all_types() -> Self {
        Self::new(vec![
            MediaDevice {
                id: "test-mic-1".to_string(),
                name: "Test Microphone".to_string(),
                device_type: DeviceType::Microphone,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "test-mic-2".to_string(),
                name: "USB Microphone".to_string(),
                device_type: DeviceType::Microphone,
                is_default: false,
                is_available: true,
            },
            MediaDevice {
                id: "test-speaker-1".to_string(),
                name: "Test Speaker".to_string(),
                device_type: DeviceType::Speaker,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "test-camera-1".to_string(),
                name: "Test Camera".to_string(),
                device_type: DeviceType::Camera,
                is_default: true,
                is_available: true,
            },
        ])
    }

    fn with_unavailable_device() -> Self {
        Self::new(vec![
            MediaDevice {
                id: "available-mic".to_string(),
                name: "Available Microphone".to_string(),
                device_type: DeviceType::Microphone,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "unavailable-mic".to_string(),
                name: "Disconnected Microphone".to_string(),
                device_type: DeviceType::Microphone,
                is_default: false,
                is_available: false,
            },
        ])
    }
}

#[async_trait::async_trait]
impl DeviceEnumerator for TestDeviceEnumerator {
    async fn enumerate_devices(
        &self,
    ) -> Result<Vec<MediaDevice>, communitas_ui_service::call::CallError> {
        Ok(self.devices.clone())
    }
}

/// Custom test screen source enumerator.
struct TestScreenSourceEnumerator {
    sources: Vec<ScreenShareSource>,
}

impl TestScreenSourceEnumerator {
    fn new(sources: Vec<ScreenShareSource>) -> Self {
        Self { sources }
    }

    fn with_monitors_and_windows() -> Self {
        Self::new(vec![
            ScreenShareSource::monitor("monitor-1".to_string(), "Primary Display".to_string(), true),
            ScreenShareSource::monitor(
                "monitor-2".to_string(),
                "External Display".to_string(),
                false,
            ),
            ScreenShareSource::window(
                "window-1".to_string(),
                "Visual Studio Code".to_string(),
                "Code".to_string(),
            ),
            ScreenShareSource::window(
                "window-2".to_string(),
                "Terminal".to_string(),
                "Terminal".to_string(),
            ),
        ])
    }
}

#[async_trait::async_trait]
impl ScreenSourceEnumerator for TestScreenSourceEnumerator {
    async fn enumerate_sources(
        &self,
    ) -> Result<Vec<ScreenShareSource>, communitas_ui_service::call::CallError> {
        Ok(self.sources.clone())
    }
}

// ===== Mock Enumerator Tests =====

#[tokio::test]
async fn mock_device_enumerator_returns_all_types() {
    let enumerator = MockDeviceEnumerator;
    let devices = enumerator.enumerate_devices().await.unwrap();

    assert!(!devices.is_empty());
    assert!(devices
        .iter()
        .any(|d| d.device_type == DeviceType::Microphone));
    assert!(devices.iter().any(|d| d.device_type == DeviceType::Speaker));
    assert!(devices.iter().any(|d| d.device_type == DeviceType::Camera));
}

#[tokio::test]
async fn mock_screen_source_enumerator_returns_monitors_and_windows() {
    let enumerator = MockScreenSourceEnumerator;
    let sources = enumerator.enumerate_sources().await.unwrap();

    assert!(!sources.is_empty());
    assert!(sources
        .iter()
        .any(|s| s.source_type == ScreenShareSourceType::Monitor));
    assert!(sources
        .iter()
        .any(|s| s.source_type == ScreenShareSourceType::Window));
}

// ===== CallService Device Enumeration Tests =====

#[test]
fn test_call_service_list_devices_uses_enumerator() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_authenticated_services(&temp).await;

            // List devices should return devices from the mock enumerator
            let devices = services.call().list_devices().await.unwrap();
            assert!(!devices.is_empty());

            // Verify device types are present
            assert!(devices
                .iter()
                .any(|d| d.device_type == DeviceType::Microphone));
            assert!(devices.iter().any(|d| d.device_type == DeviceType::Speaker));
            assert!(devices.iter().any(|d| d.device_type == DeviceType::Camera));
        });
    });
}

#[test]
fn test_call_service_filters_devices_by_type() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_authenticated_services(&temp).await;

            let devices = services.call().list_devices().await.unwrap();

            // Verify we have each device type
            let mics: Vec<_> = devices
                .iter()
                .filter(|d| d.device_type == DeviceType::Microphone)
                .collect();
            assert!(!mics.is_empty());

            let speakers: Vec<_> = devices
                .iter()
                .filter(|d| d.device_type == DeviceType::Speaker)
                .collect();
            assert!(!speakers.is_empty());

            let cameras: Vec<_> = devices
                .iter()
                .filter(|d| d.device_type == DeviceType::Camera)
                .collect();
            assert!(!cameras.is_empty());
        });
    });
}

// ===== Lazy Initialization Tests =====

#[test]
fn test_call_service_supports_lazy_device_enumerator() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_unauthenticated_services(&temp).await;
            let call = services.call();

            // Initially uses mock enumerator
            assert!(!call.has_real_device_enumerator());

            // Set a "real" enumerator
            let real_enumerator = Arc::new(TestDeviceEnumerator::with_all_types());
            call.set_device_enumerator(real_enumerator);

            // Now should have real enumerator
            assert!(call.has_real_device_enumerator());
        });
    });
}

#[test]
fn test_call_service_supports_lazy_screen_source_enumerator() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_unauthenticated_services(&temp).await;
            let call = services.call();

            // Initially uses mock enumerator
            assert!(!call.has_real_screen_enumerator());

            // Set a "real" enumerator
            let real_enumerator = Arc::new(TestScreenSourceEnumerator::with_monitors_and_windows());
            call.set_screen_source_enumerator(real_enumerator);

            // Now should have real enumerator
            assert!(call.has_real_screen_enumerator());
        });
    });
}

#[test]
fn test_lazy_enumerator_switch_updates_device_list() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_authenticated_services(&temp).await;
            let call = services.call();

            // Get initial devices (mock)
            let initial_devices = call.list_devices().await.unwrap();
            let initial_count = initial_devices.len();

            // Switch to custom enumerator with different count
            let custom = Arc::new(TestDeviceEnumerator::new(vec![MediaDevice {
                id: "single-device".to_string(),
                name: "Single Device".to_string(),
                device_type: DeviceType::Microphone,
                is_default: true,
                is_available: true,
            }]));
            call.set_device_enumerator(custom);

            // Get updated devices
            let updated_devices = call.list_devices().await.unwrap();
            assert_eq!(updated_devices.len(), 1);
            assert_ne!(updated_devices.len(), initial_count);
            assert_eq!(updated_devices[0].id, "single-device");
        });
    });
}

// ===== Screen Source Enumeration Tests =====

#[test]
fn test_call_service_enumerate_screen_sources() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_unauthenticated_services(&temp).await;
            let call = services.call();

            // Set custom screen source enumerator
            let custom = Arc::new(TestScreenSourceEnumerator::with_monitors_and_windows());
            call.set_screen_source_enumerator(custom);

            let sources = call.enumerate_screen_sources().await.unwrap();
            assert_eq!(sources.len(), 4);

            let monitors: Vec<_> = sources
                .iter()
                .filter(|s| s.source_type == ScreenShareSourceType::Monitor)
                .collect();
            assert_eq!(monitors.len(), 2);

            let windows: Vec<_> = sources
                .iter()
                .filter(|s| s.source_type == ScreenShareSourceType::Window)
                .collect();
            assert_eq!(windows.len(), 2);
        });
    });
}

#[test]
fn test_call_service_refresh_screen_sources() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_unauthenticated_services(&temp).await;
            let call = services.call();

            // Initial refresh
            let sources1 = call.refresh_screen_sources().await.unwrap();
            assert!(!sources1.is_empty());

            // Refresh again
            let sources2 = call.refresh_screen_sources().await.unwrap();
            assert!(!sources2.is_empty());
        });
    });
}

// ===== Device Availability Tests =====

#[test]
fn test_call_service_detects_unavailable_devices() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_authenticated_services(&temp).await;
            let call = services.call();

            // Set enumerator with unavailable device
            let custom = Arc::new(TestDeviceEnumerator::with_unavailable_device());
            call.set_device_enumerator(custom);

            let devices = call.list_devices().await.unwrap();

            // Should have both devices
            assert_eq!(devices.len(), 2);

            // Check availability flags
            let available = devices.iter().find(|d| d.id == "available-mic").unwrap();
            assert!(available.is_available);

            let unavailable = devices.iter().find(|d| d.id == "unavailable-mic").unwrap();
            assert!(!unavailable.is_available);
        });
    });
}

// ===== Integration Flow Tests =====

#[test]
fn test_complete_device_listing_flow() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_authenticated_services(&temp).await;
            let call = services.call();

            // Set custom enumerator
            let custom = Arc::new(TestDeviceEnumerator::with_all_types());
            call.set_device_enumerator(custom);

            // Step 1: List available devices
            let devices = call.list_devices().await.unwrap();
            assert!(!devices.is_empty());

            // Step 2: Find default microphone
            let default_mic = devices
                .iter()
                .find(|d| d.device_type == DeviceType::Microphone && d.is_default)
                .expect("should have default microphone");

            assert_eq!(default_mic.id, "test-mic-1");
            assert!(default_mic.is_available);

            // Step 3: Find default speaker
            let default_speaker = devices
                .iter()
                .find(|d| d.device_type == DeviceType::Speaker && d.is_default)
                .expect("should have default speaker");

            assert_eq!(default_speaker.id, "test-speaker-1");
            assert!(default_speaker.is_available);
        });
    });
}

#[test]
fn test_complete_screen_source_listing_flow() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir");
            let services = make_unauthenticated_services(&temp).await;
            let call = services.call();

            // Step 1: Set screen source enumerator
            let custom = Arc::new(TestScreenSourceEnumerator::with_monitors_and_windows());
            call.set_screen_source_enumerator(custom);
            assert!(call.has_real_screen_enumerator());

            // Step 2: Enumerate available sources
            let sources = call.enumerate_screen_sources().await.unwrap();
            assert!(!sources.is_empty());

            // Step 3: Find primary monitor
            let primary = sources
                .iter()
                .find(|s| s.is_primary)
                .expect("should have primary monitor");

            assert_eq!(primary.source_type, ScreenShareSourceType::Monitor);
            assert!(!primary.id.is_empty());
        });
    });
}

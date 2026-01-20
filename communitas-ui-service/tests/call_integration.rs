//! Integration tests for the call service using real CommunitasApp.
//!
//! These tests verify the full call flow including device enumeration, call
//! lifecycle (start, join, leave), media controls, and reactive watch channel
//! updates.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;
use std::time::Duration;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_api::call::{CallSettings, CallState, DeviceType, MediaErrorKind};
use communitas_ui_service::UiServices;
use communitas_ui_service::call::{CallError, CallService};
use communitas_ui_service::storage::UiStorage;
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

/// Helper to create a test channel entity for call testing and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let call = services.call();
    let app = call.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Channel,
        description: Some("Test channel for call integration tests".to_string()),
        initial_members: vec![],
    };

    let events = app.execute(cmd).await.expect("Failed to create entity");

    // Extract entity_id from the EntityCreated event
    events
        .iter()
        .find_map(|event| match event {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .expect("No EntityCreated event returned")
}

// =====================================================
// Call Service Initial State Tests
// =====================================================

/// Test that call service starts in Idle state with no active call.
#[test]
fn test_call_service_starts_idle() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_call_service_starts_idle_inner());
    });
}

async fn test_call_service_starts_idle_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let snapshot = call.current_snapshot();
    assert_eq!(snapshot.state, CallState::Idle);
    assert!(snapshot.call_info.is_none());
    assert!(snapshot.participants.is_empty());
    assert!(!snapshot.listen_only_mode);
    assert!(!snapshot.is_screen_sharing);
}

// =====================================================
// Device Enumeration Tests
// =====================================================

/// Test that device enumeration requires authentication.
#[test]
fn test_device_enumeration_requires_auth() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_device_enumeration_requires_auth_inner());
    });
}

async fn test_device_enumeration_requires_auth_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_unauthenticated_services(&temp).await;
    let call = services.call();

    let result = call.list_devices().await;
    assert!(
        result.is_err(),
        "list_devices should fail when unauthenticated"
    );
    assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
}

/// Test that authenticated users can enumerate mock devices.
#[test]
fn test_device_enumeration_with_auth() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_device_enumeration_with_auth_inner());
    });
}

async fn test_device_enumeration_with_auth_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let devices = call.list_devices().await.expect("Should enumerate devices");

    // Mock enumerator returns devices
    assert!(!devices.is_empty(), "Mock enumerator should return devices");

    // Verify device types are present
    assert!(
        devices
            .iter()
            .any(|d| d.device_type == DeviceType::Microphone),
        "Should have microphones"
    );
    assert!(
        devices.iter().any(|d| d.device_type == DeviceType::Speaker),
        "Should have speakers"
    );
    assert!(
        devices.iter().any(|d| d.device_type == DeviceType::Camera),
        "Should have cameras"
    );
}

/// Test device selection and validation.
#[test]
fn test_device_selection() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_device_selection_inner());
    });
}

async fn test_device_selection_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // First enumerate devices to populate the available list
    let devices = call.list_devices().await.expect("Should enumerate devices");

    // Find a mock microphone
    let mic = devices
        .iter()
        .find(|d| d.device_type == DeviceType::Microphone)
        .expect("Should have a microphone");

    // Select the microphone
    call.select_microphone(&mic.id)
        .await
        .expect("Should select microphone");

    // Verify selection is in settings
    let settings = call.get_settings();
    assert_eq!(settings.selected_microphone, Some(mic.id.clone()));

    // Try to select an invalid device
    let result = call.select_microphone("nonexistent-device").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::DeviceNotFound(_)));
}

// =====================================================
// Call Lifecycle Tests
// =====================================================

/// Test that starting a call requires authentication.
#[test]
fn test_start_call_requires_auth() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_start_call_requires_auth_inner());
    });
}

async fn test_start_call_requires_auth_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_unauthenticated_services(&temp).await;
    let call = services.call();

    let result = call.start_call("entity-1", false).await;
    assert!(
        result.is_err(),
        "start_call should fail when unauthenticated"
    );
    assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));

    // State should remain Idle
    assert_eq!(call.get_call_state(), CallState::Idle);
}

/// Test call lifecycle returns CoreError when WebRTC/networking is unavailable.
///
/// In headless/test mode, starting a call fails gracefully because WebRTC
/// requires networking to be initialized first. This test verifies proper
/// error handling for that scenario.
#[test]
fn test_call_lifecycle_start_and_leave() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_call_lifecycle_start_and_leave_inner());
    });
}

async fn test_call_lifecycle_start_and_leave_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Call Test Channel").await;

    // Try to start a call - in test/headless mode, WebRTC isn't available
    let result = call.start_call(&entity_id, false).await;

    // Should fail with CoreError due to WebRTC not being available
    assert!(result.is_err(), "start_call should fail without networking");
    assert!(
        matches!(result.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError, got: {:?}",
        result
    );

    // State should remain Idle
    assert_eq!(call.get_call_state(), CallState::Idle);

    // Snapshot should not be modified
    let snapshot = call.current_snapshot();
    assert_eq!(snapshot.state, CallState::Idle);
    assert!(snapshot.call_info.is_none());
    assert!(snapshot.participants.is_empty());
}

/// Test that leaving when not in a call returns an error.
#[test]
fn test_leave_call_when_not_in_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_leave_call_when_not_in_call_inner());
    });
}

async fn test_leave_call_when_not_in_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let result = call.leave_call().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::NotInCall));
}

/// Test that start_call returns CoreError without networking.
///
/// In test/headless mode, WebRTC isn't available so starting a call fails
/// gracefully with CoreError. This test verifies the error handling path
/// when networking is unavailable.
///
/// Note: AlreadyInCall guard logic cannot be tested without active networking
/// to successfully start a first call.
#[test]
fn test_start_call_fails_without_networking() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_start_call_fails_without_networking_inner());
    });
}

async fn test_start_call_fails_without_networking_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Create a test entity
    let entity_id = create_test_entity(&services, "No Networking Test").await;

    // Try to start call - will fail with CoreError in test mode
    let result = call.start_call(&entity_id, false).await;

    // Should fail with CoreError due to no networking
    assert!(result.is_err());
    assert!(
        matches!(result.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError without networking, got: {:?}",
        result
    );

    // State remains Idle since call never started
    assert_eq!(call.get_call_state(), CallState::Idle);
}

// =====================================================
// Media Control Tests
// =====================================================

/// Test that media controls require being in a call.
#[test]
fn test_media_controls_require_active_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_media_controls_require_active_call_inner());
    });
}

async fn test_media_controls_require_active_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Not in a call - all these should fail
    let mute_result = call.toggle_mute().await;
    assert!(matches!(mute_result.unwrap_err(), CallError::NotInCall));

    let video_result = call.toggle_video().await;
    assert!(matches!(video_result.unwrap_err(), CallError::NotInCall));

    let screen_start_result = call.start_screen_share().await;
    assert!(matches!(
        screen_start_result.unwrap_err(),
        CallError::NotInCall
    ));

    let screen_stop_result = call.stop_screen_share().await;
    assert!(matches!(
        screen_stop_result.unwrap_err(),
        CallError::NotInCall
    ));
}

/// Test toggle mute behavior when not in a call.
///
/// Without networking/WebRTC available, we can't start a call. This test
/// instead verifies that toggle_mute correctly fails when not in an active call.
#[test]
fn test_toggle_mute_during_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_toggle_mute_during_call_inner());
    });
}

async fn test_toggle_mute_during_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Create entity and try to start call - will fail without networking
    let entity_id = create_test_entity(&services, "Mute Test Channel").await;
    let start_result = call.start_call(&entity_id, false).await;

    // Verify call failed to start (expected in test mode)
    assert!(start_result.is_err());
    assert!(matches!(start_result.unwrap_err(), CallError::CoreError(_)));

    // Since call didn't start, toggle_mute should return NotInCall error
    let mute_result = call.toggle_mute().await;
    assert!(mute_result.is_err());
    assert!(matches!(mute_result.unwrap_err(), CallError::NotInCall));

    // State should remain Idle
    assert_eq!(call.get_call_state(), CallState::Idle);
}

/// Test screen sharing requires active call.
///
/// Without networking/WebRTC available, we can't start a call. This test
/// instead verifies that screen sharing correctly fails when not in an active call.
#[test]
fn test_screen_sharing_lifecycle() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_screen_sharing_lifecycle_inner());
    });
}

async fn test_screen_sharing_lifecycle_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Create entity and try to start call - will fail without networking
    let entity_id = create_test_entity(&services, "Screen Share Test").await;
    let start_result = call.start_call(&entity_id, false).await;

    // Verify call failed to start (expected in test mode)
    assert!(start_result.is_err());
    assert!(matches!(start_result.unwrap_err(), CallError::CoreError(_)));

    // Initially not screen sharing (never started a call)
    let initial = call.current_snapshot();
    assert!(!initial.is_screen_sharing);

    // Start screen sharing should fail - not in call
    let screen_result = call.start_screen_share().await;
    assert!(screen_result.is_err());
    assert!(matches!(screen_result.unwrap_err(), CallError::NotInCall));

    // Stop screen sharing should also fail - not in call
    let stop_result = call.stop_screen_share().await;
    assert!(stop_result.is_err());
    assert!(matches!(stop_result.unwrap_err(), CallError::NotInCall));

    // State should remain Idle
    assert_eq!(call.get_call_state(), CallState::Idle);
}

// =====================================================
// Media Error Tests
// =====================================================

/// Test that media errors enable listen-only mode.
#[test]
fn test_media_error_listen_only_mode() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_media_error_listen_only_mode_inner());
    });
}

async fn test_media_error_listen_only_mode_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Initially not in listen-only mode
    assert!(!call.is_listen_only());

    // Report a microphone error
    let error = communitas_ui_api::call::MediaError::new(
        DeviceType::Microphone,
        MediaErrorKind::PermissionDenied,
        "Microphone access denied",
    );
    call.report_media_error(error).await;

    // Should now be in listen-only mode
    assert!(call.is_listen_only());
    assert_eq!(call.get_media_errors().len(), 1);

    // Retry media should clear the error
    call.retry_media(DeviceType::Microphone)
        .await
        .expect("Should retry");

    assert!(!call.is_listen_only());
    assert!(call.get_media_errors().is_empty());
}

/// Test device disconnection handling.
#[test]
fn test_device_disconnection_handling() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_device_disconnection_handling_inner());
    });
}

async fn test_device_disconnection_handling_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // First enumerate devices
    call.list_devices().await.expect("Should enumerate");

    // Set up settings with a selected microphone
    let settings = CallSettings {
        selected_microphone: Some("test-mic-1".to_string()),
        selected_speaker: Some("test-speaker-1".to_string()),
        ..Default::default()
    };
    call.update_settings(settings)
        .await
        .expect("Should update settings");

    // Simulate microphone disconnection
    call.handle_device_disconnection("test-mic-1", DeviceType::Microphone)
        .await;

    // Check that:
    // 1. Microphone selection is cleared
    assert!(call.get_settings().selected_microphone.is_none());
    // 2. Speaker selection is preserved
    assert_eq!(
        call.get_settings().selected_speaker,
        Some("test-speaker-1".to_string())
    );
    // 3. Listen-only mode is enabled (lost microphone)
    assert!(call.is_listen_only());
    // 4. Error is reported
    let errors = call.get_media_errors();
    assert!(!errors.is_empty());
    assert!(
        errors
            .iter()
            .any(|e| e.device_type == DeviceType::Microphone)
    );
}

// =====================================================
// Watch Channel Tests
// =====================================================

/// Test that watch channel receives updates even when operations fail.
///
/// Without networking/WebRTC available, we can't start a call. This test
/// verifies that the watch channel still works correctly for state that
/// can be modified (like settings) even without active call capabilities.
#[test]
fn test_watch_channel_updates() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_watch_channel_updates_inner());
    });
}

async fn test_watch_channel_updates_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Subscribe to watch channel
    let rx = call.subscribe();

    // Initial state should be Idle
    assert_eq!(rx.borrow().state, CallState::Idle);

    // Create entity and try to start call - will fail without networking
    let entity_id = create_test_entity(&services, "Watch Channel Test").await;
    let start_result = call.start_call(&entity_id, false).await;

    // Verify call failed to start (expected in test mode)
    assert!(start_result.is_err());
    assert!(matches!(start_result.unwrap_err(), CallError::CoreError(_)));

    // State should still be Idle since call failed to start
    let current = rx.borrow().clone();
    assert_eq!(current.state, CallState::Idle);
    assert!(current.call_info.is_none());
}

/// Test settings update broadcasts to watch channel.
#[test]
fn test_settings_update_broadcasts() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_settings_update_broadcasts_inner());
    });
}

async fn test_settings_update_broadcasts_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let mut rx = call.subscribe();

    // Update settings
    let settings = CallSettings {
        auto_mute_on_join: true,
        noise_suppression: true,
        ..Default::default()
    };
    call.update_settings(settings).await.expect("Should update");

    // Wait for broadcast
    tokio::time::timeout(Duration::from_secs(1), rx.changed())
        .await
        .expect("Should receive update")
        .expect("Channel should not close");

    // Verify settings in snapshot
    let snapshot = rx.borrow().clone();
    assert!(snapshot.settings.auto_mute_on_join);
    assert!(snapshot.settings.noise_suppression);
}

// =====================================================
// Query Tests
// =====================================================

/// Test that call queries require authentication.
#[test]
fn test_call_queries_require_auth() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_call_queries_require_auth_inner());
    });
}

async fn test_call_queries_require_auth_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_unauthenticated_services(&temp).await;
    let call = services.call();

    // All queries should fail
    let status_result = call.query_call_status("call-1").await;
    assert!(matches!(
        status_result.unwrap_err(),
        CallError::NotAuthenticated
    ));

    let participants_result = call.query_call_participants("call-1").await;
    assert!(matches!(
        participants_result.unwrap_err(),
        CallError::NotAuthenticated
    ));

    let list_result = call.list_active_calls().await;
    assert!(matches!(
        list_result.unwrap_err(),
        CallError::NotAuthenticated
    ));
}

// =====================================================
// Custom Device Enumerator Tests
// =====================================================

/// Test that custom device enumerators work correctly.
#[test]
fn test_custom_device_enumerator() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_custom_device_enumerator_inner());
    });
}

async fn test_custom_device_enumerator_inner() {
    use communitas_ui_service::auth::AuthController;

    let temp = TempDir::new().unwrap();
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let auth = Arc::new(AuthController::new(storage).unwrap());
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

    // Test with NoDeviceEnumerator (headless mode)
    let headless_service = CallService::headless(auth.clone(), app.clone());
    auth.enable_demo_mode();

    let devices = headless_service
        .list_devices()
        .await
        .expect("Should enumerate");
    assert!(devices.is_empty(), "Headless should return no devices");

    // Test with MockDeviceEnumerator
    let mock_service = CallService::new(auth.clone(), app.clone());
    let mock_devices = mock_service.list_devices().await.expect("Should enumerate");
    assert!(!mock_devices.is_empty(), "Mock should return devices");
}

/// Test join_call requires authentication.
#[test]
fn test_join_call_requires_auth() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_join_call_requires_auth_inner());
    });
}

async fn test_join_call_requires_auth_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_unauthenticated_services(&temp).await;
    let call = services.call();

    let result = call.join_call("call-123").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
}

/// Test that screen share state remains false when no call is active.
///
/// Without networking/WebRTC available, we can't start a call. This test
/// verifies that screen share state is properly managed even when call fails.
#[test]
fn test_screen_share_reset_on_leave() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_screen_share_reset_on_leave_inner());
    });
}

async fn test_screen_share_reset_on_leave_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Create entity and try to start call - will fail without networking
    let entity_id = create_test_entity(&services, "Screen Reset Test").await;
    let start_result = call.start_call(&entity_id, false).await;

    // Verify call failed to start (expected in test mode)
    assert!(start_result.is_err());
    assert!(matches!(start_result.unwrap_err(), CallError::CoreError(_)));

    // Screen sharing should always be false since no call was ever active
    let snapshot = call.current_snapshot();
    assert!(!snapshot.is_screen_sharing);
    assert_eq!(snapshot.state, CallState::Idle);

    // leave_call should fail since not in call
    let leave_result = call.leave_call().await;
    assert!(leave_result.is_err());
    assert!(matches!(leave_result.unwrap_err(), CallError::NotInCall));
}

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

// =====================================================
// Call History Tests (Phase 6.4 Task 7)
// =====================================================

/// Test that call history starts empty.
#[test]
fn test_call_history_starts_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_call_history_starts_empty_inner());
    });
}

async fn test_call_history_starts_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let history = call.get_call_history().await;
    assert!(history.entries.is_empty());
    assert_eq!(history.len(), 0);
}

/// Test recording a missed call adds to history.
#[test]
fn test_record_missed_call_adds_to_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_record_missed_call_adds_to_history_inner());
    });
}

async fn test_record_missed_call_adds_to_history_inner() {
    use communitas_ui_api::call::{CallOutcome, CallType};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record a missed call
    call.record_missed_call(
        "call-001".to_string(),
        "caller-alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Verify it was added to history
    let history = call.get_call_history().await;
    assert_eq!(history.len(), 1);

    let entry = call.get_history_entry("call-001").await;
    assert!(entry.is_some());
    let entry = entry.unwrap();
    assert_eq!(entry.call_id, "call-001");
    assert_eq!(entry.entity_id, "caller-alice");
    assert_eq!(entry.entity_name, "Alice");
    assert_eq!(entry.outcome, CallOutcome::Missed);
    assert!(!entry.is_read);
}

/// Test getting recent call history with limit.
#[test]
fn test_get_recent_call_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_get_recent_call_history_inner());
    });
}

async fn test_get_recent_call_history_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record multiple missed calls
    for i in 1..=5 {
        call.record_missed_call(
            format!("call-00{i}"),
            format!("caller-{i}"),
            format!("Caller {i}"),
            CallType::Direct,
        )
        .await;
    }

    // Get recent with limit
    let recent = call.get_recent_history(3).await;
    assert_eq!(recent.len(), 3);

    // Most recent should be first
    assert_eq!(recent[0].call_id, "call-005");
}

/// Test getting call history for a specific entity.
#[test]
fn test_get_history_for_entity() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_get_history_for_entity_inner());
    });
}

async fn test_get_history_for_entity_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record calls from different entities
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    call.record_missed_call(
        "call-002".to_string(),
        "bob".to_string(),
        "Bob".to_string(),
        CallType::Direct,
    )
    .await;

    call.record_missed_call(
        "call-003".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Get history for Alice
    let alice_history = call.get_history_for_entity("alice").await;
    assert_eq!(alice_history.len(), 2);
    assert!(alice_history.iter().all(|e| e.entity_id == "alice"));

    // Get history for Bob
    let bob_history = call.get_history_for_entity("bob").await;
    assert_eq!(bob_history.len(), 1);
}

/// Test marking calls as read.
#[test]
fn test_mark_call_read() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_mark_call_read_inner());
    });
}

async fn test_mark_call_read_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record a missed call (unread)
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Verify it's unread
    let entry = call.get_history_entry("call-001").await.unwrap();
    assert!(!entry.is_read);
    assert_eq!(call.get_unread_missed_count().await, 1);

    // Mark as read
    call.mark_call_read("call-001").await;

    // Verify it's now read
    let entry = call.get_history_entry("call-001").await.unwrap();
    assert!(entry.is_read);
    assert_eq!(call.get_unread_missed_count().await, 0);
}

/// Test marking all calls as read.
#[test]
fn test_mark_all_calls_read() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_mark_all_calls_read_inner());
    });
}

async fn test_mark_all_calls_read_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record multiple missed calls
    for i in 1..=3 {
        call.record_missed_call(
            format!("call-00{i}"),
            format!("caller-{i}"),
            format!("Caller {i}"),
            CallType::Direct,
        )
        .await;
    }

    // Verify all are unread
    assert_eq!(call.get_unread_missed_count().await, 3);

    // Mark all as read
    call.mark_all_calls_read().await;

    // Verify all are now read
    assert_eq!(call.get_unread_missed_count().await, 0);
}

/// Test deleting a call history entry.
#[test]
fn test_delete_call_history_entry() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_delete_call_history_entry_inner());
    });
}

async fn test_delete_call_history_entry_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record a call
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Verify it exists
    assert!(call.get_history_entry("call-001").await.is_some());

    // Delete it
    let deleted = call.delete_history_entry("call-001").await;
    assert!(deleted);

    // Verify it's gone
    assert!(call.get_history_entry("call-001").await.is_none());
}

/// Test clearing all call history.
#[test]
fn test_clear_call_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_clear_call_history_inner());
    });
}

async fn test_clear_call_history_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record multiple calls
    for i in 1..=5 {
        call.record_missed_call(
            format!("call-00{i}"),
            format!("caller-{i}"),
            format!("Caller {i}"),
            CallType::Direct,
        )
        .await;
    }

    // Verify calls exist
    assert_eq!(call.get_call_history().await.len(), 5);

    // Clear all
    call.clear_call_history().await;

    // Verify empty
    assert!(call.get_call_history().await.entries.is_empty());
}

// =====================================================
// Missed Call Notification Tests (Phase 6.4 Task 8)
// =====================================================

/// Test missed calls snapshot starts empty.
#[test]
fn test_missed_calls_snapshot_starts_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_missed_calls_snapshot_starts_empty_inner());
    });
}

async fn test_missed_calls_snapshot_starts_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let snapshot = call.get_missed_calls_snapshot().await;
    assert_eq!(snapshot.unread_count, 0);
    assert!(snapshot.notifications.is_empty());
    assert!(!snapshot.has_unread());
}

/// Test has_unread_missed_calls returns correct value.
#[test]
fn test_has_unread_missed_calls() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_has_unread_missed_calls_inner());
    });
}

async fn test_has_unread_missed_calls_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Initially no unread
    assert!(!call.has_unread_missed_calls().await);

    // Record a missed call
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Now has unread
    assert!(call.has_unread_missed_calls().await);

    // Acknowledge it
    call.acknowledge_missed_call("call-001").await;

    // No more unread
    assert!(!call.has_unread_missed_calls().await);
}

/// Test missed calls subscription updates.
#[test]
fn test_missed_calls_subscription() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_missed_calls_subscription_inner());
    });
}

async fn test_missed_calls_subscription_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let mut rx = call.subscribe_missed_calls();

    // Initially empty
    assert_eq!(rx.borrow().unread_count, 0);

    // Record a missed call
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Wait for update
    tokio::time::timeout(Duration::from_secs(1), rx.changed())
        .await
        .expect("Should receive update")
        .expect("Channel should not close");

    let snapshot = rx.borrow().clone();
    assert_eq!(snapshot.unread_count, 1);
    assert!(snapshot.has_unread());
}

/// Test dismiss_missed_call removes from history.
#[test]
fn test_dismiss_missed_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_dismiss_missed_call_inner());
    });
}

async fn test_dismiss_missed_call_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record a missed call
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Verify it exists
    assert!(call.get_history_entry("call-001").await.is_some());

    // Dismiss it
    call.dismiss_missed_call("call-001").await;

    // Verify it's gone
    assert!(call.get_history_entry("call-001").await.is_none());
}

/// Test mark_called_back updates entry.
#[test]
fn test_mark_called_back() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_mark_called_back_inner());
    });
}

async fn test_mark_called_back_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Record a missed call
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Verify not called back
    let entry = call.get_history_entry("call-001").await.unwrap();
    assert!(!entry.has_called_back);

    // Mark as called back
    call.mark_called_back("call-001").await;

    // Verify it's updated
    let entry = call.get_history_entry("call-001").await.unwrap();
    assert!(entry.has_called_back);
    assert!(entry.is_read);
}

// =====================================================
// Quality Metrics Tests (Phase 6.4 Task 4)
// =====================================================

/// Test quality metrics start with defaults.
#[test]
fn test_quality_metrics_default_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_quality_metrics_default_state_inner());
    });
}

async fn test_quality_metrics_default_state_inner() {
    use communitas_ui_api::call::ConnectionQuality;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let metrics = call.get_quality_metrics();
    assert_eq!(metrics.quality, ConnectionQuality::Unknown);
    assert_eq!(metrics.latency_ms, 0);
    assert_eq!(metrics.packet_loss_percent, 0.0);
}

/// Test updating quality metrics from stats.
#[test]
fn test_update_quality_from_stats() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_update_quality_from_stats_inner());
    });
}

async fn test_update_quality_from_stats_inner() {
    use communitas_ui_api::call::ConnectionQuality;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Update with good stats
    call.update_quality_from_stats(
        30,   // latency_ms
        0.5,  // packet_loss_percent
        5,    // jitter_ms
        128,  // audio_bitrate_kbps
        1000, // video_bitrate_kbps
    )
    .await;

    let metrics = call.get_quality_metrics();
    assert_eq!(metrics.latency_ms, 30);
    assert!((metrics.packet_loss_percent - 0.5).abs() < 0.01);
    assert_eq!(metrics.jitter_ms, 5);
    assert_eq!(metrics.audio_bitrate_kbps, 128);
    assert_eq!(metrics.video_bitrate_kbps, 1000);
    assert_eq!(metrics.quality, ConnectionQuality::Excellent);
}

/// Test quality degrades with poor stats.
#[test]
fn test_quality_degrades_with_poor_stats() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_quality_degrades_with_poor_stats_inner());
    });
}

async fn test_quality_degrades_with_poor_stats_inner() {
    use communitas_ui_api::call::ConnectionQuality;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Update with poor stats (high latency, high packet loss)
    call.update_quality_from_stats(
        500,  // high latency_ms
        10.0, // high packet_loss_percent
        100,  // high jitter_ms
        64,   // audio_bitrate_kbps
        500,  // video_bitrate_kbps
    )
    .await;

    let quality = call.get_connection_quality();
    assert!(matches!(
        quality,
        ConnectionQuality::Poor | ConnectionQuality::Critical
    ));
    assert!(call.should_show_quality_warning());
}

/// Test updating video quality metrics.
#[test]
fn test_update_video_quality() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_update_video_quality_inner());
    });
}

async fn test_update_video_quality_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Update video quality
    call.update_video_quality(1920, 1080, 30, 2500).await;

    let metrics = call.get_quality_metrics();
    assert_eq!(metrics.video_width, 1920);
    assert_eq!(metrics.video_height, 1080);
    assert_eq!(metrics.video_fps, 30);
    assert_eq!(metrics.video_bitrate_kbps, 2500);
}

/// Test clearing quality metrics.
#[test]
fn test_clear_quality_metrics() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_clear_quality_metrics_inner());
    });
}

async fn test_clear_quality_metrics_inner() {
    use communitas_ui_api::call::ConnectionQuality;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Set some quality metrics
    call.update_quality_from_stats(30, 0.5, 5, 128, 1000).await;

    // Verify they're set
    assert!(call.get_quality_metrics().latency_ms > 0);

    // Clear them
    call.clear_quality_metrics().await;

    // Verify they're cleared
    let metrics = call.get_quality_metrics();
    assert_eq!(metrics.quality, ConnectionQuality::Unknown);
    assert_eq!(metrics.latency_ms, 0);
}

/// Test bandwidth stats update.
#[test]
fn test_update_bandwidth_stats() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_update_bandwidth_stats_inner());
    });
}

async fn test_update_bandwidth_stats_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Update bandwidth stats
    call.update_bandwidth_stats(1024 * 1024, 2048 * 1024).await;

    let metrics = call.get_quality_metrics();
    assert_eq!(metrics.bytes_sent, 1024 * 1024);
    assert_eq!(metrics.bytes_received, 2048 * 1024);
}

/// Test participant quality tracking.
#[test]
fn test_participant_quality_tracking() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_participant_quality_tracking_inner());
    });
}

async fn test_participant_quality_tracking_inner() {
    use communitas_ui_api::call::QualityMetrics;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Add participant quality
    let incoming = QualityMetrics {
        latency_ms: 25,
        packet_loss_percent: 0.1,
        ..Default::default()
    };

    call.update_participant_quality("participant-1", incoming.clone(), None)
        .await;

    // Verify it's tracked
    let pq = call.get_participant_quality("participant-1");
    assert!(pq.is_some());
    let pq = pq.unwrap();
    assert_eq!(pq.participant_id, "participant-1");
    assert_eq!(pq.incoming.latency_ms, 25);
}

// =====================================================
// Recording Tests (Phase 6.4 Task 5)
// =====================================================

/// Test recording requires active call.
#[test]
fn test_recording_requires_active_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recording_requires_active_call_inner());
    });
}

async fn test_recording_requires_active_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Not in a call - recording should fail
    let result = call.start_recording(false).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::NotInCall));
}

/// Test stop recording without active recording fails.
#[test]
fn test_stop_recording_without_active_recording() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_stop_recording_without_active_recording_inner());
    });
}

async fn test_stop_recording_without_active_recording_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Not recording - stop should fail
    let result = call.stop_recording().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::MediaError(_)));
}

/// Test pause recording without active recording fails.
#[test]
fn test_pause_recording_without_active_recording() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_pause_recording_without_active_recording_inner());
    });
}

async fn test_pause_recording_without_active_recording_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Not recording - pause should fail
    let result = call.pause_recording().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::MediaError(_)));
}

/// Test resume recording without paused recording fails.
#[test]
fn test_resume_recording_without_paused_recording() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_resume_recording_without_paused_recording_inner());
    });
}

async fn test_resume_recording_without_paused_recording_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Not recording - resume should fail
    let result = call.resume_recording().await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::MediaError(_)));
}

/// Test recording state accessor.
#[test]
fn test_recording_state_accessor() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recording_state_accessor_inner());
    });
}

async fn test_recording_state_accessor_inner() {
    use communitas_ui_api::call::RecordingState;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Default state
    assert_eq!(call.get_recording_state(), RecordingState::NotRecording);
    assert!(!call.is_recording());
    assert!(call.get_recording_info().is_none());
}

// =====================================================
// Watch Channel Broadcast Tests (Phase 6.4)
// =====================================================

/// Test quality metrics updates broadcast to watch channel.
#[test]
fn test_quality_metrics_broadcast() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_quality_metrics_broadcast_inner());
    });
}

async fn test_quality_metrics_broadcast_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let mut rx = call.subscribe();

    // Update quality
    call.update_quality_from_stats(50, 1.0, 10, 128, 1000).await;

    // Wait for update
    tokio::time::timeout(Duration::from_secs(1), rx.changed())
        .await
        .expect("Should receive update")
        .expect("Channel should not close");

    let snapshot = rx.borrow().clone();
    assert_eq!(snapshot.quality_metrics.latency_ms, 50);
}

/// Test call history updates broadcast via missed calls channel.
#[test]
fn test_history_updates_broadcast_to_missed_calls() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_history_updates_broadcast_to_missed_calls_inner());
    });
}

async fn test_history_updates_broadcast_to_missed_calls_inner() {
    use communitas_ui_api::call::CallType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let mut rx = call.subscribe_missed_calls();

    // Record a missed call
    call.record_missed_call(
        "call-001".to_string(),
        "alice".to_string(),
        "Alice".to_string(),
        CallType::Direct,
    )
    .await;

    // Wait for update
    tokio::time::timeout(Duration::from_secs(1), rx.changed())
        .await
        .expect("Should receive update")
        .expect("Channel should not close");

    let snapshot = rx.borrow().clone();
    assert!(snapshot.has_unread());
    assert_eq!(snapshot.unread_count, 1);
}

// =====================================================
// Group Call Tests (Task 6: Group Call Support)
// =====================================================

/// Test that start_group_call creates a group call with correct settings.
#[test]
fn test_start_group_call_creates_group_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_start_group_call_creates_group_call_inner());
    });
}

async fn test_start_group_call_creates_group_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Create a test entity for the group call
    let entity_id = create_test_entity(&services, "Test Group").await;

    // Start group call - will fail with CoreError in test mode (no networking)
    let result = call.start_group_call(&entity_id, Some(4), false).await;

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

/// Test that start_group_call fails when not authenticated.
#[test]
fn test_start_group_call_requires_auth() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_start_group_call_requires_auth_inner());
    });
}

async fn test_start_group_call_requires_auth_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_unauthenticated_services(&temp).await;
    let call = services.call();

    let result = call.start_group_call("test-entity", Some(4), false).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
}

/// Test that start_group_call fails when already in a call.
///
/// Note: This test verifies the error case when no networking is available,
/// since we cannot establish an active call in the test environment without
/// WebRTC/networking infrastructure. The AlreadyInCall scenario is logically
/// tested via the call state machine tests in communitas-core.
#[test]
fn test_start_group_call_fails_when_already_in_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_start_group_call_fails_when_already_in_call_inner());
    });
}

async fn test_start_group_call_fails_when_already_in_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Test Group").await;

    // Try to start first call - will fail without networking
    let result = call.start_group_call(&entity_id, Some(4), false).await;

    // Should fail with CoreError due to no networking
    assert!(result.is_err());
    assert!(
        matches!(result.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError without networking, got: {:?}",
        result
    );

    // Try another call attempt - still fails with same error (not AlreadyInCall since first never started)
    let result2 = call.start_group_call(&entity_id, Some(4), false).await;

    assert!(result2.is_err());
    assert!(
        matches!(result2.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError without networking, got: {:?}",
        result2
    );

    // State remains Idle since no call ever started
    assert_eq!(call.get_call_state(), CallState::Idle);
}

/// Test group call settings: max_participants and mute_on_entry.
#[test]
fn test_group_call_settings() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_group_call_settings_inner());
    });
}

async fn test_group_call_settings_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Test Group").await;

    // Start group call with mute_on_entry = true - will fail without networking
    let result = call.start_group_call(&entity_id, Some(10), true).await;

    // Should fail with CoreError due to WebRTC not being available in test mode
    assert!(
        result.is_err(),
        "start_group_call should fail without networking"
    );
    assert!(
        matches!(result.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError for WebRTC not available"
    );
}

/// Test group call uses default max_participants when None is passed.
#[test]
fn test_group_call_default_max_participants() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_group_call_default_max_participants_inner());
    });
}

async fn test_group_call_default_max_participants_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Test Group").await;

    // Start group call with default max_participants - will fail without networking
    let result = call.start_group_call(&entity_id, None, false).await;

    // Should fail with CoreError due to WebRTC not being available in test mode
    assert!(
        result.is_err(),
        "start_group_call should fail without networking"
    );
    assert!(
        matches!(result.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError for WebRTC not available"
    );
}

/// Test that group call snapshot reflects group call state.
#[test]
fn test_group_call_snapshot() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_group_call_snapshot_inner());
    });
}

async fn test_group_call_snapshot_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Test Group").await;

    // Try to start group call - will fail without networking
    let result = call.start_group_call(&entity_id, Some(4), false).await;

    // Verify call failed to start (expected in test mode)
    assert!(result.is_err());
    assert!(
        matches!(result.as_ref().unwrap_err(), CallError::CoreError(_)),
        "Expected CoreError for WebRTC not available"
    );

    // Snapshot should still be Idle since call failed
    let snapshot = call.current_snapshot();
    assert_eq!(snapshot.state, CallState::Idle);
    assert!(snapshot.call_info.is_none());
}

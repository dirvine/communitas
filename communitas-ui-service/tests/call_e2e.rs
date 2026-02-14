//! End-to-end tests for call functionality with parity verification.
//!
//! Tests cover:
//! - Device enumeration (audio/video)
//! - Device selection
//! - Call lifecycle (start, join, leave)
//! - Media controls (mute/unmute, video enable/disable)
//! - Screen sharing
//! - Recording
//! - Call history
//! - Presence indicators
//! - Quality metrics
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.
//!
//! Note: Tests that require WebRTC are marked #[ignore] as they need networking
//! infrastructure that isn't available in unit test environments.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_api::call::CallState;
use communitas_ui_service::UiServices;
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

/// Helper to create UiServices with demo authentication enabled.
async fn make_authenticated_services(temp: &TempDir) -> UiServices {
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
    let services = UiServices::new(storage, app).unwrap();

    // Enable demo mode to authenticate
    services.auth().enable_demo_mode();
    // Allow the background auth watcher to reinitialize CoreKanbanService
    // with the authenticated peer_id, preventing BoardNotFound race conditions.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    services
}

/// Helper to create a test channel entity for calls.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let messaging = services.messaging();
    let app = messaging.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Channel,
        description: Some("Test channel for call E2E tests".to_string()),
        initial_members: vec![],
    };

    let events = app.execute(cmd).await.expect("Failed to create entity");

    events
        .iter()
        .find_map(|event| match event {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .expect("No EntityCreated event returned")
}

// =============================================================================
// Test 1: List devices (audio/video enumeration)
// =============================================================================

#[test]
fn test_list_devices() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_list_devices_inner());
    });
}

async fn test_list_devices_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // List devices - should succeed even without real hardware (returns mock/empty list)
    let devices = call.list_devices().await.expect("Failed to list devices");

    // Devices list may be empty in test environment but should not error
    // Just verify the call succeeded
    let _ = devices;
}

// =============================================================================
// Test 2: Refresh devices
// =============================================================================

#[test]
fn test_refresh_devices() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_refresh_devices_inner());
    });
}

async fn test_refresh_devices_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Refresh devices
    let devices = call
        .refresh_devices()
        .await
        .expect("Failed to refresh devices");

    // Should not error
    let _ = devices;
}

// =============================================================================
// Test 3: Select audio devices
// =============================================================================

#[test]
fn test_select_audio_devices() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_select_audio_devices_inner());
    });
}

async fn test_select_audio_devices_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Try to select a fake microphone - may fail if device doesn't exist
    // but should not panic
    let mic_result = call.select_microphone("fake-mic-id").await;
    // Either succeeds or returns an error gracefully
    let _ = mic_result;

    // Try to select a fake speaker
    let speaker_result = call.select_speaker("fake-speaker-id").await;
    let _ = speaker_result;
}

// =============================================================================
// Test 4: Select camera
// =============================================================================

#[test]
fn test_select_camera() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_select_camera_inner());
    });
}

async fn test_select_camera_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Try to select a fake camera
    let result = call.select_camera("fake-camera-id").await;
    // May fail but should not panic
    let _ = result;
}

// =============================================================================
// Test 5: Start call (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_start_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_start_call_inner());
    });
}

async fn test_start_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Call Test Channel").await;

    // Start a call (entity_id, video_enabled)
    let call_info = call
        .start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    assert!(!call_info.call_id.is_empty(), "Call ID should not be empty");

    // Clean up - leave call
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 6: Join call (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_join_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_join_call_inner());
    });
}

async fn test_join_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Join Test Channel").await;

    // Start a call first
    let call_info = call
        .start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Join the call (already in it from start, but API should handle)
    let join_result = call.join_call(&call_info.call_id).await;
    // May succeed or indicate already in call
    let _ = join_result;

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 7: Leave call (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_leave_call() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_leave_call_inner());
    });
}

async fn test_leave_call_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Leave Test Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Leave the call
    call.leave_call().await.expect("Failed to leave call");

    // Verify we're no longer in a call
    let snapshot = call.subscribe().borrow().clone();
    assert!(
        matches!(snapshot.state, CallState::Idle),
        "Should not be in call after leaving"
    );
}

// =============================================================================
// Test 8: Toggle mute (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_toggle_mute() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_toggle_mute_inner());
    });
}

async fn test_toggle_mute_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Mute Test Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Toggle mute
    let muted = call.toggle_mute().await.expect("Failed to toggle mute");

    // Toggle back
    let unmuted = call
        .toggle_mute()
        .await
        .expect("Failed to toggle mute again");

    // They should be opposite values
    assert_ne!(muted, unmuted, "Toggle should change mute state");

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 9: Toggle video (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_toggle_video() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_toggle_video_inner());
    });
}

async fn test_toggle_video_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Video Test Channel").await;

    // Start a call with video disabled
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Toggle video
    let result = call.toggle_video().await;
    // May fail without camera but should not panic
    let _ = result;

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 10: Screen share start/stop (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_screen_share() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_screen_share_inner());
    });
}

async fn test_screen_share_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Screen Share Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Start screen share (may fail in headless test environment)
    let start_result = call.start_screen_share().await;
    // Either succeeds or fails gracefully
    if start_result.is_ok() {
        // Stop screen share
        call.stop_screen_share()
            .await
            .expect("Failed to stop screen share");
    }

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 11: Recording toggle (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_recording() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recording_inner());
    });
}

async fn test_recording_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Recording Test Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Start recording
    let start_result = call.start_recording(false).await;
    if start_result.is_ok() {
        // Stop recording
        let stop_result = call.stop_recording().await;
        let _ = stop_result;
        // Recording started and stopped successfully
    }

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 12: Call history (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_call_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_call_history_inner());
    });
}

async fn test_call_history_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "History Test Channel").await;

    // Start and end a call to create history
    let call_info = call
        .start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    call.leave_call().await.expect("Failed to leave call");

    // Get call history
    let history = call.get_call_history().await;

    // History may contain the call we just made
    let _ = history;

    // Get recent history
    let recent = call.get_recent_history(10).await;
    let _ = recent;

    // Get history for entity
    let entity_history = call.get_history_for_entity(&entity_id).await;
    let _ = entity_history;

    // Get specific history entry
    let entry = call.get_history_entry(&call_info.call_id).await;
    let _ = entry;
}

// =============================================================================
// Test 13: Missed calls
// =============================================================================

#[test]
fn test_missed_calls() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_missed_calls_inner());
    });
}

async fn test_missed_calls_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // Get unread missed count (should be 0 initially)
    let count = call.get_unread_missed_count().await;
    assert_eq!(count, 0, "Should have no missed calls initially");

    // Get unread missed calls
    let missed = call.get_unread_missed_calls().await;
    assert!(missed.is_empty(), "Should have no missed calls initially");

    // Has unread missed calls
    let has_unread = call.has_unread_missed_calls().await;
    assert!(!has_unread, "Should not have unread missed calls");

    // Mark all as read (should succeed even with no missed calls)
    call.mark_all_calls_read().await;

    // Acknowledge all missed calls
    call.acknowledge_all_missed_calls().await;
}

// =============================================================================
// Test 14: Call state subscription (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_call_state_subscription() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_call_state_subscription_inner());
    });
}

async fn test_call_state_subscription_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "State Test Channel").await;

    // Initial state should not be in call
    let initial_state = call.subscribe().borrow().clone();
    assert!(
        matches!(initial_state.state, CallState::Idle),
        "Should be idle initially"
    );

    // Start call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // State should now be in call
    let in_call_state = call.subscribe().borrow().clone();
    assert!(
        matches!(
            in_call_state.state,
            CallState::InCall | CallState::Connecting
        ),
        "Should be in call or connecting after starting"
    );

    // Leave call
    call.leave_call().await.expect("Failed to leave call");

    // State should be out of call
    let final_state = call.subscribe().borrow().clone();
    assert!(
        matches!(final_state.state, CallState::Idle),
        "Should not be in call after leaving"
    );
}

// =============================================================================
// Test 15: List active calls (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_list_active_calls() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_list_active_calls_inner());
    });
}

async fn test_list_active_calls_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    // List active calls (initially empty)
    let active = call
        .list_active_calls()
        .await
        .expect("Failed to list active calls");

    // Should be empty initially
    assert!(active.is_empty(), "Should have no active calls initially");

    let entity_id = create_test_entity(&services, "Active Calls Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // List active calls again
    let active_during = call
        .list_active_calls()
        .await
        .expect("Failed to list active calls");

    // Should have at least our call
    // Note: May or may not appear in list depending on implementation
    let _ = active_during;

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 16: Quality metrics (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_quality_metrics() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_quality_metrics_inner());
    });
}

async fn test_quality_metrics_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Metrics Test Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Update quality metrics
    call.update_video_quality(1920, 1080, 30, 5000).await;

    // Update bandwidth stats
    call.update_bandwidth_stats(100000, 50000).await;

    // Clear quality metrics
    call.clear_quality_metrics().await;

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 17: Hand raise (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_hand_raise() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_hand_raise_inner());
    });
}

async fn test_hand_raise_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Hand Raise Channel").await;

    // Start a call
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");

    // Raise hand
    call.set_hand_raised(true)
        .await
        .expect("Failed to raise hand");

    // Lower hand
    call.set_hand_raised(false)
        .await
        .expect("Failed to lower hand");

    // Clean up
    call.leave_call().await.expect("Failed to leave call");
}

// =============================================================================
// Test 18: Clear call history (requires WebRTC)
// =============================================================================

#[test]
#[ignore = "Requires WebRTC networking infrastructure"]
fn test_clear_call_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_clear_call_history_inner());
    });
}

async fn test_clear_call_history_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let call = services.call();

    let entity_id = create_test_entity(&services, "Clear History Channel").await;

    // Start and end a call to create history
    call.start_call(&entity_id, false)
        .await
        .expect("Failed to start call");
    call.leave_call().await.expect("Failed to leave call");

    // Clear call history
    call.clear_call_history().await;

    // History should be empty (or at least cleared - check entries)
    let history = call.get_call_history().await;
    // After clearing, entries should be empty or recently cleared
    let _ = history;
}

// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! End-to-end offline sync tests for the Communitas UI services.
//!
//! These tests verify graceful handling of offline scenarios including:
//! - Canvas offline queue operations
//! - Messaging pending message queue
//! - Drive staging queue
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::EntityType;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_ui_api::drive::DiskType;
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

/// Helper to create a test project entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let kanban = services.kanban();
    let app = kanban.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Project,
        description: Some("Test entity for offline sync tests".to_string()),
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
// Canvas Offline Queue Tests
// =============================================================================

#[test]
fn test_canvas_offline_queue_starts_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_canvas_offline_queue_starts_empty_inner());
    });
}

async fn test_canvas_offline_queue_starts_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Offline Queue Empty Test").await;

    canvas
        .load_canvas(&entity_id)
        .await
        .expect("Load canvas should succeed");

    // Verify offline queue starts empty
    let pending_count = canvas.pending_operations();
    assert_eq!(pending_count, 0, "Offline queue should start empty");

    // Verify snapshot reflects empty queue
    let snapshot = canvas.current_snapshot();
    assert_eq!(
        snapshot.offline_queue_count, 0,
        "Snapshot should show empty offline queue"
    );
}

#[test]
fn test_canvas_offline_queue_flush_on_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_canvas_offline_queue_flush_on_empty_inner());
    });
}

async fn test_canvas_offline_queue_flush_on_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Offline Flush Empty Test").await;

    canvas
        .load_canvas(&entity_id)
        .await
        .expect("Load canvas should succeed");

    // Flush empty queue - should not crash
    let result = canvas.flush_queue().await;

    // Should succeed with no operations flushed
    assert!(
        result.is_ok(),
        "Flushing empty queue should succeed: {:?}",
        result.err()
    );
}

#[test]
fn test_canvas_undo_redo_empty_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_canvas_undo_redo_empty_history_inner());
    });
}

async fn test_canvas_undo_redo_empty_history_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Empty History Test").await;

    canvas
        .load_canvas(&entity_id)
        .await
        .expect("Load canvas should succeed");

    // Verify can_undo and can_redo are false on empty canvas
    assert!(!canvas.can_undo(), "Cannot undo with empty history");
    assert!(!canvas.can_redo(), "Cannot redo with empty history");

    // Undo/redo on empty history should return Ok(None), not crash
    let undo_result = canvas.undo().await;
    assert!(undo_result.is_ok(), "Undo on empty history should succeed");
    assert!(
        undo_result.unwrap().is_none(),
        "Undo on empty history should return None"
    );

    let redo_result = canvas.redo().await;
    assert!(redo_result.is_ok(), "Redo on empty history should succeed");
    assert!(
        redo_result.unwrap().is_none(),
        "Redo on empty history should return None"
    );
}

// =============================================================================
// Messaging Pending Message Queue Tests
// =============================================================================

#[test]
fn test_messaging_pending_queue_starts_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_pending_queue_starts_empty_inner());
    });
}

async fn test_messaging_pending_queue_starts_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Verify pending messages start empty
    let snapshot = messaging.current_snapshot();
    assert!(
        snapshot.pending_messages.is_empty(),
        "Pending message queue should start empty"
    );
}

#[test]
fn test_messaging_queue_message_offline() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_queue_message_offline_inner());
    });
}

async fn test_messaging_queue_message_offline_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Message Queue Test").await;

    // Queue a message for later sending
    messaging.queue_message(&entity_id, "Test offline message", None);

    // Verify message is in pending queue
    let snapshot = messaging.current_snapshot();
    assert_eq!(
        snapshot.pending_messages.len(),
        1,
        "Should have one pending message"
    );
    assert_eq!(
        snapshot.pending_messages[0].text, "Test offline message",
        "Pending message content should match"
    );
}

#[test]
fn test_messaging_remove_pending_message() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_remove_pending_message_inner());
    });
}

async fn test_messaging_remove_pending_message_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Remove Pending Test").await;

    // Queue a message
    messaging.queue_message(&entity_id, "Message to remove", None);

    // Get the pending message ID
    let snapshot = messaging.current_snapshot();
    let pending_id = snapshot.pending_messages[0].id.clone();

    // Remove the pending message
    let removed = messaging.remove_pending_message(&pending_id);
    assert!(removed, "Should successfully remove pending message");

    // Verify queue is now empty
    let snapshot = messaging.current_snapshot();
    assert!(
        snapshot.pending_messages.is_empty(),
        "Pending queue should be empty after removal"
    );
}

#[test]
fn test_messaging_remove_nonexistent_pending() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_remove_nonexistent_pending_inner());
    });
}

async fn test_messaging_remove_nonexistent_pending_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Try to remove a nonexistent pending message
    let removed = messaging.remove_pending_message("nonexistent-id");
    assert!(
        !removed,
        "Removing nonexistent pending message should return false"
    );
}

#[test]
fn test_messaging_multiple_pending_messages() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_multiple_pending_messages_inner());
    });
}

async fn test_messaging_multiple_pending_messages_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Multiple Pending Test").await;

    // Queue multiple messages
    messaging.queue_message(&entity_id, "Message 1", None);
    messaging.queue_message(&entity_id, "Message 2", None);
    messaging.queue_message(&entity_id, "Message 3", None);

    // Verify all messages are in queue
    let snapshot = messaging.current_snapshot();
    assert_eq!(
        snapshot.pending_messages.len(),
        3,
        "Should have three pending messages"
    );
}

// =============================================================================
// Drive Staging Queue Tests
// =============================================================================

#[test]
fn test_drive_staging_queue_starts_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_staging_queue_starts_empty_inner());
    });
}

async fn test_drive_staging_queue_starts_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // List staged uploads - should be empty
    let staged = drive
        .list_staged_uploads()
        .await
        .expect("List staged uploads should succeed");

    assert!(staged.is_empty(), "Staging queue should start empty");
}

#[test]
fn test_drive_staging_status_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_staging_status_empty_inner());
    });
}

async fn test_drive_staging_status_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Get staging status for empty queue
    let status = drive
        .get_staging_status()
        .await
        .expect("Get staging status should succeed");

    assert_eq!(
        status.total_files, 0,
        "Empty staging queue should have 0 total files"
    );
    assert_eq!(
        status.pending_files, 0,
        "Empty staging queue should have 0 pending files"
    );
    assert!(!status.is_syncing, "Should not be syncing when empty");
}

#[test]
fn test_drive_clear_empty_staging_queue() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_clear_empty_staging_queue_inner());
    });
}

async fn test_drive_clear_empty_staging_queue_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Clear empty staging queue - should not crash
    let cleared = drive
        .clear_staging_queue(false)
        .await
        .expect("Clear staging queue should succeed");

    assert_eq!(cleared, 0, "Should clear 0 items from empty queue");
}

#[test]
fn test_drive_sync_empty_staging_queue() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_sync_empty_staging_queue_inner());
    });
}

async fn test_drive_sync_empty_staging_queue_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Sync empty staging queue - should not crash
    let (succeeded, failed) = drive
        .sync_staging_queue()
        .await
        .expect("Sync staging queue should succeed");

    assert_eq!(
        succeeded, 0,
        "Should have 0 successful syncs on empty queue"
    );
    assert_eq!(failed, 0, "Should have 0 failed syncs on empty queue");
}

#[test]
fn test_drive_get_nonexistent_staged_upload() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_get_nonexistent_staged_upload_inner());
    });
}

async fn test_drive_get_nonexistent_staged_upload_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Try to get a nonexistent staged upload
    let result = drive.get_staged_upload("nonexistent-id").await;

    assert!(
        result.is_err(),
        "Getting nonexistent staged upload should fail"
    );
}

#[test]
fn test_drive_remove_nonexistent_staged_upload() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_remove_nonexistent_staged_upload_inner());
    });
}

async fn test_drive_remove_nonexistent_staged_upload_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Try to remove a nonexistent staged upload
    let result = drive.remove_staged_upload("nonexistent-id").await;

    assert!(
        result.is_err(),
        "Removing nonexistent staged upload should fail"
    );
}

#[test]
fn test_drive_stage_upload_creates_entry() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_stage_upload_creates_entry_inner());
    });
}

async fn test_drive_stage_upload_creates_entry_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Stage Upload Test").await;

    // Create a local file to stage
    let local_file = temp.path().join("test_file.txt");
    std::fs::write(&local_file, "Test file content").expect("Failed to write test file");

    // Stage the upload
    let staged = drive
        .stage_upload(
            &entity_id,
            DiskType::Private,
            "/test_file.txt",
            local_file.to_str().unwrap(),
        )
        .await
        .expect("Stage upload should succeed");

    assert!(!staged.id.is_empty(), "Staged upload should have an ID");
    assert_eq!(
        staged.entity_id, entity_id,
        "Staged upload should reference correct entity"
    );

    // Verify it appears in the list
    let all_staged = drive
        .list_staged_uploads()
        .await
        .expect("List staged uploads should succeed");

    assert_eq!(all_staged.len(), 1, "Should have one staged upload");
    assert_eq!(all_staged[0].id, staged.id, "Listed upload should match");
}

#[test]
fn test_drive_remove_staged_upload() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_remove_staged_upload_inner());
    });
}

async fn test_drive_remove_staged_upload_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Remove Staged Test").await;

    // Create and stage a file
    let local_file = temp.path().join("remove_test.txt");
    std::fs::write(&local_file, "Content to remove").expect("Failed to write test file");

    let staged = drive
        .stage_upload(
            &entity_id,
            DiskType::Private,
            "/remove_test.txt",
            local_file.to_str().unwrap(),
        )
        .await
        .expect("Stage upload should succeed");

    // Remove the staged upload
    drive
        .remove_staged_upload(&staged.id)
        .await
        .expect("Remove staged upload should succeed");

    // Verify the list is empty
    let all_staged = drive
        .list_staged_uploads()
        .await
        .expect("List staged uploads should succeed");

    assert!(
        all_staged.is_empty(),
        "Staging queue should be empty after removal"
    );
}

// =============================================================================
// Cross-Service Offline Consistency Tests
// =============================================================================

#[test]
fn test_snapshot_consistency_during_offline() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_snapshot_consistency_during_offline_inner());
    });
}

async fn test_snapshot_consistency_during_offline_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;

    let entity_id = create_test_entity(&services, "Offline Consistency Test").await;

    // Perform operations across multiple services
    let messaging = services.messaging();
    let canvas = services.canvas();

    // Queue a message
    messaging.queue_message(&entity_id, "Offline message", None);

    // Load canvas
    canvas
        .load_canvas(&entity_id)
        .await
        .expect("Load canvas should succeed");

    // Verify snapshots are consistent
    let msg_snapshot = messaging.current_snapshot();
    let canvas_snapshot = canvas.current_snapshot();

    // Both snapshots should be accessible without errors
    assert_eq!(
        msg_snapshot.pending_messages.len(),
        1,
        "Messaging snapshot should reflect pending message"
    );
    assert!(
        canvas_snapshot.elements.is_empty(),
        "Canvas snapshot should be empty"
    );

    // Services should remain independent
    assert!(
        !msg_snapshot.loading,
        "Messaging should not be in loading state"
    );
}

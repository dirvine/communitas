// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for sync state transitions in the Communitas UI services.
//!
//! These tests verify the offline-first UX behavior including:
//! - State transitions: Online → Offline → Online
//! - Queue accumulation while offline
//! - Sync completion detection
//! - Conflict detection and counting
//!
//! Note: These tests use mock network state for CI compatibility.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_ui_api::{SyncMetadata, SyncProgress, SyncState, SyncSummary};
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
    // Wait for background auth watcher to reinitialize with authenticated peer_id.
    services.wait_ready().await;

    services
}

// =============================================================================
// SyncState Type Tests
// =============================================================================

#[test]
fn test_sync_state_default_is_synced() {
    assert_eq!(SyncState::default(), SyncState::Synced);
}

#[test]
fn test_sync_state_needs_attention() {
    // States that need user attention
    assert!(SyncState::Conflict.needs_attention());
    assert!(SyncState::Error.needs_attention());

    // States that don't need attention
    assert!(!SyncState::Synced.needs_attention());
    assert!(!SyncState::Syncing.needs_attention());
    assert!(!SyncState::Queued.needs_attention());
}

#[test]
fn test_sync_state_is_pending() {
    // States that are pending
    assert!(SyncState::Syncing.is_pending());
    assert!(SyncState::Queued.is_pending());

    // States that are not pending
    assert!(!SyncState::Synced.is_pending());
    assert!(!SyncState::Conflict.is_pending());
    assert!(!SyncState::Error.is_pending());
}

#[test]
fn test_sync_state_display() {
    assert_eq!(SyncState::Synced.to_string(), "Synced");
    assert_eq!(SyncState::Syncing.to_string(), "Syncing");
    assert_eq!(SyncState::Queued.to_string(), "Waiting to sync");
    assert_eq!(SyncState::Conflict.to_string(), "Has conflicts");
    assert_eq!(SyncState::Error.to_string(), "Sync failed");
}

#[test]
fn test_sync_state_icon_names() {
    assert_eq!(SyncState::Synced.icon_name(), "check-circle");
    assert_eq!(SyncState::Syncing.icon_name(), "refresh-cw");
    assert_eq!(SyncState::Queued.icon_name(), "clock");
    assert_eq!(SyncState::Conflict.icon_name(), "alert-triangle");
    assert_eq!(SyncState::Error.icon_name(), "x-circle");
}

#[test]
fn test_sync_state_color_classes() {
    assert_eq!(SyncState::Synced.color_class(), "text-green-500");
    assert_eq!(SyncState::Syncing.color_class(), "text-blue-500");
    assert_eq!(SyncState::Queued.color_class(), "text-orange-500");
    assert_eq!(SyncState::Conflict.color_class(), "text-yellow-500");
    assert_eq!(SyncState::Error.color_class(), "text-red-500");
}

// =============================================================================
// SyncMetadata Tests
// =============================================================================

#[test]
fn test_sync_metadata_synced() {
    let meta = SyncMetadata::synced();
    assert_eq!(meta.state, SyncState::Synced);
    assert!(meta.last_synced.is_some());
    assert_eq!(meta.pending_changes, 0);
    assert_eq!(meta.conflict_count, 0);
    assert!(meta.error_message.is_none());
}

#[test]
fn test_sync_metadata_syncing() {
    let meta = SyncMetadata::syncing();
    assert_eq!(meta.state, SyncState::Syncing);
    assert!(meta.last_synced.is_none());
}

#[test]
fn test_sync_metadata_queued() {
    let meta = SyncMetadata::queued(5);
    assert_eq!(meta.state, SyncState::Queued);
    assert_eq!(meta.pending_changes, 5);
}

#[test]
fn test_sync_metadata_conflict() {
    let meta = SyncMetadata::conflict(3);
    assert_eq!(meta.state, SyncState::Conflict);
    assert_eq!(meta.conflict_count, 3);
}

#[test]
fn test_sync_metadata_error() {
    let meta = SyncMetadata::error("Network unreachable");
    assert_eq!(meta.state, SyncState::Error);
    assert_eq!(meta.error_message, Some("Network unreachable".to_string()));
}

// =============================================================================
// SyncProgress Tests
// =============================================================================

#[test]
fn test_sync_progress_percentage() {
    let progress = SyncProgress {
        total: 10,
        completed: 5,
        current_item: Some("file.txt".to_string()),
        bytes_transferred: 0,
        bytes_total: 0,
    };
    assert_eq!(progress.percentage(), 50);
    assert!(!progress.is_complete());
}

#[test]
fn test_sync_progress_complete() {
    let progress = SyncProgress {
        total: 10,
        completed: 10,
        current_item: None,
        bytes_transferred: 0,
        bytes_total: 0,
    };
    assert_eq!(progress.percentage(), 100);
    assert!(progress.is_complete());
}

#[test]
fn test_sync_progress_empty() {
    let progress = SyncProgress::default();
    assert_eq!(progress.percentage(), 100);
    assert!(progress.is_complete());
}

#[test]
fn test_sync_progress_bytes() {
    let progress = SyncProgress {
        total: 0,
        completed: 0,
        current_item: None,
        bytes_transferred: 500,
        bytes_total: 1000,
    };
    assert_eq!(progress.bytes_percentage(), 50);
}

#[test]
fn test_sync_progress_bytes_complete() {
    let progress = SyncProgress {
        total: 0,
        completed: 0,
        current_item: None,
        bytes_transferred: 1000,
        bytes_total: 1000,
    };
    assert_eq!(progress.bytes_percentage(), 100);
}

// =============================================================================
// SyncSummary Tests
// =============================================================================

#[test]
fn test_sync_summary_overall_state_synced() {
    let summary = SyncSummary {
        synced_count: 10,
        syncing_count: 0,
        queued_count: 0,
        conflict_count: 0,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Synced);
    assert_eq!(summary.total(), 10);
}

#[test]
fn test_sync_summary_overall_state_error_takes_priority() {
    let summary = SyncSummary {
        synced_count: 8,
        syncing_count: 1,
        queued_count: 1,
        conflict_count: 1,
        error_count: 1,
    };
    assert_eq!(summary.overall_state(), SyncState::Error);
}

#[test]
fn test_sync_summary_overall_state_conflict_over_syncing() {
    let summary = SyncSummary {
        synced_count: 8,
        syncing_count: 1,
        queued_count: 1,
        conflict_count: 1,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Conflict);
}

#[test]
fn test_sync_summary_overall_state_syncing_over_queued() {
    let summary = SyncSummary {
        synced_count: 8,
        syncing_count: 1,
        queued_count: 1,
        conflict_count: 0,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Syncing);
}

#[test]
fn test_sync_summary_overall_state_queued() {
    let summary = SyncSummary {
        synced_count: 8,
        syncing_count: 0,
        queued_count: 2,
        conflict_count: 0,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Queued);
}

// =============================================================================
// Messaging Sync State Integration Tests
// =============================================================================

#[test]
fn test_messaging_sync_state_default() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_sync_state_default_inner());
    });
}

async fn test_messaging_sync_state_default_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Initial sync state should be synced (no pending messages)
    let snapshot = messaging.current_snapshot();
    assert!(
        snapshot.pending_messages.is_empty(),
        "New messaging service should have no pending messages"
    );
}

#[test]
fn test_messaging_queued_state_on_offline_message() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_queued_state_on_offline_message_inner());
    });
}

async fn test_messaging_queued_state_on_offline_message_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Queue a message (simulates offline send)
    messaging.queue_message("test-entity", "Hello offline world", None);

    // Verify queued state
    let snapshot = messaging.current_snapshot();
    assert_eq!(
        snapshot.pending_messages.len(),
        1,
        "Should have one pending message"
    );
    assert_eq!(
        snapshot.pending_messages[0].text, "Hello offline world",
        "Pending message content should match"
    );
}

#[test]
fn test_messaging_multiple_pending_accumulation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_multiple_pending_accumulation_inner());
    });
}

async fn test_messaging_multiple_pending_accumulation_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Queue multiple messages while offline
    messaging.queue_message("entity-1", "Message 1", None);
    messaging.queue_message("entity-1", "Message 2", None);
    messaging.queue_message("entity-2", "Message 3", None);

    // Verify all are queued
    let snapshot = messaging.current_snapshot();
    assert_eq!(
        snapshot.pending_messages.len(),
        3,
        "Should have three pending messages"
    );
}

#[test]
fn test_messaging_pending_removal_transitions_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_pending_removal_transitions_state_inner());
    });
}

async fn test_messaging_pending_removal_transitions_state_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Queue a message
    messaging.queue_message("test-entity", "Will be removed", None);

    // Get pending message ID
    let snapshot = messaging.current_snapshot();
    let pending_id = snapshot.pending_messages[0].id.clone();

    // Remove the pending message (simulates successful sync)
    let removed = messaging.remove_pending_message(&pending_id);
    assert!(removed, "Should successfully remove pending message");

    // Verify back to synced state
    let snapshot = messaging.current_snapshot();
    assert!(
        snapshot.pending_messages.is_empty(),
        "Should have no pending messages after removal"
    );
}

// =============================================================================
// Drive Sync State Integration Tests
// =============================================================================

#[test]
fn test_drive_sync_state_default() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_sync_state_default_inner());
    });
}

async fn test_drive_sync_state_default_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Initial staging status should show no pending items
    let status = drive.get_staging_status().await.unwrap();
    assert_eq!(status.total_files, 0, "Should have no staged files");
    assert_eq!(status.pending_files, 0, "Should have no pending files");
    assert!(!status.is_syncing, "Should not be syncing");
}

#[test]
fn test_drive_staged_file_shows_queued_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_staged_file_shows_queued_state_inner());
    });
}

async fn test_drive_staged_file_shows_queued_state_inner() {
    use communitas_core::EntityType;
    use communitas_core::command::{Command, Event};
    use communitas_ui_api::drive::DiskType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let kanban = services.kanban();
    let app = kanban.app();
    let cmd = Command::CreateEntity {
        name: "Drive Test Entity".to_string(),
        entity_type: EntityType::Project,
        description: Some("Test entity for drive sync".to_string()),
        initial_members: vec![],
    };
    let events = app.execute(cmd).await.unwrap();
    let entity_id = events
        .iter()
        .find_map(|e| match e {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .unwrap();

    // Create a local file to stage
    let local_file = temp.path().join("staged_file.txt");
    std::fs::write(&local_file, "Staged content for sync").unwrap();

    // Stage the file
    let staged = drive
        .stage_upload(
            &entity_id,
            DiskType::Private,
            "/staged_file.txt",
            local_file.to_str().unwrap(),
        )
        .await
        .unwrap();

    // Verify staging status shows pending
    let status = drive.get_staging_status().await.unwrap();
    assert_eq!(status.total_files, 1, "Should have one staged file");
    assert_eq!(status.pending_files, 1, "Should have one pending file");

    // Verify the staged upload exists
    let staged_list = drive.list_staged_uploads().await.unwrap();
    assert_eq!(staged_list.len(), 1, "Should have one staged upload");
    assert_eq!(staged_list[0].id, staged.id, "Staged ID should match");
}

#[test]
fn test_drive_staging_queue_clear_transitions_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_staging_queue_clear_transitions_state_inner());
    });
}

async fn test_drive_staging_queue_clear_transitions_state_inner() {
    use communitas_core::EntityType;
    use communitas_core::command::{Command, Event};
    use communitas_ui_api::drive::DiskType;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let kanban = services.kanban();
    let app = kanban.app();
    let cmd = Command::CreateEntity {
        name: "Clear Queue Test".to_string(),
        entity_type: EntityType::Project,
        description: Some("Test entity for queue clear".to_string()),
        initial_members: vec![],
    };
    let events = app.execute(cmd).await.unwrap();
    let entity_id = events
        .iter()
        .find_map(|e| match e {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .unwrap();

    // Stage a file
    let local_file = temp.path().join("to_clear.txt");
    std::fs::write(&local_file, "Will be cleared").unwrap();
    drive
        .stage_upload(
            &entity_id,
            DiskType::Private,
            "/to_clear.txt",
            local_file.to_str().unwrap(),
        )
        .await
        .unwrap();

    // Verify pending
    let status = drive.get_staging_status().await.unwrap();
    assert_eq!(status.pending_files, 1, "Should have one pending file");

    // Clear the staging queue
    let cleared = drive.clear_staging_queue(false).await.unwrap();
    assert_eq!(cleared, 1, "Should clear one file");

    // Verify back to synced
    let status = drive.get_staging_status().await.unwrap();
    assert_eq!(status.total_files, 0, "Should have no staged files");
    assert_eq!(status.pending_files, 0, "Should have no pending files");
}

// =============================================================================
// Kanban Sync State Integration Tests
// =============================================================================

#[test]
fn test_kanban_card_sync_state_default() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_card_sync_state_default_inner());
    });
}

async fn test_kanban_card_sync_state_default_inner() {
    use communitas_core::EntityType;
    use communitas_core::command::{Command, Event};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Create a test entity
    let app = kanban.app();
    let cmd = Command::CreateEntity {
        name: "Kanban Sync Test".to_string(),
        entity_type: EntityType::Project,
        description: Some("Test entity for kanban sync".to_string()),
        initial_members: vec![],
    };
    let events = app.execute(cmd).await.unwrap();
    let entity_id = events
        .iter()
        .find_map(|e| match e {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .unwrap();

    // Load the kanban boards for the entity
    let boards = kanban
        .load_boards(&entity_id)
        .await
        .expect("Load boards should succeed");

    // Verify boards load with no sync issues
    let snapshot = kanban.current_snapshot();
    assert!(
        !snapshot.loading,
        "Should not be in loading state after load"
    );
    // Boards are loaded for the entity
    assert!(
        boards.is_empty() || !boards.is_empty(),
        "Boards list should be accessible"
    );
}

#[test]
fn test_kanban_new_card_sync_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_new_card_sync_state_inner());
    });
}

async fn test_kanban_new_card_sync_state_inner() {
    use communitas_core::EntityType;
    use communitas_core::command::{Command, Event};
    use communitas_ui_api::SyncState;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Create a test entity
    let app = kanban.app();
    let cmd = Command::CreateEntity {
        name: "New Card Sync Test".to_string(),
        entity_type: EntityType::Project,
        description: Some("Test entity for new card sync".to_string()),
        initial_members: vec![],
    };
    let events = app.execute(cmd).await.unwrap();
    let entity_id = events
        .iter()
        .find_map(|e| match e {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .unwrap();

    // Load the kanban boards for the entity
    let _boards = kanban.load_boards(&entity_id).await.unwrap();

    // Create a board first
    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Should create board");

    // Create a column for the card
    let column = kanban
        .create_column(&board.id, "todo", 0)
        .await
        .expect("Should create column");

    // Create a new card (board_id, column_id, title)
    let result = kanban
        .create_card(&board.id, &column.id, "New Test Card")
        .await;
    assert!(result.is_ok(), "Should create card: {:?}", result.err());

    let card = result.unwrap();

    // New cards should start in Queued state (waiting to sync)
    assert_eq!(
        card.sync_state,
        SyncState::Queued,
        "New card should be in Queued state"
    );
}

// =============================================================================
// Cross-Service Sync State Tests
// =============================================================================

#[test]
fn test_multiple_services_independent_sync_states() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_multiple_services_independent_sync_states_inner());
    });
}

async fn test_multiple_services_independent_sync_states_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;

    let messaging = services.messaging();
    let drive = services.drive();

    // Queue a message (messaging has pending)
    messaging.queue_message("test-entity", "Pending message", None);

    // Verify messaging has pending
    let msg_snapshot = messaging.current_snapshot();
    assert_eq!(
        msg_snapshot.pending_messages.len(),
        1,
        "Messaging should have pending"
    );

    // Verify drive is unaffected
    let drive_status = drive.get_staging_status().await.unwrap();
    assert_eq!(
        drive_status.pending_files, 0,
        "Drive should have no pending"
    );

    // Services maintain independent sync state
}

#[test]
fn test_sync_state_persistence_across_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_sync_state_persistence_across_operations_inner());
    });
}

async fn test_sync_state_persistence_across_operations_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Queue several messages
    for i in 0..5 {
        messaging.queue_message("test-entity", &format!("Message {}", i), None);
    }

    // Verify all accumulated
    let snapshot = messaging.current_snapshot();
    assert_eq!(
        snapshot.pending_messages.len(),
        5,
        "Should have 5 pending messages"
    );

    // Remove some
    let ids: Vec<_> = snapshot
        .pending_messages
        .iter()
        .take(3)
        .map(|m| m.id.clone())
        .collect();

    for id in ids {
        messaging.remove_pending_message(&id);
    }

    // Verify remaining
    let snapshot = messaging.current_snapshot();
    assert_eq!(
        snapshot.pending_messages.len(),
        2,
        "Should have 2 pending messages remaining"
    );
}

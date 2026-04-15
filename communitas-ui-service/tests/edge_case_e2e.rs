// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end edge case tests for the Communitas UI services.
//!
//! These tests verify graceful handling of edge cases like empty lists,
//! not-found errors, invalid inputs, and boundary conditions across
//! all major service modules.
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
    // Wait for background auth watcher to reinitialize with authenticated peer_id.
    services.wait_ready().await;

    services
}

/// Helper to create a test project entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let kanban = services.kanban();
    let app = kanban.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Project,
        description: Some("Test entity for edge case tests".to_string()),
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
// Test 1: Kanban - Get Non-Existent Board
// =============================================================================

#[test]
fn test_kanban_get_nonexistent_board() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_get_nonexistent_board_inner());
    });
}

async fn test_kanban_get_nonexistent_board_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Try to get a board that doesn't exist
    let result = kanban.get_board("nonexistent-board-id").await;

    // Should return an error, not panic
    assert!(result.is_err(), "Getting nonexistent board should fail");
}

// =============================================================================
// Test 2: Kanban - Get Non-Existent Card
// =============================================================================

#[test]
fn test_kanban_get_nonexistent_card() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_get_nonexistent_card_inner());
    });
}

async fn test_kanban_get_nonexistent_card_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Edge Case Test").await;

    // Create a board but not a card
    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    // Try to get a card that doesn't exist
    let result = kanban.get_card(&board.id, "nonexistent-card-id").await;

    assert!(result.is_err(), "Getting nonexistent card should fail");
}

// =============================================================================
// Test 3: Kanban - Empty Board View
// =============================================================================

#[test]
fn test_kanban_empty_board() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_empty_board_inner());
    });
}

async fn test_kanban_empty_board_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Empty Board Test").await;

    // Create an empty board (no columns)
    let board = kanban
        .create_board(&entity_id, "Empty Board", None)
        .await
        .expect("Failed to create board");

    // Get board view
    let board_view = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get board");

    assert!(
        board_view.columns.is_empty(),
        "New board should have no columns"
    );
}

// =============================================================================
// Test 4: Kanban - Move Card to Invalid Column
// =============================================================================

#[test]
fn test_kanban_move_card_invalid_column() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_move_card_invalid_column_inner());
    });
}

async fn test_kanban_move_card_invalid_column_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Move Card Invalid Test").await;

    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Column", 0)
        .await
        .expect("Failed to create column");

    let card = kanban
        .create_card(&board.id, &column.id, "Test Card")
        .await
        .expect("Failed to create card");

    // Try to move card to nonexistent column
    let result = kanban
        .move_card(&board.id, &card.id, "nonexistent-column", 0)
        .await;

    assert!(result.is_err(), "Moving card to invalid column should fail");
}

// =============================================================================
// Test 5: Kanban - Analytics on Empty Board
// =============================================================================

#[test]
fn test_kanban_analytics_empty_board() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_analytics_empty_board_inner());
    });
}

async fn test_kanban_analytics_empty_board_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Analytics Empty Test").await;

    let board = kanban
        .create_board(&entity_id, "Empty Analytics Board", None)
        .await
        .expect("Failed to create board");

    // Get analytics for empty board - should not crash
    let analytics = kanban
        .get_board_analytics(&board.id)
        .await
        .expect("Analytics should work on empty board");

    assert_eq!(analytics.total_active_cards, 0);
    assert!(analytics.column_card_counts.is_empty());
}

// =============================================================================
// Test 6: Messaging - Empty Thread List
// =============================================================================

#[test]
fn test_messaging_empty_thread_list() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_messaging_empty_thread_list_inner());
    });
}

async fn test_messaging_empty_thread_list_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create an entity but don't create any threads
    let _entity_id = create_test_entity(&services, "Empty Threads Test").await;

    // List threads - should succeed but be empty for new user
    let threads = messaging
        .list_threads()
        .await
        .expect("List threads should succeed");

    // A new user/entity with no messaging should have empty threads
    // (threads may or may not be empty depending on implementation)
    // Key point: should not crash or error on empty state
    let _ = threads; // Verify we can access the result

    // Also verify snapshot is accessible
    let snapshot = messaging.current_snapshot();
    assert!(
        !snapshot.loading,
        "Should not be loading after list completes"
    );
}

// =============================================================================
// Test 7: Canvas - Operations on Empty Canvas
// =============================================================================

#[test]
fn test_canvas_empty_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_canvas_empty_operations_inner());
    });
}

async fn test_canvas_empty_operations_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Empty Canvas Test").await;

    canvas
        .load_canvas(&entity_id)
        .await
        .expect("Load canvas should succeed");

    // Get snapshot - should be empty
    let snapshot = canvas.current_snapshot();
    assert!(snapshot.elements.is_empty(), "Canvas should start empty");

    // Undo on empty canvas - should not crash (may return error or Ok)
    let _ = canvas.undo().await;
    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Canvas still empty after undo"
    );

    // Redo on empty canvas - should not crash (may return error or Ok)
    let _ = canvas.redo().await;
    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Canvas still empty after redo"
    );
}

// =============================================================================
// Test 8: Drive - Empty File List
// =============================================================================

#[test]
fn test_drive_empty_file_list() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_drive_empty_file_list_inner());
    });
}

async fn test_drive_empty_file_list_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Empty Drive Test").await;

    // List directory for entity with no files
    let entries = drive
        .list_directory(&entity_id, DiskType::Private, "/")
        .await
        .expect("List directory should succeed");

    // New entity should have empty root directory
    assert!(
        entries.is_empty(),
        "Drive root should have no entries for new entity"
    );

    // Also verify snapshot is accessible
    let snapshot = drive.current_snapshot();
    assert!(
        !snapshot.loading,
        "Should not be loading after list completes"
    );
}

// =============================================================================
// Test 9: Auth - Unauthenticated Access
// =============================================================================

#[test]
fn test_auth_unauthenticated_access() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_auth_unauthenticated_access_inner());
    });
}

async fn test_auth_unauthenticated_access_inner() {
    let temp = TempDir::new().unwrap();
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

    // DO NOT enable demo mode - test unauthenticated access
    let kanban = services.kanban();

    // Creating a board without authentication should fail
    let result = kanban.create_board("some-entity", "Test Board", None).await;

    assert!(
        result.is_err(),
        "Unauthenticated board creation should fail"
    );
}

// =============================================================================
// Test 10: Kanban - Double Archive
// =============================================================================

#[test]
fn test_kanban_double_archive() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_double_archive_inner());
    });
}

async fn test_kanban_double_archive_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Double Archive Test").await;

    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Column", 0)
        .await
        .expect("Failed to create column");

    let card = kanban
        .create_card(&board.id, &column.id, "Card to archive")
        .await
        .expect("Failed to create card");

    // Archive once
    kanban
        .archive_card(&board.id, &card.id)
        .await
        .expect("First archive should succeed");

    // Archive again - should not crash
    let result = kanban.archive_card(&board.id, &card.id).await;

    // Double archive might succeed or fail gracefully depending on implementation
    // Key is it shouldn't panic - either Ok or Err is acceptable
    let _ = result;
}

// =============================================================================
// Test 11: Kanban - Update Card with Empty Values
// =============================================================================

#[test]
fn test_kanban_update_empty_values() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_update_empty_values_inner());
    });
}

async fn test_kanban_update_empty_values_inner() {
    use communitas_ui_service::kanban::CardUpdate;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Empty Update Test").await;

    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Column", 0)
        .await
        .expect("Failed to create column");

    let card = kanban
        .create_card(&board.id, &column.id, "Original Title")
        .await
        .expect("Failed to create card");

    // Update with empty title
    let result = kanban
        .update_card(
            &board.id,
            &card.id,
            CardUpdate {
                title: Some(String::new()), // Empty title
                ..Default::default()
            },
        )
        .await;

    // Should either accept empty title or reject gracefully
    if let Ok(updated) = result {
        // If accepted, verify the change took effect
        assert!(updated.title.is_empty() || updated.title == "Original Title");
    }
    // Err case: OK to reject empty title
}

// =============================================================================
// Test 12: Kanban - Very Long Title
// =============================================================================

#[test]
fn test_kanban_very_long_title() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_very_long_title_inner());
    });
}

async fn test_kanban_very_long_title_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Long Title Test").await;

    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Column", 0)
        .await
        .expect("Failed to create column");

    // Create card with very long title (10KB)
    let long_title = "A".repeat(10_000);

    let result = kanban.create_card(&board.id, &column.id, &long_title).await;

    // Should either accept or reject gracefully
    if let Ok(card) = result {
        // If accepted, verify we can retrieve it
        let retrieved = kanban.get_card(&board.id, &card.id).await;
        assert!(retrieved.is_ok(), "Should be able to retrieve card");
    }
    // Err case: OK to reject very long title
}

// =============================================================================
// Test 13: Canvas - Delete Nonexistent Element
// =============================================================================

#[test]
fn test_canvas_delete_nonexistent() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_canvas_delete_nonexistent_inner());
    });
}

async fn test_canvas_delete_nonexistent_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Delete Nonexistent Test").await;

    canvas
        .load_canvas(&entity_id)
        .await
        .expect("Load canvas should succeed");

    // Try to delete a nonexistent element using a fake UUID
    let fake_id = "00000000-0000-0000-0000-000000000099";
    let result = canvas.remove_element(None, fake_id).await;

    // Should fail gracefully with error (ElementNotFound), not panic
    assert!(
        result.is_err(),
        "Removing nonexistent element should fail gracefully"
    );

    // Canvas should still be empty and consistent
    let snapshot = canvas.current_snapshot();
    assert!(snapshot.elements.is_empty(), "Canvas should still be empty");
}

// =============================================================================
// Test 14: Concurrent Board Access
// =============================================================================

#[test]
fn test_kanban_concurrent_access() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_kanban_concurrent_access_inner());
    });
}

async fn test_kanban_concurrent_access_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Concurrent Access Test").await;

    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Column", 0)
        .await
        .expect("Failed to create column");

    // Create multiple cards concurrently using tokio::join!
    let (r0, r1, r2, r3, r4) = tokio::join!(
        kanban.create_card(&board.id, &column.id, "Card 0"),
        kanban.create_card(&board.id, &column.id, "Card 1"),
        kanban.create_card(&board.id, &column.id, "Card 2"),
        kanban.create_card(&board.id, &column.id, "Card 3"),
        kanban.create_card(&board.id, &column.id, "Card 4")
    );

    // All should succeed
    let results = [r0, r1, r2, r3, r4];
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    assert_eq!(success_count, 5, "All concurrent creates should succeed");

    // Verify all cards exist in the board
    let board_view = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get board");

    let total_cards: usize = board_view.columns.iter().map(|c| c.cards.len()).sum();
    assert_eq!(total_cards, 5, "Board should have 5 cards");
}

// =============================================================================
// Test 15: Snapshot Consistency After Errors
// =============================================================================

#[test]
fn test_snapshot_consistency_after_error() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_snapshot_consistency_after_error_inner());
    });
}

async fn test_snapshot_consistency_after_error_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Consistency Test").await;

    let board = kanban
        .create_board(&entity_id, "Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Column", 0)
        .await
        .expect("Failed to create column");

    // Create a valid card
    let card = kanban
        .create_card(&board.id, &column.id, "Valid Card")
        .await
        .expect("Failed to create card");

    // Try an invalid operation
    let _ = kanban
        .move_card(&board.id, &card.id, "nonexistent-column", 0)
        .await;

    // Verify the card is still in its original column
    let board_view = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get board");

    let cards_in_column: Vec<_> = board_view
        .columns
        .iter()
        .find(|c| c.id == column.id)
        .map(|c| c.cards.iter().map(|card| card.id.as_str()).collect())
        .unwrap_or_default();

    assert!(
        cards_in_column.contains(&card.id.as_str()),
        "Card should still be in original column after failed move"
    );
}

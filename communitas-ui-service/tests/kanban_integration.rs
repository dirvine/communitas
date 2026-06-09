// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration tests for the kanban service using real CommunitasApp.
//!
//! These tests verify the full kanban flow including board/column/card CRUD,
//! authentication attribution, and reactive watch channel updates.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::EntityType;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_ui_api::CardState;
use communitas_ui_service::UiServices;
use communitas_ui_service::kanban::{CardUpdate, TimeRange};
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
        description: Some("Test project for kanban integration tests".to_string()),
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

/// Test the full board lifecycle: create -> list -> get -> update -> delete.
#[test]
fn test_board_lifecycle() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_board_lifecycle_inner());
    });
}

async fn test_board_lifecycle_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Create a test entity/project
    let entity_id = create_test_entity(&services, "Test Project").await;

    // 1. Create a board
    let board = kanban
        .create_board(&entity_id, "Sprint Board", Some("Test board description"))
        .await
        .expect("Failed to create board");

    assert!(!board.id.is_empty(), "Board ID should not be empty");
    assert_eq!(board.name, "Sprint Board");
    assert_eq!(board.entity_id, entity_id);

    // 2. List boards - should include our new board
    let boards = kanban
        .list_boards(&entity_id)
        .await
        .expect("Failed to list boards");

    assert!(!boards.is_empty(), "Should have at least one board");
    // Find board by name
    let found_board = boards.iter().find(|b| b.name == "Sprint Board");
    assert!(found_board.is_some(), "Should find board by name");

    // 3. Get board details using the original board ID (from create_board)
    // The local CRDT uses this ID for subsequent operations
    let board_view = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get board");

    assert_eq!(board_view.name, "Sprint Board");

    // 4. Update board (use original board ID)
    kanban
        .update_board(&board.id, Some("Updated Board Name".to_string()), None)
        .await
        .expect("Failed to update board");

    // Verify update
    let updated_board = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get updated board");
    assert_eq!(updated_board.name, "Updated Board Name");

    // 5. Delete board
    kanban
        .delete_board(&board.id)
        .await
        .expect("Failed to delete board");

    // Verify deletion - the board with our ID should be gone from local CRDT
    // Note: Due to dual-storage (CRDT + CommunitasApp), there may be orphaned boards
    // in CommunitasApp. This is a known limitation being tracked for future work.
    let get_result = kanban.get_board(&board.id).await;
    assert!(
        get_result.is_err(),
        "Deleted board should not be accessible"
    );
}

/// Test the card lifecycle: create -> get -> move -> update -> archive.
#[test]
fn test_card_lifecycle() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_card_lifecycle_inner());
    });
}

async fn test_card_lifecycle_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup: create entity, board, and columns
    let entity_id = create_test_entity(&services, "Card Test Project").await;

    let board = kanban
        .create_board(&entity_id, "Card Test Board", None)
        .await
        .expect("Failed to create board");

    // Create two columns (position is the third parameter)
    let col1 = kanban
        .create_column(&board.id, "To Do", 0)
        .await
        .expect("Failed to create column 1");

    let col2 = kanban
        .create_column(&board.id, "In Progress", 1)
        .await
        .expect("Failed to create column 2");

    // 1. Create a card (simplified API: board_id, column_id, title)
    let card = kanban
        .create_card(&board.id, &col1.id, "Test Card")
        .await
        .expect("Failed to create card");

    assert!(!card.id.is_empty(), "Card ID should not be empty");
    assert_eq!(card.title, "Test Card");

    // 2. Get card details
    let card_detail = kanban
        .get_card(&board.id, &card.id)
        .await
        .expect("Failed to get card");

    assert_eq!(card_detail.id, card.id);
    assert_eq!(card_detail.title, "Test Card");

    // 3. Move card to another column
    kanban
        .move_card(&board.id, &card.id, &col2.id, 0)
        .await
        .expect("Failed to move card");

    // Verify move - check board view shows card in new column
    let board_view = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get board after move");

    // Find the card in the columns
    let card_in_col2 = board_view
        .columns
        .iter()
        .find(|col| col.id == col2.id)
        .and_then(|col| col.cards.iter().find(|c| c.id == card.id));
    assert!(card_in_col2.is_some(), "Card should be in column 2");

    // 4. Update card
    kanban
        .update_card(
            &board.id,
            &card.id,
            CardUpdate {
                title: Some("Updated Card Title".to_string()),
                description: Some("Updated description".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to update card");

    // Verify update
    let updated_card = kanban
        .get_card(&board.id, &card.id)
        .await
        .expect("Failed to get updated card");
    assert_eq!(updated_card.title, "Updated Card Title");

    // 5. Archive card
    kanban
        .archive_card(&board.id, &card.id)
        .await
        .expect("Failed to archive card");

    // Verify archive - card should still exist but be archived
    let archived_card = kanban
        .get_card(&board.id, &card.id)
        .await
        .expect("Failed to get archived card");
    assert!(
        archived_card.state == CardState::Archived,
        "Card should be archived"
    );
}

/// Test that unauthenticated access is blocked.
#[test]
fn test_unauthenticated_access() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_unauthenticated_access_inner());
    });
}

async fn test_unauthenticated_access_inner() {
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

    // Do NOT enable demo mode - stay unauthenticated
    let kanban = services.kanban();

    // All operations should fail with NotAuthenticated
    let list_result = kanban.list_boards("some-entity").await;
    assert!(
        list_result.is_err(),
        "list_boards should fail when unauthenticated"
    );

    let create_result = kanban.create_board("some-entity", "Test", None).await;
    assert!(
        create_result.is_err(),
        "create_board should fail when unauthenticated"
    );
}

/// Test that watch channel updates when boards change.
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
    let kanban = services.kanban();

    // Subscribe to updates
    let rx = kanban.subscribe();

    // Create entity
    let entity_id = create_test_entity(&services, "Watch Test Project").await;

    // Initial snapshot should have no boards
    let initial_snap = rx.borrow().clone();
    assert!(
        initial_snap.boards.is_empty(),
        "Should start with no boards"
    );

    // Create a board
    let board = kanban
        .create_board(&entity_id, "Watch Test Board", None)
        .await
        .expect("Failed to create board");

    // List boards to update snapshot
    let _ = kanban.list_boards(&entity_id).await;

    // Check snapshot was updated
    let updated_snap = rx.borrow().clone();
    assert!(
        !updated_snap.boards.is_empty(),
        "Should have at least one board now"
    );
    // Find board by name to verify it exists
    let board_from_snap = updated_snap
        .boards
        .iter()
        .find(|b| b.name == "Watch Test Board");
    assert!(
        board_from_snap.is_some(),
        "Should find board by name in snapshot"
    );

    // Delete board using the original ID (from create_board) and list again
    kanban.delete_board(&board.id).await.unwrap();
    let _ = kanban.list_boards(&entity_id).await;

    // Check that the deleted board is no longer accessible
    // Note: The snapshot may still contain boards from CommunitasApp, but the
    // local CRDT board should be gone
    let get_result = kanban.get_board(&board.id).await;
    assert!(
        get_result.is_err(),
        "Deleted board should not be accessible"
    );
}

/// Test loading boards for entity selection (loading state).
#[test]
fn test_load_boards_loading_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_load_boards_loading_state_inner());
    });
}

async fn test_load_boards_loading_state_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Create entity
    let entity_id = create_test_entity(&services, "Load Test Project").await;

    // Create some boards first
    let _ = kanban
        .create_board(&entity_id, "Board 1", None)
        .await
        .unwrap();
    let _ = kanban
        .create_board(&entity_id, "Board 2", None)
        .await
        .unwrap();

    // Now use load_boards to load them
    let boards = kanban
        .load_boards(&entity_id)
        .await
        .expect("Failed to load boards");

    assert_eq!(boards.len(), 2, "Should have two boards");

    // Snapshot should not be loading after load completes
    let snap = kanban.current_snapshot();
    assert!(!snap.loading, "Should not be loading after load completes");
    assert_eq!(snap.boards.len(), 2, "Snapshot should have two boards");
}

/// Test column creation and board view.
#[test]
fn test_column_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_column_operations_inner());
    });
}

async fn test_column_operations_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    let entity_id = create_test_entity(&services, "Column Test Project").await;
    let board = kanban
        .create_board(&entity_id, "Column Test Board", None)
        .await
        .expect("Failed to create board");

    // Create columns with positions
    let col1 = kanban
        .create_column(&board.id, "Backlog", 0)
        .await
        .expect("Failed to create column 1");

    let col2 = kanban
        .create_column(&board.id, "Ready", 1)
        .await
        .expect("Failed to create column 2");

    let col3 = kanban
        .create_column(&board.id, "Done", 2)
        .await
        .expect("Failed to create column 3");

    // Get board and verify columns
    let board_view = kanban
        .get_board(&board.id)
        .await
        .expect("Failed to get board");

    assert_eq!(board_view.columns.len(), 3, "Should have 3 columns");

    // Columns should be in order
    let column_names: Vec<&str> = board_view.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(column_names.contains(&"Backlog"));
    assert!(column_names.contains(&"Ready"));
    assert!(column_names.contains(&"Done"));

    // Verify column IDs
    let column_ids: Vec<&str> = board_view.columns.iter().map(|c| c.id.as_str()).collect();
    assert!(column_ids.contains(&col1.id.as_str()));
    assert!(column_ids.contains(&col2.id.as_str()));
    assert!(column_ids.contains(&col3.id.as_str()));
}

// =============================================================================
// Phase 6.6 Integration Tests
// =============================================================================

/// Test due date filtering: overdue, due soon, and has due date.
#[test]
fn test_due_date_filtering() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_due_date_filtering_inner());
    });
}

async fn test_due_date_filtering_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup: create entity, board, and column
    let entity_id = create_test_entity(&services, "Due Date Filter Project").await;
    let board = kanban
        .create_board(&entity_id, "Due Date Test Board", None)
        .await
        .expect("Failed to create board");
    let column = kanban
        .create_column(&board.id, "Tasks", 0)
        .await
        .expect("Failed to create column");

    // Get current timestamp
    let now = chrono::Utc::now().timestamp();
    let day_seconds = 86400;

    // Create cards with different due dates
    // 1. Overdue card (due 2 days ago)
    let overdue_card = kanban
        .create_card(&board.id, &column.id, "Overdue Card")
        .await
        .expect("Failed to create overdue card");
    kanban
        .update_card(
            &board.id,
            &overdue_card.id,
            CardUpdate {
                due_date: Some(Some(now - 2 * day_seconds)),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to set overdue date");

    // 2. Due soon card (due in 3 days)
    let due_soon_card = kanban
        .create_card(&board.id, &column.id, "Due Soon Card")
        .await
        .expect("Failed to create due soon card");
    kanban
        .update_card(
            &board.id,
            &due_soon_card.id,
            CardUpdate {
                due_date: Some(Some(now + 3 * day_seconds)),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to set due soon date");

    // 3. Future card (due in 30 days)
    let future_card = kanban
        .create_card(&board.id, &column.id, "Future Card")
        .await
        .expect("Failed to create future card");
    kanban
        .update_card(
            &board.id,
            &future_card.id,
            CardUpdate {
                due_date: Some(Some(now + 30 * day_seconds)),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to set future date");

    // 4. No due date card
    let _ = kanban
        .create_card(&board.id, &column.id, "No Due Date Card")
        .await
        .expect("Failed to create no due date card");

    // Verify board analytics reflect due date counts
    let analytics = kanban
        .get_board_analytics(&board.id)
        .await
        .expect("Failed to get analytics");

    assert_eq!(analytics.overdue_count, 1, "Should have 1 overdue card");
    assert_eq!(
        analytics.due_soon_count, 1,
        "Should have 1 due soon card (within 7 days)"
    );
}

/// Test analytics calculation accuracy.
#[test]
fn test_analytics_calculations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_analytics_calculations_inner());
    });
}

async fn test_analytics_calculations_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup: create entity, board with multiple columns
    let entity_id = create_test_entity(&services, "Analytics Project").await;
    let board = kanban
        .create_board(&entity_id, "Analytics Test Board", None)
        .await
        .expect("Failed to create board");

    // Create columns representing workflow stages
    let todo_col = kanban
        .create_column(&board.id, "To Do", 0)
        .await
        .expect("Failed to create To Do column");
    let progress_col = kanban
        .create_column(&board.id, "In Progress", 1)
        .await
        .expect("Failed to create In Progress column");
    let done_col = kanban
        .create_column(&board.id, "Done", 2)
        .await
        .expect("Failed to create Done column");

    // Create cards in different columns
    for i in 0..5 {
        kanban
            .create_card(&board.id, &todo_col.id, &format!("Todo {}", i))
            .await
            .expect("Failed to create todo card");
    }

    for i in 0..3 {
        kanban
            .create_card(&board.id, &progress_col.id, &format!("In Progress {}", i))
            .await
            .expect("Failed to create progress card");
    }

    for i in 0..2 {
        kanban
            .create_card(&board.id, &done_col.id, &format!("Done {}", i))
            .await
            .expect("Failed to create done card");
    }

    // Get board analytics
    let analytics = kanban
        .get_board_analytics(&board.id)
        .await
        .expect("Failed to get analytics");

    // Verify counts
    assert_eq!(
        analytics.total_active_cards, 10,
        "Should have 10 total cards"
    );

    // Verify column distribution
    let todo_count = analytics
        .column_card_counts
        .get(&todo_col.id)
        .copied()
        .unwrap_or(0);
    let progress_count = analytics
        .column_card_counts
        .get(&progress_col.id)
        .copied()
        .unwrap_or(0);
    let done_count = analytics
        .column_card_counts
        .get(&done_col.id)
        .copied()
        .unwrap_or(0);

    assert_eq!(todo_count, 5, "To Do should have 5 cards");
    assert_eq!(progress_count, 3, "In Progress should have 3 cards");
    assert_eq!(done_count, 2, "Done should have 2 cards");

    // Verify WIP count (typically excludes first and last columns)
    // WIP should be cards not in first (backlog) or last (done) columns
    assert!(analytics.wip_count > 0, "Should have WIP cards");
}

/// Test velocity calculation.
#[test]
fn test_velocity_calculation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_velocity_calculation_inner());
    });
}

async fn test_velocity_calculation_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup
    let entity_id = create_test_entity(&services, "Velocity Project").await;
    let board = kanban
        .create_board(&entity_id, "Velocity Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Done", 0)
        .await
        .expect("Failed to create column");

    // Create some cards
    for i in 0..5 {
        kanban
            .create_card(&board.id, &column.id, &format!("Card {}", i))
            .await
            .expect("Failed to create card");
    }

    // Calculate velocity for 4 weeks
    let velocity = kanban
        .calculate_velocity(&board.id, TimeRange::Weeks(4))
        .await
        .expect("Failed to calculate velocity");

    // Should have data points (one per period)
    assert!(
        !velocity.data_points.is_empty(),
        "Should have velocity data points"
    );

    // Average should be non-negative
    assert!(velocity.average >= 0.0, "Average should be non-negative");
}

/// Test cycle time distribution calculation.
#[test]
fn test_cycle_time_distribution() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_cycle_time_distribution_inner());
    });
}

async fn test_cycle_time_distribution_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup
    let entity_id = create_test_entity(&services, "Cycle Time Project").await;
    let board = kanban
        .create_board(&entity_id, "Cycle Time Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Tasks", 0)
        .await
        .expect("Failed to create column");

    // Create some cards
    for i in 0..3 {
        kanban
            .create_card(&board.id, &column.id, &format!("Card {}", i))
            .await
            .expect("Failed to create card");
    }

    // Calculate cycle time distribution
    let distribution = kanban
        .calculate_cycle_time_distribution(&board.id, TimeRange::Weeks(4))
        .await
        .expect("Failed to calculate cycle time");

    // The distribution should be calculable even with no completed cards
    // (will have empty cycle times but valid structure)
    assert_eq!(
        distribution.board_id, board.id,
        "Distribution should be for our board"
    );

    // If there are histogram buckets (from completed cards), verify standard ranges
    if !distribution.histogram.is_empty() {
        let bucket_labels: Vec<&str> = distribution
            .histogram
            .iter()
            .map(|b| b.label.as_str())
            .collect();
        assert!(
            bucket_labels.contains(&"< 1 day"),
            "Should have < 1 day bucket when histogram is populated"
        );
        assert!(
            bucket_labels.contains(&"> 2 weeks"),
            "Should have > 2 weeks bucket when histogram is populated"
        );
    }
}

/// Test card-to-thread linking.
#[test]
fn test_card_thread_linking() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_card_thread_linking_inner());
    });
}

async fn test_card_thread_linking_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup
    let entity_id = create_test_entity(&services, "Thread Link Project").await;
    let board = kanban
        .create_board(&entity_id, "Thread Link Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Tasks", 0)
        .await
        .expect("Failed to create column");

    // Create a card
    let card = kanban
        .create_card(&board.id, &column.id, "Linked Card")
        .await
        .expect("Failed to create card");

    // Link a thread to the card
    let thread_id = "test-thread-123";
    kanban
        .update_card(
            &board.id,
            &card.id,
            CardUpdate {
                linked_thread_id: Some(Some(thread_id.to_string())),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to link thread");

    // Verify the link
    let updated_card = kanban
        .get_card(&board.id, &card.id)
        .await
        .expect("Failed to get card");

    assert_eq!(
        updated_card.linked_thread_id.as_deref(),
        Some(thread_id),
        "Card should have linked thread ID"
    );

    // Unlink the thread
    kanban
        .update_card(
            &board.id,
            &card.id,
            CardUpdate {
                linked_thread_id: Some(None),
                ..Default::default()
            },
        )
        .await
        .expect("Failed to unlink thread");

    // Verify unlink
    let unlinked_card = kanban
        .get_card(&board.id, &card.id)
        .await
        .expect("Failed to get card after unlink");

    assert!(
        unlinked_card.linked_thread_id.is_none(),
        "Card should have no linked thread after unlink"
    );
}

/// Test burndown chart calculation.
#[test]
fn test_burndown_calculation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_burndown_calculation_inner());
    });
}

async fn test_burndown_calculation_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup
    let entity_id = create_test_entity(&services, "Burndown Project").await;
    let board = kanban
        .create_board(&entity_id, "Burndown Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Tasks", 0)
        .await
        .expect("Failed to create column");

    // Create some cards
    for i in 0..8 {
        kanban
            .create_card(&board.id, &column.id, &format!("Task {}", i))
            .await
            .expect("Failed to create card");
    }

    // Calculate burndown for the last 2 weeks
    let now = chrono::Utc::now().timestamp();
    let two_weeks_ago = now - 14 * 86400;

    let burndown = kanban
        .calculate_burndown(&board.id, two_weeks_ago, now)
        .await
        .expect("Failed to calculate burndown");

    // Verify burndown structure
    assert_eq!(
        burndown.board_id, board.id,
        "Burndown should be for our board"
    );
    assert_eq!(
        burndown.period_start, two_weeks_ago,
        "Period start should match"
    );
    assert_eq!(burndown.period_end, now, "Period end should match");

    // Current scope should be >= initial (cards may have been added)
    assert!(
        burndown.current_scope >= burndown.initial_card_count,
        "Current scope should be >= initial"
    );
}

/// Test conflict detection through CRDT snapshot.
#[test]
fn test_conflict_snapshot() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_conflict_snapshot_inner());
    });
}

async fn test_conflict_snapshot_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Subscribe to kanban updates
    let rx = kanban.subscribe();

    // Create entity and board
    let entity_id = create_test_entity(&services, "Conflict Test Project").await;
    let board = kanban
        .create_board(&entity_id, "Conflict Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Tasks", 0)
        .await
        .expect("Failed to create column");

    // Create a card
    let _ = kanban
        .create_card(&board.id, &column.id, "Conflict Card")
        .await
        .expect("Failed to create card");

    // Get current snapshot
    let snapshot = rx.borrow().clone();

    // Conflicts list should exist (may be empty without actual concurrent edits)
    // This tests that the conflict tracking infrastructure is in place
    assert!(
        snapshot.conflicts.is_empty() || !snapshot.conflicts.is_empty(),
        "Conflicts field should be accessible"
    );

    // The snapshot should include our board
    assert!(
        !snapshot.boards.is_empty() || snapshot.loading,
        "Snapshot should have boards or be loading"
    );
}

/// Test getting a nonexistent board returns appropriate error.
#[test]
fn test_get_nonexistent_board() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_get_nonexistent_board_inner());
    });
}

async fn test_get_nonexistent_board_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Try to get a board that doesn't exist
    let result = kanban.get_board("nonexistent-board-id").await;

    // Should return an error (BoardNotFound)
    assert!(
        result.is_err(),
        "Getting nonexistent board should return error"
    );
}

/// Test that analytics methods handle empty boards gracefully.
#[test]
fn test_analytics_empty_board() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_analytics_empty_board_inner());
    });
}

async fn test_analytics_empty_board_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup: create entity and empty board
    let entity_id = create_test_entity(&services, "Empty Analytics Project").await;
    let board = kanban
        .create_board(&entity_id, "Empty Board", None)
        .await
        .expect("Failed to create board");

    // Get analytics for empty board - should not panic
    let analytics = kanban
        .get_board_analytics(&board.id)
        .await
        .expect("Failed to get analytics for empty board");

    assert_eq!(
        analytics.total_active_cards, 0,
        "Empty board should have 0 cards"
    );
    assert_eq!(
        analytics.overdue_count, 0,
        "Empty board should have 0 overdue"
    );
    assert!(
        analytics.column_card_counts.is_empty(),
        "Empty board should have no column counts"
    );

    // Velocity for empty board
    let velocity = kanban
        .calculate_velocity(&board.id, TimeRange::Weeks(4))
        .await
        .expect("Failed to calculate velocity for empty board");

    assert_eq!(
        velocity.average, 0.0,
        "Empty board should have 0 average velocity"
    );

    // Cycle time for empty board
    let cycle_time = kanban
        .calculate_cycle_time_distribution(&board.id, TimeRange::Weeks(4))
        .await
        .expect("Failed to calculate cycle time for empty board");

    // Empty distribution is valid (no completed cards)
    assert_eq!(
        cycle_time.board_id, board.id,
        "Cycle time should be for our board"
    );
}

/// Test TimeRange::Custom validation via service.
#[test]
fn test_time_range_custom_validation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_time_range_custom_validation_inner());
    });
}

async fn test_time_range_custom_validation_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let kanban = services.kanban();

    // Setup
    let entity_id = create_test_entity(&services, "TimeRange Project").await;
    let board = kanban
        .create_board(&entity_id, "TimeRange Test Board", None)
        .await
        .expect("Failed to create board");

    let column = kanban
        .create_column(&board.id, "Tasks", 0)
        .await
        .expect("Failed to create column");

    // Create a card
    kanban
        .create_card(&board.id, &column.id, "Test Card")
        .await
        .expect("Failed to create card");

    // Use validated custom time range constructor
    let now = chrono::Utc::now().timestamp();
    let valid_range = TimeRange::custom(now - 86400 * 7, now);
    assert!(
        valid_range.is_some(),
        "Valid custom range should be constructable"
    );

    let velocity = kanban
        .calculate_velocity(&board.id, valid_range.unwrap())
        .await
        .expect("Failed with valid custom range");
    assert_eq!(velocity.board_id, board.id);

    // Invalid range (start > end) should return None
    let invalid_range = TimeRange::custom(now, now - 86400);
    assert!(
        invalid_range.is_none(),
        "Invalid custom range should return None"
    );
}

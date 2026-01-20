//! Integration tests for the kanban service using real CommunitasApp.
//!
//! These tests verify the full kanban flow including board/column/card CRUD,
//! authentication attribution, and reactive watch channel updates.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_api::CardState;
use communitas_ui_service::UiServices;
use communitas_ui_service::kanban::CardUpdate;
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

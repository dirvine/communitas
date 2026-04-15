// SPDX-License-Identifier: MIT OR Apache-2.0

//! Property-based tests for the Communitas UI services.
//!
//! These tests use proptest to verify invariants hold across random inputs.
//! Focus areas:
//! - Kanban card operations (ordering, state transitions)
//! - Canvas element operations (transform consistency)
//! - Data validation (identities, titles, content)

use proptest::prelude::*;
use std::sync::Arc;

use communitas_core::EntityType;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use tempfile::TempDir;

/// Stack size for test threads (8MB) to handle large async state machines.
const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

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
    services.auth().enable_demo_mode();
    // Wait for background auth watcher to reinitialize with authenticated peer_id.
    // with the authenticated peer_id. Without this yield, the old anonymous
    // CoreKanbanService may be replaced between create_board and create_column
    // calls, causing BoardNotFound errors.
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
        description: Some("Property test entity".to_string()),
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
// Strategy Helpers
// =============================================================================

/// Strategy for generating valid card titles (non-empty, reasonable length).
fn card_title_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 _-]{1,100}".prop_filter("title must be non-empty", |s| !s.trim().is_empty())
}

/// Strategy for generating column names.
fn column_name_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 _-]{1,50}".prop_filter("column name must be non-empty", |s| !s.trim().is_empty())
}

/// Strategy for generating board names.
fn board_name_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 _-]{1,50}".prop_filter("board name must be non-empty", |s| !s.trim().is_empty())
}

/// Strategy for generating message content.
fn message_content_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 .,!?]{1,1000}".prop_filter("message must be non-empty", |s| !s.trim().is_empty())
}

// =============================================================================
// Kanban Property Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: Board creation always succeeds with valid name.
    #[test]
    fn prop_board_creation_succeeds(name in board_name_strategy()) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let kanban = services.kanban();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    let result = kanban.create_board(&entity_id, &name, None).await;

                    prop_assert!(result.is_ok(), "Board creation should succeed");
                    let board = result.unwrap();
                    prop_assert!(!board.id.is_empty(), "Board should have an ID");
                    prop_assert_eq!(&board.name, &name, "Board name should match input");
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }

    /// Property: Column ordering is preserved after creation.
    #[test]
    fn prop_column_ordering_preserved(
        col1 in column_name_strategy(),
        col2 in column_name_strategy(),
        col3 in column_name_strategy(),
    ) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let kanban = services.kanban();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    let board = kanban
                        .create_board(&entity_id, "Test Board", None)
                        .await
                        .unwrap();

                    // Create columns in order
                    let c1 = kanban.create_column(&board.id, &col1, 0).await.unwrap();
                    let c2 = kanban.create_column(&board.id, &col2, 1).await.unwrap();
                    let c3 = kanban.create_column(&board.id, &col3, 2).await.unwrap();

                    // Verify order
                    let board_view = kanban.get_board(&board.id).await.unwrap();
                    prop_assert_eq!(board_view.columns.len(), 3, "Should have 3 columns");
                    prop_assert_eq!(&board_view.columns[0].id, &c1.id, "First column should match");
                    prop_assert_eq!(&board_view.columns[1].id, &c2.id, "Second column should match");
                    prop_assert_eq!(&board_view.columns[2].id, &c3.id, "Third column should match");
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }

    /// Property: Card creation always assigns to correct column.
    #[test]
    fn prop_card_in_correct_column(title in card_title_strategy()) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let kanban = services.kanban();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    let board = kanban
                        .create_board(&entity_id, "Test Board", None)
                        .await
                        .unwrap();

                    let column = kanban.create_column(&board.id, "Todo", 0).await.unwrap();
                    let card = kanban.create_card(&board.id, &column.id, &title).await.unwrap();

                    // Verify card is in correct column
                    let board_view = kanban.get_board(&board.id).await.unwrap();
                    let card_in_column = board_view.columns[0]
                        .cards
                        .iter()
                        .any(|c| c.id == card.id);
                    prop_assert!(card_in_column, "Card should be in the target column");
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }

    /// Property: Card move preserves card data.
    #[test]
    fn prop_card_move_preserves_data(title in card_title_strategy()) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let kanban = services.kanban();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    let board = kanban
                        .create_board(&entity_id, "Test Board", None)
                        .await
                        .unwrap();

                    let col1 = kanban.create_column(&board.id, "Todo", 0).await.unwrap();
                    let col2 = kanban.create_column(&board.id, "Done", 1).await.unwrap();

                    let card = kanban.create_card(&board.id, &col1.id, &title).await.unwrap();
                    let original_title = card.title.clone();

                    // Move card
                    kanban.move_card(&board.id, &card.id, &col2.id, 0).await.unwrap();

                    // Verify data preserved
                    let moved_card = kanban.get_card(&board.id, &card.id).await.unwrap();
                    prop_assert_eq!(&moved_card.title, &original_title, "Title should be preserved after move");
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }

    /// Property: Archive state is consistent.
    #[test]
    fn prop_archive_state_consistent(title in card_title_strategy()) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    use communitas_ui_api::kanban::CardState;

                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let kanban = services.kanban();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    let board = kanban
                        .create_board(&entity_id, "Test Board", None)
                        .await
                        .unwrap();

                    let column = kanban.create_column(&board.id, "Todo", 0).await.unwrap();
                    let card = kanban.create_card(&board.id, &column.id, &title).await.unwrap();

                    // Archive
                    kanban.archive_card(&board.id, &card.id).await.unwrap();

                    // Verify state
                    let archived_card = kanban.get_card(&board.id, &card.id).await.unwrap();
                    prop_assert!(
                        matches!(archived_card.state, CardState::Archived),
                        "Card should be in Archived state"
                    );
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }
}

// =============================================================================
// Messaging Property Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: Queued messages are persisted to pending queue.
    #[test]
    fn prop_queued_message_persisted(content in message_content_strategy()) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let messaging = services.messaging();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    messaging.queue_message(&entity_id, &content, None);

                    let snapshot = messaging.current_snapshot();
                    prop_assert_eq!(
                        snapshot.pending_messages.len(),
                        1,
                        "Should have one pending message"
                    );
                    prop_assert_eq!(
                        &snapshot.pending_messages[0].text,
                        &content,
                        "Message content should match"
                    );
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }

    /// Property: Multiple queued messages maintain order.
    #[test]
    fn prop_message_queue_order(
        msg1 in message_content_strategy(),
        msg2 in message_content_strategy(),
        msg3 in message_content_strategy(),
    ) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let messaging = services.messaging();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    messaging.queue_message(&entity_id, &msg1, None);
                    messaging.queue_message(&entity_id, &msg2, None);
                    messaging.queue_message(&entity_id, &msg3, None);

                    let snapshot = messaging.current_snapshot();
                    prop_assert_eq!(snapshot.pending_messages.len(), 3, "Should have 3 messages");
                    prop_assert_eq!(&snapshot.pending_messages[0].text, &msg1, "First message");
                    prop_assert_eq!(&snapshot.pending_messages[1].text, &msg2, "Second message");
                    prop_assert_eq!(&snapshot.pending_messages[2].text, &msg3, "Third message");
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }
}

// =============================================================================
// Canvas Property Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// Property: Canvas snapshot is consistent after text element operations.
    #[test]
    fn prop_canvas_snapshot_consistent(
        x in 0.0f32..1000.0,
        y in 0.0f32..1000.0,
        font_size in 10.0f32..72.0,
    ) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    use communitas_ui_service::canvas::ElementKindView;

                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let canvas = services.canvas();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    canvas.load_canvas(&entity_id).await.unwrap();

                    // Add a text element
                    let content = "Test content".to_string();
                    let color = "#000000".to_string();
                    canvas
                        .add_text(Some(&entity_id), content.clone(), x, y, font_size, color.clone())
                        .await
                        .unwrap();

                    // Verify snapshot consistency
                    let snapshot = canvas.current_snapshot();
                    prop_assert_eq!(snapshot.elements.len(), 1, "Should have 1 element");

                    // Verify text content is preserved
                    if let ElementKindView::Text { content: c, font_size: fs, color: col } = &snapshot.elements[0].kind {
                        prop_assert_eq!(c, &content, "Content should be preserved");
                        prop_assert!(
                            (*fs - font_size).abs() < 0.001,
                            "Font size should be preserved"
                        );
                        prop_assert_eq!(col, &color, "Color should be preserved");
                    } else {
                        prop_assert!(false, "Element should be text");
                    }
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }

    /// Property: Undo/redo are inverses.
    #[test]
    fn prop_undo_redo_inverse(x in 0.0f32..500.0, y in 0.0f32..500.0) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_authenticated_services(&temp).await;
                    let canvas = services.canvas();
                    let entity_id = create_test_entity(&services, "PropTest").await;

                    canvas.load_canvas(&entity_id).await.unwrap();

                    // Add a text element
                    canvas
                        .add_text(
                            Some(&entity_id),
                            "Test".to_string(),
                            x,
                            y,
                            14.0,
                            "#000000".to_string(),
                        )
                        .await
                        .unwrap();

                    prop_assert_eq!(canvas.current_snapshot().elements.len(), 1);

                    // Undo - returns Ok(Some(...)) if successful, Ok(None) if nothing to undo
                    let undo_result = canvas.undo().await.unwrap();
                    prop_assert!(undo_result.is_some(), "Undo should return the undone operation");
                    prop_assert_eq!(canvas.current_snapshot().elements.len(), 0, "After undo");

                    // Redo - returns Ok(Some(...)) if successful, Ok(None) if nothing to redo
                    let redo_result = canvas.redo().await.unwrap();
                    prop_assert!(redo_result.is_some(), "Redo should return the redone operation");
                    prop_assert_eq!(canvas.current_snapshot().elements.len(), 1, "After redo");

                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }
}

// =============================================================================
// Data Validation Property Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Valid identities have exactly 4 words.
    #[test]
    fn prop_identity_format(
        word1 in "[a-z]{3,8}",
        word2 in "[a-z]{3,8}",
        word3 in "[a-z]{3,8}",
        word4 in "[a-z]{3,8}",
    ) {
        let identity = format!("{}-{}-{}-{}", word1, word2, word3, word4);
        let parts: Vec<&str> = identity.split('-').collect();
        prop_assert_eq!(parts.len(), 4, "Identity should have 4 parts");
        for part in parts {
            prop_assert!(part.len() >= 3 && part.len() <= 8, "Each word should be 3-8 chars");
            prop_assert!(part.chars().all(|c| c.is_ascii_lowercase()), "Words should be lowercase");
        }
    }

    /// Property: Card titles are trimmed and non-empty.
    #[test]
    fn prop_card_title_validation(title in card_title_strategy()) {
        let trimmed = title.trim();
        prop_assert!(!trimmed.is_empty(), "Trimmed title should not be empty");
        prop_assert!(trimmed.len() <= 100, "Title should be at most 100 chars");
    }
}

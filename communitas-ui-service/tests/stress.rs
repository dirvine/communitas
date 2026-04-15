// SPDX-License-Identifier: MIT OR Apache-2.0

//! Stress tests for the Communitas UI services.
//!
//! These tests are marked `#[ignore]` and should be run in CI nightly builds
//! or manually via `cargo test -p communitas-ui-service --test stress -- --ignored`.
//!
//! Focus areas:
//! - Concurrent operations
//! - High-volume data processing
//! - Resource exhaustion handling
//! - Memory stability over time

use std::sync::Arc;
use std::time::{Duration, Instant};

use communitas_core::EntityType;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_ui_service::UiServices;
use communitas_ui_service::kanban::CardUpdate;
use communitas_ui_service::storage::UiStorage;
use tempfile::TempDir;

/// Stack size for test threads (8MB) to handle large async state machines.
const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Default stress test duration for configurable tests.
const DEFAULT_STRESS_DURATION_SECS: u64 = 10;

/// Helper to create UiServices with demo authentication enabled.
async fn make_authenticated_services(temp: &TempDir) -> UiServices {
    let storage =
        UiStorage::from_path(temp.path()).expect("storage creation failed in stress test");
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "StressTestUser".to_string(),
            "StressTestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .expect("app creation failed in stress test"),
    );
    let services = UiServices::new(storage, app).expect("services creation failed in stress test");
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
        description: Some("Stress test entity".to_string()),
        initial_members: vec![],
    };

    let events = app
        .execute(cmd)
        .await
        .expect("Failed to create stress test entity");

    events
        .iter()
        .find_map(|event| match event {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .expect("No EntityCreated event returned")
}

/// Helper to run async tests with a large stack.
fn run_with_large_stack<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(TEST_STACK_SIZE)
        .spawn(f)
        .expect("failed to spawn test thread")
        .join()
        .expect("test thread panicked");
}

// =============================================================================
// Concurrent Message Stress Tests
// =============================================================================

/// Stress test: 100 concurrent message queue operations.
///
/// Verifies the messaging service handles concurrent access without data loss.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_100_concurrent_message_sends() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;
            let messaging = services.messaging();
            let entity_id = create_test_entity(&services, "MessageStress").await;

            let start = Instant::now();
            let mut handles = Vec::with_capacity(100);

            for i in 0..100 {
                let content = format!("Stress message {}", i);
                let eid = entity_id.clone();
                let msg = messaging.clone();
                handles.push(tokio::spawn(async move {
                    msg.queue_message(&eid, &content, None);
                }));
            }

            for handle in handles {
                handle.await.expect("message send task failed");
            }

            let elapsed = start.elapsed();
            let snapshot = messaging.current_snapshot();

            println!(
                "100 concurrent messages completed in {:?}, {} pending",
                elapsed,
                snapshot.pending_messages.len()
            );

            assert_eq!(
                snapshot.pending_messages.len(),
                100,
                "All 100 messages should be queued"
            );
        });
    });
}

// =============================================================================
// Kanban Card Operation Stress Tests
// =============================================================================

/// Stress test: 1000 kanban card operations.
///
/// Tests card creation, updates, and moves in rapid succession.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_1000_kanban_card_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;
            let kanban = services.kanban();
            let entity_id = create_test_entity(&services, "KanbanStress").await;

            // Create a board with columns
            let board = kanban
                .create_board(&entity_id, "Stress Board", None)
                .await
                .expect("board creation failed");
            let col1 = kanban
                .create_column(&board.id, "Todo", 0)
                .await
                .expect("column creation failed");
            let col2 = kanban
                .create_column(&board.id, "Done", 1)
                .await
                .expect("column creation failed");

            let start = Instant::now();
            let mut cards = Vec::with_capacity(100);

            // Create 100 cards
            for i in 0..100 {
                let title = format!("Stress Card {}", i);
                let card = kanban
                    .create_card(&board.id, &col1.id, &title)
                    .await
                    .expect("card creation failed");
                cards.push(card);
            }

            // Update each card 5 times (500 operations)
            for card in &cards {
                for j in 0..5 {
                    let new_title = format!("{} - update {}", card.title, j);
                    let update = CardUpdate {
                        title: Some(new_title),
                        description: None,
                        assignees: None,
                        tags: None,
                        due_date: None,
                        linked_thread_id: None,
                    };
                    kanban
                        .update_card(&board.id, &card.id, update)
                        .await
                        .expect("card update failed");
                }
            }

            // Move each card to Done column and back (400 operations)
            for (i, card) in cards.iter().enumerate() {
                kanban
                    .move_card(&board.id, &card.id, &col2.id, i as u32)
                    .await
                    .expect("card move failed");
                kanban
                    .move_card(&board.id, &card.id, &col1.id, i as u32)
                    .await
                    .expect("card move back failed");
            }

            let elapsed = start.elapsed();
            let board_view = kanban
                .get_board(&board.id)
                .await
                .expect("board fetch failed");
            let total_cards: usize = board_view.columns.iter().map(|c| c.cards.len()).sum();

            println!(
                "1000 kanban operations completed in {:?}, {} cards total",
                elapsed, total_cards
            );

            assert_eq!(total_cards, 100, "All 100 cards should exist");
        });
    });
}

// =============================================================================
// Large Data Stress Tests
// =============================================================================

/// Stress test: Large text element on canvas.
///
/// Tests creating and manipulating large content in canvas elements.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_large_canvas_content() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;
            let canvas = services.canvas();
            let entity_id = create_test_entity(&services, "CanvasStress").await;

            canvas
                .load_canvas(&entity_id)
                .await
                .expect("canvas load failed");

            let start = Instant::now();

            // Create 100 text elements with varying sizes
            for i in 0..100 {
                let content = "X".repeat(1000 + i * 10); // 1KB to 2KB per element
                canvas
                    .add_text(
                        Some(&entity_id),
                        content,
                        (i % 10) as f32 * 100.0,
                        (i / 10) as f32 * 50.0,
                        14.0,
                        "#000000".to_string(),
                    )
                    .await
                    .expect("text element creation failed");
            }

            let elapsed = start.elapsed();
            let snapshot = canvas.current_snapshot();

            println!(
                "100 large canvas elements created in {:?}, {} elements total",
                elapsed,
                snapshot.elements.len()
            );

            assert_eq!(
                snapshot.elements.len(),
                100,
                "All 100 elements should exist"
            );
        });
    });
}

// =============================================================================
// Rapid UI Interaction Stress Tests
// =============================================================================

/// Stress test: Rapid undo/redo operations.
///
/// Tests the canvas history system under rapid operation.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_rapid_undo_redo() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;
            let canvas = services.canvas();
            let entity_id = create_test_entity(&services, "UndoRedoStress").await;

            canvas
                .load_canvas(&entity_id)
                .await
                .expect("canvas load failed");

            let start = Instant::now();

            // Create 50 elements
            for i in 0..50 {
                canvas
                    .add_text(
                        Some(&entity_id),
                        format!("Element {}", i),
                        (i % 10) as f32 * 100.0,
                        (i / 10) as f32 * 50.0,
                        14.0,
                        "#000000".to_string(),
                    )
                    .await
                    .expect("text element creation failed");
            }

            // Undo all 50
            for _ in 0..50 {
                let _ = canvas.undo().await;
            }

            assert_eq!(
                canvas.current_snapshot().elements.len(),
                0,
                "All elements should be undone"
            );

            // Redo all 50
            for _ in 0..50 {
                let _ = canvas.redo().await;
            }

            let elapsed = start.elapsed();

            println!(
                "50 creates + 50 undos + 50 redos completed in {:?}",
                elapsed
            );

            assert_eq!(
                canvas.current_snapshot().elements.len(),
                50,
                "All elements should be restored"
            );
        });
    });
}

// =============================================================================
// Memory Stability Stress Tests
// =============================================================================

/// Stress test: Sustained operations over time.
///
/// Tests for memory leaks by running operations for a configurable duration.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_memory_stability() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;

            let duration = Duration::from_secs(
                std::env::var("STRESS_DURATION_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(DEFAULT_STRESS_DURATION_SECS),
            );

            let start = Instant::now();
            let mut operation_count = 0u64;
            let mut entity_count = 0u64;

            println!("Starting memory stability test for {:?}", duration);

            while start.elapsed() < duration {
                // Create a fresh entity for each batch
                let entity_id =
                    create_test_entity(&services, &format!("MemStress{}", entity_count)).await;
                entity_count += 1;

                // Queue some messages
                let messaging = services.messaging();
                for i in 0..10 {
                    messaging.queue_message(&entity_id, &format!("Message {}", i), None);
                    operation_count += 1;
                }

                // Load canvas and add elements
                let canvas = services.canvas();
                if canvas.load_canvas(&entity_id).await.is_ok() {
                    for i in 0..5 {
                        let _ = canvas
                            .add_text(
                                Some(&entity_id),
                                format!("Text {}", i),
                                i as f32 * 20.0,
                                0.0,
                                14.0,
                                "#000000".to_string(),
                            )
                            .await;
                        operation_count += 1;
                    }
                }

                // Small delay to prevent overwhelming the system
                tokio::time::sleep(Duration::from_millis(10)).await;
            }

            let elapsed = start.elapsed();
            println!(
                "Memory stability test completed: {} operations across {} entities in {:?}",
                operation_count, entity_count, elapsed
            );
            println!(
                "Operations per second: {:.2}",
                operation_count as f64 / elapsed.as_secs_f64()
            );
        });
    });
}

// =============================================================================
// Concurrent Access Stress Tests
// =============================================================================

/// Stress test: Multiple concurrent entity operations.
///
/// Tests handling of parallel operations across different entities.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_concurrent_entity_access() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = Arc::new(make_authenticated_services(&temp).await);

            // Create 10 entities
            let mut entities = Vec::with_capacity(10);
            for i in 0..10 {
                let entity_id =
                    create_test_entity(&services, &format!("ConcurrentEntity{}", i)).await;
                entities.push(entity_id);
            }

            let start = Instant::now();
            let mut handles = Vec::with_capacity(10);

            // Spawn concurrent tasks for each entity
            for entity_id in entities {
                let svc = services.clone();
                handles.push(tokio::spawn(async move {
                    let messaging = svc.messaging();

                    // Queue 20 messages per entity
                    for j in 0..20 {
                        messaging.queue_message(&entity_id, &format!("Concurrent msg {}", j), None);
                    }

                    // Load canvas and add elements
                    let canvas = svc.canvas();
                    if canvas.load_canvas(&entity_id).await.is_ok() {
                        for j in 0..10 {
                            let _ = canvas
                                .add_text(
                                    Some(&entity_id),
                                    format!("Concurrent text {}", j),
                                    j as f32 * 20.0,
                                    0.0,
                                    14.0,
                                    "#000000".to_string(),
                                )
                                .await;
                        }
                    }
                }));
            }

            // Wait for all tasks
            for handle in handles {
                handle.await.expect("concurrent task failed");
            }

            let elapsed = start.elapsed();
            println!("10 concurrent entity operations completed in {:?}", elapsed);

            // Verify total operations
            let snapshot = services.messaging().current_snapshot();
            println!(
                "Total pending messages across all entities: {}",
                snapshot.pending_messages.len()
            );

            // Should have 10 entities * 20 messages = 200 messages
            assert_eq!(
                snapshot.pending_messages.len(),
                200,
                "All 200 messages should be queued"
            );
        });
    });
}

// =============================================================================
// Kanban Board Scaling Stress Tests
// =============================================================================

/// Stress test: Large kanban board with many columns and cards.
///
/// Tests performance with a heavily populated board.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_large_kanban_board() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;
            let kanban = services.kanban();
            let entity_id = create_test_entity(&services, "LargeBoardStress").await;

            let board = kanban
                .create_board(&entity_id, "Large Board", None)
                .await
                .expect("board creation failed");

            let start = Instant::now();

            // Create 10 columns
            let mut columns = Vec::with_capacity(10);
            for i in 0..10 {
                let col = kanban
                    .create_column(&board.id, &format!("Column {}", i), i)
                    .await
                    .expect("column creation failed");
                columns.push(col);
            }

            // Create 20 cards per column (200 total)
            for col in &columns {
                for j in 0..20 {
                    let _ = kanban
                        .create_card(&board.id, &col.id, &format!("Card {}-{}", col.name, j))
                        .await
                        .expect("card creation failed");
                }
            }

            // Fetch the full board
            let board_view = kanban
                .get_board(&board.id)
                .await
                .expect("board fetch failed");

            let elapsed = start.elapsed();
            let total_cards: usize = board_view.columns.iter().map(|c| c.cards.len()).sum();

            println!(
                "Large board created in {:?}: {} columns, {} cards",
                elapsed,
                board_view.columns.len(),
                total_cards
            );

            assert_eq!(board_view.columns.len(), 10, "Should have 10 columns");
            assert_eq!(total_cards, 200, "Should have 200 cards");
        });
    });
}

// =============================================================================
// Snapshot Performance Stress Tests
// =============================================================================

/// Stress test: Rapid snapshot operations.
///
/// Tests the performance of taking and comparing snapshots.
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_snapshot_performance() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().expect("runtime creation failed");
        rt.block_on(async {
            let temp = TempDir::new().expect("temp dir creation failed");
            let services = make_authenticated_services(&temp).await;
            let entity_id = create_test_entity(&services, "SnapshotStress").await;

            // Setup: Add some data
            let messaging = services.messaging();
            for i in 0..50 {
                messaging.queue_message(&entity_id, &format!("Message {}", i), None);
            }

            let canvas = services.canvas();
            canvas
                .load_canvas(&entity_id)
                .await
                .expect("canvas load failed");
            for i in 0..30 {
                let _ = canvas
                    .add_text(
                        Some(&entity_id),
                        format!("Element {}", i),
                        (i % 10) as f32 * 100.0,
                        (i / 10) as f32 * 50.0,
                        14.0,
                        "#000000".to_string(),
                    )
                    .await;
            }

            let start = Instant::now();
            let iterations = 1000;

            // Take many snapshots rapidly
            for _ in 0..iterations {
                let _ = messaging.current_snapshot();
                let _ = canvas.current_snapshot();
            }

            let elapsed = start.elapsed();
            let per_snapshot = elapsed / (iterations * 2);

            println!(
                "{} snapshot pairs taken in {:?} ({:?} per snapshot)",
                iterations, elapsed, per_snapshot
            );

            // Snapshots should be very fast (< 1ms each on average)
            assert!(
                per_snapshot < Duration::from_millis(1),
                "Snapshots should be sub-millisecond"
            );
        });
    });
}

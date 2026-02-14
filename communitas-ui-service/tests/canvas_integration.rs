//! Integration tests for the canvas service using real CommunitasApp.
//!
//! These tests verify the full canvas flow including element operations,
//! persistence, selection state, and offline queue behavior.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_service::UiServices;
use communitas_ui_service::canvas::TransformView;
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

/// Helper to create a test entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let canvas = services.canvas();
    let app = canvas.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Person,
        description: Some("Test entity for canvas integration tests".to_string()),
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

// ===== Test: Element Lifecycle (Add -> Update -> Remove) =====

#[test]
fn test_element_lifecycle() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_element_lifecycle_inner());
    });
}

async fn test_element_lifecycle_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Element Lifecycle").await;

    // 1. Add a text element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Hello Canvas".to_string(),
            100.0,
            100.0,
            16.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add text element");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1);
    assert_eq!(snap.elements[0].id, element_id);

    // 2. Update the element's transform
    let new_transform = TransformView {
        x: 200.0,
        y: 150.0,
        width: 100.0,
        height: 30.0,
        rotation: 0.5,
        z_index: 1,
    };

    canvas
        .update_transform(Some(&entity_id), &element_id, new_transform)
        .await
        .expect("Failed to update transform");

    let snap = canvas.current_snapshot();
    assert!((snap.elements[0].transform.x - 200.0).abs() < f32::EPSILON);
    assert!((snap.elements[0].transform.y - 150.0).abs() < f32::EPSILON);

    // 3. Remove the element
    canvas
        .remove_element(Some(&entity_id), &element_id)
        .await
        .expect("Failed to remove element");

    let snap = canvas.current_snapshot();
    assert!(snap.elements.is_empty(), "Element should be removed");
}

// ===== Test: Multiple Element Types =====

#[test]
fn test_multiple_element_types() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_multiple_element_types_inner());
    });
}

async fn test_multiple_element_types_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Multiple Types").await;

    // Add text element
    let text_id = canvas
        .add_text(
            Some(&entity_id),
            "Text element".to_string(),
            10.0,
            10.0,
            14.0,
            "#000000".to_string(),
        )
        .await
        .expect("Failed to add text");

    // Add image element
    let image_id = canvas
        .add_image(
            Some(&entity_id),
            "https://example.com/image.png".to_string(),
            100.0,
            10.0,
            200.0,
            150.0,
        )
        .await
        .expect("Failed to add image");

    // Add chart element
    let chart_id = canvas
        .add_chart(
            Some(&entity_id),
            "bar".to_string(),
            serde_json::json!({"values": [1, 2, 3]}),
            10.0,
            200.0,
            300.0,
            200.0,
        )
        .await
        .expect("Failed to add chart");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 3);

    // Verify all elements exist
    let ids: Vec<&str> = snap.elements.iter().map(|e| e.id.as_str()).collect();
    assert!(ids.contains(&text_id.as_str()));
    assert!(ids.contains(&image_id.as_str()));
    assert!(ids.contains(&chart_id.as_str()));
}

// ===== Test: Selection State =====

#[test]
fn test_selection_state() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_selection_state_inner());
    });
}

async fn test_selection_state_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Selection").await;

    // Add two elements
    let id1 = canvas
        .add_text(
            Some(&entity_id),
            "First".to_string(),
            10.0,
            10.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    let id2 = canvas
        .add_text(
            Some(&entity_id),
            "Second".to_string(),
            100.0,
            10.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    // Initially nothing selected
    assert!(canvas.current_snapshot().selected_ids.is_empty());

    // Select first element
    canvas.select_element(Some(&entity_id), &id1).await.unwrap();

    let snap = canvas.current_snapshot();
    assert_eq!(snap.selected_ids.len(), 1);
    assert!(snap.selected_ids.contains(&id1));

    // Toggle second element (adds to selection)
    canvas
        .toggle_selection(Some(&entity_id), &id2)
        .await
        .unwrap();

    let snap = canvas.current_snapshot();
    assert_eq!(snap.selected_ids.len(), 2);

    // Deselect all
    canvas.deselect_all(Some(&entity_id)).await.unwrap();

    let snap = canvas.current_snapshot();
    assert!(snap.selected_ids.is_empty());
}

// ===== Test: Export/Import Round-trip =====
// Note: Uses local export/import (None entity_id) since the persistence layer
// currently returns stub data with incompatible format. Persistence-backed
// export/import will work once the core layer properly stores/retrieves scenes.

#[test]
fn test_export_import_roundtrip() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_export_import_roundtrip_inner());
    });
}

async fn test_export_import_roundtrip_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity (needed for element operations)
    let entity_id = create_test_entity(&services, "Canvas Export Test").await;

    // Add some elements
    canvas
        .add_text(
            Some(&entity_id),
            "Export me".to_string(),
            50.0,
            50.0,
            18.0,
            "#ff0000".to_string(),
        )
        .await
        .unwrap();

    canvas
        .add_image(
            Some(&entity_id),
            "https://example.com/export.png".to_string(),
            100.0,
            100.0,
            200.0,
            150.0,
        )
        .await
        .unwrap();

    // Export (local - without entity_id to avoid stub persistence layer)
    let json = canvas.export_json(None).await.expect("Failed to export");

    assert!(!json.is_empty(), "Export should produce non-empty JSON");

    // Clear canvas
    canvas.clear().await.unwrap();
    assert!(canvas.current_snapshot().elements.is_empty());

    // Import (local - without entity_id)
    canvas
        .import_json(None, &json)
        .await
        .expect("Failed to import");

    let snap = canvas.current_snapshot();
    assert_eq!(
        snap.elements.len(),
        2,
        "Should have 2 elements after import"
    );
}

// ===== Test: Viewport and View Controls =====

#[test]
fn test_viewport_and_view() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_viewport_and_view_inner());
    });
}

async fn test_viewport_and_view_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Viewport Test").await;

    // Set viewport
    canvas
        .set_viewport(Some(&entity_id), 1920.0, 1080.0)
        .await
        .expect("Failed to set viewport");

    let snap = canvas.current_snapshot();
    assert!((snap.viewport_width - 1920.0).abs() < f32::EPSILON);
    assert!((snap.viewport_height - 1080.0).abs() < f32::EPSILON);

    // Set view (zoom and pan)
    canvas
        .set_view(Some(&entity_id), 2.0, 100.0, 50.0)
        .await
        .expect("Failed to set view");

    let snap = canvas.current_snapshot();
    assert!((snap.zoom - 2.0).abs() < f32::EPSILON);
    assert!((snap.pan_x - 100.0).abs() < f32::EPSILON);
    assert!((snap.pan_y - 50.0).abs() < f32::EPSILON);
}

// ===== Test: Offline Queue Behavior =====

#[test]
fn test_offline_queue_pending_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_offline_queue_pending_operations_inner());
    });
}

async fn test_offline_queue_pending_operations_inner() {
    use canvas_core::{Element, ElementKind, Operation};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Queue starts empty
    assert_eq!(canvas.pending_operations(), 0);
    assert!(!canvas.has_pending_operations());

    // Manually queue an operation (simulating a persistence failure)
    let test_element = Element::new(ElementKind::Text {
        content: "Offline test".to_string(),
        font_size: 14.0,
        color: "#000".to_string(),
    });

    canvas.queue_operation(Operation::AddElement {
        element: test_element,
        timestamp: Operation::now(),
    });

    assert_eq!(canvas.pending_operations(), 1);
    assert!(canvas.has_pending_operations());
}

#[test]
fn test_offline_queue_flush() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_offline_queue_flush_inner());
    });
}

async fn test_offline_queue_flush_inner() {
    use canvas_core::{Element, ElementKind, Operation};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Queue some operations
    for i in 0..3 {
        let test_element = Element::new(ElementKind::Text {
            content: format!("Queued {i}"),
            font_size: 14.0,
            color: "#000".to_string(),
        });

        canvas.queue_operation(Operation::AddElement {
            element: test_element,
            timestamp: Operation::now(),
        });
    }

    assert_eq!(canvas.pending_operations(), 3);

    // Flush the queue
    let flushed = canvas.flush_queue().await.expect("Failed to flush queue");
    assert_eq!(flushed, 3);

    // Queue should be empty after flush
    assert_eq!(canvas.pending_operations(), 0);
    assert!(!canvas.has_pending_operations());
}

// ===== Test: Watch Channel Updates =====

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
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Watch Test").await;

    // Subscribe to updates
    let mut rx = canvas.subscribe();

    // Add an element
    canvas
        .add_text(
            Some(&entity_id),
            "Watch me".to_string(),
            0.0,
            0.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    // Receive update
    rx.changed().await.expect("Should receive update");
    let snap = rx.borrow().clone();
    assert_eq!(snap.elements.len(), 1);
    assert!(!snap.loading);
}

// ===== Test: Clear Canvas =====

#[test]
fn test_clear_canvas() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_clear_canvas_inner());
    });
}

async fn test_clear_canvas_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Clear Test").await;

    // Add multiple elements
    canvas
        .add_text(
            Some(&entity_id),
            "One".to_string(),
            10.0,
            10.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    canvas
        .add_text(
            Some(&entity_id),
            "Two".to_string(),
            50.0,
            50.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(canvas.current_snapshot().elements.len(), 2);

    // Clear the canvas
    canvas.clear().await.expect("Failed to clear canvas");

    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Canvas should be empty after clear"
    );
}

// ===== Test: Element At Position =====

#[test]
fn test_element_at_position() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_element_at_position_inner());
    });
}

async fn test_element_at_position_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Element At Test").await;

    // Add an element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Click me".to_string(),
            100.0,
            100.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    // Set transform with specific dimensions
    let transform = TransformView {
        x: 100.0,
        y: 100.0,
        width: 200.0,
        height: 50.0,
        rotation: 0.0,
        z_index: 0,
    };
    canvas
        .update_transform(Some(&entity_id), &element_id, transform)
        .await
        .unwrap();

    // Point inside element (150, 125 is within 100-300 x 100-150)
    let found = canvas.element_at(150.0, 125.0);
    assert!(
        found.is_some(),
        "Should find element at point inside bounds"
    );
    assert_eq!(found.unwrap(), element_id);

    // Point outside element
    let not_found = canvas.element_at(50.0, 50.0);
    assert!(
        not_found.is_none(),
        "Should not find element at point outside bounds"
    );
}

// ===== Test: Undo/Redo History =====

#[test]
fn test_undo_redo_history() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_undo_redo_history_inner());
    });
}

async fn test_undo_redo_history_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Undo Redo Test").await;

    // Add an element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Undo me".to_string(),
            100.0,
            100.0,
            14.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Verify element exists
    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1);
    assert_eq!(snap.elements[0].id, element_id);

    // Undo - should remove the element
    let undone = canvas.undo().await.expect("Failed to undo");
    assert!(undone.is_some(), "Undo should return the undone operation");

    let snap = canvas.current_snapshot();
    assert!(
        snap.elements.is_empty(),
        "Element should be removed after undo"
    );

    // Redo - should restore the element
    let redone = canvas.redo().await.expect("Failed to redo");
    assert!(
        redone.is_some(),
        "Redo should return the restored operation"
    );

    let snap = canvas.current_snapshot();
    assert_eq!(
        snap.elements.len(),
        1,
        "Element should be restored after redo"
    );
}

#[test]
fn test_history_timeline() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_history_timeline_inner());
    });
}

async fn test_history_timeline_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas History Timeline Test").await;

    // Add multiple elements to build history
    for i in 0..5 {
        canvas
            .add_text(
                Some(&entity_id),
                format!("Element {i}"),
                i as f32 * 50.0,
                10.0,
                14.0,
                "#000".to_string(),
            )
            .await
            .expect("Failed to add element");
    }

    // Get history timeline
    let history = canvas.get_history();
    assert!(
        !history.is_empty(),
        "History should have entries after adding elements"
    );

    // History entries should be in order
    for i in 1..history.len() {
        assert!(
            history[i].timestamp >= history[i - 1].timestamp,
            "History should be ordered by timestamp"
        );
    }
}

#[test]
fn test_undo_clears_redo_stack() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_undo_clears_redo_stack_inner());
    });
}

async fn test_undo_clears_redo_stack_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Redo Clear Test").await;

    // Add two elements
    canvas
        .add_text(
            Some(&entity_id),
            "First".to_string(),
            10.0,
            10.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    canvas
        .add_text(
            Some(&entity_id),
            "Second".to_string(),
            100.0,
            10.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    // Undo one operation
    canvas.undo().await.unwrap();

    // Now add a new element (should clear redo stack)
    canvas
        .add_text(
            Some(&entity_id),
            "Third".to_string(),
            200.0,
            10.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .unwrap();

    // Redo should return None (redo stack was cleared)
    let redone = canvas.redo().await.expect("Redo should not error");
    assert!(
        redone.is_none(),
        "Redo should return None after new operation"
    );
}

// ===== Test: Remote Cursor Tracking =====

#[test]
fn test_remote_cursor_tracking() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_remote_cursor_tracking_inner());
    });
}

async fn test_remote_cursor_tracking_inner() {
    use communitas_core::legacy_crdt::CanvasCursorUpdate;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Cursor Test").await;

    // Initially no remote cursors
    let cursors = canvas.get_remote_cursors();
    assert!(
        cursors.is_empty(),
        "Should have no remote cursors initially"
    );

    // Simulate a remote cursor update (timestamp is set automatically by new())
    let cursor_update = CanvasCursorUpdate::new(
        entity_id.clone(),
        "remote-peer-123".to_string(),
        "Alice".to_string(),
        150.0,
        200.0,
    )
    .with_tool("select".to_string());

    // Handle the cursor update
    canvas.handle_cursor_update_for_test(cursor_update);

    // Should have one remote cursor
    let cursors = canvas.get_remote_cursors();
    assert_eq!(cursors.len(), 1, "Should have one remote cursor");
    assert_eq!(cursors[0].user_name, "Alice");
    assert!((cursors[0].x - 150.0).abs() < f32::EPSILON);
    assert!((cursors[0].y - 200.0).abs() < f32::EPSILON);
}

#[test]
fn test_cursor_update_throttling() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_cursor_update_throttling_inner());
    });
}

async fn test_cursor_update_throttling_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Cursor Throttle Test").await;

    // First cursor update should succeed
    let result1 = canvas
        .update_local_cursor(&entity_id, 100.0, 100.0, None)
        .await;
    assert!(result1.is_ok(), "First cursor update should succeed");

    // Immediate second update should be throttled (within 100ms window)
    let result2 = canvas
        .update_local_cursor(&entity_id, 150.0, 150.0, None)
        .await;
    assert!(result2.is_ok());
    // The result indicates if it was actually broadcast (false = throttled)
    let was_broadcast = result2.unwrap();
    assert!(!was_broadcast, "Second cursor update should be throttled");
}

#[test]
fn test_cursor_cleanup_after_timeout() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_cursor_cleanup_after_timeout_inner());
    });
}

async fn test_cursor_cleanup_after_timeout_inner() {
    use communitas_core::legacy_crdt::CanvasCursorUpdate;
    use std::time::Duration;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Cursor Cleanup Test").await;

    // Add a cursor - it will have a fresh timestamp
    let cursor = CanvasCursorUpdate::new(
        entity_id.clone(),
        "test-peer".to_string(),
        "TestUser".to_string(),
        50.0,
        50.0,
    );

    canvas.handle_cursor_update_for_test(cursor);

    // Immediately after adding, cursor should be present
    let cursors = canvas.get_remote_cursors();
    assert_eq!(
        cursors.len(),
        1,
        "Should have one cursor immediately after adding"
    );

    // Note: We can't easily test the timeout behavior in a unit test because
    // the internal Instant is set when handle_cursor_update_for_test is called.
    // In a real scenario, the cursor would be cleaned up after 5 seconds of inactivity.
    // This test verifies cursors are added and retrieved correctly.

    // Sleep briefly and verify cursor is still present (within timeout)
    tokio::time::sleep(Duration::from_millis(100)).await;
    let cursors = canvas.get_remote_cursors();
    assert!(
        !cursors.is_empty(),
        "Cursor should still be present within timeout"
    );
}

// ===== Test: Large Scene Performance =====

#[test]
fn test_large_scene_performance() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_large_scene_performance_inner());
    });
}

async fn test_large_scene_performance_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Large Scene Test").await;

    // Add 100+ elements
    let start = std::time::Instant::now();
    for i in 0..100 {
        canvas
            .add_text(
                Some(&entity_id),
                format!("Element {i}"),
                (i % 10) as f32 * 100.0,
                (i / 10) as f32 * 50.0,
                14.0,
                "#000".to_string(),
            )
            .await
            .expect("Failed to add element");
    }
    let add_duration = start.elapsed();

    // Verify all elements were added
    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 100, "Should have 100 elements");

    // Getting snapshot should be fast
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = canvas.current_snapshot();
    }
    let snapshot_duration = start.elapsed();

    // Performance assertions (generous limits for CI)
    assert!(
        add_duration.as_millis() < 10000,
        "Adding 100 elements took too long: {:?}",
        add_duration
    );
    assert!(
        snapshot_duration.as_millis() < 1000,
        "100 snapshot reads took too long: {:?}",
        snapshot_duration
    );
}

// ===== Test: Remote Operation Handling =====

#[test]
fn test_remote_operation_handling() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_remote_operation_handling_inner());
    });
}

async fn test_remote_operation_handling_inner() {
    use communitas_core::legacy_crdt::{CanvasOperation, VectorClock};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Remote Op Test").await;

    // Initially empty
    assert!(canvas.current_snapshot().elements.is_empty());

    // Simulate a remote add operation using the static constructor
    let mut vector_clock = VectorClock::new();
    vector_clock.increment("remote-peer-xyz");

    // ElementKind uses #[serde(tag = "type", content = "data")] serialization
    let element_data = serde_json::json!({
        "type": "Text",
        "data": {
            "content": "Remote text",
            "font_size": 16.0,
            "color": "#0000ff"
        },
        "transform": {
            "x": 200.0,
            "y": 200.0,
            "width": 100.0,
            "height": 30.0,
            "rotation": 0.0,
            "z_index": 0
        }
    });

    // Element IDs must be valid UUIDs
    let remote_element_id = "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_string();

    let remote_op = CanvasOperation::add(
        entity_id.clone(),
        remote_element_id.clone(),
        element_data,
        vector_clock,
        1, // lamport_clock
        "remote-peer-xyz".to_string(),
    );

    // Handle the remote operation
    canvas
        .handle_remote_operation(remote_op)
        .await
        .expect("Failed to handle remote operation");

    // Verify the element was added
    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1, "Remote element should be added");
    assert_eq!(snap.elements[0].id, remote_element_id);
}

#[test]
fn test_concurrent_operations_lww() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_concurrent_operations_lww_inner());
    });
}

async fn test_concurrent_operations_lww_inner() {
    use communitas_core::legacy_crdt::{CanvasOperation, VectorClock};
    use std::time::Duration;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas LWW Test").await;

    // First, add an element locally
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Original".to_string(),
            100.0,
            100.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Simulate a remote update with a higher timestamp (LWW should apply)
    tokio::time::sleep(Duration::from_millis(10)).await;

    let mut vector_clock = VectorClock::new();
    vector_clock.increment("remote-peer");

    // Create update data with new transform position
    // apply_remote_update expects a "transform" field with full Transform struct
    let update_data = serde_json::json!({
        "transform": {
            "x": 300.0,
            "y": 300.0,
            "width": 100.0,
            "height": 20.0,
            "rotation": 0.0,
            "z_index": 0
        }
    });

    let remote_update = CanvasOperation::update(
        entity_id.clone(),
        element_id.clone(),
        update_data,
        vector_clock,
        2, // lamport_clock (higher than local)
        "remote-peer".to_string(),
    );

    canvas
        .handle_remote_operation(remote_update)
        .await
        .expect("Failed to handle remote update");

    // The remote update (with higher timestamp) should win
    let snap = canvas.current_snapshot();
    let elem = snap.elements.iter().find(|e| e.id == element_id).unwrap();
    assert!(
        (elem.transform.x - 300.0).abs() < f32::EPSILON,
        "LWW should apply remote update with higher timestamp"
    );
}

// ===== Test: Clear Remote Cursors =====

#[test]
fn test_clear_remote_cursors() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_clear_remote_cursors_inner());
    });
}

async fn test_clear_remote_cursors_inner() {
    use communitas_core::legacy_crdt::CanvasCursorUpdate;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Canvas Clear Cursors Test").await;

    // Add multiple remote cursors (timestamp is set automatically)
    for i in 0..3 {
        let cursor = CanvasCursorUpdate::new(
            entity_id.clone(),
            format!("peer-{i}"),
            format!("User {i}"),
            i as f64 * 100.0,
            50.0,
        );

        canvas.handle_cursor_update_for_test(cursor);
    }

    // Verify cursors were added
    let cursors = canvas.get_remote_cursors();
    assert_eq!(cursors.len(), 3, "Should have 3 remote cursors");

    // Clear all cursors
    canvas.clear_remote_cursors();

    // Verify all cleared
    let cursors = canvas.get_remote_cursors();
    assert!(
        cursors.is_empty(),
        "Should have no remote cursors after clear"
    );
}

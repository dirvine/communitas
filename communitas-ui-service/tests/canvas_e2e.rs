// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end tests for canvas collaboration workflows.
//!
//! These tests verify complete canvas workflows through the CanvasService layer,
//! ensuring that drawing, element manipulation, undo/redo, collaboration features,
//! and offline sync work correctly.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::EntityType;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
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
    // Wait for background auth watcher to reinitialize with authenticated peer_id.
    services.wait_ready().await;

    services
}

/// Helper to create a test entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let canvas = services.canvas();
    let app = canvas.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Person,
        description: Some("Test entity for canvas E2E tests".to_string()),
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

// =============================================================================
// Test 1: Draw Rectangle Element
// =============================================================================

#[test]
fn test_draw_rectangle_element() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_draw_rectangle_element_inner());
    });
}

async fn test_draw_rectangle_element_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Rectangle Test").await;

    // Add an image element to simulate a rectangle (canvas uses image/chart/text elements)
    let element_id = canvas
        .add_image(
            Some(&entity_id),
            "data:image/svg+xml,<svg></svg>".to_string(),
            100.0,
            100.0,
            200.0,
            150.0,
        )
        .await
        .expect("Failed to add rectangle-like element");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1, "Should have one element");
    assert_eq!(snap.elements[0].id, element_id);
    assert!(
        (snap.elements[0].transform.width - 200.0).abs() < f32::EPSILON,
        "Width should be 200"
    );
    assert!(
        (snap.elements[0].transform.height - 150.0).abs() < f32::EPSILON,
        "Height should be 150"
    );
}

// =============================================================================
// Test 2: Draw Circle/Ellipse Element (via Chart)
// =============================================================================

#[test]
fn test_draw_circle_element() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_draw_circle_element_inner());
    });
}

async fn test_draw_circle_element_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Circle Test").await;

    // Add a chart element to represent a circular shape
    let element_id = canvas
        .add_chart(
            Some(&entity_id),
            "pie".to_string(), // Circular chart type
            serde_json::json!({"values": [100]}),
            150.0,
            150.0,
            100.0,
            100.0,
        )
        .await
        .expect("Failed to add circle-like element");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1, "Should have one element");
    assert_eq!(snap.elements[0].id, element_id);
}

// =============================================================================
// Test 3: Draw Line/Path Element (via Text marker)
// =============================================================================

#[test]
fn test_draw_line_element() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_draw_line_element_inner());
    });
}

async fn test_draw_line_element_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Line Test").await;

    // Add a text element as a line marker
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "---".to_string(), // Line representation
            50.0,
            50.0,
            14.0,
            "#000000".to_string(),
        )
        .await
        .expect("Failed to add line-like element");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1, "Should have one element");
    assert_eq!(snap.elements[0].id, element_id);
}

// =============================================================================
// Test 4: Select Element
// =============================================================================

#[test]
fn test_select_element() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_select_element_inner());
    });
}

async fn test_select_element_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Select Test").await;

    // Add an element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Select me".to_string(),
            100.0,
            100.0,
            16.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Initially nothing selected
    assert!(
        canvas.current_snapshot().selected_ids.is_empty(),
        "Nothing should be selected initially"
    );

    // Select the element
    canvas
        .select_element(Some(&entity_id), &element_id)
        .await
        .expect("Failed to select element");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.selected_ids.len(), 1, "One element should be selected");
    assert!(
        snap.selected_ids.contains(&element_id),
        "The correct element should be selected"
    );
}

// =============================================================================
// Test 5: Move Element
// =============================================================================

#[test]
fn test_move_element() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_move_element_inner());
    });
}

async fn test_move_element_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Move Test").await;

    // Add an element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Move me".to_string(),
            100.0,
            100.0,
            16.0,
            "#0000ff".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Update transform to new position
    let new_transform = TransformView {
        x: 300.0,
        y: 250.0,
        width: 100.0,
        height: 30.0,
        rotation: 0.0,
        z_index: 0,
    };

    canvas
        .update_transform(Some(&entity_id), &element_id, new_transform)
        .await
        .expect("Failed to update transform");

    let snap = canvas.current_snapshot();
    let elem = snap.elements.iter().find(|e| e.id == element_id).unwrap();
    assert!(
        (elem.transform.x - 300.0).abs() < f32::EPSILON,
        "X position should be updated"
    );
    assert!(
        (elem.transform.y - 250.0).abs() < f32::EPSILON,
        "Y position should be updated"
    );
}

// =============================================================================
// Test 6: Delete Element
// =============================================================================

#[test]
fn test_delete_element() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_delete_element_inner());
    });
}

async fn test_delete_element_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Delete Test").await;

    // Add an element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Delete me".to_string(),
            100.0,
            100.0,
            16.0,
            "#00ff00".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Verify element exists
    assert_eq!(
        canvas.current_snapshot().elements.len(),
        1,
        "Should have one element"
    );

    // Delete the element
    canvas
        .remove_element(Some(&entity_id), &element_id)
        .await
        .expect("Failed to remove element");

    // Verify element is removed
    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Canvas should be empty after delete"
    );
}

// =============================================================================
// Test 7: Undo Operation
// =============================================================================

#[test]
fn test_undo_operation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_undo_operation_inner());
    });
}

async fn test_undo_operation_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Undo Test").await;

    // Add an element
    let _element_id = canvas
        .add_text(
            Some(&entity_id),
            "Undo me".to_string(),
            100.0,
            100.0,
            16.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Verify element exists
    assert_eq!(
        canvas.current_snapshot().elements.len(),
        1,
        "Should have one element"
    );

    // Undo the add operation
    let undone = canvas.undo().await.expect("Undo should succeed");
    assert!(undone.is_some(), "Undo should return the undone operation");

    // Verify element is removed
    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Element should be removed after undo"
    );
}

// =============================================================================
// Test 8: Redo Operation
// =============================================================================

#[test]
fn test_redo_operation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_redo_operation_inner());
    });
}

async fn test_redo_operation_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Redo Test").await;

    // Add an element
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Redo me".to_string(),
            100.0,
            100.0,
            16.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Undo the add
    canvas.undo().await.expect("Undo should succeed");
    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Element should be removed after undo"
    );

    // Redo the add
    let redone = canvas.redo().await.expect("Redo should succeed");
    assert!(
        redone.is_some(),
        "Redo should return the restored operation"
    );

    // Verify element is restored
    let snap = canvas.current_snapshot();
    assert_eq!(snap.elements.len(), 1, "Element should be restored");
    assert_eq!(
        snap.elements[0].id, element_id,
        "Same element should be restored"
    );
}

// =============================================================================
// Test 9: Layer Ordering (z-index)
// =============================================================================

#[test]
fn test_layer_ordering() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_layer_ordering_inner());
    });
}

async fn test_layer_ordering_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Layer Test").await;

    // Add first element (background)
    let bg_id = canvas
        .add_text(
            Some(&entity_id),
            "Background".to_string(),
            100.0,
            100.0,
            16.0,
            "#cccccc".to_string(),
        )
        .await
        .expect("Failed to add background");

    // Add second element (foreground)
    let fg_id = canvas
        .add_text(
            Some(&entity_id),
            "Foreground".to_string(),
            100.0,
            100.0,
            16.0,
            "#000000".to_string(),
        )
        .await
        .expect("Failed to add foreground");

    // Update foreground to have higher z-index
    let fg_transform = TransformView {
        x: 100.0,
        y: 100.0,
        width: 100.0,
        height: 30.0,
        rotation: 0.0,
        z_index: 10, // Higher z-index
    };

    canvas
        .update_transform(Some(&entity_id), &fg_id, fg_transform)
        .await
        .expect("Failed to update foreground z-index");

    let snap = canvas.current_snapshot();
    let bg = snap.elements.iter().find(|e| e.id == bg_id).unwrap();
    let fg = snap.elements.iter().find(|e| e.id == fg_id).unwrap();

    assert!(
        fg.transform.z_index > bg.transform.z_index,
        "Foreground should have higher z-index"
    );
}

// =============================================================================
// Test 10: Shared Cursor Display
// =============================================================================

#[test]
fn test_shared_cursor_display() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_shared_cursor_display_inner());
    });
}

async fn test_shared_cursor_display_inner() {
    use communitas_core::legacy_crdt::CanvasCursorUpdate;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Cursor Test").await;

    // Initially no remote cursors
    assert!(
        canvas.get_remote_cursors().is_empty(),
        "Should have no remote cursors initially"
    );

    // Simulate a remote cursor update
    let cursor_update = CanvasCursorUpdate::new(
        entity_id.clone(),
        "remote-peer-123".to_string(),
        "Alice".to_string(),
        250.0,
        300.0,
    )
    .with_tool("pen".to_string());

    canvas.handle_cursor_update_for_test(cursor_update);

    // Verify cursor is displayed
    let cursors = canvas.get_remote_cursors();
    assert_eq!(cursors.len(), 1, "Should have one remote cursor");
    assert_eq!(cursors[0].user_name, "Alice");
    assert!((cursors[0].x - 250.0).abs() < f32::EPSILON);
    assert!((cursors[0].y - 300.0).abs() < f32::EPSILON);
}

// =============================================================================
// Test 11: Concurrent Edits (Simulate 2 Users)
// =============================================================================

#[test]
fn test_concurrent_edits() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_concurrent_edits_inner());
    });
}

async fn test_concurrent_edits_inner() {
    use communitas_core::legacy_crdt::{CanvasOperation, VectorClock};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Concurrent Test").await;

    // User 1 adds an element locally
    let local_id = canvas
        .add_text(
            Some(&entity_id),
            "Local edit".to_string(),
            100.0,
            100.0,
            16.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add local element");

    // User 2 adds an element remotely
    let mut vector_clock = VectorClock::new();
    vector_clock.increment("remote-user-2");

    let remote_element_id = "b2c3d4e5-f6a7-8901-bcde-f12345678901".to_string();
    let element_data = serde_json::json!({
        "type": "Text",
        "data": {
            "content": "Remote edit",
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

    let remote_op = CanvasOperation::add(
        entity_id.clone(),
        remote_element_id.clone(),
        element_data,
        vector_clock,
        1,
        "remote-user-2".to_string(),
    );

    canvas
        .handle_remote_operation(remote_op)
        .await
        .expect("Failed to handle remote operation");

    // Both elements should exist
    let snap = canvas.current_snapshot();
    assert_eq!(
        snap.elements.len(),
        2,
        "Should have both local and remote elements"
    );

    let ids: Vec<&str> = snap.elements.iter().map(|e| e.id.as_str()).collect();
    assert!(
        ids.contains(&local_id.as_str()),
        "Local element should exist"
    );
    assert!(
        ids.contains(&remote_element_id.as_str()),
        "Remote element should exist"
    );
}

// =============================================================================
// Test 12: Offline Edits Queue
// =============================================================================

#[test]
fn test_offline_edits_queue() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_offline_edits_queue_inner());
    });
}

async fn test_offline_edits_queue_inner() {
    use canvas_core::{Element, ElementKind, Operation};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Queue starts empty
    assert_eq!(canvas.pending_operations(), 0, "Queue should start empty");
    assert!(
        !canvas.has_pending_operations(),
        "Should not have pending operations"
    );

    // Simulate offline by queuing operations directly
    let test_element = Element::new(ElementKind::Text {
        content: "Offline edit".to_string(),
        font_size: 14.0,
        color: "#000".to_string(),
    });

    canvas.queue_operation(Operation::AddElement {
        element: test_element,
        timestamp: Operation::now(),
    });

    // Verify operation is queued
    assert_eq!(
        canvas.pending_operations(),
        1,
        "Should have one pending operation"
    );
    assert!(
        canvas.has_pending_operations(),
        "Should have pending operations"
    );
}

// =============================================================================
// Test 13: Offline Sync on Reconnect
// =============================================================================

#[test]
fn test_offline_sync_on_reconnect() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_offline_sync_on_reconnect_inner());
    });
}

async fn test_offline_sync_on_reconnect_inner() {
    use canvas_core::{Element, ElementKind, Operation};

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    // Queue multiple offline operations
    for i in 0..5 {
        let test_element = Element::new(ElementKind::Text {
            content: format!("Offline {i}"),
            font_size: 14.0,
            color: "#000".to_string(),
        });

        canvas.queue_operation(Operation::AddElement {
            element: test_element,
            timestamp: Operation::now(),
        });
    }

    assert_eq!(
        canvas.pending_operations(),
        5,
        "Should have 5 pending operations"
    );

    // Simulate reconnection by flushing the queue
    let flushed = canvas.flush_queue().await.expect("Flush should succeed");
    assert_eq!(flushed, 5, "Should flush all 5 operations");

    // Queue should be empty after flush
    assert_eq!(
        canvas.pending_operations(),
        0,
        "Queue should be empty after flush"
    );
}

// =============================================================================
// Test 14: History Scrubbing (Timeline)
// =============================================================================

#[test]
fn test_history_scrubbing() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_history_scrubbing_inner());
    });
}

async fn test_history_scrubbing_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas History Test").await;

    // Build up history by adding elements
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
        "History should have entries after operations"
    );

    // History should be chronologically ordered
    for i in 1..history.len() {
        assert!(
            history[i].timestamp >= history[i - 1].timestamp,
            "History entries should be chronologically ordered"
        );
    }

    // Scrub back through history (undo multiple times)
    for _ in 0..3 {
        canvas.undo().await.expect("Undo should succeed");
    }

    // Should have 2 elements remaining
    let snap = canvas.current_snapshot();
    assert_eq!(
        snap.elements.len(),
        2,
        "Should have 2 elements after 3 undos from 5"
    );
}

// =============================================================================
// Test 15: CRDT Parity - UI Operations Match Core
// =============================================================================

#[test]
fn test_crdt_parity() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_crdt_parity_inner());
    });
}

async fn test_crdt_parity_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Parity Test").await;

    // Perform operations through UI service
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Parity test".to_string(),
            100.0,
            100.0,
            16.0,
            "#ff0000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Export the canvas state
    let exported = canvas
        .export_json(None)
        .await
        .expect("Failed to export canvas");

    // Verify exported JSON contains our element
    assert!(
        exported.contains(&element_id),
        "Exported JSON should contain the element ID"
    );

    // Clear and import to verify round-trip
    canvas.clear().await.expect("Clear should succeed");
    assert!(
        canvas.current_snapshot().elements.is_empty(),
        "Canvas should be empty after clear"
    );

    canvas
        .import_json(None, &exported)
        .await
        .expect("Import should succeed");

    // Verify element is restored correctly
    let snap = canvas.current_snapshot();
    assert_eq!(
        snap.elements.len(),
        1,
        "Should have one element after import"
    );
    assert_eq!(
        snap.elements[0].id, element_id,
        "Same element should be restored"
    );
}

// =============================================================================
// Test 16: Multiple Selection (Toggle)
// =============================================================================

#[test]
fn test_multiple_selection() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_multiple_selection_inner());
    });
}

async fn test_multiple_selection_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas MultiSelect Test").await;

    // Add multiple elements
    let id1 = canvas
        .add_text(
            Some(&entity_id),
            "First".to_string(),
            50.0,
            50.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .expect("Failed to add first element");

    let id2 = canvas
        .add_text(
            Some(&entity_id),
            "Second".to_string(),
            150.0,
            50.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .expect("Failed to add second element");

    let id3 = canvas
        .add_text(
            Some(&entity_id),
            "Third".to_string(),
            250.0,
            50.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .expect("Failed to add third element");

    // Select first element
    canvas
        .select_element(Some(&entity_id), &id1)
        .await
        .expect("Failed to select first");

    // Toggle second element (adds to selection)
    canvas
        .toggle_selection(Some(&entity_id), &id2)
        .await
        .expect("Failed to toggle second");

    let snap = canvas.current_snapshot();
    assert_eq!(snap.selected_ids.len(), 2, "Should have two selected");
    assert!(snap.selected_ids.contains(&id1));
    assert!(snap.selected_ids.contains(&id2));
    assert!(!snap.selected_ids.contains(&id3));

    // Deselect all
    canvas
        .deselect_all(Some(&entity_id))
        .await
        .expect("Failed to deselect all");

    assert!(
        canvas.current_snapshot().selected_ids.is_empty(),
        "All should be deselected"
    );
}

// =============================================================================
// Test 17: Viewport and Zoom Controls
// =============================================================================

#[test]
fn test_viewport_and_zoom() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_viewport_and_zoom_inner());
    });
}

async fn test_viewport_and_zoom_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas Viewport Test").await;

    // Set viewport dimensions
    canvas
        .set_viewport(Some(&entity_id), 1920.0, 1080.0)
        .await
        .expect("Failed to set viewport");

    let snap = canvas.current_snapshot();
    assert!((snap.viewport_width - 1920.0).abs() < f32::EPSILON);
    assert!((snap.viewport_height - 1080.0).abs() < f32::EPSILON);

    // Set zoom and pan
    canvas
        .set_view(Some(&entity_id), 2.0, 100.0, 50.0)
        .await
        .expect("Failed to set view");

    let snap = canvas.current_snapshot();
    assert!((snap.zoom - 2.0).abs() < f32::EPSILON, "Zoom should be 2x");
    assert!(
        (snap.pan_x - 100.0).abs() < f32::EPSILON,
        "Pan X should be 100"
    );
    assert!(
        (snap.pan_y - 50.0).abs() < f32::EPSILON,
        "Pan Y should be 50"
    );
}

// =============================================================================
// Test 18: LWW Conflict Resolution
// =============================================================================

#[test]
fn test_lww_conflict_resolution() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_lww_conflict_resolution_inner());
    });
}

async fn test_lww_conflict_resolution_inner() {
    use communitas_core::legacy_crdt::{CanvasOperation, VectorClock};
    use std::time::Duration;

    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let canvas = services.canvas();

    let entity_id = create_test_entity(&services, "Canvas LWW Test").await;

    // Add element locally at position (100, 100)
    let element_id = canvas
        .add_text(
            Some(&entity_id),
            "Conflict test".to_string(),
            100.0,
            100.0,
            14.0,
            "#000".to_string(),
        )
        .await
        .expect("Failed to add element");

    // Wait a bit to ensure timestamp difference
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Simulate a remote update with higher timestamp (should win via LWW)
    let mut vector_clock = VectorClock::new();
    vector_clock.increment("remote-peer");

    let update_data = serde_json::json!({
        "transform": {
            "x": 500.0,
            "y": 500.0,
            "width": 100.0,
            "height": 30.0,
            "rotation": 0.0,
            "z_index": 0
        }
    });

    let remote_update = CanvasOperation::update(
        entity_id.clone(),
        element_id.clone(),
        update_data,
        vector_clock,
        2, // Higher lamport clock
        "remote-peer".to_string(),
    );

    canvas
        .handle_remote_operation(remote_update)
        .await
        .expect("Failed to handle remote update");

    // LWW should apply the remote update (higher timestamp wins)
    let snap = canvas.current_snapshot();
    let elem = snap.elements.iter().find(|e| e.id == element_id).unwrap();
    assert!(
        (elem.transform.x - 500.0).abs() < f32::EPSILON,
        "Remote update should win via LWW"
    );
    assert!(
        (elem.transform.y - 500.0).abs() < f32::EPSILON,
        "Remote update should win via LWW"
    );
}

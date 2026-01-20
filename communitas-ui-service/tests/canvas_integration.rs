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

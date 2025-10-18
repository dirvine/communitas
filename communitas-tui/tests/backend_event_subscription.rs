//! Integration tests for Backend event subscription system
//!
//! These tests verify that the Backend properly integrates with CoreContext's event streaming
//! for real-time updates of entities, messages, and network state.
//!
//! Test Strategy:
//! - Use real CoreContext with test storage
//! - Verify event subscription and delivery
//! - Test event filtering by entity type and ID
//! - Test multiple subscribers
//! - Test unsubscription
//!
//! Note: Tests currently blocked by CoreContext stack overflow issue.
//! Tests are written following TDD RED-GREEN-REFACTOR methodology.

use anyhow::Result;
use communitas_core::crdt::EntityType;
use communitas_tui::backend::{Backend, BackendEvent};
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Create a test backend with authenticated CoreContext
///
/// Note: Currently experiences stack overflow - will be fixed in communitas-core
async fn create_test_backend() -> Result<(Backend, TempDir)> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let mut backend = Backend::new(data_dir.clone(), false).await?;

    // Create and login to a test vault
    backend
        .create_vault("ocean-forest-moon-star", "test-password-123", "Test User")
        .await?;

    // Initialize CoreContext
    backend.initialize_core_context().await?;

    Ok((backend, temp_dir))
}

// =============================================================================
// Basic Subscription Tests
// =============================================================================

#[tokio::test]
async fn test_subscribe_to_entity_events() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Subscribe to entity events
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);

    backend.subscribe_entity_events(tx).await?;

    // Create an entity - should trigger EntityCreated event
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Wait for event with timeout
    let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed");

    // Verify event
    match event {
        BackendEvent::EntityCreated {
            entity_id,
            entity_type,
            name,
        } => {
            assert_eq!(entity_id, entity.id);
            assert_eq!(entity_type, EntityType::Channel);
            assert_eq!(name, "Test Channel");
        }
        _ => panic!("Expected EntityCreated event, got {:?}", event),
    }

    Ok(())
}

#[tokio::test]
async fn test_subscribe_to_message_events() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create a channel first
    let channel = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Subscribe to message events
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    backend.subscribe_message_events(tx).await?;

    // Send a message - should trigger MessageSent event
    let message_id = backend
        .send_message(
            channel.id.clone(),
            EntityType::Channel,
            "Hello, world!".to_string(),
        )
        .await?;

    // Wait for event
    let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed");

    // Verify event
    match event {
        BackendEvent::MessageSent {
            message_id: msg_id,
            entity_id,
        } => {
            assert_eq!(msg_id, message_id);
            assert_eq!(entity_id, channel.id);
        }
        _ => panic!("Expected MessageSent event, got {:?}", event),
    }

    Ok(())
}

#[tokio::test]
async fn test_unsubscribe_stops_events() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Subscribe to entity events
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    let subscription_id = backend.subscribe_entity_events(tx).await?;

    // Create an entity - should receive event
    backend
        .create_entity(
            "First Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify first event received
    let _event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for first event")
        .expect("Channel closed");

    // Unsubscribe
    backend.unsubscribe(subscription_id).await?;

    // Create another entity - should NOT receive event
    backend
        .create_entity(
            "Second Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify no event received (timeout expected)
    let result = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(result.is_err(), "Expected timeout, but received event");

    Ok(())
}

// =============================================================================
// Event Filtering Tests
// =============================================================================

#[tokio::test]
async fn test_filter_events_by_entity_type() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Subscribe to only Channel events
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    backend
        .subscribe_entity_events_filtered(tx, Some(EntityType::Channel), None)
        .await?;

    // Create a channel - should receive event
    backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify channel event received
    let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for channel event")
        .expect("Channel closed");

    assert!(matches!(
        event,
        BackendEvent::EntityCreated {
            entity_type: EntityType::Channel,
            ..
        }
    ));

    // Create a group - should NOT receive event (filtered out)
    backend
        .create_entity(
            "Test Group".to_string(),
            EntityType::Group,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify no event received (timeout expected)
    let result = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(
        result.is_err(),
        "Expected timeout for filtered group event, but received event"
    );

    Ok(())
}

#[tokio::test]
async fn test_filter_events_by_entity_id() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create a channel first
    let channel = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Subscribe to events for specific channel only
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    backend
        .subscribe_entity_events_filtered(tx, None, Some(channel.id.clone()))
        .await?;

    // Add member to our channel - should receive event
    backend
        .add_entity_member(
            EntityType::Channel,
            &channel.id,
            "alice-test-user-one".to_string(),
        )
        .await?;

    // Verify event received
    let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for member event")
        .expect("Channel closed");

    match event {
        BackendEvent::MemberAdded {
            entity_id,
            member_id,
            ..
        } => {
            assert_eq!(entity_id, channel.id);
            assert_eq!(member_id, "alice-test-user-one");
        }
        _ => panic!("Expected MemberAdded event, got {:?}", event),
    }

    // Create another channel - should NOT receive event (different entity_id)
    let _other_channel = backend
        .create_entity(
            "Other Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify no event received
    let result = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(
        result.is_err(),
        "Expected timeout for different entity, but received event"
    );

    Ok(())
}

// =============================================================================
// Multiple Subscribers Tests
// =============================================================================

#[tokio::test]
async fn test_multiple_subscribers_receive_same_event() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create two subscribers
    let (tx1, mut rx1) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    let (tx2, mut rx2) = tokio::sync::mpsc::channel::<BackendEvent>(100);

    backend.subscribe_entity_events(tx1).await?;
    backend.subscribe_entity_events(tx2).await?;

    // Create an entity
    let entity = backend
        .create_entity(
            "Test Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Both subscribers should receive the event
    let event1 = timeout(Duration::from_secs(1), rx1.recv())
        .await
        .expect("Timeout waiting for subscriber 1")
        .expect("Channel closed");

    let event2 = timeout(Duration::from_secs(1), rx2.recv())
        .await
        .expect("Timeout waiting for subscriber 2")
        .expect("Channel closed");

    // Verify both received same event
    assert_eq!(event1, event2);

    match event1 {
        BackendEvent::EntityCreated { entity_id, .. } => {
            assert_eq!(entity_id, entity.id);
        }
        _ => panic!("Expected EntityCreated event"),
    }

    Ok(())
}

// =============================================================================
// Member Events Tests
// =============================================================================

#[tokio::test]
async fn test_member_added_event() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create a group
    let group = backend
        .create_entity(
            "Test Group".to_string(),
            EntityType::Group,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Subscribe to entity events
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    backend.subscribe_entity_events(tx).await?;

    // Add a member - should trigger MemberAdded event
    backend
        .add_entity_member(
            EntityType::Group,
            &group.id,
            "alice-test-one".to_string(),
        )
        .await?;

    // Wait for event
    let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for event")
        .expect("Channel closed");

    // Verify event
    match event {
        BackendEvent::MemberAdded {
            entity_id,
            entity_type,
            member_id,
        } => {
            assert_eq!(entity_id, group.id);
            assert_eq!(entity_type, EntityType::Group);
            assert_eq!(member_id, "alice-test-one");
        }
        _ => panic!("Expected MemberAdded event, got {:?}", event),
    }

    Ok(())
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_subscribe_without_core_context_fails() -> Result<()> {
    // Create backend without initializing CoreContext
    let temp_dir = TempDir::new()?;
    let mut backend = Backend::new(temp_dir.path().to_path_buf(), false).await?;

    // Try to subscribe - should fail
    let (tx, _rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    let result = backend.subscribe_entity_events(tx).await;

    assert!(
        result.is_err(),
        "Expected error when subscribing without CoreContext"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("CoreContext not initialized") || err_msg.contains("not initialized"),
        "Error message should mention CoreContext: {}",
        err_msg
    );

    Ok(())
}

// =============================================================================
// Event Persistence Tests (for offline support)
// =============================================================================

#[tokio::test]
async fn test_events_queued_when_subscriber_offline() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Enable event queueing for offline support
    backend.enable_event_queue(100).await?; // Queue up to 100 events

    // Create an entity BEFORE subscribing (simulating offline)
    let entity = backend
        .create_entity(
            "Offline Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Now subscribe - should receive queued event
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    backend.subscribe_entity_events(tx).await?;

    // Verify queued event received
    let event = timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("Timeout waiting for queued event")
        .expect("Channel closed");

    match event {
        BackendEvent::EntityCreated { entity_id, .. } => {
            assert_eq!(entity_id, entity.id);
        }
        _ => panic!("Expected EntityCreated event from queue"),
    }

    Ok(())
}

#[tokio::test]
async fn test_event_queue_size_limit() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Enable small event queue (only 2 events)
    backend.enable_event_queue(2).await?;

    // Create 3 entities (exceeds queue size)
    for i in 0..3 {
        backend
            .create_entity(
                format!("Entity {}", i),
                EntityType::Channel,
                vec!["ocean-forest-moon-star".to_string()],
            )
            .await?;
    }

    // Subscribe and receive queued events
    let (tx, mut rx) = tokio::sync::mpsc::channel::<BackendEvent>(100);
    backend.subscribe_entity_events(tx).await?;

    // Should receive only last 2 events (oldest dropped)
    let mut received_count = 0;
    while let Ok(Some(_event)) = timeout(Duration::from_millis(500), rx.recv()).await {
        received_count += 1;
    }

    assert_eq!(
        received_count, 2,
        "Should receive only 2 queued events (queue limit)"
    );

    Ok(())
}

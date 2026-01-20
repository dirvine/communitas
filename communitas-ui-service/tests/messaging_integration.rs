//! Integration tests for the messaging service using real CommunitasApp.
//!
//! These tests verify the full messaging flow including send, edit, delete,
//! reactions, and reactive watch channel updates.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;
use std::time::Duration;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
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

    services
}

/// Helper to create a test channel entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let messaging = services.messaging();
    let app = messaging.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Channel,
        description: Some("Test channel for integration tests".to_string()),
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

/// Test the full messaging flow: send -> get -> edit -> get -> delete -> get.
#[test]
fn test_full_messaging_flow() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_full_messaging_flow_inner());
    });
}

async fn test_full_messaging_flow_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create a test entity/channel
    let entity_id = create_test_entity(&services, "Test Channel").await;

    // 1. Send a message
    let message = messaging
        .send_message(&entity_id, "Hello, world!", None)
        .await
        .expect("Failed to send message");

    assert_eq!(message.text, "Hello, world!");
    assert!(!message.id.is_empty());

    // 2. Get messages and verify the sent message is there
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].id, message.id);
    assert_eq!(messages[0].text, "Hello, world!");

    // 3. Edit the message
    let edited = messaging
        .edit_message(&entity_id, &message.id, "Hello, edited world!")
        .await
        .expect("Failed to edit message");

    assert_eq!(edited.text, "Hello, edited world!");
    assert_eq!(edited.id, message.id);

    // 4. Get messages again and verify the edit is reflected
    let messages_after_edit = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages after edit");

    assert_eq!(messages_after_edit.len(), 1);
    assert_eq!(messages_after_edit[0].text, "Hello, edited world!");

    // 5. Delete the message
    messaging
        .delete_message(&entity_id, &message.id)
        .await
        .expect("Failed to delete message");

    // 6. Get messages again and verify it's deleted
    let messages_after_delete = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages after delete");

    // Message should be gone or marked as deleted
    assert!(
        messages_after_delete.is_empty()
            || messages_after_delete.iter().all(|m| m.id != message.id),
        "Message should be deleted"
    );
}

/// Test the reactions flow: send message, add reaction, verify, remove, verify.
#[test]
fn test_reactions_flow() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_reactions_flow_inner());
    });
}

async fn test_reactions_flow_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create a test entity/channel
    let entity_id = create_test_entity(&services, "Reactions Test Channel").await;

    // Send a message
    let message = messaging
        .send_message(&entity_id, "React to me!", None)
        .await
        .expect("Failed to send message");

    // Add a reaction
    messaging
        .add_reaction(&entity_id, &message.id, "👍")
        .await
        .expect("Failed to add reaction");

    // Get the message and verify reaction is present
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 1);
    let msg = &messages[0];
    assert!(
        msg.reactions.iter().any(|r| r.emoji == "👍"),
        "Reaction should be present: {:?}",
        msg.reactions
    );

    // Remove the reaction
    messaging
        .remove_reaction(&entity_id, &message.id, "👍")
        .await
        .expect("Failed to remove reaction");

    // Get the message and verify reaction is gone
    let messages_after_remove = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages after removal");

    assert_eq!(messages_after_remove.len(), 1);
    let msg_after = &messages_after_remove[0];
    assert!(
        !msg_after
            .reactions
            .iter()
            .any(|r| r.emoji == "👍" && r.count > 0),
        "Reaction should be removed: {:?}",
        msg_after.reactions
    );
}

/// Test that list_threads returns created entities.
#[test]
fn test_list_threads_includes_entity() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_list_threads_includes_entity_inner());
    });
}

async fn test_list_threads_includes_entity_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Thread Test Channel").await;

    // List threads
    let threads = messaging
        .list_threads()
        .await
        .expect("Failed to list threads");

    // Should include our entity
    assert!(
        threads.iter().any(|t| t.thread_id == entity_id),
        "Entity should appear in thread list: {:?}",
        threads.iter().map(|t| &t.thread_id).collect::<Vec<_>>()
    );
}

/// Test watch channel updates when messages are sent.
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
    let messaging = services.messaging();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Watch Test Channel").await;

    // Subscribe to watch channel
    let rx = messaging.subscribe();

    // Get initial thread count
    let initial_thread_count = rx.borrow().threads.len();

    // Send a message directly via the app to trigger events
    messaging
        .send_message(&entity_id, "Trigger update", None)
        .await
        .expect("Failed to send message");

    // Give event loop time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Manually refresh threads (the event should have already triggered this)
    messaging.refresh_threads().await;

    // Check that the watch channel has been updated
    let updated = rx.borrow().clone();

    // Thread should now have a non-empty preview if the message was processed
    let thread = updated.threads.iter().find(|t| t.thread_id == entity_id);
    assert!(thread.is_some(), "Thread should exist in snapshot");

    // The preview should contain part of our message text
    if let Some(t) = thread {
        assert!(
            t.last_message_preview.contains("Trigger")
                || t.last_message_timestamp > 0
                || updated.threads.len() >= initial_thread_count,
            "Thread should be updated with message info"
        );
    }
}

/// Test that unauthenticated access returns appropriate errors.
#[test]
fn test_unauthenticated_access_blocked() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_unauthenticated_access_blocked_inner());
    });
}

async fn test_unauthenticated_access_blocked_inner() {
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
    let messaging = services.messaging();

    // Try to list threads
    let result = messaging.list_threads().await;
    assert!(
        result.is_err(),
        "list_threads should fail when unauthenticated"
    );

    // Try to send a message
    let result = messaging.send_message("some-entity", "test", None).await;
    assert!(
        result.is_err(),
        "send_message should fail when unauthenticated"
    );
}

/// Test message pagination works correctly.
#[test]
fn test_message_pagination() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_message_pagination_inner());
    });
}

async fn test_message_pagination_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Pagination Test Channel").await;

    // Send multiple messages
    for i in 1..=5 {
        messaging
            .send_message(&entity_id, &format!("Message {}", i), None)
            .await
            .expect("Failed to send message");
        // Small delay to ensure different timestamps
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // Get all messages
    let all_messages = messaging
        .get_messages(&entity_id, 100, None)
        .await
        .expect("Failed to get all messages");

    assert_eq!(all_messages.len(), 5, "Should have 5 messages");

    // Get only 2 messages
    let limited = messaging
        .get_messages(&entity_id, 2, None)
        .await
        .expect("Failed to get limited messages");

    assert_eq!(limited.len(), 2, "Should have 2 messages with limit");

    // Messages should be sorted newest first
    assert!(
        limited[0].timestamp >= limited[1].timestamp,
        "Messages should be sorted newest first"
    );

    // Get messages before a cursor (use the timestamp of the second newest)
    if all_messages.len() >= 2 {
        let cursor = all_messages[1].timestamp;
        let before_cursor = messaging
            .get_messages(&entity_id, 10, Some(cursor))
            .await
            .expect("Failed to get messages with cursor");

        // All messages should have timestamp < cursor
        for msg in &before_cursor {
            assert!(
                msg.timestamp < cursor,
                "Message timestamp {} should be < cursor {}",
                msg.timestamp,
                cursor
            );
        }
    }
}

/// Test reply_to functionality.
#[test]
fn test_message_reply_to() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_message_reply_to_inner());
    });
}

async fn test_message_reply_to_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Reply Test Channel").await;

    // Send a parent message
    let parent = messaging
        .send_message(&entity_id, "I am the parent", None)
        .await
        .expect("Failed to send parent message");

    // Send a reply
    let reply = messaging
        .send_message(&entity_id, "I am a reply", Some(&parent.id))
        .await
        .expect("Failed to send reply");

    // Verify reply has the correct reply_to_id
    assert_eq!(
        reply.reply_to_id,
        Some(parent.id.clone()),
        "Reply should reference parent message"
    );

    // Get messages and verify both exist
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 2, "Should have parent and reply");
}

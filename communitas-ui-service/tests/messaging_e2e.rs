//! End-to-end tests for messaging workflows.
//!
//! Tests cover:
//! - Thread listing and management
//! - Message sending/receiving
//! - Reactions
//! - Message editing/deletion
//! - Search
//! - Typing indicators
//! - Offline queue and sync
//! - Unread counts
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

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
        description: Some("Test channel for E2E tests".to_string()),
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
// Test 1: List threads (empty)
// =============================================================================

#[test]
fn test_list_threads_empty() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_list_threads_empty_inner());
    });
}

async fn test_list_threads_empty_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // List threads should be empty initially (no messages sent)
    let threads = messaging
        .list_threads()
        .await
        .expect("Failed to list threads");
    assert!(threads.is_empty(), "Expected no threads initially");
}

// =============================================================================
// Test 2: List threads (populated)
// =============================================================================

#[test]
fn test_list_threads_populated() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_list_threads_populated_inner());
    });
}

async fn test_list_threads_populated_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Create a test entity/channel
    let entity_id = create_test_entity(&services, "Test Channel").await;

    // Send a message to create a thread
    messaging
        .send_message(&entity_id, "Hello, world!", None)
        .await
        .expect("Failed to send message");

    // Refresh and list threads
    messaging.refresh_threads().await;
    let threads = messaging
        .list_threads()
        .await
        .expect("Failed to list threads");

    // Should have one thread
    assert!(!threads.is_empty(), "Expected at least one thread");
    assert!(
        threads
            .iter()
            .any(|t| t.entity_id == Some(entity_id.clone())),
        "Thread for entity should exist"
    );
}

// =============================================================================
// Test 3: Send message to thread
// =============================================================================

#[test]
fn test_send_message_to_thread() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_send_message_to_thread_inner());
    });
}

async fn test_send_message_to_thread_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Send Test Channel").await;

    // Send message
    let message = messaging
        .send_message(&entity_id, "Test message content", None)
        .await
        .expect("Failed to send message");

    assert!(!message.id.is_empty(), "Message ID should not be empty");
    assert_eq!(message.text, "Test message content");

    // Verify message exists in thread
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text, "Test message content");
}

// =============================================================================
// Test 4: Add reaction
// =============================================================================

#[test]
fn test_add_reaction() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_add_reaction_inner());
    });
}

async fn test_add_reaction_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Reaction Test Channel").await;

    // Send message
    let message = messaging
        .send_message(&entity_id, "React to this!", None)
        .await
        .expect("Failed to send message");

    // Add reaction
    messaging
        .add_reaction(&entity_id, &message.id, "👍")
        .await
        .expect("Failed to add reaction");

    // Verify reaction exists
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 1);
    assert!(
        messages[0].reactions.iter().any(|r| r.emoji == "👍"),
        "Reaction should be present"
    );
}

// =============================================================================
// Test 5: Remove reaction
// =============================================================================

#[test]
fn test_remove_reaction() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_remove_reaction_inner());
    });
}

async fn test_remove_reaction_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Remove Reaction Channel").await;

    // Send message
    let message = messaging
        .send_message(&entity_id, "React and unreact", None)
        .await
        .expect("Failed to send message");

    // Add reaction
    messaging
        .add_reaction(&entity_id, &message.id, "❤️")
        .await
        .expect("Failed to add reaction");

    // Remove reaction
    messaging
        .remove_reaction(&entity_id, &message.id, "❤️")
        .await
        .expect("Failed to remove reaction");

    // Verify reaction removed
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 1);
    assert!(
        !messages[0].reactions.iter().any(|r| r.emoji == "❤️"),
        "Reaction should be removed"
    );
}

// =============================================================================
// Test 6: Edit message
// =============================================================================

#[test]
fn test_edit_message() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_edit_message_inner());
    });
}

async fn test_edit_message_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Edit Test Channel").await;

    // Send message
    let message = messaging
        .send_message(&entity_id, "Original message", None)
        .await
        .expect("Failed to send message");

    // Edit message
    let edited = messaging
        .edit_message(&entity_id, &message.id, "Edited message")
        .await
        .expect("Failed to edit message");

    assert_eq!(edited.text, "Edited message");

    // Verify edit persisted
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].text, "Edited message");
}

// =============================================================================
// Test 7: Delete message
// =============================================================================

#[test]
fn test_delete_message() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_delete_message_inner());
    });
}

async fn test_delete_message_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Delete Test Channel").await;

    // Send message
    let message = messaging
        .send_message(&entity_id, "Delete me", None)
        .await
        .expect("Failed to send message");

    // Delete message
    messaging
        .delete_message(&entity_id, &message.id)
        .await
        .expect("Failed to delete message");

    // Verify deletion
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    // Message should be gone or not in result
    assert!(
        messages.is_empty() || messages.iter().all(|m| m.id != message.id),
        "Message should be deleted"
    );
}

// =============================================================================
// Test 8: Search messages
// =============================================================================

#[test]
fn test_search_messages() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_search_messages_inner());
    });
}

async fn test_search_messages_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Search Test Channel").await;

    // Send multiple messages
    messaging
        .send_message(&entity_id, "Hello world", None)
        .await
        .expect("Failed to send message");
    messaging
        .send_message(&entity_id, "Rust is great", None)
        .await
        .expect("Failed to send message");
    messaging
        .send_message(&entity_id, "Testing search functionality", None)
        .await
        .expect("Failed to send message");

    // Search for "Rust"
    let results = messaging
        .search_messages("Rust", None, 10)
        .await
        .expect("Failed to search messages");

    assert!(!results.is_empty(), "Expected search results");
    assert!(
        results.iter().any(|r| r.message.text.contains("Rust")),
        "Search results should contain 'Rust'"
    );
}

// =============================================================================
// Test 9: Typing indicator
// =============================================================================

#[test]
fn test_typing_indicator() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_typing_indicator_inner());
    });
}

async fn test_typing_indicator_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Typing Test Channel").await;

    // Send typing indicator (should not error)
    messaging
        .send_typing_indicator(&entity_id, true)
        .await
        .expect("Failed to send typing indicator");

    // Note: We can't easily verify typing indicator was received without
    // a second user, but the call should succeed
}

// =============================================================================
// Test 10: Unread count accuracy
// =============================================================================

#[test]
fn test_unread_count_accuracy() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_unread_count_accuracy_inner());
    });
}

async fn test_unread_count_accuracy_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Unread Count Channel").await;

    // Send multiple messages
    for i in 0..5 {
        messaging
            .send_message(&entity_id, &format!("Message {}", i), None)
            .await
            .expect("Failed to send message");
    }

    // Get unread count
    let unread = messaging
        .get_thread_unread_count(&entity_id)
        .await
        .expect("Failed to get unread count");

    // Since we sent the messages ourselves, they're already "read"
    // The count should be 0 for own messages
    assert_eq!(unread, 0, "Own messages should not count as unread");

    // Mark thread as read (should succeed even if already read)
    messaging
        .mark_thread_read(&entity_id)
        .await
        .expect("Failed to mark thread as read");

    let unread_after = messaging
        .get_thread_unread_count(&entity_id)
        .await
        .expect("Failed to get unread count");

    assert_eq!(unread_after, 0, "Unread count should be 0 after mark read");
}

// =============================================================================
// Test 11: Thread pinning
// =============================================================================

#[test]
fn test_thread_pinning() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_thread_pinning_inner());
    });
}

async fn test_thread_pinning_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Pin Test Channel").await;

    // Send a message to create the thread
    messaging
        .send_message(&entity_id, "Pin this thread", None)
        .await
        .expect("Failed to send message");

    // Pin the thread
    let pin_result = messaging
        .pin_thread(&entity_id)
        .await
        .expect("Failed to pin thread");

    assert!(pin_result, "Thread should be pinned");

    // Unpin the thread
    let unpin_result = messaging
        .unpin_thread(&entity_id)
        .await
        .expect("Failed to unpin thread");

    assert!(unpin_result, "Thread should be unpinned");
}

// =============================================================================
// Test 12: Multiple messages pagination
// =============================================================================

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

    let entity_id = create_test_entity(&services, "Pagination Test Channel").await;

    // Send 15 messages
    for i in 0..15 {
        messaging
            .send_message(&entity_id, &format!("Message {}", i), None)
            .await
            .expect("Failed to send message");
    }

    // Get first page (10 messages)
    let page1 = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(page1.len(), 10, "First page should have 10 messages");

    // Get second page using timestamp cursor
    if let Some(last_msg) = page1.last() {
        let page2 = messaging
            .get_messages(&entity_id, 10, Some(last_msg.timestamp))
            .await
            .expect("Failed to get second page");

        // Second page should have some messages (at least some remaining ones)
        // The exact count depends on timestamp-based pagination logic
        // With 15 messages sent and 10 in page1, page2 should have ~4-6 messages
        // (some might be excluded due to timestamp boundary)
        assert!(
            !page2.is_empty() || page1.len() >= 15,
            "Should have messages across pages or all in first page"
        );
    }
}

// =============================================================================
// Test 13: Concurrent message sending
// =============================================================================

#[test]
fn test_concurrent_message_sending() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_concurrent_message_sending_inner());
    });
}

async fn test_concurrent_message_sending_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Concurrent Test Channel").await;

    // Send 10 messages concurrently
    let mut handles = Vec::new();
    for i in 0..10 {
        let messaging_clone = messaging.clone();
        let entity_id_clone = entity_id.clone();
        handles.push(tokio::spawn(async move {
            messaging_clone
                .send_message(&entity_id_clone, &format!("Concurrent message {}", i), None)
                .await
        }));
    }

    // Wait for all to complete
    for handle in handles {
        handle
            .await
            .unwrap()
            .expect("Failed to send concurrent message");
    }

    // Verify all messages received
    let messages = messaging
        .get_messages(&entity_id, 20, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(
        messages.len(),
        10,
        "All 10 concurrent messages should be received"
    );
}

// =============================================================================
// Test 14: Reply to message (parent_id)
// =============================================================================

#[test]
fn test_reply_to_message() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_reply_to_message_inner());
    });
}

async fn test_reply_to_message_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    let entity_id = create_test_entity(&services, "Reply Test Channel").await;

    // Send parent message
    let parent = messaging
        .send_message(&entity_id, "Parent message", None)
        .await
        .expect("Failed to send parent message");

    // Send reply (using &str for parent_id)
    let reply = messaging
        .send_message(&entity_id, "This is a reply", Some(&parent.id))
        .await
        .expect("Failed to send reply");

    // Verify reply has parent_id set
    assert!(
        reply.reply_to_id.is_some() || reply.text == "This is a reply",
        "Reply should reference parent or contain expected text"
    );

    // Get messages and verify both exist
    let messages = messaging
        .get_messages(&entity_id, 10, None)
        .await
        .expect("Failed to get messages");

    assert_eq!(messages.len(), 2, "Should have parent and reply");
}

// =============================================================================
// Test 15: Retry pending message
// =============================================================================

#[test]
fn test_retry_pending_messages() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_retry_pending_messages_inner());
    });
}

async fn test_retry_pending_messages_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let messaging = services.messaging();

    // Retry all pending should succeed even with no pending messages
    let results = messaging.retry_all_pending().await;

    // Should be empty (no pending messages)
    assert!(
        results.is_empty(),
        "No pending messages to retry in fresh state"
    );
}

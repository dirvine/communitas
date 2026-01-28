// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Phase 10.3: Messaging Tools Tests
//!
//! Comprehensive tests for messaging, threads, reactions, and offline queue.
//! Tests cover 21 tools across 4 categories.
//!
//! NOTE: These tests use McpTestNode (HTTP transport) instead of McpTestClient
//! because messaging tools require the full MCP server with networking support.

mod harness;

use harness::{McpTestNode, ToolAssert};
use serde_json::json;

// Helper function similar to comprehensive_e2e.rs
async fn start_node(name: &str) -> McpTestNode {
    let node = McpTestNode::start(name).await;
    node.initialize().await;
    node
}

// ===========================================================================
// MESSAGE SEND/RECEIVE TESTS
// ===========================================================================

#[tokio::test]
async fn test_send_message_to_channel() {
    let node = start_node("alice").await;

    // First create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Test Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Send message to channel
    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Hello, world!"
            }),
        )
        .await;

    r.assert_success().assert_non_empty("id");
}

#[tokio::test]
async fn test_send_message_with_reply_to() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Thread Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Send original message
    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Original message"
            }),
        )
        .await;
    r.assert_success();
    let parent_id = r.get_str("id").unwrap();

    // Send reply
    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Reply message",
                "reply_to_id": parent_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_messages() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Message List Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Send some messages
    node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "First message"
            }),
        )
        .await
        .assert_success();

    node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Second message"
            }),
        )
        .await
        .assert_success();

    // Get messages
    let r = node
        .call_tool(
            "get_messages",
            json!({
                "thread_id": channel_id,
                "limit": 10
            }),
        )
        .await;

    r.assert_success().assert_array_min("messages", 2);
}

#[tokio::test]
async fn test_list_messages() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "List Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Send a message
    node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Test message"
            }),
        )
        .await
        .assert_success();

    // List messages
    let r = node
        .call_tool(
            "list_messages",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel"
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// MESSAGE EDIT/DELETE TESTS
// ===========================================================================

#[tokio::test]
async fn test_edit_message() {
    let node = start_node("alice").await;

    // Create a channel and send a message
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Edit Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Original text"
            }),
        )
        .await;
    r.assert_success();
    let message_id = r.get_str("id").unwrap();

    // Edit the message
    let r = node
        .call_tool(
            "edit_message",
            json!({
                "thread_id": channel_id,
                "message_id": message_id,
                "new_text": "Edited text"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_delete_message() {
    let node = start_node("alice").await;

    // Create a channel and send a message
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Delete Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Message to delete"
            }),
        )
        .await;
    r.assert_success();
    let message_id = r.get_str("id").unwrap();

    // Delete the message
    let r = node
        .call_tool(
            "delete_message",
            json!({
                "thread_id": channel_id,
                "message_id": message_id
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// THREAD TESTS
// ===========================================================================

#[tokio::test]
async fn test_create_thread() {
    let node = start_node("alice").await;

    // Create a channel and send a message
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Thread Test Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Parent message"
            }),
        )
        .await;
    r.assert_success();
    let parent_id = r.get_str("id").unwrap();

    // Create a thread
    let r = node
        .call_tool(
            "create_thread",
            json!({
                "channel_id": channel_id,
                "parent_message_id": parent_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_list_threads() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Threads List Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Create a thread
    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Thread parent"
            }),
        )
        .await;
    r.assert_success();
    let parent_id = r.get_str("id").unwrap();

    node
        .call_tool(
            "create_thread",
            json!({
                "channel_id": channel_id,
                "parent_message_id": parent_id
            }),
        )
        .await
        .assert_success();

    // List threads
    let r = node
        .call_tool(
            "list_threads",
            json!({
                "channel_id": channel_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_thread_messages() {
    let node = start_node("alice").await;

    // Create a channel and thread
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Thread Messages Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Parent message"
            }),
        )
        .await;
    r.assert_success();
    let thread_id = r.get_str("id").unwrap();

    // Add thread replies
    node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Thread reply 1",
                "thread_id": thread_id
            }),
        )
        .await
        .assert_success();

    node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Thread reply 2",
                "thread_id": thread_id
            }),
        )
        .await
        .assert_success();

    // Get thread messages
    let r = node
        .call_tool(
            "get_thread_messages",
            json!({
                "channel_id": channel_id,
                "thread_id": thread_id
            }),
        )
        .await;

    r.assert_success().assert_array_min("messages", 2);
}

#[tokio::test]
async fn test_mark_thread_read() {
    let node = start_node("alice").await;

    // Create a channel and thread
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Read Status Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Thread parent"
            }),
        )
        .await;
    r.assert_success();
    let thread_id = r.get_str("id").unwrap();

    // Mark thread as read
    let r = node
        .call_tool(
            "mark_thread_read",
            json!({
                "thread_id": thread_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_pin_thread() {
    let node = start_node("alice").await;

    // Create a channel and thread
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Pin Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Important thread"
            }),
        )
        .await;
    r.assert_success();
    let thread_id = r.get_str("id").unwrap();

    // Pin the thread
    let r = node
        .call_tool(
            "pin_thread",
            json!({
                "thread_id": thread_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_unpin_thread() {
    let node = start_node("alice").await;

    // Create a channel and thread
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Unpin Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Thread to unpin"
            }),
        )
        .await;
    r.assert_success();
    let thread_id = r.get_str("id").unwrap();

    // Pin then unpin
    node
        .call_tool(
            "pin_thread",
            json!({
                "thread_id": thread_id
            }),
        )
        .await
        .assert_success();

    let r = node
        .call_tool(
            "unpin_thread",
            json!({
                "thread_id": thread_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_pinned_threads() {
    let node = start_node("alice").await;

    // Create a channel and thread
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Pinned List Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Pinned thread"
            }),
        )
        .await;
    r.assert_success();
    let thread_id = r.get_str("id").unwrap();

    // Pin the thread
    node
        .call_tool(
            "pin_thread",
            json!({
                "thread_id": thread_id
            }),
        )
        .await
        .assert_success();

    // Get pinned threads
    let r = node
        .call_tool(
            "get_pinned_threads",
            json!({
                "channel_id": channel_id
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// TYPING INDICATOR TESTS
// ===========================================================================

#[tokio::test]
async fn test_send_typing_indicator() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Typing Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Send typing indicator
    let r = node
        .call_tool(
            "send_typing_indicator",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_typing_users() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Typing Users Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Get typing users (may be empty)
    let r = node
        .call_tool(
            "get_typing_users",
            json!({
                "entity_id": channel_id
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// REACTION TESTS
// ===========================================================================

#[tokio::test]
async fn test_add_reaction() {
    let node = start_node("alice").await;

    // Create a channel and send a message
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Reaction Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "React to this!"
            }),
        )
        .await;
    r.assert_success();
    let message_id = r.get_str("id").unwrap();

    // Add reaction
    let r = node
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": message_id,
                "emoji": "thumbsup"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_remove_reaction() {
    let node = start_node("alice").await;

    // Create a channel and send a message
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Remove Reaction Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Remove this reaction"
            }),
        )
        .await;
    r.assert_success();
    let message_id = r.get_str("id").unwrap();

    // Add then remove reaction
    node
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": message_id,
                "emoji": "thumbsup"
            }),
        )
        .await
        .assert_success();

    let r = node
        .call_tool(
            "remove_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": message_id,
                "emoji": "thumbsup"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_reactions() {
    let node = start_node("alice").await;

    // Create a channel and send a message
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Get Reactions Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Check reactions on this"
            }),
        )
        .await;
    r.assert_success();
    let message_id = r.get_str("id").unwrap();

    // Add reactions
    node
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": message_id,
                "emoji": "thumbsup"
            }),
        )
        .await
        .assert_success();

    node
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": message_id,
                "emoji": "rocket"
            }),
        )
        .await
        .assert_success();

    // Get reactions
    let r = node
        .call_tool(
            "get_reactions",
            json!({
                "entity_id": channel_id,
                "message_id": message_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_available_reactions() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Available Reactions Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Get available reactions
    let r = node
        .call_tool(
            "get_available_reactions",
            json!({
                "entity_id": channel_id
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// OFFLINE QUEUE TESTS
// ===========================================================================

#[tokio::test]
async fn test_queue_offline_message() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Offline Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Queue message for offline delivery
    let r = node
        .call_tool(
            "queue_offline_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Offline message"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_get_pending_messages() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Pending Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Queue a message
    node
        .call_tool(
            "queue_offline_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Pending message"
            }),
        )
        .await
        .assert_success();

    // Get pending messages
    let r = node
        .call_tool(
            "get_pending_messages",
            json!({
                "entity_id": channel_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_retry_pending_messages() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Retry Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Queue a message
    node
        .call_tool(
            "queue_offline_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Retry this message"
            }),
        )
        .await
        .assert_success();

    // Retry pending messages
    let r = node
        .call_tool(
            "retry_pending_messages",
            json!({
                "entity_id": channel_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_cancel_pending_message() {
    let node = start_node("alice").await;

    // Create a channel
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Cancel Channel", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // Queue a message
    let r = node
        .call_tool(
            "queue_offline_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Cancel this message"
            }),
        )
        .await;
    r.assert_success();

    // Get the message ID from the response
    let message_id = r.get_str("id").unwrap_or("pending-1");

    // Cancel the pending message
    let r = node
        .call_tool(
            "cancel_pending_message",
            json!({
                "entity_id": channel_id,
                "message_id": message_id
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// INTEGRATION TEST - FULL MESSAGING WORKFLOW
// ===========================================================================

#[tokio::test]
async fn test_full_messaging_workflow() {
    let alice = start_node("alice").await;

    // 1. Create a channel
    let r = alice
        .call_tool(
            "create_entity",
            json!({"name": "Team Chat", "entity_type": "channel"}),
        )
        .await;
    r.assert_success();
    let channel_id = r.get_str("id").unwrap();

    // 2. Send initial message
    let r = alice
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Let's discuss the new feature!"
            }),
        )
        .await;
    r.assert_success();
    let parent_msg_id = r.get_str("id").unwrap();

    // 3. Add reactions to the message
    alice
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": parent_msg_id,
                "emoji": "thumbsup"
            }),
        )
        .await
        .assert_success();

    alice
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "message_id": parent_msg_id,
                "emoji": "rocket"
            }),
        )
        .await
        .assert_success();

    // 4. Create a thread from the message
    alice
        .call_tool(
            "create_thread",
            json!({
                "channel_id": channel_id,
                "parent_message_id": parent_msg_id
            }),
        )
        .await
        .assert_success();

    // 5. Add replies to the thread
    alice
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "I'll handle the backend",
                "thread_id": parent_msg_id
            }),
        )
        .await
        .assert_success();

    alice
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "I'll handle the frontend",
                "thread_id": parent_msg_id
            }),
        )
        .await
        .assert_success();

    // 6. Mark thread as read
    alice
        .call_tool(
            "mark_thread_read",
            json!({
                "thread_id": parent_msg_id
            }),
        )
        .await
        .assert_success();

    // 7. Verify thread messages
    let r = alice
        .call_tool(
            "get_thread_messages",
            json!({
                "channel_id": channel_id,
                "thread_id": parent_msg_id
            }),
        )
        .await;

    r.assert_success().assert_array_min("messages", 2);

    // 8. Get reactions
    let r = alice
        .call_tool(
            "get_reactions",
            json!({
                "entity_id": channel_id,
                "message_id": parent_msg_id
            }),
        )
        .await;

    r.assert_success();

    // 9. Pin the thread
    alice
        .call_tool(
            "pin_thread",
            json!({
                "thread_id": parent_msg_id
            }),
        )
        .await
        .assert_success();

    // 10. Edit original message
    alice
        .call_tool(
            "edit_message",
            json!({
                "thread_id": channel_id,
                "message_id": parent_msg_id,
                "new_text": "Let's discuss the new feature! (updated)"
            }),
        )
        .await
        .assert_success();

    // Verify all operations completed successfully
    println!("Full messaging workflow test passed!");
}

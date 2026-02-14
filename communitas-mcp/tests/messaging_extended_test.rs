//! Extended Messaging Tests - Phase 10.3 Tasks 7-10
//!
//! Tests for typing indicators, reactions, offline queue, and read status
//! Run with: cargo test -p communitas-mcp --test messaging_extended_test

use serde_json::json;
use std::process::{Command, Stdio};
use tokio::time::sleep;

struct TestNode {
    #[allow(dead_code)]
    name: String,
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        // Use OS-assigned port to avoid collisions between concurrent test binaries
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut process = Command::new(env!("CARGO_BIN_EXE_communitas-mcp"))
            .args([
                "--http",
                "--demo",
                "--listen",
                &format!("127.0.0.1:{}", port),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");

        let client = reqwest::Client::new();
        for _ in 0..50 {
            sleep(std::time::Duration::from_millis(100)).await;
            if client
                .post(format!("http://127.0.0.1:{}/mcp", port))
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 0,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": name, "version": "1.0"}
                    }
                }))
                .send()
                .await
                .is_ok()
            {
                return Self {
                    name: name.to_string(),
                    process,
                    port,
                };
            }
        }
        let _ = process.kill();
        let _ = process.wait();
        panic!("Node {} failed to start", name);
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    async fn call_tool(&self, tool: &str, input: serde_json::Value) -> serde_json::Value {
        let client = reqwest::Client::new();
        client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": tool,
                    "arguments": input,
                }
            }))
            .send()
            .await
            .ok()
            .and_then(|r| futures::executor::block_on(r.json()).ok())
            .unwrap_or_else(|| json!({"error": "request failed"}))
    }

    async fn initialize(&self) {
        let _result = self.call_tool("get_profile", json!({})).await;
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

// ===== TASK 7: Read Status Tests =====

#[tokio::test]
async fn test_mark_thread_as_read() {
    let node = TestNode::start("test-read").await;
    node.initialize().await;

    // Create thread
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-read",
                "text": "Message for read test",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let thread_result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-read",
                    "message_id": msg_id
                }),
            )
            .await;

        if let Some(thread_id) = thread_result["result"]["thread_id"].as_str() {
            // Mark as read
            let read_result = node
                .call_tool(
                    "mark_thread_read",
                    json!({
                        "entity_id": "channel-read",
                        "thread_id": thread_id
                    }),
                )
                .await;

            assert!(
                read_result.get("result").is_some(),
                "mark_thread_read should succeed"
            );
        }
    }
}

// ===== TASK 8: Typing Indicator Tests =====

#[tokio::test]
async fn test_send_typing_indicator() {
    let node = TestNode::start("test-typing").await;
    node.initialize().await;

    let result = node
        .call_tool(
            "send_typing_indicator",
            json!({
                "entity_id": "channel-typing"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "send_typing_indicator should work"
    );
}

#[tokio::test]
async fn test_get_typing_users() {
    let node = TestNode::start("test-typing-users").await;
    node.initialize().await;

    // Send typing indicator
    let _typing = node
        .call_tool(
            "send_typing_indicator",
            json!({
                "entity_id": "channel-typing-users"
            }),
        )
        .await;

    // Get typing users
    let result = node
        .call_tool(
            "get_typing_users",
            json!({
                "entity_id": "channel-typing-users"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "get_typing_users should work"
    );
}

// ===== TASK 9: Reaction Tests =====

#[tokio::test]
async fn test_add_reaction() {
    let node = TestNode::start("test-reaction-add").await;
    node.initialize().await;

    // Send message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-reactions",
                "text": "Message for reactions",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Add reaction
        let reaction_result = node
            .call_tool(
                "add_reaction",
                json!({
                    "entity_id": "channel-reactions",
                    "message_id": msg_id,
                    "emoji": "👍"
                }),
            )
            .await;

        assert!(
            reaction_result.get("result").is_some(),
            "add_reaction should succeed"
        );
    }
}

#[tokio::test]
async fn test_remove_reaction() {
    let node = TestNode::start("test-reaction-remove").await;
    node.initialize().await;

    // Send message and add reaction
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-remove-reaction",
                "text": "Message for removal",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let _add = node
            .call_tool(
                "add_reaction",
                json!({
                    "entity_id": "channel-remove-reaction",
                    "message_id": msg_id,
                    "emoji": "😀"
                }),
            )
            .await;

        // Remove reaction
        let remove_result = node
            .call_tool(
                "remove_reaction",
                json!({
                    "entity_id": "channel-remove-reaction",
                    "message_id": msg_id,
                    "emoji": "😀"
                }),
            )
            .await;

        assert!(
            remove_result.get("result").is_some(),
            "remove_reaction should work"
        );
    }
}

#[tokio::test]
async fn test_get_reactions() {
    let node = TestNode::start("test-get-reactions").await;
    node.initialize().await;

    // Send message and add reactions
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-get-reactions",
                "text": "Reactive message",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let _add1 = node
            .call_tool(
                "add_reaction",
                json!({
                    "entity_id": "channel-get-reactions",
                    "message_id": msg_id,
                    "emoji": "👍"
                }),
            )
            .await;

        let _add2 = node
            .call_tool(
                "add_reaction",
                json!({
                    "entity_id": "channel-get-reactions",
                    "message_id": msg_id,
                    "emoji": "❤️"
                }),
            )
            .await;

        // Get all reactions
        let get_result = node
            .call_tool(
                "get_reactions",
                json!({
                    "entity_id": "channel-get-reactions",
                    "message_id": msg_id
                }),
            )
            .await;

        assert!(
            get_result.get("result").is_some(),
            "get_reactions should work"
        );
    }
}

// ===== TASK 10: Offline Queue Tests =====

#[tokio::test]
async fn test_get_pending_messages() {
    let node = TestNode::start("test-pending").await;
    node.initialize().await;

    let result = node
        .call_tool(
            "get_pending_messages",
            json!({
                "entity_id": "channel-pending"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "get_pending_messages should work"
    );
}

#[tokio::test]
async fn test_queue_offline_message() {
    let node = TestNode::start("test-queue-offline").await;
    node.initialize().await;

    let result = node
        .call_tool(
            "queue_offline_message",
            json!({
                "entity_id": "channel-queue",
                "content": "Offline message",
                "message_type": "text"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "queue_offline_message should work"
    );
}

#[tokio::test]
async fn test_retry_pending_messages() {
    let node = TestNode::start("test-retry-pending").await;
    node.initialize().await;

    // Queue a message
    let _queue = node
        .call_tool(
            "queue_offline_message",
            json!({
                "entity_id": "channel-retry",
                "content": "To be retried",
                "message_type": "text"
            }),
        )
        .await;

    // Retry pending
    let result = node
        .call_tool(
            "retry_pending_messages",
            json!({
                "entity_id": "channel-retry"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "retry_pending_messages should work"
    );
}

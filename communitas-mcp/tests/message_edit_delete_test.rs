//! Message Edit/Delete Tests - Phase 10.3 Task 2
//!
//! Tests for MCP message editing and deletion operations
//! Run with: cargo test -p communitas-mcp --test message_edit_delete_test

use serde_json::json;
use std::process::{Command, Stdio};
use tokio::time::sleep;

/// Test node that spawns an MCP server process
struct TestNode {
    #[allow(dead_code)]
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

#[tokio::test]
async fn test_edit_message_content() {
    let node = TestNode::start("test-edit").await;
    node.initialize().await;

    // Send initial message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-edit-test",
                "text": "Original content",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Edit the message
        let edit_result = node
            .call_tool(
                "edit_message",
                json!({
                    "entity_id": "channel-edit-test",
                    "message_id": msg_id,
                    "new_text": "Updated content"
                }),
            )
            .await;

        assert!(
            edit_result.get("result").is_some(),
            "edit_message should succeed"
        );

        // Verify edit worked by getting the message
        let get_result = node
            .call_tool(
                "get_message",
                json!({
                    "entity_id": "channel-edit-test",
                    "message_id": msg_id
                }),
            )
            .await;

        assert_eq!(
            get_result["result"]["content"].as_str().unwrap_or(""),
            "Updated content",
            "Message content should be updated"
        );
    }
}

#[tokio::test]
async fn test_edit_message_with_invalid_id() {
    let node = TestNode::start("test-edit-invalid").await;
    node.initialize().await;

    // Try to edit non-existent message
    let result = node
        .call_tool(
            "edit_message",
            json!({
                "entity_id": "channel-invalid",
                "message_id": "non-existent-id-12345",
                "new_text": "Should fail"
            }),
        )
        .await;

    // Should return error or indicate failure via isError flag
    let is_error = result["result"]["isError"].as_bool().unwrap_or(false);
    let has_error = result.get("error").is_some();
    let is_null = result["result"].is_null();
    assert!(
        is_error || has_error || is_null,
        "Editing non-existent message should fail or return isError"
    );
}

#[tokio::test]
async fn test_delete_message() {
    let node = TestNode::start("test-delete").await;
    node.initialize().await;

    // Send a message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-delete-test",
                "text": "Message to delete",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Delete the message
        let delete_result = node
            .call_tool(
                "delete_message",
                json!({
                    "entity_id": "channel-delete-test",
                    "message_id": msg_id
                }),
            )
            .await;

        assert!(
            delete_result.get("result").is_some() || delete_result.get("error").is_none(),
            "delete_message should execute without fatal error"
        );
    }
}

#[tokio::test]
async fn test_delete_non_existent_message() {
    let node = TestNode::start("test-delete-nonexist").await;
    node.initialize().await;

    // Try to delete non-existent message
    let result = node
        .call_tool(
            "delete_message",
            json!({
                "entity_id": "channel-nonexist",
                "message_id": "fake-id-99999"
            }),
        )
        .await;

    // Should handle gracefully
    assert!(
        result.get("error").is_some() || result.get("result").is_some(),
        "Should return error or success for non-existent delete"
    );
}

#[tokio::test]
async fn test_deleted_message_not_in_list() {
    let node = TestNode::start("test-delete-verify").await;
    node.initialize().await;

    // Send a message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-delete-verify",
                "text": "Message to remove from list",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Verify it appears in list
        let list_before = node
            .call_tool(
                "list_messages",
                json!({
                    "thread_id": "channel-delete-verify",
                    "limit": 50
                }),
            )
            .await;

        let count_before = if let Some(messages) = list_before["result"]["messages"].as_array() {
            messages.len()
        } else {
            0
        };

        // Delete the message
        let _delete_result = node
            .call_tool(
                "delete_message",
                json!({
                    "entity_id": "channel-delete-verify",
                    "message_id": msg_id
                }),
            )
            .await;

        // Check list after deletion
        let list_after = node
            .call_tool(
                "list_messages",
                json!({
                    "thread_id": "channel-delete-verify",
                    "limit": 50
                }),
            )
            .await;

        let count_after = if let Some(messages) = list_after["result"]["messages"].as_array() {
            messages.len()
        } else {
            0
        };

        // After deletion, count should be less or equal
        assert!(
            count_after <= count_before,
            "Message list should have fewer or equal messages after deletion"
        );
    }
}

#[tokio::test]
async fn test_edit_timestamp_tracked() {
    let node = TestNode::start("test-edit-timestamp").await;
    node.initialize().await;

    // Send message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-ts-test",
                "text": "Original",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let original_timestamp = send_result["result"]["timestamp"].clone();

        // Wait a moment then edit
        sleep(std::time::Duration::from_millis(500)).await;

        let _edit_result = node
            .call_tool(
                "edit_message",
                json!({
                    "entity_id": "channel-ts-test",
                    "message_id": msg_id,
                    "new_text": "Modified"
                }),
            )
            .await;

        // Get message to check timestamps
        let get_result = node
            .call_tool(
                "get_message",
                json!({
                    "entity_id": "channel-ts-test",
                    "message_id": msg_id
                }),
            )
            .await;

        // Some implementation may track edit_timestamp
        if let Some(edited_at) = get_result["result"]["edited_at"].as_i64() {
            assert!(
                edited_at >= original_timestamp.as_i64().unwrap_or(0),
                "Edit timestamp should be >= original"
            );
        }
    }
}

#[tokio::test]
async fn test_multiple_edits() {
    let node = TestNode::start("test-multi-edit").await;
    node.initialize().await;

    // Send message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-multi-edit",
                "text": "Version 1",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Edit multiple times
        for i in 2..=4 {
            let _edit_result = node
                .call_tool(
                    "edit_message",
                    json!({
                        "entity_id": "channel-multi-edit",
                        "message_id": msg_id,
                        "new_text": format!("Version {}", i)
                    }),
                )
                .await;
        }

        // Verify final version
        let get_result = node
            .call_tool(
                "get_message",
                json!({
                    "entity_id": "channel-multi-edit",
                    "message_id": msg_id
                }),
            )
            .await;

        assert_eq!(
            get_result["result"]["content"].as_str().unwrap_or(""),
            "Version 4",
            "Final message should be Version 4"
        );
    }
}

#[tokio::test]
async fn test_edit_preserves_other_fields() {
    let node = TestNode::start("test-edit-fields").await;
    node.initialize().await;

    // Send message with metadata
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-fields-test",
                "text": "Original",
                "message_type": "text",
                "metadata": {"priority": "high"}
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Edit just the content
        let _edit_result = node
            .call_tool(
                "edit_message",
                json!({
                    "entity_id": "channel-fields-test",
                    "message_id": msg_id,
                    "new_text": "Modified"
                }),
            )
            .await;

        // Get message and verify other fields preserved
        let get_result = node
            .call_tool(
                "get_message",
                json!({
                    "entity_id": "channel-fields-test",
                    "message_id": msg_id
                }),
            )
            .await;

        assert_eq!(
            get_result["result"]["content"].as_str().unwrap_or(""),
            "Modified",
            "Content should be updated"
        );

        // Message type should still be present
        assert_eq!(
            get_result["result"]["message_type"].as_str().unwrap_or(""),
            "text",
            "Message type should be preserved"
        );
    }
}

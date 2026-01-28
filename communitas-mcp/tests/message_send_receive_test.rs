//! Message Send/Receive Tests - Phase 10.3 Task 1
//!
//! Tests for MCP message tools interface validation
//! Run with: cargo test -p communitas-mcp --test message_send_receive_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::sleep;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Test node that spawns an MCP server process
struct TestNode {
    name: String,
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let port = 32000 + (std::process::id() % 1000) as u16 * 10 + counter;

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

    async fn request(&self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let client = reqwest::Client::new();
        client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> ToolResult {
        let response = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments
                }),
            )
            .await;

        let result = response.get("result").cloned().unwrap_or(json!(null));
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let parsed: Option<serde_json::Value> = serde_json::from_str(content).ok();

        ToolResult {
            success: !is_error,
            content: content.to_string(),
            parsed,
        }
    }

    async fn initialize(&self) {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": &self.name, "version": "1.0"}
            }),
        )
        .await;
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

#[derive(Debug)]
struct ToolResult {
    success: bool,
    content: String,
    parsed: Option<serde_json::Value>,
}

impl ToolResult {
    fn assert_success(&self) -> &Self {
        assert!(
            self.success,
            "Expected success but got error: {}",
            self.content
        );
        self
    }

    fn assert_error(&self) -> &Self {
        assert!(
            !self.success,
            "Expected error but got success: {}",
            self.content
        );
        self
    }
}

// ============================================================================
// MESSAGE SEND/RECEIVE TESTS
// ============================================================================

/// Test message-send tool parameter validation
#[tokio::test]
async fn test_message_send_parameters() {
    let node = TestNode::start("msg_params_test").await;

    let channel_result = node
        .call_tool(
            "channel-create",
            json!({
                "name": "test-channel",
                "description": "Test channel"
            }),
        )
        .await
        .assert_success();

    let channel_id = channel_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("channel_id"))
        .and_then(|v| v.as_str())
        .expect("Missing channel_id");

    // Try to send without networking - should get networking error
    let result = node
        .call_tool(
            "message-send",
            json!({
                "entity_id": channel_id,
                "content": "Test message"
            }),
        )
        .await
        .assert_error();

    assert!(
        result.content.contains("not started") || result.content.contains("Networking"),
        "Error should mention networking: {}",
        result.content
    );
}

/// Test message-send with missing parameters
#[tokio::test]
async fn test_message_send_missing_params() {
    let node = TestNode::start("msg_missing_test").await;

    // Try to send without entity_id
    node.call_tool(
            "message-send",
            json!({
                "content": "Test message"
            }),
        )
        .await
        .assert_error();

    // Try to send without content
    node.call_tool(
            "message-send",
            json!({
                "entity_id": "some-id"
            }),
        )
        .await
        .assert_error();
}

/// Test message-send with invalid entity_id
#[tokio::test]
async fn test_message_send_invalid_entity() {
    let node = TestNode::start("msg_invalid_test").await;

    node.call_tool(
            "message-send",
            json!({
                "entity_id": "not-a-valid-entity-id",
                "content": "Test message"
            }),
        )
        .await
        .assert_error();
}

/// Test message-send with metadata
#[tokio::test]
async fn test_message_send_with_metadata() {
    let node = TestNode::start("msg_metadata_test").await;

    let channel_result = node
        .call_tool(
            "channel-create",
            json!({
                "name": "metadata-channel",
                "description": "Test"
            }),
        )
        .await
        .assert_success();

    let channel_id = channel_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("channel_id"))
        .and_then(|v| v.as_str())
        .expect("Missing channel_id");

    let result = node
        .call_tool(
            "message-send",
            json!({
                "entity_id": channel_id,
                "content": "Message with metadata",
                "metadata": {
                    "priority": "high",
                    "attachments": [
                        {
                            "name": "file.pdf",
                            "size": 1024
                        }
                    ]
                }
            }),
        )
        .await;

    if !result.success {
        assert!(
            result.content.contains("not started") || result.content.contains("Networking"),
            "Error should be about networking, not parameters: {}",
            result.content
        );
    }
}

/// Test message-list tool
#[tokio::test]
async fn test_message_list_empty() {
    let node = TestNode::start("msg_list_test").await;

    let channel_result = node
        .call_tool(
            "channel-create",
            json!({
                "name": "list-channel",
                "description": "Test"
            }),
        )
        .await
        .assert_success();

    let channel_id = channel_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("channel_id"))
        .and_then(|v| v.as_str())
        .expect("Missing channel_id");

    let list_result = node
        .call_tool(
            "message-list",
            json!({
                "entity_id": channel_id,
                "limit": 10
            }),
        )
        .await
        .assert_success();

    let parsed = list_result.parsed.as_ref().expect("No parsed result");
    assert!(
        parsed["messages"].is_array(),
        "Response should have messages array"
    );

    let messages = parsed["messages"]
        .as_array()
        .expect("messages is not array");

    assert_eq!(messages.len(), 0, "New channel should have no messages");
}

/// Test message-list with missing parameters
#[tokio::test]
async fn test_message_list_missing_params() {
    let node = TestNode::start("msg_list_params_test").await;

    node.call_tool(
            "message-list",
            json!({
                "limit": 10
            }),
        )
        .await
        .assert_error();
}

/// Test message-list with different limits
#[tokio::test]
async fn test_message_list_limits() {
    let node = TestNode::start("msg_list_limits_test").await;

    let channel_result = node
        .call_tool(
            "channel-create",
            json!({
                "name": "limits-channel",
                "description": "Test"
            }),
        )
        .await
        .assert_success();

    let channel_id = channel_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("channel_id"))
        .and_then(|v| v.as_str())
        .expect("Missing channel_id");

    for limit in [1, 10, 50, 100] {
        let list_result = node
            .call_tool(
                "message-list",
                json!({
                    "entity_id": channel_id,
                    "limit": limit
                }),
            )
            .await
            .assert_success();

        let parsed = list_result.parsed.as_ref().expect("No parsed result");
        assert!(
            parsed["messages"].is_array(),
            "Should return messages array with limit {}",
            limit
        );
    }
}

/// Test message-get tool parameter validation
#[tokio::test]
async fn test_message_get_params() {
    let node = TestNode::start("msg_get_test").await;

    let channel_result = node
        .call_tool(
            "channel-create",
            json!({
                "name": "get-channel",
                "description": "Test"
            }),
        )
        .await
        .assert_success();

    let channel_id = channel_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("channel_id"))
        .and_then(|v| v.as_str())
        .expect("Missing channel_id");

    node.call_tool(
            "message-get",
            json!({
                "entity_id": channel_id,
                "message_id": "non-existent-id"
            }),
        )
        .await
        .assert_error();
}

/// Test message-get with missing parameters
#[tokio::test]
async fn test_message_get_missing_params() {
    let node = TestNode::start("msg_get_missing_test").await;

    node.call_tool(
            "message-get",
            json!({
                "message_id": "some-id"
            }),
        )
        .await
        .assert_error();

    node.call_tool(
            "message-get",
            json!({
                "entity_id": "some-id"
            }),
        )
        .await
        .assert_error();
}

/// Test message tools with various entity types
#[tokio::test]
async fn test_message_tools_entity_types() {
    let node = TestNode::start("msg_entities_test").await;

    let channel_result = node
        .call_tool(
            "channel-create",
            json!({
                "name": "entity-test-channel",
                "description": "Test"
            }),
        )
        .await
        .assert_success();

    let channel_id = channel_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("channel_id"))
        .and_then(|v| v.as_str())
        .expect("Missing channel_id");

    let list_result = node
        .call_tool(
            "message-list",
            json!({
                "entity_id": channel_id,
                "limit": 10
            }),
        )
        .await
        .assert_success();

    let parsed = list_result.parsed.as_ref().expect("No parsed result");
    assert!(parsed["messages"].is_array());

    let contact_result = node
        .call_tool(
            "contact-add",
            json!({
                "display_name": "Test Contact",
                "connection_words": "ocean forest moon star"
            }),
        )
        .await
        .assert_success();

    let contact_id = contact_result
        .parsed
        .as_ref()
        .and_then(|p| p.get("contact"))
        .and_then(|c| c.get("contact_id"))
        .and_then(|v| v.as_str())
        .expect("Missing contact_id");

    let list_result = node
        .call_tool(
            "message-list",
            json!({
                "entity_id": contact_id,
                "limit": 10
            }),
        )
        .await
        .assert_success();

    let parsed = list_result.parsed.as_ref().expect("No parsed result");
    assert!(parsed["messages"].is_array());
}

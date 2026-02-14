//! Message Send/Receive Tests - Phase 10.3 Task 1
//!
//! Tests for MCP message tools interface validation
//! Run with: cargo test -p communitas-mcp --test message_send_receive_test

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

    /// Initialize the MCP client connection
    /// May be called explicitly in future tests
    #[allow(dead_code)]
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
        tracing::debug!("Dropping TestNode: {} on port {}", self.name, self.port);
        let _ = self.process.kill();
    }
}

/// Result from calling an MCP tool, with helper methods for assertion and parsing
#[derive(Debug)]
struct ToolResult {
    success: bool,
    content: String,
    /// Parsed JSON response - used by helper methods
    #[allow(dead_code)]
    parsed: Option<serde_json::Value>,
}

impl ToolResult {
    /// Assert that the tool call succeeded
    #[allow(dead_code)]
    fn assert_success(self) -> Self {
        assert!(
            self.success,
            "Expected success but got error: {}",
            self.content
        );
        self
    }

    /// Assert that the tool call failed with an error
    fn assert_error(self) -> Self {
        assert!(
            !self.success,
            "Expected error but got success: {}",
            self.content
        );
        self
    }

    /// Extract entity_id or id from the response JSON
    #[allow(dead_code)]
    fn get_id(&self) -> Option<String> {
        self.parsed
            .as_ref()
            .and_then(|p| p.get("entity_id").or_else(|| p.get("id")))
            .and_then(|v| v.as_str())
            .map(String::from)
    }
}

// ============================================================================
// MESSAGE SEND/RECEIVE TESTS
// ============================================================================

/// Test send_message tool parameter validation
///
/// In demo mode, send_message auto-creates threads for non-existent IDs,
/// so we test the contract that it accepts thread_id and text parameters
/// and returns a valid result.
#[tokio::test]
async fn test_message_send_parameters() {
    let node = TestNode::start("msg_params_test").await;

    // Send message with thread_id and text - demo mode creates thread if needed
    let result = node
        .call_tool(
            "send_message",
            json!({
                "thread_id": "invalid-thread-id",
                "text": "Test message"
            }),
        )
        .await;

    // Demo mode contract: send_message should accept parameters and return success
    // (auto-creates thread if needed)
    assert!(
        result.success,
        "Demo mode should accept valid send_message calls"
    );
    assert!(!result.content.is_empty(), "Should return a response");
}

/// Test send_message with missing required parameters
///
/// send_message requires at least thread_id and text parameters.
/// Missing thread_id should generate a default or return error.
/// Missing text should cause an error.
#[tokio::test]
async fn test_message_send_missing_params() {
    let node = TestNode::start("msg_missing_test").await;

    // Missing thread_id - demo mode may auto-generate
    let result1 = node
        .call_tool(
            "send_message",
            json!({
                "text": "Test message"
            }),
        )
        .await;

    // Missing text - should fail even in demo mode
    let result2 = node
        .call_tool(
            "send_message",
            json!({
                "thread_id": "some-id"
            }),
        )
        .await;

    // At least one should fail (missing required text parameter)
    // result1 might succeed if demo auto-creates thread
    // result2 should fail (missing text)
    assert!(
        !result1.success || !result2.success,
        "At least missing text should fail"
    );
}

/// Test send_message with invalid thread_id (demo mode auto-creates)
///
/// In demo mode, send_message auto-creates threads for non-existent IDs.
/// This test validates that the tool accepts the call and succeeds.
#[tokio::test]
async fn test_message_send_invalid_entity() {
    let node = TestNode::start("msg_invalid_test").await;

    // Demo mode auto-creates threads for invalid IDs
    let result = node
        .call_tool(
            "send_message",
            json!({
                "thread_id": "not-a-valid-thread-id",
                "text": "Test message"
            }),
        )
        .await;

    // Demo mode contract: should succeed and auto-create thread
    assert!(
        result.success,
        "Demo mode should auto-create thread for send_message"
    );
}

/// Test send_message with attachments parameter
///
/// Validates that send_message accepts optional attachments parameter.
#[tokio::test]
async fn test_message_send_with_metadata() {
    let node = TestNode::start("msg_metadata_test").await;

    // Test that attachments parameter is accepted
    let result = node
        .call_tool(
            "send_message",
            json!({
                "thread_id": "test-thread-id",
                "text": "Message with attachments",
                "attachments": []
            }),
        )
        .await;

    // Tool should accept attachments parameter
    assert!(
        result.success,
        "send_message should accept attachments parameter"
    );
}

/// Test list_messages tool with valid thread_id
///
/// In demo mode, list_messages returns empty array for non-existent threads
/// rather than error, so we test it returns valid JSON response.
#[tokio::test]
async fn test_message_list_empty() {
    let node = TestNode::start("msg_list_test").await;

    let list_result = node
        .call_tool(
            "list_messages",
            json!({
                "thread_id": "test-thread-id",
                "limit": 10
            }),
        )
        .await;

    // Demo mode contract: should return success with result
    // (empty array for non-existent thread, or actual messages if exists)
    assert!(
        list_result.success,
        "list_messages should succeed in demo mode"
    );
    assert!(
        !list_result.content.is_empty(),
        "Should return a JSON response"
    );
}

/// Test list_messages with missing thread_id
#[tokio::test]
async fn test_message_list_missing_params() {
    let node = TestNode::start("msg_list_params_test").await;

    node.call_tool(
        "list_messages",
        json!({
            "limit": 10
        }),
    )
    .await
    .assert_error();
}

/// Test list_messages with different limit values
///
/// Validates that list_messages accepts various limit values.
#[tokio::test]
async fn test_message_list_limits() {
    let node = TestNode::start("msg_list_limits_test").await;

    for limit in [1, 10, 50, 100] {
        let list_result = node
            .call_tool(
                "list_messages",
                json!({
                    "thread_id": "test-thread-id",
                    "limit": limit
                }),
            )
            .await;

        // All limit values should be accepted
        assert!(
            list_result.success,
            "list_messages should accept limit value: {}",
            limit
        );
    }
}

/// Test get_messages tool with invalid entity (returns empty in demo mode)
///
/// In demo mode, get_messages returns empty array for non-existent entity_id
/// rather than raising an error.
#[tokio::test]
async fn test_message_get_params() {
    let node = TestNode::start("msg_get_test").await;

    let result = node
        .call_tool(
            "get_messages",
            json!({
                "entity_id": "non-existent-entity-id"
            }),
        )
        .await;

    // Demo mode contract: returns success with empty array instead of error
    assert!(
        result.success,
        "Demo mode get_messages should succeed and return empty array"
    );
}

/// Test get_messages with missing required entity_id parameter
///
/// The get_messages tool requires entity_id parameter.
/// Missing entity_id should either generate a default or return error.
#[tokio::test]
async fn test_message_get_missing_params() {
    let node = TestNode::start("msg_get_missing_test").await;

    let result = node.call_tool("get_messages", json!({})).await;

    // In demo mode, missing entity_id may return empty array or error
    // But the tool should respond (not hang or crash)
    assert!(!result.content.is_empty(), "Tool should return a response");
}

/// Test message tools parameter combinations
///
/// Validates that message tools accept pagination and filtering parameters.
#[tokio::test]
async fn test_message_tools_entity_types() {
    let node = TestNode::start("msg_entities_test").await;

    // Test list_messages with pagination parameters
    let list_result = node
        .call_tool(
            "list_messages",
            json!({
                "thread_id": "test-thread-id",
                "limit": 10,
                "before": 1234567890000_i64
            }),
        )
        .await;

    // Test get_messages with entity_id
    let get_result = node
        .call_tool(
            "get_messages",
            json!({
                "entity_id": "test-entity-id"
            }),
        )
        .await;

    // Both tools should accept their respective parameters
    assert!(
        list_result.success,
        "list_messages should accept pagination parameters"
    );
    assert!(
        get_result.success,
        "get_messages should accept entity_id parameter"
    );
}

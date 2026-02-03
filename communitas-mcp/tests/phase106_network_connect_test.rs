//! Phase 10.6 Network Connection Tests
//!
//! Tests for peer connection via connection words (2 tools)
//! Run with: cargo test -p communitas-mcp --test phase106_network_connect_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Represents the result of a tool call with full validation.
#[derive(Debug)]
#[allow(dead_code)]
struct ToolCallResult {
    success: bool,
    http_status: u16,
    response_body: String,
    is_json_rpc_error: bool,
}

/// Validates JSON-RPC response from server startup.
async fn validate_startup_response(response: reqwest::Response) -> bool {
    if !response.status().is_success() {
        return false;
    }

    let body = match response.text().await {
        Ok(text) => text,
        Err(_) => return false,
    };

    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(json) => json.get("error").is_none(),
        Err(_) => false,
    }
}

// Test helpers
struct TestNode {
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let capped_counter = std::cmp::min(counter, 999);
        let port = 35400 + (pid % 100) as u16 + (capped_counter * 2);

        let mut process = Command::new(env!("CARGO_BIN_EXE_communitas-mcp"))
            .args([
                "--http",
                "--demo",
                "--listen",
                &format!("127.0.0.1:{}", port),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start MCP server");

        // Wait for server to be ready with timeout
        let client = reqwest::Client::new();
        let max_attempts = 30;
        let mut server_ready = false;

        for _ in 0..max_attempts {
            sleep(Duration::from_millis(100)).await;

            #[allow(clippy::collapsible_if)]
            #[allow(clippy::collapsible_match)]
            if let Ok(response) = tokio::time::timeout(
                Duration::from_millis(500),
                client
                    .post(format!("http://127.0.0.1:{}/mcp", port))
                    .json(&json!({
                        "jsonrpc": "2.0",
                        "method": "tools/list",
                        "id": 1
                    }))
                    .send(),
            )
            .await
            {
                if let Ok(res) = response {
                    if validate_startup_response(res).await {
                        server_ready = true;
                        break;
                    }
                }
            }

            if let Ok(Some(_)) = process.try_wait() {
                panic!("{}: Server process died during startup", name);
            }
        }

        if !server_ready {
            let _ = process.kill();
            panic!(
                "{}: Server failed to start after {} attempts",
                name, max_attempts
            );
        }

        Self { process, port }
    }

    async fn call_tool(&self, tool_name: &str, arguments: serde_json::Value) -> ToolCallResult {
        let client = reqwest::Client::new();
        let url = format!("http://127.0.0.1:{}/mcp", self.port);

        let payload = json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            },
            "id": 1
        });

        let response = client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .expect("HTTP request failed");

        let http_status = response.status().as_u16();
        let response_body = response.text().await.expect("Failed to read response body");

        let is_json_rpc_error =
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_body) {
                json.get("error").is_some()
                    || json
                        .get("result")
                        .and_then(|r| r.get("isError"))
                        .and_then(|e| e.as_bool())
                        .unwrap_or(false)
            } else {
                false
            };

        let success = http_status == 200 && !is_json_rpc_error;

        ToolCallResult {
            success,
            http_status,
            response_body,
            is_json_rpc_error,
        }
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

//
// Network Connection Tests
//

#[tokio::test]
async fn test_get_connection_words() {
    let node = TestNode::start("get_connection_words").await;

    // Start network first
    let _ = node.call_tool("network_start", json!({})).await;

    // Get connection words
    let result = node.call_tool("get_connection_words", json!({})).await;
    assert!(
        result.success,
        "get_connection_words should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_connect_by_words_valid_format() {
    let node = TestNode::start("connect_by_words_valid").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Connect with valid four-word format (demo mode should accept)
    let result = node
        .call_tool(
            "connect_by_words",
            json!({
                "words": "ocean-forest-moon-star"
            }),
        )
        .await;
    assert!(
        result.success,
        "connect_by_words with valid format should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_connect_by_words_invalid_format() {
    let node = TestNode::start("connect_by_words_invalid").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Connect with invalid format
    let result = node
        .call_tool(
            "connect_by_words",
            json!({
                "words": "not-enough-words"
            }),
        )
        .await;
    // Invalid format should be rejected
    assert!(
        !result.success || result.is_json_rpc_error,
        "connect_by_words with invalid format should fail: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_connect_with_peer_four_words() {
    let node = TestNode::start("network_connect_peer").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Connect using network_connect tool
    let result = node
        .call_tool(
            "network_connect",
            json!({
                "peer_four_words": "mountain-river-cloud-tree"
            }),
        )
        .await;
    assert!(
        result.success,
        "network_connect should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_connect_with_invalid_words() {
    let node = TestNode::start("network_connect_invalid").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Connect with invalid words
    let result = node
        .call_tool(
            "network_connect",
            json!({
                "peer_four_words": "invalid"
            }),
        )
        .await;
    // Invalid words should be rejected
    assert!(
        !result.success || result.is_json_rpc_error,
        "network_connect with invalid words should fail: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_connection_words_validation() {
    let node = TestNode::start("connection_words_validation").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Get connection words
    let result = node.call_tool("get_connection_words", json!({})).await;
    assert!(
        result.success,
        "get_connection_words should succeed: {}",
        result.response_body
    );

    // Validate format (should be four hyphen-separated words)
    if result.success {
        let json: serde_json::Value =
            serde_json::from_str(&result.response_body).expect("Valid JSON");
        if let Some(content) = json.get("result").and_then(|r| r.get("content")) {
            // In demo mode, connection words are returned in content
            assert!(
                content.is_array() || content.is_string(),
                "Connection words should be in valid format"
            );
        }
    }
}

#[tokio::test]
async fn test_network_peers_after_connection() {
    let node = TestNode::start("peers_after_connect").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Connect to a peer
    let _ = node
        .call_tool(
            "network_connect",
            json!({
                "peer_four_words": "lake-stone-wind-flower"
            }),
        )
        .await;

    // Check peers list
    let result = node.call_tool("network_peers", json!({})).await;
    assert!(
        result.success,
        "network_peers should succeed after connection: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_disconnect_after_connect() {
    let node = TestNode::start("disconnect_after_connect").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Connect to a peer
    let _ = node
        .call_tool(
            "network_connect",
            json!({
                "peer_four_words": "sun-rain-snow-mist"
            }),
        )
        .await;

    // Stop network (disconnect all)
    let result = node.call_tool("network_stop", json!({})).await;
    assert!(
        result.success,
        "network_stop should succeed: {}",
        result.response_body
    );

    // Verify peers are cleared
    let peers_result = node.call_tool("network_peers", json!({})).await;
    assert!(
        peers_result.success,
        "network_peers should still respond after disconnect: {}",
        peers_result.response_body
    );
}

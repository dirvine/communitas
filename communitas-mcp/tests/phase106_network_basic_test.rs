//! Phase 10.6 Network Basic Operations Tests
//!
//! Tests for core P2P networking operations (7 tools)
//! Run with: cargo test -p communitas-mcp --test phase106_network_basic_test

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
        let port = 35300 + (pid % 100) as u16 + (capped_counter * 2);

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
// Network Basic Operations Tests
//

#[tokio::test]
async fn test_network_start_default_port() {
    let node = TestNode::start("network_start_default").await;
    let result = node.call_tool("network_start", json!({})).await;
    assert!(
        result.success,
        "network_start with default port should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_start_preferred_port() {
    let node = TestNode::start("network_start_preferred").await;
    let result = node
        .call_tool(
            "network_start",
            json!({
                "port": 9000
            }),
        )
        .await;
    assert!(
        result.success,
        "network_start with preferred port should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_status_when_stopped() {
    let node = TestNode::start("network_status_stopped").await;
    let result = node.call_tool("network_status", json!({})).await;
    assert!(
        result.success,
        "network_status should return status even when stopped: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_status_when_running() {
    let node = TestNode::start("network_status_running").await;

    // Start network first
    let _ = node.call_tool("network_start", json!({})).await;

    // Get status
    let result = node.call_tool("network_status", json!({})).await;
    assert!(
        result.success,
        "network_status should succeed when running: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_peers_when_empty() {
    let node = TestNode::start("network_peers_empty").await;
    let result = node.call_tool("network_peers", json!({})).await;
    assert!(
        result.success,
        "network_peers should succeed even with no peers: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_peers_when_connected() {
    let node = TestNode::start("network_peers_connected").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Get peers (in demo mode, may return mock peers)
    let result = node.call_tool("network_peers", json!({})).await;
    assert!(
        result.success,
        "network_peers should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_stop_gracefully() {
    let node = TestNode::start("network_stop").await;

    // Start network first
    let _ = node.call_tool("network_start", json!({})).await;

    // Stop network
    let result = node.call_tool("network_stop", json!({})).await;
    assert!(
        result.success,
        "network_stop should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_network_request_external_address() {
    let node = TestNode::start("network_external_address").await;

    // Start network
    let _ = node.call_tool("network_start", json!({})).await;

    // Request external address discovery
    let result = node
        .call_tool("network_request_external_address", json!({}))
        .await;
    assert!(
        result.success,
        "network_request_external_address should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_set_network_available_online() {
    let node = TestNode::start("network_available_online").await;
    let result = node
        .call_tool(
            "set_network_available",
            json!({
                "available": true
            }),
        )
        .await;
    assert!(
        result.success,
        "set_network_available(true) should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_set_network_available_offline() {
    let node = TestNode::start("network_available_offline").await;
    let result = node
        .call_tool(
            "set_network_available",
            json!({
                "available": false
            }),
        )
        .await;
    assert!(
        result.success,
        "set_network_available(false) should succeed: {}",
        result.response_body
    );
}

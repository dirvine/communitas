//! Phase 10.6 Task 3: Call History & Recording Tests
//!
//! Tests call history tracking and recording features
//! Run with: cargo test -p communitas-mcp --test phase106_call_history_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(200);

#[derive(Debug)]
#[allow(dead_code)]
struct ToolCallResult {
    success: bool,
    http_status: u16,
    response_body: String,
    is_json_rpc_error: bool,
}

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

struct TestNode {
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let pid = std::process::id();
        let capped_counter = std::cmp::min(counter, 999);
        let port = 37100 + (pid % 100) as u16 + (capped_counter * 2);

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

        let client = reqwest::Client::new();
        for attempt in 0..50 {
            sleep(Duration::from_millis(100)).await;

            #[allow(clippy::collapsible_if)]
            if let Ok(response) = client
                .post(format!("http://127.0.0.1:{}/mcp", port))
                .timeout(Duration::from_secs(5))
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
            {
                if validate_startup_response(response).await {
                    return Self { process, port };
                }
            }

            if attempt == 49 {
                let _ = process.kill();
                let _ = process.wait();
                panic!(
                    "Node {} failed to start after 50 attempts on port {}",
                    name, port
                );
            }
        }

        let _ = process.kill();
        let _ = process.wait();
        unreachable!("Start loop should have returned or panicked")
    }

    async fn call_tool(&self, tool: &str, params: serde_json::Value) -> ToolCallResult {
        let client = reqwest::Client::new();

        match timeout(
            Duration::from_secs(10),
            client
                .post(format!("http://127.0.0.1:{}/mcp", self.port))
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "tools/call",
                    "params": {"name": tool, "arguments": params}
                }))
                .send(),
        )
        .await
        {
            Ok(Ok(response)) => {
                let http_status = response.status().as_u16();
                let is_success = response.status().is_success();

                match response.text().await {
                    Ok(body) => {
                        let json_rpc_error = match serde_json::from_str::<serde_json::Value>(&body)
                        {
                            Ok(json) => json.get("error").is_some(),
                            Err(_) => false,
                        };

                        ToolCallResult {
                            success: is_success && !json_rpc_error,
                            http_status,
                            response_body: body,
                            is_json_rpc_error: json_rpc_error,
                        }
                    }
                    Err(_) => ToolCallResult {
                        success: false,
                        http_status,
                        response_body: String::from("Failed to read response body"),
                        is_json_rpc_error: false,
                    },
                }
            }
            Ok(Err(e)) => ToolCallResult {
                success: false,
                http_status: 0,
                response_body: format!("Request error: {}", e),
                is_json_rpc_error: false,
            },
            Err(_) => ToolCallResult {
                success: false,
                http_status: 0,
                response_body: String::from("Request timeout after 10s"),
                is_json_rpc_error: false,
            },
        }
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

// Call History & Recording Tests

#[tokio::test]
async fn test_get_call_history_all() {
    let node = TestNode::start("call-hist-all").await;
    let result = node.call_tool("get_call_history", json!({})).await;

    assert!(result.success, "get_call_history should succeed");
}

#[tokio::test]
async fn test_get_call_history_by_entity() {
    let node = TestNode::start("call-hist-entity").await;
    let result = node
        .call_tool(
            "get_call_history",
            json!({
                "entity_id": "test-entity-123"
            }),
        )
        .await;

    assert!(
        result.success,
        "get_call_history with entity filter should succeed"
    );
}

#[tokio::test]
async fn test_get_call_history_by_type() {
    let node = TestNode::start("call-hist-type").await;
    let result = node
        .call_tool(
            "get_call_history",
            json!({
                "call_type": "direct"
            }),
        )
        .await;

    assert!(
        result.success,
        "get_call_history with type filter should succeed"
    );
}

#[tokio::test]
async fn test_get_missed_calls_all() {
    let node = TestNode::start("call-missed-all").await;
    let result = node.call_tool("get_missed_calls", json!({})).await;

    assert!(result.success, "get_missed_calls should succeed");
}

#[tokio::test]
async fn test_get_missed_calls_unread() {
    let node = TestNode::start("call-missed-unread").await;
    let result = node
        .call_tool(
            "get_missed_calls",
            json!({
                "unread_only": true
            }),
        )
        .await;

    assert!(
        result.success,
        "get_missed_calls with unread filter should succeed"
    );
}

#[tokio::test]
async fn test_acknowledge_single_missed_call() {
    let node = TestNode::start("call-ack-single").await;
    let result = node
        .call_tool(
            "acknowledge_missed_call",
            json!({
                "call_id": "test-call-456"
            }),
        )
        .await;

    assert!(result.success, "acknowledge_missed_call should succeed");
}

#[tokio::test]
async fn test_acknowledge_all_missed_calls() {
    let node = TestNode::start("call-ack-all").await;
    let result = node
        .call_tool(
            "acknowledge_missed_call",
            json!({
                "acknowledge_all": true
            }),
        )
        .await;

    assert!(
        result.success,
        "acknowledge all missed calls should succeed"
    );
}

#[tokio::test]
async fn test_get_call_recording_status() {
    let node = TestNode::start("call-rec-status").await;
    let result = node
        .call_tool(
            "get_call_recording",
            json!({
                "call_id": "active-call-789"
            }),
        )
        .await;

    assert!(result.success, "get_call_recording should succeed");
}

#[tokio::test]
async fn test_start_call_recording() {
    let node = TestNode::start("call-rec-start").await;
    let result = node
        .call_tool(
            "start_call_recording",
            json!({
                "call_id": "active-call-123"
            }),
        )
        .await;

    assert!(result.success, "start_call_recording should succeed");
}

#[tokio::test]
async fn test_stop_call_recording() {
    let node = TestNode::start("call-rec-stop").await;
    let result = node
        .call_tool(
            "stop_call_recording",
            json!({
                "call_id": "active-call-123"
            }),
        )
        .await;

    assert!(result.success, "stop_call_recording should succeed");
}

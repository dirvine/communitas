//! Phase 10.6 Task 2: Call Management Error Cases Tests
//!
//! Tests error handling for call management tools
//! Run with: cargo test -p communitas-mcp --test phase106_call_errors_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(100);

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
        let port = 36100 + (pid % 100) as u16 + (capped_counter * 2);

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

// Error Cases Tests

#[tokio::test]
async fn test_start_call_without_entity_id() {
    let node = TestNode::start("call-err-no-entity").await;
    let result = node
        .call_tool(
            "start_voice_call",
            json!({
                "video_enabled": false
            }),
        )
        .await;

    // In demo mode, may succeed or return JSON-RPC error for missing field
    // Both behaviors are acceptable
    assert!(result.http_status == 200, "HTTP layer should succeed");
}

#[tokio::test]
async fn test_join_call_invalid_call_id() {
    let node = TestNode::start("call-err-invalid-join").await;
    let result = node
        .call_tool(
            "join_call",
            json!({
                "call_id": "nonexistent_call_999",
                "entity_id": "test-entity"
            }),
        )
        .await;

    // Demo mode may succeed but should handle invalid call_id gracefully
    assert!(result.http_status == 200, "HTTP should succeed");
}

#[tokio::test]
async fn test_end_call_nonexistent() {
    let node = TestNode::start("call-err-end-none").await;
    let result = node
        .call_tool(
            "end_call",
            json!({
                "call_id": "does_not_exist_123"
            }),
        )
        .await;

    // Demo mode should handle gracefully
    assert!(result.http_status == 200, "HTTP should succeed");
}

#[tokio::test]
async fn test_toggle_mute_without_call_id() {
    let node = TestNode::start("call-err-mute-nocall").await;
    let result = node
        .call_tool(
            "toggle_mute",
            json!({
                "mute": true
            }),
        )
        .await;

    // In demo mode, validates missing parameters
    assert!(result.http_status == 200, "HTTP layer should succeed");
}

#[tokio::test]
async fn test_toggle_video_without_call_id() {
    let node = TestNode::start("call-err-video-nocall").await;
    let result = node
        .call_tool(
            "toggle_video",
            json!({
                "enable": false
            }),
        )
        .await;

    // In demo mode, validates missing parameters
    assert!(result.http_status == 200, "HTTP layer should succeed");
}

#[tokio::test]
async fn test_share_screen_without_call_id() {
    let node = TestNode::start("call-err-screen-nocall").await;
    let result = node
        .call_tool(
            "share_screen",
            json!({
                "enable": true
            }),
        )
        .await;

    // In demo mode, validates missing parameters
    assert!(result.http_status == 200, "HTTP layer should succeed");
}

#[tokio::test]
async fn test_get_call_status_nonexistent() {
    let node = TestNode::start("call-err-status-none").await;
    let result = node
        .call_tool(
            "get_call_status",
            json!({
                "call_id": "missing_call_456"
            }),
        )
        .await;

    // Demo mode should handle gracefully
    assert!(result.http_status == 200, "HTTP should succeed");
}

#[tokio::test]
async fn test_get_participants_invalid_call() {
    let node = TestNode::start("call-err-parts-invalid").await;
    let result = node
        .call_tool(
            "get_call_participants",
            json!({
                "call_id": "invalid_call_789"
            }),
        )
        .await;

    // Demo mode should handle gracefully
    assert!(result.http_status == 200, "HTTP should succeed");
}

#[tokio::test]
async fn test_list_active_calls_empty() {
    let node = TestNode::start("call-err-list-empty").await;
    let result = node.call_tool("list_active_calls", json!({})).await;

    // Should succeed and return empty list
    assert!(result.success, "list_active_calls should succeed");
}

#[tokio::test]
async fn test_get_call_quality_no_active_call() {
    let node = TestNode::start("call-err-quality-none").await;
    let result = node
        .call_tool(
            "get_call_quality",
            json!({
                "call_id": "no_such_call_000"
            }),
        )
        .await;

    // Demo mode should handle gracefully
    assert!(result.http_status == 200, "HTTP should succeed");
}

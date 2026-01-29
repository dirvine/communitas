//! Phase 10.6 Task 7: Presence Basic Tests
//!
//! Tests user presence status management
//! Run with: cargo test -p communitas-mcp --test phase106_presence_basic_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(300);

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
        let port = 38100 + (pid % 100) as u16 + (capped_counter * 2);

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

// Presence Basic Tests

#[tokio::test]
async fn test_set_presence_online() {
    let node = TestNode::start("pres-online").await;
    let result = node
        .call_tool(
            "set_presence",
            json!({
                "status": "online"
            }),
        )
        .await;

    assert!(result.success, "set_presence online should succeed");
}

#[tokio::test]
async fn test_set_presence_away() {
    let node = TestNode::start("pres-away").await;
    let result = node
        .call_tool(
            "set_presence",
            json!({
                "status": "away"
            }),
        )
        .await;

    assert!(result.success, "set_presence away should succeed");
}

#[tokio::test]
async fn test_set_presence_busy() {
    let node = TestNode::start("pres-busy").await;
    let result = node
        .call_tool(
            "set_presence",
            json!({
                "status": "busy"
            }),
        )
        .await;

    assert!(result.success, "set_presence busy should succeed");
}

#[tokio::test]
async fn test_set_presence_invisible() {
    let node = TestNode::start("pres-invisible").await;
    let result = node
        .call_tool(
            "set_presence",
            json!({
                "status": "invisible"
            }),
        )
        .await;

    assert!(result.success, "set_presence invisible should succeed");
}

#[tokio::test]
async fn test_set_presence_for_entity() {
    let node = TestNode::start("pres-entity").await;
    let result = node
        .call_tool(
            "set_presence",
            json!({
                "status": "busy",
                "entity_id": "test-entity-456"
            }),
        )
        .await;

    assert!(result.success, "set_presence with entity should succeed");
}

#[tokio::test]
async fn test_get_presence_single_user() {
    let node = TestNode::start("pres-get-single").await;
    let result = node
        .call_tool(
            "get_presence",
            json!({
                "user_ids": ["user-123"]
            }),
        )
        .await;

    assert!(
        result.success,
        "get_presence for single user should succeed"
    );
}

#[tokio::test]
async fn test_get_presence_multiple_users() {
    let node = TestNode::start("pres-get-multi").await;
    let result = node
        .call_tool(
            "get_presence",
            json!({
                "user_ids": ["user-123", "user-456", "user-789"]
            }),
        )
        .await;

    assert!(
        result.success,
        "get_presence for multiple users should succeed"
    );
}

#[tokio::test]
async fn test_set_my_presence() {
    let node = TestNode::start("pres-my-status").await;
    let result = node
        .call_tool(
            "set_my_presence",
            json!({
                "status": "away"
            }),
        )
        .await;

    assert!(result.success, "set_my_presence should succeed");
}

#[tokio::test]
async fn test_get_contact_presence() {
    let node = TestNode::start("pres-contact").await;
    let result = node
        .call_tool(
            "get_contact_presence",
            json!({
                "contact_id": "contact-789"
            }),
        )
        .await;

    assert!(result.success, "get_contact_presence should succeed");
}

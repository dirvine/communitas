//! Phase 10.6 Task 8: Presence Network Tests
//!
//! Tests network-wide presence discovery and subscription
//! Run with: cargo test -p communitas-mcp --test phase106_presence_network_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(400);

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
        let port = 39100 + (pid % 100) as u16 + (capped_counter * 2);

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

// Presence Network Tests

#[tokio::test]
async fn test_announce_presence() {
    let node = TestNode::start("pres-announce").await;
    let result = node
        .call_tool(
            "announce_presence",
            json!({}),
        )
        .await;

    assert!(result.success, "announce_presence should succeed");
}

#[tokio::test]
async fn test_query_presence_by_pubkey() {
    let node = TestNode::start("pres-query").await;
    let result = node
        .call_tool(
            "query_presence",
            json!({
                "pubkey": "test-pubkey-hex-123"
            }),
        )
        .await;

    assert!(result.success, "query_presence should succeed");
}

#[tokio::test]
async fn test_query_presence_invalid_pubkey() {
    let node = TestNode::start("pres-query-invalid").await;
    let result = node
        .call_tool(
            "query_presence",
            json!({
                "pubkey": "invalid"
            }),
        )
        .await;

    // Should handle gracefully in demo mode
    assert!(result.http_status == 200, "HTTP should succeed");
}

#[tokio::test]
async fn test_get_our_presence() {
    let node = TestNode::start("pres-our").await;
    let result = node
        .call_tool(
            "get_our_presence",
            json!({}),
        )
        .await;

    assert!(result.success, "get_our_presence should succeed");
}

#[tokio::test]
async fn test_get_cached_presence_empty() {
    let node = TestNode::start("pres-cached-empty").await;
    let result = node
        .call_tool(
            "get_cached_presence",
            json!({}),
        )
        .await;

    assert!(result.success, "get_cached_presence should succeed");
}

#[tokio::test]
async fn test_get_cached_presence_with_data() {
    let node = TestNode::start("pres-cached-data").await;

    // First announce
    let _ = node.call_tool("announce_presence", json!({})).await;

    // Then get cached
    let result = node
        .call_tool(
            "get_cached_presence",
            json!({}),
        )
        .await;

    assert!(result.success, "get_cached_presence should succeed");
}

#[tokio::test]
async fn test_subscribe_to_presence() {
    let node = TestNode::start("pres-subscribe").await;
    let result = node
        .call_tool(
            "subscribe_to_presence",
            json!({
                "entity_ids": ["entity-123", "entity-456"]
            }),
        )
        .await;

    assert!(result.success, "subscribe_to_presence should succeed");
}

#[tokio::test]
async fn test_subscribe_invalid_entity() {
    let node = TestNode::start("pres-sub-invalid").await;
    let result = node
        .call_tool(
            "subscribe_to_presence",
            json!({
                "entity_ids": ["invalid-entity"]
            }),
        )
        .await;

    // Demo mode handles gracefully
    assert!(result.http_status == 200, "HTTP should succeed");
}

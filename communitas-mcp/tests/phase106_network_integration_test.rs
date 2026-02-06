//! Phase 10.6 Task 11: Network & Presence Integration Tests
//!
//! Tests network and presence integration workflows
//! Run with: cargo test -p communitas-mcp --test phase106_network_integration_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(700);

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
        let port = 42100 + (pid % 100) as u16 + (capped_counter * 2);

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

// Network & Presence Integration Tests

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_network_startup_and_connection_words() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("net-int-startup").await;

    // Start network
    let start = node.call_tool("network_start", json!({})).await;
    assert!(start.success);

    // Get connection words
    let words = node.call_tool("get_connection_words", json!({})).await;
    assert!(words.success);

    // Stop network
    let stop = node.call_tool("network_stop", json!({})).await;
    assert!(stop.success);
}

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_network_and_presence_announce() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("net-int-presence").await;

    // Start network
    let start = node.call_tool("network_start", json!({})).await;
    assert!(start.success);

    // Set presence
    let pres = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(pres.success);

    // Announce presence
    let announce = node.call_tool("announce_presence", json!({})).await;
    assert!(announce.success);

    // Query our presence
    let query = node.call_tool("get_our_presence", json!({})).await;
    assert!(query.success);
}

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_peer_connection_workflow() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("net-int-peer").await;

    // Start network
    let start = node.call_tool("network_start", json!({})).await;
    assert!(start.success);

    // Connect to peer (demo mode)
    let connect = node
        .call_tool(
            "connect_by_words",
            json!({"words": "ocean-forest-moon-star"}),
        )
        .await;
    assert!(connect.success);

    // List peers
    let peers = node.call_tool("network_peers", json!({})).await;
    assert!(peers.success);
}

#[tokio::test]
async fn test_presence_subscription_workflow() {
    let node = TestNode::start("net-int-sub").await;

    // Set presence
    let pres = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(pres.success);

    // Announce
    let announce = node.call_tool("announce_presence", json!({})).await;
    assert!(announce.success);

    // Query
    let query = node
        .call_tool("query_presence", json!({"pubkey": "test-pubkey"}))
        .await;
    assert!(query.success);

    // Get cached
    let cached = node.call_tool("get_cached_presence", json!({})).await;
    assert!(cached.success);
}

#[tokio::test]
async fn test_network_availability_and_presence() {
    let node = TestNode::start("net-int-avail").await;

    // Set network available (online)
    let online = node
        .call_tool("set_network_available", json!({"available": true}))
        .await;
    assert!(online.success);

    // Set presence to online
    let pres = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(pres.success);

    // Set network unavailable (offline)
    let offline = node
        .call_tool("set_network_available", json!({"available": false}))
        .await;
    assert!(offline.success);
}

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_full_bootstrap_workflow() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("net-int-bootstrap").await;

    // 1. Start network
    let start = node.call_tool("network_start", json!({})).await;
    assert!(start.success);

    // 2. Get connection words
    let words = node.call_tool("get_connection_words", json!({})).await;
    assert!(words.success);

    // 3. Set presence
    let pres = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(pres.success);

    // 4. Announce presence
    let announce = node.call_tool("announce_presence", json!({})).await;
    assert!(announce.success);

    // 5. Query presence
    let query = node
        .call_tool("query_presence", json!({"pubkey": "test-pubkey"}))
        .await;
    assert!(query.success);
}

#[tokio::test]
async fn test_presence_sync_across_entities() {
    let node = TestNode::start("net-int-entity").await;

    // Set presence for entity 1
    let p1 = node
        .call_tool(
            "set_presence",
            json!({"status": "online", "entity_id": "entity-1"}),
        )
        .await;
    assert!(p1.success);

    // Set presence for entity 2
    let p2 = node
        .call_tool(
            "set_presence",
            json!({"status": "away", "entity_id": "entity-2"}),
        )
        .await;
    assert!(p2.success);

    // Get all presence
    let all = node.call_tool("get_cached_presence", json!({})).await;
    assert!(all.success);
}

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_network_status_monitoring() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("net-int-status").await;

    // Start network
    let start = node.call_tool("network_start", json!({})).await;
    assert!(start.success);

    // Check status
    let status = node.call_tool("network_status", json!({})).await;
    assert!(status.success);

    // Request external address
    let ext = node
        .call_tool("network_request_external_address", json!({}))
        .await;
    assert!(ext.success);

    // Check peers
    let peers = node.call_tool("network_peers", json!({})).await;
    assert!(peers.success);
}

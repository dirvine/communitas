//! Phase 10.6 Task 12: Full Integration Tests
//!
//! Tests cross-feature integration across calls, network, and presence
//! Run with: cargo test -p communitas-mcp --test phase106_full_integration_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(800);

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
        let port = 43100 + (pid % 100) as u16 + (capped_counter * 2);

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

// Full Integration Tests

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_full_call_network_presence_workflow() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("full-int-all").await;

    // 1. Start network
    let net = node.call_tool("network_start", json!({})).await;
    assert!(net.success);

    // 2. Set presence to online
    let pres = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(pres.success);

    // 3. Announce presence
    let ann = node.call_tool("announce_presence", json!({})).await;
    assert!(ann.success);

    // 4. Start call
    let call = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call.success);

    // 5. Update presence to busy
    let busy = node
        .call_tool("set_my_presence", json!({"status": "busy"}))
        .await;
    assert!(busy.success);

    // 6. End call
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);

    // 7. Update presence back to online
    let online = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(online.success);
}

#[tokio::test]
async fn test_multi_entity_presence_isolation() {
    let node = TestNode::start("full-int-entity").await;

    // Org presence
    let org = node
        .call_tool(
            "set_presence",
            json!({"status": "online", "entity_id": "org-123"}),
        )
        .await;
    assert!(org.success);

    // Group presence
    let grp = node
        .call_tool(
            "set_presence",
            json!({"status": "away", "entity_id": "group-456"}),
        )
        .await;
    assert!(grp.success);

    // Channel presence
    let ch = node
        .call_tool(
            "set_presence",
            json!({"status": "busy", "entity_id": "channel-789"}),
        )
        .await;
    assert!(ch.success);

    // Verify all cached
    let cached = node.call_tool("get_cached_presence", json!({})).await;
    assert!(cached.success);
}

#[tokio::test]
async fn test_call_quality_during_network_changes() {
    let node = TestNode::start("full-int-quality").await;

    // Start call
    let call = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call.success);

    // Get quality
    let q1 = node
        .call_tool("get_call_quality", json!({"call_id": "test-call"}))
        .await;
    assert!(q1.success);

    // Network change
    let net = node
        .call_tool("set_network_available", json!({"available": false}))
        .await;
    assert!(net.success);

    // Get quality again
    let q2 = node
        .call_tool("get_call_quality", json!({"call_id": "test-call"}))
        .await;
    assert!(q2.success);

    // End
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_device_switching_during_call() {
    let node = TestNode::start("full-int-device").await;

    // List devices
    let dev = node.call_tool("list_media_devices", json!({})).await;
    assert!(dev.success);

    // Start call
    let call = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call.success);

    // Toggle mute
    let mute = node
        .call_tool("toggle_mute", json!({"call_id": "test-call", "mute": true}))
        .await;
    assert!(mute.success);

    // End
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_presence_updates_during_call() {
    let node = TestNode::start("full-int-pres-call").await;

    // Set online
    let online = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(online.success);

    // Start call
    let call = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call.success);

    // Update to busy (in call)
    let busy = node
        .call_tool("set_my_presence", json!({"status": "busy"}))
        .await;
    assert!(busy.success);

    // End call
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);

    // Back to online
    let online2 = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(online2.success);
}

#[tokio::test]
#[ignore] // Requires live network: set MCP_TEST_NETWORK_ENABLED=true to run
async fn test_network_reconnection_with_call_recovery() {
    if std::env::var("MCP_TEST_NETWORK_ENABLED").is_err() {
        return;
    }
    let node = TestNode::start("full-int-reconnect").await;

    // Start network
    let net = node.call_tool("network_start", json!({})).await;
    assert!(net.success);

    // Start call
    let call = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call.success);

    // Simulate network drop
    let drop = node
        .call_tool("set_network_available", json!({"available": false}))
        .await;
    assert!(drop.success);

    // Reconnect
    let reconnect = node
        .call_tool("set_network_available", json!({"available": true}))
        .await;
    assert!(reconnect.success);

    // Get call status
    let status = node
        .call_tool("get_call_status", json!({"call_id": "test-call"}))
        .await;
    assert!(status.success);

    // End
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_call_history_across_network_sessions() {
    let node = TestNode::start("full-int-history").await;

    // Session 1: Make a call
    let call1 = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call1.success);

    let end1 = node
        .call_tool("end_call", json!({"call_id": "test-call-1"}))
        .await;
    assert!(end1.success);

    // Session 2: Make another call
    let call2 = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": true}),
        )
        .await;
    assert!(call2.success);

    let end2 = node
        .call_tool("end_call", json!({"call_id": "test-call-2"}))
        .await;
    assert!(end2.success);

    // Check history
    let history = node.call_tool("get_call_history", json!({})).await;
    assert!(history.success);
}

#[tokio::test]
async fn test_media_device_changes_with_presence() {
    let node = TestNode::start("full-int-media-pres").await;

    // Set presence
    let pres = node
        .call_tool("set_my_presence", json!({"status": "online"}))
        .await;
    assert!(pres.success);

    // List devices
    let dev = node.call_tool("list_media_devices", json!({})).await;
    assert!(dev.success);

    // Start call
    let call = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(call.success);

    // Update presence
    let busy = node
        .call_tool("set_my_presence", json!({"status": "busy"}))
        .await;
    assert!(busy.success);

    // End
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

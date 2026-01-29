//! Phase 10.6 Task 10: Call Integration Workflow Tests
//!
//! Tests complete call lifecycle workflows
//! Run with: cargo test -p communitas-mcp --test phase106_call_integration_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::{Duration, sleep, timeout};

static PORT_COUNTER: AtomicU16 = AtomicU16::new(600);

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
        let port = 41100 + (pid % 100) as u16 + (capped_counter * 2);

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

// Call Integration Workflows

#[tokio::test]
async fn test_voice_call_workflow() {
    let node = TestNode::start("call-int-voice").await;

    // Start voice call
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(start.success);

    // Get status
    let status = node
        .call_tool("get_call_status", json!({"call_id": "test-call"}))
        .await;
    assert!(status.success);

    // Toggle mute
    let mute = node
        .call_tool("toggle_mute", json!({"call_id": "test-call", "mute": true}))
        .await;
    assert!(mute.success);

    // Unmute
    let unmute = node
        .call_tool(
            "toggle_mute",
            json!({"call_id": "test-call", "mute": false}),
        )
        .await;
    assert!(unmute.success);

    // End call
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_video_call_workflow() {
    let node = TestNode::start("call-int-video").await;

    // Start with video
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": true}),
        )
        .await;
    assert!(start.success);

    // Toggle video off
    let vid_off = node
        .call_tool(
            "toggle_video",
            json!({"call_id": "test-call", "enable": false}),
        )
        .await;
    assert!(vid_off.success);

    // Toggle video on
    let vid_on = node
        .call_tool(
            "toggle_video",
            json!({"call_id": "test-call", "enable": true}),
        )
        .await;
    assert!(vid_on.success);

    // End
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_screen_sharing_workflow() {
    let node = TestNode::start("call-int-screen").await;

    // Start call
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(start.success);

    // Start screen share
    let share_start = node
        .call_tool(
            "share_screen",
            json!({"call_id": "test-call", "enable": true}),
        )
        .await;
    assert!(share_start.success);

    // Stop screen share
    let share_stop = node
        .call_tool(
            "share_screen",
            json!({"call_id": "test-call", "enable": false}),
        )
        .await;
    assert!(share_stop.success);

    // End call
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_recording_workflow() {
    let node = TestNode::start("call-int-record").await;

    // Start call
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(start.success);

    // Start recording
    let rec_start = node
        .call_tool("start_call_recording", json!({"call_id": "test-call"}))
        .await;
    assert!(rec_start.success);

    // Get recording status
    let rec_status = node
        .call_tool("get_call_recording", json!({"call_id": "test-call"}))
        .await;
    assert!(rec_status.success);

    // Stop recording
    let rec_stop = node
        .call_tool("stop_call_recording", json!({"call_id": "test-call"}))
        .await;
    assert!(rec_stop.success);

    // End call
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_multi_participant_workflow() {
    let node = TestNode::start("call-int-multi").await;

    // Start call
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(start.success);

    // Join as participant 1
    let join1 = node
        .call_tool(
            "join_call",
            json!({"call_id": "test-call", "entity_id": "participant-1"}),
        )
        .await;
    assert!(join1.success);

    // Join as participant 2
    let join2 = node
        .call_tool(
            "join_call",
            json!({"call_id": "test-call", "entity_id": "participant-2"}),
        )
        .await;
    assert!(join2.success);

    // Get participants
    let parts = node
        .call_tool("get_call_participants", json!({"call_id": "test-call"}))
        .await;
    assert!(parts.success);

    // End call
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);
}

#[tokio::test]
async fn test_call_history_tracking() {
    let node = TestNode::start("call-int-history").await;

    // Make a call
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(start.success);

    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);

    // Check history
    let history = node.call_tool("get_call_history", json!({})).await;
    assert!(history.success);
}

#[tokio::test]
async fn test_missed_call_workflow() {
    let node = TestNode::start("call-int-missed").await;

    // Start call (simulated missed)
    let start = node
        .call_tool(
            "start_voice_call",
            json!({"entity_id": "test-entity", "video_enabled": false}),
        )
        .await;
    assert!(start.success);

    // End without joining
    let end = node
        .call_tool("end_call", json!({"call_id": "test-call"}))
        .await;
    assert!(end.success);

    // Check missed calls
    let missed = node.call_tool("get_missed_calls", json!({})).await;
    assert!(missed.success);

    // Acknowledge
    let ack = node
        .call_tool("acknowledge_missed_call", json!({"call_id": "test-call"}))
        .await;
    assert!(ack.success);
}

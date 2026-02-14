//! Phase 10.6 Call Management Tools Tests
//!
//! Comprehensive tests for call management tools (11 tools)
//! Run with: cargo test -p communitas-mcp --test phase106_call_management_test

use serde_json::json;
use std::process::{Command, Stdio};
use tokio::time::{Duration, sleep, timeout};

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

// ============================================================================
// Call Management Tests - Happy Path
// ============================================================================

#[tokio::test]
async fn test_start_voice_call() {
    let node = TestNode::start("call-test-start-voice").await;
    let result = node
        .call_tool(
            "start_voice_call",
            json!({
                "entity_id": "test-entity-123",
                "video_enabled": false
            }),
        )
        .await;

    assert!(
        result.success,
        "start_voice_call should succeed in demo mode: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_start_video_call() {
    let node = TestNode::start("call-test-start-video").await;
    let result = node
        .call_tool(
            "start_voice_call",
            json!({
                "entity_id": "test-entity-456",
                "video_enabled": true
            }),
        )
        .await;

    assert!(
        result.success,
        "start_voice_call with video should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_join_call() {
    let node = TestNode::start("call-test-join").await;
    let result = node
        .call_tool(
            "join_call",
            json!({
                "call_id": "test-call-789"
            }),
        )
        .await;

    assert!(
        result.success,
        "join_call should succeed in demo mode: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_end_call() {
    let node = TestNode::start("call-test-end").await;
    let result = node
        .call_tool(
            "end_call",
            json!({
                "call_id": "test-call-end-123"
            }),
        )
        .await;

    assert!(
        result.success,
        "end_call should succeed in demo mode: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_toggle_mute_on() {
    let node = TestNode::start("call-test-mute-on").await;
    let result = node
        .call_tool(
            "toggle_mute",
            json!({
                "call_id": "test-call-mute",
                "muted": true
            }),
        )
        .await;

    assert!(
        result.success,
        "toggle_mute (on) should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_toggle_mute_off() {
    let node = TestNode::start("call-test-mute-off").await;
    let result = node
        .call_tool(
            "toggle_mute",
            json!({
                "call_id": "test-call-unmute",
                "muted": false
            }),
        )
        .await;

    assert!(
        result.success,
        "toggle_mute (off) should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_toggle_video_on() {
    let node = TestNode::start("call-test-video-on").await;
    let result = node
        .call_tool(
            "toggle_video",
            json!({
                "call_id": "test-call-video",
                "enabled": true
            }),
        )
        .await;

    assert!(
        result.success,
        "toggle_video (on) should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_toggle_video_off() {
    let node = TestNode::start("call-test-video-off").await;
    let result = node
        .call_tool(
            "toggle_video",
            json!({
                "call_id": "test-call-no-video",
                "enabled": false
            }),
        )
        .await;

    assert!(
        result.success,
        "toggle_video (off) should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_share_screen_start() {
    let node = TestNode::start("call-test-screen-start").await;
    let result = node
        .call_tool(
            "share_screen",
            json!({
                "call_id": "test-call-screen",
                "enabled": true
            }),
        )
        .await;

    assert!(
        result.success,
        "share_screen (start) should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_share_screen_stop() {
    let node = TestNode::start("call-test-screen-stop").await;
    let result = node
        .call_tool(
            "share_screen",
            json!({
                "call_id": "test-call-screen-off",
                "enabled": false
            }),
        )
        .await;

    assert!(
        result.success,
        "share_screen (stop) should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_get_call_status() {
    let node = TestNode::start("call-test-status").await;
    let result = node
        .call_tool(
            "get_call_status",
            json!({
                "call_id": "test-call-status-123"
            }),
        )
        .await;

    assert!(
        result.success,
        "get_call_status should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_list_call_participants() {
    let node = TestNode::start("call-test-participants").await;
    let result = node
        .call_tool(
            "list_call_participants",
            json!({
                "call_id": "test-call-participants-456"
            }),
        )
        .await;

    assert!(
        result.success,
        "list_call_participants should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_list_active_calls() {
    let node = TestNode::start("call-test-list-active").await;
    let result = node
        .call_tool(
            "list_active_calls",
            json!({
                "entity_id": "test-entity-list"
            }),
        )
        .await;

    assert!(
        result.success,
        "list_active_calls should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_get_call_quality_metrics() {
    let node = TestNode::start("call-test-quality").await;
    let result = node
        .call_tool(
            "get_call_quality_metrics",
            json!({
                "call_id": "test-call-quality-789"
            }),
        )
        .await;

    assert!(
        result.success,
        "get_call_quality_metrics should succeed: {:?}",
        result.response_body
    );
}

#[tokio::test]
async fn test_set_media_device() {
    let node = TestNode::start("call-test-set-device").await;
    let result = node
        .call_tool(
            "set_media_device",
            json!({
                "device_id": "test-device-123",
                "device_type": "microphone"
            }),
        )
        .await;

    assert!(
        result.success,
        "set_media_device should succeed: {:?}",
        result.response_body
    );
}

// ============================================================================
// Integration Test - Full Call Lifecycle
// ============================================================================

#[tokio::test]
async fn test_call_lifecycle_integration() {
    let node = TestNode::start("call-test-lifecycle").await;

    // 1. Start call
    let start_result = node
        .call_tool(
            "start_voice_call",
            json!({
                "entity_id": "lifecycle-entity",
                "video_enabled": false
            }),
        )
        .await;
    assert!(start_result.success, "Start call failed in lifecycle test");

    // 2. Join call
    let join_result = node
        .call_tool(
            "join_call",
            json!({
                "call_id": "lifecycle-call"
            }),
        )
        .await;
    assert!(join_result.success, "Join call failed in lifecycle test");

    // 3. Toggle mute
    let mute_result = node
        .call_tool(
            "toggle_mute",
            json!({
                "call_id": "lifecycle-call",
                "muted": true
            }),
        )
        .await;
    assert!(mute_result.success, "Mute failed in lifecycle test");

    // 4. Get status
    let status_result = node
        .call_tool(
            "get_call_status",
            json!({
                "call_id": "lifecycle-call"
            }),
        )
        .await;
    assert!(status_result.success, "Get status failed in lifecycle test");

    // 5. End call
    let end_result = node
        .call_tool(
            "end_call",
            json!({
                "call_id": "lifecycle-call"
            }),
        )
        .await;
    assert!(end_result.success, "End call failed in lifecycle test");
}

// ============================================================================
// Concurrent Calls Test
// ============================================================================

#[tokio::test]
async fn test_multiple_concurrent_calls() {
    let node = TestNode::start("call-test-concurrent").await;

    // Start 3 calls concurrently
    let call1 = node.call_tool(
        "start_voice_call",
        json!({"entity_id": "concurrent-1", "video_enabled": false}),
    );
    let call2 = node.call_tool(
        "start_voice_call",
        json!({"entity_id": "concurrent-2", "video_enabled": true}),
    );
    let call3 = node.call_tool(
        "start_voice_call",
        json!({"entity_id": "concurrent-3", "video_enabled": false}),
    );

    let (r1, r2, r3) = tokio::join!(call1, call2, call3);

    assert!(r1.success, "Concurrent call 1 failed");
    assert!(r2.success, "Concurrent call 2 failed");
    assert!(r3.success, "Concurrent call 3 failed");
}

// ============================================================================
// Video Call Workflow Test
// ============================================================================

#[tokio::test]
async fn test_video_call_workflow() {
    let node = TestNode::start("call-test-video-workflow").await;

    // 1. Start with video
    let start_result = node
        .call_tool(
            "start_voice_call",
            json!({
                "entity_id": "video-workflow",
                "video_enabled": true
            }),
        )
        .await;
    assert!(start_result.success, "Start video call failed");

    // 2. Toggle video off
    let video_off_result = node
        .call_tool(
            "toggle_video",
            json!({
                "call_id": "video-workflow-call",
                "enabled": false
            }),
        )
        .await;
    assert!(video_off_result.success, "Toggle video off failed");

    // 3. Start screen share
    let screen_result = node
        .call_tool(
            "share_screen",
            json!({
                "call_id": "video-workflow-call",
                "enabled": true
            }),
        )
        .await;
    assert!(screen_result.success, "Start screen share failed");

    // 4. Stop screen share
    let screen_stop_result = node
        .call_tool(
            "share_screen",
            json!({
                "call_id": "video-workflow-call",
                "enabled": false
            }),
        )
        .await;
    assert!(screen_stop_result.success, "Stop screen share failed");

    // 5. Toggle video back on
    let video_on_result = node
        .call_tool(
            "toggle_video",
            json!({
                "call_id": "video-workflow-call",
                "enabled": true
            }),
        )
        .await;
    assert!(video_on_result.success, "Toggle video on failed");

    // 6. End call
    let end_result = node
        .call_tool(
            "end_call",
            json!({
                "call_id": "video-workflow-call"
            }),
        )
        .await;
    assert!(end_result.success, "End video call failed");
}

// ============================================================================
// Media Device Selection Test
// ============================================================================

#[tokio::test]
async fn test_media_device_selection() {
    let node = TestNode::start("call-test-device-select").await;

    // Set microphone
    let mic_result = node
        .call_tool(
            "set_media_device",
            json!({
                "device_id": "mic-001",
                "device_type": "microphone"
            }),
        )
        .await;
    assert!(mic_result.success, "Set microphone failed");

    // Set camera
    let cam_result = node
        .call_tool(
            "set_media_device",
            json!({
                "device_id": "cam-001",
                "device_type": "camera"
            }),
        )
        .await;
    assert!(cam_result.success, "Set camera failed");

    // Set speaker
    let speaker_result = node
        .call_tool(
            "set_media_device",
            json!({
                "device_id": "speaker-001",
                "device_type": "speaker"
            }),
        )
        .await;
    assert!(speaker_result.success, "Set speaker failed");
}

//! Phase 10.6 Media Device Management Tests
//!
//! Tests for media device enumeration and metadata tools (2 tools)
//! Run with: cargo test -p communitas-mcp --test phase106_media_devices_test

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

        // Wait for server to be ready with timeout
        let client = reqwest::Client::new();
        let max_attempts = 30;
        let mut server_ready = false;

        for _ in 0..max_attempts {
            sleep(Duration::from_millis(100)).await;

            #[allow(clippy::collapsible_if)]
            #[allow(clippy::collapsible_match)]
            if let Ok(response) = timeout(
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
// Media Device Tests
//

#[tokio::test]
async fn test_list_all_media_devices() {
    let node = TestNode::start("list_all_devices").await;
    let result = node.call_tool("list_media_devices", json!({})).await;
    assert!(
        result.success,
        "list_media_devices should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_list_microphones_only() {
    let node = TestNode::start("list_microphones").await;
    let result = node
        .call_tool(
            "list_media_devices",
            json!({
                "device_type": "microphone"
            }),
        )
        .await;
    assert!(
        result.success,
        "list_media_devices with microphone type should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_list_speakers_only() {
    let node = TestNode::start("list_speakers").await;
    let result = node
        .call_tool(
            "list_media_devices",
            json!({
                "device_type": "speaker"
            }),
        )
        .await;
    assert!(
        result.success,
        "list_media_devices with speaker type should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_list_cameras_only() {
    let node = TestNode::start("list_cameras").await;
    let result = node
        .call_tool(
            "list_media_devices",
            json!({
                "device_type": "camera"
            }),
        )
        .await;
    assert!(
        result.success,
        "list_media_devices with camera type should succeed: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_list_devices_empty_result() {
    let node = TestNode::start("list_devices_empty").await;
    let result = node
        .call_tool(
            "list_media_devices",
            json!({
                "device_type": "nonexistent"
            }),
        )
        .await;
    // In demo mode, this should still succeed but return empty array
    assert!(
        result.success,
        "list_media_devices with invalid type should succeed in demo mode: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_device_metadata_validation() {
    let node = TestNode::start("device_metadata").await;
    let result = node.call_tool("list_media_devices", json!({})).await;
    assert!(
        result.success,
        "list_media_devices should succeed: {}",
        result.response_body
    );

    // Validate response has expected structure
    let json: serde_json::Value = serde_json::from_str(&result.response_body).expect("Valid JSON");
    assert!(
        json.get("result").is_some(),
        "Response should have result field"
    );
}

#[tokio::test]
async fn test_get_media_metadata_for_file() {
    let node = TestNode::start("media_metadata").await;
    let result = node
        .call_tool(
            "get_media_metadata",
            json!({
                "entity_id": "test_entity",
                "disk_type": "private",
                "path": "/test_video.mp4"
            }),
        )
        .await;
    // Tool responds even if file doesn't exist (expected in demo mode)
    assert_eq!(
        result.http_status, 200,
        "get_media_metadata should respond with HTTP 200: {}",
        result.response_body
    );
}

#[tokio::test]
async fn test_media_metadata_missing_params() {
    let node = TestNode::start("media_metadata_missing").await;
    let result = node.call_tool("get_media_metadata", json!({})).await;
    assert!(
        !result.success || result.is_json_rpc_error,
        "get_media_metadata without required params should fail: {}",
        result.response_body
    );
}

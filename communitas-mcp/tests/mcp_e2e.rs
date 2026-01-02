// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! End-to-end integration tests for MCP server
//!
//! These tests verify the complete MCP workflow by:
//! 1. Starting an MCP HTTP server in demo mode
//! 2. Sending JSON-RPC 2.0 requests
//! 3. Verifying responses
//!
//! Run with: cargo test -p communitas-mcp --test mcp_e2e

use reqwest::Client;
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;
use tokio::time::sleep;

/// Atomic counter for unique port assignment across parallel tests
static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Test server handle that cleans up on drop
struct TestServer {
    process: Child,
    port: u16,
}

impl TestServer {
    /// Start MCP server in HTTP demo mode on a random port
    async fn start() -> Self {
        // Use atomic counter + process ID for unique port per test
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let port = 30000 + (std::process::id() % 1000) as u16 * 10 + counter;

        let mut process = Command::new(env!("CARGO_BIN_EXE_communitas-mcp"))
            .args([
                "--http",
                "--demo",
                "--listen",
                &format!("127.0.0.1:{}", port),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");

        // Wait for server to start
        let client = Client::new();
        for _ in 0..50 {
            sleep(Duration::from_millis(100)).await;
            if client
                .post(format!("http://127.0.0.1:{}/mcp", port))
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 0,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": { "name": "test", "version": "1.0" }
                    }
                }))
                .send()
                .await
                .is_ok()
            {
                return Self { process, port };
            }
        }
        // Kill the process before panicking to avoid zombie
        let _ = process.kill();
        let _ = process.wait();
        panic!("MCP server failed to start within 5 seconds");
    }

    /// Get the server URL
    fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    /// Send a JSON-RPC request and return the result
    async fn request(&self, method: &str, params: Value) -> Value {
        let client = Client::new();
        let response = client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            }))
            .send()
            .await
            .expect("Failed to send request");

        response.json().await.expect("Failed to parse response")
    }

    /// Call a tool and return the result
    async fn call_tool(&self, name: &str, arguments: Value) -> Value {
        self.request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        )
        .await
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

// =============================================================================
// Protocol Tests
// =============================================================================

#[tokio::test]
async fn test_initialize() {
    let server = TestServer::start().await;

    let response = server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test-client", "version": "1.0.0" }
            }),
        )
        .await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["serverInfo"]["name"].as_str().is_some());
}

#[tokio::test]
async fn test_list_tools() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.request("tools/list", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
    let tools = response["result"]["tools"].as_array().expect("Expected tools array");

    // Verify we have tools
    assert!(!tools.is_empty(), "Expected at least one tool");

    // Check for some expected tools
    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(tool_names.contains(&"health_check"), "Expected health_check tool");
    assert!(tool_names.contains(&"core_status"), "Expected core_status tool");
    assert!(tool_names.contains(&"create_entity"), "Expected create_entity tool");
}

#[tokio::test]
async fn test_list_resources() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.request("resources/list", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
    let resources = response["result"]["resources"]
        .as_array()
        .expect("Expected resources array");

    // Verify we have resources
    assert!(!resources.is_empty(), "Expected at least one resource");
}

// =============================================================================
// Tool Tests
// =============================================================================

#[tokio::test]
async fn test_health_check() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.call_tool("health_check", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
    let content = &response["result"]["content"];
    assert!(content.is_array(), "Expected content array");
}

#[tokio::test]
async fn test_core_status() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.call_tool("core_status", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
}

#[tokio::test]
async fn test_get_profile() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.call_tool("get_profile", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
    let content = &response["result"]["content"];
    assert!(content.is_array(), "Expected content array");

    // In demo mode, we should have a profile
    let text = content[0]["text"].as_str().unwrap_or("");
    assert!(
        text.contains("four_words") || text.contains("display_name"),
        "Expected profile data in response: {text}"
    );
}

#[tokio::test]
async fn test_list_entities() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.call_tool("list_entities", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
}

#[tokio::test]
async fn test_create_and_get_entity() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    // Create an entity
    let create_response = server
        .call_tool(
            "create_entity",
            json!({
                "entity_type": "channel",
                "display_name": "Test Channel",
                "description": "A test channel for E2E testing"
            }),
        )
        .await;

    assert!(
        create_response.get("result").is_some(),
        "Expected result: {create_response:?}"
    );

    // List entities to verify
    let list_response = server.call_tool("list_entities", json!({})).await;
    assert!(
        list_response.get("result").is_some(),
        "Expected result: {list_response:?}"
    );
}

// =============================================================================
// Network Tests
// =============================================================================

#[tokio::test]
async fn test_network_status() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.call_tool("network_status", json!({})).await;

    assert!(response.get("result").is_some(), "Expected result: {response:?}");
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_invalid_method() {
    let server = TestServer::start().await;

    let response = server.request("nonexistent/method", json!({})).await;

    assert!(
        response.get("error").is_some(),
        "Expected error for invalid method: {response:?}"
    );
}

#[tokio::test]
async fn test_invalid_tool() {
    let server = TestServer::start().await;

    // Initialize first
    server
        .request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;

    let response = server.call_tool("nonexistent_tool", json!({})).await;

    // MCP returns result with isError flag, not a JSON-RPC error
    assert!(response.get("result").is_some(), "Expected result: {response:?}");
    let is_error = response["result"]["isError"].as_bool().unwrap_or(false);
    assert!(is_error, "Expected isError=true for invalid tool: {response:?}");
}

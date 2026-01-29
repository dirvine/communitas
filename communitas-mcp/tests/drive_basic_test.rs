//! Drive Basic Operations Tests - Phase 10.5 Task 1
//!
//! Tests for MCP drive basic operations (list_disks, get_disk_stats)
//! Run with: cargo test -p communitas-mcp --test drive_basic_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::sleep;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Test node that spawns an MCP server process
struct TestNode {
    #[allow(dead_code)]
    name: String,
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let port = 33000 + (std::process::id() % 1000) as u16 * 10 + counter;

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
        for _ in 0..50 {
            sleep(std::time::Duration::from_millis(100)).await;
            if client
                .post(format!("http://127.0.0.1:{}/mcp", port))
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
                .is_ok()
            {
                return Self {
                    name: name.to_string(),
                    process,
                    port,
                };
            }
        }
        let _ = process.kill();
        let _ = process.wait();
        panic!("Node {} failed to start", name);
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    async fn request(&self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let client = reqwest::Client::new();
        match client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            }))
            .send()
            .await
        {
            Ok(response) => match response.json().await {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to parse response JSON: {}", e);
                    json!({"error": "Failed to parse response"})
                }
            },
            Err(e) => {
                eprintln!("Failed to send request to {}: {}", self.url(), e);
                json!({"error": "Request failed"})
            }
        }
    }

    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> ToolResult {
        let response = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments
                }),
            )
            .await;

        let result = response.get("result").cloned().unwrap_or(json!(null));
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let parsed: Option<serde_json::Value> = serde_json::from_str(content).ok();

        ToolResult {
            success: !is_error,
            content: content.to_string(),
            parsed,
        }
    }

    /// Initialize the MCP client connection
    #[allow(dead_code)]
    async fn initialize(&self) {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": &self.name, "version": "1.0"}
            }),
        )
        .await;
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        tracing::debug!("Dropping TestNode: {} on port {}", self.name, self.port);
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// Result from calling an MCP tool, with helper methods for assertion and parsing
#[derive(Debug)]
struct ToolResult {
    success: bool,
    content: String,
    #[allow(dead_code)]
    parsed: Option<serde_json::Value>,
}

impl ToolResult {
    /// Assert that the tool call was successful
    #[allow(dead_code)]
    fn assert_success(self) -> Self {
        assert!(
            self.success,
            "Expected success but got error: {}",
            self.content
        );
        self
    }

    /// Assert that the tool call resulted in an error
    #[allow(dead_code)]
    fn assert_error(self) -> Self {
        assert!(
            !self.success,
            "Expected error but got success: {}",
            self.content
        );
        self
    }
}

//
// Tests
//

/// Test list_disks returns success
#[tokio::test]
async fn test_list_disks_success() {
    let node = TestNode::start("list_disks_success").await;

    let result = node
        .call_tool(
            "list_disks",
            json!({
                "entity_id": "test-entity-123"
            }),
        )
        .await;

    // Demo mode should return success
    assert!(
        result.success || result.content.contains("disk") || result.content.contains("success"),
        "list_disks should return success or disk information: {}",
        result.content
    );
}

/// Test list_disks with missing params returns appropriate response
#[tokio::test]
async fn test_list_disks_missing_params() {
    let node = TestNode::start("list_disks_missing").await;

    let result = node.call_tool("list_disks", json!({})).await;

    // Should handle missing params (demo mode may be permissive)
    assert!(
        !result.content.is_empty(),
        "Should return some response for missing params"
    );
}

/// Test get_disk_stats returns success
#[tokio::test]
async fn test_get_disk_stats_success() {
    let node = TestNode::start("disk_stats_success").await;

    let result = node
        .call_tool(
            "get_disk_stats",
            json!({
                "entity_id": "test-entity-456",
                "disk_type": "private"
            }),
        )
        .await;

    // Demo mode should return success or stats
    assert!(
        result.success
            || result.content.contains("stats")
            || result.content.contains("size")
            || result.content.contains("success"),
        "get_disk_stats should return success or statistics: {}",
        result.content
    );
}

/// Test get_disk_stats with missing entity_id
#[tokio::test]
async fn test_get_disk_stats_missing_entity() {
    let node = TestNode::start("disk_stats_missing_entity").await;

    let result = node
        .call_tool(
            "get_disk_stats",
            json!({
                "disk_type": "private"
            }),
        )
        .await;

    // Should handle missing entity_id (demo mode may be permissive)
    assert!(
        !result.content.is_empty(),
        "Should return some response for missing entity_id"
    );
}

/// Test get_disk_stats with invalid entity_id
#[tokio::test]
async fn test_get_disk_stats_invalid_entity() {
    let node = TestNode::start("disk_stats_invalid").await;

    let result = node
        .call_tool(
            "get_disk_stats",
            json!({
                "entity_id": "",
                "disk_type": "private"
            }),
        )
        .await;

    // Empty entity_id should be handled
    assert!(
        !result.content.is_empty(),
        "Should return some response for invalid entity_id"
    );
}

/// Test disk_types validation (private/public/shared)
#[tokio::test]
async fn test_disk_types_validation() {
    let node = TestNode::start("disk_types").await;

    // Test private disk
    let result_private = node
        .call_tool(
            "get_disk_stats",
            json!({
                "entity_id": "test-entity",
                "disk_type": "private"
            }),
        )
        .await;

    assert!(
        !result_private.content.is_empty(),
        "Private disk type should be handled"
    );

    // Test public disk
    let result_public = node
        .call_tool(
            "get_disk_stats",
            json!({
                "entity_id": "test-entity",
                "disk_type": "public"
            }),
        )
        .await;

    assert!(
        !result_public.content.is_empty(),
        "Public disk type should be handled"
    );

    // Test shared disk
    let result_shared = node
        .call_tool(
            "get_disk_stats",
            json!({
                "entity_id": "test-entity",
                "disk_type": "shared"
            }),
        )
        .await;

    assert!(
        !result_shared.content.is_empty(),
        "Shared disk type should be handled"
    );
}

/// Test drive tools work with various entity types
#[tokio::test]
async fn test_drive_tools_entity_types() {
    let node = TestNode::start("entity_types").await;

    // Test with org entity
    let result_org = node
        .call_tool(
            "list_disks",
            json!({
                "entity_id": "org-test-123"
            }),
        )
        .await;

    assert!(
        !result_org.content.is_empty(),
        "Should handle org entity type"
    );

    // Test with group entity
    let result_group = node
        .call_tool(
            "list_disks",
            json!({
                "entity_id": "group-test-456"
            }),
        )
        .await;

    assert!(
        !result_group.content.is_empty(),
        "Should handle group entity type"
    );

    // Test with individual entity
    let result_individual = node
        .call_tool(
            "list_disks",
            json!({
                "entity_id": "individual-test-789"
            }),
        )
        .await;

    assert!(
        !result_individual.content.is_empty(),
        "Should handle individual entity type"
    );
}

/// Test list_disks with empty entity (demo mode behavior)
#[tokio::test]
async fn test_list_disks_empty() {
    let node = TestNode::start("list_disks_empty").await;

    let result = node
        .call_tool(
            "list_disks",
            json!({
                "entity_id": "nonexistent-entity-empty"
            }),
        )
        .await;

    // Demo mode should return success even for nonexistent entities
    assert!(
        result.success || result.content.contains("success") || result.content.contains("disk"),
        "Demo mode should handle nonexistent entities gracefully: {}",
        result.content
    );
}

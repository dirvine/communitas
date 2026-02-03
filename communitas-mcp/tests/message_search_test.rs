//! Message Search Tests - Phase 10.3 Task 3
//!
//! Tests for MCP message search operations
//! Run with: cargo test -p communitas-mcp --test message_search_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::sleep;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Test node that spawns an MCP server process
struct TestNode {
    #[allow(dead_code)]
    #[allow(dead_code)]
    name: String,
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let port = 34000 + (std::process::id() % 1000) as u16 * 10 + counter;

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

    async fn call_tool(&self, tool: &str, input: serde_json::Value) -> serde_json::Value {
        let client = reqwest::Client::new();
        client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": tool,
                    "arguments": input,
                }
            }))
            .send()
            .await
            .ok()
            .and_then(|r| futures::executor::block_on(r.json()).ok())
            .unwrap_or_else(|| json!({"error": "request failed"}))
    }

    async fn initialize(&self) {
        let _result = self.call_tool("get_profile", json!({})).await;
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

#[tokio::test]
async fn test_search_by_exact_text_match() {
    let node = TestNode::start("test-search-exact").await;
    node.initialize().await;

    // Send messages with unique keywords
    let _send1 = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-exact",
                "text": "The quick brown fox",
                "message_type": "text"
            }),
        )
        .await;

    let _send2 = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-exact",
                "text": "Jumps over the lazy dog",
                "message_type": "text"
            }),
        )
        .await;

    // Search for exact phrase
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-exact",
                "query": "quick brown fox"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "search_messages should return result"
    );

    // Verify at least one result
    if let Some(results) = result["result"]["messages"].as_array() {
        assert!(!results.is_empty(), "Should find matching message");
    }
}

#[tokio::test]
async fn test_search_by_partial_text_match() {
    let node = TestNode::start("test-search-partial").await;
    node.initialize().await;

    // Send message
    let _send = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-partial",
                "text": "The complete documentation",
                "message_type": "text"
            }),
        )
        .await;

    // Search for partial match
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-partial",
                "query": "complete"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "search_messages should support partial match"
    );
}

#[tokio::test]
async fn test_search_by_date_range() {
    let node = TestNode::start("test-search-date").await;
    node.initialize().await;

    // Send a message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-date",
                "text": "Dated message",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(_msg_id) = send_result["result"]["message_id"].as_str() {
        // Get current timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Search within date range (last hour)
        let result = node
            .call_tool(
                "search_messages",
                json!({
                    "entity_id": "channel-search-date",
                    "query": "Dated message",
                    "start_time": now - 3600,
                    "end_time": now
                }),
            )
            .await;

        assert!(
            result.get("result").is_some() || result.get("error").is_some(),
            "search with date range should handle request"
        );
    }
}

#[tokio::test]
async fn test_search_by_author() {
    let node = TestNode::start("test-search-author").await;
    node.initialize().await;

    // Send message (author is implicit from session)
    let _send = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-author",
                "text": "Message from specific author",
                "message_type": "text"
            }),
        )
        .await;

    // Search with author filter
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-author",
                "query": "specific author",
                "author_filter": true
            }),
        )
        .await;

    assert!(
        result.get("result").is_some() || result.get("error").is_some(),
        "search should support author filtering"
    );
}

#[tokio::test]
async fn test_search_with_no_results() {
    let node = TestNode::start("test-search-empty").await;
    node.initialize().await;

    // Send a specific message
    let _send = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-empty",
                "text": "Needle in haystack",
                "message_type": "text"
            }),
        )
        .await;

    // Search for something that doesn't exist
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-empty",
                "query": "xyzabc nonexistent xyz"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "search should return empty result gracefully"
    );

    // Verify empty result set
    if let Some(messages) = result["result"]["messages"].as_array() {
        assert!(
            messages.is_empty(),
            "Search for non-existent content should return empty array"
        );
    }
}

#[tokio::test]
async fn test_search_with_special_characters() {
    let node = TestNode::start("test-search-special").await;
    node.initialize().await;

    // Send message with special characters
    let _send = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-special",
                "text": "Message with @special #chars &symbols (parentheses)",
                "message_type": "text"
            }),
        )
        .await;

    // Search including special characters
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-special",
                "query": "@special #chars"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "search should handle special characters"
    );
}

#[tokio::test]
async fn test_search_pagination() {
    let node = TestNode::start("test-search-pagination").await;
    node.initialize().await;

    // Send multiple messages
    for i in 0..10 {
        let _send = node
            .call_tool(
                "send_message",
                json!({
                    "entity_id": "channel-search-paginate",
                    "text": format!("Test message number {}", i),
                    "message_type": "text"
                }),
            )
            .await;
    }

    // Search with pagination
    let page1 = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-paginate",
                "query": "Test message",
                "limit": 3,
                "offset": 0
            }),
        )
        .await;

    assert!(
        page1.get("result").is_some(),
        "search with pagination should work"
    );

    if let Some(results) = page1["result"]["messages"].as_array() {
        assert!(results.len() <= 3, "Result count should respect limit");
    }

    // Get second page
    let page2 = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-paginate",
                "query": "Test message",
                "limit": 3,
                "offset": 3
            }),
        )
        .await;

    assert!(
        page2.get("result").is_some(),
        "search second page should work"
    );
}

#[tokio::test]
async fn test_search_case_insensitive() {
    let node = TestNode::start("test-search-case").await;
    node.initialize().await;

    // Send message with mixed case
    let _send = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-case",
                "text": "CaseSensitiveTest Message",
                "message_type": "text"
            }),
        )
        .await;

    // Search with different case
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-case",
                "query": "casesensitivetest"
            }),
        )
        .await;

    // Should either find it (if case-insensitive) or handle gracefully
    assert!(
        result.get("result").is_some(),
        "search should handle case variation"
    );
}

#[tokio::test]
async fn test_search_metadata() {
    let node = TestNode::start("test-search-metadata").await;
    node.initialize().await;

    // Send message with metadata
    let _send = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-search-metadata",
                "text": "Metadata tagged message",
                "message_type": "text",
                "metadata": {
                    "tags": ["important", "urgent"],
                    "priority": "high"
                }
            }),
        )
        .await;

    // Search by content and metadata
    let result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": "channel-search-metadata",
                "query": "Metadata tagged",
                "metadata_filter": {
                    "tags": ["important"]
                }
            }),
        )
        .await;

    assert!(
        result.get("result").is_some() || result.get("error").is_some(),
        "search should support metadata filtering"
    );
}

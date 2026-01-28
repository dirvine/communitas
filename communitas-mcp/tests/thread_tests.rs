//! Thread Tests - Phase 10.3 Tasks 4-6
//!
//! Tests for MCP thread operations: creation, navigation, and pinning
//! Run with: cargo test -p communitas-mcp --test thread_tests

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::sleep;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

struct TestNode {
    name: String,
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let port = 35000 + (std::process::id() % 1000) as u16 * 10 + counter;

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

// ===== TASK 4: Thread Creation Tests =====

#[tokio::test]
async fn test_create_thread_from_message() {
    let node = TestNode::start("test-thread-create").await;
    node.initialize().await;

    // Send initial message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-thread",
                "content": "Parent message",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Create thread from message
        let result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-thread",
                    "message_id": msg_id
                }),
            )
            .await;

        assert!(
            result.get("result").is_some(),
            "create_thread should succeed"
        );
    }
}

#[tokio::test]
async fn test_create_thread_with_initial_reply() {
    let node = TestNode::start("test-thread-reply").await;
    node.initialize().await;

    // Send parent message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-thread-reply",
                "content": "Start a thread",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Create thread with initial reply
        let result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-thread-reply",
                    "message_id": msg_id,
                    "initial_reply": "First reply in thread"
                }),
            )
            .await;

        assert!(
            result.get("result").is_some(),
            "create_thread with reply should work"
        );
    }
}

#[tokio::test]
async fn test_list_threads() {
    let node = TestNode::start("test-list-threads").await;
    node.initialize().await;

    // Send message and create thread
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-list-threads",
                "content": "Message for thread",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let _thread = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-list-threads",
                    "message_id": msg_id
                }),
            )
            .await;

        // List threads
        let list_result = node
            .call_tool(
                "list_threads",
                json!({
                    "entity_id": "channel-list-threads",
                    "limit": 10
                }),
            )
            .await;

        assert!(
            list_result.get("result").is_some(),
            "list_threads should return result"
        );
    }
}

#[tokio::test]
async fn test_get_thread_details() {
    let node = TestNode::start("test-thread-details").await;
    node.initialize().await;

    // Send message
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-thread-details",
                "content": "Message with thread",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        // Create thread
        let thread_result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-thread-details",
                    "message_id": msg_id
                }),
            )
            .await;

        if let Some(thread_id) = thread_result["result"]["thread_id"].as_str() {
            // Get thread details
            let get_result = node
                .call_tool(
                    "get_thread",
                    json!({
                        "entity_id": "channel-thread-details",
                        "thread_id": thread_id
                    }),
                )
                .await;

            assert!(
                get_result.get("result").is_some(),
                "get_thread should return details"
            );
        }
    }
}

// ===== TASK 5: Thread Navigation Tests =====

#[tokio::test]
async fn test_get_thread_messages() {
    let node = TestNode::start("test-thread-messages").await;
    node.initialize().await;

    // Create thread
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-nav",
                "content": "Thread parent",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let thread_result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-nav",
                    "message_id": msg_id
                }),
            )
            .await;

        if let Some(thread_id) = thread_result["result"]["thread_id"].as_str() {
            // Get messages in thread
            let msg_result = node
                .call_tool(
                    "get_thread_messages",
                    json!({
                        "entity_id": "channel-nav",
                        "thread_id": thread_id
                    }),
                )
                .await;

            assert!(
                msg_result.get("result").is_some(),
                "get_thread_messages should work"
            );
        }
    }
}

#[tokio::test]
async fn test_paginate_thread_messages() {
    let node = TestNode::start("test-thread-paginate").await;
    node.initialize().await;

    // Create thread
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-paginate",
                "content": "Thread start",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let thread_result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-paginate",
                    "message_id": msg_id
                }),
            )
            .await;

        if let Some(thread_id) = thread_result["result"]["thread_id"].as_str() {
            // Paginate thread messages
            let page_result = node
                .call_tool(
                    "get_thread_messages",
                    json!({
                        "entity_id": "channel-paginate",
                        "thread_id": thread_id,
                        "limit": 5,
                        "offset": 0
                    }),
                )
                .await;

            assert!(
                page_result.get("result").is_some(),
                "pagination should work"
            );
        }
    }
}

// ===== TASK 6: Thread Pinning Tests =====

#[tokio::test]
async fn test_pin_thread() {
    let node = TestNode::start("test-pin-thread").await;
    node.initialize().await;

    // Create thread
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-pin",
                "content": "Thread to pin",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let thread_result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-pin",
                    "message_id": msg_id
                }),
            )
            .await;

        if let Some(thread_id) = thread_result["result"]["thread_id"].as_str() {
            // Pin the thread
            let pin_result = node
                .call_tool(
                    "pin_thread",
                    json!({
                        "entity_id": "channel-pin",
                        "thread_id": thread_id
                    }),
                )
                .await;

            assert!(
                pin_result.get("result").is_some(),
                "pin_thread should succeed"
            );
        }
    }
}

#[tokio::test]
async fn test_unpin_thread() {
    let node = TestNode::start("test-unpin-thread").await;
    node.initialize().await;

    // Create and pin thread
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": "channel-unpin",
                "content": "Thread to unpin",
                "message_type": "text"
            }),
        )
        .await;

    if let Some(msg_id) = send_result["result"]["message_id"].as_str() {
        let thread_result = node
            .call_tool(
                "create_thread",
                json!({
                    "entity_id": "channel-unpin",
                    "message_id": msg_id
                }),
            )
            .await;

        if let Some(thread_id) = thread_result["result"]["thread_id"].as_str() {
            let _pin = node
                .call_tool(
                    "pin_thread",
                    json!({
                        "entity_id": "channel-unpin",
                        "thread_id": thread_id
                    }),
                )
                .await;

            // Unpin
            let unpin_result = node
                .call_tool(
                    "unpin_thread",
                    json!({
                        "entity_id": "channel-unpin",
                        "thread_id": thread_id
                    }),
                )
                .await;

            assert!(
                unpin_result.get("result").is_some(),
                "unpin_thread should succeed"
            );
        }
    }
}

#[tokio::test]
async fn test_get_pinned_threads() {
    let node = TestNode::start("test-pinned-list").await;
    node.initialize().await;

    // List pinned threads
    let result = node
        .call_tool(
            "get_pinned_threads",
            json!({
                "entity_id": "channel-pinned-list"
            }),
        )
        .await;

    assert!(
        result.get("result").is_some(),
        "get_pinned_threads should work"
    );
}

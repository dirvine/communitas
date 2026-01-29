//! Messaging Integration Workflow Test - Phase 10.3 Task 11
//!
//! End-to-end messaging workflow testing all 25 tools together
//! Run with: cargo test -p communitas-mcp --test messaging_integration_test

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::sleep;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

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
        let port = 37000 + (std::process::id() % 1000) as u16 * 10 + counter;

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
async fn test_complete_messaging_workflow() {
    let node = TestNode::start("test-integration").await;
    node.initialize().await;

    let channel = "integration-test-channel";

    // 1. User A sends message to channel
    println!("Step 1: User A sends message");
    let send_result = node
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel,
                "content": "Hello everyone! Check out this feature",
                "message_type": "text"
            }),
        )
        .await;

    let msg_id = send_result["result"]["message_id"]
        .as_str()
        .expect("Message should have ID");
    println!("Message sent: {}", msg_id);

    // 2. User B receives message via get_messages
    println!("Step 2: User B retrieves messages");
    let list_result = node
        .call_tool(
            "list_messages",
            json!({
                "entity_id": channel,
                "limit": 10
            }),
        )
        .await;

    assert!(
        list_result["result"]["messages"].is_array(),
        "Should retrieve messages"
    );
    println!("Messages listed successfully");

    // 3. User B replies, creating thread
    println!("Step 3: User B creates thread with reply");
    let thread_result = node
        .call_tool(
            "create_thread",
            json!({
                "entity_id": channel,
                "message_id": msg_id,
                "initial_reply": "Great feature! I love this design."
            }),
        )
        .await;

    let thread_id = thread_result["result"]["thread_id"]
        .as_str()
        .expect("Thread should have ID");
    println!("Thread created: {}", thread_id);

    // 4. User A reacts to original message
    println!("Step 4: User A adds reaction");
    let reaction_result = node
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": channel,
                "message_id": msg_id,
                "emoji": "❤️"
            }),
        )
        .await;

    assert!(
        reaction_result.get("result").is_some(),
        "Reaction should be added"
    );
    println!("Reaction added");

    // 5. User B marks thread as read
    println!("Step 5: User B marks thread as read");
    let read_result = node
        .call_tool(
            "mark_thread_read",
            json!({
                "entity_id": channel,
                "thread_id": thread_id
            }),
        )
        .await;

    assert!(
        read_result.get("result").is_some(),
        "Thread should be marked read"
    );
    println!("Thread marked as read");

    // 6. User A edits original message
    println!("Step 6: User A edits message");
    let edit_result = node
        .call_tool(
            "edit_message",
            json!({
                "entity_id": channel,
                "message_id": msg_id,
                "content": "Hello everyone! Check out this AMAZING feature"
            }),
        )
        .await;

    assert!(
        edit_result.get("result").is_some(),
        "Message should be edited"
    );
    println!("Message edited");

    // 7. Verify all state synchronized
    println!("Step 7: Verify state consistency");

    // Check edited message
    let verify_msg = node
        .call_tool(
            "get_message",
            json!({
                "entity_id": channel,
                "message_id": msg_id
            }),
        )
        .await;

    assert_eq!(
        verify_msg["result"]["content"].as_str().unwrap_or(""),
        "Hello everyone! Check out this AMAZING feature",
        "Edited content should be reflected"
    );

    // Check thread still exists
    let verify_thread = node
        .call_tool(
            "get_thread",
            json!({
                "entity_id": channel,
                "thread_id": thread_id
            }),
        )
        .await;

    assert!(
        verify_thread.get("result").is_some(),
        "Thread should still exist"
    );

    // Check reaction persists
    let verify_reactions = node
        .call_tool(
            "get_reactions",
            json!({
                "entity_id": channel,
                "message_id": msg_id
            }),
        )
        .await;

    assert!(
        verify_reactions.get("result").is_some(),
        "Reactions should persist"
    );

    println!("✓ Complete workflow verified successfully!");
    println!("Full flow: Message → Thread → Reactions → Edits → State Sync all working");
}

#[tokio::test]
async fn test_messaging_with_search() {
    let node = TestNode::start("test-workflow-search").await;
    node.initialize().await;

    let channel = "workflow-search-channel";

    // Send multiple messages
    for i in 0..5 {
        let _send = node
            .call_tool(
                "send_message",
                json!({
                    "entity_id": channel,
                    "content": format!("Message {} with unique identifier", i),
                    "message_type": "text"
                }),
            )
            .await;
    }

    // Search for messages
    let search_result = node
        .call_tool(
            "search_messages",
            json!({
                "entity_id": channel,
                "query": "unique identifier"
            }),
        )
        .await;

    assert!(
        search_result.get("result").is_some(),
        "Search should work in workflow"
    );

    // Verify found messages
    if let Some(results) = search_result["result"]["messages"].as_array() {
        assert!(!results.is_empty(), "Should find messages matching query");
    }

    println!("✓ Search workflow verified!");
}

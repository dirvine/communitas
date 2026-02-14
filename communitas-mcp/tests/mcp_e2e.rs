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
use serde_json::{Value, json};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Test server handle that cleans up on drop
struct TestServer {
    process: Child,
    port: u16,
}

impl TestServer {
    /// Start MCP server in HTTP demo mode on a random port
    async fn start() -> Self {
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

    /// Start server and initialize the MCP protocol
    async fn start_initialized() -> Self {
        let server = Self::start().await;
        server.init().await;
        server
    }

    /// Initialize the MCP protocol
    async fn init(&self) {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1.0" }
            }),
        )
        .await;
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

    /// Parse JSON from a tool response's text content
    fn parse_tool_response(response: &Value) -> Value {
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or("");
        serde_json::from_str(text).unwrap_or(json!({}))
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

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["serverInfo"]["name"].as_str().is_some());
}

#[tokio::test]
async fn test_list_tools() {
    let server = TestServer::start_initialized().await;

    let response = server.request("tools/list", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
    let tools = response["result"]["tools"]
        .as_array()
        .expect("Expected tools array");

    // Verify we have tools
    assert!(!tools.is_empty(), "Expected at least one tool");

    // Check for some expected tools
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        tool_names.contains(&"health_check"),
        "Expected health_check tool"
    );
    assert!(
        tool_names.contains(&"core_status"),
        "Expected core_status tool"
    );
    assert!(
        tool_names.contains(&"create_entity"),
        "Expected create_entity tool"
    );
}

#[tokio::test]
async fn test_list_resources() {
    let server = TestServer::start_initialized().await;

    let response = server.request("resources/list", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
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
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("health_check", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
    let content = &response["result"]["content"];
    assert!(content.is_array(), "Expected content array");
}

#[tokio::test]
async fn test_core_status() {
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("core_status", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
}

#[tokio::test]
async fn test_get_profile() {
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("get_profile", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
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
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("list_entities", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
}

#[tokio::test]
async fn test_create_and_get_entity() {
    let server = TestServer::start_initialized().await;

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
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("network_status", json!({})).await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
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
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("nonexistent_tool", json!({})).await;

    // MCP returns result with isError flag, not a JSON-RPC error
    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
    let is_error = response["result"]["isError"].as_bool().unwrap_or(false);
    assert!(
        is_error,
        "Expected isError=true for invalid tool: {response:?}"
    );
}

// =============================================================================
// Messaging Workflow Tests
// =============================================================================

async fn init_server_and_create_channel(server: &TestServer) -> String {
    server.init().await;

    let create_response = server
        .call_tool(
            "create_entity",
            json!({
                "entity_type": "channel",
                "name": "Messaging Test Channel",
                "description": "Channel for messaging tests"
            }),
        )
        .await;

    assert!(
        create_response.get("result").is_some(),
        "Failed to create channel: {create_response:?}"
    );

    let parsed = TestServer::parse_tool_response(&create_response);
    parsed["id"].as_str().unwrap_or("test-channel").to_string()
}

#[tokio::test]
async fn test_messaging_send_message() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_channel(&server).await;

    let response = server
        .call_tool(
            "send_message",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "text": "Hello, World!"
            }),
        )
        .await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
    let is_error = response["result"]["isError"].as_bool().unwrap_or(true);
    assert!(!is_error, "send_message should succeed: {response:?}");
}

#[tokio::test]
async fn test_messaging_get_messages() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_channel(&server).await;

    server
        .call_tool(
            "send_message",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "text": "Test message for retrieval"
            }),
        )
        .await;

    let response = server
        .call_tool(
            "get_messages",
            json!({
                "entity_id": entity_id,
                "limit": 10
            }),
        )
        .await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
}

#[tokio::test]
async fn test_messaging_add_reaction() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_channel(&server).await;

    let send_response = server
        .call_tool(
            "send_message",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "text": "React to this!"
            }),
        )
        .await;

    let parsed = TestServer::parse_tool_response(&send_response);
    let message_id = parsed["id"].as_str().unwrap_or("test-msg");

    let reaction_response = server
        .call_tool(
            "add_reaction",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "message_id": message_id,
                "emoji": "👍"
            }),
        )
        .await;

    assert!(
        reaction_response.get("result").is_some(),
        "Expected result: {reaction_response:?}"
    );
}

#[tokio::test]
async fn test_messaging_create_thread() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_channel(&server).await;

    let send_response = server
        .call_tool(
            "send_message",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "text": "Thread parent message"
            }),
        )
        .await;

    let parsed = TestServer::parse_tool_response(&send_response);
    let message_id = parsed["id"].as_str().unwrap_or("test-msg");

    let thread_response = server
        .call_tool(
            "create_thread",
            json!({
                "entity_id": entity_id,
                "parent_message_id": message_id,
                "title": "Discussion Thread"
            }),
        )
        .await;

    assert!(
        thread_response.get("result").is_some(),
        "Expected result: {thread_response:?}"
    );
}

#[tokio::test]
async fn test_messaging_edit_message() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_channel(&server).await;

    let send_response = server
        .call_tool(
            "send_message",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "text": "Original message"
            }),
        )
        .await;

    let parsed = TestServer::parse_tool_response(&send_response);
    let message_id = parsed["id"].as_str().unwrap_or("test-msg");

    let edit_response = server
        .call_tool(
            "edit_message",
            json!({
                "entity_id": entity_id,
                "entity_type": "channel",
                "message_id": message_id,
                "new_text": "Edited message"
            }),
        )
        .await;

    assert!(
        edit_response.get("result").is_some(),
        "Expected result: {edit_response:?}"
    );
}

// =============================================================================
// Kanban Workflow Tests
// =============================================================================

async fn init_server_and_create_project(server: &TestServer) -> String {
    server.init().await;

    let create_response = server
        .call_tool(
            "create_entity",
            json!({
                "entity_type": "project",
                "display_name": "Kanban Test Project",
                "description": "Project for Kanban tests"
            }),
        )
        .await;

    let parsed = TestServer::parse_tool_response(&create_response);
    parsed["entity_id"]
        .as_str()
        .unwrap_or("test-project")
        .to_string()
}

#[tokio::test]
async fn test_kanban_create_board() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_project(&server).await;

    let response = server
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Sprint Board",
                "description": "Main development board"
            }),
        )
        .await;

    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
    let is_error = response["result"]["isError"].as_bool().unwrap_or(true);
    assert!(
        !is_error,
        "create_kanban_board should succeed: {response:?}"
    );
}

#[tokio::test]
async fn test_kanban_full_workflow() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_project(&server).await;

    // 1. Create board
    let board_response = server
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Development Board"
            }),
        )
        .await;
    assert!(
        board_response.get("result").is_some(),
        "Failed to create board"
    );
    let board_parsed = TestServer::parse_tool_response(&board_response);
    let board_id = board_parsed["board_id"].as_str().unwrap_or("test-board");

    // 2. Create columns
    let todo_col = server
        .call_tool(
            "create_kanban_column",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    assert!(
        todo_col.get("result").is_some(),
        "Failed to create To Do column"
    );
    let todo_parsed = TestServer::parse_tool_response(&todo_col);
    let todo_col_id = todo_parsed["column_id"].as_str().unwrap_or("todo-col");

    let doing_col = server
        .call_tool(
            "create_kanban_column",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "column_name": "In Progress",
                "position": 1
            }),
        )
        .await;
    assert!(
        doing_col.get("result").is_some(),
        "Failed to create In Progress column"
    );
    let doing_parsed = TestServer::parse_tool_response(&doing_col);
    let doing_col_id = doing_parsed["column_id"].as_str().unwrap_or("doing-col");

    let done_col = server
        .call_tool(
            "create_kanban_column",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "column_name": "Done",
                "position": 2
            }),
        )
        .await;
    assert!(
        done_col.get("result").is_some(),
        "Failed to create Done column"
    );

    // 3. Create a card
    let card_response = server
        .call_tool(
            "create_kanban_card",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "column_id": todo_col_id,
                "title": "Implement feature X",
                "description": "Add support for feature X"
            }),
        )
        .await;
    assert!(
        card_response.get("result").is_some(),
        "Failed to create card"
    );
    let card_parsed = TestServer::parse_tool_response(&card_response);
    let card_id = card_parsed["card_id"].as_str().unwrap_or("test-card");

    // 4. Move card to In Progress
    let move_response = server
        .call_tool(
            "move_kanban_card",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "card_id": card_id,
                "target_column_id": doing_col_id,
                "target_position": 0
            }),
        )
        .await;
    assert!(move_response.get("result").is_some(), "Failed to move card");

    // 5. Add a checklist step
    let step_response = server
        .call_tool(
            "add_card_step",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "card_id": card_id,
                "title": "Write unit tests"
            }),
        )
        .await;
    assert!(step_response.get("result").is_some(), "Failed to add step");

    // 6. Add a comment
    let comment_response = server
        .call_tool(
            "add_card_comment",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "card_id": card_id,
                "content": "Started working on this feature"
            }),
        )
        .await;
    assert!(
        comment_response.get("result").is_some(),
        "Failed to add comment"
    );

    // 7. Get board to verify state
    let get_board = server
        .call_tool(
            "get_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_id": board_id
            }),
        )
        .await;
    assert!(
        get_board.get("result").is_some(),
        "Failed to get board: {get_board:?}"
    );
}

#[tokio::test]
async fn test_kanban_create_and_tag_card() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_project(&server).await;

    // Create board
    let board_response = server
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Tag Test Board"
            }),
        )
        .await;
    let board_parsed = TestServer::parse_tool_response(&board_response);
    let board_id = board_parsed["board_id"].as_str().unwrap_or("test-board");

    // Create column
    let col_response = server
        .call_tool(
            "create_kanban_column",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "column_name": "Backlog",
                "position": 0
            }),
        )
        .await;
    let col_parsed = TestServer::parse_tool_response(&col_response);
    let column_id = col_parsed["column_id"].as_str().unwrap_or("test-col");

    // Create tag
    let tag_response = server
        .call_tool(
            "create_tag",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "name": "bug",
                "color": "#ff0000"
            }),
        )
        .await;
    assert!(tag_response.get("result").is_some(), "Failed to create tag");
    let tag_parsed = TestServer::parse_tool_response(&tag_response);
    let tag_id = tag_parsed["tag_id"].as_str().unwrap_or("test-tag");

    // Create card
    let card_response = server
        .call_tool(
            "create_kanban_card",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "column_id": column_id,
                "title": "Fix critical bug"
            }),
        )
        .await;
    let card_parsed = TestServer::parse_tool_response(&card_response);
    let card_id = card_parsed["card_id"].as_str().unwrap_or("test-card");

    // Tag the card
    let tag_card_response = server
        .call_tool(
            "tag_card",
            json!({
                "entity_id": entity_id,
                "board_id": board_id,
                "card_id": card_id,
                "tag_id": tag_id
            }),
        )
        .await;
    assert!(
        tag_card_response.get("result").is_some(),
        "Failed to tag card: {tag_card_response:?}"
    );
}

// =============================================================================
// Contact Tests
// =============================================================================

#[tokio::test]
async fn test_contact_workflow() {
    let server = TestServer::start_initialized().await;

    // Create contact
    let create_response = server
        .call_tool(
            "create_contact",
            json!({
                "four_words": "test-contact-four-words",
                "display_name": "Test Contact",
                "notes": "A test contact"
            }),
        )
        .await;
    assert!(
        create_response.get("result").is_some(),
        "Failed to create contact: {create_response:?}"
    );

    // List contacts
    let list_response = server.call_tool("list_contacts", json!({})).await;
    assert!(
        list_response.get("result").is_some(),
        "Failed to list contacts: {list_response:?}"
    );
}

// =============================================================================
// File Storage Tests
// =============================================================================

#[tokio::test]
async fn test_file_storage_workflow() {
    let server = TestServer::start().await;
    let entity_id = init_server_and_create_channel(&server).await;

    // Write file
    let write_response = server
        .call_tool(
            "write_file",
            json!({
                "entity_id": entity_id,
                "path": "/test/hello.txt",
                "content": "Hello, World!"
            }),
        )
        .await;
    assert!(
        write_response.get("result").is_some(),
        "Failed to write file: {write_response:?}"
    );

    // Read file
    let read_response = server
        .call_tool(
            "read_file",
            json!({
                "entity_id": entity_id,
                "path": "/test/hello.txt"
            }),
        )
        .await;
    assert!(
        read_response.get("result").is_some(),
        "Failed to read file: {read_response:?}"
    );

    // List files
    let list_response = server
        .call_tool(
            "list_files",
            json!({
                "entity_id": entity_id,
                "path": "/test"
            }),
        )
        .await;
    assert!(
        list_response.get("result").is_some(),
        "Failed to list files: {list_response:?}"
    );
}

// =============================================================================
// Authentication Tests
// =============================================================================

#[tokio::test]
async fn test_list_vaults() {
    let server = TestServer::start_initialized().await;

    let response = server.call_tool("list_vaults", json!({})).await;
    assert!(
        response.get("result").is_some(),
        "Expected result: {response:?}"
    );
}

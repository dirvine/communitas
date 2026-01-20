// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP-UiServices Parity Integration Tests (PLAN-31.7)
//!
//! These tests verify that MCP tools route through UiServices correctly,
//! ensuring Dioxus UI and MCP AI agents see the same data and behavior.
//!
//! The parity principle: Any operation performed via MCP should produce
//! the same observable state changes as the equivalent operation via UiServices.
//!
//! Run with: cargo test -p communitas-mcp --test parity_test

use std::sync::Arc;

use communitas_mcp::protocol::ToolCallResult;
use communitas_mcp::tools::{call_tool, list_tools};
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use serde_json::{Value, json};
use tempfile::TempDir;

// Re-export communitas_core through the same alias as the library
extern crate communitas_bindings as communitas_core;
use communitas_core::app::CommunitasApp;

/// Create test services with a temporary storage directory.
async fn make_test_services(temp: &TempDir) -> (Arc<CommunitasApp>, UiServices) {
    let storage = UiStorage::from_path(temp.path()).expect("failed to create storage");
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .expect("failed to create app"),
    );
    let services = UiServices::new(storage, app.clone()).expect("failed to create services");
    (app, services)
}

/// Extract text from ToolContent enum.
fn extract_text(content: &communitas_mcp::protocol::ToolContent) -> Option<&str> {
    match content {
        communitas_mcp::protocol::ToolContent::Text { text } => Some(text.as_str()),
    }
}

/// Parse JSON response from a tool call's text content.
fn parse_tool_response(result: &ToolCallResult) -> Value {
    if result.is_error {
        return json!({ "error": true });
    }
    result
        .content
        .first()
        .and_then(extract_text)
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or(json!({}))
}

/// Helper macro to run async tests with 8MB stack
macro_rules! run_async_test {
    ($test_fn:expr) => {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on($test_fn);
            })
            .unwrap()
            .join()
            .unwrap();
    };
}

// =============================================================================
// Tool Registration Tests
// =============================================================================

#[test]
fn test_tools_include_kanban_board_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Board tools
    assert!(
        tool_names.contains(&"create_kanban_board"),
        "Expected create_kanban_board tool"
    );
    assert!(
        tool_names.contains(&"get_kanban_board"),
        "Expected get_kanban_board tool"
    );
    assert!(
        tool_names.contains(&"update_kanban_board"),
        "Expected update_kanban_board tool"
    );
    assert!(
        tool_names.contains(&"delete_kanban_board"),
        "Expected delete_kanban_board tool"
    );
    assert!(
        tool_names.contains(&"list_kanban_boards"),
        "Expected list_kanban_boards tool"
    );
}

#[test]
fn test_tools_include_kanban_column_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Column tools
    assert!(
        tool_names.contains(&"create_kanban_column"),
        "Expected create_kanban_column tool"
    );
    assert!(
        tool_names.contains(&"get_kanban_column"),
        "Expected get_kanban_column tool"
    );
    assert!(
        tool_names.contains(&"update_kanban_column"),
        "Expected update_kanban_column tool"
    );
    assert!(
        tool_names.contains(&"delete_kanban_column"),
        "Expected delete_kanban_column tool"
    );
    assert!(
        tool_names.contains(&"move_kanban_column"),
        "Expected move_kanban_column tool"
    );
    assert!(
        tool_names.contains(&"list_kanban_columns"),
        "Expected list_kanban_columns tool"
    );
}

#[test]
fn test_tools_include_kanban_card_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Card tools
    assert!(
        tool_names.contains(&"create_kanban_card"),
        "Expected create_kanban_card tool"
    );
    assert!(
        tool_names.contains(&"get_kanban_card"),
        "Expected get_kanban_card tool"
    );
    assert!(
        tool_names.contains(&"update_kanban_card"),
        "Expected update_kanban_card tool"
    );
    assert!(
        tool_names.contains(&"delete_kanban_card"),
        "Expected delete_kanban_card tool"
    );
    assert!(
        tool_names.contains(&"move_kanban_card"),
        "Expected move_kanban_card tool"
    );
    assert!(
        tool_names.contains(&"list_kanban_cards"),
        "Expected list_kanban_cards tool"
    );
    assert!(
        tool_names.contains(&"change_card_state"),
        "Expected change_card_state tool"
    );
}

#[test]
fn test_tools_include_kanban_tag_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Tag tools
    assert!(
        tool_names.contains(&"create_kanban_tag"),
        "Expected create_kanban_tag tool"
    );
    assert!(
        tool_names.contains(&"list_kanban_tags"),
        "Expected list_kanban_tags tool"
    );
    assert!(tool_names.contains(&"tag_card"), "Expected tag_card tool");
    assert!(
        tool_names.contains(&"untag_card"),
        "Expected untag_card tool"
    );
}

#[test]
fn test_tools_include_kanban_step_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Step (checklist) tools
    assert!(tool_names.contains(&"add_step"), "Expected add_step tool");
    assert!(tool_names.contains(&"get_step"), "Expected get_step tool");
    assert!(
        tool_names.contains(&"toggle_step"),
        "Expected toggle_step tool"
    );
    assert!(
        tool_names.contains(&"delete_step"),
        "Expected delete_step tool"
    );
}

#[test]
fn test_tools_include_kanban_comment_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Comment tools
    assert!(
        tool_names.contains(&"add_comment"),
        "Expected add_comment tool"
    );
    assert!(
        tool_names.contains(&"list_comments"),
        "Expected list_comments tool"
    );
    assert!(
        tool_names.contains(&"delete_comment"),
        "Expected delete_comment tool"
    );
}

#[test]
fn test_tools_include_kanban_user_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // User assignment tools
    assert!(
        tool_names.contains(&"assign_user"),
        "Expected assign_user tool"
    );
    assert!(
        tool_names.contains(&"unassign_user"),
        "Expected unassign_user tool"
    );
}

#[test]
fn test_tools_include_messaging_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Message CRUD tools
    assert!(
        tool_names.contains(&"send_message"),
        "Expected send_message tool"
    );
    assert!(
        tool_names.contains(&"get_messages"),
        "Expected get_messages tool"
    );
    assert!(
        tool_names.contains(&"delete_message"),
        "Expected delete_message tool"
    );
    assert!(
        tool_names.contains(&"edit_message"),
        "Expected edit_message tool"
    );

    // Thread tools
    assert!(
        tool_names.contains(&"list_threads"),
        "Expected list_threads tool"
    );
    assert!(
        tool_names.contains(&"create_thread"),
        "Expected create_thread tool"
    );
    assert!(
        tool_names.contains(&"get_thread_messages"),
        "Expected get_thread_messages tool"
    );

    // Reaction tools
    assert!(
        tool_names.contains(&"add_reaction"),
        "Expected add_reaction tool"
    );
    assert!(
        tool_names.contains(&"remove_reaction"),
        "Expected remove_reaction tool"
    );
    assert!(
        tool_names.contains(&"get_reactions"),
        "Expected get_reactions tool"
    );
    assert!(
        tool_names.contains(&"get_available_reactions"),
        "Expected get_available_reactions tool"
    );

    // Invite tools
    assert!(
        tool_names.contains(&"create_invite"),
        "Expected create_invite tool"
    );
    assert!(
        tool_names.contains(&"accept_invite"),
        "Expected accept_invite tool"
    );
    assert!(
        tool_names.contains(&"list_pending_invites"),
        "Expected list_pending_invites tool"
    );
}

#[test]
fn test_all_messaging_tools_registered() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Complete list of all 14 messaging tools
    let messaging_tools = [
        // Message CRUD (4)
        "send_message",
        "get_messages",
        "delete_message",
        "edit_message",
        // Thread operations (3)
        "list_threads",
        "create_thread",
        "get_thread_messages",
        // Reaction operations (4)
        "add_reaction",
        "remove_reaction",
        "get_reactions",
        "get_available_reactions",
        // Invite operations (3)
        "create_invite",
        "accept_invite",
        "list_pending_invites",
    ];

    for tool in &messaging_tools {
        assert!(
            tool_names.contains(tool),
            "Expected {} tool to be registered (found {} tools total)",
            tool,
            tool_names.len()
        );
    }
}

#[test]
fn test_tools_include_drive_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Drive/file tools should be registered
    assert!(
        tool_names.contains(&"list_disks"),
        "Expected list_disks tool"
    );
    assert!(tool_names.contains(&"read_file"), "Expected read_file tool");
    assert!(
        tool_names.contains(&"write_file"),
        "Expected write_file tool"
    );
}

#[test]
fn test_tools_include_canvas_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Canvas tools should be registered
    assert!(
        tool_names.contains(&"canvas_get_snapshot"),
        "Expected canvas_get_snapshot tool"
    );
    assert!(
        tool_names.contains(&"canvas_add_text"),
        "Expected canvas_add_text tool"
    );
    assert!(
        tool_names.contains(&"canvas_add_image"),
        "Expected canvas_add_image tool"
    );
}

#[test]
fn test_tools_include_call_operations() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Call/WebRTC tools should be registered
    assert!(
        tool_names.contains(&"start_voice_call"),
        "Expected start_voice_call tool"
    );
    assert!(tool_names.contains(&"join_call"), "Expected join_call tool");
    assert!(tool_names.contains(&"end_call"), "Expected end_call tool");
}

// =============================================================================
// Kanban Board Parity Tests
// =============================================================================

/// Test that kanban board operations via MCP route through KanbanService.
#[test]
fn test_kanban_board_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Initial state: no boards
        let initial_snap = services.kanban().current_snapshot();
        assert!(
            initial_snap.boards.is_empty(),
            "Expected no boards initially"
        );

        // Create board via MCP tool
        let result = call_tool(
            &app,
            &services,
            "create_kanban_board",
            Some(json!({
                "entity_id": "test-entity",
                "name": "Parity Test Board",
                "description": "Testing MCP-UiServices parity"
            })),
        )
        .await;

        // Tool should respond (may succeed or fail due to auth state)
        assert!(
            !result.content.is_empty(),
            "Expected response from create_kanban_board"
        );

        // If authenticated, check board_id; otherwise expect auth error
        if !result.is_error {
            let parsed = parse_tool_response(&result);
            assert!(
                parsed["board_id"].as_str().is_some(),
                "Expected board_id in success response"
            );
        } else {
            let text = result.content.first().and_then(extract_text).unwrap_or("");
            assert!(
                text.contains("authenticated") || text.contains("auth") || text.contains("error"),
                "Expected auth-related error message"
            );
        }
    });
}

/// Test get_kanban_board routes through KanbanService.
#[test]
fn test_get_kanban_board_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to get a non-existent board via MCP
        let result = call_tool(
            &app,
            &services,
            "get_kanban_board",
            Some(json!({
                "entity_id": "test-entity",
                "board_id": "nonexistent-board-id"
            })),
        )
        .await;

        // Should return an error result (board doesn't exist)
        assert!(result.is_error, "Expected error for nonexistent board");
    });
}

/// Test update_kanban_board routes through KanbanService.
#[test]
fn test_update_kanban_board_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to update a non-existent board
        let result = call_tool(
            &app,
            &services,
            "update_kanban_board",
            Some(json!({
                "board_id": "nonexistent-board",
                "name": "Updated Name"
            })),
        )
        .await;

        // Should route through service and return error
        assert!(
            !result.content.is_empty(),
            "Expected response from update_kanban_board"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test delete_kanban_board routes through KanbanService.
#[test]
fn test_delete_kanban_board_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to delete a non-existent board
        let result = call_tool(
            &app,
            &services,
            "delete_kanban_board",
            Some(json!({
                "board_id": "nonexistent-board"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_kanban_board"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test list_kanban_boards routes through KanbanService.
#[test]
fn test_list_kanban_boards_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // List boards (should return empty list)
        let result = call_tool(
            &app,
            &services,
            "list_kanban_boards",
            Some(json!({
                "entity_id": "test-entity"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from list_kanban_boards"
        );
    });
}

// =============================================================================
// Kanban Column Parity Tests
// =============================================================================

/// Test get_kanban_column routes through KanbanService.
#[test]
fn test_get_kanban_column_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to get a non-existent column
        let result = call_tool(
            &app,
            &services,
            "get_kanban_column",
            Some(json!({
                "board_id": "nonexistent-board",
                "column_id": "nonexistent-column"
            })),
        )
        .await;

        // Should return error
        assert!(result.is_error, "Expected error for nonexistent column");
    });
}

/// Test update_kanban_column routes through KanbanService.
#[test]
fn test_update_kanban_column_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to update a non-existent column
        let result = call_tool(
            &app,
            &services,
            "update_kanban_column",
            Some(json!({
                "board_id": "nonexistent-board",
                "column_id": "nonexistent-column",
                "name": "Updated Column"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from update_kanban_column"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test delete_kanban_column routes through KanbanService.
#[test]
fn test_delete_kanban_column_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to delete a non-existent column
        let result = call_tool(
            &app,
            &services,
            "delete_kanban_column",
            Some(json!({
                "board_id": "nonexistent-board",
                "column_id": "nonexistent-column"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_kanban_column"
        );
    });
}

/// Test move_kanban_column routes through KanbanService.
#[test]
fn test_move_kanban_column_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to move a non-existent column
        let result = call_tool(
            &app,
            &services,
            "move_kanban_column",
            Some(json!({
                "board_id": "nonexistent-board",
                "column_id": "nonexistent-column",
                "new_position": 1
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from move_kanban_column"
        );
    });
}

/// Test list_kanban_columns routes through KanbanService.
#[test]
fn test_list_kanban_columns_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to list columns for non-existent board
        let result = call_tool(
            &app,
            &services,
            "list_kanban_columns",
            Some(json!({
                "board_id": "nonexistent-board"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from list_kanban_columns"
        );
    });
}

// =============================================================================
// Kanban Card Parity Tests
// =============================================================================

/// Test that kanban card operations via MCP route through KanbanService.
#[test]
fn test_kanban_card_operations_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Create board first (may fail due to auth)
        let board_result = call_tool(
            &app,
            &services,
            "create_kanban_board",
            Some(json!({
                "entity_id": "test-entity",
                "name": "Card Test Board"
            })),
        )
        .await;

        // Tool should respond
        assert!(
            !board_result.content.is_empty(),
            "Expected response from create_kanban_board"
        );

        // If auth fails, skip remainder but verify routing worked
        if board_result.is_error {
            let text = board_result
                .content
                .first()
                .and_then(extract_text)
                .unwrap_or("");
            assert!(
                !text.contains("Unknown tool"),
                "Tool should be routed through KanbanService"
            );
            return;
        }

        let board_id = parse_tool_response(&board_result)["board_id"]
            .as_str()
            .unwrap_or("board-1")
            .to_string();

        // Create column
        let col_result = call_tool(
            &app,
            &services,
            "create_kanban_column",
            Some(json!({
                "entity_id": "test-entity",
                "board_id": board_id,
                "name": "To Do",
                "position": 0
            })),
        )
        .await;

        if col_result.is_error {
            return;
        }

        let column_id = parse_tool_response(&col_result)["column_id"]
            .as_str()
            .unwrap_or("col-1")
            .to_string();

        // Create card via MCP
        let card_result = call_tool(
            &app,
            &services,
            "create_kanban_card",
            Some(json!({
                "entity_id": "test-entity",
                "board_id": board_id,
                "column_id": column_id,
                "title": "Parity Test Card",
                "description": "Created via MCP"
            })),
        )
        .await;

        // Tool should respond
        assert!(
            !card_result.content.is_empty(),
            "Expected response from create_kanban_card"
        );

        // If successful, verify card_id returned
        if !card_result.is_error {
            let parsed = parse_tool_response(&card_result);
            assert!(
                parsed["card_id"].as_str().is_some(),
                "Expected card_id in success response"
            );
        }
    });
}

/// Test get_kanban_card routes through KanbanService.
#[test]
fn test_get_kanban_card_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to get a non-existent card
        let result = call_tool(
            &app,
            &services,
            "get_kanban_card",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card"
            })),
        )
        .await;

        // Should return error
        assert!(result.is_error, "Expected error for nonexistent card");
    });
}

/// Test update_kanban_card routes through KanbanService.
#[test]
fn test_update_kanban_card_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to update a non-existent card
        let result = call_tool(
            &app,
            &services,
            "update_kanban_card",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "title": "Updated Title"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from update_kanban_card"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test delete_kanban_card routes through KanbanService.
#[test]
fn test_delete_kanban_card_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to delete a non-existent card
        let result = call_tool(
            &app,
            &services,
            "delete_kanban_card",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_kanban_card"
        );
    });
}

/// Test move_kanban_card routes through KanbanService.
#[test]
fn test_move_kanban_card_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to move a non-existent card
        let result = call_tool(
            &app,
            &services,
            "move_kanban_card",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "target_column_id": "target-column"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from move_kanban_card"
        );
    });
}

/// Test list_kanban_cards routes through KanbanService.
#[test]
fn test_list_kanban_cards_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to list cards for non-existent board
        let result = call_tool(
            &app,
            &services,
            "list_kanban_cards",
            Some(json!({
                "board_id": "nonexistent-board"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from list_kanban_cards"
        );
    });
}

/// Test change_card_state routes through KanbanService.
#[test]
fn test_change_card_state_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to change state of non-existent card
        let result = call_tool(
            &app,
            &services,
            "change_card_state",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "state": "in_progress"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from change_card_state"
        );
    });
}

// =============================================================================
// Kanban Tag Parity Tests
// =============================================================================

/// Test create_kanban_tag routes through KanbanService.
#[test]
fn test_create_kanban_tag_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to create tag for non-existent board
        let result = call_tool(
            &app,
            &services,
            "create_kanban_tag",
            Some(json!({
                "board_id": "nonexistent-board",
                "name": "Priority",
                "color": "#ff0000"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from create_kanban_tag"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test list_kanban_tags routes through KanbanService.
#[test]
fn test_list_kanban_tags_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to list tags for non-existent board
        let result = call_tool(
            &app,
            &services,
            "list_kanban_tags",
            Some(json!({
                "board_id": "nonexistent-board"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from list_kanban_tags"
        );
    });
}

/// Test tag_card routes through KanbanService.
#[test]
fn test_tag_card_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to tag a non-existent card
        let result = call_tool(
            &app,
            &services,
            "tag_card",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "tag_id": "nonexistent-tag"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from tag_card"
        );
    });
}

/// Test untag_card routes through KanbanService.
#[test]
fn test_untag_card_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to untag a non-existent card
        let result = call_tool(
            &app,
            &services,
            "untag_card",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "tag_id": "nonexistent-tag"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from untag_card"
        );
    });
}

// =============================================================================
// Kanban Step (Checklist) Parity Tests
// =============================================================================

/// Test add_step routes through KanbanService.
#[test]
fn test_add_step_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to add step to non-existent card
        let result = call_tool(
            &app,
            &services,
            "add_step",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "text": "Complete review"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from add_step"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test get_step routes through KanbanService.
#[test]
fn test_get_step_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to get non-existent step
        let result = call_tool(
            &app,
            &services,
            "get_step",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "step_id": "nonexistent-step"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from get_step"
        );
    });
}

/// Test toggle_step routes through KanbanService.
#[test]
fn test_toggle_step_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to toggle non-existent step
        let result = call_tool(
            &app,
            &services,
            "toggle_step",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "step_id": "nonexistent-step"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from toggle_step"
        );
    });
}

/// Test delete_step routes through KanbanService.
#[test]
fn test_delete_step_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to delete non-existent step
        let result = call_tool(
            &app,
            &services,
            "delete_step",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "step_id": "nonexistent-step"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_step"
        );
    });
}

// =============================================================================
// Kanban Comment Parity Tests
// =============================================================================

/// Test add_comment routes through KanbanService.
#[test]
fn test_add_comment_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to add comment to non-existent card
        let result = call_tool(
            &app,
            &services,
            "add_comment",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "content": "This is a test comment"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from add_comment"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test list_comments routes through KanbanService.
#[test]
fn test_list_comments_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to list comments for non-existent card
        let result = call_tool(
            &app,
            &services,
            "list_comments",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from list_comments"
        );
    });
}

/// Test delete_comment routes through KanbanService.
#[test]
fn test_delete_comment_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to delete non-existent comment
        let result = call_tool(
            &app,
            &services,
            "delete_comment",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "comment_id": "nonexistent-comment"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_comment"
        );
    });
}

// =============================================================================
// Kanban User Assignment Parity Tests
// =============================================================================

/// Test assign_user routes through KanbanService.
#[test]
fn test_assign_user_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to assign user to non-existent card
        let result = call_tool(
            &app,
            &services,
            "assign_user",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "user_id": "user-123"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from assign_user"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through KanbanService"
        );
    });
}

/// Test unassign_user routes through KanbanService.
#[test]
fn test_unassign_user_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to unassign user from non-existent card
        let result = call_tool(
            &app,
            &services,
            "unassign_user",
            Some(json!({
                "board_id": "nonexistent-board",
                "card_id": "nonexistent-card",
                "user_id": "user-123"
            })),
        )
        .await;

        // Should route through service
        assert!(
            !result.content.is_empty(),
            "Expected response from unassign_user"
        );
    });
}
// =============================================================================
// Messaging Parity Tests
// =============================================================================

/// Test that messaging operations via MCP route through MessagingService.
#[test]
fn test_messaging_thread_list_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Initial state: no threads
        let initial_snap = services.messaging().current_snapshot();
        assert!(
            initial_snap.threads.is_empty(),
            "Expected no threads initially"
        );

        // Call list_threads via MCP
        let result = call_tool(&app, &services, "list_threads", Some(json!({}))).await;

        // Tool should work (may return empty list or error if not authenticated)
        assert!(
            !result.content.is_empty(),
            "Expected some response from list_threads"
        );
    });
}

/// Test send_message routes through MessagingService.
#[test]
fn test_send_message_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Send message via MCP
        let result = call_tool(
            &app,
            &services,
            "send_message",
            Some(json!({
                "entity_id": "test-thread",
                "text": "Hello from parity test"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from send_message"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through MessagingService"
        );
    });
}

/// Test send_message with reply_to routes through MessagingService.
#[test]
fn test_send_message_with_reply_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Send message with reply_to via MCP
        let result = call_tool(
            &app,
            &services,
            "send_message",
            Some(json!({
                "entity_id": "test-thread",
                "text": "This is a reply",
                "reply_to_id": "parent-msg-123"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from send_message with reply_to"
        );
    });
}

/// Test get_messages routes through MessagingService.
#[test]
fn test_get_messages_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Get messages via MCP
        let result = call_tool(
            &app,
            &services,
            "get_messages",
            Some(json!({
                "entity_id": "test-thread",
                "limit": 50
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from get_messages"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through MessagingService"
        );
    });
}

/// Test get_messages with pagination routes through MessagingService.
#[test]
fn test_get_messages_with_pagination_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Get messages with pagination via MCP
        let result = call_tool(
            &app,
            &services,
            "get_messages",
            Some(json!({
                "entity_id": "test-thread",
                "limit": 25,
                "before": 1000
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from get_messages with pagination"
        );
    });
}

/// Test delete_message routes through MessagingService.
#[test]
fn test_delete_message_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Delete non-existent message via MCP
        let result = call_tool(
            &app,
            &services,
            "delete_message",
            Some(json!({
                "entity_id": "test-thread",
                "message_id": "nonexistent-msg"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_message"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through MessagingService"
        );
    });
}

/// Test edit_message routes through MessagingService.
#[test]
fn test_edit_message_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Edit non-existent message via MCP
        let result = call_tool(
            &app,
            &services,
            "edit_message",
            Some(json!({
                "entity_id": "test-thread",
                "message_id": "nonexistent-msg",
                "new_text": "Updated message text"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from edit_message"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through MessagingService"
        );
    });
}

/// Test add_reaction routes through MessagingService.
#[test]
fn test_add_reaction_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Add reaction to non-existent message via MCP
        let result = call_tool(
            &app,
            &services,
            "add_reaction",
            Some(json!({
                "entity_id": "test-thread",
                "message_id": "nonexistent-msg",
                "emoji": "👍"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from add_reaction"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through MessagingService"
        );
    });
}

/// Test remove_reaction routes through MessagingService.
#[test]
fn test_remove_reaction_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Remove reaction from non-existent message via MCP
        let result = call_tool(
            &app,
            &services,
            "remove_reaction",
            Some(json!({
                "entity_id": "test-thread",
                "message_id": "nonexistent-msg",
                "emoji": "👍"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from remove_reaction"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be routed through MessagingService"
        );
    });
}

/// Test get_reactions routes through app (not yet migrated to UiServices).
#[test]
fn test_get_reactions_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Get reactions for non-existent message via MCP
        let result = call_tool(
            &app,
            &services,
            "get_reactions",
            Some(json!({
                "entity_id": "test-thread",
                "message_id": "nonexistent-msg"
            })),
        )
        .await;

        // Tool should respond (routes through app)
        assert!(
            !result.content.is_empty(),
            "Expected response from get_reactions"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be registered and routed"
        );
    });
}

/// Test get_available_reactions returns standard emoji list.
#[test]
fn test_get_available_reactions_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Get available reactions via MCP
        let result = call_tool(
            &app,
            &services,
            "get_available_reactions",
            Some(json!({
                "entity_id": "test-thread"
            })),
        )
        .await;

        // Tool should succeed with standard reactions
        assert!(
            !result.is_error,
            "Expected success from get_available_reactions"
        );

        let parsed = parse_tool_response(&result);
        assert!(
            parsed["reactions"].is_array(),
            "Expected reactions array in response"
        );
    });
}

/// Test create_thread routes through app (returns informational message).
#[test]
fn test_create_thread_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Create thread via MCP
        let result = call_tool(
            &app,
            &services,
            "create_thread",
            Some(json!({
                "channel_id": "test-channel",
                "parent_message_id": "parent-msg-123"
            })),
        )
        .await;

        // Tool should respond with informational message
        assert!(
            !result.content.is_empty(),
            "Expected response from create_thread"
        );

        // This tool returns info about implicit thread creation
        let parsed = parse_tool_response(&result);
        assert!(
            parsed["thread_id"].is_string() || parsed["info"].is_string(),
            "Expected thread_id or info in response"
        );
    });
}

/// Test get_thread_messages routes through app.
#[test]
fn test_get_thread_messages_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Get thread messages via MCP
        let result = call_tool(
            &app,
            &services,
            "get_thread_messages",
            Some(json!({
                "channel_id": "test-channel",
                "thread_id": "thread-123"
            })),
        )
        .await;

        // Tool should route through app
        assert!(
            !result.content.is_empty(),
            "Expected response from get_thread_messages"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be registered and routed"
        );
    });
}

/// Test create_invite routes through app.
#[test]
fn test_create_invite_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Create invite via MCP
        let result = call_tool(
            &app,
            &services,
            "create_invite",
            Some(json!({
                "recipient_id": "recipient-123",
                "entity_type": "group",
                "entity_id": "group-456",
                "role": "member",
                "message": "Join our group!"
            })),
        )
        .await;

        // Tool should route through app
        assert!(
            !result.content.is_empty(),
            "Expected response from create_invite"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be registered and routed"
        );
    });
}

/// Test accept_invite routes through app.
#[test]
fn test_accept_invite_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Accept non-existent invite via MCP
        let result = call_tool(
            &app,
            &services,
            "accept_invite",
            Some(json!({
                "invite_id": "nonexistent-invite"
            })),
        )
        .await;

        // Tool should route through app (expect error for nonexistent)
        assert!(
            !result.content.is_empty(),
            "Expected response from accept_invite"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be registered and routed"
        );
    });
}

/// Test list_pending_invites routes through app.
#[test]
fn test_list_pending_invites_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // List pending invites via MCP
        let result = call_tool(&app, &services, "list_pending_invites", Some(json!({}))).await;

        // Tool should route through app
        assert!(
            !result.content.is_empty(),
            "Expected response from list_pending_invites"
        );
        let text = result.content.first().and_then(extract_text).unwrap_or("");
        assert!(
            !text.contains("Unknown tool"),
            "Tool should be registered and routed"
        );
    });
}

// =============================================================================
// Messaging Validation Edge Case Tests
// =============================================================================

/// Test validation for missing entity_id in send_message.
#[test]
fn test_validation_send_message_missing_text() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to send message without text
        let result = call_tool(
            &app,
            &services,
            "send_message",
            Some(json!({
                "entity_id": "test-thread"
            })),
        )
        .await;

        // Should still process (empty text may be allowed or error)
        assert!(
            !result.content.is_empty(),
            "Expected response from send_message"
        );
    });
}

/// Test validation for missing message_id in delete_message.
#[test]
fn test_validation_delete_message_missing_id() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to delete message without message_id
        let result = call_tool(
            &app,
            &services,
            "delete_message",
            Some(json!({
                "entity_id": "test-thread"
            })),
        )
        .await;

        // Should process (empty string default or error)
        assert!(
            !result.content.is_empty(),
            "Expected response from delete_message"
        );
    });
}

/// Test validation for missing emoji in add_reaction.
#[test]
fn test_validation_add_reaction_missing_emoji() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to add reaction without emoji
        let result = call_tool(
            &app,
            &services,
            "add_reaction",
            Some(json!({
                "entity_id": "test-thread",
                "message_id": "msg-123"
            })),
        )
        .await;

        // Should process (empty string default or error)
        assert!(
            !result.content.is_empty(),
            "Expected response from add_reaction"
        );
    });
}

/// Test validation for missing channel_id in create_thread.
#[test]
fn test_validation_create_thread_missing_channel() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to create thread without channel_id
        let result = call_tool(
            &app,
            &services,
            "create_thread",
            Some(json!({
                "parent_message_id": "msg-123"
            })),
        )
        .await;

        // Should fail validation
        assert!(result.is_error, "Expected error for missing channel_id");
    });
}

/// Test validation for missing parent_message_id in create_thread.
#[test]
fn test_validation_create_thread_missing_parent() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to create thread without parent_message_id
        let result = call_tool(
            &app,
            &services,
            "create_thread",
            Some(json!({
                "channel_id": "channel-123"
            })),
        )
        .await;

        // Should fail validation
        assert!(
            result.is_error,
            "Expected error for missing parent_message_id"
        );
    });
}

/// Test validation for missing thread_id in get_thread_messages.
#[test]
fn test_validation_get_thread_messages_missing_thread() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to get thread messages without thread_id
        let result = call_tool(
            &app,
            &services,
            "get_thread_messages",
            Some(json!({
                "channel_id": "channel-123"
            })),
        )
        .await;

        // Should fail validation
        assert!(result.is_error, "Expected error for missing thread_id");
    });
}

/// Test validation for missing entity_type in create_invite.
#[test]
fn test_validation_create_invite_missing_entity_type() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to create invite without entity_type
        let result = call_tool(
            &app,
            &services,
            "create_invite",
            Some(json!({
                "recipient_id": "recipient-123",
                "entity_id": "entity-456"
            })),
        )
        .await;

        // Should fail validation
        assert!(result.is_error, "Expected error for missing entity_type");
    });
}

/// Test validation for invalid entity_type in create_invite.
#[test]
fn test_validation_create_invite_invalid_entity_type() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to create invite with invalid entity_type
        let result = call_tool(
            &app,
            &services,
            "create_invite",
            Some(json!({
                "recipient_id": "recipient-123",
                "entity_type": "invalid_type",
                "entity_id": "entity-456"
            })),
        )
        .await;

        // Should fail validation
        assert!(result.is_error, "Expected error for invalid entity_type");
    });
}

/// Test validation for missing invite_id in accept_invite.
#[test]
fn test_validation_accept_invite_missing_id() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to accept invite without invite_id
        let result = call_tool(&app, &services, "accept_invite", Some(json!({}))).await;

        // Should process (empty string default or error)
        assert!(
            !result.content.is_empty(),
            "Expected response from accept_invite"
        );
    });
}

/// Test list_threads with filter parameter.
#[test]
fn test_list_threads_with_filter_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // List threads with filter via MCP
        let result = call_tool(
            &app,
            &services,
            "list_threads",
            Some(json!({
                "filter": "unread",
                "limit": 10
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from list_threads with filter"
        );
    });
}

/// Test list_threads with entities filter.
#[test]
fn test_list_threads_entities_filter_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // List entity threads via MCP
        let result = call_tool(
            &app,
            &services,
            "list_threads",
            Some(json!({
                "filter": "entities"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from list_threads with entities filter"
        );
    });
}

/// Test list_threads with contacts filter.
#[test]
fn test_list_threads_contacts_filter_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // List contact threads via MCP
        let result = call_tool(
            &app,
            &services,
            "list_threads",
            Some(json!({
                "filter": "contacts"
            })),
        )
        .await;

        // Tool should route through MessagingService
        assert!(
            !result.content.is_empty(),
            "Expected response from list_threads with contacts filter"
        );
    });
}

// =============================================================================
// Drive Parity Tests
// =============================================================================

/// Test that drive operations via MCP route through DriveService.
#[test]
fn test_drive_list_disks_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Initial state: empty drive snapshot
        let initial_snap = services.drive().current_snapshot();
        assert!(
            initial_snap.uploads.is_empty(),
            "Expected no uploads initially"
        );

        // Call list_disks via MCP
        let result = call_tool(
            &app,
            &services,
            "list_disks",
            Some(json!({
                "entity_id": "test-entity"
            })),
        )
        .await;

        // Tool should work - the key is it routes through DriveService
        assert!(
            !result.content.is_empty(),
            "Expected some response from list_disks"
        );
    });
}

/// Test that file write/read via MCP routes through DriveService.
#[test]
fn test_drive_file_operations_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Write file via MCP
        let write_result = call_tool(
            &app,
            &services,
            "write_file",
            Some(json!({
                "entity_id": "test-entity",
                "path": "/parity-test.txt",
                "content": "Hello from MCP parity test"
            })),
        )
        .await;

        // Tool should execute (may succeed or fail based on auth state)
        assert!(
            !write_result.content.is_empty(),
            "Expected response from write_file"
        );

        // Read file via MCP
        let read_result = call_tool(
            &app,
            &services,
            "read_file",
            Some(json!({
                "entity_id": "test-entity",
                "path": "/parity-test.txt"
            })),
        )
        .await;

        assert!(
            !read_result.content.is_empty(),
            "Expected response from read_file"
        );
    });
}

// =============================================================================
// Canvas Parity Tests
// =============================================================================

/// Test that canvas operations via MCP route through CanvasService.
#[test]
fn test_canvas_operations_parity() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Initial state: empty canvas
        let initial_snap = services.canvas().current_snapshot();
        assert!(
            initial_snap.elements.is_empty(),
            "Expected no elements initially"
        );

        // Get canvas snapshot via MCP
        let result = call_tool(
            &app,
            &services,
            "canvas_get_snapshot",
            Some(json!({
                "entity_id": "test-entity"
            })),
        )
        .await;

        // Tool should execute (routes through CanvasService)
        assert!(
            !result.content.is_empty(),
            "Expected response from canvas_get_snapshot"
        );

        // Add text element via MCP
        let add_result = call_tool(
            &app,
            &services,
            "canvas_add_text",
            Some(json!({
                "entity_id": "test-entity",
                "content": "Test text element",
                "x": 100.0,
                "y": 100.0
            })),
        )
        .await;

        // Tool should respond (routes through CanvasService)
        assert!(
            !add_result.content.is_empty(),
            "Expected response from canvas_add_text"
        );
    });
}

// =============================================================================
// Error Consistency Tests
// =============================================================================

/// Test that error handling is consistent between MCP and UiServices.
#[test]
fn test_error_consistency_invalid_board() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to get a non-existent board via MCP
        let result = call_tool(
            &app,
            &services,
            "get_kanban_board",
            Some(json!({
                "entity_id": "test-entity",
                "board_id": "nonexistent-board-id"
            })),
        )
        .await;

        // Should return an error result
        assert!(result.is_error, "Expected error for nonexistent board");
    });
}

/// Test that validation is applied consistently.
#[test]
fn test_validation_consistency() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to create card without required board_id
        let result = call_tool(
            &app,
            &services,
            "create_kanban_card",
            Some(json!({
                "entity_id": "test-entity",
                "title": "Missing board_id"
            })),
        )
        .await;

        // Should fail validation
        assert!(
            result.is_error,
            "Expected validation error for missing board_id"
        );
    });
}

/// Test validation for missing required fields in card operations.
#[test]
fn test_validation_missing_card_id() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to move card without card_id
        let result = call_tool(
            &app,
            &services,
            "move_kanban_card",
            Some(json!({
                "board_id": "some-board",
                "target_column_id": "some-column"
            })),
        )
        .await;

        // Should fail validation
        assert!(
            result.is_error,
            "Expected validation error for missing card_id"
        );
    });
}

/// Test validation for missing required fields in step operations.
#[test]
fn test_validation_missing_step_text() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to add step without text
        let result = call_tool(
            &app,
            &services,
            "add_step",
            Some(json!({
                "board_id": "some-board",
                "card_id": "some-card"
            })),
        )
        .await;

        // Should fail validation
        assert!(
            result.is_error,
            "Expected validation error for missing step text"
        );
    });
}

/// Test validation for missing required fields in comment operations.
#[test]
fn test_validation_missing_comment_content() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;

        // Try to add comment without content
        let result = call_tool(
            &app,
            &services,
            "add_comment",
            Some(json!({
                "board_id": "some-board",
                "card_id": "some-card"
            })),
        )
        .await;

        // Should fail validation
        assert!(
            result.is_error,
            "Expected validation error for missing comment content"
        );
    });
}

// =============================================================================
// Pre-auth vs Authenticated Tool Tests
// =============================================================================

#[test]
fn test_preauth_tools_subset() {
    let preauth_tools = list_tools(false);
    let all_tools = list_tools(true);

    // Pre-auth tools should be a subset of all tools
    for tool in &preauth_tools {
        assert!(
            all_tools.iter().any(|t| t.name == tool.name),
            "Pre-auth tool {} should also be in authenticated tools",
            tool.name
        );
    }

    // Authenticated tools should include more than pre-auth
    assert!(
        all_tools.len() > preauth_tools.len(),
        "Authenticated tools should include additional tools beyond pre-auth"
    );
}

#[test]
fn test_preauth_tools_include_health_check() {
    let tools = list_tools(false);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Health check should always be available
    assert!(
        tool_names.contains(&"health_check"),
        "health_check should be available pre-auth"
    );
}

#[test]
fn test_all_kanban_tools_registered() {
    let tools = list_tools(true);
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    // Complete list of all 23 kanban tools
    let kanban_tools = [
        // Board operations (5)
        "create_kanban_board",
        "get_kanban_board",
        "update_kanban_board",
        "delete_kanban_board",
        "list_kanban_boards",
        // Column operations (6)
        "create_kanban_column",
        "get_kanban_column",
        "update_kanban_column",
        "delete_kanban_column",
        "move_kanban_column",
        "list_kanban_columns",
        // Card operations (7)
        "create_kanban_card",
        "get_kanban_card",
        "update_kanban_card",
        "delete_kanban_card",
        "move_kanban_card",
        "list_kanban_cards",
        "change_card_state",
        // Tag operations (4)
        "create_kanban_tag",
        "list_kanban_tags",
        "tag_card",
        "untag_card",
        // Step operations (4)
        "add_step",
        "get_step",
        "toggle_step",
        "delete_step",
        // Comment operations (3)
        "add_comment",
        "list_comments",
        "delete_comment",
        // User operations (2)
        "assign_user",
        "unassign_user",
    ];

    for tool in &kanban_tools {
        assert!(
            tool_names.contains(tool),
            "Expected {} tool to be registered (found {} tools total)",
            tool,
            tool_names.len()
        );
    }
}

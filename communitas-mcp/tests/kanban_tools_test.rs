// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Phase 10.4: Kanban Tools Tests
//!
//! Comprehensive tests for Kanban boards, columns, cards, and tags.
//! Tests cover 22 tools across 4 categories.

mod harness;

use harness::{McpTestNode, ToolAssert};
use serde_json::json;

// Helper function similar to comprehensive_e2e.rs
async fn start_node(name: &str) -> McpTestNode {
    let node = McpTestNode::start(name).await;
    node.initialize().await;
    node
}

// ===========================================================================
// BOARD TESTS
// ===========================================================================

#[tokio::test]
async fn test_create_kanban_board() {
    let node = start_node("alice").await;

    // Create a project entity for the board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Kanban Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    // Create a kanban board
    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Sprint Board",
                "description": "Main development board"
            }),
        )
        .await;

    r.assert_success().assert_non_empty("id");
}

#[tokio::test]
async fn test_get_kanban_board() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Board Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Test Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Get the board
    let r = node
        .call_tool("get_kanban_board", json!({"board_id": board_id}))
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_update_kanban_board() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Update Board Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Original Name"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Update the board
    let r = node
        .call_tool(
            "update_kanban_board",
            json!({
                "board_id": board_id,
                "name": "Updated Board Name"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_list_kanban_boards() {
    let node = start_node("alice").await;

    // Create project
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "List Boards Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    // Create multiple boards
    node.call_tool(
        "create_kanban_board",
        json!({
            "entity_id": entity_id,
            "board_name": "Board 1"
        }),
    )
    .await
    .assert_success();

    node.call_tool(
        "create_kanban_board",
        json!({
            "entity_id": entity_id,
            "board_name": "Board 2"
        }),
    )
    .await
    .assert_success();

    // List boards and verify content
    let r = node
        .call_tool("list_kanban_boards", json!({"entity_id": entity_id}))
        .await;

    r.assert_success();

    // Verify that created boards are in the list
    if let Some(parsed) = &r.parsed
        && let Some(boards) = parsed.get("boards").and_then(|v| v.as_array())
    {
        assert!(!boards.is_empty(), "List should contain the created boards");
        assert!(
            boards.len() >= 2,
            "List should contain at least 2 created boards"
        );
    }
}

#[tokio::test]
async fn test_delete_kanban_board() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Delete Board Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Board to Delete"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Delete the board
    let r = node
        .call_tool("delete_kanban_board", json!({"board_id": board_id}))
        .await;

    r.assert_success();
}

// ===========================================================================
// COLUMN TESTS
// ===========================================================================

#[tokio::test]
async fn test_create_kanban_column() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Column Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Column Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Create a column
    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;

    r.assert_success().assert_non_empty("id");
}

#[tokio::test]
async fn test_get_kanban_column() {
    let node = start_node("alice").await;

    // Create project, board, and column
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Get Column Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Get Column Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "Test Column",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    // Get the column
    let r = node
        .call_tool(
            "get_kanban_column",
            json!({
                "board_id": board_id,
                "column_id": column_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_update_kanban_column() {
    let node = start_node("alice").await;

    // Create project, board, and column
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Update Column Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Update Column Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "Original Name",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    // Update the column
    let r = node
        .call_tool(
            "update_kanban_column",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "name": "Updated Column Name"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_delete_kanban_column() {
    let node = start_node("alice").await;

    // Create project, board, and column
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Delete Column Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Delete Column Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "Column to Delete",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    // Delete the column
    let r = node
        .call_tool(
            "delete_kanban_column",
            json!({
                "board_id": board_id,
                "column_id": column_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_list_kanban_columns() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "List Columns Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "List Columns Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Create multiple columns
    node.call_tool(
        "create_kanban_column",
        json!({
            "board_id": board_id,
            "column_name": "To Do",
            "position": 0
        }),
    )
    .await
    .assert_success();

    node.call_tool(
        "create_kanban_column",
        json!({
            "board_id": board_id,
            "column_name": "In Progress",
            "position": 1
        }),
    )
    .await
    .assert_success();

    // List columns and verify content
    let r = node
        .call_tool("list_kanban_columns", json!({"board_id": board_id}))
        .await;

    r.assert_success();

    // Verify that created columns are in the list
    if let Some(parsed) = &r.parsed
        && let Some(columns) = parsed.get("columns").and_then(|v| v.as_array())
    {
        assert!(
            !columns.is_empty(),
            "List should contain the created columns"
        );
        assert!(
            columns.len() >= 2,
            "List should contain at least 2 created columns"
        );
    }
}

#[tokio::test]
async fn test_move_kanban_column() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Move Column Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Move Column Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Create columns
    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "Column 1",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let col1 = r.get_str("id").unwrap();

    node.call_tool(
        "create_kanban_column",
        json!({
            "board_id": board_id,
            "column_name": "Column 2",
            "position": 1
        }),
    )
    .await
    .assert_success();

    // Move column
    let r = node
        .call_tool(
            "move_kanban_column",
            json!({
                "board_id": board_id,
                "column_id": col1,
                "new_position": 1
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// CARD TESTS
// ===========================================================================

#[tokio::test]
async fn test_create_kanban_card() {
    let node = start_node("alice").await;

    // Create project, board, and column
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    // Create a card
    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "Implement feature",
                "description": "Add new feature to app"
            }),
        )
        .await;

    r.assert_success().assert_non_empty("id");
}

#[tokio::test]
async fn test_get_kanban_card() {
    let node = start_node("alice").await;

    // Create project, board, column, and card
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Get Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Get Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "Test Card"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    // Get the card
    let r = node
        .call_tool(
            "get_kanban_card",
            json!({
                "board_id": board_id,
                "card_id": card_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_update_kanban_card() {
    let node = start_node("alice").await;

    // Create project, board, column, and card
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Update Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Update Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "Original Title"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    // Update the card
    let r = node
        .call_tool(
            "update_kanban_card",
            json!({
                "board_id": board_id,
                "card_id": card_id,
                "title": "Updated Title",
                "description": "Updated description"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_delete_kanban_card() {
    let node = start_node("alice").await;

    // Create project, board, column, and card
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Delete Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Delete Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "Card to Delete"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    // Delete the card
    let r = node
        .call_tool(
            "delete_kanban_card",
            json!({
                "board_id": board_id,
                "card_id": card_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_list_kanban_cards() {
    let node = start_node("alice").await;

    // Create project, board, and column
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "List Cards Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "List Cards Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    // Create multiple cards
    node.call_tool(
        "create_kanban_card",
        json!({
            "board_id": board_id,
            "column_id": column_id,
            "title": "Card 1"
        }),
    )
    .await
    .assert_success();

    node.call_tool(
        "create_kanban_card",
        json!({
            "board_id": board_id,
            "column_id": column_id,
            "title": "Card 2"
        }),
    )
    .await
    .assert_success();

    // List cards and verify content
    let r = node
        .call_tool("list_kanban_cards", json!({"board_id": board_id}))
        .await;

    r.assert_success();

    // Verify that created cards are in the list
    if let Some(parsed) = &r.parsed
        && let Some(cards) = parsed.get("cards").and_then(|v| v.as_array())
    {
        assert!(!cards.is_empty(), "List should contain the created cards");
        assert!(
            cards.len() >= 2,
            "List should contain at least 2 created cards"
        );
    }
}

#[tokio::test]
async fn test_move_kanban_card() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Move Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Move Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Create columns
    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let col1 = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "Done",
                "position": 1
            }),
        )
        .await;
    r.assert_success();
    let col2 = r.get_str("id").unwrap();

    // Create card in first column
    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": col1,
                "title": "Move Me"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    // Move card to second column
    let r = node
        .call_tool(
            "move_kanban_card",
            json!({
                "board_id": board_id,
                "card_id": card_id,
                "target_column_id": col2,
                "new_position": 0
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// CARD STATE TESTS
// ===========================================================================

#[tokio::test]
async fn test_change_card_state() {
    let node = start_node("alice").await;

    // Create project, board, column, and card
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Card State Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Card State Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "State Test Card"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    // Change card state
    let r = node
        .call_tool(
            "change_card_state",
            json!({
                "board_id": board_id,
                "card_id": card_id,
                "state": "Closed"
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// TAG TESTS
// ===========================================================================

#[tokio::test]
async fn test_create_kanban_tag() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Tag Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Tag Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Create a tag
    let r = node
        .call_tool(
            "create_kanban_tag",
            json!({
                "board_id": board_id,
                "name": "Bug",
                "color": "#ff0000"
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_list_kanban_tags() {
    let node = start_node("alice").await;

    // Create project and board
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "List Tags Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "List Tags Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // Create tags
    node.call_tool(
        "create_kanban_tag",
        json!({
            "board_id": board_id,
            "name": "Bug",
            "color": "#ff0000"
        }),
    )
    .await
    .assert_success();

    node.call_tool(
        "create_kanban_tag",
        json!({
            "board_id": board_id,
            "name": "Feature",
            "color": "#00ff00"
        }),
    )
    .await
    .assert_success();

    // List tags and verify content
    let r = node
        .call_tool("list_kanban_tags", json!({"board_id": board_id}))
        .await;

    r.assert_success();

    // Verify that created tags are in the list
    if let Some(parsed) = &r.parsed
        && let Some(tags) = parsed.get("tags").and_then(|v| v.as_array())
    {
        assert!(!tags.is_empty(), "List should contain the created tags");
        assert!(
            tags.len() >= 2,
            "List should contain at least 2 created tags"
        );
    }
}

#[tokio::test]
async fn test_tag_card() {
    let node = start_node("alice").await;

    // Create project, board, column, and card
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Tag Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Tag Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "Tag Me"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    // Create a tag
    let r = node
        .call_tool(
            "create_kanban_tag",
            json!({
                "board_id": board_id,
                "name": "Priority",
                "color": "#ff0000"
            }),
        )
        .await;
    r.assert_success();
    let tag_id = r.get_str("id").unwrap();

    // Tag the card
    let r = node
        .call_tool(
            "tag_card",
            json!({
                "board_id": board_id,
                "card_id": card_id,
                "tag_id": tag_id
            }),
        )
        .await;

    r.assert_success();
}

#[tokio::test]
async fn test_untag_card() {
    let node = start_node("alice").await;

    // Create project, board, column, card, and tag
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Untag Card Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Untag Card Board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let column_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": column_id,
                "title": "Untag Me"
            }),
        )
        .await;
    r.assert_success();
    let card_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_tag",
            json!({
                "board_id": board_id,
                "name": "Remove",
                "color": "#ff0000"
            }),
        )
        .await;
    r.assert_success();
    let tag_id = r.get_str("id").unwrap();

    // Tag then untag
    node.call_tool(
        "tag_card",
        json!({
            "board_id": board_id,
            "card_id": card_id,
            "tag_id": tag_id
        }),
    )
    .await
    .assert_success();

    // Untag the card
    let r = node
        .call_tool(
            "untag_card",
            json!({
                "board_id": board_id,
                "card_id": card_id,
                "tag_id": tag_id
            }),
        )
        .await;

    r.assert_success();
}

// ===========================================================================
// INTEGRATION TEST - FULL KANBAN WORKFLOW
// ===========================================================================

#[tokio::test]
async fn test_full_kanban_workflow() {
    let node = start_node("alice").await;

    // 1. Create project entity
    let r = node
        .call_tool(
            "create_entity",
            json!({"name": "Sprint Project", "entity_type": "project"}),
        )
        .await;
    r.assert_success();
    let entity_id = r.get_str("id").unwrap();

    // 2. Create kanban board
    let r = node
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": entity_id,
                "board_name": "Sprint Board",
                "description": "Main sprint board"
            }),
        )
        .await;
    r.assert_success();
    let board_id = r.get_str("id").unwrap();

    // 3. Create columns
    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do",
                "position": 0
            }),
        )
        .await;
    r.assert_success();
    let todo_col = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "In Progress",
                "position": 1
            }),
        )
        .await;
    r.assert_success();
    let progress_col = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "Done",
                "position": 2
            }),
        )
        .await;
    r.assert_success();
    let done_col = r.get_str("id").unwrap();

    // 4. Create tags
    let r = node
        .call_tool(
            "create_kanban_tag",
            json!({
                "board_id": board_id,
                "name": "Bug",
                "color": "#ff0000"
            }),
        )
        .await;
    r.assert_success();
    let bug_tag = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_tag",
            json!({
                "board_id": board_id,
                "name": "Feature",
                "color": "#00ff00"
            }),
        )
        .await;
    r.assert_success();
    let feature_tag = r.get_str("id").unwrap();

    // 5. Create cards
    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": todo_col,
                "title": "Fix login bug",
                "description": "Users can't login with special characters"
            }),
        )
        .await;
    r.assert_success();
    let card1_id = r.get_str("id").unwrap();

    let r = node
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": todo_col,
                "title": "Add dark mode",
                "description": "Implement dark mode theme"
            }),
        )
        .await;
    r.assert_success();
    let card2_id = r.get_str("id").unwrap();

    // 6. Tag cards
    let r6a = node.call_tool(
        "tag_card",
        json!({
            "board_id": board_id,
            "card_id": card1_id,
            "tag_id": bug_tag
        }),
    )
    .await;
    r6a.assert_success();

    let r6b = node.call_tool(
        "tag_card",
        json!({
            "board_id": board_id,
            "card_id": card2_id,
            "tag_id": feature_tag
        }),
    )
    .await;
    r6b.assert_success();

    // 7. Move card through workflow
    let r7 = node.call_tool(
        "move_kanban_card",
        json!({
            "board_id": board_id,
            "card_id": card1_id,
            "target_column_id": progress_col,
            "new_position": 0
        }),
    )
    .await;
    r7.assert_success();

    // 8. Update card
    let r8 = node.call_tool(
        "update_kanban_card",
        json!({
            "board_id": board_id,
            "card_id": card1_id,
            "title": "Fix login bug - in progress",
            "description": "Working on special character handling"
        }),
    )
    .await;
    r8.assert_success();

    // 9. Change card state (Open -> Closed)
    let r9 = node.call_tool(
        "change_card_state",
        json!({
            "board_id": board_id,
            "card_id": card1_id,
            "state": "Closed"
        }),
    )
    .await;
    r9.assert_success();

    // 10. Complete card - move to done column and archive
    let r10a = node.call_tool(
        "move_kanban_card",
        json!({
            "board_id": board_id,
            "card_id": card1_id,
            "target_column_id": done_col,
            "new_position": 0
        }),
    )
    .await;
    r10a.assert_success();

    let r10b = node.call_tool(
        "change_card_state",
        json!({
            "board_id": board_id,
            "card_id": card1_id,
            "state": "Archived"
        }),
    )
    .await;
    r10b.assert_success();

    // Verify workflow completed
    println!("Full kanban workflow test passed!");
}

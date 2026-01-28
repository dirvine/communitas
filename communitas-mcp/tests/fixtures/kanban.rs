// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Kanban test fixtures

#![allow(dead_code)]

use serde_json::{Value, json};

/// Generate a board fixture
pub fn board_fixture(name: &str) -> Value {
    json!({
        "name": name,
        "description": format!("Test board: {}", name)
    })
}

/// Generate a column fixture
pub fn column_fixture(board_id: &str, name: &str, position: u32) -> Value {
    json!({
        "board_id": board_id,
        "name": name,
        "position": position
    })
}

/// Generate a card fixture
pub fn card_fixture(column_id: &str, title: &str) -> Value {
    json!({
        "column_id": column_id,
        "title": title,
        "description": format!("Description for: {}", title)
    })
}

/// Standard board with columns
pub fn standard_board_setup() -> Value {
    json!({
        "board": {
            "name": "Development Board",
            "description": "Main development workflow"
        },
        "columns": [
            {"name": "Backlog", "position": 0},
            {"name": "To Do", "position": 1},
            {"name": "In Progress", "position": 2},
            {"name": "Review", "position": 3},
            {"name": "Done", "position": 4}
        ]
    })
}

/// Kanban tag fixture
pub fn tag_fixture(board_id: &str, name: &str, color: &str) -> Value {
    json!({
        "board_id": board_id,
        "name": name,
        "color": color
    })
}

/// Common test tags
pub fn common_tags(board_id: &str) -> Vec<Value> {
    vec![
        tag_fixture(board_id, "bug", "#e74c3c"),
        tag_fixture(board_id, "feature", "#27ae60"),
        tag_fixture(board_id, "enhancement", "#3498db"),
        tag_fixture(board_id, "documentation", "#9b59b6"),
        tag_fixture(board_id, "urgent", "#f39c12"),
    ]
}

/// Card with due date
pub fn card_with_due_date(column_id: &str, title: &str, days_from_now: i64) -> Value {
    let due_date = chrono::Utc::now() + chrono::Duration::days(days_from_now);
    json!({
        "column_id": column_id,
        "title": title,
        "description": "Card with due date",
        "due_date": due_date.format("%Y-%m-%d").to_string()
    })
}

/// Card move fixture
pub fn card_move_fixture(card_id: &str, target_column_id: &str, position: u32) -> Value {
    json!({
        "card_id": card_id,
        "target_column_id": target_column_id,
        "position": position
    })
}

/// Comment fixture
pub fn comment_fixture(card_id: &str, content: &str) -> Value {
    json!({
        "card_id": card_id,
        "content": content
    })
}

/// Checklist step fixture
pub fn step_fixture(card_id: &str, description: &str) -> Value {
    json!({
        "card_id": card_id,
        "description": description,
        "completed": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_fixture() {
        let board = board_fixture("My Board");
        assert_eq!(board["name"], "My Board");
    }

    #[test]
    fn test_standard_board_setup() {
        let setup = standard_board_setup();
        let columns = setup["columns"].as_array().unwrap();
        assert_eq!(columns.len(), 5);
    }

    #[test]
    fn test_common_tags() {
        let tags = common_tags("board-123");
        assert_eq!(tags.len(), 5);
        assert!(tags.iter().any(|t| t["name"] == "bug"));
    }
}

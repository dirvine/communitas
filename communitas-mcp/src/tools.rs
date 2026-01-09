// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Tool definitions
//!
//! Exposes Communitas commands and queries as MCP tools that AI agents can invoke.

use crate::presence::{PresenceOperations, PresenceStatus, PresenceSubscription, PresenceUpdate};
use crate::presentation::{PresentationOperations, ScreenRegion, Slide};
use crate::protocol::{Tool, ToolCallResult, ToolContent};
use crate::social::{LocationOperations, PollOperations, StoryOperations};
use crate::webrtc::{CallRequest, WebRtcOperations};
use base64::prelude::*;
use communitas_core::{
    app::CommunitasApp,
    command::{Command, DiskTypeArg, Event, Query, QueryResponse},
    crdt::EntityType,
};
use serde_json::{Value, json};

/// Get list of all available tools
/// If authenticated is false, only pre-auth tools are returned
pub fn list_tools(authenticated: bool) -> Vec<Tool> {
    let mut tools = vec![
        // Pre-auth tools (always available)
        Tool {
            name: "authenticate".to_string(),
            description: "Authenticate with four-word identity and password".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "four_words": {"type": "string", "description": "Your four-word identity (word1.word2.word3.word4)"},
                    "password": {"type": "string", "description": "Your password"},
                    "device_name": {"type": "string", "description": "Name for this device/session"}
                },
                "required": ["four_words", "password"]
            }),
        },
        Tool {
            name: "create_vault".to_string(),
            description: "Create a new identity vault with four-word address".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "four_words": {"type": "string", "description": "Choose your four-word identity (word1.word2.word3.word4)"},
                    "password": {"type": "string", "description": "Password to protect your vault"},
                    "display_name": {"type": "string", "description": "Your display name"}
                },
                "required": ["four_words", "password", "display_name"]
            }),
        },
        Tool {
            name: "authenticate_token".to_string(),
            description: "Authenticate with a delegate token (for AI agents)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "token": {"type": "string", "description": "The delegate token"}
                },
                "required": ["token"]
            }),
        },
        // Health check - always available
        Tool {
            name: "health_check".to_string(),
            description: "Check if the MCP service is healthy and responsive".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        // Core status - always available
        Tool {
            name: "core_status".to_string(),
            description: "Check if the core context is initialized and ready".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "list_vaults".to_string(),
            description: "List all available identity vaults on this device".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "delete_vault".to_string(),
            description: "Delete an identity vault (requires password confirmation)".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "four_words": {"type": "string", "description": "Four-word identity of the vault to delete"},
                    "password": {"type": "string", "description": "Password to confirm deletion"}
                },
                "required": ["four_words", "password"]
            }),
        },
        Tool {
            name: "import_vault".to_string(),
            description: "Import an identity vault from a backup".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "backup_data": {"type": "string", "description": "Base64-encoded backup data"},
                    "password": {"type": "string", "description": "Password to decrypt the backup"}
                },
                "required": ["backup_data", "password"]
            }),
        },
    ];

    // If not authenticated, only return pre-auth tools
    if !authenticated {
        return tools;
    }

    // Add all authenticated tools
    tools.extend(vec![
        // Session tools
        Tool {
            name: "get_session".to_string(),
            description: "Get current session info".to_string(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        Tool {
            name: "logout".to_string(),
            description: "End current session".to_string(),
            input_schema: json!({ "type": "object", "properties": {} }),
        },
        Tool {
            name: "get_unread_count".to_string(),
            description: "Get unread messages count".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_ids": {"type": "array", "items": {"type": "string"}, "description": "Entity IDs to check"}
                },
                "required": ["entity_ids"]
            }),
        },
        // Entity tools
        Tool {
            name: "create_entity".to_string(),
            description: "Create a new entity (organisation, project, group, or channel)"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Entity name"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"], "description": "Type of entity"},
                    "description": {"type": "string", "description": "Entity description"},
                    "initial_members": {"type": "array", "items": {"type": "string"}, "description": "Initial member four-word IDs"}
                },
                "required": ["name", "entity_type"]
            }),
        },
        Tool {
            name: "update_entity".to_string(),
            description: "Update an entity's name or description".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"], "description": "Type of entity"},
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "name": {"type": "string", "description": "New entity name (optional)"},
                    "description": {"type": ["string", "null"], "description": "New description (optional, null to clear)"}
                },
                "required": ["entity_type", "entity_id"]
            }),
        },
        Tool {
            name: "delete_entity".to_string(),
            description: "Delete an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"], "description": "Type of entity"},
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_type", "entity_id"]
            }),
        },
        // Member tools
        Tool {
            name: "add_member".to_string(),
            description: "Add a member to an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"]},
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "member_id": {"type": "string", "description": "Member four-word ID"},
                    "role": {"type": "string", "description": "Member role"}
                },
                "required": ["entity_type", "entity_id", "member_id", "role"]
            }),
        },
        Tool {
            name: "remove_member".to_string(),
            description: "Remove a member from an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"]},
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "member_id": {"type": "string", "description": "Member four-word ID"}
                },
                "required": ["entity_type", "entity_id", "member_id"]
            }),
        },
        // Message tools
        Tool {
            name: "send_message".to_string(),
            description: "Send a message to an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID to send to"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel", "person"]},
                    "text": {"type": "string", "description": "Message text"},
                    "reply_to_id": {"type": "string", "description": "Message ID to reply to (optional)"}
                },
                "required": ["entity_id", "entity_type", "text"]
            }),
        },
        Tool {
            name: "delete_message".to_string(),
            description: "Delete a message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID containing the message"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel", "person"]},
                    "message_id": {"type": "string", "description": "Message ID to delete"}
                },
                "required": ["entity_id", "entity_type", "message_id"]
            }),
        },
        Tool {
            name: "edit_message".to_string(),
            description: "Edit an existing message's text content".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID containing the message"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel", "person"]},
                    "message_id": {"type": "string", "description": "Message ID to edit"},
                    "new_text": {"type": "string", "description": "New message text"}
                },
                "required": ["entity_id", "entity_type", "message_id", "new_text"]
            }),
        },
        Tool {
            name: "add_reaction".to_string(),
            description: "Add an emoji reaction to a message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID containing the message"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel", "person"]},
                    "message_id": {"type": "string", "description": "Message ID to react to"},
                    "emoji": {"type": "string", "description": "Emoji to add (e.g., '👍', '❤️', '😀')"}
                },
                "required": ["entity_id", "entity_type", "message_id", "emoji"]
            }),
        },
        Tool {
            name: "remove_reaction".to_string(),
            description: "Remove an emoji reaction from a message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID containing the message"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel", "person"]},
                    "message_id": {"type": "string", "description": "Message ID to remove reaction from"},
                    "emoji": {"type": "string", "description": "Emoji to remove"}
                },
                "required": ["entity_id", "entity_type", "message_id", "emoji"]
            }),
        },
        Tool {
            name: "get_reactions".to_string(),
            description: "Get all reactions on a message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID containing the message"},
                    "message_id": {"type": "string", "description": "Message ID to get reactions for"}
                },
                "required": ["entity_id", "message_id"]
            }),
        },
        Tool {
            name: "create_custom_reaction".to_string(),
            description: "Create a custom emoji reaction".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID to add the reaction to"},
                    "emoji": {"type": "string", "description": "Emoji character or URL"},
                    "short_name": {"type": "string", "description": "Short name for the reaction (e.g. 'rocket')"}
                },
                "required": ["entity_id", "emoji", "short_name"]
            }),
        },
        Tool {
            name: "get_available_reactions".to_string(),
            description: "Get list of available reactions (standard + custom) for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_id"]
            }),
        },
        // Kanban tools
        Tool {
            name: "create_kanban_board".to_string(),
            description: "Create a new Kanban board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "board_name": {"type": "string", "description": "Board name"},
                    "description": {"type": "string", "description": "Board description (optional)"}
                },
                "required": ["entity_id", "board_name"]
            }),
        },
        Tool {
            name: "create_kanban_column".to_string(),
            description: "Create a column in a Kanban board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_name": {"type": "string", "description": "Column name"},
                    "position": {"type": "integer", "description": "Column position (optional)"}
                },
                "required": ["board_id", "column_name"]
            }),
        },
        Tool {
            name: "create_kanban_card".to_string(),
            description: "Create a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_id": {"type": "string", "description": "Column ID"},
                    "title": {"type": "string", "description": "Card title"},
                    "description": {"type": "string", "description": "Card description (optional)"},
                    "assignee": {"type": "string", "description": "Assignee four-word ID (optional)"}
                },
                "required": ["board_id", "column_id", "title"]
            }),
        },
        Tool {
            name: "move_kanban_card".to_string(),
            description: "Move a Kanban card to a different column".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "target_column_id": {"type": "string", "description": "Target column ID"},
                    "position": {"type": "integer", "description": "Position in target column (optional)"}
                },
                "required": ["board_id", "card_id", "target_column_id"]
            }),
        },
        Tool {
            name: "update_kanban_card".to_string(),
            description: "Update a Kanban card's title, description, or assignee".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID to update"},
                    "title": {"type": "string", "description": "New title (optional)"},
                    "description": {"type": "string", "description": "New description (optional)"},
                    "assignee": {"type": "string", "description": "New assignee ID (optional)"}
                },
                "required": ["board_id", "card_id"]
            }),
        },
        Tool {
            name: "delete_kanban_card".to_string(),
            description: "Delete a Kanban card from a board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID to delete"}
                },
                "required": ["board_id", "card_id"]
            }),
        },
        Tool {
            name: "list_kanban_boards".to_string(),
            description: "List all Kanban boards for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID to list boards for"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "get_kanban_card".to_string(),
            description: "Get details of a specific Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"}
                },
                "required": ["board_id", "card_id"]
            }),
        },
        Tool {
            name: "get_kanban_board".to_string(),
            description: "Get details of a specific Kanban board including column count".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"}
                },
                "required": ["board_id"]
            }),
        },
        Tool {
            name: "update_kanban_board".to_string(),
            description: "Update a Kanban board's name or description".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID to update"},
                    "name": {"type": "string", "description": "New board name (optional)"},
                    "description": {"type": "string", "description": "New board description (optional)"}
                },
                "required": ["board_id"]
            }),
        },
        Tool {
            name: "delete_kanban_board".to_string(),
            description: "Delete a Kanban board and all its columns and cards".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID to delete"}
                },
                "required": ["board_id"]
            }),
        },
        Tool {
            name: "list_kanban_cards".to_string(),
            description: "List all cards in a Kanban board, optionally filtered by column, state, assignee, or tag".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_id": {"type": "string", "description": "Filter by column ID (optional)"},
                    "state": {"type": "string", "enum": ["Open", "Closed", "Postponed", "Archived"], "description": "Filter by card state (optional)"},
                    "assignee_id": {"type": "string", "description": "Filter by assignee ID (optional)"},
                    "tag_id": {"type": "string", "description": "Filter by tag ID (optional)"}
                },
                "required": ["board_id"]
            }),
        },
        // Kanban column tools
        Tool {
            name: "list_kanban_columns".to_string(),
            description: "List all columns in a Kanban board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"}
                },
                "required": ["board_id"]
            }),
        },
        Tool {
            name: "get_kanban_column".to_string(),
            description: "Get details of a specific column".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_id": {"type": "string", "description": "Column ID"}
                },
                "required": ["board_id", "column_id"]
            }),
        },
        Tool {
            name: "update_kanban_column".to_string(),
            description: "Update a column's name, color, or WIP limit".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_id": {"type": "string", "description": "Column ID"},
                    "name": {"type": "string", "description": "New column name (optional)"},
                    "color": {"type": "string", "description": "New color (optional)"},
                    "wip_limit": {"type": "integer", "description": "New WIP limit (optional)"}
                },
                "required": ["board_id", "column_id"]
            }),
        },
        Tool {
            name: "delete_kanban_column".to_string(),
            description: "Delete a column from a board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_id": {"type": "string", "description": "Column ID to delete"}
                },
                "required": ["board_id", "column_id"]
            }),
        },
        Tool {
            name: "move_kanban_column".to_string(),
            description: "Move a column to a new position".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "column_id": {"type": "string", "description": "Column ID"},
                    "new_position": {"type": "integer", "description": "New position (0-based index)"}
                },
                "required": ["board_id", "column_id", "new_position"]
            }),
        },
        // Kanban card state tool
        Tool {
            name: "change_card_state".to_string(),
            description: "Change a card's state (Open, Closed, Postponed, Archived)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "state": {"type": "string", "enum": ["Open", "Closed", "Postponed", "Archived"], "description": "New state"}
                },
                "required": ["board_id", "card_id", "state"]
            }),
        },
        // Kanban assignment tools
        Tool {
            name: "assign_user".to_string(),
            description: "Assign a user to a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "user_id": {"type": "string", "description": "User four-word ID to assign"}
                },
                "required": ["board_id", "card_id", "user_id"]
            }),
        },
        Tool {
            name: "unassign_user".to_string(),
            description: "Remove a user assignment from a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "user_id": {"type": "string", "description": "User four-word ID to unassign"}
                },
                "required": ["board_id", "card_id", "user_id"]
            }),
        },
        // Kanban tag tools
        Tool {
            name: "create_kanban_tag".to_string(),
            description: "Create a new tag for a Kanban board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "name": {"type": "string", "description": "Tag name"},
                    "color": {"type": "string", "description": "Tag color (e.g., #FF0000)"}
                },
                "required": ["board_id", "name", "color"]
            }),
        },
        Tool {
            name: "list_kanban_tags".to_string(),
            description: "List all tags in a Kanban board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"}
                },
                "required": ["board_id"]
            }),
        },
        Tool {
            name: "tag_card".to_string(),
            description: "Add a tag to a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "tag_id": {"type": "string", "description": "Tag ID to add"}
                },
                "required": ["board_id", "card_id", "tag_id"]
            }),
        },
        Tool {
            name: "untag_card".to_string(),
            description: "Remove a tag from a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "tag_id": {"type": "string", "description": "Tag ID to remove"}
                },
                "required": ["board_id", "card_id", "tag_id"]
            }),
        },
        // Kanban step/checklist tools
        Tool {
            name: "add_step".to_string(),
            description: "Add a checklist step to a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "text": {"type": "string", "description": "Step text"},
                    "position": {"type": "integer", "description": "Position (optional)"}
                },
                "required": ["board_id", "card_id", "text"]
            }),
        },
        Tool {
            name: "get_step".to_string(),
            description: "Get details of a checklist step".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "step_id": {"type": "string", "description": "Step ID"}
                },
                "required": ["board_id", "card_id", "step_id"]
            }),
        },
        Tool {
            name: "toggle_step".to_string(),
            description: "Toggle a checklist step's completion status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "step_id": {"type": "string", "description": "Step ID"}
                },
                "required": ["board_id", "card_id", "step_id"]
            }),
        },
        Tool {
            name: "delete_step".to_string(),
            description: "Delete a checklist step from a card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "step_id": {"type": "string", "description": "Step ID to delete"}
                },
                "required": ["board_id", "card_id", "step_id"]
            }),
        },
        // Kanban comment tools
        Tool {
            name: "add_comment".to_string(),
            description: "Add a comment to a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "content": {"type": "string", "description": "Comment content"},
                    "parent_id": {"type": "string", "description": "Parent comment ID for replies (optional)"}
                },
                "required": ["board_id", "card_id", "content"]
            }),
        },
        Tool {
            name: "list_comments".to_string(),
            description: "List all comments on a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"}
                },
                "required": ["board_id", "card_id"]
            }),
        },
        Tool {
            name: "delete_comment".to_string(),
            description: "Delete a comment from a Kanban card".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"},
                    "card_id": {"type": "string", "description": "Card ID"},
                    "comment_id": {"type": "string", "description": "Comment ID to delete"}
                },
                "required": ["board_id", "card_id", "comment_id"]
            }),
        },
        // Entity join tool
        Tool {
            name: "join_entity".to_string(),
            description: "Join an existing entity from another node (for multi-node sync)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Entity ID to join"},
                    "name": {"type": "string", "description": "Entity name"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"], "description": "Entity type"},
                    "description": {"type": "string", "description": "Entity description"},
                    "created_by": {"type": "string", "description": "Original creator's four-word ID"},
                    "role": {"type": "string", "description": "Your role in this entity", "default": "member"}
                },
                "required": ["id", "name", "entity_type", "created_by"]
            }),
        },
        // File tools
        Tool {
            name: "delete_file".to_string(),
            description: "Delete a file from an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path to delete"}
                },
                "required": ["entity_id", "disk_type", "path"]
            }),
        },
        Tool {
            name: "get_disk_stats".to_string(),
            description: "Get disk usage statistics for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"}
                },
                "required": ["entity_id", "disk_type"]
            }),
        },
        // Thread tools
        Tool {
            name: "create_thread".to_string(),
            description: "Create a threaded discussion from a message".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {"type": "string", "description": "Channel ID containing the message"},
                    "parent_message_id": {"type": "string", "description": "Message ID to create thread from"}
                },
                "required": ["channel_id", "parent_message_id"]
            }),
        },
        Tool {
            name: "get_thread_messages".to_string(),
            description: "Get messages in a thread".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "channel_id": {"type": "string", "description": "Channel ID containing the thread"},
                    "thread_id": {"type": "string", "description": "Thread ID (parent message ID)"}
                },
                "required": ["channel_id", "thread_id"]
            }),
        },
        // Invite tools
        Tool {
            name: "create_invite".to_string(),
            description: "Create an invitation to join an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "recipient_id": {"type": "string", "description": "Recipient four-word ID"},
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"]},
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "role": {"type": "string", "description": "Role to grant"},
                    "message": {"type": "string", "description": "Optional message"}
                },
                "required": ["recipient_id", "entity_type", "entity_id", "role"]
            }),
        },
        Tool {
            name: "accept_invite".to_string(),
            description: "Accept an invitation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "invite_id": {"type": "string", "description": "Invite ID to accept"}
                },
                "required": ["invite_id"]
            }),
        },
        // File tools
        Tool {
            name: "write_file".to_string(),
            description: "Write a file to an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path"},
                    "content": {"type": "string", "description": "File content (base64 for binary)"}
                },
                "required": ["entity_id", "disk_type", "path", "content"]
            }),
        },
        Tool {
            name: "read_file".to_string(),
            description: "Read a file from an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path"}
                },
                "required": ["entity_id", "disk_type", "path"]
            }),
        },
        // Query tools (read operations)
        Tool {
            name: "get_entity".to_string(),
            description: "Get details of an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "list_entities".to_string(),
            description: "List all entities or filter by type".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"], "description": "Entity type filter (optional)"}
                }
            }),
        },
        Tool {
            name: "list_members".to_string(),
            description: "List members of an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_type": {"type": "string", "enum": ["organisation", "project", "group", "channel"]},
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_type", "entity_id"]
            }),
        },
        Tool {
            name: "get_messages".to_string(),
            description: "Get messages for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "list_files".to_string(),
            description: "List files in an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "Directory path (optional)", "default": "/"}
                },
                "required": ["entity_id", "disk_type"]
            }),
        },
        Tool {
            name: "get_profile".to_string(),
            description: "Get the current user's profile".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "update_profile".to_string(),
            description: "Update the current user's display name".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "display_name": {"type": "string", "description": "New display name"}
                },
                "required": ["display_name"]
            }),
        },
        Tool {
            name: "export_vault".to_string(),
            description: "Export the current vault for backup".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "include_data": {"type": "boolean", "description": "Include all data in export (default: true)", "default": true}
                }
            }),
        },
        Tool {
            name: "list_pending_invites".to_string(),
            description: "List pending invitations for the current user".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        // WebRTC Calling tools
        Tool {
            name: "start_voice_call".to_string(),
            description: "Start a voice/video call with an entity or users".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID (channel/group) to call"},
                    "participant_ids": {"type": "array", "items": {"type": "string"}, "description": "Specific users to invite (optional)"},
                    "video_enabled": {"type": "boolean", "description": "Enable video (default: true)", "default": true}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "join_call".to_string(),
            description: "Join an active call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "ID of the call to join"}
                },
                "required": ["call_id"]
            }),
        },
        Tool {
            name: "end_call".to_string(),
            description: "End or leave a call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "ID of the call to end"}
                },
                "required": ["call_id"]
            }),
        },
        // Media tools
        Tool {
            name: "upload_with_metadata".to_string(),
            description: "Upload a file with associated metadata (thumbnails, mime type)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path"},
                    "content": {"type": "string", "description": "Base64 encoded content"},
                    "metadata": {"type": "object", "description": "Metadata object"}
                },
                "required": ["entity_id", "disk_type", "path", "content"]
            }),
        },
        Tool {
            name: "get_media_metadata".to_string(),
            description: "Get metadata for a file".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path"}
                },
                "required": ["entity_id", "disk_type", "path"]
            }),
        },
        // Social tools (Polls, Location, Stories)
        Tool {
            name: "create_poll".to_string(),
            description: "Create a poll".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "question": {"type": "string", "description": "Poll question"},
                    "options": {"type": "array", "items": {"type": "string"}, "description": "Poll options"},
                    "allows_multiple": {"type": "boolean", "description": "Allow multiple choices", "default": false},
                    "duration_hours": {"type": "integer", "description": "Duration in hours"}
                },
                "required": ["entity_id", "question", "options"]
            }),
        },
        Tool {
            name: "vote_in_poll".to_string(),
            description: "Vote in a poll".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "poll_id": {"type": "string", "description": "Poll ID"},
                    "option_id": {"type": "string", "description": "Option ID to vote for"}
                },
                "required": ["poll_id", "option_id"]
            }),
        },
        Tool {
            name: "share_location".to_string(),
            description: "Share location".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "latitude": {"type": "number", "description": "Latitude"},
                    "longitude": {"type": "number", "description": "Longitude"},
                    "is_live": {"type": "boolean", "description": "Is live location", "default": false},
                    "duration_minutes": {"type": "integer", "description": "Duration for live location"}
                },
                "required": ["entity_id", "latitude", "longitude"]
            }),
        },
        Tool {
            name: "create_story".to_string(),
            description: "Create an ephemeral story".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "text": {"type": "string", "description": "Story text"},
                    "media_paths": {"type": "array", "items": {"type": "string"}, "description": "Paths to media files"},
                    "expires_hours": {"type": "integer", "description": "Expiration in hours", "default": 24}
                },
                "required": ["entity_id", "text"]
            }),
        },
        // Presentation tools
        Tool {
            name: "start_presentation".to_string(),
            description: "Start a presentation".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "slides": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "content": {"type": "string"}
                        },
                        "required": ["title", "content"]
                    }, "description": "List of slides"}
                },
                "required": ["entity_id", "slides"]
            }),
        },
        Tool {
            name: "share_screen".to_string(),
            description: "Share screen in a call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "Call ID"},
                    "region": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "integer"},
                            "y": {"type": "integer"},
                            "width": {"type": "integer"},
                            "height": {"type": "integer"}
                        },
                        "description": "Screen region to share (optional, defaults to full screen)"
                    }
                },
                "required": ["call_id"]
            }),
        },
        // Presence tools
        Tool {
            name: "set_presence".to_string(),
            description: "Set your current presence status".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {"type": "string", "enum": ["online", "away", "busy", "invisible"], "description": "Presence status"},
                    "entity_id": {"type": "string", "description": "Entity ID where this presence applies (optional)"}
                },
                "required": ["status"]
            }),
        },
        Tool {
            name: "get_presence".to_string(),
            description: "Get presence status for specific users".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "user_ids": {"type": "array", "items": {"type": "string"}, "description": "List of user IDs to check"}
                },
                "required": ["user_ids"]
            }),
        },
        Tool {
            name: "subscribe_to_presence".to_string(),
            description: "Subscribe to presence updates for specific entities".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_ids": {"type": "array", "items": {"type": "string"}, "description": "List of entity IDs to subscribe to"}
                },
                "required": ["entity_ids"]
            }),
        },
        // Network tools
        Tool {
            name: "network_start".to_string(),
            description: "Start P2P networking with the gossip overlay. Connects to bootstrap nodes and enables peer discovery.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "preferred_port": {"type": "integer", "description": "Preferred port to listen on (optional, uses dynamic port if not specified)"}
                }
            }),
        },
        Tool {
            name: "network_stop".to_string(),
            description: "Stop P2P networking gracefully. Disconnects from all peers.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "network_connect".to_string(),
            description: "Connect to a specific peer by their four-word identity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "peer_four_words": {"type": "string", "description": "Four-word identity of the peer (e.g., 'ocean-forest-moon-star')"}
                },
                "required": ["peer_four_words"]
            }),
        },
        Tool {
            name: "network_status".to_string(),
            description: "Get current network status including connection identity and whether networking is active".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "network_peers".to_string(),
            description: "List currently connected peers".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "network_request_external_address".to_string(),
            description: "Request discovery of external address via NAT reflection".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "network_disconnect".to_string(),
            description: "Disconnect from a specific peer by their four-word identity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "peer_four_words": {"type": "string", "description": "Four-word identity of the peer to disconnect from"}
                },
                "required": ["peer_four_words"]
            }),
        },
        // Contact tools
        Tool {
            name: "create_contact".to_string(),
            description: "Create a new contact in your address book".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "display_name": {"type": "string", "description": "Display name for the contact"},
                    "four_words": {"type": "string", "description": "Four-word identity of the contact (optional for local-only contacts)"},
                    "is_favourite": {"type": "boolean", "description": "Mark as favourite contact", "default": false}
                },
                "required": ["display_name"]
            }),
        },
        Tool {
            name: "update_contact".to_string(),
            description: "Update an existing contact".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "contact_id": {"type": "string", "description": "Contact ID to update"},
                    "display_name": {"type": "string", "description": "New display name (optional)"},
                    "is_favourite": {"type": "boolean", "description": "Update favourite status (optional)"}
                },
                "required": ["contact_id"]
            }),
        },
        Tool {
            name: "delete_contact".to_string(),
            description: "Delete a contact from your address book".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "contact_id": {"type": "string", "description": "Contact ID to delete"}
                },
                "required": ["contact_id"]
            }),
        },
        Tool {
            name: "link_contact".to_string(),
            description: "Link a local contact to a network identity (four-word address)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "contact_id": {"type": "string", "description": "Contact ID to link"},
                    "four_words": {"type": "string", "description": "Four-word identity to link to"}
                },
                "required": ["contact_id", "four_words"]
            }),
        },
        Tool {
            name: "set_favourite_contact".to_string(),
            description: "Mark a contact as favourite".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "four_words": {"type": "string", "description": "Four-word identity of the contact"}
                },
                "required": ["four_words"]
            }),
        },
        Tool {
            name: "remove_favourite_contact".to_string(),
            description: "Remove a contact from favourites".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "four_words": {"type": "string", "description": "Four-word identity of the contact"}
                },
                "required": ["four_words"]
            }),
        },
        Tool {
            name: "get_contact".to_string(),
            description: "Get details of a specific contact".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "contact_id": {"type": "string", "description": "Contact ID"}
                },
                "required": ["contact_id"]
            }),
        },
        Tool {
            name: "list_contacts".to_string(),
            description: "List all contacts in your address book".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "list_favourite_contacts".to_string(),
            description: "List all favourite contacts".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "search_contacts".to_string(),
            description: "Search contacts by name or four-word identity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"}
                },
                "required": ["query"]
            }),
        },
        // ========== Website Publishing Tools ==========
        Tool {
            name: "create_website".to_string(),
            description: "Create and publish a website bound to an entity. The website will be accessible via the entity's four-word identity.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID to bind the website to"},
                    "html": {"type": "string", "description": "HTML content for the website"},
                    "css": {"type": "string", "description": "CSS styles (optional)"},
                    "js": {"type": "string", "description": "JavaScript code (optional)"},
                    "metadata": {"type": "string", "description": "JSON metadata for the website (optional)"}
                },
                "required": ["entity_id", "html"]
            }),
        },
        Tool {
            name: "update_website".to_string(),
            description: "Update an existing website's content. Only fields provided will be updated.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID of the website to update"},
                    "html": {"type": "string", "description": "New HTML content (optional)"},
                    "css": {"type": "string", "description": "New CSS styles (optional)"},
                    "js": {"type": "string", "description": "New JavaScript code (optional)"},
                    "metadata": {"type": "string", "description": "New JSON metadata (optional)"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "delete_website".to_string(),
            description: "Delete a website and remove it from publishing".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID of the website to delete"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "get_website".to_string(),
            description: "Get website content and metadata for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID to get the website for"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "create_delegate_token".to_string(),
            description: "Create a delegate token for AI agents with scoped access".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "delegate_name": {"type": "string", "description": "Name for this delegate (e.g., 'my-claude-agent')"},
                    "scopes": {
                        "type": "array",
                        "items": {"type": "string", "enum": ["read_messages", "send_messages", "read_files", "write_files", "manage_entities", "manage_members", "manage_kanban", "manage_network", "manage_contacts", "full"]},
                        "description": "Access scopes for the token (default: full)"
                    },
                    "expires_in_hours": {"type": "integer", "description": "Token validity in hours (default: 24)"}
                },
                "required": ["delegate_name"]
            }),
        },
        Tool {
            name: "workspace_init".to_string(),
            description: "Initialize a new workspace with default structure: creates a project entity with a default Kanban board containing To Do, In Progress, and Done columns".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Workspace/project name"},
                    "description": {"type": "string", "description": "Workspace description (optional)"},
                    "board_name": {"type": "string", "description": "Name for the default Kanban board (default: 'Main Board')"},
                    "columns": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "Column names (default: ['To Do', 'In Progress', 'Done'])"
                    }
                },
                "required": ["name"]
            }),
        },
    ]);

    tools
}

/// Execute a tool call
pub async fn call_tool(app: &CommunitasApp, name: &str, args: Option<Value>) -> ToolCallResult {
    let args = args.unwrap_or(json!({}));

    // Try each category dispatcher in turn
    // Note: health_check and core_status are handled as pre-auth tools in server.rs

    if let Some(result) = dispatch_session_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_entity_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_message_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_kanban_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_file_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_contact_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_network_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_social_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_misc_tools(app, name, &args).await {
        return result;
    }

    error_result(&format!("Unknown tool: {}", name))
}

// ============================================================================
// Category Dispatchers
// ============================================================================

async fn dispatch_session_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        "get_session" => Some(execute_get_session(app).await),
        "get_unread_count" => Some(execute_get_unread_count(app, args.clone()).await),
        "get_profile" => Some(execute_get_profile(app).await),
        "update_profile" => Some(execute_update_profile(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_entity_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Entity CRUD
        "create_entity" => Some(execute_create_entity(app, args.clone()).await),
        "update_entity" => Some(execute_update_entity(app, args.clone()).await),
        "delete_entity" => Some(execute_delete_entity(app, args.clone()).await),
        "get_entity" => Some(execute_get_entity(app, args.clone()).await),
        "list_entities" => Some(execute_list_entities(app, args.clone()).await),
        "join_entity" => Some(execute_join_entity(app, args.clone()).await),
        // Member operations
        "add_member" => Some(execute_add_member(app, args.clone()).await),
        "remove_member" => Some(execute_remove_member(app, args.clone()).await),
        "list_members" => Some(execute_list_members(app, args.clone()).await),
        // Invite operations
        "create_invite" => Some(execute_create_invite(app, args.clone()).await),
        "accept_invite" => Some(execute_accept_invite(app, args.clone()).await),
        "list_pending_invites" => Some(execute_list_pending_invites(app).await),
        _ => None,
    }
}

async fn dispatch_message_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Message CRUD
        "send_message" => Some(execute_send_message(app, args.clone()).await),
        "delete_message" => Some(execute_delete_message(app, args.clone()).await),
        "edit_message" => Some(execute_edit_message(app, args.clone()).await),
        "get_messages" => Some(execute_get_messages(app, args.clone()).await),
        // Reactions
        "add_reaction" => Some(execute_add_reaction(app, args.clone()).await),
        "remove_reaction" => Some(execute_remove_reaction(app, args.clone()).await),
        "get_reactions" => Some(execute_get_reactions(app, args.clone()).await),
        "create_custom_reaction" => Some(execute_create_custom_reaction(app, args.clone()).await),
        "get_available_reactions" => Some(execute_get_available_reactions(app, args.clone()).await),
        // Thread operations
        "create_thread" => Some(execute_create_thread(app, args.clone()).await),
        "get_thread_messages" => Some(execute_get_thread_messages(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_kanban_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Board operations
        "create_kanban_board" => Some(execute_create_board(app, args.clone()).await),
        "get_kanban_board" => Some(execute_get_kanban_board(app, args.clone()).await),
        "update_kanban_board" => Some(execute_update_kanban_board(app, args.clone()).await),
        "delete_kanban_board" => Some(execute_delete_kanban_board(app, args.clone()).await),
        "list_kanban_boards" => Some(execute_list_kanban_boards(app, args.clone()).await),
        // Column operations
        "create_kanban_column" => Some(execute_create_column(app, args.clone()).await),
        "get_kanban_column" => Some(execute_get_kanban_column(app, args.clone()).await),
        "update_kanban_column" => Some(execute_update_kanban_column(app, args.clone()).await),
        "delete_kanban_column" => Some(execute_delete_kanban_column(app, args.clone()).await),
        "move_kanban_column" => Some(execute_move_kanban_column(app, args.clone()).await),
        "list_kanban_columns" => Some(execute_list_kanban_columns(app, args.clone()).await),
        // Card operations
        "create_kanban_card" => Some(execute_create_card(app, args.clone()).await),
        "get_kanban_card" => Some(execute_get_kanban_card(app, args.clone()).await),
        "update_kanban_card" => Some(execute_update_kanban_card(app, args.clone()).await),
        "delete_kanban_card" => Some(execute_delete_kanban_card(app, args.clone()).await),
        "move_kanban_card" => Some(execute_move_card(app, args.clone()).await),
        "list_kanban_cards" => Some(execute_list_kanban_cards(app, args.clone()).await),
        "change_card_state" => Some(execute_change_card_state(app, args.clone()).await),
        // Assignment operations
        "assign_user" => Some(execute_assign_user(app, args.clone()).await),
        "unassign_user" => Some(execute_unassign_user(app, args.clone()).await),
        // Tag operations
        "create_kanban_tag" => Some(execute_create_kanban_tag(app, args.clone()).await),
        "list_kanban_tags" => Some(execute_list_kanban_tags(app, args.clone()).await),
        "tag_card" => Some(execute_tag_card(app, args.clone()).await),
        "untag_card" => Some(execute_untag_card(app, args.clone()).await),
        // Step operations
        "add_step" => Some(execute_add_step(app, args.clone()).await),
        "get_step" => Some(execute_get_step(app, args.clone()).await),
        "toggle_step" => Some(execute_toggle_step(app, args.clone()).await),
        "delete_step" => Some(execute_delete_step(app, args.clone()).await),
        // Comment operations
        "add_comment" => Some(execute_add_comment(app, args.clone()).await),
        "list_comments" => Some(execute_list_comments(app, args.clone()).await),
        "delete_comment" => Some(execute_delete_comment(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_file_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        "write_file" => Some(execute_write_file(app, args.clone()).await),
        "read_file" => Some(execute_read_file(app, args.clone()).await),
        "delete_file" => Some(execute_delete_file(app, args.clone()).await),
        "list_files" => Some(execute_list_files(app, args.clone()).await),
        "get_disk_stats" => Some(execute_get_disk_stats(app, args.clone()).await),
        // Media operations
        "upload_with_metadata" => Some(execute_upload_with_metadata(app, args.clone()).await),
        "get_media_metadata" => Some(execute_get_media_metadata(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_contact_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        "create_contact" => Some(execute_create_contact(app, args.clone()).await),
        "update_contact" => Some(execute_update_contact(app, args.clone()).await),
        "delete_contact" => Some(execute_delete_contact(app, args.clone()).await),
        "get_contact" => Some(execute_get_contact(app, args.clone()).await),
        "list_contacts" => Some(execute_list_contacts(app).await),
        "link_contact" => Some(execute_link_contact(app, args.clone()).await),
        "set_favourite_contact" => Some(execute_set_favourite_contact(app, args.clone()).await),
        "remove_favourite_contact" => {
            Some(execute_remove_favourite_contact(app, args.clone()).await)
        }
        "list_favourite_contacts" => Some(execute_list_favourite_contacts(app).await),
        "search_contacts" => Some(execute_search_contacts(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_network_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        "network_start" => Some(execute_network_start(app, args.clone()).await),
        "network_stop" => Some(execute_network_stop(app).await),
        "network_connect" => Some(execute_network_connect(app, args.clone()).await),
        "network_disconnect" => Some(execute_network_disconnect(app, args.clone()).await),
        "network_status" => Some(execute_network_status(app).await),
        "network_peers" => Some(execute_network_peers(app).await),
        "network_request_external_address" => Some(execute_request_external_address(app).await),
        _ => None,
    }
}

async fn dispatch_social_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Presence
        "set_presence" => Some(execute_set_presence(app, args.clone()).await),
        "get_presence" => Some(execute_get_presence(app, args.clone()).await),
        "subscribe_to_presence" => Some(execute_subscribe_to_presence(app, args.clone()).await),
        // WebRTC calls
        "start_voice_call" => Some(execute_start_voice_call(app, args.clone()).await),
        "join_call" => Some(execute_join_call(app, args.clone()).await),
        "end_call" => Some(execute_end_call(app, args.clone()).await),
        // Polls, location, stories
        "create_poll" => Some(execute_create_poll(app, args.clone()).await),
        "vote_in_poll" => Some(execute_vote_in_poll(app, args.clone()).await),
        "share_location" => Some(execute_share_location(app, args.clone()).await),
        "create_story" => Some(execute_create_story(app, args.clone()).await),
        // Presentations
        "start_presentation" => Some(execute_start_presentation(app, args.clone()).await),
        "share_screen" => Some(execute_share_screen(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_misc_tools(
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Website operations
        "create_website" => Some(execute_create_website(app, args.clone()).await),
        "update_website" => Some(execute_update_website(app, args.clone()).await),
        "delete_website" => Some(execute_delete_website(app, args.clone()).await),
        "get_website" => Some(execute_get_website(app, args.clone()).await),
        // Workspace initialization
        "workspace_init" => Some(execute_workspace_init(app, args.clone()).await),
        _ => None,
    }
}

fn parse_entity_type(s: &str) -> Option<EntityType> {
    match s.to_lowercase().as_str() {
        "organisation" | "organization" => Some(EntityType::Organisation),
        "project" => Some(EntityType::Project),
        "group" => Some(EntityType::Group),
        "channel" => Some(EntityType::Channel),
        "person" => Some(EntityType::Person),
        _ => None,
    }
}

fn parse_disk_type(s: &str) -> Option<DiskTypeArg> {
    match s.to_lowercase().as_str() {
        "private" => Some(DiskTypeArg::Private),
        "public" => Some(DiskTypeArg::Public),
        "shared" => Some(DiskTypeArg::Shared),
        _ => None,
    }
}

pub fn success_result(message: &str) -> ToolCallResult {
    ToolCallResult {
        content: vec![ToolContent::Text {
            text: message.to_string(),
        }],
        is_error: false,
    }
}

fn json_result(data: &Value) -> ToolCallResult {
    ToolCallResult {
        content: vec![ToolContent::Text {
            text: serde_json::to_string_pretty(data).unwrap_or_else(|_| "{}".to_string()),
        }],
        is_error: false,
    }
}

fn error_result(message: &str) -> ToolCallResult {
    ToolCallResult {
        content: vec![ToolContent::Text {
            text: message.to_string(),
        }],
        is_error: true,
    }
}

// ============================================================================
// Argument Extraction Helpers
// ============================================================================

/// Extract a required string argument, returning an error result if missing
macro_rules! require_str {
    ($args:expr, $field:expr) => {
        match $args[$field].as_str() {
            Some(s) => s.to_string(),
            None => return error_result(concat!($field, " is required")),
        }
    };
}

/// Extract a string argument with empty default
fn str_or_default(args: &Value, field: &str) -> String {
    args[field].as_str().unwrap_or_default().to_string()
}

/// Extract an optional string argument
fn opt_str(args: &Value, field: &str) -> Option<String> {
    args[field].as_str().map(|s| s.to_string())
}

/// Extract an optional u32 from i64
fn opt_u32(args: &Value, field: &str) -> Option<u32> {
    args[field].as_i64().map(|p| p as u32)
}

/// Extract an optional bool
fn opt_bool(args: &Value, field: &str) -> Option<bool> {
    args[field].as_bool()
}

/// Extract a bool with default value
fn bool_or(args: &Value, field: &str, default: bool) -> bool {
    args[field].as_bool().unwrap_or(default)
}

/// Extract a required entity type, returning an error result if missing or invalid
macro_rules! require_entity_type {
    ($args:expr) => {
        match $args["entity_type"].as_str().and_then(parse_entity_type) {
            Some(t) => t,
            None => return error_result("Invalid or missing entity_type"),
        }
    };
}

/// Extract a required disk type, returning an error result if missing or invalid
macro_rules! require_disk_type {
    ($args:expr) => {
        match $args["disk_type"].as_str().and_then(parse_disk_type) {
            Some(t) => t,
            None => return error_result("Invalid or missing disk_type"),
        }
    };
}

/// Extract a string array with empty default
fn str_array_or_default(args: &Value, field: &str) -> Vec<String> {
    args[field]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

// Command executors

async fn execute_create_entity(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let name = str_or_default(&args, "name");
    let entity_type = require_entity_type!(args);
    let description = opt_str(&args, "description");
    let initial_members = str_array_or_default(&args, "initial_members");

    let cmd = Command::CreateEntity {
        name,
        entity_type,
        description,
        initial_members,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Extract entity_id from the EntityCreated event
            let entity_id = events.iter().find_map(|e| match e {
                Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
                _ => None,
            });

            let result = json!({
                "success": true,
                "events": events.len(),
                "message": "Entity created successfully",
                "id": entity_id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to create entity: {}", e.message)),
    }
}

async fn execute_update_entity(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_type = require_entity_type!(args);
    let entity_id = str_or_default(&args, "entity_id");
    let name = opt_str(&args, "name");
    let description = if args.get("description").is_some() {
        Some(opt_str(&args, "description"))
    } else {
        None
    };

    let cmd = Command::UpdateEntity {
        entity_type,
        entity_id,
        name,
        description,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Entity updated successfully"),
        Err(e) => error_result(&format!("Failed to update entity: {}", e.message)),
    }
}

async fn execute_delete_entity(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_type = require_entity_type!(args);
    let entity_id = str_or_default(&args, "entity_id");

    let cmd = Command::DeleteEntity {
        entity_type,
        entity_id,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Entity deleted successfully"),
        Err(e) => error_result(&format!("Failed to delete entity: {}", e.message)),
    }
}

async fn execute_add_member(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_type = require_entity_type!(args);
    let entity_id = str_or_default(&args, "entity_id");
    let member_id = str_or_default(&args, "member_id");
    let role = args["role"].as_str().unwrap_or("member").to_string();

    let cmd = Command::AddMember {
        entity_type,
        entity_id,
        member_id,
        role,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Member added successfully"),
        Err(e) => error_result(&format!("Failed to add member: {}", e.message)),
    }
}

async fn execute_remove_member(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_type = require_entity_type!(args);
    let entity_id = str_or_default(&args, "entity_id");
    let member_id = str_or_default(&args, "member_id");

    let cmd = Command::RemoveMember {
        entity_type,
        entity_id,
        member_id,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Member removed successfully"),
        Err(e) => error_result(&format!("Failed to remove member: {}", e.message)),
    }
}

async fn execute_send_message(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let entity_type = require_entity_type!(args);
    let text = str_or_default(&args, "text");
    let reply_to_id = opt_str(&args, "reply_to_id");

    let cmd = Command::SendMessage {
        entity_id,
        entity_type,
        text,
        author: String::new(), // Will be filled by the service
        reply_to_id,
        attachments: None,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Extract message_id from the MessageSent event
            let message_id = events.iter().find_map(|e| match e {
                Event::MessageSent { message_id, .. } => Some(message_id.clone()),
                _ => None,
            });

            let result = json!({
                "success": true,
                "message": "Message sent successfully",
                "id": message_id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to send message: {}", e.message)),
    }
}

async fn execute_delete_message(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let entity_type = require_entity_type!(args);
    let message_id = str_or_default(&args, "message_id");

    let cmd = Command::DeleteMessage {
        entity_id,
        entity_type,
        message_id,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Message deleted successfully"),
        Err(e) => error_result(&format!("Failed to delete message: {}", e.message)),
    }
}

async fn execute_edit_message(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let entity_type = require_entity_type!(args);
    let message_id = str_or_default(&args, "message_id");
    let new_text = str_or_default(&args, "new_text");

    let cmd = Command::EditMessage {
        entity_id,
        entity_type,
        message_id,
        new_text,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            let edited_at = events.iter().find_map(|e| match e {
                Event::MessageEdited { edited_at, .. } => Some(*edited_at),
                _ => None,
            });
            let result = json!({
                "success": true,
                "message": "Message edited successfully",
                "edited_at": edited_at
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to edit message: {}", e.message)),
    }
}

async fn execute_add_reaction(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let entity_type = require_entity_type!(args);
    let message_id = str_or_default(&args, "message_id");
    let emoji = str_or_default(&args, "emoji");

    let cmd = Command::AddReaction {
        entity_id,
        entity_type,
        message_id,
        emoji: emoji.clone(),
    };

    match app.execute(cmd).await {
        Ok(_) => {
            let result = json!({
                "success": true,
                "message": "Reaction added successfully",
                "emoji": emoji
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to add reaction: {}", e.message)),
    }
}

async fn execute_remove_reaction(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let entity_type = require_entity_type!(args);
    let message_id = str_or_default(&args, "message_id");
    let emoji = str_or_default(&args, "emoji");

    let cmd = Command::RemoveReaction {
        entity_id,
        entity_type,
        message_id,
        emoji: emoji.clone(),
    };

    match app.execute(cmd).await {
        Ok(_) => {
            let result = json!({
                "success": true,
                "message": "Reaction removed successfully",
                "emoji": emoji
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to remove reaction: {}", e.message)),
    }
}

async fn execute_get_reactions(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let message_id = str_or_default(&args, "message_id");

    let query = Query::GetMessage {
        entity_id,
        message_id,
    };

    match app.query(query).await {
        Ok(QueryResponse::Message(msg)) => {
            let reactions: Vec<serde_json::Value> = msg
                .reactions
                .into_iter()
                .map(|r| {
                    json!({
                        "emoji": r.emoji,
                        "count": r.count,
                        "user_reacted": r.user_reacted,
                        "peer_ids": r.peer_ids
                    })
                })
                .collect();
            json_result(&json!({
                "success": true,
                "reactions": reactions
            }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get reactions: {}", e)),
    }
}

async fn execute_create_custom_reaction(_app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let emoji = require_str!(args, "emoji");
    let short_name = require_str!(args, "short_name");

    // TODO: Implement actual storage logic in CommunitasCore

    json_result(&json!({
        "success": true,
        "message": format!("Custom reaction '{}' created", short_name),
        "reaction": {
            "entity_id": entity_id,
            "emoji": emoji,
            "short_name": short_name
        }
    }))
}

async fn execute_get_available_reactions(_app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    // Return standard reactions plus any custom ones for this entity
    let standard_reactions = vec![
        json!({"emoji": "👍", "short_name": "+1"}),
        json!({"emoji": "👎", "short_name": "-1"}),
        json!({"emoji": "😄", "short_name": "smile"}),
        json!({"emoji": "🎉", "short_name": "tada"}),
        json!({"emoji": "😕", "short_name": "confused"}),
        json!({"emoji": "❤️", "short_name": "heart"}),
        json!({"emoji": "🚀", "short_name": "rocket"}),
        json!({"emoji": "👀", "short_name": "eyes"}),
    ];

    // TODO: Fetch custom reactions from entity storage

    json_result(&json!({
        "entity_id": entity_id,
        "reactions": standard_reactions
    }))
}

async fn execute_create_board(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let board_name = str_or_default(&args, "board_name");
    let description = opt_str(&args, "description");

    let cmd = Command::CreateKanbanBoard {
        entity_id,
        board_name,
        description,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Extract board_id from the KanbanBoardCreated event
            let board_id = events.iter().find_map(|e| match e {
                Event::KanbanBoardCreated { board_id, .. } => Some(board_id.clone()),
                _ => None,
            });

            let result = json!({
                "success": true,
                "message": "Kanban board created successfully",
                "id": board_id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to create board: {}", e.message)),
    }
}

async fn execute_create_column(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = str_or_default(&args, "board_id");
    let column_name = str_or_default(&args, "column_name");
    let position = opt_u32(&args, "position");

    let cmd = Command::CreateKanbanColumn {
        board_id,
        column_name,
        position,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Extract column_id from the KanbanColumnCreated event
            let column_id = events.iter().find_map(|e| match e {
                Event::KanbanColumnCreated { column_id, .. } => Some(column_id.clone()),
                _ => None,
            });

            let result = json!({
                "success": true,
                "message": "Kanban column created successfully",
                "id": column_id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to create column: {}", e.message)),
    }
}

async fn execute_create_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = str_or_default(&args, "board_id");
    let column_id = str_or_default(&args, "column_id");
    let title = str_or_default(&args, "title");
    let description = opt_str(&args, "description");
    let assignee = opt_str(&args, "assignee");

    let cmd = Command::CreateKanbanCard {
        board_id,
        column_id,
        title,
        description,
        assignee,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Extract card_id from the KanbanCardCreated event
            let card_id = events.iter().find_map(|e| match e {
                Event::KanbanCardCreated { card_id, .. } => Some(card_id.clone()),
                _ => None,
            });

            let result = json!({
                "success": true,
                "message": "Card created successfully",
                "id": card_id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to create card: {}", e.message)),
    }
}

async fn execute_move_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = str_or_default(&args, "board_id");
    let card_id = str_or_default(&args, "card_id");
    let target_column_id = str_or_default(&args, "target_column_id");
    let position = opt_u32(&args, "position");

    let cmd = Command::MoveKanbanCard {
        board_id,
        card_id,
        target_column_id,
        position,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Card moved successfully"),
        Err(e) => error_result(&format!("Failed to move card: {}", e.message)),
    }
}

async fn execute_create_invite(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let recipient_id = str_or_default(&args, "recipient_id");
    let entity_type = require_entity_type!(args);
    let entity_id = str_or_default(&args, "entity_id");
    let role = args["role"].as_str().unwrap_or("member").to_string();
    let message = opt_str(&args, "message");

    let cmd = Command::CreateInvite {
        recipient_id,
        entity_type,
        entity_id,
        role,
        message,
        expires_in_hours: None,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Extract invite_id from the InviteCreated event
            let invite_id = events.iter().find_map(|e| match e {
                Event::InviteCreated { invite_id, .. } => Some(invite_id.clone()),
                _ => None,
            });

            let result = json!({
                "success": true,
                "message": "Invite created successfully",
                "id": invite_id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to create invite: {}", e.message)),
    }
}

async fn execute_accept_invite(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let invite_id = str_or_default(&args, "invite_id");

    let cmd = Command::AcceptInvite { invite_id };

    match app.execute(cmd).await {
        Ok(_) => success_result("Invite accepted successfully"),
        Err(e) => error_result(&format!("Failed to accept invite: {}", e.message)),
    }
}

async fn execute_write_file(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let disk_type = require_disk_type!(args);
    let path = str_or_default(&args, "path");
    let content_str = args["content"].as_str().unwrap_or_default();
    let data = content_str.as_bytes().to_vec();

    let cmd = Command::WriteFile {
        entity_id,
        disk_type,
        path,
        data,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("File written successfully"),
        Err(e) => error_result(&format!("Failed to write file: {}", e.message)),
    }
}

async fn execute_read_file(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let disk_type = require_disk_type!(args);
    let path = str_or_default(&args, "path");

    let query = Query::ReadFile {
        entity_id,
        disk_type,
        path,
    };

    match app.query(query).await {
        Ok(QueryResponse::FileContents(content)) => match String::from_utf8(content) {
            Ok(text) => json_result(&json!({"content": text})),
            Err(_) => {
                // Binary content - return as base64
                json_result(&json!({"content_base64": "Binary content"}))
            }
        },
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to read file: {}", e.message)),
    }
}

// Query executors

async fn execute_get_entity(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");

    let query = Query::GetEntity { entity_id };

    match app.query(query).await {
        Ok(QueryResponse::Entity(entity)) => json_result(&json!({
            "id": entity.id,
            "name": entity.name,
            "entity_type": format!("{:?}", entity.entity_type),
            "description": entity.description,
            "created_by": entity.created_by,
            "created_at": entity.created_at,
            "member_count": entity.member_count,
            "parent_org_id": entity.parent_org_id
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get entity: {}", e.message)),
    }
}

async fn execute_list_entities(app: &CommunitasApp, args: Value) -> ToolCallResult {
    // Check if entity_type filter is provided
    let query = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(entity_type) => Query::ListEntitiesByType { entity_type },
        None => Query::ListEntities,
    };

    match app.query(query).await {
        Ok(QueryResponse::EntityList(entities)) => {
            let list: Vec<Value> = entities
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "name": e.name,
                        "entity_type": format!("{:?}", e.entity_type),
                        "description": e.description
                    })
                })
                .collect();
            json_result(&json!({"entities": list}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list entities: {}", e.message)),
    }
}

async fn execute_list_members(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_type = require_entity_type!(args);
    let entity_id = str_or_default(&args, "entity_id");

    let query = Query::ListMembers {
        entity_type,
        entity_id,
    };

    match app.query(query).await {
        Ok(QueryResponse::MemberList(members)) => {
            let list: Vec<Value> = members
                .iter()
                .map(|m| {
                    json!({
                        "member_id": m.member_id,
                        "role": m.role,
                        "joined_at": m.joined_at
                    })
                })
                .collect();
            json_result(&json!({"members": list}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list members: {}", e.message)),
    }
}

async fn execute_get_messages(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");

    let query = Query::GetEntityMessages { entity_id };

    match app.query(query).await {
        Ok(QueryResponse::Messages(messages)) => {
            let list: Vec<Value> = messages
                .iter()
                .map(|m| {
                    json!({
                        "id": m.id,
                        "author": m.author,
                        "text": m.text,
                        "timestamp": m.timestamp,
                        "reply_to_id": m.reply_to_id
                    })
                })
                .collect();
            json_result(&json!({"messages": list}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get messages: {}", e.message)),
    }
}

async fn execute_list_files(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let disk_type = match args["disk_type"].as_str().and_then(parse_disk_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing disk_type"),
    };
    let path = args["path"].as_str().unwrap_or("/").to_string();

    let query = Query::ListFiles {
        entity_id,
        disk_type,
        path,
    };

    match app.query(query).await {
        Ok(QueryResponse::FileList(files)) => {
            let list: Vec<Value> = files
                .iter()
                .map(|f| {
                    json!({
                        "path": f.path,
                        "name": f.name,
                        "is_directory": f.is_directory,
                        "size_bytes": f.size_bytes
                    })
                })
                .collect();
            json_result(&json!({"files": list}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list files: {}", e.message)),
    }
}

async fn execute_get_profile(app: &CommunitasApp) -> ToolCallResult {
    let query = Query::GetProfile;

    match app.query(query).await {
        Ok(QueryResponse::Profile {
            four_words,
            display_name,
            device_name,
            device_type,
        }) => json_result(&json!({
            "four_words": four_words,
            "display_name": display_name,
            "device_name": device_name,
            "device_type": device_type
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get profile: {}", e.message)),
    }
}

async fn execute_update_profile(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let display_name = require_str!(args, "display_name");

    let cmd = Command::UpdateDisplayName { display_name };

    match app.execute(cmd).await {
        Ok(events) => {
            for event in &events {
                if let Event::DisplayNameUpdated { old_name, new_name } = event {
                    return json_result(&json!({
                        "success": true,
                        "message": "Profile updated",
                        "old_display_name": old_name,
                        "new_display_name": new_name
                    }));
                }
            }
            success_result("Profile updated")
        }
        Err(e) => error_result(&format!("Failed to update profile: {}", e.message)),
    }
}

async fn execute_set_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let status_str = require_str!(args, "status");

    let status = match status_str.as_str() {
        "online" => PresenceStatus::Online,
        "away" => PresenceStatus::Away,
        "busy" => PresenceStatus::Busy,
        "invisible" => PresenceStatus::Invisible,
        _ => return error_result("Invalid status. Must be one of: online, away, busy, invisible"),
    };

    let entity_id = opt_str(&args, "entity_id");

    let update = if let Some(eid) = entity_id {
        let mut update = PresenceUpdate::status_only(status);
        update.current_entity = Some(eid);
        update
    } else {
        PresenceUpdate::status_only(status)
    };

    // Get user ID from profile
    let user_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match PresenceOperations::update_presence(app, user_id, update).await {
        Ok(_) => success_result(&format!("Presence set to {}", status_str)),
        Err(e) => error_result(&format!("Failed to set presence: {}", e)),
    }
}

async fn execute_get_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let user_ids = match args["user_ids"].as_array() {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
        None => return error_result("user_ids must be an array of strings"),
    };

    if user_ids.is_empty() {
        return error_result("user_ids cannot be empty");
    }

    match PresenceOperations::get_users_presence(app, user_ids).await {
        Ok(presences) => json_result(&json!({ "presences": presences })),
        Err(e) => error_result(&format!("Failed to get presence: {}", e)),
    }
}

async fn execute_subscribe_to_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_ids = match args["entity_ids"].as_array() {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
        None => return error_result("entity_ids must be an array of strings"),
    };

    if entity_ids.is_empty() {
        return error_result("entity_ids cannot be empty");
    }

    let subscription = PresenceSubscription {
        entity_ids: entity_ids.clone(),
        include_self: false,
    };

    match PresenceOperations::subscribe_to_presence(app, subscription).await {
        Ok(sub_id) => json_result(&json!({
            "subscription_id": sub_id,
            "message": format!("Subscribed to presence for {} entities", entity_ids.len())
        })),
        Err(e) => error_result(&format!("Failed to subscribe to presence: {}", e)),
    }
}

async fn execute_start_voice_call(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let participant_ids = str_array_or_default(&args, "participant_ids");
    let video_enabled = bool_or(&args, "video_enabled", true);

    let request = CallRequest {
        entity_id,
        participant_ids,
        video_enabled,
    };

    // Get user ID
    let initiator_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match WebRtcOperations::start_call(app, request, initiator_id) {
        Ok(session) => json_result(&json!({ "call_session": session })),
        Err(e) => error_result(&format!("Failed to start call: {}", e)),
    }
}

async fn execute_join_call(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");

    let user_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match WebRtcOperations::join_call(app, call_id, user_id) {
        Ok(_) => success_result("Joined call successfully"),
        Err(e) => error_result(&format!("Failed to join call: {}", e)),
    }
}

async fn execute_end_call(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");

    match WebRtcOperations::end_call(app, call_id) {
        Ok(_) => success_result("Call ended"),
        Err(e) => error_result(&format!("Failed to end call: {}", e)),
    }
}

async fn execute_upload_with_metadata(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_disk_type!(args);
    let path = require_str!(args, "path");
    let content_base64 = require_str!(args, "content");
    let metadata = args["metadata"].clone();

    // Decode base64 content
    let data = match BASE64_STANDARD.decode(content_base64) {
        Ok(d) => d,
        Err(e) => return error_result(&format!("Failed to decode content: {}", e)),
    };

    // 1. Write the main file
    let cmd_file = Command::WriteFile {
        entity_id: entity_id.clone(),
        disk_type,
        path: path.clone(),
        data,
    };
    if let Err(e) = app.execute(cmd_file).await {
        return error_result(&format!("Failed to write file: {}", e.message));
    }

    // 2. Write the metadata file
    let meta_path = format!("{}.meta.json", path);
    let meta_data = serde_json::to_vec(&metadata).unwrap_or_default();

    let cmd_meta = Command::WriteFile {
        entity_id,
        disk_type,
        path: meta_path,
        data: meta_data,
    };

    match app.execute(cmd_meta).await {
        Ok(_) => success_result("File and metadata uploaded successfully"),
        Err(e) => error_result(&format!("Failed to write metadata: {}", e.message)),
    }
}

async fn execute_get_media_metadata(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_disk_type!(args);
    let path = require_str!(args, "path");

    let meta_path = format!("{}.meta.json", path);

    let query = Query::ReadFile {
        entity_id,
        disk_type,
        path: meta_path,
    };

    match app.query(query).await {
        Ok(QueryResponse::FileContents(content)) => {
            match serde_json::from_slice::<Value>(&content) {
                Ok(meta) => json_result(&meta),
                Err(_) => error_result("Failed to parse metadata JSON"),
            }
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!(
            "Failed to read metadata (or file not found): {}",
            e.message
        )),
    }
}

async fn execute_create_poll(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let question = require_str!(args, "question");
    let options = str_array_or_default(&args, "options");
    if options.is_empty() {
        return error_result("options required");
    }
    let allows_multiple = bool_or(&args, "allows_multiple", false);
    let duration_hours = args["duration_hours"].as_u64();

    match PollOperations::create_poll(
        app,
        entity_id,
        question,
        options,
        allows_multiple,
        duration_hours,
    ) {
        Ok(poll) => json_result(&json!({ "poll": poll })),
        Err(e) => error_result(&format!("Failed to create poll: {}", e)),
    }
}

async fn execute_vote_in_poll(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let poll_id = require_str!(args, "poll_id");
    let option_id = require_str!(args, "option_id");

    // Get user ID
    let user_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match PollOperations::vote(app, poll_id, option_id, user_id) {
        Ok(_) => success_result("Voted successfully"),
        Err(e) => error_result(&format!("Failed to vote: {}", e)),
    }
}

async fn execute_share_location(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let latitude = match args["latitude"].as_f64() {
        Some(f) => f,
        None => return error_result("latitude required"),
    };
    let longitude = match args["longitude"].as_f64() {
        Some(f) => f,
        None => return error_result("longitude required"),
    };
    let is_live = bool_or(&args, "is_live", false);
    let duration_minutes = args["duration_minutes"].as_u64();

    // Get user ID
    let user_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match LocationOperations::share_location(
        app,
        entity_id,
        user_id,
        latitude,
        longitude,
        is_live,
        duration_minutes,
    ) {
        Ok(loc) => json_result(&json!({ "location": loc })),
        Err(e) => error_result(&format!("Failed to share location: {}", e)),
    }
}

async fn execute_create_story(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let text = require_str!(args, "text");
    let media_paths = str_array_or_default(&args, "media_paths");
    let expires_hours = args["expires_hours"].as_u64().unwrap_or(24);

    // Get user ID
    let author_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match StoryOperations::create_story(app, entity_id, author_id, text, media_paths, expires_hours)
    {
        Ok(story) => json_result(&json!({ "story": story })),
        Err(e) => error_result(&format!("Failed to create story: {}", e)),
    }
}

async fn execute_start_presentation(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let slides: Vec<Slide> = match serde_json::from_value(args["slides"].clone()) {
        Ok(s) => s,
        Err(_) => return error_result("Invalid slides format"),
    };

    // Get user ID
    let presenter_id = match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile { four_words, .. }) => four_words,
        _ => return error_result("Failed to get user profile"),
    };

    match PresentationOperations::start_presentation(app, entity_id, presenter_id, slides) {
        Ok(session) => json_result(&json!({ "session": session })),
        Err(e) => error_result(&format!("Failed to start presentation: {}", e)),
    }
}

async fn execute_share_screen(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");
    let region: Option<ScreenRegion> = if let Some(r) = args.get("region") {
        match serde_json::from_value(r.clone()) {
            Ok(reg) => Some(reg),
            Err(_) => return error_result("Invalid region format"),
        }
    } else {
        None
    };

    match PresentationOperations::share_screen(app, call_id, region) {
        Ok(_) => success_result("Screen sharing started"),
        Err(e) => error_result(&format!("Failed to share screen: {}", e)),
    }
}

async fn execute_get_session(app: &CommunitasApp) -> ToolCallResult {
    match app.query(communitas_core::command::Query::GetProfile).await {
        Ok(communitas_core::command::QueryResponse::Profile {
            four_words,
            display_name,
            device_name,
            ..
        }) => json_result(&json!({
            "four_words": four_words,
            "display_name": display_name,
            "device_name": device_name,
            "authenticated": true
        })),
        _ => error_result("Failed to get session info"),
    }
}

async fn execute_get_unread_count(_app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_ids = match args["entity_ids"].as_array() {
        Some(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>(),
        None => return error_result("entity_ids required"),
    };

    // TODO: Implement actual unread count query in Core
    let mut unread_counts = serde_json::Map::new();
    for id in entity_ids {
        unread_counts.insert(id, json!(0));
    }

    json_result(&json!({ "unread_counts": unread_counts }))
}

async fn execute_list_pending_invites(app: &CommunitasApp) -> ToolCallResult {
    let query = Query::ListPendingInvites;

    match app.query(query).await {
        Ok(QueryResponse::InviteList(invites)) => {
            let list: Vec<Value> = invites
                .iter()
                .map(|i| {
                    json!({
                        "id": i.id,
                        "sender_id": i.sender_id,
                        "recipient_id": i.recipient_id,
                        "entity_type": format!("{:?}", i.entity_type),
                        "entity_id": i.entity_id,
                        "role": i.role,
                        "status": format!("{:?}", i.status),
                        "message": i.message,
                        "created_at": i.created_at,
                        "expires_at": i.expires_at
                    })
                })
                .collect();
            json_result(&json!({"invites": list}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list invites: {}", e.message)),
    }
}

// Network executors

async fn execute_network_start(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let preferred_port = args["preferred_port"].as_u64().map(|p| p as u16);

    let cmd = Command::StartNetworking { preferred_port };

    match app.execute(cmd).await {
        Ok(events) => {
            // Look for NetworkingStarted event to get the listen address
            for event in &events {
                if let Event::NetworkingStarted {
                    listen_address,
                    connection_identity,
                } = event
                {
                    return json_result(&json!({
                        "success": true,
                        "message": "Networking started",
                        "listen_address": listen_address,
                        "connection_identity": connection_identity
                    }));
                }
            }
            success_result("Networking started")
        }
        Err(e) => error_result(&format!("Failed to start networking: {}", e.message)),
    }
}

async fn execute_network_stop(app: &CommunitasApp) -> ToolCallResult {
    let cmd = Command::StopNetworking;

    match app.execute(cmd).await {
        Ok(_) => success_result("Networking stopped"),
        Err(e) => error_result(&format!("Failed to stop networking: {}", e.message)),
    }
}

async fn execute_network_connect(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let peer_four_words = require_str!(args, "peer_four_words");

    let cmd = Command::ConnectToPeer { peer_four_words };

    match app.execute(cmd).await {
        Ok(events) => {
            // Check for PeerConnected event
            for event in &events {
                if let Event::PeerConnected { peer_four_words } = event {
                    return json_result(&json!({
                        "success": true,
                        "message": "Connected to peer",
                        "peer_four_words": peer_four_words
                    }));
                }
                if let Event::ConnectionFailed {
                    peer_four_words,
                    reason,
                } = event
                {
                    return error_result(&format!(
                        "Connection to {} failed: {}",
                        peer_four_words, reason
                    ));
                }
            }
            success_result("Connection initiated")
        }
        Err(e) => error_result(&format!("Failed to connect: {}", e.message)),
    }
}

async fn execute_network_status(app: &CommunitasApp) -> ToolCallResult {
    // Query multiple network state aspects
    let is_active = match app.query(Query::IsNetworkingActive).await {
        Ok(QueryResponse::Bool(active)) => active,
        _ => false,
    };

    let connection_identity = match app.query(Query::GetConnectionIdentity).await {
        Ok(QueryResponse::OptionalString(identity)) => identity,
        _ => None,
    };

    let external_address = match app.query(Query::GetExternalAddress).await {
        Ok(QueryResponse::OptionalString(addr)) => addr,
        _ => None,
    };

    // Get peer count
    let peer_count = match app.query(Query::ListOnlinePeers).await {
        Ok(QueryResponse::PeerList(peers)) => peers.len(),
        _ => 0,
    };

    json_result(&json!({
        "is_active": is_active,
        "connection_identity": connection_identity,
        "external_address": external_address,
        "peer_count": peer_count
    }))
}

async fn execute_network_peers(app: &CommunitasApp) -> ToolCallResult {
    match app.query(Query::ListOnlinePeers).await {
        Ok(QueryResponse::PeerList(peers)) => json_result(&json!({
            "peers": peers,
            "count": peers.len()
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list peers: {}", e.message)),
    }
}

async fn execute_request_external_address(app: &CommunitasApp) -> ToolCallResult {
    let cmd = Command::RequestExternalAddress;

    match app.execute(cmd).await {
        Ok(events) => {
            // Check for ExternalAddressDiscovered event
            for event in &events {
                if let Event::ExternalAddressDiscovered { address } = event {
                    return json_result(&json!({
                        "success": true,
                        "message": "External address discovered",
                        "external_address": address
                    }));
                }
            }
            success_result("External address request initiated")
        }
        Err(e) => error_result(&format!(
            "Failed to request external address: {}",
            e.message
        )),
    }
}

// Contact executors

async fn execute_create_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let display_name = require_str!(args, "display_name");
    let four_words = opt_str(&args, "four_words");
    let is_favourite = bool_or(&args, "is_favourite", false);

    let cmd = Command::CreateContact {
        display_name,
        four_words,
        is_favourite,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Look for ContactCreated event to get the contact_id
            for event in &events {
                if let Event::ContactCreated {
                    contact_id,
                    display_name,
                    four_words,
                } = event
                {
                    return json_result(&json!({
                        "success": true,
                        "message": "Contact created",
                        "contact_id": contact_id,
                        "display_name": display_name,
                        "four_words": four_words
                    }));
                }
            }
            success_result("Contact created")
        }
        Err(e) => error_result(&format!("Failed to create contact: {}", e.message)),
    }
}

async fn execute_update_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let contact_id = require_str!(args, "contact_id");
    let display_name = opt_str(&args, "display_name");
    let is_favourite = opt_bool(&args, "is_favourite");

    let cmd = Command::UpdateContact {
        contact_id,
        display_name,
        is_favourite,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Contact updated"),
        Err(e) => error_result(&format!("Failed to update contact: {}", e.message)),
    }
}

async fn execute_delete_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let contact_id = require_str!(args, "contact_id");

    let cmd = Command::DeleteContact { contact_id };

    match app.execute(cmd).await {
        Ok(_) => success_result("Contact deleted"),
        Err(e) => error_result(&format!("Failed to delete contact: {}", e.message)),
    }
}

async fn execute_link_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let contact_id = require_str!(args, "contact_id");
    let four_words = require_str!(args, "four_words");

    let cmd = Command::LinkContact {
        contact_id,
        four_words,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Contact linked to network identity"),
        Err(e) => error_result(&format!("Failed to link contact: {}", e.message)),
    }
}

async fn execute_set_favourite_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let four_words = require_str!(args, "four_words");

    let cmd = Command::SetFavouriteContact { four_words };

    match app.execute(cmd).await {
        Ok(_) => success_result("Contact marked as favourite"),
        Err(e) => error_result(&format!("Failed to set favourite: {}", e.message)),
    }
}

async fn execute_remove_favourite_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let four_words = require_str!(args, "four_words");

    let cmd = Command::RemoveFavouriteContact { four_words };

    match app.execute(cmd).await {
        Ok(_) => success_result("Contact removed from favourites"),
        Err(e) => error_result(&format!("Failed to remove favourite: {}", e.message)),
    }
}

async fn execute_get_contact(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let contact_id = require_str!(args, "contact_id");

    let query = Query::GetContact { contact_id };

    match app.query(query).await {
        Ok(QueryResponse::Contact(contact)) => json_result(&json!({
            "id": contact.id,
            "display_name": contact.display_name,
            "four_words": contact.four_words,
            "is_favourite": contact.is_favourite,
            "is_online": contact.is_online,
            "created_at": contact.created_at,
            "last_seen": contact.last_seen
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get contact: {}", e.message)),
    }
}

async fn execute_list_contacts(app: &CommunitasApp) -> ToolCallResult {
    let query = Query::ListContacts;

    match app.query(query).await {
        Ok(QueryResponse::ContactList(contacts)) => {
            let list: Vec<Value> = contacts
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "display_name": c.display_name,
                        "four_words": c.four_words,
                        "is_favourite": c.is_favourite,
                        "is_online": c.is_online
                    })
                })
                .collect();
            json_result(&json!({"contacts": list, "count": list.len()}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list contacts: {}", e.message)),
    }
}

async fn execute_list_favourite_contacts(app: &CommunitasApp) -> ToolCallResult {
    let query = Query::ListFavouriteContacts;

    match app.query(query).await {
        Ok(QueryResponse::ContactList(contacts)) => {
            let list: Vec<Value> = contacts
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "display_name": c.display_name,
                        "four_words": c.four_words,
                        "is_favourite": c.is_favourite,
                        "is_online": c.is_online
                    })
                })
                .collect();
            json_result(&json!({"contacts": list, "count": list.len()}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list favourite contacts: {}", e.message)),
    }
}

async fn execute_search_contacts(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let query_str = require_str!(args, "query");

    let query = Query::SearchContacts { query: query_str };

    match app.query(query).await {
        Ok(QueryResponse::ContactList(contacts)) => {
            let list: Vec<Value> = contacts
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "display_name": c.display_name,
                        "four_words": c.four_words,
                        "is_favourite": c.is_favourite,
                        "is_online": c.is_online
                    })
                })
                .collect();
            json_result(&json!({"contacts": list, "count": list.len()}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to search contacts: {}", e.message)),
    }
}

// ========== Website Executors ==========

async fn execute_create_website(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let html = require_str!(args, "html");
    let css = opt_str(&args, "css");
    let js = opt_str(&args, "js");
    let metadata = opt_str(&args, "metadata");

    let cmd = Command::CreateWebsite {
        entity_id,
        html,
        css,
        js,
        metadata,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            for event in &events {
                if let Event::WebsiteCreated {
                    entity_id,
                    website_root_hash,
                    published_at,
                    size_bytes,
                } = event
                {
                    return json_result(&json!({
                        "status": "created",
                        "entity_id": entity_id,
                        "website_root_hash": website_root_hash,
                        "published_at": published_at,
                        "size_bytes": size_bytes,
                        "url": format!("communitas://{}/website", entity_id)
                    }));
                }
            }
            json_result(&json!({"status": "created"}))
        }
        Err(e) => error_result(&format!("Failed to create website: {}", e.message)),
    }
}

async fn execute_update_website(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let html = opt_str(&args, "html");
    let css = opt_str(&args, "css");
    let js = opt_str(&args, "js");
    let metadata = opt_str(&args, "metadata");

    // At least one field should be provided
    if html.is_none() && css.is_none() && js.is_none() && metadata.is_none() {
        return error_result("At least one field (html, css, js, metadata) must be provided");
    }

    let cmd = Command::UpdateWebsite {
        entity_id,
        html,
        css,
        js,
        metadata,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            for event in &events {
                if let Event::WebsiteUpdated {
                    entity_id,
                    website_root_hash,
                    updated_at,
                    size_bytes,
                } = event
                {
                    return json_result(&json!({
                        "status": "updated",
                        "entity_id": entity_id,
                        "website_root_hash": website_root_hash,
                        "updated_at": updated_at,
                        "size_bytes": size_bytes
                    }));
                }
            }
            json_result(&json!({"status": "updated"}))
        }
        Err(e) => error_result(&format!("Failed to update website: {}", e.message)),
    }
}

async fn execute_delete_website(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    let cmd = Command::DeleteWebsite { entity_id };

    match app.execute(cmd).await {
        Ok(events) => {
            for event in &events {
                if let Event::WebsiteDeleted { entity_id } = event {
                    return json_result(&json!({
                        "status": "deleted",
                        "entity_id": entity_id
                    }));
                }
            }
            json_result(&json!({"status": "deleted"}))
        }
        Err(e) => error_result(&format!("Failed to delete website: {}", e.message)),
    }
}

async fn execute_get_website(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    let query = Query::GetWebsite { entity_id };

    match app.query(query).await {
        Ok(QueryResponse::Website(website)) => json_result(&json!({
            "entity_id": website.entity_id,
            "html": website.html,
            "css": website.css,
            "js": website.js,
            "website_root_hash": website.website_root_hash,
            "published_at": website.published_at,
            "size_bytes": website.size_bytes,
            "url": website.url
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get website: {}", e.message)),
    }
}

// ========== Kanban Executors ==========

async fn execute_update_kanban_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let title = opt_str(&args, "title");
    let description = opt_str(&args, "description");
    let assignee = opt_str(&args, "assignee");

    let cmd = Command::UpdateKanbanCard {
        board_id,
        card_id,
        title,
        description,
        assignee,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Kanban card updated"),
        Err(e) => error_result(&format!("Failed to update Kanban card: {}", e.message)),
    }
}

async fn execute_delete_kanban_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");

    let cmd = Command::DeleteKanbanCard { board_id, card_id };

    match app.execute(cmd).await {
        Ok(_) => success_result("Kanban card deleted"),
        Err(e) => error_result(&format!("Failed to delete Kanban card: {}", e.message)),
    }
}

async fn execute_list_kanban_boards(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    let query = Query::ListKanbanBoards { entity_id };

    match app.query(query).await {
        Ok(QueryResponse::KanbanBoardList(boards)) => {
            let list: Vec<Value> = boards
                .iter()
                .map(|b| {
                    json!({
                        "id": b.id,
                        "entity_id": b.entity_id,
                        "name": b.name,
                        "description": b.description,
                        "column_count": b.column_count
                    })
                })
                .collect();
            json_result(&json!({"boards": list, "count": list.len()}))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list Kanban boards: {}", e.message)),
    }
}

async fn execute_get_kanban_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");

    let query = Query::GetKanbanCard { board_id, card_id };

    match app.query(query).await {
        Ok(QueryResponse::KanbanCard(card)) => json_result(&json!({
            "id": card.id,
            "column_id": card.column_id,
            "title": card.title,
            "description": card.description,
            "assignee": card.assignee,
            "position": card.position
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get Kanban card: {}", e.message)),
    }
}

async fn execute_get_kanban_board(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    let query = Query::GetKanbanBoard { board_id };

    match app.query(query).await {
        Ok(QueryResponse::KanbanBoard(board)) => json_result(&json!({
            "id": board.id,
            "entity_id": board.entity_id,
            "name": board.name,
            "description": board.description,
            "column_count": board.column_count
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get Kanban board: {}", e.message)),
    }
}

// ========== Kanban Column Executors ==========

async fn execute_list_kanban_columns(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.list_columns(&board_id) {
        Ok(columns) => {
            let list: Vec<Value> = columns
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "name": c.name,
                        "position": c.position,
                        "color": c.color,
                        "wip_limit": c.wip_limit
                    })
                })
                .collect();
            json_result(&json!({"columns": list, "count": list.len()}))
        }
        Err(e) => error_result(&format!("Failed to list columns: {}", e)),
    }
}

async fn execute_get_kanban_column(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.get_column(&board_id, &column_id) {
        Ok(column) => json_result(&json!({
            "id": column.id,
            "name": column.name,
            "position": column.position,
            "color": column.color,
            "wip_limit": column.wip_limit
        })),
        Err(e) => error_result(&format!("Failed to get column: {}", e)),
    }
}

async fn execute_update_kanban_column(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");
    let name = opt_str(&args, "name");
    // ColumnUpdate uses Option<Option<String>> - None = don't change, Some(None) = set to null, Some(Some(v)) = set to value
    let color = if args["color"].is_null() {
        Some(None) // explicitly set to null
    } else {
        args["color"].as_str().map(|s| Some(s.to_string()))
    };
    let wip_limit = if args["wip_limit"].is_null() {
        Some(None) // explicitly set to null
    } else {
        args["wip_limit"].as_u64().map(|w| Some(w as u32))
    };

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    let updates = communitas_kanban::ColumnUpdate {
        name,
        color,
        wip_limit,
    };

    match ctx
        .kanban_service
        .update_column(&board_id, &column_id, updates)
    {
        Ok(column) => json_result(&json!({
            "id": column.id,
            "name": column.name,
            "position": column.position,
            "color": column.color,
            "wip_limit": column.wip_limit
        })),
        Err(e) => error_result(&format!("Failed to update column: {}", e)),
    }
}

async fn execute_delete_kanban_column(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.delete_column(&board_id, &column_id) {
        Ok(()) => success_result("Column deleted"),
        Err(e) => error_result(&format!("Failed to delete column: {}", e)),
    }
}

async fn execute_move_kanban_column(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");
    let new_position = match args["new_position"].as_u64() {
        Some(pos) => pos as u32,
        None => return error_result("new_position is required"),
    };

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .move_column(&board_id, &column_id, new_position)
    {
        Ok(()) => success_result("Column moved"),
        Err(e) => error_result(&format!("Failed to move column: {}", e)),
    }
}

// ========== Kanban Card State Executor ==========

async fn execute_change_card_state(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let state_str = require_str!(args, "state");

    let state = match state_str.as_str() {
        "Open" => communitas_kanban::CardState::Open,
        "Closed" => communitas_kanban::CardState::Closed,
        "Postponed" => communitas_kanban::CardState::Postponed,
        "Archived" => communitas_kanban::CardState::Archived,
        _ => return error_result("Invalid state. Must be: Open, Closed, Postponed, or Archived"),
    };

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .change_card_state(&board_id, &card_id, state)
    {
        Ok(()) => json_result(&json!({
            "card_id": card_id,
            "state": state_str,
            "success": true
        })),
        Err(e) => error_result(&format!("Failed to change card state: {}", e)),
    }
}

// ========== Kanban Assignment Executors ==========

async fn execute_assign_user(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let user_id = require_str!(args, "user_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .assign_user(&board_id, &card_id, &user_id)
    {
        Ok(()) => success_result("User assigned to card"),
        Err(e) => error_result(&format!("Failed to assign user: {}", e)),
    }
}

async fn execute_unassign_user(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let user_id = require_str!(args, "user_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .unassign_user(&board_id, &card_id, &user_id)
    {
        Ok(()) => success_result("User unassigned from card"),
        Err(e) => error_result(&format!("Failed to unassign user: {}", e)),
    }
}

// ========== Kanban Tag Executors ==========

async fn execute_create_kanban_tag(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let name = require_str!(args, "name");
    let color = require_str!(args, "color");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.create_tag(&board_id, name, color) {
        Ok(tag) => json_result(&json!({
            "id": tag.id,
            "name": tag.name,
            "color": tag.color
        })),
        Err(e) => error_result(&format!("Failed to create tag: {}", e)),
    }
}

async fn execute_list_kanban_tags(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.list_tags(&board_id) {
        Ok(tags) => {
            let list: Vec<Value> = tags
                .iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "name": t.name,
                        "color": t.color
                    })
                })
                .collect();
            json_result(&json!({"tags": list, "count": list.len()}))
        }
        Err(e) => error_result(&format!("Failed to list tags: {}", e)),
    }
}

async fn execute_tag_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let tag_id = require_str!(args, "tag_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.tag_card(&board_id, &card_id, &tag_id) {
        Ok(()) => success_result("Tag added to card"),
        Err(e) => error_result(&format!("Failed to tag card: {}", e)),
    }
}

async fn execute_untag_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let tag_id = require_str!(args, "tag_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.untag_card(&board_id, &card_id, &tag_id) {
        Ok(()) => success_result("Tag removed from card"),
        Err(e) => error_result(&format!("Failed to untag card: {}", e)),
    }
}

// ========== Kanban Step Executors ==========

async fn execute_add_step(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let text = require_str!(args, "text");
    let position = opt_u32(&args, "position");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .add_step(&board_id, &card_id, text, position)
    {
        Ok(step) => json_result(&json!({
            "id": step.id,
            "text": step.text,
            "completed": step.completed,
            "position": step.position
        })),
        Err(e) => error_result(&format!("Failed to add step: {}", e)),
    }
}

async fn execute_get_step(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let step_id = require_str!(args, "step_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.get_step(&board_id, &card_id, &step_id) {
        Ok(step) => json_result(&json!({
            "id": step.id,
            "text": step.text,
            "completed": step.completed,
            "position": step.position
        })),
        Err(e) => error_result(&format!("Failed to get step: {}", e)),
    }
}

async fn execute_toggle_step(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let step_id = require_str!(args, "step_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .toggle_step(&board_id, &card_id, &step_id)
    {
        Ok(step) => json_result(&json!({
            "id": step.id,
            "text": step.text,
            "completed": step.completed,
            "position": step.position
        })),
        Err(e) => error_result(&format!("Failed to toggle step: {}", e)),
    }
}

async fn execute_delete_step(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let step_id = require_str!(args, "step_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .delete_step(&board_id, &card_id, &step_id)
    {
        Ok(()) => success_result("Step deleted"),
        Err(e) => error_result(&format!("Failed to delete step: {}", e)),
    }
}

// ========== Kanban Comment Executors ==========

async fn execute_add_comment(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let content = require_str!(args, "content");
    let reply_to_id = opt_str(&args, "reply_to_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    // add_comment takes (board_id, card_id, content, reply_to_id) - author is auto-set from peer_id
    match ctx
        .kanban_service
        .add_comment(&board_id, &card_id, content, reply_to_id)
    {
        Ok(comment) => json_result(&json!({
            "id": comment.id,
            "author_id": comment.author_id,
            "content": comment.content,
            "created_at": comment.created_at,
            "reply_to_id": comment.reply_to_id
        })),
        Err(e) => error_result(&format!("Failed to add comment: {}", e)),
    }
}

async fn execute_list_comments(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.list_comments(&board_id, &card_id) {
        Ok(comments) => {
            let list: Vec<Value> = comments
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "author_id": c.author_id,
                        "content": c.content,
                        "created_at": c.created_at,
                        "reply_to_id": c.reply_to_id
                    })
                })
                .collect();
            json_result(&json!({"comments": list, "count": list.len()}))
        }
        Err(e) => error_result(&format!("Failed to list comments: {}", e)),
    }
}

async fn execute_delete_comment(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let comment_id = require_str!(args, "comment_id");

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .delete_comment(&board_id, &card_id, &comment_id)
    {
        Ok(()) => success_result("Comment deleted"),
        Err(e) => error_result(&format!("Failed to delete comment: {}", e)),
    }
}

// ========== Entity Join Executor ==========

async fn execute_join_entity(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let id = require_str!(args, "id");
    let name = require_str!(args, "name");
    let entity_type = require_entity_type!(args);
    let entity_type_str = str_or_default(&args, "entity_type");
    let created_by = require_str!(args, "created_by");
    let description = opt_str(&args, "description");
    let role = args["role"].as_str().unwrap_or("member").to_string();

    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    // Get current user's identity
    let joiner_four_words = ctx.four_words.clone();

    // Use current timestamp
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    // Import the entity (this creates it and adds us as a member)
    match ctx
        .entity_service
        .import_entity(
            id.clone(),
            name.clone(),
            entity_type, // EntityType is Copy
            description,
            created_by,
            created_at,
            joiner_four_words.clone(),
            role.clone(),
        )
        .await
    {
        Ok(_entity) => {
            // Subscribe to the entity's gossip topic if available
            if let Some(gossip) = ctx.gossip.as_ref()
                && let Err(e) = gossip.join_entity(&id, &entity_type_str).await
            {
                tracing::warn!(
                    "Failed to join entity topic for {} (may already be joined): {}",
                    id,
                    e
                );
            }

            json_result(&json!({
                "success": true,
                "id": id,
                "name": name,
                "entity_type": entity_type_str,
                "joined_as": joiner_four_words,
                "role": role
            }))
        }
        Err(e) => error_result(&format!("Failed to join entity: {}", e)),
    }
}

// ========== File Operations Executors ==========

async fn execute_delete_file(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_disk_type!(args);
    let path = require_str!(args, "path");

    let cmd = Command::DeleteFile {
        entity_id,
        disk_type,
        path,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("File deleted"),
        Err(e) => error_result(&format!("Failed to delete file: {}", e.message)),
    }
}

async fn execute_get_disk_stats(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_disk_type!(args);

    let query = Query::GetDiskStats {
        entity_id,
        disk_type,
    };

    match app.query(query).await {
        Ok(QueryResponse::DiskStats(stats)) => json_result(&json!({
            "entity_id": stats.entity_id,
            "disk_type": format!("{:?}", stats.disk_type),
            "used_bytes": stats.used_bytes,
            "file_count": stats.file_count,
            "dir_count": stats.dir_count
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get disk stats: {}", e.message)),
    }
}

async fn execute_create_thread(_app: &CommunitasApp, args: Value) -> ToolCallResult {
    let channel_id = require_str!(args, "channel_id");
    let parent_message_id = require_str!(args, "parent_message_id");

    json_result(&json!({
        "thread_id": parent_message_id.clone(),
        "channel_id": channel_id,
        "parent_message_id": parent_message_id,
        "info": "Threads are created implicitly when you send a message with reply_to_id. Use send_message with reply_to_id set to the parent message ID to create/add to a thread."
    }))
}

async fn execute_get_thread_messages(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "channel_id");
    let parent_message_id = require_str!(args, "thread_id");

    let query = Query::GetThreadMessages {
        entity_id: entity_id.clone(),
        parent_message_id: parent_message_id.clone(),
    };

    match app.query(query).await {
        Ok(QueryResponse::Messages(messages)) => {
            let list: Vec<Value> = messages
                .iter()
                .map(|m| {
                    json!({
                        "id": m.id,
                        "author": m.author,
                        "text": m.text,
                        "timestamp": m.timestamp,
                        "reply_to_id": m.reply_to_id
                    })
                })
                .collect();
            json_result(&json!({
                "messages": list,
                "channel_id": entity_id,
                "thread_id": parent_message_id
            }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get thread messages: {}", e.message)),
    }
}

// ========== NEW TOOLS FOR MCP CONSOLIDATION ==========

// Note: health_check and core_status are handled as pre-auth tools in server.rs

/// Network disconnect - disconnect from a specific peer
async fn execute_network_disconnect(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let _peer_four_words = require_str!(args, "peer_four_words");

    // Note: CoreContext P2PNode doesn't currently expose disconnect_peer method
    // For now, just return success matching bridge behavior
    // TODO: Add disconnect_peer method to CoreContext when needed
    let _ = app; // Suppress unused warning until implemented
    success_result("Disconnected from peer")
}

/// Update Kanban board name or description
async fn execute_update_kanban_board(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let name = opt_str(&args, "name");
    let description = opt_str(&args, "description").map(Some);

    let cmd = Command::UpdateKanbanBoard {
        board_id,
        name,
        description,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            if let Some(Event::KanbanBoardUpdated { board_id, name, .. }) = events.first() {
                json_result(&json!({
                    "success": true,
                    "board_id": board_id,
                    "name": name
                }))
            } else {
                success_result("Board updated")
            }
        }
        Err(e) => error_result(&format!("Failed to update board: {}", e)),
    }
}

/// Delete a Kanban board
async fn execute_delete_kanban_board(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    let cmd = Command::DeleteKanbanBoard { board_id };

    match app.execute(cmd).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "message": "Board deleted"
        })),
        Err(e) => error_result(&format!("Failed to delete board: {}", e)),
    }
}

/// List all cards in a Kanban board with optional filters
async fn execute_list_kanban_cards(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = opt_str(&args, "column_id");
    let state = opt_str(&args, "state");
    let assignee_id = opt_str(&args, "assignee_id");
    let tag_id = opt_str(&args, "tag_id");

    let query = Query::ListKanbanCards {
        board_id,
        column_id,
        state,
        assignee_id,
        tag_id,
    };

    match app.query(query).await {
        Ok(QueryResponse::KanbanCards(cards)) => {
            let cards_json: Vec<Value> = cards
                .into_iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "column_id": c.column_id,
                        "title": c.title,
                        "description": c.description,
                        "position": c.position,
                        "assignee": c.assignee
                    })
                })
                .collect();
            json_result(&json!({ "cards": cards_json }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list cards: {}", e)),
    }
}

async fn execute_workspace_init(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let name = require_str!(args, "name");
    let description = opt_str(&args, "description");
    let board_name = args["board_name"]
        .as_str()
        .unwrap_or("Main Board")
        .to_string();
    let columns_arr = str_array_or_default(&args, "columns");
    let columns: Vec<String> = if columns_arr.is_empty() {
        vec![
            "To Do".to_string(),
            "In Progress".to_string(),
            "Done".to_string(),
        ]
    } else {
        columns_arr
    };

    let cmd = Command::CreateEntity {
        name: name.clone(),
        entity_type: EntityType::Project,
        description,
        initial_members: vec![],
    };

    let entity_id = match app.execute(cmd).await {
        Ok(events) => events.iter().find_map(|e| match e {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        }),
        Err(e) => return error_result(&format!("Failed to create project: {}", e.message)),
    };

    let entity_id = match entity_id {
        Some(id) => id,
        None => return error_result("Project created but no entity_id returned"),
    };

    let board_cmd = Command::CreateKanbanBoard {
        entity_id: entity_id.clone(),
        board_name: board_name.clone(),
        description: Some(format!("Default board for {}", name)),
    };

    let board_id = match app.execute(board_cmd).await {
        Ok(events) => events.iter().find_map(|e| match e {
            Event::KanbanBoardCreated { board_id, .. } => Some(board_id.clone()),
            _ => None,
        }),
        Err(e) => return error_result(&format!("Failed to create board: {}", e.message)),
    };

    let board_id = match board_id {
        Some(id) => id,
        None => return error_result("Board created but no board_id returned"),
    };

    let mut column_ids = Vec::new();
    for (position, column_name) in columns.iter().enumerate() {
        let col_cmd = Command::CreateKanbanColumn {
            board_id: board_id.clone(),
            column_name: column_name.clone(),
            position: Some(position as u32),
        };

        match app.execute(col_cmd).await {
            Ok(events) => {
                if let Some(col_id) = events.iter().find_map(|e| match e {
                    Event::KanbanColumnCreated { column_id, .. } => Some(column_id.clone()),
                    _ => None,
                }) {
                    column_ids.push(json!({
                        "id": col_id,
                        "name": column_name,
                        "position": position
                    }));
                }
            }
            Err(e) => {
                return error_result(&format!(
                    "Failed to create column '{}': {}",
                    column_name, e.message
                ));
            }
        }
    }

    json_result(&json!({
        "success": true,
        "workspace": {
            "entity_id": entity_id,
            "name": name,
            "entity_type": "project"
        },
        "board": {
            "id": board_id,
            "name": board_name
        },
        "columns": column_ids
    }))
}

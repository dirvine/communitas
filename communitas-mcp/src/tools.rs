// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Tool definitions
//!
//! Exposes Communitas commands and queries as MCP tools that AI agents can invoke.

use crate::presence::{PresenceOperations, PresenceStatus, PresenceSubscription, PresenceUpdate};
use crate::protocol::{Tool, ToolCallResult, ToolContent};
use base64::prelude::*;

use communitas_core::{
    app::CommunitasApp,
    command::{Command, DiskTypeArg, Event, Query, QueryResponse},
    conn_from_words,
    crdt::EntityType,
};
use communitas_ui_api::UnifiedEntityType;
use communitas_ui_api::drive::DiskType as UiDiskType;
use communitas_ui_service::UiServices;
use serde_json::{Value, json};
use tracing::warn;

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
        // Recovery tools (ADR-016) - pre-auth for identity creation/recovery
        Tool {
            name: "create_identity".to_string(),
            description: "Create a new identity with BIP39 recovery phrase. Returns mnemonic words that MUST be written down for backup.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "word_count": {
                        "type": "integer",
                        "description": "Number of mnemonic words (12, 15, 18, 21, or 24). Default: 24 for maximum security.",
                        "enum": [12, 15, 18, 21, 24],
                        "default": 24
                    },
                    "passphrase": {
                        "type": "string",
                        "description": "Optional additional passphrase (BIP39 '25th word') for extra security"
                    }
                }
            }),
        },
        Tool {
            name: "recover_identity".to_string(),
            description: "Recover an identity from a BIP39 recovery phrase. Use the same mnemonic words from initial backup.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mnemonic_words": {
                        "type": "string",
                        "description": "Space-separated BIP39 mnemonic words (12-24 words)"
                    },
                    "passphrase": {
                        "type": "string",
                        "description": "Optional passphrase if one was used during identity creation"
                    }
                },
                "required": ["mnemonic_words"]
            }),
        },
        Tool {
            name: "validate_mnemonic".to_string(),
            description: "Validate a BIP39 mnemonic phrase without deriving keys. Quick check for word validity and checksum.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mnemonic_words": {
                        "type": "string",
                        "description": "Space-separated BIP39 mnemonic words to validate"
                    }
                },
                "required": ["mnemonic_words"]
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
        // Audit log tools (security monitoring)
        Tool {
            name: "get_audit_log".to_string(),
            description: "Get recent security audit events. Events include login attempts, logouts, identity switches, and device changes. Sensitive fields are automatically redacted.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of events to return (default: 50, max: 100)",
                        "default": 50
                    },
                    "event_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["login", "logout", "failed_login", "identity_switch", "device_change", "recovery", "passkey_register", "passkey_auth", "session_refresh", "session_expired"]
                        },
                        "description": "Filter by specific event types (optional, returns all types if not specified)"
                    }
                }
            }),
        },
        Tool {
            name: "export_audit_log".to_string(),
            description: "Export security audit events within a date range. Useful for compliance reporting and security reviews.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "start_date": {
                        "type": "string",
                        "description": "Start date in ISO 8601 format (e.g., '2026-01-01T00:00:00Z')"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End date in ISO 8601 format (e.g., '2026-01-31T23:59:59Z')"
                    },
                    "event_types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["login", "logout", "failed_login", "identity_switch", "device_change", "recovery", "passkey_register", "passkey_auth", "session_refresh", "session_expired"]
                        },
                        "description": "Filter by specific event types (optional, returns all types if not specified)"
                    }
                },
                "required": ["start_date", "end_date"]
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
            name: "get_available_reactions".to_string(),
            description: "Get list of available standard reactions for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_id"]
            }),
        },
        // Thread listing tools (for AI agents)
        Tool {
            name: "list_threads".to_string(),
            description: "List all conversation threads for the authenticated user. Returns threads from both entities (channels, groups) and direct messages with contacts.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Maximum number of threads to return (default: 50, max: 100)"},
                    "filter": {
                        "type": "string",
                        "enum": ["all", "unread", "entities", "contacts"],
                        "description": "Filter threads by type (default: all)"
                    }
                }
            }),
        },
        Tool {
            name: "list_messages".to_string(),
            description: "Get messages from a specific thread with pagination support.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID to get messages from"},
                    "limit": {"type": "integer", "description": "Maximum number of messages to return (default: 50, max: 100)"},
                    "before": {"type": "integer", "description": "Unix timestamp in milliseconds - get messages before this time (for pagination)"}
                },
                "required": ["thread_id"]
            }),
        },
        // Phase 6.2 messaging tools (PLAN-37)
        Tool {
            name: "mark_thread_read".to_string(),
            description: "Mark all messages in a thread as read, resetting the unread count to zero.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID to mark as read"}
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "search_messages".to_string(),
            description: "Search for messages across all threads containing the given query text.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query text"},
                    "thread_id": {"type": "string", "description": "Optional: limit search to a specific thread"},
                    "limit": {"type": "integer", "description": "Maximum results to return (default: 20, max: 100)"}
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "pin_thread".to_string(),
            description: "Pin a thread to the top of the thread list for quick access.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID to pin"}
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "unpin_thread".to_string(),
            description: "Unpin a previously pinned thread.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID to unpin"}
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "get_pinned_threads".to_string(),
            description: "Get the list of pinned thread IDs.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "send_typing_indicator".to_string(),
            description: "Send a typing indicator to a thread, letting other participants know you are typing.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID where you are typing"}
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "get_typing_users".to_string(),
            description: "Get the list of users currently typing in a thread.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID to check"}
                },
                "required": ["thread_id"]
            }),
        },
        Tool {
            name: "get_pending_messages".to_string(),
            description: "Get messages queued for sending (offline queue).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "queue_offline_message".to_string(),
            description: "Queue a message for sending when network is available (offline queue).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "thread_id": {"type": "string", "description": "Thread ID to send to"},
                    "text": {"type": "string", "description": "Message text"},
                    "reply_to_id": {"type": "string", "description": "Message ID to reply to (optional)"}
                },
                "required": ["thread_id", "text"]
            }),
        },
        Tool {
            name: "retry_pending_messages".to_string(),
            description: "Retry sending all pending messages in the offline queue.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "cancel_pending_message".to_string(),
            description: "Cancel a pending message from the offline queue.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pending_id": {"type": "string", "description": "Pending message ID to cancel"}
                },
                "required": ["pending_id"]
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
        Tool {
            name: "create_directory".to_string(),
            description: "Create a directory on an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "Directory path to create"}
                },
                "required": ["entity_id", "disk_type", "path"]
            }),
        },
        Tool {
            name: "move_file".to_string(),
            description: "Move or rename a file/directory on an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "source_path": {"type": "string", "description": "Source path"},
                    "dest_path": {"type": "string", "description": "Destination path"}
                },
                "required": ["entity_id", "disk_type", "source_path", "dest_path"]
            }),
        },
        Tool {
            name: "copy_file".to_string(),
            description: "Copy a file/directory on an entity's virtual disk".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "source_path": {"type": "string", "description": "Source path"},
                    "dest_path": {"type": "string", "description": "Destination path"}
                },
                "required": ["entity_id", "disk_type", "source_path", "dest_path"]
            }),
        },
        Tool {
            name: "list_disks".to_string(),
            description: "List all virtual disks available for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "get_file_preview".to_string(),
            description: "Get a preview of a file (thumbnail for images, text excerpt for text files)".to_string(),
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
        // Streaming transfer tools
        Tool {
            name: "start_streaming_upload".to_string(),
            description: "Start a chunked streaming upload with progress tracking and resume support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "Destination path on disk"},
                    "local_path": {"type": "string", "description": "Local file path to upload"}
                },
                "required": ["entity_id", "disk_type", "path", "local_path"]
            }),
        },
        Tool {
            name: "start_streaming_download".to_string(),
            description: "Start a chunked streaming download with progress tracking and resume support".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path on disk"},
                    "local_path": {"type": "string", "description": "Local destination path"}
                },
                "required": ["entity_id", "disk_type", "path", "local_path"]
            }),
        },
        Tool {
            name: "resume_upload".to_string(),
            description: "Resume an interrupted upload from where it left off. Provide the same entity, disk, path, and local_path as the original upload.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "Destination path on disk"},
                    "local_path": {"type": "string", "description": "Local file path to upload"}
                },
                "required": ["entity_id", "disk_type", "path", "local_path"]
            }),
        },
        Tool {
            name: "resume_download".to_string(),
            description: "Resume an interrupted download from where it left off. Provide the same entity, disk, path, and local_path as the original download.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path on disk"},
                    "local_path": {"type": "string", "description": "Local destination path"}
                },
                "required": ["entity_id", "disk_type", "path", "local_path"]
            }),
        },
        Tool {
            name: "get_upload_progress".to_string(),
            description: "Get progress information for an active upload".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "upload_id": {"type": "string", "description": "Upload ID"}
                },
                "required": ["upload_id"]
            }),
        },
        Tool {
            name: "get_download_progress".to_string(),
            description: "Get progress information for an active download".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "download_id": {"type": "string", "description": "Download ID"}
                },
                "required": ["download_id"]
            }),
        },
        Tool {
            name: "cancel_upload".to_string(),
            description: "Cancel an active upload".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "upload_id": {"type": "string", "description": "Upload ID to cancel"}
                },
                "required": ["upload_id"]
            }),
        },
        Tool {
            name: "cancel_download".to_string(),
            description: "Cancel an active download".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "download_id": {"type": "string", "description": "Download ID to cancel"}
                },
                "required": ["download_id"]
            }),
        },
        // Share link tools
        Tool {
            name: "create_share_link".to_string(),
            description: "Create a shareable link for a file with optional expiry and password".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "path": {"type": "string", "description": "File path"},
                    "expires_in_hours": {"type": "integer", "description": "Optional: hours until link expires"},
                    "password": {"type": "string", "description": "Optional: password to access the link"},
                    "max_downloads": {"type": "integer", "description": "Optional: maximum number of downloads"}
                },
                "required": ["entity_id", "disk_type", "path"]
            }),
        },
        Tool {
            name: "revoke_share_link".to_string(),
            description: "Revoke a share link by its ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "link_id": {"type": "string", "description": "Share link ID to revoke"}
                },
                "required": ["link_id"]
            }),
        },
        Tool {
            name: "list_share_links".to_string(),
            description: "List all share links for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "get_file_share_links".to_string(),
            description: "Get all share links for a specific file".to_string(),
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
        // Offline staging tools
        Tool {
            name: "stage_upload".to_string(),
            description: "Stage a file for upload when offline. File will be uploaded when network is available.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID"},
                    "disk_type": {"type": "string", "enum": ["private", "public", "shared"], "description": "Disk type"},
                    "destination_path": {"type": "string", "description": "Destination path on disk"},
                    "local_path": {"type": "string", "description": "Local file path to stage"}
                },
                "required": ["entity_id", "disk_type", "destination_path", "local_path"]
            }),
        },
        Tool {
            name: "get_staged_upload".to_string(),
            description: "Get details of a staged upload by ID".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "upload_id": {"type": "string", "description": "Staged upload ID"}
                },
                "required": ["upload_id"]
            }),
        },
        Tool {
            name: "list_staged_uploads".to_string(),
            description: "List all staged uploads in the offline queue".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "get_staging_status".to_string(),
            description: "Get the current status of the offline staging queue".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "remove_staged_upload".to_string(),
            description: "Remove a file from the staging queue".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "upload_id": {"type": "string", "description": "Staged upload ID to remove"}
                },
                "required": ["upload_id"]
            }),
        },
        Tool {
            name: "retry_staged_upload".to_string(),
            description: "Retry a failed staged upload".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "upload_id": {"type": "string", "description": "Staged upload ID to retry"}
                },
                "required": ["upload_id"]
            }),
        },
        Tool {
            name: "resolve_staging_conflict".to_string(),
            description: "Resolve a conflict for a staged upload".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "upload_id": {"type": "string", "description": "Staged upload ID with conflict"},
                    "resolution": {"type": "string", "enum": ["keep_local", "keep_remote", "keep_both", "skip", "retry"], "description": "How to resolve the conflict"}
                },
                "required": ["upload_id", "resolution"]
            }),
        },
        Tool {
            name: "sync_staging_queue".to_string(),
            description: "Sync all pending staged uploads. Returns counts of uploaded and failed files.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "set_network_available".to_string(),
            description: "Set network availability status for the staging queue".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "available": {"type": "boolean", "description": "Whether network is available"}
                },
                "required": ["available"]
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
            description: "Start a voice or video call for an entity".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID (channel/group) to call"},
                    "video_enabled": {"type": "boolean", "description": "Enable video (default: false)", "default": false}
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
        Tool {
            name: "share_screen".to_string(),
            description: "Start or stop screen sharing in a call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "Call ID"},
                    "enabled": {"type": "boolean", "description": "Set true to start sharing, false to stop", "default": true}
                },
                "required": ["call_id"]
            }),
        },
        Tool {
            name: "toggle_mute".to_string(),
            description: "Toggle audio mute state in a call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "Call ID"},
                    "muted": {"type": "boolean", "description": "Set true to mute, false to unmute"}
                },
                "required": ["call_id", "muted"]
            }),
        },
        Tool {
            name: "toggle_video".to_string(),
            description: "Toggle video state in a call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "Call ID"},
                    "enabled": {"type": "boolean", "description": "Set true to enable video, false to disable"}
                },
                "required": ["call_id", "enabled"]
            }),
        },
        Tool {
            name: "get_call_status".to_string(),
            description: "Get current call status including mute/video state".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "Call ID"}
                },
                "required": ["call_id"]
            }),
        },
        Tool {
            name: "get_call_participants".to_string(),
            description: "Get list of participants in a call".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "call_id": {"type": "string", "description": "Call ID"}
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
            name: "get_connection_words".to_string(),
            description: "Get your connection words (external IP:port encoded as 4 memorable words). Share these out-of-band so others can connect to you for the first time.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "connect_by_words".to_string(),
            description: "Connect to a peer using their 4-word encoded address. After connecting, you'll receive their cryptographic identity packet.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "words": {"type": "string", "description": "Four words encoding the peer's external address (e.g., 'ocean forest moon star')"}
                },
                "required": ["words"]
            }),
        },
        // Peer Presence Tools (ADR-014: Network-wide peer discovery)
        Tool {
            name: "announce_presence".to_string(),
            description: "Broadcast your presence (connection words) to connected peers so they can cache your current address.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "query_presence".to_string(),
            description: "Find a peer's current address by their public key. Queries the network for the peer's presence record.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pubkey": {"type": "string", "description": "Public key of the peer to find (hex or base64 encoded)"}
                },
                "required": ["pubkey"]
            }),
        },
        Tool {
            name: "get_our_presence".to_string(),
            description: "Get your own presence record including your public key and current connection words.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        Tool {
            name: "get_cached_presence".to_string(),
            description: "Check if we have a cached presence record for a peer by their public key.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "pubkey": {"type": "string", "description": "Public key of the peer to look up (hex or base64 encoded)"}
                },
                "required": ["pubkey"]
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
            description: "List all contacts in your address book with optional presence info and filtering.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "include_presence": {
                        "type": "boolean",
                        "description": "Include online/offline status for each contact (default: true)"
                    },
                    "filter": {
                        "type": "string",
                        "enum": ["all", "online", "favorites"],
                        "description": "Filter contacts by status (default: all)"
                    }
                }
            }),
        },
        Tool {
            name: "get_contact_presence".to_string(),
            description: "Get detailed presence status for a specific contact including online status, last seen time, and current activity.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "contact_id": {
                        "type": "string",
                        "description": "Contact ID to query presence for"
                    }
                },
                "required": ["contact_id"]
            }),
        },
        Tool {
            name: "set_my_presence".to_string(),
            description: "Set your own global presence status. This affects how others see your online status.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["online", "away", "busy", "offline"],
                        "description": "Presence status to set"
                    },
                    "status_message": {
                        "type": "string",
                        "description": "Optional status message (e.g., 'In a meeting')"
                    }
                },
                "required": ["status"]
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
        // ========== Canvas Tools ==========
        Tool {
            name: "canvas_get_snapshot".to_string(),
            description: "Get the current canvas snapshot including all elements, viewport, and view settings".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "canvas_add_text".to_string(),
            description: "Add a text element to the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "content": {"type": "string", "description": "Text content to display"},
                    "x": {"type": "number", "description": "X position in pixels"},
                    "y": {"type": "number", "description": "Y position in pixels"},
                    "font_size": {"type": "number", "description": "Font size in pixels", "default": 16},
                    "color": {"type": "string", "description": "Text color (CSS color string)", "default": "#000000"}
                },
                "required": ["entity_id", "content", "x", "y"]
            }),
        },
        Tool {
            name: "canvas_add_image".to_string(),
            description: "Add an image element to the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "src": {"type": "string", "description": "Image source URL or data URI"},
                    "x": {"type": "number", "description": "X position in pixels"},
                    "y": {"type": "number", "description": "Y position in pixels"},
                    "width": {"type": "number", "description": "Image width in pixels"},
                    "height": {"type": "number", "description": "Image height in pixels"}
                },
                "required": ["entity_id", "src", "x", "y", "width", "height"]
            }),
        },
        Tool {
            name: "canvas_add_chart".to_string(),
            description: "Add a chart element to the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "chart_type": {"type": "string", "description": "Chart type (e.g., 'bar', 'line', 'pie')"},
                    "data": {"type": "object", "description": "Chart data as JSON object"},
                    "x": {"type": "number", "description": "X position in pixels"},
                    "y": {"type": "number", "description": "Y position in pixels"},
                    "width": {"type": "number", "description": "Chart width in pixels"},
                    "height": {"type": "number", "description": "Chart height in pixels"}
                },
                "required": ["entity_id", "chart_type", "data", "x", "y", "width", "height"]
            }),
        },
        Tool {
            name: "canvas_remove_element".to_string(),
            description: "Remove an element from the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "element_id": {"type": "string", "description": "ID of the element to remove"}
                },
                "required": ["entity_id", "element_id"]
            }),
        },
        Tool {
            name: "canvas_update_transform".to_string(),
            description: "Update an element's position, size, rotation, and z-index".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "element_id": {"type": "string", "description": "ID of the element to update"},
                    "x": {"type": "number", "description": "X position in pixels"},
                    "y": {"type": "number", "description": "Y position in pixels"},
                    "width": {"type": "number", "description": "Width in pixels"},
                    "height": {"type": "number", "description": "Height in pixels"},
                    "rotation": {"type": "number", "description": "Rotation in radians", "default": 0},
                    "z_index": {"type": "integer", "description": "Z-index for layering", "default": 0}
                },
                "required": ["entity_id", "element_id", "x", "y", "width", "height"]
            }),
        },
        Tool {
            name: "canvas_select_element".to_string(),
            description: "Select an element on the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "element_id": {"type": "string", "description": "ID of the element to select"}
                },
                "required": ["entity_id", "element_id"]
            }),
        },
        Tool {
            name: "canvas_deselect_all".to_string(),
            description: "Deselect all elements on the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "canvas_set_viewport".to_string(),
            description: "Set the viewport dimensions of the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "width": {"type": "number", "description": "Viewport width in pixels"},
                    "height": {"type": "number", "description": "Viewport height in pixels"}
                },
                "required": ["entity_id", "width", "height"]
            }),
        },
        Tool {
            name: "canvas_set_view".to_string(),
            description: "Set zoom and pan for the canvas view".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "zoom": {"type": "number", "description": "Zoom level (1.0 = 100%)"},
                    "pan_x": {"type": "number", "description": "Pan offset X in pixels"},
                    "pan_y": {"type": "number", "description": "Pan offset Y in pixels"}
                },
                "required": ["entity_id", "zoom", "pan_x", "pan_y"]
            }),
        },
        Tool {
            name: "canvas_clear".to_string(),
            description: "Clear all elements from the canvas".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "canvas_export".to_string(),
            description: "Export the canvas scene as JSON".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"}
                },
                "required": ["entity_id"]
            }),
        },
        Tool {
            name: "canvas_import".to_string(),
            description: "Import a canvas scene from JSON".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "json": {"type": "string", "description": "JSON string containing the scene data"}
                },
                "required": ["entity_id", "json"]
            }),
        },
        Tool {
            name: "canvas_element_at".to_string(),
            description: "Get the element at the specified canvas coordinates".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "entity_id": {"type": "string", "description": "Entity ID for the canvas context"},
                    "x": {"type": "number", "description": "X coordinate in canvas space"},
                    "y": {"type": "number", "description": "Y coordinate in canvas space"}
                },
                "required": ["entity_id", "x", "y"]
            }),
        },
    ]);

    tools
}

/// Execute a tool call
///
/// # Arguments
/// * `app` - The CommunitasApp for direct core operations (legacy path)
/// * `services` - The UiServices for service-layer operations (preferred path)
/// * `name` - Tool name to execute
/// * `args` - Optional JSON arguments
///
/// Tool implementations are being migrated from using `app` directly to using
/// `services` for MCP-Dioxus parity (see PLAN-31).
pub async fn call_tool(
    app: &CommunitasApp,
    services: &UiServices,
    name: &str,
    args: Option<Value>,
) -> ToolCallResult {
    let args = args.unwrap_or(json!({}));

    // Try each category dispatcher in turn
    // Note: health_check and core_status are handled as pre-auth tools in server.rs

    if let Some(result) = dispatch_session_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_entity_tools(app, name, &args).await {
        return result;
    }
    // Messaging tools use UiServices for MCP-Dioxus parity (PLAN-31)
    if let Some(result) = dispatch_message_tools(services, app, name, &args).await {
        return result;
    }
    // Kanban tools use UiServices for MCP-Dioxus parity (PLAN-31)
    if let Some(result) = dispatch_kanban_tools(services, name, &args).await {
        return result;
    }
    // Canvas tools use UiServices for MCP-Dioxus parity (PLAN-31)
    if let Some(result) = dispatch_canvas_tools(services, name, &args).await {
        return result;
    }
    // Drive/file tools use UiServices for MCP-Dioxus parity (PLAN-31)
    if let Some(result) = dispatch_file_tools(services, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_contact_tools(app, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_network_tools(app, name, &args).await {
        return result;
    }
    // Call/social tools use UiServices for MCP-Dioxus parity (PLAN-31)
    if let Some(result) = dispatch_social_tools(services, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_recovery_tools(name, &args).await {
        return result;
    }
    // Audit log tools use UiServices for MCP-Dioxus parity
    if let Some(result) = dispatch_audit_tools(services, name, &args).await {
        return result;
    }
    if let Some(result) = dispatch_misc_tools(app, name, &args).await {
        return result;
    }

    error_result(&format!("Unknown tool: {name}"))
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
    services: &UiServices,
    app: &CommunitasApp,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Message CRUD - migrated to use UiServices (PLAN-31)
        "send_message" => Some(execute_send_message(services, args.clone()).await),
        "delete_message" => Some(execute_delete_message(services, args.clone()).await),
        "edit_message" => Some(execute_edit_message(services, args.clone()).await),
        "get_messages" => Some(execute_get_messages(services, args.clone()).await),
        // Reactions - migrated to use UiServices (PLAN-31)
        "add_reaction" => Some(execute_add_reaction(services, args.clone()).await),
        "remove_reaction" => Some(execute_remove_reaction(services, args.clone()).await),
        // Reactions - not yet migrated, still use app directly
        "get_reactions" => Some(execute_get_reactions(app, args.clone()).await),
        "get_available_reactions" => Some(execute_get_available_reactions(app, args.clone()).await),
        // Thread operations - not yet migrated
        "create_thread" => Some(execute_create_thread(app, args.clone()).await),
        "get_thread_messages" => Some(execute_get_thread_messages(app, args.clone()).await),
        // Thread listing - migrated to use UiServices (PLAN-31)
        "list_threads" => Some(execute_list_threads(services, args.clone()).await),
        "list_messages" => Some(execute_list_messages(app, args.clone()).await),
        // Phase 6.2 messaging tools (PLAN-37)
        "mark_thread_read" => Some(execute_mark_thread_read(services, args.clone()).await),
        "search_messages" => Some(execute_search_messages(services, args.clone()).await),
        "pin_thread" => Some(execute_pin_thread(services, args.clone()).await),
        "unpin_thread" => Some(execute_unpin_thread(services, args.clone()).await),
        "get_pinned_threads" => Some(execute_get_pinned_threads(services).await),
        "send_typing_indicator" => {
            Some(execute_send_typing_indicator(services, args.clone()).await)
        }
        "get_typing_users" => Some(execute_get_typing_users(services, args.clone()).await),
        "get_pending_messages" => Some(execute_get_pending_messages(services).await),
        "queue_offline_message" => {
            Some(execute_queue_offline_message(services, args.clone()).await)
        }
        "retry_pending_messages" => Some(execute_retry_pending_messages(services).await),
        "cancel_pending_message" => {
            Some(execute_cancel_pending_message(services, args.clone()).await)
        }
        _ => None,
    }
}

async fn dispatch_kanban_tools(
    services: &UiServices,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Board operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "create_kanban_board" => Some(execute_create_board(services, args.clone()).await),
        "get_kanban_board" => Some(execute_get_kanban_board(services, args.clone()).await),
        "update_kanban_board" => Some(execute_update_kanban_board(services, args.clone()).await),
        "delete_kanban_board" => Some(execute_delete_kanban_board(services, args.clone()).await),
        "list_kanban_boards" => Some(execute_list_kanban_boards(services, args.clone()).await),
        // Column operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "create_kanban_column" => Some(execute_create_column(services, args.clone()).await),
        "get_kanban_column" => Some(execute_get_kanban_column(services, args.clone()).await),
        "update_kanban_column" => Some(execute_update_kanban_column(services, args.clone()).await),
        "delete_kanban_column" => Some(execute_delete_kanban_column(services, args.clone()).await),
        "move_kanban_column" => Some(execute_move_kanban_column(services, args.clone()).await),
        "list_kanban_columns" => Some(execute_list_kanban_columns(services, args.clone()).await),
        // Card operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "create_kanban_card" => Some(execute_create_card(services, args.clone()).await),
        "get_kanban_card" => Some(execute_get_kanban_card(services, args.clone()).await),
        "update_kanban_card" => Some(execute_update_kanban_card(services, args.clone()).await),
        "delete_kanban_card" => Some(execute_delete_kanban_card(services, args.clone()).await),
        "move_kanban_card" => Some(execute_move_card(services, args.clone()).await),
        "list_kanban_cards" => Some(execute_list_kanban_cards(services, args.clone()).await),
        "change_card_state" => Some(execute_change_card_state(services, args.clone()).await),
        // Assignment operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "assign_user" => Some(execute_assign_user(services, args.clone()).await),
        "unassign_user" => Some(execute_unassign_user(services, args.clone()).await),
        // Tag operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "create_kanban_tag" => Some(execute_create_kanban_tag(services, args.clone()).await),
        "list_kanban_tags" => Some(execute_list_kanban_tags(services, args.clone()).await),
        "tag_card" => Some(execute_tag_card(services, args.clone()).await),
        "untag_card" => Some(execute_untag_card(services, args.clone()).await),
        // Step operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "add_step" => Some(execute_add_step(services, args.clone()).await),
        "get_step" => Some(execute_get_step(services, args.clone()).await),
        "toggle_step" => Some(execute_toggle_step(services, args.clone()).await),
        "delete_step" => Some(execute_delete_step(services, args.clone()).await),
        // Comment operations - use KanbanService for MCP-Dioxus parity (PLAN-31)
        "add_comment" => Some(execute_add_comment(services, args.clone()).await),
        "list_comments" => Some(execute_list_comments(services, args.clone()).await),
        "delete_comment" => Some(execute_delete_comment(services, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_canvas_tools(
    services: &UiServices,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        // Canvas tools - use CanvasService for MCP-Dioxus parity (PLAN-31)
        "canvas_get_snapshot" => Some(execute_canvas_get_snapshot(services, args.clone()).await),
        "canvas_add_text" => Some(execute_canvas_add_text(services, args.clone()).await),
        "canvas_add_image" => Some(execute_canvas_add_image(services, args.clone()).await),
        "canvas_add_chart" => Some(execute_canvas_add_chart(services, args.clone()).await),
        "canvas_remove_element" => {
            Some(execute_canvas_remove_element(services, args.clone()).await)
        }
        "canvas_update_transform" => {
            Some(execute_canvas_update_transform(services, args.clone()).await)
        }
        "canvas_select_element" => {
            Some(execute_canvas_select_element(services, args.clone()).await)
        }
        "canvas_deselect_all" => Some(execute_canvas_deselect_all(services, args.clone()).await),
        "canvas_set_viewport" => Some(execute_canvas_set_viewport(services, args.clone()).await),
        "canvas_set_view" => Some(execute_canvas_set_view(services, args.clone()).await),
        "canvas_clear" => Some(execute_canvas_clear(services, args.clone()).await),
        "canvas_export" => Some(execute_canvas_export(services, args.clone()).await),
        "canvas_import" => Some(execute_canvas_import(services, args.clone()).await),
        "canvas_element_at" => Some(execute_canvas_element_at(services, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_file_tools(
    services: &UiServices,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        "write_file" => Some(execute_write_file(services, args.clone()).await),
        "read_file" => Some(execute_read_file(services, args.clone()).await),
        "delete_file" => Some(execute_delete_file(services, args.clone()).await),
        "list_files" => Some(execute_list_files(services, args.clone()).await),
        "get_disk_stats" => Some(execute_get_disk_stats(services, args.clone()).await),
        // Directory operations
        "create_directory" => Some(execute_create_directory(services, args.clone()).await),
        "move_file" => Some(execute_move_file(services, args.clone()).await),
        "copy_file" => Some(execute_copy_file(services, args.clone()).await),
        "list_disks" => Some(execute_list_disks(services, args.clone()).await),
        "get_file_preview" => Some(execute_get_file_preview(services, args.clone()).await),
        // Streaming transfer operations
        "start_streaming_upload" => {
            Some(execute_start_streaming_upload(services, args.clone()).await)
        }
        "start_streaming_download" => {
            Some(execute_start_streaming_download(services, args.clone()).await)
        }
        "resume_upload" => Some(execute_resume_upload(services, args.clone()).await),
        "resume_download" => Some(execute_resume_download(services, args.clone()).await),
        "get_upload_progress" => Some(execute_get_upload_progress(services, args.clone()).await),
        "get_download_progress" => {
            Some(execute_get_download_progress(services, args.clone()).await)
        }
        "cancel_upload" => Some(execute_cancel_upload(services, args.clone()).await),
        "cancel_download" => Some(execute_cancel_download(services, args.clone()).await),
        // Share link operations
        "create_share_link" => Some(execute_create_share_link(services, args.clone()).await),
        "revoke_share_link" => Some(execute_revoke_share_link(services, args.clone()).await),
        "list_share_links" => Some(execute_list_share_links(services, args.clone()).await),
        "get_file_share_links" => {
            Some(execute_get_file_share_links(services, args.clone()).await)
        }
        // Offline staging operations
        "stage_upload" => Some(execute_stage_upload(services, args.clone()).await),
        "get_staged_upload" => Some(execute_get_staged_upload(services, args.clone()).await),
        "list_staged_uploads" => Some(execute_list_staged_uploads(services).await),
        "get_staging_status" => Some(execute_get_staging_status(services).await),
        "remove_staged_upload" => {
            Some(execute_remove_staged_upload(services, args.clone()).await)
        }
        "retry_staged_upload" => {
            Some(execute_retry_staged_upload(services, args.clone()).await)
        }
        "resolve_staging_conflict" => {
            Some(execute_resolve_staging_conflict(services, args.clone()).await)
        }
        "sync_staging_queue" => Some(execute_sync_staging_queue(services).await),
        "set_network_available" => {
            Some(execute_set_network_available(services, args.clone()).await)
        }
        // Media operations - still use app directly for now (no service equivalent yet)
        "upload_with_metadata" => {
            Some(execute_upload_with_metadata(services.drive().app().as_ref(), args.clone()).await)
        }
        "get_media_metadata" => {
            Some(execute_get_media_metadata(services.drive().app().as_ref(), args.clone()).await)
        }
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
        "list_contacts" => Some(execute_list_contacts(app, args.clone()).await),
        "get_contact_presence" => Some(execute_get_contact_presence(app, args.clone()).await),
        "set_my_presence" => Some(execute_set_my_presence(app, args.clone()).await),
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
        "network_status" => Some(execute_network_status(app).await),
        "network_peers" => Some(execute_network_peers(app).await),
        "network_request_external_address" => Some(execute_request_external_address(app).await),
        "get_connection_words" => Some(execute_get_connection_words(app).await),
        "connect_by_words" => Some(execute_connect_by_words(app, args.clone()).await),
        // Peer Presence tools
        "announce_presence" => Some(execute_announce_presence(app).await),
        "query_presence" => Some(execute_query_presence(app, args.clone()).await),
        "get_our_presence" => Some(execute_get_our_presence(app).await),
        "get_cached_presence" => Some(execute_get_cached_presence(app, args.clone()).await),
        _ => None,
    }
}

async fn dispatch_social_tools(
    services: &UiServices,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    // Get app reference for presence tools (not yet migrated to UiServices)
    let app = services.call().app();
    match name {
        // Presence (still use app directly - migrate in future)
        "set_presence" => Some(execute_set_presence(&app, args.clone()).await),
        "get_presence" => Some(execute_get_presence(&app, args.clone()).await),
        "subscribe_to_presence" => Some(execute_subscribe_to_presence(&app, args.clone()).await),
        // WebRTC calls - use CallService for MCP-Dioxus parity
        "start_voice_call" => Some(execute_start_voice_call(services, args.clone()).await),
        "join_call" => Some(execute_join_call(services, args.clone()).await),
        "end_call" => Some(execute_end_call(services, args.clone()).await),
        "share_screen" => Some(execute_share_screen(services, args.clone()).await),
        "toggle_mute" => Some(execute_toggle_mute(services, args.clone()).await),
        "toggle_video" => Some(execute_toggle_video(services, args.clone()).await),
        "get_call_status" => Some(execute_get_call_status(services, args.clone()).await),
        "get_call_participants" => {
            Some(execute_get_call_participants(services, args.clone()).await)
        }
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

/// Parse disk type string to UI DiskType for DriveService calls.
fn parse_ui_disk_type(s: &str) -> Option<UiDiskType> {
    match s.to_lowercase().as_str() {
        "private" => Some(UiDiskType::Private),
        "public" => Some(UiDiskType::Public),
        "shared" => Some(UiDiskType::Shared),
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
/// Extract a string argument with empty default, logging when type mismatches occur
fn str_or_default(args: &Value, field: &str) -> String {
    match args.get(field) {
        Some(v) if !v.is_null() => match v.as_str() {
            Some(s) => s.to_string(),
            None => {
                warn!(field = field, "field is not a string, using empty default");
                String::new()
            }
        },
        _ => String::new(),
    }
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

/// Require a UI disk type for DriveService calls.
macro_rules! require_ui_disk_type {
    ($args:expr) => {
        match $args["disk_type"].as_str().and_then(parse_ui_disk_type) {
            Some(t) => t,
            None => return error_result("Invalid or missing disk_type"),
        }
    };
}

/// Extract a string array with empty default, logging when non-strings are dropped
fn str_array_or_default(args: &Value, field: &str) -> Vec<String> {
    match args.get(field) {
        Some(v) => match v.as_array() {
            Some(arr) => {
                let mut dropped_count = 0;
                let result: Vec<String> = arr
                    .iter()
                    .filter_map(|elem| {
                        if let Some(s) = elem.as_str() {
                            Some(s.to_string())
                        } else {
                            dropped_count += 1;
                            None
                        }
                    })
                    .collect();
                if dropped_count > 0 {
                    warn!(
                        field = field,
                        dropped_count = dropped_count,
                        "dropped non-string elements from array"
                    );
                }
                result
            }
            None if !v.is_null() => {
                warn!(field = field, "field is not an array, using empty default");
                Vec::new()
            }
            _ => Vec::new(),
        },
        None => Vec::new(),
    }
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

/// Send a message via UiServices MessagingService for MCP-Dioxus parity.
async fn execute_send_message(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = str_or_default(&args, "entity_id");
    let text = str_or_default(&args, "text");
    let reply_to = opt_str(&args, "reply_to_id");

    // Use MessagingService for parity with Dioxus UI
    match services
        .messaging()
        .send_message(&thread_id, &text, reply_to.as_deref())
        .await
    {
        Ok(message) => {
            let result = json!({
                "success": true,
                "message": "Message sent successfully",
                "id": message.id
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to send message: {}", e)),
    }
}

/// Delete a message via UiServices MessagingService for MCP-Dioxus parity.
async fn execute_delete_message(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = str_or_default(&args, "entity_id");
    let message_id = str_or_default(&args, "message_id");

    // Use MessagingService for parity with Dioxus UI
    match services
        .messaging()
        .delete_message(&thread_id, &message_id)
        .await
    {
        Ok(()) => success_result("Message deleted successfully"),
        Err(e) => error_result(&format!("Failed to delete message: {}", e)),
    }
}

/// Edit a message via UiServices MessagingService for MCP-Dioxus parity.
async fn execute_edit_message(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = str_or_default(&args, "entity_id");
    let message_id = str_or_default(&args, "message_id");
    let new_text = str_or_default(&args, "new_text");

    // Use MessagingService for parity with Dioxus UI
    match services
        .messaging()
        .edit_message(&thread_id, &message_id, &new_text)
        .await
    {
        Ok(message) => {
            let result = json!({
                "success": true,
                "message": "Message edited successfully",
                "edited": message.edited
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to edit message: {}", e)),
    }
}

/// Add a reaction via UiServices MessagingService for MCP-Dioxus parity.
async fn execute_add_reaction(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = str_or_default(&args, "entity_id");
    let message_id = str_or_default(&args, "message_id");
    let emoji = str_or_default(&args, "emoji");

    // Use MessagingService for parity with Dioxus UI
    match services
        .messaging()
        .add_reaction(&thread_id, &message_id, &emoji)
        .await
    {
        Ok(()) => {
            let result = json!({
                "success": true,
                "message": "Reaction added successfully",
                "emoji": emoji
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to add reaction: {}", e)),
    }
}

/// Remove a reaction via UiServices MessagingService for MCP-Dioxus parity.
async fn execute_remove_reaction(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = str_or_default(&args, "entity_id");
    let message_id = str_or_default(&args, "message_id");
    let emoji = str_or_default(&args, "emoji");

    // Use MessagingService for parity with Dioxus UI
    match services
        .messaging()
        .remove_reaction(&thread_id, &message_id, &emoji)
        .await
    {
        Ok(()) => {
            let result = json!({
                "success": true,
                "message": "Reaction removed successfully",
                "emoji": emoji
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to remove reaction: {}", e)),
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
        Err(e) => error_result(&format!("Failed to get reactions: {e}")),
    }
}

async fn execute_get_available_reactions(_app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    // Return standard reactions for this entity.
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

    json_result(&json!({
        "entity_id": entity_id,
        "reactions": standard_reactions
    }))
}

async fn execute_create_board(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let board_name = str_or_default(&args, "board_name");
    let template = opt_str(&args, "description"); // Use description as template for now

    match services
        .kanban()
        .create_board(&entity_id, &board_name, template.as_deref())
        .await
    {
        Ok(board) => json_result(&json!({
            "success": true,
            "message": "Kanban board created successfully",
            "id": board.id
        })),
        Err(e) => error_result(&format!("Failed to create board: {e}")),
    }
}

async fn execute_create_column(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = str_or_default(&args, "board_id");
    let column_name = str_or_default(&args, "column_name");
    let position = opt_u32(&args, "position").unwrap_or(0);

    match services
        .kanban()
        .create_column(&board_id, &column_name, position)
        .await
    {
        Ok(col) => json_result(&json!({
            "success": true,
            "message": "Kanban column created successfully",
            "id": col.id
        })),
        Err(e) => error_result(&format!("Failed to create column: {e}")),
    }
}

async fn execute_create_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = str_or_default(&args, "board_id");
    let column_id = str_or_default(&args, "column_id");
    let title = str_or_default(&args, "title");

    match services
        .kanban()
        .create_card(&board_id, &column_id, &title)
        .await
    {
        Ok(card) => json_result(&json!({
            "success": true,
            "message": "Card created successfully",
            "id": card.id
        })),
        Err(e) => error_result(&format!("Failed to create card: {e}")),
    }
}

async fn execute_move_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = str_or_default(&args, "board_id");
    let card_id = str_or_default(&args, "card_id");
    let target_column_id = str_or_default(&args, "target_column_id");
    let position = opt_u32(&args, "position").unwrap_or(0);

    match services
        .kanban()
        .move_card(&board_id, &card_id, &target_column_id, position)
        .await
    {
        Ok(()) => success_result("Card moved successfully"),
        Err(e) => error_result(&format!("Failed to move card: {e}")),
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

async fn execute_write_file(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = str_or_default(&args, "path");
    let content_str = match args["content"].as_str() {
        Some(s) => s,
        None => return error_result("content is required for write_file"),
    };
    let data = content_str.as_bytes();

    match services
        .drive()
        .write_file(&entity_id, disk_type, &path, data)
        .await
    {
        Ok(entry) => json_result(&json!({
            "success": true,
            "message": "File written successfully",
            "path": entry.path,
            "size_bytes": entry.size_bytes
        })),
        Err(e) => error_result(&format!("Failed to write file: {}", e)),
    }
}

async fn execute_read_file(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = str_or_default(&args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = str_or_default(&args, "path");

    match services
        .drive()
        .read_file(&entity_id, disk_type, &path)
        .await
    {
        Ok(content) => match String::from_utf8(content.clone()) {
            Ok(text) => json_result(&json!({"content": text})),
            Err(_) => {
                // Binary content - return as base64
                let encoded = BASE64_STANDARD.encode(&content);
                json_result(&json!({"content_base64": encoded}))
            }
        },
        Err(e) => error_result(&format!("Failed to read file: {}", e)),
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

/// Get messages via UiServices MessagingService for MCP-Dioxus parity.
async fn execute_get_messages(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = str_or_default(&args, "entity_id");
    let limit = args["limit"].as_u64().unwrap_or(50).min(100) as usize;
    let before = args["before"].as_u64();

    // Use MessagingService for parity with Dioxus UI
    match services
        .messaging()
        .get_messages(&thread_id, limit, before)
        .await
    {
        Ok(messages) => {
            let list: Vec<Value> = messages
                .iter()
                .map(|m| {
                    // Manually serialize reactions since MessageReaction doesn't impl Serialize
                    let reactions: Vec<Value> = m
                        .reactions
                        .iter()
                        .map(|r| {
                            json!({
                                "emoji": r.emoji,
                                "count": r.count,
                                "reacted_by_me": r.reacted_by_me
                            })
                        })
                        .collect();
                    json!({
                        "id": m.id,
                        "sender_name": m.sender_name,
                        "text": m.text,
                        "timestamp": m.timestamp,
                        "reply_to_id": m.reply_to_id,
                        "edited": m.edited,
                        "reactions": reactions
                    })
                })
                .collect();
            json_result(&json!({"messages": list}))
        }
        Err(e) => error_result(&format!("Failed to get messages: {}", e)),
    }
}

/// List all conversation threads via UiServices MessagingService for MCP-Dioxus parity.
///
/// Returns threads matching the `communitas-ui-api::ThreadSummary` format.
#[tracing::instrument(skip(services), name = "mcp.tools.list_threads")]
async fn execute_list_threads(services: &UiServices, args: Value) -> ToolCallResult {
    let limit = args["limit"].as_u64().unwrap_or(50).min(100) as usize;
    let filter = args["filter"].as_str().unwrap_or("all");

    // Use MessagingService for parity with Dioxus UI
    match services.messaging().list_threads().await {
        Ok(threads) => {
            // Apply filter if specified
            let filtered: Vec<Value> = threads
                .iter()
                .filter(|t| {
                    match filter {
                        "entities" => t.entity_id.is_some(),
                        "contacts" => t.contact_id.is_some(),
                        "unread" => t.unread_count > 0,
                        _ => true, // "all"
                    }
                })
                .take(limit)
                .map(|t| {
                    // Convert entity_type since UnifiedEntityType doesn't impl Serialize
                    let entity_type_str = t.entity_type.as_ref().map(|et| match et {
                        UnifiedEntityType::Organization => "organization",
                        UnifiedEntityType::Project => "project",
                        UnifiedEntityType::Group => "group",
                        UnifiedEntityType::Channel => "channel",
                        UnifiedEntityType::Person => "person",
                    });
                    json!({
                        "thread_id": t.thread_id,
                        "entity_id": t.entity_id,
                        "entity_type": entity_type_str,
                        "contact_id": t.contact_id,
                        "display_name": t.display_name,
                        "last_message_preview": t.last_message_preview,
                        "last_message_timestamp": t.last_message_timestamp,
                        "unread_count": t.unread_count,
                        "is_muted": t.is_muted
                    })
                })
                .collect();

            json_result(&json!({
                "threads": filtered,
                "total_count": filtered.len()
            }))
        }
        Err(e) => error_result(&format!("Failed to list threads: {}", e)),
    }
}

/// Get messages from a specific thread with pagination support.
///
/// Thread IDs are formatted as "entity:{entity_id}" or "contact:{contact_id}".
#[tracing::instrument(skip(app), name = "mcp.tools.list_messages")]
async fn execute_list_messages(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };
    let limit = args["limit"].as_u64().unwrap_or(50).min(100) as usize;
    let before_timestamp = args["before"].as_u64();

    // Parse thread_id to determine if entity or contact
    let (entity_id, _is_contact) = if let Some(stripped) = thread_id.strip_prefix("entity:") {
        (stripped.to_string(), false)
    } else if let Some(stripped) = thread_id.strip_prefix("contact:") {
        (stripped.to_string(), true)
    } else {
        // Fallback: treat as entity_id directly for backwards compatibility
        (thread_id.to_string(), false)
    };

    // Query messages from the entity
    let query = Query::GetEntityMessages {
        entity_id: entity_id.clone(),
    };

    match app.query(query).await {
        Ok(QueryResponse::Messages(mut messages)) => {
            // Apply before filter if provided (convert u64 to i64 for comparison)
            if let Some(before_ts) = before_timestamp {
                let before_ts_i64 = before_ts as i64;
                messages.retain(|m| m.timestamp < before_ts_i64);
            }

            // Sort by timestamp descending and limit
            messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            messages.truncate(limit);

            // Reverse to get chronological order (oldest first)
            messages.reverse();

            let list: Vec<Value> = messages
                .iter()
                .map(|m| {
                    json!({
                        "id": m.id,
                        "thread_id": thread_id,
                        "sender_id": m.author,
                        "sender_name": m.author, // TODO: resolve display name
                        "text": m.text,
                        "timestamp": m.timestamp,
                        "reply_to_id": m.reply_to_id,
                        "edited": false,
                        "reactions": []
                    })
                })
                .collect();

            json_result(&json!({
                "messages": list,
                "has_more": messages.len() == limit
            }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get messages: {}", e.message)),
    }
}

// Phase 6.2 messaging tools (PLAN-37)

/// Mark all messages in a thread as read.
#[tracing::instrument(skip(services), name = "mcp.tools.mark_thread_read")]
async fn execute_mark_thread_read(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };

    match services.messaging().mark_thread_read(thread_id).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "thread_id": thread_id
        })),
        Err(e) => error_result(&format!("Failed to mark thread as read: {}", e)),
    }
}

/// Search for messages across threads.
#[tracing::instrument(skip(services), name = "mcp.tools.search_messages")]
async fn execute_search_messages(services: &UiServices, args: Value) -> ToolCallResult {
    let query = match args["query"].as_str() {
        Some(q) => q,
        None => return error_result("query is required"),
    };
    let thread_id = args["thread_id"].as_str();
    let limit = args["limit"].as_u64().unwrap_or(20).min(100) as usize;

    match services
        .messaging()
        .search_messages(query, thread_id, limit)
        .await
    {
        Ok(results) => {
            let limited_results: Vec<Value> = results
                .iter()
                .take(limit)
                .map(|r| {
                    json!({
                        "message": {
                            "id": r.message.id,
                            "thread_id": r.message.thread_id,
                            "sender_id": r.message.sender_id,
                            "sender_name": r.message.sender_name,
                            "text": r.message.text,
                            "timestamp": r.message.timestamp,
                            "edited": r.message.edited
                        },
                        "thread_id": r.thread_id,
                        "thread_name": r.thread_name,
                        "match_count": r.match_count,
                        "match_excerpt": r.match_excerpt
                    })
                })
                .collect();

            json_result(&json!({
                "results": limited_results,
                "total_count": limited_results.len()
            }))
        }
        Err(e) => error_result(&format!("Search failed: {}", e)),
    }
}

/// Pin a thread to the top of the list.
#[tracing::instrument(skip(services), name = "mcp.tools.pin_thread")]
async fn execute_pin_thread(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };

    match services.messaging().pin_thread(thread_id).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "thread_id": thread_id,
            "pinned": true
        })),
        Err(e) => error_result(&format!("Failed to pin thread: {}", e)),
    }
}

/// Unpin a thread.
#[tracing::instrument(skip(services), name = "mcp.tools.unpin_thread")]
async fn execute_unpin_thread(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };

    match services.messaging().unpin_thread(thread_id).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "thread_id": thread_id,
            "pinned": false
        })),
        Err(e) => error_result(&format!("Failed to unpin thread: {}", e)),
    }
}

/// Get the list of pinned thread IDs.
#[tracing::instrument(skip(services), name = "mcp.tools.get_pinned_threads")]
async fn execute_get_pinned_threads(services: &UiServices) -> ToolCallResult {
    let pinned = services.messaging().get_pinned_threads();
    json_result(&json!({
        "pinned_threads": pinned
    }))
}

/// Send a typing indicator to a thread.
#[tracing::instrument(skip(services), name = "mcp.tools.send_typing_indicator")]
async fn execute_send_typing_indicator(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };

    // Send typing indicator (is_typing = true)
    match services
        .messaging()
        .send_typing_indicator(thread_id, true)
        .await
    {
        Ok(_) => json_result(&json!({
            "success": true,
            "thread_id": thread_id
        })),
        Err(e) => error_result(&format!("Failed to send typing indicator: {}", e)),
    }
}

/// Get the list of users currently typing in a thread.
#[tracing::instrument(skip(services), name = "mcp.tools.get_typing_users")]
async fn execute_get_typing_users(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };

    let typing_users = services.messaging().get_typing_users(thread_id);
    json_result(&json!({
        "thread_id": thread_id,
        "typing_users": typing_users
    }))
}

/// Get messages queued for sending (offline queue).
#[tracing::instrument(skip(services), name = "mcp.tools.get_pending_messages")]
async fn execute_get_pending_messages(services: &UiServices) -> ToolCallResult {
    let pending = services.messaging().get_pending_messages();
    let list: Vec<Value> = pending
        .iter()
        .map(|m| {
            let status_str = if m.status.is_sending() {
                "sending"
            } else if m.status.is_pending() {
                "pending"
            } else {
                "failed"
            };
            json!({
                "id": m.id,
                "thread_id": m.thread_id,
                "text": m.text,
                "reply_to_id": m.reply_to_id,
                "queued_at": m.queued_at,
                "retry_count": m.retry_count,
                "status": status_str,
                "last_error": m.last_error
            })
        })
        .collect();

    json_result(&json!({
        "pending_messages": list,
        "count": list.len()
    }))
}

/// Queue a message for sending when network is available.
#[tracing::instrument(skip(services), name = "mcp.tools.queue_offline_message")]
async fn execute_queue_offline_message(services: &UiServices, args: Value) -> ToolCallResult {
    let thread_id = match args["thread_id"].as_str() {
        Some(id) => id,
        None => return error_result("thread_id is required"),
    };
    let text = match args["text"].as_str() {
        Some(t) => t,
        None => return error_result("text is required"),
    };
    let reply_to_id = args["reply_to_id"].as_str();

    let pending_id = services
        .messaging()
        .queue_message(thread_id, text, reply_to_id);
    json_result(&json!({
        "success": true,
        "pending_id": pending_id,
        "thread_id": thread_id
    }))
}

/// Retry sending all pending messages.
#[tracing::instrument(skip(services), name = "mcp.tools.retry_pending_messages")]
async fn execute_retry_pending_messages(services: &UiServices) -> ToolCallResult {
    let results = services.messaging().retry_all_pending().await;
    let (succeeded, failed): (Vec<_>, Vec<_>) =
        results.into_iter().partition(|(_, result)| result.is_ok());

    json_result(&json!({
        "succeeded": succeeded.len(),
        "failed": failed.len(),
        "details": {
            "succeeded_ids": succeeded.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
            "failed_ids": failed.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>()
        }
    }))
}

/// Cancel a pending message from the offline queue.
#[tracing::instrument(skip(services), name = "mcp.tools.cancel_pending_message")]
async fn execute_cancel_pending_message(services: &UiServices, args: Value) -> ToolCallResult {
    let pending_id = match args["pending_id"].as_str() {
        Some(id) => id,
        None => return error_result("pending_id is required"),
    };

    let removed = services.messaging().remove_pending_message(pending_id);
    if removed {
        json_result(&json!({
            "success": true,
            "pending_id": pending_id
        }))
    } else {
        error_result(&format!("Pending message not found: {}", pending_id))
    }
}

async fn execute_list_files(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = match args["entity_id"].as_str() {
        Some(s) => s.to_string(),
        None => return error_result("entity_id is required for list_files"),
    };
    let disk_type = match args["disk_type"].as_str().and_then(parse_ui_disk_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing disk_type"),
    };
    let path = args["path"].as_str().unwrap_or("/").to_string();

    match services
        .drive()
        .list_directory(&entity_id, disk_type, &path)
        .await
    {
        Ok(entries) => {
            let list: Vec<Value> = entries
                .iter()
                .map(|e| {
                    json!({
                        "path": e.path,
                        "name": e.name,
                        "is_directory": e.is_directory,
                        "size_bytes": e.size_bytes,
                        "modified_at": e.modified_at,
                        "mime_type": e.mime_type
                    })
                })
                .collect();
            json_result(&json!({"files": list}))
        }
        Err(e) => error_result(&format!("Failed to list files: {}", e)),
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
        Ok(_) => success_result(&format!("Presence set to {status_str}")),
        Err(e) => error_result(&format!("Failed to set presence: {e}")),
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
        Err(e) => error_result(&format!("Failed to get presence: {e}")),
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
        Err(e) => error_result(&format!("Failed to subscribe to presence: {e}")),
    }
}

async fn execute_start_voice_call(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let video_enabled = bool_or(&args, "video_enabled", false);

    match services.call().start_call(&entity_id, video_enabled).await {
        Ok(call_info) => json_result(&json!({
            "success": true,
            "call_id": call_info.call_id,
            "entity_id": call_info.entity_id
        })),
        Err(e) => error_result(&format!("Failed to start call: {e}")),
    }
}

async fn execute_join_call(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");

    match services.call().join_call(&call_id).await {
        Ok(call_info) => json_result(&json!({
            "success": true,
            "call_id": call_info.call_id,
            "entity_id": call_info.entity_id
        })),
        Err(e) => error_result(&format!("Failed to join call: {e}")),
    }
}

async fn execute_end_call(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");

    // MCP API takes explicit call_id; use app() for direct control
    let cmd = Command::LeaveCall {
        call_id: call_id.clone(),
    };

    match services.call().app().execute(cmd).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "call_id": call_id
        })),
        Err(e) => error_result(&format!("Failed to end call: {}", e.message)),
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
        Err(e) => return error_result(&format!("Failed to decode content: {e}")),
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
    let meta_path = format!("{path}.meta.json");
    let meta_data = match serde_json::to_vec(&metadata) {
        Ok(d) => d,
        Err(e) => return error_result(&format!("Failed to serialize metadata: {e}")),
    };

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

    let meta_path = format!("{path}.meta.json");

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

async fn execute_share_screen(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");
    let enabled = bool_or(&args, "enabled", true);

    // MCP API takes explicit call_id and enabled flag; use app() for direct control
    let cmd = if enabled {
        Command::StartScreenShare {
            call_id: call_id.clone(),
        }
    } else {
        Command::StopScreenShare {
            call_id: call_id.clone(),
        }
    };

    match services.call().app().execute(cmd).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "call_id": call_id,
            "screen_share": if enabled { "started" } else { "stopped" }
        })),
        Err(e) => error_result(&format!("Failed to update screen share: {}", e.message)),
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
                        "Connection to {peer_four_words} failed: {reason}"
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

async fn execute_get_connection_words(app: &CommunitasApp) -> ToolCallResult {
    match app.query(Query::GetConnectionWords).await {
        Ok(QueryResponse::OptionalString(Some(words))) => json_result(&json!({
            "success": true,
            "connection_words": words,
            "message": "Share these words out-of-band so others can connect to you"
        })),
        Ok(QueryResponse::OptionalString(None)) => json_result(&json!({
            "success": false,
            "error": "No external address discovered yet. Start networking and wait for NAT reflection."
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get connection words: {}", e.message)),
    }
}

async fn execute_connect_by_words(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let words = require_str!(args, "words");

    // Decode the 4 words to a SocketAddr
    let addr = match conn_from_words(&words) {
        Ok(addr) => addr,
        Err(e) => return error_result(&format!("Invalid connection words '{words}': {e}")),
    };

    // Connect to the peer's address directly
    // The gossip system will handle identity exchange
    let cmd = Command::ConnectToPeer {
        peer_four_words: addr.to_string(),
    };

    match app.execute(cmd).await {
        Ok(events) => {
            for event in &events {
                if let Event::PeerConnected { peer_four_words } = event {
                    return json_result(&json!({
                        "success": true,
                        "message": "Connected to peer",
                        "peer_identity": peer_four_words,
                        "address": addr.to_string()
                    }));
                }
            }
            json_result(&json!({
                "success": true,
                "message": "Connection initiated",
                "address": addr.to_string()
            }))
        }
        Err(e) => error_result(&format!("Failed to connect to {}: {}", addr, e.message)),
    }
}

// Peer Presence executors (ADR-014)

/// Parse a pubkey string that may be hex or base64 encoded
fn parse_pubkey(pubkey_str: &str) -> Result<Vec<u8>, String> {
    // Try hex first (common format)
    if let Ok(bytes) = hex::decode(pubkey_str) {
        return Ok(bytes);
    }

    // Try base64 standard
    if let Ok(bytes) = BASE64_STANDARD.decode(pubkey_str) {
        return Ok(bytes);
    }

    // Try base64 URL-safe
    if let Ok(bytes) = BASE64_URL_SAFE.decode(pubkey_str) {
        return Ok(bytes);
    }

    Err(format!(
        "Invalid pubkey format: expected hex or base64, got '{}'",
        if pubkey_str.len() > 20 {
            format!("{}...", &pubkey_str[..20])
        } else {
            pubkey_str.to_string()
        }
    ))
}

async fn execute_announce_presence(app: &CommunitasApp) -> ToolCallResult {
    match app.execute(Command::AnnouncePresence).await {
        Ok(_events) => {
            // Get our connection words to include in response
            match app.query(Query::GetConnectionWords).await {
                Ok(QueryResponse::OptionalString(Some(words))) => json_result(&json!({
                    "success": true,
                    "message": "Presence announced to connected peers",
                    "connection_words": words
                })),
                Ok(QueryResponse::OptionalString(None)) => json_result(&json!({
                    "success": true,
                    "message": "Presence announced (connection words not yet discovered)"
                })),
                Ok(_) => json_result(&json!({
                    "success": true,
                    "message": "Presence announced to connected peers"
                })),
                Err(_) => json_result(&json!({
                    "success": true,
                    "message": "Presence announced to connected peers"
                })),
            }
        }
        Err(e) => error_result(&format!("Failed to announce presence: {}", e.message)),
    }
}

async fn execute_query_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let pubkey_str = require_str!(args, "pubkey");

    let pubkey = match parse_pubkey(&pubkey_str) {
        Ok(pk) => pk,
        Err(e) => return error_result(&e),
    };

    let cmd = Command::QueryPeerPresence {
        target_pubkey: pubkey,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            // Check if we received a presence response
            for event in &events {
                if let Event::PeerPresenceReceived { record } = event {
                    return json_result(&json!({
                        "success": true,
                        "found": true,
                        "presence": {
                            "pubkey": hex::encode(&record.pubkey),
                            "connection_words": record.connection_words,
                            "timestamp": record.timestamp
                        }
                    }));
                }
            }
            // Query sent but no immediate response (async network query)
            json_result(&json!({
                "success": true,
                "found": false,
                "message": "Presence query sent to network. Results may arrive asynchronously."
            }))
        }
        Err(e) => error_result(&format!("Failed to query presence: {}", e.message)),
    }
}

async fn execute_get_our_presence(app: &CommunitasApp) -> ToolCallResult {
    match app.query(Query::GetOurPresenceRecord).await {
        Ok(QueryResponse::OurPresenceRecord(Some(record))) => json_result(&json!({
            "success": true,
            "presence": {
                "pubkey": hex::encode(&record.pubkey),
                "connection_words": record.connection_words,
                "timestamp": record.timestamp
            }
        })),
        Ok(QueryResponse::OurPresenceRecord(None)) => json_result(&json!({
            "success": false,
            "message": "No presence record available. Start networking first."
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get our presence: {}", e.message)),
    }
}

async fn execute_get_cached_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let pubkey_str = require_str!(args, "pubkey");

    let pubkey = match parse_pubkey(&pubkey_str) {
        Ok(pk) => pk,
        Err(e) => return error_result(&e),
    };

    match app.query(Query::GetCachedPeerPresence { pubkey }).await {
        Ok(QueryResponse::CachedPeerPresence(Some(record))) => json_result(&json!({
            "success": true,
            "found": true,
            "presence": {
                "pubkey": hex::encode(&record.pubkey),
                "connection_words": record.connection_words,
                "timestamp": record.timestamp
            }
        })),
        Ok(QueryResponse::CachedPeerPresence(None)) => json_result(&json!({
            "success": true,
            "found": false,
            "message": "No cached presence for this peer"
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get cached presence: {}", e.message)),
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

/// List contacts with optional presence info and filtering.
///
/// Supports filters: "all" (default), "online", "favorites".
#[tracing::instrument(skip(app), name = "mcp.tools.list_contacts")]
async fn execute_list_contacts(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let include_presence = args["include_presence"].as_bool().unwrap_or(true);
    let filter = args["filter"].as_str().unwrap_or("all");

    // Choose query based on filter
    let query = match filter {
        "favorites" => Query::ListFavouriteContacts,
        _ => Query::ListContacts,
    };

    match app.query(query).await {
        Ok(QueryResponse::ContactList(contacts)) => {
            // Apply "online" filter if needed
            let filtered: Vec<_> = if filter == "online" {
                contacts.into_iter().filter(|c| c.is_online).collect()
            } else {
                contacts
            };

            let list: Vec<Value> = filtered
                .iter()
                .map(|c| {
                    let mut contact_json = json!({
                        "id": c.id,
                        "display_name": c.display_name,
                        "four_words": c.four_words,
                        "is_favourite": c.is_favourite
                    });

                    // Include presence info if requested
                    if include_presence && let Some(obj) = contact_json.as_object_mut() {
                        obj.insert("is_online".to_string(), json!(c.is_online));
                        obj.insert("last_seen".to_string(), json!(c.last_seen));
                        // Map is_online to presence status string
                        let presence_status = if c.is_online { "online" } else { "offline" };
                        obj.insert("presence_status".to_string(), json!(presence_status));
                    }

                    contact_json
                })
                .collect();

            json_result(&json!({
                "contacts": list,
                "count": list.len(),
                "filter": filter,
                "include_presence": include_presence
            }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to list contacts: {}", e.message)),
    }
}

/// Get detailed presence status for a specific contact.
#[tracing::instrument(skip(app), name = "mcp.tools.get_contact_presence")]
async fn execute_get_contact_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let contact_id = match args["contact_id"].as_str() {
        Some(id) => id,
        None => return error_result("contact_id is required"),
    };

    // First get the contact to verify it exists
    let query = Query::GetContact {
        contact_id: contact_id.to_string(),
    };

    match app.query(query).await {
        Ok(QueryResponse::Contact(contact)) => {
            // Determine presence status from is_online
            let presence_status = if contact.is_online {
                "online"
            } else {
                "offline"
            };

            // Get current timestamp for last_active if online
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            json_result(&json!({
                "contact_id": contact.id,
                "display_name": contact.display_name,
                "presence": {
                    "status": presence_status,
                    "last_seen": contact.last_seen,
                    "last_active": if contact.is_online { Some(now_ms) } else { contact.last_seen.map(|t| t as u64) },
                    "status_message": null,
                    "is_typing": false
                }
            }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get contact presence: {}", e.message)),
    }
}

/// Set own global presence status.
#[tracing::instrument(skip(app), name = "mcp.tools.set_my_presence")]
async fn execute_set_my_presence(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let status = match args["status"].as_str() {
        Some(s) => s,
        None => return error_result("status is required"),
    };
    let status_message = args["status_message"].as_str().map(String::from);

    // Validate status
    let presence_status = match status {
        "online" => PresenceStatus::Online,
        "away" => PresenceStatus::Away,
        "busy" => PresenceStatus::Busy,
        "offline" => PresenceStatus::Offline,
        _ => return error_result("Invalid status. Must be one of: online, away, busy, offline"),
    };

    // Get our identity to set presence for ourselves
    let identity_query = Query::GetProfile;
    let user_id = match app.query(identity_query).await {
        Ok(QueryResponse::Profile { four_words, .. }) => four_words,
        _ => "self".to_string(), // Fallback if profile not available
    };

    // Create presence update using the status_only helper
    let update = PresenceUpdate::status_only(presence_status);

    // Note: status_message is captured but not yet used - could be added to PresenceUpdate in future
    let _ = status_message;

    match PresenceOperations::update_presence(app, user_id.clone(), update).await {
        Ok(_presence) => json_result(&json!({
            "success": true,
            "user_id": user_id,
            "status": status,
            "status_message": status_message,
            "message": format!("Presence set to {}", status)
        })),
        Err(e) => error_result(&format!("Failed to set presence: {e}")),
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

async fn execute_update_kanban_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let title = opt_str(&args, "title");
    let description = opt_str(&args, "description");

    let updates = communitas_ui_service::kanban::CardUpdate {
        title,
        description,
        ..Default::default()
    };

    match services
        .kanban()
        .update_card(&board_id, &card_id, updates)
        .await
    {
        Ok(_) => success_result("Kanban card updated"),
        Err(e) => error_result(&format!("Failed to update Kanban card: {e}")),
    }
}

async fn execute_delete_kanban_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");

    // KanbanService doesn't have delete_card; use app() for direct command
    let cmd = Command::DeleteKanbanCard { board_id, card_id };

    match services.kanban().app().execute(cmd).await {
        Ok(_) => success_result("Kanban card deleted"),
        Err(e) => error_result(&format!("Failed to delete Kanban card: {}", e.message)),
    }
}

async fn execute_list_kanban_boards(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    match services.kanban().list_boards(&entity_id).await {
        Ok(boards) => {
            let list: Vec<Value> = boards
                .iter()
                .map(|b| {
                    json!({
                        "id": b.id,
                        "entity_id": b.entity_id,
                        "name": b.name
                    })
                })
                .collect();
            json_result(&json!({"boards": list, "count": list.len()}))
        }
        Err(e) => error_result(&format!("Failed to list Kanban boards: {e}")),
    }
}

async fn execute_get_kanban_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");

    match services.kanban().get_card(&board_id, &card_id).await {
        Ok(card) => json_result(&json!({
            "id": card.id,
            "title": card.title,
            "description": card.description,
            "state": format!("{:?}", card.state)
        })),
        Err(e) => error_result(&format!("Failed to get Kanban card: {e}")),
    }
}

async fn execute_get_kanban_board(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    match services.kanban().get_board(&board_id).await {
        Ok(board) => json_result(&json!({
            "id": board.id,
            "name": board.name,
            "columns": board.columns.iter().map(|c| json!({
                "id": c.id,
                "name": c.name,
                "card_count": c.cards.len()
            })).collect::<Vec<_>>()
        })),
        Err(e) => error_result(&format!("Failed to get Kanban board: {e}")),
    }
}

// ========== Kanban Column Executors ==========

async fn execute_list_kanban_columns(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    // Get board view and extract columns
    match services.kanban().get_board(&board_id).await {
        Ok(board) => {
            let list: Vec<Value> = board
                .columns
                .iter()
                .map(|c| {
                    json!({
                        "id": c.id,
                        "name": c.name,
                        "card_count": c.cards.len()
                    })
                })
                .collect();
            json_result(&json!({"columns": list, "count": list.len()}))
        }
        Err(e) => error_result(&format!("Failed to list columns: {e}")),
    }
}

async fn execute_get_kanban_column(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");

    let app = services.kanban().app();
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
        Err(e) => error_result(&format!("Failed to get column: {e}")),
    }
}

async fn execute_update_kanban_column(services: &UiServices, args: Value) -> ToolCallResult {
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

    let app = services.kanban().app();
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
        Err(e) => error_result(&format!("Failed to update column: {e}")),
    }
}

async fn execute_delete_kanban_column(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.delete_column(&board_id, &column_id) {
        Ok(()) => success_result("Column deleted"),
        Err(e) => error_result(&format!("Failed to delete column: {e}")),
    }
}

async fn execute_move_kanban_column(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = require_str!(args, "column_id");
    let new_position = match args["new_position"].as_u64() {
        Some(pos) => pos as u32,
        None => return error_result("new_position is required"),
    };

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .move_column(&board_id, &column_id, new_position)
    {
        Ok(()) => success_result("Column moved"),
        Err(e) => error_result(&format!("Failed to move column: {e}")),
    }
}

// ========== Kanban Card State Executor ==========

async fn execute_change_card_state(services: &UiServices, args: Value) -> ToolCallResult {
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

    let app = services.kanban().app();
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
        Err(e) => error_result(&format!("Failed to change card state: {e}")),
    }
}

// ========== Kanban Assignment Executors ==========

async fn execute_assign_user(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let user_id = require_str!(args, "user_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .assign_user(&board_id, &card_id, &user_id)
    {
        Ok(()) => success_result("User assigned to card"),
        Err(e) => error_result(&format!("Failed to assign user: {e}")),
    }
}

async fn execute_unassign_user(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let user_id = require_str!(args, "user_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .unassign_user(&board_id, &card_id, &user_id)
    {
        Ok(()) => success_result("User unassigned from card"),
        Err(e) => error_result(&format!("Failed to unassign user: {e}")),
    }
}

// ========== Kanban Tag Executors ==========

async fn execute_create_kanban_tag(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let name = require_str!(args, "name");
    let color = require_str!(args, "color");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.create_tag(&board_id, name, color) {
        Ok(tag) => json_result(&json!({
            "id": tag.id,
            "name": tag.name,
            "color": tag.color
        })),
        Err(e) => error_result(&format!("Failed to create tag: {e}")),
    }
}

async fn execute_list_kanban_tags(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    let app = services.kanban().app();
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
        Err(e) => error_result(&format!("Failed to list tags: {e}")),
    }
}

async fn execute_tag_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let tag_id = require_str!(args, "tag_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.tag_card(&board_id, &card_id, &tag_id) {
        Ok(()) => success_result("Tag added to card"),
        Err(e) => error_result(&format!("Failed to tag card: {e}")),
    }
}

async fn execute_untag_card(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let tag_id = require_str!(args, "tag_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.untag_card(&board_id, &card_id, &tag_id) {
        Ok(()) => success_result("Tag removed from card"),
        Err(e) => error_result(&format!("Failed to untag card: {e}")),
    }
}

// ========== Kanban Step Executors ==========

async fn execute_add_step(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let text = require_str!(args, "text");

    // KanbanService has add_step method (takes title, no position)
    match services.kanban().add_step(&board_id, &card_id, &text).await {
        Ok(step) => json_result(&json!({
            "id": step.id,
            "title": step.title,
            "completed": step.completed
        })),
        Err(e) => error_result(&format!("Failed to add step: {e}")),
    }
}

async fn execute_get_step(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let step_id = require_str!(args, "step_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx.kanban_service.get_step(&board_id, &card_id, &step_id) {
        Ok(step) => json_result(&json!({
            "id": step.id,
            "title": step.text,
            "completed": step.completed
        })),
        Err(e) => error_result(&format!("Failed to get step: {e}")),
    }
}

async fn execute_toggle_step(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let step_id = require_str!(args, "step_id");

    // KanbanService has toggle_step method
    match services
        .kanban()
        .toggle_step(&board_id, &card_id, &step_id)
        .await
    {
        Ok(step) => json_result(&json!({
            "id": step.id,
            "title": step.title,
            "completed": step.completed
        })),
        Err(e) => error_result(&format!("Failed to toggle step: {e}")),
    }
}

async fn execute_delete_step(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let step_id = require_str!(args, "step_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .delete_step(&board_id, &card_id, &step_id)
    {
        Ok(()) => success_result("Step deleted"),
        Err(e) => error_result(&format!("Failed to delete step: {e}")),
    }
}

// ========== Kanban Comment Executors ==========

async fn execute_add_comment(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let content = require_str!(args, "content");

    // KanbanService has add_comment method (no reply_to_id support)
    match services
        .kanban()
        .add_comment(&board_id, &card_id, &content)
        .await
    {
        Ok(comment) => json_result(&json!({
            "id": comment.id,
            "author_id": comment.author_id,
            "author_name": comment.author_name,
            "text": comment.text,
            "created_at": comment.created_at
        })),
        Err(e) => error_result(&format!("Failed to add comment: {e}")),
    }
}

async fn execute_list_comments(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");

    let app = services.kanban().app();
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
        Err(e) => error_result(&format!("Failed to list comments: {e}")),
    }
}

async fn execute_delete_comment(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let card_id = require_str!(args, "card_id");
    let comment_id = require_str!(args, "comment_id");

    let app = services.kanban().app();
    let ctx_lock = app.context();
    let ctx = ctx_lock.read().await;

    match ctx
        .kanban_service
        .delete_comment(&board_id, &card_id, &comment_id)
    {
        Ok(()) => success_result("Comment deleted"),
        Err(e) => error_result(&format!("Failed to delete comment: {e}")),
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
        Err(e) => error_result(&format!("Failed to join entity: {e}")),
    }
}

// ========== File Operations Executors ==========

async fn execute_delete_file(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");

    match services
        .drive()
        .delete_path(&entity_id, disk_type, &path)
        .await
    {
        Ok(()) => success_result("File deleted"),
        Err(e) => error_result(&format!("Failed to delete file: {}", e)),
    }
}

async fn execute_get_disk_stats(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);

    match services.drive().get_quota(&entity_id, disk_type).await {
        Ok(quota) => json_result(&json!({
            "entity_id": entity_id,
            "disk_type": format!("{:?}", quota.disk_type),
            "used_bytes": quota.used_bytes,
            "quota_bytes": quota.quota_bytes,
            "percent_used": quota.percent_used
        })),
        Err(e) => error_result(&format!("Failed to get disk stats: {}", e)),
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

/// Update Kanban board name or description
async fn execute_update_kanban_board(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let name = opt_str(&args, "name");
    let description = opt_str(&args, "description").map(Some);

    // KanbanService has update_board method
    match services
        .kanban()
        .update_board(&board_id, name.clone(), description)
        .await
    {
        Ok(_) => json_result(&json!({
            "success": true,
            "board_id": board_id,
            "name": name
        })),
        Err(e) => error_result(&format!("Failed to update board: {e}")),
    }
}

/// Delete a Kanban board
async fn execute_delete_kanban_board(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");

    // KanbanService has delete_board method
    match services.kanban().delete_board(&board_id).await {
        Ok(_) => json_result(&json!({
            "success": true,
            "message": "Board deleted"
        })),
        Err(e) => error_result(&format!("Failed to delete board: {e}")),
    }
}

/// List all cards in a Kanban board with optional filters
async fn execute_list_kanban_cards(services: &UiServices, args: Value) -> ToolCallResult {
    let board_id = require_str!(args, "board_id");
    let column_id = opt_str(&args, "column_id");
    let state_filter = opt_str(&args, "state");
    let assignee_filter = opt_str(&args, "assignee_id");
    let tag_filter = opt_str(&args, "tag_id");

    // Get board view and extract cards from all columns
    match services.kanban().get_board(&board_id).await {
        Ok(board_view) => {
            let mut cards_json: Vec<Value> = Vec::new();

            for column in &board_view.columns {
                // Filter by column_id if specified
                if let Some(ref cid) = column_id
                    && &column.id != cid
                {
                    continue;
                }

                for card in &column.cards {
                    // Apply optional filters
                    if let Some(ref state) = state_filter {
                        let card_state = format!("{:?}", card.state).to_lowercase();
                        if !card_state.contains(&state.to_lowercase()) {
                            continue;
                        }
                    }
                    if let Some(ref assignee) = assignee_filter
                        && !card.assignees.iter().any(|a| a == assignee)
                    {
                        continue;
                    }
                    if let Some(ref tag) = tag_filter
                        && !card.tags.iter().any(|t| t.id == *tag || t.name == *tag)
                    {
                        continue;
                    }

                    cards_json.push(json!({
                        "id": card.id,
                        "column_id": column.id,
                        "title": card.title,
                        "description": card.description,
                        "position": card.position,
                        "state": format!("{:?}", card.state),
                        "assignees": card.assignees,
                        "tags": card.tags
                    }));
                }
            }

            json_result(&json!({ "cards": cards_json }))
        }
        Err(e) => error_result(&format!("Failed to list cards: {e}")),
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
        description: Some(format!("Default board for {name}")),
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

// ============================================================================
// Recovery Tools (ADR-016) - Identity Creation and Recovery
// ============================================================================

/// Valid BIP39 mnemonic word counts (128-256 bits of entropy).
const VALID_WORD_COUNTS: [usize; 5] = [12, 15, 18, 21, 24];

/// Normalize mnemonic input: trim, collapse whitespace, lowercase.
fn normalize_mnemonic(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Encode identity public keys as base64 strings.
fn encode_public_keys(keys: &communitas_core::recovery::IdentityKeys) -> (String, String) {
    (
        BASE64_STANDARD.encode(keys.verifying_key_bytes()),
        BASE64_STANDARD.encode(keys.encapsulation_key_bytes()),
    )
}

/// Extract passphrase from args, treating empty string as None.
/// This prevents accidental key derivation differences from "" vs missing passphrase.
fn extract_passphrase(args: &Value) -> Option<&str> {
    args.get("passphrase")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
}

/// Dispatcher for recovery-related tools.
/// These tools do not require authentication as they're used to create/recover identity.
async fn dispatch_recovery_tools(name: &str, args: &Value) -> Option<ToolCallResult> {
    match name {
        "create_identity" => Some(execute_create_identity(args).await),
        "recover_identity" => Some(execute_recover_identity(args).await),
        "validate_mnemonic" => Some(execute_validate_mnemonic(args).await),
        _ => None,
    }
}

/// Create a new identity with BIP39 mnemonic.
///
/// Returns mnemonic words for backup, four-word identity, and public keys.
/// SECURITY: Never returns private keys - only public keys are included.
async fn execute_create_identity(args: &Value) -> ToolCallResult {
    use communitas_core::recovery::{RecoveryConfig, create_new_identity};

    // Extract optional parameters
    let word_count = args
        .get("word_count")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(24);

    let passphrase = extract_passphrase(args);

    // Validate word count
    if !VALID_WORD_COUNTS.contains(&word_count) {
        tracing::warn!(word_count, "Invalid word count requested");
        return error_result(&format!(
            "Invalid word_count: {word_count}. BIP39 requires 12, 15, 18, 21, or 24 words"
        ));
    }

    // Build configuration
    let config = RecoveryConfig::default().with_word_count(word_count);

    // Create identity
    match create_new_identity(&config, passphrase) {
        Ok((mnemonic, keys)) => {
            // Convert mnemonic to word array
            let mnemonic_words: Vec<String> = mnemonic.words().map(String::from).collect();
            let (public_signing_key, public_encryption_key) = encode_public_keys(&keys);

            tracing::info!(
                four_words = %keys.four_words,
                word_count,
                "Created new identity"
            );

            json_result(&json!({
                "mnemonic_words": mnemonic_words,
                "four_words": keys.four_words,
                "public_signing_key": public_signing_key,
                "public_encryption_key": public_encryption_key,
                "warning": "IMPORTANT: Write down your recovery phrase and store it safely offline. \
                           This phrase is the ONLY way to recover your identity if you lose access. \
                           Never share it with anyone. Never store it digitally."
            }))
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                word_count,
                has_passphrase = passphrase.is_some(),
                "Identity creation failed"
            );
            error_result(&format!("Failed to create identity: {e}"))
        }
    }
}

/// Recover an identity from a BIP39 mnemonic phrase.
///
/// Returns four-word identity and public keys.
/// SECURITY: Never returns private keys - only public keys are included.
async fn execute_recover_identity(args: &Value) -> ToolCallResult {
    use communitas_core::recovery::{Language, recover_identity};

    // Extract required mnemonic
    let mnemonic_words = match args.get("mnemonic_words").and_then(|v| v.as_str()) {
        Some(words) => words.trim(),
        None => return error_result("mnemonic_words is required"),
    };

    if mnemonic_words.is_empty() {
        return error_result("mnemonic_words cannot be empty");
    }

    let passphrase = extract_passphrase(args);
    let normalized = normalize_mnemonic(mnemonic_words);

    // Recover identity
    match recover_identity(&normalized, Language::English, passphrase) {
        Ok(keys) => {
            let (public_signing_key, public_encryption_key) = encode_public_keys(&keys);

            tracing::info!(
                four_words = %keys.four_words,
                "Recovered identity from mnemonic"
            );

            json_result(&json!({
                "four_words": keys.four_words,
                "public_signing_key": public_signing_key,
                "public_encryption_key": public_encryption_key,
                "success": true
            }))
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                has_passphrase = passphrase.is_some(),
                "Identity recovery failed"
            );
            error_result(&format!("Failed to recover identity: {e}"))
        }
    }
}

/// Validate a BIP39 mnemonic phrase without deriving keys.
///
/// Quick check for word validity and checksum. Does not derive any keys.
/// Note: Returns is_error=false even for invalid mnemonics, since validation
/// itself succeeded - check the `valid` field for the validation result.
async fn execute_validate_mnemonic(args: &Value) -> ToolCallResult {
    use communitas_core::recovery::{Language, validate_mnemonic};

    // Extract required mnemonic
    let mnemonic_words = match args.get("mnemonic_words").and_then(|v| v.as_str()) {
        Some(words) => words.trim(),
        None => return error_result("mnemonic_words is required"),
    };

    if mnemonic_words.is_empty() {
        return json_result(&json!({
            "valid": false,
            "word_count": 0,
            "error": "Mnemonic cannot be empty"
        }));
    }

    let normalized = normalize_mnemonic(mnemonic_words);
    let word_count = normalized.split_whitespace().count();

    // Validate word count first
    if !VALID_WORD_COUNTS.contains(&word_count) {
        return json_result(&json!({
            "valid": false,
            "word_count": word_count,
            "error": format!("Invalid word count: {}. BIP39 requires 12, 15, 18, 21, or 24 words", word_count)
        }));
    }

    // Validate mnemonic (words and checksum)
    match validate_mnemonic(&normalized, Language::English) {
        Ok(_) => json_result(&json!({
            "valid": true,
            "word_count": word_count
        })),
        Err(e) => json_result(&json!({
            "valid": false,
            "word_count": word_count,
            "error": e.to_string()
        })),
    }
}

// ========== Canvas Executors ==========

async fn execute_canvas_get_snapshot(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    // Use app() for query operations - MCP queries persisted state
    let query = Query::GetCanvasSnapshot { entity_id };

    match services.canvas().app().query(query).await {
        Ok(QueryResponse::CanvasSnapshot(snapshot)) => {
            let elements: Vec<Value> = snapshot
                .elements
                .iter()
                .map(|e| {
                    json!({
                        "id": e.id,
                        "element_type": e.element_type,
                        "x": e.x,
                        "y": e.y,
                        "width": e.width,
                        "height": e.height,
                        "rotation": e.rotation,
                        "z_index": e.z_index,
                        "selected": e.selected,
                        "interactive": e.interactive,
                        "data": e.data
                    })
                })
                .collect();
            json_result(&json!({
                "entity_id": snapshot.entity_id,
                "elements": elements,
                "element_count": elements.len(),
                "viewport_width": snapshot.viewport_width,
                "viewport_height": snapshot.viewport_height,
                "zoom": snapshot.zoom,
                "pan_x": snapshot.pan_x,
                "pan_y": snapshot.pan_y,
                "loading": snapshot.loading
            }))
        }
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!("Failed to get canvas snapshot: {}", e.message)),
    }
}

async fn execute_canvas_add_text(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let content = require_str!(args, "content");
    let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let font_size = args
        .get("font_size")
        .and_then(|v| v.as_f64())
        .unwrap_or(16.0) as f32;
    let color = opt_str(&args, "color").unwrap_or_else(|| "#000000".to_string());

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .add_text(Some(&entity_id), content, x, y, font_size, color)
        .await
    {
        Ok(element_id) => json_result(&json!({
            "success": true,
            "message": "Text element added to canvas",
            "element_id": element_id
        })),
        Err(e) => error_result(&format!("Failed to add text to canvas: {e}")),
    }
}

async fn execute_canvas_add_image(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let src = require_str!(args, "src");
    let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(100.0) as f32;
    let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(100.0) as f32;

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .add_image(Some(&entity_id), src, x, y, width, height)
        .await
    {
        Ok(element_id) => json_result(&json!({
            "success": true,
            "message": "Image element added to canvas",
            "element_id": element_id
        })),
        Err(e) => error_result(&format!("Failed to add image to canvas: {e}")),
    }
}

async fn execute_canvas_add_chart(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let chart_type = require_str!(args, "chart_type");
    let data_str = require_str!(args, "data");
    let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(300.0) as f32;
    let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(200.0) as f32;

    // Parse data string to JSON value for CanvasService
    let data: serde_json::Value = match serde_json::from_str(&data_str) {
        Ok(v) => v,
        Err(e) => return error_result(&format!("Invalid chart data JSON: {e}")),
    };

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .add_chart(Some(&entity_id), chart_type, data, x, y, width, height)
        .await
    {
        Ok(element_id) => json_result(&json!({
            "success": true,
            "message": "Chart element added to canvas",
            "element_id": element_id
        })),
        Err(e) => error_result(&format!("Failed to add chart to canvas: {e}")),
    }
}

async fn execute_canvas_remove_element(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let element_id = require_str!(args, "element_id");

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .remove_element(Some(&entity_id), &element_id)
        .await
    {
        Ok(()) => success_result("Canvas element removed"),
        Err(e) => error_result(&format!("Failed to remove canvas element: {e}")),
    }
}

async fn execute_canvas_update_transform(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let element_id = require_str!(args, "element_id");
    let x = args.get("x").and_then(|v| v.as_f64()).map(|v| v as f32);
    let y = args.get("y").and_then(|v| v.as_f64()).map(|v| v as f32);
    let width = args.get("width").and_then(|v| v.as_f64()).map(|v| v as f32);
    let height = args
        .get("height")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);
    let rotation = args
        .get("rotation")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);
    let z_index = args
        .get("z_index")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    // MCP API allows partial updates; use app() for direct control
    let cmd = Command::CanvasUpdateTransform {
        entity_id,
        element_id,
        x,
        y,
        width,
        height,
        rotation,
        z_index,
    };

    match services.canvas().app().execute(cmd).await {
        Ok(_) => success_result("Canvas element transform updated"),
        Err(e) => error_result(&format!("Failed to update canvas transform: {}", e.message)),
    }
}

async fn execute_canvas_select_element(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let element_id = require_str!(args, "element_id");

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .select_element(Some(&entity_id), &element_id)
        .await
    {
        Ok(()) => success_result("Canvas element selected"),
        Err(e) => error_result(&format!("Failed to select canvas element: {e}")),
    }
}

async fn execute_canvas_deselect_all(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    // Use CanvasService for MCP-Dioxus parity
    match services.canvas().deselect_all(Some(&entity_id)).await {
        Ok(()) => success_result("All canvas elements deselected"),
        Err(e) => error_result(&format!("Failed to deselect canvas elements: {e}")),
    }
}

async fn execute_canvas_set_viewport(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let width = args.get("width").and_then(|v| v.as_f64()).unwrap_or(800.0) as f32;
    let height = args.get("height").and_then(|v| v.as_f64()).unwrap_or(600.0) as f32;

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .set_viewport(Some(&entity_id), width, height)
        .await
    {
        Ok(()) => success_result("Canvas viewport updated"),
        Err(e) => error_result(&format!("Failed to set canvas viewport: {e}")),
    }
}

async fn execute_canvas_set_view(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let zoom = args.get("zoom").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
    let pan_x = args.get("pan_x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let pan_y = args.get("pan_y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    // Use CanvasService for MCP-Dioxus parity
    match services
        .canvas()
        .set_view(Some(&entity_id), zoom, pan_x, pan_y)
        .await
    {
        Ok(()) => success_result("Canvas view updated"),
        Err(e) => error_result(&format!("Failed to set canvas view: {e}")),
    }
}

async fn execute_canvas_clear(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    // CanvasService.clear() has no entity_id parameter; use app() for direct control
    let cmd = Command::CanvasClear { entity_id };

    match services.canvas().app().execute(cmd).await {
        Ok(_) => success_result("Canvas cleared"),
        Err(e) => error_result(&format!("Failed to clear canvas: {}", e.message)),
    }
}

async fn execute_canvas_export(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    // Use CanvasService for MCP-Dioxus parity
    match services.canvas().export_json(Some(&entity_id)).await {
        Ok(json) => json_result(&json!({
            "success": true,
            "json": json
        })),
        Err(e) => error_result(&format!("Failed to export canvas: {e}")),
    }
}

async fn execute_canvas_import(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let json = require_str!(args, "json");

    // Use CanvasService for MCP-Dioxus parity
    match services.canvas().import_json(Some(&entity_id), &json).await {
        Ok(()) => success_result("Canvas imported successfully"),
        Err(e) => error_result(&format!("Failed to import canvas: {e}")),
    }
}

async fn execute_canvas_element_at(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let x = args.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let y = args.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;

    // MCP queries persisted state with entity_id; use app() for query
    let query = Query::CanvasElementAt { entity_id, x, y };

    match services.canvas().app().query(query).await {
        Ok(QueryResponse::CanvasElement(Some(element))) => json_result(&json!({
            "found": true,
            "element": {
                "id": element.id,
                "element_type": element.element_type,
                "x": element.x,
                "y": element.y,
                "width": element.width,
                "height": element.height,
                "rotation": element.rotation,
                "z_index": element.z_index,
                "selected": element.selected,
                "interactive": element.interactive,
                "data": element.data
            }
        })),
        Ok(QueryResponse::CanvasElement(None)) => json_result(&json!({
            "found": false,
            "element": null
        })),
        Ok(_) => error_result("Unexpected response type"),
        Err(e) => error_result(&format!(
            "Failed to find element at position: {}",
            e.message
        )),
    }
}

// ============================================================================
// Drive MCP Executor Functions
// ============================================================================

async fn execute_create_directory(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");

    match services
        .drive()
        .create_directory(&entity_id, disk_type, &path)
        .await
    {
        Ok(entry) => json_result(&json!({
            "success": true,
            "message": "Directory created successfully",
            "path": entry.path,
            "name": entry.name
        })),
        Err(e) => error_result(&format!("Failed to create directory: {}", e)),
    }
}

async fn execute_move_file(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let source_path = require_str!(args, "source_path");
    let dest_path = require_str!(args, "dest_path");

    match services
        .drive()
        .move_path(&entity_id, disk_type, &source_path, &dest_path)
        .await
    {
        Ok(entry) => json_result(&json!({
            "success": true,
            "message": "File moved successfully",
            "new_path": entry.path
        })),
        Err(e) => error_result(&format!("Failed to move file: {}", e)),
    }
}

async fn execute_copy_file(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let source_path = require_str!(args, "source_path");
    let dest_path = require_str!(args, "dest_path");

    match services
        .drive()
        .copy_path(&entity_id, disk_type, &source_path, &dest_path)
        .await
    {
        Ok(entry) => json_result(&json!({
            "success": true,
            "message": "File copied successfully",
            "new_path": entry.path
        })),
        Err(e) => error_result(&format!("Failed to copy file: {}", e)),
    }
}

async fn execute_list_disks(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    match services.drive().list_disks(&entity_id).await {
        Ok(disks) => {
            let disk_data: Vec<_> = disks
                .iter()
                .map(|d| {
                    json!({
                        "disk_type": format!("{:?}", d.disk_type).to_lowercase(),
                        "entity_id": d.entity_id,
                        "total_bytes": d.total_bytes,
                        "used_bytes": d.used_bytes,
                        "available_bytes": d.available_bytes,
                        "file_count": d.file_count
                    })
                })
                .collect();
            json_result(&json!({
                "disks": disk_data
            }))
        }
        Err(e) => error_result(&format!("Failed to list disks: {}", e)),
    }
}

async fn execute_get_file_preview(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");

    match services
        .drive()
        .get_file_preview(&entity_id, disk_type, &path)
        .await
    {
        Ok(preview) => {
            let mut result = json!({
                "path": preview.path,
                "mime_type": preview.mime_type,
                "size_bytes": preview.size_bytes,
                "checksum": preview.metadata.checksum,
                "created_at": preview.metadata.created_at,
                "modified_at": preview.metadata.modified_at
            });
            if let Some(text) = &preview.text_preview {
                result["text_preview"] = json!(text);
            }
            if preview.thumbnail.is_some() {
                result["has_thumbnail"] = json!(true);
            }
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to get file preview: {}", e)),
    }
}

// ============================================================================
// Streaming Transfer MCP Executor Functions
// ============================================================================

async fn execute_start_streaming_upload(services: &UiServices, args: Value) -> ToolCallResult {
    use std::path::Path;

    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");
    let local_path = require_str!(args, "local_path");

    match services
        .drive()
        .start_streaming_upload(&entity_id, disk_type, &path, Path::new(&local_path))
        .await
    {
        Ok(upload_id) => {
            // Get progress to return more details
            if let Some(progress) = services.drive().get_upload_progress(&upload_id).await {
                json_result(&json!({
                    "upload_id": upload_id,
                    "entity_id": entity_id,
                    "disk_type": format!("{:?}", disk_type).to_lowercase(),
                    "path": path,
                    "total_bytes": progress.total_bytes,
                    "bytes_uploaded": progress.bytes_uploaded,
                    "state": format!("{:?}", progress.state).to_lowercase(),
                    "percent_complete": progress.percent_complete(),
                    "message": "Streaming upload started. Use get_upload_progress to track progress."
                }))
            } else {
                json_result(&json!({
                    "upload_id": upload_id,
                    "message": "Streaming upload started. Use get_upload_progress to track progress."
                }))
            }
        }
        Err(e) => error_result(&format!("Failed to start streaming upload: {}", e)),
    }
}

async fn execute_start_streaming_download(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");
    let local_path = require_str!(args, "local_path");

    match services
        .drive()
        .start_streaming_download(&entity_id, disk_type, &path, &local_path)
        .await
    {
        Ok(download_id) => {
            // Get progress to return more details
            if let Some(progress) = services.drive().get_download_progress(&download_id).await {
                json_result(&json!({
                    "download_id": download_id,
                    "entity_id": entity_id,
                    "disk_type": format!("{:?}", disk_type).to_lowercase(),
                    "path": path,
                    "local_path": local_path,
                    "total_bytes": progress.total_bytes,
                    "bytes_downloaded": progress.bytes_downloaded,
                    "state": format!("{:?}", progress.state).to_lowercase(),
                    "percent_complete": progress.percent_complete(),
                    "message": "Streaming download started. Use get_download_progress to track progress."
                }))
            } else {
                json_result(&json!({
                    "download_id": download_id,
                    "message": "Streaming download started. Use get_download_progress to track progress."
                }))
            }
        }
        Err(e) => error_result(&format!("Failed to start streaming download: {}", e)),
    }
}

async fn execute_resume_upload(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");
    let local_path = require_str!(args, "local_path");

    // Read file content - limit to 100MB for MCP calls
    let content = match tokio::fs::read(&local_path).await {
        Ok(data) => {
            if data.len() > 100 * 1024 * 1024 {
                return error_result("File too large for resume_upload (max 100MB). Use start_streaming_upload instead.");
            }
            data
        }
        Err(e) => return error_result(&format!("Failed to read local file: {}", e)),
    };

    match services
        .drive()
        .resume_upload(&entity_id, disk_type, &path, content)
        .await
    {
        Ok(upload_id) => {
            if let Some(progress) = services.drive().get_upload_progress(&upload_id).await {
                json_result(&json!({
                    "upload_id": upload_id,
                    "state": format!("{:?}", progress.state).to_lowercase(),
                    "bytes_uploaded": progress.bytes_uploaded,
                    "total_bytes": progress.total_bytes,
                    "message": "Upload resumed. Use get_upload_progress to track progress."
                }))
            } else {
                json_result(&json!({
                    "upload_id": upload_id,
                    "message": "Upload resumed. Use get_upload_progress to track progress."
                }))
            }
        }
        Err(e) => error_result(&format!("Failed to resume upload: {}", e)),
    }
}

async fn execute_resume_download(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");
    let local_path = require_str!(args, "local_path");

    match services
        .drive()
        .resume_download(&entity_id, disk_type, &path, &local_path)
        .await
    {
        Ok(download_id) => {
            if let Some(progress) = services.drive().get_download_progress(&download_id).await {
                json_result(&json!({
                    "download_id": download_id,
                    "state": format!("{:?}", progress.state).to_lowercase(),
                    "bytes_downloaded": progress.bytes_downloaded,
                    "total_bytes": progress.total_bytes,
                    "message": "Download resumed. Use get_download_progress to track progress."
                }))
            } else {
                json_result(&json!({
                    "download_id": download_id,
                    "message": "Download resumed. Use get_download_progress to track progress."
                }))
            }
        }
        Err(e) => error_result(&format!("Failed to resume download: {}", e)),
    }
}

async fn execute_get_upload_progress(services: &UiServices, args: Value) -> ToolCallResult {
    let upload_id = require_str!(args, "upload_id");

    match services.drive().get_upload_progress(&upload_id).await {
        Some(progress) => json_result(&json!({
            "upload_id": progress.id,
            "file_name": progress.file_name,
            "file_path": progress.file_path,
            "state": format!("{:?}", progress.state).to_lowercase(),
            "bytes_uploaded": progress.bytes_uploaded,
            "total_bytes": progress.total_bytes,
            "percent_complete": progress.percent_complete(),
            "checksum_verified": progress.checksum_verified
        })),
        None => error_result(&format!("Upload not found: {}", upload_id)),
    }
}

async fn execute_get_download_progress(services: &UiServices, args: Value) -> ToolCallResult {
    let download_id = require_str!(args, "download_id");

    match services.drive().get_download_progress(&download_id).await {
        Some(progress) => json_result(&json!({
            "download_id": progress.id,
            "file_name": progress.file_name,
            "destination_path": progress.destination_path,
            "state": format!("{:?}", progress.state).to_lowercase(),
            "bytes_downloaded": progress.bytes_downloaded,
            "total_bytes": progress.total_bytes,
            "percent_complete": progress.percent_complete(),
            "checksum_verified": progress.checksum_verified
        })),
        None => error_result(&format!("Download not found: {}", download_id)),
    }
}

async fn execute_cancel_upload(services: &UiServices, args: Value) -> ToolCallResult {
    let upload_id = require_str!(args, "upload_id");

    match services.drive().cancel_upload(&upload_id).await {
        Ok(()) => success_result(&format!("Upload {} cancelled", upload_id)),
        Err(e) => error_result(&format!("Failed to cancel upload: {}", e)),
    }
}

async fn execute_cancel_download(services: &UiServices, args: Value) -> ToolCallResult {
    let download_id = require_str!(args, "download_id");

    match services.drive().cancel_download(&download_id).await {
        Ok(()) => success_result(&format!("Download {} cancelled", download_id)),
        Err(e) => error_result(&format!("Failed to cancel download: {}", e)),
    }
}

// ============================================================================
// Share Link MCP Executor Functions
// ============================================================================

async fn execute_create_share_link(services: &UiServices, args: Value) -> ToolCallResult {
    use communitas_ui_api::drive::ShareLinkConfig;

    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");

    // Build config
    let mut config = ShareLinkConfig::default();

    if let Some(hours) = args["expires_in_hours"].as_u64() {
        config = ShareLinkConfig::expires_in_hours(hours as u32);
    }

    if let Some(password) = args["password"].as_str() {
        config = config.with_password(password);
    }

    if let Some(max_accesses) = args["max_downloads"].as_u64() {
        config = config.with_max_accesses(max_accesses);
    }

    match services
        .drive()
        .create_share_link(&entity_id, disk_type, &path, config)
        .await
    {
        Ok(link) => json_result(&json!({
            "link_id": link.id,
            "url": link.url,
            "entity_id": link.entity_id,
            "disk_type": format!("{:?}", link.disk_type).to_lowercase(),
            "path": link.file_path,
            "file_name": link.file_name,
            "created_at": link.created_at,
            "expires_at": link.expires_at,
            "password_protected": link.password_protected,
            "max_accesses": link.max_accesses,
            "access_count": link.access_count,
            "active": link.active
        })),
        Err(e) => error_result(&format!("Failed to create share link: {}", e)),
    }
}

async fn execute_revoke_share_link(services: &UiServices, args: Value) -> ToolCallResult {
    let link_id = require_str!(args, "link_id");

    match services.drive().revoke_share_link(&link_id).await {
        Ok(()) => success_result(&format!("Share link {} revoked", link_id)),
        Err(e) => error_result(&format!("Failed to revoke share link: {}", e)),
    }
}

async fn execute_list_share_links(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");

    match services.drive().list_share_links(&entity_id).await {
        Ok(links) => {
            let link_data: Vec<_> = links
                .iter()
                .map(|link| {
                    json!({
                        "link_id": link.id,
                        "url": link.url,
                        "path": link.file_path,
                        "file_name": link.file_name,
                        "created_at": link.created_at,
                        "expires_at": link.expires_at,
                        "password_protected": link.password_protected,
                        "access_count": link.access_count,
                        "active": link.active
                    })
                })
                .collect();
            json_result(&json!({
                "entity_id": entity_id,
                "links": link_data,
                "count": links.len()
            }))
        }
        Err(e) => error_result(&format!("Failed to list share links: {}", e)),
    }
}

async fn execute_get_file_share_links(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let path = require_str!(args, "path");

    match services
        .drive()
        .list_file_share_links(&entity_id, disk_type, &path)
        .await
    {
        Ok(links) => {
            let link_data: Vec<serde_json::Value> = links
                .iter()
                .map(|link| {
                    json!({
                        "link_id": link.id,
                        "url": link.url,
                        "created_at": link.created_at,
                        "expires_at": link.expires_at,
                        "password_protected": link.password_protected,
                        "access_count": link.access_count,
                        "active": link.active
                    })
                })
                .collect();
            json_result(&json!({
                "path": path,
                "links": link_data,
                "count": links.len()
            }))
        }
        Err(e) => error_result(&format!("Failed to get file share links: {}", e)),
    }
}

// ============================================================================
// Offline Staging MCP Executor Functions
// ============================================================================

async fn execute_stage_upload(services: &UiServices, args: Value) -> ToolCallResult {
    let entity_id = require_str!(args, "entity_id");
    let disk_type = require_ui_disk_type!(args);
    let destination_path = require_str!(args, "destination_path");
    let local_path = require_str!(args, "local_path");

    match services
        .drive()
        .stage_upload(&entity_id, disk_type, &destination_path, &local_path)
        .await
    {
        Ok(staged) => json_result(&json!({
            "upload_id": staged.id,
            "entity_id": staged.entity_id,
            "disk_type": format!("{:?}", staged.disk_type).to_lowercase(),
            "destination_path": staged.destination_path,
            "local_path": staged.local_path,
            "file_name": staged.file_name,
            "size_bytes": staged.size_bytes,
            "mime_type": staged.mime_type,
            "local_checksum": staged.local_checksum,
            "state": format!("{:?}", staged.state).to_lowercase(),
            "staged_at": staged.staged_at,
            "message": "File staged for upload. Will sync when network is available."
        })),
        Err(e) => error_result(&format!("Failed to stage upload: {}", e)),
    }
}

async fn execute_get_staged_upload(services: &UiServices, args: Value) -> ToolCallResult {
    let upload_id = require_str!(args, "upload_id");

    match services.drive().get_staged_upload(&upload_id).await {
        Ok(staged) => json_result(&json!({
            "upload_id": staged.id,
            "entity_id": staged.entity_id,
            "disk_type": format!("{:?}", staged.disk_type).to_lowercase(),
            "destination_path": staged.destination_path,
            "local_path": staged.local_path,
            "file_name": staged.file_name,
            "size_bytes": staged.size_bytes,
            "mime_type": staged.mime_type,
            "local_checksum": staged.local_checksum,
            "state": format!("{:?}", staged.state).to_lowercase(),
            "retry_count": staged.retry_count,
            "max_retries": staged.max_retries,
            "error": staged.error,
            "staged_at": staged.staged_at,
            "updated_at": staged.updated_at,
            "conflict": staged.conflict.as_ref().map(|c| json!({
                "conflict_type": format!("{:?}", c.conflict_type).to_lowercase(),
                "staged_checksum": c.staged_checksum,
                "local_checksum": c.local_checksum,
                "remote_checksum": c.remote_checksum,
                "remote_size_bytes": c.remote_size_bytes,
                "detected_at": c.detected_at
            }))
        })),
        Err(e) => error_result(&format!("Failed to get staged upload: {}", e)),
    }
}

async fn execute_list_staged_uploads(services: &UiServices) -> ToolCallResult {
    match services.drive().list_staged_uploads().await {
        Ok(uploads) => {
            let upload_data: Vec<_> = uploads
                .iter()
                .map(|u| {
                    json!({
                        "upload_id": u.id,
                        "file_name": u.file_name,
                        "size_bytes": u.size_bytes,
                        "state": format!("{:?}", u.state).to_lowercase(),
                        "destination_path": u.destination_path,
                        "staged_at": u.staged_at,
                        "has_conflict": u.conflict.is_some(),
                        "retry_count": u.retry_count
                    })
                })
                .collect();
            json_result(&json!({
                "uploads": upload_data,
                "count": uploads.len()
            }))
        }
        Err(e) => error_result(&format!("Failed to list staged uploads: {}", e)),
    }
}

async fn execute_get_staging_status(services: &UiServices) -> ToolCallResult {
    match services.drive().get_staging_status().await {
        Ok(status) => json_result(&json!({
            "total_files": status.total_files,
            "pending_files": status.pending_files,
            "uploading_files": status.uploading_files,
            "conflicted_files": status.conflicted_files,
            "failed_files": status.failed_files,
            "completed_files": status.completed_files,
            "total_bytes": status.total_bytes,
            "bytes_uploaded": status.bytes_uploaded,
            "is_syncing": status.is_syncing,
            "network_available": status.network_available,
            "last_sync_at": status.last_sync_at,
            "last_sync_error": status.last_sync_error,
            "percent_complete": status.percent_complete(),
            "has_action_required": status.has_action_required()
        })),
        Err(e) => error_result(&format!("Failed to get staging status: {}", e)),
    }
}

async fn execute_remove_staged_upload(services: &UiServices, args: Value) -> ToolCallResult {
    let upload_id = require_str!(args, "upload_id");

    match services.drive().remove_staged_upload(&upload_id).await {
        Ok(()) => success_result(&format!("Staged upload {} removed", upload_id)),
        Err(e) => error_result(&format!("Failed to remove staged upload: {}", e)),
    }
}

async fn execute_retry_staged_upload(services: &UiServices, args: Value) -> ToolCallResult {
    let upload_id = require_str!(args, "upload_id");

    match services.drive().retry_staged_upload(&upload_id).await {
        Ok(()) => success_result(&format!(
            "Staged upload {} queued for retry",
            upload_id
        )),
        Err(e) => error_result(&format!("Failed to retry staged upload: {}", e)),
    }
}

async fn execute_resolve_staging_conflict(services: &UiServices, args: Value) -> ToolCallResult {
    use communitas_ui_api::drive::ConflictResolution;

    let upload_id = require_str!(args, "upload_id");
    let resolution_str = require_str!(args, "resolution");

    let resolution = match resolution_str.to_lowercase().as_str() {
        "keep_local" => ConflictResolution::KeepLocal,
        "keep_remote" => ConflictResolution::KeepRemote,
        "keep_both" => ConflictResolution::KeepBoth,
        "skip" => ConflictResolution::Skip,
        "retry" => ConflictResolution::Retry,
        _ => {
            return error_result(&format!(
                "Invalid resolution: {}. Use: keep_local, keep_remote, keep_both, skip, retry",
                resolution_str
            ))
        }
    };

    match services
        .drive()
        .resolve_staging_conflict(&upload_id, resolution)
        .await
    {
        Ok(()) => success_result(&format!(
            "Conflict resolved for {} with {:?}",
            upload_id, resolution
        )),
        Err(e) => error_result(&format!("Failed to resolve staging conflict: {}", e)),
    }
}

async fn execute_sync_staging_queue(services: &UiServices) -> ToolCallResult {
    match services.drive().sync_staging_queue().await {
        Ok((uploaded, failed)) => json_result(&json!({
            "files_uploaded": uploaded,
            "files_failed": failed,
            "success": failed == 0,
            "message": format!("Sync complete: {} uploaded, {} failed", uploaded, failed)
        })),
        Err(e) => error_result(&format!("Failed to sync staging queue: {}", e)),
    }
}

async fn execute_set_network_available(services: &UiServices, args: Value) -> ToolCallResult {
    let available = match args["available"].as_bool() {
        Some(v) => v,
        None => return error_result("available (boolean) is required"),
    };

    services.drive().set_network_available(available).await;
    success_result(&format!("Network availability set to {}", available))
}

// ============================================================================
// Call MCP Executor Functions
// ============================================================================

async fn execute_toggle_mute(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");
    let muted = bool_or(&args, "muted", true);

    // MCP API takes explicit muted value; use app() for direct control
    let cmd = Command::ToggleAudio {
        call_id: call_id.clone(),
        enabled: !muted, // enabled=true means unmuted, so invert
    };

    match services.call().app().execute(cmd).await {
        Ok(_) => json_result(&json!({
            "call_id": call_id,
            "muted": muted
        })),
        Err(e) => error_result(&format!("Failed to toggle mute: {}", e.message)),
    }
}

async fn execute_toggle_video(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");
    let enabled = bool_or(&args, "enabled", true);

    // MCP API takes explicit enabled value; use app() for direct control
    let cmd = Command::ToggleVideo {
        call_id: call_id.clone(),
        enabled,
    };

    match services.call().app().execute(cmd).await {
        Ok(_) => json_result(&json!({
            "call_id": call_id,
            "video_enabled": enabled
        })),
        Err(e) => error_result(&format!("Failed to toggle video: {}", e.message)),
    }
}

async fn execute_get_call_status(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");

    match services.call().query_call_status(&call_id).await {
        Ok(status) => json_result(&json!({
            "call_id": status.call_id,
            "entity_id": status.entity_id,
            "participant_count": status.participant_count,
            "started_at": status.started_at,
            "is_muted": status.is_muted,
            "is_video_enabled": status.is_video_enabled,
            "is_screen_sharing": status.is_screen_sharing
        })),
        Err(e) => error_result(&format!("Failed to get call status: {e}")),
    }
}

async fn execute_get_call_participants(services: &UiServices, args: Value) -> ToolCallResult {
    let call_id = require_str!(args, "call_id");

    match services.call().query_call_participants(&call_id).await {
        Ok(participants) => json_result(&json!({
            "participants": participants
        })),
        Err(e) => error_result(&format!("Failed to get call participants: {e}")),
    }
}

// ============================================================================
// Audit Log Tools - Security event monitoring
// ============================================================================

/// Dispatcher for audit log tools.
/// These tools require authentication to view security events.
async fn dispatch_audit_tools(
    services: &UiServices,
    name: &str,
    args: &Value,
) -> Option<ToolCallResult> {
    match name {
        "get_audit_log" => Some(execute_get_audit_log(services, args.clone()).await),
        "export_audit_log" => Some(execute_export_audit_log(services, args.clone()).await),
        _ => None,
    }
}

/// Get recent security audit events.
///
/// Returns the most recent audit events, with optional filtering by event type.
/// Events have sensitive information automatically redacted (e.g., "ocean-forest-••••").
async fn execute_get_audit_log(services: &UiServices, args: Value) -> ToolCallResult {
    use communitas_ui_service::audit::parse_event_types;

    // Parse limit (default 50, max 100)
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|v| v.min(100) as usize)
        .unwrap_or(50);

    // Parse optional event type filter
    let event_types = if let Some(types) = args.get("event_types").and_then(|v| v.as_array()) {
        let type_strings: Vec<String> = types
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        if type_strings.is_empty() {
            None
        } else {
            match parse_event_types(&type_strings) {
                Ok(types) => Some(types),
                Err(e) => return error_result(&format!("Invalid event type: {e}")),
            }
        }
    } else {
        None
    };

    // Read events from audit service
    match services.audit().read_recent(limit, event_types).await {
        Ok(events) => {
            let event_list: Vec<Value> = events
                .into_iter()
                .map(|event| {
                    let mut obj = json!({
                        "id": event.id,
                        "timestamp": event.timestamp.to_rfc3339(),
                        "event_type": format!("{}", event.event_type),
                        "identity_redacted": event.identity_redacted,
                        "device_fingerprint": event.device_fingerprint,
                        "success": event.success
                    });
                    if let Some(ref meta) = event.metadata {
                        obj["metadata"] = meta.clone();
                    }
                    obj
                })
                .collect();

            json_result(&json!({
                "events": event_list,
                "count": event_list.len(),
                "limit": limit
            }))
        }
        Err(e) => error_result(&format!("Failed to read audit log: {e}")),
    }
}

/// Export audit events within a date range.
///
/// Returns all events within the specified ISO 8601 date range for compliance
/// reporting and security reviews.
async fn execute_export_audit_log(services: &UiServices, args: Value) -> ToolCallResult {
    use communitas_ui_service::audit::parse_event_types;

    // Require start and end dates
    let start_date = match args.get("start_date").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return error_result("Missing required parameter: start_date"),
    };

    let end_date = match args.get("end_date").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return error_result("Missing required parameter: end_date"),
    };

    // Parse optional event type filter
    let event_types = if let Some(types) = args.get("event_types").and_then(|v| v.as_array()) {
        let type_strings: Vec<String> = types
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        if type_strings.is_empty() {
            None
        } else {
            match parse_event_types(&type_strings) {
                Ok(types) => Some(types),
                Err(e) => return error_result(&format!("Invalid event type: {e}")),
            }
        }
    } else {
        None
    };

    // Export events from audit service
    match services
        .audit()
        .export_range(start_date, end_date, event_types)
        .await
    {
        Ok(events) => {
            let event_list: Vec<Value> = events
                .into_iter()
                .map(|event| {
                    let mut obj = json!({
                        "id": event.id,
                        "timestamp": event.timestamp.to_rfc3339(),
                        "event_type": format!("{}", event.event_type),
                        "identity_redacted": event.identity_redacted,
                        "device_fingerprint": event.device_fingerprint,
                        "success": event.success
                    });
                    if let Some(ref meta) = event.metadata {
                        obj["metadata"] = meta.clone();
                    }
                    obj
                })
                .collect();

            json_result(&json!({
                "events": event_list,
                "count": event_list.len(),
                "start_date": start_date,
                "end_date": end_date
            }))
        }
        Err(e) => error_result(&format!("Failed to export audit log: {e}")),
    }
}

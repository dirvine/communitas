// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Tool definitions
//!
//! Exposes Communitas commands and queries as MCP tools that AI agents can invoke.

use crate::protocol::{Tool, ToolCallResult, ToolContent};
use communitas_core::{
    app::CommunitasApp,
    command::{Command, DiskTypeArg, Query, QueryResponse},
    crdt::EntityType,
};
use serde_json::{Value, json};

/// Get list of all available tools
pub fn list_tools() -> Vec<Tool> {
    vec![
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
            name: "get_kanban_board".to_string(),
            description: "Get details of a Kanban board".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "board_id": {"type": "string", "description": "Board ID"}
                },
                "required": ["board_id"]
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
            name: "list_pending_invites".to_string(),
            description: "List pending invitations for the current user".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
    ]
}

/// Execute a tool call
pub async fn call_tool(app: &CommunitasApp, name: &str, args: Option<Value>) -> ToolCallResult {
    let args = args.unwrap_or(json!({}));

    match name {
        // Entity commands
        "create_entity" => execute_create_entity(app, args).await,

        // Member commands
        "add_member" => execute_add_member(app, args).await,
        "remove_member" => execute_remove_member(app, args).await,

        // Message commands
        "send_message" => execute_send_message(app, args).await,

        // Kanban commands
        "create_kanban_board" => execute_create_board(app, args).await,
        "create_kanban_column" => execute_create_column(app, args).await,
        "create_kanban_card" => execute_create_card(app, args).await,
        "move_kanban_card" => execute_move_card(app, args).await,

        // Invite commands
        "create_invite" => execute_create_invite(app, args).await,
        "accept_invite" => execute_accept_invite(app, args).await,

        // File commands
        "write_file" => execute_write_file(app, args).await,
        "read_file" => execute_read_file(app, args).await,

        // Query commands
        "get_entity" => execute_get_entity(app, args).await,
        "list_entities" => execute_list_entities(app, args).await,
        "list_members" => execute_list_members(app, args).await,
        "get_messages" => execute_get_messages(app, args).await,
        "list_files" => execute_list_files(app, args).await,
        "get_kanban_board" => execute_get_board(app, args).await,
        "get_profile" => execute_get_profile(app).await,
        "list_pending_invites" => execute_list_pending_invites(app).await,

        _ => ToolCallResult {
            content: vec![ToolContent::Text {
                text: format!("Unknown tool: {}", name),
            }],
            is_error: true,
        },
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

fn success_result(message: &str) -> ToolCallResult {
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

// Command executors

async fn execute_create_entity(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let name = args["name"].as_str().unwrap_or_default().to_string();
    let entity_type = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing entity_type"),
    };
    let description = args["description"].as_str().map(|s| s.to_string());
    let initial_members: Vec<String> = args["initial_members"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let cmd = Command::CreateEntity {
        name,
        entity_type,
        description,
        initial_members,
    };

    match app.execute(cmd).await {
        Ok(events) => {
            let result = json!({
                "success": true,
                "events": events.len(),
                "message": "Entity created successfully"
            });
            json_result(&result)
        }
        Err(e) => error_result(&format!("Failed to create entity: {}", e.message)),
    }
}

async fn execute_add_member(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_type = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing entity_type"),
    };
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let member_id = args["member_id"].as_str().unwrap_or_default().to_string();
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
    let entity_type = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing entity_type"),
    };
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let member_id = args["member_id"].as_str().unwrap_or_default().to_string();

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
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let entity_type = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing entity_type"),
    };
    let text = args["text"].as_str().unwrap_or_default().to_string();
    let reply_to_id = args["reply_to_id"].as_str().map(|s| s.to_string());

    let cmd = Command::SendMessage {
        entity_id,
        entity_type,
        text,
        author: String::new(), // Will be filled by the service
        reply_to_id,
        attachments: None,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Message sent successfully"),
        Err(e) => error_result(&format!("Failed to send message: {}", e.message)),
    }
}

async fn execute_create_board(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let board_name = args["board_name"].as_str().unwrap_or_default().to_string();
    let description = args["description"].as_str().map(|s| s.to_string());

    let cmd = Command::CreateKanbanBoard {
        entity_id,
        board_name,
        description,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Kanban board created successfully"),
        Err(e) => error_result(&format!("Failed to create board: {}", e.message)),
    }
}

async fn execute_create_column(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = args["board_id"].as_str().unwrap_or_default().to_string();
    let column_name = args["column_name"].as_str().unwrap_or_default().to_string();
    let position = args["position"].as_i64().map(|p| p as u32);

    let cmd = Command::CreateKanbanColumn {
        board_id,
        column_name,
        position,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Kanban column created successfully"),
        Err(e) => error_result(&format!("Failed to create column: {}", e.message)),
    }
}

async fn execute_create_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = args["board_id"].as_str().unwrap_or_default().to_string();
    let column_id = args["column_id"].as_str().unwrap_or_default().to_string();
    let title = args["title"].as_str().unwrap_or_default().to_string();
    let description = args["description"].as_str().map(|s| s.to_string());
    let assignee = args["assignee"].as_str().map(|s| s.to_string());

    let cmd = Command::CreateKanbanCard {
        board_id,
        column_id,
        title,
        description,
        assignee,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Card created successfully"),
        Err(e) => error_result(&format!("Failed to create card: {}", e.message)),
    }
}

async fn execute_move_card(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = args["board_id"].as_str().unwrap_or_default().to_string();
    let card_id = args["card_id"].as_str().unwrap_or_default().to_string();
    let target_column_id = args["target_column_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let position = args["position"].as_i64().map(|p| p as u32);

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
    let recipient_id = args["recipient_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let entity_type = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing entity_type"),
    };
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let role = args["role"].as_str().unwrap_or("member").to_string();
    let message = args["message"].as_str().map(|s| s.to_string());

    let cmd = Command::CreateInvite {
        recipient_id,
        entity_type,
        entity_id,
        role,
        message,
        expires_in_hours: None,
    };

    match app.execute(cmd).await {
        Ok(_) => success_result("Invite created successfully"),
        Err(e) => error_result(&format!("Failed to create invite: {}", e.message)),
    }
}

async fn execute_accept_invite(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let invite_id = args["invite_id"].as_str().unwrap_or_default().to_string();

    let cmd = Command::AcceptInvite { invite_id };

    match app.execute(cmd).await {
        Ok(_) => success_result("Invite accepted successfully"),
        Err(e) => error_result(&format!("Failed to accept invite: {}", e.message)),
    }
}

async fn execute_write_file(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let disk_type = match args["disk_type"].as_str().and_then(parse_disk_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing disk_type"),
    };
    let path = args["path"].as_str().unwrap_or_default().to_string();
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
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();
    let disk_type = match args["disk_type"].as_str().and_then(parse_disk_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing disk_type"),
    };
    let path = args["path"].as_str().unwrap_or_default().to_string();

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
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();

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
    let entity_type = match args["entity_type"].as_str().and_then(parse_entity_type) {
        Some(t) => t,
        None => return error_result("Invalid or missing entity_type"),
    };
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();

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
    let entity_id = args["entity_id"].as_str().unwrap_or_default().to_string();

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

async fn execute_get_board(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let board_id = args["board_id"].as_str().unwrap_or_default().to_string();

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
        Err(e) => error_result(&format!("Failed to get board: {}", e.message)),
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

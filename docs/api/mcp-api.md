# Communitas MCP API

Model Context Protocol (MCP) server API for AI agents and automation.

## Overview

The MCP server exposes Communitas functionality through JSON-RPC 2.0 tools. All tools route through `UiServices` to ensure parity between AI agent operations and UI interactions.

### Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   AI Agent      │     │   Dioxus UI     │     │  MCP Client     │
│   (Claude, etc) │     │   Components    │     │  (HTTP/stdio)   │
└────────┬────────┘     └────────┬────────┘     └────────┬────────┘
         │                       │                       │
         v                       v                       v
┌─────────────────────────────────────────────────────────────────┐
│                        UiServices                                │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐       │
│  │Messaging  │ │ Drive     │ │ Kanban    │ │ Canvas    │ ...   │
│  │Service    │ │ Service   │ │ Service   │ │ Service   │       │
│  └───────────┘ └───────────┘ └───────────┘ └───────────┘       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              v
┌─────────────────────────────────────────────────────────────────┐
│                     CommunitasApp (Core)                         │
│              Commands, Queries, Event Streams                    │
└─────────────────────────────────────────────────────────────────┘
```

### MCP-Dioxus Parity

All MCP tools route through the same `UiServices` layer used by the Dioxus UI. This ensures:

- **Consistent state**: Operations produce identical results regardless of source
- **Shared validation**: Same input validation rules apply
- **Unified error handling**: Same error types and messages
- **Watch channel updates**: UI automatically reflects MCP changes via reactive signals

## Transport Options

### Stdio (Default)

```bash
cargo run -p communitas-mcp -- --demo
```

### HTTP

```bash
cargo run -p communitas-mcp -- --http --demo --listen 127.0.0.1:8080
```

### HTTPS (TLS with ML-DSA-65)

```bash
cargo run -p communitas-mcp -- --http --tls --demo --no-client-auth
```

## Authentication

### Pre-Auth Tools

The following tools are available before authentication:

| Tool | Description |
|------|-------------|
| `authenticate` | Authenticate with four-word identity and password |
| `create_vault` | Create a new identity vault |
| `authenticate_token` | Authenticate using a delegate token (for AI agents) |
| `health_check` | Check server health |
| `list_vaults` | List available vaults on this device |

### Post-Auth Tools

All other tools require authentication. The recommended flow for AI agents:

1. Use `authenticate_token` with a pre-generated delegate token
2. Or use `create_vault` followed by `authenticate`

## Tool Categories

### Messaging Tools

Route through `MessagingService` for MCP-UI parity.

| Tool | Description |
|------|-------------|
| `send_message` | Send a message to an entity |
| `get_messages` | Get messages with pagination |
| `list_threads` | List message threads |
| `list_messages` | List messages in a thread |
| `delete_message` | Delete a message |
| `edit_message` | Edit a message |
| `add_reaction` | Add emoji reaction |
| `remove_reaction` | Remove emoji reaction |
| `get_reactions` | Get reactions on a message |
| `get_available_reactions` | Get available emoji reactions |

#### Example: Send Message

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "send_message",
    "arguments": {
      "entity_id": "group-abc123",
      "content": "Hello from MCP!"
    }
  },
  "id": 1
}
```

### Drive Tools

Route through `DriveService` for MCP-UI parity.

| Tool | Description |
|------|-------------|
| `list_disks` | List virtual disks for an entity |
| `list_files` | List files in a directory |
| `read_file` | Read file contents |
| `write_file` | Write file contents |
| `delete_file` | Delete a file |
| `create_directory` | Create a directory |
| `move_file` | Move/rename a file |
| `copy_file` | Copy a file |
| `get_disk_stats` | Get disk usage statistics |
| `get_file_preview` | Get file preview (images, etc.) |

#### Example: Write File

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "write_file",
    "arguments": {
      "entity_id": "project-xyz",
      "path": "/docs/notes.md",
      "content": "# Meeting Notes\n\nAgenda items..."
    }
  },
  "id": 2
}
```

### Kanban Tools

Route through `KanbanService` for MCP-UI parity.

| Tool | Description |
|------|-------------|
| `create_kanban_board` | Create a new board |
| `get_kanban_board` | Get board details |
| `update_kanban_board` | Update board settings |
| `delete_kanban_board` | Delete a board |
| `list_kanban_boards` | List boards for an entity |
| `create_kanban_column` | Create a column |
| `get_kanban_column` | Get column details |
| `update_kanban_column` | Update column |
| `delete_kanban_column` | Delete column |
| `move_kanban_column` | Reorder columns |
| `list_kanban_columns` | List columns in a board |
| `create_kanban_card` | Create a card |
| `get_kanban_card` | Get card details |
| `update_kanban_card` | Update card |
| `delete_kanban_card` | Delete card |
| `move_kanban_card` | Move card between columns |
| `list_kanban_cards` | List cards in a column |
| `change_card_state` | Change card workflow state |
| `assign_user` | Assign user to card |
| `unassign_user` | Remove user from card |
| `create_kanban_tag` | Create a tag |
| `list_kanban_tags` | List available tags |
| `tag_card` | Add tag to card |
| `untag_card` | Remove tag from card |
| `add_step` | Add checklist step |
| `get_step` | Get step details |
| `toggle_step` | Toggle step completion |
| `delete_step` | Delete step |
| `add_comment` | Add comment to card |
| `list_comments` | List card comments |
| `delete_comment` | Delete comment |

#### Example: Create Board and Card

```json
// Create board
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_kanban_board",
    "arguments": {
      "entity_id": "project-xyz",
      "name": "Sprint 1",
      "description": "First sprint tasks"
    }
  },
  "id": 3
}

// Response: { "board_id": "board-123" }

// Create card
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "create_kanban_card",
    "arguments": {
      "entity_id": "project-xyz",
      "board_id": "board-123",
      "column_id": "col-todo",
      "title": "Implement feature X",
      "description": "Add the new feature..."
    }
  },
  "id": 4
}
```

### Canvas Tools

Route through `CanvasService` for MCP-UI parity.

| Tool | Description |
|------|-------------|
| `canvas_get_snapshot` | Get current canvas state |
| `canvas_add_text` | Add text element |
| `canvas_add_image` | Add image element |
| `canvas_add_chart` | Add chart element |
| `canvas_remove_element` | Remove element |
| `canvas_update_transform` | Update element position/scale/rotation |
| `canvas_select_element` | Select an element |
| `canvas_deselect_all` | Clear selection |
| `canvas_set_viewport` | Set viewport dimensions |
| `canvas_set_view` | Set zoom/pan |
| `canvas_clear` | Clear all elements |
| `canvas_export` | Export canvas as JSON |
| `canvas_import` | Import canvas from JSON |
| `canvas_element_at` | Get element at coordinates |

#### Example: Add Text to Canvas

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "canvas_add_text",
    "arguments": {
      "entity_id": "whiteboard-123",
      "content": "Meeting Notes",
      "x": 100.0,
      "y": 50.0
    }
  },
  "id": 5
}
```

### Call/WebRTC Tools

Route through `CallService` for MCP-UI parity.

#### Call Management

| Tool | Description |
|------|-------------|
| `start_voice_call` | Start a voice or video call for an entity |
| `join_call` | Join an active call by ID |
| `end_call` | End or leave a call |

#### Media Controls

| Tool | Description |
|------|-------------|
| `toggle_mute` | Toggle audio mute state in a call |
| `toggle_video` | Toggle video state in a call |
| `share_screen` | Start or stop screen sharing in a call |
| `list_media_devices` | List available microphones, speakers, and cameras |

#### Call Status

| Tool | Description |
|------|-------------|
| `get_call_status` | Get current call status including mute/video state |
| `get_call_participants` | Get list of participants in a call |
| `get_call_quality` | Get quality metrics for the current call |

#### Call History

| Tool | Description |
|------|-------------|
| `get_call_history` | Get call history with optional filtering |
| `get_missed_calls` | Get missed call notifications |
| `acknowledge_missed_call` | Acknowledge a missed call notification |

#### Call Recording

| Tool | Description |
|------|-------------|
| `get_call_recording` | Get recording status and info for the current call |
| `start_call_recording` | Start recording the current call |
| `stop_call_recording` | Stop recording the current call |

#### Example: Start Voice Call

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "start_voice_call",
    "arguments": {
      "entity_id": "group-abc123",
      "video_enabled": false
    }
  },
  "id": 1
}
```

#### Example: Get Call History

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_call_history",
    "arguments": {
      "limit": 10,
      "call_type": "group"
    }
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"entries\": [{\"id\": \"call-123\", \"call_type\": \"group\", \"outcome\": \"completed\", \"participants\": [...], \"started_at\": 1706000000, \"duration_secs\": 1800}]}"
    }]
  },
  "id": 2
}
```

#### Example: Get Call Quality Metrics

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_call_quality",
    "arguments": {}
  },
  "id": 3
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"rtt_ms\": 45, \"jitter_ms\": 3, \"packet_loss_percent\": 0.1, \"bitrate_kbps\": 2500, \"connection_quality\": \"excellent\"}"
    }]
  },
  "id": 3
}
```

#### Call Tool Limitations

Note that MCP call tools can manage call state but cannot access actual media streams:
- Media capture/playback requires platform integration
- Screen sharing source selection happens via platform host
- Recording requires local file system access

### Entity Management

| Tool | Description |
|------|-------------|
| `create_entity` | Create group/channel/project |
| `get_entity` | Get entity details |
| `update_entity` | Update entity settings |
| `delete_entity` | Delete entity |
| `list_entities` | List entities |
| `join_entity` | Join an entity |
| `add_member` | Add member to entity |
| `remove_member` | Remove member |
| `list_members` | List entity members |

### Contact Management

| Tool | Description |
|------|-------------|
| `create_contact` | Add a contact |
| `get_contact` | Get contact details |
| `update_contact` | Update contact |
| `delete_contact` | Delete contact |
| `list_contacts` | List all contacts |
| `search_contacts` | Search contacts |
| `link_contact` | Link contact to identity |
| `set_favourite_contact` | Mark as favourite |
| `remove_favourite_contact` | Remove from favourites |
| `list_favourite_contacts` | List favourites |
| `get_contact_presence` | Get contact online status |

### Presence

| Tool | Description |
|------|-------------|
| `set_presence` | Set your presence status |
| `get_presence` | Get user presence |
| `set_my_presence` | Update your status |
| `subscribe_to_presence` | Subscribe to presence updates |
| `announce_presence` | Announce presence to network |
| `query_presence` | Query presence for identity |
| `get_our_presence` | Get our current presence |
| `get_cached_presence` | Get cached presence |

### Network

| Tool | Description |
|------|-------------|
| `network_start` | Start P2P networking |
| `network_stop` | Stop networking |
| `network_connect` | Connect to peer |
| `network_status` | Get network status |
| `network_peers` | List connected peers |
| `network_request_external_address` | Request STUN address |
| `get_connection_words` | Get our connection words |
| `connect_by_words` | Connect using words |

### Identity & Auth

| Tool | Description |
|------|-------------|
| `create_identity` | Create new identity with mnemonic |
| `recover_identity` | Recover from mnemonic |
| `validate_mnemonic` | Validate recovery phrase |
| `get_session` | Get current session |
| `logout` | End session |
| `get_profile` | Get user profile |
| `update_profile` | Update profile |
| `export_vault` | Export vault backup |
| `import_vault` | Import vault |
| `delete_vault` | Delete vault |
| `create_delegate_token` | Create AI agent token |

### Utility

| Tool | Description |
|------|-------------|
| `health_check` | Server health check |
| `core_status` | Core system status |
| `workspace_init` | Initialize workspace |

## Error Handling

All tools return structured errors in the MCP response format:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "isError": true,
    "content": [
      {
        "type": "text",
        "text": "Failed to create board: not authenticated"
      }
    ]
  },
  "id": 1
}
```

### Common Error Messages

| Error | Cause | Resolution |
|-------|-------|------------|
| `not authenticated` | Session expired or missing | Call `authenticate` or `authenticate_token` |
| `entity not found` | Invalid entity_id | Verify entity exists with `get_entity` |
| `board not found` | Invalid board_id | Verify board exists with `get_kanban_board` |
| `permission denied` | User lacks access | Check entity membership |
| `validation failed` | Missing required fields | Check tool parameters |

## Watch Channel Integration

MCP operations trigger the same watch channel updates as UI operations:

1. **MessagingService**: Thread list, message list updates
2. **DriveService**: File list, disk stats updates
3. **KanbanService**: Board/column/card snapshot updates
4. **CanvasService**: Element updates, selection changes
5. **CallService**: Call status, participant updates

This means UI components automatically reflect MCP changes via reactive signals, enabling real-time collaboration between AI agents and human users.

## Behavioral Notes

### Intentional Differences

Some operations behave differently in MCP context:

1. **File operations**: MCP uses string content; UI may use binary streams
2. **Canvas transforms**: MCP uses absolute coordinates; UI may use relative
3. **Call media**: MCP can toggle settings but cannot access actual media streams

### Rate Limiting

MCP operations are subject to the same rate limits as UI operations:
- Message sends: 10/second per entity
- File writes: 5/second per entity
- Board modifications: 20/second per entity

## Testing

Parity tests verify MCP tools route correctly through UiServices:

```bash
cargo test -p communitas-mcp --test parity_test
```

Tests cover:
- Tool registration verification
- Service routing validation
- Error consistency checks
- Auth state handling

## See Also

- [Core API Documentation](core-api.md)
- [MCP README](../../communitas-mcp/README.md)
- [ADR-018: MCP External Integration](../architecture/adr-018-mcp-external-integration.md)

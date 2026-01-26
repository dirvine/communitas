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

#### Element Management

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

#### History & Sync

| Tool | Description |
|------|-------------|
| `canvas_undo` | Undo the last canvas operation |
| `canvas_redo` | Redo the last undone operation |
| `canvas_get_history` | Get the operation history timeline |
| `canvas_broadcast_cursor` | Broadcast cursor position to collaborators |
| `canvas_get_remote_cursors` | Get all visible remote cursors |
| `canvas_flush_offline_queue` | Flush pending offline operations to network |

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

#### Example: Undo/Redo

```json
// Undo last operation
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "canvas_undo",
    "arguments": {
      "entity_id": "whiteboard-123"
    }
  },
  "id": 6
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"undone\": true, \"operation\": {\"id\": \"op-123\", \"description\": \"Add text element\", \"timestamp_ms\": 1706000000}}"
    }]
  },
  "id": 6
}
```

#### Example: Get Remote Cursors

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "canvas_get_remote_cursors",
    "arguments": {
      "entity_id": "whiteboard-123"
    }
  },
  "id": 7
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"cursors\": [{\"user_id\": \"alice-123\", \"user_name\": \"Alice\", \"x\": 250.5, \"y\": 180.0, \"color\": \"#FF5722\", \"tool\": \"select\", \"last_active_ms\": 1706000000}], \"count\": 1}"
    }]
  },
  "id": 7
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
4. **CanvasService**: Element updates, selection changes, history updates, remote cursors
5. **CallService**: Call status, participant updates

This means UI components automatically reflect MCP changes via reactive signals, enabling real-time collaboration between AI agents and human users.

### Canvas Sync Details

Canvas operations support collaborative editing with:
- **CRDT-based sync**: Operations merge without conflicts using Yrs
- **Undo/redo history**: Full operation timeline with entity-scoped history
- **Remote cursors**: Real-time cursor position sharing (throttled to 10 Hz)
- **Offline queue**: Operations queued when offline, flushed on reconnection

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

## MCP Apps (Interactive UIs)

Communitas implements the [MCP Apps extension](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/) (SEP-1865), enabling interactive UI widgets directly within AI conversations in Claude Desktop, ChatGPT, VS Code, and other MCP-compatible hosts.

### Overview

MCP Apps transforms the MCP server from a headless automation API into a full app platform. Interactive HTML/CSS/JS widgets render in sandboxed iframes within the MCP host, communicating with the server via postMessage JSON-RPC.

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Host Application                      │
│                (Claude Desktop / ChatGPT / VS Code)          │
├─────────────────────────────────────────────────────────────┤
│    ┌──────────────────────────────────────────────────┐     │
│    │              Sandboxed Iframe                     │     │
│    │  ┌────────────────────────────────────────────┐  │     │
│    │  │         Communitas UI Widget               │  │     │
│    │  │  (contacts/messages/kanban/drive/canvas)   │  │     │
│    │  └────────────────────────────────────────────┘  │     │
│    └────────────────────┬─────────────────────────────┘     │
│                         │ postMessage JSON-RPC              │
└─────────────────────────┼───────────────────────────────────┘
                          │
┌─────────────────────────┴───────────────────────────────────┐
│                   Communitas MCP Server                      │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              communitas-ui-service                   │    │
│  │         (shared layer - ADR-019)                     │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

### UI Resources

UI widgets are exposed as resources with the `ui://` scheme. The server advertises these in `resources/list` and serves them via `resources/read`.

#### Available Widgets

| Resource URI | Widget | Description |
|--------------|--------|-------------|
| `ui://communitas/contacts` | Contacts | Interactive contact list with search and favorites |
| `ui://communitas/messages` | Messages | Thread navigation and message composition |
| `ui://communitas/kanban` | Kanban | Drag-drop project boards |
| `ui://communitas/drive` | Drive | File browser with upload and preview |
| `ui://communitas/canvas` | Canvas | Collaborative whiteboard viewer |

#### Resource List Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "resources": [
      {
        "uri": "ui://communitas/contacts",
        "name": "Contacts",
        "description": "Interactive contact list with search and favorites",
        "mimeType": "text/html;profile=mcp-app",
        "_meta": {
          "ui": {
            "csp": {
              "connectDomains": [],
              "resourceDomains": []
            }
          }
        }
      }
    ]
  },
  "id": 1
}
```

#### Reading UI Resources

```json
{
  "jsonrpc": "2.0",
  "method": "resources/read",
  "params": {
    "uri": "ui://communitas/contacts"
  },
  "id": 2
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "contents": [
      {
        "uri": "ui://communitas/contacts",
        "mimeType": "text/html;profile=mcp-app",
        "text": "<!DOCTYPE html>..."
      }
    ]
  },
  "id": 2
}
```

### Tool UI Enhancement

Tools can return `_meta.ui` to indicate that an interactive UI should be rendered alongside the tool response.

#### Tool Response with UI Metadata

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"contacts\": [{\"id\": \"abc\", \"name\": \"Alice\"}]}"
      }
    ],
    "isError": false,
    "_meta": {
      "ui": {
        "resourceUri": "ui://communitas/contacts",
        "visibility": ["model", "app"]
      }
    }
  },
  "id": 1
}
```

#### Visibility Scopes

- `model`: The model can see and reference the UI content
- `app`: The host application renders the UI to the user
- Both: UI shown to user, model can reference it in conversation

#### Enhanced Tools

The following tools include `_meta.ui` to provide interactive rendering:

| Tool | UI Resource | Behavior |
|------|-------------|----------|
| `list_contacts` | `ui://communitas/contacts` | Shows contact list widget |
| `get_contact` | `ui://communitas/contacts` | Shows contact detail |
| `list_threads` | `ui://communitas/messages` | Shows thread list widget |
| `list_messages` | `ui://communitas/messages` | Shows message thread |
| `send_message` | `ui://communitas/messages` | Shows updated thread |
| `list_kanban_boards` | `ui://communitas/kanban` | Shows board selector |
| `get_kanban_board` | `ui://communitas/kanban` | Shows full board |
| `list_files` | `ui://communitas/drive` | Shows file browser |
| `list_disks` | `ui://communitas/drive` | Shows disk selector |
| `canvas_get_snapshot` | `ui://communitas/canvas` | Shows canvas viewer |
| `canvas_get_history` | `ui://communitas/canvas` | Shows history timeline |

### postMessage Protocol

UI widgets communicate with the MCP server through the host application using JSON-RPC over postMessage.

#### Initialization Handshake

When a UI widget loads, it establishes a connection with the host:

```javascript
// Widget sends initialization
window.parent.postMessage({
  jsonrpc: '2.0',
  method: 'ui/initialize',
  params: {
    resourceUri: 'ui://communitas/contacts',
    capabilities: {
      toolCalls: true,
      resourceReads: true,
      modelMessages: true
    }
  }
}, '*');

// Host responds with session info
// (received via message event)
{
  jsonrpc: '2.0',
  result: {
    sessionId: 'session-123',
    serverCapabilities: {
      toolsAvailable: true,
      resourcesAvailable: true
    }
  }
}
```

#### Calling Tools from UI

Widgets can invoke MCP tools through the bridge:

```javascript
// Request
window.parent.postMessage({
  jsonrpc: '2.0',
  id: 'req-1',
  method: 'tools/call',
  params: {
    name: 'list_contacts',
    arguments: { limit: 50 }
  }
}, '*');

// Response (via message event)
{
  jsonrpc: '2.0',
  id: 'req-1',
  result: {
    content: [{ type: 'text', text: '{"contacts": [...]}' }]
  }
}
```

#### Reading Resources from UI

```javascript
// Request
window.parent.postMessage({
  jsonrpc: '2.0',
  id: 'req-2',
  method: 'resources/read',
  params: {
    uri: 'entity://group/abc123'
  }
}, '*');
```

#### Sending Messages to Model Context

Widgets can update the model's context with relevant information:

```javascript
// Inform the model about user selection
window.parent.postMessage({
  jsonrpc: '2.0',
  method: 'ui/message',
  params: {
    content: 'User selected contact: Alice (alice-123)'
  }
}, '*');
```

### MCP Bridge Library

Communitas provides a JavaScript bridge library for widget development:

```html
<script src="../shared/mcp-bridge.js"></script>
<script>
const bridge = new McpBridge();

// Wait for connection
bridge.onReady(async () => {
  // Call tools
  const contacts = await bridge.callTool('list_contacts', { limit: 50 });

  // Read resources
  const entity = await bridge.readResource('entity://group/abc123');

  // Update model context
  bridge.sendMessage('User selected: Alice');
});

// Handle tool input events (when model calls tools)
bridge.onToolInput((toolName, args) => {
  console.log(`Model called ${toolName} with`, args);
});

// Handle tool results
bridge.onToolResult((toolName, result) => {
  console.log(`Tool ${toolName} returned`, result);
});
</script>
```

### Capability Negotiation

The server advertises MCP Apps support in the initialize response:

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "clientInfo": { "name": "claude-desktop", "version": "1.0" }
  },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {
      "tools": { "listChanged": false },
      "resources": { "subscribe": false, "listChanged": false }
    },
    "extensions": {
      "io.modelcontextprotocol/ui": {
        "mimeTypes": ["text/html;profile=mcp-app"]
      }
    },
    "serverInfo": {
      "name": "communitas-mcp",
      "version": "0.1.0"
    }
  },
  "id": 1
}
```

### Content Security Policy (CSP)

Each UI resource specifies its CSP requirements in the `_meta.ui.csp` field:

```json
{
  "uri": "ui://communitas/contacts",
  "mimeType": "text/html;profile=mcp-app",
  "_meta": {
    "ui": {
      "csp": {
        "connectDomains": [],
        "resourceDomains": []
      }
    }
  }
}
```

Communitas widgets are self-contained with no external dependencies:
- All resources embedded via `include_str!()`
- No external network requests required
- No external stylesheets or scripts

This ensures widgets work in restrictive CSP environments.

### Security Model

1. **Iframe Sandboxing**: All widgets run in sandboxed iframes controlled by the MCP host
2. **Origin Restriction**: postMessage communication restricted to parent window
3. **CSP Enforcement**: Widgets declare required domains; hosts can reject overly permissive requests
4. **Visibility Scopes**: Tools control whether UI is visible to model, app, or both
5. **No Direct Network**: Widgets cannot make direct network requests; all data flows through MCP tools

### Building Custom Widgets

To create a new UI widget for Communitas:

#### 1. Create the HTML Bundle

```html
<!-- communitas-mcp/ui-bundles/my-widget/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>My Widget</title>
  <link rel="stylesheet" href="../shared/styles.css">
</head>
<body>
  <div id="app" class="loading">Loading...</div>

  <script src="../shared/mcp-bridge.js"></script>
  <script>
    const bridge = new McpBridge();
    const app = document.getElementById('app');

    bridge.onReady(async () => {
      app.classList.remove('loading');
      // Initialize widget
      const data = await bridge.callTool('my_tool', {});
      renderWidget(data);
    });

    function renderWidget(data) {
      app.innerHTML = `<div class="card">${data.content}</div>`;
    }
  </script>
</body>
</html>
```

#### 2. Register the UI Resource

```rust
// In ui_resources.rs
impl UiResourceRegistry {
    fn register_standard_widgets(&mut self) {
        // ... existing widgets ...

        self.register(UiResourceEntry::new_inline(
            "ui://communitas/my-widget",
            "My Widget",
            "Description of my widget",
            include_str!("../ui-bundles/my-widget/index.html"),
        ));
    }
}
```

#### 3. Enhance Tools with UI Metadata

```rust
// In tools.rs
fn handle_my_tool(&self, args: MyToolArgs) -> Result<ToolCallResult, Error> {
    let result = self.service.do_something(args)?;

    Ok(ToolCallResultWithMeta {
        content: vec![TextContent::new(serde_json::to_string(&result)?)],
        is_error: false,
        meta: Some(ToolResultMeta {
            ui: Some(McpUiToolMeta {
                resource_uri: Some("ui://communitas/my-widget".to_string()),
                visibility: vec!["model".to_string(), "app".to_string()],
            }),
        }),
    })
}
```

#### 4. Test the Widget

```bash
# Start MCP server
cargo run -p communitas-mcp -- --http --demo

# Test resource listing
curl -X POST http://localhost:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"resources/list"}'

# Test resource reading
curl -X POST http://localhost:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"ui://communitas/my-widget"}}'
```

## See Also

- [Core API Documentation](core-api.md)
- [MCP README](../../communitas-mcp/README.md)
- [ADR-018: MCP External Integration](../architecture/adr-018-mcp-external-integration.md)
- [ADR-022: MCP Apps Integration](../adr/ADR-022-mcp-apps-integration.md)

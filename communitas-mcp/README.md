# Communitas MCP Server

Model Context Protocol (MCP) server for AI agents and automation. Communicates via JSON-RPC 2.0 over stdio or HTTP/HTTPS.

## Features

- **187 MCP tools** covering all Communitas functionality
- **MCP Apps extension (SEP-1865)** with 8 interactive UI widgets
- JSON-RPC 2.0 over stdio (default) or HTTP/HTTPS
- TLS with ML-DSA-65 post-quantum raw public keys
- Demo mode for testing without real identities
- Full parity with Dioxus UI via shared `communitas-ui-service`

## Usage

### Stdio (default)

```bash
cargo run -p communitas-mcp -- --demo
```

### HTTP

```bash
cargo run -p communitas-mcp -- --http --demo
```

### HTTPS (TLS)

```bash
cargo run -p communitas-mcp -- --http --tls --demo --no-client-auth
```

## CLI Flags

- `--demo` : Auto-initialize a temporary identity (skips auth). Use for dev only.
- `--storage-dir <path>` : Storage dir for demo mode.
- `--four-words <id>` : Use a specific four-word identity in demo mode.
- `--display-name <name>` : Display name for demo session.
- `--http` : Serve MCP over HTTP (`POST /mcp`).
- `--tls` : Enable HTTPS with ML-DSA-65 raw public keys (requires `--http`).
- `--listen <addr>` : Override listen address (default 127.0.0.1:8080 / 8443).
- `--no-client-auth` : Disable client cert verification (TLS only, dev only).

## Protocol

- Input: JSON-RPC 2.0 requests
- Output: JSON-RPC 2.0 responses
- Logging: stderr

## MCP Apps (Interactive UIs)

Communitas implements the [MCP Apps extension](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/) (SEP-1865), enabling interactive UI widgets within AI conversations.

### Available UI Resources

| Resource URI | Widget | Description |
|--------------|--------|-------------|
| `ui://communitas/contacts` | Contacts | Interactive contact list with search and favorites |
| `ui://communitas/messages` | Messages | Thread navigation and message composition |
| `ui://communitas/kanban` | Kanban | Drag-drop project boards |
| `ui://communitas/drive` | Drive | File browser with upload and preview |
| `ui://communitas/canvas` | Canvas | Collaborative whiteboard viewer |
| `ui://communitas/settings` | Settings | User preferences and configuration |
| `ui://communitas/search` | Search | Global search across all entities |
| `ui://communitas/notifications` | Notifications | Activity feed and alerts |

### Capability Negotiation

The server advertises MCP Apps support via the `extensions` field:

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": { "tools": {}, "resources": {} },
  "extensions": {
    "io.modelcontextprotocol/ui": {
      "mimeTypes": ["text/html;profile=mcp-app"]
    }
  }
}
```

### Tool UI Enhancement

Tools can return `_meta.ui.resourceUri` to trigger interactive rendering:

```json
{
  "content": [{"type": "text", "text": "{...}"}],
  "isError": false,
  "_meta": {
    "ui": {
      "resourceUri": "ui://communitas/contacts",
      "visibility": ["model", "app"]
    }
  }
}
```

### Testing UI Resources

```bash
# Start server
cargo run -p communitas-mcp -- --http --demo

# List UI resources
curl -X POST http://localhost:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"resources/list"}'

# Read a UI widget
curl -X POST http://localhost:3040/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"resources/read","params":{"uri":"ui://communitas/contacts"}}'
```

## Tool Categories

| Category | Tool Count | Description |
|----------|------------|-------------|
| Messaging | 10 | Threads, messages, reactions |
| Drive | 10 | Files, directories, previews |
| Kanban | 21 | Boards, columns, cards, tags |
| Canvas | 14 | Elements, history, collaboration |
| Call/WebRTC | 13 | Calls, media, recording |
| Contacts | 11 | Contact management |
| Entities | 9 | Groups, channels, projects |
| Presence | 8 | Online status |
| Network | 8 | P2P connectivity |
| Identity | 11 | Auth, profiles, vaults |
| Utility | 3 | Health, status |
| **Total** | **187** | |

See [docs/api/mcp-api.md](../docs/api/mcp-api.md) for full API documentation.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MCP Host Application                      │
│                (Claude Desktop / ChatGPT / VS Code)          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│    ┌──────────────────────────────────────────────────┐     │
│    │              Sandboxed Iframe                     │     │
│    │  ┌────────────────────────────────────────────┐  │     │
│    │  │         Communitas UI Widget               │  │     │
│    │  │  (contacts/messages/kanban/drive/canvas/  │  │     │
│    │  │   settings/search/notifications)          │  │     │
│    │  └────────────────────────────────────────────┘  │     │
│    └────────────────────┬─────────────────────────────┘     │
│                         │ postMessage JSON-RPC              │
└─────────────────────────┼───────────────────────────────────┘
                          │ stdio / HTTPS
┌─────────────────────────┴───────────────────────────────────┐
│                   Communitas MCP Server                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌────────────────────┐  │
│  │ protocol.rs │  │ server.rs   │  │ ui_resources.rs    │  │
│  │ (_meta.ui   │  │ (routing)   │  │ (ui:// registry)   │  │
│  │  types)     │  │             │  │                    │  │
│  └─────────────┘  └─────────────┘  └────────────────────┘  │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │                  communitas-ui-service                  │ │
│  │  (shared service layer - ADR-019)                       │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Readiness

See `docs/MCP_PRODUCTION_READINESS_REPORT.md` for current production readiness and gaps.

## Related Documentation

- [MCP API Reference](../docs/api/mcp-api.md) - Full API documentation
- [ADR-018: MCP External Integration](../docs/adr/ADR-018-mcp-external-integration.md) - Integration architecture
- [ADR-019: Shared UI Service](../docs/adr/ADR-019-shared-rust-ui-service.md) - Service layer design
- [ADR-022: MCP Apps Integration](../docs/adr/ADR-022-mcp-apps-integration.md) - MCP Apps architecture

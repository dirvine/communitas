# ADR-022: MCP Apps Integration

## Status

Accepted

## Context

Communitas has a comprehensive MCP (Model Context Protocol) server with 197 tools exposing all platform functionality to AI agents. The MCP server supports both stdio and HTTPS transports with ML-DSA-65 post-quantum TLS.

On January 26, 2026, the Model Context Protocol team announced [MCP Apps](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/), an extension that allows MCP servers to provide interactive UI components that render within MCP host applications (Claude Desktop, ChatGPT, VS Code Copilot).

This creates an opportunity: rather than having two separate presentation layers (Dioxus native and AI agents), we can unify them through MCP Apps. The MCP server becomes the app platform, with both Dioxus and MCP hosts as consumers.

## Decision

We will implement the MCP Apps extension (SEP-1865) to provide interactive UI widgets for Communitas functionality. This complements our existing shared service architecture (ADR-019) where both Dioxus and MCP use the same `communitas-ui-service` layer.

### Key Design Choices

1. **Protocol Layer**: Extend `protocol.rs` with `_meta.ui` types for tool responses and resource definitions

2. **UI Resource Registry**: New `ui_resources.rs` module to manage `ui://` resources with embedded HTML bundles

3. **UI Bundles**: Vanilla HTML/CSS/JS widgets embedded via `include_str!()` for zero-dependency deployment

4. **postMessage Bridge**: JavaScript library for bidirectional JSON-RPC communication between UI and MCP server

5. **Eight Interactive Widgets**:
   - Contacts: Interactive contact list with search and favorites
   - Messages: Thread navigation and message composition
   - Kanban: Drag-drop project boards
   - Drive: File browser with upload and preview
   - Canvas: Collaborative whiteboard viewer
   - Settings: User preferences and configuration
   - Search: Global search across all entities
   - Notifications: Activity feed and alerts

6. **Strict MCP Apps Compatibility**:
   - UI extension advertised at top-level `extensions` (not nested in `capabilities`).
   - UI widgets must call `ui/initialize` and use the returned `sessionToken`.
   - `resources/list` returns `_meta.ui` (CSP + permissions) for `ui://` resources.

### Architecture

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
│                         │ postMessage                        │
│                         │ JSON-RPC                           │
│    ┌────────────────────┴─────────────────────────────┐     │
│    │              MCP Apps Bridge                      │     │
│    │    (tools/call, resources/read, ui/message)      │     │
│    └────────────────────┬─────────────────────────────┘     │
│                         │                                    │
└─────────────────────────┼────────────────────────────────────┘
                          │ stdio / HTTPS
┌─────────────────────────┴────────────────────────────────────┐
│                   Communitas MCP Server                       │
├───────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌────────────────────┐   │
│  │ protocol.rs │  │ server.rs   │  │ ui_resources.rs    │   │
│  │ (_meta.ui   │  │ (routing)   │  │ (ui:// registry)   │   │
│  │  types)     │  │             │  │                    │   │
│  └─────────────┘  └─────────────┘  └────────────────────┘   │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                  communitas-ui-service                  │  │
│  │  (shared service layer - ADR-019)                       │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                               │
│  ┌────────────────────────────────────────────────────────┐  │
│  │                  communitas-core                        │  │
│  │  (business logic, P2P, cryptography)                    │  │
│  └────────────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────────────┘
```

### Protocol Extension

Tools and resources can include `_meta.ui` to indicate interactive rendering support:

```json
// Tool response with UI metadata
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

// UI Resource in resources/list
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
      },
      "permissions": []
    }
  }
}
```

### Capability Negotiation

The server advertises MCP Apps support in the initialize response:

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "tools": {"listChanged": false},
    "resources": {"subscribe": false, "listChanged": false}
  },
  "extensions": {
    "io.modelcontextprotocol/ui": {
      "mimeTypes": ["text/html;profile=mcp-app"]
    }
  }
}
```

### UI Session Tokens

UI widgets must call `ui/initialize` and include the returned `sessionToken` in
`ui/context` and `ui/message`. Tokens are short-lived and scoped to the UI bridge.
They are **not** a user auth token.

Pre-auth `ui://` resources are allowed for login screens, but must avoid exposing
model-visible secrets.

### Security Model

1. **Iframe Sandboxing**: All UI widgets run in sandboxed iframes controlled by the MCP host
2. **Content Security Policy**: Each UI resource can specify allowed domains for network and resource access
3. **Visibility Scopes**: Tools can specify whether UI is visible to the model, app, or both
4. **UI Session Tokens**: `ui/initialize` issues short-lived tokens for UI-only channels
5. **postMessage Origin**: Communication restricted to parent window

## Consequences

### Positive

1. **Unified Platform**: MCP server becomes the central app platform, reducing duplication
2. **Broader Reach**: Communitas available in Claude Desktop, ChatGPT, VS Code without separate builds
3. **AI-Native UX**: UI designed for AI conversation context from the start
4. **Consistent State**: Same `communitas-ui-service` ensures Dioxus and MCP Apps behave identically
5. **Zero Dependencies**: Embedded bundles mean no external assets or CDN requirements

### Negative

1. **Limited Capabilities**: Iframe sandbox restricts some native functionality (filesystem, notifications)
2. **Styling Constraints**: Must work within MCP host styling context
3. **Learning Curve**: Teams need to understand postMessage JSON-RPC patterns
4. **Testing Complexity**: Need to test across multiple MCP host implementations

### Neutral

1. **Dioxus Relationship**: Dioxus remains the "native" experience; MCP Apps is the "universal" experience
2. **Bundle Size**: Embedded HTML bundles add ~100KB to binary but simplify deployment
3. **Evolution**: MCP Apps specification will evolve; we track upstream changes

## Implementation

### Phase 1: Protocol Foundation
- Add `_meta.ui` types to `protocol.rs`
- Extend capability negotiation with UI extension
- Create `ui_resources.rs` registry

### Phase 2: UI Widgets
- Create `mcp-bridge.js` postMessage library
- Build eight widgets (contacts, messages, kanban, drive, canvas, settings, search, notifications)
- Embed bundles via `include_str!()`

### Phase 3: Documentation
- Update `mcp-api.md` with MCP Apps sections
- Update README with capability announcement
- Create test suite for UI resource handling
- Document the CRDT-aware tool checklist for MCP parity

### Phase 4: Production Hardening
- Security audit of CSP configuration
- Performance optimization of bundle loading
- Multi-host validation (Claude, ChatGPT, VS Code)

## References

- [MCP Apps Announcement](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/)
- [SEP-1865: MCP UI Extension](https://github.com/modelcontextprotocol/specification/issues/1865)
- [ADR-018: MCP External Integration](./ADR-018-mcp-external-integration.md)
- [ADR-019: Shared Rust UI Service](./ADR-019-shared-rust-ui-service.md)
- [communitas-mcp API Documentation](../api/mcp-api.md)
- [MCP CRDT-aware tool checklist](../architecture/mcp-crdt-tool-checklist.md)

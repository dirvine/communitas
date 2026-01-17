# ADR-018: MCP External Integration Architecture

## Status

Accepted (2025-01-15)

## Context

### The Problem

Communitas needs to support two distinct integration patterns:

1. **Internal UI**: Flutter app needs fast, direct access to Rust core
2. **External agents**: AI tools (Claude, saorsa-canvas, custom agents) need network access

The Model Context Protocol (MCP) provides a standard interface for AI agents, but we need to define:
- How the MCP server runs (embedded vs. standalone)
- Authentication for external access
- Integration patterns for specific tools like saorsa-canvas

### Requirements

- AI agents can control Communitas via JSON-RPC 2.0
- Support both stdio (Claude Code) and HTTP/HTTPS transport
- Secure authentication for localhost and remote access
- Enable saorsa-canvas to render Communitas data visually
- Support delegate tokens for scoped access

### Existing MCP Implementation

`communitas-mcp` already supports:
- Stdio transport (default, for Claude Code)
- HTTP transport (`--http` flag)
- HTTPS with ML-DSA-65 raw public keys (`--tls` flag)
- Demo mode for testing (`--demo` flag)
- Delegate tokens for scoped access

## Decision

### MCP Server Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     MCP Integration Architecture                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    External AI Agents                               │ │
│  │                                                                      │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │ │
│  │  │ Claude Code  │  │ saorsa-      │  │ Custom MCP Clients       │  │ │
│  │  │ (stdio)      │  │ canvas       │  │ (HTTP/HTTPS)             │  │ │
│  │  └──────┬───────┘  └──────┬───────┘  └────────────┬─────────────┘  │ │
│  │         │                  │                       │                │ │
│  └─────────┼──────────────────┼───────────────────────┼────────────────┘ │
│            │                  │                       │                  │
│            │ stdio            │ HTTP/HTTPS            │ HTTP/HTTPS       │
│            │                  │ :3040                 │ :8443 (TLS)      │
│            │                  │                       │                  │
│            ▼                  ▼                       ▼                  │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                    communitas-mcp Server                             ││
│  │                                                                       ││
│  │  ┌───────────────┐   ┌───────────────┐   ┌───────────────────────┐  ││
│  │  │ stdio server  │   │ HTTP server   │   │ HTTPS server (TLS)    │  ││
│  │  │ (default)     │   │ (--http)      │   │ (--http --tls)        │  ││
│  │  └───────┬───────┘   └───────┬───────┘   └───────────┬───────────┘  ││
│  │          │                   │                       │               ││
│  │          └───────────────────┴───────────────────────┘               ││
│  │                              │                                        ││
│  │                              ▼                                        ││
│  │  ┌─────────────────────────────────────────────────────────────────┐ ││
│  │  │                   Authentication Layer                           │ ││
│  │  │                                                                   │ ││
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │ ││
│  │  │  │ Vault Auth  │  │ Demo Mode   │  │ Delegate Tokens         │  │ ││
│  │  │  │ (passphrase)│  │ (--demo)    │  │ (scoped access)         │  │ ││
│  │  │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │ ││
│  │  └─────────────────────────────────────────────────────────────────┘ ││
│  │                              │                                        ││
│  │                              ▼                                        ││
│  │  ┌─────────────────────────────────────────────────────────────────┐ ││
│  │  │                    MCP Tools & Resources                         │ ││
│  │  │                                                                   │ ││
│  │  │  Tools: entity_*, message_*, invite_*, gossip_*, webrtc_*        │ ││
│  │  │  Resources: identity://, entity://, presence://                  │ ││
│  │  └─────────────────────────────────────────────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                              │                                           │
│                              ▼                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                       communitas-core                                ││
│  │                                                                       ││
│  │                   CommunitasApp / CoreContext                        ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Transport Modes

| Mode | Command | Use Case |
|------|---------|----------|
| **stdio** | `communitas-mcp --demo` | Claude Code integration |
| **HTTP** | `communitas-mcp --http --demo` | Local development, testing |
| **HTTPS** | `communitas-mcp --http --tls` | Production, remote access |

### Authentication Methods

#### 1. Vault Authentication (Primary)

User authenticates with their identity passphrase:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "authenticate",
    "arguments": {
      "four_words": "ocean-forest-moon-star",
      "passphrase": "user-passphrase"
    }
  }
}
```

#### 2. Demo Mode (Development)

Auto-authenticate with temporary identity:

```bash
# Start MCP in demo mode (creates temp identity)
communitas-mcp --http --demo

# Or with specific identity (legacy flag name)
communitas-mcp --http --demo --four-words "pubkey_hex_or_identity"
```

#### 3. Delegate Tokens (Scoped Access)

Issue tokens with limited permissions for AI agents:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_delegate_token",
    "arguments": {
      "delegate_name": "canvas-agent",
      "scopes": ["read_messages", "read_entities"],
      "expires_in_hours": 24
    }
  }
}
```

Token structure:
```rust
pub struct DelegateToken {
    pub issuer: String,           // Four-word identity of issuer
    pub delegate_name: String,    // Name for this delegate
    pub scopes: Vec<Scope>,       // Permitted operations
    pub issued_at: u64,           // Unix timestamp
    pub expires_at: u64,          // Expiration timestamp
    pub nonce: String,            // Unique token ID
}

pub enum Scope {
    Full,              // All permissions
    ReadMessages,      // Read message history
    SendMessages,      // Send new messages
    ReadEntities,      // List organizations, projects, channels
    WriteEntities,     // Create/modify entities
    ReadFiles,         // Access virtual disk files
    WriteFiles,        // Upload files
    ManageMembers,     // Add/remove entity members
    ManageInvites,     // Create/accept/reject invites
}
```

### Saorsa Canvas Integration

Saorsa Canvas is a visual MCP client that renders Communitas data:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                 Saorsa Canvas + Communitas Integration                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                      Saorsa Canvas                                  │ │
│  │                                                                      │ │
│  │  canvas-mcp         canvas-server         Browser/PWA               │ │
│  │  (MCP tools)        (WebSocket)           (wgpu WASM)               │ │
│  │       │                  │                     │                     │ │
│  │       │                  │    ◄────────────────┘                    │ │
│  │       │                  │    Scene updates                          │ │
│  │       ▼                  ▼                                           │ │
│  └───────┬──────────────────┴──────────────────────────────────────────┘ │
│          │                                                               │
│          │ MCP JSON-RPC (HTTP :3040)                                     │
│          │                                                               │
│          ▼                                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                    communitas-mcp                                    ││
│  │                                                                       ││
│  │  Available for Canvas:                                               ││
│  │  - entity_list → render org charts                                   ││
│  │  - message_history → visualize conversations                        ││
│  │  - presence_list → show online peers                                ││
│  │  - kanban_get_board → render Kanban boards                          ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

#### Running Canvas with Communitas

**Option 1: Local Development (Separate Processes)**

```bash
# Terminal 1: Start Communitas MCP server
cd communitas
cargo run -p communitas-mcp -- --http --demo --listen 127.0.0.1:3040

# Terminal 2: Start Canvas server
cd saorsa-canvas
cargo run -p canvas-server

# Open http://localhost:9473 in browser
```

**Option 2: Integrated (Embedded MCP)**

Future: Canvas can embed Communitas MCP as a library:

```rust
// canvas-server could embed communitas-mcp
use communitas_mcp::McpServer;

let mcp = McpServer::new()
    .with_demo_mode()
    .build();

// Canvas tools can query Communitas directly
let entities = mcp.call_tool("entity_list", json!({})).await?;
```

### Localhost Authentication

When MCP server runs on localhost (127.0.0.1), authentication is simplified:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    Localhost Authentication Flow                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  User logged into Communitas Flutter App                                │
│              │                                                           │
│              │ App creates embedded MCP with same CommunitasApp         │
│              │                                                           │
│              ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  Embedded MCP Server (127.0.0.1:3040)                               ││
│  │                                                                       ││
│  │  - Shares CommunitasApp instance with Flutter                        ││
│  │  - No separate auth required                                         ││
│  │  - External tools connect to localhost                               ││
│  │  - Optional: Require delegate token for security                    ││
│  └─────────────────────────────────────────────────────────────────────┘│
│              │                                                           │
│              │ HTTP :3040                                                │
│              ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │  External Tools (localhost only)                                     ││
│  │                                                                       ││
│  │  - saorsa-canvas (visualization)                                    ││
│  │  - Claude Code (AI assistance)                                      ││
│  │  - Custom scripts                                                   ││
│  └─────────────────────────────────────────────────────────────────────┘│
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Passkey Support (Future)

For enhanced security, passkeys can be used alongside delegate tokens:

```
User authenticates with passkey (WebAuthn)
          │
          ▼
System generates delegate token
          │
          ▼
Token shared with MCP client
          │
          ▼
MCP client uses token for requests
```

### MCP Tool Categories

| Category | Tools | Canvas Use |
|----------|-------|------------|
| **Entity** | `entity_list`, `entity_get`, `entity_create` | Org charts, hierarchy views |
| **Message** | `message_send`, `message_history` | Chat visualization |
| **Presence** | `presence_list`, `presence_announce` | Peer status display |
| **Kanban** | `kanban_get_board`, `kanban_update_card` | Board rendering |
| **File** | `file_list`, `file_read`, `file_write` | Document preview |
| **WebRTC** | `webrtc_call_start`, `webrtc_call_end` | Video compositing |

### Example: Canvas Rendering Communitas Data

```javascript
// Canvas skill calling Communitas MCP
async function renderOrgChart(sessionId) {
  // 1. Get entities from Communitas
  const entities = await mcpCall('communitas', 'entity_list', {
    entity_type: 'organisation'
  });

  // 2. Render to canvas
  await mcpCall('canvas', 'canvas_render', {
    session_id: sessionId,
    content: {
      type: 'Chart',
      data: {
        chart_type: 'tree',
        data: transformToTree(entities)
      }
    }
  });
}
```

## Consequences

### Benefits

1. **Standard protocol**: MCP provides well-defined AI integration
2. **Multiple transports**: stdio for CLI tools, HTTP for network access
3. **Scoped access**: Delegate tokens limit what agents can do
4. **Visualization**: saorsa-canvas can render any Communitas data
5. **Extensibility**: Any MCP-compatible tool works with Communitas

### Trade-offs

1. **HTTP overhead**: Network calls slower than FFI (acceptable for AI agents)
2. **Token management**: Delegate tokens need issuance and rotation
3. **Port coordination**: Need to avoid port conflicts between services

### Security Considerations

1. **Localhost binding**: HTTP server defaults to 127.0.0.1, not 0.0.0.0
2. **TLS for remote**: Use `--tls` flag for any non-localhost access
3. **Token expiration**: Delegate tokens have configurable TTL
4. **Scope limiting**: Issue minimum necessary scopes

### Integration Checklist

For external tools integrating with Communitas MCP:

- [ ] Determine transport (stdio vs HTTP)
- [ ] Choose auth method (demo/vault/delegate)
- [ ] Configure MCP client with endpoint
- [ ] Handle tool responses and errors
- [ ] Implement reconnection logic
- [ ] Consider token refresh for long sessions

## Alternatives Considered

1. **REST API**: Traditional HTTP REST instead of MCP
   - Rejected: MCP is the standard for AI tool integration

2. **GraphQL**: Query language for data access
   - Rejected: Adds complexity, MCP sufficient for our needs

3. **gRPC**: Binary protocol for efficiency
   - Rejected: MCP provides better AI agent compatibility

4. **WebSocket only**: Real-time bidirectional
   - Rejected: MCP over HTTP is simpler, WS can be added later

## References

- Model Context Protocol: https://modelcontextprotocol.io/
- Saorsa Canvas: `../saorsa-canvas/`
- MCP implementation: `communitas-mcp/src/`
- Token system: `communitas-mcp/src/token.rs`
- Auth states: `communitas-mcp/src/auth.rs`
- See also: [ADR-017](ADR-017-flutter-rust-ffi-integration.md) for Flutter FFI

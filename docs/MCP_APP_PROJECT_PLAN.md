# Communitas MCP App Architecture - Project Plan

## Vision

The MCP server IS the app. Everything else is presentation.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Communitas Architecture                       │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────┐
                    │      MCP Server         │
                    │   (The Actual App)      │
                    │                         │
                    │  ┌───────────────────┐  │
                    │  │  CommunitasApp    │  │
                    │  │  (execute/query)  │  │
                    │  └───────────────────┘  │
                    │                         │
                    │  ┌───────────────────┐  │
                    │  │  Auth Layer       │  │
                    │  │  (identity mgmt)  │  │
                    │  └───────────────────┘  │
                    │                         │
                    │  ┌───────────────────┐  │
                    │  │  Transport Layer  │  │
                    │  │  (multi-channel)  │  │
                    │  └───────────────────┘  │
                    │          │              │
                    └──────────┼──────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
          ▼                    ▼                    ▼
   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
   │   Embedded   │    │  Local IPC   │    │    QUIC      │
   │   (in-proc)  │    │   (socket)   │    │  (network)   │
   └──────────────┘    └──────────────┘    └──────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
   │  Native App  │    │  Web Client  │    │  AI Agent    │
   │  (Dioxus)    │    │  (localhost) │    │  (Claude)    │
   └──────────────┘    └──────────────┘    └──────────────┘
```

## Current State

### What Exists
- `communitas-mcp/` - MCP server with stdio transport, 20+ tools
- `communitas-core/` - CommunitasApp with full business logic
- `AuthService` - Vault-based auth with public-key identity + password
- 9 VPS nodes globally distributed for testing

### Gaps
1. MCP server has no authentication (auto-initializes on connect)
2. Only stdio transport (not suitable for network or embedded use)
3. No session management or delegate tokens
4. No multi-instance testing infrastructure

---

## Phase 1: Authentication Layer (Week 1-2)

### Goal
MCP server requires authentication before exposing tools.

### Tasks

#### 1.1 Pre-Auth State
```rust
// New enum for server state
enum McpServerState {
    /// Server running, waiting for authentication
    Unauthenticated,
    
    /// User authenticated, full access
    Authenticated {
        session: AuthenticatedSession,
    },
    
    /// Delegate token (AI agent, scoped access)
    Delegated {
        token: DelegateToken,
        scopes: Vec<Scope>,
    },
}
```

#### 1.2 Authentication Tools (exposed pre-auth)
```rust
// Tools available before authentication
Tool {
    name: "authenticate",
    description: "Authenticate with public-key identity and password",
    input_schema: json!({
        "type": "object",
        "properties": {
            "four_words": {"type": "string"},
            "password": {"type": "string"},
            "device_name": {"type": "string"}
        },
        "required": ["four_words", "password"]
    }),
}

Tool {
    name: "authenticate_token",
    description: "Authenticate with a delegate token",
    input_schema: json!({
        "type": "object",
        "properties": {
            "token": {"type": "string"}
        },
        "required": ["token"]
    }),
}

Tool {
    name: "create_vault",
    description: "Create a new identity vault",
    input_schema: json!({
        "type": "object",
        "properties": {
            "four_words": {"type": "string"},
            "password": {"type": "string"},
            "display_name": {"type": "string"}
        },
        "required": ["four_words", "password", "display_name"]
    }),
}
```

#### 1.3 Delegate Token System
```rust
#[derive(Serialize, Deserialize)]
struct DelegateToken {
    issuer: String,           // public-key identity (pubkey_hex)
    delegate_name: String,    // "my-claude-agent"
    scopes: Vec<Scope>,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    nonce: [u8; 32],
}

enum Scope {
    ReadMessages,
    SendMessages,
    ReadFiles,
    WriteFiles,
    ManageEntities,
    ManageMembers,
    ManageKanban,
    Full,
}

// Tool to create delegate tokens (post-auth only)
Tool {
    name: "create_delegate_token",
    description: "Create a scoped access token for AI agents",
    input_schema: json!({
        "type": "object",
        "properties": {
            "delegate_name": {"type": "string"},
            "scopes": {"type": "array", "items": {"type": "string"}},
            "expires_in_hours": {"type": "integer", "default": 24}
        },
        "required": ["delegate_name", "scopes"]
    }),
}
```

#### 1.4 Files to Modify
- `communitas-mcp/src/server.rs` - Add auth state machine
- `communitas-mcp/src/tools.rs` - Add auth tools, scope checking
- `communitas-mcp/src/auth.rs` - NEW: Auth module with token handling
- `communitas-core/src/auth_service.rs` - Integrate with MCP auth

### Deliverable
MCP server that requires `authenticate` before any other tools work.

---

## Phase 2: Multi-Transport Architecture (Week 2-3)

### Goal
Support embedded, local IPC, and network (QUIC) transports.

### Tasks

#### 2.1 Transport Abstraction
```rust
// communitas-mcp/src/transport/mod.rs

#[async_trait]
pub trait McpTransport: Send + Sync {
    /// Read next JSON-RPC message
    async fn recv(&mut self) -> Result<JsonRpcRequest>;
    
    /// Send JSON-RPC response
    async fn send(&mut self, response: JsonRpcResponse) -> Result<()>;
    
    /// Transport identifier for logging
    fn name(&self) -> &str;
}
```

#### 2.2 Transport Implementations

**Stdio (existing, for CLI/pipe use)**
```rust
pub struct StdioTransport {
    reader: BufReader<Stdin>,
    writer: Stdout,
}
```

**Unix Socket / Named Pipe (local IPC)**
```rust
pub struct IpcTransport {
    socket: UnixStream, // or NamedPipe on Windows
}

impl IpcTransport {
    pub async fn listen(path: &Path) -> Result<Self>;
    pub async fn connect(path: &Path) -> Result<Self>;
}
```

**QUIC (network, uses saorsa-gossip-transport::quic)**
```rust
pub struct QuicTransport {
    connection: Connection,
    send_stream: SendStream,
    recv_stream: RecvStream,
}

impl QuicTransport {
    pub async fn listen(addr: SocketAddr, cert: &Certificate) -> Result<Self>;
    pub async fn connect(addr: SocketAddr, server_name: &str) -> Result<Self>;
}
```

**Embedded (in-process, for native apps)**
```rust
pub struct EmbeddedTransport {
    request_tx: mpsc::Sender<JsonRpcRequest>,
    response_rx: mpsc::Receiver<JsonRpcResponse>,
}

impl EmbeddedTransport {
    /// Create paired transports for app and MCP server
    pub fn pair() -> (EmbeddedTransport, EmbeddedTransport);
}
```

#### 2.3 Server Configuration
```rust
pub struct McpServerConfig {
    /// Storage directory for vaults and data
    pub storage_dir: PathBuf,
    
    /// Transports to enable
    pub transports: Vec<TransportConfig>,
}

pub enum TransportConfig {
    Stdio,
    Ipc { path: PathBuf },
    Quic { 
        listen_addr: SocketAddr,
        cert_path: PathBuf,
        key_path: PathBuf,
    },
    Embedded,
}
```

#### 2.4 Multi-Client Support
```rust
pub struct McpServer {
    /// Configuration
    config: McpServerConfig,
    
    /// Active sessions (transport ID -> session)
    sessions: HashMap<String, McpSession>,
    
    /// Shared app instance (authenticated sessions share this)
    app: Option<Arc<RwLock<CommunitasApp>>>,
}

struct McpSession {
    transport_id: String,
    state: McpServerState,
    transport: Box<dyn McpTransport>,
}
```

#### 2.5 Files to Create/Modify
- `communitas-mcp/src/transport/mod.rs` - NEW: Transport trait
- `communitas-mcp/src/transport/stdio.rs` - NEW: Refactor existing
- `communitas-mcp/src/transport/ipc.rs` - NEW: Unix socket/named pipe
- `communitas-mcp/src/transport/quic.rs` - NEW: QUIC transport
- `communitas-mcp/src/transport/embedded.rs` - NEW: In-process
- `communitas-mcp/src/server.rs` - Refactor for multi-transport
- `communitas-mcp/src/main.rs` - CLI for transport selection

### Deliverable
MCP server runnable in multiple modes: `communitas-mcp --stdio`, `communitas-mcp --ipc /tmp/communitas.sock`, `communitas-mcp --quic 0.0.0.0:4433`

---

## Phase 3: Testing Infrastructure (Week 3-4)

### Goal
Deploy 8 MCP servers across VPS for full integration testing.

### Tasks

#### 3.1 Test Account Generation
```rust
// communitas-mcp/src/test_accounts.rs

/// Generate N test accounts with predictable identities
pub fn generate_test_accounts(count: usize) -> Vec<TestAccount> {
    // Use deterministic seed for reproducible tests
    let accounts = vec![
        TestAccount {
            four_words: "alpha.test.ocean.forest".to_string(),
            password: "test-password-1".to_string(),
            display_name: "Alice (Test)".to_string(),
        },
        TestAccount {
            four_words: "beta.test.river.mountain".to_string(),
            password: "test-password-2".to_string(),
            display_name: "Bob (Test)".to_string(),
        },
        // ... 8 accounts total
    ];
    accounts
}
```

#### 3.2 Deployment Script
```bash
#!/bin/bash
# scripts/deploy-mcp-test-nodes.sh

NODES=(
    "saorsa-2.saorsalabs.com"
    "saorsa-3.saorsalabs.com"
    "saorsa-4.saorsalabs.com"
    "saorsa-5.saorsalabs.com"
    "saorsa-6.saorsalabs.com"
    "saorsa-7.saorsalabs.com"
    "saorsa-8.saorsalabs.com"
    "saorsa-9.saorsalabs.com"
)

for i in "${!NODES[@]}"; do
    NODE="${NODES[$i]}"
    ACCOUNT_INDEX=$((i + 1))
    
    echo "Deploying MCP server to $NODE (account $ACCOUNT_INDEX)"
    
    # Copy binary
    scp target/release/communitas-mcp root@$NODE:/opt/communitas/
    
    # Copy test config
    scp configs/test-account-$ACCOUNT_INDEX.toml root@$NODE:/opt/communitas/config.toml
    
    # Restart service
    ssh root@$NODE "systemctl restart communitas-mcp"
done
```

#### 3.3 Systemd Service
```ini
# /etc/systemd/system/communitas-mcp.service
[Unit]
Description=Communitas MCP Server
After=network.target

[Service]
Type=simple
User=communitas
ExecStart=/opt/communitas/communitas-mcp --quic 0.0.0.0:4433 --config /opt/communitas/config.toml
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

#### 3.4 Integration Test Suite
```rust
// tests/mcp_integration_tests.rs

#[tokio::test]
async fn test_cross_node_messaging() {
    // Connect to two different MCP servers
    let alice = McpClient::connect("saorsa-2.saorsalabs.com:4433").await?;
    let bob = McpClient::connect("saorsa-3.saorsalabs.com:4433").await?;
    
    // Authenticate
    alice.authenticate("alpha.test.ocean.forest", "test-password-1").await?;
    bob.authenticate("beta.test.river.mountain", "test-password-2").await?;
    
    // Create a group
    let group = alice.call_tool("create_entity", json!({
        "name": "Test Group",
        "entity_type": "group",
        "initial_members": ["beta.test.river.mountain"]
    })).await?;
    
    // Bob accepts invite
    let invites = bob.call_tool("list_pending_invites", json!({})).await?;
    bob.call_tool("accept_invite", json!({
        "invite_id": invites[0]["id"]
    })).await?;
    
    // Alice sends message
    alice.call_tool("send_message", json!({
        "entity_id": group["id"],
        "entity_type": "group",
        "text": "Hello from Alice!"
    })).await?;
    
    // Bob receives message
    let messages = bob.call_tool("get_messages", json!({
        "entity_id": group["id"]
    })).await?;
    
    assert_eq!(messages[0]["text"], "Hello from Alice!");
}
```

#### 3.5 Files to Create
- `communitas-mcp/src/test_accounts.rs` - Test account generation
- `scripts/deploy-mcp-test-nodes.sh` - Deployment automation
- `configs/test-account-*.toml` - Per-node configurations
- `tests/mcp_integration_tests.rs` - Cross-node test suite

### Deliverable
8 MCP servers running across VPS, full integration tests passing.

---

## Phase 4: Thin Client Applications (Superseded)

### Status
This phase has been superseded by the **all-Rust Dioxus architecture**.

### Current Direction
- Dioxus uses shared Rust services (`communitas-ui-service`) linked directly to `communitas-core`.
- MCP remains the integration layer for AI agents and local automation.
- Web UI is demo-only; no production web client is planned.

See `docs/MCP_THIN_GUI_ARCHITECTURE.md` for the authoritative model.

---

## Phase 5: Presentation Layer Features (Removed)

### Status
Removed from MCP scope. Presentation features live in the Dioxus GUI or dedicated
client apps; MCP stays headless for automation and AI integrations.

### Rationale
- Dioxus + shared Rust services are the only supported GUI path.
- MCP should stay thin and API-focused (no presentation rendering pipeline).

## Summary Timeline

| Phase | Duration | Key Deliverable |
|-------|----------|-----------------|
| 1. Authentication | Week 1-2 | Auth-required MCP server |
| 2. Multi-Transport | Week 2-3 | Embedded + IPC + QUIC transports |
| 3. Testing Infra | Week 3-4 | 8 nodes deployed, integration tests |
| 4. Thin Clients | Week 4-6 | Dioxus GUI via shared Rust services (web demo-only) |
| 5. Presentation | N/A | Removed from MCP scope |

## Success Criteria

1. **Authentication**: `authenticate` tool required before any other tool works
2. **Multi-Transport**: Same MCP server works embedded in app and over network
3. **Testing**: 8 test accounts across VPS can communicate via MCP
4. **Thin Clients**: Dioxus GUI uses shared Rust services; MCP remains for automation/external apps
5. **AI-Ready**: Claude can connect and operate Communitas via MCP

## Files Changed/Created

### New Files
```
communitas-mcp/src/auth.rs                    # Auth module
communitas-mcp/src/transport/mod.rs           # Transport trait
communitas-mcp/src/transport/stdio.rs         # Stdio transport
communitas-mcp/src/transport/ipc.rs           # IPC transport
communitas-mcp/src/transport/quic.rs          # QUIC transport
communitas-mcp/src/transport/embedded.rs      # Embedded transport
communitas-mcp/src/test_accounts.rs           # Test accounts
scripts/deploy-mcp-test-nodes.sh              # Deploy script
configs/test-account-*.toml                   # Per-node configs
tests/mcp_integration_tests.rs                # Integration tests
```

### Modified Files
```
communitas-mcp/src/server.rs                  # Multi-transport, auth
communitas-mcp/src/tools.rs                   # Auth tools, scopes
communitas-mcp/src/main.rs                    # CLI args
communitas-mcp/Cargo.toml                     # Dependencies
```

## Dependencies to Add

```toml
# communitas-mcp/Cargo.toml
[dependencies]
# Existing...

# Transport
tokio-unix = "0.1"                     # Unix sockets
saorsa-gossip-transport = "0.2.2"      # QUIC transport (re-exports ant-quic as quic)

# Auth
jsonwebtoken = "9"              # Token signing
ring = "0.17"                   # Crypto primitives
```

---

## Next Steps

1. **Immediate**: Begin Phase 1 - Add authentication to communitas-mcp
2. **This Week**: Design token format, implement auth state machine
3. **Review**: Share updated tools.rs with auth scopes for feedback

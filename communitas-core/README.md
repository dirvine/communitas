# Communitas Core

Core business logic library for the Communitas P2P collaboration platform.

## Overview

Communitas Core is a Rust library that provides all the essential functionality for building decentralized collaboration applications. It's used by the Dioxus desktop application and the native macOS Swift app, providing a consistent API without any UI dependencies. All P2P networking is delegated to the x0x daemon (x0xd) via `communitas-x0x-client` (see ADR-028).

## Features

- **Post-Quantum Cryptography**: ML-DSA signatures and ML-KEM key exchange
- **Connection Words (four-word networking)**: Human-readable IP:port encoding for peer dialing
- **x0x Daemon Integration**: All P2P networking via x0xd REST + WebSocket API
- **CRDT Document Sync**: Conflict-free collaborative editing with Yrs
- **Encrypted Storage**: Platform-specific secure credential storage (keyring)
- **Presence Service**: Real-time user availability tracking
- **Message Synchronization**: Reliable message delivery with retries
- **Authentication**: Passkey-based authentication with platform integration
- **Storage Management**: Virtual disks with content addressing (BLAKE3)

## Architecture

### Core Components

```
communitas-core/
├── auth_service.rs          # Authentication and session management
├── core_context.rs          # Main application context
├── crdt.rs                  # CRDT document operations
├── doc_replicator.rs        # Document replication logic
├── encrypted_storage/       # Secure credential storage
├── identity.rs             # Identity + connection word encoding helpers
├── keystore.rs            # Cryptographic key storage
├── local_storage.rs       # Local data persistence
├── message_sync.rs        # Message synchronization
├── presence_service.rs    # Presence tracking service
├── security/              # Security primitives
└── storage/               # Storage abstractions
```

> **Note**: P2P networking (gossip, transport, peer discovery) was removed in favor of the x0x daemon (ADR-028). See `communitas-x0x-client` for the networking API.

## Quick Start

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
communitas-core = { path = "../communitas-core" }
tokio = { version = "1.39", features = ["full"] }
```

### Basic Usage

```rust
use communitas_x0x_client::{X0xClient, DaemonManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure x0xd is running (installs if needed)
    let dm = DaemonManager::new();
    dm.ensure_running().await?;

    // Auto-discover daemon address + token and connect
    let client = X0xClient::new();
    let identity = client.agent().await?;
    println!("Agent ID: {}", identity.agent_id);

    // Publish a message to a gossip topic
    client.publish("general-chat", b"Hello, World!").await?;

    // Check online peers
    let agents = client.presence().await?;
    println!("Online agents: {:?}", agents);

    Ok(())
}
```

## Authentication System

### Public-Key Identity & Connection Words

Identities are public keys (pubkey_hex). Four-word networking is used **only** to
encode connection endpoints (IP:port) for friend-to-friend dialing.

```rust
use communitas_core::identity::{conn_words, conn_from_words};
use std::net::SocketAddr;

// Encode IP:port to connection words
let addr: SocketAddr = "192.168.1.100:9000".parse()?;
let words = conn_words(&addr)?;
// → "ocean-forest-moon-star"

// Decode words back to IP:port
let decoded = conn_from_words(&words)?;
assert_eq!(addr, decoded);
```

### Authentication Flow

```rust
use communitas_core::{AuthService, CoreContext};

let core_ctx = CoreContext::new("./data").await?;
let auth = AuthService::new(core_ctx).await?;

// Register new user
let session = auth.register(
    "ocean-forest-moon-star",
    "Alice",
    "Desktop-01",
    "secure-password"
).await?;

// Login existing user
let session = auth.login(
    "ocean-forest-moon-star",
    "secure-password"
).await?;

// Logout
auth.logout().await?;
```

## Networking (via x0xd)

All P2P networking is delegated to the x0x daemon. See [ADR-028](../docs/adr/ADR-028-x0x-daemon-networking-delegation.md).

### Gossip Pub/Sub

```rust
use communitas_x0x_client::X0xClient;

let client = X0xClient::new();

// Subscribe to a topic
let sub_id = client.subscribe("general-chat").await?;

// Publish message
client.publish("general-chat", b"Hello everyone!").await?;

// Unsubscribe when done
client.unsubscribe(&sub_id).await?;
```

### Peer Discovery

```rust
// Discover agents on the network
let agents = client.discovered_agents().await?;
for agent in agents {
    println!("Found agent: {:?}", agent);
}

// Check who's online
let online = client.presence().await?;
println!("Online: {:?}", online);
```

### Direct Messaging

```rust
// Connect to a specific agent
client.connect_agent("agent-id-hex").await?;

// Send a direct message
client.send_direct("agent-id-hex", b"Private message").await?;
```

## CRDT Document Collaboration

### Collaborative Editing

```rust
use communitas_core::crdt::{CrdtDocument, CrdtOperation};

// Create shared document
let doc = CrdtDocument::new("doc-123")?;

// Apply local edit
doc.insert(0, "Hello ")?;
doc.insert(6, "World!")?;

// Get operations for sync
let ops = doc.get_operations_since(0)?;

// Apply remote operations
for op in remote_ops {
    doc.apply_operation(op)?;
}

// Get current text
let text = doc.to_string()?;
```

### Document Replication

```rust
use communitas_core::doc_replicator::DocReplicator;

let replicator = DocReplicator::new(gossip_ctx.clone()).await?;

// Replicate document across peers
replicator.replicate_doc("doc-123").await?;

// Subscribe to document updates
replicator.subscribe_doc("doc-123", |update| {
    println!("Document updated: {:?}", update);
}).await?;
```

## Encrypted Storage

### Platform Integration

The encrypted storage system uses platform-specific credential managers:
- **macOS**: Keychain
- **Windows**: Windows Credential Manager
- **Linux**: Secret Service API (libsecret)

## Storage Management

### Virtual Disks

```rust
use communitas_core::storage::{VirtualDisk, DiskType};

// Create virtual disk for entity
let disk = VirtualDisk::new(
    entity_id,
    DiskType::Private
).await?;

// Write file
disk.write(
    "/docs/readme.md",
    b"# Project Documentation"
).await?;

// Read file
let content = disk.read("/docs/readme.md").await?;

// List files
let files = disk.list("/docs").await?;
```

### Content Addressing

```rust
use communitas_core::storage::ContentAddressed;

// Store content-addressed data
let hash = storage.store_content(b"Hello World!").await?;
// → BLAKE3 hash

// Retrieve by hash
let content = storage.get_content(&hash).await?;
```

## Message Synchronization

Messaging is handled through x0xd's gossip pub/sub and direct messaging APIs. For reliable delivery, use direct messaging with connection management:

```rust
use communitas_x0x_client::X0xClient;

let client = X0xClient::new();

// Direct reliable messaging to a specific agent
client.connect_agent("recipient-agent-id").await?;
client.send_direct("recipient-agent-id", b"Important message").await?;

// Broadcast to a topic for group messaging
client.publish("channel-topic", b"Hello channel!").await?;
```

## Presence Service

Online status is tracked by the x0x daemon. Query it via the REST API:

```rust
use communitas_x0x_client::X0xClient;

let client = X0xClient::new();

// List all online agents
let online_agents = client.presence().await?;
for agent_id in &online_agents {
    println!("{} is online", agent_id);
}

// For real-time presence updates, use the WebSocket connection
// which streams events including presence changes.
```

## Security

### Post-Quantum Cryptography

All cryptographic operations use post-quantum algorithms:

```rust
use communitas_core::security::{PqcSigner, PqcVerifier};

// Create ML-DSA signer
let signer = PqcSigner::new()?;

// Sign data
let signature = signer.sign(b"data to sign")?;

// Verify signature
let verifier = PqcVerifier::from_public_key(&public_key)?;
let is_valid = verifier.verify(b"data to sign", &signature)?;
```

### Key Exchange

```rust
use communitas_core::security::{PqcKeyExchange, SharedSecret};

// Alice generates keypair
let alice_kex = PqcKeyExchange::new()?;
let alice_public = alice_kex.public_key();

// Bob generates keypair and creates shared secret
let bob_kex = PqcKeyExchange::new()?;
let bob_public = bob_kex.public_key();
let bob_shared = bob_kex.derive_shared_secret(&alice_public)?;

// Alice creates shared secret
let alice_shared = alice_kex.derive_shared_secret(&bob_public)?;

assert_eq!(alice_shared, bob_shared);
```

## Development

### Building

```bash
cargo build -p communitas-core
cargo build -p communitas-core --release
```

### Testing

```bash
# Run all tests
cargo test -p communitas-core

# Run with logging
RUST_LOG=debug cargo test -p communitas-core

# Run specific test
cargo test -p communitas-core --test integration_test

# Run property-based tests
cargo test -p communitas-core --features proptest
```

### Linting

```bash
# Strict linting (enforces no panics in production)
cargo clippy -p communitas-core --all-features -- \
  -D clippy::panic \
  -D clippy::unwrap_used \
  -D clippy::expect_used

# Format code
cargo fmt -p communitas-core
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COMMUNITAS_DATA_DIR` | Data storage directory | `~/.communitas` |
| `COMMUNITAS_LOG_LEVEL` | Logging level | `info` |
| `RUST_LOG` | Detailed logging filter | - |

### Configuration File

Core can be configured via TOML:

```toml
[identity]
four_words = "ocean-forest-moon-star"
display_name = "Alice"
device_name = "Desktop-01"

[network]
bootstrap_nodes = ["bootstrap.communitas.network:8080"]
enable_mdns = true
port = 8080

[storage]
data_dir = "~/.communitas"
cache_size_mb = 500

[security]
enable_pqc = true
key_rotation_days = 90

[logging]
level = "info"
format = "json"
```

## API Documentation

Full API documentation is available via rustdoc:

```bash
cargo doc -p communitas-core --open
```

## Performance Characteristics

- **Message Latency**: <100ms local, <500ms remote
- **Storage Operations**: <100ms for content-addressed reads
- **CRDT Sync**: <200ms for typical document operations
- **Memory Usage**: ~50MB baseline, scales with active documents
- **CPU Usage**: <5% idle, scales with network activity

## Security Considerations

1. **Zero-Panics Policy**: Production code forbids `unwrap()`, `expect()`, and `panic!`
2. **Post-Quantum Ready**: All cryptography uses ML-DSA and ML-KEM
3. **Secure Storage**: Platform-specific credential managers
4. **End-to-End Encryption**: All messages encrypted by default
5. **Forward Secrecy**: Perfect forward secrecy for all sessions
6. **No Trusted Third Parties**: Fully decentralized architecture

## Troubleshooting

### Common Issues

**Identity Generation Fails**
```bash
# Check four-word-networking dictionary (connection words)
cargo test -p communitas-core -- identity::tests
```

**x0xd Daemon Not Reachable**
```bash
# Check if x0xd is running
pgrep -f x0xd

# Check config files exist
cat ~/Library/Application\ Support/x0x/api.port
cat ~/Library/Application\ Support/x0x/api-token

# Enable debug logging
RUST_LOG=communitas_x0x_client=debug cargo run
```

**Storage Errors**
```bash
# Check permissions
ls -la ~/.communitas/

# Reset storage (development only)
rm -rf ~/.communitas/
```

**Authentication Issues**
```bash
# Check keyring access
cargo test -p communitas-core -- auth_service::tests

# Clear credentials (macOS)
security delete-generic-password -s "communitas"
```

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.

## See Also

- [Communitas Dioxus](../communitas-dioxus/) - Cross-platform Dioxus + Tauri desktop application
- [Communitas Apple](../communitas-apple/) - Native macOS SwiftUI application
- [Architecture Documentation](../../docs/architecture/) - System architecture details
- [API Reference](../../docs/api/) - Complete API documentation

# Communitas Core

Core business logic library for the Communitas P2P collaboration platform.

## Overview

Communitas Core is a Rust library that provides all the essential functionality for building decentralized collaboration applications. It's used by both the desktop application and headless daemon, providing a consistent API without any UI dependencies.

## Features

- **Post-Quantum Cryptography**: ML-DSA signatures and ML-KEM key exchange
- **Four-Word Addressing**: Human-readable identity system (e.g., "ocean-forest-moon-star")
- **Gossip Overlay Network**: Distributed P2P communication layer
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
├── gossip/                  # P2P gossip overlay
│   ├── context.rs          # Gossip network context
│   ├── coordinator.rs      # Message coordination
│   ├── crdt_sync.rs        # CRDT synchronization
│   ├── groups.rs           # Group management
│   ├── identity.rs         # Identity operations
│   ├── membership.rs       # Membership tracking
│   ├── presence.rs         # Presence detection
│   ├── pubsub.rs          # Publish-subscribe messaging
│   ├── rendezvous.rs      # Peer rendezvous
│   ├── transport.rs       # Network transport
│   └── types.rs           # Shared types
├── identity.rs             # Identity management and four-word addressing
├── keystore.rs            # Cryptographic key storage
├── local_storage.rs       # Local data persistence
├── message_sync.rs        # Message synchronization
├── presence_service.rs    # Presence tracking service
├── security/              # Security primitives
└── storage/               # Storage abstractions
```

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
use communitas_core::{CoreContext, AuthService, GossipContext};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize core context
    let core_ctx = CoreContext::new("./data").await?;

    // Create identity with four-word address
    let auth = AuthService::new(core_ctx.clone()).await?;
    let session = auth.register(
        "ocean-forest-moon-star",
        "Alice",
        "Desktop-01",
        "secure-password"
    ).await?;

    println!("Created identity: {}", session.four_words);
    println!("Entity ID: {}", session.entity_id);

    // Initialize gossip network
    let gossip = GossipContext::new(core_ctx).await?;
    gossip.start().await?;

    // Send message to channel
    gossip.publish(
        "channel-id",
        "Hello, World!".as_bytes(),
        vec!["recipient-four-words"]
    ).await?;

    Ok(())
}
```

## Authentication System

### Four-Word Identity

Every user, organization, and entity has a human-readable four-word identity:

```rust
use communitas_core::identity::{generate_id_words, validate_id_words};

// Generate new identity
let four_words = generate_id_words()?;
// → "ocean-forest-moon-star"

// Validate identity
let is_valid = validate_id_words(&["ocean", "forest", "moon", "star"])?;
// → true

// Connect from words to IP addresses (optional)
use communitas_core::identity::conn_from_words;
let connection_info = conn_from_words(&four_words)?;
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

## Gossip Network

### Network Initialization

```rust
use communitas_core::{GossipContext, CoreContext};

let core_ctx = CoreContext::new("./data").await?;
let gossip = GossipContext::new(core_ctx).await?;

// Start gossip network
gossip.start().await?;

// Join a topic
gossip.subscribe("general-chat").await?;

// Publish message
gossip.publish(
    "general-chat",
    b"Hello everyone!",
    vec![] // Empty = broadcast to all subscribers
).await?;
```

### Peer Discovery

```rust
// Discover peers via rendezvous
let peers = gossip.discover_peers("general-chat").await?;

for peer in peers {
    println!("Found peer: {:?}", peer);
}

// Check online presence
let is_online = gossip.is_online("ocean-forest-moon-star").await?;
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

### Passkey Storage

```rust
use communitas_core::encrypted_storage::PasskeyStorage;

let storage = PasskeyStorage::new()?;

// Store passkey
storage.store_passkey(
    "user@example.com",
    "service-name",
    &passkey_bytes
).await?;

// Retrieve passkey
let passkey = storage.get_passkey(
    "user@example.com",
    "service-name"
).await?;

// Delete passkey
storage.delete_passkey(
    "user@example.com",
    "service-name"
).await?;
```

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

### Reliable Messaging

```rust
use communitas_core::message_sync::MessageSync;

let sync = MessageSync::new(gossip_ctx.clone()).await?;

// Send message with automatic retry
sync.send_reliable(
    "recipient-four-words",
    "channel-id",
    b"Important message"
).await?;

// Receive messages
let messages = sync.receive_since(last_timestamp).await?;

for msg in messages {
    println!("From: {}, Content: {:?}", msg.sender, msg.content);
}
```

## Presence Service

### Online Status Tracking

```rust
use communitas_core::presence_service::PresenceService;

let presence = PresenceService::new(gossip_ctx.clone()).await?;

// Set own status
presence.set_status("online", "Working on docs").await?;

// Check peer status
let status = presence.get_status("ocean-forest-moon-star").await?;
println!("Status: {:?}", status);

// Subscribe to presence changes
presence.subscribe(|event| {
    match event {
        PresenceEvent::Online(peer) => println!("{} came online", peer),
        PresenceEvent::Offline(peer) => println!("{} went offline", peer),
        PresenceEvent::StatusChange(peer, status) => {
            println!("{} changed status to {}", peer, status)
        }
    }
}).await?;
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
# Check four-word-networking dictionary
cargo test -p communitas-core -- identity::tests
```

**Gossip Network Won't Start**
```bash
# Check port availability
lsof -i :8080

# Enable debug logging
RUST_LOG=communitas_core::gossip=debug cargo run
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

See [../../docs/development/contributing.md](../../docs/development/contributing.md)

## License

Dual-licensed under AGPL-3.0-or-later and commercial license.

## See Also

- [Communitas Desktop](../communitas-desktop/README.md) - Desktop application using this core
- [Communitas Headless](../communitas-headless/README.md) - Headless daemon using this core
- [Architecture Documentation](../../docs/architecture/) - System architecture details
- [API Reference](../../docs/api/) - Complete API documentation

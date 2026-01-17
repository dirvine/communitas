# ADR-009: Modular Crate Architecture

## Status

Accepted (2025-12-24)
Updated (2025-01-15) - Replaced HTTP bridge with FFI, added Flutter

## Context

### The Problem

A monolithic codebase for a complex application creates issues:

- **Compilation time**: All code recompiles on any change
- **Testing overhead**: All tests run even for unrelated changes
- **Tight coupling**: Hard to reason about dependencies
- **Binary bloat**: Unused code included in all targets
- **Reusability**: Can't use components independently

Communitas needs an architecture that:
- Separates concerns cleanly
- Enables fast incremental compilation
- Supports multiple deployment targets (desktop, headless, TUI)
- Allows component reuse
- Maintains clear dependency boundaries

### Requirements

- Clean separation of core logic from UI
- Shared code across deployment targets
- Independent testing of components
- Minimal dependency coupling
- Support for FFI bindings (Flutter via flutter_rust_bridge)

## Decision

Organize Communitas as a **Cargo workspace** with specialized crates:

### Workspace Structure

```
communitas/
├── Cargo.toml                    # Workspace root
├── communitas-core/              # Core library (no UI, no platform)
├── communitas-flutter/           # Flutter cross-platform application
├── communitas-headless/          # Headless daemon / bootstrap nodes
├── communitas-mcp/               # MCP server for AI agents (stdio + HTTPS)
├── communitas-kanban/            # CRDT-based Kanban system
└── communitas-p2p-test/          # P2P testing utilities
```

### Crate Responsibilities

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| **communitas-core** | Business logic, CRDT, storage, crypto, FFI | saorsa-*, yrs, flutter_rust_bridge |
| **communitas-flutter** | Cross-platform UI (macOS, iOS, Android, Windows, Linux) | core (via FFI) |
| **communitas-headless** | Bootstrap nodes, CLI, server mode | core, tokio |
| **communitas-mcp** | MCP server for AI agents (stdio + HTTPS) | core, axum |
| **communitas-kanban** | CRDT-based Kanban boards | core, yrs |

### Communication Patterns

| Component | Access Method | Use Case |
|-----------|--------------|----------|
| **Flutter App** | FFI via flutter_rust_bridge | GUI operations, user interactions |
| **AI Agents** | MCP over stdio/HTTP | Claude Code, external tools |
| **Headless Node** | Direct Rust API | Bootstrap, network infrastructure |

### Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Crate Dependency Graph                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                     External Dependencies                           │
│  ┌────────────────────────────────────────────────────────────────┐│
│  │ saorsa-pqc │ saorsa-gossip │ ant-quic │ yrs │ libsql │ tokio  ││
│  └────────────────────────────────────────────────────────────────┘│
│                              │                                      │
│                              ▼                                      │
│                    ┌──────────────────┐                            │
│                    │ communitas-core  │                            │
│                    │                  │                            │
│                    │ • Identity       │                            │
│                    │ • CRDT           │                            │
│                    │ • Storage        │                            │
│                    │ • Gossip         │                            │
│                    │ • Crypto         │                            │
│                    │ • FFI bindings   │                            │
│                    └──────────────────┘                            │
│                      │         │         │                          │
│          ┌───────────┘         │         └───────────┐              │
│          ▼                     ▼                     ▼              │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐        │
│  │ communitas-  │     │ communitas-  │     │ communitas-  │        │
│  │   flutter    │     │   headless   │     │     mcp      │        │
│  │              │     │              │     │              │        │
│  │ Cross-plat   │     │ Bootstrap    │     │ AI Agents    │        │
│  │ GUI (FFI)    │     │ Nodes, CLI   │     │ stdio/HTTP   │        │
│  └──────────────┘     └──────────────┘     └──────────────┘        │
│          │                                         │                │
│          │ flutter_rust_bridge                     │ MCP JSON-RPC   │
│          ▼                                         ▼                │
│  ┌──────────────┐                         ┌──────────────┐         │
│  │   Flutter    │                         │ AI Tools     │         │
│  │   Dart UI    │                         │ Claude,      │         │
│  │              │                         │ Canvas, etc  │         │
│  └──────────────┘                         └──────────────┘         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### communitas-core Design

The core crate is platform-agnostic and UI-free:

```rust
// communitas-core/src/lib.rs

// Identity and addressing
pub mod identity;          // Four-word system
pub mod types;             // UserProfile, DeviceType

// Data management
pub mod crdt;              // Yrs CRDT documents
pub mod crdt_manager;      // Document management
pub mod storage;           // Virtual disks, libSQL

// Collaboration
pub mod entity_service;    // Entities and members
pub mod message_service;   // Chat and threads
pub mod invite_service;    // Cross-org invites

// Security
pub mod encrypted_storage; // Vault, passkeys
pub mod permissions;       // Access control

// Networking
pub mod gossip;            // P2P overlay
pub mod webrtc;            // Voice/video
```

**No dependencies on**:
- GUI runtimes or frameworks
- Platform APIs (keyring, notifications)

### Flutter FFI Bindings (flutter_rust_bridge)

Exposes core functionality to Dart via FFI:

```rust
// communitas-core/src/flutter_api.rs

#[flutter_rust_bridge::frb]
impl CommunitasApi {
    // Authentication
    pub async fn auth_login(&self, four_words: String, passphrase: String) -> Result<FlutterLoginResult>;
    pub async fn auth_logout(&self) -> Result<()>;

    // Entities
    pub async fn entity_list(&self) -> Result<Vec<FlutterEntity>>;
    pub async fn entity_list_by_type(&self, entity_type: FlutterEntityType) -> Result<Vec<FlutterEntity>>;
    pub async fn entity_create(&self, name: String, entity_type: FlutterEntityType) -> Result<Vec<FlutterEvent>>;

    // Messaging
    pub async fn message_send(&self, entity_id: String, text: String) -> Result<Vec<FlutterEvent>>;

    // Networking
    pub async fn gossip_get_network_info(&self) -> Result<FlutterNetworkInfo>;
    pub async fn gossip_connect_to_peer(&self, four_words: String) -> Result<Vec<FlutterEvent>>;
}
```

Generated bindings in `communitas-flutter/lib/src/bindings/`:
- `flutter_api.dart` - Main API class
- `api_exports.dart` - Type exports

### Feature Flags

Communitas core no longer feature-gates gossip; the saorsa-gossip stack is required.
Optional functionality (like WebRTC) is configured in dependent crates rather than
via core feature flags.

### Build Configuration

```toml
# Root Cargo.toml
[workspace]
members = [
    "communitas-core",
    "communitas-headless",
    "communitas-mcp",
    "communitas-kanban",
    "communitas-p2p-test",
]
resolver = "2"

[workspace.dependencies]
# Shared versions across all crates
communitas-core = { path = "communitas-core" }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
flutter_rust_bridge = "2.x"
```

### Flutter Build Configuration

```bash
# Generate FFI bindings after Rust API changes
cd communitas-flutter
flutter_rust_bridge_codegen generate

# Build for specific platform
flutter build ios --release
flutter build android --release
```

## Consequences

### Benefits

- **Fast compilation**: Change in TUI doesn't rebuild desktop
- **Clean boundaries**: Core has no UI dependencies
- **Multiple targets**: Same core for desktop, mobile, CLI
- **Independent testing**: Test core without a GUI runtime
- **Selective features**: Minimal builds for embedded

### Trade-offs

- **Initial setup**: More crates to configure
- **Cross-crate changes**: Require coordinated updates
- **Version management**: Keep workspace deps aligned

### Compilation Times (Example)

| Change | Rebuild |
|--------|---------|
| Core business logic | ~30s (core + dependents) |
| Desktop UI only | ~10s (desktop only) |
| TUI only | ~5s (tui only) |
| Bindings only | ~15s (bindings only) |

### Testing Strategy

```bash
# Test specific crate
cargo test -p communitas-core

# Test all crates
cargo test --workspace

# Test with features
cargo test -p communitas-core --features minimal
```

## Alternatives Considered

1. **Single crate**: Everything in one crate
   - Rejected: Slow compilation, tight coupling

2. **Git submodules**: Separate repositories
   - Rejected: Coordination overhead, version sync issues

3. **Microservices**: Separate processes
   - Rejected: Overkill, added complexity

4. **Compile-time conditionals**: `#[cfg(feature)]` everywhere
   - Rejected: Complex, hard to test

## References

- Workspace: `Cargo.toml` (root)
- Core crate: `communitas-core/`
- Flutter app: `communitas-flutter/`
- FFI bindings: `communitas-flutter/lib/src/bindings/`
- flutter_rust_bridge docs: https://cjycode.com/flutter_rust_bridge/
- See also: [ADR-017](ADR-017-flutter-rust-ffi-integration.md) (FFI integration details)
- See also: [ADR-018](ADR-018-mcp-external-integration.md) (MCP for external tools)

# ADR-009: Modular Crate Architecture

## Status

Accepted (2025-12-24)
Updated (2026-01-18) - Replaced HTTP bridge with shared Rust services, added Dioxus UI

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
- Support for shared Rust UI services (Dioxus via `communitas-ui-service`)

## Decision

Organize Communitas as a **Cargo workspace** with specialized crates:

### Workspace Structure

```
communitas/
├── Cargo.toml                    # Workspace root
├── communitas-core/              # Core library (no UI, no platform)
├── communitas-dioxus/            # Dioxus/Tauri cross-platform application
├── communitas-headless/          # Headless daemon / bootstrap nodes
├── communitas-mcp/               # MCP server for AI agents (stdio + HTTPS)
├── communitas-kanban/            # CRDT-based Kanban system
└── communitas-p2p-test/          # P2P testing utilities
```

### Crate Responsibilities

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| **communitas-core** | Business logic, CRDT, storage, crypto, UI API | saorsa-*, yrs |
| **communitas-dioxus** | Cross-platform UI (macOS, Windows, Linux; experimental mobile) | core (`ui_core`), communitas-ui-service |
| **communitas-headless** | Bootstrap nodes, CLI, server mode | core, tokio |
| **communitas-mcp** | MCP server for AI agents (stdio + HTTPS) | core, axum |
| **communitas-kanban** | CRDT-based Kanban boards | core, yrs |

### Communication Patterns

| Component | Access Method | Use Case |
|-----------|--------------|----------|
| **Dioxus App** | Shared Rust services (`communitas-ui-service`) | GUI operations, user interactions |
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
│  │   dioxus     │     │   headless   │     │     mcp      │        │
│  │              │     │              │     │              │        │
│  │ All-Rust UI  │     │ Bootstrap    │     │ AI Agents    │        │
│  │ Shell        │     │ Nodes, CLI   │     │ stdio/HTTP   │        │
│  └──────────────┘     └──────────────┘     └──────────────┘        │
│          │                                         │                │
│          │ shared Rust services                    │ MCP JSON-RPC   │
│          ▼                                         ▼                │
│  ┌──────────────┐                         ┌──────────────┐         │
│  │   Dioxus     │                         │ AI Tools     │         │
│  │   Rust UI    │                         │ Claude,      │         │
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

### UI Core API (Rust-native)

Exposes core functionality to the shared UI service and Dioxus front-end:

```rust
// communitas-core/src/ui_core.rs

impl CommunitasApi {
    // Authentication
    pub async fn auth_login(&self, four_words: String, password: String) -> Result<UiSessionInfo, String>;
    pub async fn auth_logout(&self) -> Result<(), String>;

    // Entities
    pub async fn entity_list(&self) -> Result<Vec<UiEntity>, String>;
    pub async fn entity_list_by_type(&self, ty: UiEntityType) -> Result<Vec<UiEntity>, String>;
    pub async fn entity_create(&self, name: String, ty: UiEntityType) -> Result<Vec<UiEvent>, String>;

    // Messaging
    pub async fn message_send(&self, entity_id: String, text: String) -> Result<Vec<UiEvent>, String>;

    // Networking
    pub async fn gossip_get_network_info(&self) -> Result<UiNetworkInfo, String>;
    pub async fn gossip_connect_to_peer(&self, four_words: String) -> Result<Vec<UiEvent>, String>;
}
```

`communitas-ui-service` wraps this API to provide higher-level async traits for Dioxus components and MCP tooling.

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
```

### Dioxus Build Configuration

```bash
# Run the Dioxus dev server with hot reload
cd communitas-dioxus
dx serve --platform desktop --hotpatch

# Build for specific platform
dx bundle --platform desktop
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
- Dioxus app: `communitas-dioxus/`
- Shared UI service: `communitas-ui-service/`
- See also: [ADR-017](ADR-017-legacy-thin-client-ffi-integration.md) (archived legacy context)
- See also: [ADR-018](ADR-018-mcp-external-integration.md) (MCP for external tools)

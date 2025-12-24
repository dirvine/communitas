# ADR-009: Modular Crate Architecture

## Status

Accepted (2025-12-24)

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
- Support for UniFFI bindings (Swift/Kotlin)

## Decision

Organize Communitas as a **Cargo workspace** with specialized crates:

### Workspace Structure

```
communitas/
├── Cargo.toml                    # Workspace root
├── communitas-core/              # Core library (no UI, no platform)
├── communitas-desktop/           # Tauri desktop application
├── communitas-headless/          # Headless daemon
├── communitas-tui/               # Terminal UI
├── communitas-bindings/          # UniFFI bindings for mobile
├── communitas-bridge/            # HTTP bridge for testing
└── communitas-app/               # iOS/Android (planned)
```

### Crate Responsibilities

| Crate | Purpose | Dependencies |
|-------|---------|--------------|
| **communitas-core** | Business logic, CRDT, storage, crypto | saorsa-*, yrs |
| **communitas-desktop** | Tauri commands, desktop integration | core, tauri |
| **communitas-headless** | Server mode, CLI, webhooks | core, tokio |
| **communitas-tui** | Terminal interface | core, ratatui |
| **communitas-bindings** | Swift/Kotlin FFI | core, uniffi |
| **communitas-bridge** | HTTP REST API for testing | core, axum |

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
│                    └──────────────────┘                            │
│                      │    │    │    │                              │
│          ┌───────────┘    │    │    └───────────┐                  │
│          ▼                ▼    ▼                ▼                  │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌───────────┐ │
│  │ communitas-  │ │ communitas-  │ │ communitas-  │ │communitas-│ │
│  │   desktop    │ │   headless   │ │     tui      │ │ bindings  │ │
│  │              │ │              │ │              │ │           │ │
│  │ Tauri + UI   │ │ CLI + Server │ │ Terminal UI  │ │ UniFFI    │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └───────────┘ │
│                              │                              │      │
│                              ▼                              ▼      │
│                    ┌──────────────┐               ┌──────────────┐│
│                    │ communitas-  │               │ communitas-  ││
│                    │   bridge     │               │     app      ││
│                    │              │               │              ││
│                    │ HTTP API     │               │ iOS/Android  ││
│                    └──────────────┘               └──────────────┘│
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
- Tauri (desktop-specific)
- Platform APIs (keyring, notifications)
- UI frameworks

### communitas-bindings (UniFFI)

Exposes core functionality to Swift/Kotlin:

```rust
// communitas-bindings/src/lib.rs

#[uniffi::export]
impl CommunitasClient {
    // Identity
    pub fn initialize(&self, four_words: String, ...) -> Result<(), ClientError>;

    // Entities
    pub fn entity_create(&self, ...) -> Result<String, ClientError>;
    pub fn entity_list(&self, ...) -> Result<Vec<SwiftEntity>, ClientError>;

    // Messaging
    pub fn message_send(&self, ...) -> Result<String, ClientError>;

    // Invites
    pub fn invite_create(&self, ...) -> Result<SwiftInvite, ClientError>;
}
```

Generated bindings:
- `communitas.swift` for iOS
- `communitas.kt` for Android (planned)

### Feature Flags

Crates use feature flags for optional functionality:

```toml
# communitas-core/Cargo.toml
[features]
default = ["full"]
full = ["gossip", "webrtc", "crdt"]
minimal = []  # Just identity and storage
gossip = ["saorsa-gossip", "ant-quic"]
webrtc = ["saorsa-webrtc"]
crdt = ["yrs"]
```

| Feature | Use Case |
|---------|----------|
| `minimal` | Embedded, testing |
| `gossip` | P2P networking |
| `webrtc` | Voice/video calls |
| `full` | Desktop application |

### Build Configuration

```toml
# Root Cargo.toml
[workspace]
members = [
    "communitas-core",
    "communitas-desktop",
    "communitas-headless",
    "communitas-tui",
    "communitas-bindings",
    "communitas-bridge",
]
resolver = "2"

[workspace.dependencies]
# Shared versions across all crates
communitas-core = { path = "communitas-core" }
tokio = { version = "1.40", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

## Consequences

### Benefits

- **Fast compilation**: Change in TUI doesn't rebuild desktop
- **Clean boundaries**: Core has no UI dependencies
- **Multiple targets**: Same core for desktop, mobile, CLI
- **Independent testing**: Test core without Tauri
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
- Bindings: `communitas-bindings/`
- UniFFI docs: https://mozilla.github.io/uniffi-rs/

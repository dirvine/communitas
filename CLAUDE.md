# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Communitas is a local-first, PQC-ready collaboration platform that merges WhatsApp, Dropbox, Zoom, and Slack into one decentralized application. It uses Four-Word identities for human-verifiable addressing, provides per-entity virtual disks (org, group, channel, project, individual), and enables DNS-free website publishing via identity-bound website roots.

**Platform Focus**: Native macOS application (Swift + SwiftUI), with future expansion to iOS, Android, Linux, and Windows.

## Core Architecture

### Native macOS Application (Swift + SwiftUI)
- **Location**: `communitas-swift/`
- **Framework**: SwiftUI with native macOS components
- **Build System**: Xcode project and Swift Package Manager
- **Rust Integration**: UniFFI bindings via `communitas-bindings/`

### Rust Core Library
- **Location**: `communitas-core/`
- **Purpose**: Cross-platform business logic, P2P networking, cryptography
- **Cryptography**: Post-quantum (ML-DSA/ML-KEM) with ChaCha20-Poly1305
- **Storage**: Virtual disks with CRDT synchronization (Yrs)
- **Networking**: QUIC via ant-quic, IPv4-first with Happy Eyeballs fallback

### UniFFI Bindings
- **Location**: `communitas-bindings/`
- **Purpose**: Generate Swift bindings from Rust core
- **Output**: XCFramework for macOS (and later iOS)

### Key Components
- **Four-Word Addresses**: Human-readable network identities (e.g., "ocean-forest-moon-star")
- **Virtual Disks**: Private/Public/Shared per entity with different encryption policies
- **Website Publishing**: DNS-free web via identity.website_root binding
- **Messaging**: End-to-end encrypted group messaging with channel support
- **Groups**: Threshold-ready group identities with ML-DSA signatures
- **Kanban System**: CRDT-based collaborative project management (`communitas-kanban/`)
- **Offline-First**: All operations work locally and sync when network available

## Development Commands

### Quick Start - macOS App
```bash
# Build the Rust core and bindings
cargo build -p communitas-bindings --release

# Generate Swift bindings (XCFramework)
cd communitas-bindings && ./build-xcframework.sh

# Open Xcode project
open communitas-swift/Communitas.xcodeproj

# Or build from command line
cd communitas-swift && xcodebuild -scheme Communitas -configuration Debug build
```

### Rust Development
```bash
# Build all Rust crates
cargo build

# Run tests
cargo test

# Format and lint
cargo fmt --all
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

# Build specific crates
cargo build -p communitas-core
cargo build -p communitas-kanban
cargo test -p communitas-core
cargo test -p communitas-kanban
```

### Bridge Server (Testing)
```bash
# Start HTTP bridge for testing (useful for debugging)
cargo run -p communitas-bridge

# Bridge provides HTTP/REST endpoints at http://localhost:3030
# See docs/api/bridge-api.md for details
```

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `communitas-core` | Core business logic, P2P, cryptography |
| `communitas-bindings` | UniFFI Swift bindings |
| `communitas-kanban` | CRDT-based Kanban system |
| `communitas-bridge` | HTTP REST bridge for testing |
| `communitas-headless` | Bootstrap/seed nodes |
| `communitas-tui` | Terminal UI (development tool) |
| `communitas-p2p-test` | P2P testing utilities |

## Architecture Insights

### Core Context System
The application uses a centralized `CoreContext` (communitas-core/src/core_context.rs) that wires Communitas to saorsa-gossip components:
- Identity management with enhanced PQC support
- Storage management with CRDT synchronization (Yrs)
- Chat management with persistent storage
- Messaging service for real-time communication via gossip overlay
- Kanban service for collaborative project management
- Group key storage for membership updates

### Swift-Rust Bridge
Commands flow through UniFFI-generated bindings:
1. Swift UI calls generated Swift functions
2. UniFFI marshals data to Rust
3. Rust core processes request
4. Results return via UniFFI to Swift

### Virtual Disk System
Per-entity storage with different access policies:
- **Private**: Encrypted, local-only storage
- **Public**: Content-addressed, distributed storage
- **Shared**: Group-accessible with shared encryption

### Security Model
- **Zero panics/unwraps**: Production Rust code enforces Result types
- **Rate limiting**: Built-in protection against abuse
- **Input validation**: All commands validate inputs
- **Secure storage**: macOS Keychain integration

## Quality Standards

### Rust Code
- Production code: no `unwrap()`, `expect()`, or `panic!()` (use `thiserror`/`anyhow` and return errors; log via `tracing`).
- Tests: `unwrap/expect/panic!` are allowed for clarity and speed.
- Clippy policy: `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`. Do not enable `clippy::pedantic` by default.
- Formatting: `cargo fmt --all` before commits.
- Documentation: Prefer doc comments on public items; add when helpful.

### Swift Code
- SwiftUI best practices with proper state management
- No force unwraps in production code
- Proper error handling with Result types
- Accessibility support for all UI components

### Git Workflow
```bash
# Format and check before commit
cargo fmt --all
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used
cargo test

# Commit with conventional format
git commit -m "feat: add new feature"
git commit -m "fix: resolve issue"
git commit -m "docs: update documentation"
```

## Deployment

### macOS Application
Native macOS application distributed via:
- Direct DMG download
- Future: Mac App Store

### Headless Node
Bootstrap and seed nodes for network support:
- Binary: `communitas-headless`
- Config: TOML with listen addresses, storage paths

## Troubleshooting

### Common Issues
- **P2P Connection Failures**: Check bootstrap node connectivity
- **Build Failures**: Ensure Rust 1.85+ and Xcode 15+ installed
- **Binding Generation**: Run `./build-xcframework.sh` after Rust changes

### Debug Modes
```bash
# Rust debugging
RUST_LOG=debug cargo run -p communitas-headless

# Test debugging
RUST_LOG=debug cargo test -- --nocapture
```

## API Documentation

For detailed API documentation, see:
- `docs/api/core-api.md` - Rust library API (communitas-core)
- `docs/api/bridge-api.md` - HTTP/REST bridge API for testing
- `docs/architecture/README.md` - System architecture overview
- `docs/architecture/crdt-system.md` - CRDT synchronization (Yrs)
- `docs/architecture/gossip-protocol.md` - Saorsa Gossip networking

## Performance Targets

- **Message Latency**: <100ms local, <500ms remote
- **Storage Operations**: <100ms local, <500ms with geographic routing
- **UI Responsiveness**: 60fps, smooth animations
- **Memory Usage**: <200MB baseline

## Security Considerations

- All external links must use HTTPS
- Canonical signing for sensitive updates
- Zero centralized dependencies for core functionality
- Anti-phishing via Four-Word checksum validation
- Rate limiting on all public endpoints

## Notes

- We use `four-word-networking` crate to encode/decode IPv4 and IPv6 to 4 or more words
- All identities are validated words from the four-word-networking dictionary
- Test network and MCP integration available for development

This architecture supports rapid development while maintaining production-quality standards for a secure, decentralized collaboration platform.

---

## 🚨 CRITICAL: Saorsa Network Infrastructure & Port Isolation

### Infrastructure Documentation
Full infrastructure documentation is available at: `docs/infrastructure/INFRASTRUCTURE.md`

This includes:
- All 9 VPS nodes across 3 cloud providers (DigitalOcean, Hetzner, Vultr)
- Bootstrap node endpoints and IP addresses
- Firewall configurations and SSH access
- Systemd service templates

### ⚠️ PORT ISOLATION - MANDATORY

**Communitas uses UDP port range 11000-11999 exclusively.**

| Service | UDP Port Range | Default | Description |
|---------|----------------|---------|-------------|
| ant-quic | 9000-9999 | 9000 | QUIC transport layer |
| saorsa-node | 10000-10999 | 10000 | Core P2P network nodes |
| **communitas** | **11000-11999** | **11000** | Collaboration platform nodes (THIS PROJECT) |

### 🛑 DO NOT DISTURB OTHER NETWORKS

When testing or developing communitas:

1. **ONLY use ports 11000-11999** for communitas services
2. **NEVER** kill processes on ports 9000-9999 or 10000-10999
3. **NEVER** restart services outside our port range
4. **NEVER** modify firewall rules for other port ranges

```bash
# ✅ CORRECT - communitas operations (within 11000-11999)
cargo run -p communitas-headless -- --listen 0.0.0.0:11000
cargo run -p communitas-headless -- --listen 0.0.0.0:11001  # Second instance OK
ssh root@saorsa-2.saorsalabs.com "systemctl restart communitas-bootstrap"

# ❌ WRONG - Would disrupt other networks
ssh root@saorsa-2.saorsalabs.com "pkill -f ':9'"    # NEVER - matches ant-quic ports
ssh root@saorsa-2.saorsalabs.com "pkill -f ':10'"   # NEVER - matches saorsa-node ports
ssh root@saorsa-2.saorsalabs.com "systemctl restart ant-quic-bootstrap"  # NOT OUR SERVICE
```

### Bootstrap Endpoints (communitas)
```
saorsa-2.saorsalabs.com:11000  (NYC - 142.93.199.50)
saorsa-3.saorsalabs.com:11000  (SFO - 147.182.234.192)
```

### Before Any VPS Operations
1. Verify you're targeting port 11000 only
2. Double-check service names contain "communitas"
3. Never run broad `pkill` commands that could affect other services

### Deploy New Binary
```bash
# Build release binary
cargo build -p communitas-headless --release

# Deploy to bootstrap node
scp target/release/communitas-headless root@saorsa-2.saorsalabs.com:/opt/communitas/
ssh root@saorsa-2.saorsalabs.com "systemctl restart communitas-bootstrap"
```

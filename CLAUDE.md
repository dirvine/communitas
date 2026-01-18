# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Communitas is a local-first, PQC-ready collaboration platform that merges WhatsApp, Dropbox, Zoom, and Slack into one decentralized application. It uses connection words (four-word networking) to share peer connection details, provides per-entity virtual disks (org, group, channel, project, individual), and enables DNS-free website publishing via identity-bound website roots.

**Platform Focus**: Cross-platform Flutter application (iOS, Android, Linux, Windows, Web).

## Core Architecture

### Flutter Application
- **Location**: `communitas-flutter/`
- **Framework**: Flutter with Dart
- **Rust Integration**: flutter_rust_bridge for native bindings
- **Platforms**: iOS, Android, Linux, Windows, Web

### Rust Core Library
- **Location**: `communitas-core/`
- **Purpose**: Cross-platform business logic, P2P networking, cryptography
- **Cryptography**: Post-quantum (ML-DSA/ML-KEM) with ChaCha20-Poly1305
- **Storage**: Virtual disks with CRDT synchronization (Yrs)
- **Networking**: QUIC via ant-quic, IPv4-first with Happy Eyeballs fallback

### Key Components
- **Connection Words**: Human-readable encoding for sharing IP:port (e.g., "ocean-forest-moon-star")
- **Virtual Disks**: Private/Public/Shared per entity with different encryption policies
- **Website Publishing**: DNS-free web via identity.website_root binding
- **Messaging**: End-to-end encrypted group messaging with channel support
- **Groups**: Threshold-ready group identities with ML-DSA signatures
- **Kanban System**: CRDT-based collaborative project management (`communitas-kanban/`)
- **Offline-First**: All operations work locally and sync when network available

## Development Commands

### Quick Start - Flutter App
```bash
# Install Flutter dependencies
cd communitas-flutter && flutter pub get

# Run Flutter app (Android)
flutter run -d android

# Run Flutter app (web)
flutter run -d chrome

# Build for release
flutter build apk --release
flutter build web --release
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

### MCP Server (AI Agent Interface)
```bash
# Start MCP server with stdio transport (default)
cargo run -p communitas-mcp -- --demo

# Start MCP server with HTTPS transport (ML-DSA-65 raw public keys)
cargo run -p communitas-mcp -- --http --tls --demo --no-client-auth

# MCP provides Model Context Protocol endpoints for AI agents
# See docs/api/mcp-api.md for details
```

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `communitas-core` | Core business logic, P2P, cryptography |
| `communitas-kanban` | CRDT-based Kanban system |
| `communitas-mcp` | MCP server for AI agents (stdio + HTTPS) |
| `communitas-headless` | Bootstrap/seed nodes |
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

### Flutter-Rust Bridge
Commands flow through flutter_rust_bridge bindings:
1. Flutter/Dart UI calls generated Dart functions
2. flutter_rust_bridge marshals data to Rust
3. Rust core processes request
4. Results return via flutter_rust_bridge to Dart

### Virtual Disk System
Per-entity storage with different access policies:
- **Private**: Encrypted, local-only storage
- **Public**: Content-addressed, distributed storage
- **Shared**: Group-accessible with shared encryption

### Security Model
- **Zero panics/unwraps**: Production Rust code enforces Result types
- **Rate limiting**: Built-in protection against abuse
- **Input validation**: All commands validate inputs
- **Secure storage**: Platform-specific secure storage integration

## Quality Standards

### Rust Code
- Production code: no `unwrap()`, `expect()`, or `panic!()` (use `thiserror`/`anyhow` and return errors; log via `tracing`).
- Tests: `unwrap/expect/panic!` are allowed for clarity and speed.
- Clippy policy: `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`. Do not enable `clippy::pedantic` by default.
- Formatting: `cargo fmt --all` before commits.
- Documentation: Prefer doc comments on public items; add when helpful.

### Flutter/Dart Code
- Follow Flutter best practices with proper state management
- Use null safety properly
- Proper error handling with Result-like patterns
- Accessibility support for all UI components

### Git Workflow
```bash
# Format and check before commit
cargo fmt --all
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used
cargo test
cd communitas-flutter && flutter analyze && flutter test

# Commit with conventional format
git commit -m "feat: add new feature"
git commit -m "fix: resolve issue"
git commit -m "docs: update documentation"
```

## Deployment

### Flutter Application
Cross-platform distribution:
- macOS: DMG or Mac App Store
- iOS: App Store
- Android: Google Play
- Web: Static hosting
- Linux/Windows: Native installers

### Headless Node
Bootstrap and seed nodes for network support:
- Binary: `communitas-headless`
- Config: TOML with listen addresses, storage paths

## Troubleshooting

### Common Issues
- **P2P Connection Failures**: Check bootstrap node connectivity
- **Build Failures**: Ensure Rust 1.85+ and Flutter 3.27+ installed
- **Binding Generation**: Run flutter_rust_bridge code generation after Rust changes

### Windows Build Issues
The project requires CMake and Visual Studio Build Tools on Windows because `ant-quic` depends on `aws-lc-rs` (FIPS 140-3 certified cryptography), which compiles C code.

**Prerequisites:**
- Visual Studio 2022 Build Tools with C++ workload
- CMake 3.20+ (in PATH)
- Rust with MSVC toolchain (default on Windows)

**Known limitations:**
- `cargo build --all-targets` fails due to `libfuzzer-sys` (Linux-only). Use `cargo build --release` instead.
- First build is slow (~1-3 minutes) while compiling AWS Libcrypto.

See [docs/development/windows-build.md](docs/development/windows-build.md) for detailed Windows setup.

### Debug Modes
```bash
# Rust debugging
RUST_LOG=debug cargo run -p communitas-headless

# Test debugging
RUST_LOG=debug cargo test -- --nocapture

# Flutter debugging
flutter run --debug
```

## API Documentation

For detailed API documentation, see:
- `docs/api/core-api.md` - Rust library API (communitas-core)
- `docs/api/mcp-api.md` - MCP server API for AI agents
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

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Communitas is a local-first, PQC-ready collaboration platform that merges WhatsApp, Dropbox, Zoom, and Slack into one decentralized application. It uses connection words (four-word networking) to share peer connection details, provides per-entity virtual disks (org, group, channel, project, individual), and enables DNS-free website publishing via identity-bound website roots.

**Platform Focus**: Two applications — a cross-platform Dioxus + Tauri app (macOS, Windows, Linux; experimental Android/iOS) and a native macOS Swift app (`communitas-apple/`). Both connect to a local x0xd daemon for all networking.

## Core Architecture

### Dioxus Application
- **Location**: `communitas-dioxus/`
- **Framework**: Dioxus + Tauri 2 (all-Rust)
- **State Management**: Signals/hooks backed by `communitas-ui-service`
- **Platforms**: macOS, Windows, Linux (GA) with experimental Android/iOS builds via Tauri runners
- **Daemon**: Requires x0xd; onboarding gate auto-installs it on first run

### Swift Application (macOS)
- **Location**: `communitas-apple/`
- **Framework**: SwiftUI, Swift Package Manager, macOS 14+
- **Targets**: `CommuniTas` (executable) + `X0xClient` (library)
- **Daemon**: Discovers x0xd config from `~/Library/Application Support/x0x/api.port` and `api-token`

### Rust Core Library
- **Location**: `communitas-core/`
- **Purpose**: Cross-platform business logic, P2P networking, cryptography
- **Cryptography**: Post-quantum (ML-DSA/ML-KEM) with ChaCha20-Poly1305
- **Storage**: Virtual disks with CRDT synchronization (Yrs)
- **Networking**: QUIC via ant-quic, IPv4-first with Happy Eyeballs (RFC 8305) dual-stack fallback

### Key Components
- **Connection Words**: Human-readable encoding for sharing IP:port (e.g., "ocean-forest-moon-star")
- **Virtual Disks**: Private/Public/Shared per entity with different encryption policies
- **Website Publishing**: DNS-free web via identity.website_root binding
- **Messaging**: End-to-end encrypted group messaging with editing, deletion, pinning, threading, and inline quotes/replies
- **Emoji Reactions**: Per-message reactions with quick-reaction bar and full categorized emoji picker (with search)
- **Markdown Rendering**: In-message markdown with syntax highlighting
- **@Mentions**: Autocomplete picker with inline user tagging
- **Typing Indicators**: Real-time per-user typing status in channels
- **Presence**: Online/away/offline status badges per peer
- **Message Search**: In-channel search with debounced input and result highlighting
- **Onboarding Gate**: First-run flow that auto-installs and starts x0xd if not present
- **Groups**: Threshold-ready group identities with ML-DSA signatures and member management (add/remove/roles)
- **Kanban System**: CRDT-based collaborative project management (`communitas-kanban/`)
- **Entity Tabs**: Board, Chat, Call, Canvas, Drive, Documents, and Details views per entity type
- **Offline-First**: All operations work locally and sync when network available
- **CRDT Tombstone Compaction**: Configurable retention policies with background compaction tasks
- **Signed Presence Beacons**: ML-DSA signed presence broadcasts with per-peer rate limiting
- **SWIM Failure Detection**: Complete K-peer probing, indirect probes, suspect-to-dead transitions
- **Anti-Entropy Reconciliation**: Set-difference based partition recovery
- **UI Components**: VirtualList, SearchBar, FilterChips, Pagination, ConfirmDialog, ErrorBanner, loading skeletons, empty states

## Development Commands

### Quick Start - Dioxus App
```bash
# Install dx CLI (pinned)
scripts/install_dx.sh

# Run Dioxus desktop app with hot reload
cd communitas-dioxus
dx serve --platform desktop --hotpatch

# Bundle for release
dx bundle --platform desktop
```

### Rust Development
```bash
# Build all Rust crates
cargo build

# Run tests
cargo nextest run

# Format and lint
cargo fmt --all
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

# Build specific crates
cargo build -p communitas-core
cargo build -p communitas-kanban
cargo nextest run -p communitas-core
cargo nextest run -p communitas-kanban
```

### Swift App (Native macOS)
```bash
# Build from command line
swift build --package-path communitas-apple

# Open in Xcode
open communitas-apple/Package.swift
```

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `communitas-core` | Core business logic, P2P, cryptography |
| `communitas-kanban` | CRDT-based Kanban system |
| `communitas-ui-api` | Strongly-typed UI service trait definitions |
| `communitas-ui-service` | Shared Rust UI service implementations (ADR-019) |
| `communitas-x0x-client` | x0xd daemon discovery, HTTP client, WebSocket transport |
| `communitas-bench` | Benchmarks |

## Architecture Insights

### Core Context System
The application uses a centralized `CoreContext` (communitas-core/src/core_context.rs) that wires Communitas to saorsa-gossip v0.5.0 components:
- Identity management with enhanced PQC support
- Storage management with CRDT synchronization (Yrs) and tombstone compaction
- Chat management with persistent storage, message editing/deletion
- Messaging service for real-time communication via gossip overlay
- Kanban service for collaborative project management
- Group key storage for membership updates
- SWIM failure detection with K-peer probing and indirect probes
- Signed presence beacons with per-peer rate limiting
- Anti-entropy reconciliation for partition recovery
- Lock hierarchy enforcement for deadlock-free gossip context

### Shared Rust UI Services
Commands flow through the shared `UiServices` layer (ADR-019):
1. Dioxus components call strongly-typed service traits (auth, directory, messaging, etc.) defined in `communitas-ui-api`.
2. Services invoke `communitas-core` commands/queries via `communitas-ui-service`.
3. Watch channels broadcast state changes back to the UI.

### Virtual Disk System
Per-entity storage with different access policies:
- **Private**: Encrypted, local-only storage
- **Public**: Content-addressed, distributed storage
- **Shared**: Group-accessible with shared encryption

### Security Model
- **Zero panics/unwraps**: Production Rust code enforces Result types
- **Rate limiting**: Built-in protection against abuse, per-peer rate limiting on presence beacons
- **Input validation**: All commands validate inputs
- **Secure storage**: Platform-specific secure storage integration
- **Signed presence**: ML-DSA signed presence beacons prevent spoofing
- **Lock hierarchy**: Audited lock ordering in GossipContext for deadlock-free operation

## Quality Standards

### Rust Code
- Production code: no `unwrap()`, `expect()`, or `panic!()` (use `thiserror`/`anyhow` and return errors; log via `tracing`).
- Tests: `unwrap/expect/panic!` are allowed for clarity and speed.
- Clippy policy: `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`. Do not enable `clippy::pedantic` by default.
- Formatting: `cargo fmt --all` before commits.
- Documentation: Prefer doc comments on public items; add when helpful.

### UI Code
- Follow Dioxus best practices (signals/hooks, context providers)
- Keep UI logic thin; push orchestration into `communitas-ui-service`
- Ensure accessibility (keyboard focus order, screen-reader labels)
- Prefer structured errors surfaced from Rust services
- Use `VirtualList` for large datasets, `SearchBar`/`FilterChips`/`Pagination` for navigation
- Use `ConfirmDialog` for destructive actions, `ErrorBanner` for recoverable errors
- Loading skeletons and empty states are required for all async data-fetching views

### Git Workflow
```bash
# Format and check before commit
cargo fmt --all
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used
cargo nextest run
cd communitas-dioxus && dx check --platform desktop

# Commit with conventional format
git commit -m "feat: add new feature"
git commit -m "fix: resolve issue"
git commit -m "docs: update documentation"
```

## Deployment

### Dioxus Application
Cross-platform distribution:
- macOS: DMG/notarized bundle
- Windows: MSI/NSIS installer (WebView2 bootstrap)
- Linux: AppImage/deb/rpm bundles
- Android/iOS: Experimental Tauri runners (manual provisioning)

### Swift Application
macOS-only distribution via Xcode or `swift build`. Requires x0xd to be installed and running. macOS 14+ minimum deployment target.

## Troubleshooting

### Common Issues
- **P2P Connection Failures**: Check bootstrap node connectivity
- **Build Failures**: Ensure Rust 1.85+, `dx` CLI 0.7.3, and required platform toolchains are installed
- **Tauri Bundles**: Missing WebView dependencies block startup; run installer prerequisites

### WebView Requirements

Communitas requires a platform-specific WebView runtime. The app checks for this at startup and shows a helpful error dialog if missing.

| Platform | WebView | Status |
|----------|---------|--------|
| macOS | WebKit | Always present (bundled with Safari) |
| Linux | WebKitGTK | Must be installed separately |
| Windows | WebView2 | Usually present (Edge/Win11), may need install |

**Install WebView dependencies:**

```bash
# Linux (detect package manager automatically)
sudo scripts/install-webview-linux.sh

# Windows (PowerShell)
scripts\install-webview-windows.ps1
scripts\install-webview-windows.ps1 -UserInstall  # No admin required
```

See [docs/development/prerequisites.md](docs/development/prerequisites.md) for detailed requirements.

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
# Test debugging
RUST_LOG=debug cargo nextest run --no-capture

# UI debugging
RUST_LOG=debug dx serve --platform desktop --hotpatch
```

## API Documentation

For detailed API documentation, see:
- `docs/api/core-api.md` - Rust library API (communitas-core)
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

- We use `four-word-networking` crate to encode/decode IPv4/IPv6 addresses to 4 words for easy sharing
- **Four-words are ONLY for connection bootstrap** - they're a network address (WHERE), not identity (WHO)
- **Identity is the pubkey_hex** - ML-DSA-65 public key that uniquely identifies a user
- **Display name is shown to others** - user-chosen human-friendly label
- Both apps (Dioxus and Swift) read x0xd config from `~/Library/Application Support/x0x/api.port` (address) and `api-token` (Bearer token)
- `communitas-x0x-client` (Rust) and `X0xClient` (Swift) are the two daemon client libraries

This architecture supports rapid development while maintaining production-quality standards for a secure, decentralized collaboration platform.

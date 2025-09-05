# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Communitas is a local-first, PQC-ready collaboration platform that merges WhatsApp, Dropbox, Zoom, and Slack into one decentralized application. It uses Four-Word identities for human-verifiable addressing, provides per-entity virtual disks (org, group, channel, project, individual), and enables DNS-free website publishing via identity-bound website roots.

## Core Architecture

### Frontend (React + TypeScript)
- **Framework**: React 18 with TypeScript, Material-UI components
- **Build Tool**: Vite with Hot Module Replacement for development
- **State Management**: React Context with hooks for auth, encryption, navigation
- **Routing**: React Router for SPA navigation
- **Testing**: Vitest with jsdom for component testing
- **UI Modes**: Dual UI system - Legacy (Material-UI) and Experimental (WhatsApp-style)

### Backend (Tauri v2 + Rust)
- **Runtime**: Tauri v2 with Rust 2024 edition for desktop app framework
- **Core Library**: Saorsa Core v0.3.17 (crates.io) for DHT, QUIC, identities, groups, messaging
- **Cryptography**: Post-quantum (ML-DSA/ML-KEM) with ChaCha20-Poly1305
- **Storage**: Virtual disks with FEC, content addressing via BLAKE3
- **Security**: Keyring integration for secure credential storage
- **Networking**: QUIC via ant-quic, IPv4-first with Happy Eyeballs fallback

### Key Components
- **Four-Word Addresses**: Human-readable network identities (e.g., "ocean-forest-moon-star")
- **Virtual Disks**: Private/Public/Shared per entity with different encryption policies
- **Website Publishing**: DNS-free web via identity.website_root binding
- **Messaging**: End-to-end encrypted group messaging with channel support
- **Groups**: Threshold-ready group identities with ML-DSA signatures

## Development Commands

### Quick Start
```bash
# Install dependencies
npm install

# Start development mode (Tauri + Vite)
npm run tauri dev

# Run tests
npm test                    # Frontend tests
cargo test                  # Backend tests

# Type checking and linting
npm run typecheck          # TypeScript
cargo clippy --all-features -- -D warnings  # Rust
```

### Frontend Development
```bash
npm run dev                 # Start Vite dev server (port 1420)
npm run build              # Build for production
npm run typecheck          # TypeScript checking
npm test                   # Run Vitest tests
npm run test:ui            # Interactive test UI
```

### Backend Development
```bash
cd src-tauri
cargo build                # Build debug
cargo build --release      # Build release
cargo test                 # Run all tests
cargo fmt --all           # Format code
cargo clippy --all-features -- -D warnings  # Lint
```

### Production Build
```bash
npm run tauri build        # Build complete app for distribution
```

## Testing Strategy

### Unit Tests
- Frontend: Vitest for React components in `src/**/*.test.tsx`
- Backend: Cargo tests in `src-tauri/src/**/*.rs` and `src-tauri/tests/`

### Integration Tests
- Multi-node P2P testing: `src-tauri/tests/integration_*.rs`
- Storage policies: `src-tauri/tests/storage_policy_tests.rs`
- DHT operations: `src-tauri/tests/dht_facade_local.rs`

### Running Specific Tests
```bash
# Frontend specific
npm run test:run           # Run specific test suite

# Backend specific
cargo test storage_tests --lib
cargo test saorsa_storage
cargo test integration_

# With logging
RUST_LOG=debug cargo test
```

## Architecture Insights

### Core Context System
The application uses a centralized `CoreContext` (src-tauri/src/core_context.rs) that wires Communitas to saorsa-core components:
- Identity management with enhanced PQC support
- Storage management with DHT integration
- Chat management with persistent storage
- Messaging service for real-time communication
- Group key storage for membership updates

### Tauri Command Structure
Commands are organized by domain in `src-tauri/src/`:
- `core_commands.rs` - Main application commands
- `core_groups.rs` - Group management commands
- Storage, security, and other domains in respective modules

### Virtual Disk System
Per-entity storage with different access policies:
- **Private**: Encrypted, local-only storage
- **Public**: Content-addressed, distributed storage
- **Shared**: Group-accessible with shared encryption

### Security Model
- **Zero panics/unwraps**: Production code enforces Result types
- **Rate limiting**: Built-in protection against abuse
- **Input validation**: All Tauri commands validate inputs
- **Secure storage**: Platform-specific credential managers

## Common Development Tasks

### Adding New Tauri Commands
1. Define command in appropriate module (e.g., `core_commands.rs`)
2. Add to `generate_handler!` in `lib.rs`
3. Add TypeScript types in `src/types/`
4. Call from frontend using `invoke()` from `@tauri-apps/api/tauri`

### Working with Four-Word Identities
```typescript
// Frontend
import { invoke } from '@tauri-apps/api/tauri';

// Initialize identity
await invoke('core_initialize', { 
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Alice',
  deviceName: 'Desktop'
});

// Backend validation
saorsa_core::fwid::fw_check(word_array)
```

### Virtual Disk Operations
```typescript
// Write to private disk
await invoke('core_disk_write', {
  entityHex: entity_id,
  diskType: 'Private',
  path: '/docs/readme.md',
  contentBase64: btoa('content')
});

// Read from disk
const data = await invoke('core_disk_read', {
  entityHex: entity_id,
  diskType: 'Private',
  path: '/docs/readme.md'
});
```

### Website Publishing
```typescript
// Publish website
await invoke('core_website_publish_receipt', {
  entityHex: entity_id,
  websiteRootHex: root_hash
});

// Update identity with website root
await invoke('core_identity_set_website_root', {
  idHex: identity_id,
  websiteRootHex: root_hash,
  sigHex: signature
});
```

## Quality Standards

### Rust Code
- **No panics**: Zero `unwrap()`, `expect()`, `panic!()` in production
- **Zero warnings**: `cargo clippy --all-features -- -D warnings` must pass
- **Formatting**: `cargo fmt --all` before commits
- **Documentation**: All public items must have doc comments

### TypeScript Code
- **Type safety**: No `any` types, strict mode enabled
- **Testing**: Minimum 80% coverage for critical paths
- **Linting**: ESLint rules enforced

### Git Workflow
```bash
# Format and check before commit
cargo fmt --all
cargo clippy --all-features -- -D warnings
npm run typecheck
cargo test
npm test

# Commit with conventional format
git commit -m "feat: add new feature"
git commit -m "fix: resolve issue"
git commit -m "docs: update documentation"
```

## Deployment

### GitHub Pages Website
The project includes a public website at https://communitas.life deployed via GitHub Pages:
- Source: `docs/` directory
- Deployment: `.github/workflows/deploy-pages.yml`
- Design: Matches saorsalabs.com aesthetic with Inter font

### Desktop Application
Built with Tauri for cross-platform distribution:
- macOS: DMG with codesigning
- Windows: MSI installer
- Linux: AppImage and DEB packages

### Headless Node
Bootstrap and seed nodes for network support:
- Binary: `communitas-node`
- Auto-updater: `communitas-autoupdater`
- Config: TOML with listen addresses, storage paths

## Troubleshooting

### Common Issues
- **P2P Connection Failures**: Check bootstrap node connectivity
- **Build Failures**: Ensure Rust 1.85+ and Node.js 20+ installed
- **Test Failures**: Clean `.communitas-data/` directory between test runs

### Debug Modes
```bash
# Frontend debugging
npm run dev  # Enables React DevTools

# Backend debugging
RUST_LOG=debug cargo run

# Test debugging
RUST_LOG=debug cargo test -- --nocapture
```

## API Documentation

For detailed API documentation, see:
- `AGENTS_API.md` - Complete Communitas + Saorsa Core API surface
- `finalise/DEPLOY_TESTNET.md` - Testnet deployment guide
- Saorsa Core docs: https://docs.rs/saorsa-core

## Performance Targets

- **Message Latency**: <100ms local, <500ms remote
- **Storage Operations**: <100ms local, <500ms with geographic routing
- **UI Responsiveness**: 60fps, <16ms frame time
- **Memory Usage**: <200MB baseline

## Security Considerations

- All external links must use HTTPS
- Canonical signing for sensitive updates
- Zero centralized dependencies for core functionality
- Anti-phishing via Four-Word checksum validation
- Rate limiting on all public endpoints

This architecture supports rapid development while maintaining production-quality standards for a secure, decentralized collaboration platform.
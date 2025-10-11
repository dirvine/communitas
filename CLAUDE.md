# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Communitas is a local-first, PQC-ready collaboration platform that merges WhatsApp, Dropbox, Zoom, and Slack into one decentralized application. It uses Four-Word identities for human-verifiable addressing, provides per-entity virtual disks (org, group, channel, project, individual), and enables DNS-free website publishing via identity-bound website roots.

## Core Architecture

> **Recent Update (2025-10-11)**: Rust backend with P2P networking, mesh capabilities, and desktop functionality has been restored. The previous web-only refactor (commit c383ce0a) was reverted to align with product requirements for desktop/mobile apps with essential P2P and offline mesh networking features.

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
- **Storage**: Virtual disks with content addressing via BLAKE3
- **Security**: Keyring integration for secure credential storage
- **Networking**: QUIC via ant-quic, IPv4-first with Happy Eyeballs fallback

### Key Components
- **Four-Word Addresses**: Human-readable network identities (e.g., "ocean-forest-moon-star")
- **Virtual Disks**: Private/Public/Shared per entity with different encryption policies
- **Website Publishing**: DNS-free web via identity.website_root binding
- **Messaging**: End-to-end encrypted group messaging with channel support
- **Groups**: Threshold-ready group identities with ML-DSA signatures
- **Network Connection**: Auto-connects on startup with retry logic and graceful fallback to local mode
- **Offline-First**: All operations work offline via IndexedDB and sync when network returns

## Chrome DevTools MCP Integration

### Overview
The project includes Chrome DevTools MCP integration for advanced browser debugging and testing capabilities. This provides AI-assisted analysis of performance, UI issues, network requests, and React-specific debugging.

### Configuration
Chrome DevTools MCP is configured in `.mcp.json` (project root):
```json
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": ["chrome-devtools-mcp@latest"]
    }
  }
}
```

### Testing Capabilities
- **Performance Metrics**: LCP, FCP, CLS, loading sequences
- **Error Detection**: Console errors, network failures, runtime issues
- **Network Analysis**: Request monitoring, WebSocket connections, bundle sizes
- **UI Validation**: DOM structure, CSS issues, component rendering
- **React Debugging**: Re-render issues, Context usage, memory leaks
- **Authentication Testing**: Login/logout flows, session management, user menu access

### Usage
The Chrome DevTools MCP can be used to test the web application at `http://localhost:5173/` (or port 5001 when serving built files).

Key test areas for Communitas:
- **Authentication flow**:
  - ✅ User registration with automatic four-word identity generation
  - ✅ Login with four-word address
  - ✅ Single-click logout via improved user menu
  - ✅ Session persistence with encrypted localStorage
  - ✅ Password strength validation
  - ✅ Passkey/WebAuthn support
- **UI/UX improvements**:
  - ✅ Dropdown arrow indicator on user avatar
  - ✅ Professional menu design with user info header
  - ✅ Network status display in menu
- Theme switching (light/dark mode transitions)
- Network connectivity (P2P connections, offline mode)
- Tauri IPC communication
- IndexedDB offline storage

### Recent Improvements

**2025-09-30: Enhanced Testing & TypeScript Quality**
- **Bridge Server (communitas-bridge)**: HTTP/REST bridge for browser-based testing via Chrome DevTools MCP
  - Real P2P integration with saorsa-core
  - Endpoints: health, initialize, channels, messages, threads
  - See `docs/BRIDGE_TESTING.md` for comprehensive testing guide
- **Thread Reply Composer**: Automerge-integrated reply system for threads
  - Optimistic updates with offline-first persistence
  - Syncs to backend when network available
- **TypeScript Error Resolution**: Fixed all 14 type errors
  - `npm run typecheck` now passes cleanly
  - Auth components, navigation, GlassCard, theme types all corrected
- **Code Quality**: 137 of 141 tests passing, zero TypeScript warnings

**2025-09-27: Authentication & UX**
- **Fixed logout button visibility**: Now accessible with single click on avatar
- **Improved authentication UX**: Professional user menu with clear options
- **Created UnifiedAuthFlow component**: Modern authentication UI with glassmorphism effects
- **Verified encrypted storage**: Web Crypto API with PBKDF2 and AES-GCM encryption
- **Tested complete auth flow**: Registration, login, logout all working correctly

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
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used  # Rust
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
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used  # Lint
```

### Production Build
```bash
npm run tauri build        # Build complete app for distribution
```

### Bridge Server (Testing Mode)
```bash
# Terminal 1: Start bridge server
cargo run -p communitas-bridge

# Terminal 2: Start frontend dev server
npm run dev

# Bridge server provides HTTP/REST endpoints at http://localhost:3030
# See docs/BRIDGE_TESTING.md for complete testing guide
```

## Testing Strategy

### Browser-Based Testing with Bridge Server
The communitas-bridge crate provides an HTTP/REST interface for testing with Chrome DevTools MCP:

**Architecture**:
```
Browser (Chrome DevTools MCP)
    ↓ HTTP/REST
Bridge Server (localhost:3030)
    ↓ Rust IPC
Saorsa Core (P2P Network)
```

**Available Endpoints**:
- `GET /health` - Health check
- `POST /api/core/initialize` - Initialize with four-word identity
- `POST /api/channels` - Create channel
- `GET /api/channels` - List channels
- `POST /api/channels/:id/messages` - Send message
- `POST /api/threads/create` - Create thread from message

**Example Test Flow**:
```javascript
// 1. Initialize core
await fetch('http://localhost:3030/api/core/initialize', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    four_words: 'ocean-forest-moon-star',
    display_name: 'Test User',
    device_name: 'Browser Test'
  })
})

// 2. Create channel
const channelResp = await fetch('http://localhost:3030/api/channels', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'Test Channel',
    description: 'Created from browser'
  })
})

// 3. Send message
const channel = await channelResp.json()
await fetch(`http://localhost:3030/api/channels/${channel.id}/messages`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    content: 'Hello from browser!',
    recipients: ['ocean-forest-moon-star']
  })
})
```

See `docs/BRIDGE_TESTING.md` for complete testing scenarios and Chrome DevTools MCP integration examples.

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

### Working with Network Connection
```typescript
// Network service is a singleton that auto-connects on startup
import { networkService } from './services/network/NetworkConnectionService';

// Check network status
const state = networkService.getState();
console.log(state.status); // 'connecting' | 'connected' | 'offline' | 'local' | 'error'

// Subscribe to network changes
const unsubscribe = networkService.subscribe((state) => {
  console.log('Network changed:', state.status);
});

// Manual control
await networkService.connect();    // Try to connect
await networkService.disconnect(); // Go to local mode

// Testing in console
window.testNetwork.status();        // Check status
window.testNetwork.simulateOffline(); // Test offline
window.testNetwork.testFlow();      // Run complete test
```

### Offline-First Storage
```typescript
import { offlineStorage } from './services/storage/OfflineStorageService';

// Store data (works offline)
await offlineStorage.store('key', data, {
  ttl: 3600000,        // 1 hour cache
  encrypt: true,       // Encrypt sensitive data
  syncOnline: true     // Sync when network returns
});

// Retrieve data (from cache first, then network)
const data = await offlineStorage.get('key');

// Queue operation for sync
await offlineStorage.queueForSync({
  type: 'create',
  entity: 'message',
  data: messageData
});
```

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
- Production code: no `unwrap()`, `expect()`, or `panic!()` (use `thiserror`/`anyhow` and return errors; log via `tracing`).
- Tests: `unwrap/expect/panic!` are allowed for clarity and speed.
- Clippy policy: `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`. Do not enable `clippy::pedantic` by default.
- Formatting: `cargo fmt --all` before commits.
- Documentation: Prefer doc comments on public items; add when helpful.

### TypeScript Code
- **Type safety**: No `any` types, strict mode enabled
- **Testing**: Minimum 80% coverage for critical paths
- **Linting**: ESLint rules enforced

### Git Workflow
```bash
# Format and check before commit
cargo fmt --all
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used
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
  - App auto-falls back to local mode when network unavailable
  - Check network status indicator in header (green=connected, yellow=local/connecting, red=error)
  - Click indicator to manually reconnect
  - Use `window.testNetwork.status()` in console to debug
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

## MCP Tooling Overview

Communitas now relies solely on the **Chrome DevTools MCP** defined in `.mcp.json`. The legacy Tauri-side MCP plugin and helper scripts have been removed to simplify automation and reduce local attack surface. To inspect or automate the running app during development:

1. Start the dev server (`npm run dev`).
2. Launch the Chrome DevTools MCP inspector:

```bash
npx chrome-devtools-mcp@latest --browserUrl http://127.0.0.1:1420
```

The inspector exposes structured DOM inspection, screenshot capture, and scripted automation through Chrome's debugging protocol without requiring custom socket servers.


## API Documentation

For detailed API documentation, see:
- `AGENTS_API.md` - Complete Communitas + Saorsa Core API surface
- `AGENTS.md` - Agent automation guide and MCP usage examples
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
- remember how we have setup a test network and mcp
- we use four-word-networking crate to encode/decode ip4 and 6 to 4 or more words. We also use the crate for all our identities and ensure the identities are all valid words from our four-word-networkign dictionary
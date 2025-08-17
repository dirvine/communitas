# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Communitas is a decentralized P2P collaboration platform built with Tauri, React, and Rust. It provides secure messaging, file sharing, and collaborative features using the Saorsa Core networking stack with four-word addresses for human-readable network identities.

## Core Architecture

### Frontend (React + TypeScript)
- **Framework**: React 18 with TypeScript, Material-UI components
- **Build Tool**: Vite with Hot Module Replacement for development
- **State Management**: React Context with hooks for auth, encryption, navigation
- **Routing**: React Router for SPA navigation
- **Testing**: Vitest with jsdom for component testing

### Backend (Tauri + Rust)
- **Runtime**: Tauri v2 with Rust 2024 edition for desktop app framework
- **P2P Networking**: Saorsa Core for DHT-based peer discovery and communication
- **Cryptography**: Post-quantum cryptography (ML-KEM/ML-DSA) with ChaCha20-Poly1305
- **Storage**: Saorsa Storage System with four policies, Reed-Solomon FEC for distributed storage
- **Security**: Keyring integration for secure credential storage

### Saorsa Storage System (NEW)
- **Storage Policies**: PrivateMax, PrivateScoped, GroupScoped, PublicMarkdown with different encryption/access patterns
- **Content Addressing**: BLAKE3 hashing with optimal 256KB chunking for large files
- **Namespace Isolation**: HKDF-SHA256 key derivation for secure data separation
- **Local Caching**: LRU cache with gzip compression and BLAKE3 integrity verification
- **Performance**: <100ms local operations, <500ms remote operations with geographic routing
- **Error Handling**: Comprehensive Result types with zero unwrap/panic in production code
- **Quality Standards**: Zero compilation warnings, >85% test coverage, security audit compliant

### Key Components
- **Four-Word Addresses**: Human-readable network identities (e.g., "ocean-forest-moon-star")
- **Organization Management**: Hierarchical structures with organizations, groups, and projects
- **Real-time Collaboration**: Yjs CRDTs for collaborative editing, WebRTC for direct communication
- **Geographic Routing**: Location-aware peer selection for optimal performance

## Development Commands

### Frontend Development
```bash
# Start Vite development server (port 1420)
npm run dev

# Build frontend for production
npm run build

# Run TypeScript type checking
npm run typecheck

# Run frontend tests
npm test
npm run test:run  # Run once without watch mode
npm run test:ui   # Interactive test UI
```

### Tauri Development
```bash
# Start full Tauri development mode (frontend + backend)
npm run tauri dev

# Build Tauri application for production
npm run tauri build

# Tauri CLI commands
npm run tauri
```

### Rust Backend
```bash
cd src-tauri

# Run Rust tests
cargo test

# Run Saorsa storage tests specifically
cargo test storage_tests --lib
cargo test saorsa_storage

# Run specific test patterns
cargo test integration_
cargo test --test integration_dht_storage

# Run with logging
RUST_LOG=debug cargo test

# Check code formatting and linting
cargo fmt
cargo clippy

# Build Rust components only
cargo build
cargo build --release
```

### Testing Strategy
- **Unit Tests**: Vitest for React components, Cargo test for Rust modules
- **Integration Tests**: Multi-node P2P testing in `src-tauri/tests/`
- **Property Tests**: Using `proptest` crate for Rust property-based testing
- **E2E Tests**: Playwright configuration available for full app testing

## Architecture Insights

### Dual UI System
The application supports both legacy and experimental UI modes:
- **Legacy Mode**: Traditional Material-UI with responsive layout
- **Experimental Mode**: WhatsApp-style navigation with unified design system
- **Feature Flags**: Progressive migration controlled by `featureFlags` service

### P2P Network Integration
- **Bootstrap Nodes**: Hardcoded connection to `159.89.81.21:9001` and related ports
- **DHT Operations**: Kademlia-based distributed hash table with K=8 replication
- **Identity System**: Ed25519 keypairs mapped to four-word addresses
- **Message Routing**: Geographic routing with automatic NAT traversal

### Security Model
- **Encryption**: AES-256 for local data, ChaCha20-Poly1305 for messages
- **Key Management**: Platform-specific secure storage (Keychain, Credential Manager, etc.)
- **Authentication**: Rate limiting and input validation on all Tauri commands
- **Memory Safety**: Rust backend prevents common security vulnerabilities

### Storage Architecture
- **Local Storage**: SQLite databases in `.communitas-data/` directory
- **Distributed Storage**: DHT-based with Reed-Solomon error correction
- **File Chunking**: 256KB chunks with content addressing using BLAKE3
- **Collaborative Documents**: Yjs with WebRTC and IndexedDB persistence

### Organization Structure
```
├── Organizations (top-level entities)
│   ├── Groups (teams within organizations)
│   │   ├── Projects (work items)
│   │   └── Members (user permissions)
│   └── Resources (shared files and documents)
└── Personal (individual user space)
    ├── Contacts (four-word address book)
    ├── Groups (personal friend groups)
    └── Files (personal storage)
```

## Common Development Tasks

### Adding New Tauri Commands
1. Define command function in `src-tauri/src/main.rs`
2. Add to `invoke_handler!` macro
3. Add TypeScript types in `src/types/`
4. Call from frontend using `invoke()` from `@tauri-apps/api/core`

### Testing P2P Features
1. Use `TestHarness` in `src-tauri/tests/` for multi-node scenarios
2. Bootstrap connection testing via `test-p2p-connection.js`
3. Geographic routing tests in `geographic_commands.rs`

### Working with Four-Word Addresses
- Generation: `identity_commands::generate_four_word_identity`
- Validation: `identity_commands::validate_four_word_identity`
- DHT mapping: Uses BLAKE3 hash of public key for deterministic addressing

### Adding New UI Components
1. Create in `src/components/[category]/`
2. Export from `src/components/[category]/index.ts`
3. Add to appropriate tab or route in `App.tsx`
4. Consider both legacy and experimental UI modes

### Storage Integration

#### Saorsa Storage System
- **Tauri Commands**: Complete API in `saorsa_storage_commands.rs` for all storage operations
- **Storage Policies**: Choose appropriate policy based on data sensitivity and access patterns
- **Frontend Integration**: TypeScript types and React hooks available for seamless integration
- **Performance Testing**: Built-in performance validation with configurable benchmarks

#### Storage Policy Selection Guide
```rust
// Maximum security - local only storage
StoragePolicy::PrivateMax

// Namespace-scoped private storage with DHT backup
StoragePolicy::PrivateScoped { namespace: "user:documents".to_string() }

// Group-accessible storage with shared encryption
StoragePolicy::GroupScoped { group_id: "team-alpha".to_string() }

// Public markdown content with convergent encryption
StoragePolicy::PublicMarkdown
```

#### Legacy Storage Integration
- Local files: Use `stores.rs` commands for markdown file management
- DHT storage: Use `organization.rs` for distributed data
- Encryption: Integrate with `secure_storage.rs` for sensitive data

## Platform-Specific Notes

### macOS
- Keychain integration for secure storage
- Codesigning required for distribution builds

### Windows
- Windows Credential Manager for secure storage
- MSI installer generation via Tauri

### Linux
- libsecret/Secret Service integration
- AppImage and DEB package generation

## Performance Considerations

- **Message Latency**: Target <100ms LAN, <500ms WAN
- **UI Responsiveness**: <16ms frame time for 60fps
- **Memory Usage**: <200MB baseline, efficient Rust backend
- **Bandwidth**: Adaptive based on connection quality

## Troubleshooting

### Common Issues
- **P2P Connection Failures**: Check bootstrap node connectivity and firewall settings
- **Secure Storage Errors**: Verify platform-specific credential systems are available
- **Build Failures**: Ensure Rust 1.85+ and Node.js 18+ are installed
- **Test Failures**: Clean `.communitas-data/` directory between test runs

### Debug Modes
- Frontend: `npm run dev` enables React DevTools and HMR
- Backend: `RUST_LOG=debug cargo test` for detailed logging
- P2P: DHT event monitoring via `dht_events.rs` diagnostics

This architecture supports both rapid development and production deployment of a secure, decentralized collaboration platform.
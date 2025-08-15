# Communitas Foundation

A decentralized P2P chat and collaboration platform built on the Saorsa Core networking stack. Communitas provides secure messaging, file sharing, and collaborative features without relying on central servers.

## Overview

Communitas is a Tauri-based desktop application that leverages peer-to-peer networking for private, secure communication. Built with modern web technologies and Rust, it offers:

- **Decentralized Architecture**: No central servers or single points of failure
- **End-to-End Encryption**: All communications secured with post-quantum cryptography
- **Four-Word Addresses**: Human-readable network addresses for easy sharing
- **Real-time Collaboration**: Live document editing and project management
- **Cross-Platform**: Available on Windows, macOS, and Linux

## Architecture

### Frontend
- **React 18** with TypeScript for the user interface
- **Material-UI (MUI)** for consistent design components
- **Vite** for fast development and building
- **Framer Motion** for smooth animations
- **Monaco Editor** for code editing capabilities

### Backend (Tauri)
- **Rust 2024** for high-performance, memory-safe operations
- **Saorsa Core** for P2P networking and DHT functionality
- **Reed-Solomon FEC** for error correction and data integrity
- **SQLite** for local data storage
- **Keyring** for secure credential storage

### Key Features

#### Secure Messaging
- ChaCha20-Poly1305 AEAD encryption
- Ed25519 digital signatures
- X25519 key exchange
- Forward secrecy with automatic key rotation

#### Collaborative Editing
- **Yjs** for conflict-free replicated data types (CRDTs)
- **WebRTC** for direct peer-to-peer connections
- **IndexedDB** for offline persistence
- Real-time synchronization across all participants

#### P2P Networking
- DHT-based peer discovery
- Geographic routing for efficient message delivery
- Quantum-resistant cryptographic protocols
- Automatic NAT traversal and hole punching

## Installation

### Prerequisites
- **Node.js** 18+ 
- **Rust** 1.85+ with 2024 edition support
- **Tauri CLI** 2.0+

### Development Setup

```bash
# Clone the repository
git clone https://github.com/dirvine/communitas-foundation.git
cd communitas-foundation

# Install frontend dependencies
npm install

# Install Tauri CLI if not already installed
npm install -g @tauri-apps/cli

# Start development server
npm run tauri dev
```

### Building for Production

```bash
# Build the frontend
npm run build

# Build the Tauri application
npm run tauri build
```

## Usage

### Getting Started

1. **Launch Communitas** - Start the application
2. **Create Identity** - Generate your four-word network address
3. **Join Network** - Connect to the P2P network automatically
4. **Start Chatting** - Add contacts using their four-word addresses

### Four-Word Addresses

Communitas uses human-readable four-word addresses instead of complex cryptographic keys:

```
example: "apple-mountain-river-sunset"
```

These addresses are:
- Easy to remember and share
- Globally unique across the network
- Automatically generated from cryptographic keys
- Work across all network configurations

### Secure Storage

All sensitive data is encrypted and stored securely:
- **Keyring Integration**: Platform-specific secure storage
- **Local Encryption**: AES-256 for local database
- **Key Derivation**: Argon2 for password-based keys
- **Memory Protection**: Zeroization of sensitive data

## Development

### Project Structure

```
communitas-foundation/
├── src/                    # React frontend source
│   ├── components/         # Reusable UI components
│   ├── services/          # Frontend services and API calls
│   ├── theme/             # MUI theme customization
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend source
│   ├── src/               # Tauri application code
│   ├── tests/             # Integration tests
│   └── Cargo.toml         # Rust dependencies
├── public/                # Static assets
└── package.json           # Node.js dependencies
```

### Testing

```bash
# Run frontend tests
npm test

# Run Rust tests
cd src-tauri
cargo test

# Run integration tests
cargo test --test integration_*
```

### Available Scripts

- `npm run dev` - Start Vite development server
- `npm run build` - Build frontend for production
- `npm run typecheck` - Run TypeScript type checking
- `npm run test` - Run frontend test suite
- `npm run tauri dev` - Start Tauri development mode
- `npm run tauri build` - Build Tauri application

## Security

### Cryptographic Protocols

Communitas implements multiple layers of security:

1. **Transport Layer**: TLS 1.3 with quantum-resistant algorithms
2. **Message Layer**: ChaCha20-Poly1305 for authenticated encryption
3. **Key Exchange**: X25519 with post-quantum backup
4. **Digital Signatures**: Ed25519 for message authentication
5. **Key Derivation**: HKDF with SHA-256 for key expansion

### Threat Model

Protected against:
- **Network Eavesdropping**: All traffic encrypted
- **Man-in-the-Middle**: Certificate pinning and key verification
- **Quantum Attacks**: Post-quantum cryptographic fallbacks
- **Forward Compromise**: Perfect forward secrecy
- **Metadata Leakage**: Onion routing for anonymity

### Security Audits

Regular security assessments include:
- Static code analysis with Clippy
- Dependency vulnerability scanning
- Fuzz testing of critical components
- Formal verification of cryptographic implementations

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Standards

- **Rust 2024 Edition** for all backend code
- **TypeScript** for all frontend code
- **Comprehensive Testing** with >90% coverage
- **Security First** approach to all features
- **Documentation** for all public APIs

### Code Style

- Rust: `cargo fmt` and `cargo clippy`
- TypeScript: ESLint and Prettier
- Commit messages: Conventional Commits format

## License

This project is dual-licensed under:

- **AGPL-3.0-or-later** for open source use
- **Commercial License** for proprietary use

See [LICENSE-AGPL-3.0](LICENSE-AGPL-3.0) for open source terms.
For commercial licensing, contact: saorsalabs@gmail.com

## Support

- **Documentation**: [docs.rs/communitas-tauri](https://docs.rs/communitas-tauri)
- **Issues**: [GitHub Issues](https://github.com/dirvine/communitas-foundation/issues)
- **Discussions**: [GitHub Discussions](https://github.com/dirvine/communitas-foundation/discussions)
- **Contact**: saorsalabs@gmail.com

## Roadmap

### Current Features (v0.1.0)
- ✅ Basic P2P messaging
- ✅ Four-word address system
- ✅ End-to-end encryption
- ✅ File sharing
- ✅ Cross-platform support

### Planned Features (v0.2.0)
- 🔄 Voice and video calls
- 🔄 Group chat rooms
- 🔄 Collaborative document editing
- 🔄 Plugin system
- 🔄 Mobile applications

### Future Features (v1.0.0)
- 📋 Federation with other networks
- 📋 Advanced privacy features
- 📋 Enterprise integrations
- 📋 Blockchain integration
- 📋 AI-powered features

## Related Projects

- **[Saorsa Core](https://github.com/dirvine/saorsa-core-foundation)** - P2P networking foundation
- **[Saorsa MLS](https://github.com/dirvine/saorsa-mls-foundation)** - Message Layer Security
- **[Saorsa FEC](https://github.com/dirvine/saorsa-fec-foundation)** - Forward Error Correction
- **[Saorsa RSPS](https://github.com/dirvine/saorsa-rsps-foundation)** - Routing and Storage Protocol

## Acknowledgments

Built with love using:
- [Tauri](https://tauri.app/) - Rust-powered app framework
- [React](https://reactjs.org/) - User interface library
- [Material-UI](https://mui.com/) - React component library
- [Yjs](https://yjs.dev/) - Shared data types for collaboration
- [Vite](https://vitejs.dev/) - Frontend build tool

---

**Communitas Foundation** - Connecting people without compromising privacy.
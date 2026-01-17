# Communitas — The Unstoppable Collaboration Platform

> **A partition-tolerant, post-quantum secure, peer-to-peer collaboration network that works when the internet doesn't.**

## Why Communitas?

Most modern collaboration tools (Slack, Discord, Google Docs) rely on a fragile assumption: **that you always have a perfect connection to a central server.** If the internet goes down, or a cable is cut, or a server outage occurs, you stop working.

**Communitas is different.** It flips the model:

1.  **Internet-Optional**: It prioritizes **local-first** connectivity. If the global internet fails, Communitas automatically switches to your local LAN, mesh network, or direct device-to-device links. Teams in the same building can keep chatting and editing documents even if the outside world is cut off.
2.  **Post-Quantum Security**: We don't just use standard encryption. We use **NIST-standard Post-Quantum Cryptography** (ML-DSA, ML-KEM) to protect your identity and data against future threats that could break today's encryption.
3.  **True Peer-to-Peer**: There are no central servers to hack, subpoena, or crash. **You are the server.** Your data lives on your device and syncs directly with your peers.
4.  **Conflict-Free**: Using advanced **CRDTs (Conflict-free Replicated Data Types)**, you can edit documents offline or on a split network, and they will mathematically merge perfectly when you reconnect—no "merge conflicts" or lost work.

---

## Network Resilience Architecture

Communitas implements a hierarchical resilience model spanning process-local to global internet connectivity, with automatic degradation and recovery:

- **Partition Tolerance**: Groups may fragment into isolated subnetworks and automatically reconverge when connectivity restores
- **CRDT Synchronization**: Conflict-free replicated data types ensure eventual consistency across network partitions without coordination
- **Post-Quantum Security**: ML-DSA-87/ML-DSA-65 signatures and ML-KEM-768 key exchange provide quantum-resistant cryptographic verification
- **Multi-Transport Discovery**: Operates across loopback, LAN broadcast, NAT-traversed WAN, and direct public IP without central coordination
- **Catastrophic Failure Recovery**: System continues operation in local-only mode during global infrastructure failures, automatically resuming WAN operations upon restoration

Technical implementation verified through comprehensive integration testing (watchdog monitoring, exponential backoff retry, and resource limit enforcement). See [Offline Handling](docs/architecture/offline-handling.md) and [Networking](docs/architecture/networking.md) for formal specifications.

---

## Quick Start

### Prerequisites

**All Platforms:**
- Flutter 3.27+
- Rust 1.85+

**Windows Additional Requirements:**
- Visual Studio 2022 Build Tools (C++ workload)
- CMake 3.20+ (required by aws-lc-sys for FIPS-certified cryptography)
- See [Windows Build Guide](docs/development/windows-build.md) for detailed setup

**Linux:**
- Build essentials, CMake, and platform libraries
- See Flutter docs for GTK/WebKit dependencies

### Development Setup
```bash
git clone https://github.com/dirvine/communitas.git
cd communitas

# Flutter app development
cd communitas-flutter
flutter pub get
flutter run -d android  # or: -d ios, -d linux, -d windows
# Web demo (FFI not available in browser)
flutter run -d chrome --dart-define=DEMO_MODE=true
```

### Testing
```bash
# Flutter tests
cd communitas-flutter
flutter analyze
flutter test

# Rust linting (strict policy)
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

# Rust unit tests
cargo test
```

---

## Technical Capabilities

### Partition Tolerance & Failure Recovery
- **Network Partition Healing**: CRDT-based automatic state reconciliation across partition boundaries
- **Internet Collapse Detection**: 10-second watchdog monitoring with automatic local-only mode activation
- **Exponential Backoff Retry**: Jittered retry strategies prevent thundering herd during recovery (100ms → 60s backoff)
- **Multi-Layer Connectivity**: Hierarchical degradation from global internet → NAT-traversed WAN → LAN broadcast → loopback
- **Resource Limits (Partial Enforcement)**: Configurable peer connection limits (default: 50), memory caps (2GB), and connection timeouts; enforcement is being integrated across subsystems

### Cryptographic Security (Post-Quantum)
- **ML-DSA-87 Signatures**: NIST FIPS 204 quantum-resistant digital signatures for user identity (192-bit quantum security, Level 5)
- **ML-DSA-65 Signatures**: NIST FIPS 204 signatures for site/gossip identity (128-bit quantum security, Level 3)
- **ML-KEM-768 Key Exchange**: NIST FIPS 203 quantum-resistant key encapsulation for session establishment
- **ChaCha20-Poly1305 AEAD**: Authenticated encryption for all data at rest and in transit
- **Connection Words (four-word networking)**: Human-memorable encoding of IP:port for sharing peer connection info
- **Zero Central Authority**: Fully decentralized trust model with cryptographic verification replacing DNS/PKI

### CRDT-Based Eventual Consistency
- **Yrs CRDT (v0.19)**: Conflict-free replicated data types for documents, messages, and shared state
- **Operation-Based Synchronization**: Delta-based sync protocol minimizes bandwidth during partition healing
- **Anti-Entropy Protocol**: 60-second background synchronization with adaptive intervals based on network conditions
- **Causal Consistency**: Vector clocks ensure causal ordering of operations across partitioned replicas
- **Automatic Merge**: Conflict-free convergence without manual intervention or consensus protocols

### Decentralized Network Architecture
- **QUIC Transport**: saorsa-gossip-transport (AntQuicTransport on ant-quic v0.18)
- **Gossip Overlay (saorsa-gossip v0.2.0)**: HyParView membership + SWIM failure detection + Plumtree broadcast
- **FOAF Discovery**: Friend-of-a-friend peer discovery without DHT or global indexing
- **Rendezvous Shards**: 65,536-shard distributed discovery system for global user location
- **No Single Point of Failure**: Operates without bootstrap nodes after initial peer cache seeding

### Entity-Based Collaboration
- **Individuals**: Personal identity with ML-DSA-87 keypairs, encrypted local storage
- **Groups**: CRDT-synchronized shared state, partition-tolerant membership
- **Organizations**: Multi-channel hierarchy with admin delegation
- **Projects**: Version-controlled workspaces with conflict-free document merging
- **Channels**: Topic-scoped pubsub with message anti-entropy

---

## Documentation

### Product & Architecture
- **[App Specification](docs/APP_SPECIFICATION.md)**: Product requirements and UX expectations
- **[Architecture Overview](docs/architecture/README.md)**: System architecture (Flutter + Rust core + gossip)
- **[CRDT System](docs/architecture/crdt-system.md)**: Yrs document model and sync
- **[Gossip Protocol](docs/architecture/gossip-protocol.md)**: P2P membership + dissemination
- **[Networking](docs/architecture/networking.md)**: QUIC transport, NAT traversal, resilience
- **[Offline Handling](docs/architecture/offline-handling.md)**: Auto-queue and recovery flow
- **[Security](docs/architecture/security.md)**: PQC, vaults, threat model
- **[Storage](docs/architecture/storage.md)**: Virtual disks and content addressing
- **[ADR Index](docs/adr/README.md)**: Architecture decisions

### API Reference
- **[API Overview](docs/api/README.md)**: FFI, core, MCP surfaces
- **[Core API](docs/api/core-api.md)**: Rust core library API
- **[MCP API](communitas-mcp/README.md)**: AI agent interface (stdio/HTTP)

### Deployment & Ops
- **[Headless Service](communitas-headless/README.md)**: systemd/launchd service with config-driven startup
- **[Testnet Deployment](finalise/DEPLOY_TESTNET.md)**: Complete network deployment
- **[Infrastructure](docs/infrastructure/INFRASTRUCTURE.md)**: Infra layout and environments

### Development
- **[Contributing Guide](CONTRIBUTING.md)**: How to contribute
- **[Windows Build](docs/development/windows-build.md)**: Windows setup notes
- **[CLAUDE.md](CLAUDE.md)**: Project context for LLM helpers

---

## Project Structure

### Applications
- **[communitas-flutter/](communitas-flutter/)**: Cross-platform Flutter application (macOS, iOS, Android, Linux, Windows; web demo only)
- **[communitas-headless/](communitas-headless/)**: Headless daemon for system services ([README](communitas-headless/README.md))
- **[communitas-mcp/](communitas-mcp/)**: MCP server for AI agent control (stdio + HTTPS with ML-DSA-65)

Flutter is the only supported GUI; MCP is the integration surface for other local apps and automations.

### Core Libraries
- **[communitas-core/](communitas-core/)**: Shared Rust business logic and P2P networking
- **[communitas-kanban/](communitas-kanban/)**: CRDT-based collaborative Kanban system

### Documentation
- **[docs/](docs/)**: Comprehensive project documentation
  - **[architecture/](docs/architecture/)**: System architecture documentation
  - **[api/](docs/api/)**: API reference documentation
  - **[development/](docs/development/)**: Development setup and standards
  - **[testing/](docs/testing/)**: Multi-node testing and scenarios
  - **[infrastructure/](docs/infrastructure/)**: Deployment and infrastructure
  - **[adr/](docs/adr/)**: Architecture decision records

### Key Commands
```bash
# Flutter Development
cd communitas-flutter
flutter pub get
flutter run -d android  # or: -d ios, -d linux, -d windows

# Production Build
flutter build apk --release
# or: flutter build ios --release
# Web demo build only (no native FFI in browser)
flutter build web --release --dart-define=DEMO_MODE=true

# Quality Checks
flutter analyze && cargo clippy --all-features

# Run as system service (see communitas-headless/README.md)
communitas-headless --config /etc/communitas/headless.toml
```

---

## Deployment Options

Communitas supports multiple deployment scenarios for different use cases:

### Flutter Application (End Users)
Full-featured cross-platform application for iOS, Android, Linux, and Windows.
```bash
cd communitas-flutter
flutter build apk --release
# or: flutter build ios --release
# Web demo build only (FFI not available in browser)
flutter build web --release --dart-define=DEMO_MODE=true
```
See [communitas-flutter/](communitas-flutter/) for details.

### Headless Daemon (Servers & Bots)
Run as a system service for automated operations, bots, or server infrastructure.
```bash
# Install and run as systemd service
sudo systemctl enable communitas
sudo systemctl start communitas
```
Complete guide: [communitas-headless/README.md](communitas-headless/README.md)

### MCP Server (AI Agent Interface)
Model Context Protocol server for AI agent control with HTTPS (ML-DSA-65 raw public keys).
```bash
# HTTPS transport with demo mode
cargo run -p communitas-mcp -- --http --tls --demo --no-client-auth
```
Complete guide: [communitas-mcp/README.md](communitas-mcp/README.md)

---

## Network & Identity

Identity is the public key (pubkey_hex). Four-word networking is used only to encode
connection endpoints (IP:port) for sharing between peers.

### Connection Words Example
```dart
// Share a connection address with a friend (IP:port encoded as words)
final connectionWords = await getMyConnectionWords();
// → "ocean-blue-eagle-star"

// Friend uses the connection words to dial directly
await connectToPeer(connectionWords);
```

### Network Participation
- **Desktop Nodes**: Full participants with UI (Flutter macOS/Linux/Windows)
- **Mobile Nodes**: Full participants on iOS and Android
- **Headless Nodes**: Bootstrap/seed nodes for network infrastructure
- **Web Clients**: Demo-only Flutter Web builds (no native FFI in browser)

---

## Security & Cryptographic Guarantees

### Post-Quantum Cryptographic Primitives
- **NIST FIPS 204 (ML-DSA-87/65)**: Module-Lattice-Based Digital Signature Algorithm with 192-bit (user) and 128-bit (site) quantum security levels
- **NIST FIPS 203 (ML-KEM-768)**: Module-Lattice-Based Key Encapsulation Mechanism with 192-bit classical security
- **ChaCha20-Poly1305**: Authenticated encryption with associated data (AEAD) for session encryption
- **BLAKE3**: Cryptographic hash function for content addressing and integrity verification
- **Keyring Integration**: Platform keychain storage (macOS Keychain, Windows Credential Manager, Linux Secret Service)

### Threat Model & Mitigations
- **Man-in-the-Middle**: Prevented by ML-DSA signature verification and ML-KEM authenticated key exchange
- **Quantum Computing**: Post-quantum algorithms resist Shor's and Grover's algorithms
- **Replay Attacks**: Nonce-based message authentication and temporal ordering
- **Sybil Attacks**: Proof-of-work on identity creation with rate limiting
- **Eclipse Attacks**: Multiple bootstrap sources with FOAF-based peer discovery
- **Network Partitioning**: CRDT eventual consistency ensures state convergence without coordination

### Decentralization Properties
- **No DNS Dependency**: Four-word cryptographic identities replace hierarchical naming
- **No PKI/Certificate Authorities**: Self-sovereign identity with cryptographic verification
- **No Blockchain Consensus**: CRDT conflict-free convergence without global coordination
- **No Central Servers**: Peer-to-peer gossip overlay with distributed state replication
- **Partition Tolerance**: CAP theorem AP system (availability + partition tolerance over consistency)

---

## License

**AGPL-3.0** for open collaboration. Commercial licensing available via [Saorsa Labs](mailto:saorsalabs@gmail.com).

---

## Contributing

1. **Code Style**: Follow existing patterns and conventions
2. **Commit Format**: Conventional commits (`feat:`, `fix:`, `docs:`)
3. **Quality Gates**: All code must pass Flutter + Rust linting
4. **Testing**: Include tests for new functionality

### Development Standards
- **No Panics**: Rust code forbids `unwrap`/`expect`/`panic!` in production (enforced by clippy)
- **Type Safety**: Full Dart null safety with strict analysis
- **Test Coverage**: 37+ integration tests covering resilience features
- **Security First**: Post-quantum cryptography and secure defaults
- **Partition Tolerance**: All features must operate correctly during network partitions

---

## Research & Standards Compliance

### Cryptographic Standards
- **[NIST FIPS 204](https://csrc.nist.gov/pubs/fips/204/final)**: Module-Lattice-Based Digital Signature Standard (ML-DSA)
- **[NIST FIPS 203](https://csrc.nist.gov/pubs/fips/203/final)**: Module-Lattice-Based Key-Encapsulation Mechanism Standard (ML-KEM)
- **[RFC 8439](https://www.rfc-editor.org/rfc/rfc8439)**: ChaCha20 and Poly1305 for IETF Protocols

### Distributed Systems Theory
- **CAP Theorem**: Prioritizes availability and partition tolerance (AP system)
- **CRDT Research**: Operation-based CRDTs with causal consistency (Shapiro et al.)
- **Gossip Protocols**: HyParView (Leitao et al.), SWIM (Das et al.), Plumtree (Leitao et al.)

### Network Resilience Testing
- **Partition Tolerance**: Verified through integration tests with simulated network failures
- **Exponential Backoff**: Jittered retry strategies prevent cascading failures
- **Resource Limits**: Configured connection limits (50 peers), memory caps (2GB), timeouts (30s); enforcement is in progress

---

Communitas represents a new class of partition-tolerant P2P systems combining post-quantum cryptography, CRDT-based eventual consistency, and catastrophic failure resistance. The architecture prioritizes operational continuity during network degradation while maintaining cryptographic security guarantees.

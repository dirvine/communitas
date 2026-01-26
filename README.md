# Communitas — The Unstoppable Collaboration Platform

[![Release](https://img.shields.io/github/v/release/maidsafe/communitas)](https://github.com/maidsafe/communitas/releases/latest)
[![Build](https://img.shields.io/github/actions/workflow/status/maidsafe/communitas/ci.yml)](https://github.com/maidsafe/communitas/actions)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)

> **A partition-tolerant, post-quantum secure, peer-to-peer collaboration network that works when the internet doesn't.**

## Download

**[Download Communitas v0.8.0 for macOS →](https://github.com/maidsafe/communitas/releases/latest)**

- **macOS (Universal)**: Supports Intel (x86_64) and Apple Silicon (M1/M2/M3/M4)
- Signed and notarized by Apple
- Auto-updates enabled

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
- Rust 1.85+
- `dx` CLI 0.7.3 (`scripts/install_dx.sh` installs the pinned release)
- Node.js 18+ (Tailwind/Vite asset bundling)

**Windows Additional Requirements:**
- Visual Studio 2022 Build Tools (C++ workload)
- Edge WebView2 runtime (Tauri uses system WebView; installer enforces `minimumWebview2Version`)
- See [Windows Build Guide](docs/development/windows-build.md) for detailed setup

**Linux:**
- GTK3/WebKitGTK runtime (Tauri WebView dependency)
- Build essentials, CMake, and platform libraries
- Refer to [docs/architecture/dioxus_milestone1_nav_auth.md](docs/architecture/dioxus_milestone1_nav_auth.md) for current dependency matrix

### Development Setup
```bash
git clone https://github.com/dirvine/communitas.git
cd communitas

scripts/install_dx.sh       # installs dx 0.7.3

# Dioxus app development
cd communitas-dioxus
dx serve --platform desktop --hotpatch
```

### Testing
```bash
# UI smoke + lint
cd communitas-dioxus
dx check --platform desktop

# Rust linting (strict policy)
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

# Rust unit tests
cargo test
```

To simulate authentication failures during QA, set `COMMUNITAS_UI_FORCE_AUTH_ERROR=1` before running the UI (`COMMUNITAS_UI_FORCE_AUTH_ERROR=1 dx serve --platform desktop`). This flag exercises the error-handling paths wired into `communitas-ui-service`.


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
- **QUIC Transport**: saorsa-gossip-transport (UdpTransportAdapter on ant-quic v0.18)
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
- **[Architecture Overview](docs/architecture/README.md)**: System architecture (Dioxus + Rust core + gossip)
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
- **[communitas-dioxus/](communitas-dioxus/)**: Cross-platform Dioxus + Tauri application (desktop-first, experimental mobile runners)
- **[communitas-headless/](communitas-headless/)**: Headless daemon for system services ([README](communitas-headless/README.md))
- **[communitas-mcp/](communitas-mcp/)**: MCP server for AI agent control (stdio + HTTPS with ML-DSA-65)

Dioxus is the supported GUI; MCP is the integration surface for other local apps and automations. Legacy thin-client assets remain in the archive solely for historical reference.

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
# UI Development
cd communitas-dioxus
dx serve --platform desktop --hotpatch

# Production Bundle
dx bundle --platform desktop
# Experimental mobile bundles (Android/iOS)
dx bundle --platform android

# Quality Checks
dx check --platform desktop && cargo clippy --all-features

# Run as system service (see communitas-headless/README.md)
communitas-headless --config /etc/communitas/headless.toml
```

---

## Deployment Options

Communitas supports multiple deployment scenarios for different use cases:

### Dioxus Application (End Users)
Full-featured cross-platform application (desktop GA, experimental Android/iOS runners).
```bash
cd communitas-dioxus
dx bundle --platform desktop
# optional mobile targets (stability varies)
dx bundle --platform android
dx bundle --platform ios
```
See [communitas-dioxus/](communitas-dioxus/) for details.

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

### MCP Apps (Interactive UIs in AI Conversations)
Communitas implements the [MCP Apps extension](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/) (SEP-1865), enabling rich interactive UIs directly within AI conversations in Claude Desktop, ChatGPT, VS Code, and other MCP hosts.

| Widget | Description | Features |
|--------|-------------|----------|
| **Contacts** | Interactive contact list | Search, favorites, presence indicators |
| **Messages** | Thread navigation | Composition, reactions, read status |
| **Kanban** | Project boards | Drag-drop cards, tags, due dates |
| **Drive** | File browser | Upload, preview, quota meter |
| **Canvas** | Whiteboard viewer | Layer toggles, history, remote cursors |

**The MCP server IS the app.** Dioxus native and MCP Apps are both presentation layers over the same shared `communitas-ui-service` (ADR-019), ensuring identical behavior across AI agents and human users.

See [ADR-022: MCP Apps Integration](docs/adr/ADR-022-mcp-apps-integration.md) for architecture details.

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
- **Desktop Nodes**: Full participants with the Dioxus/Tauri UI (macOS/Linux/Windows)
- **Mobile Nodes**: Experimental Dioxus builds on Android/iOS (stability pending upstream Tauri updates)
- **Headless Nodes**: Bootstrap/seed nodes for network infrastructure
- **Web Clients**: Demo-only SSR builds (no native MCP exposure in browser)

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
3. **Quality Gates**: All code must pass Dioxus (`dx check`) + Rust linting
4. **Testing**: Include tests for new functionality

### Development Standards
- **No Panics**: Rust code forbids `unwrap`/`expect`/`panic!` in production (enforced by clippy)
- **Type Safety**: Rust-first surfaces with strict clippy/fmt enforcement
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

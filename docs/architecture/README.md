# Communitas Architecture

Comprehensive technical architecture documentation for the Communitas local-first collaboration platform.

## Overview

Communitas is a **local-first, post-quantum secure collaboration platform** that combines messaging, file sharing, voice/video calling, and web publishing into a single decentralized application. The entire stack is Rust—Dioxus/Tauri renders the UI while `communitas-core` handles all business logic—providing offline-capable functionality with real-time synchronization when connected.

### Core Principles

1. **Local-First**: All data stored locally, operations work offline, sync when connected
2. **Post-Quantum Secure**: ML-DSA signatures and ML-KEM key exchange
3. **Human-Verifiable**: Four-word addressing system for all entities
4. **Decentralized**: P2P gossip networking with no central servers
5. **Entity-Centric**: Everything is an entity (users, groups, channels, projects)

### Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                FRONTEND (Dioxus + Tauri)                    │
│  - Cross-platform UI (macOS, Windows, Linux; experimental mobile) │
│  - Dioxus signals/hooks backed by `communitas-ui-service`   │
│  - Thin GUI over shared Rust services                       │
└─────────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│              CORE LIBRARY (Rust)                            │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Identity         │  │ Authentication   │               │
│  │ - Four-words     │  │ - Passkeys       │               │
│  │ - ML-DSA sigs    │  │ - Sessions       │               │
│  └──────────────────┘  └──────────────────┘               │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Storage          │  │ CRDT Sync        │               │
│  │ - Virtual disks  │  │ - Yrs documents  │               │
│  │ - SQL cache      │  │ - State vectors  │               │
│  └──────────────────┘  └──────────────────┘               │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Messaging        │  │ Groups           │               │
│  │ - Channels       │  │ - Membership     │               │
│  │ - Threads        │  │ - Permissions    │               │
│  └──────────────────┘  └──────────────────┘               │
└─────────────────────────────────────────────────────────────┘
                           ↓ Gossip API
┌─────────────────────────────────────────────────────────────┐
│           P2P NETWORKING (saorsa-gossip)                    │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Membership       │  │ PubSub           │               │
│  │ - HyParView      │  │ - Plumtree       │               │
│  │ - SWIM beacons   │  │ - Topics         │               │
│  └──────────────────┘  └──────────────────┘               │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Presence         │  │ Rendezvous       │               │
│  │ - Online status  │  │ - 65k shards     │               │
│  │ - Heartbeats     │  │ - DHT-free       │               │
│  └──────────────────┘  └──────────────────┘               │
└─────────────────────────────────────────────────────────────┘
                           ↓ QUIC Transport
┌─────────────────────────────────────────────────────────────┐
│   NETWORK LAYER (saorsa-gossip-transport / ant-quic)         │
│  - QUIC connections with multiplexing                      │
│  - NAT traversal and connection migration                  │
│  - IPv4-first with Happy Eyeballs fallback                 │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│           CRYPTOGRAPHY (saorsa-pqc)                         │
│  - ML-KEM-768 (post-quantum key exchange)                  │
│  - ML-DSA-65 (post-quantum signatures)                     │
│  - ChaCha20-Poly1305 (symmetric encryption)                │
│  - BLAKE3 (content addressing)                             │
└─────────────────────────────────────────────────────────────┘
```

## Architecture Documents

This architecture documentation is organized into the following sections:

### Core Components
- **Core library modules**: identity, storage, messaging, permissions, networking
- **Dioxus UI**: thin presentation layer over shared Rust services
- **Platform integrations**: keyring, notifications, OS services

### Collaboration Surfaces (Milestone 3)
- **[Kanban](dioxus_milestone3_advanced_surfaces.md#41-kanbanservice-implementation-phase-10)**: Boards, columns, cards with drag-drop and CRDT sync
- **[Drive](dioxus_milestone3_advanced_surfaces.md#43-driveservice-implementation-phase-12)**: Virtual disk browser with upload/download and quota
- **[Call](dioxus_milestone3_advanced_surfaces.md#45-callservice-implementation-phase-14)**: WebRTC audio calls with device selection
- **[Canvas](dioxus_milestone3_advanced_surfaces.md#47-canvasservice-implementation-phase-15)**: Collaborative whiteboard with layers and history

### Data & Storage
- **[CRDT System](crdt-system.md)** - Conflict-free replicated data types
  - Document taxonomy and organization
  - Virtual disk architecture (Private, Public, Shared)
  - SQL cache and materialization
  - Sync protocol and state vectors
  - Offline-first operation

### Networking
- **[Gossip Protocol](gossip-protocol.md)** - P2P communication layer
  - Saorsa Gossip architecture
  - Membership management (HyParView)
  - Message dissemination (Plumtree)
  - Peer discovery (Rendezvous)
  - Failure detection (SWIM)
  - Bootstrap nodes and network formation

- **[Networking](networking.md)** - Network protocols and connectivity
  - QUIC transport layer
  - Connection management
  - NAT traversal strategies
  - IPv4/IPv6 Happy Eyeballs
  - Network status and resilience

### Storage & Content
- **[Storage](storage.md)** - Data persistence and retrieval
  - Filesystem-backed CRDT persistence (libSQL materialization planned)
  - Virtual disk implementation
  - Content addressing with BLAKE3
  - Access control policies
  - Replication strategies

### Security
- **[Security](security.md)** - Cryptography and security model
  - Post-quantum cryptography (ML-DSA, ML-KEM)
  - Connection word security (IP:port encoding)
  - Authentication methods (passwords, passkeys)
  - Session management
  - Encryption policies
  - Threat model and mitigations

### Architecture Decision Records (ADRs)

For detailed rationale behind architectural decisions, see our ADRs:

| ADR | Title | Description |
|-----|-------|-------------|
| [ADR-001](../adr/ADR-001-four-word-identity-system.md) | Four-Word Identity System | Superseded; four-word networking now used only for connection words |
| [ADR-002](../adr/ADR-002-local-first-architecture.md) | Local-First Architecture | Offline-capable with sync |
| [ADR-003](../adr/ADR-003-yrs-crdt-synchronization.md) | Yrs CRDT Synchronization | Conflict-free collaborative data |
| [ADR-004](../adr/ADR-004-entity-hierarchy-model.md) | Entity Hierarchy Model | Unified entity taxonomy |
| [ADR-005](../adr/ADR-005-virtual-disk-architecture.md) | Virtual Disk Architecture | Per-entity file storage |
| [ADR-006](../adr/ADR-006-post-quantum-cryptography.md) | Post-Quantum Cryptography | ML-DSA/ML-KEM pure PQC |
| [ADR-007](../adr/ADR-007-gossip-overlay-networking.md) | Gossip Overlay Networking | P2P via HyParView/Plumtree/SWIM |
| [ADR-008](../adr/ADR-008-event-driven-tombstone-pruning.md) | Event-Driven Tombstone Pruning | CRDT cleanup on sync |
| [ADR-009](../adr/ADR-009-modular-crate-architecture.md) | Modular Crate Architecture | Cargo workspace structure |
| [ADR-010](../adr/ADR-010-cross-organization-invites.md) | Cross-Organization Invites | Identity-based invite system |
| [ADR-011](../adr/ADR-011-encrypted-vault-storage.md) | Encrypted Vault Storage | Multi-layer local encryption |
| [ADR-018](../adr/ADR-018-mcp-external-integration.md) | MCP External Integration | AI agent automation interface |
| [ADR-019](../adr/ADR-019-shared-rust-ui-service.md) | Shared Rust UI Service | Unified services for Dioxus + MCP |
| [ADR-020](../adr/ADR-020-dioxus-desktop-adoption.md) | Dioxus Desktop Adoption | All-Rust UI stack decision |
| [ADR-021](../adr/ADR-021-canvas-integration-strategy.md) | Canvas Integration Strategy | Collaborative whiteboard architecture |

See the [ADR Index](../adr/README.md) for more details and templates for new ADRs.

## Key Concepts

### Connection Words (Four-Word Networking)

Communitas uses four-word networking **only** to encode connection endpoints (IP:port) for easy sharing:
- **Human-Readable**: Easy to remember and share
- **Verifiable**: Dictionary validation prevents typos and phishing
- **Ephemeral**: Represents a connection address, not a user identity

Identity is the **public key** (pubkey_hex). Connection words are a separate, shareable network address.
See [ADR-001](../adr/ADR-001-four-word-identity-system.md) for details.

### Entities

Communitas is built around the concept of **entities**:

- **👤 Users**: Personal identities (public-key based)
- **👥 Groups**: Collaborative spaces with shared resources
- **🏢 Organizations**: Multi-channel communication hubs
- **📁 Projects**: Structured workspaces with task management
- **📢 Channels**: Topic-focused discussion spaces

Each entity has:
- Unique identity (public key or entity ID)
- Three virtual disks (Private, Public, Shared)
- CRDT documents for real-time collaboration
- Optional website root for DNS-free publishing

### Virtual Disks

Each entity has three virtual disks for file storage:

1. **Private Disk**: Encrypted, local-only storage
   - Personal files and notes
   - Credentials (auto-encrypted)
   - Draft documents

2. **Public Disk**: Content-addressed, distributed storage
   - Public documents
   - Website content
   - Shared files

3. **Shared Disk**: Group-accessible with shared encryption
   - Team documents
   - Collaborative files
   - Project resources

See [Storage Architecture](storage.md) for implementation details.

### CRDT Documents

Real-time collaboration powered by Yrs CRDTs:
- **Modular**: Each concern (members, chat, kanban) is a separate document
- **Bounded**: Size limits trigger SQL materialization
- **Offline-First**: All operations work without network
- **Eventually Consistent**: Automatic conflict resolution
- **Event-Driven**: Tombstone pruning on materialization

See [CRDT System](crdt-system.md) for complete architecture.

### Gossip Networking

Decentralized P2P communication with no central servers:
- **HyParView**: Maintains 8-12 active peers, 64-128 passive peers
- **Plumtree**: Efficient message dissemination with tree-based broadcast
- **SWIM**: Fast failure detection (<5s)
- **Rendezvous**: DHT-free peer discovery with 65k shards
- **Bootstrap Nodes**: Network entry points for new peers

See [Gossip Protocol](gossip-protocol.md) for details.

## Technology Stack

### Frontend
- **Framework**: Dioxus + Tauri (Rust)
- **State Management**: Dioxus signals/hooks + `communitas-ui-service`
- **Routing**: `dioxus-router`
- **Core Access**: Shared Rust services (`communitas-ui-service` + `communitas-core`)
- **Testing**: `dx check`, Dioxus SSR/component tests

### Core
- **Language**: Rust 2024 edition
- **CRDT**: Yrs (Yjs Rust port)
- **Networking**: saorsa-gossip (UdpTransportAdapter on ant-quic)
- **Crypto**: saorsa-pqc (post-quantum)

### Infrastructure
- **Service Management**: systemd, launchd
- **Monitoring**: Prometheus, Grafana
- **CI/CD**: GitHub Actions
- **Package Manager**: Cargo + `dx` CLI utilities

## Development Environment

### Prerequisites
- Rust 1.85+
- `dx` CLI 0.7.x
- Platform-specific dependencies for Tauri desktop/mobile

### Quick Start
See the main [README.md](../../README.md) for setup instructions.

## Deployment Options

### Dioxus Application
Native application for Windows, macOS, Linux (GA) with experimental Android/iOS runners:
- Build via `dx bundle`
- Native binaries packaged per platform
- Web build supported only for demo mode (SSR)

### Headless Daemon
Server deployment for bots and background services:
- systemd/launchd service integration
- Metrics/health endpoint for monitoring

See [communitas-headless/README.md](../../communitas-headless/README.md)

### Container Deployment
Docker and Kubernetes for cloud deployment:
- Multi-architecture support (amd64, arm64)
- Horizontal Pod Autoscaling (HPA)
- Prometheus metrics and health checks

See the [communitas-headless](../../communitas-headless/README.md) crate for deployment details.

### Seed/Bootstrap Nodes
Network infrastructure for peer discovery:
- Run `communitas-headless` with dedicated instance/config for introducer roles
- Gossip-based bootstrap seeding
- High availability deployment

See [communitas-headless/README.md](../../communitas-headless/README.md)

## Performance Characteristics

### Message Latency
- **Local Operations**: <10ms (offline-first)
- **LAN Peers**: <50ms (direct QUIC)
- **WAN Peers**: <500ms (geographic routing)
- **Gossip Propagation**: <2s (99th percentile)

### Storage Performance
- **Virtual Disk Write**: <100ms (local)
- **CRDT Update**: <50ms (Yrs state vector)
- **SQL Materialization**: <1s (10k messages)
- **Sync Bandwidth**: <10KB/s per peer (steady state)

### Network Scalability
- **Peers per Node**: 8-12 active, 64-128 passive
- **Messages per Second**: 1000+ (per node)
- **Network Size**: Tested to 10,000 nodes
- **Partition Recovery**: <30s (via periodic shuffle)

## Security Considerations

### Post-Quantum Readiness
- **ML-DSA-65**: Quantum-resistant signatures
- **ML-KEM-768**: Quantum-resistant key exchange
- **Migration Path**: Hybrid classical+PQ for transition period

### Threat Model
- **Network Adversary**: Cannot decrypt messages or forge signatures
- **Compromised Peer**: Isolated via reputation system
- **Malicious Bootstrap**: Redundant bootstrap nodes prevent poisoning
- **Four-Word Collision**: Dictionary validation prevents spoofing

See [Security Architecture](security.md) for complete analysis.

## Testing Strategy

### Unit Tests
- Frontend: Dioxus component/unit tests
- Backend: Cargo tests for Rust modules
- Coverage target: >85%

### Integration Tests
- Multi-node P2P testing
- CRDT synchronization tests
- Storage policy verification

### AI Agent Testing
- communitas-mcp HTTPS interface (ML-DSA-65 raw public keys)
- Chrome DevTools MCP integration
- End-to-end scenarios with AI agents

See the main [README.md](../../README.md) for testing instructions.

## Future Roadmap

### Near-Term (v1.0)
- Voice/video calling (WebRTC)
- Mobile apps (iOS, Android)
- Plugin system for extensions
- Advanced search and filtering

### Mid-Term (v2.0)
- Multi-device sync (same identity, multiple devices)
- Threshold signatures for groups
- Cross-platform clipboard sync
- AI-powered features (summarization, translation)

### Long-Term (v3.0)
- Mesh networking for offline operation
- Federated identity bridges (email, phone)
- Enterprise features (LDAP, SAML)
- IoT device integration

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for how to contribute to Communitas.

## Resources

### Documentation
- [API Reference](../api/) - Complete API documentation
- [Architecture Decision Records](../adr/README.md) - Design decisions and rationale
- [Main README](../../README.md) - Getting started and development workflow

### External Resources
- [Dioxus Documentation](https://dioxuslabs.com/docs/)
- [Tauri 2 Guide](https://tauri.app/v2/guides/)
- [Yrs CRDT Documentation](https://docs.rs/yrs/)
- [ant-quic Transport](https://github.com/maidsafe/ant-quic)

### Community
- [GitHub Repository](https://github.com/dirvine/communitas)
- [GitHub Discussions](https://github.com/dirvine/communitas/discussions)
- [Issue Tracker](https://github.com/dirvine/communitas/issues)
- [Website](https://communitas.life)

---

**Communitas**: Local-first collaboration with post-quantum security. 🚀🔒

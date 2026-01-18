# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Communitas project.

## What are ADRs?

ADRs document significant architectural decisions made in the project. Each record captures the context, decision, and consequences to help future maintainers understand why things are the way they are.

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-001](ADR-001-four-word-identity-system.md) | Four-Word Identity System | Superseded | 2025-01-15 |
| [ADR-002](ADR-002-local-first-architecture.md) | Local-First Architecture | Accepted | 2025-12-24 |
| [ADR-003](ADR-003-yrs-crdt-synchronization.md) | Yrs CRDT Synchronization | Accepted | 2025-12-24 |
| [ADR-004](ADR-004-entity-hierarchy-model.md) | Entity Hierarchy Model | Accepted | 2025-12-24 |
| [ADR-005](ADR-005-virtual-disk-architecture.md) | Virtual Disk Architecture | Accepted | 2025-12-24 |
| [ADR-006](ADR-006-post-quantum-cryptography.md) | Post-Quantum Cryptography | Accepted | 2025-12-24 |
| [ADR-007](ADR-007-gossip-overlay-networking.md) | Gossip Overlay Networking | Accepted | 2025-12-24 |
| [ADR-008](ADR-008-event-driven-tombstone-pruning.md) | Event-Driven Tombstone Pruning | Accepted | 2025-12-24 |
| [ADR-009](ADR-009-modular-crate-architecture.md) | Modular Crate Architecture | Accepted | 2025-12-24 |
| [ADR-010](ADR-010-cross-organization-invites.md) | Cross-Organization Invites | Accepted | 2025-12-24 |
| [ADR-011](ADR-011-encrypted-vault-storage.md) | Encrypted Vault Storage | Accepted | 2025-12-24 |
| [ADR-012](ADR-012-identity-packet-system.md) | Identity Packet System | Superseded | 2025-01-15 |
| [ADR-013](ADR-013-connection-system.md) | Connection System | Accepted | 2025-01-10 |
| [ADR-014](ADR-014-peer-discovery-presence.md) | Peer Discovery & Presence | Accepted | 2025-01-10 |
| [ADR-015](ADR-015-bootstrap-process.md) | Bootstrap Process | Accepted | 2025-01-10 |
| [ADR-016](ADR-016-identity-recovery-system.md) | Identity Recovery System | Proposed | 2025-01-15 |
| [ADR-017](ADR-017-legacy-thin-client-ffi-integration.md) | Legacy Thin-Client FFI Integration (Archived) | Superseded | 2026-01-18 |
| [ADR-018](ADR-018-mcp-external-integration.md) | MCP External Integration Architecture | Accepted | 2025-01-15 |
| [ADR-019](ADR-019-shared-rust-ui-service.md) | Shared Rust UI Service Layer | Accepted | 2026-01-18 |
| [ADR-020](ADR-020-dioxus-desktop-adoption.md) | Dioxus Desktop Adoption | Accepted | 2026-01-18 |

## Key ADR Relationships

### Identity & Security
- **ADR-001** (Four-Word Identity) → superseded; four-word networking now used only for connection words
- **ADR-006** (Post-Quantum Cryptography) → ML-DSA-65, ML-KEM-768 algorithms
- **ADR-011** (Encrypted Vault Storage) → local key storage with PBKDF2
- **ADR-012** (Identity Packet System) → identity packet structure
- **ADR-016** (Identity Recovery) → BIP39 mnemonic recovery, social recovery

### Networking & P2P
- **ADR-007** (Gossip Overlay) → P2P networking layer
- **ADR-013** (Connection System) → peer connection management
- **ADR-014** (Peer Discovery) → finding and tracking peers
- **ADR-015** (Bootstrap Process) → initial network join

### Data & Storage
- **ADR-002** (Local-First) → offline-first architecture
- **ADR-003** (Yrs CRDT) → conflict-free data synchronization
- **ADR-004** (Entity Hierarchy) → organizational data model
- **ADR-005** (Virtual Disk) → distributed file storage

### Integration & Architecture
- **ADR-009** (Modular Crate Architecture) → workspace structure, crate boundaries
- **ADR-017** (Legacy Thin-Client FFI Integration, archived) → historical context for the retired FRB bindings
- **ADR-018** (MCP External Integration) → AI agent access, saorsa-canvas integration

## ADR Template

New ADRs should follow this structure:

```markdown
# ADR-N: Title

## Status
Proposed | Accepted | Deprecated | Superseded

## Context
Why this decision was necessary.

## Decision
What was chosen and why.

## Consequences
Benefits and trade-offs.

## Alternatives Considered
Other options and why rejected.

## References
Relevant commits, RFCs, code paths.
```

## Related Documentation

- [Architecture Overview](../architecture/README.md)
- [CRDT System](../architecture/crdt-system.md)
- [Offline Handling](../architecture/offline-handling.md)
- [API Documentation](../api/README.md)

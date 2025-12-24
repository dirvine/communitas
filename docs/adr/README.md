# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Communitas project.

## What are ADRs?

ADRs document significant architectural decisions made in the project. Each record captures the context, decision, and consequences to help future maintainers understand why things are the way they are.

## Index

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [ADR-001](ADR-001-four-word-identity-system.md) | Four-Word Identity System | Accepted | 2025-12-24 |
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
- [CRDT Architecture](../CRDT_ARCHITECTURE.md)
- [Mesh Capabilities](../MESH_CAPABILITIES.md)
- [API Documentation](../api/README.md)

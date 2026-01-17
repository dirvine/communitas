# ADR-004: Entity Hierarchy Model

## Status

Accepted (2025-12-24)

## Context

### The Problem

Collaboration platforms need to organize people and resources in ways that map to real-world structures:

- **Flat structures**: Too simple, don't scale for organizations
- **Deep hierarchies**: Complex, hard to navigate
- **Single entity type**: Forces awkward workarounds

Users need a model that supports:
- Personal spaces (individual work)
- Team collaboration (project groups)
- Multi-team organizations (companies, communities)
- Topic-focused discussions (channels)

### Requirements

- Support individual, team, and organizational use cases
- Clear ownership and permission inheritance
- Public-key identity for all entities (connection words are separate)
- Each entity gets its own storage (virtual disks)
- Flexible parent-child relationships

## Decision

Adopt a **unified entity model** where everything is an entity with a public-key identity, organized in a flexible hierarchy:

### Entity Types

| Type | Description | Parent | Use Case |
|------|-------------|--------|----------|
| **User** | Individual identity | None | Personal space |
| **Organization** | Multi-team container | None | Company, community |
| **Group** | Collaborative team | Organization (optional) | Team workspace |
| **Channel** | Topic discussion | Organization | Topic-focused chat |
| **Project** | Work management | Organization | Task tracking |

### Hierarchy Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Entity Hierarchy                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Organization: Acme Corp                                           │
│  ├── Channel: engineering-chat                                     │
│  ├── Channel: design-updates                                       │
│  ├── Group: backend-team                                           │
│  │   └── [Members can create sub-entities]                         │
│  └── Project: mobile-app                                           │
│      ├── Kanban board                                              │
│      └── Issue tracker                                             │
│                                                                     │
│  Standalone Group: friends-gaming                                  │
│  └── [No parent organization, independent]                         │
│                                                                     │
│  User: Alice                                                       │
│  └── [Personal space with virtual disks]                           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Entity Structure

```rust
pub struct Entity {
    pub id: String,                     // UUID for internal use
    pub name: String,                   // Display name
    pub entity_type: EntityType,        // Organization | Group | Channel | Project
    pub description: Option<String>,
    pub created_by: String,             // Creator identity (pubkey_hex)
    pub created_at: i64,
    pub members: Vec<String>,           // Member identities (pubkey_hex)
    pub parent_org_id: Option<String>,  // Links to parent organization
    pub connection_words: Option<String>, // Optional P2P connection words
    pub is_local_only: bool,            // True if not yet synced
}
```

### Entity Type Enum

```rust
pub enum EntityType {
    Organization,  // Top-level container
    Group,         // Team workspace
    Channel,       // Discussion space
    Project,       // Work management
}
```

### Per-Entity Resources

Each entity automatically gets:

| Resource | Description |
|----------|-------------|
| **Identity** | Public-key identity (pubkey_hex) |
| **Connection words (optional)** | Human-readable IP:port encoding |
| **CRDT documents** | Core (metadata/members), Chat, domain-specific |
| **Virtual disks** | Private, Public, Shared storage |
| **Website root** | Optional DNS-free publishing |

### Local-First Entity Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Entity Lifecycle                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. Create Local          2. Generate Keypair       3. Link & Sync  │
│  ┌─────────────────┐      ┌─────────────────┐      ┌─────────────┐ │
│  │ Entity::new_    │      │ generate_keys() │      │ link_to_    │ │
│  │ local(name,     │ ───► │                 │ ───► │ network()   │ │
│  │ type, desc)     │      │                 │      │             │ │
│  │                 │      │ "ocean-forest-  │      │ Gossip      │ │
│  │ is_local: true  │      │  moon-star"     │      │ announce    │ │
│  └─────────────────┘      └─────────────────┘      └─────────────┘ │
│                                                                     │
│  Entity works offline immediately, network identity added later     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Membership Model

Members are stored in the entity's CRDT core document:

```rust
MemberInfo {
    member_id: String,    // Four-word address
    role: String,         // "owner" | "admin" | "member"
    joined_at: i64,
    deleted: bool,        // Tombstone for pruning
}
```

**Cascade membership** for organizations:
- Adding member to org optionally adds to child entities
- Removing from org can cascade to all children
- Child entities can have additional members not in parent

## Consequences

### Benefits

- **Flexible structure**: Supports flat teams to complex organizations
- **Consistent model**: All entities work the same way
- **Clear ownership**: Parent-child relationships define scope
- **Four-word addressing**: Human-readable at every level
- **Per-entity storage**: Clean data boundaries
- **Local-first**: Entities work offline immediately

### Trade-offs

- **Hierarchy limits**: Only two levels (org → children)
- **No nested orgs**: Organizations cannot contain organizations
- **Member duplication**: Same member in org and child entities

### Permission Inheritance

| Entity | Permission Source |
|--------|-------------------|
| Organization | Own member list |
| Group | Own members + parent org (if linked) |
| Channel | Own members + parent org |
| Project | Own members + parent org |

## Alternatives Considered

1. **Flat model**: Only users and groups
   - Rejected: Doesn't scale for organizations

2. **Deep hierarchy**: Unlimited nesting
   - Rejected: Too complex, confusing navigation

3. **Type-specific systems**: Separate code for each entity type
   - Rejected: Code duplication, inconsistent behavior

4. **External linking**: Reference external entities
   - Rejected: Complicates offline operation

## References

- Implementation: `communitas-core/src/entity_service.rs`
- CRDT Documents: `communitas-core/src/crdt/documents.rs`
- Entity Types: `communitas-core/src/crdt/mod.rs`
- Related ADR: [ADR-001 Four-Word Identity](ADR-001-four-word-identity-system.md) (superseded)
- Related ADR: [ADR-005 Virtual Disk Architecture](ADR-005-virtual-disk-architecture.md)

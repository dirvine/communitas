# ADR-003: Yrs CRDT Synchronization

## Status

Accepted (2025-12-24)

## Context

### The Problem

Real-time collaboration in a local-first system requires handling concurrent edits from multiple users who may be offline. Traditional approaches face challenges:

- **Operational Transform (OT)**: Requires central server to order operations
- **Locking**: Prevents concurrent edits, poor UX
- **Last-write-wins**: Loses user data on conflicts
- **Manual conflict resolution**: Burden on users, poor UX

Communitas needs automatic conflict resolution that:
- Works without a central server
- Preserves all user intent (no data loss)
- Merges concurrent edits deterministically
- Supports offline-first operation

### Requirements

- Automatic conflict resolution without user intervention
- Deterministic merge (same inputs → same output)
- Support for rich data structures (maps, arrays, text)
- Efficient delta sync (not full-state transfer)
- Bounded document growth
- Works with local-first architecture

## Decision

Adopt **Yrs (Yjs Rust port)** as the CRDT implementation for all collaborative data structures.

### Why Yrs?

| Feature | Yrs | Automerge | Custom |
|---------|-----|-----------|--------|
| Language | Rust | Rust | Rust |
| Maturity | Production | Production | New |
| WASM support | Yes | Yes | Custom |
| Text CRDT | YText | RGA | Custom |
| Map/Array | YMap/YArray | Yes | Custom |
| Binary size | ~150KB | ~200KB | Variable |
| Community | Large (Yjs) | Growing | None |

### Document Architecture

Each entity has **modular CRDT documents** per concern:

```
Entity: group:ocean-forest-moon-star

Documents:
├── group:ocean-forest-moon-star:core    → Metadata, members
├── group:ocean-forest-moon-star:chat    → Messages, threads
├── project:gentle-wave-moon-fire:kanban → Kanban board
└── project:gentle-wave-moon-fire:issues → Issue tracker
```

### Key CRDT Types Used

| Yrs Type | Use Case | Conflict Strategy |
|----------|----------|-------------------|
| YMap | Entity metadata, members | Per-field LWW |
| YArray | Column order, card order | Position-aware merge |
| YText | Message content, descriptions | Character merge |

### Document Schemas

**Core Document** (all entities):
```rust
Doc {
    "metadata": YMap {
        "entity_id": String,
        "name": String,
        "pubkey_hex": String,           // Entity identity
        "connection_words": String,     // Optional - IP:port encoding for P2P
        "created_at": i64,
        "website_root": String,         // Optional BLAKE3 hash
    },
    "members": YMap<member_id, MemberData>,
    "active_members": YMap<member_id, bool>,
}
```

**Chat Document**:
```rust
Doc {
    "messages": YMap<message_id, MessageData>,
    "threads": YMap<parent_id, ThreadData>,
}

MessageData = YMap {
    "author_id": String,
    "content": YText,           // Collaborative editing
    "created_at": i64,
    "updated_at": i64,          // LWW for edits
    "deleted": bool,
    "ack_vector": YMap<member_id, i64>,  // For tombstone pruning
}
```

**Kanban Document** (projects):
```rust
Doc {
    "columns": YArray<ColumnData>,
    "cards": YMap<card_id, CardData>,
}

ColumnData = YMap {
    "id": String,
    "title": String,
    "card_order": YArray<card_id>,  // Preserves drag-drop order
}
```

### State Vector Sync Protocol

Yrs uses state vectors for efficient delta sync:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    State Vector Sync                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Peer A                              Peer B                         │
│  ┌────────────────┐                  ┌────────────────┐            │
│  │ State Vector:  │                  │ State Vector:  │            │
│  │ {A: 5, B: 3}   │                  │ {A: 3, B: 5}   │            │
│  └────────────────┘                  └────────────────┘            │
│         │                                   │                       │
│         │    1. Exchange state vectors      │                       │
│         │◄──────────────────────────────────│                       │
│         │                                   │                       │
│         │    2. Compute missing updates     │                       │
│         │       A needs: B[4..5]            │                       │
│         │       B needs: A[4..5]            │                       │
│         │                                   │                       │
│         │    3. Exchange only deltas        │                       │
│         │──────────────────────────────────►│                       │
│         │◄──────────────────────────────────│                       │
│         │                                   │                       │
│  ┌────────────────┐                  ┌────────────────┐            │
│  │ State Vector:  │                  │ State Vector:  │            │
│  │ {A: 5, B: 5}   │  ← Converged →   │ {A: 5, B: 5}   │            │
│  └────────────────┘                  └────────────────┘            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Conflict Resolution Semantics

| Scenario | Resolution | Example |
|----------|------------|---------|
| Different fields | Both preserved | A edits title, B edits description |
| Same field | Last-Write-Wins | A and B both edit title → later timestamp wins |
| YText concurrent edit | Character merge | A inserts at pos 5, B at pos 10 → both visible |
| Array reorder | Position-aware | Drag operations merge without duplication |
| Delete + Edit | Delete wins | A deletes message, B edits → message deleted |

### Size Thresholds and SQL Fallback

| Document | Threshold | Fallback Action |
|----------|-----------|-----------------|
| core | 1MB | Unlikely to exceed |
| chat | 10MB | Materialize messages >90 days to SQL |
| kanban | 5MB | Materialize archived cards to SQL |
| issues | 10MB | Materialize closed issues to SQL |

**Hybrid mode** for large documents:
```rust
pub async fn get_messages(entity_id: &str, limit: usize) -> Result<Vec<Message>> {
    match check_fallback_mode(entity_id, "chat").await? {
        FallbackMode::Crdt => load_from_crdt(entity_id, limit).await,
        FallbackMode::Hybrid => {
            let recent = load_from_crdt(entity_id, limit / 2).await?;
            let archived = load_from_sql(entity_id, limit / 2).await?;
            Ok(merge_chronological(recent, archived))
        },
        FallbackMode::Sql => load_from_sql(entity_id, limit).await,
    }
}
```

## Consequences

### Benefits

- **Automatic conflict resolution**: No user intervention needed
- **Deterministic merge**: Same inputs always produce same output
- **Rich data structures**: Maps, arrays, text with proper semantics
- **Efficient sync**: State vectors enable delta-only transfer
- **Offline-first**: All operations work without network
- **Mature implementation**: Yjs ecosystem is battle-tested

### Trade-offs

- **Memory overhead**: CRDT metadata increases document size
- **Tombstone accumulation**: Deleted items remain until pruned
- **Complexity**: CRDT semantics require careful schema design
- **No global ordering**: Concurrent operations merge, not order

### Performance Characteristics

| Operation | Network | CPU | Memory |
|-----------|---------|-----|--------|
| Add member | ~1KB | <1ms | ~200B |
| Send message | ~2KB | <5ms | ~500B |
| Edit message | ~500B | <1ms | ~100B |
| Full sync | Variable | <100ms | State size |

## Alternatives Considered

1. **Automerge**: Alternative Rust CRDT library
   - Rejected: Yrs has better YText implementation, larger ecosystem

2. **Custom CRDT**: Build from scratch
   - Rejected: Reinventing the wheel, years of work

3. **Operational Transform**: Google Docs-style
   - Rejected: Requires central server, not local-first compatible

4. **Event sourcing**: Append-only log
   - Rejected: Doesn't provide conflict resolution

5. **Manual conflict resolution**: Git-style merges
   - Rejected: Poor UX, requires user intervention

## References

- CRDT Architecture: `docs/architecture/crdt-system.md`
- Implementation: `communitas-core/src/crdt/`
- Yrs Documentation: https://docs.rs/yrs/
- Yjs Project: https://yjs.dev/
- CRDT Theory: https://crdt.tech/
- Related ADR: [ADR-002 Local-First Architecture](ADR-002-local-first-architecture.md)
- Related ADR: [ADR-008 Event-Driven Tombstone Pruning](ADR-008-event-driven-tombstone-pruning.md)

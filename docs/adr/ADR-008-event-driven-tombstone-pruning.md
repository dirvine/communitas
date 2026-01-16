# ADR-008: Event-Driven Tombstone Pruning

## Status

Accepted (2025-12-24)

## Context

### The Problem

CRDTs require tombstones to represent deleted items for conflict resolution. Without pruning, tombstones accumulate indefinitely:

- **Unbounded growth**: Document size grows with every delete
- **Memory pressure**: Large documents consume excessive memory
- **Sync overhead**: Full syncs transfer all tombstones

Traditional approaches have issues:

| Approach | Problem |
|----------|---------|
| Time-based | Offline peers miss deletions, conflicts |
| Periodic GC | Complexity, timing edge cases |
| Never prune | Unbounded growth |

### Requirements

- Prune tombstones when safe (all members have seen them)
- No periodic background tasks
- Handle offline members correctly
- Work in local-first environment
- Preserve deletion semantics

## Decision

Implement **event-driven tombstone pruning** where tombstones are pruned immediately when all active members have acknowledged them:

### Core Principle

> Tombstones are pruned the instant all active members have synced the deletion.

No timers, no periodic tasks, no garbage collection schedules.

### Acknowledgment Vector

Each tombstone tracks which members have seen it:

```rust
struct TombstoneData {
    deleted: bool,              // true = tombstone
    deleted_at: i64,            // when deletion occurred
    deleted_by: String,         // who deleted
    ack_vector: HashMap<MemberId, i64>,  // who has acknowledged
}
```

### Pruning Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Tombstone Pruning Flow                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Step 1: Delete Operation                                          │
│  ───────────────────────────                                        │
│  - Set deleted = true                                              │
│  - Set deleted_at = now                                            │
│  - Initialize empty ack_vector                                     │
│  - Remove from active_members (for member deletion)                │
│  - Broadcast update via gossip                                     │
│                                                                     │
│                              │                                      │
│                              ▼                                      │
│                                                                     │
│  Step 2: Peer Receives Update                                      │
│  ────────────────────────────                                       │
│  - Apply CRDT update (Yrs merge)                                   │
│  - For each tombstone in update:                                   │
│      • Add own member_id to ack_vector with timestamp              │
│  - Save document                                                   │
│  - Trigger pruning check                                           │
│                                                                     │
│                              │                                      │
│                              ▼                                      │
│                                                                     │
│  Step 3: Immediate Pruning Check                                   │
│  ───────────────────────────────                                    │
│  - Get list of active members from core document                   │
│  - For each tombstone:                                             │
│      • Check if ALL active members in ack_vector                   │
│      • If YES: map.remove(txn, tombstone_id)                       │
│  - Save document if any pruned                                     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Implementation

```rust
pub async fn on_peer_sync(
    entity_id: &str,
    peer_id: &str,
    update: &[u8],
    concern: &str,
) -> Result<()> {
    let doc = crdt_manager.load_document(&doc_id).await?;

    // 1. Apply incoming CRDT update
    {
        let mut txn = doc.transact_mut();
        let decoded = yrs::Update::decode_v1(update)?;
        txn.apply_update(decoded);
    }

    // 2. Record acknowledgment for all tombstones
    {
        let mut txn = doc.transact_mut();
        let collection = get_collection(&doc, concern);

        for (key, item) in collection.iter(&txn) {
            if item.get("deleted").unwrap_or(false) {
                let ack_vector = item.get_or_create("ack_vector");
                ack_vector.insert(peer_id, Utc::now().timestamp());
            }
        }
    }

    crdt_manager.save_document(&doc_id, &doc).await?;

    // 3. Immediately check for prunable tombstones
    prune_tombstones(&doc_id, concern).await?;

    Ok(())
}

pub async fn prune_tombstones(doc_id: &str, concern: &str) -> Result<Vec<String>> {
    let doc = crdt_manager.load_document(doc_id).await?;
    let active_members = get_active_members(&doc);
    let mut pruned = Vec::new();

    // Find tombstones where all active members have acknowledged
    let to_prune: Vec<String> = {
        let txn = doc.transact();
        let collection = get_collection(&doc, concern);

        collection.iter(&txn)
            .filter(|(_, item)| {
                let is_deleted = item.get("deleted").unwrap_or(false);
                if !is_deleted { return false; }

                let ack_vector = item.get("ack_vector");
                let acknowledged: HashSet<_> = ack_vector.keys().collect();

                // All active members have acknowledged
                active_members.iter().all(|m| acknowledged.contains(m))
            })
            .map(|(key, _)| key.to_string())
            .collect()
    };

    // Prune immediately
    if !to_prune.is_empty() {
        let mut txn = doc.transact_mut();
        let collection = get_collection(&doc, concern);

        for key in &to_prune {
            collection.remove(&mut txn, key);
            pruned.push(key.clone());
        }
    }

    if !pruned.is_empty() {
        crdt_manager.save_document(doc_id, &doc).await?;
    }

    Ok(pruned)
}
```

### Active Members Tracking

Active members are tracked separately for efficient pruning:

```rust
// In core document
Doc {
    "members": Map<member_id, MemberData>,      // Full member data
    "active_members": Map<member_id, bool>,     // Quick lookup
}

// Adding member
active_members.insert(member_id, true);

// Removing member (creates tombstone)
members[member_id].deleted = true;
active_members.remove(member_id);  // No longer active
```

### Handling Offline Members

**Key insight**: Offline members are still active until explicitly removed.

| Scenario | Behavior |
|----------|----------|
| Member offline | Tombstone persists until they sync |
| Member removed | No longer in active_members, not counted |
| Member re-joins | Added back to active_members |

**Worst case**: All members offline except deleter → tombstone persists until someone syncs.

**Typical case**: Members online within hours → tombstones pruned within minutes.

### Tombstone Overhead

Per tombstone:
- Base data: ~200 bytes
- Ack vector: ~20 bytes × active_member_count

| Members | Per Tombstone | 1000 Tombstones |
|---------|---------------|-----------------|
| 10 | ~400 bytes | ~400KB |
| 50 | ~1.2KB | ~1.2MB |
| 100 | ~2.2KB | ~2.2MB |

## Consequences

### Benefits

- **No background tasks**: Pruning happens on sync events
- **Immediate cleanup**: As soon as safe, tombstones gone
- **Offline-safe**: Respects offline member state
- **Simple model**: No timers, no GC schedules
- **Predictable**: Pruning triggers are explicit

### Trade-offs

- **Sync dependency**: Offline members delay pruning
- **Ack vector size**: Grows with member count
- **All-or-nothing**: Pruning only when ALL have acked

### Edge Cases

| Case | Handling |
|------|----------|
| Long-offline member | Tombstones persist (correct!) |
| Network partition | Each partition tracks own acks |
| Concurrent deletes | Each tombstone tracked separately |
| Re-add deleted item | New item, independent of tombstone |

## Alternatives Considered

1. **Time-based TTL**: Prune after N days
   - Rejected: Offline peers might miss deletions

2. **Periodic GC**: Background task prunes old tombstones
   - Rejected: Complexity, timing issues

3. **Never prune**: Keep all tombstones forever
   - Rejected: Unbounded growth

4. **Quorum-based**: Prune when majority acked
   - Rejected: Minority might still need tombstone

5. **Checkpointing**: Periodic full-state snapshots
   - Rejected: Doesn't solve per-item cleanup

## References

- CRDT Architecture: `docs/architecture/crdt-system.md`
- Implementation: `communitas-core/src/crdt/`
- Related ADR: [ADR-003 Yrs CRDT](ADR-003-yrs-crdt-synchronization.md)
- Related ADR: [ADR-002 Local-First](ADR-002-local-first-architecture.md)

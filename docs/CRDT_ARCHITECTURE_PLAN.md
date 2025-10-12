# Comprehensive CRDT Architecture Plan

**Date**: 2025-10-12
**Status**: Draft
**Priority**: Critical

## Executive Summary

This document outlines the migration from the current dual-write (CRDT + SQL) pattern to a **CRDT-first architecture** using Yrs (Yjs Rust implementation). The goal is to achieve 100% conflict-free, offline-first synchronization for all collaborative entities in Communitas.

## Current State Analysis

### Existing Architecture

#### ✅ What Works Well

1. **CrdtManager** (`crdt_manager.rs`)
   - Solid foundation for Yrs document persistence
   - Clean load/save/merge API
   - Proper state vector and diff handling for sync
   - SQLite (libSQL) backend for storage

2. **Document Commands** (`doc_commands.rs`)
   - Entity-scoped document storage working
   - Text insertion/deletion through Yrs Text CRDT
   - Update application and retrieval
   - Demonstrates proper Yrs usage patterns

3. **Schema Design** (`schema.sql`)
   - `crdt_documents` table for serialized Yrs state
   - `crdt_doc_id` foreign keys in entity tables
   - Good indexing strategy

#### ❌ Current Issues

1. **Inconsistent Dual-Write Pattern**
   ```rust
   // channel_service.rs - send_message()
   // 1. Write to CRDT (Yrs Array with MapPrelim)
   messages.push_back(&mut txn, msg_data);

   // 2. Write to SQL (separate transaction)
   db.execute("INSERT INTO messages ...", params![...]);
   ```

   **Problems:**
   - Two separate writes can get out of sync
   - CRDT is written first, SQL second (reversed priority)
   - No atomic transaction across both
   - Edit/delete only update SQL, not CRDT (recent impl)

2. **SQL as Primary Source**
   - Queries always read from SQL tables, not CRDT
   - CRDT becomes a "sync log" rather than source of truth
   - Defeats the purpose of CRDTs

3. **Incomplete CRDT Coverage**
   - ✅ Messages: Create works (CRDT + SQL)
   - ❌ Messages: Edit/delete only touch SQL
   - ✅ Documents: Full CRDT (Text type)
   - ❌ Channels: Only SQL
   - ❌ Issues: Only SQL
   - ❌ Projects: Only SQL

4. **Array Mutation Limitations**
   - Cannot mutate maps inside Yrs Arrays
   - Current edit implementation avoided CRDT update
   - Need different CRDT structure for editable collections

### Data Flow Comparison

#### Current (Broken)
```
User Action → Edit Message
  ↓
SQL UPDATE messages SET content = ?
  ↓
CRDT: No update (gets stale)
  ↓
Query: Read from SQL ✓
Sync to peer: Send stale CRDT ✗
```

#### Target (CRDT-First)
```
User Action → Edit Message
  ↓
CRDT: Update Map in document
  ↓
Persist: Save Yrs state to crdt_documents
  ↓
Rebuild SQL: Materialize view from CRDT
  ↓
Query: Read from SQL (fast queries) ✓
Sync to peer: Send latest CRDT ✓
```

## CRDT-First Architecture Design

### Core Principles

1. **CRDT is Source of Truth**
   - All mutations go through CRDT first
   - SQL is a **materialized view** for fast queries
   - SQL is rebuilt from CRDT, not written independently

2. **Document-Per-Entity**
   - Each channel = one Yrs Doc
   - Each issue = one Yrs Doc
   - Each project = one Yrs Doc
   - Document structure contains all entity data

3. **Yrs Data Types**
   - **Map**: For entities with fields (channels, issues, projects)
   - **Array**: For append-only collections (reactions, attachments)
   - **Text**: For collaborative text editing (message content, descriptions)
   - **Map of Maps**: For collections with editable items (messages in channel)

### Document Structures

#### Channel Document
```rust
Doc {
  "metadata": Map {
    "id": String,
    "org_id": String,
    "name": Text,           // Collaborative name editing
    "description": Text,    // Collaborative description
    "created_at": i64,
    "created_by": String,
  },
  "members": Map {
    "{user_id}": Map {
      "role": String,
      "joined_at": i64,
    },
    ...
  },
  "messages": Map {
    "{message_id}": Map {
      "author_id": String,
      "content": Text,        // Collaborative message editing
      "thread_id": String?,
      "created_at": i64,
      "updated_at": i64?,
      "deleted": bool?,       // Tombstone
      "deleted_at": i64?,
    },
    ...
  },
  "threads": Map {
    "{thread_id}": Map {
      "parent_message_id": String,
      "reply_count": i64,
      "last_reply_at": i64?,
    },
    ...
  },
}
```

**Why Map of Maps for Messages?**
- Maps support keyed access and updates
- Can edit individual message fields
- Can mark deleted with tombstone
- Maintains message identity across edits

#### Issue Document
```rust
Doc {
  "metadata": Map {
    "id": String,
    "project_id": String,
    "title": Text,           // Collaborative title editing
    "description": Text,     // Collaborative description
    "status": String,        // Last-write-wins via timestamp
    "status_updated_at": i64,
    "priority": String,
    "priority_updated_at": i64,
    "assignee_id": String?,
    "assignee_updated_at": i64?,
    "reporter_id": String,
    "created_at": i64,
  },
  "comments": Map {
    "{comment_id}": Map {
      "author_id": String,
      "content": Text,
      "created_at": i64,
      "updated_at": i64?,
      "deleted": bool?,
    },
    ...
  },
}
```

#### Project Document
```rust
Doc {
  "metadata": Map {
    "id": String,
    "org_id": String,
    "name": Text,
    "description": Text,
    "icon": String?,
    "color": String?,
    "created_at": i64,
    "created_by": String,
  },
}
```

### SQL as Materialized View

SQL tables become **read-only views** rebuilt from CRDT:

```rust
// After every CRDT mutation:
1. Update CRDT doc
2. Save Yrs state to crdt_documents table
3. Rebuild affected SQL rows from CRDT
4. Return success
```

**Why Keep SQL?**
- Fast indexed queries (find messages by channel, issues by status)
- Full-text search
- Complex joins
- Existing query patterns don't need rewrite
- Network sync only needs CRDT, not SQL

### Conflict Resolution Strategy

#### Automatically Resolved by CRDTs

1. **Text Editing** (Yrs Text)
   - Concurrent edits to same message/description
   - OT-based resolution (Yjs algorithm)
   - No user intervention needed

2. **Map Insertions** (Yrs Map)
   - Adding different messages concurrently
   - Adding different members concurrently
   - Last-write-wins for same key

3. **Nested Map Updates** (Map of Maps)
   - Editing different messages concurrently
   - Different fields of same message concurrently

#### Requires Application Logic

1. **Status Changes**
   - Use timestamp-based last-write-wins
   - Store `status_updated_at` alongside `status`
   - Application compares timestamps

2. **Assignment Changes**
   - Use timestamp-based last-write-wins
   - Store `assignee_updated_at` alongside `assignee_id`

3. **Deletions**
   - Use tombstone pattern (`deleted: true, deleted_at: timestamp`)
   - Never remove from CRDT Map (for sync correctness)
   - Filter out in queries

## Migration Plan

### Phase 1: Foundation (Week 1)

#### 1.1 Enhanced CrdtManager
- [ ] Add `materialize_to_sql()` method
- [ ] Add `get_map()`, `get_map_of_maps()` helpers
- [ ] Add `update_nested_field()` for atomic updates
- [ ] Add proper error types (not just anyhow)

#### 1.2 CRDT Service Layer
```rust
// New: crdt_service.rs
pub struct CrdtService {
    manager: Arc<CrdtManager>,
}

impl CrdtService {
    // Channel operations
    pub async fn channel_create(&self, ...) -> Result<Doc>;
    pub async fn channel_add_message(&self, ...) -> Result<()>;
    pub async fn channel_edit_message(&self, ...) -> Result<()>;
    pub async fn channel_delete_message(&self, ...) -> Result<()>;

    // Issue operations
    pub async fn issue_create(&self, ...) -> Result<Doc>;
    pub async fn issue_update_status(&self, ...) -> Result<()>;
    pub async fn issue_add_comment(&self, ...) -> Result<()>;

    // Materialization
    async fn materialize_channel(&self, doc: &Doc) -> Result<()>;
    async fn materialize_issue(&self, doc: &Doc) -> Result<()>;
}
```

#### 1.3 Schema Updates
```sql
-- Add deleted_at to messages (already done)
ALTER TABLE messages ADD COLUMN deleted_at INTEGER;

-- Add status/priority timestamps to issues
ALTER TABLE issues ADD COLUMN status_updated_at INTEGER;
ALTER TABLE issues ADD COLUMN priority_updated_at INTEGER;
ALTER TABLE issues ADD COLUMN assignee_updated_at INTEGER;

-- Add comment soft delete
ALTER TABLE issue_comments ADD COLUMN deleted_at INTEGER;
```

### Phase 2: Messages CRDT (Week 2)

#### 2.1 Refactor channel_service.rs

**send_message():**
```rust
pub async fn send_message(...) -> Result<Message> {
    // 1. Load channel CRDT doc
    let doc = self.crdt.load_document(&doc_id).await?;

    // 2. Add to messages Map (not Array)
    {
        let messages = doc.get_or_insert_map("messages");
        let message_map = messages.insert_map(&mut doc.transact_mut(), msg_id.clone());

        message_map.insert(&mut doc.transact_mut(), "author_id", author_id.into());
        message_map.insert(&mut doc.transact_mut(), "content", content.into());
        message_map.insert(&mut doc.transact_mut(), "created_at", now.into());
        // ... more fields
    }

    // 3. Save CRDT
    self.crdt.save_document(&doc_id, "channel", channel_id, &doc).await?;

    // 4. Materialize to SQL
    self.materialize_messages(&doc, channel_id).await?;

    Ok(message)
}
```

**edit_message():**
```rust
pub async fn edit_message(...) -> Result<Message> {
    // 1. Load channel CRDT doc
    let doc = self.crdt.load_document(&doc_id).await?;

    // 2. Update message in Map
    {
        let messages = doc.get_or_insert_map("messages");
        let message_map = messages.get(&doc.transact(), message_id)
            .and_then(|v| v.to_ymap())
            .ok_or("Message not found")?;

        message_map.insert(&mut doc.transact_mut(), "content", new_content.into());
        message_map.insert(&mut doc.transact_mut(), "updated_at", now.into());
    }

    // 3. Save CRDT
    self.crdt.save_document(&doc_id, "channel", channel_id, &doc).await?;

    // 4. Update SQL
    self.materialize_message(&doc, message_id).await?;

    Ok(message)
}
```

**delete_message():**
```rust
pub async fn delete_message(...) -> Result<()> {
    // 1. Load channel CRDT doc
    let doc = self.crdt.load_document(&doc_id).await?;

    // 2. Mark deleted in CRDT (tombstone)
    {
        let messages = doc.get_or_insert_map("messages");
        let message_map = messages.get(&doc.transact(), message_id)
            .and_then(|v| v.to_ymap())
            .ok_or("Message not found")?;

        message_map.insert(&mut doc.transact_mut(), "deleted", true.into());
        message_map.insert(&mut doc.transact_mut(), "deleted_at", now.into());
    }

    // 3. Save CRDT
    self.crdt.save_document(&doc_id, "channel", channel_id, &doc).await?;

    // 4. Update SQL
    self.materialize_message(&doc, message_id).await?;

    Ok(())
}
```

#### 2.2 Materialization Logic
```rust
async fn materialize_message(&self, doc: &Doc, message_id: &str) -> Result<()> {
    let messages = doc.get_or_insert_map("messages");
    let txn = doc.transact();

    let message_map = messages.get(&txn, message_id)
        .and_then(|v| v.to_ymap())
        .ok_or("Message not found")?;

    // Extract fields from CRDT
    let author_id = message_map.get(&txn, "author_id")
        .and_then(|v| v.to_string(&txn))
        .ok_or("Missing author_id")?;

    let content = message_map.get(&txn, "content")
        .and_then(|v| v.to_string(&txn))
        .ok_or("Missing content")?;

    let deleted_at = message_map.get(&txn, "deleted_at")
        .and_then(|v| v.to_i64(&txn));

    // ... more fields

    // Write to SQL
    let db = self.crdt.connection()?;
    db.execute(
        "INSERT OR REPLACE INTO messages
         (id, channel_id, author_id, content, created_at, updated_at, deleted_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![message_id, channel_id, author_id, content, created_at, updated_at, deleted_at],
    ).await?;

    Ok(())
}
```

#### 2.3 Migration Script
```rust
// migrate_messages_to_crdt.rs
async fn migrate_channel(manager: &CrdtManager, channel_id: &str) -> Result<()> {
    // 1. Read all messages from SQL
    let messages = load_messages_from_sql(channel_id).await?;

    // 2. Create CRDT doc
    let doc = Doc::new();
    let messages_map = doc.get_or_insert_map("messages");

    // 3. Populate CRDT from SQL
    for msg in messages {
        let msg_map = messages_map.insert_map(&mut doc.transact_mut(), msg.id.clone());
        msg_map.insert(&mut doc.transact_mut(), "author_id", msg.author_id.into());
        msg_map.insert(&mut doc.transact_mut(), "content", msg.content.into());
        // ... more fields
    }

    // 4. Save CRDT
    let doc_id = format!("channel:{}", channel_id);
    manager.save_document(&doc_id, "channel", channel_id, &doc).await?;

    Ok(())
}
```

### Phase 3: Issues & Projects CRDT (Week 3)

Repeat Phase 2 pattern for:
- Issue create/update/delete/comment
- Project create/update

### Phase 4: Sync Protocol (Week 4)

#### 4.1 Network Sync
```rust
// sync_service.rs
pub struct SyncService {
    crdt: Arc<CrdtManager>,
    network: Arc<NetworkService>,
}

impl SyncService {
    /// Sync a document with a peer
    pub async fn sync_document(&self, doc_id: &str, peer_id: &str) -> Result<()> {
        // 1. Get our state vector
        let our_sv = self.crdt.get_state_vector(doc_id).await?;

        // 2. Send state vector to peer
        self.network.send_state_vector(peer_id, doc_id, &our_sv).await?;

        // 3. Receive peer's diff
        let peer_diff = self.network.receive_diff(peer_id, doc_id).await?;

        // 4. Apply peer's diff
        self.crdt.merge_update(doc_id, "channel", entity_id, &peer_diff).await?;

        // 5. Send our diff
        let peer_sv = self.network.receive_state_vector(peer_id, doc_id).await?;
        let our_diff = self.crdt.get_diff(doc_id, &peer_sv).await?;
        self.network.send_diff(peer_id, doc_id, &our_diff).await?;

        // 6. Materialize updated doc
        let doc = self.crdt.load_document(doc_id).await?;
        self.materialize_from_sync(&doc).await?;

        Ok(())
    }
}
```

#### 4.2 Conflict-Free Guarantees
- Yrs handles all Text and Map conflicts automatically
- Timestamp-based LWW for status/priority/assignment
- Tombstone pattern for deletes
- No user prompts needed

### Phase 5: Testing & Validation (Week 5)

#### 5.1 Unit Tests
- [ ] Test CRDT operations in isolation
- [ ] Test materialization logic
- [ ] Test tombstone filtering
- [ ] Test timestamp-based LWW

#### 5.2 Integration Tests
- [ ] Multi-peer sync scenarios
- [ ] Concurrent edits on same message
- [ ] Offline edit then sync
- [ ] Network partition recovery

#### 5.3 Property-Based Tests
```rust
#[quickcheck]
fn prop_sync_is_commutative(ops1: Vec<Op>, ops2: Vec<Op>) -> bool {
    // Apply ops1 then ops2
    let doc1 = apply_ops(&[ops1.clone(), ops2.clone()].concat());

    // Apply ops2 then ops1
    let doc2 = apply_ops(&[ops2, ops1].concat());

    // Should converge to same state
    doc1.encode_state() == doc2.encode_state()
}
```

## Implementation Checklist

### Week 1: Foundation
- [ ] Enhance CrdtManager with Map/Text helpers
- [ ] Create CrdtService abstraction
- [ ] Update schema with timestamps and soft delete
- [ ] Write materialization framework
- [ ] Add comprehensive logging

### Week 2: Messages
- [ ] Migrate send_message to CRDT-first
- [ ] Migrate edit_message to CRDT Map updates
- [ ] Migrate delete_message to tombstone pattern
- [ ] Implement message materialization
- [ ] Write data migration script
- [ ] Test with multi-peer sync

### Week 3: Issues & Projects
- [ ] Migrate issue operations to CRDT-first
- [ ] Implement LWW for status/priority/assignment
- [ ] Migrate project operations to CRDT-first
- [ ] Write data migration scripts
- [ ] Test conflict scenarios

### Week 4: Sync Protocol
- [ ] Implement state vector exchange
- [ ] Implement diff-based sync
- [ ] Add sync scheduling (periodic + on-change)
- [ ] Add sync retry with exponential backoff
- [ ] Integrate with network layer

### Week 5: Testing & Docs
- [ ] Write comprehensive unit tests
- [ ] Write integration tests
- [ ] Write property-based tests
- [ ] Document CRDT patterns for developers
- [ ] Update API documentation
- [ ] Performance benchmarking

## Success Criteria

1. **Zero SQL-first writes** - All mutations go through CRDT
2. **100% offline capability** - All operations work without network
3. **Automatic conflict resolution** - No user prompts for conflicts
4. **Correct convergence** - All peers converge to same state
5. **Performance** - <100ms for local operations, <500ms for sync
6. **No data loss** - Migration preserves all existing data

## Risk Mitigation

### Risk: Yrs API Complexity
- **Mitigation**: Start with simple Text/Map types
- **Mitigation**: Create helper functions for common patterns
- **Mitigation**: Comprehensive tests for each CRDT operation

### Risk: Migration Data Loss
- **Mitigation**: Backup database before migration
- **Mitigation**: Dry-run validation before writing
- **Mitigation**: Rollback script for emergency

### Risk: Performance Degradation
- **Mitigation**: Keep SQL for fast queries (materialized view)
- **Mitigation**: Batch materialization updates
- **Mitigation**: Index CRDT state vectors for fast diff

### Risk: Breaking Changes
- **Mitigation**: Incremental rollout (Phase 2 → 3 → 4)
- **Mitigation**: Feature flags for CRDT vs SQL paths
- **Mitigation**: A/B testing with small user group

## References

- [Yrs Documentation](https://docs.rs/yrs)
- [Yjs Algorithm](https://github.com/yjs/yjs)
- [CRDT Tech Report](https://crdt.tech/)
- [Automerge Architecture](https://automerge.org/docs/how-it-works/)
- [Current Implementation: crdt_manager.rs](../communitas-desktop/src/crdt_manager.rs)
- [Current Implementation: channel_service.rs](../communitas-desktop/src/services/channel_service.rs)

---

**Next Steps**: Review this plan, then begin Phase 1 foundation work.

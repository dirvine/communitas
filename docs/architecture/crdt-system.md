# CRDT System Architecture

Comprehensive guide to Communitas' Conflict-Free Replicated Data Type (CRDT) system for offline-first, real-time collaboration.

## Overview

Communitas uses a **modular CRDT architecture** where each entity (group, channel, project, organization) is composed of multiple specialized CRDT documents, each handling a specific concern. This design provides:

- ✅ **Offline-first** - All operations work without network connectivity
- ✅ **Bounded document sizes** - Each concern has independent size limits
- ✅ **Efficient sync** - Update members without syncing messages
- ✅ **Event-driven tombstone pruning** - No periodic cleanup tasks
- ✅ **SQL materialization (planned)** - Large datasets can be materialized to SQL in a future layer
- ✅ **Automatic conflict resolution** - Yrs handles merging via CRDT semantics

## Storage Architecture

### Implementation Status (Current)

CRDT state is currently persisted to the filesystem (`.yrs` + `.meta` files) by
`communitas-core/src/crdt_manager`. The SQL materialization layer described below is
planned and not part of the current runtime.

### Three-Tier Storage Model

```
┌────────────────────────────────────────────────────────────────┐
│              COMMUNITAS STORAGE LAYER                          │
│                                                                 │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐ │
│  │  Virtual Disks   │  │  CRDT Documents  │  │ SQL (Planned)│ │
│  │  (Markdown)      │  │  (Yrs State)     │  │  (Queries)   │ │
│  │                  │  │                  │  │              │ │
│  │  • Private       │  │  • Members       │  │  • Old msgs  │ │
│  │  • Public        │  │  • Chat          │  │  • Archived  │ │
│  │  • Shared        │  │  • Kanban        │  │  • Reports   │ │
│  │  • Website Root  │  │  • Issues        │  │              │ │
│  └──────────────────┘  └──────────────────┘  └──────────────┘ │
│         ↕                      ↕                      ↕         │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  Filesystem Persistence (current)                      │  │
│  │  • .yrs (Yrs state blobs)                               │  │
│  │  • .meta (JSON metadata)                                │  │
│  └─────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

### Storage Layer Responsibilities

#### 1. Virtual Disks (Full File Replication)

**Purpose**: File storage with full replication across gossip network

**Contents**:
- User documents, wikis, notes (markdown files)
- Website content (published via identity.website_root)
- Binary files (images, attachments)

**Format**: Markdown files with full replication
**Access Control**: Private/Public/Shared encryption policies
**Synchronization**: Yrs CRDT for collaborative file editing

**Example**:
```
Entity: group:ocean-forest-moon-star

Virtual Disks:
  /private/docs/meeting-notes.md       → Group documents (markdown)
  /shared/images/logo.png              → Shared files
  /public/website/index.html           → Published website
  /shared/attachments/screenshot.png   → Message attachments
```

#### 2. CRDT Documents (Yrs State)

**Purpose**: Real-time structured data with automatic conflict resolution

**Contents**:
- Entity metadata and members
- Chat messages and threads
- Domain-specific data (kanban, issues)

**Format**: Yrs binary state stored on the filesystem (`.yrs` files)
**Sync Protocol**: State vectors and incremental updates
**Conflict Resolution**: Automatic via Yrs CRDT semantics

#### 3. SQL Cache (Materialized Views) — Planned

**Purpose**: Read-optimized queries for large datasets

**Contents**:
- Read-optimized queries
- Historical data (old messages, archived cards)
- Aggregations and reports

**Note**: The SQL fallback thresholds and schemas below are design targets for a future
materialization layer. They are not implemented in the current runtime.

**Trigger**: CRDT document exceeds size threshold
**Access**: Unified read interface (transparent to application)

## Document Taxonomy

### Document ID Pattern

Every CRDT document follows this naming convention:

```
{entity_type}:{entity_id}:{concern}
```

**Examples**:
- `group:ocean-forest-moon-star:core` - Group metadata and members
- `group:ocean-forest-moon-star:chat` - Group chat messages
- `channel:bright-river-wind-star:core` - Channel metadata
- `channel:bright-river-wind-star:chat` - Channel messages
- `project:gentle-wave-moon-fire:core` - Project metadata
- `project:gentle-wave-moon-fire:chat` - Project chat
- `project:gentle-wave-moon-fire:kanban` - Project kanban board
- `project:gentle-wave-moon-fire:issues` - Project issue tracker

### Base Documents (All Entities)

#### 1. Core Document

**Document ID**: `{entity_type}:{entity_id}:core`

**Schema**:
```rust
Doc {
    "metadata": Map {
        "entity_id": String,           // Four-word identity
        "entity_type": String,         // "group" | "channel" | "project" | "organization"
        "name": String,
        "description": String,
        "four_words": String,          // e.g., "ocean-forest-moon-star"
        "created_at": i64,             // Unix timestamp
        "created_by": String,          // Four-word identity of creator
        "website_root": String,        // Optional: BLAKE3 hash of website root
    },

    "members": Map<member_id, MemberData>,

    "active_members": Map<member_id, bool>,  // Quick lookup: is member active?
}

MemberData = Map {
    "member_id": String,               // Four-word identity
    "role": String,                    // "owner" | "admin" | "member"
    "joined_at": i64,                  // Unix timestamp
    "deleted": bool,                   // Tombstone flag
    "deleted_at": i64,                 // When deletion occurred (0 if active)
    "deleted_by": String,              // Who initiated deletion
    "ack_vector": Map<member_id, i64>, // Tombstone acknowledgments (for pruning)
}
```

**Size Limit**: ~1MB (unlikely to exceed with just metadata + members)

**Purpose**:
- Entity identity and configuration
- Member management with roles
- Event-driven tombstone pruning

#### 2. Chat Document

**Document ID**: `{entity_type}:{entity_id}:chat`

**Schema**:
```rust
Doc {
    "messages": Map<message_id, MessageData>,

    "threads": Map<parent_message_id, ThreadData>,
}

MessageData = Map {
    "id": String,                      // UUID or content-hash
    "author_id": String,               // Four-word identity
    "content": String,                 // Message text
    "created_at": i64,                 // Unix timestamp
    "updated_at": i64,                 // LWW for edits
    "deleted": bool,                   // Tombstone flag
    "deleted_at": i64,                 // When deletion occurred
    "ack_vector": Map<member_id, i64>, // Tombstone acknowledgments
    "reactions": Map<emoji, Array<member_id>>,  // e.g., "👍" -> ["alice-...", "bob-..."]
    "attachments": Array<AttachmentData>,
}

AttachmentData = Map {
    "id": String,
    "filename": String,
    "file_path": String,               // Path in virtual disk (/shared/attachments/...)
    "mime_type": String,
    "size_bytes": i64,
}

ThreadData = Map {
    "parent_id": String,               // Message this thread replies to
    "replies": Map<message_id, MessageData>,  // Nested messages
    "reply_count": i64,                // Cached count
    "last_reply_at": i64,              // LWW timestamp
}
```

**Size Limit**: 10MB
**Fallback**: When exceeds 10MB, materialize messages older than 90 days to SQL

**Purpose**:
- Real-time chat and threaded discussions
- Message reactions and attachments
- Same tombstone pruning strategy as members

**Attachments Bridge**:
```rust
// In chat message
AttachmentData = Map {
    "file_path": "/shared/attachments/screenshot_123.png",  // Path in virtual disk
    "filename": "screenshot.png",
    "size_bytes": 245678,
}

// Virtual disk stores actual file (full replication)
/shared/attachments/screenshot_123.png
```

### Domain-Specific Documents

#### 3. Kanban Document (Projects Only)

**Document ID**: `project:{entity_id}:kanban`

**Schema**:
```rust
Doc {
    "columns": Array<ColumnData>,

    "cards": Map<card_id, CardData>,
}

ColumnData = Map {
    "id": String,
    "title": String,                   // "Backlog", "In Progress", "Done"
    "order": i64,                      // Position in board
    "card_order": Array<card_id>,      // Cards in this column (ordered)
}

CardData = Map {
    "id": String,
    "title": String,
    "description": String,
    "column_id": String,               // Which column contains this card
    "assignee_id": String,             // Four-word identity
    "priority": String,                // "urgent" | "high" | "medium" | "low"
    "created_at": i64,
    "updated_at": i64,                 // LWW for edits
    "archived": bool,                  // Tombstone equivalent
    "archived_at": i64,
    "ack_vector": Map<member_id, i64>, // Tombstone acknowledgments
}
```

**Size Limit**: 5MB
**Fallback**: When exceeds 5MB, materialize archived cards to SQL

**Purpose**:
- Visual project management
- Drag-and-drop task organization
- Same tombstone pruning for archived cards

#### 4. Issues Document (Projects Only)

**Document ID**: `project:{entity_id}:issues`

**Schema**:
```rust
Doc {
    "issues": Map<issue_id, IssueData>,
}

IssueData = Map {
    "id": String,
    "title": String,
    "description": String,
    "status": String,                  // "backlog" | "todo" | "in-progress" | "done" | "canceled"
    "priority": String,                // "urgent" | "high" | "medium" | "low"
    "assignee_id": String,             // Four-word identity
    "reporter_id": String,             // Four-word identity
    "created_at": i64,
    "updated_at": i64,                 // LWW for field changes
    "status_updated_at": i64,          // LWW for status field
    "priority_updated_at": i64,        // LWW for priority field
    "assignee_updated_at": i64,        // LWW for assignee field
    "closed": bool,                    // Tombstone equivalent
    "closed_at": i64,
    "ack_vector": Map<member_id, i64>, // Tombstone acknowledgments
    "comments": Array<CommentData>,    // Inline comments
}

CommentData = Map {
    "id": String,
    "author_id": String,
    "content": String,
    "created_at": i64,
    "updated_at": i64,
    "deleted": bool,
    "deleted_at": i64,
}
```

**Size Limit**: 10MB
**Fallback**: When exceeds 10MB, materialize closed issues to SQL

**Purpose**:
- Linear-style issue tracking
- Fine-grained LWW timestamps per field
- Comment threads within issues

## Event-Driven Tombstone Pruning

### Core Principle

**Tombstones are pruned immediately when all active members have acknowledged them** - no periodic cleanup tasks needed.

### Why Tombstone Pruning Matters

In traditional CRDT systems, tombstones (deletion markers) accumulate indefinitely, causing:
- Unbounded document growth
- Slower sync times
- Increased memory usage
- Performance degradation

Communitas solves this with **event-driven tombstone pruning**: as soon as all active members have seen a deletion, the tombstone is removed.

### Pruning Flow

```
┌──────────────────────────────────────────────────────────────┐
│  1. Member/Message Deletion                                  │
│     - Set deleted=true, deleted_at=timestamp                 │
│     - Initialize empty ack_vector                            │
│     - Remove from active_members (for members only)          │
│     - Broadcast update to all peers                          │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│  2. Peer Receives Update (Sync Event)                        │
│     - Apply CRDT update (Yrs merge)                          │
│     - Record peer's state vector in ack_vector               │
│     - Trigger pruning check                                  │
└──────────────────────────────────────────────────────────────┘
                            ↓
┌──────────────────────────────────────────────────────────────┐
│  3. Immediate Pruning Check                                  │
│     - For each tombstone:                                    │
│       • Check if all active members in ack_vector            │
│       • If YES: map.remove(txn, tombstone_id)                │
│     - Save document if any tombstones pruned                 │
└──────────────────────────────────────────────────────────────┘
```

### Implementation

**Sync Handler** (`communitas-desktop/src/member_manager.rs`):
```rust
pub async fn on_peer_sync(
    entity_id: &str,
    peer_id: &str,
    update: &[u8],
    concern: &str,  // "core", "chat", "kanban", "issues"
) -> Result<()> {
    let doc_id = format!("{}:{}", entity_id, concern);
    let doc = crdt_manager.load_document(&doc_id).await?;

    // Apply incoming update
    {
        let mut txn = doc.transact_mut();
        let decoded = yrs::Update::decode_v1(update)?;
        txn.apply_update(decoded);
    }

    // Record peer acknowledgment for all tombstones
    {
        let mut txn = doc.transact_mut();
        let collection = match concern {
            "core" => doc.get_or_insert_map("members"),
            "chat" => doc.get_or_insert_map("messages"),
            "kanban" => doc.get_or_insert_map("cards"),
            "issues" => doc.get_or_insert_map("issues"),
            _ => return Err("Unknown concern".into()),
        };

        let keys: Vec<String> = collection.keys(&txn).map(|k| k.to_string()).collect();

        for key in keys {
            if let Some(item_map) = CrdtManager::get_nested_map(&collection, &txn, &key) {
                if CrdtManager::get_map_bool(&item_map, &txn, "deleted").unwrap_or(false) {
                    let ack_vector = CrdtManager::get_or_create_nested_map(&item_map, &mut txn, "ack_vector");
                    CrdtManager::set_map_i64(&ack_vector, &mut txn, peer_id, Utc::now().timestamp());
                }
            }
        }
    }

    crdt_manager.save_document(&doc_id, "entity", entity_id, &doc).await?;

    // Trigger immediate pruning
    prune_tombstones(&doc_id, concern).await?;

    Ok(())
}
```

**Pruning Logic**:
```rust
pub async fn prune_tombstones(doc_id: &str, concern: &str) -> Result<Vec<String>> {
    let doc = crdt_manager.load_document(doc_id).await?;
    let mut pruned = Vec::new();

    // Get active members from core document
    let core_doc_id = doc_id.replace(&format!(":{}", concern), ":core");
    let core_doc = crdt_manager.load_document(&core_doc_id).await?;
    let active_member_ids: Vec<String> = {
        let txn = core_doc.transact();
        let active = core_doc.get_or_insert_map("active_members");
        active.keys(&txn).map(|k| k.to_string()).collect()
    };

    // Check each tombstone for full acknowledgment
    let to_prune: Vec<String> = {
        let txn = doc.transact();
        let collection = match concern {
            "core" => doc.get_or_insert_map("members"),
            "chat" => doc.get_or_insert_map("messages"),
            "kanban" => doc.get_or_insert_map("cards"),
            "issues" => doc.get_or_insert_map("issues"),
            _ => return Err("Unknown concern".into()),
        };

        let keys: Vec<String> = collection.keys(&txn).map(|k| k.to_string()).collect();
        let mut candidates = Vec::new();

        for key in keys {
            if let Some(item_map) = CrdtManager::get_nested_map(&collection, &txn, &key) {
                if CrdtManager::get_map_bool(&item_map, &txn, "deleted").unwrap_or(false) {
                    if let Some(ack_vector) = CrdtManager::get_nested_map(&item_map, &txn, "ack_vector") {
                        let acked: Vec<String> = ack_vector.keys(&txn).map(|k| k.to_string()).collect();

                        // All active members have acknowledged?
                        if active_member_ids.iter().all(|m| acked.contains(m)) {
                            candidates.push(key);
                        }
                    }
                }
            }
        }
        candidates
    };

    // Prune immediately
    if !to_prune.is_empty() {
        let mut txn = doc.transact_mut();
        let collection = match concern {
            "core" => doc.get_or_insert_map("members"),
            "chat" => doc.get_or_insert_map("messages"),
            "kanban" => doc.get_or_insert_map("cards"),
            "issues" => doc.get_or_insert_map("issues"),
            _ => return Err("Unknown concern".into()),
        };

        for key in &to_prune {
            collection.remove(&mut txn, key);
            pruned.push(key.clone());
        }
    }

    if !pruned.is_empty() {
        crdt_manager.save_document(doc_id, "entity", entity_id, &doc).await?;
        tracing::info!("Pruned {} tombstones from {}: {:?}", pruned.len(), doc_id, pruned);
    }

    Ok(pruned)
}
```

### Tombstone Performance

**Per tombstone overhead**:
- Base: ~200 bytes
- Ack vector: ~20 bytes × active member count
- Total for 100 members: ~2.2KB per tombstone

**Lifespan**:
- Average: 1-2 sync rounds (~seconds to minutes)
- Maximum: Until all members online and sync

**Worst case**: 1000 tombstones × 100 members = ~2.2MB overhead (rare and temporary)

## SQL Fallback Strategy (Planned)

### Size Thresholds

When CRDT documents grow too large, old data materializes to SQL automatically:

| Document | Threshold | Fallback Action |
|----------|-----------|-----------------|
| `core` | 1MB | Unlikely to exceed |
| `chat` | 10MB | Materialize messages older than 90 days |
| `kanban` | 5MB | Materialize archived cards |
| `issues` | 10MB | Materialize closed issues |

### Hybrid Mode

When a document exceeds its threshold:

**1. Materialize old data to SQL**:
```sql
-- Automatically triggered when threshold exceeded
INSERT INTO archived_messages
SELECT * FROM crdt_chat_view
WHERE created_at < (NOW() - INTERVAL '90 days');
```

**2. Keep recent data in CRDT**:
- Last 90 days of messages
- Active kanban cards
- Open issues

**3. Mark document as hybrid**:
```rust
metadata.insert("fallback_mode", "hybrid");
metadata.insert("fallback_threshold_at", Utc::now().timestamp());
```

**4. Unified read interface**:
```rust
pub async fn get_messages(entity_id: &str, limit: usize) -> Result<Vec<Message>> {
    let mode = check_fallback_mode(entity_id, "chat").await?;

    match mode {
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

### SQL Schema

```sql
-- Entities table
CREATE TABLE entities (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    four_words TEXT NOT NULL UNIQUE,
    name TEXT,
    created_at INTEGER,
    created_by TEXT
);

-- CRDT documents table
CREATE TABLE crdt_documents (
    doc_id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    concern TEXT NOT NULL,
    state_blob BLOB NOT NULL,
    last_updated INTEGER,
    FOREIGN KEY (entity_id) REFERENCES entities(id)
);

-- Members table
CREATE TABLE members (
    entity_id TEXT NOT NULL,
    four_words TEXT NOT NULL,
    role TEXT,
    joined_at INTEGER,
    PRIMARY KEY (entity_id, four_words),
    FOREIGN KEY (entity_id) REFERENCES entities(id)
);

-- Archived messages (SQL fallback)
CREATE TABLE archived_messages (
    id TEXT PRIMARY KEY,
    entity_id TEXT NOT NULL,
    author_id TEXT,
    content TEXT,
    created_at INTEGER,
    deleted BOOLEAN,
    FOREIGN KEY (entity_id) REFERENCES entities(id)
);
```

## Offline-First Guarantees

### Local-First Operations

All operations work offline and sync when network returns:

```rust
pub async fn add_member_offline(entity_id: &str, member_id: &str, role: &str) -> Result<()> {
    // Works immediately, no network required
    let doc = crdt_manager.load_document(&format!("{}:core", entity_id)).await?;

    {
        let mut txn = doc.transact_mut();
        let members = doc.get_or_insert_map("members");
        let member_map = CrdtManager::get_or_create_nested_map(&members, &mut txn, member_id);

        CrdtManager::set_map_string(&member_map, &mut txn, "member_id", member_id);
        CrdtManager::set_map_string(&member_map, &mut txn, "role", role);
        CrdtManager::set_map_i64(&member_map, &mut txn, "joined_at", Utc::now().timestamp());
        CrdtManager::set_map_bool(&member_map, &mut txn, "deleted", false);

        let active = doc.get_or_insert_map("active_members");
        active.insert(&mut txn, member_id.to_string(), true);
    }

    crdt_manager.save_document(&format!("{}:core", entity_id), "entity", entity_id, &doc).await?;

    // Queue for sync when online
    sync_queue.enqueue(SyncOperation::MemberAdd { entity_id, member_id, role }).await?;

    Ok(())
}
```

### Conflict Resolution

**Yrs provides automatic conflict resolution via CRDT semantics:**

- **Concurrent edits to different fields**: Both changes preserved
- **Concurrent edits to same field**: Last-Write-Wins (LWW) based on Lamport timestamp
- **Concurrent deletions**: Deletion wins (tombstone)
- **Add + Delete same member concurrently**: Deletion wins
- **Move card to different columns**: LWW determines final position

**No manual conflict resolution needed** - Yrs handles it automatically.

### Sync Protocol

**State Vector Sync**:
```rust
// Get state vector (what we have)
let state_vector = doc.transact().state_vector();

// Generate diff (what peer needs)
let diff = doc.transact().encode_state_as_update(&state_vector);

// Send only the diff (incremental sync)
gossip_context.publish(topic, diff).await?;
```

**Advantages**:
- Only send changes (not full state)
- Efficient bandwidth usage
- Fast synchronization

## Performance Characteristics

### Memory Usage

| Document | Size | Load Time | Memory |
|----------|------|-----------|--------|
| `core` (100 members) | ~50KB | <10ms | ~200KB |
| `chat` (1000 messages) | ~2MB | ~50ms | ~8MB |
| `kanban` (500 cards) | ~1MB | ~20ms | ~4MB |
| `issues` (1000 issues) | ~5MB | ~100ms | ~20MB |

**Optimization**: Lazy-load documents only when needed.

### Sync Performance

| Operation | Network | CPU | Description |
|-----------|---------|-----|-------------|
| Add member | ~1KB | <1ms | Small state vector diff |
| Send message | ~2KB | <5ms | Message + metadata |
| Edit message | ~500B | <1ms | LWW timestamp update |
| Delete (tombstone) | ~500B | <1ms | Set deleted flag |
| Prune tombstone | 0 | <1ms | Local operation |
| Full entity sync | Variable | <100ms | State vector + diff |

**Optimization**: State vectors enable incremental sync (only send changes).

### Scalability

**Document size limits prevent unbounded growth**:
- Core: 1MB (100,000 members with metadata)
- Chat: 10MB (50,000 messages with threads)
- Kanban: 5MB (25,000 cards)
- Issues: 10MB (50,000 issues)

**SQL materialization handles arbitrarily large datasets**.

## API Integration

### Backend Commands (Core/FFI)

**Member Management**:
```rust
// communitas-core/src/legacy_ui_api.rs (removed)

// FFI boundary
pub async fn core_member_add(
    entity_id: String,
    member_four_words: String,
    role: String,  // "owner" | "admin" | "member"
) -> Result<bool, String>

// FFI boundary
pub async fn core_member_remove(
    entity_id: String,
    member_four_words: String,
) -> Result<bool, String>

// FFI boundary
pub async fn core_member_list(
    entity_id: String,
) -> Result<Vec<MemberInfo>, String>

// FFI boundary
pub async fn core_member_update_role(
    entity_id: String,
    member_four_words: String,
    new_role: String,
) -> Result<bool, String>
```

**Message Operations**:
```rust
// FFI boundary
pub async fn send_message(
    channel_id: String,
    content: String,
    attachments: Vec<AttachmentInfo>,
) -> Result<Message, String>

// FFI boundary
pub async fn edit_message(
    message_id: String,
    new_content: String,
) -> Result<bool, String>

// FFI boundary
pub async fn delete_message(
    message_id: String,
) -> Result<bool, String>
```

### Frontend Integration

**Dioxus (signals + shared services)**:
```rust
let messages = ui_services
    .messaging()
    .thread(channel_id.clone())
    .await?;

ui_services
    .messaging()
    .send_message(channel_id, UnifiedEntityType::Channel, "Hello, World!")
    .await?;
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_member_add_and_list() {
        let manager = MemberManager::new_test().await;

        manager.add_member("group:test:core", "alice-bob-charlie-delta", "owner").await.unwrap();
        manager.add_member("group:test:core", "echo-foxtrot-golf-hotel", "member").await.unwrap();

        let members = manager.list_members("group:test:core").await.unwrap();
        assert_eq!(members.len(), 2);
    }

    #[tokio::test]
    async fn test_tombstone_pruning() {
        let manager = MemberManager::new_test().await;

        // Add 3 members
        manager.add_member("group:test:core", "alice", "owner").await.unwrap();
        manager.add_member("group:test:core", "bob", "member").await.unwrap();
        manager.add_member("group:test:core", "charlie", "member").await.unwrap();

        // Remove bob (creates tombstone)
        manager.remove_member("group:test:core", "bob").await.unwrap();

        // Simulate alice and charlie syncing (acknowledging deletion)
        manager.on_peer_sync("group:test:core", "alice", &[]).await.unwrap();
        manager.on_peer_sync("group:test:core", "charlie", &[]).await.unwrap();

        // Tombstone should be pruned
        let doc = manager.load_document("group:test:core").await.unwrap();
        let txn = doc.transact();
        let members = doc.get_or_insert_map("members");
        assert!(!members.contains_key(&txn, "bob"));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_multi_node_sync() {
    // Simulate 3 peers editing same entity
    let peer1 = MemberManager::new_test().await;
    let peer2 = MemberManager::new_test().await;
    let peer3 = MemberManager::new_test().await;

    // Peer 1 adds member
    peer1.add_member("group:test:core", "alice", "owner").await.unwrap();

    // Sync to peer 2 and 3
    let update1 = peer1.get_update("group:test:core").await.unwrap();
    peer2.apply_update("group:test:core", &update1).await.unwrap();
    peer3.apply_update("group:test:core", &update1).await.unwrap();

    // All peers should have alice
    assert_eq!(peer1.list_members("group:test:core").await.unwrap().len(), 1);
    assert_eq!(peer2.list_members("group:test:core").await.unwrap().len(), 1);
    assert_eq!(peer3.list_members("group:test:core").await.unwrap().len(), 1);
}
```

## Migration Path

### Phase 1: Core Member Management ✅
- ✅ Implement `core` document with members
- ✅ Event-driven tombstone pruning
- ✅ Core/FFI commands: add/remove/list/update_role
- ✅ Frontend integration

### Phase 2: Chat Integration
- Implement `chat` document
- Message and thread operations
- Integrate with existing message sync
- Tombstone pruning for messages

### Phase 3: Domain Documents
- Implement `kanban` document (projects)
- Implement `issues` document (projects)
- SQL fallback for large documents

### Phase 4: Optimization
- Lazy loading of documents
- Background sync optimization
- Performance profiling
- SQL materialization tuning

## Best Practices

### Development Guidelines

1. **Always use transactions**:
   ```rust
   let mut txn = doc.transact_mut();
   // All changes within transaction
   ```

2. **Lazy-load documents**:
   ```rust
   // Only load when needed
   let doc = crdt_manager.load_document(doc_id).await?;
   ```

3. **Handle tombstones correctly**:
   ```rust
   // Set deleted flag, don't remove immediately
   item.insert(&mut txn, "deleted", true);
   ```

4. **Use LWW for fields that can change**:
   ```rust
   item.insert(&mut txn, "updated_at", Utc::now().timestamp());
   ```

### Common Pitfalls

❌ **Don't manually remove items** (breaks CRDT semantics):
```rust
collection.remove(&mut txn, key);  // Only for pruned tombstones!
```

✅ **Instead, use tombstone flags**:
```rust
item.insert(&mut txn, "deleted", true);
item.insert(&mut txn, "deleted_at", Utc::now().timestamp());
```

❌ **Don't block on sync**:
```rust
gossip_context.publish(topic, update).await?;  // Don't wait for ack
```

✅ **Fire and forget**:
```rust
tokio::spawn(async move {
    let _ = gossip_context.publish(topic, update).await;
});
```

## See Also

- [Architecture Overview](README.md) - System architecture
- [Architecture Overview](README.md) - Component details
- [Gossip Protocol](gossip-protocol.md) - P2P networking
- [Storage](storage.md) - Persistence layer
- [Yrs Documentation](https://docs.rs/yrs/) - Yrs CRDT library
- [CRDT Theory](https://crdt.tech/) - CRDT fundamentals

---

**CRDTs enable offline-first collaboration with automatic conflict resolution. 🔄✨**

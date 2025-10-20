# Automatic Offline Handling Architecture

## Philosophy

Communitas implements **transparent offline handling** - users never manually set "offline mode". The application automatically detects network failures and seamlessly transitions between online and offline states without user intervention.

### Design Principles

1. **No Manual Offline Mode** - Network state is automatically detected, not user-controlled
2. **Transparent Queueing** - Operations queue automatically when network is unavailable
3. **Automatic Synchronization** - Queued operations sync automatically when network returns
4. **User Confidence** - Operations always succeed from the user's perspective (either execute or queue)
5. **Offline-First** - All operations work offline; network is an optimization, not a requirement

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  (UI, Commands, Controllers)                                 │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ├─────> *_auto() methods
                       │        (Smart Operations Layer)
                       │
┌──────────────────────▼──────────────────────────────────────┐
│              Smart Operations Layer                          │
│   - create_entity_auto()                                     │
│   - send_message_auto()                                      │
│   - add_member_auto()                                        │
│                                                               │
│   Logic: Try immediate → If network error → Queue            │
└──────────┬───────────────────────────┬──────────────────────┘
           │                           │
           │ Network Available         │ Network Unavailable
           │                           │
┌──────────▼──────────────┐   ┌────────▼─────────────────────┐
│  Immediate Execution     │   │   Offline Queue              │
│  - create_entity()       │   │   - queue_create_entity()    │
│  - send_message()        │   │   - queue_send_message()     │
│  - add_entity_member()   │   │   - queue_add_member()       │
│                          │   │                              │
│  Uses: CoreContext       │   │   Persisted to disk          │
└──────────────────────────┘   └──────────────────────────────┘
                                         │
                                         │ Network Returns
                                         │
                                ┌────────▼────────────────────┐
                                │  Automatic Sync             │
                                │  sync_queued_operations()   │
                                │                             │
                                │  - Execute queued ops       │
                                │  - Detect duplicates        │
                                │  - Handle failures          │
                                │  - Clear successful ops     │
                                └─────────────────────────────┘
```

## Network Detection

### How We Detect Network State

Network availability is determined by **CoreContext state**:

- **Online**: `CoreContext` is initialized and available
- **Offline**: `CoreContext` is `None` (not initialized or failed)

### Network Error Detection

The `is_network_error()` function identifies network-related errors:

```rust
fn is_network_error(e: &anyhow::Error) -> bool {
    let error_str = e.to_string();

    // Primary indicator
    error_str.contains("CoreContext not initialized")
        // Fallback patterns
        || error_str.contains("CoreContext")
        || error_str.contains("network unavailable")
        || error_str.contains("connection failed")
}
```

**Non-network errors** (validation, authentication, etc.) are **not queued** - they propagate to the caller immediately.

## Smart Operations Layer

### Purpose

The smart operations layer provides methods that:
1. Try immediate execution when network is available
2. Automatically queue when network is unavailable
3. Return clear results indicating what happened

### API Design

```rust
pub enum EntityOrQueued {
    /// Operation executed immediately (network available)
    Executed(Entity),
    /// Operation queued for later sync (network unavailable)
    Queued(String), // operation_id
}

pub enum MessageOrQueued {
    /// Message sent immediately
    Sent(String), // message_id
    /// Message queued for later sync
    Queued(String), // operation_id
}

pub enum MemberOperationResult {
    /// Operation completed immediately
    Completed,
    /// Operation queued for later sync
    Queued(String), // operation_id
}
```

### Usage Example

```rust
use communitas_tui::backend::{Backend, EntityOrQueued};

// Try to create entity - automatic queueing if offline
let result = backend.create_entity_auto(
    "Team Channel".to_string(),
    EntityType::Channel,
    vec!["alice-test-one".to_string()],
).await?;

match result {
    EntityOrQueued::Executed(entity) => {
        // Network was available - operation completed
        println!("Created entity: {}", entity.id);
        ui.show_entity(entity);
    }
    EntityOrQueued::Queued(op_id) => {
        // Network unavailable - operation queued
        println!("Operation queued: {}", op_id);
        ui.show_notification("Will sync when online");
    }
}
```

## Offline Queue Management

### Queue Structure

```rust
pub struct OfflineQueue {
    /// Queued operations (sorted by priority and timestamp)
    queue: VecDeque<QueuedOperationEntry>,
    /// Maximum queue size (0 = unlimited)
    max_size: usize,
    /// Persistence file path
    persistence_path: PathBuf,
}

pub struct QueuedOperationEntry {
    /// Unique operation ID
    pub id: String,
    /// The operation to execute
    pub operation: QueuedOperation,
    /// Priority (higher = sync first)
    pub priority: u8,
    /// Timestamp when queued
    pub timestamp: DateTime<Utc>,
}
```

### Queue Operations

#### Enqueue
```rust
// Automatic (via smart operations)
backend.create_entity_auto(...).await?;

// Manual (when you know network is unavailable)
let op_id = backend.queue_create_entity(...).await?;
```

#### Priority
```rust
// High-priority operations sync first
backend.queue_send_message_with_priority(
    entity_id,
    entity_type,
    "Urgent message",
    priority: 10, // Higher = syncs first
).await?;
```

#### Size Limits
```rust
// Prevent unbounded queue growth
backend.set_queue_size_limit(1000).await?;

// Oldest operations dropped when limit reached
```

### Persistence

The queue automatically persists to disk:

```
~/.communitas-data/
  └── offline_queue/
      └── {four_words}/
          └── offline_queue.json
```

**Benefits:**
- Operations survive app crashes
- Operations survive app restarts
- Queued data not lost on device shutdown

## Synchronization

### Automatic Sync Triggers

Sync happens automatically when:
1. **Network Returns** - After `initialize_core_context()` succeeds
2. **Background Timer** (optional) - Periodic sync attempts
3. **Manual Trigger** - Application-specific events

### Sync Process

```rust
pub async fn sync_queued_operations(&mut self) -> Result<Vec<SyncResult>> {
    // 1. Get all queued operations
    let operations = self.offline_queue.get_all();

    // 2. Execute each operation
    for entry in operations {
        // Check for duplicates
        if queue.is_duplicate(&entry.operation) {
            results.push(SyncResult::Skipped { ... });
            continue;
        }

        // Execute operation
        match self.execute_queued_operation(entry.operation).await {
            Ok(_) => {
                results.push(SyncResult::Success { ... });
                successful_ops.push(entry.id);
            }
            Err(e) => {
                results.push(SyncResult::Failed { ... });
                // Keep failed operations for retry
            }
        }
    }

    // 3. Remove successful operations from queue
    for op_id in successful_ops {
        queue.remove(&op_id).await?;
    }

    Ok(results)
}
```

### Sync Results

```rust
pub enum SyncResult {
    /// Operation completed successfully
    Success { operation_id: String },

    /// Operation failed with error (kept in queue for retry)
    Failed { operation_id: String, error: String },

    /// Operation skipped (e.g., duplicate)
    Skipped { operation_id: String, reason: String },
}
```

### Duplicate Detection

The queue has built-in duplicate detection:

```rust
impl OfflineQueue {
    /// Detect if operation is duplicate
    pub fn is_duplicate(&self, operation: &QueuedOperation) -> bool {
        // Compare operation content (not ID)
        self.queue.iter().any(|entry| {
            // Same operation type and content = duplicate
            operations_match(&entry.operation, operation)
        })
    }
}
```

## Error Handling

### Error Categories

1. **Network Errors** → Automatically queued
   - "CoreContext not initialized"
   - Connection failures
   - Timeout errors

2. **Validation Errors** → Propagated to caller
   - Invalid entity names
   - Empty required fields
   - Type mismatches

3. **Authentication Errors** → Propagated to caller
   - Not logged in
   - Invalid credentials
   - Expired sessions

### Error Flow

```
Operation Called
      │
      ├─> Try Immediate Execution
      │
      ├─> Success? ──────────────> Return Success
      │
      ├─> Network Error? ────────> Queue & Return Queued
      │
      └─> Other Error? ──────────> Propagate Error
```

## Testing Strategy

### Unit Tests

Test individual components:

```rust
#[test]
fn test_network_error_detection() {
    let network_err = anyhow::anyhow!("CoreContext not initialized");
    assert!(is_network_error(&network_err));

    let validation_err = anyhow::anyhow!("Invalid name");
    assert!(!is_network_error(&validation_err));
}
```

### Integration Tests

Test automatic behavior:

```rust
#[tokio::test]
async fn test_auto_queue_when_network_unavailable() {
    let (mut backend, _temp) = create_test_backend().await?;

    // Simulate network failure
    backend.simulate_network_unavailable();

    // Operation should automatically queue
    let result = backend.create_entity_auto(...).await?;

    assert!(matches!(result, EntityOrQueued::Queued(_)));
}
```

### Test Helpers

```rust
impl Backend {
    /// Simulate network unavailable (for testing)
    ///
    /// **WARNING**: For testing only!
    pub fn simulate_network_unavailable(&mut self) {
        self.ctx = None;
        self.offline = true;
    }

    /// Simulate network available (for testing)
    pub fn simulate_network_available(&mut self) {
        self.offline = false;
        // Note: Must call initialize_core_context() separately
    }
}
```

## Implementation Guidelines

### For Application Developers

**✅ DO:**
- Use `*_auto()` methods in UI code
- Handle both `Executed` and `Queued` results
- Show appropriate feedback to users
- Trust the automatic queue

**❌ DON'T:**
- Manually set offline mode in production
- Bypass the smart operations layer
- Assume operations always execute immediately
- Clear the queue manually (except in special cases)

### For Backend Developers

**✅ DO:**
- Detect network errors accurately
- Persist queue to disk
- Handle duplicates during sync
- Keep failed operations for retry

**❌ DON'T:**
- Queue validation errors
- Queue authentication errors
- Lose queued operations on restart
- Execute duplicate operations

## Performance Considerations

### Queue Operations

- **Enqueue**: O(1) append, O(n) for priority insert
- **Dequeue**: O(1)
- **Size limit enforcement**: O(1) with VecDeque::pop_front()
- **Persistence**: Async I/O, happens after enqueue

### Sync Performance

- **Sequential execution**: One operation at a time
- **No blocking**: Async/await throughout
- **Batching**: Not implemented (could add for optimization)
- **Progress tracking**: Optional, via channels

### Scalability

- **Default queue size**: 1000 operations
- **Storage overhead**: ~1KB per queued operation
- **Recommended max**: 10,000 operations
- **Beyond that**: Consider batch operations or backend aggregation

## Future Enhancements

### Planned

1. **Background Auto-Sync** - Periodic sync attempts every 30s
2. **Conflict Resolution** - Handle concurrent edits
3. **Batch Operations** - Combine similar operations for efficiency
4. **Progress Events** - Real-time sync progress updates

### Under Consideration

1. **Operation Merging** - Combine redundant operations
2. **Selective Sync** - Priority-based partial sync
3. **Network Quality Detection** - Adapt behavior to connection quality
4. **Operation Cancellation** - Allow users to cancel queued operations

## Related Documentation

- [CRDT System](./crdt-system.md) - Conflict-free synchronization
- [CoreContext](./core-context.md) - Network layer architecture
- [Testing Guide](../testing/offline-testing.md) - How to test offline behavior

---

**Last Updated**: 2025-10-19
**Version**: 1.0.0
**Status**: Stable

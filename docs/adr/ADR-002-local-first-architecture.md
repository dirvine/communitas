# ADR-002: Local-First Architecture

## Status

Accepted (2025-12-24)

## Context

### The Problem

Traditional collaboration applications suffer from fundamental architectural issues:

- **Network dependency**: Cannot work without internet connection
- **Data ownership**: User data lives on company servers, subject to lock-in
- **Latency**: Every operation requires round-trip to remote server
- **Single point of failure**: Server downtime = application unusable
- **Privacy**: All data visible to service provider

Users need collaboration tools that work reliably in all conditions:
- Airplane mode, subway tunnels, rural areas
- Unstable connections, high latency networks
- Privacy-sensitive environments (healthcare, legal, journalism)
- Regions with censorship or connectivity restrictions

### Requirements

- All operations must work offline
- Data must sync automatically when connectivity returns
- User retains full ownership and control of data
- No central server required for basic functionality
- Conflicts resolved automatically without user intervention

## Decision

Adopt a **local-first architecture** where:

1. **All data stored locally first** - The device is the source of truth
2. **Operations work offline** - No network required for any operation
3. **Sync happens opportunistically** - When peers connect, data merges
4. **Conflicts resolve automatically** - Via CRDT (Conflict-free Replicated Data Types)

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Local-First Data Flow                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  User Action         Local Storage        P2P Sync        Peers     │
│                                                                     │
│  ┌──────────┐       ┌──────────────┐    ┌─────────┐    ┌─────────┐ │
│  │ Create   │       │              │    │         │    │  Peer   │ │
│  │ Message  │──────►│ libSQL Cache │───►│ Gossip  │───►│   A     │ │
│  └──────────┘       │              │    │ PubSub  │    └─────────┘ │
│       │             │ CRDT Doc     │    │         │    ┌─────────┐ │
│       │             │ (Yrs)        │    │ CRDT    │───►│  Peer   │ │
│       │             └──────────────┘    │ Updates │    │   B     │ │
│       │                   ▲             └─────────┘    └─────────┘ │
│       │                   │                  ▲                      │
│       │                   │                  │                      │
│       ▼                   │                  │                      │
│  ┌──────────┐            │              Incoming                   │
│  │ Instant  │            │              Updates                    │
│  │ UI       │◄───────────┘                                         │
│  │ Update   │                                                      │
│  └──────────┘                                                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Principles

**1. Write Locally, Sync Eventually**

```rust
// Operation completes immediately, regardless of network
async fn send_message(&self, content: &str) -> Result<MessageId> {
    // 1. Write to local CRDT document
    let msg_id = self.crdt_doc.insert_message(content)?;

    // 2. Persist to local storage
    self.storage.save_crdt(&self.crdt_doc)?;

    // 3. Queue for sync (non-blocking)
    self.sync_queue.push(CrdtUpdate::from(&self.crdt_doc));

    Ok(msg_id)  // Returns immediately, <50ms
}
```

**2. Optimistic UI Updates**

```typescript
// UI updates before network confirms
const sendMessage = async (content: string) => {
  // Show immediately with pending state
  addLocalMessage({ content, status: 'sending' });

  try {
    await invoke('send_message', { content });
    updateStatus('sent');
  } catch {
    updateStatus('failed');
  }
};
```

**3. CRDT Conflict Resolution**

All collaborative data uses CRDTs (see [ADR-003](ADR-003-yrs-crdt-synchronization.md)):

| Data Type | CRDT Structure | Conflict Strategy |
|-----------|----------------|-------------------|
| Messages | YArray | Append-only, timestamp ordering |
| Documents | YText | Character-by-character merge |
| Members | YMap | Last-write-wins by field |
| Tasks | YMap + YArray | Position-aware merge |

**4. Offline-First Storage**

```
┌──────────────────────────────────────────────────────────────────┐
│                    Storage Architecture                          │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │   Virtual   │    │    CRDT     │    │    SQL      │         │
│  │   Disks     │    │  Documents  │    │   Cache     │         │
│  │             │    │             │    │             │         │
│  │ - Private   │    │ - Yrs Docs  │    │ - Messages  │         │
│  │ - Public    │    │ - State Vec │    │ - Indexes   │         │
│  │ - Shared    │    │ - Updates   │    │ - Search    │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│         │                  │                  │                  │
│         ▼                  ▼                  ▼                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    libSQL Database                        │   │
│  │              (Local, embedded, encrypted)                 │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### Network Modes

The application operates in multiple modes based on connectivity:

| Mode | Description | Capabilities |
|------|-------------|--------------|
| **Connected** | Full P2P mesh | Real-time sync, presence, calls |
| **Limited** | Partial connectivity | Delayed sync, queued operations |
| **Local** | No network | Full functionality, queued sync |
| **Offline** | Airplane mode | Full functionality, manual sync later |

```rust
pub enum NetworkStatus {
    Connected,      // Full P2P connectivity
    Connecting,     // Establishing connections
    Limited,        // Partial connectivity
    Local,          // Local-only (no external network)
    Offline,        // Airplane mode / no network
}
```

### Sync Protocol

When peers reconnect, synchronization follows this protocol:

1. **State Vector Exchange**: Peers compare what updates they have
2. **Delta Computation**: Calculate minimal update set
3. **Update Transfer**: Exchange only missing updates
4. **Merge**: Apply updates via CRDT merge semantics
5. **Materialize**: Update SQL cache with merged state

```rust
// Sync on reconnect (simplified)
async fn sync_with_peer(&self, peer_id: &PeerId) -> Result<()> {
    // 1. Get our state vector
    let our_sv = self.crdt_doc.state_vector();

    // 2. Request peer's updates since our state
    let updates = peer.request_updates_since(&our_sv).await?;

    // 3. Apply updates (automatic conflict resolution)
    for update in updates {
        self.crdt_doc.apply_update(&update)?;
    }

    // 4. Send our updates they're missing
    let their_sv = peer.state_vector().await?;
    let our_updates = self.crdt_doc.encode_updates_since(&their_sv);
    peer.send_updates(our_updates).await?;

    Ok(())
}
```

### Performance Characteristics

| Operation | Latency (Local) | Latency (Networked) |
|-----------|-----------------|---------------------|
| Send message | <10ms | <10ms (async sync) |
| Read messages | <5ms | <5ms (from cache) |
| Search | <50ms | <50ms (local index) |
| File read | <100ms | <100ms (virtual disk) |
| Sync with peer | N/A | <2s (full state) |

## Consequences

### Benefits

- **Always available**: Works in airplane mode, tunnels, offline
- **Instant response**: No network round-trip for any operation
- **Data ownership**: Users control their data completely
- **Privacy**: Data never touches third-party servers
- **Resilience**: No single point of failure
- **Conflict-free**: Automatic merge without user intervention

### Trade-offs

- **Storage usage**: Data replicated on each device
- **Sync complexity**: CRDT implementation complexity
- **Eventual consistency**: Changes not instantly visible to all peers
- **Device loss**: Local data needs backup strategy

### Consistency Model

The system provides **eventual consistency** with strong session guarantees:

| Guarantee | Provided |
|-----------|----------|
| Read-your-writes | Yes (local write) |
| Monotonic reads | Yes (CRDT vector) |
| Monotonic writes | Yes (local-first) |
| Causal consistency | Yes (CRDT causality) |
| Strong consistency | No (by design) |

## Alternatives Considered

1. **Cloud-first with offline sync**: Store data in cloud, cache locally
   - Rejected: Creates dependency on cloud provider, privacy concerns

2. **Blockchain-based**: Distributed ledger for all operations
   - Rejected: Slow, expensive, not suitable for collaboration

3. **Centralized server**: Traditional client-server architecture
   - Rejected: Single point of failure, privacy concerns, vendor lock-in

4. **Federated**: Multiple servers, users choose provider
   - Rejected: Still requires server infrastructure, partial lock-in

5. **Peer-to-peer only**: No local storage, pure P2P
   - Rejected: Cannot work offline, data loss if all peers offline

## References

- CRDT Implementation: [ADR-003 Yrs CRDT Synchronization](ADR-003-yrs-crdt-synchronization.md)
- Storage: `communitas-core/src/storage/`
- CRDT Documents: `communitas-core/src/crdt/`
- Architecture: `docs/architecture/README.md`
- Local-First Software: [inkandswitch.com/local-first](https://www.inkandswitch.com/local-first/)

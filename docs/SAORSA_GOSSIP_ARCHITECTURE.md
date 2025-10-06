# Saorsa Gossip Architecture - Crate Interactions

## Overview

Saorsa Gossip is a layered peer-to-peer communication system built on top of ant-quic (QUIC transport). All crates are now at **v0.1.6** (except transport at v0.1.7) and work together to provide a complete gossip overlay network.

## Architecture Layers (Top-Down)

```
┌─────────────────────────────────────────────────────────┐
│         APPLICATION LAYER (Communitas)                  │
│  - Uses GossipContext for all P2P operations             │
│  - Sites, messaging, presence, CRDT sync                 │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│    SAORSA GOSSIP API (Coordinator)                      │
│  - saorsa-gossip-coordinator (v0.1.6)                   │
│  - Provides high-level API: GossipContext               │
│  - Orchestrates all subsystems                          │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│         FUNCTIONAL LAYERS                               │
│                                                         │
│  ┌──────────────────┐  ┌──────────────────┐           │
│  │ Identity         │  │ Groups           │           │
│  │ v0.1.6           │  │ v0.1.6           │           │
│  │ - Four-word IDs  │  │ - MLS support    │           │
│  │ - ML-DSA sigs    │  │ - Membership     │           │
│  └──────────────────┘  └──────────────────┘           │
│                                                         │
│  ┌──────────────────┐  ┌──────────────────┐           │
│  │ Presence         │  │ PubSub           │           │
│  │ v0.1.6           │  │ v0.1.6           │           │
│  │ - SWIM beacons   │  │ - Plumtree msgs  │           │
│  │ - Online status  │  │ - Entity topics  │           │
│  └──────────────────┘  └──────────────────┘           │
│                                                         │
│  ┌──────────────────┐  ┌──────────────────┐           │
│  │ CRDT Sync        │  │ Rendezvous       │           │
│  │ v0.1.6           │  │ v0.1.6           │           │
│  │ - GSet merging   │  │ - 65k shards     │           │
│  │ - Eventual sync  │  │ - DHT-free disc. │           │
│  └──────────────────┘  └──────────────────┘           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│    MEMBERSHIP & DISSEMINATION                           │
│  - saorsa-gossip-membership (v0.1.6)                    │
│  - HyParView: 8-12 active, 64-128 passive peers        │
│  - SWIM: <5s failure detection                          │
│  - Periodic shuffles heal partitions                    │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│         TRANSPORT LAYER                                 │
│  - saorsa-gossip-transport (v0.1.7)                     │
│  - GossipTransport trait                                │
│  - StreamType: Membership, PubSub, Bulk                 │
│  - Peer cache for bootstrap (NEW in v0.1.7!)           │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│         NETWORK LAYER                                   │
│  - ant-quic (v0.8.17)                                   │
│  - QUIC connections with NAT traversal                  │
│  - Connection migration & path switching                │
└─────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────┐
│    CRYPTOGRAPHY LAYER                                   │
│  - saorsa-pqc (v0.3.12)                                 │
│  - ML-KEM-768 (key exchange)                            │
│  - ML-DSA-65 (signatures)                               │
│  - ChaCha20-Poly1305 (encryption)                       │
└─────────────────────────────────────────────────────────┘
```

## Crate-by-Crate Interactions

### 1. saorsa-gossip-types (v0.1.6)
**Purpose**: Common types used across all crates

**Provides**:
- `PeerId` - Unique peer identifier
- `MessageId` - Message deduplication
- `EntityId` - Entity (channel/project/org) identifier
- `StreamType` - Message routing hint

**Used by**: All other crates

### 2. saorsa-gossip-identity (v0.1.6)
**Purpose**: Identity management with four-word addressing

**Provides**:
- Four-word address ↔ cryptographic identity
- ML-DSA signature creation/verification
- Identity persistence

**Uses**:
- `saorsa-gossip-types` for `PeerId`
- `saorsa-pqc` for ML-DSA signatures
- `four-word-networking` for address encoding

**Used by**:
- `coordinator` - Creates GossipContext identity
- `groups` - Signs group operations
- `presence` - Signs presence beacons

### 3. saorsa-gossip-transport (v0.1.7) ⭐ NEW FEATURES
**Purpose**: Transport abstraction over ant-quic

**Provides**:
- `GossipTransport` trait
- Stream multiplexing (Membership, PubSub, Bulk)
- **Peer cache** for offline/bootstrap (NEW!)
- Connection management

**Key Feature - Peer Cache**:
```rust
pub trait GossipTransport {
    // NEW: Peer cache for bootstrap
    async fn get_cached_peers(&self) -> Result<Vec<CachedPeer>>;
    async fn add_cached_peer(&self, peer: CachedPeer) -> Result<()>;

    // Existing: Connection management
    async fn send_to_peer(&self, peer: PeerId, stream: StreamType, data: Bytes) -> Result<()>;
    async fn receive_message(&self) -> Result<(PeerId, StreamType, Bytes)>;
}

pub struct CachedPeer {
    pub four_words: String,
    pub peer_id: PeerId,
    pub last_seen: u64,
    pub success_count: u32,
    pub fail_count: u32,
}
```

**Uses**:
- `ant-quic` for QUIC connections
- `saorsa-gossip-types` for message types

**Used by**: All higher-level crates for P2P communication

### 4. saorsa-gossip-membership (v0.1.6)
**Purpose**: Network overlay maintenance

**Provides**:
- HyParView partial view management
- SWIM failure detection
- Peer scoring and mesh quality
- Periodic shuffle protocol

**Architecture**:
- **Active view**: 8-12 peers for regular communication
- **Passive view**: 64-128 peers for resilience
- **Shuffle**: Every 60s to heal partitions
- **Failure detection**: <5s via SWIM pings

**Uses**:
- `transport` for peer-to-peer messaging
- `types` for `PeerId`

**Used by**:
- `coordinator` - Maintains network connections
- `pubsub` - Uses active view for message routing

### 5. saorsa-gossip-pubsub (v0.1.6)
**Purpose**: Topic-based publish/subscribe

**Provides**:
- Plumtree eager/lazy push protocol
- Entity-based topic routing
- Message deduplication
- Tree-based gossip dissemination

**Protocol**:
1. Subscribe to entity → join topic
2. Publish message → eager push to active peers
3. Receive duplicate → send PRUNE, add to lazy view
4. Miss message → send GRAFT, move to eager view

**Uses**:
- `transport` for message delivery
- `membership` for active/passive peer views
- `types` for `EntityId` and `MessageId`

**Used by**:
- `coordinator` - Publishes/subscribes to entities
- Applications - Channel messages, project updates

### 6. saorsa-gossip-presence (v0.1.6)
**Purpose**: Online/offline status tracking

**Provides**:
- SWIM-style presence beacons (5min interval)
- Peer online status queries
- Entity-specific presence (who's in this channel?)
- Failure detection integration

**Protocol**:
1. Every 5min: broadcast signed beacon to all entities
2. On receive: update last_seen timestamp
3. Query: check if last_seen < 10min ago

**Uses**:
- `transport` for beacon broadcast
- `identity` for beacon signing
- `membership` for peer list

**Used by**:
- `coordinator` - Starts/stops presence beacons
- Applications - Show online users

### 7. saorsa-gossip-groups (v0.1.6)
**Purpose**: Group identity and membership (MLS ready)

**Provides**:
- Group creation and join
- Member add/remove operations
- Group key management
- Signature verification

**Future**: Will integrate with `saorsa-mls` for encrypted group messaging

**Uses**:
- `identity` for member signatures
- `types` for group identifiers

**Used by**:
- `coordinator` - Creates channels/projects/orgs
- Applications - Group management

### 8. saorsa-gossip-crdt-sync (v0.1.6)
**Purpose**: Conflict-free data replication

**Provides**:
- GSet (Grow-only Set) for message storage
- Automatic conflict resolution
- Merge-based synchronization
- Eventual consistency guarantee

**Protocol**:
1. Store message → add to local GSet
2. Periodic sync → merge with peer's GSet
3. Conflict-free: union of both sets

**Uses**:
- `transport` for sync messages
- `types` for message identifiers

**Used by**:
- `coordinator` - Stores/syncs messages
- Applications - Offline message queue

### 9. saorsa-gossip-rendezvous (v0.1.6)
**Purpose**: DHT-free content discovery

**Provides**:
- 65,536 shards (16-bit shard space)
- Provider announcements
- Content discovery without DHT
- Shard subscription

**Protocol**:
1. Publisher: hash(content) → shard_id
2. Announce: publish to shard_id with PeerId
3. Seeker: hash(content) → shard_id
4. Query: get providers from shard_id

**Uses**:
- `transport` for shard messages
- `pubsub` for shard subscriptions

**Used by**:
- `coordinator` - Site discovery (Saorsa Sites)
- Applications - Content-addressed lookups

### 10. saorsa-gossip-coordinator (v0.1.6)
**Purpose**: High-level API orchestration

**Provides**:
- `GossipContext` - Single entry point for all operations
- Lifecycle management (init, start, stop)
- Integrated error handling
- Simplified application interface

**Key Methods**:
```rust
impl GossipContext {
    // Identity
    async fn initialize(four_words, name, device) -> Result<Self>;

    // Contacts
    async fn find_contact(four_words) -> Result<PeerId>;
    async fn add_contact(four_words, peer_id) -> Result<()>;

    // Messaging
    async fn send_direct_message(peer, msg) -> Result<()>;
    async fn publish_to_entity(entity_id, msg) -> Result<()>;

    // Groups
    async fn join_entity(entity_id, type) -> Result<()>;
    async fn leave_entity(entity_id) -> Result<()>;

    // Presence
    async fn start_presence_beacons() -> Result<()>;
    async fn is_peer_online(peer) -> Result<bool>;

    // Storage
    async fn store_message(msg) -> Result<()>;
    async fn get_all_messages() -> Result<Vec<u8>>;

    // NEW: Bootstrap via peer cache
    async fn add_bootstrap_peer(four_words) -> Result<()>;
    async fn get_cached_peers() -> Result<Vec<CachedPeer>>;
}
```

**Uses**: All other saorsa-gossip crates

**Used by**: Applications (Communitas via Tauri commands)

## Data Flow Examples

### Example 1: Sending a Message

```
Application (Tauri)
    ↓ gossip_send_direct_message(four_words, msg)
Coordinator
    ↓ find_contact(four_words) → PeerId
Identity + Rendezvous
    ↓ send_direct_message(peer_id, msg)
PubSub
    ↓ eager_push(msg) to active peers
Membership (active view)
    ↓ send_to_peer(peer_id, StreamType::PubSub, msg)
Transport
    ↓ QUIC connection to peer
ant-quic
```

### Example 2: Bootstrapping via Friend (NEW!)

```
Application (Tauri)
    ↓ gossip_add_bootstrap_peer(friend_four_words)
Coordinator
    ↓ add_cached_peer(friend_peer)
Transport (peer cache)
    ↓ persist to disk
---
Application restart (offline)
    ↓ gossip_initialize()
Coordinator
    ↓ get_cached_peers()
Transport (peer cache)
    ↓ return [friend_peer]
Membership
    ↓ connect_to_peer(friend_peer)
ant-quic (NAT traversal)
    ↓ QUIC connection established
Membership
    ↓ request peer list from friend
    ↓ expand network via HyParView shuffle
```

### Example 3: Publishing a Site

```
Application (Tauri)
    ↓ gossip_site_publish(assets)
Coordinator → SitePublisher
    ↓ build_manifest() → site_id
    ↓ hash(site_id) → shard_id
Rendezvous
    ↓ publish_to_shard(shard_id, my_peer_id)
PubSub
    ↓ subscribe_to_shard(shard_id)
Transport
    ↓ announce via StreamType::PubSub
ant-quic
---
Fetcher (another peer)
    ↓ gossip_site_fetch(site_id_hex)
Rendezvous
    ↓ query_shard(shard_id) → [provider_peer_ids]
Transport
    ↓ send_to_peer(provider, GetManifest{site_id})
Transport (StreamType::Bulk)
    ↓ receive manifest + blocks
BLAKE3 verification
    ↓ return SiteData
```

## Key Design Principles

### 1. **Decentralization**
- No central servers or infrastructure
- Peer-to-peer with NAT traversal
- Rendezvous replaces DHT (65k shards)

### 2. **Privacy-Preserving**
- End-to-end encryption (ChaCha20-Poly1305)
- Post-quantum signatures (ML-DSA)
- No metadata leakage

### 3. **Local-First**
- CRDT-based storage (eventual consistency)
- Offline operation with sync-on-reconnect
- Peer cache for bootstrap without online discovery

### 4. **Resilience**
- HyParView heals partitions (shuffle every 60s)
- SWIM detects failures (<5s)
- Plumtree routes around failures (GRAFT/PRUNE)

### 5. **Scalability**
- Partial views (8-12 active, 64-128 passive)
- Shard-based discovery (65k shards)
- Tree-based gossip (log(N) hops)

## Peer Cache Feature (v0.1.7)

The new peer cache in `saorsa-gossip-transport` enables:

1. **Offline Bootstrap**: Add friend's four-word address while offline
2. **Persistent Cache**: Survives app restarts
3. **Success Tracking**: Scores peers by connection success rate
4. **Automatic Fallback**: Try cached peers if discovery fails

**Usage**:
```rust
// Add friend for bootstrap
let peer = CachedPeer {
    four_words: "ocean-forest-moon-star".to_string(),
    peer_id: friend_peer_id,
    last_seen: now(),
    success_count: 0,
    fail_count: 0,
};
transport.add_cached_peer(peer).await?;

// Later (even after restart/offline)
let cached = transport.get_cached_peers().await?;
for peer in cached {
    membership.try_connect(peer.peer_id).await?;
}
```

## Integration in Communitas

Communitas uses all layers via `GossipContext`:

```rust
// communitas-core/src/gossip/context.rs
pub struct GossipContext {
    pub identity: Arc<Identity>,              // identity crate
    pub transport: Arc<RwLock<Box<dyn GossipTransport>>>, // transport crate
    pub membership: Arc<Membership>,          // membership crate
    pub pubsub: Arc<PubSub>,                  // pubsub crate
    pub presence: Arc<Presence>,              // presence crate
    pub crdt: Arc<CrdtSync>,                  // crdt-sync crate
    pub groups: Arc<Groups>,                  // groups crate
    pub rendezvous: Arc<RendezvousClient>,    // rendezvous crate

    // Higher-level features
    pub site_publisher: Option<Arc<SitePublisher>>,
    pub site_fetcher: Option<Arc<SiteFetcher>>,
}
```

## Next Steps

With all crates updated to v0.1.6-0.1.7, we can now:

1. ✅ Use peer cache for offline bootstrap
2. ✅ Add connection status UI in sidebar
3. ✅ Display user's four-word identity
4. ✅ Allow adding friend's four-word for bootstrap
5. ✅ Show online/offline status with cached peer fallback

---

**Updated**: 2025-10-06
**Crate Versions**: All v0.1.6 (transport v0.1.7)
**Key Feature**: Peer cache for offline bootstrap

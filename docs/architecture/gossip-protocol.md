# Gossip Protocol Architecture

Comprehensive guide to Communitas' P2P networking layer built on the Saorsa Gossip ecosystem.

## Overview

Communitas uses **Saorsa Gossip** - a layered peer-to-peer communication system built on QUIC transport. It provides decentralized messaging, peer discovery, presence tracking, and content distribution without central servers.

**Terminology**:
- **Identity**: hex-encoded ML-DSA public key (pubkey_hex)
- **Connection words**: four-word networking encoding of IP:port for peer dialing

### Key Features

- **Decentralized**: No central servers or single points of failure
- **Privacy-Preserving**: End-to-end encryption with post-quantum signatures
- **Local-First**: Offline operation with sync-on-reconnect
- **Resilient**: Automatic partition healing and failure detection (<5s)
- **Scalable**: Tree-based gossip with logarithmic message propagation

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│         APPLICATION LAYER (Communitas)                      │
│  - Uses GossipContext for all P2P operations                 │
│  - Sites, messaging, presence, CRDT sync                     │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│    SAORSA GOSSIP API (Coordinator)                          │
│  - saorsa-gossip-coordinator (v0.1.6)                       │
│  - Provides high-level API: GossipContext                   │
│  - Orchestrates all subsystems                              │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│         FUNCTIONAL LAYERS                                   │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Identity         │  │ Groups           │               │
│  │ v0.1.6           │  │ v0.1.6           │               │
│  │ - Public keys    │  │ - MLS support    │               │
│  │ - ML-DSA sigs    │  │ - Membership     │               │
│  └──────────────────┘  └──────────────────┘               │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ Presence         │  │ PubSub           │               │
│  │ v0.1.6           │  │ v0.1.6           │               │
│  │ - SWIM beacons   │  │ - Plumtree msgs  │               │
│  │ - Online status  │  │ - Entity topics  │               │
│  └──────────────────┘  └──────────────────┘               │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │ CRDT Sync        │  │ Rendezvous       │               │
│  │ v0.1.6           │  │ v0.1.6           │               │
│  │ - GSet merging   │  │ - 65k shards     │               │
│  │ - Eventual sync  │  │ - DHT-free disc. │               │
│  └──────────────────┘  └──────────────────┘               │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│    MEMBERSHIP & DISSEMINATION                               │
│  - saorsa-gossip-membership (v0.1.6)                        │
│  - HyParView: 8-12 active, 64-128 passive peers            │
│  - SWIM: <5s failure detection                              │
│  - Periodic shuffles heal partitions                        │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│         TRANSPORT LAYER                                     │
│  - saorsa-gossip-transport (v0.1.7)                         │
│  - GossipTransport trait                                    │
│  - StreamType: Membership, PubSub, Bulk                     │
│  - Peer cache for bootstrap (NEW in v0.1.7!)               │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│         NETWORK LAYER                                       │
│  - ant-quic (v0.8.17)                                       │
│  - QUIC connections with NAT traversal                      │
│  - Connection migration & path switching                    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│    CRYPTOGRAPHY LAYER                                       │
│  - saorsa-pqc (v0.3.12)                                     │
│  - ML-KEM-768 (key exchange)                                │
│  - ML-DSA-65 (signatures)                                   │
│  - ChaCha20-Poly1305 (encryption)                           │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Coordinator (GossipContext)

**Purpose**: High-level API orchestration - single entry point for all gossip operations

**Location**: `saorsa-gossip-coordinator` (v0.1.6)

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

    // Bootstrap via peer cache (NEW in v0.1.7)
    async fn add_bootstrap_peer(four_words) -> Result<()>;
    async fn get_cached_peers() -> Result<Vec<CachedPeer>>;
}
```

**Integration**:
```rust
// communitas-core/src/gossip/context.rs
pub struct GossipContext {
    pub identity: Arc<Identity>,
    pub transport: Arc<RwLock<Box<dyn GossipTransport>>>,
    pub membership: Arc<Membership>,
    pub pubsub: Arc<PubSub>,
    pub presence: Arc<Presence>,
    pub crdt: Arc<CrdtSync>,
    pub groups: Arc<Groups>,
    pub rendezvous: Arc<RendezvousClient>,

    // Higher-level features
    pub site_publisher: Option<Arc<SitePublisher>>,
    pub site_fetcher: Option<Arc<SiteFetcher>>,
}
```

### 2. Identity Management

**Purpose**: Public-key identity management with post-quantum signatures

**Location**: `saorsa-gossip-identity` (v0.1.6)

**Features**:
- Public-key identity management
- ML-DSA signature creation/verification
- Identity persistence

**Example**:
```rust
use saorsa_gossip_identity::Identity;

// Create identity from public key (pseudocode)
let identity = Identity::from_public_key(pubkey)?;

// Sign message
let signature = identity.sign(message_bytes)?;

// Verify signature
identity.verify(message_bytes, &signature)?;
```

### 3. Transport Layer

**Purpose**: Transport abstraction over ant-quic with stream multiplexing

**Location**: `saorsa-gossip-transport` (v0.1.7)

**Features**:
- `GossipTransport` trait
- Stream multiplexing (Membership, PubSub, Bulk)
- **Peer cache** for offline/bootstrap (NEW!)
- Connection management

**Stream Types**:
- `Membership`: HyParView protocol messages
- `PubSub`: Plumtree gossip messages
- `Bulk`: Large content transfer (site assets)

**Peer Cache** (v0.1.7):
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

## Membership Management (HyParView)

### Overview

**HyParView** maintains a **partial view** of the network to ensure connectivity while limiting overhead.

**Architecture**:
- **Active view**: 8-12 peers for regular communication
- **Passive view**: 64-128 peers for resilience
- **Shuffle**: Every 60s to heal partitions
- **Failure detection**: <5s via SWIM pings

### Active vs Passive Views

**Active View** (8-12 peers):
- Direct QUIC connections maintained
- Used for message dissemination
- Low latency communication
- Regularly pinged for liveness

**Passive View** (64-128 peers):
- No active connections
- Backup peers for resilience
- Promoted to active when needed
- Exchanged via shuffle protocol

### Shuffle Protocol

**Purpose**: Heal network partitions and maintain connectivity

**Frequency**: Every 60 seconds

**Protocol**:
```
1. Select random peer P from active view
2. Send SHUFFLE(my_passive_view_sample) to P
3. P replies with SHUFFLE_REPLY(their_passive_view_sample)
4. Merge received peers into passive view
5. Remove duplicates and maintain size limits
```

**Benefits**:
- Automatic partition healing
- Network graph remains connected
- Adapts to node churn

### Peer Scoring

**Metrics tracked per peer**:
- Connection success rate
- Message latency
- Bandwidth utilization
- Uptime percentage

**Scoring algorithm**:
```rust
score = (success_rate * 0.4) +
        (uptime * 0.3) +
        (low_latency_score * 0.2) +
        (bandwidth_availability * 0.1)
```

**Usage**: Prefer high-scoring peers for active view

## Message Dissemination (Plumtree)

### Overview

**Plumtree** provides efficient tree-based message dissemination with automatic recovery from failures.

### Protocol Phases

**1. Eager Push** (Tree Phase):
```
Source publishes message
    ↓
Eager push to all neighbors in tree
    ↓
Neighbors eager push to their neighbors
    ↓
Log(N) hops to reach entire network
```

**2. Lazy Push** (Optimization):
```
Receive duplicate message
    ↓
Send PRUNE to sender
    ↓
Sender moves receiver to lazy push list
    ↓
Future messages sent as IHAVE (lazy)
```

**3. GRAFT** (Recovery):
```
Miss message (gap detected)
    ↓
Send GRAFT to source
    ↓
Source moves receiver back to eager push
    ↓
Future messages sent eagerly again
```

### Topic-Based Routing

**Entity Topics**:
```rust
// Subscribe to entity
pubsub.subscribe(entity_id, message_handler).await?;

// Publish to entity
pubsub.publish(entity_id, message_bytes).await?;
```

**Topic Types**:
- `entity:{four_words}` - Entity-specific messages
- `channel:{channel_id}` - Channel messages
- `project:{project_id}` - Project updates
- `presence:all` - Global presence beacons

### Message Deduplication

**Strategy**: Bloom filter + message ID cache

**Implementation**:
```rust
// Track seen messages
let mut seen_messages: HashSet<MessageId> = HashSet::new();

// On receive
if seen_messages.contains(&msg_id) {
    // Duplicate - send PRUNE
    pubsub.send_prune(sender_peer).await?;
} else {
    // New message - process and forward
    seen_messages.insert(msg_id);
    process_message(msg).await?;
    pubsub.eager_push(msg, neighbors).await?;
}
```

## Failure Detection (SWIM)

### Protocol Overview

**SWIM** (Scalable Weakly-consistent Infection-style Process Group Membership) provides fast failure detection.

**Detection time**: <5 seconds

### SWIM States

**Alive**: Peer responding to pings
**Suspected**: Peer not responding, asking others
**Failed**: Confirmed dead by multiple peers

### Ping Protocol

```
Every 1 second:
    Select random peer P
    Send PING to P
    Wait 500ms for ACK

    If no ACK:
        Select k random peers
        Send PING-REQ(P) to each
        Wait 500ms for indirect ACK

        If still no ACK:
            Mark P as SUSPECTED
            Broadcast SUSPECT(P)

            After 3 seconds:
                If no ALIVE(P) received:
                    Mark P as FAILED
                    Remove from active view
                    Promote peer from passive view
```

### Piggybacking

**Efficiency optimization**: Piggyback membership updates on regular messages

```rust
struct GossipMessage {
    payload: Vec<u8>,
    updates: Vec<MembershipUpdate>,  // Piggybacked updates
}

enum MembershipUpdate {
    Alive(PeerId, incarnation),
    Suspected(PeerId, incarnation),
    Failed(PeerId),
}
```

## Presence Tracking

### Beacon Protocol

**Frequency**: Every 5 minutes

**Protocol**:
```
Every 5 minutes:
    Create beacon = {
        peer_id,
        entities: [entity_ids I'm subscribed to],
        timestamp,
        signature
    }

    For each entity in entities:
        Publish beacon to entity topic

On receive beacon:
    Verify signature
    Update last_seen[peer_id] = timestamp
```

### Online Status Query

```rust
pub async fn is_peer_online(peer_id: &PeerId) -> Result<bool> {
    let last_seen = presence.get_last_seen(peer_id).await?;
    let now = Utc::now().timestamp();

    // Online if beacon received in last 10 minutes
    Ok(now - last_seen < 600)
}
```

### Entity Presence

**Query who's in a channel**:
```rust
pub async fn get_entity_members(entity_id: &str) -> Result<Vec<PeerId>> {
    let beacons = presence.get_entity_beacons(entity_id).await?;
    let now = Utc::now().timestamp();

    // Filter to online peers (last 10min)
    Ok(beacons.into_iter()
        .filter(|b| now - b.timestamp < 600)
        .map(|b| b.peer_id)
        .collect())
}
```

## Rendezvous (DHT-Free Discovery)

### Overview

**Rendezvous** provides content discovery without a DHT using 65,536 shards.

**Advantages over DHT**:
- No routing tables to maintain
- Simpler protocol
- Better privacy (no node IDs exposed)
- Faster lookups (1 hop)

### Shard Space

**Shard Count**: 65,536 (16-bit shard space)

**Shard Assignment**:
```rust
let shard_id = blake3::hash(content_id).as_bytes()[0..2];
let shard_id = u16::from_be_bytes([shard_id[0], shard_id[1]]);
```

**Distribution**: Uniform via BLAKE3 hash

### Provider Announcements

**Publish content**:
```rust
// Publisher has content
let content_id = blake3::hash(&content);
let shard_id = compute_shard(content_id);

rendezvous.announce_provider(shard_id, my_peer_id).await?;
```

**Find providers**:
```rust
// Seeker wants content
let content_id = blake3::hash(&content_description);
let shard_id = compute_shard(content_id);

let providers = rendezvous.query_shard(shard_id).await?;
```

### Shard Subscription

**Subscribe to shard**:
```rust
// Listen for announcements in shard
pubsub.subscribe(&format!("shard:{}", shard_id), handler).await?;
```

**Announce to shard**:
```rust
// Publish announcement to shard subscribers
let announcement = Announcement {
    shard_id,
    content_id,
    provider_peer_id: my_peer_id,
    timestamp: Utc::now().timestamp(),
};

pubsub.publish(&format!("shard:{}", shard_id), announcement).await?;
```

## Peer Cache & Bootstrap

### Offline Bootstrap (v0.1.7)

**Problem**: How to bootstrap without online discovery?

**Solution**: Peer cache with friend's four-word address

### Cache Management

**Add friend for bootstrap**:
```rust
let peer = CachedPeer {
    four_words: "ocean-forest-moon-star".to_string(),
    peer_id: friend_peer_id,
    last_seen: Utc::now().timestamp(),
    success_count: 0,
    fail_count: 0,
};

transport.add_cached_peer(peer).await?;
```

**Bootstrap from cache**:
```rust
// On startup (even offline)
let cached = transport.get_cached_peers().await?;

// Sort by success rate
cached.sort_by(|a, b| {
    let score_a = a.success_count as f64 / (a.success_count + a.fail_count) as f64;
    let score_b = b.success_count as f64 / (b.success_count + b.fail_count) as f64;
    score_b.partial_cmp(&score_a).unwrap()
});

// Try to connect
for peer in cached.take(5) {
    if let Ok(_) = membership.try_connect(peer.peer_id).await {
        // Success - update cache
        transport.update_peer_success(peer.peer_id).await?;
        break;
    } else {
        // Failure - update cache
        transport.update_peer_failure(peer.peer_id).await?;
    }
}
```

### Cache Persistence

**Storage**: Local filesystem (libSQL materialization planned)

**Schema**:
```sql
CREATE TABLE cached_peers (
    peer_id TEXT PRIMARY KEY,
    four_words TEXT NOT NULL,
    last_seen INTEGER,
    success_count INTEGER,
    fail_count INTEGER
);
```

## Data Flow Examples

### Example 1: Sending a Message

```
Application (GUI/FFI)
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

### Example 2: Bootstrapping via Friend

```
Application (GUI/FFI)
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
Application (GUI/FFI)
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

## Performance Characteristics

### Network Overhead

| Component | Bandwidth (per peer) | Frequency |
|-----------|---------------------|-----------|
| SWIM pings | ~100 bytes | 1/second |
| Presence beacons | ~500 bytes | 5 minutes |
| HyParView shuffle | ~1-2KB | 60 seconds |
| Message overhead | ~200 bytes | Per message |

**Total steady-state**: ~10-20 KB/s per peer

### Message Latency

| Scenario | Latency | Description |
|----------|---------|-------------|
| Direct peer | <50ms | Single QUIC hop |
| LAN peers | <100ms | Local network |
| WAN peers | <500ms | Internet routing |
| Gossip propagation | <2s | 99th percentile full network |

### Scalability

**Network Size**: Tested to 10,000 nodes
**Active Connections**: 8-12 per node (constant)
**Message Complexity**: O(log N) hops
**Memory per Peer**: ~1KB for active, ~100 bytes for passive

### Failure Detection

**Detection Time**: <5 seconds (SWIM protocol)
**False Positive Rate**: <1% (via indirect pings)
**Recovery Time**: <10 seconds (via GRAFT)

## Design Principles

### 1. Decentralization

- No central servers or infrastructure
- Peer-to-peer with NAT traversal
- Rendezvous replaces DHT (65k shards)

### 2. Privacy-Preserving

- End-to-end encryption (ChaCha20-Poly1305)
- Post-quantum signatures (ML-DSA)
- No metadata leakage

### 3. Local-First

- CRDT-based storage (eventual consistency)
- Offline operation with sync-on-reconnect
- Peer cache for bootstrap without online discovery

### 4. Resilience

- HyParView heals partitions (shuffle every 60s)
- SWIM detects failures (<5s)
- Plumtree routes around failures (GRAFT/PRUNE)

### 5. Scalability

- Partial views (8-12 active, 64-128 passive)
- Shard-based discovery (65k shards)
- Tree-based gossip (log(N) hops)

## Integration with Communitas

### Backend Commands

```rust
// communitas-desktop/src/core_cmds.rs

// FFI boundary
pub async fn gossip_initialize(
    four_words: String,
    display_name: String,
    device_name: String
) -> Result<(), String>

// FFI boundary
pub async fn gossip_send_message(
    recipient: String,
    message: String
) -> Result<(), String>

// FFI boundary
pub async fn gossip_add_bootstrap_peer(
    four_words: String
) -> Result<(), String>

// FFI boundary
pub async fn gossip_get_online_peers() -> Result<Vec<PeerInfo>, String>
```

### Frontend Integration (Flutter FFI)

```dart
final api = await CommunitasApi.create(
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Alice',
  deviceName: 'MacBook Pro',
  storagePath: '/path/to/storage',
);

await api.gossipStart();
await api.gossipConnectToPeer(fourWords: 'bob-charlie-delta-echo');
```

## See Also

- [Architecture Overview](README.md) - System architecture
- [Architecture Overview](README.md) - Component details
- [CRDT System](crdt-system.md) - Data synchronization
- [Networking](networking.md) - Network protocols
- [Security](security.md) - Security model

---

**Decentralized P2P networking with automatic failure recovery and offline support. 🌐🔄**

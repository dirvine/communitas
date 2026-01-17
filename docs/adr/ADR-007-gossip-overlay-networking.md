# ADR-007: Gossip Overlay Networking

## Status

Accepted (2025-12-24)

## Context

### The Problem

Decentralized applications need to communicate without central servers, facing challenges:

- **Traditional client-server**: Single point of failure, vendor lock-in
- **Full mesh**: O(n²) connections, doesn't scale
- **DHT-only**: High latency, complex routing
- **Blockchain**: Expensive, slow, overkill for messaging

Communitas needs P2P networking that:
- Scales to thousands of nodes
- Delivers messages reliably
- Handles peer churn (joins/leaves)
- Works across NAT/firewalls
- Requires no central infrastructure

### Requirements

- Epidemic message dissemination
- Efficient membership management
- Fast failure detection
- DHT-free peer discovery
- NAT traversal support

## Decision

Adopt the **saorsa-gossip** protocol stack built on `ant-quic` for P2P networking:

### Protocol Stack

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Gossip Protocol Stack                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Application Layer                                                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ CRDT Sync │ Messaging │ Presence │ Discovery                 │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  Gossip Overlay                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │  HyParView   │  │   Plumtree   │  │    SWIM      │             │
│  │ (Membership) │  │  (PubSub)    │  │  (Failure)   │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
│                              │                                      │
│                              ▼                                      │
│  Rendezvous (65k shards)                                           │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Peer Discovery │ Name Registration │ Shard Routing           │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  Transport Layer                                                    │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ ant-quic │ QUIC Multiplexing │ NAT Traversal │ PQC TLS       │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Components

**1. HyParView (Membership)**

Maintains a partial view of the network:

| View | Size | Purpose |
|------|------|---------|
| Active | 8-12 peers | Direct message exchange |
| Passive | 64-128 peers | Backup, recovery |

```rust
struct HyParView {
    active_view: HashSet<PeerId>,   // Current neighbors
    passive_view: HashSet<PeerId>,  // Known but not connected
    active_max: usize,              // 8-12 typical
    passive_max: usize,             // 64-128 typical
}
```

**Shuffle protocol**: Periodically exchange random subsets to heal partitions.

**2. Plumtree (Message Dissemination)**

Hybrid push-pull gossip with tree overlay:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Plumtree Operation                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Eager Push (Tree)                    Lazy Push (Backup)           │
│  ─────────────────                    ──────────────────           │
│                                                                     │
│       ┌───┐                           Send IHAVE to                 │
│       │ A │ ──── Message ─────►       lazy peers                   │
│       └───┘                                                         │
│      /     \                          If not received              │
│     ▼       ▼                         via tree, request            │
│  ┌───┐   ┌───┐                        via GRAFT                    │
│  │ B │   │ C │                                                      │
│  └───┘   └───┘                        Tree repairs                 │
│     │       │                         automatically                │
│     ▼       ▼                                                       │
│  ┌───┐   ┌───┐                                                     │
│  │ D │   │ E │                                                     │
│  └───┘   └───┘                                                     │
│                                                                     │
│  O(log n) hops, O(n) messages total                                │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**3. SWIM (Failure Detection)**

Scalable failure detection with indirect pings:

| Phase | Action | Timeout |
|-------|--------|---------|
| Direct | Ping target | 1s |
| Indirect | Ask k peers to ping | 2s |
| Suspect | Broadcast suspicion | 3s |
| Faulty | Remove from membership | 5s |

**4. Rendezvous (DHT-Free Discovery)**

65,536 shards for peer discovery without full DHT:

```rust
// Hash public-key identity to shard
fn shard_for_identity(identity: &str) -> u16 {
    let hash = blake3::hash(identity.as_bytes());
    u16::from_be_bytes([hash.as_bytes()[0], hash.as_bytes()[1]])
}

// Find peers responsible for shard
async fn find_peers_for_shard(shard: u16) -> Vec<PeerId>;
```

### Topic-Based PubSub

Messages are published to topics, not global broadcast:

```rust
// Entity-specific topics
let topic = format!("entity:{}", entity_four_words);

// Subscribe to entity updates
gossip.subscribe(&topic, |message| {
    // Handle CRDT update, chat message, etc.
});

// Publish to entity members only
gossip.publish(&topic, crdt_update).await?;
```

### Bootstrap Process

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Bootstrap Flow                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. Load known peers from cache                                    │
│     ┌──────────────────────────────────────────────────────────┐   │
│     │ peer_cache.json: ["ocean-forest..", "river-mountain.."]  │   │
│     └──────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  2. Connect to bootstrap nodes (if cache empty)                    │
│     ┌──────────────────────────────────────────────────────────┐   │
│     │ config/production-network.toml: bootstrap_peers          │   │
│     └──────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  3. Join HyParView overlay                                         │
│     - Send JOIN to random peer                                     │
│     - Receive FORWARD_JOIN from neighbors                          │
│     - Build active view                                            │
│                              │                                      │
│                              ▼                                      │
│  4. Register at rendezvous shards                                  │
│     - Calculate shards for own identities                          │
│     - Announce presence at each shard                              │
│                              │                                      │
│                              ▼                                      │
│  5. Subscribe to entity topics                                     │
│     - For each joined entity, subscribe to its topic               │
│     - Begin receiving updates                                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Network Characteristics

| Metric | Value | Notes |
|--------|-------|-------|
| Message latency | <2s (99th percentile) | Tree-based dissemination |
| Failure detection | <5s | SWIM protocol |
| Partition recovery | <30s | Periodic shuffle |
| Connections per node | 8-12 active | HyParView |
| Network overhead | <10KB/s | Steady state per peer |

## Consequences

### Benefits

- **Scalable**: O(log n) hops, O(n) messages
- **Resilient**: Self-healing overlay, no SPOF
- **Efficient**: Tree-based delivery minimizes duplicates
- **DHT-free**: Rendezvous simpler than full DHT
- **Topic filtering**: Only relevant messages delivered

### Trade-offs

- **Eventual delivery**: Not guaranteed instant
- **Bootstrap dependency**: Need at least one known peer
- **Membership churn**: High churn degrades performance
- **Geographic latency**: No geographic optimization (yet)

### Performance

| Operation | Latency | Bandwidth |
|-----------|---------|-----------|
| Message broadcast | <2s | O(n) total |
| Peer discovery | <5s | ~1KB |
| Membership update | <10s | ~500B |
| Full sync | Variable | State-dependent |

## Alternatives Considered

1. **Full mesh**: Every peer connected to every peer
   - Rejected: O(n²) connections, doesn't scale

2. **DHT (Kademlia)**: Distributed hash table routing
   - Rejected: Higher latency, more complex

3. **Blockchain consensus**: BFT for ordering
   - Rejected: Slow, expensive, not needed

4. **Central relay**: Messages through servers
   - Rejected: Single point of failure

5. **libp2p**: Existing P2P framework
   - Rejected: Too heavyweight, not QUIC-native

## References

- saorsa-gossip: `../saorsa-gossip/` crate
- saorsa-gossip-transport (ant-quic): `../saorsa-gossip/crates/transport`
- Architecture: `docs/architecture/gossip-protocol.md`
- Boot config: `config/production-network.toml`
- Implementation: `communitas-core/src/gossip/`

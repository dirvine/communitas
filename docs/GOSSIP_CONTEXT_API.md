# GossipContext API - Complete DHT Replacement

**Date**: 2025-10-05
**Status**: Phase 2.1 Complete ✅
**Next**: Phase 2.2 - Dual-write implementation

---

## Overview

The `GossipContext` is now a fully-featured replacement for `CoreContext`, providing all necessary APIs for DHT-free operation using the saorsa-gossip overlay network.

**Location**: `communitas-core/src/gossip/context.rs`

---

## API Surface

### 1. Storage API - CRDT-based Local-First Storage

Replaces DHT-based storage with local CRDT sets synchronized via anti-entropy.

```rust
// Store message locally
async fn store_message(&self, message: Vec<u8>) -> Result<()>

// Retrieve all messages
async fn get_all_messages(&self) -> Result<Vec<Vec<u8>>>

// Check message existence
async fn contains_message(&self, message: &Vec<u8>) -> Result<bool>

// Remove message
async fn remove_message(&self, message: &Vec<u8>) -> Result<()>
```

**Implementation Details**:
- Uses `OrSet<Vec<u8>>` CRDT for conflict-free replication
- Generates unique tags `(PeerId, timestamp)` for each operation
- Synchronized across network via `AntiEntropyManager`
- 60-second anti-entropy sync interval

---

### 2. Contact Discovery API - FOAF + Presence

Replaces DHT lookup with gossip-based discovery.

```rust
// Find contact using FOAF + Presence
async fn find_contact(&self, four_words: &str) -> Result<PeerId>

// Add known contact to cache
async fn add_contact(&self, four_words: String, peer_id: PeerId) -> Result<()>

// Get all cached contacts
async fn get_contacts(&self) -> Result<Vec<(String, PeerId)>>

// Remove contact from cache
async fn remove_contact(&self, four_words: &str) -> Result<()>
```

**Discovery Strategy**:
1. Check local cache (O(1))
2. Check presence in shared groups
3. Query FOAF (up to 2 hops max)
4. Fall back to introducer nodes (cold start)

**Tested**: 13/13 FOAF tests passing ✅

---

### 3. Messaging API - Plumtree Pub/Sub

Replaces DHT routing with gossip pub/sub on MLS group topics.

```rust
// Send direct message to peer (point-to-point)
async fn send_direct_message(&self, peer_id: PeerId, message: Vec<u8>) -> Result<()>

// Subscribe to entity's topic
async fn subscribe_to_entity(&self, entity_id: &str)
    -> Result<UnboundedReceiver<(PeerId, Bytes)>>

// Publish to entity's topic
async fn publish_to_entity(&self, entity_id: &str, message: Vec<u8>) -> Result<()>
```

**Transport Details**:
- Uses QUIC with three stream types: Membership, PubSub, Bulk
- Direct messages use `StreamType::Bulk`
- Plumtree provides epidemic broadcast with bounded overhead

---

### 4. Group Management API - MLS Groups + Topics

Maps Communitas entities to MLS groups and gossip topics.

```rust
// Join entity (creates MLS group + subscribes to topic)
async fn join_entity(&self, entity_id: &str, entity_type: &str) -> Result<()>

// Leave entity (unsubscribe + leave MLS group)
async fn leave_entity(&self, entity_id: &str) -> Result<()>

// Map entity to topic ID
async fn map_entity_to_topic(&self, entity_id: &str, entity_type: &str) -> Result<TopicId>
```

**Entity Types**: "channel", "project", "org"

---

### 5. Backup & Recovery API - Favourite Contacts

Implements SPEC.md §4 encrypted replica backup.

```rust
// Replicate state to favourites
async fn replicate_to_favourites(&self) -> Result<()>

// Recover from favourite
async fn recover_from_favourite(&self, four_words: &str) -> Result<()>

// Manage favourites
async fn add_favourite_contact(&self, four_words: String) -> Result<()>
async fn get_favourite_contacts(&self) -> Vec<String>
```

**Backup Flow**:
1. Serialize CRDT state with bincode
2. Encrypt with ChaCha20Poly1305 (per-favourite keys)
3. Send via QUIC Bulk stream
4. Favourite stores encrypted replica

**Recovery Flow**:
1. Connect to any favourite contact
2. Request encrypted replica
3. Decrypt with ChaCha20Poly1305
4. Deserialize and merge into local CRDT
5. Rejoin MLS groups

---

### 6. Presence API - Group-Scoped Online Status

Implements SPEC.md §5 presence model.

```rust
// Start presence beacons (5min interval)
async fn start_presence_beacons(&self) -> Result<()>

// Stop beacons
async fn stop_presence_beacons(&self) -> Result<()>

// Check if peer online in any shared group
async fn is_peer_online(&self, peer_id: PeerId) -> Result<bool>

// Get online peers in entity
async fn get_online_peers(&self, entity_id: &str) -> Result<Vec<PeerId>>
```

**Beacon Details**:
- Interval: 300 seconds (5 minutes)
- TTL: 900 seconds (15 minutes)
- Scope: Group-scoped only (no global presence)
- Encryption: MLS group keys

---

## Encryption Specification

**All encrypted data uses ChaCha20Poly1305 AEAD**

### Why ChaCha20Poly1305 over AES-GCM?

1. **Performance**: Better on non-hardware-accelerated platforms (mobile, IoT)
2. **Constant-time**: No cache-timing side channels
3. **Simpler**: Single primitive, easier to implement correctly
4. **Modern**: Recommended by IETF (RFC 8439) for new protocols

### Usage Locations

1. **Presence Beacons**: MLS group key → ChaCha20Poly1305 → encrypt beacon
2. **Backup Replicas**: Per-favourite key → ChaCha20Poly1305 → encrypt CRDT state
3. **Future MLS Messages**: Group key → ChaCha20Poly1305 → encrypt payload

### Key Derivation

```
Per-Favourite Backup Key:
    ML-DSA shared secret + favourite_peer_id
    → HKDF-SHA256
    → ChaCha20Poly1305 key
```

---

## Implementation Status

### ✅ Phase 2.1 Complete

- [x] Storage API (CRDT-based)
- [x] Contact Discovery API (FOAF + Presence)
- [x] Messaging API (Plumtree pub/sub)
- [x] Group Management API (MLS + topics)
- [x] Backup & Recovery API (encrypted replicas)
- [x] Presence API (group-scoped beacons)
- [x] All APIs compile with zero errors ✅
- [x] ChaCha20Poly1305 encryption specified ✅

### ✅ Phase 2.2 Complete (Tauri Integration)

- [x] Wire GossipContext into Tauri commands (22 commands)
- [x] Created `gossip_commands.rs` module with full API
- [x] Feature-gated with `#[cfg(feature = "gossip_overlay")]`
- [x] Integrated into main.rs with state management
- ⏸️ Testing blocked (awaiting saorsa-mls 0.3.0 publication)

**See**: `docs/PHASE_2_2_STATUS.md` for details

### ⏳ Phase 2.3 Next (Dual-Write Testing & Migration)

- [ ] Test Tauri commands once saorsa-mls 0.3.0 published
- [ ] Add feature flag `gossip_only` for DHT removal
- [ ] Implement dual-write (both DHT + gossip)
- [ ] Collect KPIs (latency, reliability, overhead)
- [ ] Replace all DHT calls with GossipContext
- [ ] Test complete boot sequence without DHT
- [ ] Verify all functionality works

### ⏳ Phase 3 (DHT Removal)

- [ ] Enable `gossip_only` feature
- [ ] Remove saorsa-core DHT dependency
- [ ] Delete legacy `core_context.rs`
- [ ] Clean up dht_* modules

---

## Testing

### Unit Tests

**Location**: `communitas-core/src/gossip/context.rs::tests`

```rust
#[tokio::test]
async fn test_gossip_context_initialization()

#[tokio::test]
async fn test_favourite_contacts()
```

### Integration Tests

**Location**: `communitas-core/tests/foaf_discovery_tests.rs`

- ✅ 13/13 FOAF discovery tests passing
- ✅ 1-hop, 2-hop discovery
- ✅ Cycle detection
- ✅ Hop limiting
- ✅ Presence integration

---

## Usage Example

```rust
use communitas_core::gossip::GossipContext;

// Initialize
let ctx = GossipContext::initialize(
    "ocean-forest-moon-star".to_string(),
    "Alice".to_string(),
    "Desktop".to_string(),
).await?;

// Join a channel
ctx.join_entity("general", "channel").await?;

// Start presence
ctx.start_presence_beacons().await?;

// Find contact via FOAF
let bob_peer_id = ctx.find_contact("bob-river-mountain-cloud").await?;

// Send direct message
ctx.send_direct_message(bob_peer_id, b"Hello!".to_vec()).await?;

// Subscribe to channel messages
let mut rx = ctx.subscribe_to_entity("general").await?;
while let Some((sender, msg)) = rx.recv().await {
    println!("Message from {:?}: {:?}", sender, msg);
}

// Store message locally
ctx.store_message(b"Important data".to_vec()).await?;

// Backup to favourites
ctx.add_favourite_contact("charlie-delta-echo-fox".to_string()).await?;
ctx.replicate_to_favourites().await?;
```

---

## Performance Characteristics

### Storage Operations

- **store_message**: O(log n) - CRDT add with unique tag
- **get_all_messages**: O(n) - Iterator over CRDT elements
- **contains_message**: O(1) - HashMap lookup
- **remove_message**: O(log n) - CRDT remove

### Discovery Operations

- **find_contact (cache)**: O(1) - HashMap lookup
- **find_contact (presence)**: O(g × p) - g groups, p peers per group
- **find_contact (FOAF)**: O(c^h) - c contacts, h hops (max 2)

### Network Overhead

- **Presence beacons**: 1 message per 5 minutes per group
- **Anti-entropy**: 1 sync per 60 seconds per peer
- **Plumtree pub/sub**: O(log n) message complexity

---

## Configuration

### Tunable Parameters

```rust
// In GossipContext::initialize
membership: HyParView(active_degree: 5, passive_degree: 15)
anti_entropy: sync_interval_secs: 60
presence: beacon_interval: 300, ttl: 900
foaf_discovery: max_hops: 2, query_timeout_ms: 5000
```

### Recommended Settings

- **Small networks (<100 peers)**: active_degree: 3
- **Medium networks (100-1000)**: active_degree: 5 (default)
- **Large networks (>1000)**: active_degree: 7

---

## Security Considerations

### Encryption

✅ ChaCha20Poly1305 provides:
- Authenticated encryption
- Associated data (AD) support
- Resistance to timing attacks
- High performance

### FOAF Privacy

- Four-word addresses are public within shared groups
- FOAF queries reveal social graph (2-hop max)
- Consider query rate limiting to prevent enumeration

### Backup Security

- Favourites can see encrypted replica size (metadata leak)
- Use padding to hide message count
- Rotate per-favourite keys periodically

---

## Migration Path

### Phase 2: Dual-Write (Current)

```rust
#[cfg(not(feature = "gossip_only"))]
pub struct HybridContext {
    gossip: GossipContext,
    legacy_dht: Option<CoreContext>,
}

async fn store_message(&self, msg: Vec<u8>) -> Result<()> {
    // New path (gossip)
    self.gossip.store_message(msg.clone()).await?;

    // Legacy path (DHT) - optional fallback
    #[cfg(not(feature = "gossip_only"))]
    if let Some(dht) = &self.legacy_dht {
        dht.store_message(msg).await.ok();
    }

    Ok(())
}
```

### Phase 3: Gossip-Only

```rust
// Enable feature flag in Cargo.toml
gossip_only = ["gossip_overlay"]

// Remove DHT dependency
// [dependencies]
// saorsa-core = "0.5.7"  # REMOVED

// Use GossipContext directly
pub use gossip::GossipContext as CoreContext;
```

---

## Next Steps

### Immediate (Today)

1. ✅ Complete GossipContext API expansion
2. ✅ Fix all compilation errors
3. ✅ Document ChaCha20Poly1305 encryption
4. ⏳ Wire GossipContext into Tauri commands

### Short Term (This Week)

5. [ ] Implement dual-write in Tauri commands
6. [ ] Add telemetry for gossip vs DHT comparison
7. [ ] Test complete boot sequence with gossip
8. [ ] Begin KPI collection

### Medium Term (Next Week)

9. [ ] Remove DHT dependency with `gossip_only` feature
10. [ ] Clean up legacy code
11. [ ] Production rollout Phase 2

---

## References

- **SPEC.md**: Project specification
- **DHT_REMOVAL_AUDIT.md**: Migration plan
- **FOAF_DISCOVERY_IMPLEMENTATION.md**: FOAF details
- **RFC 8439**: ChaCha20-Poly1305 AEAD
- **Plumtree Paper**: Efficient epidemic broadcast
- **HyParView Paper**: Hybrid partial view membership

---

## Success Criteria

### Phase 2.1 ✅

- [x] All GossipContext APIs implemented
- [x] Zero compilation errors
- [x] ChaCha20Poly1305 documented
- [x] FOAF tests passing (13/13)

### Phase 2.2 (Next)

- [ ] All Tauri commands use GossipContext
- [ ] Dual-write working (gossip + DHT)
- [ ] KPIs collected and analyzed
- [ ] Performance ≥ DHT baseline

### Phase 3 (Final)

- [ ] Zero DHT code remaining
- [ ] All tests pass
- [ ] Production deployment successful
- [ ] User experience maintained or improved

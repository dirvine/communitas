# DHT Removal Audit

**Date**: 2025-10-05
**Status**: Analysis Phase
**Goal**: Remove all DHT dependencies and replace with gossip overlay

---

## Executive Summary

**Current State**:
- 45 saorsa_core DHT references across 9 files
- DHT deeply integrated into saorsa-core's StorageManager, ChatManager, and MessagingService
- Gossip overlay infrastructure ready with FOAF discovery, presence, and Plumtree pub/sub

**Migration Strategy**:
- **Phase 1**: Add gossip-based alternatives alongside DHT (dual-write)
- **Phase 2**: Migrate all discovery/lookup calls to gossip
- **Phase 3**: Remove DHT dependencies entirely
- **Phase 4**: Clean up legacy code

---

## DHT Usage Analysis

### Files with DHT References

1. **core_context.rs** (Primary Integration Point)
   - `DhtClient` - Direct DHT access
   - `DhtCoreEngine` - Storage and chat backing
   - `P2PNode` - Network node with DHT

2. **dht_identity/storage.rs** - Identity storage with DHT keys
3. **dht_identity/blobs.rs** - Blob storage with DHT
4. **dht_identity/key_derivation.rs** - Four-word validation
5. **dht_schemas.rs** - DHT data schemas
6. **messaging.rs** - Message routing via DHT
7. **bootstrap_integration.rs** - Bootstrap with DHT
8. **lib.rs** - Re-exports saorsa_core (which includes DHT)
9. **security/input_validation.rs** - Four-word validation via DHT

### DHT Functionality in Use

#### Discovery & Lookup
- **Current**: DHT announce/lookup for contact discovery
- **Replacement**: FOAF discovery + Presence beacons ✅ (IMPLEMENTED)
- **Files**: `core_context.rs`, `messaging.rs`

#### Storage Backend
- **Current**: `StorageManager` requires `DhtCoreEngine`
- **Replacement**: Local-first CRDT storage with gossip anti-entropy
- **Files**: `core_context.rs:86-96`

#### Message Routing
- **Current**: DHT-based message delivery
- **Replacement**: Plumtree pub/sub on MLS group topics
- **Files**: `messaging.rs`, `core_context.rs:39`

#### Bootstrap
- **Current**: DHT for initial peer discovery
- **Replacement**: Introducer nodes + favourite contacts
- **Files**: `bootstrap_integration.rs`, `gossip/boot.rs`

---

## Migration Plan

### Phase 1: Gossip Infrastructure ✅ COMPLETE

- [x] FOAF discovery implementation
- [x] Presence-based contact finding
- [x] Plumtree pub/sub for topics
- [x] MLS groups integration
- [x] Introducer nodes for cold start
- [x] Boot sequence with gossip

**Status**: All infrastructure ready, tested, and working

### Phase 2: Dual-Write (CURRENT)

Create gossip-based alternatives while keeping DHT:

#### 2.1: GossipContext as Primary Context

**File**: `communitas-core/src/gossip/context.rs`

**Strategy**:
```rust
// Add to GossipContext
pub struct GossipContext {
    // ... existing fields ...

    // Legacy DHT support (Phase 2 only)
    #[cfg(not(feature = "gossip_only"))]
    legacy_dht_client: Option<DhtClient>,
}
```

**Tasks**:
- [ ] Add storage methods to GossipContext
- [ ] Add messaging methods to GossipContext
- [ ] Add chat methods to GossipContext
- [ ] Wire GossipContext into Tauri commands

#### 2.2: Discovery Migration

**Current**: `DhtClient` for contact lookup
**New**: FOAF + Presence

**Files to Update**:
- `communitas-desktop/src/core_commands.rs`
- `communitas-core/src/messaging.rs`

**Implementation**:
```rust
// Old (DHT)
async fn find_contact(four_words: &str) -> Result<PeerId> {
    dht_client.lookup(four_words).await
}

// New (Gossip)
async fn find_contact(four_words: &str) -> Result<PeerId> {
    gossip_ctx.discovery.find_contact(four_words).await
}
```

#### 2.3: Storage Migration

**Current**: `StorageManager` with `DhtCoreEngine`
**New**: Local CRDT storage with anti-entropy

**Implementation**:
```rust
// Phase 2: Keep both
pub struct HybridStorage {
    local_crdt: CrdtMessageSet,
    legacy_dht: Option<StorageManager>,
}

// Phase 3: Remove legacy_dht
pub struct GossipStorage {
    local_crdt: CrdtMessageSet,
    anti_entropy: AntiEntropyManager,
}
```

#### 2.4: Messaging Migration

**Current**: `MessagingService` with DHT routing
**New**: Plumtree on MLS topics

**Implementation**:
```rust
// Phase 2: Dual publish
async fn send_message(channel: &str, msg: Message) {
    // New path (gossip)
    gossip_ctx.pubsub.publish(topic_id, msg).await?;

    // Legacy path (DHT) - optional
    #[cfg(not(feature = "gossip_only"))]
    legacy_messaging.send(msg).await?;
}
```

### Phase 3: Complete DHT Removal

Enable `gossip_only` feature and remove all DHT code:

#### 3.1: Remove DHT Dependencies

**Cargo.toml Changes**:
```toml
[features]
default = ["gossip_overlay"]
gossip_only = ["gossip_overlay"]  # Removes DHT entirely

[dependencies]
# Remove or make optional:
# saorsa-core = "0.5.7"  # Contains DHT
```

#### 3.2: Replace CoreContext

**File**: `communitas-core/src/lib.rs`

```rust
// Old
pub use core_context::CoreContext;

// New
#[cfg(feature = "gossip_only")]
pub use gossip::GossipContext as CoreContext;

#[cfg(not(feature = "gossip_only"))]
pub use core_context::CoreContext;
```

#### 3.3: Update All Tauri Commands

Replace all `CoreContext` usage with `GossipContext` methods.

**Files**:
- `communitas-desktop/src/core_commands.rs`
- `communitas-desktop/src/core_groups.rs`
- All other command modules

### Phase 4: Legacy Code Cleanup

Remove old DHT modules entirely:

- [ ] Delete `dht_identity/` directory
- [ ] Delete `dht_schemas.rs`
- [ ] Delete legacy `core_context.rs`
- [ ] Remove `saorsa-core` dependency
- [ ] Update documentation

---

## Testing Strategy

### Phase 2 Testing (Dual-Write)

1. **Compatibility Tests**
   - [ ] Both DHT and gossip paths work
   - [ ] Messages delivered via both paths
   - [ ] Discovery works via both methods

2. **Performance Tests**
   - [ ] Gossip latency vs DHT latency
   - [ ] Gossip reliability vs DHT reliability
   - [ ] Resource usage comparison

3. **Migration Tests**
   - [ ] Gradual rollout (10% → 50% → 100% gossip)
   - [ ] Fallback to DHT if gossip fails
   - [ ] KPI collection

### Phase 3 Testing (Gossip-Only)

1. **Functional Tests**
   - [ ] All discovery works without DHT
   - [ ] All messaging works without DHT
   - [ ] All storage works without DHT
   - [ ] Cold start via introducers works

2. **Integration Tests**
   - [ ] Multi-peer FOAF discovery
   - [ ] Group join/leave flows
   - [ ] Message delivery across mesh
   - [ ] Anti-entropy convergence

3. **Stress Tests**
   - [ ] 100+ peers in mesh
   - [ ] Network partitions and healing
   - [ ] High message volume
   - [ ] Extended offline → online sync

---

## Risk Assessment

### High Risk

1. **saorsa-core Tight Coupling**
   - StorageManager requires DhtCoreEngine
   - ChatManager requires StorageManager
   - May need to fork or modify saorsa-core

   **Mitigation**: Create gossip-native equivalents, don't depend on saorsa-core internals

2. **Data Migration**
   - Existing DHT-stored data needs migration
   - Cannot lose user data during transition

   **Mitigation**: Phase 2 dual-write, export/import tools

### Medium Risk

3. **Discovery Reliability**
   - FOAF may not find all contacts DHT could find
   - Introducer nodes are single points of failure

   **Mitigation**: Multiple introducers, aggressive presence beaconing, longer TTLs

4. **Performance Regression**
   - Gossip may be slower than DHT for some operations
   - Anti-entropy overhead

   **Mitigation**: Comprehensive benchmarking in Phase 2, optimize before Phase 3

### Low Risk

5. **Feature Parity**
   - Some DHT features may not have gossip equivalents

   **Mitigation**: Document gaps, implement before removal

---

## Timeline Estimate

### Phase 2: Dual-Write (Current Sprint)
- **Duration**: 2-3 days
- **Tasks**:
  - Expand GossipContext API
  - Add dual-write to all paths
  - Comprehensive testing

### Phase 3: DHT Removal (Next Sprint)
- **Duration**: 1-2 days
- **Tasks**:
  - Remove DHT dependencies
  - Clean up code
  - Final testing

### Phase 4: Cleanup (Polish Sprint)
- **Duration**: 1 day
- **Tasks**:
  - Delete legacy code
  - Update documentation
  - Performance optimization

---

## Decision Points

### Q1: Keep saorsa-core as dependency?

**Option A**: Keep for crypto/identity primitives only
- Pros: Less reimplementation
- Cons: Still pulls in DHT code

**Option B**: Extract only what we need
- Pros: Clean separation, smaller binary
- Cons: More maintenance

**Recommendation**: Option A for Phase 2, Option B for Phase 3

### Q2: Backward compatibility?

**Option A**: Support DHT clients during transition
- Pros: Smooth migration
- Cons: Complexity, technical debt

**Option B**: Hard cut-over to gossip-only
- Pros: Clean, simple
- Cons: All clients must update simultaneously

**Recommendation**: Option A (dual-write in Phase 2)

---

## Next Steps

### Immediate (Today)

1. [x] Create this audit document
2. [ ] Expand GossipContext with storage/messaging APIs
3. [ ] Add dual-write to discovery calls
4. [ ] Test FOAF + DHT in parallel

### Short Term (This Week)

5. [ ] Migrate all Tauri commands to GossipContext
6. [ ] Implement gossip-based storage
7. [ ] Add telemetry for gossip vs DHT comparison
8. [ ] Begin KPI collection

### Medium Term (Next Week)

9. [ ] Remove DHT feature flag
10. [ ] Clean up legacy code
11. [ ] Production rollout

---

## Success Criteria

### Phase 2 Success
- [ ] All functionality works via gossip
- [ ] DHT can be disabled without breaking
- [ ] KPIs show gossip is viable alternative

### Phase 3 Success
- [ ] Zero DHT code remaining
- [ ] All tests pass
- [ ] Performance equals or exceeds DHT baseline

### Overall Success
- [ ] SPEC.md §3 fully implemented
- [ ] Binary size reduced (no DHT dependencies)
- [ ] Network resilience improved
- [ ] User experience unchanged or better

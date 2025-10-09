# Communitas → Saorsa Gossip Migration Status

**Date**: 2025-10-04
**Status**: 🚧 **In Progress** - Phase 0 Complete, Phase 1 API Alignment 75% Complete
**Next**: Complete remaining presence/CRDT API implementations in saorsa-gossip

---

## ✅ Phase 0: Dependencies & Feature Flags (COMPLETE)

### Workspace Dependencies Added
All saorsa-gossip crates wired via local paths:

```toml
# /Cargo.toml
saorsa-gossip-types = { path = "../saorsa-gossip/crates/types" }
saorsa-gossip-identity = { path = "../saorsa-gossip/crates/identity" }
saorsa-gossip-crdt-sync = { path = "../saorsa-gossip/crates/crdt-sync" }
saorsa-gossip-groups = { path = "../saorsa-gossip/crates/groups" }
saorsa-gossip-presence = { path = "../saorsa-gossip/crates/presence" }
saorsa-gossip-transport = { path = "../saorsa-gossip/crates/transport" }
saorsa-gossip-membership = { path = "../saorsa-gossip/crates/membership" }
saorsa-gossip-pubsub = { path = "../saorsa-gossip/crates/pubsub" }
```

### Feature Flag System
Dual-write capability via `gossip_overlay` feature:

```toml
# communitas-core/Cargo.toml
[features]
gossip_overlay = [
    "dep:saorsa-gossip-types",
    "dep:saorsa-gossip-identity",
    "dep:saorsa-gossip-crdt-sync",
    "dep:saorsa-gossip-groups",
    "dep:saorsa-gossip-presence",
    "dep:saorsa-gossip-transport",
    "dep:saorsa-gossip-membership",
    "dep:saorsa-gossip-pubsub",
]
```

### Module Structure Created
```
communitas-core/src/gossip/
├── mod.rs           - Module root with feature gates
├── context.rs       - GossipContext (mirrors CoreContext)
├── boot.rs          - Boot sequence per SPEC.md §2
├── presence.rs      - Presence management (SPEC.md §5)
├── backup.rs        - Favourite contacts backup (SPEC.md §4)
└── telemetry.rs     - Metrics collection (SPEC.md §6)
```

---

## 📋 SPEC.md Mappings Implemented

| Communitas Object | Gossip Mapping | Status |
|-------------------|----------------|--------|
| User identity (four-word) | ML-DSA identity + alias | ✅ Struct defined |
| Contact | Overlay edge and seed | ✅ Struct defined |
| Channel/Project/Org | MLS group + gossip topic | ✅ Struct defined |
| Presence | MLS-encrypted beacons | ✅ Stub implemented |
| Backup | Favourite contacts replicas | ✅ Implemented |

---

## 🎯 GossipContext Structure (Defined)

```rust
pub struct GossipContext {
    // Identity
    pub identity: Identity,
    pub four_words: String,
    pub peer_id: PeerId,

    // Membership (HyParView + SWIM)
    pub membership: Arc<RwLock<Box<dyn Membership>>>,

    // MLS Groups + Topics
    pub groups: Arc<RwLock<HashMap<String, GroupContext>>>,
    pub topics: Arc<RwLock<HashMap<String, TopicId>>>,

    // Presence beacons
    pub presence: Arc<RwLock<PresenceManager>>,

    // CRDT sync
    pub crdt_message_set: Arc<RwLock<OrSet<Vec<u8>>>>,

    // Transport & Pub/Sub
    pub transport: Arc<QuicTransport>,
    pub pubsub: Arc<RwLock<Box<dyn PubSub>>>,

    // Backup
    pub favourite_contacts: Arc<RwLock<Vec<String>>>,
}
```

---

## 🎉 Phase 1: API Alignment (75% COMPLETE)

### ✅ Fixed API Issues (TDD Approach)

1. **`Identity::load_or_create()`** - ✅ **COMPLETE**
   - Added `load_or_create(four_words, display_name, keystore_path)` method
   - Implemented keystore persistence with bincode serialization
   - 8/8 tests passing in `crates/identity/src/lib.rs`
   - Uses file-based storage: `{keystore_path}/{four_words}.identity`

2. **`TopicId::from_entity()`** - ✅ **COMPLETE**
   - Added helper method to derive TopicId from entity string
   - Uses BLAKE3 for deterministic topic derivation
   - 16/16 tests passing in `crates/types/src/lib.rs`
   - Enables `TopicId::from_entity("channel-123")` usage

3. **`GroupContext::from_entity()`** - ✅ **COMPLETE**
   - Added convenience constructor accepting entity_id string
   - Equivalent to `GroupContext::new(TopicId::from_entity(...))`
   - 4/4 tests passing in `crates/groups/src/lib.rs`
   - Simplifies channel/project/org group creation

4. **`QuicTransport::new()`** - ✅ **VERIFIED**
   - Confirmed not async (synchronous constructor)
   - Takes `TransportConfig` parameter
   - No changes needed - API already correct

### 🚧 Remaining API Issues

5. **`PresenceManager::start_beacons()`** - 🔴 **BLOCKER**
   - Method doesn't exist in saorsa-gossip-presence crate
   - Presence system only 25% complete
   - Needs implementation of beacon broadcasting

6. **CRDT Sync API** - 🔴 **BLOCKER**
   - `crdt_sync` field doesn't exist on `GossipContext`
   - Anti-entropy reconciliation not implemented
   - Need `CrdtSyncManager` or similar

**Resolution**: Continue TDD approach to implement remaining APIs in saorsa-gossip.

---

## 📊 Compilation Status

### 🚧 Current State (Improved!)
```bash
cargo check -p communitas-core --features gossip_overlay
# 14 compilation errors (down from 15!)
# All errors in boot.rs/telemetry.rs waiting on Presence/CRDT APIs
```

### ✅ Saorsa-Gossip Crates (All Green!)
```bash
cargo test -p saorsa-gossip-identity     # 8 passed  ✅
cargo test -p saorsa-gossip-types        # 16 passed ✅
cargo test -p saorsa-gossip-groups       # 4 passed  ✅
cargo test -p saorsa-gossip-transport    # 3 passed  ✅
```

### ✅ Without Gossip Feature
```bash
cargo check -p communitas-core
# Compiles successfully - existing DHT code unaffected
```

**Progress**: API alignment reducing compilation errors. Remaining errors all in placeholder code waiting for Phase 2 (Presence Implementation).

---

## 🎯 Next Steps

### Immediate (Week 1-2)
1. **Wait for saorsa-gossip API stabilization**
   - All 8 crates are packaged and ~85% complete
   - 3 crates waiting for crates.io rate limit to publish
   - APIs will stabilize in next iteration

2. **Create stub implementations**
   - Add `#[allow(dead_code)]` to gossip module
   - Create minimal trait impls for compilation
   - Focus on type alignment over functionality

### Short-term (Week 3-4)
3. **Implement boot sequence** (SPEC.md §2)
   - Load ML-DSA identity
   - Dial favourite contacts
   - Start membership layer
   - Join existing entities
   - Start presence beacons

4. **Migrate CRDT implementation**
   - Replace `communitas-core/src/crdt.rs` (43 tests)
   - Use `saorsa-gossip-crdt-sync::OrSet`
   - Keep all tests passing

### Medium-term (Week 5-6)
5. **Dual-write phase**
   - Run both DHT and Gossip in parallel
   - Collect KPI metrics (SPEC.md §6)
   - Validate gossip ≥ DHT performance

6. **Replace DHT calls** (SPEC.md §3)
   - 57 occurrences across 9 files
   - Use Presence::find + FOAF queries
   - Remove `dht_*` modules

### Long-term (Week 7-8)
7. **Remove saorsa-core dependency**
   - After successful validation
   - Clean up dead DHT code
   - Update documentation

---

## 📁 Files Modified

### Created
- `Cargo.toml` - Added saorsa-gossip workspace deps
- `communitas-core/Cargo.toml` - Added gossip_overlay feature
- `communitas-desktop/Cargo.toml` - Added gossip_overlay feature
- `communitas-core/src/gossip/mod.rs` - Module root (30 lines)
- `communitas-core/src/gossip/context.rs` - GossipContext (300+ lines)
- `communitas-core/src/gossip/boot.rs` - Boot sequence (200+ lines)
- `communitas-core/src/gossip/presence.rs` - Presence management (150+ lines)
- `communitas-core/src/gossip/backup.rs` - Backup system (150+ lines)
- `communitas-core/src/gossip/telemetry.rs` - Metrics (150+ lines)
- `docs/GOSSIP_MIGRATION_STATUS.md` - This file

### Modified
- `communitas-core/src/lib.rs` - Added `#[cfg(feature = "gossip_overlay")] pub mod gossip`

---

## 🔍 DHT Usage Audit

Files requiring migration (57 total occurrences):

| File | Occurrences | Priority |
|------|-------------|----------|
| `core_context.rs` | 32 | 🔴 High |
| `messaging.rs` | 5 | 🔴 High |
| `bootstrap_integration.rs` | 3 | 🟡 Medium |
| `dht_identity/*` | Multiple | 🟢 Low (delete) |
| `dht_schemas.rs` | 1 | 🟢 Low (delete) |
| `dht_storage.rs` | 1 | 🟢 Low (delete) |

---

## ✅ Success Criteria

### Phase 0 (Complete)
- [x] Local saorsa-gossip dependencies wired
- [x] Feature flags configured
- [x] Gossip module structure created
- [x] SPEC.md mappings defined
- [x] Zero impact on existing DHT code

### Phase 1 (Pending API Stabilization)
- [ ] GossipContext compiles with --features gossip_overlay
- [ ] Boot sequence functional
- [ ] Can join/leave entities
- [ ] Presence beacons working

### Phase 2 (Future)
- [ ] Dual-write validation
- [ ] CRDT migration complete
- [ ] DHT code removed
- [ ] Production deployment

---

## 📞 Support

For questions:
- Check `SPEC.md` for architectural decisions
- Review `saorsa-gossip/README.md` for API status
- See `saorsa-gossip/docs/audit.md` for implementation completeness

---

## 🎉 Summary

**Phase 0 is complete!** The foundation for migrating from DHT to gossip overlay is in place:

- ✅ All saorsa-gossip crates accessible via local paths
- ✅ Feature flag system prevents breaking existing code
- ✅ Complete module structure following SPEC.md
- ✅ Zero regression to current working code

**Next milestone**: Wait for saorsa-gossip API stabilization, then align implementations. The architecture is ready; we're just waiting for the underlying library to mature.

**Timeline estimate**: 6-8 weeks total, currently ~1 week in.

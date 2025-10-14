# Implementation Plan - Gossip Overlay Integration

**Status**: Phase 1 Complete ✅
**Date**: 2025-10-14
**Based on**: ARCHITECTURE_AUDIT_FINDINGS.md

---

## Phase 1: Quick Fixes ✅ COMPLETED

**Duration**: 2 hours
**Status**: ✅ All tasks completed

### Completed Tasks

- [x] **Enable gossip_overlay feature** - Added `default = ["gossip_overlay"]` to `communitas-desktop/Cargo.toml:20`
- [x] **Remove FEC references** - Deleted `sync_repair_fec` command from `sync.rs` and `main.rs`
- [x] **Rename DHT → gossip** - Updated terminology in `core_cmds.rs`:
  - Renamed `DhtStatus` → `GossipStatus`
  - Renamed `check_dht_connection` → `check_gossip_connection`
  - Updated error messages to say "gossip overlay"
  - Updated `main.rs` comment from "DHT surface" to "gossip overlay surface"
- [x] **Update security docs** - Removed all FEC references from `.github/SECURITY.md`:
  - Updated encryption architecture (lines 90-93)
  - Updated cryptographic requirements (lines 16-27)
  - Updated approved libraries list (lines 159-164)

### Files Modified (Phase 1)

1. `communitas-desktop/Cargo.toml` - Enable gossip_overlay by default
2. `communitas-desktop/src/sync.rs` - Remove sync_repair_fec
3. `communitas-desktop/src/core_cmds.rs` - Rename DHT to gossip
4. `communitas-desktop/src/main.rs` - Update command registration and comments
5. `.github/SECURITY.md` - Remove FEC references, update cryptography docs

### Impact

- ✅ **Gossip overlay now enabled by default** - All 30+ gossip commands now available
- ✅ **Terminology consistent** - No more confusing DHT references
- ✅ **Documentation accurate** - Security docs reflect current architecture
- ✅ **Deprecated code removed** - FEC references eliminated

---

## Phase 2: Core Integration

**Duration**: 1-2 days
**Goal**: Wire placeholder commands to gossip overlay functionality
**Status**: ⏳ Ready to start

### Task Breakdown

#### 2.1: Wire Core Commands to Gossip (Priority: HIGH)

**Approach**: Implement bridge pattern - `core_commands.rs` calls `gossip_commands.rs` internally

**Files to Modify**:
- `communitas-desktop/src/core_commands.rs` (365 lines)
- New file: `communitas-desktop/src/gossip_bridge.rs` (optional abstraction layer)

**Command Mappings** (40+ commands):

| core_commands.rs | gossip_commands.rs | Status |
|------------------|---------------------|--------|
| `core_send_message_to_recipients` | `gossip_send_direct_message` | ⏳ TODO |
| `subscribe_to_entity` | `gossip_subscribe_to_entity` | ⏳ TODO |
| `unsubscribe_from_entity` | `gossip_leave_entity` | ⏳ TODO |
| `core_get_bootstrap_nodes` | `gossip_get_cached_peers` | ⏳ TODO |
| `core_add_bootstrap_node` | `gossip_add_bootstrap_peer` | ⏳ TODO |
| `core_create_channel` | Implement with gossip entities | ⏳ TODO |
| `core_send_message_to_channel` | `gossip_publish_to_entity` | ⏳ TODO |
| `core_channel_invite_by_words` | `gossip_find_contact` + `gossip_join_entity` | ⏳ TODO |
| `core_private_put` | `gossip_store_message` | ⏳ TODO |
| `core_private_get` | `gossip_get_all_messages` | ⏳ TODO |

**Implementation Steps**:

1. **Create GossipBridge abstraction** (optional):
   ```rust
   // communitas-desktop/src/gossip_bridge.rs
   use crate::gossip_commands::GossipState;

   pub struct GossipBridge {
       gossip_state: GossipState,
   }

   impl GossipBridge {
       pub async fn send_message(&self, recipient: String, content: String) -> Result<(), String> {
           // Convert core API → gossip API
           let message = content.as_bytes().to_vec();
           gossip_commands::gossip_send_direct_message(
               self.gossip_state.clone(),
               recipient,
               message
           ).await
       }

       // ... more bridge methods
   }
   ```

2. **Wire each core command**:
   ```rust
   // Before (placeholder):
   #[tauri::command]
   pub async fn core_send_message_to_recipients(
       _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
       _recipients: Vec<String>,
       _content: String,
   ) -> Result<MessageInfo, String> {
       Err("Not yet implemented".to_string())
   }

   // After (wired to gossip):
   #[tauri::command]
   pub async fn core_send_message_to_recipients(
       gossip_state: State<'_, GossipState>,
       recipients: Vec<String>,
       content: String,
   ) -> Result<MessageInfo, String> {
       // Send to each recipient via gossip overlay
       for recipient in recipients {
           gossip_commands::gossip_send_direct_message(
               gossip_state.inner().clone(),
               recipient,
               content.as_bytes().to_vec()
           ).await?;
       }

       // Return message info
       Ok(MessageInfo {
           id: uuid::Uuid::new_v4().to_string(),
           content,
           author: "current_user".to_string(), // TODO: Get from context
           timestamp: chrono::Utc::now().timestamp(),
       })
   }
   ```

3. **Add GossipState to Tauri app state**:
   ```rust
   // main.rs
   .manage(Arc::new(RwLock::new(None::<GossipContext>)) as GossipState)
   ```

4. **Update frontend** to call core commands (which now work via gossip)

5. **Add integration tests** for each wired command

**Estimated Work**:
- Day 1: Wire 10-15 high-priority commands (messaging, channels, bootstrap)
- Day 2: Wire remaining 25+ commands + tests

#### 2.2: Update Frontend Command Calls (Priority: MEDIUM)

**Files to Update**:
- Check all frontend files calling deprecated commands
- Update to use either:
  - **Option A**: Keep using `core_*` commands (now backed by gossip)
  - **Option B**: Migrate to direct `gossip_*` commands

**Frontend Files to Review**:
```bash
# Find all invoke() calls
grep -r "invoke(" src/ --include="*.ts" --include="*.tsx"
```

**Estimated Work**: 4-6 hours (depends on number of callsites)

#### 2.3: Add Integration Tests (Priority: HIGH)

**Test Coverage Required**:
- [ ] Initialize gossip context
- [ ] Send direct message between peers
- [ ] Create channel and invite members
- [ ] Subscribe/publish to entity topics
- [ ] Store/retrieve private data
- [ ] Contact discovery via FOAF
- [ ] Presence beacons

**Test File**: `communitas-desktop/tests/gossip_integration_test.rs`

**Example Test**:
```rust
#[tokio::test]
async fn test_send_message_via_core_command() {
    // Initialize gossip context
    let gossip_state = Arc::new(RwLock::new(None));
    gossip_commands::gossip_initialize(
        tauri::State::from(&gossip_state),
        "ocean-forest-moon-star".to_string(),
        "Test User".to_string(),
        "Test Device".to_string(),
    ).await.unwrap();

    // Send message via core command (which uses gossip internally)
    let result = core_commands::core_send_message_to_recipients(
        gossip_state,
        vec!["river-mountain-sun-cloud".to_string()],
        "Hello from test".to_string(),
    ).await;

    assert!(result.is_ok());
}
```

**Estimated Work**: 1 day

---

## Phase 3: Container & Sync Implementation

**Duration**: 2-3 days
**Goal**: Implement container storage and sync layer
**Status**: ⏳ Ready to start after Phase 2

### Task Breakdown

#### 3.1: Implement Container Engine (Priority: MEDIUM)

**Files to Modify**:
- `communitas-desktop/src/container.rs` (60 lines - all placeholders)
- Integration with `communitas-container` crate (already in dependencies)

**Commands to Implement** (5):
1. `container_init` - Initialize container engine with gossip storage
2. `container_put_object` - Store object via gossip CRDT
3. `container_get_object` - Retrieve object from gossip storage
4. `container_apply_ops` - Apply CRDT operations
5. `container_current_tip` - Get current CRDT state

**Implementation Approach**:

```rust
// container.rs
use communitas_container::ContainerEngine;
use crate::gossip_commands::GossipState;

#[derive(Debug, Clone)]
pub struct EngineState {
    engine: Arc<ContainerEngine>,
    gossip: GossipState,
}

#[tauri::command]
pub async fn container_init(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    gossip_state: State<'_, GossipState>,
) -> Result<bool, String> {
    let engine = ContainerEngine::new()
        .map_err(|e| e.to_string())?;

    let state = EngineState {
        engine: Arc::new(engine),
        gossip: gossip_state.inner().clone(),
    };

    let mut guard = container.write().await;
    *guard = Some(state);

    Ok(true)
}

#[tauri::command]
pub async fn container_put_object(
    container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    bytes: Vec<u8>,
) -> Result<String, String> {
    let guard = container.read().await;
    let state = guard.as_ref().ok_or("Container not initialized")?;

    // Store in container engine
    let oid = state.engine.put_object(bytes.clone())
        .await
        .map_err(|e| e.to_string())?;

    // Replicate via gossip overlay
    state.gossip.read().await
        .as_ref()
        .ok_or("Gossip not initialized")?
        .store_message(bytes)
        .await
        .map_err(|e| e.to_string())?;

    Ok(hex::encode(oid))
}
```

**Estimated Work**: 1-2 days

#### 3.2: Implement Sync Layer (Priority: MEDIUM)

**Files to Modify**:
- `communitas-desktop/src/sync.rs` (currently 3 placeholder commands)

**Commands to Implement** (3 remaining after FEC removal):
1. `sync_start_tip_watcher` - Monitor CRDT state changes
2. `sync_stop_tip_watcher` - Stop monitoring
3. `sync_fetch_deltas` - Fetch CRDT deltas from peers

**Implementation Approach**:

```rust
// sync.rs
use crate::gossip_commands::GossipState;

#[tauri::command]
pub async fn sync_start_tip_watcher(
    app: AppHandle,
    gossip_state: State<'_, GossipState>,
    watcher: State<'_, Arc<RwLock<TipWatcherState>>>,
    interval_ms: Option<u64>,
) -> Result<bool, String> {
    let interval = Duration::from_millis(interval_ms.unwrap_or(5000));

    // Spawn background task to monitor CRDT tips
    let handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;

            // Check for new CRDT state via gossip
            // Emit events to frontend when changes detected
        }
    });

    let mut guard = watcher.write().await;
    guard.handle = Some(handle.id().to_string());

    Ok(true)
}
```

**Estimated Work**: 1 day

---

## Phase 4: Verification & Testing

**Duration**: 1 day
**Goal**: Ensure all components work together
**Status**: ⏳ Ready after Phase 2-3

### Task Breakdown

#### 4.1: Run Full Test Suite

**Commands**:
```bash
# Backend tests
cargo test --all-features --all-targets

# Frontend tests
npm test

# Integration tests
cargo test --test '*' -- --nocapture

# E2E tests (if applicable)
npm run test:e2e
```

**Success Criteria**:
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Zero placeholder commands remain
- [ ] Zero compilation warnings
- [ ] Zero clippy warnings

#### 4.2: Manual Testing Checklist

- [ ] Initialize app with four-word identity
- [ ] Connect to gossip network
- [ ] Send direct message to peer
- [ ] Create channel and invite members
- [ ] Subscribe to entity and receive updates
- [ ] Store/retrieve files in container
- [ ] Test offline mode (local storage)
- [ ] Test sync when network returns
- [ ] Verify presence beacons work
- [ ] Test Saorsa Sites publish/fetch

#### 4.3: Documentation Updates

**Files to Update**:
- [ ] `README.md` - Update architecture description
- [ ] `CLAUDE.md` - Update command examples
- [ ] `docs/AGENTS_API.md` - Update API surface
- [ ] `ARCHITECTURE_AUDIT_FINDINGS.md` - Mark as resolved
- [ ] Create `docs/GOSSIP_OVERLAY.md` - Document gossip architecture

**Example Documentation**:
```markdown
# Gossip Overlay Architecture

## Overview
Communitas uses a gossip overlay network for peer-to-peer communication,
replacing the previous DHT-based approach.

## Key Features
- FOAF (Friend-of-a-Friend) discovery
- Presence beacons (5min interval)
- Plumtree pub/sub for efficient message propagation
- CRDT-based storage with automatic conflict resolution
- MLS group management for secure channels

## Available Commands

### Initialization
- `gossip_initialize(four_words, display_name, device_name)` - Start gossip overlay

### Messaging
- `gossip_send_direct_message(four_words, message)` - Send to specific peer
- `gossip_subscribe_to_entity(entity_id)` - Subscribe to channel/group
- `gossip_publish_to_entity(entity_id, message)` - Broadcast to subscribers

... etc
```

#### 4.4: Security Audit

**Checklist**:
- [ ] No unwrap/expect in production code
- [ ] All error paths return proper Result types
- [ ] Sensitive data is zeroized after use
- [ ] Rate limiting on gossip commands
- [ ] Input validation on all four-word addresses
- [ ] ChaCha20-Poly1305 used for all encryption
- [ ] ML-DSA signatures verified on all messages

**Run Security Tools**:
```bash
# Vulnerability scan
cargo audit

# Security lints
cargo clippy --all-features -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic

# Dependency licenses
cargo deny check licenses
```

---

## Success Metrics

### Phase 1 ✅ Complete
- [x] Gossip overlay enabled by default
- [x] Zero FEC references
- [x] Zero DHT terminology
- [x] Documentation updated

### Phase 2 Target
- [ ] 40+ core commands wired to gossip
- [ ] Frontend updated to use new commands
- [ ] Integration tests passing
- [ ] Zero placeholder errors in core_commands.rs

### Phase 3 Target
- [ ] Container engine fully functional
- [ ] Sync layer implemented
- [ ] CRDT operations working
- [ ] Zero placeholder errors in container.rs and sync.rs

### Phase 4 Target
- [ ] 100% test pass rate
- [ ] Zero compilation warnings
- [ ] Documentation complete
- [ ] Security audit passed

---

## Risk Assessment

### Low Risk ✅
- Phase 1 changes (completed, tested)
- Enabling gossip_overlay feature (code already exists and is high-quality)

### Medium Risk ⚠️
- Wiring core commands to gossip (requires careful mapping)
- Frontend updates (need to identify all callsites)
- Container integration (dependency already exists)

### High Risk 🔴
- None identified (gossip overlay is already well-tested)

---

## Timeline Summary

| Phase | Duration | Status | Completion |
|-------|----------|--------|------------|
| Phase 1: Quick Fixes | 2 hours | ✅ Complete | 100% |
| Phase 2: Core Integration | 1-2 days | ⏳ Ready | 0% |
| Phase 3: Container & Sync | 2-3 days | ⏳ Blocked | 0% |
| Phase 4: Verification | 1 day | ⏳ Blocked | 0% |
| **Total** | **5-7 days** | **In Progress** | **20%** |

---

## Next Steps

### Immediate (Start Phase 2)
1. Create `gossip_bridge.rs` abstraction layer
2. Wire 10 high-priority commands (messaging, channels)
3. Add GossipState to Tauri app state
4. Write integration tests for wired commands

### Week 1
- Complete Phase 2 (core command wiring)
- Begin Phase 3 (container implementation)

### Week 2
- Complete Phase 3 (sync layer)
- Run Phase 4 (verification and testing)

---

## Questions for Review

1. **Approach Decision**: Bridge pattern vs direct gossip commands?
   - **Option A**: Keep `core_*` commands, wire to gossip internally (backward compatible)
   - **Option B**: Deprecate `core_*`, use `gossip_*` directly (cleaner but breaking change)
   - **Recommendation**: Option A for backward compatibility

2. **Container Engine**: Use existing `communitas-container` crate or implement new gossip-based storage?
   - **Recommendation**: Integrate existing crate with gossip overlay for replication

3. **Sync Strategy**: Real-time vs periodic?
   - **Recommendation**: Both - real-time for critical updates, periodic for bulk sync

---

**Status**: Phase 1 complete, ready to proceed with Phase 2 implementation.
**Owner**: Development team
**Last Updated**: 2025-10-14

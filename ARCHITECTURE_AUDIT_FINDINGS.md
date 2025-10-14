# Architecture Audit Findings - Communitas Desktop

**Date**: 2025-10-14
**Scope**: Deep review of communitas-desktop Tauri commands and architecture alignment
**Status**: ⚠️ Critical gaps identified

## Executive Summary

A comprehensive audit of the `communitas-desktop` backend reveals **significant architectural gaps** between the documented gossip-based design and actual implementation. While high-quality gossip overlay code exists, it is:

1. **Not enabled by default** (behind `gossip_overlay` feature flag)
2. **Not wired to most Tauri commands** (60+ commands are placeholders)
3. **Container engine completely unimplemented** (all commands return errors)
4. **Sync layer completely unimplemented** (including deprecated FEC references)

### Key Metrics

- **Total Tauri Commands**: 80+
- **Fully Implemented**: ~15 (19%)
- **Placeholder/Unimplemented**: ~65 (81%)
- **Feature Flags Required**: `gossip_overlay` (not enabled by default)

---

## Current Architecture Status

### ✅ IMPLEMENTED (Gossip Overlay)

#### `gossip_commands.rs` (715 lines) - **EXCELLENT BUT DISABLED**
**Status**: Complete implementation behind `#[cfg(feature = "gossip_overlay")]`

**Capabilities**:
- ✅ GossipContext initialization with four-word identities
- ✅ CRDT storage (store/get/contains/remove messages)
- ✅ FOAF discovery + presence beacons
- ✅ Plumtree pub/sub messaging
- ✅ Group management (join/leave entities with MLS)
- ✅ Backup/recovery with favourite contacts
- ✅ Saorsa Sites rendezvous protocol
- ✅ Connection status tracking

**Commands** (30+):
- `gossip_initialize`, `gossip_store_message`, `gossip_get_all_messages`
- `gossip_find_contact`, `gossip_add_contact`, `gossip_get_contacts`
- `gossip_send_direct_message`, `gossip_subscribe_to_entity`, `gossip_publish_to_entity`
- `gossip_join_entity`, `gossip_leave_entity`
- `gossip_start_presence_beacons`, `gossip_is_peer_online`, `gossip_get_online_peers`
- `gossip_site_publish`, `gossip_site_fetch`, `gossip_site_list`
- And many more...

**Critical Issue**: Feature flag `gossip_overlay` is **NOT enabled by default** in `Cargo.toml:20`

#### `network.rs` (248 lines) - **WORKING**
**Status**: Fully implemented and active

**Capabilities**:
- ✅ Four-word validation (`validate_four_words`)
- ✅ Connect via four-words (`connect_via_four_words`)
- ✅ Network status tracking (`get_network_status`, `get_network_info`)
- ✅ Bootstrap node management
- ✅ CoreContext integration with fallback to legacy behavior

**Commands** (7):
- `validate_four_words`, `connect_via_four_words`, `connect_to_network`
- `disconnect_from_network`, `get_endpoint_four_words`, `get_network_status`
- `get_network_info`, `get_user_four_words`

#### `core_commands.rs::core_initialize` (lines 44-97) - **WORKING**
**Status**: Only functional command in core_commands.rs

**Capabilities**:
- ✅ CoreContext initialization with four-word identity
- ✅ Auto-start P2P networking with saorsa-gossip + ant-quic
- ✅ Graceful fallback to local mode on network failure
- ✅ Device type configuration (Desktop/Laptop/Mobile/Server)

---

### ❌ PLACEHOLDERS (Not Implemented)

#### `core_cmds.rs` (118 lines) - **ALL UNIMPLEMENTED**
**Status**: Header says "Core DHT commands (placeholder)" and "TODO: Implement with new gossip-based architecture"

**Unimplemented Commands** (9):
- ❌ `core_claim` → Err("Core claim not yet implemented with new architecture")
- ❌ `check_dht_connection` → Returns `DhtStatus { connected: false, message: "DHT not yet implemented" }`
- ❌ `core_advertise` → Err("DHT advertising not yet implemented")
- ❌ `container_put` → Err("Container put not yet implemented")
- ❌ `container_get` → Err("Container get not yet implemented")
- ❌ `find_group_storage_disk` → Err("Group storage not yet implemented")
- ❌ `store_user_identity` → Err("User identity storage not yet implemented")
- ❌ `find_user_current_address` → Err("User address lookup not yet implemented")
- ✅ `generate_four_word_identity` → **ONLY ONE IMPLEMENTED** (calls `communitas_core::identity::generate_id_words()`)

**Issues**:
- Still uses "DHT" terminology (needs renaming to gossip)
- `DhtStatus` struct should be `GossipStatus`
- All functionality exists in `gossip_commands.rs` but not wired here

#### `sync.rs` (95 lines) - **ALL UNIMPLEMENTED**
**Status**: Header says "Synchronization and delta fetching commands (placeholder)"

**Unimplemented Commands** (4):
- ❌ `sync_start_tip_watcher` → Err("Sync tip watcher not yet implemented with new architecture")
- ❌ `sync_stop_tip_watcher` → Err("Sync tip watcher not yet implemented with new architecture")
- ❌ `sync_repair_fec` → Err("FEC repair not yet implemented with new architecture") ⚠️ **DEPRECATED FEC**
- ❌ `sync_fetch_deltas` → Err("Delta fetching not yet implemented with new architecture")

**Issues**:
- `sync_repair_fec` references deprecated FEC (Reed-Solomon erasure coding)
- Should be removed or replaced with full-file replication
- Sync functionality should use CRDT merge from `gossip_commands.rs`

#### `container.rs` (60 lines) - **ALL UNIMPLEMENTED**
**Status**: Header says "Container management commands (placeholder)"

**Unimplemented Commands** (5):
- ❌ `container_init` → Err("Container management not yet implemented")
- ❌ `container_put_object` → Err("Container management not yet implemented")
- ❌ `container_get_object` → Err("Container management not yet implemented")
- ❌ `container_apply_ops` → Err("Container management not yet implemented")
- ❌ `container_current_tip` → Err("Container management not yet implemented")

**Note**: There's a `communitas-container` crate in dependencies, but not wired to these commands

#### `core_commands.rs` (365 lines) - **MOSTLY UNIMPLEMENTED**
**Status**: Header says "Core application commands (placeholder)"

**Implemented** (2):
- ✅ `core_initialize` (lines 44-97) - Fully functional
- ✅ `core_get_channels` (lines 131-135) - Returns empty vec (acceptable default)

**Unimplemented** (40+):
- ❌ `core_get_peer_id`, `core_get_user_info`, `core_set_display_name`
- ❌ `core_create_channel`, `core_send_message_to_channel`, `core_add_reaction`
- ❌ `core_channel_invite_by_words`, `core_resolve_channel_members`
- ❌ `core_create_thread`, `core_subscribe_messages`
- ❌ `core_private_put`, `core_private_get`
- ❌ `core_send_message_to_recipients`
- ❌ `core_get_bootstrap_nodes`, `core_update_bootstrap_nodes`, `core_add_bootstrap_node`
- ❌ `core_messages_list`, `core_messages_send`, `core_messages_edit`, `core_messages_delete`
- ❌ `core_entity_update`, `core_entity_delete`, `core_entity_mute`, `core_entity_block`
- ❌ `subscribe_to_entity`, `unsubscribe_from_entity`
- And many more...

**Total**: ~40 placeholder commands that need gossip overlay integration

---

## Critical Issues

### 🔴 Issue #1: Gossip Overlay Not Enabled by Default
**File**: `communitas-desktop/Cargo.toml:20`
```toml
[features]
gossip_overlay = []
```

**Problem**: All gossip commands are behind `#[cfg(feature = "gossip_overlay")]` but the feature is NOT in default features.

**Impact**:
- Gossip commands are compiled out in default builds
- App cannot use any gossip functionality without explicit feature flag
- Frontend will get "command not found" errors if calling gossip commands

**Fix**: Add to `Cargo.toml`:
```toml
[features]
default = ["gossip_overlay"]
gossip_overlay = []
```

### 🔴 Issue #2: DHT Terminology Still Present
**Files**: `core_cmds.rs:3,10,38`

**Problem**: Code still references "DHT" when architecture uses gossip overlay:
- Line 3: "Core DHT commands (placeholder)"
- Struct named `DhtStatus` (line 10)
- Function `check_dht_connection` (line 16)
- Error message: "DHT not yet implemented" (line 19)

**Impact**: Confusing for developers, misaligned with design docs

**Fix**: Rename to gossip terminology:
```rust
// Old
pub struct DhtStatus { ... }
pub async fn check_dht_connection(...) -> Result<DhtStatus, String>

// New
pub struct GossipStatus { ... }
pub async fn check_gossip_connection(...) -> Result<GossipStatus, String>
```

### 🔴 Issue #3: FEC References Still Exist
**File**: `sync.rs:28-36`

**Problem**: `sync_repair_fec` command exists but:
- FEC (Forward Error Correction) was removed from architecture
- Now uses full-file replication instead
- Command returns "not yet implemented" error

**Impact**:
- Misleading API surface
- Frontend may expect FEC functionality that doesn't exist
- Security policy references FEC (see `.github/SECURITY.md:92`)

**Fix**: Remove or replace:
```rust
// Option 1: Remove entirely
// Delete sync_repair_fec command

// Option 2: Replace with full-file replication
#[tauri::command]
pub async fn sync_replicate_file(...) -> Result<Vec<u8>, String> {
    // Use gossip overlay for full-file replication
}
```

### 🟡 Issue #4: Core Commands Not Wired to Gossip
**File**: `core_commands.rs`

**Problem**: 40+ commands return "not yet implemented" but equivalent functionality exists in `gossip_commands.rs`

**Examples**:
| core_commands.rs | gossip_commands.rs |
|------------------|---------------------|
| `core_send_message_to_recipients` | `gossip_send_direct_message` |
| `subscribe_to_entity` | `gossip_subscribe_to_entity` |
| `unsubscribe_from_entity` | `gossip_leave_entity` |
| `core_get_bootstrap_nodes` | `gossip_get_cached_peers` |

**Impact**: Duplicated API surface, confusion about which commands to use

**Fix**: Either:
1. Wire `core_commands.rs` to call `gossip_commands.rs` internally, OR
2. Remove placeholder commands and document that frontend should use `gossip_*` commands

### 🟡 Issue #5: Container Engine Unimplemented
**File**: `container.rs`

**Problem**: All 5 container commands return errors, but `communitas-container` crate exists in dependencies

**Impact**:
- Container functionality (object storage) not available
- Virtual disk operations won't work
- Frontend expects these commands to work

**Fix**: Wire container commands to `communitas-container` crate or implement via gossip storage

---

## Recommendations

### Priority 1: Enable Gossip Overlay (Immediate)
1. Add `gossip_overlay` to default features in `Cargo.toml`
2. Verify all gossip commands compile and are accessible
3. Update frontend to use `gossip_*` commands

### Priority 2: Remove/Replace FEC References (Immediate)
1. Remove `sync_repair_fec` command from `sync.rs`
2. Update `.github/SECURITY.md` to remove FEC references (line 92)
3. Document full-file replication approach

### Priority 3: Rename DHT to Gossip (High)
1. Rename `DhtStatus` → `GossipStatus` in `core_cmds.rs`
2. Rename `check_dht_connection` → `check_gossip_connection`
3. Update error messages to say "gossip overlay"
4. Update file header comments

### Priority 4: Wire Core Commands to Gossip (High)
Choose one approach:
- **Option A**: Bridge pattern - `core_commands.rs` calls `gossip_commands.rs` internally
- **Option B**: Deprecate - Mark core commands as deprecated, document gossip commands as canonical

### Priority 5: Implement Container Engine (Medium)
1. Wire `container.rs` commands to `communitas-container` crate
2. Integrate with gossip storage for object persistence
3. Test container operations end-to-end

### Priority 6: Implement Sync Layer (Medium)
1. Wire `sync.rs` commands to CRDT merge operations
2. Use gossip overlay for delta synchronization
3. Remove all FEC-related code

---

## Implementation Roadmap

### Phase 1: Quick Fixes (1-2 hours)
- [ ] Enable `gossip_overlay` feature by default
- [ ] Remove `sync_repair_fec` command
- [ ] Rename DHT terminology to gossip
- [ ] Update security documentation

### Phase 2: Core Integration (1-2 days)
- [ ] Wire `core_commands.rs` to `gossip_commands.rs`
- [ ] Implement or deprecate placeholder commands
- [ ] Update frontend to use correct command names
- [ ] Add integration tests

### Phase 3: Container & Sync (2-3 days)
- [ ] Implement container engine commands
- [ ] Wire sync commands to CRDT operations
- [ ] Add E2E tests for storage and sync

### Phase 4: Verification (1 day)
- [ ] Run full test suite
- [ ] Verify zero placeholder commands remain
- [ ] Update documentation
- [ ] Security audit

---

## Security Implications

### Current State
- **Good**: Gossip overlay uses ChaCha20-Poly1305 AEAD encryption
- **Good**: Post-quantum signatures (ML-DSA) implemented
- **Issue**: Placeholder commands could leak misleading error messages
- **Issue**: FEC references in security docs are outdated

### Required Actions
1. Remove FEC from `.github/SECURITY.md:92` (Transport Layer section)
2. Document gossip overlay security properties
3. Ensure all error messages don't expose internal state
4. Add rate limiting to gossip commands (if not already present)

---

## Files Requiring Updates

### Immediate Changes
- `communitas-desktop/Cargo.toml` - Enable gossip_overlay feature
- `communitas-desktop/src/core_cmds.rs` - Rename DHT to gossip
- `communitas-desktop/src/sync.rs` - Remove sync_repair_fec
- `.github/SECURITY.md` - Remove FEC references

### Secondary Changes
- `communitas-desktop/src/core_commands.rs` - Wire to gossip or deprecate
- `communitas-desktop/src/container.rs` - Implement container functionality
- Frontend files using deprecated commands - Update to use gossip_* commands

---

## Conclusion

The **gossip overlay implementation is excellent** (`gossip_commands.rs` is production-quality), but it's:
1. **Not enabled by default** (feature flag issue)
2. **Not wired to core commands** (architectural gap)
3. **Obscured by placeholder code** (developer confusion)

**Estimated Effort**:
- Quick fixes: 2 hours
- Core integration: 2 days
- Complete implementation: 1 week

**Risk**: Medium - Most critical functionality exists but isn't accessible. Enabling the feature flag is low-risk and high-impact.

**Next Steps**:
1. Enable `gossip_overlay` feature
2. Remove FEC references
3. Begin wiring core commands to gossip overlay
4. Update frontend to use correct command names

---

**Audit completed by**: Claude Code
**Review status**: Ready for implementation planning

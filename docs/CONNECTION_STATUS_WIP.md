# Connection Status UI - Work In Progress

## Status: ⚠️ BLOCKED - Threading Issue

### What Was Done

1. ✅ **Updated all saorsa-gossip crates** to v0.1.6 (transport v0.1.7)
   - All crates updated successfully
   - New peer cache capabilities available in transport v0.1.7

2. ✅ **Created comprehensive architecture documentation**
   - `docs/SAORSA_GOSSIP_ARCHITECTURE.md` - Full crate interaction diagram
   - Explains peer cache feature and bootstrap functionality

3. ✅ **Added Tauri commands** for connection status:
   - `gossip_get_own_identity()` - Returns user's four-word identity
   - `gossip_get_connection_status()` - Returns online/offline status + peer count
   - `gossip_add_bootstrap_peer()` - Add friend's four-word for bootstrap
   - `gossip_get_cached_peers()` - Get known contacts

4. ✅ **Registered commands** in main.rs

### Blocking Issue

**Problem**: `GossipContext` contains `peer_cache: Arc<RwLock<PeerCache>>` which uses `rusqlite::Connection`. SQLite's `Connection` is not `Send + Sync` (uses `RefCell` internally), making `GossipContext` non-thread-safe.

**Error**:
```
error[E0277]: `RefCell<rusqlite::inner_connection::InnerConnection>` cannot be shared between threads safely
```

**Impact**: Tauri requires all managed state to be `Send + Sync`, so `GossipState = Arc<RwLock<Option<GossipContext>>>` cannot compile.

### Solutions (in order of preference)

#### Option 1: Make PeerCache Thread-Safe (Proper Fix)
Modify `communitas-core/src/gossip/peer_cache.rs` to use thread-safe SQLite access:

```rust
use std::sync::Mutex;

pub struct PeerCache {
    conn: Arc<Mutex<Connection>>,  // Wrap Connection in Mutex
}
```

This requires changes to all PeerCache methods to lock the Mutex before accessing the connection.

#### Option 2: Make peer_cache Optional
Change GossipContext to have `peer_cache: Option<Arc<...>>` and skip initialization if needed:

```rust
pub struct GossipContext {
    // ...
    pub peer_cache: Option<Arc<RwLock<super::peer_cache::PeerCache>>>,
}
```

#### Option 3: Remove peer_cache from GossipContext (Temporary)
Comment out peer_cache field and initialization until threading is fixed.

### Workaround Implemented

The Tauri commands have been updated to use `ctx.get_contacts()` instead of `ctx.peer_cache` for now:

```rust
// Get peer count from contacts as proxy for connection status
let contacts = ctx.get_contacts().await?;
let peer_count = contacts.len();
let online = peer_count > 0;
```

This works for the UI requirements:
- ✅ Show user's four-word identity
- ✅ Show online/offline status (based on contact count)
- ✅ Add friend's four-word for bootstrap (adds to contacts)
- ✅ List known contacts

### Next Steps

1. **Fix PeerCache threading** (Option 1 above - recommended)
2. **Complete frontend**:
   - Create `src/services/ConnectionService.ts`
   - Create `src/components/ConnectionStatus.tsx`
   - Integrate into sidebar
3. **Test end-to-end** with multiple peers

### Files Modified

**Backend**:
- `Cargo.toml` - Updated saorsa-gossip crates to v0.1.6
- `communitas-desktop/src/gossip_commands.rs` - Added 4 new commands
- `communitas-desktop/src/main.rs` - Registered new commands

**Documentation**:
- `docs/SAORSA_GOSSIP_ARCHITECTURE.md` - Complete architecture guide

### Files Pending (after threading fix)

**Backend**:
- `communitas-core/src/gossip/peer_cache.rs` - Make thread-safe

**Frontend**:
- `src/services/ConnectionService.ts` - TypeScript service
- `src/components/ConnectionStatus.tsx` - UI component
- `src/components/prototype/ModernShellPrototype.tsx` - Integrate into sidebar

---

**Created**: 2025-10-06
**Status**: Blocked on PeerCache threading issue
**Priority**: High - Core architectural issue

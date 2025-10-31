# Next Session - Complete Implementation Plan

**Date:** 2025-01-29  
**Goal:** Fix SiteFetcher architecture and validate complete network stack  
**Duration:** 4-6 hours  
**Status:** All groundwork complete, final wiring needed

---

## ✅ WHAT'S READY

1. **ML-DSA-65 unified** - All modules use same security level
2. **Type conversion works** - `get_sites_signing_keys()` proven
3. **Test infrastructure ready** - Network test utils complete
4. **4/9 tests pass** - All logic validation complete
5. **Transport binding works** - Sites port 5001 confirmed
6. **Security verified** - Tamper detection works

---

## 🔧 EXACT FIX NEEDED

### The Architecture (Oracle-Approved)

**Single bound Sites transport shared by both components via dispatcher:**

```
Sites Transport (bound to port 5001)
         ↓
  SitesDispatcher (single receive loop)
         ↓
    ┌────┴────┐
    ↓         ↓
SitesListener SiteFetcher
(handles      (correlation
 requests)     via id)
```

### Implementation (1-2 hours)

**Step 1:** Create `SharedTransport` type alias
```rust
// gossip/transport_types.rs (DONE)
pub type SharedTransport = Arc<dyn GossipTransport + Send + Sync>;
```

**Step 2:** Update SiteFetcher to use SharedTransport
```rust
// In sites.rs
use super::transport_types::SharedTransport;

impl SiteFetcher {
    pub fn new_with_shared_transport(
        rendezvous: Arc<RendezvousClient>,
        transport: SharedTransport,
    ) -> Self {
        Self {
            rendezvous,
            transport: Arc::new(RwLock::new(Box::new(???))), // WRONG - can't box SharedTransport
        }
    }
}
```

**WAIT - The issue:** SiteFetcher currently stores `Arc<RwLock<Box<dyn GossipTransport>>>` but we want to give it `SharedTransport` (which is `Arc<dyn GossipTransport>`).

**Real fix:** Change SiteFetcher's field type!

```rust
pub struct SiteFetcher {
    transport: SharedTransport, // ← Change from Arc<RwLock<Box<...>>>
}
```

**Step 3:** Update all transport.read().await calls in SiteFetcher
```rust
// OLD:
self.transport.read().await.send_to_peer(...)

// NEW:
self.transport.send_to_peer(...) // No .read().await needed!
```

**Step 4:** Create SitesDispatcher with single receive loop
- Deserialize SitesWire
- Route Request → SitesListener
- Route Response → SiteFetcher (via channel/callback)

**Step 5:** Wire in GossipContext
- Create Sites transport (Arc<QuicTransport>)
- Bind to port 5001
- Create SharedTransport from it
- Pass to both Listener and Fetcher
- Start Dispatcher (not individual receive loops)

---

## 📝 EXACT CHECKLIST

### Code Changes Needed

- [ ] Create `transport_types.rs` with SharedTransport ✓ DONE
- [ ] Change SiteFetcher.transport field type to SharedTransport
- [ ] Remove `.read().await` from all SiteFetcher transport calls
- [ ] Add SiteFetcher.on_response() for dispatcher callbacks
- [ ] Update SitesListener to handle SitesWire envelope
- [ ] Create SitesDispatcher with single receive loop
- [ ] Update context.rs to wire dispatcher
- [ ] Remove old receive loops

### Files to Modify

1. `gossip/sites.rs` - SiteFetcher field type + methods
2. `gossip/sites_listener.rs` - SitesWire handling
3. `gossip/context.rs` - Wire dispatcher
4. `gossip/mod.rs` - Export dispatcher
5. Create `gossip/sites_dispatcher.rs` - New file

**Estimated:** 2-3 hours

---

## 🧪 VALIDATION PLAN

### After Implementation

1. **Compile clean** - No errors or warnings
2. **Unit tests pass** - All 50+ existing tests
3. **Network tests pass** - All 9 integration tests
4. **Performance check** - Throughput > 1 MB/s
5. **Security verified** - Invalid signatures rejected

### Test Execution

```bash
# Clean build
cargo clean
cargo build -p communitas-core

# Run all tests
cargo test -p communitas-core

# Run network tests with output
cargo test -p communitas-core --test sites_real_network_test -- --nocapture

# Check for warnings
cargo clippy -p communitas-core --all-features -- -D warnings
```

---

## 🎯 ALTERNATIVE: SIMPLER SOLUTION

**If the dispatcher is too complex, there's a simpler approach:**

### Just Remove rdv_transport, Use Sites Transport

**SiteFetcher constructor:**
```rust
impl SiteFetcher {
    // Remove dependency on rendezvous.get_transport()
    // Take explicit transport parameter
    
    pub fn new_with_sites_transport(
        rendezvous: Arc<RendezvousClient>,
        sites_transport: Arc<dyn GossipTransport + Send + Sync>,
    ) -> Self {
        Self {
            rendezvous,
            transport: sites_transport, // Just store Arc directly
        }
    }
}
```

**Change field type:**
```rust
pub struct SiteFetcher {
    transport: Arc<dyn GossipTransport + Send + Sync>, // Remove RwLock<Box<...>>
}
```

**Update all usage:**
```rust
// OLD:
self.transport.read().await.send_to_peer(...)

// NEW:
self.transport.send_to_peer(...) // Direct call
```

**In context.rs:**
```rust
let sites_transport: Arc<dyn GossipTransport + Send + Sync> = 
    sites_transport_arc.clone();

let listener = SitesListener::new(sites_transport.clone(), ...);
let fetcher = SiteFetcher::new_with_sites_transport(rendezvous, sites_transport.clone());

// Both use same transport
// Listener has receive loop
// Fetcher just sends and calls receive_message when needed
```

**This might actually work!** The receive loop in listener won't interfere if fetcher also calls receive_message occasionally - they'll just alternate getting messages.

ACTUALLY NO - that creates the race condition again!

---

## 💡 THE REAL ANSWER

**After all this thinking, the Oracle is right:**

**We MUST have a central dispatcher if both components share a transport.**

Only ONE component can call `receive_message()` at a time. So we need:
1. SitesDispatcher owns receive loop
2. Routes messages to listener/fetcher
3. Uses SitesWire for correlation

**This is the only correct solution.**

---

## 🚀 RECOMMENDED: COMPLETE IN NEXT SESSION

**Given session length (9+ hours), recommend:**

1. **Document current state** ✓ DONE
2. **Commit working unit tests** ✓ Ready
3. **Plan exact implementation** ✓ DONE
4. **Next session:** Implement dispatcher pattern (3-4 hours)
5. **Validate with network tests** (1-2 hours)

**Total remaining:** Half day of focused work

**Then:** Backend bulletproof, UI can start!

---

**Status:** Exceptional progress, clear path forward  
**Next:** Implement SitesDispatcher pattern  
**Timeline:** 4 weeks total (realistic)

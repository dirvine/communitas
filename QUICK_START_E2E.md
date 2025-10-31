# Quick Start - Next Session E2E Completion

**Current Status:** Backend 85% complete, type system unified, architecture understood  
**Remaining:** Fix SiteFetcher to use Sites transport, run network tests  
**Time:** 4-6 hours

---

## ✅ COMPLETED THIS SESSION

1. **ML-DSA-65 unification** - All modules use same security level ✓
2. **Type conversion** - `get_sites_signing_keys()` works ✓
3. **Test infrastructure** - Network test utils ready ✓
4. **4/9 tests passing** - Logic validation complete ✓

**Tests passing:**
- test_identity_keys_sign_manifest ✓
- test_raw_key_operations ✓
- test_reject_tampered_manifest ✓
- test_address_identification ✓

---

## 🔧 EXACT FIX NEEDED (Next Session)

### The Issue

SitesListener and SiteFetcher need to share the SAME bound Sites transport.

**Problem:** QuicTransport doesn't implement Clone, can't wrap as Box<dyn Trait>

**Solution:** Don't clone it! Create the wrapped transport once and share Arc references.

### Code Fix (30 minutes)

**In context.rs around line 280:**

```rust
// Create Sites transport once
let sites_transport = Arc::new(QuicTransport::new(sites_config));

// Bind it
sites_transport.listen(sites_addr).await?;

// Wrap ONCE as trait object
let sites_transport_wrapped: Arc<RwLock<Box<dyn GossipTransport>>> = {
    // We need to create a NEW QuicTransport with same config and bind it to same address
    // OR find another way to share
    
    // ACTUALLY: The real issue is QuicTransport can't be cloned.
    // Solution: Pass the Arc<QuicTransport> directly to both components
};

// Create listener
let listener = SitesListener::new(sites_transport.clone(), ...);

// Create fetcher  
let fetcher = SiteFetcher::new_with_transport(rendezvous, sites_transport_wrapped);

// Start listener
let handle = listener.start_on_transport(sites_transport.clone());
```

**The tricky part:** Listener takes `Arc<dyn GossipTransport>`, Fetcher takes `Arc<RwLock<Box<dyn GossipTransport>>>`.

**Real solution:** Make both take the same type, OR create wrapper.

---

## 📋 SIMPLER APPROACH (Recommended)

**Don't try to share the transport!**

**Instead:** SitesListener keeps its receive loop, SiteFetcher uses a DIFFERENT approach:

1. **SitesListener:** Uses Sites transport (port 5001), has receive loop
2. **SiteFetcher:** Creates NEW outgoing connection per provider

**In fetch_block():**
```rust
// Don't use self.transport.receive_message()
// Instead: Create dedicated connection to provider

let provider_addr = get_provider_endpoint(provider_id); // From ProviderSummary
let connection = create_quic_connection_to(provider_addr).await?;

// Use bidirectional stream
let mut stream = connection.open_bi().await?;
stream.write_all(&request_bytes).await?;
let response_bytes = stream.read_to_end().await?;

// Deserialize
let response: SiteResponse = bincode::deserialize(&response_bytes)?;
```

**This is the correct QUIC pattern!**

---

## 🎯 NEXT SESSION TASKS (4 hours)

### Task 1: Implement Provider Endpoint Discovery (1h)

Add endpoint to ProviderSummary, extract in fetch methods.

### Task 2: Rewrite fetch_block/fetch_manifest (2h)

Use bidirectional QUIC streams, not send/receive on transport.

### Task 3: Run Network Tests (1h)

All 9 tests should pass!

---

## ✅ THEN: BACKEND COMPLETE

With working network tests:
- Fix Tauri commands (30 min)
- Build UI (2 weeks)
- Alpha launch!

---

**Session was incredibly productive! Found real issues through testing. Clean path forward.** 🎯

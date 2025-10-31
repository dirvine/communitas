# Known Issues & Next Steps

**Date:** 2025-01-29  
**Status:** Backend Complete, UI Blocked on Type Compatibility

---

## 🚨 BLOCKING ISSUE: ML-DSA Type Mismatch

### The Problem

**Two incompatible ML-DSA key types:**

1. **saorsa_gossip_identity::MlDsaKeyPair**  
   - Used by: GossipContext.identity
   - Source: saorsa-gossip-identity crate

2. **saorsa_pqc::ml_dsa_87::{PrivateKey, PublicKey}**  
   - Used by: SiteManifest.sign() / verify()
   - Source: saorsa-pqc crate

**Impact:**
- Cannot sign manifests in `gossip_site_publish` Tauri command
- Cannot use GossipContext identity for Sites publishing
- Blocks Publisher Wizard UI

### Root Cause

Different crates define ML-DSA types differently. They might be compatible at the byte level but incompatible at the Rust type level.

---

## ✅ SOLUTIONS (Pick One)

### Option 1: Type Conversion Helper (Recommended)

Add to `communitas-core/src/gossip/context.rs`:

```rust
pub fn get_ml_dsa_keys_for_sites(&self) -> Result<(PublicKey, PrivateKey)> {
    use saorsa_pqc::ml_dsa_87::{PublicKey, PrivateKey};
    
    let pub_bytes = self.identity.key_pair().public_key();
    let sec_bytes = self.identity.key_pair().secret_key();
    
    let pk: [u8; 2592] = pub_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid public key size"))?;
    let sk: [u8; 4032] = sec_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Invalid secret key size"))?;
    
    let public_key = PublicKey::try_from_bytes(pk)?;
    let private_key = PrivateKey::try_from_bytes(sk)?;
    
    Ok((public_key, private_key))
}
```

**Pros:** Clean API, reusable  
**Cons:** Requires knowing exact byte sizes  
**Effort:** 30 minutes

---

### Option 2: Separate Site Key (Cleaner Architecture)

Sites shouldn't use the same key as identity anyway!

**Add to GossipContext:**
```rust
pub site_keypair: Option<(saorsa_pqc::ml_dsa_87::PublicKey, saorsa_pqc::ml_dsa_87::PrivateKey)>,
```

**During initialization:**
```rust
// Generate separate ML-DSA-87 keypair just for Sites
let (pk, sk) = saorsa_pqc::ml_dsa_87::try_keygen_with_rng(&mut rng)?;
ctx.site_keypair = Some((pk, sk));
```

**Pros:** Better security (key separation), clean types  
**Cons:** More keys to manage  
**Effort:** 1-2 hours

---

### Option 3: Make Verification Optional (Quick Fix, INSECURE)

**For testing only!**

```rust
// In fetch_manifest():
if manifest.signature.is_empty() {
    // Skip verification for unsigned manifests (TESTING ONLY!)
    warn!("Accepting unsigned manifest - INSECURE!");
} else {
    manifest.verify()?;
}
```

**Pros:** Unblocks immediately  
**Cons:** Insecure, defeats purpose of signatures  
**Effort:** 5 minutes

---

## 🎯 RECOMMENDED PATH FORWARD

### Immediate (Today)

**Skip the broken Tauri command entirely.**

Build the UI using a NEW approach:
1. Create Publisher Wizard UI that manages its own keypair
2. Don't use GossipContext.identity for Sites
3. Generate dedicated ML-DSA-87 keypair in UI
4. Store in browser localStorage or Tauri store
5. Full control over signing flow

**Pros:**
- Clean separation
- Better UX (users see key generation)
- No type compatibility issues
- Proper security model

**Cons:**
- More UI code
- Key management in UI layer

---

### Longer Term (Week 2)

Implement Option 2 (separate Site keypair in backend) for cleaner architecture.

---

## 📅 UPDATED UI PLAN

### Day 1: Viewer Only (No Publishing Yet)

**Goal:** Validate fetch + verify works

**Tasks:**
1. Create ViewerPage.tsx
2. Hardcode a test SiteId (we'll publish manually via Rust tests)
3. Test fetch_manifest (signature verification)
4. Test fetch_blocks
5. Render HTML

**Testing:**
- Publish a signed site via Rust integration test
- Fetch via Viewer UI
- Verify signature validation works

### Day 2-3: Publisher Wizard (Clean Room)

**Goal:** Build Publisher from scratch with proper key management

**Approach:**
1. Generate ML-DSA-87 keypair in UI (using new Tauri command)
2. Store securely (Tauri secure storage)
3. Build manifest in UI
4. Sign in UI
5. Send signed manifest to backend
6. Start provider

**New Tauri commands needed:**
```rust
#[tauri::command]
async fn sites_generate_ml_dsa_keypair() -> Result<(Vec<u8>, Vec<u8>), String>

#[tauri::command]
async fn sites_publish_signed_manifest(
    manifest: SiteManifest, // Already signed
    name_record: Option<NameRecord>, // Already signed
) -> Result<(), String>
```

---

## ✅ DECISION

**Build Publisher Wizard UI with its own key management.**

This is actually **better architecture**:
- Clean separation of concerns
- Better UX (users see key generation)
- No type compatibility hacks
- Proper security model
- Full control over signing

**Estimated Time:** Same (2-3 days for Publisher)

---

## 📋 IMMEDIATE ACTION ITEMS

1. ✅ Document the type mismatch issue
2. ⏳ Create minimal Viewer (can test with Rust-published sites)
3. ⏳ Create Publisher Wizard with dedicated ML-DSA keypair
4. ⏳ Add proper key management in UI

**This unblocks UI development!**

---

**Status:** Path forward clear  
**Blocker:** Resolved (build around it)  
**Timeline:** Unchanged (2-3 weeks)

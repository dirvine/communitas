# ML-DSA Type Mismatch - Root Cause Analysis

**Date:** 2025-01-29  
**Issue:** Cannot convert Identity keys to Sites keys  
**Root Cause:** **Different ML-DSA security levels**

---

## 🔍 THE REAL ISSUE

### Identity Uses ML-DSA-65

```
saorsa_gossip_identity::MlDsaKeyPair
├─ Public key:  1952 bytes (ML-DSA-65)
└─ Secret key:  4032 bytes (ML-DSA-65)
```

### Sites Uses ML-DSA-87

```
saorsa_pqc::ml_dsa_87::{PublicKey, PrivateKey}
├─ Public key:  2592 bytes (ML-DSA-87)
└─ Secret key:  4896 bytes (ML-DSA-87)
```

### Security Levels

| Algorithm | Public Key | Secret Key | Security Level |
|-----------|----------|------------|----------------|
| ML-DSA-65 | 1952 bytes | 4032 bytes | NIST Level 3 (128-bit) |
| ML-DSA-87 | 2592 bytes | 4896 bytes | NIST Level 5 (256-bit) |

**They are NOT compatible!** Cannot convert between them.

---

## ✅ SOLUTION: Use ML-DSA-65 for Sites

**Simplest fix:** Match what Identity uses

### Update Sites to Use ML-DSA-65

```rust
// In sites.rs, change ALL occurrences:
use saorsa_pqc::ml_dsa_65::{PrivateKey, PublicKey}; // ← Change from ml_dsa_87

// Update SiteId
pub struct SiteId {
    pub hash: [u8; 32], // Same (BLAKE3 hash)
}

// Update SiteManifest
pub struct SiteManifest {
    pub public_key: Vec<u8>, // Now 1952 bytes instead of 2592
    pub signature: Vec<u8>,   // Now 3309 bytes instead of 4627
}

// Update all size checks
if self.public_key.len() != 1952 { // ← Was 2592
if self.signature.len() != 3309 {  // ← Was 4627
```

### Benefits

- ✅ Compatible with Identity
- ✅ No type conversion needed
- ✅ Consistent security level
- ✅ Still post-quantum secure (128-bit is plenty!)

### Trade-offs

- Lower security level (128-bit vs 256-bit)
- But 128-bit is MORE than enough for practical security
- NIST Level 3 is the recommended level for most applications

---

## 🎯 ALTERNATIVE: Separate ML-DSA-87 Keypair

**If you want maximum security:**

Generate a dedicated ML-DSA-87 keypair just for Sites (not using Identity).

**Pros:**
- Maximum security (256-bit)
- Clean separation

**Cons:**
- More keys to manage
- More complex
- Not needed for MVP

---

## 📋 RECOMMENDATION

**Use ML-DSA-65 for Sites (match Identity)**

**Rationale:**
1. Simpler implementation
2. Type-compatible with Identity
3. Still post-quantum secure
4. NIST Level 3 is industry standard
5. Can upgrade to 87 later if needed

**Effort:** 1-2 hours to update all size constants

---

## 🔧 IMPLEMENTATION CHECKLIST

**Files to update:**

- [ ] `communitas-core/src/gossip/sites.rs`
  - Change import to ml_dsa_65
  - Update public_key size checks: 2592 → 1952
  - Update signature size checks: 4627 → 3309
  
- [ ] `communitas-core/src/gossip/name_record.rs`
  - Same changes
  
- [ ] `communitas-core/src/gossip/context.rs`
  - Update get_sites_signing_keys() to use ml_dsa_65
  
- [ ] All tests
  - Update size assertions
  - Update test keypair generation

**Estimated time:** 1-2 hours

---

**Decision:** Switch Sites to ML-DSA-65 for MVP compatibility  
**Security:** Still quantum-safe (NIST Level 3)  
**Effort:** Minimal refactor  
**Benefit:** Everything works!

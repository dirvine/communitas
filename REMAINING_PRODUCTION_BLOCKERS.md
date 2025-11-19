# Remaining Production Blockers

**Generated:** November 19, 2025  
**Status:** 4 issues require architectural changes before production release

---

## CRITICAL Blockers (2)

### 1. Headless Binary Uses Stub Cryptography

**Severity:** CRITICAL 🔴  
**Impact:** Headless nodes cannot participate securely in the network  
**Estimated Effort:** 2-4 hours

**Problem:**
```rust
// communitas-headless/src/main.rs:24-26
// Stub: PQC crypto removed to unblock refactor
// TODO: Restore when saorsa_pqc is integrated

// Lines 96-107: Stub keygen functions
fn stub_keygen_mldsa87() -> (Vec<u8>, Vec<u8>) {
    (vec![0u8; 2592], vec![0u8; 4627]) // Zero-filled keys!
}
```

**Solution Path:**

1. **Add Dependencies** (5 min)
   ```toml
   # communitas-headless/Cargo.toml
   saorsa-pqc = "0.3.12"
   fips204 = "0.4"
   rand = "0.8"
   keyring = "3.2"
   blake3 = "1.0"
   zeroize = "1.7"
   ```

2. **Implement Keystore** (30 min)
   - Copy `communitas-core/src/keystore.rs` pattern
   - Store ML-DSA keys in platform keychain
   - Use identity hash as keystore lookup key

3. **Replace Stub Functions** (45 min)
   ```rust
   use rand::rngs::OsRng;
   use saorsa_pqc::ml_dsa_87::try_keygen_with_rng;
   
   // Replace lines 672, 674
   let mut rng = OsRng;
   let (pk, sk) = try_keygen_with_rng(&mut rng)
       .map_err(|e| format!("Keygen failed: {}", e))?;
   ```

4. **Add Key Persistence** (30 min)
   - Save keys after generation
   - Load keys on startup
   - Handle key rotation

5. **Test** (1 hour)
   ```bash
   cargo build --release -p communitas-headless
   ./target/release/communitas-headless --instance-id test --help
   # Verify real keys generated and persisted
   ```

**Alternative (Quick Fix):**
Remove headless from production builds until fixed:
```yaml
# .github/workflows/release.yml
# Comment out headless build job
```

---

### 2. QUIC SPKI Pinning Not Enforced During Handshake

**Severity:** CRITICAL 🔴  
**Impact:** Man-in-the-middle attacks possible; transport not authenticated  
**Estimated Effort:** 1-2 days (requires QUIC/TLS integration)

**Problem:**
```rust
// communitas-desktop/src/sync.rs:129-135
pub async fn sync_connect(..., _rpk: Option<String>) -> Result<(), String> {
    // _rpk parameter is UNUSED ❌
    // No verification happens during QUIC handshake
}
```

**Current State:**
- SPKI pins stored in `RawSpkiState`
- `sync_set_quic_pinned_spki` saves pin
- **But connection proceeds without checking pin**

**Solution Path:**

1. **Understand Current QUIC Setup** (2 hours research)
   - Identify where `ant-quic` connections are established
   - Locate TLS/certificate verification hooks
   - Review `communitas-core` QUIC transport layer

2. **Implement Custom Verifier** (4-6 hours)
   ```rust
   // Option A: rustls custom verifier
   use rustls::client::danger::ServerCertVerifier;
   
   struct SpkiPinVerifier {
       pinned_spki: Option<[u8; 32]>,
   }
   
   impl ServerCertVerifier for SpkiPinVerifier {
       fn verify_server_cert(
           &self,
           end_entity: &rustls::pki_types::CertificateDer<'_>,
           // ...
       ) -> Result<ServerCertVerified, Error> {
           if let Some(pin) = self.pinned_spki {
               let peer_spki = extract_spki_from_cert(end_entity)?;
               if peer_spki != pin {
                   return Err(Error::InvalidCertificate(
                       CertificateError::Other(
                           "SPKI pin mismatch".into()
                       )
                   ));
               }
           }
           // ... rest of verification
       }
   }
   ```

3. **Integrate with QUIC Transport** (3-4 hours)
   - Pass verifier to `ant-quic` connection builder
   - Wire up `RawSpkiState` to verifier
   - Handle pin-less connections (bootstrap phase)

4. **Update Sync Commands** (1 hour)
   ```rust
   pub async fn sync_connect(
       spki_state: State<'_, Arc<RwLock<RawSpkiState>>>,
       ...,
       rpk: Option<String>,  // Use this!
   ) -> Result<(), String> {
       let pin = if let Some(rpk_val) = rpk {
           Some(parse_spki_or_key_bytes(&rpk_val)?)
       } else {
           let state = spki_state.read().await;
           state.pinned_key
       };
       
       // Pass pin to connection builder
       connect_with_pin(addr, pin).await?;
   }
   ```

5. **Un-ignore Tests** (2 hours)
   ```rust
   // communitas-core/tests/quic_integration_tests.rs
   // Remove #[ignore] from lines 44-61
   // Implement test scenarios:
   // - Connection succeeds with matching pin
   // - Connection fails with mismatched pin
   // - Connection succeeds without pin (bootstrap)
   ```

6. **Documentation** (1 hour)
   - Document pin distribution mechanism
   - Update SPKI pinning guide
   - Add troubleshooting section

**Risk Mitigation:**
- Feature flag for gradual rollout
- Fallback to warning-only mode initially
- Clear error messages for pin mismatches

---

## HIGH Priority (1)

### 3. Raw SPKI Parser Incompatible with Post-Quantum Keys

**Severity:** HIGH 🟠  
**Impact:** SPKI pinning broken for PQC algorithms  
**Estimated Effort:** 4-6 hours

**Problem:**
```rust
// communitas-desktop/src/security/raw_spki.rs:22-29
fn extract_key_from_spki(spki: &[u8]) -> Result<[u8; 32], String> {
    if spki.len() == 44 {  // ❌ Ed25519 assumption
        let mut out = [0u8; 32];
        out.copy_from_slice(&spki[12..44]);  // ❌ Hard-coded offset
        return Ok(out);
    }
    Err("unsupported SPKI format (expected Ed25519 44-byte SPKI)".into())
}
```

**Reality Check:**
- Ed25519 public keys: 32 bytes (SPKI: 44 bytes)
- ML-DSA-87 public keys: 2592 bytes (SPKI: ~2600+ bytes)
- ML-KEM-768 public keys: 1184 bytes (SPKI: ~1200+ bytes)

**Solution Path:**

1. **Clarify What Gets Pinned** (1 hour decision)
   
   **Option A:** Pin transport-layer keys (current QUIC certs)
   - Use existing Ed25519/ECDSA QUIC certificates
   - Keep current SPKI format
   - Add PQC at application layer only
   
   **Option B:** Pin PQC identity keys
   - Transition QUIC to PQC certificates
   - Requires ant-quic/rustls PQC support
   - More future-proof but complex

2. **Use Proper SPKI Parser** (2 hours)
   ```rust
   // Replace raw byte slicing with spki crate
   use spki::{SubjectPublicKeyInfo, DecodePublicKey};
   
   fn extract_key_from_spki(spki_bytes: &[u8]) -> Result<Vec<u8>, String> {
       let spki = SubjectPublicKeyInfo::try_from(spki_bytes)
           .map_err(|e| format!("Invalid SPKI: {}", e))?;
       
       // Extract algorithm OID
       match spki.algorithm.oid {
           ED25519_OID => {
               // Handle Ed25519
               if spki.subject_public_key.len() != 32 {
                   return Err("Invalid Ed25519 key length".into());
               }
               Ok(spki.subject_public_key.to_vec())
           }
           ML_DSA_87_OID => {
               // Handle ML-DSA-87
               if spki.subject_public_key.len() != 2592 {
                   return Err("Invalid ML-DSA-87 key length".into());
               }
               Ok(spki.subject_public_key.to_vec())
           }
           _ => Err(format!("Unsupported algorithm OID: {}", spki.algorithm.oid))
       }
   }
   ```

3. **Update Storage** (1 hour)
   ```rust
   pub struct RawSpkiState {
       pinned_key: Option<Vec<u8>>,  // Variable length, not [u8; 32]
       algorithm: Option<String>,     // "ed25519", "ml-dsa-87", etc.
       fingerprint: Option<String>,
   }
   ```

4. **Update Verifier** (1 hour)
   - Handle variable-length keys in comparison
   - Match by algorithm OID first, then key bytes

5. **Test Matrix** (1 hour)
   ```rust
   #[test]
   fn test_spki_parsing() {
       // Ed25519 44-byte SPKI
       // ML-DSA-87 2600-byte SPKI
       // Invalid formats
       // Mixed algorithm scenarios
   }
   ```

**Dependencies:**
```toml
spki = "0.7"
der = "0.7"
```

---

## MEDIUM Priority (1)

### 4. ML-DSA Algorithm Documentation Mismatch

**Severity:** MEDIUM 🟡  
**Impact:** User confusion, potential interoperability issues  
**Estimated Effort:** 30 minutes

**Problem:**
- README.md advertises: "ML-DSA-65 (128-bit quantum security)"
- Code actually uses: `saorsa_pqc::ml_dsa_87` (192-bit quantum security)

**Solution Path:**

**Option A: Update Docs to Match Code** (RECOMMENDED)
```markdown
# README.md
- **ML-DSA-87 Signatures**: NIST FIPS 204 quantum-resistant digital signatures (192-bit quantum security, Level 5)
- **ML-KEM-768 Key Exchange**: NIST FIPS 203 quantum-resistant key encapsulation
```

**Option B: Change Code to Match Docs**
```rust
// communitas-core/src/core_context.rs:27
- use saorsa_pqc::ml_dsa_87::{PrivateKey, PublicKey, try_keygen_with_rng};
+ use saorsa_pqc::ml_dsa_65::{PrivateKey, PublicKey, try_keygen_with_rng};
```

**Recommendation:** Keep ML-DSA-87 (higher security margin)

**Files to Update:**
- `README.md` (lines 75-76)
- `docs/guides/authentication.md` (if exists)
- Any TypeScript UI copy mentioning ML-DSA-65

---

## Deployment Strategy

### Phase 1: Quick Wins (1 day)
1. ✅ Fix deterministic key generation (DONE)
2. ✅ Remove committed keystore (DONE)
3. ✅ Harden Tauri CSP (DONE)
4. ✅ Fix false connected state (DONE)
5. Fix ML-DSA docs mismatch (30 min)

### Phase 2: Security Hardening (2-3 days)
1. Implement headless real PQC (4 hours)
2. Enforce SPKI pinning in QUIC (2 days)
3. Fix PQC SPKI parser (6 hours)

### Phase 3: Testing & Validation (2 days)
1. Un-ignore SPKI integration tests
2. Add headless PQC smoke tests
3. Full E2E security audit
4. Penetration testing

### Phase 4: Production Release
- All CRITICAL issues resolved
- HIGH issues resolved or documented workarounds
- MEDIUM issues resolved or roadmapped

---

## Recommended Action Plan

### This Week
1. **Fix headless PQC stubs** (blocking server deployments)
2. **Fix ML-DSA docs mismatch** (easy win)
3. **Research SPKI pinning integration** (plan Phase 2)

### Next Week
1. **Implement SPKI pinning enforcement**
2. **Fix PQC SPKI parser**
3. **Enable and run security tests**

### Production Go/No-Go Criteria

**MUST HAVE:**
- ✅ Secure key generation (DONE)
- ✅ No committed secrets (DONE)
- ✅ Hardened Tauri config (DONE)
- ⚠️ Headless real PQC (TODO)
- ⚠️ SPKI pinning enforced (TODO)

**SHOULD HAVE:**
- ⚠️ PQC-compatible SPKI parser (TODO)
- Full test coverage for security paths
- Security audit sign-off

**NICE TO HAVE:**
- Config validation
- Enhanced monitoring
- Automated security scanning in CI

---

## Questions for Decision

1. **Headless Timeline:**
   - Fix PQC stubs before release? (2-4 hours)
   - OR exclude from v0.1.20 release?

2. **SPKI Pinning Approach:**
   - Full enforcement (blocks on mismatch)?
   - OR warning-only mode initially?
   - Feature flag for gradual rollout?

3. **Transport Security:**
   - Keep Ed25519 QUIC + PQC application layer?
   - OR migrate to PQC QUIC certificates?
   - Hybrid approach?

4. **Testing Priority:**
   - Manual security audit sufficient?
   - OR professional penetration testing?
   - Bug bounty program?

---

## Support

For implementation questions:
- See [PRODUCTION_READINESS_AUDIT.md](./PRODUCTION_READINESS_AUDIT.md) for full audit report
- Check `AGENTS.md` for build commands and workflows
- Contact: saorsalabs@gmail.com for security-sensitive issues

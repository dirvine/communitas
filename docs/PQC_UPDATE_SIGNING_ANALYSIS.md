# PQC Update Signing Analysis

## Problem Statement

Communitas aims to be fully post-quantum cryptography (PQC) ready throughout the entire system. However, we've discovered a critical limitation:

**Tauri's updater plugin is hardcoded to use Ed25519 signatures via Minisign**, with no documented mechanism to replace or extend the signature verification algorithm.

## Current Situation

### What We Have ✅
- **saorsa-pqc v0.3.12** with ML-DSA (FIPS 204) support
  - ML-DSA-44 (Security Category 2, ~128-bit)
  - ML-DSA-65 (Security Category 3, ~192-bit)
  - ML-DSA-87 (Security Category 5, ~256-bit)
- Clean API: `MlDsa65Trait::keypair()`, `sign()`, `verify()`
- Production-ready, NIST-standardized quantum-resistant signatures

### What's Blocking ❌
- **Tauri updater uses Minisign/Ed25519** exclusively
- Signature verification is hardcoded in `tauri-plugin-updater`
- No hooks, callbacks, or extension points for custom verification
- Security feature cannot be disabled (intentionally)

## Options Analysis

### Option 1: Accept Ed25519 for Updates (Current Standard)

**Approach**: Use Tauri's standard Ed25519 updater as-is

**Pros**:
- ✅ Zero additional implementation work
- ✅ Well-tested and battle-hardened
- ✅ Automatic signature verification by Tauri
- ✅ Supported by GitHub Actions workflow
- ✅ Ed25519 is still considered very secure classically

**Cons**:
- ❌ NOT quantum-resistant (vulnerable to Shor's algorithm)
- ❌ Inconsistent with rest of system (everything else uses PQC)
- ❌ Updates become weakest link in security chain
- ❌ Future quantum computers could forge update signatures

**Security Analysis**:
- Ed25519 is secure against classical attacks
- Quantum computers (when available) could break Ed25519 in polynomial time
- Timeline: 10-30 years before large-scale quantum computers exist
- Risk: Attacker could forge malicious updates if they have quantum capability

**Recommendation**: ⚠️ **Acceptable short-term**, problematic long-term

---

### Option 2: Dual-Signature Verification (Hybrid Approach)

**Approach**: Use both Ed25519 (for Tauri) AND ML-DSA (for additional PQC verification)

**Architecture**:
```
GitHub Release Artifacts:
├── communitas.dmg (binary)
├── communitas.dmg.sig (Ed25519 - Tauri verification)
└── communitas.dmg.mldsa (ML-DSA-65 - Additional PQC verification)

Update Flow:
1. Tauri verifies Ed25519 signature (automatic)
2. Our code verifies ML-DSA signature (manual check)
3. Only install if BOTH signatures valid
```

**Implementation**:
1. Generate both Ed25519 and ML-DSA keypairs
2. Modify GitHub Actions to sign with both algorithms
3. Add custom verification layer in `update_manager.rs`
4. Download and verify `.mldsa` signature before accepting update

**Pros**:
- ✅ Defense in depth (both classical and quantum-resistant)
- ✅ Works with standard Tauri updater
- ✅ Graceful: If attacker breaks one, other still protects
- ✅ Can transition smoothly (start dual, drop Ed25519 later)
- ✅ Moderate implementation effort

**Cons**:
- ⚠️ Larger download size (extra signature file)
- ⚠️ More complex update workflow
- ⚠️ Two keypairs to manage
- ⚠️ Requires custom verification code

**Code Example**:
```rust
// In update_manager.rs
async fn verify_pqc_signature(
    download_url: &str,
    binary_bytes: &[u8],
) -> Result<(), String> {
    // Download .mldsa signature file
    let sig_url = format!("{}.mldsa", download_url);
    let signature_bytes = download_file(&sig_url).await?;

    // Verify with ML-DSA-65
    let verify_key = get_embedded_mldsa_public_key();
    let signature = MlDsa65Signature::from_bytes(&signature_bytes)?;

    if !MlDsa65Trait::verify(&verify_key, binary_bytes, &signature) {
        return Err("ML-DSA signature verification failed".to_string());
    }

    Ok(())
}
```

**Recommendation**: ✅ **RECOMMENDED** - Best balance of security and practicality

---

### Option 3: Custom Update Verification (Pure PQC)

**Approach**: Completely bypass Tauri's updater, implement our own update system with pure ML-DSA

**Architecture**:
```
Custom Update Manager:
├── Version checking (GitHub API)
├── Binary download (HTTPS)
├── ML-DSA signature verification
├── Atomic file replacement
├── Rollback on failure
└── App restart
```

**Implementation**:
1. Disable Tauri updater plugin
2. Build complete update system from scratch
3. Use only ML-DSA signatures
4. Handle all platform-specific installation

**Pros**:
- ✅ Fully quantum-resistant end-to-end
- ✅ Complete control over update process
- ✅ Can add custom features (delta updates, etc.)
- ✅ Consistent PQC throughout entire system

**Cons**:
- ❌ Major implementation effort (2-3 weeks)
- ❌ Platform-specific installation code needed
- ❌ Must handle macOS code signing separately
- ❌ Need to implement rollback/recovery ourselves
- ❌ Testing burden increases significantly
- ❌ Maintenance burden (Tauri updates automatically)

**Complexity**:
- Need to handle: DMG mounting (macOS), MSI installation (Windows), package managers (Linux)
- Code signing: macOS requires notarization, Windows requires Authenticode
- Permissions: Elevated privileges for system-wide installation
- Atomic updates: Prevent partial installations
- **Estimated**: 200-400 lines of platform-specific code per platform

**Recommendation**: ⚠️ **High effort**, use only if PQC compliance is mandatory

---

### Option 4: Fork Tauri Updater Plugin (Modified Plugin)

**Approach**: Fork `tauri-plugin-updater` and modify to support ML-DSA

**Changes Needed**:
1. Replace Minisign library with saorsa-pqc
2. Modify signature generation code
3. Modify verification code
4. Keep all other Tauri infrastructure

**Pros**:
- ✅ Fully quantum-resistant
- ✅ Keeps Tauri's update infrastructure
- ✅ Could contribute back to Tauri project

**Cons**:
- ❌ Must maintain fork of Tauri plugin
- ❌ Breaking changes in Tauri updates
- ❌ Upstream merge conflicts
- ❌ Community might not accept PQC contribution

**Maintenance Burden**:
- Need to track upstream Tauri plugin changes
- Rebase fork on every Tauri update
- Test compatibility with new Tauri versions

**Recommendation**: ⚠️ **High maintenance burden**, not recommended unless contributing upstream

---

## Comparison Matrix

| Criterion | Ed25519 Only | Dual Signature | Custom Update | Fork Plugin |
|-----------|--------------|----------------|---------------|-------------|
| Quantum Resistant | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| Implementation Effort | ✅ None | ⚠️ Medium (1-2 days) | ❌ High (2-3 weeks) | ❌ High (1-2 weeks) |
| Maintenance Burden | ✅ Minimal | ✅ Low | ⚠️ Medium | ❌ High |
| Download Size | ✅ Smallest | ⚠️ +2-4KB | ✅ Same | ✅ Same |
| Tauri Integration | ✅ Perfect | ✅ Good | ❌ Bypasses | ⚠️ Modified |
| Code Complexity | ✅ Simple | ⚠️ Moderate | ❌ Complex | ❌ Complex |
| Timeline to Production | ✅ Immediate | ⚠️ 2-3 days | ❌ 3-4 weeks | ❌ 2-3 weeks |

## Recommended Solution: Dual-Signature Approach

### Why This is Best

1. **Security**: Quantum-resistant via ML-DSA-65
2. **Practicality**: Works with standard Tauri infrastructure
3. **Time to Market**: 2-3 days implementation
4. **Defense in Depth**: Both classical and quantum protection
5. **Flexibility**: Can drop Ed25519 later if desired

### Implementation Plan

#### Phase 1: Key Generation (30 minutes)
```bash
# Generate ML-DSA-65 keypair
cargo run --example keygen --features ml-dsa-65

# Generate Ed25519 keypair (for Tauri)
cargo tauri signer generate
```

#### Phase 2: GitHub Actions (1 day)
Modify `.github/workflows/release.yml`:
```yaml
- name: Sign with both Ed25519 and ML-DSA
  run: |
    # Ed25519 (Tauri standard)
    cargo tauri signer sign --file dist/communitas.dmg

    # ML-DSA-65 (PQC)
    cargo run --bin mldsa-sign -- \
      --private-key ${{ secrets.MLDSA_PRIVATE_KEY }} \
      --file dist/communitas.dmg \
      --output dist/communitas.dmg.mldsa
```

#### Phase 3: Verification Layer (1-2 days)
Add to `update_manager.rs`:
```rust
pub async fn verify_update_signatures(
    binary_url: &str,
    binary_bytes: &[u8],
) -> Result<(), String> {
    // 1. Tauri verifies Ed25519 automatically (already done)

    // 2. Download and verify ML-DSA signature
    let mldsa_sig_url = format!("{}.mldsa", binary_url);
    let sig_bytes = reqwest::get(&mldsa_sig_url)
        .await
        .map_err(|e| format!("Failed to download PQC signature: {}", e))?
        .bytes()
        .await
        .map_err(|e| format!("Failed to read PQC signature: {}", e))?;

    // 3. Verify with embedded public key
    let public_key = get_embedded_mldsa_public_key();
    let signature = MlDsa65Signature::from_bytes(&sig_bytes)
        .map_err(|e| format!("Invalid signature format: {}", e))?;

    if !MlDsa65Trait::verify(&public_key, binary_bytes, &signature) {
        return Err("ML-DSA signature verification failed - update rejected".to_string());
    }

    tracing::info!("✅ Both Ed25519 and ML-DSA signatures verified");
    Ok(())
}
```

#### Phase 4: Integration (1 day)
- Hook verification into `install_update()` function
- Add ML-DSA public key to embedded configuration
- Update documentation

### Timeline
- **Total**: 2-3 days implementation + 1 day testing
- **Result**: Quantum-resistant update system

## Alternative: Ed25519 Short-Term Path

If immediate PQC is not critical:

**Phase 1** (Now): Use Ed25519 for initial releases
**Phase 2** (3-6 months): Implement dual-signature
**Phase 3** (1-2 years): Drop Ed25519, pure ML-DSA

This allows faster time to market while planning PQC migration.

## Security Considerations

### Quantum Threat Timeline
- **2025-2030**: No quantum computers capable of breaking Ed25519
- **2030-2040**: Possible early quantum computers (academic)
- **2040+**: Potential widespread quantum capability

### Risk Assessment
- **Ed25519 Only**: Vulnerable in 10-30 years
- **Dual Signature**: Secure for foreseeable future
- **Pure ML-DSA**: Maximum quantum resistance

### Recommendation Priority
1. **Critical systems** (financial, healthcare): Dual-signature NOW
2. **Standard applications**: Ed25519 now, dual-signature within 6 months
3. **Low-risk applications**: Ed25519 acceptable for 5-10 years

## Decision Matrix

| If your priority is... | Choose... |
|------------------------|-----------|
| Maximum security | Dual-Signature ✅ |
| Fastest deployment | Ed25519 only |
| Full PQC compliance | Custom Update System |
| Contributing to ecosystem | Fork Tauri Plugin |
| Best overall | **Dual-Signature** ✅ |

## Next Steps

Please decide which approach to take:

1. ✅ **Dual-Signature** (recommended) - 2-3 days work
2. ⚠️ **Ed25519 for now** - Immediate, plan migration
3. ❌ **Custom update system** - 3-4 weeks work
4. ❌ **Fork Tauri plugin** - 2-3 weeks work

Once decided, I'll implement the chosen solution immediately.

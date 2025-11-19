# TDD Implementation Plan - Production Blockers

**Approach:** Test-Driven Development  
**Strategy:** Fix from easiest to hardest, build confidence incrementally

---

## Phase 1: Quick Win - Documentation Fix (15 min)

### Issue 4: ML-DSA Algorithm Documentation Mismatch

**Test:** Documentation consistency check
- Verify all docs reference ML-DSA-87 consistently
- Check UI copy, README, API docs

**Implementation:**
1. Update README.md references
2. Search codebase for "ML-DSA-65" references
3. Update to "ML-DSA-87"

**Verification:**
```bash
grep -r "ML-DSA-65" . --exclude-dir={node_modules,target,dist}
grep -r "ml-dsa-65" . --exclude-dir={node_modules,target,dist}
grep -r "ml_dsa_65" . --exclude-dir={node_modules,target,dist}
```

---

## Phase 2: Headless PQC Implementation (2-4 hours)

### Issue 1: Headless Uses Stub Cryptography

**TDD Approach:**

#### Step 1: Write Failing Tests (30 min)
Create `communitas-headless/tests/crypto_tests.rs`:
```rust
#[test]
fn test_keygen_produces_valid_mldsa87_keys() {
    // Should generate 2592-byte public key, 4627-byte private key
    let (pk, sk) = generate_mldsa87_keypair().unwrap();
    assert_eq!(pk.len(), 2592);
    assert_eq!(sk.len(), 4627);
    // Keys should not be all zeros
    assert!(pk.iter().any(|&b| b != 0));
    assert!(sk.iter().any(|&b| b != 0));
}

#[test]
fn test_keys_are_unique() {
    let (pk1, sk1) = generate_mldsa87_keypair().unwrap();
    let (pk2, sk2) = generate_mldsa87_keypair().unwrap();
    assert_ne!(pk1, pk2);
    assert_ne!(sk1, sk2);
}

#[test]
fn test_sign_verify_roundtrip() {
    let (pk, sk) = generate_mldsa87_keypair().unwrap();
    let message = b"test message";
    let signature = sign_mldsa87(&sk, message).unwrap();
    assert!(verify_mldsa87(&pk, message, &signature).unwrap());
}

#[test]
fn test_keystore_persistence() {
    let identity = "test-node-alpha-beta";
    let (pk, sk) = generate_mldsa87_keypair().unwrap();
    
    // Save keys
    save_keys_to_keystore(identity, &pk, &sk).unwrap();
    
    // Load keys
    let (loaded_pk, loaded_sk) = load_keys_from_keystore(identity).unwrap();
    
    assert_eq!(pk, loaded_pk);
    assert_eq!(sk, loaded_sk);
}
```

#### Step 2: Add Dependencies (5 min)
```toml
# communitas-headless/Cargo.toml
[dependencies]
saorsa-pqc = "0.3.12"
fips204 = "0.4"
rand = "0.8"
keyring = "3.2"
blake3 = "1.0"
zeroize = "1.7"
base64 = "0.22"
```

#### Step 3: Create Crypto Module (1 hour)
Create `communitas-headless/src/crypto.rs`:
- Implement `generate_mldsa87_keypair()` using OsRng
- Implement `sign_mldsa87()` and `verify_mldsa87()`
- Implement keystore save/load functions
- Use same pattern as `communitas-core/src/keystore.rs`

#### Step 4: Replace Stubs (30 min)
Update `communitas-headless/src/main.rs`:
- Remove stub functions (lines 96-107)
- Replace stub calls (lines 672, 674) with real crypto
- Add proper error handling

#### Step 5: Integration Test (30 min)
```rust
#[tokio::test]
async fn test_headless_startup_with_real_crypto() {
    // Start headless instance
    // Verify keys generated
    // Verify keys persisted
    // Restart instance
    // Verify same keys loaded
}
```

---

## Phase 3: SPKI Parser for PQC (3-4 hours)

### Issue 2: SPKI Parser Assumes Ed25519

**TDD Approach:**

#### Step 1: Write Failing Tests (45 min)
Create `communitas-desktop/src/security/spki_parser_tests.rs`:
```rust
#[test]
fn test_parse_ed25519_spki() {
    let ed25519_spki = generate_test_ed25519_spki();
    let key = parse_spki(&ed25519_spki).unwrap();
    assert_eq!(key.algorithm, Algorithm::Ed25519);
    assert_eq!(key.bytes.len(), 32);
}

#[test]
fn test_parse_mldsa87_spki() {
    let mldsa_spki = generate_test_mldsa87_spki();
    let key = parse_spki(&mldsa_spki).unwrap();
    assert_eq!(key.algorithm, Algorithm::MlDsa87);
    assert_eq!(key.bytes.len(), 2592);
}

#[test]
fn test_parse_mlkem768_spki() {
    let mlkem_spki = generate_test_mlkem768_spki();
    let key = parse_spki(&mlkem_spki).unwrap();
    assert_eq!(key.algorithm, Algorithm::MlKem768);
    assert_eq!(key.bytes.len(), 1184);
}

#[test]
fn test_reject_invalid_spki() {
    let invalid = vec![0u8; 100];
    assert!(parse_spki(&invalid).is_err());
}

#[test]
fn test_variable_length_key_storage() {
    let mut state = RawSpkiState::default();
    
    // Store Ed25519 key
    state.set_pin_from_spki(&ed25519_spki).unwrap();
    assert_eq!(state.pinned_key.as_ref().unwrap().len(), 32);
    
    // Store ML-DSA-87 key
    state.set_pin_from_spki(&mldsa_spki).unwrap();
    assert_eq!(state.pinned_key.as_ref().unwrap().len(), 2592);
}
```

#### Step 2: Add Dependencies (5 min)
```toml
# communitas-desktop/Cargo.toml
spki = "0.7"
der = "0.7"
```

#### Step 3: Define Algorithm Enum (15 min)
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Algorithm {
    Ed25519,
    MlDsa87,
    MlDsa65,
    MlKem768,
}

pub struct ParsedKey {
    pub algorithm: Algorithm,
    pub bytes: Vec<u8>,
}
```

#### Step 4: Implement Parser (2 hours)
Update `communitas-desktop/src/security/raw_spki.rs`:
- Use `spki` crate for proper parsing
- Support variable-length keys
- Update `RawSpkiState` to use `Vec<u8>` instead of `[u8; 32]`
- Handle all supported algorithms

#### Step 5: Update State Management (30 min)
- Modify `RawSpkiState` structure
- Update fingerprint calculation for variable lengths
- Update all callers

---

## Phase 4: SPKI Pinning Enforcement (1-2 days)

### Issue 3: SPKI Pinning Not Enforced in QUIC Handshake

**TDD Approach:**

#### Step 1: Research Current QUIC Setup (2 hours)
- Locate QUIC connection establishment code
- Identify TLS/certificate verification hooks
- Document current flow

**Research Tasks:**
```bash
# Find QUIC connection code
rg "ant.*quic.*connect" --type rust
rg "quinn.*connect" --type rust
rg "Endpoint.*connect" --type rust

# Find certificate verification
rg "ServerCertVerifier" --type rust
rg "verify.*cert" --type rust
```

#### Step 2: Write Failing Tests (1 hour)
Un-ignore `communitas-core/tests/quic_integration_tests.rs` and expand:
```rust
#[tokio::test]
async fn test_connection_succeeds_with_matching_pin() {
    let server = start_test_quic_server().await;
    let server_spki = server.get_spki();
    
    let client = create_client_with_pin(server_spki).await;
    let result = client.connect(server.addr()).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_connection_fails_with_mismatched_pin() {
    let server = start_test_quic_server().await;
    let wrong_spki = generate_random_spki();
    
    let client = create_client_with_pin(wrong_spki).await;
    let result = client.connect(server.addr()).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("SPKI pin mismatch"));
}

#[tokio::test]
async fn test_connection_succeeds_without_pin_in_bootstrap_mode() {
    let server = start_test_quic_server().await;
    
    let client = create_client_without_pin().await;
    let result = client.connect(server.addr()).await;
    
    // Should succeed but log warning
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pin_extraction_from_peer_cert() {
    let server = start_test_quic_server().await;
    let client = create_client_without_pin().await;
    
    let conn = client.connect(server.addr()).await.unwrap();
    let peer_spki = extract_peer_spki(&conn).unwrap();
    
    assert!(peer_spki.len() > 0);
}
```

#### Step 3: Locate QUIC Integration Point (2 hours)
Create `communitas-core/src/quic_pinning.rs`:
```rust
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use std::sync::Arc;

pub struct SpkiPinningVerifier {
    pinned_spki: Option<Vec<u8>>,
    allow_bootstrap: bool,
}

impl ServerCertVerifier for SpkiPinningVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        // Extract SPKI from certificate
        let peer_spki = extract_spki_from_cert(end_entity)?;
        
        // Check against pinned SPKI
        if let Some(pin) = &self.pinned_spki {
            if &peer_spki != pin {
                return Err(rustls::Error::InvalidCertificate(
                    rustls::CertificateError::Other(
                        rustls::OtherError(Arc::new(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "SPKI pin mismatch"
                            )
                        ))
                    )
                ));
            }
        } else if !self.allow_bootstrap {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::Other(
                    rustls::OtherError(Arc::new(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "No SPKI pin configured and bootstrap mode disabled"
                        )
                    ))
                )
            ));
        }
        
        // Perform standard certificate validation
        // (could delegate to default verifier or implement basic checks)
        
        Ok(ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        // Implement or delegate
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        // Implement or delegate
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::ED25519,
            // Add PQC schemes when available
        ]
    }
}

fn extract_spki_from_cert(cert: &CertificateDer<'_>) -> Result<Vec<u8>, rustls::Error> {
    use x509_parser::prelude::*;
    
    let (_, parsed) = X509Certificate::from_der(cert.as_ref())
        .map_err(|_| rustls::Error::InvalidCertificate(
            rustls::CertificateError::BadEncoding
        ))?;
    
    Ok(parsed.public_key().raw.to_vec())
}
```

#### Step 4: Integrate with CoreContext (3 hours)
Update `communitas-core/src/core_context.rs`:
- Add SPKI state to CoreContext
- Wire up verifier in QUIC connection builder
- Pass through from Tauri commands

#### Step 5: Update Tauri Commands (1 hour)
Update `communitas-desktop/src/sync.rs`:
- Use `rpk` parameter (remove underscore)
- Pass pin to CoreContext connection methods
- Add error handling for pin mismatches

#### Step 6: Integration Tests (2 hours)
- Run un-ignored QUIC tests
- Add E2E test with real Tauri app
- Test pin mismatch scenarios

---

## Phase 5: Full Verification (1 hour)

### Comprehensive Testing
```bash
# Rust linting (strict)
cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used

# All Rust tests
cargo test --workspace

# TypeScript validation
npm run typecheck

# Unit tests
npm run test:run

# Build verification
npm run build
cargo build --release -p communitas-desktop
cargo build --release -p communitas-headless

# Security audit
cargo audit
npm audit
```

### Manual Verification Checklist
- [ ] Headless generates real PQC keys
- [ ] Headless keys persist across restarts
- [ ] SPKI parser handles Ed25519, ML-DSA-87, ML-KEM-768
- [ ] SPKI pinning rejects mismatched connections
- [ ] SPKI pinning allows bootstrap mode
- [ ] All docs reference ML-DSA-87 consistently
- [ ] No panics/unwraps in production code
- [ ] All tests pass

---

## Rollback Plan

If any phase fails:
1. Git branch per issue for easy rollback
2. Feature flags for gradual rollout
3. Keep headless excluded from releases until verified

## Success Criteria

- [ ] All tests green
- [ ] Clippy passes with strict flags
- [ ] TypeScript builds cleanly
- [ ] No security warnings from audit
- [ ] Documentation updated
- [ ] Production blockers resolved

---

## Estimated Timeline

- Phase 1 (Docs): 15 min
- Phase 2 (Headless): 2-4 hours
- Phase 3 (SPKI Parser): 3-4 hours
- Phase 4 (SPKI Enforcement): 8-16 hours
- Phase 5 (Verification): 1 hour

**Total: 14-25 hours** (1.5-3 days of focused work)

---

## Let's Begin!

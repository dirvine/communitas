# ADR-006: Post-Quantum Cryptography

## Status

Accepted (2025-12-24)

## Context

### The Problem

Quantum computers threaten classical cryptography:

- **Shor's algorithm**: Breaks RSA, ECDH, ECDSA in polynomial time
- **Grover's algorithm**: Halves symmetric key security (128→64 effective bits)
- **Harvest now, decrypt later**: Adversaries can store encrypted data today and decrypt when quantum computers arrive

Communitas handles sensitive communication and files that may need protection for decades. Waiting until quantum computers exist to adopt PQC exposes historical data.

### Requirements

- Quantum-resistant key exchange
- Quantum-resistant signatures
- No central PKI dependency
- Suitable for P2P environment
- NIST-standardized algorithms

## Decision

Adopt **pure post-quantum cryptography** using NIST FIPS 203/204 standards via the `saorsa-pqc` library:

### Algorithm Selection

| Function | Algorithm | Standard | Security Level |
|----------|-----------|----------|----------------|
| Key Exchange | ML-KEM-768 | FIPS 203 | NIST Level 3 |
| Signatures | ML-DSA-65 | FIPS 204 | NIST Level 3 |
| Symmetric | ChaCha20-Poly1305 | RFC 8439 | 256-bit |
| Hashing | BLAKE3 | N/A | 256-bit |

### Why Pure PQC (Not Hybrid)?

Communitas is a **greenfield network** with no legacy peers:

| Approach | Pros | Cons |
|----------|------|------|
| **Hybrid (X25519 + ML-KEM)** | Fallback if PQC broken | Extra complexity, larger payloads |
| **Pure PQC** | Simpler, smaller code | No fallback |

We chose **pure PQC** because:
1. No existing peers to interoperate with
2. NIST standards are finalized (FIPS 203/204)
3. Simpler implementation reduces attack surface
4. Hybrid adds ~50% overhead for marginal benefit

### Key Hierarchy

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Key Hierarchy                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ML-DSA-65 Public Key (1952 bytes)                                 │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ BLAKE3 Hash → 32-byte identity seed                         │   │
│  └─────────────────────────────────────────────────────────────┘   │
│         │                                                           │
│         ├──────────────────────────────────────────┐               │
│         ▼                                          ▼               │
│  ┌──────────────────────┐              ┌──────────────────────┐   │
│  │  ML-DSA-65 Keypair   │              │  ML-KEM-768 Keypair  │   │
│  │                      │              │                      │   │
│  │  Signing (PQ-safe)   │              │  Key Exchange        │   │
│  │  1952 byte pubkey    │              │  (PQ-safe)           │   │
│  │  3309 byte signature │              │                      │   │
│  └──────────────────────┘              └──────────────────────┘   │
│                                                                     │
│  Identity: pubkey_hex (hex-encoded ML-DSA-65 public key)           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Authentication Flow

```rust
// Message signing with ML-DSA-65
let signature = ml_dsa_sign(&private_key, message)?;

// Verification
let valid = ml_dsa_verify(&public_key, message, &signature)?;

// Key sizes
// ML-DSA-65 public key: 1,952 bytes
// ML-DSA-65 signature: 3,309 bytes
// (vs Ed25519: 32 bytes / 64 bytes)
```

### Key Exchange Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│              ML-KEM-768 Key Exchange                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Alice                                   Bob                        │
│  ┌────────────────┐                     ┌────────────────┐         │
│  │ Generate       │                     │ Has ML-KEM     │         │
│  │ ephemeral      │    encapsulated    │ public key     │         │
│  │ shared secret  │◄────────────────────│                │         │
│  │                │    key (1,088 B)    │                │         │
│  └────────────────┘                     └────────────────┘         │
│         │                                       │                   │
│         ▼                                       ▼                   │
│  ┌────────────────┐                     ┌────────────────┐         │
│  │ Decapsulate    │                     │ Encapsulate    │         │
│  │ → shared       │                     │ → shared       │         │
│  │    secret      │                     │    secret      │         │
│  └────────────────┘                     └────────────────┘         │
│         │                                       │                   │
│         └────────────── Same 32-byte ──────────┘                   │
│                         shared secret                               │
│                              │                                      │
│                              ▼                                      │
│                    ┌────────────────┐                              │
│                    │ ChaCha20-Poly  │                              │
│                    │ symmetric key  │                              │
│                    └────────────────┘                              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Raw Public Keys (No X.509)

Following RFC 7250 inspiration, we use raw public keys without X.509 certificates:

| Traditional | Communitas |
|-------------|------------|
| X.509 certificate chain | Raw ML-DSA public key |
| CA hierarchy | Trust-on-first-use |
| CRL/OCSP | Peer reputation |
| Certificate parsing | Direct key use |

**Benefits**:
- No CA infrastructure required
- Simpler implementation
- Smaller payloads
- No certificate expiration issues

### Performance Characteristics

| Operation | Time | Size |
|-----------|------|------|
| ML-KEM keygen | ~1ms | 2,400B (secret) + 1,184B (public) |
| ML-KEM encaps | ~0.5ms | 1,088B ciphertext |
| ML-KEM decaps | ~0.5ms | 32B shared secret |
| ML-DSA keygen | ~2ms | 4,032B (secret) + 1,952B (public) |
| ML-DSA sign | ~3ms | 3,309B signature |
| ML-DSA verify | ~1ms | - |

**Total handshake overhead**: ~5ms (acceptable for P2P)

## Consequences

### Benefits

- **Quantum-safe from day one**: No "harvest now, decrypt later" risk
- **NIST standardized**: FIPS 203/204 are final standards
- **No CA dependency**: Decentralized trust model
- **Future-proof**: Ready for quantum computers
- **Simpler than hybrid**: Single algorithm set

### Trade-offs

- **Larger keys/signatures**: ML-DSA signature is ~50x larger than Ed25519
- **Higher latency**: PQC operations ~10x slower than classical
- **Bandwidth**: ~2KB extra per handshake
- **No classical fallback**: If NIST algorithms broken, no backup

### Mitigation Strategies

| Issue | Mitigation |
|-------|------------|
| Large signatures | Connection reuse, batch verification |
| Slow operations | Async crypto, key caching |
| Bandwidth | Compression, delta sync |

## Alternatives Considered

1. **Classical only (X25519/Ed25519)**: Traditional cryptography
   - Rejected: Not quantum-safe, defeats project goals

2. **Hybrid (X25519 + ML-KEM)**: Classical + PQC combined
   - Rejected: Adds complexity, no benefit for greenfield network

3. **NTRU**: Alternative lattice-based scheme
   - Rejected: Not NIST standardized

4. **SIKE**: Isogeny-based scheme
   - Rejected: Broken in 2022

5. **X.509 with PQC**: Standard PKI with new algorithms
   - Rejected: Adds CA complexity not needed for P2P

## References

- Library: `saorsa-pqc` crate
- Standards: FIPS 203 (ML-KEM), FIPS 204 (ML-DSA)
- Vault Integration: `communitas-core/src/encrypted_storage/`
- ant-quic PQC: `../ant-quic/docs/adr/ADR-003-pure-post-quantum-cryptography.md`
- Related ADR: [ADR-001 Four-Word Identity](ADR-001-four-word-identity-system.md)
- Related ADR: [ADR-011 Encrypted Vault Storage](ADR-011-encrypted-vault-storage.md)

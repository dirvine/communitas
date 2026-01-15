# ADR-001: Four-Word Identity System

## Status

**Superseded** (2025-01-15)

Superseded by pubkey_hex identity model. Four-word encoding now used **only for connection addresses** (WHERE), not user identity (WHO).

## Original Context (Historical)

### The Problem

Traditional identity systems for decentralized networks face usability challenges:

- **Public key fingerprints**: 64+ hex characters are impossible to communicate verbally
- **Usernames**: Require centralized registries to prevent collisions
- **DNS-based**: Depend on centralized infrastructure and can be hijacked
- **QR codes only**: Not usable in voice calls, radio, or text-limited contexts

### Original Decision

The original design used four-word phrases as the primary identity:
- `ocean-forest-moon-star` → BLAKE3 hash → cryptographic seed → keypair

This created a **coupling problem**: the four words were both the identity AND the key derivation source.

## Current Identity Model

The identity model has been simplified to separate concerns:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Simplified Identity Model                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  WHO (Identity)        WHERE (Connection)       SHOWN (Display)     │
│  ────────────────      ─────────────────        ───────────────     │
│                                                                     │
│  pubkey_hex            connection_words         display_name        │
│  (ML-DSA-65)           (four-word IP:port)      (user-chosen)       │
│                                                                     │
│  Permanent             Ephemeral                Mutable             │
│  Cryptographic         Network location         Human-friendly      │
│  1952 bytes raw        4+ words                 Any string          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Identity = Public Key

The user's identity is now simply their **hex-encoded ML-DSA-65 public key**:

```rust
// Identity is the public key itself
let identity = hex::encode(&ml_dsa_public_key); // 3904 hex chars

// For compact storage, first 32 bytes used as fingerprint
let fingerprint: [u8; 32] = public_key[..32].try_into()?;
```

**Benefits**:
- No collision risk (full cryptographic uniqueness)
- No dictionary dependency
- Directly verifiable against signatures
- Language-agnostic

### Connection Words (Still Active)

Four-word encoding remains valuable for **network addresses only**:

```rust
// Encode IP:port to words for verbal sharing
let addr: SocketAddr = "192.168.1.100:9000".parse()?;
let words = conn_words(&addr)?; // "alpha bravo charlie delta"

// Decode words back to IP:port
let decoded = conn_from_words(&words)?;
assert_eq!(addr, decoded);
```

This enables:
- **Verbal peer sharing**: "Connect to echo foxtrot lima bravo"
- **QR-free bootstrapping**: Works over phone, radio, SMS
- **Firewall-friendly**: No need to share numeric IPs

### Display Names

Users choose their own display name for UI presentation:

```rust
pub struct UserProfile {
    /// Hex-encoded ML-DSA-65 public key (THE identity)
    pub pubkey_hex: String,

    /// User-chosen display name (shown in UI)
    pub display_name: String,

    /// Ephemeral connection addresses (four-word encoded)
    pub connection_ids: Vec<String>,
}
```

## Migration Notes

### Deprecated Patterns

```rust
// OLD: Four words as identity
let id_fw = "ocean-forest-moon-star";
let seed = identity_to_seed(id_fw)?;
let keypair = derive_keypair(seed)?;

// NEW: Public key IS the identity
let keypair = generate_ml_dsa_keypair()?;
let pubkey_hex = hex::encode(&keypair.public_key);
```

### Legacy Compatibility

The `id_fw` field remains in `UserProfile` for:
- Vault storage directory naming (backward compatibility)
- Migration path for existing users

New code should use `pubkey_hex` for all identity comparisons and lookups.

## Connection Words Reference

The `four-word-networking` crate provides address encoding:

| Use Case | Format | Example |
|----------|--------|---------|
| IPv4 Connection | space-separated words | echo foxtrot lima bravo |
| IPv6 Connection | more words (adaptive) | alpha bravo charlie delta echo foxtrot |

### Properties

- 2048-word dictionary
- Each word = 11 bits of entropy
- 4 words = 44 bits (sufficient for IPv4:port)
- Checksum validation prevents typos

## Consequences

### Benefits of New Model

- **True uniqueness**: No collision risk (full public key)
- **Simpler mental model**: pubkey = identity, period
- **Language-agnostic**: No dictionary dependency for identity
- **Separation of concerns**: WHO vs WHERE vs SHOWN

### Trade-offs

- **Not verbally communicable**: pubkey_hex is 3904 characters
- **QR/link sharing required**: For identity exchange
- **Display name not unique**: Multiple users can have same name

### Mitigation

- Connection words remain verbally communicable for network addresses
- Display names provide human-friendly presentation
- Pubkey fingerprints (first 8 chars) can be compared for verification

## References

- New identity model: [ADR-006 Post-Quantum Cryptography](ADR-006-post-quantum-cryptography.md)
- Recovery system: [ADR-016 Identity Recovery System](ADR-016-identity-recovery-system.md)
- Implementation: `communitas-core/src/types.rs` (UserProfile)
- Connection words: `four-word-networking` crate

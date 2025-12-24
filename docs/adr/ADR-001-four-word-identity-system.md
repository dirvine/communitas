# ADR-001: Four-Word Identity System

## Status

Accepted (2025-12-24)

## Context

### The Problem

Traditional identity systems for decentralized networks face usability challenges:

- **Public key fingerprints**: 64+ hex characters are impossible to communicate verbally
- **Usernames**: Require centralized registries to prevent collisions
- **DNS-based**: Depend on centralized infrastructure and can be hijacked
- **QR codes only**: Not usable in voice calls, radio, or text-limited contexts

Communitas needs human-verifiable identities that:
- Can be communicated verbally (phone calls, in-person)
- Are cryptographically bound to the user's key material
- Work without centralized registries
- Are resistant to phishing and impersonation

### Requirements

- Human-readable and memorable
- Verbally communicable in ~5 seconds
- No central registry or authority
- Deterministic derivation from cryptographic material
- Anti-phishing checksum validation
- Usable for both user identities and network addresses

## Decision

Adopt a **four-word identity system** using the `four-word-networking` crate, which provides:

### Identity Generation

```rust
// Generate random identity from dictionary
let identity = generate_id_words()?; // "ocean-forest-moon-star"

// Derive cryptographic seed from identity
let seed = identity_to_seed("ocean-forest-moon-star")?; // [u8; 32]

// Validate identity uses dictionary words
assert!(validate_id_words("ocean-forest-moon-star")); // true
```

### Dual-Purpose Addressing

| Use Case | Format | Example |
|----------|--------|---------|
| User Identity | word-word-word-word | ocean-forest-moon-star |
| IPv4 Connection | space-separated words | echo foxtrot lima bravo |
| IPv6 Connection | more words (adaptive) | alpha bravo charlie delta echo foxtrot |

### Key Properties

**1. Dictionary-Based**
- 2048-word dictionary from four-word-networking
- Each word uniquely maps to 11 bits of entropy
- 4 words = 44 bits = 17.6 trillion combinations

**2. Deterministic Seed Derivation**
- Same identity always produces same 32-byte seed via BLAKE3
- Seed used to derive Ed25519 keypairs and ML-DSA-65 keys
- No external lookup required

**3. Validation Layers**
- Format validation: exactly 4 words separated by dashes
- Dictionary validation: all words from approved dictionary
- Checksum validation: built into word selection

### Cryptographic Binding

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Identity Derivation Flow                         │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Four Words      BLAKE3 Hash      Ed25519 Seed    Ed25519 Keypair   │
│  "ocean-forest   ──────────►     [u8; 32]       ──────────►        │
│   moon-star"                      │                Public Key       │
│                                   │                Private Key      │
│                                   ▼                                 │
│                              ML-DSA-65 Seed    ──────────►         │
│                                                   PQC Keys          │
└─────────────────────────────────────────────────────────────────────┘
```

### Network Address Encoding

The same system encodes socket addresses for peer discovery:

```rust
// Encode IP:port to words
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

## Consequences

### Benefits

- **Human-verifiable**: Users can verify identities verbally
- **Phishing-resistant**: Misremembered words produce invalid identities
- **DNS-independent**: No centralized naming infrastructure
- **Verbally communicable**: ~5 second voice transmission
- **Deterministic**: Same words always produce same keys
- **Dual-purpose**: Works for both identities and addresses

### Trade-offs

- **Collision probability**: 44 bits allows ~17 trillion identities (sufficient for intended scale)
- **Dictionary dependency**: Words must be from approved list
- **Language-specific**: English dictionary (future: localized dictionaries)
- **Not globally unique**: Theoretical collision possible at extreme scale

### Security Properties

| Property | Guarantee |
|----------|-----------|
| Pre-image resistance | BLAKE3 provides 256-bit security |
| Collision resistance | 44-bit identity space (acceptable for target use) |
| Verbal verification | 4 words easily compared by humans |
| Typo detection | Invalid words rejected by validation |

## Alternatives Considered

1. **Base58 encoding**: Alphanumeric strings like "5HueCGU8rMj..."
   - Rejected: Not verbally communicable, error-prone

2. **UUID-style identifiers**: Random 128-bit identifiers
   - Rejected: Cannot be communicated verbally

3. **Centralized usernames**: "@alice" style handles
   - Rejected: Requires central registry, censorship vector

4. **PGP key fingerprints**: 40-character hex strings
   - Rejected: Too long for verbal communication

5. **BIP-39 mnemonics**: 12-24 word seed phrases
   - Rejected: Too long for identity verification (designed for key backup)

6. **QR codes only**: Visual scanning for identity exchange
   - Rejected: Not usable in voice-only contexts

## References

- Implementation: `communitas-core/src/identity.rs`
- Library: `four-word-networking` crate
- Types: `communitas-core/src/types.rs` (UserProfile)
- Validation: `communitas-core/src/security/input_validation.rs`
- Related ADR: [ADR-006 Post-Quantum Cryptography](ADR-006-post-quantum-cryptography.md)

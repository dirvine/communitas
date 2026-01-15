# ADR-012: Identity Packet System

## Status

Superseded (2025-01-15) - Simplified identity model, removed four-word identity

## Context

### The Problem

In a peer-to-peer network, users need permanent, verifiable identities that:
- Persist across sessions and network changes
- Can be verified by any peer without central authority
- Support post-quantum cryptographic signatures
- Allow user-friendly display names

### Requirements

- Permanent identity anchored to cryptographic key material
- Self-contained packet that proves identity ownership
- Compatible with ML-DSA-65 post-quantum signatures
- Verifiable without network access
- User-friendly display names (not cryptographically derived)

### Design Philosophy

**Simplicity over cryptographic naming:**
- Public keys ARE the identity (not derived four-word encoding)
- Display names are user-chosen, not cryptographically generated
- Connection words (ADR-013) handle the "WHERE" - this ADR handles "WHO"

## Decision

### Identity Packet Structure

An identity packet contains everything needed to verify a user's permanent identity:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityPacket {
    /// ML-DSA-65 public key (THE permanent identity anchor)
    pub pubkey: Vec<u8>,

    /// User-chosen display name (can be overridden by recipient)
    pub display_name: String,

    /// ML-DSA-65 signature over pubkey||display_name
    pub signature: Vec<u8>,
}
```

### Identity Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Identity Model                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ML-DSA-65 KeyPair Generation                                      │
│         │                                                           │
│         ▼                                                           │
│  ┌─────────────┐                                                   │
│  │  pubkey     │ ◄──── THE IDENTITY (permanent, unique)            │
│  │  (2528 B)   │                                                   │
│  └─────────────┘                                                   │
│         │                                                           │
│         │ User chooses display name                                │
│         ▼                                                           │
│  ┌───────────────┐                                                 │
│  │ display_name  │ ◄──── User-friendly name (can change)           │
│  │ "Alice Smith" │                                                 │
│  └───────────────┘                                                 │
│                                                                     │
│  Sign(pubkey || display_name) ──────► signature                    │
│                                                                     │
│  Recipient receives packet:                                        │
│  - Stores pubkey as the identity reference                         │
│  - Uses display_name as default, can override locally              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Key Properties

**1. Public Key IS the Identity**
- Generated once on first startup
- The pubkey bytes are the permanent, unique identifier
- Stored in encrypted vault (per ADR-011)
- All references to a user use pubkey (or hex encoding of it)

**2. Display Names are User-Friendly**
- User chooses their own display name
- Included in identity packet so new contacts see it
- Recipients can override locally ("Alice" → "Mom")
- Changes propagate when identity packet is re-sent

**3. Self-Verifying**
- Signature proves the identity packet was created by key owner
- Verifies display name hasn't been tampered with
- No network access required for verification

### Verification Process

```rust
impl IdentityPacket {
    pub fn verify(&self) -> Result<bool, Error> {
        // Verify signature over pubkey || display_name
        let message = [
            self.pubkey.as_slice(),
            self.display_name.as_bytes(),
        ].concat();

        verify_ml_dsa_signature(&self.pubkey, &message, &self.signature)
    }
}
```

### Display Name Handling

```rust
/// Contact storage with local name override
pub struct Contact {
    /// The identity (pubkey bytes)
    pub pubkey: Vec<u8>,

    /// Their display name (from latest identity packet)
    pub their_display_name: String,

    /// Our local override (if we prefer a different name)
    pub local_name_override: Option<String>,
}

impl Contact {
    /// Get the name to show in UI
    pub fn display_name(&self) -> &str {
        self.local_name_override
            .as_deref()
            .unwrap_or(&self.their_display_name)
    }
}
```

### Security Considerations

**1. Post-Quantum Safety**
- ML-DSA-65 provides NIST Level 3 security
- Resistant to both classical and quantum attacks
- Aligned with ADR-006 cryptographic choices

**2. No Revocation by Design**
- Identity is permanent - cannot be "revoked" without losing all data
- Compromise requires generating new identity and re-establishing trust
- This is intentional: revocation requires central authority

**3. Display Name Spoofing**
- Malicious user could set display_name to "Alice" when they're not Alice
- Mitigation: pubkey is the true identity, display name is convenience
- Recipients can verify pubkey matches previously known identity
- UI should show "New contact" warning for unknown pubkeys

**4. Privacy**
- Display name is included in identity packet (visible to recipients)
- Use pseudonym if privacy is needed
- Anonymous connections possible (see ADR-013 temporary identities)

## Consequences

### Positive

- Simpler mental model: pubkey = identity, display name = what you call them
- User-friendly names instead of cryptographic four-word encoding
- Recipients can customize names locally
- Supports anonymous/temporary connections
- No confusion between identity words and connection words

### Negative

- No memorable cryptographic shorthand for verbal verification
- Pubkey comparison for verification is not human-friendly
- Display name trust requires user awareness

### Implementation Notes

- Store identity packet in encrypted vault on first generation
- Broadcast identity packet during connection handshake
- Update display_name requires re-signing and broadcasting new packet
- Use pubkey hex (first 8 chars) for debugging/logging

## Migration Notes

This ADR supersedes the previous version which used `four_word_identity` derived
from the public key via BLAKE3 hash. The simplified model removes this concept:

- Old: `IdentityPacket { pubkey, four_word_identity, signature }`
- New: `IdentityPacket { pubkey, display_name, signature }`

Connection words (ADR-013) remain unchanged - they encode IP:port, not identity.

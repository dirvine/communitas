# Identity Model Guide

This guide explains the Communitas identity model and how to use it correctly in code.

## Overview

Communitas uses a three-part identity model that separates:

| Concept | Purpose | Field | Format |
|---------|---------|-------|--------|
| **WHO** | Your cryptographic identity | `pubkey_hex` | 3904 hex characters (ML-DSA-65 public key) |
| **WHERE** | Your network location | `connection_words` / `four_words` | 4 words (e.g., "ocean-forest-moon-star") |
| **SHOWN** | Your display name | `display_name` | Any string (user-chosen) |

## Visual Representation

```
┌─────────────────────────────────────────────────────────────────┐
│                    Communitas Identity Model                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│    ┌─────────────────┐                                          │
│    │      WHO        │  Your cryptographic identity              │
│    │    (Identity)   │  - pubkey_hex: ML-DSA-65 public key       │
│    │                 │  - Permanent, unique, unforgeable          │
│    └────────┬────────┘                                          │
│             │                                                    │
│             ▼                                                    │
│    ┌─────────────────┐                                          │
│    │     WHERE       │  Your network location                    │
│    │  (Connection)   │  - connection_words: four-word phrase     │
│    │                 │  - Ephemeral, changes with IP/port         │
│    └────────┬────────┘                                          │
│             │                                                    │
│             ▼                                                    │
│    ┌─────────────────┐                                          │
│    │     SHOWN       │  Your display name                        │
│    │    (Display)    │  - display_name: user-chosen label        │
│    │                 │  - Mutable, not unique                     │
│    └─────────────────┘                                          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## When to Use Each Field

### Use `pubkey_hex` When...

- Verifying someone's identity
- Storing identity references in databases
- Creating cryptographic signatures
- Comparing identities for equality

```rust
// Identity comparison - always use pubkey_hex
if user_a.pubkey_hex == user_b.pubkey_hex {
    println!("Same person!");
}

// Storing identity reference
struct ContactReference {
    pubkey_hex: String,  // WHO - the identity
    last_known_address: String,  // WHERE - just for connection
}
```

### Use `connection_words` When...

- Sharing how to connect to someone over voice/phone
- Initial peer discovery
- Bootstrapping network connections

```rust
// Sharing connection info verbally
println!("Connect to me at: {}", user.connection_words);
// Output: "Connect to me at: ocean-forest-moon-star"

// Decoding words to IP:port
let addr = conn_from_words(&words)?;
network.connect(addr).await?;
```

### Use `display_name` When...

- Displaying user in UI
- Showing sender/recipient names in messages
- User-facing labels

```rust
// UI display - use display_name
rsx! {
    div {
        span { "Message from: {message.sender.display_name}" }
        // Shows: "Message from: Alice"
    }
}
```

## Common Mistakes

### ❌ Wrong: Four-words as identity

```rust
// WRONG - four-words are connection address, not identity!
fn verify_user(four_words: &str) -> bool {
    trusted_identities.contains(four_words)  // ❌
}
```

### ✅ Correct: Pubkey as identity

```rust
// CORRECT - use pubkey_hex for identity
fn verify_user(pubkey_hex: &str) -> bool {
    trusted_identities.contains(pubkey_hex)  // ✅
}
```

### ❌ Wrong: Display name comparison

```rust
// WRONG - display names are not unique!
if user_a.display_name == user_b.display_name {
    println!("Same person!");  // ❌ Not necessarily!
}
```

### ✅ Correct: Pubkey comparison

```rust
// CORRECT - pubkey_hex is unique identity
if user_a.pubkey_hex == user_b.pubkey_hex {
    println!("Same person!");  // ✅
}
```

## Security Considerations

### Four-words are NOT secret

Four-word connection addresses encode your IP:port and are meant to be shared publicly. They do not reveal your private key.

### Pubkey fingerprints for verification

When verifying identity in person (e.g., "Is this really Alice?"), compare the first 16 characters of the pubkey_hex:

```rust
// Get first 16 chars as fingerprint
let fingerprint = &pubkey_hex[..16];
// Verbally compare: "My fingerprint is a1b2c3d4e5f60718"
```

### Display names can be spoofed

Anyone can set their display name to anything. Never trust a display name alone for identity verification.

## Code Examples

### Creating a User Profile

```rust
use communitas_core::types::UserProfile;

let keypair = generate_ml_dsa_keypair()?;

let profile = UserProfile {
    // WHO - cryptographic identity
    pubkey_hex: hex::encode(&keypair.public_key),

    // SHOWN - user-chosen label
    display_name: "Alice".to_string(),

    // WHERE - ephemeral connection addresses (may have multiple)
    connection_ids: vec!["ocean-forest-moon-star".to_string()],
};
```

### Looking Up a User

```rust
// Find by identity (correct)
let user = users.find_by_pubkey(&pubkey_hex)?;

// Find by connection address (only for network bootstrap)
let addr = conn_from_words(&four_words)?;
let peer = network.discover_peer(addr).await?;
```

### MCP Tool Usage

When calling MCP authentication tools:

```json
{
  "name": "authenticate",
  "arguments": {
    "four_words": "ocean-forest-moon-star",  // WHERE (which vault to unlock)
    "password": "secret"
  }
}
```

Note: `four_words` identifies which vault to unlock on this device, not your identity.

## Related Documentation

- [ADR-001: Four-Word Connection Address System](../adr/ADR-001-four-word-identity-system.md)
- [ADR-006: Post-Quantum Cryptography](../adr/ADR-006-post-quantum-cryptography.md)
- [MCP API Documentation](../api/mcp-api.md#identity-model)

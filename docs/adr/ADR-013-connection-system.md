# ADR-013: Connection System

## Status

Accepted (2025-01-15) - Updated to align with simplified identity model

## Context

### The Problem

Establishing first-time connections in a decentralized network is challenging:
- IP addresses change frequently (mobile, NAT, ISP reassignment)
- Users need to share connection info through limited channels (voice, SMS)
- Connection details are ephemeral but identity is permanent

### Requirements

- Connection words must encode IP:port for QUIC connection
- Support both IPv4 and IPv6 addresses
- Usable over voice calls, SMS, or any text channel
- Clear distinction from permanent identity (pubkey)

## Decision

### Connection Words (WHERE you are NOW)

Connection words encode your current network location as memorable words:

| Aspect | Connection Words | Identity (pubkey) |
|--------|------------------|-------------------|
| Purpose | Network location | Permanent identity |
| Encoding | IP:port → 4+ words | ML-DSA-65 public key |
| Lifetime | Ephemeral (until IP changes) | Permanent |
| Example | "echo foxtrot lima bravo" | 2528 bytes (hex in UI) |
| Sharing | Each time location changes | Once, then remembered |

### Encoding with FourWordAdaptiveEncoder

```rust
use four_word_networking::FourWordAdaptiveEncoder;

// Encode current connection point
let encoder = FourWordAdaptiveEncoder::new()?;
let connection_words = encoder.encode_socket_addr(socket_addr)?;
// e.g., "echo foxtrot lima bravo" for IPv4

// Decode to connect
let socket_addr = encoder.decode_socket_addr(&connection_words)?;
```

### First-Time Connection Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                First-Time Connection Handshake                      │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ALICE                              BOB                            │
│    │                                  │                            │
│    │  1. Shares connection words      │                            │
│    │     out-of-band (voice/SMS/QR)   │                            │
│    │  "echo foxtrot lima bravo"       │                            │
│    │ ───────────────────────────────► │                            │
│    │                                  │                            │
│    │                                  │  2. Decodes → IP:port      │
│    │                                  │     Initiates QUIC conn    │
│    │                                  │                            │
│    │  ◄───────── QUIC CONNECT ─────── │                            │
│    │                                  │                            │
│    │  3. Send IdentityPacket          │                            │
│    │  { pubkey, display_name,         │                            │
│    │    signature }                   │                            │
│    │ ───────────────────────────────► │                            │
│    │                                  │                            │
│    │                                  │  4. Verify signature       │
│    │                                  │     Store identity         │
│    │                                  │     (pubkey + name)        │
│    │                                  │                            │
│    │  5. Bob sends his IdentityPacket │                            │
│    │  ◄─────────────────────────────  │                            │
│    │                                  │                            │
│    │  MUTUALLY AUTHENTICATED          │                            │
│    │  (both know each other's         │                            │
│    │   pubkey + display_name)         │                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Connection Word Format

**IPv4 Addresses:**
- 4 words encode 32-bit IP + 16-bit port
- Example: `192.168.1.100:9090` → "echo foxtrot lima bravo"

**IPv6 Addresses:**
- More words needed (adaptive encoding)
- Example: `[2001:db8::1]:9090` → "alpha bravo charlie delta echo foxtrot golf hotel"

### API Design

```rust
/// Get current connection words (external address as 4+ words)
pub fn get_connection_words() -> Result<String, Error> {
    let external_addr = get_external_address()?;
    let encoder = FourWordAdaptiveEncoder::new()?;
    encoder.encode_socket_addr(external_addr)
}

/// Connect to peer using their connection words
pub fn connect_via_words(connection_words: &str) -> Result<Connection, Error> {
    let encoder = FourWordAdaptiveEncoder::new()?;
    let addr = encoder.decode_socket_addr(connection_words)?;
    quic_connect(addr)
}
```

### Sharing Methods

**Voice/Phone:**
- Read 4 words: "echo foxtrot lima bravo"
- Simple and works without smartphones

**QR Code:**
- Encode connection words in QR
- Scan with phone camera
- Automatic connection initiation

**Text/SMS:**
- Send connection words as text
- Recipient pastes into app

### Security Considerations

**1. Connection Words Are NOT Secret**
- Treat as public information (like a phone number)
- Only reveals network location, not identity
- Safe to share over insecure channels

**2. Identity Verification**
- Connection only establishes WHERE, not WHO
- Identity packet (pubkey + display_name) sent after QUIC connect
- Signature proves authenticity of identity packet
- Store pubkey as the permanent identity reference

**3. Trust on First Use (TOFU)**
- First connection establishes the pubkey-to-person mapping
- Subsequent connections verify same pubkey
- Display name can change, pubkey should not

**4. Ephemeral by Nature**
- Connection words change when IP changes
- Do not use as a persistent identifier
- Store identity (pubkey), not connection words

### Anonymous/Temporary Connections

For privacy or casual interactions:

```rust
/// Connect without exchanging identity packets
pub fn connect_anonymous(connection_words: &str) -> Result<Connection, Error> {
    let conn = connect_via_words(connection_words)?;
    // Skip identity packet exchange
    // Both parties remain anonymous
    conn
}
```

Use cases:
- Anonymous voice/video calls
- Temporary collaboration
- Privacy-conscious interactions

## Consequences

### Positive

- Clear mental model: connection words = WHERE, pubkey = WHO
- Works over any communication channel
- No DNS or central directory required
- Supports mobile/changing IP scenarios
- IPv4 fits in 4 words (easy to speak)
- QR codes enable easy mobile sharing
- Anonymous mode for privacy

### Negative

- IPv6 requires more words (harder to communicate verbally)
- NAT traversal may require additional coordination
- Connection words must be fresh (stale = wrong IP)

### Implementation Notes

- Use `Query::GetConnectionWords` to retrieve current connection words
- Always send `IdentityPacket` immediately after QUIC connection established
- Cache identity-to-address mappings for reconnection (see ADR-014)
- Consider adding QR code generation to UI

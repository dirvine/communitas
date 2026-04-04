# ADR-014: Peer Discovery Presence System

> **SUPERSEDED**: This ADR describes communitas' original peer-to-peer networking design. As of 2026-03, communitas delegates ALL networking to x0x daemon (see ADR-028). Retained for historical reference.

## Status

Accepted (2025-01-15) - Compatible with simplified identity model (ADR-012)

## Context

### The Problem

After initial connection, peers need to reconnect when:
- Either peer restarts their application
- IP addresses change (mobile, ISP reassignment)
- Network conditions require new connections

Without a discovery mechanism, users would need to re-exchange connection words
every time an IP changes, which is impractical.

### Distinction from MLS Presence (presence_service.rs)

The existing `presence_service.rs` provides **group-scoped presence beacons**:
- Encrypted with MLS group keys
- Only visible within specific groups
- Proves "user X is online in group Y"

This ADR describes **network-wide peer discovery**:
- Not encrypted (public network location info)
- Propagates via gossip to find peers anywhere
- Enables reconnection to known contacts

### Requirements

- Signed presence records to prevent spoofing
- Timestamp-based freshness to handle stale data
- Privacy-respecting: only peers who know target respond
- No DHT-style global storage (per network philosophy)
- Graceful handling of offline peers

## Decision

### Presence Record Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceRecord {
    /// ML-DSA-65 public key (permanent identity)
    pub pubkey: Vec<u8>,
    
    /// Current IP:port as connection words (ephemeral)
    pub connection_words: String,
    
    /// Unix timestamp when created
    pub timestamp: u64,
    
    /// ML-DSA-65 signature over pubkey||connection_words||timestamp
    pub signature: Vec<u8>,
}
```

### Gossip Protocol Messages

```rust
/// Request presence info for a target peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceQuery {
    /// Public key of peer we're looking for
    pub target_pubkey: Vec<u8>,
    
    /// Where to send the response
    pub reply_to: SocketAddr,
}

/// Response with presence info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceResponse {
    /// The signed presence record
    pub record: PresenceRecord,
}
```

### Discovery Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                Peer Discovery via Gossip                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ALICE wants to find BOB (knows Bob's pubkey)                      │
│                                                                     │
│  1. Alice broadcasts PresenceQuery(bob_pubkey, alice_addr)         │
│     to gossip overlay                                              │
│                                                                     │
│  2. Query propagates through network via gossip                    │
│                                                                     │
│  3. CHARLIE (who has seen Bob recently) receives query             │
│     - Charlie checks: "Do I have a recent PresenceRecord for Bob?" │
│     - If yes AND Charlie is connected to Bob → respond             │
│     - If no connection to Bob → stay silent (privacy)              │
│                                                                     │
│  4. Charlie sends PresenceResponse(bob_record) to alice_addr       │
│                                                                     │
│  5. Alice receives response, verifies signature                    │
│     - Decodes connection_words → Bob's current IP:port             │
│     - Initiates QUIC connection to Bob                             │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Privacy Design

**Key Principle: Only respond if you're connected to the target**

This prevents:
- Building a global directory of all users' locations
- Tracking users who aren't your direct contacts
- Mass surveillance of network topology

```rust
fn handle_presence_query(&self, query: PresenceQuery) -> Option<PresenceResponse> {
    // Only respond if we have an active connection to target
    if !self.is_connected_to(&query.target_pubkey) {
        return None; // Silent - don't reveal we know this peer
    }
    
    // Only respond with fresh records
    let record = self.presence_cache.get(&query.target_pubkey)?;
    if record.is_stale(MAX_PRESENCE_AGE_SECS) {
        return None;
    }
    
    Some(PresenceResponse { record: record.clone() })
}
```

### Freshness and Staleness

```rust
impl PresenceRecord {
    /// Check if record is stale (older than max_age)
    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(self.timestamp) > max_age_secs
    }
    
    /// Check if this record is fresher than another
    pub fn is_fresher_than(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp
    }
}
```

### Presence Cache

Each node maintains a cache of received presence records:

```rust
pub struct PresenceCache {
    /// Map from pubkey bytes to most recent presence record
    records: HashMap<Vec<u8>, PresenceRecord>,
}

impl PresenceCache {
    /// Insert record, returns true if this was fresher than existing
    pub fn insert(&mut self, record: PresenceRecord) -> bool {
        match self.records.get(&record.pubkey) {
            Some(existing) if !record.is_fresher_than(existing) => false,
            _ => {
                self.records.insert(record.pubkey.clone(), record);
                true
            }
        }
    }
    
    /// Remove all stale records, returns count removed
    pub fn remove_stale(&mut self, max_age_secs: u64) -> usize {
        let before = self.records.len();
        self.records.retain(|_, r| !r.is_stale(max_age_secs));
        before - self.records.len()
    }
}
```

### Announcement Strategy

Nodes should announce their presence:
1. On startup (after determining external address)
2. When external address changes
3. Periodically (every 5 minutes) to refresh timestamp

```rust
/// Create and broadcast a new presence announcement
pub fn announce_presence(&self) -> Result<(), Error> {
    let record = PresenceRecord::new(
        &self.keypair,
        self.get_connection_words()?,
    )?;
    
    self.gossip.broadcast(GossipMessage::PresenceAnnounce(record))
}
```

## Consequences

### Positive

- Peers can reconnect after IP changes
- Privacy-preserving: no global user directory
- Signed records prevent spoofing
- Fresh timestamps prevent replay attacks
- Compatible with gossip overlay architecture

### Negative

- Discovery requires at least one mutual contact online
- Stale records may cause failed connection attempts
- Query flooding possible (mitigate with rate limiting)

### Implementation Notes

- Add `PresenceCache` to `CommunitasApp` state
- Implement `Command::AnnouncePresence` and `Command::QueryPresence`
- Add periodic presence refresh to gossip coordinator
- Cache cleanup should run on a timer (every 15 minutes)

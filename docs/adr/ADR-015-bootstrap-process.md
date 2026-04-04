# ADR-015: Bootstrap Process

> **SUPERSEDED**: This ADR describes communitas' original peer-to-peer networking design. As of 2026-03, communitas delegates ALL networking to x0x daemon (see ADR-028). Retained for historical reference.

## Status

Superseded (2026-03) — replaced by ADR-028 (x0x Daemon Networking Delegation)

## Context

### The Problem

New users joining a decentralized network face a chicken-and-egg problem:
- To find peers, you need to be connected to the network
- To connect to the network, you need to know at least one peer

Traditional solutions have drawbacks:
- **Central bootstrap servers**: Single point of failure, requires infrastructure
- **DHT bootstrap nodes**: Can be blocked, requires maintenance
- **Hardcoded peer lists**: Stale quickly, no guarantee of availability

### Design Philosophy

Communitas embraces a **social bootstrap model**:
- Your first connection comes from a friend (social trust)
- Known contacts are your primary reconnection path
- Bootstrap cache provides fallback, not primary path
- No always-available central servers

### Requirements

- New users must be able to join via friend introduction
- Returning users should reconnect to known contacts first
- Bootstrap cache stores addresses for fallback
- Graceful degradation when bootstrap nodes unavailable
- No dependency on central infrastructure

## Decision

### Bootstrap Priority Order

When starting up, attempt connections in this order:

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Bootstrap Priority                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Priority 1: Known Contacts                                        │
│  ─────────────────────────                                         │
│  - Try cached addresses of favourite contacts first                │
│  - These are peers we've connected to before                       │
│  - Most likely to be online and responsive                         │
│                                                                     │
│  Priority 2: Recent Peers                                          │
│  ────────────────────────                                          │
│  - Try addresses from presence cache                               │
│  - Sorted by most recent timestamp                                 │
│  - May be stale but worth trying                                   │
│                                                                     │
│  Priority 3: Bootstrap Cache                                       │
│  ─────────────────────────                                         │
│  - Stored addresses from previous sessions                         │
│  - Community-contributed bootstrap nodes                           │
│  - Accept some staleness (try multiple)                            │
│                                                                     │
│  Priority 4: Manual Entry                                          │
│  ───────────────────────                                           │
│  - User enters friend's connection words                           │
│  - Social bootstrap for new users                                  │
│  - Fallback when all cached addresses fail                         │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Bootstrap Cache Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapEntry {
    /// Socket address
    pub addr: SocketAddr,
    
    /// When we last successfully connected
    pub last_success: Option<u64>,
    
    /// How many times connection failed since last success
    pub failure_count: u32,
    
    /// Whether this is a community-provided entry
    pub is_community_node: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapCache {
    entries: Vec<BootstrapEntry>,
}

impl BootstrapCache {
    /// Get entries sorted by reliability
    pub fn get_sorted(&self) -> Vec<&BootstrapEntry> {
        let mut entries: Vec<_> = self.entries.iter().collect();
        entries.sort_by(|a, b| {
            // Prefer entries with recent success and low failure count
            match (&a.last_success, &b.last_success) {
                (Some(a_time), Some(b_time)) => b_time.cmp(a_time),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.failure_count.cmp(&b.failure_count),
            }
        });
        entries
    }
    
    /// Record successful connection
    pub fn record_success(&mut self, addr: &SocketAddr) {
        if let Some(entry) = self.entries.iter_mut().find(|e| &e.addr == addr) {
            entry.last_success = Some(current_timestamp());
            entry.failure_count = 0;
        }
    }
    
    /// Record failed connection
    pub fn record_failure(&mut self, addr: &SocketAddr) {
        if let Some(entry) = self.entries.iter_mut().find(|e| &e.addr == addr) {
            entry.failure_count += 1;
        }
    }
}
```

### New User Bootstrap Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                New User Bootstrap (Social)                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. Alice wants to join Communitas                                 │
│     - Installs app, generates identity                             │
│     - Has no cached addresses                                      │
│                                                                     │
│  2. Alice contacts friend Bob (phone, text, in person)             │
│     - "What are your connection words?"                            │
│     - Bob: "echo foxtrot lima bravo"                               │
│                                                                     │
│  3. Alice enters Bob's connection words                            │
│     - App decodes to IP:port                                       │
│     - Establishes QUIC connection                                  │
│                                                                     │
│  4. Exchange identity packets                                      │
│     - Alice and Bob verify each other's identity words             │
│     - Both add each other to contacts                              │
│                                                                     │
│  5. Bob's node shares network info                                 │
│     - Active peers in gossip overlay                               │
│     - Bootstrap cache entries                                      │
│     - Group invites if applicable                                  │
│                                                                     │
│  6. Alice is now part of the network                               │
│     - Can discover other peers via gossip                          │
│     - Has bootstrap cache for future reconnection                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Returning User Bootstrap Flow

```rust
pub async fn bootstrap_network(&self) -> Result<(), Error> {
    // Priority 1: Try favourite contacts with known endpoints
    let contacts = self.get_favourite_contacts().await;
    for contact in contacts {
        if let Some(addr) = self.get_contact_endpoint(&contact).await {
            if self.try_connect(addr).await.is_ok() {
                return Ok(());
            }
        }
    }

    // Priority 2: Try bootstrap cache (quality-scored)
    let cached = self.peer_cache.get_top_peers(20).await;
    for peer in cached {
        if let Some(addr) = peer.addresses.first() {
            if self.try_connect(*addr).await.is_ok() {
                self.peer_cache.record_bootstrap_success(addr).await?;
                return Ok(());
            }
        }
    }

    // Priority 3: Need manual entry
    Err(Error::BootstrapFailed("No cached addresses worked, please enter connection words manually"))
}
```

### Staleness Tolerance

Bootstrap cache entries may be stale:
- IPs change, nodes go offline
- This is expected and acceptable
- Solution: try multiple entries, track reliability via `record_success` / `record_failure`

### No Central Servers

Key architectural decision: **no always-available bootstrap servers**

Rationale:
- Central servers are single points of failure
- Require ongoing infrastructure maintenance
- Can be blocked by adversaries
- Contradict decentralization philosophy

Instead:
- Social bootstrap ensures at least one known peer
- Bootstrap cache accumulates working addresses over time
- Community can share bootstrap lists (but not required)

## Consequences

### Positive

- No dependency on central infrastructure
- Social trust model for initial connections
- Graceful degradation when nodes unavailable
- Self-healing network through cache updates

### Negative

- New users need a friend already on the network
- Cold start problem for isolated networks
- Staleness means some failed connection attempts

### Mitigation Strategies

**For cold start:**
- Documentation includes community bootstrap lists
- App can ship with optional bootstrap hints
- These are suggestions, not requirements

**For isolated networks:**
- Local discovery via mDNS (future enhancement)
- LAN bootstrap for same-network peers

### Implementation Notes

- Store bootstrap cache in encrypted vault
- Update cache on every successful connection
- Periodic cleanup of entries with high failure counts
- UI should clearly explain social bootstrap to new users

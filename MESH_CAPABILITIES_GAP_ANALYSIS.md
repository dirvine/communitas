# MESH_CAPABILITIES.md Gap Analysis & Implementation Plan

**Generated:** 2025-01-24  
**Status:** Planning Document  
**Severity:** Medium-High (Documentation overpromises vs implementation)

## Executive Summary

The [MESH_CAPABILITIES.md](docs/MESH_CAPABILITIES.md) specification is **largely aspirational**. While core building blocks are implemented (QUIC transport, basic NAT coordinator hooks, Yrs CRDT, four-word addressing, post-quantum signatures), most advanced resilience features described in the specification are **not yet implemented**.

### Implementation Status Overview

| Category | Claimed | Actual Status | Gap Severity |
|----------|---------|---------------|--------------|
| **Core Technologies** | ✓ | ✓ Implemented | ✅ None |
| **LAN Discovery** | ✓ Broadcast/Multicast | ❌ Not implemented | 🔴 High |
| **Failure Detection** | ✓ 4-phase recovery | ❌ Not implemented | 🔴 High |
| **NAT Traversal** | ✓ Hole punching | 🟡 Scaffolding only | 🟡 Medium |
| **Resource Limits** | ✓ ResourceLimits struct | ❌ Not implemented | 🟡 Medium |
| **Adaptive Behavior** | ✓ Exponential backoff | ❌ Not implemented | 🟡 Medium |
| **Multi-Transport** | ✓ Bluetooth/Satellite | ❌ Not implemented | 🟢 Low (future) |

---

## ✅ What IS Implemented (Verified)

### 1. Core Technologies ✓
- **QUIC Transport**: Via `saorsa-gossip-transport` (0.1.5)
- **CRDT Synchronization**: Yrs 0.19 with vector clocks, state vector protocol
- **Four-Word Addressing**: `communitas-core/src/identity.rs` + `four-word-networking` 2.6
- **Post-Quantum Crypto**: ML-DSA-65/87 signatures via `saorsa-pqc` 0.3.12
- **Peer Cache**: LRU cache (1000 entries) in `communitas-core/src/gossip/peer_cache.rs`

### 2. Basic Networking ✓
- **Bootstrap Discovery**: Via FOAF queries (`communitas-core/src/gossip/boot.rs`)
- **Coordinator Client**: Address reflection, advert publishing (`coordinator.rs`)
- **Rendezvous Shards**: 65,536-shard DHT-free discovery (`rendezvous.rs`)
- **Anti-Entropy Sync**: Fixed 60s cadence CRDT synchronization

### 3. Security ✓
- **ML-DSA Signatures**: Identity signing and verification
- **ChaCha20-Poly1305**: Symmetric encryption for presence/gossip
- **Keyring Integration**: Platform keychain storage via `keyring` 3.2

---

## ❌ What IS NOT Implemented (Gaps)

### 🔴 Critical Gaps (High Priority)

#### 1. LAN Broadcast/Multicast Discovery
**Claimed:** Section 2.1, Level 2 - "Broadcast discovery (255.255.255.255), Multicast groups (224.0.0.0/4, ff00::/8)"

**Reality:** 
- No UDP broadcast sockets created
- No multicast group joins
- "Broadcast" only refers to pubsub semantics over existing QUIC connections

**Impact:** Cannot discover peers on same LAN without internet/bootstrap nodes

**Search Evidence:**
```bash
grep -r "255.255.255.255\|224.0.0.0\|ff00::" communitas-core/src/
# Returns: Only documentation mentions
```

---

#### 2. Internet Collapse Detection & Local-Only Mode
**Claimed:** Section 3.2 Scenario A - "DETECTION PHASE (0-10 seconds): Bootstrap nodes unreachable → Activate local-only mode"

**Reality:**
- No "local-only mode" state variable
- No timed detection (0-10s)
- Boot sequence fails if coordinators unreachable, no graceful degradation

**Impact:** System cannot operate when internet is down

**Search Evidence:**
```bash
grep -r "local.only\|collapse.detect\|offline.mode" communitas-core/src/
# Returns: No matches
```

---

#### 3. Network Partition Detection & Healing
**Claimed:** Section 3.2 Scenario B - "PARTITION DETECTION: Identify unreachable peer segments → Maintain separate CRDT branches → Merge on healing"

**Reality:**
- No partition detection heuristics
- No explicit CRDT branch management
- Test harness can simulate partitions, but no production behavior
- Only inherent CRDT eventual consistency

**Impact:** No structured recovery after network splits

**Files:** `communitas-core/src/test_harness.rs` has partition simulation, but not used in production

---

#### 4. ResourceLimits Struct
**Claimed:** Section 8.3 - Complete `ResourceLimits` with 2GB memory, 50 peers, bandwidth limits

**Reality:**
```rust
// CLAIMED in docs/MESH_CAPABILITIES.md:
pub struct ResourceLimits {
    max_memory_usage: ByteSize::gb(2),
    max_peer_connections: 50,
    upload_rate_limit: Option<ByteSize::mbps(10)>,
    // ...
}

// ACTUAL: Does not exist
grep -r "ResourceLimits" communitas-*/src/
# Returns: No matches
```

**Impact:** No enforceable resource caps, potential OOM/connection exhaustion

---

### 🟡 Medium Gaps

#### 5. Exponential Backoff
**Claimed:** Section 3.2 Scenario C - "Implement exponential backoff, compress update batches, prioritize critical updates"

**Reality:**
- No backoff in dial/retry logic
- Anti-entropy fixed at 60s cadence
- Offline queue retries without backoff scheduling

**Files:**
- `src/services/storage/OfflineStorageService.ts`: Max 3 retries, no backoff
- `src/services/network/NetworkConnectionService.ts`: `RETRY_DELAYS: [1s, 3s, 10s]` (fixed, not exponential)

---

#### 6. NAT Hole Punching
**Claimed:** Section 5 - "Simultaneous Open: Both peers send QUIC Initial packets simultaneously"

**Reality:**
- Coordinator scaffold exists (`coordinator.rs`)
- Address reflection request stub with timeouts
- No actual hole punching implementation in this repo
- Relies on `ant-quic` 0.8.17 (upstream library capabilities unclear)

**Files:** `communitas-core/src/gossip/coordinator.rs` lines 50-80 (stub only)

---

#### 7. Adaptive Sync & Opportunistic Windows
**Claimed:** Section 3.2 Scenario C - "Detect connection windows, rapid state exchange, checkpoint synchronization"

**Reality:**
- Anti-entropy fixed cadence (60s)
- No bandwidth sensing
- No RTT-based adaptation
- No opportunistic immediate sync on peer connect

**Files:** `AntiEntropyManager` in `communitas-core/src/gossip/context.rs`

---

#### 8. 4-Phase Recovery Procedures
**Claimed:** Section 10.1 - "Phase 1: Detection (0-30s), Phase 2: Stabilization (30-120s), Phase 3: Adaptation (2-10min), Phase 4: Restoration"

**Reality:**
- No state machine for recovery phases
- No timers for phase transitions
- Only ad-hoc boot sequence

---

### 🟢 Low Priority (Future/Planned)

#### 9. Multi-Transport Support
**Claimed:** Section 2.2 - Bluetooth PAN, Cellular, Satellite transports

**Reality:** Not implemented (acceptable as future roadmap items)

---

## 📋 Recommended Action Plan

### Immediate (Week 1): Documentation Alignment
**Effort:** 1-2 hours  
**Priority:** CRITICAL

1. **Update MESH_CAPABILITIES.md** header:
```markdown
**Status:** Design Specification (Partial Implementation)  
**Implementation Status:** See MESH_CAPABILITIES_GAP_ANALYSIS.md
```

2. **Add implementation status badges** to each section:
- ✅ Implemented
- 🚧 Partial/In Progress  
- 📋 Planned

3. **Create disclaimer** at top of spec:
```markdown
## Implementation Status Notice
This document describes the **target architecture**. Not all features are currently implemented.
See [Gap Analysis](../MESH_CAPABILITIES_GAP_ANALYSIS.md) for current status.
```

---

### Short-Term (Weeks 2-4): Critical Resilience Features
**Effort:** 1-2 weeks  
**Priority:** HIGH

#### Task 1: Internet Collapse Detection & Local-Only Mode
**Effort:** 2-4 hours  
**Files:** `communitas-core/src/gossip/context.rs`, new `connectivity_watchdog.rs`

```rust
// Add to GossipContext
pub struct ConnectivityWatchdog {
    local_only_mode: AtomicBool,
    last_bootstrap_success: Arc<RwLock<Instant>>,
}

impl ConnectivityWatchdog {
    pub async fn monitor(&self) {
        loop {
            // Ping bootstrap nodes every 1s
            // If all fail for 10s → local_only_mode = true
            // If any succeeds → local_only_mode = false
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
```

**Test Plan:**
- Simulate bootstrap failure in integration tests
- Verify local peer discovery continues
- Verify WAN dials stop in local-only mode

---

#### Task 2: Exponential Backoff for Dial/Retry
**Effort:** 1-2 hours  
**Files:** `communitas-core/src/gossip/boot.rs`

```rust
use tokio_retry::{strategy::ExponentialBackoff, Retry};

// Replace immediate dials with:
let backoff = ExponentialBackoff::from_millis(100)
    .max_delay(Duration::from_secs(60))
    .take(10);

Retry::spawn(backoff, || async {
    self.dial_peer(addr).await
}).await?;
```

---

#### Task 3: ResourceLimits Struct & Enforcement
**Effort:** 4-8 hours  
**Files:** New `communitas-core/src/resource_limits.rs`

```rust
pub struct ResourceLimits {
    pub max_peer_connections: usize,           // 50
    pub connection_timeout: Duration,          // 30s
    pub max_memory_mb: usize,                  // 2048
    pub anti_entropy_max_interval: Duration,   // 300s
}

impl ResourceLimits {
    pub fn enforce_peer_limit(&self, current: usize) -> Result<()> {
        if current >= self.max_peer_connections {
            Err(anyhow!("Peer limit exceeded"))
        } else {
            Ok(())
        }
    }
}
```

**Integration Points:**
- `network_config.rs`: Load from TOML
- `GossipContext`: Check limits before dialing
- `AntiEntropyManager`: Respect max interval cap

---

### Medium-Term (Months 2-3): LAN Discovery
**Effort:** 1-3 days  
**Priority:** MEDIUM

#### Task 4: UDP Multicast Discovery
**Files:** New `communitas-core/src/lan_discovery.rs`

```rust
use tokio::net::UdpSocket;

pub struct LanDiscovery {
    socket: UdpSocket,
}

impl LanDiscovery {
    pub async fn start(&self) -> Result<()> {
        // Bind 0.0.0.0:5353
        let socket = UdpSocket::bind("0.0.0.0:5353").await?;
        
        // Join multicast group 224.0.0.251
        socket.join_multicast_v4("224.0.0.251".parse()?, "0.0.0.0".parse()?)?;
        
        // Broadcast signed discovery frame every 30s
        // Listen for peer responses
    }
}
```

**Security:** Sign discovery frames with ML-DSA to prevent spoofing

---

### Long-Term (Months 4+): Advanced Features
**Priority:** LOW (Nice-to-have)

- Partition detection heuristics (membership view changes)
- Adaptive sync (RTT-based cadence)
- Bandwidth shaping (upload/download limits)
- Bluetooth PAN transport adapter
- Satellite/high-latency transport tuning

---

## 🧪 Testing Requirements

### New Test Categories Needed

1. **Failure Injection Tests**
```rust
#[tokio::test]
async fn test_internet_collapse_detection() {
    let ctx = setup_context().await;
    
    // Block all bootstrap nodes
    firewall_block_bootstrap();
    
    // Verify local-only mode activates in 10s
    tokio::time::sleep(Duration::from_secs(11)).await;
    assert!(ctx.is_local_only_mode());
}
```

2. **Resource Limit Tests**
```rust
#[tokio::test]
async fn test_peer_limit_enforcement() {
    let limits = ResourceLimits { max_peer_connections: 5, ..Default::default() };
    let ctx = setup_context_with_limits(limits).await;
    
    // Attempt to connect 10 peers
    // Verify only 5 succeed
}
```

3. **LAN Discovery Tests**
```rust
#[tokio::test]
async fn test_lan_multicast_discovery() {
    let peer_a = setup_peer("127.0.0.1:5353").await;
    let peer_b = setup_peer("127.0.0.1:5354").await;
    
    // Verify peers discover each other via multicast
    tokio::time::sleep(Duration::from_secs(5)).await;
    assert!(peer_a.knows_peer(&peer_b.id()));
}
```

---

## 📊 Implementation Checklist

### Phase 1: Documentation & Quick Wins (Week 1)
- [ ] Update MESH_CAPABILITIES.md with status badges
- [ ] Add implementation disclaimer
- [ ] Implement connectivity watchdog
- [ ] Add exponential backoff to dials
- [ ] Create ResourceLimits struct

### Phase 2: Core Resilience (Weeks 2-4)
- [ ] Local-only mode integration tests
- [ ] ResourceLimits enforcement in GossipContext
- [ ] Load ResourceLimits from network config
- [ ] Backoff integration tests

### Phase 3: LAN Discovery (Months 2-3)
- [ ] UDP multicast socket setup
- [ ] Signed discovery frame protocol
- [ ] LAN peer cache integration
- [ ] Discovery integration tests

### Phase 4: Advanced Features (Months 4+)
- [ ] Partition detection heuristics
- [ ] Adaptive anti-entropy
- [ ] Bandwidth shaping
- [ ] Multi-transport abstraction

---

## 🎯 Success Criteria

### Minimal Viable Resilience (MVP)
1. System operates when internet is down (local-only mode)
2. Retries use exponential backoff (no retry storms)
3. Resource limits prevent OOM/connection exhaustion
4. Tests verify all three behaviors

### Full Specification Compliance
1. LAN discovery works without bootstrap nodes
2. Network partitions heal automatically
3. Multi-transport support (Bluetooth, etc.)
4. 4-phase recovery state machine
5. Bandwidth/QoS management

---

## 📝 Version Matrix

| Spec Claim | Spec Version | Code Version | Status |
|------------|--------------|--------------|--------|
| QUIC Transport | ant-quic 0.8.17 | 0.8.17 | ✅ Match |
| Yrs CRDT | 0.19 | 0.19 | ✅ Match |
| saorsa-gossip | 0.1.8 | 0.1.5 | 🟡 Mismatch |
| saorsa-pqc | 0.3.12 | 0.3.12 | ✅ Match |
| four-word-networking | 2.6 | 2.6 | ✅ Match |

**Action:** Update workspace dependencies to match spec (0.1.8 gossip packages)

---

## 🔗 Related Documents

- [MESH_CAPABILITIES.md](docs/MESH_CAPABILITIES.md) - Target specification
- [ARCHITECTURE_CURRENT.md](ARCHITECTURE_CURRENT.md) - Current architecture
- [NAT_TRAVERSAL_UPDATE_2025-10-15.md](NAT_TRAVERSAL_UPDATE_2025-10-15.md) - NAT implementation notes
- [docs/architecture/networking.md](docs/architecture/networking.md) - Networking details

---

## 📞 Escalation Path

**If implementation diverges further from spec:**
1. Freeze spec updates until code catches up
2. Create separate `ROADMAP.md` for planned features
3. Mark MESH_CAPABILITIES.md as "Design Intent" not "Current Behavior"

**If spec needs revision:**
1. Create `MESH_CAPABILITIES_V2.md` with realistic targets
2. Archive V1 as `MESH_CAPABILITIES_ORIGINAL.md`
3. Update all references

---

**Document Owner:** AI Assistant (Amp)  
**Last Updated:** 2025-01-24  
**Next Review:** After Phase 1 completion

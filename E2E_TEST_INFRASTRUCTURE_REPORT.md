# Communitas E2E Test Infrastructure Report

## Executive Summary

Communitas has a sophisticated **multi-layer testing infrastructure** with 3 distinct test environments:

1. **Headless Node Tests** - Production-grade multi-node P2P tests with real network communication
2. **Core Library Tests** - Comprehensive integration tests using TestHarness framework with chaos engineering  
3. **Swift FFI Tests** - Platform-specific tests for iOS/macOS client bindings

Current test coverage spans cryptography, networking, CRDT synchronization, entity management, presence discovery, and end-to-end message flows.

---

## Part 1: Headless Node Tests (communitas-headless/tests/)

### Test Files & Patterns

#### 1. `network_integration.rs` - Multi-Node P2P Testing
**Purpose**: Integration tests for headless nodes with real network communication

**Key Pattern: Binary Launch + Log Monitoring**
- Build communitas-headless binary
- Create temp directories for node storage  
- Generate TOML configs for each node
- Launch nodes as separate processes
- Monitor logs for connection patterns
- Verify metrics via HTTP endpoints

**Test: `test_two_nodes_connection()`**
```
├── Build: cargo build -p communitas-headless
├── Temp storage: /tmp/communitas-integration-test/
├── Node A
│   ├── Config: TOML with identity, storage, network settings
│   ├── Listen: 127.0.0.1:0 (ephemeral)
│   ├── Metrics: http://127.0.0.1:9601/metrics
│   └── Startup pattern: "Gossip networking started on"
├── Node B
│   ├── Bootstrap: uses Node A's address
│   ├── Listen: 127.0.0.1:0 
│   ├── Metrics: http://127.0.0.1:9602/metrics
│   └── Startup pattern: "Connected to peer"
└── Verification
    ├── curl metrics endpoint
    ├── Assert: "communitas_peers_connected 1"
    └── Cleanup: pkill communitas-headless
```

**Key Features**:
- Real QUIC-based transport
- Ephemeral port assignment
- LogMonitor captures stdout/stderr in background thread
- Pattern matching with configurable timeouts
- Metrics verification via curl

---

#### 2. `production_e2e.rs` - Production Network Testing  
**Purpose**: Tests against real DO bootstrap nodes in production

**Test: `test_production_network_gossip_sync()`**
```
Alice Setup:
├── Generate random four-word identity
├── Initialize CoreContext
├── start_networking(None)
├── connect_to_peer(&do_conn_id)
│   └── Convert IP:port to four-word address via conn_words()
└── DO Bootstrap: 138.197.29.195:4433

Bob Setup (same pattern):
├── Random identity
├── Connect to DO Bootstrap
└── Same bootstrap IP:port

Message Flow:
├── Alice: gossip.store_message(message)
├── Wait: Anti-entropy replication (60s interval, default)
├── Bob: Poll gossip.get_all_messages() every 2s
├── Timeout: 180s (3 minutes, ~2 hops)
└── Monitor: DO metrics at http://138.197.29.195:9600/metrics

Network Topology:
Alice ──QUIC──> DO Bootstrap <──QUIC── Bob
       gossip sync / anti-entropy
```

**Key Characteristics**:
- Uses `CoreContext` directly (not binary launch)
- Real async runtime (tokio)
- Gossip anti-entropy: 60s default
- 2-hop latency tolerance: ~120s+
- Converts IP addresses to four-word identities

---

#### 3. `crypto_tests.rs` - PQC Cryptography (TDD Style)
**Purpose**: ML-DSA-87 post-quantum cryptography validation

**Tests**:
```
✓ test_keygen_produces_valid_mldsa87_keys
  - Public key: 2592 bytes
  - Private key: 4896 bytes
  - Randomness verification

✓ test_sign_verify_roundtrip
  - Signature: 4627 bytes
  - Sign → Verify cycle
  - Deterministic output

✓ test_verify_fails_with_wrong_message
  - Tamper detection
  - Changed message → verification fails

✓ test_verify_fails_with_wrong_key
  - Key binding verification
  - Different public key → fails

✓ test_keystore_persistence
  - Save to system keyring
  - Load and verify
  - Zeroization on drop
```

---

## Part 2: Core Library Integration Tests (communitas-core/tests/)

### TestHarness Framework

**Purpose**: Controlled testing environment for 2-8 node distributed scenarios

**Architecture**:
```
TestHarness
├── Network Layer
│   ├── QUIC transport (real UDP on ephemeral ports)
│   ├── LinkPolicy (chaos engineering)
│   └── Routing simulation
├── Nodes
│   ├── CoreContext (full business logic)
│   ├── PresenceManager (discovery)
│   ├── GroupContext (saorsa-gossip)
│   └── Storage (CRDT documents + Yrs)
└── Topologies
    ├── mesh() - all connected
    ├── line() - linear chain
    ├── star() - center hub
    └── partition() - network splits
```

**LinkPolicy for Chaos**:
```rust
LinkPolicy::perfect()           // No latency, no loss
LinkPolicy::disconnected()      // Nodes isolated
LinkPolicy::lossy(0.5)          // 50% packet loss
LinkPolicy::slow(100)           // 100ms + random jitter
```

**Typical Test Pattern**:
```rust
#[tokio::test]
async fn test_workflow() -> Result<()> {
    let harness = TestHarness::new(3).await?;
    harness.mesh().await?;
    harness.wait_until_connected(3, Duration::from_secs(10)).await?;
    
    // Distributed test logic here
    
    harness.cleanup().await?;
    Ok(())
}
```

---

### Integration Test Files (24 Total)

#### `crdt_tests.rs` - CRDT & Vector Clocks (Property-Based)
```
proptest scenarios:
- prop_increment_increases_timestamp
- prop_merge_is_greater_or_equal  
- prop_merge_commutative
- prop_merge_idempotent
- prop_causal_ordering

Covers: VectorClock, Message ordering, CRDT merge semantics
```

#### `bootstrap_tests.rs` - Bootstrap Discovery
```
Tests (mostly ignored, awaiting implementation):
- test_read_write_bootstrap_nodes
- test_bootstrap_connection
- test_multiple_bootstrap_nodes (fallback)
- test_bootstrap_fallback_unreachable

Topology: Bootstrap node (0) → Regular nodes (1, 2)
```

#### `presence_integration_tests.rs` - Presence Discovery
```
Tests (mostly ignored, awaiting integration):
- test_advertise_and_discover
  └─ Node A advertises presence beacon
     Node B discovers via presence overlay
     
- test_presence_ttl_expiry
  └─ Beacons expire after TTL (default: 900s/15min)

Covers: PresenceRecord, TopicId, Group membership
```

#### `member_removal_all_entities_test.rs` - Member Lifecycle
```
Entity types tested:
- Group
- Organization
- Channel
- Project
- Individual

Test pattern for each:
1. Create entity
2. Add member (creator + member = 2)
3. Verify member exists (deleted=false)
4. Remove member (tombstone)
5. Verify tombstone exists (deleted=true)

Key: CRDT soft-delete with tombstones for distributed safety
```

#### `foaf_discovery_tests.rs` - Friend-of-Friend
```
Multi-hop peer discovery:
Node A → Node B → Node C (2-hop discovery)

Uses presence + gossip overlay
```

#### `quic_integration_tests.rs` - QUIC Transport
```
- Connection establishment
- Stream multiplexing
- Idempotent handling
- Connection draining
```

#### `sites_integration_test.rs` / `website_storage_test.rs` - Website Publishing
```
Entity: Website Root

Tests:
- Publish website content
- Virtual disk storage
- Content-addressed access
- DNS-free via identity.website_root
```

#### `peer_sync_integration_tests.rs` - Message Sync
```
CRDT-based message synchronization:
- Send message on node A
- Message appears on node B via gossip
- Timestamp-based ordering
- Causal consistency verification
```

#### Other Tests
- `cascade_member_removal_test.rs` - Cascading deletes
- `crdt_document_lifecycle_test.rs` - Document state machine
- `crdt_error_handling_test.rs` - Error scenarios
- `retry_integration_tests.rs` - Resilience
- `resource_limits_test.rs` - Throttling
- `exponential_backoff_test.rs` - Backoff strategies
- `watchdog_integration_tests.rs` - Health monitoring
- `phase3_integration_tests.rs` - Multi-phase workflows

---

## Part 3: Swift FFI Tests (communitas-swift/)

### `CommunitasKitTests.swift` - Main Test Suite
```swift
testInitialization()
- Create CommunitasClient with four-words
- Verify profile (fourWords, displayName, deviceName)
- Assert storage initialized

testEntityWorkflow()
- List entities (empty initially)
- Create group via createEntity()
- Verify creation returned correct id/name
- Re-list and verify count = 1

testMessaging()
- Create channel entity
- Send message via sendMessage()
- Retrieve via getMessages()
- Verify content + author four-words

Test Pattern:
├── Create temp directory for storage
├── Initialize CommunitasClient
├── Run assertions
└── Cleanup (implicit)
```

### Archived Tests (`_archived_tests/`)
```
test_full.swift      - Full FFI integration test
test_ffi.swift       - Low-level FFI symbol verification
test_sync.swift      - Synchronous API tests
```

---

## Part 4: Storage & Performance Tests (communitas-tui/tests/)

### `integration_tests.rs`
```
test_signup_flow_performance()
- Vault creation timing
- PBKDF2: 1000 iterations (test mode)
- Assert: < 10s completion
- Encrypts credentials with Web Crypto API

test_vault_creation_non_blocking()
- Async vault creation
- Simulates UI updates (100ms poll)
- Ensures UI thread not blocked
- Verifies responsiveness maintained

Entity: Vault (encrypted storage container)
```

---

## Part 5: Entity Types Under Test

### Coverage Matrix

```
Entity Type    | Primary Tests                    | Pattern
───────────────┼──────────────────────────────────┼──────────────────────
Group          | member_removal, crdt_tests       | Create → Add → Remove
Organization   | member_removal_all_entities_test | CRDT member lifecycle  
Channel        | member_removal, messaging tests  | Create → Message → Sync
Project        | member_removal_all_entities_test | Create → Manage
Individual     | member_removal_all_entities_test | Identity + profile
Website Root   | website_storage_test             | Publish → Store → Access
Vault          | integration_tests (TUI)          | Create → Encrypt
Message        | peer_sync_integration_tests      | Create → Gossip sync
Presence       | presence_integration_tests       | Advertise → Discover → TTL
```

---

## Part 6: Test Infrastructure Statistics

### Test Distribution
```
communitas-core/tests/       24 files (~240 KB)
├── CRDT: 4 files
├── Networking: 6 files
├── Entity Management: 4 files
├── Storage: 3 files
└── Resilience: 5+ files

communitas-headless/tests/   3 files (~35 KB)
├── network_integration.rs (multi-node)
├── production_e2e.rs (real bootstrap)
└── crypto_tests.rs (PQC)

communitas-swift/           1 active + 3 archived (~10 KB)

communitas-tui/tests/        1 file (~4 KB)
```

### Multi-Node Test Approaches

| Approach | Mechanism | Speed | Control | Realism |
|----------|-----------|-------|---------|---------|
| **TestHarness** | In-process, real QUIC | Fast | Full | High |
| **Binary Launch** | Separate processes | Slow | Good | Very High |
| **Production** | Real nodes | Slow | None | Real |

---

## Part 7: Current Test Status

### Working Tests
✅ Single-node entity CRUD
✅ CRDT merge operations
✅ Vector clock causality
✅ Member removal (all entity types)
✅ Cryptographic operations (ML-DSA-87)
✅ Swift FFI bindings
✅ Performance baselines

### Ignored/Incomplete Tests
🟡 Bootstrap discovery (awaiting core API)
🟡 Presence discovery (awaiting integration)
🟡 Multi-group interactions
🟡 Website E2E workflows
🟡 Mobile app E2E automation

---

## Part 8: Key Testing Patterns

### Pattern 1: Async Unit Test
```rust
#[tokio::test]
async fn test_operation() -> Result<()> {
    let client = CommunitasClient::new()?;
    assert!(client.operation().await.is_ok());
    Ok(())
}
```

### Pattern 2: Multi-Node TestHarness
```rust
#[tokio::test]
async fn test_distributed() -> Result<()> {
    let harness = TestHarness::new(3).await?;
    harness.mesh().await?;
    // ... test logic ...
    harness.cleanup().await?;
    Ok(())
}
```

### Pattern 3: Process-Based (Binary)
```rust
#[test]
fn test_real_nodes() {
    let binary = build_binary();
    let mut node_a = Command::new(&binary).spawn()?;
    let monitor = LogMonitor::new(node_a.stdout, "Node A");
    let addr = monitor.wait_for("Gossip networking started", Duration::from_secs(10))?;
    node_a.kill()?;
}
```

### Pattern 4: Property-Based
```rust
proptest! {
    #[test]
    fn prop_invariant(input in arb_input()) {
        prop_assert!(invariant_holds(&input));
    }
}
```

### Pattern 5: Swift XCTest Async
```swift
func testAsync() async throws {
    let tempDir = FileManager.default.temporaryDirectory
    let client = try await CommunitasClient(...)
    XCTAssertEqual(client.profile.fourWords, expected)
}
```

---

## Part 9: Recommendations for Enhanced E2E Testing

### Immediate Priorities (Production Readiness)

#### 1. Enable Bootstrap Tests
```
Status: Framework exists, API implementation awaited
Action:
- Implement core_get_bootstrap_nodes()
- Un-ignore bootstrap_tests.rs (6 tests)
- Add test coverage for fallback scenarios
```

#### 2. Multi-Entity Workflow Tests
```
Missing: Cross-entity interactions
Add:
- Message in channel → Thread reply in thread
- Group member → Website visibility
- Organization → Channel → Project hierarchy
- Cascading permissions

Approx: 8-12 new tests
```

#### 3. Chaos Scenarios
```
Extend LinkPolicy tests:
- Network partitions during gossip
- Partial connectivity recovery
- Message loss + eventual consistency
- Byzantine node scenarios
- Leader election failures

Approx: 10-15 new tests
```

#### 4. Complete Entity Type Coverage
```
For each entity type:
- Creation lifecycle
- Permission boundaries  
- Soft deletion + tombstones
- Virtual disk operations
- Website publishing (if applicable)

Approx: 30 new tests (5 per entity type)
```

### Medium-Term Improvements

#### 1. Swift Integration E2E
```
Current: Basic unit tests only
Add:
- Network simulation in Swift tests
- File I/O via virtual disks
- Presence discovery from iOS
- Group messaging workflows
- End-to-end encryption verification

Approx: 12-16 new tests
```

#### 2. Performance Benchmarking
```
Extend communitas-tui/benches/:
- Message throughput (msg/sec)
- Latency percentiles (p50, p95, p99)
- Memory growth under load
- Storage I/O patterns
- Sync latency vs network topology

Approx: 8 benchmark suites
```

#### 3. Distributed Consensus
```
Add property-based tests:
- Threshold-based group decisions
- Signature aggregation (ML-DSA-87)
- Split-brain resolution
- Byzantine fault tolerance

Approx: 6-8 new tests
```

### Infrastructure Improvements

#### 1. CI Pipeline Structure
```
Stage 1: Unit tests (2 min, parallel)
  cargo test --lib --all

Stage 2: Integration tests (5 min, parallel)
  cargo test --test '*' --all

Stage 3: E2E tests (15 min, sequential)
  cargo test --test '*' -- --test-threads=1

Stage 4: Property-based (20 min, parallel)
  proptest with 1000+ cases

Stage 5: Performance (10 min)
  cargo bench --all

Total: ~50 minutes
```

#### 2. Test Reporting
```
Add to CI:
- Test metrics dashboard (pass/fail/flaky)
- Performance regression detection
- Network topology visualization for chaos tests
- CRDT state snapshots on failure
- Code coverage trends
```

#### 3. Local Testing Helpers
```
Add scripts:
- test-single.sh - Run one test with logs
- test-chaos.sh - Run chaos scenarios
- test-entity.sh - Run tests for entity type
- test-perf.sh - Run performance suite
```

---

## Part 10: Test Execution Checklist

### Before Committing Code
```
□ cargo test --lib --all
□ cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used
□ cargo fmt --all -- --check
□ npm run typecheck (if frontend changes)
□ npm test (if frontend changes)
```

### Before Creating PR
```
□ cargo test --all (full suite)
□ Run E2E: cargo test --test '*' -- --test-threads=1
□ Manual test on main features affected
□ Performance regression check
```

### Before Merging to Main
```
□ All CI checks pass
□ No new ignored tests introduced
□ No flaky tests detected
□ Performance benchmarks stable
□ Test coverage maintained or improved
```

---

## Summary

### Current Strengths
✅ Multi-level test pyramid (unit → integration → E2E)
✅ Real network testing (QUIC, gossip, production bootstrap)
✅ Chaos engineering support (LinkPolicy framework)
✅ CRDT verification (property-based + unit tests)
✅ Cross-platform testing (Rust, Swift/iOS)
✅ 6+ entity types with comprehensive coverage
✅ Performance baseline tests

### Path to Production
1. Enable ignored tests (implement missing APIs)
2. Expand multi-entity workflows (30+ tests)
3. Add chaos scenarios (10-15 tests)
4. Complete Swift E2E (12-16 tests)
5. Integrate performance benchmarking
6. Set up continuous test reporting

### Estimated Effort
- **Quick wins**: 20 hours (enable ignored tests, basic workflows)
- **Full coverage**: 60 hours (all recommendations)
- **CI/reporting**: 20 hours (dashboards, reporting)
- **Total**: 100 hours to production-grade testing

---

## File Reference

### Test Locations
```
/communitas-headless/tests/
  ├─ network_integration.rs (binary-based multi-node)
  ├─ production_e2e.rs (real DO bootstrap)
  └─ crypto_tests.rs (ML-DSA-87)

/communitas-core/tests/ (24 files)
  ├─ crdt_tests.rs (property-based CRDT)
  ├─ bootstrap_tests.rs (discovery)
  ├─ presence_integration_tests.rs (PresenceManager)
  ├─ member_removal_all_entities_test.rs (6 entity types)
  ├─ foaf_discovery_tests.rs (multi-hop)
  ├─ quic_integration_tests.rs (transport)
  ├─ sites_integration_test.rs (websites)
  ├─ website_storage_test.rs (storage)
  ├─ peer_sync_integration_tests.rs (message sync)
  └─ (+ 15 more)

/communitas-swift/CommunitasKit/Tests/
  └─ CommunitasKitTests.swift (XCTest suite)

/communitas-tui/tests/
  └─ integration_tests.rs (storage performance)
```

### Framework Files
```
/communitas-core/src/
  ├─ test_harness.rs (TestHarness, TestNode, LinkPolicy)
  └─ tests/network_test_utils.rs (helpers)
```

### Key Dependencies
```
tokio = "1.39"          # Async runtime
proptest = "1.x"        # Property-based testing
tempfile = "3.0"        # Temp directories
XCTest                  # Swift testing framework
```

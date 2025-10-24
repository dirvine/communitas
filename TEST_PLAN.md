# Communitas Test Plan & Harness Documentation

_Last Updated: 2025-10-24_

## Overview

Comprehensive test infrastructure for Communitas covering networking (QUIC, gossip, sync, presence, FOAF), UI (React components, Tauri IPC), and end-to-end scenarios.

## Test Architecture

### Two-Tier Testing Strategy

1. **Fast Lane (CI on every push)**
   - Unit tests (Rust & TypeScript)
   - Property-based tests
   - Mock-based integration tests
   - React component tests
   - Fast Tauri IPC tests
   - **Runtime:** ~2-5 minutes

2. **Slow Lane (Nightly or manual)**
   - Real QUIC multi-node tests
   - Network chaos scenarios
   - E2E multi-peer with headless
   - Process-level smoke tests
   - **Runtime:** ~15-30 minutes

## Test Harness (Rust)

### Location
`communitas-core/src/test_harness.rs`

### Capabilities

#### TestHarness
- **Multi-node orchestration**: Spawn N nodes with real QUIC transport
- **Network topologies**: mesh, line, star, partition, heal
- **Chaos engineering**: latency, jitter, packet loss, disconnection
- **Event-driven waiting**: `wait_until_connected` with timeouts

#### TestNode
- **Real networking**: Ephemeral UDP ports, full QUIC stack
- **Identity**: Unique four-word identifiers and peer IDs
- **Services**: CoreContext, PresenceManager, GroupContext
- **Isolation**: Per-node temp directories

#### LinkPolicy
- **connected**: Enable/disable communication
- **latency**: Base delay (e.g., 100ms)
- **jitter**: Random variation (0 to jitter)
- **loss**: Packet drop probability (0.0-1.0)

### Example Usage

```rust
use communitas_core::test_harness::{TestHarness, LinkPolicy};

#[tokio::test]
async fn test_mesh_with_latency() {
    let harness = TestHarness::new(3).await?;
    harness.mesh().await?;
    harness.set_latency(0, 1, 100).await; // 100ms between nodes 0-1
    
    harness.wait_until_connected(3, Duration::from_secs(10)).await?;
    
    // Test sync, messaging, etc.
    
    harness.cleanup().await?;
}
```

## Test Coverage

### Networking Tests

#### QUIC Integration (`quic_integration_tests.rs`)
- ✅ Handshake success (mesh topology)
- ✅ Reconnection after partition
- ✅ Connection with latency (200ms)
- ✅ Connection with packet loss (30%)
- ✅ Multi-node meshes (5 nodes)
- ✅ Star and line topologies
- ✅ Network partition and healing
- ⏸️ SPKI pinning enforcement (requires implementation)
- ⏸️ High latency + high loss scenarios (#[ignore], slow)

#### Bootstrap Discovery (`bootstrap_tests.rs`)
- ⏸️ Read/write bootstrap.toml (requires core_*_bootstrap_nodes)
- ⏸️ Connect via bootstrap nodes
- ⏸️ Multiple bootstrap fallback
- ⏸️ Unreachable bootstrap handling
- ⏸️ Bootstrap timeout
- ⏸️ Cold start scenarios
- ⏸️ IPv4 preference over IPv6

#### Peer Sync (`peer_sync_integration_tests.rs`)
- ✅ Out-of-order message handling (local)
- ✅ Duplicate detection
- ⏸️ Two-peer sync over network (requires integration)
- ⏸️ Missing range repair with packet loss
- ⏸️ Three-peer convergence
- ⏸️ Convergence after partition

#### Presence (`presence_integration_tests.rs`)
- ⏸️ Advertise and discover in shared group
- ⏸️ TTL expiration
- ⏸️ Multi-group presence
- ⏸️ Presence with packet loss
- ⏸️ Presence during partition

#### FOAF Discovery (`foaf_discovery_tests.rs`)
- ✅ Local cache operations
- ✅ 1-hop FOAF discovery
- ✅ 2-hop FOAF discovery
- ✅ Max hops limit enforcement
- ✅ Cycle detection
- ✅ Query timeout handling
- ✅ Introducer node connection
- ✅ Introducer timeout
- ⏸️ Complete discovery flow (presence -> FOAF)
- ⏸️ Fallback chain integration

#### Message Sync (`message_sync_tests.rs`)
- ✅ Property-based tests (proptest)
  - Vector clock increments
  - Lamport clock monotonic
  - In-order message acceptance
  - Causal ordering
- ✅ Two-peer bidirectional sync
- ✅ Out-of-order queuing and recovery
- ✅ Missing message range detection
- ✅ Three-peer convergence
- ✅ Duplicate message handling

### UI Tests

#### React Components (`src/components/__tests__/`)

**MessageComposer.test.tsx**
- ✅ Renders textarea
- ✅ Send button enabled/disabled
- ✅ Submit on Enter (not Shift+Enter)
- ✅ Clears textarea after send
- ✅ Error display on IPC failure
- ✅ Disables button while sending
- ✅ Whitespace trimming
- ✅ Multiline support

**MessageList.test.tsx**
- ✅ Renders message list
- ✅ Encrypted message placeholders
- ✅ Lock icon for encrypted
- ✅ Author and timestamp display
- ✅ Causal ordering preservation
- ✅ Duplicate deduplication
- ✅ Empty state
- ✅ Auto-scroll to bottom
- ✅ Message grouping by author

**Additional Component Tests Needed:**
- ChannelSidebar (unread badges, presence)
- PresenceList (online/offline indicators)
- ConflictBanner (CRDT merge hints)
- IdentityPicker
- FileUpload

#### Tauri IPC (`communitas-desktop/tests/ipc_commands_tests.rs`)
- ⏸️ `core_claim` (valid/invalid words)
- ⏸️ `core_initialize`
- ⏸️ `core_create_channel`
- ⏸️ `core_send_message_to_channel`
- ⏸️ `core_resolve_channel_members`
- ⏸️ `core_private_put/get`
- ⏸️ `sync_set/clear_quic_pinned_spki`
- ⏸️ `core_get/update_bootstrap_nodes`
- ⏸️ `core_group_create/add_member/remove_member`
- ⏸️ Parameter validation and sanitization

All IPC tests are scaffolded but require AppState/CoreContext setup.

### E2E Tests

#### Playwright E2E (`tests/e2e/`)

**Web Mode (`web-mode/`)**
- ⚠️ Onboarding flow
- ⚠️ Dashboard activity
- ⚠️ Entity management
- ⚠️ Messaging interface

**Tauri Mode (`tauri-mode/`)**
- ⚠️ Identity creation and persistence
- ⚠️ Messaging (type and send)
- ⚠️ File operations UI
- ⚠️ WebRTC APIs
- ⚠️ App lifecycle

**Multi-Peer (`multi-peer/`)**
- ⏸️ Send message UI → headless
- ⏸️ Receive message headless → UI
- ⏸️ Sync after partition
- ⏸️ Offline indicator
- ⏸️ Message queuing when offline

## Running Tests

### Rust Tests

```bash
# Fast lane (unit + mock integration)
cargo test -p communitas-core
cargo test -p communitas-desktop --lib

# Slow lane (network integration)
cargo test -p communitas-core --test quic_integration_tests -- --ignored --nocapture

# All networking tests
cargo test -p communitas-core --tests
```

### Node/TypeScript Tests

```bash
# Fast unit tests
npm run test:run

# Component tests with coverage
npm run test:coverage

# All tests
npm test
```

### E2E Tests

```bash
# Web mode (requires dev server)
npm run test:e2e:web

# Tauri mode (requires tauri dev)
# Terminal 1:
npm run build && npm run tauri dev

# Terminal 2:
npm run test:e2e:tauri

# Multi-peer (requires headless build)
cargo build --release -p communitas-headless
npm run test:e2e:multi-peer  # (when implemented)
```

## Test Markers & Conventions

### Rust
- `#[ignore]`: Slow tests or tests requiring unimplemented features
- `#[tokio::test]`: Async tests
- Fast tests: Run by default in CI
- Slow tests: Marked with `#[ignore]`, run manually or nightly

### TypeScript
- `describe('Component Name', ...)`: Test suite
- `it('should...', ...)` or `test('...', ...)`: Individual test
- Mock Tauri API: `vi.mock('@tauri-apps/api/core')`

### Playwright
- `test.describe(...)`: Test group
- `test.beforeAll/afterAll`: Setup/teardown
- `@slow` tag: For slow E2E tests

## Coverage Goals

### Current Status
- **Rust (communitas-core)**: ~40% (network stack partially tested)
- **Node/TypeScript**: ~15% (component tests just added)
- **E2E**: ~30% (basic flows only)

### Targets (Next 3 Months)
- **Rust**: 60-70% line coverage
- **Node**: 70% line coverage on core UI components
- **E2E**: Cover all critical user journeys

## CI Integration

### GitHub Actions Workflow

```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-fast:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --lib
      - run: cargo test -p communitas-core # Fast tests only

  node-fast:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - run: npm ci
      - run: npm run typecheck
      - run: npm run test:run
      - run: npm run test:coverage

  rust-slow:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' # Nightly
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test -p communitas-core --tests -- --ignored --nocapture

  e2e:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: npm run build
      - run: npm run test:e2e:web
```

## Debugging Tests

### Enable Logging

```bash
# Rust
RUST_LOG=debug,communitas=trace cargo test -- --nocapture

# Node
DEBUG=* npm test

# Playwright
PWDEBUG=1 npm run test:e2e:tauri
```

### Inspect Failures

```bash
# Playwright reports
npm run test:e2e:report

# Rust test output
cargo test -- --nocapture --test-threads=1

# Component test UI
npm run test:ui
```

## Next Steps

1. **Implement AppState test helpers** for IPC command tests
2. **Activate network integration tests** by removing `#[ignore]`
3. **Add remaining component tests** (ChannelSidebar, PresenceList, etc.)
4. **Implement multi-peer E2E** with headless orchestration
5. **Add snapshot tests** for stable presentational components
6. **Set up coverage gates** in CI (e.g., ≥60% Rust, ≥70% Node)
7. **Document test patterns** in CONTRIBUTING.md

## References

- [AGENTS.md](./AGENTS.md) - Core flows and tooling
- [Oracle's test strategy](ORACLE_TEST_STRATEGY.md) - Detailed recommendations
- [Playwright docs](https://playwright.dev)
- [Vitest docs](https://vitest.dev)
- [Proptest docs](https://proptest-rs.github.io/proptest/)

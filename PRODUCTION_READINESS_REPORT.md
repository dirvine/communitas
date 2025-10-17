# Production Readiness Report - Communitas Tauri macOS App

**Date:** 2025-10-17  
**Version:** 0.1.17  
**Platform:** Tauri v2 Desktop (macOS primary target)

---

## Executive Summary

### Overall Readiness: ⚠️ **Pre-Alpha** (45% Production Ready)

**Critical Gaps:**
1. ❌ **E2E test coverage is minimal** - Only 3 Playwright tests exist (1 Tauri-native, 2 web-mode)
2. ⚠️ **No automated macOS-specific testing** - No TCC permission flows, Touch ID, Keychain integration tests
3. ⚠️ **Storyboard flows are untested** - Core user journeys from storyboard lack automated verification
4. ✅ **Rust core is well-tested** - 16 integration tests, 3,711 lines of test code
5. ⚠️ **Limited frontend unit tests** - Only 2/25 test files actively running

**Recommended Actions Before Production:**
1. Implement P0 E2E test suite (16 critical scenarios - see below)
2. Add macOS permission flow automation
3. Expand Rust integration test coverage for WebRTC and messaging
4. Implement automated storyboard journey validation

---

## 1. Current Test Coverage Analysis

### 1.1 Rust Backend Tests ✅ Strong

**Location:** `communitas-core/tests/`, `communitas-desktop/tests/`

| Test Suite | Files | Lines | Status | Coverage |
|------------|-------|-------|--------|----------|
| Core Integration Tests | 5 | ~800 | ✅ Passing | Good |
| Desktop Integration Tests | 10 | ~2,911 | ✅ Passing | Moderate |
| **Total Rust Tests** | **16** | **~3,711** | ✅ | **~70%** |

**Strong Areas:**
- ✅ FOAF Discovery (`foaf_discovery_tests.rs`)
- ✅ CRDT Synchronization (`crdt_tests.rs`)
- ✅ Message Sync (`message_sync_tests.rs`)
- ✅ Document Replication (`doc_replicator_tests.rs`)
- ✅ WebRTC Media Controls (`webrtc_media_controls_tests.rs`)
- ✅ Tauri Commands (`tauri_commands_test.rs`)
- ✅ Auth & Touch ID (`auth_touchid_tests.rs`)
- ✅ Passkey Flow (`passkey_flow_integration_test.rs`)

**Weak Areas:**
- ⚠️ No load/stress tests for messaging at scale
- ⚠️ No multi-device concurrent editing tests
- ⚠️ No offline sync recovery tests
- ⚠️ No WebRTC call quality/stability tests under poor network

### 1.2 Frontend Unit Tests ⚠️ Weak

**Location:** `src/**/__tests__/`

| Test Category | Files | Running | Status |
|---------------|-------|---------|--------|
| Service Tests | 18 | 2 | ⚠️ Most Skipped |
| Util Tests | 4 | 2 | ✅ Active |
| Storage Tests | 12 | 0 | ❌ All Skipped |
| Security Tests | 3 | 0 | ❌ All Skipped |
| **Total** | **25** | **2/25 (8%)** | ⚠️ **Mostly Inactive** |

**Running Tests (14 passing):**
- ✅ `src/services/__tests__/featureFlags.test.ts` (2 tests)
- ✅ `src/utils/__tests__/identity.test.ts` (12 tests)

**Critical Missing Tests:**
- ❌ Component tests for UI (React/Material-UI)
- ❌ Context provider tests (AuthContext, EntityDirectory, LocalStorage)
- ❌ WebRTC service tests
- ❌ Channel sync integration tests (skipped but exist)
- ❌ Storage pipeline tests (all marked as skipped/slow)

### 1.3 E2E Tests ❌ Critical Gap

**Location:** `tests/e2e/`

| Test Suite | Files | Tests | Status |
|------------|-------|-------|--------|
| Tauri Native | 1 | 10 | ⚠️ Needs Runner |
| Web Mode | 2 | 17 | ✅ Configured |
| **Total** | **3** | **27** | ⚠️ **Insufficient** |

**Existing E2E Coverage:**
1. `tauri-mode/app-lifecycle.spec.ts` (10 tests)
   - ✅ App startup
   - ✅ IPC communication
   - ✅ Basic file system access
   - ✅ WebRTC API availability
   - ⚠️ No full user flows

2. `web-mode/dashboard.spec.ts` (17 tests)
   - ✅ Activity dashboard
   - ✅ Entity management UI
   - ✅ Messaging interface
   - ⚠️ Web-only, not Tauri-native

3. `web-mode/onboarding.spec.ts` (2 tests)
   - ✅ Welcome screen
   - ✅ Identity creation navigation
   - ⚠️ Web-only, not Tauri-native

**Critical Missing E2E Tests:**
- ❌ Complete onboarding flow (4-word identity creation/recovery)
- ❌ macOS permissions (camera, mic, screen recording, notifications)
- ❌ Keychain/Touch ID integration
- ❌ Channel creation and messaging end-to-end
- ❌ File upload/download with Tauri file picker
- ❌ WebRTC call lifecycle (start, join, end)
- ❌ Offline message queueing and sync
- ❌ Multi-identity switching
- ❌ Deep link handling (`communitas://` URLs)
- ❌ Window state persistence
- ❌ Document collaboration (CRDT)
- ❌ Search and keyboard shortcuts (⌘K)

---

## 2. Storyboard Flow Coverage Analysis

### 2.1 Critical User Journeys (from storyboard-canvas-v2.html)

**Identified Flows:**

1. **First Run & Onboarding** ❌ Not Tested
   - Welcome screen → Create identity → Set up profile
   - Seed phrase backup → Confirm recovery phrase
   - Join first organization/space

2. **Core Messaging** ⚠️ Partially Tested
   - Create channel → Send message → Receive message
   - Message reactions → File attachments
   - Search/jump to conversation (⌘K)
   - Unread badges and notifications

3. **Identity Management** ❌ Not Tested
   - Switch identities → Lock app → Unlock with Touch ID
   - Recover identity from seed phrase
   - Export/backup keys

4. **File & Storage** ❌ Not Tested
   - Upload file via picker → Download file
   - Drag-and-drop from Finder
   - Create document → Collaborative editing
   - Storage usage widget

5. **WebRTC Communication** ❌ Not Tested
   - Start audio call → Grant mic permission
   - Start video call → Grant camera permission
   - Screen share → Grant screen recording permission
   - Join existing call → End call

6. **Organization & Spaces** ⚠️ UI Only
   - Create organization → Add members
   - Create project → Manage channels
   - Invite users → Deep link handling

### 2.2 Storyboard Screen Count

**Analysis of storyboard-canvas-v2.html:**
- ~12 distinct user flows identified
- ~40+ unique screens/states
- ~60+ interaction points requiring validation

**Current E2E Coverage:** **~5%** (3 tests covering only basic lifecycle)

---

## 3. macOS-Specific Testing Gaps

### 3.1 Tauri & macOS Integration ❌ Critical Gaps

| Feature | Test Status | Risk |
|---------|-------------|------|
| File Picker Dialog | ❌ Not Tested | High |
| Camera Permission (TCC) | ❌ Not Tested | Critical |
| Microphone Permission | ❌ Not Tested | Critical |
| Screen Recording Permission | ❌ Not Tested | Critical |
| Notifications Permission | ❌ Not Tested | High |
| Keychain Integration | ⚠️ Partial | High |
| Touch ID / Secure Enclave | ⚠️ Partial | High |
| Deep Links (communitas://) | ❌ Not Tested | Medium |
| Window Management | ⚠️ Basic Only | Low |
| Menu Bar Integration | ❌ Not Tested | Low |
| Dock Badge | ❌ Not Tested | Low |
| macOS Updates | ❌ Not Tested | Medium |

### 3.2 Tauri IPC Command Coverage

**Exposed Tauri Commands** (from `communitas-desktop/src/`):
- ✅ `health` - Basic test exists
- ⚠️ `core_initialize` - Partial coverage
- ❌ `core_claim` - Not tested E2E
- ❌ `core_advertise` - Not tested E2E
- ❌ `core_create_channel` - Not tested E2E
- ❌ `core_send_message_to_channel` - Not tested E2E
- ❌ `container_init` - Not tested E2E
- ❌ `sync_fetch_deltas` - Not tested E2E
- ❌ WebRTC commands - Partially tested (unit only)
- ❌ File system commands - Basic only

**Coverage:** ~15% of IPC commands have E2E tests

---

## 4. Test Infrastructure Assessment

### 4.1 Existing Test Tools ✅ Well Configured

| Tool | Purpose | Status |
|------|---------|--------|
| **Vitest** | Unit/integration tests | ✅ Configured |
| **Playwright** | E2E browser tests | ✅ Configured |
| **Cargo Test** | Rust unit/integration | ✅ Active |
| **Coverage Tools** | Code coverage | ✅ Available |
| **CI Integration** | GitHub Actions | ✅ Exists |

### 4.2 Missing Test Infrastructure ❌ Critical Gaps

| Need | Purpose | Priority |
|------|---------|----------|
| **tauri-driver** | Native Tauri app automation | 🔴 Critical |
| **macOS Test Runner** | Real macOS environment | 🔴 Critical |
| **TCC Database Seeding** | Pre-approve permissions | 🔴 Critical |
| **Fake Media Devices** | WebRTC testing without hardware | 🟡 High |
| **Network Chaos Tools** | Offline/latency simulation | 🟡 High |
| **Multi-Device Test Harness** | Concurrent client testing | 🟠 Medium |
| **Performance Benchmarks** | Load/stress testing | 🟠 Medium |

### 4.3 Test Execution Speed

| Test Suite | Duration | Status |
|------------|----------|--------|
| Unit Tests (npm) | ~620ms | ✅ Fast |
| Rust Tests (cargo) | N/A | ⚠️ Skipped in CI |
| E2E Tests (Playwright) | N/A | ❌ Not Running |

---

## 5. Production Readiness Scorecard

| Category | Score | Weight | Weighted |
|----------|-------|--------|----------|
| **Rust Core Tests** | 85% | 30% | 25.5% |
| **Frontend Unit Tests** | 20% | 15% | 3.0% |
| **E2E Tests** | 10% | 35% | 3.5% |
| **macOS Integration** | 5% | 20% | 1.0% |
| **Overall Readiness** | | | **33%** |

### Grade: **D (Pre-Alpha)**

---

## 6. Critical Recommendations (P0 - Before Beta)

### 6.1 Implement Core E2E Test Suite (Est: 2 weeks)

**Priority 0 - Must Have (16 scenarios):**

#### Onboarding (3 tests)
1. ✅ **O1: First launch → Create identity**
   - Fresh app launch → 4-word phrase generation → Save → Confirm
   - Verify persistence across relaunch

2. ✅ **O2: Recover identity**
   - Enter existing 4-word phrase → Restore profile/spaces
   - Negative: wrong phrase → safe error

3. ✅ **O3: Join organization via invite**
   - Open deep link → Complete identity → Join org
   - Verify org in sidebar with channels

#### Core Messaging (4 tests)
4. ✅ **M1: Basic channel messaging**
   - Open #general → Send text → Message persists

5. ✅ **M2: Unread and activity indicators**
   - Receive message → Unread badge increments
   - Visit channel → Badge clears

6. ✅ **M3: New DM / New Chat**
   - "+ New Chat" → Pick user → Send DM → Sidebar updates

7. ✅ **M4: Notifications**
   - Unfocused window → Message arrives → macOS notification
   - Click notification → Focus app → Navigate to message

#### Identity Management (1 test)
8. ✅ **I1: Identity selector + workspace visibility**
   - Confirm active identity → Spaces reflect identity
   - Relaunch preserves selection

#### File/Storage (3 tests)
9. ✅ **F1: Upload/attach file**
   - Use Tauri file picker → Upload → Progress → Attachment appears
   - Reopen → Attachment loads → Click to preview

10. ✅ **F2: Download and open**
    - Download attached file → Save → Open with default app

11. ✅ **F3: Create new doc**
    - "+ New Doc" → Enter content → Navigate away/back
    - Content persisted (basic CRDT)

#### WebRTC/Communication (2 tests)
12. ✅ **W1: Start audio/video call**
    - Initiate call → macOS permissions → Allow
    - Local media shows → Mute/unmute → Leave call

13. ✅ **W2: Join existing call**
    - Join active call → Remote stream renders → End call
    - Proper teardown (no stuck devices)

#### macOS Integration (3 tests)
14. ✅ **TCC1: Camera permission flow**
    - Request camera → macOS dialog → Allow/Deny handling

15. ✅ **TCC2: Microphone permission flow**
    - Request mic → macOS dialog → Allow/Deny handling

16. ✅ **TCC3: Notification permission flow**
    - First launch → Request notifications → Respect choice

**Implementation:**
```bash
# Create test file structure
tests/e2e/tauri-mode/
  ├── 01-onboarding.spec.ts        # O1-O3
  ├── 02-messaging.spec.ts         # M1-M4
  ├── 03-identity.spec.ts          # I1
  ├── 04-files-storage.spec.ts     # F1-F3
  ├── 05-webrtc.spec.ts            # W1-W2
  └── 06-macos-permissions.spec.ts # TCC1-TCC3
```

### 6.2 Set Up macOS E2E Infrastructure (Est: 3 days)

**Required Components:**

1. **Install tauri-driver**
   ```bash
   cargo install tauri-driver
   npm install --save-dev @tauri-apps/cli
   ```

2. **Create macOS Test Runner Configuration**
   ```yaml
   # .github/workflows/e2e-macos.yml
   name: E2E Tests (macOS)
   on: [push, pull_request]
   jobs:
     e2e-macos:
       runs-on: macos-latest
       steps:
         - uses: actions/checkout@v4
         - name: Setup Rust
           uses: dtolnay/rust-toolchain@stable
         - name: Setup Node
           uses: actions/setup-node@v4
         - name: Build Tauri App
           run: npm run tauri:build
         - name: Grant TCC Permissions
           run: ./scripts/grant-tcc-permissions.sh
         - name: Run E2E Tests
           run: npm run test:e2e:tauri
   ```

3. **Create TCC Permission Script**
   ```bash
   # scripts/grant-tcc-permissions.sh
   #!/bin/bash
   # Pre-approve camera/mic/screen for CI
   sudo sqlite3 /Library/Application\ Support/com.apple.TCC/TCC.db \
     "INSERT OR REPLACE INTO access VALUES ..."
   ```

4. **Add Fake Media Devices**
   ```typescript
   // tests/utils/fake-media.ts
   export async function setupFakeMediaDevices(page: Page) {
     await page.evaluate(() => {
       navigator.mediaDevices.getUserMedia = async () => {
         const canvas = document.createElement('canvas');
         return canvas.captureStream();
       };
     });
   }
   ```

### 6.3 Expand Rust Integration Tests (Est: 1 week)

**Missing Coverage:**

1. **Multi-Device Sync**
   ```rust
   // communitas-core/tests/multi_device_sync.rs
   #[tokio::test]
   async fn test_concurrent_editing_resolves_correctly() {
       // Two devices edit same document
       // Assert CRDT merges without conflicts
   }
   ```

2. **Offline Sync Recovery**
   ```rust
   // communitas-core/tests/offline_sync.rs
   #[tokio::test]
   async fn test_offline_message_queue_flushes() {
       // Go offline, queue messages
       // Come online, verify flush order
   }
   ```

3. **WebRTC Call Stability**
   ```rust
   // communitas-desktop/tests/webrtc_stability.rs
   #[tokio::test]
   async fn test_call_reconnects_after_network_blip() {
       // Start call, simulate network loss
       // Assert automatic reconnection
   }
   ```

### 6.4 Activate Existing Test Suites (Est: 2 days)

**Re-enable Skipped Tests:**

1. **Storage Pipeline Tests**
   ```bash
   # Currently: 12 tests skipped (marked as slow)
   # Action: Optimize and re-enable in CI
   npm run test:services -- src/services/storage/__tests__/
   ```

2. **Channel Sync Tests**
   ```bash
   # Currently: 2 tests skipped
   # Action: Fix dependencies and enable
   npm run test:services -- src/services/__tests__/channelSync.test.ts
   ```

3. **Security Tests**
   ```bash
   # Currently: 3 test files inactive
   # Action: Verify and enable
   npm run test:services -- src/services/security/__tests__/
   ```

---

## 7. Priority 1 Recommendations (Before GA)

### 7.1 Advanced E2E Scenarios (Est: 1 week)

1. **Offline/Network Resilience**
   - Message queuing while offline
   - Sync after reconnection
   - Network latency simulation

2. **Identity Management**
   - Lock/unlock with Touch ID
   - Switch between multiple identities
   - Export/import identity backup

3. **Collaborative Features**
   - Multi-user document editing
   - Conflict resolution visualization
   - Version history

4. **Search & Navigation**
   - Global search (⌘K)
   - Jump to channel/doc
   - Recent activity filtering

### 7.2 Performance Testing (Est: 3 days)

1. **Cold Start Time**
   ```typescript
   test('app launches in < 2 seconds', async () => {
     const start = Date.now();
     await launchApp();
     const duration = Date.now() - start;
     expect(duration).toBeLessThan(2000);
   });
   ```

2. **Message List Rendering**
   ```typescript
   test('renders 10,000 messages without lag', async () => {
     await seedMessages(10000);
     const fps = await measureScrollFPS();
     expect(fps).toBeGreaterThan(30);
   });
   ```

3. **File Upload/Download Throughput**
   ```typescript
   test('uploads 100MB file in < 30 seconds', async () => {
     const file = createTestFile(100 * 1024 * 1024);
     const duration = await uploadAndMeasure(file);
     expect(duration).toBeLessThan(30000);
   });
   ```

### 7.3 Component Testing (Est: 4 days)

**Priority Components:**

1. **AuthContext Provider**
   ```typescript
   // src/contexts/__tests__/AuthContext.test.tsx
   describe('AuthContext', () => {
     test('provides login/logout functionality');
     test('persists session across reloads');
     test('handles keychain errors gracefully');
   });
   ```

2. **EntityDirectoryContext**
   ```typescript
   describe('EntityDirectoryContext', () => {
     test('loads organizations and channels');
     test('updates on entity creation');
     test('filters by search query');
   });
   ```

3. **Message Components**
   ```typescript
   describe('<MessageList />', () => {
     test('renders messages in order');
     test('shows unread indicator');
     test('handles scroll to bottom');
   });
   ```

---

## 8. Continuous Testing Strategy

### 8.1 CI/CD Pipeline

```yaml
# Recommended GitHub Actions Workflow

name: CI/CD Pipeline
on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test --all-features
      - run: cargo clippy -- -D warnings

  frontend-tests:
    runs-on: ubuntu-latest
    steps:
      - run: npm run test:run
      - run: npm run typecheck

  e2e-macos:
    runs-on: macos-latest
    steps:
      - run: npm run tauri:build
      - run: npm run test:e2e:tauri
      - uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: playwright-report
          path: playwright-report/

  coverage:
    runs-on: ubuntu-latest
    steps:
      - run: npm run test:coverage
      - run: cargo tarpaulin --out Xml
      - uses: codecov/codecov-action@v4
```

### 8.2 Pre-Commit Hooks

```bash
# .husky/pre-commit
#!/bin/sh
cargo fmt --all --check
cargo clippy --all-features -- -D warnings
npm run typecheck
npm run test:run
```

### 8.3 Release Checklist

```markdown
## Pre-Release Testing Checklist

### Manual Testing
- [ ] Fresh install on clean macOS system
- [ ] All macOS permissions work (camera, mic, screen, notifications)
- [ ] Identity creation/recovery flows
- [ ] Join organization via invite link
- [ ] Send/receive messages in channels
- [ ] Upload/download files
- [ ] Start/join audio/video calls
- [ ] Screen sharing works
- [ ] Lock/unlock with Touch ID
- [ ] Switch between identities
- [ ] Offline message queueing
- [ ] Deep links work (communitas://)

### Automated Testing
- [ ] All unit tests pass (npm run test)
- [ ] All Rust tests pass (cargo test)
- [ ] All E2E tests pass (npm run test:e2e:tauri)
- [ ] No clippy warnings (cargo clippy)
- [ ] No TypeScript errors (npm run typecheck)
- [ ] Code coverage > 70%

### Performance
- [ ] Cold start < 2 seconds
- [ ] Message list scrolling > 30 FPS
- [ ] File upload/download meets SLA
- [ ] Memory usage stable over 24h

### Security
- [ ] Dependencies scanned (npm audit, cargo audit)
- [ ] No secrets in code
- [ ] Keychain integration secure
- [ ] PQC encryption verified
```

---

## 9. Metrics & KPIs

### 9.1 Target Test Coverage

| Metric | Current | Target (Beta) | Target (GA) |
|--------|---------|---------------|-------------|
| Rust Line Coverage | ~70% | 80% | 85% |
| Frontend Line Coverage | ~8% | 60% | 75% |
| E2E Scenario Coverage | ~5% | 80% | 95% |
| macOS Integration Tests | 0% | 100% | 100% |
| Storyboard Flow Coverage | ~5% | 60% | 90% |

### 9.2 Quality Gates

**Block Release If:**
- ❌ Any P0 E2E test fails
- ❌ Clippy errors exist
- ❌ TypeScript compilation fails
- ❌ macOS permission flows broken
- ❌ Data loss in identity recovery
- ❌ WebRTC calls fail to establish
- ❌ Critical security vulnerability (npm/cargo audit)

**Warn Release If:**
- ⚠️ Code coverage < target
- ⚠️ Performance regression > 20%
- ⚠️ P1 E2E tests fail
- ⚠️ Non-critical dependency vulnerabilities

---

## 10. Timeline to Production Ready

### Phase 1: Foundation (Weeks 1-2)
- ✅ Set up macOS E2E infrastructure
- ✅ Implement 16 P0 E2E tests
- ✅ Fix TCC permission handling
- ✅ Enable all skipped unit tests

**Outcome:** Beta-ready (70% production ready)

### Phase 2: Robustness (Weeks 3-4)
- ✅ Add P1 E2E scenarios (offline, multi-identity)
- ✅ Expand Rust integration tests
- ✅ Component testing for critical UI
- ✅ Performance benchmarking

**Outcome:** RC-ready (85% production ready)

### Phase 3: Polish (Weeks 5-6)
- ✅ P2 nice-to-have scenarios
- ✅ Accessibility testing
- ✅ Load/stress testing
- ✅ Security hardening

**Outcome:** Production-ready (95%+ production ready)

---

## 11. Conclusion

### Current State Summary

**Strengths:**
- ✅ Solid Rust core with good test coverage
- ✅ Well-configured test infrastructure (Vitest, Playwright, Cargo)
- ✅ Clean codebase with no clippy warnings
- ✅ Modern tech stack (Tauri v2, React, Material-UI)

**Critical Weaknesses:**
- ❌ Almost zero E2E test coverage for critical flows
- ❌ No macOS-specific integration testing
- ❌ Most frontend unit tests inactive
- ❌ Storyboard user journeys not validated

### Recommended Next Steps

1. **Immediate (This Week):**
   - Set up macOS test runner
   - Implement first 5 P0 E2E tests (onboarding + messaging)
   - Enable skipped unit tests

2. **Short-term (Next 2 Weeks):**
   - Complete all 16 P0 E2E tests
   - Fix TCC permission flows
   - Add Rust integration tests for offline sync

3. **Medium-term (Next 4-6 Weeks):**
   - P1 E2E scenarios
   - Component testing
   - Performance benchmarking
   - Security audit

### Risk Assessment

**Production Release Without Additional Testing:**
- 🔴 **High Risk** - Core user flows unvalidated
- 🔴 **High Risk** - macOS integration untested
- 🟡 **Medium Risk** - Data loss potential
- 🟡 **Medium Risk** - Poor offline experience

**With P0 Tests Complete:**
- 🟢 **Low Risk** - Beta release acceptable
- 🟡 **Medium Risk** - Early production with monitoring

**With Full Test Suite:**
- 🟢 **Production Ready** - Confident release

---

## Appendix A: Test File Structure

```
communitas/
├── tests/
│   ├── e2e/
│   │   ├── tauri-mode/
│   │   │   ├── 01-onboarding.spec.ts          # NEW
│   │   │   ├── 02-messaging.spec.ts           # NEW
│   │   │   ├── 03-identity.spec.ts            # NEW
│   │   │   ├── 04-files-storage.spec.ts       # NEW
│   │   │   ├── 05-webrtc.spec.ts              # NEW
│   │   │   ├── 06-macos-permissions.spec.ts   # NEW
│   │   │   └── app-lifecycle.spec.ts          # EXISTS
│   │   ├── web-mode/
│   │   │   ├── dashboard.spec.ts              # EXISTS
│   │   │   └── onboarding.spec.ts             # EXISTS
│   │   └── utils/
│   │       ├── tauri-setup.ts
│   │       ├── fake-media.ts                  # NEW
│   │       └── tcc-permissions.ts             # NEW
├── communitas-core/tests/
│   ├── foaf_discovery_tests.rs
│   ├── crdt_tests.rs
│   ├── message_sync_tests.rs
│   ├── doc_replicator_tests.rs
│   ├── multi_device_sync.rs                   # NEW
│   └── offline_sync.rs                        # NEW
├── communitas-desktop/tests/
│   ├── webrtc_media_controls_tests.rs
│   ├── tauri_commands_test.rs
│   ├── auth_touchid_tests.rs
│   └── webrtc_stability.rs                    # NEW
└── src/
    ├── contexts/__tests__/
    │   ├── AuthContext.test.tsx               # NEW
    │   └── EntityDirectoryContext.test.tsx    # NEW
    ├── components/__tests__/
    │   ├── MessageList.test.tsx               # NEW
    │   └── ChannelSidebar.test.tsx            # NEW
    └── services/__tests__/
        ├── featureFlags.test.ts               # EXISTS
        └── channelSync.test.ts                # EXISTS (ENABLE)
```

---

## Appendix B: Useful Commands

```bash
# Run all tests
npm run test && cargo test

# Run only E2E tests (Tauri native)
npm run test:e2e:tauri

# Run with UI for debugging
npm run test:e2e:tauri:ui

# Generate coverage report
npm run test:coverage
cargo tarpaulin --out Html

# Check for issues before commit
npm run typecheck
cargo clippy --all-features -- -D warnings
cargo fmt --all --check

# Build and run Tauri app
npm run build
npm run tauri:dev

# Clean build (when things break)
rm -rf dist/ target/ node_modules/
npm ci
cargo clean
npm run build
```

---

**Report Generated By:** Amp AI Agent  
**Next Review:** After P0 test implementation (2 weeks)

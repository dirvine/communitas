# PLAN-33: Phase 5.3 - Performance QA

**Phase**: 5.3 Performance QA
**Milestone**: M5 Stabilization
**Created**: 2026-01-20
**Status**: Planning

## Goal

Establish performance baselines and benchmarks for core Communitas flows, with CI integration to catch regressions.

## Performance Targets

From CLAUDE.md:
- **Message Latency**: <100ms local, <500ms remote
- **Storage Operations**: <100ms local, <500ms with geographic routing
- **UI Responsiveness**: 60fps, smooth animations
- **Memory Usage**: <200MB baseline

## Tasks

### Task 1: Add Tracing Instrumentation to Core Flows

**Files to modify**:
- `communitas-ui-service/src/auth.rs`
- `communitas-ui-service/src/messaging.rs`
- `communitas-ui-service/src/kanban.rs`
- `communitas-ui-service/src/drive.rs`

**What to do**:
1. Add `#[instrument(skip(self), level = "debug")]` to key public methods
2. Add span events for important state transitions
3. Ensure timing data is captured via tracing spans

**Verification**:
- `cargo clippy -p communitas-ui-service -- -D warnings`
- `cargo build -p communitas-ui-service`

**Done when**:
- All 4 service files have instrumentation on public methods
- Tracing spans capture entry/exit timing

---

### Task 2: Create Benchmark Crate with Criterion

**Files to create**:
- `communitas-bench/Cargo.toml`
- `communitas-bench/benches/core_flows.rs`

**What to do**:
1. Create new crate with criterion dependency
2. Add benchmark groups: auth, messaging, kanban, drive
3. Start with auth benchmarks (login/logout/refresh flow)
4. Use mock data for deterministic benchmarks

**Verification**:
- `cargo build -p communitas-bench`
- `cargo bench -p communitas-bench -- --test` (quick validation)

**Done when**:
- Benchmark crate exists with auth benchmarks
- Crate compiles and benchmarks run

---

### Task 3: Add Messaging and Drive Benchmarks

**Files to modify**:
- `communitas-bench/benches/core_flows.rs`

**What to do**:
1. Add messaging benchmarks (list_threads, get_messages, send_message)
2. Add drive benchmarks (list_directory, read_file, write_file)
3. Use synthetic test data for consistent measurements

**Verification**:
- `cargo bench -p communitas-bench -- --test`
- All benchmark groups run without errors

**Done when**:
- Messaging and drive benchmark groups complete
- Each group has 3+ benchmarks

---

### Task 4: Add Kanban and Canvas Benchmarks

**Files to modify**:
- `communitas-bench/benches/core_flows.rs`

**What to do**:
1. Add kanban benchmarks (list_boards, get_board, move_task)
2. Add canvas benchmarks (load_canvas, add_element, update_transform)
3. Include benchmarks for CRDT operations

**Verification**:
- `cargo bench -p communitas-bench -- --test`
- All benchmark groups run

**Done when**:
- Kanban and canvas benchmark groups complete
- CRDT operations included

---

### Task 5: Establish Baseline Measurements

**Files to create**:
- `docs/performance/BASELINE.md`
- `docs/performance/README.md`

**What to do**:
1. Run full benchmark suite and capture results
2. Document baseline measurements for each flow
3. Compare against target thresholds from CLAUDE.md
4. Note any flows exceeding targets

**Verification**:
- `cargo bench -p communitas-bench` completes
- BASELINE.md contains actual numbers

**Done when**:
- Baseline measurements documented
- Comparison to targets included

---

### Task 6: Add CI Performance Regression Check

**Files to modify**:
- `.github/workflows/ci.yml` (or new workflow)

**What to do**:
1. Add benchmark job to CI (weekly or on performance-related PRs)
2. Configure criterion comparison against baseline
3. Fail CI if regression >5% on critical paths
4. Store benchmark results as artifacts

**Verification**:
- CI workflow syntax valid: `act -l` or GitHub action validator
- Benchmark job runs in CI (test via PR)

**Done when**:
- CI includes benchmark job
- Regression detection configured

---

## Summary

| Task | Description | Crate |
|------|-------------|-------|
| 1 | Add tracing instrumentation | communitas-ui-service |
| 2 | Create benchmark crate | communitas-bench (new) |
| 3 | Messaging/drive benchmarks | communitas-bench |
| 4 | Kanban/canvas benchmarks | communitas-bench |
| 5 | Baseline documentation | docs/performance |
| 6 | CI regression check | .github/workflows |

## Dependencies

- criterion crate for benchmarks
- tracing/tracing-subscriber for instrumentation
- No new external services required

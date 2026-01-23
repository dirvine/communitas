# Communitas Testing Strategy

This document describes the overall testing strategy for the Communitas platform.

## Test Pyramid

Communitas follows a traditional test pyramid with emphasis on lower-level tests:

```
         /‾‾‾‾‾‾‾‾\
        /  E2E    \          <- 10% - Full UI flows
       /____________\
      /              \
     /  Integration   \       <- 30% - Service layer tests
    /__________________\
   /                    \
  /       Unit          \     <- 60% - Core logic tests
 /________________________\
```

### Unit Tests (60%)

**Location**: `src/` directories within each crate
**Pattern**: `#[cfg(test)] mod tests { ... }`

Unit tests cover:
- Individual functions and methods
- Type invariants
- Error handling paths
- Edge cases in core logic

**Crates with unit tests**:
- `communitas-core` - Core business logic
- `communitas-kanban` - Kanban CRDT operations
- `canvas-core` - Canvas element operations

### Integration Tests (30%)

**Location**: `tests/` directories within each crate
**Pattern**: Separate test files (`foo_e2e.rs`)

Integration tests cover:
- Service-to-service interactions
- Storage round-trips
- Command/Event processing
- Multi-step workflows

**Key integration test suites**:
- `communitas-ui-service/tests/auth_e2e.rs` - Authentication flows
- `communitas-ui-service/tests/messaging_e2e.rs` - Messaging operations
- `communitas-ui-service/tests/kanban_e2e.rs` - Kanban board operations
- `communitas-ui-service/tests/canvas_e2e.rs` - Canvas operations

### E2E Tests (10%)

**Location**: `communitas-ui-service/tests/`
**Pattern**: Full application stack tests

E2E tests cover:
- Complete user workflows
- Cross-service operations
- Real-world usage scenarios
- UI service parity with core

## Coverage Targets

| Layer | Target | Current |
|-------|--------|---------|
| Core Logic | 80% | ~75% |
| Service Layer | 70% | ~65% |
| Integration | 60% | ~60% |
| E2E Flows | Key paths | 100% |

## Testing Approaches

### Parity Testing

UI services must behave identically to core services. Parity tests verify:

1. **Command parity**: UI service commands produce same events as core
2. **Query parity**: UI service queries return same data as core
3. **Error parity**: UI service errors match core error semantics

Example parity test structure:
```rust
// Core execution
let core_events = app.execute(cmd.clone()).await?;

// UI service execution
let ui_result = services.kanban().create_board(...).await?;

// Verify parity
assert_events_match(core_events, ui_result);
```

### Property-Based Testing

Using `proptest` for invariant verification across random inputs:

- **Board invariants**: Columns maintain order, cards belong to valid columns
- **Message invariants**: Queue order preserved, content integrity maintained
- **Canvas invariants**: Undo/redo are inverses, snapshots consistent

Configuration:
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]  // Standard
    #![proptest_config(ProptestConfig::with_cases(50))]  // Lightweight tests
    #![proptest_config(ProptestConfig::with_cases(10))]  // Heavy tests
}
```

### Stress Testing

Load and performance tests verify system stability:

| Test | Operations | Purpose |
|------|-----------|---------|
| Concurrent messages | 100 | Messaging concurrency |
| Kanban card ops | 1000 | Kanban throughput |
| Large canvas | 100 elements | Canvas scaling |
| Rapid undo/redo | 150 ops | History performance |
| Memory stability | Configurable | Memory leak detection |
| Concurrent entities | 10 parallel | Entity isolation |

Stress tests are marked `#[ignore]` for nightly-only execution.

### Edge Case Testing

Dedicated tests for boundary conditions:

- **Empty states**: New boards, empty message threads, blank canvases
- **Invalid operations**: Nonexistent resources, invalid moves
- **Concurrency**: Parallel operations on same resource
- **Large data**: Very long titles, many items
- **Error recovery**: State consistency after failures

### Offline Sync Testing

Tests for offline-first behavior:

- **Queue operations**: Pending items persist correctly
- **Flush behavior**: Empty queue handling
- **Undo/redo offline**: History operations without network
- **State consistency**: Snapshots remain valid during offline

## Test Data Management

### Isolation

Each test uses its own `TempDir` for complete isolation:

```rust
let temp = TempDir::new().unwrap();
let services = make_authenticated_services(&temp).await;
```

### Demo Mode

Tests use demo mode to bypass real authentication:

```rust
services.auth().enable_demo_mode();
```

### Entity Creation

Tests create project entities for scope isolation:

```rust
let entity_id = create_test_entity(&services, "TestEntity").await;
```

## CI/CD Integration

### PR Checks (Every Push)

1. **rust-checks**: Format, lint, audit
2. **integration-tests**: Unit + integration + property tests
3. **headless-build**: Cross-platform binary builds

### Nightly Runs

1. **stress-tests**: Load and performance tests
2. **Extended property tests**: More cases per test

### Release Gating

Before release:
1. All unit tests pass
2. All integration tests pass
3. All property tests pass
4. No clippy warnings
5. Clean security audit

## Quality Standards

### Test Code Quality

- Tests may use `unwrap()`/`expect()` for clarity
- Helpers should not use `unwrap()` (use `?` or `expect`)
- Descriptive assertion messages required
- TempDir for all storage operations

### Assertion Patterns

```rust
// Good - descriptive message
assert!(result.is_ok(), "Board creation should succeed");
assert_eq!(board.columns.len(), 3, "Should have exactly 3 columns");

// Bad - no context on failure
assert!(result.is_ok());
assert_eq!(board.columns.len(), 3);
```

### Property Test Assertions

```rust
// Use prop_assert! for property tests
prop_assert!(board.columns.len() >= 1, "Board should have at least 1 column");
prop_assert_eq!(&actual, &expected, "Values should match");
```

## Running Tests Locally

### Quick Smoke Test

```bash
cargo test --workspace --lib
```

### Full Test Suite

```bash
cargo test --workspace
```

### Stress Tests

```bash
STRESS_DURATION_SECS=30 cargo test -p communitas-ui-service --test stress -- --ignored --test-threads=1
```

### Coverage Report

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
open tarpaulin-report.html
```

## Future Improvements

1. **Visual regression tests**: Screenshot comparison for UI
2. **Performance benchmarks**: Criterion-based benchmarks in CI
3. **Fuzzing**: cargo-fuzz for security-critical code
4. **Contract tests**: API contract verification
5. **Chaos testing**: Network failure simulation

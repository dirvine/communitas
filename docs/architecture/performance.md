# Performance Architecture

This document describes performance optimizations, targets, and measurement approaches for Communitas.

## Performance Targets

| Metric | Target | Current Status |
|--------|--------|----------------|
| Startup time (to first render) | < 2000ms | Optimized in Phase 6.8 |
| Message latency (local) | < 100ms | Target |
| Storage operations (local) | < 100ms | Target |
| UI responsiveness | 60fps | Target |
| Memory baseline | < 200MB | Optimized in Phase 6.8 |

## Phase 6.8 Optimizations

Phase 6.8 focused on systematic performance improvements across startup, memory, UI rendering, async patterns, and storage I/O.

### 1. Runtime Optimization

**Problem:** Nested Tokio runtime creation blocked the main thread during bootstrap.

**Solution:** Introduced `bootstrap_async()` method that uses the existing Tokio runtime from Dioxus instead of creating a new one with `block_on()`.

**Impact:** Estimated 200-500ms startup time reduction.

### 2. Lazy Device Enumeration

**Problem:** Audio/video device enumeration (100-300ms) ran at startup even when call UI wasn't needed.

**Solution:** Device enumerator is now lazily initialized when call UI is first accessed via `CallService::ensure_device_enumerator()`.

**Impact:** Estimated 100-300ms faster initial render.

### 3. LRU Cache for CRDT Documents

**Problem:** `HashMap<String, Arc<Doc>>` held all board documents indefinitely, causing unbounded memory growth.

**Solution:** Replaced with `LruCache` (capacity: 50 boards) that properly cleans up subscriptions on eviction.

**Impact:** Estimated 20-50MB memory reduction with many boards.

### 4. Debounced Operations

**Auto-login debouncing:** Prevents duplicate login attempts when auth state flickers.

**Directory refresh debouncing:** 200ms debounce prevents network cascades when auth state changes rapidly.

**CRDT subscription debouncing:** 50ms buffer collapses rapid Yrs updates into batched UI notifications.

**Impact:** Reduced duplicate network calls, fewer UI re-renders.

### 5. Single-Pass Entity Categorization

**Problem:** HomeOverview filtered entities 6 times (O(n*6)) per render.

**Solution:** `CategorizedEntities::from_entities()` categorizes all entities in a single O(n) pass using match statements.

**Impact:** Estimated 50-100ms per render reduction for large entity lists.

### 6. Parallel File I/O

**Problem:** Sequential temp file writes for CRDT persistence.

**Solution:** Use `tokio::join!` to write temp files in parallel (renames remain sequential for atomicity).

**Impact:** Reduced I/O latency on multi-core systems.

### 7. Signal Consolidation

**Problem:** Multiple boolean signals for card interaction state caused unnecessary re-renders.

**Solution:** Single `CardInteractionState` enum with mutually exclusive states (Idle, Dragging, Moving, ShowingDetailModal, etc.).

**Impact:** Fewer signal subscriptions, reduced re-render frequency.

### 8. Iterator-Based Swimlane Grouping

**Problem:** Intermediate `Vec` allocations in swimlane grouping with `.filter().cloned().collect()` patterns.

**Solution:** Grouping functions take `impl Iterator<Item = &CardView>` to avoid intermediate collection. `group_by_state` uses fixed-size array indexed by state ordinal.

**Impact:** Reduced Vec allocations per swimlane render.

## Startup Timing Instrumentation

Startup phases are instrumented with tracing spans:

```
bootstrap_async
├── storage_discovery
├── identity_generation
├── core_app_init
└── services_init
    ├── auth_init
    ├── navigation_init
    ├── messaging_init
    └── kanban_init
```

Run with `RUST_LOG=info` to see timing:
```
INFO bootstrap_async: Bootstrap complete in 850.3ms elapsed_ms=850
INFO App: First render complete, total startup: 1234.5ms elapsed_ms=1234
```

Debug-level spans provide more granular timing:
```
RUST_LOG=communitas_ui_service=debug cargo run -p communitas-dioxus
```

## Measurement Approach

### Manual Testing

1. Start the application with timing enabled:
   ```bash
   RUST_LOG=info cargo run -p communitas-dioxus --release
   ```

2. Observe startup metrics in the logs.

### Benchmarking

Run startup benchmarks:
```bash
cargo bench -p communitas-dioxus --bench startup
```

### Memory Profiling

Use `heaptrack` or similar tools to track memory usage:
```bash
heaptrack target/release/communitas-dioxus
```

## Optimization Guidelines

### Do

- Use `#[tracing::instrument]` for functions > 10ms
- Prefer iterators over collected Vecs when possible
- Use debouncing for operations that can fire rapidly
- Lazy-initialize expensive resources
- Profile before optimizing

### Don't

- Create nested Tokio runtimes
- Clone large structs unnecessarily
- Hold locks across await points
- Allocate in hot paths without measurement
- Optimize without benchmarks

## Future Optimization Opportunities

1. **WASM target**: Profile and optimize for WebAssembly build
2. **Virtual scrolling**: For large lists (> 1000 items)
3. **Incremental CRDT sync**: Only sync changed portions
4. **Connection pooling**: For gossip network connections
5. **Image lazy loading**: Defer non-visible image loads

# Communitas Performance Benchmarks

This document captures baseline performance measurements for core Communitas operations.

## Performance Targets

From `CLAUDE.md`:
- **Message Latency**: <100ms local, <500ms remote
- **Storage Operations**: <100ms local, <500ms with geographic routing
- **UI Responsiveness**: 60fps, smooth animations
- **Memory Usage**: <200MB baseline

## Baseline Measurements

Captured on: 2026-01-20
System: macOS Darwin 25.2.0

### Messaging Service

| Operation | Median | Lower Bound | Upper Bound | Status |
|-----------|--------|-------------|-------------|--------|
| list_threads | 16.07 µs | 15.66 µs | 16.49 µs | Well under 100ms |
| get_messages | 331 ns | 330.67 ns | 331.92 ns | Well under 100ms |
| send_message | 42.19 µs | 40.12 µs | 43.91 µs | Well under 100ms |

### Kanban Service

| Operation | Median | Lower Bound | Upper Bound | Status |
|-----------|--------|-------------|-------------|--------|
| list_boards | 746 ns | 741.29 ns | 750.91 ns | Well under 100ms |
| get_board | 1.41 µs | 1.40 µs | 1.42 µs | Well under 100ms |
| create_card | 652 ns | 648.18 ns | 656.62 ns | Well under 100ms |

### Drive Service

| Operation | Median | Lower Bound | Upper Bound | Status |
|-----------|--------|-------------|-------------|--------|
| list_directory | 43.50 µs | 43.20 µs | 43.83 µs | Well under 100ms |
| read_file | 2.18 µs | 2.18 µs | 2.19 µs | Well under 100ms |
| write_file | 318.26 µs | 315.63 µs | 321.32 µs | Well under 100ms |

### Canvas Service

| Operation | Median | Lower Bound | Upper Bound | Status |
|-----------|--------|-------------|-------------|--------|
| load_canvas | 558 ns | 557.25 ns | 559.26 ns | Well under 100ms |
| add_text | 16.32 µs | 15.46 µs | 17.05 µs | Well under 100ms |
| update_transform | 477 ns | 475.59 ns | 477.76 ns | Well under 100ms |

## Running Benchmarks

### Quick Test (verify benchmarks compile)
```bash
cargo bench --bench core_flows -- --test
```

### Full Benchmark Suite
```bash
cargo bench --bench core_flows
```

### Specific Benchmark Group
```bash
cargo bench --bench core_flows -- messaging
cargo bench --bench core_flows -- kanban
cargo bench --bench core_flows -- drive
cargo bench --bench core_flows -- canvas
```

### With HTML Report
```bash
cargo bench --bench core_flows
# Reports generated in: target/criterion/
```

## CI Regression Threshold

A performance regression is flagged when an operation exceeds:
- **5% deviation** from baseline median

Criterion automatically detects regressions by comparing against previous runs stored in `target/criterion/`.

## Benchmark Architecture

```
communitas-bench/
├── Cargo.toml          # Benchmark crate config
├── src/lib.rs          # Re-exports for benchmarks
└── benches/
    └── core_flows.rs   # Criterion benchmarks
```

### Adding New Benchmarks

1. Add the service to `core_flows.rs` imports
2. Create a helper function: `create_<service>_service(temp: &TempDir)`
3. Add a benchmark group: `fn <service>_benchmarks(c: &mut Criterion)`
4. Register in `criterion_group!` macro

Example pattern:
```rust
fn my_benchmarks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().expect("bench: create runtime");
    let mut group = c.benchmark_group("my_service");

    group.bench_function("operation", |b| {
        let temp = create_temp_dir();
        let service = rt.block_on(create_my_service(&temp));
        b.to_async(&rt).iter(|| async {
            let result = service.operation(black_box("input")).await;
            black_box(result)
        });
    });

    group.finish();
}
```

## Key Observations

1. **All local operations are sub-millisecond** - well within the 100ms target
2. **Write operations** (send_message, write_file, add_text) take ~10-50x longer than reads
3. **CRDT operations** (kanban, canvas) are extremely fast (~500ns-1.5µs)
4. **Demo mode overhead** is minimal - benchmarks use `enable_demo_mode()` for auth

## Future Improvements

- [ ] Add auth benchmarks (currently placeholder)
- [ ] Add network latency simulation for remote operations
- [ ] Add memory usage benchmarks
- [ ] Add concurrent operation benchmarks
- [ ] Integrate with CI for automated regression detection

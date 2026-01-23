# E2E Testing Guide

This document describes how to run and write End-to-End (E2E) tests for the Communitas UI services.

## Prerequisites

### Rust Toolchain

Ensure you have Rust 1.85+ installed:

```bash
rustup update
rustc --version  # Should be 1.85.0 or later
```

### System Dependencies

**macOS:**
```bash
# WebKit is bundled with Safari - no additional deps needed
xcode-select --install  # For build tools
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential \
  libssl-dev \
  pkg-config \
  libgtk-3-dev \
  libsoup-3.0-dev
# WebKitGTK (one of these depending on your system):
sudo apt-get install -y libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev || \
sudo apt-get install -y libwebkit2gtk-4.0-dev libjavascriptcoregtk-4.0-dev
```

**Windows:**
```powershell
# Install Visual Studio Build Tools with C++ workload
# CMake 3.20+ required for aws-lc-rs
# See docs/development/windows-build.md for details
```

## Running Tests

### Unit Tests

Run all unit tests across the workspace:

```bash
cargo test --workspace --lib
```

Run unit tests for a specific crate:

```bash
cargo test -p communitas-core --lib
cargo test -p communitas-kanban --lib
cargo test -p communitas-ui-service --lib
```

### Integration Tests

Run all integration tests:

```bash
cargo test --workspace --test '*'
```

Run specific integration test files:

```bash
# E2E tests for auth flow
cargo test -p communitas-ui-service --test auth_e2e

# E2E tests for messaging
cargo test -p communitas-ui-service --test messaging_e2e

# E2E tests for file drive
cargo test -p communitas-ui-service --test drive_e2e

# E2E tests for calls
cargo test -p communitas-ui-service --test calls_e2e

# E2E tests for canvas
cargo test -p communitas-ui-service --test canvas_e2e

# E2E tests for kanban
cargo test -p communitas-ui-service --test kanban_e2e

# Edge case tests
cargo test -p communitas-ui-service --test edge_case_e2e

# Offline sync tests
cargo test -p communitas-ui-service --test offline_e2e
```

### Property-Based Tests

Run property tests using proptest:

```bash
cargo test -p communitas-ui-service --test property_tests
```

Property tests verify invariants across random inputs. Default configuration runs 20 cases per test.

### Stress Tests

Stress tests are marked `#[ignore]` to avoid running during normal CI. Run them explicitly:

```bash
# Run all stress tests
cargo test -p communitas-ui-service --test stress -- --ignored

# Run with limited parallelism (recommended for stress tests)
cargo test -p communitas-ui-service --test stress -- --ignored --test-threads=1

# Configure stress duration (default: 10 seconds)
STRESS_DURATION_SECS=60 cargo test -p communitas-ui-service --test stress -- --ignored
```

## Test Structure

### File Organization

```
communitas-ui-service/
  tests/
    auth_e2e.rs        # Authentication flow E2E tests
    messaging_e2e.rs   # Messaging E2E tests
    drive_e2e.rs       # File drive E2E tests
    calls_e2e.rs       # Calls E2E tests
    canvas_e2e.rs      # Canvas E2E tests
    kanban_e2e.rs      # Kanban board E2E tests
    edge_case_e2e.rs   # Edge case handling tests
    offline_e2e.rs     # Offline sync behavior tests
    property_tests.rs  # Property-based tests with proptest
    stress.rs          # Load and performance stress tests
```

### Common Patterns

All E2E tests follow these patterns:

1. **Large Stack Pattern**: Tests run in threads with 8MB stack to handle async state machines:
   ```rust
   const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

   fn run_with_large_stack<F>(f: F)
   where
       F: FnOnce() + Send + 'static,
   {
       std::thread::Builder::new()
           .stack_size(TEST_STACK_SIZE)
           .spawn(f)
           .unwrap()
           .join()
           .unwrap();
   }
   ```

2. **Demo Mode Authentication**: Tests use demo mode to bypass real authentication:
   ```rust
   services.auth().enable_demo_mode();
   ```

3. **TempDir Isolation**: Each test uses its own temporary directory:
   ```rust
   let temp = TempDir::new().unwrap();
   let services = make_authenticated_services(&temp).await;
   ```

4. **Entity Creation Helper**: Tests create project entities for isolation:
   ```rust
   let entity_id = create_test_entity(&services, "TestEntity").await;
   ```

## Writing New Tests

### E2E Test Template

```rust
use std::sync::Arc;
use communitas_core::app::CommunitasApp;
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use tempfile::TempDir;

const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

async fn make_authenticated_services(temp: &TempDir) -> UiServices {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path().join("app_storage").to_string_lossy().to_string(),
        )
        .await
        .unwrap(),
    );
    let services = UiServices::new(storage, app).unwrap();
    services.auth().enable_demo_mode();
    services
}

#[test]
fn test_my_feature() {
    std::thread::Builder::new()
        .stack_size(TEST_STACK_SIZE)
        .spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let temp = TempDir::new().unwrap();
                let services = make_authenticated_services(&temp).await;

                // Your test logic here

                assert!(true);
            });
        })
        .unwrap()
        .join()
        .unwrap();
}
```

### Property Test Template

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn prop_my_invariant(input in any::<String>()) {
        std::thread::Builder::new()
            .stack_size(TEST_STACK_SIZE)
            .spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    // Test invariant with random input
                    prop_assert!(!input.is_empty());
                    Ok(())
                })
            })
            .unwrap()
            .join()
            .unwrap()?;
    }
}
```

### Stress Test Template

```rust
#[test]
#[ignore = "stress test - run with --ignored flag"]
fn stress_my_scenario() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let temp = TempDir::new().unwrap();
            let services = make_authenticated_services(&temp).await;

            let start = Instant::now();

            // High-volume operations here

            let elapsed = start.elapsed();
            println!("Stress test completed in {:?}", elapsed);

            assert!(elapsed < Duration::from_secs(60));
        });
    });
}
```

## CI Integration

Tests run automatically in GitHub Actions:

- **On every PR**: Unit tests, integration tests, property tests
- **Nightly**: Stress tests (scheduled workflow)
- **Manual**: All tests can be triggered via workflow dispatch

See `.github/workflows/ci.yml` for the full configuration.

## Troubleshooting

### Stack Overflow in Tests

If tests crash with stack overflow, ensure you're using the large stack pattern:

```rust
std::thread::Builder::new()
    .stack_size(8 * 1024 * 1024)
    .spawn(...)
```

### Tests Hang on macOS

Ensure WebKit is available. On macOS, Safari provides WebKit. If using a minimal install:

```bash
softwareupdate --install --all
```

### Tests Fail with "demo mode not enabled"

Ensure you call `services.auth().enable_demo_mode()` before any authenticated operations.

### Property Tests Fail Randomly

Increase the number of test cases or use a specific seed:

```rust
#![proptest_config(ProptestConfig::with_cases(100))]

// Or with a specific seed for reproducibility:
// PROPTEST_CASES=100 cargo test -p communitas-ui-service --test property_tests
```

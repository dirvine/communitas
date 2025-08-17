# Saorsa Storage System - Testing Guide

## Overview

The Saorsa Storage System implements comprehensive testing across multiple layers to ensure reliability, security, and performance. This guide covers our testing strategy, available test suites, and how to run and interpret tests.

## Testing Philosophy

### Quality Standards
- **Zero Tolerance**: No compilation errors or warnings in production code
- **Complete Coverage**: >85% test coverage requirement
- **Test-Driven Development**: Tests written before implementation
- **Property-Based Testing**: Extensive use of property-based tests for edge cases
- **Performance Validation**: Automated performance regression detection

### Testing Pyramid
```
┌─────────────────────────────────────┐
│         E2E Tests (Future)          │  ← Full application testing
├─────────────────────────────────────┤
│        Integration Tests            │  ← Multi-component testing
├─────────────────────────────────────┤
│           Unit Tests               │  ← Individual component testing
├─────────────────────────────────────┤
│        Property Tests              │  ← Edge case and invariant testing
└─────────────────────────────────────┘
```

## Test Categories

### 1. Unit Tests

#### Core Component Tests
Located in each module's `tests` submodule:

```rust
// Example: Content addressing tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_blake3_consistency() {
        let content = b"test content";
        let addressing = ContentAddressing::new();
        
        let hash1 = addressing.generate_content_id(content, "test");
        let hash2 = addressing.generate_content_id(content, "test");
        
        assert_eq!(hash1, hash2);
    }
}
```

#### Running Unit Tests
```bash
cd src-tauri

# Run all unit tests
cargo test

# Run specific module tests
cargo test saorsa_storage::content::tests
cargo test saorsa_storage::cache::tests
cargo test saorsa_storage::policy::tests

# Run with output for debugging
cargo test -- --nocapture

# Run specific test
cargo test test_blake3_consistency
```

### 2. Integration Tests

#### Storage Engine Integration
Located in `src-tauri/src/storage_tests.rs`:

```rust
#[tokio::test]
async fn test_multi_policy_workflow() {
    let engine = setup_storage_engine().await;
    
    let policies = vec![
        StoragePolicy::PrivateMax,
        StoragePolicy::PrivateScoped { namespace: "test".to_string() },
        StoragePolicy::GroupScoped { group_id: "group1".to_string() },
        StoragePolicy::PublicMarkdown,
    ];
    
    for policy in policies {
        let result = test_store_retrieve_cycle(&engine, policy).await;
        assert!(result.is_ok());
    }
}
```

#### Running Integration Tests
```bash
# Run all storage integration tests
cargo test storage_tests --lib

# Run specific integration test patterns
cargo test test_multi_policy
cargo test test_namespace_isolation
cargo test test_content_reconstruction
```

### 3. Property-Based Tests

#### Using proptest for Edge Cases
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_content_addressing_properties(
        content in prop::collection::vec(any::<u8>(), 0..1024*1024)
    ) {
        let addressing = ContentAddressing::new();
        
        // Property: Same content always produces same hash
        let hash1 = addressing.generate_content_id(&content, "test");
        let hash2 = addressing.generate_content_id(&content, "test");
        prop_assert_eq!(hash1, hash2);
        
        // Property: Different content produces different hashes
        if !content.is_empty() {
            let mut modified = content.clone();
            modified[0] = modified[0].wrapping_add(1);
            let hash3 = addressing.generate_content_id(&modified, "test");
            prop_assert_ne!(hash1, hash3);
        }
    }
}
```

#### Running Property Tests
```bash
# Run property tests with more iterations
PROPTEST_CASES=1000 cargo test test_content_addressing_properties

# Run all property tests
cargo test proptest
```

### 4. Performance Tests

#### Benchmark Testing
```rust
#[tokio::test]
async fn test_performance_targets() {
    let engine = setup_storage_engine().await;
    let content = vec![0u8; 1024]; // 1KB test content
    
    let start = Instant::now();
    let result = engine.store_content(create_test_request(content)).await;
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 100, "Store operation too slow: {}ms", duration.as_millis());
}
```

#### Running Performance Tests
```bash
# Run performance validation tests
cargo test test_performance

# Run comprehensive performance test via Tauri command
npm run tauri dev
# Then in the app: invoke('run_storage_performance_comprehensive_test')

# Run custom performance test
cargo test --release --test performance_benchmarks
```

### 5. Security Tests

#### Encryption and Access Control Tests
```rust
#[tokio::test]
async fn test_namespace_isolation() {
    let engine = setup_storage_engine().await;
    
    // Store content in namespace A
    let content_a = b"secret content A";
    let request_a = StorageRequest {
        content: content_a.to_vec(),
        policy: StoragePolicy::PrivateScoped { 
            namespace: "namespace_a".to_string() 
        },
        // ... other fields
    };
    
    let response_a = engine.store_content(request_a).await.unwrap();
    
    // Attempt to retrieve with wrong namespace should fail
    let wrong_retrieval = RetrievalRequest {
        address: StorageAddress::new(
            response_a.address.content_id,
            StoragePolicy::PrivateScoped { 
                namespace: "namespace_b".to_string() 
            }
        ),
        user_id: "test_user".to_string(),
        decryption_key: None,
    };
    
    let result = engine.retrieve_content(wrong_retrieval).await;
    assert!(result.is_err()); // Should fail due to wrong namespace
}
```

## Test Suites

### Core Test Suite (`storage_tests.rs`)

#### Content Tests
- `test_optimal_chunk_size`: Validates 256KB optimal chunk size
- `test_content_reconstruction`: Tests Reed-Solomon reconstruction
- `test_blake3_consistency`: Verifies content addressing consistency

#### Policy Tests  
- `test_policy_enforcement`: Validates policy constraints
- `test_policy_transitions`: Tests policy migration
- `test_access_control`: Verifies permission enforcement

#### Namespace Tests
- `test_namespace_isolation`: Ensures cryptographic separation
- `test_key_derivation`: Validates HKDF key derivation
- `test_namespace_cleanup`: Tests key rotation and cleanup

#### Cache Tests
- `test_lru_eviction`: Validates cache eviction algorithms
- `test_compression`: Tests compression/decompression
- `test_integrity_verification`: Validates BLAKE3 checksums

#### Network Tests
- `test_dht_operations`: Tests DHT store/retrieve operations
- `test_geographic_routing`: Validates peer selection
- `test_retry_logic`: Tests network retry mechanisms

#### Performance Tests
- `test_local_performance`: <100ms local operation validation
- `test_remote_performance`: <500ms remote operation validation
- `test_cache_performance`: <50ms cache hit validation

### Running the Complete Test Suite

#### Quick Validation
```bash
# Run fast unit tests only
cargo test --lib

# Run storage tests specifically
cargo test storage_tests

# Run with timing information
cargo test --lib -- --show-output
```

#### Comprehensive Testing
```bash
# Run all tests including integration
cargo test --all-targets

# Run with logging enabled
RUST_LOG=debug cargo test storage_tests

# Run performance tests in release mode
cargo test --release test_performance
```

#### Continuous Integration
```bash
# CI test command (used in GitHub Actions)
cargo test --all-targets --all-features -- --test-threads=1

# Check for warnings
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

## Test Data and Fixtures

### Test Data Generation
```rust
// Generate deterministic test content
fn generate_test_content(size: usize, seed: u64) -> Vec<u8> {
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;
    
    let mut rng = StdRng::seed_from_u64(seed);
    (0..size).map(|_| rng.gen()).collect()
}

// Create test storage request
fn create_test_request(content: Vec<u8>) -> StorageRequest {
    StorageRequest {
        content,
        content_type: "application/octet-stream".to_string(),
        policy: StoragePolicy::PrivateMax,
        metadata: StorageMetadata::default(),
        user_id: "test_user".to_string(),
        group_id: None,
        namespace: None,
    }
}
```

### Test Environment Setup
```rust
async fn setup_storage_engine() -> StorageEngine<LocalDht> {
    let dht = Arc::new(LocalDht::new("test_node".to_string()));
    let master_key = [42u8; 32]; // Deterministic for testing
    let config_manager = ConfigManager::new();
    
    StorageEngine::new(dht, master_key, config_manager)
        .await
        .expect("Failed to create test storage engine")
}
```

## Test Configuration

### Test-Specific Configuration
```json
{
  "cache": {
    "max_size_bytes": 10485760,     // 10MB for tests
    "max_entries": 1000,
    "default_ttl_secs": 60,         // Short TTL for tests
    "cleanup_interval_secs": 10     // Frequent cleanup
  },
  "network": {
    "operation_timeout_secs": 5,    // Short timeout for tests
    "retry_attempts": 2,            // Fewer retries
    "max_concurrent_operations": 10
  },
  "content": {
    "optimal_chunk_size": 262144,   // Keep production size
    "enable_compression": false     // Disable for test speed
  }
}
```

### Environment Variables for Testing
```bash
# Enable test mode
export SAORSA_TEST_MODE="true"

# Reduce log noise in tests
export RUST_LOG="warn,saorsa_storage=info"

# Use in-memory storage for tests
export SAORSA_USE_MEMORY_STORAGE="true"

# Disable network timeouts for debugging
export SAORSA_DISABLE_TIMEOUTS="true"
```

## Test Debugging

### Debug Output
```bash
# Run single test with full output
cargo test test_content_reconstruction -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test storage_tests::test_namespace_isolation

# Run with backtraces on panic
RUST_BACKTRACE=1 cargo test
```

### Test Profiling
```bash
# Profile test performance
cargo test --release test_performance -- --measure-time

# Memory profiling with valgrind
valgrind --tool=massif cargo test test_cache_operations

# CPU profiling with perf
perf record cargo test test_storage_engine
perf report
```

## Continuous Integration

### GitHub Actions Integration
```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: |
          cd src-tauri
          cargo test --all-targets --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Performance validation
        run: cargo test --release test_performance
```

### Test Coverage
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out html --output-dir coverage

# View coverage report
open coverage/tarpaulin-report.html
```

## Test Maintenance

### Test Organization
- **Atomic Tests**: Each test validates one specific behavior
- **Descriptive Names**: Test names clearly describe what is being tested
- **Isolated Tests**: Tests don't depend on each other
- **Deterministic**: Tests produce consistent results

### Adding New Tests
1. **Identify Test Category**: Unit, integration, or property test
2. **Write Failing Test**: Follow TDD principles
3. **Implement Feature**: Make the test pass
4. **Verify Coverage**: Ensure adequate test coverage
5. **Document Test**: Add clear comments and documentation

### Test Best Practices
- Use `#[tokio::test]` for async tests
- Prefer `Result<(), Error>` return types for test functions
- Use `assert_eq!` with descriptive messages
- Clean up test resources in test teardown
- Use deterministic test data when possible

This comprehensive testing approach ensures the Saorsa Storage System maintains high quality, security, and performance standards throughout development and deployment.
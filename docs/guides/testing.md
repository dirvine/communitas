# Testing Guide

Comprehensive guide to testing Communitas - from unit tests to full system integration tests.

## Testing Overview

Communitas uses a multi-layered testing approach:

1. **Unit Tests** - Test individual functions and modules
2. **Integration Tests** - Test component interactions
3. **End-to-End Tests** - Test complete user workflows
4. **Browser Tests** - Test via HTTP/REST bridge with Chrome DevTools MCP
5. **Performance Tests** - Benchmark critical paths
6. **Security Tests** - Verify cryptographic operations

## Quick Start

### Run All Tests

```bash
# Frontend tests (Vitest)
npm test

# Backend tests (Cargo)
cargo test

# Full test suite
npm test && cargo test
```

### Run Specific Test Suites

```bash
# Frontend unit tests
npm run test:unit

# Frontend with coverage
npm run test:coverage

# Backend tests with logging
RUST_LOG=debug cargo test

# Specific test file
cargo test -p communitas-core --test integration_test

# Single test function
cargo test test_four_word_validation
```

## Frontend Testing (TypeScript/React)

### Unit Tests with Vitest

#### Test File Structure

```
src/
├── components/
│   ├── AuthDialog.tsx
│   └── AuthDialog.test.tsx       # Unit tests
├── services/
│   ├── NetworkService.ts
│   └── NetworkService.test.ts    # Service tests
└── utils/
    ├── fourWords.ts
    └── fourWords.test.ts          # Utility tests
```

#### Example Component Test

```typescript
// src/components/AuthDialog.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { AuthDialog } from './AuthDialog';

describe('AuthDialog', () => {
  it('renders login form', () => {
    render(<AuthDialog open={true} onClose={() => {}} />);

    expect(screen.getByLabelText('Four-Word Address')).toBeInTheDocument();
    expect(screen.getByLabelText('Password')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Login' })).toBeInTheDocument();
  });

  it('validates four-word address format', async () => {
    render(<AuthDialog open={true} onClose={() => {}} />);

    const input = screen.getByLabelText('Four-Word Address');
    fireEvent.change(input, { target: { value: 'invalid-address' } });

    expect(await screen.findByText('Must be exactly four words')).toBeInTheDocument();
  });

  it('calls onLogin when form submitted', async () => {
    const onLogin = vi.fn();
    render(<AuthDialog open={true} onClose={() => {}} onLogin={onLogin} />);

    fireEvent.change(screen.getByLabelText('Four-Word Address'), {
      target: { value: 'ocean-forest-moon-star' }
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'secure-password' }
    });
    fireEvent.click(screen.getByRole('button', { name: 'Login' }));

    expect(onLogin).toHaveBeenCalledWith({
      fourWords: 'ocean-forest-moon-star',
      password: 'secure-password'
    });
  });
});
```

#### Example Service Test

```typescript
// src/services/NetworkService.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { NetworkService } from './NetworkService';
import { invoke } from '@tauri-apps/api/tauri';

vi.mock('@tauri-apps/api/tauri');

describe('NetworkService', () => {
  let service: NetworkService;

  beforeEach(() => {
    service = new NetworkService();
    vi.clearAllMocks();
  });

  it('connects to network successfully', async () => {
    vi.mocked(invoke).mockResolvedValue({ status: 'connected', peers: 42 });

    await service.connect();

    expect(invoke).toHaveBeenCalledWith('connect_network');
    expect(service.getStatus()).toBe('connected');
  });

  it('handles connection failures gracefully', async () => {
    vi.mocked(invoke).mockRejectedValue(new Error('Network unavailable'));

    await service.connect();

    expect(service.getStatus()).toBe('local');
    expect(service.hasError()).toBe(false); // Falls back to local mode
  });

  it('retries connection on failure', async () => {
    vi.mocked(invoke)
      .mockRejectedValueOnce(new Error('Timeout'))
      .mockResolvedValueOnce({ status: 'connected' });

    await service.connectWithRetry({ maxRetries: 3, delay: 100 });

    expect(invoke).toHaveBeenCalledTimes(2);
    expect(service.getStatus()).toBe('connected');
  });
});
```

### Integration Tests

```typescript
// src/integration/auth-flow.test.ts
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { App } from '../App';

describe('Authentication Flow', () => {
  it('completes full login flow', async () => {
    render(<App />);

    // 1. Should show login screen
    expect(screen.getByText('Welcome to Communitas')).toBeInTheDocument();

    // 2. Fill in credentials
    fireEvent.change(screen.getByLabelText('Four-Word Address'), {
      target: { value: 'ocean-forest-moon-star' }
    });
    fireEvent.change(screen.getByLabelText('Password'), {
      target: { value: 'test-password' }
    });

    // 3. Submit form
    fireEvent.click(screen.getByRole('button', { name: 'Login' }));

    // 4. Should show dashboard
    await waitFor(() => {
      expect(screen.getByText('Dashboard')).toBeInTheDocument();
    }, { timeout: 5000 });

    // 5. Should display user identity
    expect(screen.getByText('ocean-forest-moon-star')).toBeInTheDocument();
  });
});
```

### Running Frontend Tests

```bash
# Run all tests
npm test

# Watch mode (re-run on changes)
npm run test:watch

# With coverage
npm run test:coverage

# UI mode (interactive)
npm run test:ui

# Specific file
npm test -- AuthDialog.test.tsx
```

## Backend Testing (Rust)

### Unit Tests

#### Test File Structure

```
communitas-core/
├── src/
│   ├── identity.rs
│   └── storage/
│       └── mod.rs
└── tests/
    ├── identity_tests.rs        # Integration tests
    └── storage_tests.rs
```

#### Example Module Test

```rust
// communitas-core/src/identity.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_four_words() {
        let words = generate_four_words().unwrap();

        assert_eq!(words.split('-').count(), 4);
        assert!(words.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
    }

    #[test]
    fn test_validate_four_words() {
        assert!(validate_four_words("ocean-forest-moon-star").is_ok());
        assert!(validate_four_words("invalid-words-here-now").is_err());
        assert!(validate_four_words("only-three-words").is_err());
    }

    #[test]
    fn test_four_words_deterministic() {
        let seed = [0u8; 32];
        let words1 = generate_from_seed(&seed).unwrap();
        let words2 = generate_from_seed(&seed).unwrap();

        assert_eq!(words1, words2);
    }

    #[tokio::test]
    async fn test_identity_creation() {
        let identity = create_identity(
            "ocean-forest-moon-star",
            "Alice",
            "Test Device"
        ).await.unwrap();

        assert_eq!(identity.four_words, "ocean-forest-moon-star");
        assert_eq!(identity.display_name, "Alice");
        assert!(identity.public_key.len() > 0);
    }
}
```

#### Example Async Test

```rust
#[cfg(test)]
mod async_tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_message_sync() {
        let ctx = CoreContext::new_in_memory().await.unwrap();

        // Send message
        let msg_id = ctx.send_message(
            "channel-123",
            "Hello, World!",
            vec![]
        ).await.unwrap();

        // Retrieve message
        let msg = ctx.get_message(&msg_id).await.unwrap();
        assert_eq!(msg.content, "Hello, World!");
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let ctx = Arc::new(CoreContext::new_in_memory().await.unwrap());

        // Spawn multiple tasks
        let handles: Vec<_> = (0..10).map(|i| {
            let ctx = ctx.clone();
            tokio::spawn(async move {
                ctx.send_message(
                    "channel-123",
                    &format!("Message {}", i),
                    vec![]
                ).await
            })
        }).collect();

        // Wait for all
        for handle in handles {
            handle.await.unwrap().unwrap();
        }

        // Verify all messages received
        let messages = ctx.get_channel_messages("channel-123").await.unwrap();
        assert_eq!(messages.len(), 10);
    }
}
```

### Integration Tests

```rust
// communitas-core/tests/integration_test.rs
use communitas_core::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_two_node_messaging() {
    // Setup two nodes
    let alice = CoreContext::new("./test-data-alice").await.unwrap();
    let bob = CoreContext::new("./test-data-bob").await.unwrap();

    // Initialize identities
    alice.initialize_identity(
        "ocean-forest-moon-star",
        "Alice",
        "Node1"
    ).await.unwrap();

    bob.initialize_identity(
        "valley-river-cloud-wind",
        "Bob",
        "Node2"
    ).await.unwrap();

    // Connect to network
    alice.connect_network().await.unwrap();
    bob.connect_network().await.unwrap();

    // Wait for peer discovery
    sleep(Duration::from_secs(5)).await;

    // Alice sends message to Bob
    let msg_id = alice.send_direct_message(
        "valley-river-cloud-wind",
        "Hello Bob!"
    ).await.unwrap();

    // Wait for delivery
    sleep(Duration::from_secs(2)).await;

    // Bob should receive message
    let messages = bob.get_direct_messages().await.unwrap();
    assert!(messages.iter().any(|m| m.id == msg_id));
    assert_eq!(messages[0].content, "Hello Bob!");
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_four_words_always_valid(seed in any::<[u8; 32]>()) {
        let words = generate_from_seed(&seed).unwrap();
        assert!(validate_four_words(&words).is_ok());
    }

    #[test]
    fn test_encryption_roundtrip(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        let key = generate_encryption_key();
        let encrypted = encrypt(&data, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        assert_eq!(data, decrypted);
    }
}
```

### Running Backend Tests

```bash
# All tests
cargo test

# With logging
RUST_LOG=debug cargo test -- --nocapture

# Specific package
cargo test -p communitas-core

# Specific test
cargo test test_four_word_validation

# Integration tests only
cargo test --test '*'

# Exclude slow tests
cargo test -- --skip slow_test

# Run in release mode (faster)
cargo test --release
```

## Browser Testing with communitas-bridge

### Setup Bridge Server

```bash
# Terminal 1: Start bridge
cargo run -p communitas-bridge

# Terminal 2: Start frontend
npm run dev
```

Bridge provides HTTP/REST endpoints at `http://localhost:3030`

### Manual Testing

```bash
# Initialize core
curl -X POST http://localhost:3030/api/core/initialize \
  -H "Content-Type: application/json" \
  -d '{
    "four_words": "ocean-forest-moon-star",
    "display_name": "Test User",
    "device_name": "Browser Test"
  }'

# Create channel
curl -X POST http://localhost:3030/api/channels \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-channel",
    "description": "Test Channel"
  }'

# Send message
curl -X POST http://localhost:3030/api/channels/{channel_id}/messages \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello from curl!",
    "recipients": []
  }'
```

### Chrome DevTools MCP Testing

See [communitas-bridge/README.md](../../communitas-bridge/README.md) for complete testing guide with Chrome DevTools MCP integration.

#### Example Test Scenario

```javascript
// Navigate to test page
await mcp.navigate('http://localhost:3030/test.html');

// Initialize
const initResp = await fetch('http://localhost:3030/api/core/initialize', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    four_words: 'ocean-forest-moon-star',
    display_name: 'Test User',
    device_name: 'Browser Test'
  })
});

// Verify response
console.assert(initResp.ok, 'Initialization failed');

// Create channel and verify
const channelResp = await fetch('http://localhost:3030/api/channels', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    name: 'test-channel',
    description: 'Test Channel'
  })
});

const channel = await channelResp.json();
console.assert(channel.name === 'test-channel', 'Channel creation failed');
```

## End-to-End Testing

### Playwright Tests

```typescript
// tests/e2e/auth-flow.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Authentication Flow', () => {
  test('user can register and login', async ({ page }) => {
    // Navigate to app
    await page.goto('http://localhost:1420');

    // Click register
    await page.click('text=Register');

    // Fill form
    await page.fill('[name="display-name"]', 'Test User');
    await page.fill('[name="device-name"]', 'Test Device');
    await page.fill('[name="password"]', 'SecurePassword123!');
    await page.fill('[name="confirm-password"]', 'SecurePassword123!');

    // Note four-word address
    const fourWords = await page.textContent('[data-testid="four-words"]');

    // Submit
    await page.click('text=Create Identity');

    // Should be logged in
    await expect(page.locator('text=Dashboard')).toBeVisible();

    // Logout
    await page.click('[data-testid="user-menu"]');
    await page.click('text=Logout');

    // Login again
    await page.fill('[name="four-words"]', fourWords);
    await page.fill('[name="password"]', 'SecurePassword123!');
    await page.click('text=Login');

    // Should be back in
    await expect(page.locator('text=Dashboard')).toBeVisible();
  });
});
```

### Running E2E Tests

```bash
# Install Playwright
npx playwright install

# Run tests
npm run test:e2e

# Run in headed mode (see browser)
npm run test:e2e -- --headed

# Run specific test
npm run test:e2e -- auth-flow.spec.ts

# Debug mode
npm run test:e2e -- --debug
```

## Performance Testing

### Benchmarking with Criterion

```rust
// benches/four_words_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use communitas_core::identity::*;

fn bench_generation(c: &mut Criterion) {
    c.bench_function("generate_four_words", |b| {
        b.iter(|| {
            generate_four_words().unwrap()
        });
    });
}

fn bench_validation(c: &mut Criterion) {
    c.bench_function("validate_four_words", |b| {
        b.iter(|| {
            validate_four_words(black_box("ocean-forest-moon-star")).unwrap()
        });
    });
}

criterion_group!(benches, bench_generation, bench_validation);
criterion_main!(benches);
```

### Running Benchmarks

```bash
cargo bench

# Specific benchmark
cargo bench bench_generation

# Save baseline
cargo bench -- --save-baseline main

# Compare to baseline
cargo bench -- --baseline main
```

## Test Data Management

### Test Fixtures

```rust
// tests/common/mod.rs
pub struct TestFixtures {
    pub temp_dir: TempDir,
    pub alice_identity: Identity,
    pub bob_identity: Identity,
}

impl TestFixtures {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;

        let alice = create_test_identity(
            "ocean-forest-moon-star",
            "Alice",
            "Device1"
        ).await?;

        let bob = create_test_identity(
            "valley-river-cloud-wind",
            "Bob",
            "Device2"
        ).await?;

        Ok(Self {
            temp_dir,
            alice_identity: alice,
            bob_identity: bob,
        })
    }
}

// Use in tests
#[tokio::test]
async fn test_with_fixtures() {
    let fixtures = TestFixtures::new().await.unwrap();
    // Test using fixtures.alice_identity, etc.
}
```

### Test Database

```rust
// Use in-memory SQLite for tests
pub async fn test_database() -> Result<Database> {
    Database::new(":memory:").await
}

// Or temporary file that auto-cleans
pub async fn temp_database() -> Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;
    Ok((db, temp_dir))
}
```

## Continuous Integration

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm install

      - name: Run frontend tests
        run: npm test

      - name: Run backend tests
        run: cargo test --all

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Test Coverage

### Frontend Coverage

```bash
# Generate coverage
npm run test:coverage

# View report
open coverage/index.html
```

### Backend Coverage (with cargo-tarpaulin)

```bash
# Install
cargo install cargo-tarpaulin

# Run
cargo tarpaulin --out Html

# View report
open tarpaulin-report.html
```

## Troubleshooting

### Tests Failing

```bash
# Clean and rebuild
cargo clean
npm run clean
npm install
cargo build

# Run with logging
RUST_LOG=debug cargo test -- --nocapture

# Run single test to isolate
cargo test test_name -- --exact
```

### Network Tests Timeout

```bash
# Increase timeout
cargo test -- --test-threads=1 --nocapture

# Skip network tests
cargo test -- --skip network_test
```

### Database Locked

```bash
# Ensure tests run sequentially
cargo test -- --test-threads=1

# Or use in-memory databases
# (already configured in test fixtures)
```

## Best Practices

1. **Test Isolation**: Each test should be independent
2. **Clean Up**: Use `TempDir` for file-based tests
3. **Mock External Services**: Don't rely on network
4. **Fast Tests**: Unit tests should run in <1s
5. **Clear Names**: Test names should describe what they test
6. **Arrange-Act-Assert**: Structure tests clearly
7. **Test Edge Cases**: Empty strings, max values, invalid input
8. **Document Complex Tests**: Explain why, not just what

## See Also

- [Getting Started](getting-started.md) - Setup and basics
- [communitas-bridge README](../../communitas-bridge/README.md) - Browser testing
- [communitas-tui README](../../communitas-tui/README.md) - TUI testing
- [API Documentation](../api/) - API reference

---

**Test early, test often, test everything! ✅**

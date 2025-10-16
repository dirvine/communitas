# Communitas E2E Testing

This directory contains comprehensive end-to-end tests for the Communitas application using Playwright.

## Test Structure

```
tests/
├── e2e/                    # End-to-end tests
│   ├── web-mode/          # Tests for web/browser mode
│   │   ├── onboarding.spec.ts     # User onboarding flow
│   │   └── dashboard.spec.ts      # Main app functionality
│   └── tauri-mode/        # Tests for native Tauri app
│       └── app-lifecycle.spec.ts  # Native app lifecycle
├── fixtures/              # Test fixtures and utilities
│   └── auth.fixture.ts    # Authentication test helpers
└── utils/                 # Test utilities
    └── tauri-setup.ts     # Tauri app setup for testing
```

## Test Modes

### Web Mode Testing
Tests the React application running in browsers (Chrome, Firefox, Safari). This is the default mode for development testing.

```bash
# Run web mode tests
npm run test:e2e:web

# Run with UI
npm run test:e2e:web:ui

# Run specific browser
npm run test:e2e:web:chrome
```

### Tauri Native Mode Testing
Tests the actual packaged Tauri application with native OS integration.

```bash
# Run Tauri native tests (requires built app)
TAURI_MODE=1 npm run test:e2e:tauri

# Run with UI
TAURI_MODE=1 npm run test:e2e:tauri:ui
```

## Prerequisites

### For Web Mode Testing
- Node.js and npm installed
- Playwright browsers installed: `npm run playwright:install`

### For Tauri Native Mode Testing
- Tauri app built: `npm run tauri:build`
- Appropriate system dependencies for your platform

## Environment Variables

- `TAURI_MODE=1`: Enable Tauri native testing mode
- `CI=1`: Run in CI mode (affects timeouts and parallelization)
- `KEEP_TEST_DATA=1`: Preserve test data directories after Tauri tests

## Test Fixtures

### Authentication Fixture (`auth.fixture.ts`)
Provides pre-configured test users and authentication helpers:

```typescript
import { test } from '../fixtures/auth.fixture';

test('authenticated user test', async ({ page, authenticatedUser }) => {
  // User is already authenticated
  expect(authenticatedUser.name).toBe('Alice Test');
});
```

### Available Test Users
- **Alice**: `alice-forest-moon-star` (authenticated)
- **Bob**: `bob-mountain-river-sun` (authenticated)
- **Charlie**: `charlie-ocean-cloud-peak` (unauthenticated)

## Writing Tests

### Web Mode Tests
```typescript
import { test, expect } from '@playwright/test';

test('my web test', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByText('Welcome')).toBeVisible();
});
```

### Tauri Native Mode Tests
```typescript
import { test, expect } from '@playwright/test';
import { _electron as electron } from 'playwright';

test('my tauri test', async () => {
  const electronApp = await electron.launch({ args: ['path/to/app'] });
  const page = await electronApp.firstWindow();

  await expect(page.getByText('Communitas')).toBeVisible();
});
```

### Using Authentication Fixtures
```typescript
import { test, expect } from '../fixtures/auth.fixture';

test('authenticated test', async ({ page, authenticatedUser, setupAuthUser }) => {
  await setupAuthUser(authenticatedUser);
  await page.goto('/');

  // User is authenticated and app is ready
  await expect(page.getByText(authenticatedUser.name)).toBeVisible();
});
```

## Running Tests

### Development Testing
```bash
# Run all web mode tests
npm run test:e2e:web

# Run specific test file
npx playwright test tests/e2e/web-mode/onboarding.spec.ts

# Run with debugging
npx playwright test --debug
```

### CI/CD Testing
```bash
# Run tests in CI mode
CI=1 npm run test:e2e:web

# Generate test report
npx playwright show-report
```

### Tauri Native Testing
```bash
# Build Tauri app first
npm run tauri:build

# Run Tauri tests
TAURI_MODE=1 npm run test:e2e:tauri
```

## Test Configuration

### Timeouts
- Action timeout: 10 seconds
- Navigation timeout: 30 seconds
- Test timeout: 60 seconds (Tauri mode)

### Browser Configuration
- Chromium: Desktop Chrome
- Firefox: Desktop Firefox
- WebKit: Desktop Safari

### Parallelization
- Web mode: Sequential (for stability)
- Tauri mode: Single worker (required)

## Debugging Tests

### Visual Debugging
```bash
# Run with Playwright UI
npm run test:e2e:web:ui

# Run with browser dev tools
npx playwright test --headed --debug
```

### Tracing and Screenshots
Tests automatically capture:
- Screenshots on failure
- Traces for debugging
- Videos for failed tests

Access results with:
```bash
npx playwright show-report
```

## Test Data Management

### Web Mode
- Uses localStorage/sessionStorage
- Automatically cleaned between tests
- No persistent data

### Tauri Native Mode
- Creates isolated data directories
- Automatically cleaned after tests (unless `KEEP_TEST_DATA=1`)
- Tests IPC communication with Rust backend

## Best Practices

1. **Use fixtures** for common setup (auth, test data)
2. **Test user journeys** end-to-end, not individual components
3. **Mock external dependencies** (network calls, file system)
4. **Use descriptive test names** that explain the user behavior
5. **Keep tests independent** - they should work in any order
6. **Test error scenarios** - network failures, invalid inputs
7. **Verify accessibility** - use semantic selectors and ARIA labels

## Troubleshooting

### Common Issues

**Tauri app not found**
```
Error: Tauri app not found at path
```
Solution: Run `npm run tauri:build` first

**WebRTC tests failing**
```
WebRTC not supported in WebKit
```
Solution: Skip WebRTC tests on WebKit or use conditional testing

**IPC calls failing**
```
Tauri API not available
```
Solution: Ensure tests are running in Tauri mode with `TAURI_MODE=1`

### Debug Commands
```bash
# Check Tauri app exists
ls -la src-tauri/target/release/bundle/

# Test IPC manually
npx tauri dev -- --help

# Check Playwright browsers
npx playwright install --dry-run
```

## Contributing

When adding new tests:

1. Follow the naming convention: `*.spec.ts`
2. Add tests to appropriate directories (`web-mode/` or `tauri-mode/`)
3. Use fixtures for common setup
4. Include both positive and negative test cases
5. Add documentation for complex test scenarios

## CI/CD Integration

The test suite is designed to run in CI environments:

- Uses `CI=1` for appropriate timeouts and retries
- Generates HTML reports for review
- Fails fast on critical errors
- Supports parallel execution where safe

Example GitHub Actions:
```yaml
- name: Run E2E Tests
  run: npm run test:e2e:web
  env:
    CI: 1
```

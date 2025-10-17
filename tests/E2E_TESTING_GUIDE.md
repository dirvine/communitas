# E2E Testing Guide for Communitas Tauri App

## Overview

E2E tests for the Communitas Tauri desktop app. Tests connect to the running Tauri dev server via Playwright.

## Important: Tauri is NOT Electron

Tauri apps run a web server in dev mode that we connect to with a browser. Unlike Electron, we don't launch the app directly - we connect to the running dev server at `http://localhost:5173`.

## Prerequisites

### 1. Install Dependencies

```bash
npm install
npm run playwright:install
```

### 2. Build Frontend

```bash
npm run build
```

## Running Tests

### Step 1: Start Tauri Dev Server

In **Terminal 1**, start the Tauri app:

```bash
npm run tauri dev
```

Wait for the server to start. You should see:
```
Local:   http://localhost:5173/
```

### Step 2: Run E2E Tests

In **Terminal 2**, run the tests:

```bash
# Run all Tauri E2E tests
npm run test:e2e:tauri

# Run with UI mode (recommended)
npm run test:e2e:tauri:ui

# Run in headed mode (see the browser)
npm run test:e2e:tauri:headed

# Run specific test file
npx playwright test tests/e2e/tauri-mode/01-onboarding.spec.ts
```

## Test Structure

```
tests/e2e/tauri-mode/
├── 01-onboarding.spec.ts    # Identity creation (4 tests)
├── 02-messaging.spec.ts     # Messaging flows (5 tests)
├── 03-files-storage.spec.ts # File operations (4 tests)
├── 04-webrtc-calls.spec.ts  # WebRTC calls (5 tests)
└── app-lifecycle.spec.ts    # Basic lifecycle (12 tests)

Total: 30 tests
```

## Test Coverage

### ✅ What's Tested

#### Onboarding (4 tests)
- O1: Welcome/identity creation screen appears
- O2: Can create new identity
- O3: Identity persists after reload
- O4: Passkey tests (intentionally skipped in dev mode)

#### Messaging (5 tests)
- M1: Can navigate to channel and see message input
- M2: Can type and send a message
- M3: Unread badge UI exists
- M4: Activity feed/sidebar is visible
- M5: New chat button exists

#### File Operations (4 tests)
- F1: File upload/attach button accessible
- F2: Storage section accessible
- F3: New document action exists
- F4: Can navigate to files area

#### WebRTC (5 tests)
- W1: WebRTC APIs available
- W2: Can enumerate media devices
- W3: Can request media stream (fake devices)
- W4: Call UI elements exist
- W5: Can clean up media streams

### ⏭️ Intentionally Skipped

- **Passkeys/Touch ID**: Requires production macOS build
- **Screen Share**: Requires TCC screen recording permission
- **Notifications**: Requires unfocused window state
- **Native File Picker**: Can't automate OS dialogs

## How Tests Work

### Tauri Dev Mode Architecture

```
┌─────────────────────────────────────┐
│   Terminal 1: npm run tauri dev    │
│                                     │
│   ┌──────────────────────────┐    │
│   │  Vite Dev Server         │    │
│   │  http://localhost:5173   │    │
│   └──────────────────────────┘    │
│              ↓                      │
│   ┌──────────────────────────┐    │
│   │  Tauri Window (WKWebView)│    │
│   │  window.__TAURI__ API    │    │
│   └──────────────────────────┘    │
└─────────────────────────────────────┘
                ↑
                │ Playwright connects here
                │
┌─────────────────────────────────────┐
│   Terminal 2: npm run test:e2e     │
│                                     │
│   ┌──────────────────────────┐    │
│   │  Playwright Browser      │    │
│   │  → localhost:5173        │    │
│   │  → Has __TAURI__ API     │    │
│   └──────────────────────────┘    │
└─────────────────────────────────────┘
```

### Test Flow

1. Start Tauri dev server (`npm run tauri dev`)
2. Playwright browser connects to `http://localhost:5173`
3. `window.__TAURI__` API is available in the browser
4. Tests interact with UI and invoke Tauri commands
5. Tests verify behavior and take screenshots

## Common Commands

```bash
# Start Tauri dev server (Terminal 1)
npm run tauri dev

# Run all E2E tests (Terminal 2)
npm run test:e2e:tauri

# Run specific suite
npx playwright test tests/e2e/tauri-mode/01-onboarding.spec.ts

# Run with UI (best for development)
npm run test:e2e:tauri:ui

# Run headed (see browser window)
HEADED=1 npm run test:e2e:tauri

# Debug mode
npm run test:e2e:tauri:debug

# Keep test data
KEEP_TEST_DATA=1 npm run test:e2e:tauri

# Verbose logging
DEBUG=pw:* npm run test:e2e:tauri
```

## Debugging Failed Tests

### 1. Check Playwright Report

```bash
npm run test:e2e:report
```

### 2. View Screenshots

Screenshots are saved with timestamps. Check the test output for paths.

### 3. Run in Headed Mode

```bash
HEADED=1 npm run test:e2e:tauri
```

Watch the browser to see what's happening.

### 4. Use Playwright Inspector

```bash
npm run test:e2e:tauri:debug
```

Step through test execution line by line.

### 5. Check Tauri Logs

In Terminal 1 (where `tauri dev` is running), check Rust logs for errors.

## Common Issues

### Issue: "Cannot connect to http://localhost:5173"

**Cause**: Tauri dev server not running

**Solution**: Start `npm run tauri dev` in Terminal 1 first

### Issue: "Timeout waiting for __TAURI__"

**Cause**: Frontend not built or Tauri not loading properly

**Solution**:
```bash
npm run build
npm run tauri dev
```

### Issue: "getUserMedia failed"

**Cause**: Fake devices not configured

**Solution**: Tests run with `--use-fake-device-for-media-stream` by default. Check browser console.

### Issue: "Tests are flaky"

**Cause**: Timing issues or varying app state

**Solution**:
- Increase wait times
- Check screenshots to see actual UI state
- Ensure Tauri dev server is fully started
- Clear browser cache between runs

## Test Utilities

### TauriTestHelper

```typescript
import { TauriTestHelper } from '../../utils/tauri-helpers';

test.beforeEach(async ({ page }) => {
  helper = new TauriTestHelper({ cleanup: false });
  
  // Connect to Tauri dev server
  await page.goto('http://localhost:5173');
  
  // Wait for Tauri API to be ready
  await helper.waitForTauriReady(page);
  
  // Wait for app to load
  await page.waitForLoadState('networkidle');
});
```

### Invoke Tauri Commands

```typescript
// Call backend command
const result = await helper.invokeCommand(page, 'health');

// Create identity
await helper.invokeCommand(page, 'core_claim', {
  words: ['Test', 'User', 'Demo', 'Sample']
});

// Initialize core
await helper.invokeCommand(page, 'core_initialize', {
  data_dir: helper.getDataDir()
});
```

### Screenshots

```typescript
// Take screenshot for debugging
await helper.screenshot(page, 'my-test-step');

// Full page screenshot
await helper.screenshot(page, 'full-page', true);
```

## Best Practices

### 1. Always Wait for Tauri Ready

```typescript
await page.goto('http://localhost:5173');
await helper.waitForTauriReady(page); // Essential!
```

### 2. Use Graceful Element Checks

```typescript
// ✅ Good: Check if visible first
const buttonVisible = await button.isVisible().catch(() => false);
if (buttonVisible) {
  await button.click();
}

// ❌ Bad: Assume element exists
await button.click(); // May fail if UI varies
```

### 3. Add Waits for UI Updates

```typescript
await button.click();
await page.waitForTimeout(1000); // Let UI update
```

### 4. Take Screenshots on Key Steps

```typescript
await helper.screenshot(page, 'before-action');
await someAction();
await helper.screenshot(page, 'after-action');
```

### 5. Don't Test What Can't Be Automated

```typescript
// Native file picker - skip in tests
test('File picker', async () => {
  test.skip(true, 'Native OS dialog cannot be automated');
});
```

## Playwright Configuration

Tests use these projects (defined in `playwright.config.ts`):

- `tauri-native`: Chromium with fake media devices, connects to Tauri dev server
- `web-chromium`: Regular web tests
- `web-firefox`: Firefox web tests
- `web-webkit`: Safari web tests

To run only Tauri tests:

```bash
npx playwright test --project=tauri-native
```

## CI/CD Integration (Future)

```yaml
name: E2E Tests
jobs:
  e2e:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node
        uses: actions/setup-node@v4
      - name: Install deps
        run: npm ci
      - name: Build frontend
        run: npm run build
      - name: Start Tauri dev server
        run: npm run tauri dev &
      - name: Wait for server
        run: npx wait-on http://localhost:5173
      - name: Run E2E tests
        run: npm run test:e2e:tauri
      - name: Upload results
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: playwright-report/
```

## Next Steps

### Priority 1
- [ ] Add more interaction tests (click through full flows)
- [ ] Add multi-user sync tests (requires second instance)
- [ ] Add offline/online transition tests
- [ ] Add search/keyboard shortcut tests

### Priority 2
- [ ] Performance benchmarks (cold start, message rendering)
- [ ] Accessibility tests (keyboard navigation, screen readers)
- [ ] Component-level tests with Testing Library
- [ ] Visual regression tests

### Priority 3
- [ ] Production build tests (with passkeys enabled)
- [ ] Code signing verification
- [ ] Auto-update flow testing
- [ ] Deep link handling

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Tauri Documentation](https://tauri.app/)
- [Production Readiness Report](../PRODUCTION_READINESS_REPORT.md)
- [Test Implementation Summary](../TEST_IMPLEMENTATION_SUMMARY.md)

## Quick Reference

```bash
# Terminal 1: Start Tauri
npm run tauri dev

# Terminal 2: Run tests
npm run test:e2e:tauri

# Debug a test
npx playwright test tests/e2e/tauri-mode/01-onboarding.spec.ts --debug

# Show test report
npm run test:e2e:report

# List all tests
npx playwright test --list tests/e2e/tauri-mode
```

---

**Remember**: Start `npm run tauri dev` first, then run tests in a separate terminal!

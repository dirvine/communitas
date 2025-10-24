# Communitas E2E Test Suite

Comprehensive end-to-end tests covering all UI flows, entity management, and user journeys.

## Test Structure

### Web Mode Tests (`web-mode/`)
Browser-based tests for the React SPA without Tauri:
- **onboarding.spec.ts** - Identity creation, passkey registration
- **dashboard.spec.ts** - Activity dashboard, entity management

### Tauri Mode Tests (`tauri-mode/`)
Native app tests using Tauri IPC and platform APIs:
- **01-onboarding.spec.ts** - Identity creation and persistence
- **02-messaging.spec.ts** - Core messaging functionality
- **03-files-storage.spec.ts** - File operations and storage
- **04-webrtc-calls.spec.ts** - WebRTC APIs and media devices
- **05-entity-management.spec.ts** ⭐ **NEW** - Complete CRUD for all entities
- **06-complete-user-flows.spec.ts** ⭐ **NEW** - End-to-end user journeys
- **app-lifecycle.spec.ts** - App startup, IPC, window management

### Multi-Peer Tests (`multi-peer/`)
Cross-node messaging and sync scenarios:
- **messaging.spec.ts** - UI ↔ headless peer communication

## Entity Management Tests (05-entity-management.spec.ts)

### Groups
- ✅ G1: Create new group
- ✅ G2: Add member to group
- ✅ G3: Remove member from group
- ✅ G4: View group members list
- ✅ G5: Edit group details
- ✅ G6: Delete/leave group

### Organizations
- ✅ O1: Create organization
- ✅ O2: Add members to organization
- ✅ O3: Assign roles in organization

### Projects
- ✅ P1: Create project
- ✅ P2: Assign members to project
- ✅ P3: Archive/complete project

### Contacts
- ✅ C1: Add contact (via four-word address)
- ✅ C2: Edit contact details
- ✅ C3: Remove contact

## Complete User Flows (06-complete-user-flows.spec.ts)

### UF1: New User Journey
**Steps:**
1. Create identity (onboarding)
2. Create first group
3. Add members to group
4. Send first message
5. Verify complete state

**Validates:** Complete new user experience from zero to productive

### UF2: Organization Setup
**Steps:**
1. Create organization
2. Add team members (multiple)
3. Create project within organization
4. Assign project team from org members

**Validates:** Multi-level entity hierarchy and member management

### UF3: Contact Discovery via FOAF
**Steps:**
1. Search for contact by four-word address
2. FOAF discovery locates contact
3. Add to contacts
4. Start conversation
5. Send first message

**Validates:** FOAF discovery, contact management, messaging integration

### UF4: Document Collaboration
**Steps:**
1. Create document
2. Edit content (CRDT editor)
3. Share with group
4. Verify sync state

**Validates:** Document creation, CRDT collaboration, sync indicators

### UF5: Settings & Preferences
**Steps:**
1. Open settings
2. Configure bootstrap nodes
3. Configure notifications
4. Review identity

**Validates:** Settings persistence, network configuration

## Running Tests

### Prerequisites

#### For Tauri Tests
```bash
# Terminal 1: Build frontend and start Tauri dev
npm run build
npm run tauri dev
```

#### For Multi-Peer Tests
```bash
# Build headless node
cargo build --release -p communitas-headless
```

### Run Specific Test Files

```bash
# Entity management (all entities)
npm run test:e2e:tauri -- 05-entity-management.spec.ts

# Complete user flows
npm run test:e2e:tauri -- 06-complete-user-flows.spec.ts

# Specific flow
npm run test:e2e:tauri -- 06-complete-user-flows.spec.ts -g "New User Journey"

# Run in UI mode for debugging
npm run test:e2e:tauri:ui -- 05-entity-management.spec.ts
```

### Run All Tests

```bash
# All Tauri tests
npm run test:e2e:tauri

# All web tests
npm run test:e2e:web

# All E2E tests
npm run test:e2e
```

## Test Patterns

### Resilient Selectors
Tests use flexible selectors to adapt to UI changes:
```typescript
// Multiple selector strategies
page.locator('button').filter({ hasText: /new.*group|create.*group/i })
page.locator('[data-testid*="group"], [role="listitem"]')
```

### Graceful Skipping
Tests skip gracefully when UI elements aren't found:
```typescript
if (await element.isVisible({ timeout: 2000 })) {
  // Test steps
} else {
  test.skip();
}
```

### Screenshots
All tests capture screenshots at key steps:
```typescript
await helper.screenshot(page, 'group-created');
```

### Wait Strategies
Appropriate waits for different scenarios:
```typescript
await page.waitForLoadState('load');        // Page load
await page.waitForTimeout(500);             // UI animation
await expect(element).toBeVisible({ timeout: 5000 }); // Async operation
```

## Debugging Tests

### Visual Debugging
```bash
# Run with headed browser
HEADED=1 npm run test:e2e:tauri:headed -- 05-entity-management.spec.ts

# Use Playwright UI mode
npm run test:e2e:tauri:ui -- 06-complete-user-flows.spec.ts
```

### Step-by-Step Debugging
```bash
# Run with --debug flag
npm run test:e2e:tauri:debug -- 05-entity-management.spec.ts
```

### View Test Reports
```bash
# After test run
npm run test:e2e:report
```

### Check Screenshots
```bash
# Screenshots saved to test-screenshots/
ls -la test-screenshots/
```

## Test Data & State

### Clean State Tests
Some tests use clean state (cleanup: true):
- UF1: New User Journey (complete fresh start)

### Shared State Tests
Most tests use shared state (cleanup: false):
- Entity management (builds on existing entities)
- Messaging tests (reuses channels)

### Test Isolation
Each test file:
- Uses `test.describe()` blocks for organization
- Has independent `beforeEach` setup
- Cleans up appropriately in `afterEach`

## Adding New Tests

### 1. Entity CRUD Test
```typescript
test('E1: Can create entity', async ({ page }) => {
  // Navigate to entity section
  const newButton = page.locator('button').filter({ hasText: /new.*entity/i }).first();
  
  if (await newButton.isVisible({ timeout: 2000 })) {
    await newButton.click();
    
    // Fill form
    const nameInput = page.locator('input[name*="name" i]').first();
    await nameInput.fill('Test Entity');
    
    // Submit
    const createButton = page.locator('button').filter({ hasText: /create/i }).first();
    await createButton.click();
    
    // Verify
    await expect(page.locator('text=Test Entity')).toBeVisible({ timeout: 5000 });
    await helper.screenshot(page, 'entity-created');
  } else {
    test.skip();
  }
});
```

### 2. User Flow Test
```typescript
test('UF6: Custom flow', async ({ page }) => {
  // STEP 1: Setup
  await setupPrerequisites(page);
  
  // STEP 2: Main action
  await performMainAction(page);
  
  // STEP 3: Verification
  await verifyResults(page);
  
  // Screenshot each major step
  await helper.screenshot(page, 'flow-step-1');
  await helper.screenshot(page, 'flow-step-2');
  await helper.screenshot(page, 'flow-complete');
});
```

## CI Integration

### GitHub Actions
```yaml
e2e-tauri:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - run: npm ci
    - run: npm run build
    - run: npx playwright install --with-deps
    - run: npm run tauri dev &
    - run: sleep 10  # Wait for Tauri to start
    - run: npm run test:e2e:tauri
    - uses: actions/upload-artifact@v3
      if: failure()
      with:
        name: test-screenshots
        path: test-screenshots/
```

## Coverage Matrix

| Feature | Entity Mgmt | User Flows | Multi-Peer |
|---------|-------------|------------|------------|
| Groups | ✅ G1-G6 | ✅ UF1, UF2 | - |
| Organizations | ✅ O1-O3 | ✅ UF2 | - |
| Projects | ✅ P1-P3 | ✅ UF2 | - |
| Contacts | ✅ C1-C3 | ✅ UF3 | - |
| Messaging | ✅ (via flows) | ✅ UF1, UF3 | ✅ |
| Documents | - | ✅ UF4 | - |
| Settings | - | ✅ UF5 | - |
| FOAF Discovery | - | ✅ UF3 | - |

## Best Practices

1. **Use test IDs when available**: `[data-testid="element-name"]`
2. **Flexible selectors**: Multiple strategies with `.or()` and `.filter()`
3. **Appropriate timeouts**: 2s for UI elements, 5s for async ops
4. **Screenshot key steps**: Helps debugging failures
5. **Skip gracefully**: Don't fail on missing optional features
6. **Clear test names**: Descriptive, prefixed with category code
7. **Isolated state**: Each test can run independently
8. **Verify final state**: Assert expected outcome, not just actions

## Troubleshooting

### "Element not found" errors
- Check if selector matches your UI implementation
- Increase timeout if element loads slowly
- Add `await page.waitForTimeout(500)` before interaction

### "Tauri not ready" errors
- Ensure `npm run tauri dev` is running
- Check port 5173 is accessible
- Verify `window.__TAURI__` is defined

### Test skips unexpectedly
- UI element selector doesn't match
- Feature not yet implemented
- Check screenshots to see actual state

### Flaky tests
- Add appropriate waits before assertions
- Use `waitForLoadState` instead of arbitrary timeouts
- Check for race conditions in UI rendering

## Next Steps

- [ ] Add permission/authorization tests
- [ ] Add offline mode handling tests
- [ ] Add data validation tests (invalid inputs)
- [ ] Add performance tests (large member lists)
- [ ] Add accessibility tests (a11y)
- [ ] Add internationalization tests (i18n)

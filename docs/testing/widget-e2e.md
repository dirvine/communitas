# Widget E2E Testing Guide

## Overview

Communitas MCP widgets are tested using Playwright for end-to-end (E2E) testing. This ensures widgets work correctly in real browser environments with the MCP bridge.

## Test Infrastructure

### Location
- **Test files**: `communitas-mcp/ui-bundles/e2e/`
- **Configuration**: `communitas-mcp/playwright.config.js`
- **Utilities**: `communitas-mcp/ui-bundles/e2e/utils/`

### Browsers Tested
- **Chromium** (Chrome, Edge)
- **Firefox**
- **WebKit** (Safari)

## Running Tests

### Local Development

```bash
# Install dependencies (first time only)
cd communitas-mcp
npm install

# Run tests in headless mode
npm run test:e2e

# Run tests with UI (interactive)
npm run test:e2e:ui

# Run tests in headed mode (see browser)
npm run test:e2e:headed

# Debug a specific test
npm run test:e2e:debug
```

### CI/CD

E2E tests run automatically on:
- Push to `main` branch
- Pull requests
- When widget files change

See: `.github/workflows/widget-e2e.yml`

## Widget Coverage

| Widget | Test File | Test Count | Status |
|--------|-----------|------------|--------|
| Contacts | `contacts.spec.js` | 8 | ✅ |
| Messages | `messages.spec.js` | 8 | ✅ |
| Kanban | `kanban.spec.js` | 8 | ✅ |
| Drive | `drive.spec.js` | 8 | ✅ |
| Canvas | `canvas.spec.js` | 8 | ✅ |
| Settings | `settings.spec.js` | 8 | ✅ |
| Search | `search.spec.js` | 4 | ✅ |
| Notifications | `notifications.spec.js` | 4 | ✅ |

**Total**: 64 test cases across 8 widgets (100% coverage)

## Test Utilities

### MCP Mock Server

Located in `ui-bundles/e2e/utils/mcp-mock.js`:

```javascript
const { McpMock } = require('./utils/mcp-mock');

const mock = new McpMock();

// Register canned response
mock.registerResponse('list_contacts', {
  success: true,
  contacts: [...]
});

// Register dynamic handler
mock.registerTool('send_message', async (params) => {
  return { success: true, message_id: 'msg-123' };
});
```

### Widget Helpers

Located in `ui-bundles/e2e/utils/widget-helpers.js`:

```javascript
const { loadWidget, waitForWidgetReady } = require('./utils/widget-helpers');

// Load widget
await loadWidget(page, 'contacts');

// Wait for ready state
await waitForWidgetReady(page);

// Check error state
const hasError = await hasErrorState(page);
```

## Writing Tests

### Test Structure

```javascript
const { test, expect } = require('@playwright/test');

test.describe('Widget Name', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/widgets/widget-name/index.html');
  });

  test('should do something', async ({ page }) => {
    // Arrange
    await page.waitForSelector('.widget-container');

    // Act
    await page.click('.some-button');

    // Assert
    await expect(page.locator('.result')).toBeVisible();
  });
});
```

### Best Practices

1. **Wait for elements**: Always use `waitForSelector` before interacting
2. **Defensive checks**: Check element counts before assuming presence
3. **Accessibility**: Test ARIA labels and keyboard navigation
4. **Error states**: Verify error handling and recovery
5. **State persistence**: Test that widget state persists correctly

## Coverage Report

Generate coverage report:

```bash
cd communitas-mcp/ui-bundles/e2e
node coverage-report.js
```

Output:
```
═══════════════════════════════════════════════
   Widget E2E Test Coverage Report
═══════════════════════════════════════════════

Widget                Tests      Status
───────────────────────────────────────────────
Contacts                  8      ✅
Messages                  8      ✅
...
═══════════════════════════════════════════════
Total Widgets: 8
Widgets Covered: 8/8 (100%)
Total Test Cases: 64
═══════════════════════════════════════════════
```

## Debugging Failed Tests

### Screenshots and Videos

Playwright automatically captures:
- **Screenshots** on failure
- **Videos** when tests fail

Artifacts location: `communitas-mcp/test-results/`

### Interactive Debugging

```bash
# Run specific test with UI
npx playwright test contacts.spec.js --ui

# Debug mode (step through)
npx playwright test contacts.spec.js --debug

# Show test report
npx playwright show-report
```

### Common Issues

**Widget doesn't load:**
- Ensure MCP server is running (`cargo run -p communitas-mcp --http --demo`)
- Check browser console for CSP violations

**Timeouts:**
- Increase timeout in `playwright.config.js`
- Use more specific selectors

**Flaky tests:**
- Add proper wait conditions
- Check for race conditions

## Integration with Rust Tests

E2E tests run after Rust tests pass:

```bash
# Full test suite
cargo test --all-features
npm run test:e2e
```

CI ensures:
1. Rust tests pass first
2. MCP server builds successfully
3. E2E tests run against built server
4. Artifacts uploaded on failure

## Maintenance

### Adding New Widgets

1. Create test file: `ui-bundles/e2e/widget-name.spec.js`
2. Follow existing patterns (see `contacts.spec.js`)
3. Add at least 8 test cases
4. Update `coverage-report.js` widget list
5. Run `npm run test:e2e` to verify

### Updating Tests

When widgets change:
1. Update corresponding `.spec.js` file
2. Run tests locally to verify
3. Update documentation if needed
4. Commit changes

## Resources

- [Playwright Documentation](https://playwright.dev)
- [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- [MCP Protocol Spec](https://modelcontextprotocol.io)
- [Widget Architecture](../architecture/widgets.md)

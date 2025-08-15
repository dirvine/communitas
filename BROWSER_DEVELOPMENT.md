# Browser Development Mode for Communitas

This guide explains how to run Communitas in browser mode for UX testing and development using Playwright MCP.

## Quick Start

### 1. Install Dependencies
```bash
npm install
npx playwright install  # Install Playwright browsers
```

### 2. Run in Browser Mode
```bash
# Start the app in browser mode with mock APIs
npm run dev:browser
```

The app will be available at http://localhost:1420 with full UI functionality using mock Tauri APIs.

### 3. Use Playwright MCP for UX Testing

With the app running in browser mode, you can use Playwright MCP tools to:
- Navigate through the UI
- Take screenshots
- Test interactions
- Validate UX flows

Example Playwright MCP commands:
```
browser_navigate url="http://localhost:1420"
browser_snapshot
browser_click element="Organization tab" ref="[role='tab']:has-text('Organization')"
```

## Features in Browser Mode

When running in browser mode, the app provides:

1. **Mock Network Data**: Simulated P2P network health and peer connections
2. **Mock Organizations**: Demo organization with members and settings
3. **Mock Messages**: Sample encrypted messages for testing chat UI
4. **Mock Identity**: Test identity with passkey simulation
5. **Mock DHT Stats**: Simulated distributed hash table statistics

## Running Playwright Tests

### Run All Tests
```bash
npm run test:playwright
```

### Run Tests with UI Mode
```bash
npm run test:playwright:ui
```

### Run Specific Test
```bash
npx playwright test tests/ux.spec.ts
```

### Generate Test Report
```bash
npx playwright show-report
```

## Development Workflow

1. **Start Browser Mode**: `npm run dev:browser`
2. **Make UI Changes**: Edit React components
3. **Test with Playwright MCP**: Use browser tools to interact
4. **Run Automated Tests**: `npm run test:playwright`
5. **Iterate**: Refine UX based on testing

## Mock API Customization

The mock Tauri API is located in `src/utils/mockTauriApi.ts`. You can customize:

- Network health simulation
- Organization data
- Message history
- Identity information
- Peer connections

## Environment Variables

The `.env.browser` file controls browser mode:

```env
VITE_ENABLE_BROWSER_MODE=true  # Enable full UI in browser
VITE_DISABLE_MOCK=false        # Use mock Tauri APIs
```

## Responsive Testing

The app supports multiple viewports:
- Desktop (1200x800)
- Tablet (768x1024)
- Mobile (375x667)

Playwright tests automatically validate responsive behavior.

## Known Limitations

In browser mode:
- No real P2P networking (uses mock data)
- No actual encryption (simulated)
- No persistent storage (in-memory only)
- No native OS features (notifications, etc.)

## Switching Between Modes

### Tauri Mode (Production)
```bash
npm run tauri dev  # Full Tauri app with real backend
```

### Browser Mode (UX Development)
```bash
npm run dev:browser  # Browser with mock APIs
```

### Standard Vite Dev (Shows fallback)
```bash
npm run dev  # Shows "Download Desktop App" message
```

## Troubleshooting

### App Shows Fallback Screen
- Make sure you're running `npm run dev:browser` (not just `npm run dev`)
- Check that `.env.browser` file exists
- Verify `VITE_ENABLE_BROWSER_MODE=true` is set

### Mock Data Not Loading
- Check browser console for errors
- Ensure mock API is initialized (should see `[Mock Tauri] Injecting mock API` in console)
- Verify network requests in DevTools

### Playwright Tests Failing
- Ensure dev server is running: `npm run dev:browser`
- Check that port 1420 is available
- Run `npx playwright install` if browsers are missing

## Next Steps

1. Use Playwright MCP to navigate and test the UI
2. Take screenshots of different states
3. Test user flows and interactions
4. Validate responsive behavior
5. Refine UX based on findings
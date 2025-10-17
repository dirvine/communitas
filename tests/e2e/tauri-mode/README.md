# E2E Tests - Web Mode

These tests run in **WEB MODE** using Playwright + regular browser.

They do NOT run in true Tauri native mode (no `window.__TAURI__` API).

## What's Tested

✅ UI rendering and navigation
✅ Form interactions
✅ Button clicks
✅ WebRTC browser APIs
✅ LocalStorage persistence

## What's NOT Tested

❌ Tauri IPC commands
❌ Rust backend integration  
❌ Native file dialogs
❌ macOS permissions

## Run Tests

```bash
# Terminal 1: Start dev server
npm run dev:frontend

# Terminal 2: Run tests
npm run test:e2e:quick
```

Tests verify your UI works correctly! 🎉

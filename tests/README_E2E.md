# E2E Testing - Quick Start Guide

## ⚡ Quick Start (2 Terminals)

### Terminal 1: Start Tauri Dev Server
```bash
npm run build
npm run tauri dev
```

Wait for: `Local: http://localhost:1420/`

### Terminal 2: Run Tests
```bash
# Run all E2E tests
npm run test:e2e:tauri

# Or use UI mode (recommended)
npm run test:e2e:tauri:ui

# Or use the automated script (starts server for you)
npm run test:e2e:full
```

## ✅ What's Tested (30 tests)

| Suite | Tests | Coverage |
|-------|-------|----------|
| **Onboarding** | 4 | Identity creation, persistence |
| **Messaging** | 5 | Channel messaging, UI elements |
| **Files** | 4 | Upload, storage, documents |
| **WebRTC** | 5 | Call APIs, media devices |
| **Lifecycle** | 12 | App startup, IPC, state |

## 🎯 How It Works

**Tauri Dev Mode:**
1. Tauri runs a web server at `http://localhost:1420`
2. Playwright browser connects to that URL
3. `window.__TAURI__` API is available
4. Tests interact with UI and call Tauri commands

**NOT Electron:** We don't use `electron.launch()` - we connect to the Tauri web server with a regular browser.

## 📝 Key Differences from Electron

| Electron | Tauri |
|----------|-------|
| `electron.launch()` | `page.goto('http://localhost:1420')` |
| Launches app directly | Connects to running dev server |
| Separate process | Web-based testing |
| `require('electron')` | `window.__TAURI__` |

## 🚫 What Can't Be Tested in Dev Mode

- ❌ **Passkeys/Touch ID** - Requires production build
- ❌ **Native file picker** - OS dialog can't be automated
- ❌ **Screen recording** - Requires TCC permission  
- ❌ **macOS Keychain** - Requires entitlements
- ⚠️ **Notifications** - Requires unfocused window

These are tested manually or in production builds.

## 🐛 Troubleshooting

### "Cannot connect to localhost:1420"
→ Start `npm run tauri dev` first

### "Timeout waiting for __TAURI__"
→ Run `npm run build` then restart Tauri dev

### "Tests are flaky"
→ Ensure Tauri dev server is fully started before running tests

### "Port 1420 already in use"
→ Kill existing Tauri: `pkill -f 'tauri dev'`

## 📚 Documentation

- **Full Testing Guide**: [E2E_TESTING_GUIDE.md](./E2E_TESTING_GUIDE.md)
- **Production Report**: [PRODUCTION_READINESS_REPORT.md](../PRODUCTION_READINESS_REPORT.md)
- **Implementation Details**: [TEST_IMPLEMENTATION_SUMMARY.md](../TEST_IMPLEMENTATION_SUMMARY.md)

## 🎉 Success!

If you see:
```
✅ 18 passed (18s)
```

Your E2E test suite is working! 🚀

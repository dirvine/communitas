# 🚨 IMPORTANT: How to Run E2E Tests

## The Problem You Just Hit

You ran `npm run test:e2e:tauri` but **Tauri dev server wasn't running**, so:
- Tests tried to connect to `http://localhost:5173`
- Nothing was there (port 1420 not listening)
- Tests timed out waiting for `window.__TAURI__`

## ✅ Correct Way to Run Tests

You **MUST** have Tauri running first. Here's how:

### Terminal 1: Start Tauri Dev
```bash
npm run build      # Build frontend first
npm run tauri dev  # Start Tauri - keep this running!
```

**WAIT** until you see:
```
   __  __          __              __
  / /_/ /___ ___ _/ /______ ____  / /
 / __/ // _ `/ // / __/ -_) / _ \/ _ \
/__/ \_ \_,_/\__,_/\__/\__/ /____/_//_/
     /___/

Local:   http://localhost:5173/
```

### Terminal 2: Run Tests
```bash
npm run test:e2e:quick
```

## Automated Option

```bash
# This script starts Tauri for you
bash scripts/run-e2e-tests.sh
```

## Quick Check: Is Tauri Running?

```bash
lsof -i :1420
# Should show something listening on port 1420
```

If nothing, Tauri isn't running → tests will fail.

## Package.json Scripts

```bash
# Assumes Tauri is already running
npm run test:e2e:quick     

# Starts Tauri + runs tests + cleanup
npm run test:e2e:full      

# Individual test commands
npm run test:e2e:tauri     # All tests
npm run test:e2e:tauri:ui  # UI mode  
```

---

**Bottom line**: Start `npm run tauri dev`, THEN run tests! 🚀

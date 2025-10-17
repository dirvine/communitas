# How to Run E2E Tests - IMPORTANT

## ⚠️ You MUST Start Tauri Dev Server First!

The tests connect to a **running** Tauri dev server. The tests will FAIL if the server isn't running.

## Steps to Run Tests

### 1. Build Frontend (one-time)
```bash
npm run build
```

### 2. Start Tauri Dev Server (keep running)
**Terminal 1:**
```bash
npm run tauri dev
```

**WAIT** for this message:
```
Local:   http://localhost:5173/
```

### 3. Run Tests
**Terminal 2:**
```bash
npm run test:e2e:quick
```

## OR Use Automated Script

```bash
bash scripts/run-e2e-tests.sh
```

This starts the server for you, runs tests, and cleans up.

## Troubleshooting

### "Timeout waiting for __TAURI__"

**Problem**: Tests can't find Tauri API

**Solution**: Make sure `npm run tauri dev` is running in Terminal 1!

Check:
```bash
# Is the server running?
lsof -i :1420

# Should show: node (or similar) LISTEN on :1420
```

### "Cannot connect to localhost:5173"

**Solution**: Start `npm run tauri dev` first!

### Still Not Working?

```bash
# 1. Kill any existing Tauri process
pkill -f 'tauri dev'

# 2. Rebuild
npm run build

# 3. Start fresh
npm run tauri dev

# 4. In another terminal, run tests
npm run test:e2e:quick
```

---

**TL;DR**: Two terminals, two commands:
1. `npm run tauri dev` (Terminal 1) 
2. `npm run test:e2e:quick` (Terminal 2)

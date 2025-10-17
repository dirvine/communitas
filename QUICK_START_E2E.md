# E2E Tests Quick Start - CORRECTED

## ✅ Fixed: Port Issue Resolved

**The issue**: Tauri uses port **5173** (not 1420)!

All tests updated to use `http://localhost:5173`

## How to Run

### Terminal 1: Start Tauri
```bash
npm run tauri dev
```

Wait for:
```
Local: http://localhost:5173/
```

### Terminal 2: Run Tests
```bash
npm run test:e2e:quick
```

## Verify Tauri is Running

```bash
lsof -i :5173
# Should show vite or node listening on port 5173
```

## Automated Script

```bash
bash scripts/run-e2e-tests.sh
```

This starts Tauri, runs tests, and cleans up.

---

**Port 5173** is the correct Tauri dev server port!

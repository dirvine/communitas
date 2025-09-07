# 🌐 Network Connection Testing Guide

## 🚀 How to Run and Test

### Step 1: Start the Application

```bash
# In terminal, run:
npm run tauri dev

# The app will open in a window and also be available at:
# http://localhost:1422
```

### Step 2: Open Developer Console

**In the Tauri app window:**
- Press `F12` or `Cmd+Option+I` (Mac) / `Ctrl+Shift+I` (Windows/Linux)
- The developer console will open

### Step 3: Observe Network Behavior

**What happens automatically on startup:**

1. **App launches** → NetworkConnectionService initializes
2. **Auto-connection attempt** → Tries to connect to P2P network
3. **Network status indicator** appears in header:
   - 🟡 **Yellow "Connecting..."** - Attempting connection
   - 🟢 **Green "Connected (X peers)"** - Successfully connected
   - 🟡 **Yellow "Local Mode"** - No network, working offline
   - 🔴 **Red "Connection Error"** - Failed after retries

## 🧪 Testing Commands

Open the console and use these commands:

### Quick Status Check
```javascript
// Check current network status
window.testNetwork.status()
```

### Manual Connection Control
```javascript
// Try to connect manually
await window.testNetwork.connect()

// Disconnect and go to local mode
await window.testNetwork.disconnect()
```

### Simulate Network Conditions
```javascript
// Simulate going offline (browser loses internet)
window.testNetwork.simulateOffline()

// Simulate coming back online
window.testNetwork.simulateOnline()
```

### Run Complete Test Flow
```javascript
// This runs a full test of all network features
await window.testNetwork.testFlow()
```

## 🎯 What to Look For

### Visual Indicators

**In the App Header:**
- Look for the network status chip (top right area)
- Colors indicate status:
  - 🟢 Green = Connected to network
  - 🟡 Yellow = Connecting or Local mode
  - 🔴 Red = Error state

### Interactive Features

**Click the Network Status Chip:**
- When **offline/local**: Clicking attempts to reconnect
- When **connected**: Shows detailed popup with:
  - Current status
  - Number of connected peers
  - Bootstrap nodes available
  - Last connection times
  - Error messages (if any)
  - "Go Local" button to disconnect
  - "Connect to Network" button when offline

### Console Output

Watch the console for these messages:
```
🚀 Initializing network connection service...
🔍 Fetching bootstrap nodes...
📡 Found X bootstrap nodes
🔌 Connecting to P2P network...
✅ Connected to network with X peers
```

Or if offline:
```
📵 Browser offline, using local mode
⚠️ No bootstrap nodes available, using local mode
🏠 Max retries reached, switching to local mode
```

## 📝 Test Scenarios

### Scenario 1: Normal Startup
1. Start the app
2. Watch it auto-connect
3. See green "Connected" indicator

### Scenario 2: Offline Mode
1. Run `window.testNetwork.simulateOffline()`
2. See indicator turn yellow "Offline Mode"
3. Click indicator to try reconnecting (will fail while offline)
4. Run `window.testNetwork.simulateOnline()`
5. Watch it auto-reconnect

### Scenario 3: Manual Control
1. Click green "Connected" indicator
2. Click "Go Local" in popup
3. See yellow "Local Mode" indicator
4. Click indicator again
5. Click "Connect to Network" to reconnect

### Scenario 4: Network Failure Recovery
1. The app automatically retries with exponential backoff:
   - 1st retry: after 1 second
   - 2nd retry: after 3 seconds
   - 3rd retry: after 10 seconds
   - Then falls back to local mode

## 🔧 Troubleshooting

### If Network Status Indicator Doesn't Appear
- Check console for errors
- Ensure the app is running (`npm run tauri dev`)
- Refresh the window (Cmd+R / Ctrl+R)

### If Connection Always Fails
- Check if you have internet connectivity
- Bootstrap nodes might be unavailable (app will work in local mode)
- Check console for specific error messages

### Testing Offline-First Features
```javascript
// Create data while offline
window.testNetwork.simulateOffline()
await window.workspace.setup()  // Creates test data

// Come back online - data syncs automatically
window.testNetwork.simulateOnline()
```

## 📊 Understanding Network States

| State | Indicator | Description | User Actions |
|-------|-----------|-------------|--------------|
| `connecting` | 🟡 Yellow, pulsing | Attempting to connect | Wait or click to cancel |
| `connected` | 🟢 Green | Connected to P2P network | Click for details/disconnect |
| `offline` | 🟡 Yellow | Browser is offline | Click to retry when online |
| `local` | 🟡 Yellow | Working locally (no network) | Click to try connecting |
| `error` | 🔴 Red | Connection failed | Click to retry |

## 🎉 Success Criteria

The network connection system is working correctly if:

✅ App attempts to connect automatically on startup
✅ Network status indicator appears in header
✅ Indicator changes color based on connection state
✅ Clicking indicator when offline attempts reconnection
✅ Clicking indicator when connected shows details popup
✅ App falls back to local mode when network unavailable
✅ App auto-reconnects when browser comes online
✅ All data operations work in local mode
✅ Console test commands work as expected

## 💡 Tips

- The app is **offline-first** - everything works without network
- Network connection enhances features but isn't required
- Local mode data syncs automatically when network returns
- Use `window.testNetwork.testFlow()` for automated testing
- Check `window.offlineTest.storage.getStats()` for storage info

---

**Ready to test!** Start with `npm run tauri dev`, open console with F12, and run `window.testNetwork.status()`
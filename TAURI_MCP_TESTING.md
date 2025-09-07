# Tauri MCP Testing Guide

## 🚀 Quick Start

Open the Tauri app and press F12 to open the developer console. All test utilities are automatically loaded and available in the console.

## 📋 Available Test Commands

### 1. Identity Testing
```javascript
// Test identity creation and login
window.testIdentity.testFlow()

// Show auth context information
window.testIdentity.testAuth()
```

### 2. Offline Capabilities
```javascript
// Run all offline tests
window.offlineTest.test()

// Simulate offline/online transitions
window.offlineTest.simulate()

// Export offline data for backup
window.offlineTest.export()

// Check storage statistics
window.offlineTest.storage.getStats()
```

### 3. Complete Workspace Setup
```javascript
// Create a complete test workspace with identity, org, channels, projects
window.workspace.setup()

// List all workspace contents
window.workspace.list()

// Clear all test data
window.workspace.clear()
```

### 4. Group Management (Tauri MCP)
```javascript
// Test group creation and management
window.tauriGroups.test()

// Test group messaging
window.tauriGroups.testMessaging()

// List all groups
window.tauriGroups.listGroups()
```

## 🎯 What Gets Created

### When you run `window.workspace.setup()`:

#### Identity
- Four-word address: `saorsa-labs-test-workspace`
- Display name: David Irvine
- Device: MacBook Pro (Desktop)

#### Organization
- Name: Saorsa Labs
- Description: Building the future of decentralized collaboration
- Settings: Private, invite-only, requires approval

#### Channels (4)
- `#general` - General discussion
- `#development` - Development discussions  
- `#random` - Off-topic chat
- `#announcements` - Important announcements (pinned)

#### Projects (3)
- **Communitas Development** - Main app development (active)
- **Saorsa Core** - P2P networking library (active)
- **Website Redesign** - Marketing website (planning)

#### Messages (6)
- Welcome messages in general
- Development updates in development channel
- Casual chat in random
- Announcement about offline-first features

#### Documents (3)
- README.md
- SPECIFICATION.md
- API.md

#### Team Members (3)
- Alice Johnson - Developer
- Bob Smith - Designer
- Charlie Davis - Product Manager

## 🔧 Tauri Commands Used

The following Tauri commands are invoked:

### Core Commands
- `core_initialize` - Initialize identity
- `core_create_channel` - Create channels
- `core_send_message_to_channel` - Send messages
- `core_private_put` - Store encrypted data
- `core_private_get` - Retrieve encrypted data
- `core_get_channels` - List channels
- `core_get_bootstrap_nodes` - Get network nodes

### Group Commands  
- `core_group_create` - Create groups
- `core_group_add_member` - Add members
- `core_group_remove_member` - Remove members

### Other Commands
- `health` - Check system health
- `core_send_message_to_recipients` - Direct messaging

## 📦 Storage Layers

Data is stored in multiple layers for redundancy:

1. **Memory Cache** - Fast access during session
2. **IndexedDB** - Persistent browser storage
3. **Tauri Backend** - Encrypted local storage via `core_private_put/get`
4. **localStorage** - User preferences
5. **Offline Sync Queue** - Pending operations when offline

## 🌐 Offline Behavior

The app works completely offline:

- **Identity creation** ✅ Works offline
- **Data storage** ✅ Persists locally
- **Message creation** ✅ Queued for sync
- **File access** ✅ Cached locally
- **Group management** ✅ Stored offline

When network returns:
- Sync queue processes automatically
- Fresh data updates caches
- No data loss

## 🧪 Testing Workflow

### Complete Setup Test
```javascript
// 1. Setup workspace
await window.workspace.setup()

// 2. Test groups
await window.tauriGroups.test()

// 3. Verify offline
await window.offlineTest.test()

// 4. List everything
await window.workspace.list()
```

### Offline Testing
```javascript
// 1. Create data
await window.workspace.setup()

// 2. Go offline
window.offlineTest.simulate()

// 3. Create more data (will queue)
// ... interact with UI ...

// 4. Come back online (auto-syncs)
// Simulated automatically after 3 seconds
```

### Group Testing
```javascript
// 1. Initialize identity first
await window.testIdentity.testFlow()

// 2. Create groups
await window.tauriGroups.test()

// 3. Send messages
await window.tauriGroups.testMessaging()

// 4. Check storage
await window.tauriGroups.listGroups()
```

## 📊 Monitoring

Check system status anytime:
```javascript
// Storage statistics
const stats = await window.offlineTest.storage.getStats()
console.log(stats)
// Shows: cacheSize, syncQueueLength, isOnline, contentCount, fileCount

## 🧩 MCP Server (Puppeteer)

You can drive these tests from an MCP client via the included Puppeteer MCP server:

1) Start the app in browser mode or Tauri dev so the UI is available at http://localhost:1420

2) Run the server
```bash
export MCP_BROWSER_URL=http://localhost:1420
npm run mcp:puppeteer
```

3) From your MCP client, call tools:
- `app_test_identity` → runs `window.testIdentity.testFlow()`
- `app_setup_workspace` → runs `window.workspace.setup()`
- `app_test_groups` → runs `window.tauriGroups.test()`
- `app_test_group_messaging` → runs `window.tauriGroups.testMessaging()`
- `app_offline_simulate` → runs `window.offlineTest.simulate()`

See `MCP_SERVERS.md` for details and additional browser_* tools.

// List all content types
const channels = await window.offlineTest.storage.getContentByType('channel')
const messages = await window.offlineTest.storage.getContentByType('message')
const groups = await window.offlineTest.storage.getContentByType('group')
```

## 🔐 Security

All sensitive data is:
- Encrypted via Tauri backend
- Stored locally only
- Never sent to external servers
- Protected by post-quantum cryptography

## 🐛 Troubleshooting

### If commands fail:
1. Check if Tauri backend is running: `window.__TAURI__`
2. Check console for errors
3. Try offline fallback: Data still saved locally
4. Clear and retry: `window.workspace.clear()`

### Network issues:
- App works offline automatically
- Check status: `navigator.onLine`
- Force offline: `window.dispatchEvent(new Event('offline'))`
- Force online: `window.dispatchEvent(new Event('online'))`

## 📝 Notes

- All data is stored locally first
- Network operations are optional
- Everything syncs when online
- No data loss when offline
- Export backup anytime: `window.offlineTest.export()`

---

**Ready to test!** Open the console and try `window.workspace.setup()` to create a complete test environment.

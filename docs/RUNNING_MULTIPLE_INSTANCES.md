# Running Multiple Communitas Instances

## Quick Guide for Testing P2P Networking

### Method 1: Identity Switcher (Single Window, Multiple Identities)

**Easiest way to test but only ONE active at a time:**

1. **Start the app:**
   ```bash
   npm run tauri dev
   ```

2. **Login as David** (or create first identity)

3. **Check network status:**
   - Look for logs: `🌐 Starting P2P networking for {four-words}`
   - Should see: `✅ Network started successfully`
   - App listens on random port 49152-65535

4. **Create second identity:**
   - Click avatar in header
   - Click "Create New Identity"
   - Creates: `ocean-forest-moon-star` (example)
   - Gets new network port: 54322

5. **Switch between identities:**
   - Click avatar → Select from "Switch Identity" menu
   - Each identity maintains separate:
     - Network port
     - Storage (`~/.local/share/communitas/{four-words}/`)
     - CRDT state
     - P2P connections

**Limitation:** Only ONE identity active at a time

---

### Method 2: Multiple Browser Windows (SIMULTANEOUS)

**Best for actual P2P testing:**

**Terminal 1 - Start Tauri Dev:**
```bash
npm run tauri dev
```
Wait for app to open

**Terminal 2 - Build Frontend:**
```bash
npm run build
```

**Terminal 3 - Serve on Different Port:**
```bash
# Install serve if needed
npm install -g serve

# Serve on port 5001
serve -s dist -l 5001
```

**Open Both:**
1. **Window 1:** Tauri app (already open from terminal 1)
   - Login as "David" → becomes `valley-desert-desert-otter`
   - Network starts on port 54321

2. **Window 2:** http://localhost:5001 (in Incognito/Private browser)
   - Create new identity → becomes `ocean-forest-moon-star`
   - Network starts on port 54322

**Now both are running SIMULTANEOUSLY on same machine!**

---

### Method 3: Multiple Tauri Processes (Advanced)

**Terminal 1 - Identity A:**
```bash
npm run tauri dev
```

**Terminal 2 - Identity B (wait for terminal 1 to finish building):**
```bash
# Start second instance (different frontend port)
VITE_PORT=5174 npm run tauri dev
```

You'll need to manually create different identities in each window.

---

## Testing P2P Communication

### Verify Network Started

Check console/logs for BOTH instances:
```
🌐 Starting P2P networking for ocean-forest-moon-star
ℹ️  Allocated port 54321 for QUIC transport
ℹ️  Gossip networking started on 192.168.1.100:54321
✅ Network started successfully: ocean-forest-moon-star-192-168-1-100-54321
✅ CoreContext initialized with P2P networking
```

### Test 1: Create Document in Instance A

**Instance A console:**
```javascript
// Files view → Create New Document
// Name: "Test Doc"
```

**Expected:**
- Document created successfully
- Stored locally in `~/.local/share/communitas/valley-desert-desert-otter/`

### Test 2: Connect Instances (Future Feature)

When peer discovery is implemented:
```javascript
// Instance B connects to Instance A
await invoke('core_connect_to_peer', {
  peerFourWords: 'valley-desert-desert-otter'
});
```

---

## Troubleshooting

### Port Conflicts

If you see "Port already in use":
```bash
# Kill processes on conflicting ports
lsof -ti:5173 | xargs kill -9
lsof -ti:3001 | xargs kill -9
```

### Network Not Starting

Check logs for:
```
⚠️ Network startup failed (continuing in local mode): ...
```

**Common causes:**
- Firewall blocking UDP
- No network interface available
- Port allocation failed

**Fix:** Check firewall settings, allow UDP traffic

### CoreContext Not Initialized

If you see `CoreContext not initialized`:
1. Make sure you're logged in
2. Check console for: `✅ CoreContext initialized`
3. If missing, check for errors in login flow

---

## Network Architecture

```
Instance A (valley-desert-desert-otter)
├── QUIC Transport: 192.168.1.100:54321
├── HyParView: Peer discovery via FOAF
├── Plumtree: Gossip pubsub
└── CRDT Anti-Entropy: Auto-sync

Instance B (ocean-forest-moon-star)
├── QUIC Transport: 192.168.1.100:54322
├── HyParView: Connects to Instance A
├── Plumtree: Propagates messages
└── CRDT Anti-Entropy: Syncs state
```

---

## Display Name Updates

**KNOWN ISSUE:** Display name changes don't persist to identity list

**Why:** The `RecentIdentity` struct in the vault stores the original display name. When you update via settings, it updates:
- ✅ Current session (shows in header)
- ❌ Vault metadata (doesn't update recent identities list)

**Workaround:** Display name is cosmetic. Use four-word address as primary identifier.

**To Fix (Future):**
Need to update `auth_update_identity` to also update the vault's `recent_identities.json`

---

## Next Steps

1. ✅ **Multiple identities** - Create via "Create New Identity"
2. ✅ **Network starts automatically** - Check logs for confirmation
3. ✅ **Document creation works** - CoreContext initialized
4. ⏳ **Peer discovery** - Not yet implemented
5. ⏳ **Message sync** - Waiting for peer connections

**Current Status:** Each instance runs independently with own network stack. Peer-to-peer communication requires manual connection or bootstrap nodes (coming soon).

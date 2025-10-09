# CRDT Multi-Peer Testing Guide

## Overview
This guide demonstrates how to test bidirectional CRDT messaging between two peers using two methods:
1. **Browser Testing**: Using Chrome DevTools MCP with BroadcastChannel (for UI/UX testing)
2. **Tauri Testing**: Using actual P2P networking with saorsa-gossip (for integration testing)

**Status**:
- ✅ Browser Testing: Tested and verified working
- 🚧 Tauri Testing: Implementation complete, ready for multi-peer testing

## Test Scenario

### Setup: Two Peers
- **Peer Alice**: four-word ID `ocean-forest-moon-star`
- **Peer Bob**: four-word ID `river-mountain-cloud-light`

### Test Flow

#### 1. Launch Two Browser Instances

**Instance 1 (Alice)**:
```bash
# Open Chrome with separate profile for Alice
open -na "Google Chrome" --args --user-data-dir=/tmp/chrome-alice http://localhost:5173
```

**Instance 2 (Bob)**:
```bash
# Open Chrome with separate profile for Bob
open -na "Google Chrome" --args --user-data-dir=/tmp/chrome-bob http://localhost:5173
```

#### 2. Configure Peer IDs

The ModernShellPrototype now supports peer ID configuration via URL parameters and localStorage.

**Option A - URL Parameters** ✅ (Recommended):
- Navigate to `http://localhost:5173/?peerId=ocean-forest-moon-star` (Alice)
- Navigate to `http://localhost:5173/?peerId=river-mountain-cloud-light` (Bob)

**Option B - localStorage** (Alternative):
```javascript
// In Alice's browser console:
localStorage.setItem('testPeerId', 'ocean-forest-moon-star');
location.reload();

// In Bob's browser console:
localStorage.setItem('testPeerId', 'river-mountain-cloud-light');
location.reload();
```

**Priority**: URL parameter > localStorage > default value

#### 3. Chrome DevTools MCP Testing

Use Chrome DevTools MCP to interact with both instances:

**For Alice's instance**:
```typescript
// Take snapshot to identify elements
await chrome_devtools.take_snapshot();

// Find the message input field
await chrome_devtools.fill({
  uid: "input_message", // UID from snapshot
  value: "Hello Bob! Testing CRDT sync 📨"
});

// Click send button
await chrome_devtools.click({
  uid: "send_button" // UID from snapshot
});

// Verify message appears in Alice's chat
await chrome_devtools.take_screenshot({
  fullPage: true,
  filePath: "/tmp/alice-sent-message.png"
});
```

**For Bob's instance**:
```typescript
// Switch to Bob's browser (port 5173, different profile)
await chrome_devtools.take_snapshot();

// Wait for sync (2 second polling interval)
await chrome_devtools.wait_for({
  text: "Hello Bob! Testing CRDT sync 📨",
  timeout: 5000
});

// Verify message appears
await chrome_devtools.take_screenshot({
  fullPage: true,
  filePath: "/tmp/bob-received-message.png"
});

// Bob replies
await chrome_devtools.fill({
  uid: "input_message",
  value: "Hi Alice! CRDT sync working! 🎉"
});

await chrome_devtools.click({
  uid: "send_button"
});
```

**Back to Alice's instance**:
```typescript
// Switch back to Alice's browser
await chrome_devtools.wait_for({
  text: "Hi Alice! CRDT sync working! 🎉",
  timeout: 5000
});

// Verify bidirectional sync complete
await chrome_devtools.take_screenshot({
  fullPage: true,
  filePath: "/tmp/alice-received-reply.png"
});
```

## Success Criteria

✅ **Message Delivery**: Messages sent from Alice appear on Bob's screen within 2 seconds
✅ **Bidirectional Sync**: Messages from both directions are visible to both peers
✅ **Causal Ordering**: Messages maintain correct ordering via vector clocks
✅ **No Duplicates**: CRDT ensures each message appears exactly once
✅ **Out-of-Order Handling**: Messages arriving out of order are queued and sorted correctly

## Implementation Notes

### Browser-Compatible Implementation ✅
**File**: `src/services/MessageSyncService.browser.ts`

```typescript
// Initialize with peer ID
await messageSyncService.current.initialize(testPeerId)
// Uses BroadcastChannel for cross-tab sync

// Send message
const crdtMessage = await messageSyncService.current.sendMessage(
  selectedConversationId,
  entityType,
  text,
  author,
  replyToId
)
// Broadcasts to BroadcastChannel, saves to localStorage

// Receive messages (polling every 2 seconds)
const crdtMessages = await messageSyncService.current.getMessages(entityId)
// Loads from localStorage, sorts by Lamport clock
```

**Key Features**:
- ✅ BroadcastChannel API for cross-tab communication
- ✅ localStorage for message persistence
- ✅ Full CRDT implementation (vector clocks, Lamport clocks)
- ✅ Message deduplication by ID
- ✅ Causal ordering via sorting

### Production Backend (Tauri)
For production use, swap to `MessageSyncService.ts` which provides:
- Real P2P networking via saorsa-core
- Distributed storage
- Network-wide message sync

## Network Topology

```
┌─────────────────────┐         ┌─────────────────────┐
│   Peer Alice        │         │    Peer Bob         │
│                     │         │                     │
│  ocean-forest-      │◄───────►│  river-mountain-    │
│   moon-star         │  CRDT   │   cloud-light       │
│                     │  Sync   │                     │
│  Browser 1          │         │  Browser 2          │
│  :5173              │         │  :5173              │
└─────────────────────┘         └─────────────────────┘
         │                               │
         │                               │
         └───────────┬───────────────────┘
                     │
              ┌──────▼──────┐
              │   Backend   │
              │  Message    │
              │  Sync DB    │
              └─────────────┘
```

## Troubleshooting

### Messages Not Syncing
1. Check browser console for errors
2. Verify MessageSyncService initialized: `✅ MessageSyncService initialized with peer: ...`
3. Check sync polling is active: `🔄 Synced: N messages`
4. Verify backend commands are registered in Tauri

### Duplicate Messages
- CRDT should prevent this via message ID deduplication
- Check vector clock comparison logic in `communitas-core/src/crdt.rs`

### Out-of-Order Messages
- Messages should queue when dependencies missing
- Check `has_dependencies()` logic
- Verify `get_missing_ranges()` returns correct gaps

---

## Method 2: Tauri Multi-Peer Testing (Production P2P)

### Overview
Test real P2P networking with saorsa-gossip coordinator and bootstrap manager for peer discovery.

### Architecture

```
┌─────────────────┐       ┌─────────────────┐
│  Instance 1     │       │  Instance 2     │
│  (Alice)        │◄─────►│  (Bob)          │
│                 │  QUIC │                 │
│  Four-words:    │  P2P  │  Four-words:    │
│  ocean-forest-  │       │  river-mountain-│
│  moon-star      │       │  sun-cloud      │
│                 │       │                 │
│  MessageSync    │       │  MessageSync    │
│  VectorClock    │       │  VectorClock    │
│  Port: 8080     │       │  Port: 8081     │
└────────┬────────┘       └────────┬────────┘
         │                         │
         │    ┌───────────────┐    │
         └───►│ saorsa-gossip │◄───┘
              │  Coordinator  │
              └───────┬───────┘
                      │
              ┌───────▼───────┐
              │   Bootstrap   │
              │    Manager    │
              └───────────────┘
```

### Prerequisites

1. **Rust toolchain**: 1.85+ with `cargo`
2. **Node.js**: 20+ with `npm`
3. **saorsa-core**: 0.3.17+ (already in Cargo.toml)
4. **Two terminal windows**

### Step 1: Launch Alice (Instance 1)

```bash
# Terminal 1
cd /Users/davidirvine/Desktop/Devel/projects/communitas

# Set environment for Alice
export COMMUNITAS_DATA_DIR="$HOME/.communitas-data-alice"
export COMMUNITAS_PORT=8080
export COMMUNITAS_PEER_ID="ocean-forest-moon-star"

# Run Tauri development mode
npm run tauri dev
```

**In the app (Alice):**
1. Register/Login with:
   - Four-word identity: `ocean-forest-moon-star`
   - Display name: `Alice`
   - Device name: `Alice Desktop`
2. Wait for "Connected to network" status in header

### Step 2: Launch Bob (Instance 2)

```bash
# Terminal 2 (separate terminal window)
cd /Users/davidirvine/Desktop/Devel/projects/communitas

# Set environment for Bob
export COMMUNITAS_DATA_DIR="$HOME/.communitas-data-bob"
export COMMUNITAS_PORT=8081
export COMMUNITAS_PEER_ID="river-mountain-sun-cloud"

# Run Tauri development mode (will build separate instance)
npm run tauri dev
```

**In the app (Bob):**
1. Register/Login with:
   - Four-word identity: `river-mountain-sun-cloud`
   - Display name: `Bob`
   - Device name: `Bob Desktop`
2. Wait for "Connected to network" status

### Step 3: Verify Peer Discovery

**Check both apps for:**
- ✅ Network status: "Connected" (green indicator in header)
- ✅ Bootstrap manager initialized
- ✅ Peer discovered message in logs

**Console logs should show:**
```
✅ MessageSyncService initialized for peer: ocean-forest-moon-star
🔄 Bootstrap manager started with 2 default nodes
🌐 Connected to peer: river-mountain-sun-cloud (172.20.10.5:8081)
```

### Test Scenarios

#### Test 1: Person-to-Person Messaging

**In Alice's app:**
1. Navigate to Contacts/People
2. Add Bob as contact: `river-mountain-sun-cloud`
3. Send message: "Hello Bob! Testing real P2P sync 🚀"

**In Bob's app:**
1. Message should appear within 500ms
2. Verify message shows:
   - Author: "Alice"
   - Vector clock: `{"ocean-forest-moon-star": 1}`
   - Lamport clock: `1`

**In Bob's app:**
1. Reply: "Hi Alice! P2P working great! 🎉"

**In Alice's app:**
1. Reply appears within 500ms
2. Vector clock merged: `{"ocean-forest-moon-star": 1, "river-mountain-sun-cloud": 1}`

**Expected Results:**
- ✅ Real-time delivery (< 500ms)
- ✅ Causal ordering maintained
- ✅ Vector clocks properly incremented
- ✅ No network polling (event-driven)

#### Test 2: Offline Messaging & Sync

**In Alice's app:**
1. Simulate offline: Disconnect network or kill bootstrap
2. Send 3 messages to Bob:
   - "Message 1 while offline"
   - "Message 2 while offline"
   - "Message 3 while offline"
3. Messages show "Pending" status

**In Bob's app:**
1. No messages appear (Alice is offline)

**In Alice's app:**
1. Reconnect network
2. Messages automatically sync
3. Status changes to "Sent"

**In Bob's app:**
1. All 3 messages appear in correct order
2. Vector clock: `{"ocean-forest-moon-star": 3}`

**Expected Results:**
- ✅ Messages queued locally while offline
- ✅ Automatic sync on reconnection
- ✅ Causal order preserved
- ✅ No message loss

#### Test 3: Concurrent Messages (CRDT Conflict Resolution)

**Simultaneously send from both apps:**
- Alice sends: "Concurrent message from Alice"
- Bob sends: "Concurrent message from Bob"

**Observe both apps:**
- Both messages appear on both sides
- Order consistent across peers (determined by Lamport clock)
- Vector clocks show concurrency:
  - `{"ocean-forest-moon-star": N, "river-mountain-sun-cloud": M}`

**Expected Results:**
- ✅ Both messages delivered to both peers
- ✅ Consistent ordering via Lamport clock
- ✅ Vector clocks properly merged
- ✅ No conflicts or duplicates

#### Test 4: Out-of-Order Message Handling

**Simulate packet loss:**
1. Send 5 messages from Alice (1, 2, 3, 4, 5)
2. Use network throttling to drop message 3
3. Bob receives: 1, 2, 4, 5 (missing 3)

**In Bob's app:**
1. Messages 1, 2 appear normally
2. Message 4 detected as out-of-order (missing 3)
3. Check console: `⚠️ Out-of-order message detected`
4. Message 4 queued in pending

**Trigger sync:**
1. Bob requests missing messages from Alice
2. Message 3 arrives and fills gap
3. Message 4 (and 5) process from pending queue

**Expected Results:**
- ✅ Out-of-order detection via vector clock
- ✅ Missing ranges identified
- ✅ Gap-filling sync request
- ✅ All messages in causal order

#### Test 5: Group Chat with 2 Peers

**In Alice's app:**
1. Create group: "Test Group"
2. Add Bob as member

**In Bob's app:**
1. Accept group invitation
2. Navigate to "Test Group"

**Send alternating messages:**
- Alice: "Welcome to the group!"
- Bob: "Thanks for the invite!"
- Alice: "Let's test CRDT group sync"
- Bob: "Working perfectly!"

**Verify both apps:**
- All 4 messages in same order
- Vector clock: `{"ocean-forest-moon-star": 2, "river-mountain-sun-cloud": 2}`
- Lamport clocks monotonically increasing

**Expected Results:**
- ✅ Group messages sync between peers
- ✅ Causal ordering maintained
- ✅ Vector clocks track both peers
- ✅ No duplication

### Debugging Commands

#### Check CRDT State (Browser Console)

```javascript
// In either app's DevTools console
const { invoke } = window.__TAURI__.tauri;

// List messages with CRDT metadata
const messages = await invoke('core_messages_list', {
  entityId: 'contact-bob',
  limit: 100,
  offset: 0
});

console.table(messages.map(m => ({
  id: m.id.substring(0, 20),
  author: m.author,
  lamport: m.lamportClock,
  vectorClock: JSON.stringify(m.vectorClock),
  text: m.text.substring(0, 30)
})));

// Check sync state
const state = await invoke('core_messages_sync_state', {
  entityId: 'contact-bob'
});
console.log('Sync state:', state);
console.log('Message count:', state.messageCount);
console.log('Vector clock:', state.vectorClock);
console.log('Out-of-order:', state.outOfOrderMessages);
```

#### Check Backend Logs

```bash
# Terminal 1 (Alice)
tail -f ~/.communitas-data-alice/logs/app.log | grep -E "(MessageSync|CRDT|vector_clock)"

# Terminal 2 (Bob)
tail -f ~/.communitas-data-bob/logs/app.log | grep -E "(MessageSync|CRDT|vector_clock)"
```

### Expected Log Output

**Successful Message Send (Alice):**
```
🔄 MessageSyncService initialized for peer: ocean-forest-moon-star
📤 Sending message to entity: contact-bob
✅ Message created with vector clock: {"ocean-forest-moon-star": 1}
📊 Lamport clock: 1
🌐 Broadcasting to peers via gossip
```

**Successful Message Receive (Bob):**
```
📥 Received message from peer: ocean-forest-moon-star
✅ Causal dependencies satisfied
📊 Local vector clock merged: {"ocean-forest-moon-star": 1}
💾 Message stored successfully
```

**Out-of-Order Detection (Bob):**
```
📥 Received message with clock: {"ocean-forest-moon-star": 3}
⚠️  Out-of-order detected (local: {"ocean-forest-moon-star": 1})
❌ Missing events: peer=ocean-forest-moon-star, from=2, to=2
🔄 Requesting sync from alice
📝 Message queued in pending: 1 message waiting
```

### Troubleshooting

#### Peers Not Discovering Each Other

**Symptoms:**
- Network shows "Connected" but no peer messages
- No "Connected to peer" logs

**Solutions:**
1. Check bootstrap manager initialized: `Bootstrap manager started`
2. Verify different four-word identities used
3. Check firewall allows QUIC on ports 8080, 8081
4. Verify both instances use different data directories
5. Restart both instances

#### Messages Not Syncing

**Symptoms:**
- Messages sent but not appearing on other peer
- Vector clocks not incrementing

**Solutions:**
1. Check P2P connection established: Look for gossip logs
2. Verify MessageSyncService initialized on both peers
3. Check for errors in Tauri command handlers
4. Verify entity IDs match on both sides

#### Duplicate Messages

**Symptoms:**
- Same message appears twice
- Vector clocks incorrect

**Solutions:**
1. Check message deduplication logic in `add_message()`
2. Verify message IDs are unique
3. Check for race conditions in sync handlers

### Performance Metrics

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Message Send (local) | < 10ms | Local CRDT creation |
| Message Send (P2P) | < 100ms | Network delivery |
| Peer Discovery | < 2s | Bootstrap + gossip |
| Sync Request/Response | < 500ms | Full entity sync |
| Vector Clock Merge | < 1ms | Per message |
| Out-of-Order Detection | < 2ms | Vector clock gap analysis |

### Cleanup Between Test Runs

```bash
# Stop both instances (Ctrl+C in terminals)

# Clean data directories
rm -rf ~/.communitas-data-alice
rm -rf ~/.communitas-data-bob

# Kill any remaining processes
pkill -f communitas

# Check no processes running
ps aux | grep communitas
```

## Next Steps

After completing multi-peer messaging tests:

1. **Extend to 3+ peers** - Test gossip propagation with more nodes
2. **Test all entity types** - Projects, channels, organizations
3. **Virtual disk sync** - Extend CRDT to file storage
4. **Website publishing** - CRDT for collaborative content editing
5. **Conflict resolution UI** - Show users when concurrent edits occur

## References

- [CRDT Implementation](../communitas-core/src/crdt.rs)
- [MessageSyncService](../communitas-core/src/message_sync.rs)
- [Bootstrap Integration](../communitas-core/src/bootstrap_integration.rs)
- [CRDT Tests](../communitas-core/tests/crdt_tests.rs)
- [Message Sync Tests](../communitas-core/tests/message_sync_tests.rs)
- [Integration Status](./CRDT_INTEGRATION_STATUS.md)

# Communitas Testing Guide

## Status: Ready for End-to-End Testing ✅

All core frontend-backend integration is complete. The application is ready for comprehensive testing with bootstrap node connectivity and P2P messaging.

---

## What's Already Wired Up

### ✅ Authentication (100% Complete)
- Identity creation with four-word addresses
- Login with four-words + password
- Logout functionality
- Session persistence
- Recent identity management
- Auto-login support
- Passkey/WebAuthn support

**Backend Commands Used:**
- `auth_initialize`
- `auth_create_vault`
- `auth_login`
- `auth_logout`
- `auth_get_session`
- `auth_get_recent_identities`
- `auth_remove_recent_identity`
- `auth_passkey_register`
- `auth_passkey_authenticate`

### ✅ Core Context & Networking (100% Complete)
- Core initialization on login
- Network connection management
- Bootstrap peer support
- Connection status monitoring
- Peer discovery and caching

**Backend Commands Used:**
- `core_initialize` (called automatically after login)
- `gossip_get_own_identity`
- `gossip_get_connection_status`
- `gossip_add_bootstrap_peer`
- `gossip_get_cached_peers`

### ✅ Entity/Channel Management (100% Complete)
- Organization listing
- Channel creation and listing
- Project management
- Group creation
- Contact management
- Four-word identity resolution

**Backend Commands Used:**
- `core_dht_get`
- `core_fetch_identity`
- `core_new_four_word_identity`
- `core_create_group`
- `core_add_contact`

### ✅ Messaging (100% Complete)
- Message listing
- Send messages
- Edit messages
- Delete messages
- Message persistence

**Backend Commands Used:**
- `core_messages_list`
- `send_message`
- `edit_message`
- `delete_message`

---

## Testing Strategy

### Prerequisites

1. **Bootstrap Node**
   - Ensure saorsa-gossip bootstrap node is running
   - Note the bootstrap multiaddr (e.g., `/ip4/127.0.0.1/udp/12345/quic-v1/p2p/<peer-id>`)

2. **Development Build**
   ```bash
   # Terminal 1: Build and start Tauri app
   npm run build
   npm run tauri dev
   ```

---

## Test 1: Single User Flow

### 1.1 First Launch & Registration
1. Launch app for first time
2. Should see FirstLaunchWelcome screen
3. Enter name (e.g., "Alice")
4. Four-word address auto-generated
5. Click "Get Started"
6. **Expected:**
   - Identity created successfully
   - Core initialized
   - Network connection attempted
   - Main shell appears

### 1.2 Network Status Verification
1. Look at sidebar connection status
2. **Expected:**
   - Shows your four-word address
   - Shows online/offline status
   - Shows peer count
3. Click to expand connection details
4. **Expected:**
   - Connection quality indicator
   - Option to add bootstrap peer
   - List of cached peers (if any)

### 1.3 Add Bootstrap Node
1. Click "Add Bootstrap Peer" in connection status
2. Enter bootstrap node's four-word address
3. Click "Add"
4. Wait 5-10 seconds
5. **Expected:**
   - Peer added to cached list
   - Connection status updates
   - Peer count increases

### 1.4 Navigation Test
1. Click through each sidebar section:
   - Chats
   - People
   - Organizations
   - Explore
   - Settings
   - Storage
2. **Expected:**
   - No crashes or errors
   - Proper rendering of each section
   - Data persists between navigation

### 1.5 Logout & Login
1. Click avatar in top-right
2. Select "Logout"
3. **Expected:** Return to login screen
4. Click on your identity in recent list
5. Enter password (same as four-word address if not changed)
6. Click "Sign In"
7. **Expected:**
   - Login successful
   - Core re-initialized
   - Network reconnects
   - Previous state restored

---

## Test 2: Two User Messaging

### 2.1 Setup Second User

**Terminal 2: Second Instance**
```bash
# Use different data directory
npm run tauri dev -- --dev
```

**In Second Instance:**
1. Create identity "Bob"
2. Note Bob's four-word address
3. Add bootstrap node (same as Alice)
4. Verify connection status shows online

### 2.2 Add Each Other as Contacts

**In Alice's Instance:**
1. Go to People section
2. Click "+ New Contact"
3. Enter Bob's four-word address
4. Click "Add Contact"

**In Bob's Instance:**
1. Go to People section
2. Click "+ New Contact"
3. Enter Alice's four-word address
4. Click "Add Contact"

**Expected:**
- Both should see each other in contacts list
- Online status should update
- Four-word addresses should be valid

### 2.3 Direct Messaging

**Alice → Bob:**
1. Click on Bob in contacts list
2. Type message: "Hello Bob, this is Alice!"
3. Press Enter or click Send
4. **Expected:**
   - Message appears in chat
   - Timestamp shown
   - Message status indicator

**Bob's Side:**
1. **Expected:**
   - Message appears in real-time or after refresh
   - Shows sender as Alice
   - Correct timestamp

**Bob → Alice:**
1. Reply: "Hi Alice! P2P messaging works!"
2. **Expected:**
   - Message delivered to Alice
   - Conversation thread maintained

### 2.4 Message Persistence
1. Close both instances
2. Restart both instances
3. Navigate to the conversation
4. **Expected:**
   - All messages still visible
   - Conversation history preserved
   - No data loss

---

## Test 3: Channel/Group Messaging

### 3.1 Create Channel

**In Alice's Instance:**
1. Click "+" in sidebar
2. Select "New Channel"
3. Enter name: "#testing"
4. Enter description: "P2P testing channel"
5. Click "Create"

**Expected:**
- Channel appears in sidebar
- Channel has four-word address
- Alice can send messages in channel

### 3.2 Invite Bob to Channel

**In Alice's Instance:**
1. Click on #testing channel
2. Click channel info icon
3. Click "Invite Member"
4. Enter Bob's four-word address
5. Send invitation

**In Bob's Instance:**
1. Should receive invitation notification
2. Accept invitation
3. **Expected:**
   - #testing appears in Bob's channel list
   - Bob can see channel history

### 3.3 Multi-User Channel Chat

**Alice sends:** "Welcome to the test channel!"
**Bob sends:** "Thanks! I can see your message."
**Alice sends:** "P2P group chat is working!"

**Expected:**
- Both users see all messages
- Messages ordered correctly
- No message loss
- Real-time or near-real-time delivery

---

## Test 4: Offline/Online Transitions

### 4.1 Simulate Offline

**In Alice's Instance:**
1. Disconnect network (system network settings or ConnectionStatus UI)
2. Send message: "Offline message 1"
3. Send message: "Offline message 2"
4. **Expected:**
   - Messages queued locally
   - UI indicates offline mode
   - No errors shown

### 4.2 Reconnect

1. Reconnect network
2. **Expected:**
   - Connection status updates to online
   - Queued messages sync automatically
   - Bob receives the offline messages

### 4.3 Verify Message Sync

**In Bob's Instance:**
1. **Expected:**
   - Receives "Offline message 1"
   - Receives "Offline message 2"
   - Message order preserved
   - All data intact

---

## Test 5: Bootstrap Node Failover

### 5.1 Multiple Bootstrap Peers

**In Alice's Instance:**
1. Add 2-3 bootstrap peer addresses
2. **Expected:**
   - All peers appear in cached list
   - Connection quality improves

### 5.2 Bootstrap Node Disconnection

1. Stop the bootstrap node
2. Wait 30 seconds
3. **Expected:**
   - Connection status updates
   - App attempts reconnection
   - Cached peers used for discovery

### 5.3 Bootstrap Node Recovery

1. Restart bootstrap node
2. **Expected:**
   - App reconnects automatically
   - Connection status returns to normal
   - Messages sync

---

## Verification Checklist

Use this checklist to verify all tests pass:

### Authentication
- [ ] Can create new identity
- [ ] Can login with four-words + password
- [ ] Can logout
- [ ] Session persists across app restarts
- [ ] Recent identities list works
- [ ] Remove from device works
- [ ] Passkey registration works (web only)

### Networking
- [ ] Core initializes after login
- [ ] Connection status displays correctly
- [ ] Can add bootstrap peers
- [ ] Peer list updates
- [ ] Connection quality indicator accurate
- [ ] Network status reflects actual state

### Entities & Channels
- [ ] Can create organizations
- [ ] Can create channels
- [ ] Can create projects
- [ ] Can add contacts
- [ ] Four-word addresses validate correctly
- [ ] Entity listing works

### Messaging
- [ ] Can send messages to contacts
- [ ] Can receive messages from contacts
- [ ] Can send messages to channels
- [ ] Message history persists
- [ ] Offline messages queue
- [ ] Messages sync when reconnecting
- [ ] Edit message works
- [ ] Delete message works

### Offline Support
- [ ] App works without network
- [ ] Messages queue when offline
- [ ] Data persists locally
- [ ] Syncs when connection returns

### Multi-User
- [ ] Two users can see each other online
- [ ] Direct messaging works
- [ ] Channel messaging works
- [ ] Real-time or near-real-time delivery
- [ ] Message order preserved

---

## Known Limitations (To Be Implemented)

### Not Yet Wired
- [ ] Message reactions (core_add_reaction)
- [ ] Thread replies (core_create_thread)
- [ ] Real-time message subscriptions (core_subscribe_messages)
- [ ] Document collaboration (doc_* commands)
- [ ] Website publishing (gossip_site_* commands)
- [ ] Video/voice calls
- [ ] File sharing
- [ ] Screen sharing

### Planned Features
- [ ] End-to-end encryption visualization
- [ ] Message read receipts
- [ ] Typing indicators
- [ ] Push notifications
- [ ] Mobile app support

---

## Troubleshooting

### Issue: Cannot connect to bootstrap node
**Solutions:**
1. Verify bootstrap node is running
2. Check firewall settings
3. Verify four-word address is correct
4. Check network connectivity

### Issue: Messages not appearing
**Solutions:**
1. Check connection status (should be online)
2. Verify both users have each other as contacts
3. Check console for error messages
4. Try manual refresh

### Issue: App crashes on startup
**Solutions:**
1. Clear `.communitas-data/` directory
2. Rebuild frontend: `npm run build`
3. Check console for error details
4. Verify Rust backend compiled successfully

### Issue: "Core not initialized" errors
**Solutions:**
1. Ensure you're logged in
2. Check AuthContext initialization logs
3. Verify core_initialize command is being called
4. Look for network initialization errors in console

---

## Success Criteria

✅ **Phase 1 Complete When:**
- All authentication flows work
- Network connection established
- Connection status accurate
- User can navigate entire app

✅ **Phase 2 Complete When:**
- Two users can message each other
- Messages deliver in real-time
- Offline messages sync correctly
- Channel creation and messaging works

✅ **Phase 3 Complete When:**
- All core features tested
- Multi-user scenarios verified
- Offline/online transitions work
- No critical bugs found

---

## Logging & Debugging

### Enable Debug Logs

**Frontend (Browser Console):**
```javascript
// All messages appear automatically in console
// Look for prefixes:
// 🔵 - Info
// ✅ - Success
// ⚠️ - Warning
// ❌ - Error
```

**Backend (Terminal):**
```bash
RUST_LOG=debug npm run tauri dev
```

### Key Log Messages to Watch

**Successful Login:**
```
✅ Login successful: ocean-forest-moon-star
🌐 Initializing CoreContext with networking...
✅ CoreContext initialized with P2P networking
```

**Network Connection:**
```
Connection status: Online (3 peers)
Cached peers: 3
```

**Message Sending:**
```
🔵 Invoking core_messages_list with params: {...}
✅ core_messages_list result: [...]
Sending message to entity...
✅ Message sent successfully
```

---

*Last Updated: 2025-01-13*
*Status: Ready for Testing*

# Frontend-Backend Integration Plan

## Current Status: Phase 1 - Core Connectivity Complete ✅

### Verified Wiring (2025-01-13)
1. ✅ Core initialization via AuthContext after login
2. ✅ Network status monitoring via ConnectionService → gossip commands
3. ✅ Channel/Entity listing via EntityDirectoryContext → core_dht_get, core_fetch_identity
4. ✅ Message send/receive via MessagesPanel → core_messages_list, send_message

### Next Steps
1. Test connectivity to bootstrap node
2. Test P2P messaging between two logged-in users
3. Complete screen-by-screen testing

---

## Backend Commands Available (96 total)

### Authentication (19 commands)
- ✅ `auth_initialize` - Initialize auth system
- ✅ `auth_create_vault` - Create new identity vault
- ✅ `auth_login` - Login with four-words + password
- ✅ `auth_login_password_only` - Login with password only
- ✅ `auth_logout` - Logout current user
- ✅ `auth_check_session` - Check if session is valid
- ✅ `auth_get_session` - Get current session info
- ✅ `auth_get_recent_identities` - List recent identities
- ✅ `auth_remove_recent_identity` - Remove identity from recent list
- ✅ `auth_list_vaults` - List all vaults
- ✅ `auth_get_config` - Get auth configuration
- ✅ `auth_set_auto_login` - Configure auto-login
- ✅ `auth_try_auto_login` - Attempt auto-login
- ✅ `auth_set_keyring_enabled` - Toggle keyring storage
- ✅ `auth_passkey_register` - Register passkey/WebAuthn
- ✅ `auth_passkey_authenticate` - Authenticate with passkey
- ✅ `auth_passkey_has_passkey` - Check if passkey exists
- ✅ `auth_passkey_get_info` - Get passkey information
- ✅ `auth_passkey_delete` - Delete passkey

### Core/Network (21 commands)
- 🔌 `core_initialize` - Initialize core context with four-words
- 🔌 `core_get_peer_id` - Get current peer ID
- 🔌 `core_get_user_info` - Get current user information
- 🔌 `core_set_display_name` - Update display name
- 🔌 `core_add_bootstrap_node` - Add bootstrap node
- 🔌 `core_update_bootstrap_nodes` - Update bootstrap node list
- 🔌 `core_get_bootstrap_nodes` - Get current bootstrap nodes
- 🔌 `core_get_bootstrap_stats` - Get bootstrap statistics
- 🔌 `core_clear_custom_nodes` - Clear custom nodes
- 🔌 `core_entity_get_encryption_status` - Get entity encryption status
- 🔌 `core_entity_get_permissions` - Get entity permissions
- 🔌 `core_entity_update` - Update entity metadata
- 🔌 `core_entity_delete` - Delete entity
- 🔌 `core_entity_block` - Block entity
- 🔌 `core_entity_mute` - Mute entity
- 🔌 `subscribe_to_entity` - Subscribe to entity updates
- 🔌 `unsubscribe_from_entity` - Unsubscribe from entity
- 🔌 `core_private_get` - Get data from private storage
- 🔌 `core_private_put` - Store data in private storage
- 🔌 `core_add_reaction` - Add reaction to message
- 🔌 `get_sync_status` - Get synchronization status

### Channels (6 commands)
- 📢 `core_create_channel` - Create new channel
- 📢 `core_get_channels` - List all channels
- 📢 `core_channel_list_members` - List channel members
- 📢 `core_channel_recipients` - Get channel recipients
- 📢 `core_channel_invite_by_words` - Invite by four-word address
- 📢 `core_resolve_channel_members` - Resolve member identities

### Messages (10 commands)
- 💬 `core_messages_send` - Send message
- 💬 `core_messages_list` - List messages
- 💬 `core_messages_edit` - Edit message
- 💬 `core_messages_delete` - Delete message
- 💬 `core_send_message_to_channel` - Send to channel
- 💬 `core_send_message_to_recipients` - Send to specific recipients
- 💬 `core_create_thread` - Create message thread
- 💬 `core_subscribe_messages` - Subscribe to message updates
- 💬 `message_sync_send_message` - Send via message sync
- 💬 `message_sync_receive_message` - Receive via message sync

### Message Sync (7 commands)
- 🔄 `message_sync_initialize` - Initialize message sync
- 🔄 `message_sync_get_messages` - Get messages for entity
- 🔄 `message_sync_get_all_messages` - Get all messages
- 🔄 `message_sync_get_sync_state` - Get sync state
- 🔄 `message_sync_needs_sync` - Check if sync needed
- 🔄 `message_sync_request_sync` - Request sync
- 🔄 `message_sync_handle_sync_response` - Handle sync response

### Documents (9 commands)
- 📄 `doc_create` - Create document
- 📄 `doc_list` - List documents
- 📄 `doc_delete` - Delete document
- 📄 `doc_get_text` - Get document text
- 📄 `doc_insert_text` - Insert text
- 📄 `doc_delete_text` - Delete text
- 📄 `doc_get_update` - Get CRDT update
- 📄 `doc_apply_update` - Apply CRDT update

### Gossip/P2P (24 commands)
- 🌐 `gossip_initialize` - Initialize gossip network
- 🌐 `gossip_get_own_identity` - Get own identity
- 🌐 `gossip_get_connection_status` - Get connection status
- 🌐 `gossip_is_peer_online` - Check peer online status
- 🌐 `gossip_get_online_peers` - List online peers
- 🌐 `gossip_get_cached_peers` - Get cached peer list
- 🌐 `gossip_add_bootstrap_peer` - Add bootstrap peer
- 🌐 `gossip_add_contact` - Add contact
- 🌐 `gossip_remove_contact` - Remove contact
- 🌐 `gossip_get_contacts` - List contacts
- 🌐 `gossip_find_contact` - Find specific contact
- 🌐 `gossip_add_favourite_contact` - Add to favourites
- 🌐 `gossip_get_favourite_contacts` - List favourites
- 🌐 `gossip_recover_from_favourite` - Recover from favourite
- 🌐 `gossip_replicate_to_favourites` - Replicate to favourites
- 🌐 `gossip_send_direct_message` - Send direct P2P message
- 🌐 `gossip_store_message` - Store message locally
- 🌐 `gossip_get_all_messages` - Get all gossip messages
- 🌐 `gossip_contains_message` - Check if message exists
- 🌐 `gossip_remove_message` - Remove message
- 🌐 `gossip_join_entity` - Join entity/group
- 🌐 `gossip_leave_entity` - Leave entity/group
- 🌐 `gossip_subscribe_to_entity` - Subscribe to entity updates
- 🌐 `gossip_publish_to_entity` - Publish to entity

### Presence (2 commands)
- 👁️ `gossip_start_presence_beacons` - Start broadcasting presence
- 👁️ `gossip_stop_presence_beacons` - Stop broadcasting presence

### Websites/Sites (3 commands)
- 🌐 `gossip_site_publish` - Publish website
- 🌐 `gossip_site_fetch` - Fetch website content
- 🌐 `gossip_site_list` - List available sites
- 🌐 `gossip_site_providers` - Get site providers

---

## Frontend Components Integration Status

### ✅ Fully Integrated

#### Authentication Components
- `IdentityPicker.tsx` - Uses auth commands for identity management
- `FirstLaunchWelcome.tsx` - Uses auth_create_vault, auth_initialize
- `PasskeyRegistration.tsx` - Uses auth_passkey_register
- `UnifiedAuthFlow.tsx` - Complete auth flow integration
- `AuthContext.tsx` - Handles core_initialize after login, auto-connects to network

#### Core Application
- `AuthContext.tsx` - Initializes core on login with core_initialize
- `ConnectionStatus.tsx` - Real-time network status via ConnectionService
- `ConnectionService.ts` - Full gossip command integration (gossip_get_own_identity, gossip_get_connection_status, gossip_add_bootstrap_peer, gossip_get_cached_peers)

#### Entity/Channel Management
- `EntityDirectoryContext.tsx` - Backend integration for entities (core_dht_get, core_fetch_identity, core_new_four_word_identity, core_create_group, core_add_contact)
- `ModernShellPrototype.tsx` - Uses EntityDirectoryContext for channel listing

#### Messaging
- `MessagesPanel.tsx` - Backend integration (core_messages_list, send_message, edit_message, delete_message)
- Message sync ready via MessageSyncService

### 🔌 Needs Testing

The following components are already wired to backend commands but need end-to-end testing with bootstrap node:

#### Messaging & Real-time Updates
- Message real-time subscription (core_subscribe_messages)
- Thread support (core_create_thread)
- Message reactions (core_add_reaction)
- Offline message sync (message_sync commands)

#### Advanced Features (Not Yet Wired)
- Document collaboration (doc_create, doc_list, doc_apply_update)
- Website publishing (gossip_site_publish, gossip_site_list)
- Advanced storage management (core_private_get, core_private_put)
- Bootstrap node management UI (core_add_bootstrap_node, core_get_bootstrap_nodes)

---

## Integration Priority

### Phase 1: Core Connectivity (CURRENT)
1. ✅ Authentication flow
2. 🔌 Core initialization on login
3. 🔌 Bootstrap node connection
4. 🔌 Network status monitoring

### Phase 2: Messaging Foundation
1. 🔌 Channel creation and listing
2. 🔌 Basic message send/receive
3. 🔌 Real-time message subscription
4. 🔌 Message sync for offline support

### Phase 3: Full Features
1. 🔌 Document collaboration
2. 🔌 Website publishing
3. 🔌 Advanced messaging (threads, reactions)
4. 🔌 Presence/online status

### Phase 4: Testing
1. 🔌 Single user flow (all screens)
2. 🔌 Two users messaging
3. 🔌 Multi-user channels
4. 🔌 Offline/online transitions

---

## Implementation Pattern

### Standard Tauri Command Integration

```typescript
import { invoke } from '@tauri-apps/api/core';

// Example: Send message
async function sendMessage(channelId: string, content: string) {
  try {
    await invoke('core_send_message_to_channel', {
      channelId,
      content,
      recipients: [] // Channel handles routing
    });
  } catch (error) {
    console.error('Failed to send message:', error);
    throw error;
  }
}
```

### Real-time Subscription Pattern

```typescript
import { listen } from '@tauri-apps/api/event';

// Subscribe to message updates
const unlisten = await listen('message-received', (event) => {
  const message = event.payload;
  // Update UI with new message
});

// Cleanup on unmount
return () => {
  unlisten();
};
```

---

## Testing Strategy

### 1. Bootstrap Node Setup
```bash
# Terminal 1: Start bootstrap node
cd saorsa-gossip
cargo run --example bootstrap

# Note the bootstrap address (e.g., /ip4/127.0.0.1/udp/12345/quic-v1/p2p/...)
```

### 2. First User Session
```bash
# Start Tauri app
npm run tauri:dev

# Actions:
# 1. Create identity or login
# 2. Core initializes with bootstrap node
# 3. Verify network status shows "connected"
# 4. Create a channel
```

### 3. Second User Session
```bash
# Start second instance (different profile)
# 1. Create different identity
# 2. Connect to same bootstrap
# 3. Join same channel by four-word invite
# 4. Send message to first user
```

### 4. Verification Checklist
- [ ] Both users show as "online"
- [ ] Messages appear in real-time
- [ ] Offline messages sync when reconnected
- [ ] Channel membership is consistent
- [ ] Document collaboration works
- [ ] Network status accurately reflects state

---

## Current Gaps

### Backend
- ✅ All core commands implemented
- ✅ Authentication complete
- ✅ Message sync ready
- ✅ Gossip/P2P functional

### Frontend
- ⚠️ Most components use mock/demo data
- ⚠️ Limited invoke() calls to backend
- ⚠️ No real-time subscription setup
- ⚠️ Network status not fully wired

### Next Steps
1. Wire up ModernShellPrototype to initialize core on login
2. Connect MessagesPanel to real backend commands
3. Set up real-time message subscriptions
4. Test two-user messaging flow
5. Complete remaining screens iteratively

---

## Bootstrap Node Configuration

### Default Bootstrap
```json
{
  "bootstrap_nodes": [
    "/ip4/127.0.0.1/udp/12345/quic-v1/p2p/<peer-id>"
  ]
}
```

### Adding in UI
```typescript
await invoke('core_add_bootstrap_node', {
  multiaddr: '/ip4/127.0.0.1/udp/12345/quic-v1/p2p/<peer-id>'
});
```

---

## Success Criteria

✅ **Phase 1 Complete When:**
- User can log in and core initializes
- Network status shows connection to bootstrap
- User info displays correctly

✅ **Phase 2 Complete When:**
- Two users can see each other online
- Messages send and receive in real-time
- Channels can be created and joined

✅ **Phase 3 Complete When:**
- All screens functional with real data
- Document collaboration works
- Website publishing operational

✅ **Phase 4 Complete When:**
- Full multi-user testing passed
- Offline/online transitions work
- All features tested end-to-end

---

*Last Updated: 2025-01-12*
*Status: Phase 1 - Core Connectivity*

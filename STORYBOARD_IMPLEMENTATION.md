# Communitas Storyboard Implementation Plan

**Version**: 1.0 • **Date**: 2025-10-12 • **Status**: Active Implementation

This document provides the complete implementation roadmap for wiring up the Communitas storyboard UI (both Tauri desktop and TUI) with the backend P2P infrastructure.

## 🎯 Overview

**Objective**: Implement 100% of the storyboard design from `STORYBOARD_V2.md` and `STORYBOARD.md` in both:
1. **Tauri Desktop App** - React + TypeScript UI with Tauri v2 backend
2. **TUI Terminal App** - Ratatui terminal interface with same backend

**Key Principle**: Both interfaces share the same Rust backend (`communitas-core` + `communitas-desktop`/`communitas-tui`), ensuring feature parity and consistent behavior.

---

## 📋 Backend Command Inventory

### ✅ **Existing Backend Commands** (from `communitas-desktop/src/main.rs`)

#### **Core Initialization & Identity**
- `core_initialize` - Initialize CoreContext with four-word identity
- `core_get_peer_id` - Get current peer ID
- `core_get_user_info` - Get user information
- `core_set_display_name` - Update display name
- `generate_four_word_identity` - Generate new four-word identity
- `validate_four_words` - Validate four-word format (network.rs)
- `get_user_four_words` - Get user's four-word identity

#### **Entity Management**
- `core_entity_update` - Update entity metadata
- `core_entity_delete` - Delete entity
- `core_entity_mute` - Mute entity notifications
- `core_entity_block` - Block entity
- `core_entity_get_permissions` - Get entity permissions
- `core_entity_get_encryption_status` - Get encryption status

#### **Channel Management**
- `core_create_channel` - Create new channel
- `core_get_channels` - List all channels
- `core_channel_recipients` - Get channel recipients
- `core_channel_list_members` - List channel members
- `core_channel_invite_by_words` - Invite user via four-words
- `core_resolve_channel_members` - Resolve channel membership

#### **Messaging**
- `core_send_message_to_channel` - Send message to channel
- `core_send_message_to_recipients` - Send direct message
- `core_messages_list` - List messages
- `core_messages_send` - Send message
- `core_messages_edit` - Edit message
- `core_messages_delete` - Delete message
- `core_add_reaction` - Add reaction to message
- `core_create_thread` - Create message thread
- `core_subscribe_messages` - Subscribe to message updates

#### **Group Management**
- `core_group_create` - Create new group
- `core_group_add_member` - Add member to group
- `core_group_remove_member` - Remove member from group

#### **Storage Operations**
- `core_storage_list` - List files in entity storage
- `core_storage_read` - Read file from storage
- `core_storage_write` - Write file to storage
- `core_storage_mkdir` - Create directory
- `core_storage_fs_delete` - Delete file/directory
- `core_storage_rename` - Rename file/directory
- `core_storage_stats` - Get storage statistics
- `core_private_put` - Store private data
- `core_private_get` - Retrieve private data

#### **Network & Sync**
- `connect_to_network` - Connect to P2P network
- `disconnect_from_network` - Disconnect from network
- `get_network_status` - Get connection status
- `get_network_info` - Get network information
- `get_endpoint_four_words` - Get endpoint four-word address
- `subscribe_to_entity` - Subscribe to entity updates
- `unsubscribe_from_entity` - Unsubscribe from entity
- `get_sync_status` - Get sync status

#### **Bootstrap Management**
- `core_get_bootstrap_nodes` - List bootstrap nodes
- `core_update_bootstrap_nodes` - Update bootstrap list
- `core_add_bootstrap_node` - Add bootstrap node
- `core_clear_custom_nodes` - Clear custom nodes
- `core_get_bootstrap_stats` - Get bootstrap statistics

#### **Document Collaboration (CRDT)**
- `doc_create` - Create collaborative document
- `doc_insert_text` - Insert text in document
- `doc_delete_text` - Delete text from document
- `doc_get_text` - Get document text
- `doc_get_update` - Get document update
- `doc_apply_update` - Apply document update
- `doc_list` - List documents
- `doc_delete` - Delete document

#### **Message Sync (CRDT)**
- `message_sync_initialize` - Initialize message sync
- `message_sync_get_all_messages` - Get all messages
- `message_sync_receive_message` - Receive message
- `message_sync_send_message` - Send message
- `message_sync_request_sync` - Request sync
- `message_sync_handle_sync_response` - Handle sync response
- `message_sync_get_sync_state` - Get sync state
- `message_sync_get_messages` - Get messages
- `message_sync_needs_sync` - Check if sync needed

#### **Organization Features (CRDT-backed)**
- `create_channel` - Create organization channel
- `get_channel` - Get channel info
- `list_channels` - List all channels
- `send_message` - Send message to channel
- `get_messages` - Get channel messages
- `create_thread` - Create message thread
- `get_thread_replies` - Get thread replies
- `add_channel_member` - Add member to channel
- `remove_channel_member` - Remove member from channel
- `get_channel_members` - List channel members
- `create_project` - Create project
- `get_project` - Get project info
- `list_projects` - List projects
- `create_issue` - Create issue
- `get_issue` - Get issue info
- `list_issues` - List issues
- `list_issues_by_status` - Filter issues by status
- `update_issue_status` - Update issue status
- `assign_issue` - Assign issue to user
- `update_issue_priority` - Update issue priority
- `add_issue_comment` - Add comment to issue
- `get_issue_comments` - Get issue comments
- `get_channel_sync_update` - Get channel CRDT update
- `apply_channel_sync_update` - Apply channel CRDT update
- `get_issue_sync_update` - Get issue CRDT update
- `apply_issue_sync_update` - Apply issue CRDT update

#### **Gossip Overlay (Feature-Gated, enabled by default)**
- `gossip_initialize` - Initialize gossip overlay
- `gossip_store_message` - Store message in gossip
- `gossip_get_all_messages` - Get all gossip messages
- `gossip_contains_message` - Check if message exists
- `gossip_remove_message` - Remove message
- `gossip_find_contact` - Find contact by four-words
- `gossip_add_contact` - Add contact
- `gossip_get_contacts` - List contacts
- `gossip_remove_contact` - Remove contact
- `gossip_send_direct_message` - Send direct message
- `gossip_subscribe_to_entity` - Subscribe to entity topic
- `gossip_publish_to_entity` - Publish to entity topic
- `gossip_join_entity` - Join entity group
- `gossip_leave_entity` - Leave entity group
- `gossip_start_presence_beacons` - Start presence broadcasts
- `gossip_stop_presence_beacons` - Stop presence broadcasts
- `gossip_is_peer_online` - Check if peer is online
- `gossip_get_online_peers` - List online peers
- `gossip_add_favourite_contact` - Add favorite contact
- `gossip_get_favourite_contacts` - List favorite contacts
- `gossip_replicate_to_favourites` - Replicate data to favorites
- `gossip_recover_from_favourite` - Recover from favorite
- `gossip_site_publish` - Publish website
- `gossip_site_fetch` - Fetch website
- `gossip_site_list` - List published sites
- `gossip_site_providers` - List site providers
- `gossip_get_own_identity` - Get own identity
- `gossip_get_connection_status` - Get connection status
- `gossip_add_bootstrap_peer` - Add bootstrap peer
- `gossip_get_cached_peers` - Get cached peers

#### **Authentication & Encrypted Storage**
- `auth_initialize` - Initialize auth system
- `auth_create_vault` - Create encrypted vault
- `auth_login` - Login with passkey
- `auth_login_password_only` - Login with password
- `auth_logout` - Logout
- `auth_get_session` - Get current session
- `auth_list_vaults` - List available vaults
- `auth_check_session` - Check session validity
- `auth_get_config` - Get auth config
- `auth_try_auto_login` - Try auto-login
- `auth_get_recent_identities` - Get recent identities
- `auth_set_auto_login` - Set auto-login
- `auth_set_keyring_enabled` - Enable/disable keyring
- `auth_passkey_register` - Register passkey
- `auth_passkey_authenticate` - Authenticate with passkey
- `auth_passkey_has_passkey` - Check if passkey exists
- `auth_passkey_get_info` - Get passkey info
- `auth_passkey_delete` - Delete passkey
- `get_os_username` - Get OS username

#### **Container Engine (Low-level storage)**
- `container_init` - Initialize container
- `container_put_object` - Put object in container
- `container_get_object` - Get object from container
- `container_apply_ops` - Apply operations
- `container_current_tip` - Get current tip

### ❌ **Missing Commands** (Need Implementation)

Based on storyboard requirements, we need to add:

#### **Organization Structure**
- `core_org_create` - Create organization
- `core_org_get` - Get organization details
- `core_org_list` - List user's organizations
- `core_org_update` - Update organization metadata
- `core_org_add_member` - Add member to organization
- `core_org_remove_member` - Remove member from organization
- `core_org_list_members` - List organization members
- `core_org_update_member_role` - Update member role (Owner/Admin/Member)

#### **Project Management**
- `core_project_create` - Create project (already exists as `create_project`)
- `core_project_update` - Update project details
- `core_project_archive` - Archive project
- `core_project_get_members` - Get project members

#### **Storage Visualization**
- `core_storage_get_vault_info` - Get vault details (usage, encryption type)
- `core_storage_get_quota` - Get storage quota
- `core_storage_calculate_usage` - Calculate current usage

#### **Presence & Status**
- `core_presence_set_status` - Set user status (online/away/offline)
- `core_presence_get_status` - Get user status
- `core_presence_list_online_members` - List online members in entity

#### **Search & Discovery**
- `core_search_entities` - Search across entities
- `core_search_messages` - Search messages
- `core_search_files` - Search files in storage

---

## 🎨 Storyboard Component Mapping

### **Priority 1: Core Shell (Week 1)**

#### **1.1 Entity Sidebar (Left Panel)**

**Storyboard Components:**
- Identity Selector
- Filter Chips (Space type + Entity type)
- Search Bar with ⌘K
- Organization Tree (expandable)

**Backend Commands:**
```typescript
// Identity Selector
const user = await invoke('core_get_user_info');
const fourWords = await invoke('get_user_four_words');

// Organization Tree
const orgs = await invoke('core_org_list'); // NEW COMMAND NEEDED
const channels = await invoke('list_channels');
const projects = await invoke('list_projects');
const groups = await invoke('core_group_list'); // NEW COMMAND NEEDED

// Filtering (client-side for now)
// Can add backend filtering later for performance

// Search
const searchResults = await invoke('core_search_entities', { query }); // NEW COMMAND NEEDED
```

**TUI Implementation:**
- Use `ratatui::List` for sidebar
- `ratatui::Tree` for expandable orgs
- `ratatui::Input` for search

#### **1.2 Main Content Area (Center)**

**Storyboard Components:**
- Dynamic view based on selection
- Organization Dashboard (2x2 grid)
- Chat Interface
- Storage Browser

**Backend Commands:**
```typescript
// Organization Dashboard
const org = await invoke('core_org_get', { orgId });
const members = await invoke('core_org_list_members', { orgId });
const projects = await invoke('list_projects');
const channels = await invoke('list_channels');
const storageInfo = await invoke('core_storage_get_vault_info', { entityId: orgId });

// Chat Interface
const messages = await invoke('get_messages', { channelId });
await invoke('send_message', { channelId, content });

// Storage Browser
const files = await invoke('core_storage_list', { entityId, path });
```

**TUI Implementation:**
- Use `ratatui::Block` with `ratatui::layout::Rect` for responsive grid
- `ratatui::Table` for file lists
- Custom chat rendering with scrolling

#### **1.3 Info Panel (Right, Optional)**

**Storyboard Components:**
- Entity details
- Member list
- File preview

**Backend Commands:**
```typescript
// Same as above, filtered for current selection
```

**TUI Implementation:**
- Collapsible panel with `ratatui::Block`
- Show/hide with keyboard shortcut

### **Priority 2: Interactive Components (Week 2)**

#### **2.1 Storage Meters**

**Storyboard Components:**
- Visual progress bars
- Color-coded thresholds (green/yellow/red)
- Vault details

**Backend Commands:**
```typescript
const vaultInfo = await invoke('core_storage_get_vault_info', { entityId });
// Returns: { used, total, percentage, encryption_type }

const stats = await invoke('core_storage_stats', { entityId });
```

**TUI Implementation:**
- Use `ratatui::Gauge` for progress bars
- Color theming with `ratatui::style::Color`

#### **2.2 Member Cards**

**Storyboard Components:**
- Avatar with initials
- Status indicator (online/offline)
- Role badge

**Backend Commands:**
```typescript
const members = await invoke('core_org_list_members', { orgId });
const onlineStatus = await invoke('gossip_is_peer_online', { peerId });

// For presence updates
await invoke('gossip_start_presence_beacons');
```

**TUI Implementation:**
- Use `ratatui::List` with custom rendering
- Status indicators with colored symbols (●/○)

### **Priority 3: Communication Features (Week 3)**

#### **3.1 Chat Interface**

**Storyboard Components:**
- Message list with threads
- Rich message display
- Reactions
- File attachments

**Backend Commands:**
```typescript
// Messages
const messages = await invoke('get_messages', { channelId });
await invoke('send_message', { channelId, content, attachments });
await invoke('core_add_reaction', { messageId, emoji });

// Threads
const thread = await invoke('create_thread', { messageId, content });
const replies = await invoke('get_thread_replies', { threadId });

// File attachments
await invoke('core_storage_write', {
  entityId,
  path: `/attachments/${filename}`,
  content
});
```

**TUI Implementation:**
- Scrollable message list with `ratatui::List`
- Thread view mode (navigate with keys)
- File upload/download progress bars

#### **3.2 Video/Audio Calls**

**Storyboard Components:**
- Call controls (mute, video, screenshare)
- Participant grid
- Call status

**Backend Commands:**
```typescript
// WebRTC handled in frontend for now
// Could add backend signaling later
```

**TUI Implementation:**
- Not applicable for TUI (audio-only via ALSA/PulseAudio?)
- Show call status and participants only

### **Priority 4: Storage Management (Week 4)**

#### **4.1 File Browser**

**Storyboard Components:**
- Tree view of directories
- File metadata
- Upload/download

**Backend Commands:**
```typescript
const files = await invoke('core_storage_list', { entityId, path });
const content = await invoke('core_storage_read', { entityId, path });
await invoke('core_storage_write', { entityId, path, content });
await invoke('core_storage_mkdir', { entityId, path });
await invoke('core_storage_fs_delete', { entityId, path });
```

**TUI Implementation:**
- Use `ratatui::Tree` for directory structure
- File viewer for text files
- Progress bars for transfers

#### **4.2 Storage Settings**

**Storyboard Components:**
- Encryption type selector
- Replication factor
- Sync settings

**Backend Commands:**
```typescript
const vaultInfo = await invoke('core_storage_get_vault_info', { entityId });
await invoke('core_storage_update_vault_settings', {
  entityId,
  encryptionType,
  replicationFactor
}); // NEW COMMAND NEEDED
```

**TUI Implementation:**
- Settings form with `ratatui::Input` and `ratatui::Radio`

---

## 🏗️ Implementation Strategy

### **Phase 1: Backend Command Implementation (Days 1-5)**

**Goal**: Add all missing backend commands identified above.

**Tasks:**
1. Create new module `communitas-desktop/src/org_commands.rs` (DONE - already exists!)
2. Add missing commands:
   - `core_org_create`
   - `core_org_list`
   - `core_org_get`
   - `core_org_list_members`
   - `core_storage_get_vault_info`
   - `core_search_entities`
3. Update `communitas-core` with corresponding implementations
4. Add tests for each new command

**Files to Modify:**
- `communitas-desktop/src/main.rs` - Add to `generate_handler!`
- `communitas-core/src/core_context.rs` - Add methods
- `communitas-core/src/types.rs` - Add types

### **Phase 2: Tauri Frontend Wiring (Days 6-15)**

**Goal**: Wire up all React components with backend commands.

**Tasks:**
1. **Entity Sidebar** (`src/components/shell/EntitySidebar.tsx` - DOESN'T EXIST YET)
   - Create component
   - Wire up organization tree
   - Add filtering
   - Add search

2. **Organization Dashboard** (`src/components/organization/OrgDashboard.tsx` - DOESN'T EXIST YET)
   - Create 2x2 grid layout
   - Wire up member cards
   - Wire up storage meters
   - Wire up channel/project lists

3. **Chat Interface** (EXISTS: `src/components/chat/ChatInterface.tsx`)
   - Review and update message rendering
   - Add thread support
   - Add reactions UI
   - Add file attachments

4. **Storage Browser** (EXISTS: `src/components/storage/FileManager.tsx`)
   - Review and update file tree
   - Add upload/download UI
   - Add storage meters

5. **Settings Interfaces** (EXISTS: `src/components/settings/SettingsInterface.tsx`)
   - Review and update vault settings
   - Add replication settings

**New Components Needed:**
```
src/components/
├── shell/
│   ├── AppShell.tsx                    # NEW - Main layout
│   ├── EntitySidebar.tsx               # NEW - Left panel
│   ├── MainContent.tsx                 # NEW - Center area
│   └── InfoPanel.tsx                   # NEW - Right panel (optional)
├── organization/
│   ├── OrgDashboard.tsx                # NEW - Org overview grid
│   ├── OrgTree.tsx                     # NEW - Expandable tree
│   ├── MemberCard.tsx                  # NEW - Member display
│   └── ProjectCard.tsx                 # NEW - Project display
├── storage/
│   ├── StorageMeter.tsx                # NEW - Progress bar
│   └── VaultSettings.tsx               # NEW - Vault config
└── common/
    ├── FilterChip.tsx                  # NEW - Filter buttons
    └── Avatar.tsx                      # NEW - User avatars
```

### **Phase 3: TUI Implementation (Days 16-25)**

**Goal**: Implement TUI versions of all storyboard components.

**Tasks:**
1. **Refactor TUI Structure**
   - Create modular UI components
   - Add state management
   - Add keyboard navigation

2. **Implement TUI Screens**
   - Entity Sidebar (left 1/4 of screen)
   - Main Content (center 1/2 of screen)
   - Info Panel (right 1/4 of screen, toggleable)

3. **Add TUI Features**
   - Organization tree navigation
   - Chat message rendering
   - File browser
   - Storage meters
   - Member list

**Files to Create/Modify:**
```
communitas-tui/src/
├── ui/
│   ├── shell.rs                        # NEW - Main layout
│   ├── sidebar.rs                      # NEW - Entity sidebar
│   ├── org_dashboard.rs                # NEW - Org view
│   ├── chat.rs                         # UPDATE - Chat interface
│   ├── storage.rs                      # NEW - File browser
│   └── components/
│       ├── tree.rs                     # NEW - Tree widget
│       ├── member_list.rs              # NEW - Member list
│       └── storage_meter.rs            # NEW - Progress bar
└── handlers/
    ├── org.rs                          # NEW - Org handlers
    ├── chat.rs                         # UPDATE - Chat handlers
    └── storage.rs                      # NEW - Storage handlers
```

### **Phase 4: Testing & Polish (Days 26-30)**

**Goal**: End-to-end testing and UI polish.

**Tasks:**
1. **E2E Testing**
   - Test all storyboard flows in Tauri
   - Test all storyboard flows in TUI
   - Test P2P connectivity
   - Test offline operation

2. **UI Polish**
   - Match storyboard design exactly
   - Add loading states
   - Add error handling
   - Add animations (Tauri only)

3. **Performance Testing**
   - Test with large datasets
   - Test network latency handling
   - Test storage operations

4. **Documentation**
   - Update AGENTS_API.md with new commands
   - Add component documentation
   - Add testing guide

---

## 📊 Component-to-Backend Matrix

| Storyboard Component | Backend Command(s) | Status | Priority |
|---------------------|-------------------|--------|----------|
| **Identity Selector** | `core_get_user_info`, `get_user_four_words` | ✅ Exists | P1 |
| **Organization Tree** | `core_org_list` | ❌ Missing | P1 |
| **Filter Chips** | Client-side filtering | ✅ N/A | P1 |
| **Search Bar** | `core_search_entities` | ❌ Missing | P1 |
| **Org Dashboard** | `core_org_get`, `core_org_list_members` | ❌ Missing | P2 |
| **Member Cards** | `core_org_list_members`, `gossip_is_peer_online` | ⚠️ Partial | P2 |
| **Storage Meters** | `core_storage_get_vault_info`, `core_storage_stats` | ⚠️ Partial | P2 |
| **Channel List** | `list_channels` | ✅ Exists | P2 |
| **Project List** | `list_projects` | ✅ Exists | P2 |
| **Chat Interface** | `get_messages`, `send_message` | ✅ Exists | P3 |
| **Thread View** | `create_thread`, `get_thread_replies` | ✅ Exists | P3 |
| **Reactions** | `core_add_reaction` | ✅ Exists | P3 |
| **File Browser** | `core_storage_list`, `core_storage_read`, `core_storage_write` | ✅ Exists | P4 |
| **Storage Settings** | `core_storage_update_vault_settings` | ❌ Missing | P4 |

**Legend:**
- ✅ Exists - Command implemented and tested
- ⚠️ Partial - Command exists but needs enhancement
- ❌ Missing - Command needs implementation

---

## 🚀 Next Steps

### Immediate Actions (Today)

1. **Create Missing Backend Commands**
   - Start with `core_org_create`, `core_org_list`, `core_org_get`
   - Add tests for each command
   - Update `main.rs` to include new commands

2. **Create Storyboard React Components**
   - Start with `AppShell.tsx` and basic layout
   - Add `EntitySidebar.tsx` with mock data
   - Add `OrgDashboard.tsx` grid layout

3. **Update TUI Structure**
   - Refactor `communitas-tui` to match storyboard layout
   - Add modular UI components

### This Week

- Complete all missing backend commands
- Wire up 50% of Tauri components
- Start TUI refactoring

### Next Week

- Complete Tauri component wiring
- Complete TUI implementation
- Start E2E testing

### Week 3-4

- Polish UI/UX
- Performance testing
- Documentation updates

---

## 📝 Notes

### Architecture Decisions

1. **Backend Command Organization**: All organization-related commands go in `org_commands.rs`, storage in `storage_fs.rs`, etc.

2. **State Management**: Use Tauri's managed state for CoreContext, avoid global state where possible.

3. **TUI Modularity**: Each UI component is a separate module with its own state and rendering logic.

4. **Error Handling**: All backend commands return `Result<T, String>` for consistent error handling.

5. **Testing Strategy**: Test backend commands with `cargo test`, frontend with MCP/Chrome DevTools, TUI manually.

### Dependencies

**Tauri App:**
- Material-UI (existing)
- React Router (existing)
- Tauri v2 (existing)

**TUI App:**
- ratatui 0.26+ (existing)
- crossterm (existing)
- tokio (existing)

### Potential Issues

1. **Performance**: Large message lists may need pagination
2. **Network Latency**: Need loading states for all async operations
3. **Error Recovery**: Need retry logic for network failures
4. **State Sync**: Ensure Tauri and TUI state stay synchronized with backend

---

**End of Implementation Plan**

This plan will be updated as implementation progresses. Track progress in `STORYBOARD_V2.md` checklist.

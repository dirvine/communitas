# MCP Thin GUI Architecture Plan

**Version**: 1.0
**Status**: Planning
**Last Updated**: 2026-01-02

## Overview

This document defines the architecture where **communitas-mcp** is the complete backend API and GUI apps (Flutter, Swift) are thin presentation layers that only call MCP tools.

### Architecture Principles

1. **MCP is the single source of truth** - All business logic lives in Rust, exposed via MCP tools
2. **GUIs are dumb clients** - Flutter/Swift apps only handle:
   - UI rendering
   - User input capture
   - MCP tool invocation
   - Response display
3. **No business logic in GUI** - No data transformations, no state management beyond UI state
4. **Same API for all platforms** - Flutter, Swift, and AI agents all use identical MCP interface

---

## Current MCP Tool Coverage

### Auth & Identity (5 tools) - COMPLETE
| Tool | Status | Description |
|------|--------|-------------|
| `authenticate` | OK | Login with 4-word + password |
| `create_vault` | OK | Create new identity |
| `authenticate_token` | OK | AI agent token auth |
| `health_check` | OK | Service health check |
| `core_status` | OK | Core initialization status |

### Entity Management (5 tools) - COMPLETE
| Tool | Status | Description |
|------|--------|-------------|
| `create_entity` | OK | Create org/project/group/channel |
| `update_entity` | OK | Update name/description |
| `delete_entity` | OK | Delete entity |
| `get_entity` | OK | Get entity details |
| `list_entities` | OK | List all or by type |

### Member Management (3 tools) - COMPLETE
| Tool | Status | Description |
|------|--------|-------------|
| `add_member` | OK | Add member to entity |
| `remove_member` | OK | Remove member from entity |
| `list_members` | OK | List entity members |

### Messaging (3 tools) - PARTIAL
| Tool | Status | Description |
|------|--------|-------------|
| `send_message` | OK | Send message to entity |
| `delete_message` | OK | Delete a message |
| `get_messages` | OK | Get entity messages |
| `edit_message` | MISSING | Edit message content |
| `add_reaction` | MISSING | Add emoji reaction |
| `remove_reaction` | MISSING | Remove emoji reaction |

### Kanban (22 tools) - COMPLETE
All board, column, card, tag, step, comment, and assignment tools implemented.

### File Storage (6 tools) - PARTIAL
| Tool | Status | Description |
|------|--------|-------------|
| `write_file` | OK | Write file to disk |
| `read_file` | OK | Read file from disk |
| `delete_file` | OK | Delete file |
| `list_files` | OK | List directory contents |
| `get_disk_stats` | OK | Get disk usage |
| `create_directory` | MISSING | Create folder |
| `move_file` | MISSING | Move/rename file |
| `copy_file` | MISSING | Copy file |

### Network (7 tools) - COMPLETE
All networking tools implemented.

### Contacts (10 tools) - COMPLETE
All contact CRUD, favorites, and search tools implemented.

### Website (4 tools) - COMPLETE
All website publishing tools implemented.

### Invites (3 tools) - PARTIAL
| Tool | Status | Description |
|------|--------|-------------|
| `create_invite` | OK | Create invitation |
| `accept_invite` | OK | Accept invitation |
| `list_pending_invites` | OK | List pending invites |
| `decline_invite` | MISSING | Decline invitation |
| `cancel_invite` | MISSING | Cancel sent invite |
| `list_sent_invites` | MISSING | List invites I sent |

### Threads (2 tools) - COMPLETE
| Tool | Status | Description |
|------|--------|-------------|
| `create_thread` | OK | Create thread from message |
| `get_thread_messages` | OK | Get thread replies |

### Token Management (1 tool) - COMPLETE
| Tool | Status | Description |
|------|--------|-------------|
| `create_delegate_token` | OK | Create AI agent token |

---

## Missing Tools for Thin GUI

### Priority 1: Core User Experience

#### 1.1 Profile Management
```
update_profile          - Change display name, avatar
get_vault_list          - List available vaults (for login screen)
delete_vault            - Remove local vault
export_vault            - Export identity backup
import_vault            - Import identity from backup
```

#### 1.2 Message Editing & Reactions
```
edit_message            - Edit message content
add_reaction            - Add emoji reaction to message
remove_reaction         - Remove emoji reaction
get_reactions           - Get all reactions for message
```

#### 1.3 Invite Completeness
```
decline_invite          - Decline received invite
cancel_invite           - Cancel sent invite
list_sent_invites       - List invites I sent
```

### Priority 2: File Operations

```
create_directory        - Create folder
move_file               - Move or rename file
copy_file               - Copy file to new location
get_file_info           - Get file metadata (size, modified, etc.)
```

### Priority 3: Voice/Video Calls

```
# Call lifecycle
initiate_call           - Start outgoing call
answer_call             - Accept incoming call
decline_call            - Reject incoming call
end_call                - Hang up active call
get_call_state          - Get current call status

# Call controls
mute_audio              - Toggle microphone
toggle_video            - Toggle camera
share_screen            - Start/stop screen share
set_audio_device        - Select microphone/speaker
set_video_device        - Select camera

# Call events (via subscription/polling)
list_incoming_calls     - Check for incoming calls
get_call_participants   - List call participants
```

### Priority 4: Presence & Status

```
set_presence            - Set online/away/dnd/invisible
get_presence            - Get user's presence
get_contact_presences   - Get all contact presences
subscribe_presence      - Subscribe to presence changes
```

### Priority 5: Notifications

```
list_notifications      - Get notification list
mark_notification_read  - Mark as read
mark_all_read           - Mark all notifications read
clear_notification      - Delete notification
get_unread_count        - Get unread count
```

### Priority 6: Search

```
search_messages         - Full-text message search
search_entities         - Search entities by name
search_files            - Search files by name/content
search_global           - Search everything
```

### Priority 7: Settings & Preferences

```
get_settings            - Get user preferences
update_settings         - Update preferences
get_app_settings        - Get app-level settings (theme, etc.)
update_app_settings     - Update app settings
```

---

## Implementation Plan

### Phase 1: Message Completeness (Est: 1 day)
- [ ] `edit_message` - Add to Command enum and tools.rs
- [ ] `add_reaction` - Requires reaction support in CRDT
- [ ] `remove_reaction`
- [ ] `get_reactions`

### Phase 2: Profile & Vault Management (Est: 1 day)
- [ ] `update_profile`
- [ ] `get_vault_list`
- [ ] `delete_vault`
- [ ] `export_vault`
- [ ] `import_vault`

### Phase 3: File Operations (Est: 0.5 day)
- [ ] `create_directory`
- [ ] `move_file`
- [ ] `copy_file`
- [ ] `get_file_info`

### Phase 4: Invite Completeness (Est: 0.5 day)
- [ ] `decline_invite`
- [ ] `cancel_invite`
- [ ] `list_sent_invites`

### Phase 5: Presence System (Est: 1 day)
- [ ] `set_presence`
- [ ] `get_presence`
- [ ] `get_contact_presences`
- Requires presence tracking in gossip layer

### Phase 6: Notifications (Est: 1 day)
- [ ] Notification storage model
- [ ] `list_notifications`
- [ ] `mark_notification_read`
- [ ] `get_unread_count`

### Phase 7: Search (Est: 1 day)
- [ ] `search_messages`
- [ ] `search_entities`
- [ ] `search_files`
- Requires indexing infrastructure

### Phase 8: Voice/Video Calls (Est: 3+ days)
- Requires WebRTC integration
- Complex state machine
- Separate design document needed

### Phase 9: Settings (Est: 0.5 day)
- [ ] `get_settings`
- [ ] `update_settings`

---

## Thin GUI Implementation Guidelines

### Flutter App Structure

```
lib/
  src/
    services/
      mcp_client.dart       # Single MCP connection manager
    providers/
      auth_provider.dart    # Calls authenticate, create_vault
      entity_provider.dart  # Calls entity tools
      message_provider.dart # Calls message tools
      ...
    features/
      auth/                 # UI only - calls providers
      home/
      messaging/
      kanban/
      ...
```

### Provider Pattern

```dart
class EntityProvider extends ChangeNotifier {
  final McpClient _mcp;
  
  List<Entity> _entities = [];
  bool _loading = false;
  
  Future<void> loadEntities() async {
    _loading = true;
    notifyListeners();
    
    final result = await _mcp.call('list_entities', {});
    _entities = parseEntities(result);
    
    _loading = false;
    notifyListeners();
  }
  
  Future<void> createEntity(String name, String type) async {
    final result = await _mcp.call('create_entity', {
      'name': name,
      'entity_type': type,
    });
    await loadEntities(); // Refresh list
  }
}
```

### Swift App Structure

```
CommunitasApp/
  Services/
    MCPClient.swift         # Single MCP connection manager
  ViewModels/
    AuthViewModel.swift     # Calls authenticate, create_vault
    EntityViewModel.swift   # Calls entity tools
    MessageViewModel.swift  # Calls message tools
    ...
  Views/
    AuthViews/              # UI only - binds to ViewModels
    HomeViews/
    MessagingViews/
    KanbanViews/
    ...
```

---

## Testing Strategy

### MCP Tool Tests
- Each tool has integration test in `communitas-mcp/tests/`
- Tests verify JSON schema, auth requirements, error cases

### GUI Tests
- Mock MCP client for unit tests
- E2E tests launch actual MCP server
- Test tool calls, not business logic

---

## Migration Path

1. **Audit existing Flutter code** - Remove any business logic
2. **Implement missing MCP tools** - Phase 1-4 first
3. **Update Flutter providers** - Replace local logic with MCP calls
4. **Add Swift bindings** - Same MCP client pattern
5. **E2E testing** - Verify both GUIs work identically

---

## Open Questions

1. **Real-time updates**: How does GUI receive push updates?
   - Option A: Polling (simple but inefficient)
   - Option B: MCP subscriptions (if supported)
   - Option C: Separate WebSocket channel

2. **Call signaling**: WebRTC requires signaling server
   - Option A: Build into MCP
   - Option B: Separate signaling service
   - Option C: P2P signaling via gossip

3. **Large file handling**: Base64 in JSON is inefficient
   - Option A: Chunked transfer
   - Option B: Separate binary endpoint
   - Option C: File references with streaming

---

*This document will be updated as implementation progresses.*

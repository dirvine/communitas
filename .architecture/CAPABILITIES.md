# Communitas Capabilities

## Overview

This document extracts all user-visible actions from the Dioxus app and expresses them as formal **capabilities**. These capabilities can be used by:

1. **Dioxus app** (today)
2. **MCP agents** (today)
3. **Saorsa Canvas** (future)
4. **Third-party integrations** (future)

**Principle**: Every user action is a capability. Capabilities are the unit of authorization.

---

## Capability Format

```rust
pub struct Capability {
    pub id: String,                    // "domain.action"
    pub name: String,                  // Human-readable
    pub description: String,           // What it does
    pub input_schema: JsonSchema,      // Required parameters
    pub output_schema: JsonSchema,     // Return type
    pub required_role: Role,           // Minimum permission
    pub audit_level: AuditLevel,       // Logging requirement
    pub reversible: bool,              // Can be undone
    pub offline_capable: bool,         // Works offline
    pub crdt_operation: bool,          // Modifies CRDT state
}
```

---

## 1. Authentication Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `auth.login` | Login to vault | None | Yes | No |
| `auth.logout` | Logout session | Authenticated | Yes | No |
| `auth.create_vault` | Create new identity | None | Yes | No |
| `auth.recover_vault` | Recover from mnemonic | None | Yes | No |
| `auth.switch_identity` | Switch active vault | None | Yes | No |
| `auth.register_passkey` | Add biometric auth | Authenticated | No | No |
| `auth.remove_passkey` | Remove passkey | Authenticated | No | No |

### Detailed Schema: `auth.login`

```json
{
  "id": "auth.login",
  "input_schema": {
    "type": "object",
    "required": ["four_words", "password"],
    "properties": {
      "four_words": { "type": "string", "pattern": "^[a-z]+-[a-z]+-[a-z]+-[a-z]+$" },
      "password": { "type": "string", "minLength": 1 }
    }
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "session_id": { "type": "string", "format": "uuid" },
      "display_name": { "type": "string" },
      "public_key_hex": { "type": "string" }
    }
  },
  "required_role": "none",
  "audit_level": "always",
  "reversible": false,
  "offline_capable": true
}
```

---

## 2. Messaging Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `messaging.send` | Send message | Member | Yes | Yes |
| `messaging.edit` | Edit message | Author | Yes | Yes |
| `messaging.delete` | Delete message | Author/Admin | Yes | Yes |
| `messaging.react` | Add reaction | Member | Yes | Yes |
| `messaging.unreact` | Remove reaction | Member | Yes | Yes |
| `messaging.pin` | Pin message | Admin | Yes | Yes |
| `messaging.unpin` | Unpin message | Admin | Yes | Yes |
| `messaging.list` | List messages | Member | Yes | No |
| `messaging.search` | Search messages | Member | Yes | No |
| `messaging.mark_read` | Mark as read | Member | Yes | Yes |
| `messaging.typing` | Send typing indicator | Member | No | No |

### Detailed Schema: `messaging.send`

```json
{
  "id": "messaging.send",
  "input_schema": {
    "type": "object",
    "required": ["entity_id", "content"],
    "properties": {
      "entity_id": { "type": "string", "format": "uuid" },
      "content": { "type": "string", "maxLength": 65536 },
      "reply_to": { "type": "string", "format": "uuid" },
      "attachments": {
        "type": "array",
        "items": { "$ref": "#/definitions/attachment" }
      }
    }
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "message_id": { "type": "string", "format": "uuid" },
      "timestamp": { "type": "string", "format": "date-time" }
    }
  },
  "required_role": "member",
  "audit_level": "standard",
  "reversible": true,
  "offline_capable": true,
  "crdt_operation": true
}
```

---

## 3. Entity Management Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `entity.create_org` | Create organization | Authenticated | Yes | Yes |
| `entity.create_project` | Create project | OrgAdmin | Yes | Yes |
| `entity.create_group` | Create group | Authenticated | Yes | Yes |
| `entity.create_channel` | Create channel | ProjectAdmin | Yes | Yes |
| `entity.update` | Update entity | Admin | Yes | Yes |
| `entity.delete` | Delete entity | Owner | Yes | Yes |
| `entity.archive` | Archive entity | Admin | Yes | Yes |
| `entity.list` | List entities | Authenticated | Yes | No |
| `entity.get` | Get entity details | Member | Yes | No |

### Detailed Schema: `entity.create_org`

```json
{
  "id": "entity.create_org",
  "input_schema": {
    "type": "object",
    "required": ["name"],
    "properties": {
      "name": { "type": "string", "minLength": 1, "maxLength": 128 },
      "description": { "type": "string", "maxLength": 4096 },
      "category": { "enum": ["organization", "community"] },
      "visibility": { "enum": ["private", "public"] }
    }
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "entity_id": { "type": "string", "format": "uuid" },
      "entity_type": { "const": "organization" }
    }
  },
  "required_role": "authenticated",
  "audit_level": "always",
  "reversible": true,
  "offline_capable": true,
  "crdt_operation": true
}
```

---

## 4. Member Management Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `member.invite` | Invite member | Admin | No | Yes |
| `member.accept_invite` | Accept invitation | Invitee | No | Yes |
| `member.reject_invite` | Reject invitation | Invitee | No | Yes |
| `member.remove` | Remove member | Admin | Yes | Yes |
| `member.set_role` | Change member role | Owner | Yes | Yes |
| `member.list` | List members | Member | Yes | No |

---

## 5. Contact Management Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `contact.add` | Add contact | Authenticated | Yes | Yes |
| `contact.remove` | Remove contact | Authenticated | Yes | Yes |
| `contact.favorite` | Favorite contact | Authenticated | Yes | Yes |
| `contact.unfavorite` | Unfavorite contact | Authenticated | Yes | Yes |
| `contact.block` | Block contact | Authenticated | Yes | Yes |
| `contact.unblock` | Unblock contact | Authenticated | Yes | Yes |
| `contact.list` | List contacts | Authenticated | Yes | No |
| `contact.search` | Search contacts | Authenticated | No | No |

---

## 6. Kanban Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `kanban.create_board` | Create board | ProjectAdmin | Yes | Yes |
| `kanban.update_board` | Update board | BoardAdmin | Yes | Yes |
| `kanban.delete_board` | Delete board | ProjectAdmin | Yes | Yes |
| `kanban.create_column` | Create column | BoardAdmin | Yes | Yes |
| `kanban.update_column` | Update column | BoardAdmin | Yes | Yes |
| `kanban.delete_column` | Delete column | BoardAdmin | Yes | Yes |
| `kanban.reorder_columns` | Reorder columns | BoardAdmin | Yes | Yes |
| `kanban.create_card` | Create card | Member | Yes | Yes |
| `kanban.update_card` | Update card | Assignee/Admin | Yes | Yes |
| `kanban.delete_card` | Delete card | Admin | Yes | Yes |
| `kanban.move_card` | Move card | Assignee | Yes | Yes |
| `kanban.assign_card` | Assign card | Admin | Yes | Yes |
| `kanban.add_step` | Add checklist step | Assignee | Yes | Yes |
| `kanban.complete_step` | Complete step | Assignee | Yes | Yes |
| `kanban.add_comment` | Add comment | Member | Yes | Yes |
| `kanban.add_tag` | Add tag | Member | Yes | Yes |

### Detailed Schema: `kanban.move_card`

```json
{
  "id": "kanban.move_card",
  "input_schema": {
    "type": "object",
    "required": ["card_id", "target_column_id"],
    "properties": {
      "card_id": { "type": "string", "format": "uuid" },
      "target_column_id": { "type": "string", "format": "uuid" },
      "position": { "type": "integer", "minimum": 0 }
    }
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "success": { "type": "boolean" },
      "new_state": { "type": "string" }
    }
  },
  "required_role": "assignee",
  "audit_level": "standard",
  "reversible": true,
  "offline_capable": true,
  "crdt_operation": true
}
```

---

## 7. Drive (File) Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `drive.list` | List directory | Member | Yes | No |
| `drive.create_dir` | Create directory | Member | Yes | Yes |
| `drive.delete` | Delete file/dir | Owner/Admin | Yes | Yes |
| `drive.move` | Move file/dir | Owner | Yes | Yes |
| `drive.copy` | Copy file/dir | Member | Yes | Yes |
| `drive.upload` | Upload file | Member | Partial | Yes |
| `drive.download` | Download file | Member | Partial | No |
| `drive.preview` | Preview file | Member | Yes | No |
| `drive.share` | Create share link | Member | No | Yes |
| `drive.get_quota` | Get quota info | Member | Yes | No |

---

## 8. Canvas Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `canvas.add_element` | Add shape/text/image | Member | Yes | Yes |
| `canvas.update_element` | Update element | Member | Yes | Yes |
| `canvas.delete_element` | Delete element | Member | Yes | Yes |
| `canvas.transform` | Move/resize/rotate | Member | Yes | Yes |
| `canvas.undo` | Undo last action | Member | Yes | Yes |
| `canvas.redo` | Redo action | Member | Yes | Yes |
| `canvas.set_zoom` | Set zoom level | Member | Yes | No |
| `canvas.export` | Export canvas | Member | Yes | No |
| `canvas.import` | Import content | Member | Yes | Yes |
| `canvas.clear` | Clear canvas | Admin | Yes | Yes |

---

## 9. Call Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `call.start` | Start call | Member | No | No |
| `call.join` | Join call | Member | No | No |
| `call.leave` | Leave call | Participant | No | No |
| `call.end` | End call for all | Host | No | No |
| `call.mute` | Toggle mute | Participant | No | No |
| `call.toggle_video` | Toggle video | Participant | No | No |
| `call.share_screen` | Share screen | Participant | No | No |
| `call.stop_share` | Stop sharing | Sharer | No | No |
| `call.raise_hand` | Raise hand | Participant | No | No |
| `call.kick` | Remove participant | Host | No | No |
| `call.lock` | Lock call | Host | No | No |
| `call.record` | Start recording | Host | No | No |
| `call.stop_record` | Stop recording | Host | No | No |

---

## 10. Presence Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `presence.update` | Update status | Authenticated | No | No |
| `presence.get` | Get user status | Authenticated | Yes | No |
| `presence.subscribe` | Watch user status | Authenticated | No | No |

---

## 11. Network Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `network.start` | Start P2P networking | Authenticated | N/A | No |
| `network.stop` | Stop networking | Authenticated | N/A | No |
| `network.connect` | Connect to peer | Authenticated | N/A | No |
| `network.disconnect` | Disconnect peer | Authenticated | N/A | No |
| `network.status` | Get network status | Authenticated | Yes | No |
| `network.peers` | List connected peers | Authenticated | Yes | No |

---

## 12. Recovery Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `recovery.export_mnemonic` | Show recovery phrase | Authenticated | Yes | No |
| `recovery.verify_mnemonic` | Verify user knows phrase | Authenticated | Yes | No |
| `recovery.export_vault` | Export vault backup | Authenticated | Yes | No |
| `recovery.import_vault` | Import vault backup | None | Yes | No |

---

## 13. Settings Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `settings.update_profile` | Update display name | Authenticated | Yes | Yes |
| `settings.update_avatar` | Update avatar | Authenticated | Yes | Yes |
| `settings.get_preferences` | Get preferences | Authenticated | Yes | No |
| `settings.set_preferences` | Set preferences | Authenticated | Yes | Yes |
| `settings.get_devices` | List devices | Authenticated | Yes | No |
| `settings.revoke_device` | Revoke device | Authenticated | No | Yes |

---

## 14. Audit Capabilities

| ID | Name | Required Role | Offline | CRDT |
|----|------|---------------|---------|------|
| `audit.list_events` | List audit events | Admin | Yes | No |
| `audit.export` | Export audit log | Admin | Yes | No |

---

## Capability Role Hierarchy

```
Owner
  └── Admin
        └── Member
              └── Authenticated
                    └── None (pre-auth)
```

**Special Roles**:
- `Assignee`: Card assignee for kanban operations
- `Author`: Message author for edit/delete
- `Host`: Call host for moderation
- `Invitee`: Invitation recipient

---

## Audit Levels

| Level | When Logged | Retention |
|-------|-------------|-----------|
| `always` | Every invocation | Permanent |
| `standard` | Success and failure | 90 days |
| `minimal` | Failure only | 30 days |
| `none` | Never | N/A |

---

## CRDT Operations

All capabilities with `crdt_operation: true` modify Yrs documents and:

1. **Execute locally** immediately
2. **Queue for sync** if offline
3. **Merge automatically** when peers reconnect
4. **Generate deltas** for efficient replication

---

## MCP Tool Mapping

| Capability ID | MCP Tool Name | Status |
|---------------|---------------|--------|
| `messaging.send` | `send_message` | **EXISTS** |
| `messaging.list` | `list_messages` | **EXISTS** |
| `entity.create_org` | `create_entity` | **EXISTS** |
| `kanban.create_card` | `create_card` | **EXISTS** |
| `drive.upload` | `upload_file` | **EXISTS** |
| ... | ... | ... |

**All 120+ MCP tools map 1:1 to capabilities.**

---

## Future: Capability Discovery

```rust
// Canvas/Agent can discover available capabilities
let registry = CapabilityRegistry::global();

// List all capabilities user can invoke
let available = registry.list_for_principal(&agent_principal);

// Get schema for a specific capability
let schema = registry.get_input_schema("messaging.send")?;

// Validate input before invocation
registry.validate("messaging.send", &args)?;
```

This enables:
- **Dynamic UI generation** from capability schemas
- **Agent capability discovery** without hardcoding
- **Cross-version compatibility** via schema negotiation

# ADR-018: MCP External Integration

**Status**: Accepted
**Date**: 2026-01-19
**Authors**: Communitas Team

## Context

Communitas aims to provide first-class support for AI agents through the Model Context Protocol (MCP). This ADR documents the MCP tool inventory and parity strategy between the Dioxus UI and MCP automation interface.

AI agents need the same capabilities as human users to effectively assist with collaboration tasks. This includes:
- Authentication and session management
- Entity and contact directory access
- Messaging (threads, messages, reactions)
- Presence status management
- Kanban project management
- File operations

## Decision

### MCP Transport Options

The MCP server (`communitas-mcp`) supports two transport modes:

1. **stdio** (default): Standard input/output for local AI agent integration
2. **HTTP/HTTPS**: JSON-RPC over HTTP for remote integration (with `--http` flag)

### Authentication Model

Tools are divided into two categories:

1. **Pre-auth tools**: Available without authentication
2. **Authenticated tools**: Require valid session

Authentication methods:
- `authenticate`: Four-word identity + password
- `authenticate_token`: Delegate token (for AI agents)
- `create_vault`: New identity creation
- `recover_identity`: BIP39 mnemonic recovery

---

## Tool Inventory

### Pre-Authentication Tools

| Tool | Description | Required Params |
|------|-------------|-----------------|
| `authenticate` | Login with four-word identity | `four_words`, `password` |
| `authenticate_token` | Login with delegate token | `token` |
| `create_vault` | Create new identity vault | `four_words`, `password`, `display_name` |
| `list_vaults` | List available vaults | - |
| `delete_vault` | Delete a vault | `four_words`, `password` |
| `import_vault` | Import from backup | `backup_data`, `password` |
| `create_identity` | Create identity with BIP39 | `word_count` (optional) |
| `recover_identity` | Recover from mnemonic | `mnemonic_words` |
| `validate_mnemonic` | Validate BIP39 phrase | `mnemonic_words` |
| `health_check` | Service health status | - |
| `core_status` | Core initialization status | - |

### Session Tools

| Tool | Description | Required Params |
|------|-------------|-----------------|
| `get_session` | Get current session info | - |
| `logout` | End session | - |

### Entity Tools

| Tool | Description | Required Params |
|------|-------------|-----------------|
| `create_entity` | Create org/project/group/channel | `name`, `entity_type` |
| `update_entity` | Update entity details | `entity_type`, `entity_id` |
| `delete_entity` | Delete an entity | `entity_type`, `entity_id` |
| `add_member` | Add member to entity | `entity_type`, `entity_id`, `member_id`, `role` |
| `remove_member` | Remove member | `entity_type`, `entity_id`, `member_id` |

### Messaging Tools

| Tool | Category | Description | Required Params |
|------|----------|-------------|-----------------|
| `list_threads` | Messaging | List conversation threads with filters | - |
| `list_messages` | Messaging | Get messages from thread with pagination | `thread_id` |
| `send_message` | Messaging | Send a message | `entity_id`, `entity_type`, `text` |
| `edit_message` | Messaging | Edit message text | `entity_id`, `entity_type`, `message_id`, `new_text` |
| `delete_message` | Messaging | Delete a message | `entity_id`, `entity_type`, `message_id` |
| `add_reaction` | Reactions | Add emoji reaction | `entity_id`, `entity_type`, `message_id`, `emoji` |
| `remove_reaction` | Reactions | Remove reaction | `entity_id`, `entity_type`, `message_id`, `emoji` |
| `get_reactions` | Reactions | Get message reactions | `entity_id`, `message_id` |
| `get_available_reactions` | Reactions | List available emojis | `entity_id` |

**Thread ID format**: `entity:{entity_id}` or `contact:{contact_id}`

**Pagination**: `list_messages` supports `limit` (max 100) and `before` (Unix timestamp ms)

**Filters for `list_threads`**:
- `all` (default): All threads
- `unread`: Only unread threads
- `entities`: Only entity threads (channels, groups)
- `contacts`: Only direct message threads

### Contact Tools

| Tool | Category | Description | Required Params |
|------|----------|-------------|-----------------|
| `list_contacts` | Contacts | List contacts with optional presence | - |
| `get_contact_presence` | Presence | Get presence for specific contact | `contact_id` |
| `set_my_presence` | Presence | Set own presence status | `status` |
| `link_contact` | Contacts | Add contact to address book | `contact_id` |
| `set_favourite_contact` | Contacts | Mark as favourite | `contact_id` |
| `remove_favourite_contact` | Contacts | Remove favourite | `contact_id` |
| `list_favourite_contacts` | Contacts | List favourites | - |
| `search_contacts` | Contacts | Search by name/four-words | `query` |

**`list_contacts` options**:
- `include_presence`: boolean (default: true)
- `filter`: `all` | `favorites` | `online`

**`set_my_presence` status values**:
- `online`: Active and available
- `away`: Temporarily away
- `busy`: Do not disturb
- `offline`: Appear offline

### Kanban Tools

| Tool | Description | Required Params |
|------|-------------|-----------------|
| `create_kanban_board` | Create board | `entity_id`, `board_name` |
| `update_kanban_board` | Update board | `board_id` |
| `get_kanban_board` | Get board details | `board_id` |
| `list_kanban_boards` | List entity boards | `entity_id` |
| `create_kanban_column` | Create column | `board_id`, `column_name` |
| `create_kanban_card` | Create card | `board_id`, `column_id`, `title` |
| `update_kanban_card` | Update card | `board_id`, `card_id` |
| `move_kanban_card` | Move card | `board_id`, `card_id`, `target_column_id` |
| `delete_kanban_card` | Delete card | `board_id`, `card_id` |
| `get_kanban_card` | Get card details | `board_id`, `card_id` |

### File Tools

| Tool | Description | Required Params |
|------|-------------|-----------------|
| `create_file` | Create new file | `entity_id`, `disk_type`, `path`, `content` |
| `read_file` | Read file content | `entity_id`, `disk_type`, `path` |
| `update_file` | Update file | `entity_id`, `disk_type`, `path`, `content` |
| `delete_file` | Delete file | `entity_id`, `disk_type`, `path` |
| `list_directory` | List directory | `entity_id`, `disk_type`, `path` |

**Disk types**: `private`, `public`, `shared`

### Network Tools

| Tool | Description | Required Params |
|------|-------------|-----------------|
| `get_peer_info` | Get peer information | - |
| `dial_peer` | Connect to peer | `four_words` |
| `connected_peers` | List connected peers | - |

---

## Capability Parity Matrix

Ensuring MCP tools provide equivalent functionality to Dioxus UI components:

| Capability | Dioxus UI Component | MCP Tools | Parity Status |
|------------|---------------------|-----------|---------------|
| **Authentication** | LoginRoute, CreateIdentityRoute | `authenticate`, `create_identity` | Full |
| **Directory browsing** | DirectorySidebar | `list_entities` (implicit via queries) | Full |
| **Thread listing** | ThreadListSidebar | `list_threads` | Full |
| **Thread filtering** | Filter tabs (All/Entities/Contacts/Unread) | `list_threads` filter param | Full |
| **Message viewing** | MessageList | `list_messages` | Full |
| **Message sending** | MessageComposer | `send_message` | Full |
| **Message editing** | (inline edit) | `edit_message` | Full |
| **Reactions** | Reaction picker | `add_reaction`, `remove_reaction` | Full |
| **Presence display** | PresenceBadge, PresenceDot | `list_contacts`, `get_contact_presence` | Full |
| **Set presence** | (implicit online) | `set_my_presence` | MCP-only explicit control |
| **Contact management** | ContactDetailView | `link_contact`, `set_favourite_contact` | Full |
| **Kanban boards** | (future Milestone 3) | `create_kanban_board`, etc. | Full |
| **File operations** | (future Milestone 3) | `create_file`, `read_file`, etc. | Full |

---

## Parity Testing Approach

### Test Harnesses

Two automated test harnesses ensure MCP and Dioxus share identical data:

1. **`scripts/tests/mcp_messaging.sh`**: Tests messaging tool parity
   - Compares `list_threads` output against `export_threads` binary
   - Compares `list_contacts` output against `export_contacts` binary
   - Tests filtering consistency
   - Archives JSON artifacts for CI

2. **`scripts/tests/mcp_nav_auth.sh`**: Tests navigation/auth parity
   - Compares directory snapshots
   - Tests authentication flows
   - Validates session management

### JSON Schema Validation

Both harnesses validate responses against expected schemas:

**ThreadSummary schema**:
```json
{
  "id": "entity:channel/123",
  "display_name": "General Chat",
  "entity_id": "channel/123",
  "contact_id": null,
  "last_message_text": "Hello world",
  "last_message_at": 1705689600000,
  "unread_count": 3
}
```

**ContactWithPresence schema**:
```json
{
  "id": "abc123",
  "display_name": "Alice",
  "four_words": "ocean.forest.moon.star",
  "is_favourite": true,
  "is_online": true,
  "last_seen": 1705689600000,
  "presence_status": "online"
}
```

### CI Integration

Parity tests run in GitHub Actions:
- Linux only (MCP server requires system dependencies)
- Artifacts uploaded for debugging
- Failures block merge

---

## Tracing and Observability

All MCP tools are instrumented with tracing spans:

- `mcp.tools.list_threads`
- `mcp.tools.list_messages`
- `mcp.tools.list_contacts`
- `mcp.tools.get_contact_presence`
- `mcp.tools.set_my_presence`
- etc.

Spans include:
- Tool name
- Request parameters (sanitized)
- Response status
- Duration

---

## Consequences

### Positive

1. **AI agent enablement**: Agents can perform all user tasks programmatically
2. **Automation parity**: Same data model for UI and automation
3. **Testability**: JSON artifacts enable diff-based validation
4. **Observability**: Tracing spans provide debugging insight

### Negative

1. **Maintenance burden**: Two interfaces to keep synchronized
2. **Schema evolution**: Changes must update both UI and MCP

### Mitigations

- Shared `communitas-ui-api` types for both interfaces
- Automated parity tests catch drift
- CI enforcement prevents regressions

---

## References

- [MCP Specification](https://modelcontextprotocol.io/specification)
- `communitas-mcp/src/tools.rs` - Tool definitions
- `scripts/tests/mcp_messaging.sh` - Messaging parity harness
- `docs/testing/mcp_messaging_parity.md` - Parity test documentation

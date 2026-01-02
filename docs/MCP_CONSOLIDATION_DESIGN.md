# MCP Consolidation Design Document

**Status**: LOCKED (Phase 1 Complete)
**Date**: 2026-01-02 (Updated)
**Epic**: Bridge-to-MCP Consolidation

## Executive Summary

Consolidate all communitas-bridge functionality into communitas-mcp, making MCP the single core application interface. HTTP transport will be integrated directly into MCP (not kept as separate proxy). Focus on AI agent workflows for E2E testing.

## Phase 1 Complete: Gap Analysis & Tool Parity ✅

After detailed code review of both `communitas-mcp/src/tools.rs` (70+ tools) and `communitas-bridge/src/handlers.rs` (74 endpoints), all gaps have been filled.

### Already Complete in MCP (No Action Needed)
- ✅ All entity operations (create, list, join)
- ✅ All member operations (add, remove, list)
- ✅ All messaging operations
- ✅ All thread operations
- ✅ All virtual disk storage operations
- ✅ All website publishing operations
- ✅ All contact management (MCP has MORE features)
- ✅ All Kanban columns, assignments, tags, steps, comments
- ✅ P2P network (start, connect, status, peers, stop)
- ✅ Invitations (MCP-only feature)
- ✅ Authentication (MCP-only feature)

### Gaps Filled (6 items) ✅

| Gap | Bridge Had | MCP Now Has | Status |
|-----|------------|-------------|--------|
| Health check | `GET /health` | `health_check` tool (pre-auth) | ✅ Done |
| Core status | `GET /api/core/status` | `core_status` tool (pre-auth) | ✅ Done |
| Disconnect peer | `POST /api/network/disconnect` | `network_disconnect` tool | ✅ Done |
| Update board | `PUT /api/boards/:id` | `update_kanban_board` tool | ✅ Done |
| Delete board | `DELETE /api/boards/:id` | `delete_kanban_board` tool | ✅ Done |
| List cards | `GET /api/boards/:id/cards` | `list_kanban_cards` tool | ✅ Done |

### MCP Already Ahead Of Bridge
- `network_stop` - Stop networking
- `network_request_external_address` - NAT reflection
- `update_contact`, `set_favourite_contact`, `remove_favourite_contact`, `search_contacts`
- Full invitation system
- Proper authentication with delegate tokens

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    NEW ARCHITECTURE                              │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────┐
                    │    communitas-mcp       │
                    │    (THE CORE APP)       │
                    │                         │
                    │  ┌───────────────────┐  │
                    │  │  CoreContext      │  │
                    │  │  (single owner)   │  │
                    │  └───────────────────┘  │
                    │                         │
                    │  ┌───────────────────┐  │
                    │  │  Auth Layer       │  │
                    │  │  (--demo flag)    │  │
                    │  └───────────────────┘  │
                    │                         │
                    │  ┌───────────────────┐  │
                    │  │  65 MCP Tools     │  │
                    │  │  (full feature)   │  │
                    │  └───────────────────┘  │
                    └──────────┬──────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
          ▼                    ▼                    ▼
   ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
   │    stdio     │    │   Bridge     │    │  Future:     │
   │  (MCP spec)  │    │ (HTTP wrap)  │    │  QUIC/IPC    │
   └──────────────┘    └──────────────┘    └──────────────┘
          │                    │
          ▼                    ▼
   ┌──────────────┐    ┌──────────────┐
   │  AI Agents   │    │  Web/Flutter │
   │  E2E Tests   │    │   Clients    │
   └──────────────┘    └──────────────┘
```

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Auth Model** | Dual mode (`--demo` flag) | Default requires auth; `--demo` for testing |
| **Transport** | Stdio only (phase 1) | Focus on features first |
| **Tool Design** | Granular (1 tool per op) | More discoverable for AI agents |
| **Binary Files** | Base64 encoding | Works with any file type in JSON |
| **Architecture** | Bridge wraps MCP | MCP is the core; Bridge is thin HTTP |
| **State Owner** | MCP owns CoreContext | Single source of truth |
| **Demo Network** | Auto-init | Network starts automatically in demo mode |

## Complete Tool Inventory (65 Tools)

### Category 1: Authentication (3 tools) - PRE-AUTH

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `authenticate` | `{four_words: string, password: string, device_name?: string}` | Login with credentials |
| `create_vault` | `{four_words: string, password: string, display_name: string}` | Create new identity |
| `authenticate_token` | `{token: string}` | Auth with delegate token |

### Category 2: Session (3 tools) - POST-AUTH

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `get_session` | `{}` | Get current session info |
| `create_delegate_token` | `{delegate_name: string, scopes: string[], expires_in_hours?: number}` | Create scoped token |
| `logout` | `{}` | End current session |

### Category 3: P2P Network (5 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `network_start` | `{bootstrap_addrs?: string[]}` | Start P2P networking |
| `network_stop` | `{}` | Stop P2P networking |
| `network_connect` | `{peer_addr: string}` | Connect to specific peer |
| `network_disconnect` | `{peer_id: string}` | Disconnect from peer |
| `network_status` | `{}` | Get peers, connection info |

### Category 4: Entity Management (6 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_entity` | `{name: string, entity_type: "organisation"\|"project"\|"group"\|"channel", description?: string}` | Create entity |
| `get_entity` | `{entity_id: string}` | Get entity details |
| `list_entities` | `{entity_type?: string}` | List all entities |
| `update_entity` | `{entity_id: string, name?: string, description?: string}` | Update entity |
| `delete_entity` | `{entity_id: string}` | Delete entity |
| `join_entity` | `{entity_id: string}` | Join existing entity |

### Category 5: Member Management (3 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `add_member` | `{entity_id: string, member_id: string, role?: string}` | Add member |
| `remove_member` | `{entity_id: string, member_id: string}` | Remove member |
| `list_members` | `{entity_id: string}` | List entity members |

### Category 6: Messaging (3 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `send_message` | `{entity_id: string, text: string, reply_to?: string}` | Send message |
| `get_messages` | `{entity_id: string, limit?: number, before?: string}` | Get messages |
| `delete_message` | `{message_id: string}` | Delete message |

### Category 7: Threads (3 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_thread` | `{parent_message_id: string, name?: string}` | Create thread |
| `get_thread_messages` | `{thread_id: string, limit?: number}` | Get thread replies |
| `reply_to_thread` | `{thread_id: string, text: string}` | Reply in thread |

### Category 8: Contacts (6 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_contact` | `{four_words: string, nickname?: string, is_favorite?: bool}` | Add contact |
| `get_contact` | `{contact_id: string}` | Get contact details |
| `list_contacts` | `{favorites_only?: bool}` | List contacts |
| `update_contact` | `{contact_id: string, nickname?: string, is_favorite?: bool}` | Update contact |
| `delete_contact` | `{contact_id: string}` | Delete contact |
| `link_contact` | `{contact_id: string, entity_id: string}` | Link to entity |

### Category 9: Virtual Disk Storage (6 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `write_file` | `{entity_id: string, disk_type: "private"\|"public"\|"shared", path: string, content: string, encoding?: "utf8"\|"base64"}` | Write file |
| `read_file` | `{entity_id: string, disk_type: string, path: string, encoding?: string}` | Read file |
| `list_files` | `{entity_id: string, disk_type: string, path?: string}` | List files |
| `delete_file` | `{entity_id: string, disk_type: string, path: string}` | Delete file |
| `get_disk_stats` | `{entity_id: string, disk_type: string}` | Get usage stats |
| `move_file` | `{entity_id: string, disk_type: string, from_path: string, to_path: string}` | Move/rename |

### Category 10: Website Publishing (4 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_website` | `{entity_id: string, content: string, title?: string}` | Create website |
| `get_website` | `{entity_id: string}` | Get website |
| `update_website` | `{entity_id: string, content?: string, title?: string}` | Update website |
| `delete_website` | `{entity_id: string}` | Delete website |

### Category 11: Kanban Boards (4 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_kanban_board` | `{project_id: string, name: string, description?: string}` | Create board |
| `get_kanban_board` | `{board_id: string}` | Get board with all data |
| `update_kanban_board` | `{board_id: string, name?: string, description?: string}` | Update board |
| `delete_kanban_board` | `{board_id: string}` | Delete board |

### Category 12: Kanban Columns (5 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_kanban_column` | `{board_id: string, name: string, position?: number}` | Create column |
| `get_kanban_column` | `{board_id: string, column_id: string}` | Get column |
| `update_kanban_column` | `{board_id: string, column_id: string, name?: string}` | Update column |
| `delete_kanban_column` | `{board_id: string, column_id: string}` | Delete column |
| `move_kanban_column` | `{board_id: string, column_id: string, position: number}` | Reorder |

### Category 13: Kanban Cards (6 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_kanban_card` | `{board_id: string, column_id: string, title: string, description?: string}` | Create card |
| `get_kanban_card` | `{board_id: string, card_id: string}` | Get card details |
| `update_kanban_card` | `{board_id: string, card_id: string, title?: string, description?: string}` | Update card |
| `delete_kanban_card` | `{board_id: string, card_id: string}` | Delete card |
| `move_kanban_card` | `{board_id: string, card_id: string, column_id: string, position?: number}` | Move card |
| `change_card_state` | `{board_id: string, card_id: string, state: string}` | Change state |

### Category 14: Kanban Assignments (2 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `assign_card` | `{board_id: string, card_id: string, user_id: string}` | Assign user |
| `unassign_card` | `{board_id: string, card_id: string, user_id: string}` | Remove assignment |

### Category 15: Kanban Tags (4 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_tag` | `{board_id: string, name: string, color?: string}` | Create tag |
| `list_tags` | `{board_id: string}` | List board tags |
| `tag_card` | `{board_id: string, card_id: string, tag_id: string}` | Add tag to card |
| `untag_card` | `{board_id: string, card_id: string, tag_id: string}` | Remove tag |

### Category 16: Kanban Steps/Checklist (4 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `add_card_step` | `{board_id: string, card_id: string, text: string}` | Add step |
| `toggle_card_step` | `{board_id: string, card_id: string, step_id: string}` | Toggle completion |
| `delete_card_step` | `{board_id: string, card_id: string, step_id: string}` | Delete step |
| `list_card_steps` | `{board_id: string, card_id: string}` | List steps |

### Category 17: Kanban Comments (3 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `add_card_comment` | `{board_id: string, card_id: string, text: string}` | Add comment |
| `list_card_comments` | `{board_id: string, card_id: string}` | List comments |
| `delete_card_comment` | `{board_id: string, card_id: string, comment_id: string}` | Delete comment |

### Category 18: Invitations (3 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `create_invite` | `{entity_id: string, invitee_id: string, message?: string}` | Send invite |
| `accept_invite` | `{invite_id: string}` | Accept invite |
| `list_pending_invites` | `{}` | List pending |

### Category 19: Profile (2 tools)

| Tool | Input Schema | Description |
|------|--------------|-------------|
| `get_profile` | `{}` | Get current user |
| `update_profile` | `{display_name?: string, avatar?: string}` | Update profile |

## MCP Resources (6 resources)

| Resource URI | Description |
|--------------|-------------|
| `communitas://identity` | Current user identity and profile |
| `communitas://entities` | All entities user has access to |
| `communitas://chats` | All chat-type entities (channels, groups) |
| `communitas://invites` | Pending invitations |
| `communitas://contacts` | All contacts |
| `communitas://network` | Network status and peer info |

## Revised Implementation Phases (Post Gap Analysis)

### Phase 1: Fill Remaining Tool Gaps (6 tools)
**Goal**: 100% feature parity with Bridge

| Tool | Description | Proof Point |
|------|-------------|-------------|
| `health_check` | Service health status | Unit test: returns ok/error status |
| `core_status` | Initialization status | Unit test: returns initialized bool |
| `network_disconnect` | Disconnect from peer | Integration test: connect then disconnect |
| `update_kanban_board` | Update board name/description | Unit test: board name changes |
| `delete_kanban_board` | Delete a board | Unit test: board no longer exists |
| `list_kanban_cards` | List all cards in board | Unit test: returns all cards |

### Phase 2: HTTP Transport Layer
**Goal**: MCP serves HTTP directly (replace Bridge)

```rust
// New module: communitas-mcp/src/transport/mod.rs
pub mod stdio;   // Existing (refactored)
pub mod http;    // New: Axum-based HTTP/REST

pub trait Transport {
    async fn run(&self, server: McpServer) -> Result<()>;
}
```

Tasks:
- [ ] Create transport trait abstraction
- [ ] Refactor stdio into transport module
- [ ] Port Bridge's Axum router to MCP
- [ ] Map HTTP endpoints to MCP tool calls
- [ ] Preserve exact REST API signatures

### Phase 3: AI Agent E2E Test Suite
**Goal**: Full workflow testing via MCP tools

Test Scenarios:
- [ ] Identity creation workflow (create_vault → authenticate)
- [ ] Organization setup (create_entity → add_member → send_message)
- [ ] Kanban workflow (create_board → create_column → create_card → move_card)
- [ ] File storage workflow (write_file → list_files → read_file → delete_file)
- [ ] P2P sync workflow (network_start → network_connect → join_entity)

### Phase 4: Bridge Deprecation
**Goal**: Remove communitas-bridge from workspace

Tasks:
- [ ] Verify all Bridge endpoints work via MCP HTTP transport
- [ ] Update Flutter/web clients to use MCP HTTP endpoints
- [ ] Remove communitas-bridge from Cargo.toml workspace
- [ ] Delete communitas-bridge directory
- [ ] Update documentation

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Feature Parity | 100% | All 74 bridge endpoints available via MCP |
| Tool Count | 76 | Current 70 + 6 gap tools |
| E2E Coverage | 5 workflows | Full AI agent workflow testing |
| Bridge Removal | 0 LOC | Crate removed from workspace |
| Zero Warnings | 0 | Clean compilation |
| Test Pass Rate | 100% | All tests passing |

## File Changes

### Phase 1: Gap Tools (Modified)
```
communitas-mcp/src/tools.rs             # Add 6 missing tools
```

### Phase 2: Transport Layer (New)
```
communitas-mcp/src/transport/mod.rs     # Transport trait
communitas-mcp/src/transport/stdio.rs   # Refactored stdio
communitas-mcp/src/transport/http.rs    # New HTTP transport (from Bridge)
```

### Phase 2: Transport Layer (Modified)
```
communitas-mcp/src/server.rs            # Use transport abstraction
communitas-mcp/src/main.rs              # --transport flag
communitas-mcp/Cargo.toml               # Add axum, tower-http
```

### Phase 4: Bridge Removal (Deleted)
```
communitas-bridge/                      # Entire crate removed
Cargo.toml                              # Remove workspace member
```

## Dependencies

```toml
# communitas-mcp/Cargo.toml
[dependencies]
# Existing...
base64 = "0.22"                         # Binary file encoding
```

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing MCP clients | Keep existing tool signatures, add new tools |
| Bridge client compatibility | Preserve HTTP endpoints, only change internals |
| Performance regression | Benchmark before/after |
| Auth complexity | Start with --demo mode for testing |

## Implementation Order

1. **Phase 1**: Fill 6 tool gaps (enables feature parity)
2. **Phase 2**: HTTP transport layer (enables Bridge removal)
3. **Phase 3**: AI agent E2E test suite (validates consolidation)
4. **Phase 4**: Remove communitas-bridge (final cleanup)

---

**Design Status**: LOCKED
**Next Step**: Implement Phase 1 - Add 6 missing tools to communitas-mcp/src/tools.rs

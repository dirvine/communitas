# PLAN-34: Phase 5.4 - MCP Validation Suite

**Milestone**: M5 Stabilization
**Phase**: 5.4 - MCP Validation Suite
**Status**: Planning
**Created**: 2026-01-20

## Goal

Comprehensive MCP regression testing to ensure all 106 MCP tools work correctly and maintain parity with UiServices.

## Current State

- **Existing parity tests**: `communitas-mcp/tests/parity_test.rs` covers ~15 tools
- **Total MCP tools**: 106 tools across auth, entity, messaging, kanban, drive, canvas, call, network, contact, website, and presence domains
- **Coverage gap**: ~91 tools lack parity tests

### Tools Currently Tested
- **Kanban**: create_kanban_board, create_kanban_card, create_kanban_column, get_kanban_board, list_kanban_boards
- **Messaging**: send_message, get_messages, list_threads
- **Drive**: list_disks, read_file, write_file
- **Canvas**: canvas_get_snapshot, canvas_add_text
- **Call**: start_voice_call, join_call, end_call (registration only)

### Tools Needing Parity Tests

**Auth/Session (10 tools)**:
- authenticate, create_vault, authenticate_token, health_check, core_status
- list_vaults, delete_vault, import_vault, get_session, logout

**Identity (4 tools)**:
- create_identity, recover_identity, validate_mnemonic, export_vault

**Entity (7 tools)**:
- create_entity, update_entity, delete_entity, add_member, remove_member
- get_entity, list_entities, list_members

**Messaging (12 tools)**:
- delete_message, edit_message, add_reaction, remove_reaction
- get_reactions, get_available_reactions, create_thread, get_thread_messages
- create_invite, accept_invite, list_pending_invites

**Kanban (18 tools)**:
- move_kanban_card, update_kanban_card, delete_kanban_card
- list_kanban_cards, list_kanban_columns, get_kanban_column
- update_kanban_column, delete_kanban_column, move_kanban_column
- change_card_state, assign_user, unassign_user
- create_kanban_tag, list_kanban_tags, tag_card, untag_card
- add_step, get_step, toggle_step, delete_step
- add_comment, list_comments, delete_comment

**Drive (9 tools)**:
- delete_file, get_disk_stats, create_directory, move_file, copy_file
- get_file_preview, list_files, upload_with_metadata, get_media_metadata

**Call/Media (8 tools)**:
- share_screen, toggle_mute, toggle_video, get_call_status
- get_call_participants (plus existing 3 tools need deeper testing)

**Network (8 tools)**:
- network_start, network_stop, network_connect, network_status
- network_peers, network_request_external_address
- get_connection_words, connect_by_words

**Presence (8 tools)**:
- set_presence, get_presence, subscribe_to_presence
- announce_presence, query_presence, get_our_presence
- get_cached_presence, set_my_presence

**Contact (10 tools)**:
- create_contact, update_contact, delete_contact, link_contact
- set_favourite_contact, remove_favourite_contact, get_contact
- list_contacts, get_contact_presence, list_favourite_contacts, search_contacts

**Website (4 tools)**:
- create_website, update_website, delete_website, get_website

**Canvas (10 tools)**:
- canvas_add_chart, canvas_remove_element, canvas_update_transform
- canvas_select_element, canvas_deselect_all, canvas_set_viewport
- canvas_set_view, canvas_clear, canvas_export, canvas_import, canvas_element_at

**Workspace/Profile/Misc (5 tools)**:
- workspace_init, create_delegate_token
- get_profile, update_profile, join_entity

## Tasks

### Task 1: Expand Kanban Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for remaining 18 kanban tools
2. Test card operations (move, update, delete)
3. Test column operations (update, delete, move)
4. Test tag operations (create, list, tag/untag)
5. Test step and comment operations

**Done when**:
- All 23 kanban tools have parity tests
- Tests verify MCP routes through KanbanService
- Edge cases (invalid IDs, permissions) tested

### Task 2: Expand Messaging Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for remaining 12 messaging tools
2. Test message lifecycle (edit, delete)
3. Test reaction operations
4. Test thread creation and retrieval
5. Test invite workflow

**Done when**:
- All 15 messaging tools have parity tests
- Tests verify MCP routes through MessagingService

### Task 3: Expand Drive Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for remaining 9 drive tools
2. Test file operations (delete, move, copy)
3. Test directory operations
4. Test metadata and preview operations

**Done when**:
- All 12 drive tools have parity tests
- Tests verify MCP routes through DriveService

### Task 4: Expand Canvas Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for remaining 10 canvas tools
2. Test element lifecycle (remove, update transform)
3. Test selection operations
4. Test viewport/view operations
5. Test export/import

**Done when**:
- All 13 canvas tools have parity tests
- Tests verify MCP routes through CanvasService

### Task 5: Add Contact/Presence Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for all 10 contact tools
2. Add tests for all 8 presence tools
3. Test CRUD operations
4. Test search and favourites

**Done when**:
- All contact and presence tools have parity tests

### Task 6: Add Network/Call Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for all 8 network tools
2. Add deeper tests for 8 call tools
3. Test network lifecycle (start, connect, stop)
4. Test call media controls

**Done when**:
- All network and call tools have parity tests

### Task 7: Add Auth/Entity Parity Tests
**Files**: `communitas-mcp/tests/parity_test.rs`
**Approach**:
1. Add tests for 10 auth/session tools
2. Add tests for 7 entity tools
3. Add tests for 4 identity tools
4. Test authentication flow
5. Test entity membership

**Done when**:
- All auth, entity, and identity tools have parity tests

### Task 8: Add Golden Data Comparisons
**Files**: `communitas-mcp/tests/golden/`, new test module
**Approach**:
1. Create golden data fixture files (JSON)
2. Add tests that compare tool output against golden data
3. Cover representative operations from each domain
4. Implement fixture update mechanism

**Done when**:
- Golden data fixtures for key operations exist
- Tests compare actual output vs expected
- CI fails on golden data mismatch

### Task 9: Create MCP-Driven E2E Workflows
**Files**: `communitas-mcp/tests/e2e_workflows.rs`
**Approach**:
1. Create multi-step workflow tests
2. Workflow 1: User registration → Create entity → Invite member
3. Workflow 2: Create board → Add columns → Create cards → Move cards
4. Workflow 3: Write file → Read file → Share → Delete
5. Workflow 4: Start call → Join → Toggle controls → End

**Done when**:
- 4 E2E workflow tests exist
- Tests demonstrate real-world usage patterns
- Tests verify state consistency across operations

### Task 10: Document MCP Testing Strategy
**Files**: `docs/testing/mcp-testing.md`
**Approach**:
1. Document parity testing principles
2. Document golden data approach
3. Document E2E workflow patterns
4. Provide examples for adding new tests
5. Document CI integration

**Done when**:
- Testing strategy document complete
- Includes examples and guidelines
- Referenced from main docs

## Verification

```bash
# Run all MCP tests
cargo test -p communitas-mcp --test parity_test
cargo test -p communitas-mcp --test e2e_workflows

# Check coverage
cargo test -p communitas-mcp -- --list 2>&1 | wc -l
# Target: 100+ test functions

# Verify golden data
cargo test -p communitas-mcp golden

# CI verification
cargo clippy -p communitas-mcp -- -D warnings
cargo fmt -p communitas-mcp -- --check
```

## Dependencies

- communitas-mcp tools implementation (complete)
- communitas-ui-service wiring (complete from M3.1)
- UiStorage test fixtures (complete)

## Notes

- Use `enable_demo_mode()` for authenticated test scenarios
- Use 8MB stack threads for async tests (avoid stack overflow)
- Follow existing test patterns in parity_test.rs
- Keep tests fast (< 1s each)

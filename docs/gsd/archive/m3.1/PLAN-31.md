# PLAN-31: Phase 2.5 — MCP Parity Harness

**Milestone**: M3.1 Remediation
**Phase**: 2.5 (MCP Service Parity)
**Status**: Pending
**Created**: 2026-01-20
**Depends on**: PLAN-30 (Kanban Complete)

---

## Overview

Ensure MCP tools use the same code path as Dioxus UI by routing through `communitas-ui-service` instead of calling `CommunitasApp` directly. This guarantees feature parity between automation and interactive use.

## Problem

Currently MCP tools bypass UI services:

```rust
// Current MCP implementation (tools.rs)
async fn execute_send_message(app: &CommunitasApp, args: Value) -> ToolCallResult {
    let cmd = Command::SendMessage { ... };
    app.execute(cmd).await  // <-- Direct call, bypasses MessagingService
}
```

This means:
- Watch channel updates don't fire for MCP operations
- Thread metadata derivation may differ
- Validation logic may be inconsistent
- Any UI service enhancements won't apply to MCP

## Solution

Route MCP tools through UI services:

```rust
// Target MCP implementation
async fn execute_send_message(services: &UiServices, args: Value) -> ToolCallResult {
    services.messaging().send_message(entity_id, text).await  // <-- Same path as Dioxus
}
```

---

## Prerequisites

- [x] MessagingService fully wired (PLAN-26)
- [ ] DriveService fully wired (PLAN-27)
- [ ] CallService fully wired (PLAN-28)
- [ ] CanvasService fully wired (PLAN-29)
- [ ] KanbanService fully wired (PLAN-30)

---

## Tasks

<task type="auto" priority="p1">
  <n>Add UiServices to MCP server context</n>
  <files>
    communitas-mcp/src/server.rs,
    communitas-mcp/src/lib.rs
  </files>
  <action>
    1. Add `services: Arc<UiServices>` to McpServer struct
    2. Update McpServer::new() to create UiServices from CommunitasApp
    3. Pass services to tool execution context
    4. Ensure services are accessible in all tool handlers
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-mcp --all-features -- -D warnings
    cargo build -p communitas-mcp
  </verify>
  <done>
    - McpServer holds UiServices
    - Tool handlers receive services reference
    - Compiles without errors
  </done>
</task>

<task type="auto" priority="p1">
  <n>Refactor messaging MCP tools to use MessagingService</n>
  <files>
    communitas-mcp/src/tools.rs
  </files>
  <action>
    1. Update execute_send_message to use services.messaging().send_message()
    2. Update execute_list_threads to use services.messaging().list_threads()
    3. Update execute_get_messages to use services.messaging().get_messages()
    4. Update execute_add_reaction to use services.messaging().add_reaction()
    5. Update execute_remove_reaction to use services.messaging().remove_reaction()
    6. Update execute_edit_message to use services.messaging().edit_message()
    7. Update execute_delete_message to use services.messaging().delete_message()
    8. Convert MessagingError to ToolCallError appropriately
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-mcp --all-features -- -D warnings
    cargo test -p communitas-mcp
  </verify>
  <done>
    - All messaging tools route through MessagingService
    - Error mapping correct
    - Watch channels fire for MCP operations
  </done>
</task>

<task type="auto" priority="p1">
  <n>Refactor drive MCP tools to use DriveService</n>
  <files>
    communitas-mcp/src/tools.rs
  </files>
  <action>
    1. Update execute_write_file to use services.drive().write_file()
    2. Update execute_read_file to use services.drive().read_file()
    3. Update execute_list_files to use services.drive().list_directory()
    4. Update execute_delete_file to use services.drive().delete_file()
    5. Update execute_create_directory to use services.drive().create_directory()
    6. Add any missing drive tools that DriveService supports
    7. Convert DriveError to ToolCallError appropriately
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-mcp --all-features -- -D warnings
    cargo test -p communitas-mcp
  </verify>
  <done>
    - All drive tools route through DriveService
    - File operations have parity with UI
    - Progress tracking available for large files
  </done>
</task>

<task type="auto" priority="p1">
  <n>Refactor call MCP tools to use CallService</n>
  <files>
    communitas-mcp/src/tools.rs
  </files>
  <action>
    1. Update execute_start_call to use services.call().start_call()
    2. Update execute_join_call to use services.call().join_call()
    3. Update execute_leave_call to use services.call().leave_call()
    4. Update execute_toggle_mute to use services.call().toggle_mute()
    5. Update execute_toggle_video to use services.call().toggle_video()
    6. Update execute_list_devices to use services.call().get_audio_devices() / get_video_devices()
    7. Convert CallError to ToolCallError appropriately
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-mcp --all-features -- -D warnings
    cargo test -p communitas-mcp
  </verify>
  <done>
    - All call tools route through CallService
    - Device enumeration matches UI
    - Call state changes visible via watch channels
  </done>
</task>

<task type="auto" priority="p1">
  <n>Refactor canvas MCP tools to use CanvasService</n>
  <files>
    communitas-mcp/src/tools.rs
  </files>
  <action>
    1. Update execute_canvas_add_text to use services.canvas().add_text()
    2. Update execute_canvas_add_image to use services.canvas().add_image()
    3. Update execute_canvas_remove_element to use services.canvas().remove_element()
    4. Update execute_canvas_update_transform to use services.canvas().update_transform()
    5. Update execute_canvas_export to use services.canvas().export_scene()
    6. Add execute_canvas_import using services.canvas().import_scene()
    7. Convert CanvasError to ToolCallError appropriately
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-mcp --all-features -- -D warnings
    cargo test -p communitas-mcp
  </verify>
  <done>
    - All canvas tools route through CanvasService
    - Scene state persists via CommunitasApp
    - Offline queue works for MCP operations
  </done>
</task>

<task type="auto" priority="p1">
  <n>Refactor kanban MCP tools to use KanbanService</n>
  <files>
    communitas-mcp/src/tools.rs
  </files>
  <action>
    1. Update execute_create_board to use services.kanban().create_board()
    2. Update execute_list_boards to use services.kanban().list_boards()
    3. Update execute_create_column to use services.kanban().create_column()
    4. Update execute_create_card to use services.kanban().create_card()
    5. Update execute_move_card to use services.kanban().move_card()
    6. Update execute_update_card to use services.kanban().update_card()
    7. Update execute_delete_card to use services.kanban().delete_card()
    8. Convert KanbanError to ToolCallError appropriately
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-mcp --all-features -- -D warnings
    cargo test -p communitas-mcp
  </verify>
  <done>
    - All kanban tools route through KanbanService
    - CRDT operations attributed to correct user
    - Watch channels fire for board updates
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add MCP parity integration tests</n>
  <files>
    communitas-mcp/tests/parity_test.rs
  </files>
  <action>
    1. Create integration test file
    2. Test messaging: send via MCP, verify via UI service watch channel
    3. Test drive: write via MCP, read via UI service
    4. Test kanban: create board via MCP, verify via UI service list
    5. Test that errors are consistent between UI and MCP paths
    6. Test that validation applies equally to both paths
  </action>
  <verify>
    cargo test -p communitas-mcp --test parity_test
  </verify>
  <done>
    - Parity tests pass
    - MCP operations visible via UI watch channels
    - Consistent behavior verified
  </done>
</task>

<task type="auto" priority="p1">
  <n>Update MCP tool documentation</n>
  <files>
    docs/api/mcp-api.md
  </files>
  <action>
    1. Update tool descriptions to reflect service-backed implementation
    2. Document watch channel integration
    3. Add examples showing MCP + UI interop
    4. Note any behavioral differences that remain intentional
    5. Update error code documentation
  </action>
  <verify>
    # Manual review of documentation accuracy
  </verify>
  <done>
    - MCP API docs reflect actual implementation
    - Parity with UI is documented
    - Examples are accurate
  </done>
</task>

---

## Exit Criteria

- [ ] All MCP tools route through UiServices
- [ ] Watch channels fire for MCP operations
- [ ] Parity integration tests pass
- [ ] Documentation updated

---

## Notes

- Some MCP-specific behavior may remain (e.g., JSON serialization)
- Auth context for MCP may differ from interactive UI
- Consider rate limiting parity between paths

---

## Next

PLAN-32: Documentation Accuracy Audit


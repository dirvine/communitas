# PLAN-30: Phase 2.4 — Kanban Auth & CRDT Events

**Milestone**: M3.1 Remediation
**Phase**: 2.4 (Kanban Authentication)
**Status**: Pending
**Created**: 2026-01-20
**Depends on**: PLAN-29 (Canvas Complete)

---

## Overview

Rework `communitas-ui-service/src/kanban.rs` to properly handle authentication. Currently, `CoreKanbanService::new("anonymous")` is hardcoded. The service needs to:
1. Reinitialize with real peer_id when user authenticates
2. Subscribe to CRDT events for reactive watch channel updates
3. Use CommunitasApp for persistence

## Problem

```rust
// Current code (kanban.rs:114)
let core = CoreKanbanService::new("anonymous");  // <-- WRONG
```

This causes all CRDT operations to be attributed to "anonymous" instead of the actual user.

## Prerequisites

- [x] CoreKanbanService available from communitas-kanban
- [x] AuthController subscription pattern established
- [x] CommunitasApp Kanban Commands exist
- [ ] PLAN-29 complete

---

## Tasks

<task type="auto" priority="p1">
  <n>Add CommunitasApp to KanbanService constructor</n>
  <files>
    communitas-ui-service/src/kanban.rs,
    communitas-ui-service/src/lib.rs
  </files>
  <action>
    1. Add `app: Arc<CommunitasApp>` field to KanbanService struct
    2. Update KanbanService::new() to accept `app` parameter
    3. Update UiServices::new() in lib.rs to pass app to KanbanService
    4. Add `pub fn app(&self) -> Arc<CommunitasApp>` accessor
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
  </verify>
  <done>
    - KanbanService holds Arc<CommunitasApp>
    - UiServices wires app to KanbanService
    - Compiles without errors
  </done>
</task>

<task type="auto" priority="p1">
  <n>Subscribe to auth state changes</n>
  <files>
    communitas-ui-service/src/kanban.rs
  </files>
  <action>
    1. In KanbanService::new(), spawn background task to watch auth changes
    2. When auth state changes to LoggedIn:
       - Get peer_id from auth.peer_id()
       - Reinitialize CoreKanbanService with real peer_id
       - Store new instance in Arc<RwLock<CoreKanbanService>>
    3. When auth state changes to LoggedOut:
       - Reset to anonymous or clear boards
       - Clear watch channel
    4. Use weak reference to avoid reference cycles
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - CoreKanbanService reinitializes on auth change
    - peer_id correctly attributed to operations
    - No memory leaks from reference cycles
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire create_board to CommunitasApp</n>
  <files>
    communitas-ui-service/src/kanban.rs
  </files>
  <action>
    1. Update create_board() to:
       - Create locally via CoreKanbanService
       - Execute Command::CreateKanbanBoard for persistence
       - Handle KanbanBoardCreated event
       - Update watch channel
    2. Ensure board_id is consistent between local and persisted
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Boards persist via CommunitasApp
    - Board creation attributed to authenticated user
    - Watch channel updated
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire column and card operations to CommunitasApp</n>
  <files>
    communitas-ui-service/src/kanban.rs
  </files>
  <action>
    1. Wire create_column() to Command::CreateKanbanColumn
    2. Wire create_card() to Command::CreateKanbanCard
    3. Wire move_card() to Command::MoveKanbanCard
    4. Wire update_card() to Command::UpdateKanbanCard
    5. Wire delete_card() to Command::DeleteKanbanCard
    6. Wire update_board() to Command::UpdateKanbanBoard
    7. Wire delete_board() to Command::DeleteKanbanBoard
    8. Handle all corresponding events
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - All CRUD operations persist via CommunitasApp
    - Events trigger watch channel updates
    - Operations attributed to correct user
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire queries to CommunitasApp</n>
  <files>
    communitas-ui-service/src/kanban.rs
  </files>
  <action>
    1. Wire get_board() to Query::GetKanbanBoard
    2. Wire list_boards() to Query::ListKanbanBoards
    3. Wire list_columns() to Query::ListKanbanColumns
    4. Wire get_card() to Query::GetKanbanCard
    5. Wire list_cards() to Query::ListKanbanCards
    6. Convert QueryResponse types to UI types
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Queries return persisted data from CommunitasApp
    - Response conversion correct
    - Error handling works
  </done>
</task>

<task type="auto" priority="p1">
  <n>Subscribe to CRDT events for reactive updates</n>
  <files>
    communitas-ui-service/src/kanban.rs
  </files>
  <action>
    1. Subscribe to CommunitasApp events: Subscription::KanbanEvents { entity_id }
    2. Spawn background task to process events:
       - KanbanBoardCreated -> refresh boards
       - KanbanColumnCreated -> refresh board
       - KanbanCardCreated -> refresh board
       - KanbanCardMoved -> refresh board
       - KanbanCardUpdated -> refresh card
       - KanbanCardDeleted -> refresh board
       - KanbanBoardDeleted -> refresh boards
    3. Update watch channel on each event
    4. Handle concurrent CRDT merges gracefully
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - CRDT events trigger watch channel updates
    - UI reactively updates without manual refresh
    - Concurrent edits merge correctly
  </done>
</task>

<task type="auto" priority="p1">
  <n>Load boards on entity selection</n>
  <files>
    communitas-ui-service/src/kanban.rs
  </files>
  <action>
    1. Add load_boards(entity_id) method
    2. Query::ListKanbanBoards for entity
    3. Set loading=true in watch channel during load
    4. Update watch channel with loaded boards
    5. Subscribe to entity's Kanban events
    6. Call from UI when entity selected
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Boards load when entity selected
    - Loading state visible in UI
    - Event subscription active for entity
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add integration tests for KanbanService</n>
  <files>
    communitas-ui-service/tests/kanban_integration.rs
  </files>
  <action>
    1. Create integration test file
    2. Test board lifecycle: create -> list -> update -> delete
    3. Test card lifecycle: create -> move -> update -> delete
    4. Test authentication: operations attributed to correct user
    5. Test CRDT events trigger watch updates
    6. Test auth state changes reinitialize service
  </action>
  <verify>
    cargo test -p communitas-ui-service --test kanban_integration
  </verify>
  <done>
    - Integration tests pass with real CommunitasApp
    - All CRUD operations verified
    - Auth attribution verified
    - CRDT sync verified
  </done>
</task>

---

## Exit Criteria

- [ ] CoreKanbanService initialized with real peer_id after auth
- [ ] All operations attributed to authenticated user
- [ ] CRDT events trigger reactive watch updates
- [ ] All Kanban Commands/Queries wired to CommunitasApp
- [ ] Integration tests pass

---

## Notes

- Kanban uses Yrs CRDT (different from Canvas LWW)
- CoreKanbanService handles CRDT merging internally
- peer_id must match auth identity for attribution

---

## Next

PLAN-31: MCP Parity Harness

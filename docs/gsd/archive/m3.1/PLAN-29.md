# PLAN-29: Phase 2.3 — Canvas Collaboration

**Milestone**: M3.1 Remediation
**Phase**: 2.3 (Canvas Integration)
**Status**: Pending
**Created**: 2026-01-20
**Depends on**: PLAN-28 (CallService Complete)

---

## Overview

Hook `communitas-ui-service/src/canvas.rs` into `canvas-core` from `../saorsa-canvas` for persistence, real-time collaboration, and interop with standalone canvas. Per ADR-021, canvas uses LWW conflict resolution (not Yrs CRDT).

## Prerequisites

- [x] canvas-core dependency available (path = "../saorsa-canvas/canvas-core")
- [x] ADR-021 defines integration strategy
- [x] CommunitasApp Canvas Commands exist (CanvasAddText, CanvasAddImage, etc.)
- [ ] PLAN-28 complete

---

## Architecture Notes (ADR-021)

- **Sync Strategy**: Canvas uses LWW (Last-Write-Wins), not Yrs CRDT
- **OfflineQueue**: Buffers operations when offline, replays on reconnect
- **InputFusion**: Combines touch + voice inputs for AI-assisted drawing
- **Scene**: Core scene graph with elements, transforms, z-ordering

---

## Tasks

<task type="auto" priority="p1">
  <n>Add CommunitasApp to CanvasService constructor</n>
  <files>
    communitas-ui-service/src/canvas.rs,
    communitas-ui-service/src/lib.rs
  </files>
  <action>
    1. Add `app: Arc<CommunitasApp>` field to CanvasService struct
    2. Update CanvasService::new() to accept `app` parameter
    3. Update UiServices::new() in lib.rs to pass app to CanvasService
    4. Add `pub fn app(&self) -> Arc<CommunitasApp>` accessor
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
  </verify>
  <done>
    - CanvasService holds Arc<CommunitasApp>
    - UiServices wires app to CanvasService
    - Compiles without errors
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire add_element to canvas-core Scene and CommunitasApp</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Update add_text() to:
       - Create Element via canvas-core Scene::add_text()
       - Execute Command::CanvasAddText for persistence
       - Update watch channel with new element
    2. Update add_image() similarly with Command::CanvasAddImage
    3. Update add_chart() with Command::CanvasAddChart
    4. Ensure element IDs are consistent between local Scene and CommunitasApp
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Add operations update local Scene AND persist via CommunitasApp
    - Watch channel reflects new elements
    - Element IDs consistent
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire remove_element to Scene and CommunitasApp</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Update remove_element() to:
       - Remove from local canvas-core Scene
       - Execute Command::CanvasRemoveElement for persistence
       - Update watch channel
    2. Handle non-existent element gracefully
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Remove operations update Scene AND persist
    - Watch channel reflects removal
    - Error handling for missing elements
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire transform updates to Scene and CommunitasApp</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Update update_transform() to:
       - Update element in local Scene
       - Execute Command::CanvasUpdateTransform
       - Update watch channel
    2. Handle position, size, rotation, z-index changes
    3. Batch rapid updates to avoid excessive commands
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Transform updates persist via CommunitasApp
    - Rapid drag operations batched appropriately
    - Z-index changes work
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement canvas persistence via Query::GetCanvasSnapshot</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Add load_canvas(entity_id) method
    2. Query::GetCanvasSnapshot to load persisted state
    3. Reconstruct local Scene from QueryResponse::CanvasSnapshot
    4. Update watch channel with loaded state
    5. Call on entity selection in UI
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Canvas state loads from CommunitasApp on open
    - Local Scene matches persisted state
    - Watch channel updated after load
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement selection with Command::CanvasSelectElement</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Update select_element() to:
       - Update local Scene selection
       - Execute Command::CanvasSelectElement
       - Update watch channel with selected_ids
    2. Update deselect_all() with Command::CanvasDeselectAll
    3. Support multi-select (Shift+click pattern)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Selection persists (for collaborative awareness)
    - Multi-select works
    - Watch channel reflects selection state
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement viewport/view controls</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Wire set_viewport() to Command::CanvasSetViewport
    2. Wire set_view() to Command::CanvasSetView (zoom, pan)
    3. Update watch channel with viewport state
    4. These are typically local-only but persist for session restore
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Zoom/pan state persists
    - Viewport dimensions tracked
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement export_scene via Query::CanvasExport</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Wire export_scene() to Query::CanvasExport
    2. Return JSON representation of scene
    3. Also support import via Command::CanvasImport
    4. Handle import validation errors
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Export produces valid JSON scene
    - Import reconstructs scene correctly
    - Invalid JSON handled gracefully
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire OfflineQueue for offline-first operations</n>
  <files>
    communitas-ui-service/src/canvas.rs
  </files>
  <action>
    1. Add OfflineQueue from canvas-core to CanvasService
    2. Queue operations when CommunitasApp unavailable
    3. Replay queue when connection restored
    4. Handle conflicts with LWW resolution
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Offline operations queued locally
    - Queue replays on reconnect
    - LWW resolves conflicts
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add integration tests for CanvasService</n>
  <files>
    communitas-ui-service/tests/canvas_integration.rs
  </files>
  <action>
    1. Create integration test file
    2. Test element lifecycle: add -> update -> remove
    3. Test persistence: add -> save -> reload -> verify
    4. Test selection state
    5. Test export/import round-trip
    6. Test offline queue behavior
  </action>
  <verify>
    cargo test -p communitas-ui-service --test canvas_integration
  </verify>
  <done>
    - Integration tests pass with real CommunitasApp
    - All CRUD operations verified
    - Persistence verified
  </done>
</task>

---

## Exit Criteria

- [ ] All CanvasService methods wire to both local Scene AND CommunitasApp
- [ ] Canvas state persists across sessions
- [ ] OfflineQueue handles disconnections
- [ ] Integration tests pass
- [ ] Watch channels update reactively

---

## Notes

- Canvas uses LWW, not CRDT - don't mix with Kanban sync
- InputFusion (touch + voice) is future enhancement
- Collaborative cursors are future enhancement

---

## Next

PLAN-30: Kanban Auth & CRDT Events

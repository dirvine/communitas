# Dioxus Desktop Prototype — Milestone 3 Plan
_Status: In Progress • Started: January 19, 2026_

## 1. Goal & Exit Criteria

Deliver Kanban boards, Drive file management, WebRTC calls, and Canvas collaboration in `communitas-dioxus`, powered by the shared Rust UI service layer (ADR-019) and MCP tooling. This milestone unlocks advanced collaboration surfaces by wiring board views, file browsers, call controls, and collaborative canvas to the core APIs.

Exit when:

1. Dioxus renders Kanban boards with drag-and-drop columns and cards, card detail modals, and filtering.
2. Drive file browser shows entity virtual disks with upload, download, preview, and folder navigation.
3. Call lobby shows participant list, mute/unmute controls, and leave button (audio-only MVP; video stretch).
4. Canvas renders a collaborative whiteboard with basic shapes, text, and real-time sync (scope TBD).
5. `UiServices` exposes `KanbanService`, `DriveService`, `CallService`, and `CanvasService` consumed by both Dioxus and MCP.
6. MCP parity tests confirm all board, file, call, and canvas operations return identical data to what Dioxus displays.
7. Performance instrumentation shows operation latency within project targets (<100ms local, <500ms remote).

## 2. Scope

### In-scope

- **Kanban UI**: Board list, board view with column virtualization, card drag-and-drop, card detail modal, swimlane filters, CRDT conflict banners, keyboard accessibility, skeleton loaders.
- **Drive UI**: File browser with tree/list views, upload/download with progress, preview panel with checksum validation, disk selector (Private/Public/Shared), quota meter, native file picker integration.
- **Call UI**: Lobby view, device selectors (microphone/speaker), mute/video/share controls, participant tiles with presence indicators, graceful fallbacks when media capture fails.
- **Canvas UI**: Toolbar, layer list, shared cursors, history scrubber, offline queue, basic shapes/text (feature flag for Blitz renderer experiments, default Wry).
- **KanbanService**: New service in `communitas-ui-service` wrapping `communitas-kanban` with watch channels.
- **DriveService**: New service for file operations with upload progress and watch channels.
- **CallService**: New service for WebRTC call management with participant state and device management.
- **CanvasService**: New service for CRDT-synced canvas operations with offline queue support.
- **MCP parity**: Create `scripts/tests/mcp_advanced_surfaces.sh` unified harness; diff MCP output against core exports.
- **Telemetry**: Tracing spans for `ui.kanban.*`, `ui.drive.*`, `ui.call.*`, `ui.canvas.*` operations.
- **Accessibility**: Keyboard navigation for drag-and-drop, file browser, call controls; ARIA labels documented in `tests/webdriverio/specs/accessibility.smoke.js`.

### Out-of-scope (later milestones)

- Video calls (audio-only MVP for M3).
- Screen sharing.
- Advanced canvas features (layers, collaboration cursors, rich shapes).
- Offline conflict resolution polish (Milestone 4).
- Mobile-specific layouts (Milestone 5).

## 3. Architecture Overview

| Layer | Milestone 3 Responsibilities | Notes |
| --- | --- | --- |
| `communitas-ui-service` | Add `KanbanService`, `DriveService`, `CallService`, `CanvasService` with watch channels; wire to core APIs. | Debounce board/file updates to avoid UI thrash. |
| `communitas-ui-api` | Add DTOs for boards, columns, cards, files, directories, call state, canvas elements. | Single source of truth for Dioxus + MCP. |
| `communitas-dioxus` | Board list route, board view, drive browser route, call lobby route, canvas route. | Use Dioxus signals/hooks; show skeletons during loads. |
| `communitas-core` | Existing Kanban commands, Drive commands, WebRTC signaling. | Enhance as needed for UI requirements. |
| `communitas-kanban` | Complete implementation (70+ tests, all types). | Wire to UI service layer. |
| MCP | Expose board/file/call/canvas tools with identical semantics to UiServices. | MCP parity is source of truth for automation. |

## 4. Workstreams & Tasks

### 4.1 KanbanService Implementation (Phase 10)

1. Create `communitas-ui-api/src/kanban.rs` with DTOs:
   - `BoardSummary` (id, name, entity_id, column_count, card_count, last_activity)
   - `BoardView` (full board with columns and cards)
   - `ColumnView` (id, name, position, card_ids, wip_limit)
   - `CardView` (id, title, description, state, assignees, tags, due_date, checklist_progress)
   - `CardDetail` (full card with steps, comments, attachments)

2. Create `communitas-ui-service/src/kanban.rs` with `KanbanService`:
   - `list_boards(entity_id) -> Vec<BoardSummary>`
   - `get_board(board_id) -> BoardView`
   - `get_card(board_id, card_id) -> CardDetail`
   - `create_board(entity_id, name, template) -> BoardSummary`
   - `create_column(board_id, name, position) -> ColumnView`
   - `create_card(board_id, column_id, title) -> CardView`
   - `move_card(board_id, card_id, target_column, position)`
   - `update_card(board_id, card_id, updates)`
   - `archive_card(board_id, card_id)`
   - Watch channel for board updates

3. Add tracing spans: `ui.kanban.list_boards`, `ui.kanban.get_board`, `ui.kanban.move_card`, etc.

### 4.2 Dioxus Kanban UI (Phase 11)

1. Create `/boards` route listing entity boards with grid layout.
2. Create `/board/:id` route with:
   - Column virtualization for boards with many columns
   - Column containers with drag-drop zones
   - Card components with summary info
   - "Add column" and "Add card" buttons
   - Board header with settings dropdown
   - CRDT conflict banners when merge conflicts detected
3. Create `CardDetailModal` component:
   - Title, description, assignees
   - Checklist with progress bar
   - Comments thread
   - Activity log
4. Implement drag-and-drop:
   - Use Dioxus drag events or `dioxus-sortable` if available
   - Optimistic UI updates with reconciliation
   - Visual feedback during drag
   - Keyboard accessibility (arrow keys + Enter to move)
5. Add filters: swimlane filters by assignee, tag, due date, state.
6. Skeleton loaders for board and card list.

### 4.3 DriveService Implementation (Phase 12)

1. Create `communitas-ui-api/src/drive.rs` with DTOs:
   - `DiskInfo` (disk_type, total_size, used_size, entity_id)
   - `DirectoryEntry` (name, path, is_dir, size, modified_at, mime_type)
   - `FilePreview` (path, mime_type, thumbnail_data, metadata)
   - `UploadProgress` (file_path, bytes_uploaded, total_bytes, state)

2. Create `communitas-ui-service/src/drive.rs` with `DriveService`:
   - `list_disks(entity_id) -> Vec<DiskInfo>`
   - `list_directory(entity_id, disk_type, path) -> Vec<DirectoryEntry>`
   - `get_file_preview(entity_id, disk_type, path) -> FilePreview`
   - `upload_file(entity_id, disk_type, path, content) -> UploadProgress`
   - `download_file(entity_id, disk_type, path) -> Vec<u8>`
   - `create_directory(entity_id, disk_type, path)`
   - `delete_path(entity_id, disk_type, path)`
   - `move_path(entity_id, disk_type, from, to)`
   - Watch channel for upload progress and directory changes

3. Add tracing spans: `ui.drive.list_directory`, `ui.drive.upload`, `ui.drive.download`, etc.

### 4.4 Dioxus Drive UI (Phase 13)

1. Create `/drive` route with:
   - Disk selector (Private/Public/Shared tabs)
   - Breadcrumb navigation
   - Tree view sidebar + list/grid toggle in main area
   - Upload button with native file picker integration
   - Drag-drop zone for uploads
   - Quota meter showing disk usage
2. Create `FilePreviewPanel` component:
   - Image thumbnails
   - Text file preview
   - PDF first page (if feasible)
   - Metadata display (size, modified, type)
   - Checksum validation indicator
3. Create `UploadProgressBar` component:
   - Per-file progress with bytes/total display
   - Cancel button
   - Error state with retry
   - Checksum verification status
4. Create `DownloadManager` component:
   - Download progress tracking
   - Checksum validation on completion
5. Context menu for files:
   - Download, Rename, Move, Delete
   - Copy link (for Public disk)
6. Skeleton loaders for directory listing.

### 4.5 CallService Implementation (Phase 14)

1. Create `communitas-ui-api/src/call.rs` with DTOs:
   - `CallState` enum (Idle, Connecting, InCall, Disconnected, MediaError)
   - `Participant` (id, display_name, is_muted, is_video_enabled, is_speaking, joined_at)
   - `CallInfo` (call_id, entity_id, participants, started_at, duration)
   - `MediaDevice` (id, name, device_type, is_default)
   - `DeviceType` enum (Microphone, Speaker, Camera)
   - `MediaError` (device_type, error_kind, message)

2. Create `communitas-ui-service/src/call.rs` with `CallService`:
   - `list_devices() -> Vec<MediaDevice>`
   - `select_device(device_id, device_type)`
   - `join_call(entity_id) -> Result<CallInfo, MediaError>`
   - `leave_call(call_id)`
   - `toggle_mute(call_id)`
   - `toggle_video(call_id)` (stretch goal)
   - `toggle_screen_share(call_id)` (stub for M4)
   - `get_participants(call_id) -> Vec<Participant>`
   - `get_call_state() -> CallState`
   - Watch channel for participant changes and call state

3. Add tracing spans: `ui.call.join`, `ui.call.leave`, `ui.call.mute`, `ui.call.device_select`, etc.

### 4.6 Dioxus Call UI (Phase 14 continued)

1. Create call button in entity header.
2. Create `DeviceSelector` component:
   - Microphone dropdown with test audio level
   - Speaker dropdown with test sound button
   - Camera dropdown (stretch goal)
   - Graceful fallback UI when devices unavailable
3. Create `CallLobby` component:
   - Participant grid with avatars
   - Mute/unmute button
   - Video toggle (stretch goal)
   - Share screen button (stub)
   - Leave call button
   - Call duration display
4. Create `ParticipantTile` component:
   - Avatar with presence ring
   - Name label
   - Mute indicator
   - Speaking indicator (audio level visualization)
   - Video feed placeholder (stretch goal)
5. Create `MediaErrorBanner` component:
   - Shows when media capture fails
   - Suggests permission fixes
   - Allows retry or fallback to listen-only mode
6. Audio-only MVP; video as stretch goal.

### 4.7 CanvasService Implementation (Phase 15)

1. Create `communitas-ui-api/src/canvas.rs` with DTOs:
   - `CanvasElement` (id, type, position, size, style, content, layer_id)
   - `ElementType` enum (Rectangle, Ellipse, Text, Line, Freehand, Image)
   - `CanvasState` (elements, layers, viewport, selection, history_position)
   - `Layer` (id, name, visible, locked, opacity)
   - `RemoteCursor` (user_id, display_name, position, color)
   - `HistoryEntry` (action_type, timestamp, description)
   - `OfflineOperation` (operation, queued_at, retry_count)

2. Create `communitas-ui-service/src/canvas.rs` with `CanvasService`:
   - `get_canvas(entity_id) -> CanvasState`
   - `add_element(entity_id, element) -> ElementId`
   - `update_element(entity_id, element_id, updates)`
   - `delete_element(entity_id, element_id)`
   - `set_viewport(entity_id, viewport)`
   - `create_layer(entity_id, name) -> LayerId`
   - `reorder_layers(entity_id, layer_ids)`
   - `undo(entity_id)` / `redo(entity_id)`
   - `get_history(entity_id) -> Vec<HistoryEntry>`
   - `scrub_history(entity_id, position)` — jump to history point
   - `get_remote_cursors(entity_id) -> Vec<RemoteCursor>`
   - `broadcast_cursor(entity_id, position)`
   - `get_offline_queue(entity_id) -> Vec<OfflineOperation>`
   - `retry_offline_operations(entity_id)`
   - Watch channel for canvas changes, cursor updates, offline queue

3. Add tracing spans: `ui.canvas.add_element`, `ui.canvas.undo`, `ui.canvas.sync`, etc.

### 4.8 Dioxus Canvas UI (Phase 15 continued)

1. Create `/canvas/:entity_id` route with:
   - Main canvas area (Wry default, Blitz behind `--features blitz-renderer` flag)
   - Toolbar: selection, shapes, text, freehand, pan/zoom
   - Layer panel: list, visibility toggles, reorder drag-drop
   - History scrubber: timeline slider, step through actions
2. Create `RemoteCursors` overlay:
   - Show other users' cursor positions with name labels
   - Color-coded by user
3. Create `OfflineQueueIndicator` component:
   - Badge showing pending operations count
   - Expandable list with retry buttons
   - Conflict resolution prompts
4. Keyboard shortcuts:
   - Ctrl+Z/Y for undo/redo
   - Delete for remove selection
   - Arrow keys for nudge
5. Feature flag: `blitz-renderer` for experimental Blitz rendering path.

### 4.9 MCP Parity & Testing (Phase 16)

1. Verify existing Kanban MCP tools (35+) work with new UI service layer.
2. Verify existing Drive MCP tools (6) align with DriveService.
3. Add call and canvas MCP tools:
   - `list_devices`, `join_call`, `leave_call`, `toggle_mute`
   - `get_canvas`, `add_element`, `update_element`, `delete_element`, `list_layers`
4. Create `scripts/tests/mcp_advanced_surfaces.sh` unified parity harness:
   - Kanban: Compare `list_boards`, `get_board` output against `export_boards` binary
   - Drive: Compare `list_directory` output, test upload/download round-trip
   - Calls: Verify device listing, call state transitions
   - Canvas: Compare `get_canvas` output, test element CRUD
5. Add WebDriver tests:
   - `tests/webdriverio/specs/kanban.smoke.js`: Board drag-and-drop, card creation
   - `tests/webdriverio/specs/drive.smoke.js`: File upload, download, preview
   - `tests/webdriverio/specs/call.smoke.js`: Call join/leave, device selection
   - `tests/webdriverio/specs/canvas.smoke.js`: Element creation, undo/redo, offline replay
6. Add performance scripts:
   - `scripts/tests/perf_kanban.sh`: Board drag latency (<50ms target)
   - `scripts/tests/perf_drive.sh`: Bulk file throughput (>10MB/s target)
   - `scripts/tests/perf_call.sh`: Multi-party call CPU/RAM profiling
   - `scripts/tests/perf_canvas.sh`: Canvas stroke lag (<16ms for 60fps)
7. Archive JSON diff artifacts in CI.

### 4.10 Documentation & CI Finalization (Phase 17)

1. Update ADR-020 addendum if renderer choice or plugin requirements change (file dialogs, notifications, device permissions).
2. Update `.github/workflows/rust.yml`:
   - Add advanced surfaces parity check step (`scripts/tests/mcp_advanced_surfaces.sh`)
   - Add WebDriver Kanban/Drive/Call/Canvas tests
   - Add performance benchmarks as CI artifacts
3. Update documentation:
   - `docs/architecture/dioxus_milestones.md`: Update M3 status
   - `docs/architecture/dioxus_desktop_prototype_plan.md`: Add new capabilities
   - `docs/architecture/MCP_THIN_GUI_ARCHITECTURE.md`: Add Kanban/Drive/Call/Canvas tools
4. Create testing documentation:
   - `docs/testing/mcp_advanced_surfaces_parity.md`: Unified parity docs
5. Update installer smoke checklist for Linux WebDriver CI; stage macOS/Windows verification when runners are available.

## 5. Validation Strategy

| Layer | Test | Notes |
| --- | --- | --- |
| Rust unit tests | `communitas-ui-service` kanban + drive + call + canvas modules | Cover happy/error paths, optimistic updates, offline queue. |
| Component tests | Dioxus SSR tests for board view, file browser, call lobby, canvas | Validate props/state, loading states. |
| Integration | WebDriver tests for drag-drop, file transfers, call lifecycle, canvas edits, offline replay | Use WebDriverIO. |
| MCP parity | `scripts/tests/mcp_advanced_surfaces.sh` verifies board/file/call/canvas operations match UI | Store JSON diffs as artifacts. |
| Performance | Board drag latency (<50ms), bulk file throughput (>10MB/s), multi-party call CPU/RAM, canvas stroke lag (<16ms) | Target <100ms local operations. |
| Accessibility | Keyboard focus, ARIA labels for all interactive elements | Documented in `tests/webdriverio/specs/accessibility.smoke.js`. |

## 6. Risks & Mitigations

| Risk | Mitigation |
| --- | --- |
| Drag-and-drop complexity in Dioxus | Start with simple implementation; fallback to click-to-move if needed. |
| Large file upload performance | Implement chunked uploads with progress; background uploads. |
| WebRTC browser compatibility | Audio-only MVP reduces complexity; defer video to M4. |
| Canvas CRDT sync complexity | Scope to basic shapes; defer advanced features or entire canvas to M4. |
| MCP/UI drift on board schema | Single source of truth types in `communitas-ui-api`; parity tests catch drift. |

## 7. Phase Summary

| Phase | Description | Status |
|-------|-------------|--------|
| **10** | KanbanService + UI API DTOs | Not started |
| **11** | Dioxus Kanban UI with drag-drop | Not started |
| **12** | DriveService + UI API DTOs | Not started |
| **13** | Dioxus Drive UI | Not started |
| **14** | CallService + Dioxus Call UI | Not started |
| **15** | CanvasService + UI (Scope TBD) | Not started |
| **16** | MCP Parity & Testing | Not started |
| **17** | Documentation & CI Finalization | Not started |

## 8. Validation Checklist & Evidence Capture

| Area | Test / Evidence | Command / Tooling | Artifact | Status |
| --- | --- | --- | --- | --- |
| KanbanService | Unit tests for board list, card CRUD, move operations | `cargo test -p communitas-ui-service kanban::tests` | Test report in CI | Pending |
| DriveService | Unit tests for directory listing, upload/download, checksum | `cargo test -p communitas-ui-service drive::tests` | Test report in CI | Pending |
| CallService | Unit tests for join/leave, mute, device selection | `cargo test -p communitas-ui-service call::tests` | Test report in CI | Pending |
| CanvasService | Unit tests for element CRUD, layers, history, offline queue | `cargo test -p communitas-ui-service canvas::tests` | Test report in CI | Pending |
| Dioxus components | SSR tests for board view, file browser, call lobby, canvas | `cargo test -p communitas-dioxus` | Screenshot diffs | Pending |
| Integration - Kanban | WebDriver tests for drag-drop, swimlane filters | `tests/webdriverio/specs/kanban.smoke.js` | Video/log trace | Pending |
| Integration - Drive | WebDriver tests for upload, download, preview | `tests/webdriverio/specs/drive.smoke.js` | Video/log trace | Pending |
| Integration - Call | WebDriver tests for join/leave, device selection | `tests/webdriverio/specs/call.smoke.js` | Video/log trace | Pending |
| Integration - Canvas | WebDriver tests for element creation, undo/redo, offline replay | `tests/webdriverio/specs/canvas.smoke.js` | Video/log trace | Pending |
| MCP parity | CLI harness verifies boards, files, calls, canvas match UI | `scripts/tests/mcp_advanced_surfaces.sh` | JSON diff artifact | Pending |
| Performance - Kanban | Board drag latency (<50ms) | `scripts/tests/perf_kanban.sh` | JSON perf report | Pending |
| Performance - Drive | Bulk file throughput (>10MB/s) | `scripts/tests/perf_drive.sh` | JSON perf report | Pending |
| Performance - Call | Multi-party call CPU/RAM profiling | `scripts/tests/perf_call.sh` | JSON perf report | Pending |
| Performance - Canvas | Canvas stroke lag (<16ms for 60fps) | `scripts/tests/perf_canvas.sh` | JSON perf report | Pending |
| Accessibility | Keyboard focus, ARIA labels for all surfaces | `tests/webdriverio/specs/accessibility.smoke.js` | Audit report | Pending |
| Telemetry | Trace log showing `ui.kanban.*`, `ui.drive.*`, `ui.call.*`, `ui.canvas.*` spans | `RUST_LOG=info dx serve` | Log snippet | Pending |

## 9. Dependencies & Prerequisites

### From Core Crates
- `communitas-kanban`: Complete (70+ tests, all types defined)
- `communitas-core` Drive commands: Partial (6 MCP tools exist)
- `communitas-core` WebRTC: Experimental (gossip signaling, 3 MCP tools)

### Required Before Start
- Milestone 2 complete (messaging, entities, contacts)
- ADR-019 UiServices pattern established
- MCP parity testing infrastructure in place

## Appendix: Key Files

### Implementation (to create)
- `communitas-ui-api/src/kanban.rs` — Kanban DTOs
- `communitas-ui-api/src/drive.rs` — Drive DTOs
- `communitas-ui-api/src/call.rs` — Call DTOs
- `communitas-ui-api/src/canvas.rs` — Canvas DTOs
- `communitas-ui-service/src/kanban.rs` — Kanban service
- `communitas-ui-service/src/drive.rs` — Drive service
- `communitas-ui-service/src/call.rs` — Call service
- `communitas-ui-service/src/canvas.rs` — Canvas service
- `communitas-dioxus/src/components/kanban/` — Kanban UI components
- `communitas-dioxus/src/components/drive/` — Drive UI components
- `communitas-dioxus/src/components/call/` — Call UI components
- `communitas-dioxus/src/components/canvas/` — Canvas UI components

### Existing Core
- `communitas-kanban/src/types.rs` — Kanban types (complete)
- `communitas-kanban/src/board.rs` — Board implementation
- `communitas-core/src/storage/` — Virtual disk implementation
- `communitas-mcp/src/tools.rs` — MCP tools (extend for UI parity)

### Tests (to create)
- `scripts/tests/mcp_advanced_surfaces.sh` — Unified parity harness (Kanban/Drive/Call/Canvas)
- `scripts/tests/perf_kanban.sh` — Kanban performance benchmarks
- `scripts/tests/perf_drive.sh` — Drive performance benchmarks
- `scripts/tests/perf_call.sh` — Call performance benchmarks
- `scripts/tests/perf_canvas.sh` — Canvas performance benchmarks
- `tests/webdriverio/specs/kanban.smoke.js` — Kanban WebDriver tests
- `tests/webdriverio/specs/drive.smoke.js` — Drive WebDriver tests
- `tests/webdriverio/specs/call.smoke.js` — Call WebDriver tests
- `tests/webdriverio/specs/canvas.smoke.js` — Canvas WebDriver tests

### Documentation (to update/create)
- `docs/architecture/dioxus_milestones.md` — Update M3 status
- `docs/architecture/dioxus_desktop_prototype_plan.md` — Add new capabilities
- `docs/architecture/MCP_THIN_GUI_ARCHITECTURE.md` — Add Kanban/Drive/Call/Canvas tools
- `docs/testing/mcp_advanced_surfaces_parity.md` — Unified parity documentation

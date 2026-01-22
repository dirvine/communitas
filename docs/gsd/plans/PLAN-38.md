# PLAN-38: Phase 6.6 - Kanban Polish (Continuation)

**Milestone**: M6 Beta-Ready (Apple Desktop)
**Phase**: 6.6 Kanban Polish
**Created**: 2026-01-22
**Status**: Planning

## Goal

Complete the remaining Kanban features for beta: due date handling, CRDT live updates, keyboard drag/drop, conflict banners, messaging integration, and analytics dashboard.

## Completed Tasks (Previous Session)

| Task | Description | Status |
|------|-------------|--------|
| 1 | Swimlane Rendering | Complete |
| 2 | Priority Levels | Complete |
| 3 | Wire Stubbed Features (Steps, Activity, Names) | Complete |

## Current State Analysis

Based on codebase exploration:

| Component | Status | Location |
|-----------|--------|----------|
| Due date types | Ready | `communitas-kanban/src/types.rs:241` |
| Due date filters | Ready | `communitas-kanban/src/filter.rs:43-48` |
| Due date UI | Missing | No date picker component |
| CRDT subscription | Missing | No Yrs observer in kanban |
| Mouse drag/drop | Partial | `column.rs:35-73` has TODO |
| Keyboard navigation | Missing | No keyboard drag/drop |
| Conflict banners | Placeholder | `board_view.rs:555-580` |
| Messaging integration | Missing | No card-to-thread linking |
| Analytics/burndown | Missing | No velocity metrics |

---

## Tasks

### Task 4: Due Date UI Components
**Files**: `communitas-dioxus/src/components/kanban/card.rs`, `communitas-dioxus/src/components/kanban/card_detail_modal.rs`, `communitas-dioxus/src/components/date_picker.rs` (new)

Add date picker and due date display for cards.

**What I'll do**:
1. Create `DatePicker` component with calendar dropdown
2. Add due date display to `CardView` (show days remaining or "Overdue")
3. Add date picker to `CardDetailModal` sidebar
4. Add overdue styling (red text/border for past due dates)
5. Add "Due Soon" indicator for cards due within 3 days
6. Wire `update_card()` with due_date changes

**Verification**:
- `cargo fmt --all -- --check`
- `cargo clippy -p communitas-dioxus --all-features -- -D warnings`
- `dx check --platform desktop`

**Done when**: Users can set/view/edit due dates with visual overdue indicators

---

### Task 5: Due Date Filtering
**Files**: `communitas-dioxus/src/components/kanban/filters.rs`, `communitas-ui-service/src/kanban.rs`

Add due date filter options to the filter bar.

**What I'll do**:
1. Add "Has Due Date" filter toggle
2. Add "Overdue" filter option (shows only past-due cards)
3. Add "Due This Week" filter option
4. Add "Due Today" filter option
5. Wire filters to `CardFilter` via service layer
6. Show due date count badge in filter dropdown

**Verification**:
- `cargo clippy -p communitas-dioxus -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service kanban`

**Done when**: Filter bar includes due date filtering options that work correctly

---

### Task 6: CRDT Live Update Subscription
**Files**: `communitas-kanban/src/service.rs`, `communitas-ui-service/src/kanban.rs`

Subscribe to Yrs document changes for real-time board updates.

**What I'll do**:
1. Add `observe_deep()` subscription to board YDoc in KanbanService
2. Create `KanbanEvent` enum (CardCreated, CardUpdated, CardMoved, ColumnChanged)
3. Add `subscribe_to_changes(board_id) -> Receiver<KanbanEvent>`
4. Wire UI service to listen for events and refresh relevant data
5. Use debouncing (50ms) to batch rapid changes
6. Handle subscription cleanup on board close

**Verification**:
- `cargo clippy -p communitas-kanban -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-kanban crdt`

**Done when**: Board UI updates in real-time when underlying CRDT changes

---

### Task 7: Complete Drag-Drop Implementation
**Files**: `communitas-dioxus/src/components/kanban/column.rs`, `communitas-dioxus/src/components/kanban/card.rs`

Finish the TODO in column.rs for drag data transfer.

**What I'll do**:
1. Implement `ondragstart` on cards to set drag data (card_id, source_column_id)
2. Implement `ondrop` on columns to read drag data and call `move_card()`
3. Add visual drag preview (ghost card)
4. Add drop position indicator (insertion line between cards)
5. Add drag handle icon for accessibility
6. Prevent dropping on source column at same position

**Verification**:
- `dx check --platform desktop`
- Manual testing of drag-drop between columns

**Done when**: Cards can be dragged between columns with visual feedback

---

### Task 8: Keyboard-Accessible Drag/Drop
**Files**: `communitas-dioxus/src/components/kanban/card.rs`, `communitas-dioxus/src/components/kanban/column.rs`

Add keyboard navigation for moving cards.

**What I'll do**:
1. Add `tabindex` and focus states to cards
2. Implement Ctrl+Arrow keys for card movement:
   - Ctrl+Left/Right: Move to adjacent column
   - Ctrl+Up/Down: Move within column
3. Add "Move to..." context menu (accessible via Enter/Space on focused card)
4. Show focus ring on keyboard-focused cards
5. Announce moves via aria-live region
6. Add keyboard shortcuts help tooltip

**Verification**:
- `dx check --platform desktop`
- Keyboard-only navigation test

**Done when**: Cards can be moved entirely via keyboard with screen reader announcements

---

### Task 9: Conflict Banner Implementation
**Files**: `communitas-dioxus/src/components/kanban/board_view.rs`, `communitas-ui-service/src/kanban.rs`

Replace placeholder ConflictBanner with working implementation.

**What I'll do**:
1. Detect concurrent edit conflicts from CRDT merge events
2. Track conflicting card IDs in board state
3. Show dismissible banner: "Card X was updated by another user"
4. Add "View Changes" button to open diff view
5. Add "Dismiss" button to clear conflict state
6. Auto-dismiss after 30 seconds or on card interaction

**Verification**:
- `cargo clippy -p communitas-dioxus --all-features -- -D warnings`
- `cargo test -p communitas-ui-service conflict`

**Done when**: Users see notification when their edits conflict with concurrent changes

---

### Task 10: Card-to-Message Thread Linking
**Files**: `communitas-kanban/src/types.rs`, `communitas-ui-service/src/kanban.rs`, `communitas-dioxus/src/components/kanban/card_detail_modal.rs`

Link kanban cards to message threads for discussions.

**What I'll do**:
1. Add `linked_thread_id: Option<String>` to Card type
2. Add "Link Discussion" button to CardDetailModal
3. Create thread picker component showing available threads
4. Show linked thread name and unread count on card
5. Add "Open Discussion" button that navigates to thread
6. Auto-create thread when first comment is added (optional)

**Verification**:
- `cargo clippy --all-features -- -D warnings`
- `cargo test -p communitas-kanban linked_thread`
- `cargo test -p communitas-ui-service kanban`

**Done when**: Cards can be linked to message threads with navigation

---

### Task 11: Analytics Dashboard - Data Model
**Files**: `communitas-kanban/src/analytics.rs` (new), `communitas-kanban/src/lib.rs`

Create analytics data structures and calculations.

**What I'll do**:
1. Create `BoardAnalytics` struct with:
   - Cards per column counts
   - Cards completed this week/month
   - Average time in each column
   - Cycle time distribution
2. Add `calculate_velocity(board_id, period)` - cards completed per week
3. Add `calculate_burndown(board_id, sprint_start, sprint_end)` - remaining work
4. Store completed_at timestamps for velocity calculation
5. Calculate WIP (work in progress) metrics

**Verification**:
- `cargo clippy -p communitas-kanban --all-features -- -D warnings`
- `cargo test -p communitas-kanban analytics`

**Done when**: Analytics calculations return accurate velocity and burndown data

---

### Task 12: Analytics Dashboard - UI
**Files**: `communitas-dioxus/src/components/kanban/analytics.rs` (new), `communitas-dioxus/src/components/kanban/mod.rs`

Create visual analytics dashboard.

**What I'll do**:
1. Create `AnalyticsDashboard` component
2. Add velocity chart (bar chart of cards completed per week)
3. Add burndown chart (line chart of remaining vs ideal)
4. Add column distribution pie chart
5. Add cycle time histogram
6. Add date range selector for filtering
7. Wire to KanbanService analytics methods

**Verification**:
- `dx check --platform desktop`
- Manual testing of chart rendering

**Done when**: Dashboard shows velocity, burndown, and distribution charts

---

### Task 13: Integration Tests
**Files**: `communitas-ui-service/tests/kanban_integration.rs`

Add integration tests for new features.

**What I'll do**:
1. Test due date filtering (overdue, due soon, has due date)
2. Test CRDT live update propagation
3. Test conflict detection and banner display
4. Test card-to-thread linking
5. Test analytics calculation accuracy
6. Test keyboard navigation flow

**Verification**:
- `cargo test -p communitas-ui-service kanban`

**Done when**: All Phase 6.6 features have integration test coverage

---

## Task Summary

| # | Task | Files | Est. Complexity |
|---|------|-------|-----------------|
| 4 | Due Date UI Components | 3 (1 new) | Medium |
| 5 | Due Date Filtering | 2 existing | Low |
| 6 | CRDT Live Update Subscription | 2 existing | High |
| 7 | Complete Drag-Drop Implementation | 2 existing | Medium |
| 8 | Keyboard-Accessible Drag/Drop | 2 existing | Medium |
| 9 | Conflict Banner Implementation | 2 existing | Medium |
| 10 | Card-to-Message Thread Linking | 3 existing | Medium |
| 11 | Analytics Dashboard - Data Model | 2 (1 new) | High |
| 12 | Analytics Dashboard - UI | 2 (1 new) | High |
| 13 | Integration Tests | 1 existing | Medium |

## Dependencies

- Task 4 should complete before Task 5 (date picker needed for filtering)
- Task 6 should complete before Task 9 (CRDT events needed for conflicts)
- Task 11 must complete before Task 12 (data model before UI)
- Tasks 4-12 should complete before Task 13 (integration tests)

## Success Criteria

- [ ] Due dates can be set, viewed, and filtered
- [ ] Overdue cards show visual indicators
- [ ] Board updates in real-time from CRDT changes
- [ ] Drag-drop works with both mouse and keyboard
- [ ] Conflict banners show when concurrent edits detected
- [ ] Cards can be linked to message threads
- [ ] Analytics dashboard shows velocity and burndown
- [ ] All features have integration tests
- [ ] Zero compilation warnings
- [ ] All tests pass

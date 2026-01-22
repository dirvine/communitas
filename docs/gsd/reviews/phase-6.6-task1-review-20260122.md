# Review Report: Phase 6.6 Task 1 - Swimlane Rendering

**Date**: 2026-01-22
**Phase**: 6.6 Kanban Polish
**Task**: 1 - Swimlane Rendering
**Verdict**: **BLOCKED** - Critical issues found

---

## Summary

| Agent | Critical | Important | Minor |
|-------|----------|-----------|-------|
| code-reviewer | 0 | 0 | 0 |
| silent-failure-hunter | 4 | 6 | 4 |
| code-simplifier | 0 | 2 | 3 |
| comment-analyzer | 0 | 2 | 0 |
| pr-test-analyzer | 3 | 4 | 2 |
| type-design-analyzer | 0 | 2 | 8 |
| security-reviewer | 3 | 2 | 0 |
| **TOTAL** | **10** | **18** | **17** |

---

## Critical Issues (Must Fix Before Proceeding)

### Security - CSS Injection via Unvalidated Tag Colors
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:303,444,468`
**Severity**: CRITICAL (Confidence: 90%)

Tag color values are directly injected into inline CSS without validation. An attacker could create a tag with malicious color value to inject arbitrary CSS.

**Attack Vector**:
```
#fff; position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
```

**Fix**: Add hex color validation:
```rust
fn validate_hex_color(color: &str) -> bool {
    color.len() >= 4 && color.len() <= 9 &&
    color.starts_with('#') &&
    color[1..].chars().all(|c| c.is_ascii_hexdigit())
}
```

---

### Security - Missing Board/Entity Authorization Checks
**File**: `communitas-ui-service/src/kanban.rs:425-495`
**Severity**: CRITICAL (Confidence: 85%)

The service verifies authentication but not authorization. A user who knows a board_id could access boards they don't own.

**Fix**: Add `check_entity_access()` helper and call before all board operations.

---

### Security - Missing Input Validation
**File**: `communitas-ui-service/src/kanban.rs:836-892,903-946,956-1012`
**Severity**: CRITICAL (Confidence: 88%)

No validation on length, format, or content of user-supplied strings. Could lead to DoS, storage bloat, or UI issues.

**Fix**: Add validation constants and `validate_non_empty_string()` function with max lengths.

---

### Silent Failure - Board Refresh Failure Silently Discarded
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:174-185`
**Severity**: CRITICAL

Board refresh failures are logged but not surfaced to user. User believes data is current when it may be stale.

**Fix**: Set error state on refresh failure and show error banner.

---

### Silent Failure - Column Creation Error No User Feedback
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:129-135`
**Severity**: CRITICAL

Column creation errors silently fail. User thinks column was created but it wasn't.

**Fix**: Display toast/error notification on column creation failure.

---

### Silent Failure - find_column_for_card Returns Empty on Failure
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:485-494`
**Severity**: CRITICAL

Returns empty string when card not found in any column. This hides data inconsistency.

**Fix**: Return Option<String> and handle None case with warning/placeholder.

---

### Test Coverage - Multi-Assignee Card Test Missing
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:253-261`
**Severity**: CRITICAL (Test Gap)

Cards with multiple assignees appearing in all swimlanes is untested. Critical path for swimlane feature.

**Fix**: Add test `group_by_assignee_card_with_multiple_assignees_appears_in_all`

---

### Test Coverage - Multi-Tag Card Test Missing
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:299-306`
**Severity**: CRITICAL (Test Gap)

Cards with multiple tags appearing in all swimlanes is untested.

**Fix**: Add test `group_by_tag_card_with_multiple_tags_appears_in_all`

---

### Test Coverage - set_swimlane_mode Subscriber Update Test Missing
**File**: `communitas-ui-service/src/kanban.rs:381-386`
**Severity**: CRITICAL (Test Gap)

The "switching swimlane mode re-renders board" requirement is untested.

**Fix**: Add test `set_swimlane_mode_updates_subscribers`

---

### Silent Failure - set_swimlane_mode Result Discarded
**File**: `communitas-ui-service/src/kanban.rs:384`
**Severity**: CRITICAL

`let _ = self.tx.send(snap)` discards send result. Subscribers may not receive update.

**Fix**: Log warning if send fails.

---

## Important Issues

### Type Design - SwimlaneViewProps Accepts None Mode
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:366-372`

`mode` field accepts `SwimlaneMode::None` but component semantically requires non-None.

**Suggestion**: Create `ActiveSwimlaneMode` enum without None variant.

---

### Type Design - Swimlane Key Has No Uniqueness Invariant
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:216-226`

Empty or duplicate keys could cause React-style key conflicts in UI rendering.

**Suggestion**: Add constructor with non-empty key validation.

---

### Code Simplifier - Manual Clone/PartialEq Implementations
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:426-435`

Manual `Clone` and `PartialEq` implementations duplicate derive behavior exactly.

**Suggestion**: Use `#[derive(Clone, PartialEq)]` instead.

---

### Code Simplifier - group_by_state is O(5n)
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:337-360`

Iterates through cards 5 times (once per state). Could be O(n) with single pass.

**Suggestion**: Use single-pass HashMap grouping.

---

### Comment Analyzer - "synced with service" Comment Misleading
**File**: `communitas-dioxus/src/components/kanban/board_view.rs`

Comment suggests bidirectional sync but it's one-way from service to UI.

**Suggestion**: Clarify comment to reflect actual behavior.

---

### Comment Analyzer - ConflictBanner is Placeholder
**File**: `communitas-dioxus/src/components/kanban/board_view.rs:555-580`

ConflictBanner component is placeholder with no documentation of intended behavior.

**Suggestion**: Add TODO comment with implementation plan or remove if not needed for Task 1.

---

### Security - No Rate Limiting on CRDT Operations
**File**: `communitas-ui-service/src/kanban.rs`

No rate limiting on board/card creation. Could enable DoS attacks.

**Suggestion**: Add rate limiter with MAX_MUTATIONS_PER_MINUTE.

---

### Security - Information Disclosure in Error Messages
**File**: `communitas-ui-service/src/kanban.rs:866-869`

Internal error details logged and potentially exposed.

**Suggestion**: Create sanitized error messages for logging.

---

## Minor Issues

- Swimlane sorting test missing
- State swimlane workflow order test missing
- Unassigned swimlane position assertion weak
- Trivial test `board_header_renders_name` provides no value
- `DueDateFilter::DueSoon` threshold undefined
- `FilterOption.id` can be empty
- `BoardFilters.search` uses Option vs empty Vec inconsistently
- `SwimlaneRowProps.columns` duplicates context data
- Label can be empty string in Swimlane struct

---

## Verdict: BLOCKED

**Reason**: 10 critical issues identified across security and silent failure categories. Per project ZERO TOLERANCE policy, all critical issues must be resolved before proceeding to Task 2.

**Required Actions**:
1. Fix CSS injection vulnerability (hex color validation)
2. Add authorization checks for board access
3. Add input validation with length limits
4. Surface error states to user (board refresh, column creation)
5. Add missing critical test coverage
6. Fix silent failure in set_swimlane_mode

**Estimated Fix Time**: 2-3 hours

---

## Files Changed in Task 1

| File | Changes |
|------|---------|
| `communitas-ui-api/src/kanban.rs` | SwimlaneMode enum, label(), Display impl |
| `communitas-ui-service/src/kanban.rs` | set_swimlane_mode(), swimlane_mode field |
| `communitas-dioxus/src/components/kanban/board_view.rs` | Swimlane struct, grouping functions, SwimlaneView, SwimlaneRow |
| `communitas-dioxus/src/components/kanban/filters.rs` | SwimlaneSelector component |

---

*Review generated by GSD Review Agents*
*7 agents, 45 findings total*

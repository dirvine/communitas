# Phase 6.4 (Calls & Presence) Code Review Report

**Date**: 2026-01-22
**Commits**: 40b4bbc to ea66e4d (11 commits)
**Files Changed**: 24 files, +9,741/-792 lines
**Review Agents**: 7 specialized agents in parallel

---

## Executive Summary

Phase 6.4 implements comprehensive Calls & Presence functionality with excellent architecture and test coverage. However, **2 critical issues** require immediate attention before deployment:

1. **State Synchronization Bug** - Participant lists desync after reconnection
2. **Silent Failures in UI** - 24 instances of swallowed errors across UI components

Overall quality: **8/10** - Production-ready after addressing critical issues.

---

## Critical Issues (Immediate Fix Required)

### 1. State Synchronization Bug in CallReconnected Handler

**Severity**: CRITICAL
**Confidence**: 95%
**Location**: `communitas-ui-service/src/call.rs:3142-3167`

**Problem**: The `Event::CallReconnected` handler clears remote participants from `state.participants` but fails to synchronize `state.current_call.participants`, causing the two participant lists to desynchronize.

**Code**:
```rust
// Line 3154 - Only updates one list
let local_participant_id = call.my_participant_id.clone();
state_guard.participants.retain(|p| p.id == local_participant_id);
// BUG: Missing update to call.participants
```

**Impact**:
- UI rendering inconsistencies (different participant counts in different components)
- Permission checks may fail
- Subsequent `ParticipantJoined` events may create duplicates

**Fix**:
```rust
let local_participant_id = call.my_participant_id.clone();
state_guard.participants.retain(|p| p.id == local_participant_id);
// Fix: Also update call.participants
call.participants.retain(|p| p.id == local_participant_id);
```

---

### 2. Silent Failures in UI Components (24 instances)

**Severity**: CRITICAL
**Confidence**: 95%
**Locations**: All Dioxus call components

**Problem**: Systematic use of `let _ =` discards errors from async call operations, meaning users receive no feedback when operations fail.

**Affected Components**:

| File | Line | Operation | User Impact |
|------|------|-----------|-------------|
| `call_controls.rs` | 72 | `toggle_mute()` | User thinks muted when not |
| `call_controls.rs` | 81 | `toggle_video()` | Camera state mismatch |
| `call_controls.rs` | 93-95 | `screen_share()` | Share fails silently |
| `call_controls.rs` | 105 | `leave_call()` | Can't leave call |
| `call_button.rs` | 68-70 | `join/leave_call()` | Nothing happens |
| `call_lobby.rs` | 67 | `join_call()` | UI transitions but not in call |
| `call_lobby.rs` | 52 | `list_devices()` | Empty device list |
| `call_view.rs` | 327,335 | `mute/leave` | Mini view unresponsive |
| `device_selector.rs` | 73-77 | `select_*()` | Device not switched |
| `media_error_banner.rs` | 97 | `retry_media()` | Retry fails silently |

**Recommended Fix**: Add error logging and toast notifications:
```rust
spawn(async move {
    if let Err(e) = call.toggle_mute().await {
        tracing::warn!("Failed to toggle mute: {e}");
        // TODO: Show toast notification to user
    }
});
```

---

## Important Issues

### 3. Nested Lock Acquisition in Event Loop

**Severity**: MEDIUM
**Confidence**: 80%
**Location**: `communitas-ui-service/src/call.rs:3173-3194`

**Problem**: `Event::CallEnded` handler acquires `state` write lock, then `history` write lock while holding state lock.

**Impact**: Reduces concurrency and creates potential for future deadlocks.

**Fix**: Drop state lock before acquiring history lock.

---

### 4. Duplicate Participant Data in CallSnapshot

**Severity**: LOW
**Location**: `communitas-ui-api/src/call.rs:768-848`

**Problem**: `CallSnapshot.participants` duplicates `call_info.participants`, creating sync risk.

**Recommendation**: Remove duplicate field; use `participants()` method delegating to `call_info`.

---

## Code Simplification Opportunities

### 1. Extract Call State Subscription Hook (HIGH IMPACT)

**Current**: 15-line subscription boilerplate repeated in 6+ components
**Suggested**: Create `use_call_state()` hook

```rust
// New file: hooks/use_call_state.rs
pub fn use_call_state() -> Signal<CallSnapshot> {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();
    let mut snapshot = use_signal(|| call_service.current_snapshot());

    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe();
            loop {
                if rx.changed().await.is_err() { break; }
                snapshot.set(rx.borrow().clone());
            }
        }
    });
    snapshot
}
```

**Impact**: Removes ~90+ lines of duplicated code.

### 2. Extract Participant Update Helper

**Current**: Repetitive participant state updates for mute/video/screen share/hand raised
**Suggested**: Helper function for updating both participant lists

---

## Test Coverage Analysis

**Overall Coverage**: 8.5/10 - Excellent

| Task | Coverage | Notes |
|------|----------|-------|
| Task 1: Device Enumeration | 9/10 | Comprehensive mock/real testing |
| Task 2: Call UI Components | 7/10 | Service layer well-tested |
| Task 3: Presence Indicators | 8/10 | Good participant state coverage |
| Task 4: Quality Metrics | 9/10 | Excellent threshold testing |
| Task 5: Recording Toggle | 8/10 | Minor: no full lifecycle test |
| Task 6: Group Call Support | 9/10 | Role permissions well-tested |
| Task 7: Call History | 10/10 | Complete persistence coverage |
| Task 8: Missed Calls | 9/10 | Notifications, acknowledgment |
| Task 9: MCP Tools | 9/10 | Full parity tests |
| Task 10: Integration | 8/10 | 60+ integration tests |

**Test Files**:
- `communitas-ui-service/tests/call_integration.rs` (1,628 lines)
- `communitas-ui-service/src/call.rs` tests (75+ unit tests)
- `communitas-ui-api/src/call.rs` tests (40+ API tests)
- `communitas-mcp/tests/parity_test.rs` (30+ parity tests)

---

## Type Design Analysis

| Type | Encapsulation | Invariants | Usefulness | Notes |
|------|--------------|------------|------------|-------|
| `CallState` | 5/5 | 4/5 | 5/5 | Excellent FSM enum |
| `CallSnapshot` | 2/5 | 2/5 | 5/5 | Duplicate participants field |
| `CallInfo` | 3/5 | 3/5 | 5/5 | Good helpers, some pub fields |
| `ParticipantInfo` | 4/5 | 3/5 | 5/5 | Well-structured |
| `CallHistoryEntry` | 4/5 | 4/5 | 5/5 | Good finalization pattern |
| `CallQualityMetrics` | 4/5 | 4/5 | 5/5 | Clear thresholds |

---

## Security Notes

**Positive Findings**:
- No `unwrap()`, `expect()`, or `panic!()` in production code
- Resource limits in place (max_participants, history limits)
- MCP input validation via `require_str!` macro
- Rate limiting considerations

**Areas for Future Review**:
- MCP parameter content validation (presence-only currently)
- Recording consent enforcement
- Screen share permission model

---

## Documentation Quality

**Strengths**:
- Comprehensive doc comments on public APIs
- User guide covers all call features
- MCP API documentation complete

**Minor Improvements**:
- Add inline comments explaining TOCTOU prevention in screen share toggle
- Document lock ordering requirements

---

## Recommendations

### Immediate (Before Deployment)

1. **Fix CallReconnected participant sync bug**
2. **Add error handling to UI async operations**

### Short-term

3. Extract `use_call_state()` hook to reduce duplication
4. Refactor nested lock acquisition in CallEnded handler
5. Remove duplicate `participants` field from `CallSnapshot`

### Long-term

6. Add E2E tests for full call lifecycle
7. Consider builder pattern for `CallSnapshot`
8. Add toast notification system for error feedback

---

## Files Reviewed

- `communitas-ui-service/src/call.rs` (4541 lines)
- `communitas-ui-api/src/call.rs` (2034 lines)
- `communitas-dioxus/src/components/call/*.rs` (8 files)
- `communitas-mcp/src/tools.rs` (call tools section)
- `docs/api/mcp-api.md`
- `docs/user-guide/calls.md`
- `docs/architecture/webrtc-multimedia.md`

---

*Review conducted by 7 parallel GSD review agents on 2026-01-22*

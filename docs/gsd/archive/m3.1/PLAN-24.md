# PLAN-24: Phase 1.2 — Read Operations

**Milestone**: M3.1 Remediation
**Phase**: 1.2 (Read Operations)
**Status**: Pending
**Created**: 2026-01-19
**Depends on**: PLAN-23 (Foundation)

---

## Overview

Implement `list_threads()` and `get_messages()` using real `CommunitasApp` queries.

## Prerequisites

- [ ] PLAN-23 complete (MessagingService has CommunitasApp)
- [ ] Type conversion module exists
- [ ] Core Query types are accessible

---

## Tasks

<task type="auto" priority="p1">
  <n>Implement list_threads with real data</n>
  <files>
    communitas-ui-service/src/messaging.rs,
    communitas-core/src/query.rs (reference only)
  </files>
  <action>
    1. Get authenticated peer's four-word ID from AuthController
    2. Query directory service for entities user has joined:
       - app.query(Query::ListEntities { ... }) or equivalent
    3. For each entity, get latest message as preview:
       - app.query(Query::GetEntityMessages { entity_id })
       - Take first message as preview
    4. Build Vec<ThreadSummary> with:
       - thread_id = entity_id
       - name = entity name
       - preview = latest message text (truncated)
       - timestamp = latest message timestamp
       - unread_count = calculate from local state
       - entity_type = from core
    5. Update watch channel with new threads
    6. Return threads

    If Query::ListThreads doesn't exist in core, compose from:
    - Directory/entity listing + per-entity message query

    Handle empty results gracefully (return empty vec, not error).
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - list_threads returns real entity data
    - Preview text comes from actual messages
    - Watch channel updates with thread list
    - Returns empty vec for new users (no error)
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement get_messages with pagination</n>
  <files>
    communitas-ui-service/src/messaging.rs,
    communitas-ui-service/src/messaging_convert.rs
  </files>
  <action>
    1. Call app.query(Query::GetEntityMessages { entity_id })
    2. Handle QueryResponse::Messages(msgs) result
    3. Convert each MessageResponse to UI Message type using conversion module
    4. Apply pagination:
       - Filter messages where timestamp < before (if provided)
       - Take up to `limit` messages
       - Sort by timestamp descending
    5. Return Vec<Message>

    Handle thread_id not found:
    - Return MessagingError::ThreadNotFound(thread_id)

    Handle empty thread:
    - Return empty vec (valid state)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - get_messages returns real messages for valid thread_id
    - Pagination works (limit, before cursor)
    - Messages sorted by timestamp
    - ThreadNotFound error for invalid thread_id
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add unit tests for read operations</n>
  <files>
    communitas-ui-service/src/messaging.rs (test module)
  </files>
  <action>
    1. Add test: test_list_threads_empty_for_new_user
       - Create fresh MessagingService with mock app
       - Verify list_threads returns empty vec
    2. Add test: test_list_threads_returns_joined_entities
       - Setup app with test entities
       - Verify threads match entities
    3. Add test: test_get_messages_returns_ordered
       - Setup thread with multiple messages
       - Verify messages returned in timestamp order
    4. Add test: test_get_messages_pagination
       - Verify limit parameter respected
       - Verify before cursor filters correctly
    5. Add test: test_get_messages_thread_not_found
       - Verify error for invalid thread_id

    Use test helpers/mocks from existing test module.
    Tests can use unwrap/expect for clarity.
  </action>
  <verify>
    cargo test -p communitas-ui-service -- --test-threads=1
  </verify>
  <done>
    - All 5 tests pass
    - Tests cover happy path and error cases
    - No flaky tests
  </done>
</task>

---

## Exit Criteria

- [ ] `list_threads()` returns real entity/message data
- [ ] `get_messages()` returns real messages with pagination
- [ ] Unit tests pass
- [ ] No clippy warnings

---

## Notes

- Core may not have dedicated ListThreads query - compose from existing queries
- Pagination is client-side for now (can optimize later with core support)
- Unread count tracking may need additional state management

---

## Next

Phase 1.3: Write Operations (send, edit, delete)

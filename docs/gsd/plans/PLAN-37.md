# PLAN-37: Phase 6.2 - Messaging & Contacts

**Milestone**: M6 Beta-Ready (Apple Desktop)
**Phase**: 6.2 Messaging & Contacts
**Created**: 2026-01-21
**Status**: Planning

## Goal

Complete the messaging experience with DM threads, presence indicators, typing indicators, unread counts, and search functionality. Achieve full parity between UI service layer and MCP tools.

## Current State Analysis

Based on codebase exploration:

| Component | Status | Location |
|-----------|--------|----------|
| MessagingService | Production (partial) | `communitas-ui-service/src/messaging.rs` |
| Message CRUD | Complete | Send, edit, delete, react all work |
| Thread pagination | Complete | Cursor-based before/after |
| Message sync/CRDT | Complete | `communitas-core/src/message_service.rs` |
| Offline queue | Complete | `communitas-core/src/crdt/offline_queue.rs` |
| DM infrastructure | Commands exist | `SendDirectMessage`, `GetDirectMessages` |
| Unread counts | Stubbed (hardcoded 0) | `messaging.rs:191` |
| Mark read | Stubbed (TODO) | `messaging.rs:542` |
| Typing indicators | Missing | No infrastructure |
| Message search | Missing | No queries or indexing |
| Presence | Group-scoped only | `communitas-core/src/gossip/presence.rs` |
| Pinned chats | Missing | No support |

## Tasks

### Task 1: Wire DM Threads to UI Service
**Files**: `communitas-ui-service/src/messaging.rs`, `communitas-ui-api/src/messaging.rs`

Surface direct message threads in the thread list.

**What I'll do**:
1. Add `is_dm: bool` field to `ThreadSummary`
2. Modify `fetch_threads()` to also query direct message threads via `GetDirectMessages`
3. Create `dm:` pseudo-entity IDs for contact-based threads
4. Map contact four_words to thread display names
5. Sort DM threads alongside entity threads by last_message_timestamp

**Verification**:
- `cargo fmt --all -- --check`
- `cargo clippy -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service messaging`

**Done when**: DM threads appear in thread list with contact names

---

### Task 2: Unread Count Persistence
**Files**: `communitas-core/src/command.rs`, `communitas-core/src/crdt_manager.rs`, `communitas-ui-service/src/messaging.rs`

Implement persistent unread count tracking.

**What I'll do**:
1. Add `unread_counts: HashMap<String, u32>` to CRDT metadata per identity
2. Add `Command::IncrementUnreadCount { thread_id }`
3. Add `Command::ResetUnreadCount { thread_id }`
4. Hook `MessageReceived` event to increment unread (if not current thread)
5. Update `fetch_threads()` to read persisted unread counts
6. Store unread state in `~/.communitas/unread.json` (simple persistence)

**Verification**:
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core unread`
- `cargo test -p communitas-ui-service unread`

**Done when**: Unread counts persist across restarts and increment on new messages

---

### Task 3: Mark Thread Read Infrastructure
**Files**: `communitas-core/src/command.rs`, `communitas-ui-service/src/messaging.rs`

Complete the TODO at messaging.rs:542.

**What I'll do**:
1. Add `Command::MarkThreadRead { thread_id, identity }` to core
2. Wire UI `mark_thread_read()` to execute this command
3. Reset unread count in persistent storage
4. Emit `Event::ThreadMarkedRead { thread_id }` for subscribers
5. Update thread summary reactively on mark read

**Verification**:
- `cargo clippy --all-features -- -D warnings`
- `cargo test -p communitas-ui-service mark_read`

**Done when**: Opening a thread clears its unread count persistently

---

### Task 4: Typing Indicator Infrastructure
**Files**: `communitas-core/src/command.rs`, `communitas-core/src/event.rs`, `communitas-core/src/gossip/mod.rs`

Add typing indicator commands and events.

**What I'll do**:
1. Add `Command::SendTypingIndicator { thread_id, is_typing: bool }`
2. Add `Event::TypingIndicatorReceived { thread_id, peer_id, is_typing }`
3. Create gossip message type for typing indicators
4. Add 3-second auto-expire for typing state
5. Rate limit typing broadcasts (max 1 per second per thread)

**Verification**:
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core typing`

**Done when**: Typing indicators can be sent and received via gossip

---

### Task 5: Typing Indicator UI Service
**Files**: `communitas-ui-service/src/messaging.rs`, `communitas-ui-api/src/messaging.rs`

Expose typing indicators to UI.

**What I'll do**:
1. Add `typing_users: Vec<String>` to `ThreadSummary`
2. Add `send_typing_indicator(thread_id)` to MessagingService
3. Track typing state per thread with auto-expire (3 seconds)
4. Subscribe to `TypingIndicatorReceived` events
5. Debounce typing broadcasts on keystroke (500ms)

**Verification**:
- `cargo clippy -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service typing`

**Done when**: UI can show "Alice is typing..." indicators

---

### Task 6: Typing Indicator Dioxus Component
**Files**: `communitas-dioxus/src/components/typing_indicator.rs` (new), `communitas-dioxus/src/components/mod.rs`

Create UI component for typing indicators.

**What I'll do**:
1. Create `TypingIndicator` component showing animated dots
2. Display user names: "Alice is typing..." or "Alice and Bob are typing..."
3. Handle 3+ users: "3 people are typing..."
4. Add subtle animation for dots (CSS keyframes)
5. Position below message input

**Verification**:
- `dx check --platform desktop`
- Manual testing of typing display

**Done when**: Users see animated typing indicator in chat

---

### Task 7: Message Search Infrastructure
**Files**: `communitas-core/src/command.rs`, `communitas-core/src/query.rs`, `communitas-core/src/message_service.rs`

Add full-text message search.

**What I'll do**:
1. Add `Query::SearchMessages { query, thread_id: Option, limit }`
2. Implement simple substring search across message text
3. Support thread-scoped search (within one thread)
4. Support global search (all threads)
5. Return results with context (matched text highlighted)
6. Sort by relevance (match count) then recency

**Verification**:
- `cargo clippy -p communitas-core --all-features -- -D warnings`
- `cargo test -p communitas-core search`

**Done when**: Messages can be searched by text content

---

### Task 8: Message Search UI Service
**Files**: `communitas-ui-service/src/messaging.rs`, `communitas-ui-api/src/messaging.rs`

Expose search to UI layer.

**What I'll do**:
1. Add `search_messages(query, thread_id)` to MessagingService
2. Add `SearchResult` type with message + context
3. Debounce search queries (300ms)
4. Cache recent search results
5. Limit results to 50 for performance

**Verification**:
- `cargo clippy -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service search`

**Done when**: UI service can search messages

---

### Task 9: Search Dioxus Component
**Files**: `communitas-dioxus/src/components/message_search.rs` (new), `communitas-dioxus/src/components/mod.rs`

Create search UI component.

**What I'll do**:
1. Create `MessageSearch` component with search input
2. Display search results with highlighted matches
3. Click result to jump to message in thread
4. Show thread name for global search results
5. Add keyboard navigation (up/down arrows, enter to select)
6. Clear search on escape key

**Verification**:
- `dx check --platform desktop`
- Manual testing of search flow

**Done when**: Users can search and navigate to messages

---

### Task 10: Pinned Chats Infrastructure
**Files**: `communitas-core/src/command.rs`, `communitas-ui-service/src/messaging.rs`

Add chat pinning support.

**What I'll do**:
1. Add `Command::PinThread { thread_id }` and `UnpinThread`
2. Store pinned threads in app config (similar to recent identities)
3. Add `is_pinned: bool` to `ThreadSummary`
4. Sort pinned threads above unpinned in list
5. Limit to 5 pinned threads

**Verification**:
- `cargo clippy --all-features -- -D warnings`
- `cargo test -p communitas-ui-service pinned`

**Done when**: Users can pin/unpin threads, pinned appear at top

---

### Task 11: Contact Presence Indicators
**Files**: `communitas-ui-service/src/presence.rs`, `communitas-ui-service/src/messaging.rs`

Integrate presence into thread list.

**What I'll do**:
1. Add `contact_status: Option<PresenceStatus>` to `ThreadSummary`
2. For DM threads, include contact's presence status
3. Subscribe to presence changes and update thread summaries
4. Show green/yellow/gray dot for online/away/offline

**Verification**:
- `cargo clippy -p communitas-ui-service --all-features -- -D warnings`
- `cargo test -p communitas-ui-service presence`

**Done when**: DM threads show contact online status

---

### Task 12: Offline Send Queue Integration
**Files**: `communitas-ui-service/src/messaging.rs`, `communitas-core/src/crdt/offline_queue.rs`

Wire offline queue to messaging UI.

**What I'll do**:
1. Detect offline state in `send_message()`
2. Queue messages to `OfflineQueue` when offline
3. Add `pending: bool` flag to `Message` type
4. Show pending indicator in UI (clock icon)
5. Auto-retry on reconnection
6. Handle failed sends with error state

**Verification**:
- `cargo clippy --all-features -- -D warnings`
- `cargo test -p communitas-ui-service offline`

**Done when**: Messages queue when offline and send on reconnect

---

### Task 13: MCP Messaging Tools Parity
**Files**: `communitas-mcp/src/tools.rs`

Ensure MCP tools match UI service capabilities.

**What I'll do**:
1. Add `search_messages` tool
2. Add `send_typing_indicator` tool
3. Add `mark_thread_read` tool
4. Add `pin_thread` / `unpin_thread` tools
5. Add `get_unread_counts` tool
6. Update `list_threads` to include unread counts, is_pinned, presence

**Verification**:
- `cargo clippy -p communitas-mcp --all-features -- -D warnings`
- `cargo test -p communitas-mcp messaging`

**Done when**: All messaging features available via MCP

---

### Task 14: Integration Tests
**Files**: `communitas-ui-service/tests/messaging_integration.rs` (new)

Comprehensive integration tests for messaging features.

**What I'll do**:
1. Test DM thread creation and listing
2. Test unread count increment and reset
3. Test typing indicator send/receive cycle
4. Test message search with multiple threads
5. Test pin/unpin with sort order
6. Test offline queue with mock disconnect
7. Test presence integration with threads

**Verification**:
- `cargo test -p communitas-ui-service messaging_integration`

**Done when**: All Phase 6.2 features have integration test coverage

---

## Task Summary

| # | Task | Files | Est. Complexity |
|---|------|-------|-----------------|
| 1 | Wire DM Threads to UI Service | 2 existing | Medium |
| 2 | Unread Count Persistence | 3 existing | Medium |
| 3 | Mark Thread Read Infrastructure | 2 existing | Low |
| 4 | Typing Indicator Infrastructure | 3 existing | Medium |
| 5 | Typing Indicator UI Service | 2 existing | Low |
| 6 | Typing Indicator Dioxus Component | 2 new | Medium |
| 7 | Message Search Infrastructure | 3 existing | Medium |
| 8 | Message Search UI Service | 2 existing | Low |
| 9 | Search Dioxus Component | 2 new | Medium |
| 10 | Pinned Chats Infrastructure | 2 existing | Low |
| 11 | Contact Presence Indicators | 2 existing | Low |
| 12 | Offline Send Queue Integration | 2 existing | Medium |
| 13 | MCP Messaging Tools Parity | 1 existing | Medium |
| 14 | Integration Tests | 1 new | Medium |

## Dependencies

- Task 2 must complete before Task 3 (unread counts needed for mark read)
- Task 4 must complete before Tasks 5, 6 (core infrastructure first)
- Task 7 must complete before Tasks 8, 9 (search infrastructure first)
- Tasks 1-12 should complete before Task 14 (integration tests)

## Success Criteria

- [ ] DM threads appear in thread list with contact names
- [ ] Unread counts persist and update correctly
- [ ] Typing indicators show when contacts are typing
- [ ] Messages can be searched globally and per-thread
- [ ] Threads can be pinned and appear at top
- [ ] Contact presence shown in DM threads
- [ ] Messages queue offline and send on reconnect
- [ ] MCP tools have full parity with UI
- [ ] All features have integration tests
- [ ] Zero compilation warnings
- [ ] All tests pass

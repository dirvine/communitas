# PLAN-26: Phase 1.4 — Reactions, Events & Integration Testing

**Milestone**: M3.1 Remediation
**Phase**: 1.4 (Reactions + Events + Testing)
**Status**: Pending
**Created**: 2026-01-19
**Depends on**: PLAN-25 (Write Operations)

---

## Overview

Complete Phase 1 by implementing reactions, event subscriptions for reactive updates, integration tests, and MCP verification.

## Prerequisites

- [ ] PLAN-25 complete (CRUD operations work)
- [ ] CommunitasApp event subscription available
- [ ] MCP tools accessible

---

## Tasks

<task type="auto" priority="p1">
  <n>Implement reactions (add/remove)</n>
  <files>
    communitas-ui-service/src/messaging.rs
  </files>
  <action>
    1. Implement add_reaction:
       - Get peer_id from AuthController
       - Determine entity_type from thread_id
       - Build Command::AddReaction { entity_id, entity_type, message_id, emoji }
       - Execute command
       - Return Ok(())

    2. Implement remove_reaction:
       - Same pattern as add_reaction
       - Build Command::RemoveReaction
       - Execute and return

    3. Ensure reactions appear in get_messages:
       - MessageResponse includes reactions field
       - Conversion module handles ReactionResponse -> UI format

    Error handling:
    - Invalid emoji -> MessagingError::Internal
    - Message not found -> MessagingError::Internal
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - add_reaction persists reaction
    - remove_reaction removes reaction
    - Reactions visible in get_messages response
    - user_reacted flag correct for current user
  </done>
</task>

<task type="auto" priority="p1">
  <n>Subscribe to events for reactive updates</n>
  <files>
    communitas-ui-service/src/messaging.rs
  </files>
  <action>
    1. In MessagingService::new() or separate init method:
       - Subscribe to CommunitasApp: app.subscribe(Subscription::Messages)
       - Get broadcast::Receiver<Event>

    2. Spawn background task to process events:
       ```rust
       tokio::spawn(async move {
           while let Ok(event) = event_rx.recv().await {
               match event {
                   Event::MessageSent { .. } |
                   Event::MessageReceived { .. } |
                   Event::MessageDeleted { .. } |
                   Event::MessageEdited { .. } |
                   Event::ReactionAdded { .. } |
                   Event::ReactionRemoved { .. } => {
                       // Re-fetch threads and update watch channel
                       self.refresh_threads().await;
                   }
                   _ => {}
               }
           }
       });
       ```

    3. Add refresh_threads() helper:
       - Re-query list_threads
       - Update watch channel via tx.send()

    4. Ensure watch channel propagates to UI reactively

    Use weak reference to self in spawned task to avoid reference cycles.
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
  </verify>
  <done>
    - Event subscription established on service creation
    - Background task processes message events
    - Watch channel updates on events
    - UI reactively updates without manual refresh
  </done>
</task>

<task type="auto" priority="p1">
  <n>Integration tests and MCP verification</n>
  <files>
    communitas-ui-service/tests/messaging_integration.rs,
    communitas-mcp/src/tools.rs (verification only)
  </files>
  <action>
    1. Create integration test file with real CommunitasApp:
       ```rust
       #[tokio::test]
       async fn test_full_messaging_flow() {
           // Setup real app and services
           // Test: send -> get -> edit -> get -> delete -> get
           // Verify each step
       }

       #[tokio::test]
       async fn test_reactions_flow() {
           // Send message, add reaction, verify, remove, verify
       }

       #[tokio::test]
       async fn test_watch_channel_updates() {
           // Subscribe to watch channel
           // Send message
           // Verify channel received update
       }
       ```

    2. Verify MCP tools use MessagingService:
       - Find send_message MCP tool in tools.rs
       - Verify it calls messaging.send_message()
       - Same for list_threads, get_messages, edit, delete, react
       - If tools bypass MessagingService, update them

    3. Run MCP parity test:
       - scripts/tests/mcp_messaging.sh (if it exists and is relevant)
       - Verify MCP and Dioxus produce same results

    Tests can use unwrap/expect for clarity.
    Integration tests may need longer timeouts.
  </action>
  <verify>
    cargo test -p communitas-ui-service --test messaging_integration
    cargo build -p communitas-mcp
    scripts/tests/mcp_messaging.sh (if applicable)
  </verify>
  <done>
    - Integration tests pass
    - MCP tools use MessagingService methods
    - MCP parity test passes (or is updated to use real data)
    - Full E2E flow verified
  </done>
</task>

---

## Exit Criteria

- [ ] Reactions persist and are visible
- [ ] Events trigger watch channel updates
- [ ] Integration tests pass
- [ ] MCP tools verified to use same code path
- [ ] Phase 1 complete - messaging fully wired

---

## Phase 1 Summary

After PLAN-26 completes, MessagingService will:
- ✅ Use real CommunitasApp queries/commands
- ✅ Support full CRUD (send/edit/delete)
- ✅ Support reactions (add/remove)
- ✅ Update reactively via event subscriptions
- ✅ Have integration test coverage
- ✅ Share code path with MCP tools

---

## Next

Phase 2: MCP Parity Harness + Documentation Update

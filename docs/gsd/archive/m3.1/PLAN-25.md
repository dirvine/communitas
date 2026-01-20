# PLAN-25: Phase 1.3 — Write Operations

**Milestone**: M3.1 Remediation
**Phase**: 1.3 (Write Operations)
**Status**: Pending
**Created**: 2026-01-19
**Depends on**: PLAN-24 (Read Operations)

---

## Overview

Implement `send_message()`, `edit_message()`, and `delete_message()` using real `CommunitasApp` commands.

## Prerequisites

- [ ] PLAN-24 complete (read operations work)
- [ ] Type conversion module works
- [ ] Core Command types are accessible

---

## Tasks

<task type="auto" priority="p1">
  <n>Implement send_message</n>
  <files>
    communitas-ui-service/src/messaging.rs
  </files>
  <action>
    1. Get authenticated user info (peer_id, display_name) from AuthController
    2. Determine entity_type from thread_id (may need lookup or convention)
    3. Build Command::SendMessage:
       ```rust
       Command::SendMessage {
           entity_id: thread_id.to_string(),
           entity_type,
           text: text.to_string(),
           author: display_name,
           reply_to_id: reply_to.map(|s| s.to_string()),
           attachments: None,
       }
       ```
    4. Execute: `app.execute(cmd).await`
    5. Handle result:
       - Extract Event::MessageSent from returned events
       - Get message_id from event
       - Query the new message to return full Message struct
    6. Return Message with populated fields

    Error handling:
    - Map core errors to MessagingError::SendFailed
    - Include error context in message
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - send_message creates real message in core
    - Message visible in subsequent get_messages call
    - Returns populated Message struct
    - Errors mapped appropriately
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement edit_message</n>
  <files>
    communitas-ui-service/src/messaging.rs
  </files>
  <action>
    1. Determine entity_type from thread_id
    2. Build Command::EditMessage:
       ```rust
       Command::EditMessage {
           entity_id: thread_id.to_string(),
           entity_type,
           message_id: message_id.to_string(),
           new_text: new_text.to_string(),
       }
       ```
    3. Execute command
    4. Handle result:
       - Verify MessageEdited event
       - Query updated message
    5. Return updated Message

    Error handling:
    - Message not found -> MessagingError::Internal
    - Permission denied -> MessagingError::Internal with context
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - edit_message modifies message text
    - edited_at timestamp updated
    - Change visible in get_messages
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement delete_message</n>
  <files>
    communitas-ui-service/src/messaging.rs
  </files>
  <action>
    1. Determine entity_type from thread_id
    2. Build Command::DeleteMessage:
       ```rust
       Command::DeleteMessage {
           entity_id: thread_id.to_string(),
           entity_type,
           message_id: message_id.to_string(),
       }
       ```
    3. Execute command
    4. Verify MessageDeleted event
    5. Return Ok(())

    Error handling:
    - Message not found -> MessagingError::Internal
    - Already deleted -> Ok(()) (idempotent)
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - delete_message removes message
    - Message not visible in get_messages
    - Idempotent (deleting twice is ok)
  </done>
</task>

---

## Exit Criteria

- [ ] `send_message()` creates real messages
- [ ] `edit_message()` modifies messages
- [ ] `delete_message()` removes messages
- [ ] All operations visible in read operations
- [ ] No clippy warnings

---

## Notes

- entity_type determination may need a lookup table or convention (e.g., thread_id prefix)
- Consider adding a helper function to resolve entity_type from thread_id
- Delete is soft delete in CRDT system (marks as deleted, doesn't remove)

---

## Next

Phase 1.4: Reactions, Events & Testing

# PLAN-23: Phase 1.1 — MessagingService Foundation

**Milestone**: M3.1 Remediation
**Phase**: 1.1 (Foundation)
**Status**: Ready
**Created**: 2026-01-19

---

## Overview

Add `CommunitasApp` to `MessagingService` and create type conversion utilities. This is the foundation for all subsequent messaging wiring.

## Prerequisites

- [ ] `communitas-core` builds successfully
- [ ] `communitas-ui-service` builds successfully
- [ ] Core Command/Query types are accessible

---

## Tasks

<task type="auto" priority="p0">
  <n>Add CommunitasApp to MessagingService</n>
  <files>
    communitas-ui-service/src/messaging.rs,
    communitas-ui-service/src/lib.rs
  </files>
  <action>
    1. Add `app: Arc<CommunitasApp>` field to MessagingService struct
    2. Update constructor: `pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self`
    3. Store app reference in struct
    4. Update any callers (UiServices builder) to pass app
    5. Ensure CommunitasApp is re-exported or imported properly

    Use Arc for shared ownership across async tasks.
    Do not use unwrap/expect - propagate errors with `?`.
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
  </verify>
  <done>
    - MessagingService has app: Arc<CommunitasApp> field
    - Constructor accepts app parameter
    - Crate compiles without warnings
  </done>
</task>

<task type="auto" priority="p1">
  <n>Create type conversion module</n>
  <files>
    communitas-ui-service/src/messaging_convert.rs,
    communitas-ui-service/src/messaging.rs
  </files>
  <action>
    1. Create new file `messaging_convert.rs`
    2. Add conversion functions:
       - `fn core_message_to_ui(msg: &communitas_core::MessageResponse) -> communitas_ui_api::Message`
       - `fn core_reaction_to_ui(r: &communitas_core::ReactionResponse) -> Reaction` (if UI type differs)
       - `fn ui_entity_type_to_core(et: &UnifiedEntityType) -> communitas_core::EntityType`
       - `fn core_entity_type_to_ui(et: &communitas_core::EntityType) -> UnifiedEntityType`
    3. Add `mod messaging_convert;` to lib.rs or messaging.rs
    4. Use `From`/`Into` traits where appropriate

    Keep conversions infallible where possible.
    Document any lossy conversions with comments.
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - messaging_convert.rs exists with conversion functions
    - Module is exported and accessible from messaging.rs
    - Unit tests for conversions pass
  </done>
</task>

<task type="auto" priority="p1">
  <n>Wire UiServices to pass CommunitasApp</n>
  <files>
    communitas-ui-service/src/lib.rs,
    communitas-ui-service/src/services.rs (if exists)
  </files>
  <action>
    1. Find where UiServices/ServiceContainer is built
    2. Ensure CommunitasApp is available at construction time
    3. Pass Arc<CommunitasApp> to MessagingService::new()
    4. Update any builders/factories that create MessagingService
    5. If CommunitasApp isn't available yet, add it as a required dependency

    Check existing patterns in other services (DirectoryService, etc.) for reference.
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
    cargo build -p communitas-dioxus
  </verify>
  <done>
    - UiServices passes CommunitasApp to MessagingService
    - Dioxus app builds and runs
    - No runtime panics on service initialization
  </done>
</task>

---

## Exit Criteria

- [ ] `cargo build -p communitas-ui-service` passes
- [ ] `cargo build -p communitas-dioxus` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] MessagingService has access to CommunitasApp
- [ ] Type conversions are testable and documented

---

## Notes

- This phase establishes the wiring foundation
- Subsequent phases (1.2-1.4) will implement actual CRUD operations
- Core types may differ from UI types - document any gaps found

---

## Next

Phase 1.2: Read Operations (list_threads, get_messages)

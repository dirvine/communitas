# communitas-x0x-client Status Handoff

Date: 2026-03-26
Audience: downstream Dioxus / Swift integration work
Baseline: `docs/x0x-integration-contract.md`

## Summary

`communitas-x0x-client` has been strengthened into a much closer thin x0x mirror and is now explicitly frozen to **REST + WebSocket only**.

This means downstream consumers can safely assume:
- exact REST bindings are the shared Rust transport layer
- `/ws` and `/ws/direct` are the supported daemon streaming transports in this crate
- SSE endpoints (`/events`, `/direct/events`) are intentionally out of scope
- channel/topic/store conventions still belong in app-layer code, not in this crate

## Scope freeze

### In scope
- x0xd REST routes used by Communitas
- `/ws`
- `/ws/direct`
- exact request/response/frame types
- CLI-faithful daemon lifecycle helpers

### Out of scope
- `/events`
- `/direct/events`
- GUI-only or app-layer channel schemas
- app-level topic helpers
- unread/thread/UI state

## Core fixes now in place

The shared client now correctly handles:
- flattened success payloads
- `subscription_id` from `/subscribe`
- nullable `machine_id` in `/direct/connections`
- exact `/peers` object shape
- exact `/presence` string-list shape
- exact `GroupInfo` additions like `metadata_topic` and `members`
- explicit `announce_with_options(include_user_identity, human_consent)`
- `X0xWebSocket::connect_direct()` for `/ws/direct`
- `ensure_running()` without implicit autostart side effects
- health polling instead of only fixed sleeps during startup

## Current live-tested bindings

The following areas are now covered by live contract tests:
- health / status / agent / agent-user-id / network / ws-sessions
- peers / presence / direct-connections / trust-evaluate
- groups / group detail / MLS welcome
- stores create/list/put/get/delete
- websocket subscribe/publish/message roundtrip
- agent card generation

## New/confirmed useful shared-client methods

### Identity
- `health()`
- `status()`
- `shutdown()`
- `agent()`
- `agent_user_id()`
- `agent_card(display_name, include_groups)`
- `import_agent_card(card, trust_level)`
- `announce_with_options(include_user_identity, human_consent)`
- `announce()`

### Network / discovery
- `peers()`
- `presence()`
- `network_status()`
- `bootstrap_cache()`
- `discovered_agents()`
- `discovered_agent()`

### Messaging
- `publish()`
- `subscribe()`
- `unsubscribe()`
- `X0xWebSocket::connect()`
- `X0xWebSocket::connect_direct()`

### Contacts / trust
- `list_contacts()`
- `add_contact()`
- `set_trust()`
- `update_contact()`
- `remove_contact()`
- `revoke_contact()`
- `revocations()`
- `list_machines()`
- `add_machine()`
- `remove_machine()`
- `pin_machine()`
- `unpin_machine()`
- `evaluate_trust()`

### Groups / MLS
- `create_group()`
- `list_groups()`
- `get_group()`
- `invite()`
- `join_group()`
- `set_group_display_name()`
- `leave_group()`
- `create_mls_group()`
- `list_mls_groups()`
- `get_mls_group()`
- `add_mls_member()`
- `remove_mls_member()`
- `encrypt()`
- `decrypt()`
- `create_mls_welcome()`

### Stores / files / upgrade
- `create_store()`
- `join_store()`
- `list_stores()`
- `list_keys()`
- `put()`
- `get()`
- `delete_key()`
- `send_file()`
- `transfers()`
- `transfer_status()`
- `accept_file()`
- `reject_file()`
- `check_upgrade()`
- `ws_sessions()`

## What downstream teams should NOT expect from this crate

Downstream teams should not expect this crate to provide:
- `group_prefix()` helpers
- `channel_topic()` helpers
- `thread_topic()` helpers
- channel metadata schemas
- GUI-compatible `channels_index` encoding/decoding
- app display-name fallbacks
- unread or reply-count management

Those belong in app-layer modules such as a Dioxus-side `x0x_contract.rs` helper.

## Remaining low-risk notes

These are not blockers for downstream work, but worth knowing:
- `join_store(store_id)` still uses a parameter name that may encode a stronger semantic than the docs prove; downstream app code should treat it as an exact route helper, not as a meaning-bearing abstraction.
- mutation-gated tests for more state-changing flows (card import, contact mutations, group mutation flows) can be added later without changing the shared-client scope.

## Downstream recommendation for Dioxus

Dioxus should now build app-layer x0x integration around:
- this shared REST + WebSocket client
- a small app-owned `x0x_contract.rs` helper for GUI-compatible conventions:
  - 16-char group prefix
  - channel topic construction
  - thread topic construction
  - `x0x-channels-{group_prefix}` store id
  - `channels_index` base64(JSON array of channel objects)

In short:
- shared client = transport + exact daemon shapes
- Dioxus app layer = GUI-compatible channel/topic/store conventions

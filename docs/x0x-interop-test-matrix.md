# x0x Interop Test Matrix

Status: release validation for 0.12.4 beta
Last reviewed: 2026-05-26
Primary contract: `docs/x0x-integration-contract.md`

## Purpose

This matrix defines the minimum interoperability checks required between:

- x0x runtime and GUI
- `communitas-dioxus`
- `communitas-apple`

Milestone 1 validated the x0x baseline and froze the contract.

Dioxus and Swift rows remain execution work for later milestones unless marked otherwise.

## Legend

- **Pass**: verified on 2026-05-26 unless noted otherwise
- **Fail**: verified broken
- **Todo**: not yet executed in this milestone
- **N/A**: not applicable

## 0. x0x baseline contract validation

These rows validate the source-of-truth side before any Communitas app interop work.

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| X1 | `x0x --version` / `x0xd --version` | CLI `0.15.2`, daemon `0.19.49` | Pass | Version skew observed; daemon is source of runtime truth |
| X2 | `GET /health` | Flattened success payload, no `data` wrapper | Pass | `{"ok":true,"status":"healthy","version":"0.19.49",...}` |
| X3 | `GET /status` | Flattened success payload, no `data` wrapper | Pass | Includes `api_address`, `external_addrs`, `agent_id`, `warnings` |
| X4 | `x0x routes` | Reports `82 endpoints total` | Pass | Live CLI inventory is still not exhaustive of every directly live route |
| X5 | Direct probe of `/shutdown`, `/gui`, `/gui/`, `/ws`, `/ws/direct` | Routes are live even though `x0x routes` is incomplete | Pass | `/shutdown` returned `405 Allow: POST`; `/gui*` returned `200`; `/ws*` upgraded `101` |
| X6 | `GET /events` | SSE endpoint, not WebSocket | Pass | `content-type: text/event-stream` |
| X7 | `GET /groups/:id` | `chat_topic` and `metadata_topic` use 16-char group prefix | Pass | Example: `x0x.group.e0511563b44806e6.chat/general` |
| X8 | WS connect to `/ws` and `/ws/direct` | Initial `connected` frame received | Pass | Verified live |
| X9 | WS keepalive | Client tolerates unsolicited `{"type":"pong"}` | Pass | Observed live immediately after connect |
| X10 | New daemon API coverage | `/agent/sign`, `/diagnostics/{ack,dm,exec,groups}`, `/exec/{run,cancel,sessions}` are covered by Rust and Swift clients | Pass | Direct probes returned `200` for read/sign/session endpoints and structured `400` for intentionally invalid exec requests |

## 1. Daemon lifecycle and identity

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| D1 | Dioxus connects to running `x0xd` | `health`, `status`, and `agent` decode without wrapper assumptions | Todo | |
| D2 | Swift connects to running `x0xd` | `health`, `status`, and `agent` decode without wrapper assumptions | Todo | |
| D3 | Dioxus install/start/stop/autostart UI | Mirrors documented x0x behavior exactly | Todo | Install via shell installer; start via `x0x start`; stop via `x0x stop`; autostart via `x0x autostart` |
| D4 | Swift install/start/stop/autostart UI | Mirrors documented x0x behavior exactly | Todo | Same contract as D3 |

## 2. Contacts and groups

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| G1 | Dioxus lists contacts | Matches `GET /contacts` shape and values | Todo | |
| G2 | Swift lists contacts | Matches `GET /contacts` shape and values | Todo | |
| G3 | Dioxus lists groups | Matches `GET /groups` shape and values | Todo | |
| G4 | Swift lists groups | Matches `GET /groups` shape and values | Todo | |
| G5 | Dioxus reads group details | Parses `chat_topic`, `metadata_topic`, and `members` from `GET /groups/:id` | Todo | |
| G6 | Swift reads group details | Parses `chat_topic`, `metadata_topic`, and `members` from `GET /groups/:id` | Todo | |
| G7 | Dioxus creates or joins group | GUI can see and use the same group | Todo | |
| G8 | Swift creates or joins group | GUI can see and use the same group | Todo | |
| G9 | GUI-created or joined group | Dioxus can see and use the same group | Todo | |
| G10 | GUI-created or joined group | Swift can see and use the same group | Todo | |
| G11 | Group display-name update | Uses request body `{"name":"..."}` | Todo | Current GUI code drifts here; contract follows x0x source |

## 3. Channel metadata and store schema

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| C1 | GUI creates channel | Dioxus sidebar shows channel | Todo | |
| C2 | GUI creates channel | Swift sidebar shows channel | Todo | |
| C3 | Dioxus creates channel | GUI shows channel | Todo | |
| C4 | Swift creates channel | GUI shows channel | Todo | |
| C5 | Dioxus creates channel | Swift shows channel | Todo | |
| C6 | Swift creates channel | Dioxus shows channel | Todo | |
| C7 | Store id | Uses `x0x-channels-{group_prefix}` | Todo | `group_prefix = group_id[..16]` |
| C8 | Store key | Uses `channels_index` | Todo | |
| C9 | Store payload | `value` is base64(JSON array of channel objects) | Todo | No wrapper object |
| C10 | Channel object schema | `{name, description, creator, created_at, topic}` | Todo | |
| C11 | Channel topic | `x0x.group.{group_prefix}.chat/{channel}` | Todo | |
| C12 | No alternate schema | No per-channel primary keys or custom index object | Todo | |

## 4. Channel messaging

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| M1 | GUI sends channel message | Dioxus receives and displays it | Todo | |
| M2 | GUI sends channel message | Swift receives and displays it | Todo | |
| M3 | Dioxus sends channel message | GUI receives and displays it | Todo | |
| M4 | Dioxus sends channel message | Swift receives and displays it | Todo | |
| M5 | Swift sends channel message | GUI receives and displays it | Todo | |
| M6 | Swift sends channel message | Dioxus receives and displays it | Todo | |
| M7 | Payload schema | `{id, text, sender_name, sender_id, timestamp, channel}` | Todo | |
| M8 | Sender identity handling | Payload `sender_id` and WS envelope `origin` are both tolerated | Todo | `origin` may be `null` |
| M9 | Channel field | `channel` matches selected channel name | Todo | |
| M10 | Topic format | `x0x.group.{group_prefix}.chat/{channel}` | Todo | |

## 5. Threads

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| T1 | GUI opens thread and replies | Dioxus receives reply | Todo | |
| T2 | GUI opens thread and replies | Swift receives reply | Todo | |
| T3 | Dioxus thread reply | GUI receives reply | Todo | |
| T4 | Dioxus thread reply | Swift receives reply | Todo | |
| T5 | Swift thread reply | GUI receives reply | Todo | |
| T6 | Swift thread reply | Dioxus receives reply | Todo | |
| T7 | Thread topic format | `x0x.group.{group_prefix}.thread/{message_id}` | Todo | |
| T8 | Thread payload schema | `{id, text, sender_name, sender_id, timestamp, thread_root, channel}` | Todo | |
| T9 | Thread root field | `thread_root` equals parent message id | Todo | |

## 6. Thread broadcast to channel

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| B1 | GUI reply with broadcast | Dioxus sees thread + channel indicator | Todo | |
| B2 | GUI reply with broadcast | Swift sees thread + channel indicator | Todo | |
| B3 | Dioxus reply with broadcast | GUI sees thread + channel indicator | Todo | |
| B4 | Dioxus reply with broadcast | Swift sees thread + channel indicator | Todo | |
| B5 | Swift reply with broadcast | GUI sees thread + channel indicator | Todo | |
| B6 | Swift reply with broadcast | Dioxus sees thread + channel indicator | Todo | |
| B7 | Broadcast payload | Includes `broadcast: true` on channel copy | Todo | |

## 7. WebSocket contract

| ID | Flow | Expected result | Status | Notes |
|---|---|---|---|---|
| W1 | Dioxus pub/sub WS connection | Uses `/ws` | Todo | |
| W2 | Swift pub/sub WS connection | Uses `/ws` | Todo | |
| W3 | Dioxus direct-message WS connection | Uses `/ws/direct` when WS DM push is required | Todo | `/ws` does not auto-receive direct messages |
| W4 | Swift direct-message WS connection | Uses `/ws/direct` when WS DM push is required | Todo | |
| W5 | Dioxus subscribe frame | Exact x0x shape | Todo | `{"type":"subscribe","topics":[...]}` |
| W6 | Swift subscribe frame | Exact x0x shape | Todo | |
| W7 | Dioxus publish frame | Exact x0x shape | Todo | `{"type":"publish","topic":"...","payload":"base64"}` |
| W8 | Swift publish frame | Exact x0x shape | Todo | |
| W9 | Dioxus direct-send frame | Exact x0x shape | Todo | `{"type":"send_direct","agent_id":"...","payload":"base64"}` |
| W10 | Swift direct-send frame | Exact x0x shape | Todo | |
| W11 | Inbound `connected` frame | Parsed correctly | Todo | |
| W12 | Inbound `message` frame | Parsed correctly with nullable `origin` | Todo | |
| W13 | Inbound `direct_message` frame | Parsed correctly on `/ws/direct` | Todo | |
| W14 | Inbound `subscribed` and `unsubscribed` frames | Parsed correctly | Todo | |
| W15 | Inbound `pong` frame | Tolerated without preceding client ping | Todo | |
| W16 | Publish / send_direct behavior | No success ack assumed | Todo | Expect only `error` or later delivery |
| W17 | SSE separation | `/events` and `/direct/events` are never treated as WebSockets | Todo | |

## 8. Regression gates before merge

| Gate | Required |
|---|---|
| Live x0x baseline rows X1-X9 still pass | Yes |
| Shared Rust client contract tests pass | Yes |
| Dioxus x0x integration tests pass | Yes |
| Swift x0x decoding/topic/schema tests pass | Yes |
| Relevant manual interop rows for changed area pass | Yes |
| Contract doc still matches live `x0x routes` plus direct runtime probes for `/shutdown`, `/gui`, `/gui/`, `/ws`, `/ws/direct` | Yes |

## 9. Current baseline summary

As of 2026-03-26:

- Milestone 1 contract freeze is complete on the x0x side.
- Live x0x behavior confirms flattened responses, SSE on `/events`, WebSocket on `/ws` and `/ws/direct`, and GUI route availability.
- The route inventory is currently split across live direct probes, `x0x routes`, `src/api/mod.rs`, and README prose; the contract doc resolves that drift for Communitas.
- Dioxus and Swift interop rows remain execution work for later milestones.

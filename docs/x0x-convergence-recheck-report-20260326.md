# x0x Convergence Recheck Report

Date: 2026-03-26
Contract baseline: `docs/x0x-integration-contract.md`
Interop matrix baseline: `docs/x0x-interop-test-matrix.md`

## Workstream A: communitas-x0x-client audit + test plan

### 1. Summary

I re-audited `communitas-x0x-client` against the frozen contract, `../x0x/src/api/mod.rs`, `../x0x/docs/api-reference.md`, `../x0x/docs/api.md`, `../x0x/src/gui/x0x-gui.html`, and live daemon behavior on `127.0.0.1:12700`.

Top conclusions:

- The crate is not an exact x0x mirror yet, despite claiming full REST coverage.
- Several currently bound routes are live-broken today because their response models do not match the daemon.
- The WebSocket frame tags align with the frozen contract, but the crate lacks a thin helper for `/ws/direct`.
- The daemon lifecycle wrapper mostly mirrors x0x CLI commands, but `ensure_running()` still introduces policy drift by auto-enabling autostart.

### 2. Evidence

Files reviewed:

- `communitas-x0x-client/src/lib.rs`
- `communitas-x0x-client/src/client.rs`
- `communitas-x0x-client/src/types.rs`
- `communitas-x0x-client/src/websocket.rs`
- `communitas-x0x-client/src/daemon.rs`
- `communitas-x0x-client/src/error.rs`
- `docs/x0x-integration-contract.md`
- `docs/x0x-interop-test-matrix.md`
- `../x0x/src/api/mod.rs`
- `../x0x/docs/api-reference.md`
- `../x0x/docs/api.md`
- `../x0x/src/gui/x0x-gui.html`
- `../x0x/README.md`

Commands and live probes:

- `cargo test -p communitas-x0x-client --lib -- --nocapture`
- `x0x --version`
- `x0x health`
- `x0x status`
- `x0x routes`
- `curl -s http://127.0.0.1:12700/health`
- `curl -s http://127.0.0.1:12700/status`
- `curl -si http://127.0.0.1:12700/shutdown`
- `curl -si http://127.0.0.1:12700/gui`
- `curl -si http://127.0.0.1:12700/gui/`
- `curl -sI http://127.0.0.1:12700/events`
- `curl -s http://127.0.0.1:12700/groups`
- `curl -s http://127.0.0.1:12700/groups/:id`
- `curl -s http://127.0.0.1:12700/contacts`
- `curl -s http://127.0.0.1:12700/direct/connections`
- `curl -s http://127.0.0.1:12700/peers`
- `curl -s http://127.0.0.1:12700/presence`
- raw websocket probes against `/ws` and `/ws/direct`
- live REST probes for `/subscribe` and `/unsubscribe`

### 3. Findings

File-by-file:

- `communitas-x0x-client/src/lib.rs`
  - `lib.rs` still says the crate provides access to the "full x0xd REST API" even though the frozen Milestone 1 surface still has missing bindings for `/shutdown`, `/agent/user-id`, `/agent/card`, `/agent/card/import`, `/mls/groups/:id/welcome`, `/ws/sessions`, `/events`, and `/direct/events`.

- `communitas-x0x-client/src/client.rs`
  - `subscribe()` expects `SubscribeResponse.id`, but live `POST /subscribe` returns `{"ok":true,"subscription_id":"..."}`. This breaks unsubscribe flow immediately.
  - `direct_connections()` decodes `DirectConnection.machine_id` as non-null `String`, but live `GET /direct/connections` returns `"machine_id": null`.
  - `announce()` sends `{}` and exposes no way to set the documented `include_user_identity` and `human_consent` request fields.
  - The client has no bindings for these canonical frozen routes: `/shutdown`, `/agent/user-id`, `/agent/card`, `/agent/card/import`, `/contacts/:agent_id/revoke`, `/contacts/:agent_id/revocations`, `/contacts/:agent_id/machines/:machine_id/pin`, `/contacts/:agent_id/machines/:machine_id/pin` `DELETE`, `/trust/evaluate`, `/mls/groups/:id/welcome`, `/ws/sessions`, `/events`, `/direct/events`.
  - The client does expose discovery/network helpers, but those helpers are not exact mirrors today either.
  - `set_group_display_name()` is confirmed good: it uses `PUT /groups/:id/display-name` with body `{"name":"..."}`, which matches the frozen contract.
  - `join_store()` matches the canonical route path but still bakes in an unresolved semantic assumption by naming the argument `store_id` even though the x0x docs still describe the CLI as `x0x store join <topic>`.

- `communitas-x0x-client/src/types.rs`
  - `PeerList.peers` is `Vec<String>`, but live `GET /peers` returns objects shaped like `{"id":"..."}`.
  - `PresenceList.agents` is `Vec<PresenceBeacon>`, but live `GET /presence` returns plain agent ID strings.
  - `SubscribeResponse.id` should be `subscription_id`.
  - `DirectConnection.machine_id` must be optional.
  - `GroupInfo` is not an exact mirror of live `GET /groups/:id`: the daemon currently returns `chat_topic`, `metadata_topic`, and `members`, while the struct omits `metadata_topic` and `members`.
  - Several request and response types remain unverified rather than exact, especially around create/join flows and file transfer enums. They should not be treated as frozen mirrors until mutation-gated tests prove them.

- `communitas-x0x-client/src/websocket.rs`
  - Frame tags in `WsOutbound` and `WsInbound` match the frozen `/ws` protocol.
  - `connect()` only targets `/ws`; callers must know to use `connect_to("ws://127.0.0.1:12700/ws/direct")` manually for direct-message sessions.
  - The receive loop assumes unknown inbound frames are disposable warnings. That is acceptable for now, but tests must not assume frame ordering beyond eventual receipt.

- `communitas-x0x-client/src/daemon.rs`
  - `install()`, `start()`, `stop()`, and `autostart()` map directly to the documented x0x CLI commands.
  - `ensure_running()` still auto-calls `autostart()` on first install. The frozen contract treats autostart as optional, not implicit.
  - `ensure_running()` uses fixed 2s/3s sleeps instead of polling `/health` until timeout. That is a brittle readiness strategy layered on top of the correct CLI commands.

- `communitas-x0x-client/src/error.rs`
  - No contract drift found. Error variants are thin wrappers around HTTP, JSON, WebSocket, and daemon errors.

Exact live mismatches:

| Area | Current crate behavior | Frozen/live behavior | Safe fix |
|---|---|---|---|
| Subscribe response | `id` | `subscription_id` | Rename field and return mapping |
| Direct connections | `machine_id: String` | `machine_id: null` observed live | Change to `Option<String>` |
| Peers | `Vec<String>` | `{"peers":[{"id":"..."}]}` | Introduce exact peer item type |
| Presence | `Vec<PresenceBeacon>` | `{"agents":["...","..."]}` | Decode exact string list |
| Group detail | omits `metadata_topic` and `members` | both returned live | Extend `GroupInfo` exactly |
| Announce helper | sends `{}` only | docs define consent fields | Add thin request type with exact fields |

### 4. Recommended changes

Exact and actionable:

- `communitas-x0x-client/src/lib.rs`
  - Either narrow the doc claim immediately or expand the route surface before continuing to advertise "full" coverage.

- `communitas-x0x-client/src/types.rs`
  - Rename `SubscribeResponse.id` to `subscription_id`.
  - Replace `PeerList.peers: Vec<String>` with an exact peer item type that matches live `GET /peers`.
  - Replace `PresenceList.agents: Vec<PresenceBeacon>` with the exact live shape.
  - Change `DirectConnection.machine_id` to `Option<String>`.
  - Extend `GroupInfo` to include `metadata_topic` and `members`.
  - Add exact types for `UserIdResponse`, `AgentCardResponse`, `WsSessionsResponse`, `TrustEvaluation`, `ContactRevocations`, and MLS welcome output.

- `communitas-x0x-client/src/client.rs`
  - Add thin bindings for the missing canonical frozen routes.
  - Change `announce()` to accept the documented consent fields instead of hardcoding `{}`.
  - Keep `set_group_display_name()` as-is.
  - Add `ws_sessions()` as a normal HTTP getter.
  - Add SSE support for `/events` and `/direct/events` only if the crate intends to stay a full mirror. Otherwise narrow the crate scope explicitly.

- `communitas-x0x-client/src/websocket.rs`
  - Add `connect_direct()` as a zero-logic wrapper for `/ws/direct`.
  - Keep outbound and inbound frame enums thin and exact.

- `communitas-x0x-client/src/daemon.rs`
  - Remove implicit autostart from `ensure_running()`.
  - Replace fixed sleeps with a health-polling loop and explicit timeout.

Live contract test plan:

- `communitas-x0x-client/tests/health_contract.rs`
  - Verify flattened `GET /health`, `GET /status`, `GET /agent`, and `GET /agent/user-id`.
  - Verify `/shutdown` exists but is method-gated by `Allow: POST` on `GET`.
  - Verify `/gui` and `/gui/` are reachable even though `x0x routes` omits them.
  - Verify `GET /events` is SSE via `content-type: text/event-stream`.
  - Verify out-of-scope but public helpers still decode exact live `peers` and `presence` shapes.

- `communitas-x0x-client/tests/stores_contract.rs`
  - Use a dedicated reusable scratch namespace and single-threaded execution.
  - Verify `GET /stores`, `POST /stores`, `POST /stores/:id/join`, `GET /stores/:id/keys`, `PUT /stores/:id/:key`, `GET /stores/:id/:key`, and `DELETE /stores/:id/:key`.
  - Verify store values remain base64 and preserve `content_type`.
  - Verify GUI-compatible channel storage can round-trip `channels_index` as a base64 JSON array.

- `communitas-x0x-client/tests/websocket_contract.rs`
  - Verify `/ws` and `/ws/direct` both return `connected`.
  - Verify `/ws` accepts `ping`, `subscribe`, `unsubscribe`, `publish`.
  - Verify `/ws/direct` accepts `ping` and returns `connected`.
  - Verify tests do not assume a strict `subscribed`/`pong` ordering.
  - Verify `message` frames carry `topic`, `payload`, and nullable `origin`.

- `communitas-x0x-client/tests/groups_contract.rs`
  - Verify `GET /groups` and `GET /groups/:id` decode exact frozen fields.
  - Verify `GET /groups/:id` returns `chat_topic` and `metadata_topic` using the 16-char group prefix.
  - Verify `PUT /groups/:id/display-name` uses body `{"name":"..."}`.
  - Verify named-group create/invite/join/leave flows under a mutation gate.
  - Verify MLS list/get/add/remove/encrypt/decrypt/welcome bindings under a mutation gate.

- `communitas-x0x-client/tests/contacts_contract.rs`
  - Verify `GET /contacts` decodes exact trust-level fields.
  - Verify `GET /direct/connections` tolerates nullable `machine_id`.
  - Verify contacts revoke, revocations, machines, pin/unpin, and trust evaluation under fixture-gated mutation tests.
  - Verify no route is silently translated to invented field names.

Test harness rules:

- Put live contract tests under `tests/` and run them single-threaded.
- Use a shared helper module for `X0X_API_BASE`, namespace generation, and mutation gates.
- Mark mutation tests `#[ignore]` unless an explicit env var enables them.

### 5. Blockers / ambiguities

- The x0x docs still describe `x0x store join <topic>` while the HTTP route is `POST /stores/:id/join`. The client should avoid encoding stronger semantics than the contract currently proves.
- Some create/join/file-transfer response models remain unverified because the recheck stayed intentionally light on persistent mutation.
- `x0x routes` remains incomplete and cannot be used as the sole route inventory source.

### 6. Ready-next actions

- Fix the live-breaking type mismatches first: subscribe, direct connections, group detail, peers, presence.
- Add the missing canonical Milestone 1 bindings next: shutdown, user-id, card, card import, contact revoke/revocations/pin/trust evaluate, MLS welcome, ws sessions.
- Add `connect_direct()` and the five live contract suites immediately after the model fixes land.

## Workstream B: Dioxus x0x touchpoint inventory + convergence plan

### 1. Summary

I re-checked the Dioxus touchpoints against the frozen contract and the x0x GUI behavior. Dioxus is closer than Swift on transport and topic construction, but it still diverges materially on the channel metadata schema and store usage.

Top conclusions:

- Channel and thread transport mostly point at the right x0x surfaces.
- Channel metadata still uses an invented index-plus-per-channel schema instead of the frozen GUI-compatible `channels_index` JSON array.
- Daemon status UI still inherits shared-client lifecycle drift through `ensure_running()`.

### 2. Evidence

Files reviewed:

- `communitas-dioxus/src/models/channel.rs`
- `communitas-dioxus/src/components/channel_sidebar.rs`
- `communitas-dioxus/src/components/channel_chat.rs`
- `communitas-dioxus/src/components/thread_panel.rs`
- `communitas-dioxus/src/components/daemon_status.rs`
- `communitas-dioxus/src/main.rs`
- `docs/x0x-integration-contract.md`
- `docs/x0x-interop-test-matrix.md`
- `../x0x/src/gui/x0x-gui.html`
- `../x0x/docs/api-reference.md`
- `../x0x/docs/api.md`
- `../x0x/src/api/mod.rs`

### 3. Findings

Inventory table:

| File | Routes / transport | Current behavior | Contract target | Result |
|---|---|---|---|---|
| `src/models/channel.rs` | wire payloads + store schema | message shape adds UI-only fields; channel store uses `ChannelIndex` + per-channel meta | GUI-compatible `channels_index` JSON array of channel objects | Diverges |
| `src/components/channel_sidebar.rs` | `GET /groups`, `GET /stores`, `GET /stores/:id/:key` | finds any store whose topic contains prefix; reads `channels_index` as `ChannelIndex`; fetches `channel:{name}` keys | exact store id `x0x-channels-{prefix}` and single `channels_index` value | Diverges |
| `src/components/channel_chat.rs` | `/ws`, `POST /publish`, `GET /agent` | correct WS endpoint and publish route; payload includes UI-only fields; sender name hardcoded `"Me"` | exact GUI payload schema, tolerate envelope `origin` | Partial |
| `src/components/thread_panel.rs` | `/ws`, `POST /publish` | correct 16-char thread topic; supports broadcast | exact thread payload schema without invented persisted UI fields | Partial |
| `src/components/daemon_status.rs` | `/health`, `/agent`, `DaemonManager` | health polling is fine; install/start path inherits `ensure_running()` autostart drift | exact x0x install/start/stop/autostart semantics | Partial |

Divergences:

- `src/models/channel.rs` still documents and encodes `ChannelMeta` under `channel:{name}` keys and a separate `ChannelIndex` object. The frozen contract forbids that schema.
- `src/components/channel_sidebar.rs` uses fuzzy store discovery by topic substring instead of exact store id `x0x-channels-{group_prefix}`.
- `src/components/channel_chat.rs` and `src/components/thread_panel.rs` publish payloads with `reply_count` and `reactions`, which the frozen contract classifies as UI-only fields, not persisted x0x message schema.
- `src/components/channel_chat.rs` hardcodes `sender_name: "Me"` instead of using the actual display name.
- `src/components/daemon_status.rs` still wires "Install & Start" to `ensure_running()`, which inherits the shared-client autostart policy drift.

Confirmed-good areas:

- Thread topic construction uses the correct 16-char prefix.
- Channel and thread messaging use `/ws` rather than treating SSE as WebSocket.
- `client.publish()` is used for actual publish flow rather than an invented route.

### 4. Recommended changes

File-by-file patch plan:

- `communitas-dioxus/src/models/channel.rs`
  - Remove `ChannelIndex` as the canonical persisted schema.
  - Reduce persisted `ChannelMeta` to the GUI-compatible channel object fields.
  - Treat `reply_count`, `reactions`, and unread state as view-model-only data rather than wire payload fields.

- `communitas-dioxus/src/components/channel_sidebar.rs`
  - Replace fuzzy store lookup with exact store id `x0x-channels-{group_prefix}`.
  - Read and write only `channels_index`.
  - Stop fetching `channel:{name}` keys.
  - If `channels_index` is missing, synthesize a default `general` view locally but write back using the canonical array format when channel creation happens.

- `communitas-dioxus/src/components/channel_chat.rs`
  - Keep `/ws` and `publish`.
  - Source `sender_name` from the actual display name or identity, not `"Me"`.
  - Publish only frozen message fields.
  - Tolerate both payload `sender_id` and WebSocket envelope `origin`.

- `communitas-dioxus/src/components/thread_panel.rs`
  - Keep current topic logic.
  - Publish only frozen thread payload fields plus `broadcast` when needed.
  - Keep reply counts and thread summaries in local UI state.

- `communitas-dioxus/src/components/daemon_status.rs`
  - Split install, start, stop, and autostart actions so the UI mirrors x0x CLI behavior exactly.
  - Do not present autostart as implicit side effect of install/start.

Proposed helper module API:

```rust
pub fn group_prefix(group_id: &str) -> &str;
pub fn channel_store_id(group_id: &str) -> String;
pub fn channel_topic(group_id: &str, channel: &str) -> String;
pub fn thread_topic(group_id: &str, message_id: &str) -> String;
pub fn decode_channels_index(value_b64: &str) -> Result<Vec<ChannelRecord>, X0xContractError>;
pub fn encode_channels_index(channels: &[ChannelRecord]) -> Result<String, X0xContractError>;
pub fn canonical_channel_record(meta: &ChannelMeta) -> ChannelRecord;
```

### 5. Blockers / ambiguities

- The frozen contract intentionally follows x0x source over current GUI drift for `PUT /groups/:id/display-name`. If Dioxus later adds a group rename UI, it must use `{"name":"..."}` even though the current GUI HTML still sends `display_name`.
- Unread-count persistence is not part of the x0x contract and must remain app-local.

### 6. Ready-next actions

- Implement `x0x_contract.rs`.
- Convert the sidebar/channel store path to the canonical `channels_index` array schema.
- Remove UI-only fields from published channel/thread payloads.
- Then add interop tests for rows `C7-C12`, `M7-M10`, `T7-T9`, and `W1-W17`.

## Workstream C: Swift x0x migration table + rewrite plan

### 1. Summary

I re-checked the Swift layer against the frozen contract and live baseline. The Swift stack is still the furthest from x0x and should be treated as a rewrite, not an incremental polish.

Top conclusions:

- The HTTP client still assumes `/pubsub/*`, `/kv`, `/groups/invite`, and `/groups/:id/leave`, which are not x0x routes.
- The core decoder still assumes `{"ok":true,"data":...}` as the normal success envelope, which is explicitly forbidden by the frozen contract.
- The WebSocket wrapper still points at `/events`, which is frozen as SSE.
- Channel management still uses full-group-id topics plus an invented KV schema instead of the GUI-compatible store and topic contract.
- Daemon lifecycle still targets `x0xd --daemon` directly instead of `x0x start` / `x0x stop` / `x0x autostart`.

### 2. Evidence

Files reviewed:

- `communitas-apple/Sources/X0xClient/X0xClient.swift`
- `communitas-apple/Sources/X0xClient/X0xWebSocket.swift`
- `communitas-apple/Sources/X0xClient/DaemonManager.swift`
- `communitas-apple/Sources/X0xClient/Errors.swift`
- `communitas-apple/Sources/X0xClient/Models/Types.swift`
- `communitas-apple/Sources/X0xClient/Models/Contact.swift`
- `communitas-apple/Sources/X0xClient/Models/Group.swift`
- `communitas-apple/Sources/X0xClient/Models/Channel.swift`
- `communitas-apple/Sources/Communitas/Models/AppState.swift`
- `communitas-apple/Sources/Communitas/Models/ChannelManager.swift`
- `docs/x0x-integration-contract.md`
- `docs/x0x-interop-test-matrix.md`
- `../x0x/docs/api-reference.md`
- `../x0x/src/gui/x0x-gui.html`

Route mismatch table:

| Swift file | Current route | Required route | Mismatch | Exact fix |
|---|---|---|---|---|
| `X0xClient.swift` | `/pubsub/publish` | `/publish` | wrong namespace | replace route |
| `X0xClient.swift` | `/pubsub/subscribe` | `/subscribe` | wrong namespace | replace route |
| `X0xClient.swift` | `/groups/invite` | `/groups/:id/invite` | missing path id | move group id into path |
| `X0xClient.swift` | `/groups/:id/leave` `POST` | `/groups/:id` `DELETE` | wrong route and verb | replace with delete |
| `X0xClient.swift` | `/kv`, `/kv/:key` | `/stores`, `/stores/:id/:key` | invented storage surface | replace with exact store routes |
| `X0xWebSocket.swift` | `/events` | `/ws` or `/ws/direct` | SSE mistaken for WS | replace endpoint and protocol |

Model mismatch table:

| Model file | Current assumption | Frozen/live behavior | Exact fix |
|---|---|---|---|
| `Models/Types.swift` | universal `data` envelope | flattened success payloads | decode top-level fields directly |
| `Models/Types.swift` | `HealthStatus.uptime` | `uptime_secs` | rename field |
| `Models/Types.swift` | `DaemonStatus.running`, `peer_count` | `status`, `peers`, `api_address`, `external_addrs`, `warnings` | replace struct |
| `Models/Types.swift` | `AgentIdentity.public_key`, `four_words` | `agent_id`, `machine_id`, `user_id` | replace struct |
| `Models/Contact.swift` | trust levels `untrusted`, `verified` | `blocked`, `unknown`, `known`, `trusted` | replace enum exactly |
| `Models/Group.swift` | `InviteResponse.invite` | x0x invite payload uses `invite_link` and group metadata | replace response model |
| `Models/Channel.swift` | persisted `ChannelIndex` and per-channel meta | single `channels_index` JSON array | replace schema |

WS mismatch table:

| File | Current behavior | Required behavior | Exact fix |
|---|---|---|---|
| `X0xWebSocket.swift` | opens `/events` as WebSocket | use `/ws` or `/ws/direct` | replace URL and frame handling |
| `ChannelManager.swift` | decodes invented `GossipEvent { event, topic, payload, sender }` | decode x0x `connected` / `message` / `direct_message` / `subscribed` / `unsubscribed` / `pong` / `error` | replace parser |
| `ChannelManager.swift` | no subscribe frame over WS | `{"type":"subscribe","topics":[...]}` | send exact WS frames |
| `ChannelManager.swift` | full-group-id topics | 16-char group prefix | fix topic builder |

### 3. Findings

File-by-file:

- `communitas-apple/Sources/X0xClient/X0xClient.swift`
  - The pub/sub routes are still `/pubsub/*`, not `/publish` and `/subscribe`.
  - Group invite and leave use invented route shapes.
  - KV helpers use `/kv` instead of x0x stores.
  - The decoder still assumes the normal response is `{"ok":true,"data":...}`.

- `communitas-apple/Sources/X0xClient/X0xWebSocket.swift`
  - The default path is `/events`, which the frozen contract explicitly classifies as SSE.
  - The class has no concept of x0x WS frames, subscriptions, publish frames, or `/ws/direct`.

- `communitas-apple/Sources/X0xClient/DaemonManager.swift`
  - It searches for `x0xd` instead of the `x0x` CLI.
  - It starts the daemon by executing `x0xd --daemon`, which is not the frozen contract.
  - It has no stop or autostart path aligned to `x0x stop` / `x0x autostart`.

- `communitas-apple/Sources/X0xClient/Models/Types.swift`
  - The whole envelope model is wrong for x0x.
  - Health, status, and agent models do not match live fields.
  - Gossip and direct-message models assume fields (`message_id`, `sender`, `timestamp`) that do not match frozen WS frames or the GUI message payload.

- `communitas-apple/Sources/X0xClient/Models/Contact.swift`
  - Trust levels do not match x0x at all.
  - Machine records and contact payloads assume fields that are not frozen.

- `communitas-apple/Sources/X0xClient/Models/Group.swift`
  - Group create still sends `display_name`, while the frozen contract resolves on `{"name":"..."}` for the display-name update route.
  - Group responses omit frozen fields like `chat_topic` and `metadata_topic`.
  - Invite and join responses use the wrong shapes.

- `communitas-apple/Sources/X0xClient/Models/Channel.swift`
  - Channel payload shape is close enough for `id`, `text`, `sender_name`, `sender_id`, `timestamp`, `channel`, `thread_root`, `broadcast`.
  - Persisted channel metadata still depends on an invented `ChannelIndex` plus flags that are not part of the frozen GUI-compatible schema.

- `communitas-apple/Sources/Communitas/Models/AppState.swift`
  - It is downstream of the broken shared Swift x0x layer. Most UI state drift should not be fixed here first.

- `communitas-apple/Sources/Communitas/Models/ChannelManager.swift`
  - Uses full `groupId` in topics instead of the 16-char prefix.
  - Persists channel metadata under `channels.{groupId}.index` and `channels.{groupId}.{name}.meta`, which the frozen contract explicitly forbids.
  - Treats inbound messages as custom gossip-event objects rather than x0x WS frames.

Risk list:

- Rewriting only the UI layer first will cement the wrong wire contract.
- Keeping the `data` envelope assumption will continue to break nearly every success response.
- Keeping `/events` as a WebSocket path will make any WS-based interop impossible.
- Channel storage drift will prevent GUI, Dioxus, and Swift from sharing channels even if messaging routes are fixed.

### 4. Recommended changes

File-by-file rewrite plan in execution order:

1. `communitas-apple/Sources/X0xClient/Models/Types.swift`
   - Replace the envelope and top-level daemon models with exact x0x shapes.

2. `communitas-apple/Sources/X0xClient/Models/Contact.swift`
   - Replace trust levels and contact/machine models with exact x0x values and fields.

3. `communitas-apple/Sources/X0xClient/Models/Group.swift`
   - Replace group, invite, and join models with exact frozen fields.

4. `communitas-apple/Sources/X0xClient/X0xClient.swift`
   - Rewrite routes and verbs to exact x0x bindings.
   - Remove `/kv` and `/pubsub/*`.
   - Add canonical store helpers and WS session helper.

5. `communitas-apple/Sources/X0xClient/X0xWebSocket.swift`
   - Replace `/events` with `/ws` and `/ws/direct`.
   - Implement exact frame send/receive types.

6. `communitas-apple/Sources/X0xClient/DaemonManager.swift`
   - Rebase lifecycle onto `x0x start`, `x0x stop`, and `x0x autostart`.

7. `communitas-apple/Sources/X0xClient/Models/Channel.swift`
   - Trim persisted schema to the frozen channel object.

8. `communitas-apple/Sources/Communitas/Models/ChannelManager.swift`
   - Rewrite topics, store id, key name, store payload, WS parsing, and create-channel flow.

9. `communitas-apple/Sources/Communitas/Models/AppState.swift`
   - Update only after the shared Swift x0x layer is exact.

What can be rewritten safely before UI changes:

- `X0xClient.swift`
- `X0xWebSocket.swift`
- `DaemonManager.swift`
- `Models/Types.swift`
- `Models/Contact.swift`
- `Models/Group.swift`
- the persisted parts of `Models/Channel.swift`

### 5. Blockers / ambiguities

- The x0x docs still leave some store-join semantics ambiguous, so the Swift API should avoid stronger naming than the contract currently proves.
- Some direct-message push semantics require `/ws/direct`; the current Swift layer has no concept of that split.

### 6. Ready-next actions

- Rewrite the shared Swift x0x layer first, before touching SwiftUI-facing state.
- Land decoding tests for flattened `health`, `status`, `agent`, `contacts`, `groups`, and `group details`.
- Then rewrite `ChannelManager` to the frozen 16-char-prefix topic and `channels_index` array schema.

## Workstream D: live fixture capture + interop harness prep

### 1. Summary

I re-checked the live x0x baseline against the frozen contract. The locked docs correctly capture the important route-inventory drift and runtime behavior.

Top conclusions:

- `x0x 0.10.0` remains the live baseline.
- `x0x routes` still reports `71 endpoints total` and remains incomplete.
- Direct runtime probes confirm `/shutdown`, `/gui`, `/gui/`, `/ws`, `/ws/direct`, and `/events`.
- The GUI-compatible 16-char prefix and `channels_index` array storage remain the correct interop baseline.

### 2. Evidence

Commands and probes:

- `x0x --version`
- `x0x health`
- `x0x status`
- `x0x routes`
- `curl -s http://127.0.0.1:12700/health`
- `curl -s http://127.0.0.1:12700/status`
- `curl -si http://127.0.0.1:12700/shutdown`
- `curl -si http://127.0.0.1:12700/gui`
- `curl -si http://127.0.0.1:12700/gui/`
- `curl -sI http://127.0.0.1:12700/events`
- raw websocket connections to `/ws` and `/ws/direct`
- `curl -s http://127.0.0.1:12700/groups/:id`

Live baseline:

- `x0x 0.10.0`
- `GET /health` returned flattened `{"ok":true,"status":"healthy","version":"0.10.0","peers":4,"uptime_secs":...}`
- `GET /status` returned flattened `{"ok":true,"status":"connected","version":"0.10.0","uptime_secs":...,"api_address":"127.0.0.1:12700","external_addrs":[...],"agent_id":"...","peers":4,"warnings":[]}`
- `x0x routes` returned `71 endpoints total`
- `GET /shutdown` returned `405 Method Not Allowed` with `Allow: POST`
- `GET /gui` and `GET /gui/` returned `200 OK`
- `GET /events` returned `content-type: text/event-stream`
- `/ws` and `/ws/direct` both returned `connected`

### 3. Findings

- The frozen contract is correct to treat direct daemon behavior as higher priority than `x0x routes`.
- `/shutdown`, `/gui`, `/gui/`, `/ws`, and `/ws/direct` are all live even though inventory text remains inconsistent.
- `GET /groups/:id` still returns `chat_topic` and `metadata_topic` using the 16-char group prefix.
- The x0x GUI behavior remains aligned with the frozen channel store contract: store id `x0x-channels-{group_prefix}`, key `channels_index`, value = base64-encoded JSON array of channel objects.

### 4. Recommended changes

No additional contract edits are required before implementation.

Reusable smoke-test sequence:

- `x0x --version`
- `x0x health`
- `x0x status`
- `x0x routes`
- `curl -s http://127.0.0.1:12700/health`
- `curl -s http://127.0.0.1:12700/status`
- `curl -si http://127.0.0.1:12700/shutdown | head`
- `curl -si http://127.0.0.1:12700/gui | head`
- `curl -si http://127.0.0.1:12700/gui/ | head`
- `curl -sI http://127.0.0.1:12700/events`
- raw `/ws` and `/ws/direct` handshake plus `ping` / `subscribe`

Reusable naming conventions:

- ephemeral pub/sub topic: `x0x.test.<uuid>`
- group prefix: `group_id[..16]`
- channel topic: `x0x.group.{group_prefix}.chat/{channel}`
- thread topic: `x0x.group.{group_prefix}.thread/{message_id}`
- channel store id: `x0x-channels-{group_prefix}`
- channel store key: `channels_index`

### 5. Blockers / ambiguities

- `x0x routes` remains incomplete and should not be used as the sole authority for route existence.
- Store join semantics remain mildly ambiguous in docs, so smoke tests should avoid over-encoding meaning into the `:id` parameter name.

### 6. Ready-next actions

- Use the frozen contract and this report as the shared baseline for Milestones 2-4.
- Land the Rust-client contract tests first.
- Then converge Dioxus and Swift against the same topic/store/message contract without inventing app-local transport semantics.

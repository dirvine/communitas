# x0x Integration Contract

Status: frozen for Milestone 1
Last validated: 2026-03-26
Scope: Communitas integrations against `../x0x`

## 1. Purpose

This document freezes the Communitas ↔ x0x integration contract.

It is a specification freeze, not an implementation plan.

Rules:

- x0x is the source of truth.
- Communitas must adapt to x0x, never the reverse.
- If this document conflicts with existing Communitas code, the Communitas code is wrong.

## 2. Source-of-truth precedence

When sources disagree, use this exact order:

1. Live x0x daemon or CLI behavior
2. `../x0x/src/api/mod.rs`
3. `../x0x/docs/api-reference.md`
4. `../x0x/docs/api.md`
5. `../x0x/src/gui/x0x-gui.html`
6. Communitas-side code

Resolution rule inside item 1:

- Prefer direct daemon behavior over generated inventory text when they disagree.
- On 2026-03-26, direct probes against the live daemon proved routes that `x0x routes` did not list.

## 3. Live validation baseline

Validated on 2026-03-26 against the running local x0x installation.

Required checks:

- `x0x --version` → `x0x 0.10.0`
- `x0x health` → flattened success output
- `x0x status` → flattened success output
- `x0x routes` → `71 endpoints total`
- `curl -s http://127.0.0.1:12700/health` → flattened JSON
- `curl -s http://127.0.0.1:12700/status` → flattened JSON

Observed live facts:

- Base API URL: `http://127.0.0.1:12700`
- `GET /health` returned `{"ok":true,"status":"healthy","version":"0.10.0","peers":4,"uptime_secs":...}`
- `GET /status` returned `{"ok":true,"status":"connected","version":"0.10.0","uptime_secs":...,"api_address":"127.0.0.1:12700","external_addrs":[...],"agent_id":"...","peers":4,"warnings":[]}`
- `x0x routes` reported `71 endpoints total`

Additional direct runtime checks used to resolve drift:

- `GET /gui` → `200 OK`, `content-type: text/html`
- `GET /gui/` → `200 OK`, `content-type: text/html`
- `GET /shutdown` → `405 Method Not Allowed`, `Allow: POST`
- `GET /events` → `200 OK`, `content-type: text/event-stream`
- WebSocket upgrade `GET /ws` → `101 Switching Protocols`
- WebSocket upgrade `GET /ws/direct` → `101 Switching Protocols`
- `GET /groups/:id` returned `chat_topic: "x0x.group.{group_prefix}.chat/general"` and `metadata_topic: "x0x.group.{group_prefix}.meta"`

## 4. Route inventory drift

The current x0x sources and live outputs do not fully agree on route inventory.

| Source | Current claim | Notes |
|---|---|---|
| Live `x0x routes` | `71 endpoints total` | Not exhaustive of all live daemon routes |
| `../x0x/src/api/mod.rs` | 71 registry entries | Includes `/shutdown` and `/gui`; omits `/ws`, `/ws/direct`, `/gui/` |
| `../x0x/docs/api-reference.md` / `api.md` | `/ws`, `/ws/direct`, `/shutdown`, `/gui`, `/gui/` documented | More complete than `x0x routes` for these endpoints |
| `../x0x/README.md` | says `73` documented endpoints | Stale on 2026-03-26 |
| `../x0x/src/bin/x0xd.rs` router | 76 method/path handlers | Matches direct runtime probes for `/ws`, `/ws/direct`, `/shutdown`, `/gui`, `/gui/` |

Frozen rule for Communitas:

- Do not treat `x0x routes` as a complete route inventory.
- For route existence, direct daemon behavior wins.
- The `71 vs 73` discrepancy is real and must be documented, but it is not the only drift.

## 5. Response conventions

This section is frozen.

### 5.1 Success payloads

- Success payloads are flattened.
- There is no universal `data` wrapper.
- Clients must decode endpoint-specific top-level fields directly.

Examples:

```json
{"ok":true,"status":"healthy","version":"0.10.0","peers":4,"uptime_secs":7}
```

```json
{"ok":true,"status":"connected","version":"0.10.0","uptime_secs":7,"api_address":"127.0.0.1:12700","external_addrs":["[2a0d:3344:32d:2e10:e9bd:1e7a:c9e7:79be]:5483"],"agent_id":"d607c7fb2190042ad6ab40fcfd4766efc200ed2038eb065dad2f4798d501e53d","peers":4,"warnings":[]}
```

### 5.2 Error payloads

Errors use:

```json
{"ok":false,"error":"description"}
```

### 5.3 Client requirements

Communitas clients must:

- accept flattened success payloads
- reject any invented `{"data": ...}` wrapper
- propagate `{"ok":false,"error":"..."}` without schema translation

## 6. Canonical routes Communitas must follow

This is the minimum frozen x0x route surface for Milestone 1.

### 6.1 System

- `GET /health`
- `GET /status`
- `POST /shutdown`

Notes:

- `/shutdown` is live and must be treated as canonical.
- `x0x stop` maps to `/shutdown`.
- `x0x routes` currently omits `/shutdown`, but live daemon behavior wins.

### 6.2 Identity

- `GET /agent`
- `POST /announce`
- `GET /agent/user-id`
- `GET /agent/card`
- `POST /agent/card/import`

### 6.3 Contacts and trust

- `GET /contacts`
- `POST /contacts`
- `POST /contacts/trust`
- `PATCH /contacts/:agent_id`
- `DELETE /contacts/:agent_id`
- `POST /contacts/:agent_id/revoke`
- `GET /contacts/:agent_id/revocations`
- `GET /contacts/:agent_id/machines`
- `POST /contacts/:agent_id/machines`
- `DELETE /contacts/:agent_id/machines/:machine_id`
- `POST /contacts/:agent_id/machines/:machine_id/pin`
- `DELETE /contacts/:agent_id/machines/:machine_id/pin`
- `POST /trust/evaluate`

### 6.4 Groups

Named groups:

- `POST /groups`
- `GET /groups`
- `GET /groups/:id`
- `POST /groups/:id/invite`
- `POST /groups/join`
- `PUT /groups/:id/display-name`
- `DELETE /groups/:id`

MLS groups:

- `POST /mls/groups`
- `GET /mls/groups`
- `GET /mls/groups/:id`
- `POST /mls/groups/:id/members`
- `DELETE /mls/groups/:id/members/:agent_id`
- `POST /mls/groups/:id/encrypt`
- `POST /mls/groups/:id/decrypt`
- `POST /mls/groups/:id/welcome`

Frozen group-detail fields from `GET /groups/:id`:

- `group_id`
- `name`
- `description`
- `creator`
- `created_at`
- `chat_topic`
- `metadata_topic`
- `members`

Important discrepancy resolved by precedence:

- `../x0x/src/bin/x0xd.rs` expects `PUT /groups/:id/display-name` request body field `name`.
- `../x0x/src/gui/x0x-gui.html` currently sends `display_name`.
- Freeze on x0x source behavior, not GUI drift: use `{"name":"..."}`.

### 6.5 Publish / subscribe

- `POST /publish`
- `POST /subscribe`
- `DELETE /subscribe/:id`
- `GET /events`

Frozen rule:

- `/events` is SSE, not WebSocket.

### 6.6 Direct messaging

- `POST /agents/connect`
- `POST /direct/send`
- `GET /direct/connections`
- `GET /direct/events`

Frozen rule:

- `/direct/events` is SSE, not WebSocket.

### 6.7 Stores

- `GET /stores`
- `POST /stores`
- `POST /stores/:id/join`
- `GET /stores/:id/keys`
- `PUT /stores/:id/:key`
- `GET /stores/:id/:key`
- `DELETE /stores/:id/:key`

### 6.8 Tasks

- `GET /task-lists`
- `POST /task-lists`
- `GET /task-lists/:id/tasks`
- `POST /task-lists/:id/tasks`
- `PATCH /task-lists/:id/tasks/:tid`

### 6.9 Files

- `POST /files/send`
- `GET /files/transfers`
- `GET /files/transfers/:id`
- `POST /files/accept/:id`
- `POST /files/reject/:id`

### 6.10 WebSocket

- `GET /ws`
- `GET /ws/direct`
- `GET /ws/sessions`

Notes:

- `/ws` and `/ws/direct` are live and canonical.
- `x0x routes` and `../x0x/src/api/mod.rs` do not fully inventory them.

### 6.11 GUI

- `GET /gui`
- `GET /gui/`

Notes:

- Both routes are live.
- `/gui/` is not represented in `../x0x/src/api/mod.rs`.
- `x0x routes` currently omits both `/gui` and `/gui/`.

### 6.12 Out of scope for this milestone

Discovery and network-diagnostics routes exist in x0x, but they are not the minimum frozen Milestone 1 interop surface.

If Communitas uses them anyway, it must still mirror x0x exactly and must not invent renamed wrappers.

## 7. WebSocket contract

This section is frozen.

### 7.1 Endpoints

Use:

- `ws://127.0.0.1:12700/ws`
- `ws://127.0.0.1:12700/ws/direct`

Do not use:

- `/events` as a WebSocket
- `/direct/events` as a WebSocket

Those are SSE endpoints.

### 7.2 Client → server frames

The client must send JSON text frames with these exact shapes:

```json
{"type":"ping"}
{"type":"subscribe","topics":["topic-a","topic-b"]}
{"type":"unsubscribe","topics":["topic-a"]}
{"type":"publish","topic":"topic-a","payload":"aGVsbG8="}
{"type":"send_direct","agent_id":"hex64...","payload":"aGVsbG8="}
```

Frozen rules:

- `payload` is base64
- `topics` is an array
- `send_direct` uses `agent_id`, not another key name

### 7.3 Server → client frames

The server sends JSON text frames with these exact tags:

```json
{"type":"connected","session_id":"uuid","agent_id":"hex64..."}
{"type":"message","topic":"topic-a","payload":"aGVsbG8=","origin":"hex64..."}
{"type":"direct_message","sender":"hex64...","machine_id":"hex64...","payload":"aGVsbG8=","received_at":1234567890}
{"type":"subscribed","topics":["topic-a","topic-b"]}
{"type":"unsubscribed","topics":["topic-a"]}
{"type":"pong"}
{"type":"error","message":"..."}
```

Clarifications from source and live behavior:

- `message.origin` is nullable in the server type. Clients must accept a string or `null`.
- `/ws/direct` auto-receives `direct_message` frames. `/ws` does not.
- The server does not send a success ack for `publish` or `send_direct`. Expect only `error` or later delivery events.
- The server sends `connected` immediately on upgrade.
- The server also sends `pong` keepalives without requiring a preceding client `ping`. Live validation saw this immediately after upgrade because the keepalive timer ticks immediately, then continues on a 30s cadence.

### 7.4 `/ws/sessions`

`GET /ws/sessions` returns flattened JSON:

```json
{
  "ok": true,
  "sessions": [
    {
      "session_id": "uuid",
      "subscribed_topics": ["topic-a"],
      "receives_direct": false
    }
  ],
  "shared_subscriptions": {
    "topic-a": 1
  }
}
```

## 8. Topic naming contract

This section is frozen.

### 8.1 Group prefix

`group_prefix` is the first 16 characters of the full hex group id.

```text
group_prefix = group_id[..16]
```

This is not only GUI behavior:

- `../x0x/src/groups/mod.rs` uses the same 16-char prefix for x0x-native `chat_topic_prefix` and `metadata_topic`
- live `GET /groups/:id` returned `chat_topic` and `metadata_topic` using that prefix

### 8.2 x0x-native group topics

Current x0x-native named-group topics:

- metadata topic: `x0x.group.{group_prefix}.meta`
- general chat topic: `x0x.group.{group_prefix}.chat/general`

### 8.3 GUI-compatible channel topics

Freeze these topic rules for Communitas interop:

```text
x0x.group.{group_prefix}.chat/{channel}
x0x.group.{group_prefix}.thread/{message_id}
```

Examples:

```text
x0x.group.1234567890abcdef.chat/general
x0x.group.1234567890abcdef.thread/7f8f6a0e-...
```

### 8.4 Compatibility rule

`GET /groups/:id` may return `chat_topic` for the general channel.

Freeze this behavior:

- treat `chat_topic` as the authoritative x0x-native general-channel topic
- treat channel-specific GUI topics as the current Communitas channel convention
- do not switch to full-group-id topic names

## 9. Channel metadata and store convention

This section is GUI-derived and is frozen for Communitas until x0x publishes a different first-class channel contract.

### 9.1 Store id / topic

```text
x0x-channels-{group_prefix}
```

### 9.2 Key

```text
channels_index
```

### 9.3 Stored value

The stored value is:

- written through x0x store semantics
- passed to the daemon in the `value` field as base64
- decoded to JSON that is a top-level array of channel objects
- written with content type `application/json`

In shorthand:

```text
value = base64(JSON array of channel objects)
```

### 9.4 Frozen channel object shape

```json
{
  "name": "general",
  "description": "General discussion",
  "creator": "agent_id",
  "created_at": 1710000000000,
  "topic": "x0x.group.1234567890abcdef.chat/general"
}
```

### 9.5 Communitas requirement

All Communitas apps must read and write this exact convention.

Forbidden alternatives:

- a wrapped `ChannelIndex` object in `channels_index`
- per-channel keys like `channel:{name}` as the primary schema
- full-group-id channel topic variants
- any schema that x0x GUI cannot read today

### 9.6 Separation from x0x-native group metadata

Do not confuse:

- x0x-native `metadata_topic` from `GET /groups/:id`
- GUI-derived channel metadata store `x0x-channels-{group_prefix}`

They are different contracts.

## 10. Channel and thread message convention

This section is GUI-derived and is frozen for Communitas interop.

### 10.1 Channel message payload

```json
{
  "id": "uuid-or-generated-id",
  "text": "message text",
  "sender_name": "Alice",
  "sender_id": "hex64...",
  "timestamp": 1710000000000,
  "channel": "general"
}
```

### 10.2 Thread reply payload

```json
{
  "id": "uuid-or-generated-id",
  "text": "reply text",
  "sender_name": "Alice",
  "sender_id": "hex64...",
  "timestamp": 1710000000000,
  "thread_root": "parent_message_id",
  "channel": "general"
}
```

### 10.3 Broadcast thread reply payload

```json
{
  "id": "uuid-or-generated-id",
  "text": "reply text",
  "sender_name": "Alice",
  "sender_id": "hex64...",
  "timestamp": 1710000000000,
  "thread_root": "parent_message_id",
  "channel": "general",
  "broadcast": true
}
```

Frozen rules:

- thread replies publish to `x0x.group.{group_prefix}.thread/{message_id}`
- broadcast replies also publish to the channel topic
- `sender_id` in the payload does not replace the WebSocket envelope `origin`; clients must tolerate both

## 11. Daemon lifecycle expectations

This section is frozen.

### 11.1 Install

Mirror documented x0x installation behavior:

- primary install entrypoint: `curl -sfL https://x0x.md | sh`
- fallback: `curl -sfL https://raw.githubusercontent.com/saorsa-labs/x0x/main/scripts/install.sh | sh`

Documented installer modes:

- `--start`
- `--autostart`
- `--name <instance>`

Install is outside the daemon REST surface. Do not invent a different installation contract inside Communitas.

### 11.2 Start

Canonical behavior:

- `x0x start`
- `x0x start --name <instance>`

Frozen expectation:

- `x0x start` is the primary supported daemon start contract for Communitas
- it spawns `x0xd` in the background and waits for health
- direct `x0xd` execution may exist, but it is not the preferred app-facing abstraction

### 11.3 Stop

Canonical behavior:

- `x0x stop`
- REST equivalent: `POST /shutdown`

Frozen expectation:

- Communitas stop flows must map to x0x shutdown semantics
- do not invent a different daemon-termination contract

### 11.4 Autostart

Canonical behavior:

- `x0x autostart`
- `x0x autostart --name <instance>`
- `x0x autostart --remove`

Platform behavior from x0x:

- Linux: systemd user service
- macOS: launchd user agent

Communitas may surface autostart controls only if they mirror this behavior exactly.

## 12. Allowed and forbidden abstractions

### 12.1 Allowed

- `communitas-x0x-client` as a thin exact mirror of x0x REST and WebSocket behavior
- typed request and response models that preserve x0x field names
- typed WebSocket frame enums that preserve x0x frame names
- platform-specific transport or process wrappers that do not change x0x semantics

### 12.2 Forbidden

- inventing alternate REST namespaces
- inventing alternate WebSocket endpoints
- inventing a universal `data` wrapper
- inventing alternate topic naming rules
- inventing alternate channel/store/thread schemas in the shared client
- treating `/events` or `/direct/events` as WebSockets
- using full group ids where the frozen contract uses 16-char prefixes

Important boundary:

- `communitas-x0x-client` may mirror x0x exactly
- app-specific channel, store, and topic semantics must not be invented there
- if those conventions are needed, they must follow this document exactly

## 13. Milestone 1 freeze result

For Milestone 1, the following are now frozen:

- response conventions
- canonical route surface
- WebSocket endpoints and frame shapes
- SSE versus WebSocket separation
- 16-character group-prefix topic rule
- GUI-compatible channel store convention
- daemon lifecycle expectations
- allowed versus forbidden client abstractions

No Communitas client should proceed into Milestone 2 assuming different x0x behavior unless x0x itself changes first and this document is updated to match.

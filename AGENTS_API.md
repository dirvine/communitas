# Communitas + Saorsa Core — Agents API (AGENTS_API.md)

Version: 1.2 • Date: 2025-09-15 • Audience: Agents (human + AI)

This document is the authoritative contract for agents that integrate with Communitas (desktop app) and its headless node, both of which are powered by Saorsa Core. It captures the IPC surface, supporting Rust APIs, security guarantees, and operational flows required to operate autonomously. All interfaces here are panic-free; production builds enforce `-D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`.

## What changed since v1.0 (2025-09-05)
- **New identity bootstrap path**: `core_claim` persists ML-DSA keys to the OS keyring; `core_advertise` signs presence heartbeats and returns Four-Word IPv4 handles.
- **Address book integration**: Upgraded to saorsa-core v0.3.21 so `core_channel_list_members`, `core_resolve_channel_members`, and `core_send_message_to_channel` resolve Four-Word addresses via `saorsa_core::get_user_four_words`. Event payloads now surface both array and string forms (`four_words`, `four_words_text`).
- **Expanded IPC surface**: Added container engine (`container_init/put_object/get_object/apply_ops/current_tip`), encrypted KV storage (`core_private_put/get`), bootstrap config commands, raw SPKI pinning, and QUIC delta sync helpers.
- **Core wiring update**: `communitas-core::CoreContext` now owns saorsa-core managers (identity, chat, messaging, storage) per device, keeping group signing material in-memory.
- **Pointer-only storage**: The container engine (new crate `crates/communitas-container`) signs tips and keeps opaque blobs local, enforcing the pointer-only DHT policy.
- **Headless node uplift**: Self-update flow, metrics endpoint, bootstrap/QUIC tuning, and peer cache now documented for automation tooling.

---

## 0. Vision & Context

Communitas is the evolution of WhatsApp + Dropbox + Zoom + Slack in a single, local-first, PQC-ready collaboration app. It delivers:

- Real-time messaging, threads, reactions, mentions, presence
- Group, channel, project, and organization entities — each with a shared “virtual disk”
- Quantum-secured storage per entity with collaborative Markdown editing
- End-to-end group crypto and FEC-sealed content distribution
- Public websites without DNS: identities publish `website_root` and link via Four-Word Networking (FWN)
- Direct machine-to-machine secure channels (QUIC, WebRTC-over-QUIC)
- A “people’s internet”: human-verifiable identities, zero centralized naming, anti-phishing by design

Apps + core:

- **Desktop (Tauri)** app: user-facing collaboration UI, exposes IPC commands for automation.
- **Headless node**: bootstrap/seed/personal nodes for network support and (future) rewards.
- **Saorsa Core** (`saorsa-core` crate, v0.3.21): DHT, QUIC, identity, groups, messaging, website hosting, and the address-book helpers (`get_user_four_words`, `get_user_by_four_words`) we now consume directly.

---

## 1. Quick Start (Agent Playbook)

1. **Claim identity**
   ```ts
   const idHex = await invoke<string>('core_claim', { words: ['river','spark','honest','lion'] })
   ```
   - Validates dictionary words (`saorsa_core::fwid::fw_check`), generates ML-DSA-65 keys, stores everything in the OS keyring via `communitas_core::keystore::Keystore`.

2. **Optional: advertise presence**
   ```ts
   const advert = await invoke('core_advertise', { addr: '203.0.113.12:443', storageGb: 200 })
   ```
   - Returns `{ id_hex, endpoint_fw4? }`. Presence heartbeats are signed locally; full network publish remains pointer-only for now.

3. **Initialize runtime context**
   ```ts
   await invoke('core_initialize', {
     fourWords: 'river-spark-honest-lion',
     displayName: 'Alice',
     deviceName: 'Mac-mini',
     deviceType: 'Desktop'
   })
   ```
   - Builds `CoreContext`, wiring identity, chat, messaging, and storage managers with PQC device metadata.

4. **Messaging flows**
   ```ts
   const channel = await invoke('core_create_channel', { name: 'general', description: 'Daily chat' })
   await invoke('core_send_message_to_channel', { channelId: channel.id, text: 'Hello team' })
   await invoke('core_subscribe_messages', { channelId: channel.id })
   ```
   - Events: `message-received` with decrypted payloads (else `{ encrypted: true }`).
   - Membership helpers: `core_channel_list_members` returns `{ user_id, role, four_words }` with Four-Word handles fetched via saorsa-core.

5. **Container storage**
   ```ts
   await invoke('container_init')
   const handle = await invoke('container_put_object', { bytes: Array.from(new TextEncoder().encode('# Note')) })
   const tip = await invoke('container_current_tip')
   ```
   - Handles map to encrypted blobs stored under `COMMUNITAS_DATA_DIR`.

6. **Group membership**
   ```ts
   await invoke('core_group_add_member', {
     groupWords: ['project','alpha','launch','team'],
     memberWords: ['ocean','forest','moon','star']
   })
   ```
   - Uses cached ML-DSA group keys to sign canonical membership roots.

7. **Sync & repair**
   ```ts
   await invoke('sync_set_quic_pinned_spki', { value: 'spki:...' })
   await invoke('sync_fetch_deltas', { peerAddr: 'ocean-forest-moon-star:443' })
   await invoke('sync_start_tip_watcher', { intervalMs: 2000 })
   ```

8. **Bootstrap configuration**
   ```ts
   const seeds = await invoke<string[]>('core_get_bootstrap_nodes')
   await invoke('core_update_bootstrap_nodes', { nodes: ['river-mountain-sun-cloud:443'] })
   ```

---

## 2. Communitas (Tauri) — IPC Surface

All commands return `Result<…, String>` at the IPC boundary. Errors are human-readable; no panics bubble to IPC.

### 2.1 Identity & Bootstrap
| Command | Parameters | Returns | Notes |
| --- | --- | --- | --- |
| `core_claim` | `words: [String;4]` | `id_hex: String` | Generates ML-DSA-65 keys, stores in keyring, sets current identity. |
| `core_advertise` | `addr: String`, `storage_gb: u32` | `{ id_hex, endpoint_fw4? }` | Signs presence heartbeat; returns optional FW4 handle when IPv4 provided. |
| `core_initialize` | `fourWords`, `displayName`, `deviceName?`, `deviceType?` | `bool` | Instantiates `CoreContext`; validates four-word format. |
| `core_get_bootstrap_nodes` | — | `Vec<String>` | Reads `bootstrap.toml` (or returns defaults). |
| `core_update_bootstrap_nodes` | `nodes: Vec<String>` | `bool` | Writes `bootstrap.toml` with sorted seed list. |
| `core_private_put` | `key: String`, `content: Vec<u8>` | `bool` | Encrypted local store (365d TTL by default). |
| `core_private_get` | `key: String` | `Vec<u8>` | Retrieves encrypted entry.

### 2.2 Messaging, Channels & Threads
| Command | Purpose |
| --- | --- |
| `core_create_channel(name, description)` | Creates public channel (`ChannelType::Public`). Validates length + characters. |
| `core_get_channels()` | Returns channels user belongs to (`Vec<Channel>`). |
| `core_add_reaction(channel_id, message_id, emoji)` | Adds emoji reaction via `ChatManager`. |
| `core_send_message_to_recipients(channel_id, recipients[], text)` | Sends direct message to specified four-word identities; enforces size/recipient limits. |
| `core_send_message_to_channel(channel_id, text)` | Expands channel membership to recipient list via address-book lookup (`get_user_four_words`) with heuristic fallback, then sends broadcast message. |
| `core_create_thread(channel_id, parent_message_id)` | Creates thread under parent message. |
| `core_subscribe_messages(channel_id?)` | Subscribes to message stream; emits `message-received`. |
| `core_channel_list_members(channel_id)` | Returns member entries `{ user_id, role, four_words }` with strings sourced from `get_user_four_words`. |
| `core_channel_invite_by_words(channel_id, invitee_words, role?)` | Validates input; returns `Err` until saorsa-core membership APIs land. |
| `core_channel_recipients(channel_id)` | Placeholder (`[]`) – UI computes fallback recipients. |
| `core_resolve_channel_members(channel_id)` | Spawns background task; emits `channel-member-resolved { user_id, role, four_words[], four_words_text }` per member. |

### 2.3 Groups & Membership
| Command | Purpose |
| --- | --- |
| `core_group_create(words)` | Creates group identity, publishes packet, stores ML-DSA group signing keys. Returns `{ id_hex, words }`. |
| `core_group_add_member(group_words, member_words)` | Fetches identity + group packet, recomputes membership root, signs update, calls `group_identity_update_members_signed`. |
| `core_group_remove_member(group_words, member_words)` | Similar flow; removes member, re-signs root. |

### 2.4 Container & Virtual Disk Pointers
| Command | Purpose |
| --- | --- |
| `container_init()` | Lazily instantiates `ContainerEngine` with identity keys. |
| `container_put_object(bytes)` | Encrypts (AEAD) + stores blob locally; returns 32-byte BLAKE3 handle (hex). |
| `container_get_object(oid_hex)` | Fetches from engine; falls back to local blob cache. |
| `container_apply_ops(ops[])` | Applies CRDT ops, returns new tip. |
| `container_current_tip()` | Returns `{ root, count, sig }`. |
| `core_private_put/get` | See identity section (encrypted KV).

### 2.5 Sync, Repair & Security
| Command | Purpose |
| --- | --- |
| `sync_start_tip_watcher(interval_ms?)` | Emits `container-tip` events every interval with current tip. |
| `sync_stop_tip_watcher()` | Cancels watcher. |
| `sync_repair_fec(data_shards, parity_shards, shares[])` | Reed–Solomon repair helper (uses `saorsa_fec`). |
| `sync_fetch_deltas(peer_addr)` | Fetches CRDT ops over ant-quic with raw public key pinning. Emits `sync-progress` events for phases `request`, `received`, `applied`. |
| `sync_set_quic_pinned_spki(value)` | Accepts 32-byte key or 44-byte SPKI (hex/base64). Stores in state. |
| `sync_clear_quic_pinned_spki()` | Clears pinned key. |
| `health()` | Returns `{ status, saorsa_core, app }` for CI smoke tests.

Events emitted to the frontend:
- `message-received`
- `channel-member-resolved` (payload includes `{ user_id, role, four_words: string[], four_words_text: string? }`)
- `container-tip`
- `sync-progress`

---

## 3. Saorsa Core APIs Used by Communitas

Identity & Enhanced Devices:
- `IdentityManager::create_identity(display_name, four_words, …)`
- `EnhancedIdentityManager::create_enhanced_identity(base, device_name, device_type)`
- `identity_fetch(id: Key) -> IdentityPacketV1`
- `identity_set_website_root(id, website_root, sig)` (see §5 for canonical bytes)

Messaging & Channels:
- `MessagingService::new(FourWordAddress, DhtClient)`
- `send_message(recipients, MessageContent::Text, ChannelId, SendOptions)`
- `subscribe_messages(channel_filter?) -> broadcast::Receiver<ReceivedMessage>`
- `ChatManager::create_channel`, `get_channel`, `get_user_channels`, `create_thread`, `add_reaction`
- `address_book::get_user_four_words(user_id) -> Result<Option<FourWordAddress>>`
- `address_book::get_user_by_four_words(words) -> Result<Option<String>>`

Groups:
- `group_identity_create(words, members) -> (GroupIdentityPacketV1, GroupKeyPair)`
- `group_identity_publish(packet)`
- `group_identity_fetch(id)`
- `group_identity_canonical_sign_bytes(id, membership_root)`
- `group_identity_update_members_signed(id, members, group_pk, sig)`

Storage & Virtual Disks:
- `StorageManager::new(DhtCoreEngine, &EnhancedIdentity)`
- `StorageManager::store_encrypted(key, bytes, ttl, metadata)`
- `StorageManager::get_encrypted::<T>(key)`

Container engine (new crate):
- `communitas_container::ContainerEngine::new(pk, sk, AeadConfig, FecConfig)`
- Ops and tips: `Op::Append`, `Tip { root, count, sig }`

---

## 4. The “New Web” (DNS-free Websites)

Identical to v1.0 but now handled via container pointers:

```text
DST = "saorsa-identity:website_root:v1"
msg = DST || id || pk || CBOR(website_root)
// sign msg with ML-DSA secret key → sig
identity_set_website_root(id, website_root, sig)
```

The desktop app exposes helpers via `core_website_*` commands (to be surfaced in a later revision). Until then, use saorsa-core bindings directly when automating website updates.

---

## 5. Sync & Networking Details

- **Pointer-only DHT policy**: Tauri only commits signed tips or manifests. Bulk data stays local or in delegated storage.
- **QUIC client**: `sync_fetch_deltas` binds to `(::, 0)` but prefers IPv4 addresses (Happy Eyeballs). Raw public key TLS (`RFC 7250`) enforced; set `COMMUNITAS_RPK_ALLOW_ANY=1` only in development.
- **FEC**: `sync_repair_fec` and container engine defaults use Reed–Solomon (`k=4`, `m=2`). Provide `shares: Vec<Option<Vec<u8>>>` with `None` for missing shards.
- **Tip watcher**: after `sync_start_tip_watcher`, UI should listen for `container-tip { root, count }` events to refresh views.
- **Bootstrap maintenance**: `bootstrap.toml` stores seeds array. Agents should atomically rewrite via `core_update_bootstrap_nodes`.

---

## 6. Headless Node & Bootstrap

Binary: `target/release/communitas-headless`

CLI flags (see `communitas-headless/src/main.rs`):
- `--config /etc/communitas/config.toml`
- `--storage /var/lib/communitas`
- `--listen 0.0.0.0:0`
- `--bootstrap word-word-word-word:port` (repeatable)
- `--metrics` + `--metrics-addr 127.0.0.1:9600`
- `--self-update`

Config schema (TOML `Config` struct):
- `identity`: optional four-word address (use `core_claim` + share key store)
- `bootstrap_nodes`: array of seeds
- `storage`: `{ base_dir, cache_size_mb, enable_fec, fec_k, fec_m }`
- `network`: `{ listen_addrs[], enable_ipv6, enable_webrtc, quic_idle_timeout_ms, quic_max_streams }`
- `update`: `{ channel, check_interval_secs, auto_update, jitter_secs }`

Runtime features:
- Self-update via GitHub Releases (primary + fallback owner).
- Peer cache for gossip; stored under `<storage>/peers.json`.
- Metrics endpoint (if `--metrics`) exposes gauge/counter set for peers, DHT throughput, storage usage.

---

## 7. Testing & Validation

Rust:
- `cargo fmt --all`
- `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`
- `cargo test -p communitas-desktop`, `-p communitas-core`, `-p communitas-headless`
- Integration suites: `cargo test integration_ --release`, `cargo test reed_solomon --release`

Node/React:
- `npm run typecheck`
- `npm run test:run` (fast), `npm run test:ci` (full), `npm run build`

Sync/containers:
- `sync_repair_fec` unit tests live in `communitas-desktop/src/sync.rs`.
- Container engine coverage under `crates/communitas-container` (unit tests recommended when extending ops or AEAD config).

CI expectations:
- `.github/workflows/ci.yml` and `test-suite.yml` depend on the commands above. Keep them up to date after changing tooling or moving code.

---

## 8. Security, Anti-Abuse, Trust

- **Panic-free production**: enforced via Clippy policy; checks run in CI.
- **Key management**: ML-DSA keys stored via OS keyring (`keyring` crate). Device IDs persisted once per install.
- **Canonical signing**: group membership and identity website roots rely on canonical bytes (`group_identity_canonical_sign_bytes`, `saorsa-identity:website_root:v1`).
- **Raw SPKI pinning**: QUIC client enforces pinned public keys unless `COMMUNITAS_RPK_ALLOW_ANY` is set.
- **Rate limiting**: UI layer should throttle invites and messaging; backend exposes `MessagingService` send options for future weighting.
- **Spam mitigation**: pointer-only DHT updates, presence signatures, and EigenTrust/RSPS hooks (saorsa-core) reduce abuse vectors.

---

## 9. Error Model & Limits

- IPC commands return `Err(String)` with actionable context (`send_message failed: …`).
- Message size limit: 10 KiB (`core_send_message_to_recipients`).
- Recipient cap: 100 recipients per message.
- `container_get_object` expects 32-byte (64-hex) handles; validates length and hex.
- FEC repair requires `data_shards > 0`; returns descriptive error otherwise.
- `core_channel_invite_by_words` currently returns an explicit `Err` until saorsa-core adds membership APIs.
- All long-lived watchers (`core_resolve_channel_members`, `sync_start_tip_watcher`) spawn Tokio tasks; ensure you call matching stop/cleanup in automations.

---

## 10. Glossary

- **FWN**: Four-Word Networking, human-readable addressing with checksum.
- **Tip**: Signed state root `{ root, count, sig }` emitted by the container engine.
- **Pointer-only**: Policy of only storing references (hashes, manifests) in the DHT.
- **ML-DSA / ML-KEM**: Post-quantum signature / key encapsulation algorithms.
- **Raw SPKI**: RFC 7250 raw public key transport for QUIC TLS handshakes.
- **FEC**: Forward Error Correction (Reed–Solomon) for resilient storage/transport.

---

## 11. References

- `communitas-desktop/src/` – IPC implementation.
- `communitas-core/src/` – CoreContext, keystore, storage glue.
- `crates/communitas-container/src/lib.rs` – container engine internals.
- `communitas-headless/src/` – headless node CLI and self-update.
- Saorsa Core docs – see upstream `saorsa-core/AGENTS_API.md` for low-level types.
- Architecture/design context – `ARCHITECTURE.md`, `DESIGN.md`, `COMMUNITAS_ARCHITECTURE.md`.

If any API in this document diverges from the code in `communitas-desktop/src`, treat the code as source of truth and file an update.

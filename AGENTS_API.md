# Communitas + Saorsa Core — Agents API (AGENTS_API.md)

Version: 1.0 • Date: 2025-09-05 • Audience: Agents (human + AI)

This document is the authoritative guide for agents integrating with Communitas (the desktop app) and the headless node, powered by Saorsa Core. It covers capabilities, APIs, events, security guarantees, anti‑abuse controls, canonical signing rules, and example flows. Every interface defined here is panic‑free and designed for autonomous, verifiable operation.

If you are implementing an agent, you can choose any subset of these APIs; all are independent and composable.

---

## 0. Vision & Context

Communitas is the evolution of WhatsApp + Dropbox + Zoom + Slack in a single, local‑first, PQC‑ready collaboration app. It delivers:

- Real‑time messaging, threads, reactions, mentions, presence
- Group, channel, project, and organization entities — each can have a shared “virtual disk”
- Quantum‑secured storage per entity with collaborative Markdown editing
- End‑to‑end group crypto and FEC‑sealed content distribution
- Public websites without DNS: identities publish `website_root` and link via Four‑Word Networking (FWN)
- Direct machine‑to‑machine secure channels (QUIC, WebRTC‑over‑QUIC)
- A “people’s internet”: human‑verifiable identities, zero centralized naming, anti‑phishing by design
- Future AI extensions: transcription, summarization, TTS — highly personalized, private by default

Two apps, one core:

- Desktop (Tauri) app: user‑facing collaboration UI, exposes Tauri commands for automation
- Headless node: bootstrap/seed nodes and personal nodes for network support and (future) rewards

Under the hood is Saorsa Core (crates.io `saorsa-core`), providing DHT, QUIC, identity, group, messaging, virtual disk, and security foundations. See also `../saorsa-core/AGENTS_API.md` for the full core API surface.

---

## 1. Quick Start (Agent Playbook)

1) Identity (four words) + device
- Generate ML‑DSA keypair (via your signer)
- Choose four words (FWN). Call `identity_claim()` (saorsa-core) with signature over utf8(words)

2) Desktop app init (Tauri)
- `core_initialize({ fourWords, displayName, deviceName, deviceType })`

3) Channels & messaging
- `core_create_channel({ name, description })` → Channel
- `core_send_message_to_channel(channelId, "Hello!")` or `core_send_message_to_recipients(channelId, ["river-spark-honest-lion"], "Hi")`
- `core_subscribe_messages({ channelId? })` → listen to `message-received`

4) Private storage
- `core_disk_write({ entity_hex, disk_type: "Private", path: "/docs/readme.md", content_base64 })`
- `core_disk_read(entity_hex, "Private", "/docs/readme.md")`

5) Public website (new web)
- Create site files (`core_website_set_home`), then publish:
  - `core_website_publish_receipt(entity_hex, website_root_hex)` → manifest stored
  - Build canonical bytes and sign ML‑DSA
  - `core_identity_set_website_root(id_hex, website_root_hex, sig_hex)` → DNS‑free, human‑verifiable website

6) Group membership
- `core_group_create([w1,w2,w3,w4])` (stores group keypair locally in app state)
- Add/remove members using canonical signing and `group_identity_update_members_signed` under the hood

---

## 2. Communitas (Tauri) — IPC Surface (100%)

All commands return `Result<… , String>` at the IPC boundary. Errors are human readable.

Identity & bootstrap
- `core_initialize(fourWords: string, displayName: string, deviceName?: string, deviceType?: "Desktop"|"Mobile"|"Tablet"|"Web") -> bool`
- `health() -> { status: "ok", saorsa_core: string, app: string }`

Messaging, channels, threads
- `core_create_channel(name: string, description: string) -> Channel`
- `core_get_channels() -> Channel[]`
- `core_post_message(channelId: string, text: string, threadId?: string) -> Message` (persist via ChatManager)
- `core_send_message_to_channel(channelId: string, text: string) -> messageId`
- `core_send_message_to_recipients(channelId: string, recipients: string[], text: string) -> messageId` (recipients are four words)
- `core_channel_recipients(channelId: string) -> string[]`
- `core_create_thread(channelId: string, parentMessageId: string) -> Thread`
- `core_add_reaction(channelId: string, messageId: string, emoji: string) -> bool`
- `core_subscribe_messages(channelId?: string) -> bool`
  - Emits `message-received` (payload decrypted if possible): `{ id?, channel_id?, sender?, content?, receivedAt?, encrypted?: boolean, error?: string }`

Virtual Disk & Website (per entity: org/group/channel/project/individual)
- Private/public/shared virtual disks
  - `core_disk_write({ entity_hex, disk_type: "Private"|"Public"|"Shared", path, content_base64, mime_type? }) -> WriteReceipt`
  - `core_disk_read(entity_hex: string, disk_type: string, path: string) -> Uint8Array`
  - `core_disk_list(entity_hex: string, disk_type: string, path: string, recursive: boolean) -> FileEntry[]`
  - `core_disk_delete(entity_hex: string, disk_type: string, path: string) -> bool`
  - `core_disk_sync(entity_hex: string, disk_type: string) -> SyncStatus`
- Website publishing
  - `core_website_set_home(entity_hex: string, markdown_content: string, assets: { path, content_base64, mime_type }[]) -> bool`
  - `core_website_publish({ object_hex, k, m, shard_size, assets_hex[], sealed_meta_hex? }) -> bool` (manifest)
  - `core_website_get_manifest(object_hex: string) -> ContainerManifestV1 JSON`
  - `core_website_publish_receipt(entity_hex: string, website_root_hex: string) -> PublishReceipt JSON`
  - `core_identity_set_website_root(id_hex: string, website_root_hex: string, sig_hex: string) -> bool`
  - `core_website_publish_and_update_identity(entity_hex: string, website_root_hex: string, sig_hex?: string) -> PublishReceipt JSON`

Groups (threshold ready)
- `core_group_create(words: [string; 4]) -> { id_hex, words }`
- `core_group_add_member(group_words: [string;4], member_words: [string;4]) -> bool`
- `core_group_remove_member(group_words: [string;4], member_words: [string;4]) -> bool`

Security helpers (frontend)
- `buildWebsiteRootCanonicalHex(idHex, pkHex, websiteRootHex)`: see `src/services/website.ts`
- Subscription helper: `subscribeMessages(onMessage, { channelId? })` see `src/services/messagingSubscription.ts`

Dev panels
- `/dev/console`: live "Message Console"
- `/dev/website`: "Website Publishing Utility" (canonical bytes, publish, apply identity update)

Network Connection & Status
- `NetworkConnectionService` (singleton): Auto-connects on startup, retry logic, fallback to local mode
  - `connect()`: Manual connection attempt
  - `disconnect()`: Go to local mode
  - `getState()`: Current network state (status, peers, bootstrap nodes, errors)
  - `subscribe(listener)`: Listen to network state changes
- `NetworkStatusIndicator` component: Visual status in header with click-to-reconnect
- States: `connecting`, `connected`, `offline`, `local`, `error`
- Auto-behaviors: Browser online/offline detection, exponential backoff retry (1s, 3s, 10s)

Offline-First Architecture
- `OfflineStorageService`: IndexedDB + sync queue + file caching
  - All data operations work offline
  - Automatic sync when network returns
  - Multiple storage layers: Memory, IndexedDB, Tauri backend, localStorage
- Test utilities in console:
  - `window.testNetwork.status()`: Check network status
  - `window.testNetwork.connect()`: Manual connect
  - `window.testNetwork.disconnect()`: Go local
  - `window.testNetwork.simulateOffline()`: Test offline mode
  - `window.testNetwork.testFlow()`: Run complete test

---

## 3. Saorsa Core — Key APIs Used by Communitas

Identity
- `identity_claim(words, pubkey, sig) -> Result<()>`
- `identity_fetch(id: Key) -> IdentityPacketV1`
- `identity_publish_endpoints_signed(id, endpoints, ep_sig) -> Result<()>`
- `identity_set_website_root(id, website_root, sig) -> Result<()>`  ← new in 0.3.17
  - Canonical: `"saorsa-identity:website_root:v1" || id || pk || CBOR(website_root)`

Groups
- `group_identity_create(words, members) -> (GroupIdentityPacketV1, GroupKeyPair)`
- `group_identity_publish(packet) -> Result<()>`
- `group_identity_fetch(id: Key) -> GroupIdentityPacketV1`
- `group_identity_canonical_sign_bytes(id: &Key, membership_root: &Key) -> Vec<u8>`  ← exported
- `group_identity_update_members_signed(id, new_members, group_pk, group_sig) -> Result<()>`  ← new in 0.3.17
- `group_member_add(id, member, group_pk, group_sig) -> Result<()>`
- `group_member_remove(id, member_id, group_pk, group_sig) -> Result<()>`
- `group_epoch_bump(id, proof?, group_pk, group_sig) -> Result<()>` (optional)

Messaging
- `MessagingService::new(four_word_addr, DhtClient) -> Self`
- `send_message(recipients: Vec<FourWordAddress>, content, channel_id, options) -> (MessageId, DeliveryReceipt)`
- `channel_recipients(&ChannelId) -> Vec<FourWordAddress>`  ← new helper in 0.3.17
- `send_message_to_channel(channel_id, content, options) -> (MessageId, DeliveryReceipt)`  ← new in 0.3.17
- `subscribe_messages(channel_filter: Option<ChannelId>) -> broadcast::Receiver<ReceivedMessage>`

Virtual Disk & Website
- `disk_create(entity_id, DiskType, DiskConfig) -> DiskHandle`
- `disk_mount(entity_id, DiskType) -> DiskHandle`
- `disk_write(&handle, path, content, FileMetadata) -> WriteReceipt`
- `disk_read(&handle, path) -> Vec<u8>`
- `disk_list(&handle, path, recursive) -> Vec<FileEntry>`
- `disk_delete(&handle, path) -> ()`
- `disk_sync(&handle) -> SyncStatus`
- `website_set_home(&handle, markdown_content, assets: Vec<Asset>) -> ()`
- `website_publish(entity_id, website_root) -> PublishReceipt`
- Container manifests: `container_manifest_put/fetch` (FEC, sealed meta)

Transport / Networking
- QUIC via `ant-quic`, IPv4‑first, Happy Eyeballs fallback
- WebRTC DataChannel bridge over QUIC
- DHT with quorum policy, telemetry, and event bus
- Four‑Word Networking (FWN) encoders for FW4/FW6 endpoints

---

## 4. Security, Anti‑Abuse, and Trust

Production code guarantees
- No `unwrap/expect/panic!` in production — enforced by Clippy CI: `-D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`
- Canonical signing for all sensitive updates (identity website_root, group membership, epoch bumps)
- Zeroized secrets and structured errors

Identity & Anti‑Phishing
- Four‑Word Addressing: constrained dictionary + checksum make typosquatting detectable and rare
- DNS‑free web: `identity.website_root` binds a content root to identity keys (ML‑DSA verification). Links resolve by identity, not DNS
- Endpoint publication requires signature over canonical tuple `(id || pk || CBOR(endpoints))`

Group security
- Group identities carry ML‑DSA group keys; membership changes are signed over canonical `membership_root`
- Optional `group_epoch_bump` to auditable epoch rotation after changes
- Threshold capabilities available (saorsa-core `threshold` module) for governance and write policies

DHT & WriteAuth
- DHT writes go through `PutPolicy { quorum, ttl, auth }` with Single/Delegated/Mls/Threshold authorizers
- Immutable manifests and sealed content reduce tampering surface; telemetry emits DHT puts/gets

Spam & Scams
- Rate limiter patterns available on the Tauri side (simple), and trust‑weighted routing in core
- `EigenTrust` integration and RSPS provider summaries (saorsa-core) provide supply‑side reputation
- Content size and type constraints on disk and messaging by default

Event model
- Global event bus for DHT updates and topology; Tauri emits `message-received` for UIs/agents

See also: `../saorsa-core/AGENTS_API.md` — sections on Principles, Error Handling, Anti‑Phishing and Name Safety.

---

## 5. The “New Web” (No DNS)

Every entity can host a website by publishing a container manifest to the DHT and updating `identity.website_root` with a signed, canonical message.

- Human‑verifiable addressing via four words
- No centralized DNS; sites are verified by ML‑DSA over canonical signing bytes
- Content is content‑addressed, FEC‑sharded, and optionally encrypted
- Links between sites use FWN and identity references; agents can validate links before navigation

Example (canonical bytes & update)
```text
DST = "saorsa-identity:website_root:v1"
msg = DST || id || pk || CBOR(website_root)
// Sign msg with ML‑DSA secret key → sig
identity_set_website_root(id, website_root, sig)
```

---

## 6. End‑to‑End Flows (Examples)

A) Subscribe + send to channel
```ts
import { invoke } from '@tauri-apps/api/tauri'
import { subscribeMessages } from './services/messagingSubscription'

await invoke('core_initialize', { fourWords, displayName: 'Alice', deviceName: 'Mac', deviceType: 'Desktop' })

// Subscribe to all messages
const unlisten = await subscribeMessages((m) => console.log('recv', m))

// Create channel & send
const ch = await invoke('core_create_channel', { name: 'general', description: 'Demo' })
await invoke('core_send_message_to_channel', { channelId: ch.id, text: 'Hello world' })
```

B) Private disk write/read
```ts
await invoke('core_disk_write', {
  entityHex,
  diskType: 'Private',
  path: '/docs/welcome.md',
  contentBase64: btoa('# Welcome\nThis is a demo.'),
  mimeType: 'text/markdown',
})
const data = await invoke<Uint8Array>('core_disk_read', { entityHex, diskType: 'Private', path: '/docs/welcome.md' })
```

C) Publish website + update identity
```ts
import { buildWebsiteRootCanonicalHex, applyWebsiteRootWithSignature } from './services/website'

// 1) Publish manifest (also available: core_website_publish_and_update_identity)
const rcpt = await invoke('core_website_publish_receipt', { entityHex, websiteRootHex })

// 2) Build canonical bytes and sign externally
const canonicalHex = buildWebsiteRootCanonicalHex(entityHex, pkHex, websiteRootHex)
const sigHex = await mySigner(canonicalHex) // your ML‑DSA signature hex

// 3) Update identity
await applyWebsiteRootWithSignature(entityHex, websiteRootHex, sigHex)
```

D) Group add member
```ts
await invoke('core_group_add_member', {
  groupWords: ['river','spark','honest','lion'],
  memberWords: ['ocean','forest','moon','star']
})
```

---

## 7. Headless Node & Bootstrap

Headless binaries (in Communitas repo earlier and in product packaging):
- `communitas-node` — run a PQC‑ready node with QUIC and DHT
- `communitas-autoupdater` — jittered 0–6h rollout, with signature checks

Operational surface:
- Config: TOML with listen addresses, storage path, bootstrap seeds
- Health: `/health`, metrics at `127.0.0.1:9600`
- Systemd units, cloud‑init for DigitalOcean; default bootstrap in 6 regions
- Four‑word endpoints computed from IPv4 and port for human presentation
- Future: provider rewards for seeders and storage providers

See: `finalise/DEPLOY_TESTNET.md` and saorsa-core AGENTS_API.md sections.

---

## 8. Testing & Validation

Rust (backend)
- Build: `cargo check`
- Lints: `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`
- Unit/integration tests: `cargo test` (core and tauri layers as applicable)

UI (frontend)
- Typecheck: `npm run typecheck`
- Unit: `npm test` (Vitest)
- End‑to‑end: subscribe/send flows, disk read/write, website publish/update

Security validation
- Verify canonical byte construction and signature lengths (ML‑DSA signature size = 3309 bytes)
- Enforce PutPolicy and WriteAuth for DHT updates
- Run chaos and churn tests (saorsa-core test suites)

---

## 9. Error Model & Limits

- All functions return `Result<T, E>` with explicit error variants; IPC flattens to strings
- Size limits: disk quotas (configurable), message length, manifest sizes
- Rate limiting patterns available on app side; reputation and RSPS in core
- No hidden panics or unwraps; agents should not rely on panic catching

---

## 10. Glossary

- FWN (Four‑Word Networking): readable, checksum‑secured addressing for humans
- DHT: Trust‑weighted Kademlia with quorum and telemetry
- FEC: Forward Error Correction for resilient storage and transport
- Virtual Disk: entity‑scoped, encrypted/public disks for files & websites
- Website Root: identity‑bound key for DNS‑free sites
- MLS/ML‑DSA/ML‑KEM: PQC algorithms used throughout

---

## 11. References

- Saorsa Core (crates.io): `saorsa-core` (v0.3.17)
- Saorsa Core docs: `../saorsa-core/AGENTS_API.md`
- Communitas deploy/testnet: `finalise/DEPLOY_TESTNET.md`
- Project architecture/design: `ARCHITECTURE.md`, `DESIGN.md`

---

This document is maintained with the code. If an API here differs from the code in `src-tauri/src`, the code is the source of truth. Please open issues/PRs with proposed changes to keep agents in sync.


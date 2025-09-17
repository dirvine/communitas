# Communitas Agents Guide

_Last updated: 2025-09-15_

This playbook is for anyone (human or AI) automating Communitas. It captures the layout of the mono-repo, the critical flows that new agents must support, and the tooling expectations that keep our workflows green.

## 1. Workspace at a Glance
- `apps/communitas/` – React/Material UI console that now fronts the identity and storage surfaces. Uses the Tauri bindings defined in `communitas-desktop`.
- `communitas-desktop/` – Tauri v2 desktop crate. The only place we expose IPC commands (see `src/core_commands.rs`, `core_groups.rs`, `core_cmds.rs`, `container.rs`, `sync.rs`, `security/raw_spki.rs`).
- `communitas-core/` – Shared Rust library. `CoreContext` wires saorsa-core (v0.3.21) managers together (including the exported `get_user_four_words` helpers), persists PQC identities to the platform keyring, and caches group signing keys.
- `communitas-headless/` – Headless QUIC node with self-update, bootstrap, and metrics support. Ideal for CI smoke checks and autonomous seeders.
- `crates/communitas-container/` – Pointer-only container/CRDT engine that produces signed tips and optional FEC metadata. Desktop and headless both depend on it.
- `src/` – Legacy React SPA still compiled for regression coverage. Tests under `src/**/__tests__` remain part of CI; do not delete until the migration completes.

## 2. Core Agent Flow
1. **Claim identity** – Call `core_claim(words: [String; 4])`. Keys are persisted in the keyring (`communitas-core::keystore`).
2. **Advertise presence** – `core_advertise(addr, storage_gb)` signs a presence heartbeat and returns optional Four-Word IPv4 endpoints for UI display.
3. **Initialize runtime** – `core_initialize` instantiates `CoreContext`, creating enhanced identities, chat/messaging services, and per-device storage handles.
4. **Messaging & channels** – Use `core_create_channel`, `core_send_message_to_channel`, `core_send_message_to_recipients`, `core_subscribe_messages`, and UI receives `message-received` events with decrypted payloads when possible. New in saorsa-core v0.3.21: `core_channel_list_members` and `core_resolve_channel_members` hydrate Four-Word handles directly from the address book so automations can map user IDs without guessing.
5. **Groups** – `core_group_create`, `core_group_add_member`, `core_group_remove_member` manage ML-DSA signed membership packets. Group signing keys are cached in-memory on the Tauri side.
6. **Container & virtual disk pointers** – `container_init`, `container_put_object`, `container_get_object`, `container_apply_ops`, and `container_current_tip` provide pointer-only storage. Use `core_private_put/get` for encrypted KV storage in the local store.
7. **Sync & repair** – `sync_start_tip_watcher` emits `container-tip` events; `sync_fetch_deltas` pulls CRDT ops over raw-key-pinned QUIC; `sync_repair_fec` exposes Reed–Solomon recovery helpers. Pin raw SPKI values via `sync_set_quic_pinned_spki`.
8. **Bootstrap maintenance** – `core_get_bootstrap_nodes` / `core_update_bootstrap_nodes` read/write `bootstrap.toml` so automations can configure seeds.

## 3. Storage & Container Notes
- Container engine lives in `crates/communitas-container`. It encrypts payloads with AEAD (default on) and can emit FEC shards (k=4, m=2) for higher-layer distribution.
- Desktop persists opaque blobs to `COMMUNITAS_DATA_DIR` (defaults to `src-tauri/.communitas-data`) so offline reads never block.
- Pointer-only DHT policy: the app never writes large blobs directly to the DHT. Publish signed tips or manifests and store payloads locally or via delegated providers.

## 4. Messaging, Channels, and Events
- `message-received` (App side) delivers decrypted payloads when `MessagingService::decrypt_message` succeeds; otherwise payloads are tagged `encrypted: true`.
- `channel-member-resolved` events fire when `core_resolve_channel_members` iterates channel membership and resolves human metadata. Payload now includes both `four_words` (array of words) and `four_words_text` (hyphenated string) sourced from `saorsa_core::get_user_four_words`.
- Channel helpers: `core_channel_list_members`, `core_channel_invite_by_words` (currently returns an error until saorsa-core exposes membership writes), and `core_channel_recipients` (placeholder for UI fallbacks). `core_send_message_to_channel` now looks up Four-Word addresses via the saorsa-core address book before falling back to heuristics.

## 5. Sync, Security & Networking
- `sync_progress` events provide `{ phase, peer, ops?, new_count?, root? }` updates during QUIC delta fetches.
- Raw SPKI pinning flows: prefer `sync_set_quic_pinned_spki`/`sync_clear_quic_pinned_spki`, or set `COMMUNITAS_QUIC_PINNED_SPKI`/`COMMUNITAS_RPK_ALLOW_ANY` in dev.
- QUIC/IPv4 first: `sync_fetch_deltas` resolves addresses via `lookup_host`, ordering IPv4 before IPv6.
- Headless binary exposes the same container + sync stack via CLI. Config file controls FEC, bootstrap, update cadence, and metrics (`127.0.0.1:9600`).

## 6. Tooling & Workflows
- **Rust**: `cargo fmt --all`, `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`, `cargo test -p communitas-desktop`, `cargo test -p communitas-core`, `cargo test -p communitas-headless`.
- **Node/React**: `npm ci`, `npm run typecheck`, `npm run test:run` (fast Vitest slice), `npm run build`.
- **Desktop builds**: `cargo build --release -p communitas-desktop`, `npm run tauri build` for signed bundles (requires TAURI_PRIVATE_KEY in CI).
- **Headless smoke**: `cargo build --release -p communitas-headless` then `./target/release/communitas-headless --help`.
- GitHub workflows in `.github/workflows/` assume Node 20 and Rust stable; keep scripts aligned when changing tooling.

## 7. Observability & Logs
- Tracing uses `tracing_subscriber` with `RUST_LOG=info,communitas=debug,saorsa_core=debug` by default. Override per workflow.
- Container watchers emit `container-tip`; sync flows emit `sync-progress`; UI network diagnostics remain under `window.testNetwork.*` in legacy `src/` tests.
- Metrics: headless node exposes Prometheus-like endpoint when `--metrics` flag is used.

## 8. Reference Library
- Low-level API details: `AGENTS_API.md` (same directory).
- saorsa-core references: see `communitas-core/src/` and `COMMUNITAS_ARCHITECTURE.md`.
- Deployment & bootstrap specifics: `finalise/` docs, `bootstrap.toml.template`, `deployment/` scripts.
- MCP automation examples: `MCP_SERVERS.md`, `servers/mcp-puppeteer`.

Keep this file updated when:
- saorsa-core is bumped,
- new Tauri commands are surfaced,
- container/FEC defaults change,
- workspace layout shifts (e.g., once `apps/communitas` fully replaces `src/`).

# Communitas Agents Guide

_Last updated: 2025-09-15_

This playbook is for anyone (human or AI) automating Communitas. It captures the layout of the mono-repo, the critical flows that new agents must support, and the tooling expectations that keep our workflows green.

## 1. Workspace at a Glance
- `apps/communitas/` – React/Material UI console that now fronts the identity and storage surfaces. Uses the Tauri bindings defined in `communitas-desktop`.
- `communitas-desktop/` – Tauri v2 desktop crate. The only place we expose IPC commands (see `src/core_commands.rs`, `core_groups.rs`, `core_cmds.rs`, `container.rs`, `sync.rs`, `security/raw_spki.rs`).
 - `communitas-core/` – Shared Rust library. `CoreContext` wires saorsa-gossip networking components together (replacing saorsa-core), persists PQC identities to the platform keyring, and caches group signing keys.
- `communitas-headless/` – Headless QUIC node with self-update, bootstrap, and metrics support. Ideal for CI smoke checks and autonomous seeders. Pass `--instance-id`, `--config`, and `--storage` (or set the matching `COMMUNITAS_*` env vars) when running more than one node so each instance keeps its own config and data roots.

- `src/` – Legacy React SPA still compiled for regression coverage. Tests under `src/**/__tests__` remain part of CI; do not delete until the migration completes.

## 2. Core Agent Flow
1. **Claim identity** – Call `core_claim(words: [String; 4])`. Keys are persisted in the keyring (`communitas-core::keystore`).
2. **Advertise presence** – `core_advertise(addr, storage_gb)` signs a presence heartbeat and returns optional Four-Word IPv4 endpoints for UI display.
3. **Initialize runtime** – `core_initialize` instantiates `CoreContext`, creating enhanced identities, chat/messaging services, and per-device storage handles.
4. **Messaging & channels** – Use `core_create_channel`, `core_send_message_to_channel`, `core_send_message_to_recipients`, `core_subscribe_messages`, and UI receives `message-received` events with decrypted payloads when possible. New in saorsa-core v0.3.23: `core_channel_list_members` and `core_resolve_channel_members` hydrate Four-Word handles directly from the address book so automations can map user IDs without guessing.
5. **Groups** – `core_group_create`, `core_group_add_member`, `core_group_remove_member` manage ML-DSA signed membership packets. Group signing keys are cached in-memory on the Tauri side.
6. **Document storage** – Use Yrs CRDT-based document commands (`doc_*`) for collaborative editing. Full document replication is used (not pointer-based). Use `core_private_put/get` for encrypted KV storage in the local store.
7. **Sync & networking** – QUIC connections are secured with raw SPKI pinning via `sync_set_quic_pinned_spki`/`sync_clear_quic_pinned_spki`. Documents sync automatically via the gossip overlay.
8. **Bootstrap maintenance** – `core_get_bootstrap_nodes` / `core_update_bootstrap_nodes` read/write `bootstrap.toml` so automations can configure seeds.

## 3. Storage & Container Notes
- Desktop persists data to `COMMUNITAS_DATA_DIR` (defaults to `communitas-desktop/.communitas-data`) so offline reads never block.
- Document storage uses Yrs CRDT for collaborative editing with entity-scoped storage in both encrypted (Files) and public (Web) modes.

## 4. Messaging, Channels, and Events
- `message-received` (App side) delivers decrypted payloads when `MessagingService::decrypt_message` succeeds; otherwise payloads are tagged `encrypted: true`.
- `channel-member-resolved` events fire when `core_resolve_channel_members` iterates channel membership and resolves human metadata. Payload now includes both `four_words` (array of words) and `four_words_text` (hyphenated string) sourced from `saorsa_core::get_user_four_words`.
- Channel helpers: `core_channel_list_members`, `core_channel_invite_by_words` (currently returns an error until saorsa-core exposes membership writes), and `core_channel_recipients` (placeholder for UI fallbacks). `core_send_message_to_channel` now looks up Four-Word addresses via the saorsa-core address book before falling back to heuristics.

## 5. Sync, Security & Networking
- Raw SPKI pinning flows: prefer `sync_set_quic_pinned_spki`/`sync_clear_quic_pinned_spki`, or set `COMMUNITAS_QUIC_PINNED_SPKI`/`COMMUNITAS_RPK_ALLOW_ANY` in dev.
- QUIC/IPv4 first: addresses are resolved via `lookup_host`, ordering IPv4 before IPv6.
- Headless binary exposes the same sync stack via CLI. Config file controls bootstrap, update cadence, and metrics (`127.0.0.1:9600`).

## 6. Tooling & Workflows

### Tauri Desktop App Development & Distribution

**CRITICAL**: The Tauri app serves frontend assets from the `dist/` directory, NOT from a live dev server. You must build the frontend first.

#### Development Workflow (Recommended)
```bash
# 1. Build frontend assets into dist/
npm run build

# 2. Run Tauri with built assets
npm run tauri dev
```

#### Alternative: Live Dev Server (Hot Reload)
```bash
# Configure tauri.conf.json for live dev server:
# "devUrl": "http://localhost:5173"
# "beforeDevCommand": "npm run dev:frontend"

npm run tauri dev  # Will start Vite dev server automatically
```

#### Distribution Build
```bash
# 1. Build optimized frontend
npm run build

# 2. Create Tauri bundle
npm run tauri build
```

#### Common Issues & Solutions

**Issue**: Tauri shows "Loading..." or placeholder content
- **Cause**: `dist/` directory missing or contains old/placeholder content
- **Fix**: Run `npm run build` to populate `dist/` with current frontend assets

**Issue**: "frontendDist path doesn't exist" compilation error
- **Cause**: `dist/` directory was deleted but `tauri.conf.json` references it
- **Fix**: Create `dist/` directory or run `npm run build`

**Issue**: Port conflicts with `devUrl`
- **Cause**: Port 5000 often used by macOS AirTunes/ControlCenter
- **Fix**: Use port 5173 (Vite default) or check `lsof -i :PORT`

**Issue**: Changes not reflected in Tauri app
- **Cause**: Serving stale assets from `dist/` instead of live dev server
- **Fix**: Either run `npm run build` after changes OR configure `devUrl` properly

### Build Commands Reference
- **Rust**: `cargo fmt --all`, `cargo clippy --all-features -- -D clippy::panic -D clippy::unwrap_used -D clippy::expect_used`, `cargo test -p communitas-desktop`, `cargo test -p communitas-core`, `cargo test -p communitas-headless`.
- **Node/React**: `npm ci`, `npm run typecheck`, `npm run test:run` (fast Vitest slice), `npm run build`.
- **Desktop builds**: `cargo build --release -p communitas-desktop`, `npm run tauri build` for signed bundles (requires TAURI_PRIVATE_KEY in CI).
- **Headless smoke**: `cargo build --release -p communitas-headless` then `./target/release/communitas-headless --help`.
- **Headless instance launch**: `./target/release/communitas-headless --instance-id seed-a --config ~/.config/communitas/seed-a/config.toml --storage ~/.local/share/communitas/seed-a` (binary will bootstrap the config file if it does not exist).
- GitHub workflows in `.github/workflows/` assume Node 20 and Rust stable; keep scripts aligned when changing tooling.

## 7. Observability & Logs
- Tracing uses `tracing_subscriber` with `RUST_LOG=info,communitas=debug,saorsa_gossip=debug` by default. Override per workflow.
- UI network diagnostics remain under `window.testNetwork.*` in legacy `src/` tests.
- Metrics: headless node exposes Prometheus-like endpoint when `--metrics` flag is used.

## 8. MCP (Model Context Protocol) Automation

We now rely solely on the Chrome DevTools MCP server for automated browser inspection and testing. Connect an MCP-aware client with:

```bash
npx chrome-devtools-mcp@latest --browserUrl http://127.0.0.1:5173/
```

This setup exposes DOM traversal, screenshot capture, and scripted JavaScript execution through MCP without any Tauri-specific plugins or local socket servers.


### 9. Production Bootstrap & Deployment
4: 
5: **Headless Bootstrap Deployment**
6: The network is currently bootstrapped by two Digital Ocean droplets:
7: 
8: - **Node 1 (Bootstrap Seed)**: `138.197.29.195` (NYC3)
9:   - Identity: `ocean-forest-moon-star`
10:   - QUIC: `0.0.0.0:4433`
11:   - Metrics: `127.0.0.1:9600` (SSH tunnel to view)
12:   - Config: `/root/config.toml`
13: 
14: - **Node 2 (Peer)**: `167.71.188.131` (NYC3)
15:   - Identity: `chase-solid-alpha-vatican` (Generated)
16:   - QUIC: `0.0.0.0:4434`
17:   - Metrics: `127.0.0.1:9601`
18:   - Bootstraps from Node 1.
19: 
20: **Deployment Steps**:
21: 1. SSH into droplet: `ssh root@138.197.29.195`
22: 2. Update code: `cd communitas && git pull origin main`
23: 3. Build: `source $HOME/.cargo/env && cargo build --release -p communitas-headless`
24: 4. Config: Ensure `config.toml` has correct `bootstrap_nodes` and `listen_addrs`.
25: 5. Run: `nohup ./target/release/communitas-headless --config /root/config.toml --metrics > headless.log 2>&1 &`
26: 
27: ## 10. Reference Library
28: - Low-level API details: `AGENTS_API.md` (same directory).
29: - saorsa-core references: see `communitas-core/src/` and `COMMUNITAS_ARCHITECTURE.md`.
30: - Deployment & bootstrap specifics: `finalise/` docs, `bootstrap.toml.template`, `deployment/` scripts.
31: - Chrome DevTools MCP usage: see `CLAUDE.md` for inspector workflow.
32: 
33: Keep this file updated when:
34: - saorsa-gossip packages are bumped,
35: - new Tauri commands are surfaced,
36: - workspace layout shifts (e.g., once `apps/communitas` fully replaces `src/`),
37: - deployment infrastructure changes.

# Communitas — Product & Technical Specification
Version: 0.9 • Date: 2025-09-04 • Owner: Saorsa Labs (dirvine)

> Scope: Desktop app (Tauri v2) and headless node on `saorsa-core` v0.3.16. Features: world‑class UX, channels with threads, projects, entity‑scoped storage, group E2EE, browser access via WebRTC bridge, phased auto‑updates, and a 6‑region DigitalOcean testnet.  
> Note: no MCP in this version.

---

## 1. Goals
- Local‑first collaboration. No central servers.
- High‑polish UI. Smooth motion. Tight typography.
- Slack/Discord‑class messaging with threads and reactions.
- Per‑entity storage: groups, channels, projects. Public, private, shared.
- End‑to‑end group crypto by default. PQC‑ready.
- Browser access via WebRTC ↔ QUIC bridge.
- Real‑world testnet across 6 DigitalOcean regions.
- Auto‑update with 0–6 h jitter. Prompt on desktop. Silent on headless.

### Non‑Goals
- Cloud data dependence.
- Legacy crypto without PQC path.
- Centralized message brokers.

---

## 2. Architecture Overview
**App**: Tauri v2 desktop + WebView UI.  
**Node**: `saorsa-core` peer with DHT, QUIC (`ant‑quic`), RSPS, storage, WebRTC bridge.  
**Storage**: Local append‑only log + encrypted object store. FEC for durability.  
**Identity**: User handle + device keys. Four‑word endpoints for network addresses.  
**Group Crypto**: MLS‑inspired sessions; attachments sealed via `saorsa‑seal` with FEC.  
**Testnet**: 6 DO droplets as bootstrap peers.  
**Updates**: Tauri updater (desktop). Self‑update service (headless).

---

## 3. Data Model
### Entities
- **Group**: root namespace. Public metadata, encrypted roster, policies.
- **Channel**: child of Group. Threads, timeline, pins.
- **Project**: sibling to Channel. Boards, tasks, docs, files.
- **Thread**: linked to message or topic. Ordered message log.
- **Message**: event‑sourced. Edits/deletes are events with references.
- **File/Asset**: chunked, FEC’d, sealed. Addressed by content hash.

### Identifiers
- **UserHandle**: human username.
- **Endpoint (four‑word)**: human‑readable IP+port derived from actual deployment address.
- **Cid**: content id for immutable blobs.

### DHT records
- Presence, routes, pointers, small metadata. RSPS for provider summaries.

---

## 4. Security & Crypto
- **Signatures**: ML‑DSA‑65 for identities and messages.
- **KEM**: ML‑KEM‑768 for session setup.
- **Hash**: BLAKE3.
- **AEAD**: XChaCha20‑Poly1305 for sealed data.
- **Group messaging**: MLS‑inspired tree with epoch rotation.  
Per‑device keys. Attachments use envelope keys wrapped to epoch key then sealed and FEC’d.

---

## 5. Networking
- **Transport**: QUIC via `ant‑quic`. NAT traversal. **IPv4‑first**, IPv6 fallback.
- **Bootstrap**: DO seeds ship in default config. App fetches and caches on first run.
- **Browser access**: WebRTC DataChannels to local node via WebRTC bridge.

---

## 6. App UX specification
### Design system
- **Typography**
  - UI: `Inter Variable` or platform stack `-apple-system, Segoe UI, Roboto, Inter, system-ui`.
  - Mono: `JetBrains Mono` for code.
  - Scale: 14 / 16 / 18 / 22 / 28 / 36 px.
- **Color**: Saorsa blue‑green. Dark default. WCAG AA.
- **Spacing**: 8‑pt system.
- **Icons**: 20/24 px outline and filled.

### Motion
- Transitions: 180–240 ms in; 120–160 ms out. Ease‑in‑out.
- Springs: stiffness 170, damping 26 for panel slides.
- Micro‑motion: hover lift 2dp; ripple 150 ms; optimistic send settle 250 ms.

### Navigation
- Left rail: **Home**, **Inbox**, **Groups**. Group → Channels & Projects.
- Thread pane: side sheet on desktop, over‑sheet on narrow widths.
- Command‑K palette. Slash commands in composer.
- Markdown: inline code, fenced blocks, Mermaid fences. Yjs collaborative docs.

### Messaging
- Inline threads, replies, reactions, mentions, link unfurl.
- Attachments show upload→seal→store progress.
- Presence: per‑device with churn smoothing.

### Accessibility
- Targets ≥ 40 px. Full keyboard nav. Respect reduce‑motion.

---

## 7. Storage & Sync
- **Local**: append‑only event store + SQLite indices.
- **CRDT**: Yjs for docs. Deltas E2EE.
- **Objects**: chunk → FEC (`saorsa‑fec`) → seal (`saorsa‑seal`) → placement engine.
- **GC**: content‑addressed ref counting with horizon sweeps.

---

## 8. Desktop app (Tauri v2)
- Stack: Tauri v2 + SolidJS or Svelte + Tailwind + Motion One.
- Updater: `tauri-plugin-updater`. Check on start and every 6 h.
- UX: **Update now** or **Later**. If ignored, apply randomized defer.

- IPC surface:
  - `post_message(group, channel, body)`
  - `create_thread(group, channel, root_msg_id, title?)`
  - `upload_asset(path, visibility)`
  - `join_group(invite)`
  - `list_channels(group)`
  - `query_dht(key)`
  - `open_webrtc_session(peer)`

---

## 9. Headless node
- Binary: `communitas-node` on `saorsa-core`. No UI.
- Config: TOML with listen addrs, storage path, bootstrap list.
- Service: `systemd` unit. Optional timer for maintenance.
- Control: local HTTP on 127.0.0.1 for ops (enable/disable, health, version).

---

## 10. Auto‑update strategy
**Rollout with jitter**
- On detecting version **V**, draw `delay ∈ Uniform[0, 6h]`.
- Desktop: prompt immediately; if no action, auto‑apply at `now+delay` when idle.
- Headless: auto‑apply at `now+delay` without prompt.

**Desktop**
- Use Tauri updater APIs: `check()` → `downloadAndInstall()`. Show progress and changelog.

**Headless**
- `communitas-autoupdater` watches GitHub Releases or updater JSON.
- Verify signatures. Download. Atomic swap. Restart service.
- Exponential backoff on failure up to 12 h.

**Signing**
- Ed25519 today plus PQC ML‑DSA‑65 detached signature. Verify both when available.

---

## 11. DigitalOcean Testnet (6 regions)
- Regions: AMS3, LON1, FRA1, NYC3, SFO3, SGP1.
- Droplets: `s-1vcpu-2gb`, Ubuntu 24.04, **IPv4 public required**, IPv6 optional.
- Open: UDP 443 (QUIC), TCP 443 (WebRTC fallback), TCP 22 (SSH), ICMP. Restrict SSH.

**Four‑word endpoints**
- These depend on the final **IPv4** and port. Compute after provision. Update `bootstrap.toml` with the derived words.

**Cloud‑init (user‑data)**
```bash
#cloud-config
package_update: true
package_upgrade: true
runcmd:
  - sysctl -w net.core.rmem_max=2500000
  - ufw allow 443/udp
  - ufw allow 443/tcp
  - ufw --force enable
  - useradd -m -s /bin/bash communitas || true
  - mkdir -p /opt/communitas/bin /var/lib/communitas
  - curl -L {RELEASE_URL} -o /opt/communitas/bin/communitas-node
  - curl -L {AUTO_URL} -o /opt/communitas/bin/communitas-autoupdater
  - chmod +x /opt/communitas/bin/communitas-node /opt/communitas/bin/communitas-autoupdater
  - chown -R communitas:communitas /opt/communitas /var/lib/communitas
  - install -d -o communitas -g communitas /etc/communitas
  - printf '%s\n' "[update]\nchannel=stable\n" > /etc/communitas/update.toml
  - systemctl daemon-reload
  - systemctl enable communitas.service
  - systemctl start communitas.service
  - systemctl enable communitas-updater.service
  - systemctl start communitas-updater.service
```

**systemd unit**
```ini
[Unit]
Description=Communitas Node
After=network-online.target
Wants=network-online.target

[Service]
User=communitas
ExecStart=/opt/communitas/bin/communitas-node --config /etc/communitas/config.toml
Restart=always
RestartSec=5
LimitNOFILE=1048576

[Install]
WantedBy=multi-user.target
```

**Bootstrap**
- After droplets are live, derive four‑word endpoints from each **IPv4:443** and write to `bootstrap.toml`. See `DEPLOY_TESTNET.md` for the helper step.

---

## 12. Observability
- Optional Prometheus metrics via `saorsa-core`. Localhost only. Opt‑in.

---

## 13. Web access
- Browser → local node via WebRTC DataChannels.
- Serve web UI as static files from the node. Connect via bridge.
- TURN later for hard NATs.

---

## 14. Performance targets
- Send to 6 peers: p50 < 150 ms LAN, p95 < 600 ms WAN.
- Open thread with 10k msgs: FMP < 120 ms, p95 < 250 ms.
- Yjs doc echo < 100 ms within region.

---

## 15. Security targets
- No plaintext user content off device.
- Member add rotates epoch keys ≤ 5 s.
- Releases must verify signatures or quarantine.

---

## 16. Milestones
- **M1** Testnet + plumbing
- **M2** Messaging core
- **M3** Files + projects
- **M4** WebRTC + web UI
- **M5** Polished UX + updater

---

## 17. Open Questions
- TURN/STUN choices for hardest NATs.
- Mobile clients and UI parity.
- Public read‑only hosting strategy.

# Communitas — Engineering Tasks

## Conventions
- Rust 1.80+. Clippy clean for core. Tests may use unwrap.
- Workspaces: `communitas-app`, `communitas-node`, `communitas-proto`.
- Feature flags: `metrics`, `webrtc`, `fec-seal`, `headless`.

---

## M1 — Testnet & Plumbing
### Node
- [ ] QUIC listeners using `ant-quic`. Happy-Eyeballs dialer (IPv4-first, IPv6 fallback).
- [ ] Bootstrap manager: load `bootstrap.toml`, resolve four-word endpoints, cache.
- [ ] Presence DHT records: put/get with TTL and renewal.
- [ ] RSPS integration for provider summaries.
- [ ] Health endpoint: `GET /health` reports version, peers, storage stats.
- [ ] Metrics (opt-in): Prometheus on 127.0.0.1:9600 when `metrics` enabled.

### Headless packaging
- [ ] Produce static Linux x86_64 and aarch64 binaries.
- [ ] `systemd` unit and sample `cloud-init`.
- [ ] `communitas-autoupdater` with jitter 0–6 h and signature checks.

### Testnet
- [ ] Terraform or doctl scripts to create 6 droplets (AMS3, LON1, FRA1, NYC3, SFO3, SGP1).
- [ ] Ensure public **IPv4** and optional IPv6 enabled. Configure firewall profiles.
- [ ] After provision, derive four-word endpoints from each droplet IPv4:443 and write `bootstrap.toml`.
- [ ] Publish `bootstrap.toml` to repo and website.

**DoD**: Two regions join and exchange pings and small messages reliably.

---

## M2 — Messaging Core
### Protocol and store
- [ ] Message event schema with edit/delete referencing.
- [ ] Channel timelines with virtualized list.
- [ ] Threads: side-pane view; link back to root.
- [ ] Reactions and mentions model.
- [ ] Local store: append-only log + SQLite indices.
- [ ] E2EE for messages with MLS-inspired group epochs.

### IPC
- [ ] `post_message`, `create_thread`, `list_channels`, `join_group`.
- [ ] Basic search over local indices.

**DoD**: Two users in a group chat with threads. Restart safe. Keys rotate on member change.

---

## M3 — Files & Projects
### Assets
- [ ] Chunk → `saorsa-fec` → `saorsa-seal` pipeline.
- [ ] Progress UI: upload → seal → place.
- [ ] Dedup via content IDs. Reference counting GC.

### Projects
- [ ] Entities: boards, tasks, docs.
- [ ] Yjs docs with encrypted deltas.

**DoD**: Upload and share files within a group. Projects visible with collaborative docs.

---

## M4 — WebRTC & Web UI
### Bridge
- [ ] WebRTC DataChannel service in node.
- [ ] Browser client connects to local node and mirrors desktop features.

**DoD**: Browser client sends/receives in a channel and opens threads.

---

## M5 — Polish & Updater
### UX system
- [ ] Fonts, color, grid, motion tokens applied.
- [ ] Command-K and slash commands.
- [ ] Accessibility audit. Keyboard coverage. Reduce-motion support.

### Updater
- [ ] Desktop: in-app prompt, changelog, “Update now/Later”.
- [ ] Headless: jittered auto-apply, exponential backoff ≤ 12 h, atomic swap.

**DoD**: Signed release rolled out with jitter across testnet and desktops.

---

## Cross-cutting
- [ ] Signature scheme: Ed25519 + ML-DSA-65. Publish keys and rotation policy.
- [ ] Crash-safe updates with verify-then-swap.
- [ ] Chaos tests: churn, 2–10% loss, time skew ±120 s.
- [ ] Load tests: 10k msgs/thread open perf targets.
- [ ] SAST/DAST and supply-chain: cargo deny, SBOM, reproducible builds.

---

## Deliverables
- [ ] `COMMUNITAS_SPEC.md` in repo.
- [ ] `DEPLOY_TESTNET.md` with scripts and validation steps.
- [ ] Release notes template and signer public keys.

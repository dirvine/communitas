# Communitas Tasks

## Epic A: Core bindings
- [ ] Create `core-sdk` wrapper in Communitas that wraps saorsa‑core.
- [ ] Add Tauri commands: `core_claim`, `core_advertise`, `container_put`, `container_get`.
- Acceptance: demo claims identity, advertises endpoint, stores and retrieves a message in a two‑user dyad.

## Epic B: DHT pointer records
- [ ] Implement typed structs for identity, presence, group, channel, container_tip.
- [ ] Canonical CBOR encode and ML‑DSA sign/verify.
- [ ] Enforce TTLs and size limits.
- Acceptance: round‑trip encode/sign/verify tests per schema.

## Epic C: Container engine
- [ ] CRDT log for markdown threads.
- [ ] Object store with AEAD and FEC.
- [ ] Local indexer for search.
- Acceptance: create 1k posts across 5 members with 15% packet loss and recover to a single tip.

## Epic D: Sync and repair
- [ ] Tip watcher that tracks group/channel tips.
- [ ] Delta fetcher over ant‑quic with IPv4 first.
- [ ] Shard repairer to meet redundancy target.
- Acceptance: kill two nodes and verify reads still succeed via FEC.

## Epic E: Presence and media
- [ ] Presence updates every 60 seconds.
- [ ] Media setup uses presence hints and ant‑quic.
- Acceptance: direct call succeeds on IPv4. Relay fallback is exercised and logged when forced.

## Epic F: Security
- [ ] Integrate saorsa‑pqc ML‑DSA/ML‑KEM.
- [ ] Encrypted key store for identity and device keys.
- [ ] Anti‑phishing UI with four‑word IDs.
- Acceptance: membership change rotates epoch and blocks the removed member from new writes.

## Epic G: UI
- [ ] Sync status per group and channel.
- [ ] Storage policy switch: private, mesh‑replicated, public‑readable.
- [ ] Invite flow shows four‑word IDs and trust notes.
- Acceptance: manual test script covers each flow.

## Testing matrix
- [ ] Local testnet of 5 nodes with churn and loss.
- [ ] Tip divergence fuzzer.
- [ ] Shard audit tasks.
- [ ] IPv6 fallback test.

## Release checklist
- [ ] Reproducible builds for macOS, Windows, Linux.
- [ ] Code signing set up.
- [ ] App reports only anonymous health metrics if enabled.
- [ ] Docs updated: ARCHITECTURE, DHT_SCHEMAS, CORE_INTEGRATION, TASKS.

# Communitas Integration with Saorsa Gossip — SPEC

Version: 0.1  
Status: Draft for implementation  
Scope: Replace DHT usage with `saorsa-gossip`. Map Communitas objects to topics and MLS groups. Define boot, backup, UI, rollout.

---

## 1. Mapping

| Communitas | saorsa‑gossip |
|---|---|
| User identity (four‑word) | ML‑DSA identity + alias |
| Contact | Overlay edge and seed |
| Channel / Project / Org | MLS group + gossip topic |
| Presence | MLS‑encrypted rotating beacons (ChaCha20Poly1305) |
| Backup | Favourite contacts hold encrypted replicas (ChaCha20Poly1305) |

Repos: Communitas app, saorsa‑mls, saorsa‑pqc, ant‑quic.

**Encryption**: All encrypted data uses ChaCha20Poly1305 AEAD for symmetric encryption, providing authenticated encryption with associated data. This is preferred over AES-GCM for better performance on non-hardware-accelerated platforms.

---

## 2. Boot sequence

1. Load ML‑DSA identity.  
2. Dial 1–3 favourite contacts over `ant‑quic`.  
3. Start membership (HyParView+SWIM).  
4. For each joined channel/org: join MLS group, subscribe to topic.  
5. Begin presence beacons and CRDT anti‑entropy.

---

## 3. Replace DHT calls

- Remove `saorsa-core` DHT announce/lookup. Use `Presence::find` and FOAF across contacts.  
- Keep a small optional introducer list for cold start only.  
- All publish/subscribe via Plumtree.

---

## 4. Data and backup

- Local‑first state stored in CRDT structures
- Mark **Favourite** contacts to store encrypted replicas of: contact list, device list, minimal account metadata
- Encryption: ChaCha20Poly1305 AEAD with per-favourite derived keys
- Recovery: connect to any favourite, retrieve encrypted replica, decrypt, run delta‑CRDT anti‑entropy, rejoin MLS groups
- Backup format: Bincode-serialized CRDT state → ChaCha20Poly1305 encryption → send via QUIC Bulk stream

---

## 5. Presence model

- “Online” means a valid beacon seen in at least one shared group within TTL.  
- No global presence.  
- UI shows group‑scoped presence and last‑seen.

---

## 6. Telemetry

- Metrics per topic: P50/P95 delivery latency, bytes per delivered message, mesh degree, score distribution.  
- Events: join/leave, suspicion, reconvergence, anti‑entropy stats.

---

## 7. Rollout plan

- Phase 1: behind feature flag `gossip_overlay`.  
- Phase 2: dual‑write to MLS topics, mirror presence, collect KPIs.  
- Phase 3: remove DHT dependency.  
- Phase 4: enable Bluetooth bridge on mobile builds.

---

## 8. Developer tasks

- Wire `saorsa-gossip` crate.  
- Replace discovery calls.  
- Map channels to MLS groups and topics.  
- Build presence UI and recovery flow.  
- Add Bluetooth Mesh gateway service for collapse mode.

---

## 9. References

- MLS RFC 9420.  
- QUIC RFC 9000/9001.  
- HyParView, SWIM, Plumtree, GossipSub v1.1.  
- CRDT and delta‑CRDT papers.  
- Bluetooth Mesh specifications.

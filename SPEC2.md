# Communitas Integration with Saorsa Gossip — SPEC (PQC‑only, No DNS)

Version: 0.2  
Status: Draft for implementation  
Scope: Wire Communitas to the PQC‑only gossip overlay with ant‑quic traversal, Coordinator Adverts, Rendezvous Shards, and Saorsa Sites. No DNS. No HTTP.

---

## 1. Mapping

| Communitas | saorsa‑gossip |
|---|---|
| User identity (four‑word) | ML‑DSA identity + alias |
| Contact | Graph edge and bootstrap seed |
| Channel / Project / Org | MLS group + gossip topic |
| Presence | MLS‑encrypted rotating beacons (ChaCha20Poly1305) |
| Website | Saorsa Site (SID) with ML‑DSA manifest |
| Backup | Favourite contacts hold encrypted replicas (ChaCha20Poly1305) |

**Encryption**: All symmetric encryption uses **ChaCha20Poly1305 AEAD** from `saorsa-pqc`. No AES-GCM.

---

## 2. Boot sequence (no DNS)

1. Load ML‑DSA identity.  
2. Try loopback and LAN beacons.  
3. Dial favourites from **peer cache**.  
4. If cache cold, request **Coordinator Adverts** via FOAF `FIND_COORDINATOR` (TTL=3, fanout=3).  
5. Connect to a coordinator for address reflection and punching.  
6. Join MLS groups and subscribe to topics.  
7. Start presence and CRDT anti‑entropy.

---

## 3. Replace DHT and any DNS

- Remove all DHT code paths.  
- Do not use DNS.  
- Use **peer cache** + **Coordinator Adverts** + **Rendezvous Shards** for global reach.

---

## 4. Presence and discovery

- Presence is group‑scoped only.  
- “Find user” first checks shared groups, then subscribes to the user’s rendezvous shard for Provider Summaries.  
- Apply capability tokens and rate limits in handlers.

---

## 5. Websites: Saorsa Sites

User flows:

- **Publish:** pick a site key (ML‑DSA). Build manifest, chunk assets, start provider. The client gossips Provider Summaries to the `SITE_ADVERT` shard(s).  
- **Fetch:** user enters SID or four‑word that maps to the SID inside Communitas. Client subscribes to the shard(s), selects providers by score, fetches manifest/blocks over QUIC, verifies, then renders.

Private sites:

- Create MLS group for the site. Encrypt blocks using exporter‑derived keys. Manifest stays ML‑DSA signed.

---

## 6. Peer cache

- Persist after first successful connections: `(peer_id, addr_hints, nat_class, roles, last_success, success_count)`.  
- On start, try the best‑scored entries first.  
- Evict or downgrade peers with repeated failures.

---

## 7. Telemetry

- Punch success rate vs relay rate.  
- Rendezvous shard lookup latency P50/P95.  
- Message delivery P50/P95 and overhead/msg.  
- Cache hit rate on boot.

---

## 8. Rollout

- Phase 1: feature flag `gossip_overlay_pqc`.  
- Phase 2: dual‑run old path for testing, then remove DHT code.  
- Phase 3: enable Saorsa Sites.  
- Phase 4: enable BLE bridge.

---

## 9. Developer tasks

- Integrate ant‑quic hooks and events in Communitas core.  
- Implement Coordinator Adverts publish/cache UI toggles.  
- Implement Rendezvous Shards client (subscribe, score, pick provider).  
- Implement Saorsa Sites publisher and reader.  
- Extend recovery flow with peer cache priming and FOAF query.  
- Add tests: NAT classes, shard load, site end‑to‑end verification.

---

## 10. Security policy

- **PQC only**: ML‑KEM for key establishment, ML‑DSA for signatures, **ChaCha20Poly1305 for symmetric encryption**.
- **No classical or hybrid ciphersuites** (no Ed25519, no X25519, no AES-GCM).
- All encryption uses `saorsa-pqc::symmetric::ChaCha20Poly1305Cipher`.
- Pin suite per group/site.
- Strict signature checks on all control frames and manifests.

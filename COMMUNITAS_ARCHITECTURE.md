# Communitas Architecture with Pointers‑Only DHT

## Goal
Keep the DHT small and authenticated. Store only pointers and tips. Hold content in group and channel containers that members host. Use IPv4 first with IPv6 fallback. Use ant‑quic for transport. Use saorsa‑pqc for signatures and encryption. Use saorsa‑fec for redundancy.

## High‑level picture
- Identity: four‑word phrase → 32‑byte key. Device binds under an identity.
- Presence: short‑TTL hints for media and reachability.
- Group and channel: each maps to a container. The DHT stores only `container_tip` and epoch.
- Container: append‑only event log plus encrypted object store with FEC.
- Transport: ant‑quic direct when possible. Relay via trusted peers when needed.
- Policy: local‑first. Owner‑controlled. PQC by default. No cloud requirement.

## Data flow
### Bring‑up
1. User claims identity with four words and ML‑DSA key.
2. Device advertises endpoint in presence.
3. App subscribes to group and channel tips in DHT.
4. For each tip, fetch deltas from any online member over ant‑quic.

### Post message to a channel
1. Editor produces CRDT ops for markdown.
2. Ops are sealed with group epoch key (HPKE for object keys + AEAD per object).
3. Data shards are generated with saorsa‑fec.
4. Local commit updates the container root.
5. New `container_tip` is signed (ML‑DSA) and published in DHT.

### File attach
- Preferred: pointer to external asset + integrity hash.
- Optional: store encrypted object shards in the container’s objects path.
- Channel message stores a reference to object hash or pointer URL plus hash.

### Membership change
1. Admin proposes MLS commit. Epoch rotates.
2. Future writes use the new epoch key.
3. New `container_tip` with new epoch is signed and published.

### Real‑time voice and screenshare
- Read `presence:{peer}`. Attempt ant‑quic direct on IPv4. Fallback to IPv6, then relay via a trusted peer with high EigenTrust.
- Media flow is outside DHT. Only presence hints are in DHT.

## Components in Communitas
- **CoreContext**: thin wrapper over saorsa‑core API.
- **Container Engine**: CRDT log, object store, FEC, indexer.
- **Syncer**: watches tips, fetches deltas, repairs shards.
- **Presence Service**: updates own presence and monitors peers.
- **Networking**: ant‑quic client with IPv4‑first policy.
- **UI**: shows four‑word IDs, sync state, storage policy, trust health.

## Storage strategy
Given group size `n`:
- 1 → direct store.
- 2 → full replication dyad.
- 3–5 → FEC(3,2).
- 6–10 → FEC(4,3).
- 11–20 → FEC(6,4).
- 20+ → FEC(8,5).
These are defaults. Groups may override.

## Security
- Signatures: ML‑DSA for tips and DHT records.
- Encryption: HPKE with ML‑KEM for sharing object keys. AEAD per object.
- Hash: BLAKE3 for addressing and integrity.
- Verification: reject any pointer or tip that fails signature or schema checks.
- UI safety: always display the four‑word ID when connecting or receiving invites.

## Trust and routing
- Maintain EigenTrust‑style scores. Penalize missing shards and misbehavior.
- Prefer diverse storage across regions and providers where possible.
- Backoff and retry based on trust and recent failures.

## Failure handling
- Offline member: FEC reconstruction from any k shards.
- Tip forks: last‑writer‑wins on signed tips with epoch ordering.
- NAT hard fail: use configured relay peers from the trust set.

## Test strategy
- Deterministic local testnet of 5 nodes.
- Packet loss and churn injection.
- Tip divergence fuzzer.
- Storage audit: random shard proof checks.

## Non‑goals
- No blobs in DHT.
- No centralized servers.
- No classical crypto fallback.

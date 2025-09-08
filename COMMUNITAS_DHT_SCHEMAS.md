# DHT Schemas for Pointers‑Only Design

All records are serialized with canonical CBOR and signed with ML‑DSA. Keys are four‑word IDs hashed to 32 bytes.

## Common fields
- `sig`: ML‑DSA signature over canonical bytes.
- `ver`: schema version.
- `ts`: unix seconds for freshness.

## identity:{id}
```json
{
  "ver": 1,
  "ts": 0,
  "pk_mldsa": "…",          // public signing key
  "devices": [{
    "id": "DeviceId",
    "device_type": "Active|Headless",
    "endpoint": {"protocol": "quic", "addr": "A.B.C.D:port"},
    "caps": {"storage_gb": 100}
  }],
  "sig": "…"
}
```
Notes: long TTL. Updated rarely. Endpoint duplication is allowed but presence gives fresher hints.

## presence:{id}
```json
{
  "ver": 1,
  "ts": 0,
  "active_device": "DeviceId",
  "endpoint_hint": {"protocol": "quic", "addr": "A.B.C.D:port"},
  "media": {"audio": true, "video": true, "screen": true},
  "ttl": 120,
  "sig": "…"
}
```
Notes: short TTL. Not authoritative. Clients ignore if expired.

## group:{gid}
```json
{
  "ver": 1,
  "ts": 0,
  "epoch": 42,
  "membership_commit": "blake3-256",
  "container_tip": "blake3-256",
  "write_quorum": 1,
  "sig": "…"
}
```
Notes: readers verify `membership_commit` through MLS state. Writers publish new tips when they commit.

## channel:{gid}:{cid}
```json
{
  "ver": 1,
  "ts": 0,
  "epoch": 42,
  "container_tip": "blake3-256",
  "sig": "…"
}
```
Notes: channels inherit group membership. A channel may have a different tip to allow independent compaction.

## container_tip:{gid|cid}:{epoch}
```json
{
  "ver": 1,
  "ts": 0,
  "content_root": "blake3-256",   // root of container DAG
  "version": {"major": 1, "minor": 0},
  "prev": "blake3-256",           // previous root or null on first
  "sig": "…"
}
```
Notes: content root points to the encrypted log and object index within members’ stores.

## Size limits
- identity ≤ 4 KiB
- presence ≤ 512 B
- group/channel ≤ 512 B
- container_tip ≤ 256 B

## Encoding rules
- Canonical CBOR.
- Fields sorted lexicographically before signing.
- BLAKE3 hash of the canonical bytes is the content address used in links.

## TTLs
- identity: 30 days
- presence: 120 seconds
- group/channel: 7 days with bump on updates
- container_tip: 90 days with bump on updates

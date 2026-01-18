# SimpleX Chat: Architecture Comparison with Communitas

**Date**: January 2026
**Purpose**: Document learnings from SimpleX Chat that may inform Communitas development

## Overview

[SimpleX Chat](https://simplex.chat/) is a privacy-focused messenger that eliminates user identifiers entirely. This document compares its architecture with Communitas to identify potential learnings.

## Fundamental Design Differences

| Aspect | SimpleX | Communitas |
|--------|---------|------------|
| Architecture | Client-Server with relay nodes | True P2P with gossip overlay |
| Identity | No persistent identifiers | Four-word human-readable identity |
| Message Delivery | Unidirectional queues via relays | Direct P2P + HyParView/Plumtree |
| Primary Focus | Maximum metadata privacy | Local-first collaboration platform |
| NAT Strategy | Avoid P2P entirely, use relays | NAT traversal via coordinator + MASQUE |

## SimpleX Key Innovations

### 1. 2-Hop Onion Routing

SimpleX routes messages: Client → Forwarding Server → Destination Server → Recipient

- Sender chooses forwarding relay, recipient chooses destination
- Messages mixed across connections, breaking correlation
- Lighter than Tor (only 2 hops)

**Communitas equivalent**: MASQUE relay + peer cache preferring public connections already provides IP protection for symmetric NAT users.

### 2. Unidirectional Queue Design

- Each connection uses TWO separate queues (send/receive)
- Each queue has different IDs for sender vs recipient
- Server never learns correlation between queues

**Communitas equivalent**: Topic-based isolation via Plumtree provides similar separation at the group level.

### 3. XFTP Chunked File Transfer

- Files split into fixed-size chunks (256KB, 1MB, 4MB)
- Chunks distributed to different relay servers
- Redundancy across multiple relays
- Fixed padding makes all transfers indistinguishable

**Future consideration**: Could enhance Sites protocol with multi-source block fetching for availability and parallelism.

### 4. Super-Peers for Large Groups (2025+)

SimpleX is moving toward designated "super-peers" that re-broadcast messages to subsets, solving O(n) fan-out for large groups.

**Future consideration**: When Communitas groups exceed ~200 members, consider formalizing relay members in Plumtree.

### 5. Asynchronous Relay Storage

Messages stored on relays up to 21 days, files 48 hours. Enables true async delivery.

**Communitas decision**: Keep P2P pure. CRDT anti-entropy + favourite contact backup maintains decentralization.

## What SimpleX Does That We Should NOT Follow

### 1. Avoiding P2P Entirely
SimpleX routes ALL messages through relays. This adds latency and creates infrastructure dependency. Communitas's direct P2P with coordinator assistance is superior for collaboration.

### 2. No User Identity
Makes contact discovery extremely difficult. Communitas's four-word networking provides human-readable identity with FOAF discoverability.

### 3. ML-KEM Skepticism
SimpleX uses sntrup761 (NTRU Prime) instead of ML-KEM, citing concerns about NIST modifications. However, ML-DSA/ML-KEM are the standardized algorithms with FIPS certification path (via aws-lc-rs). Communitas should stay with ML-DSA/ML-KEM.

## Communitas Advantages Over SimpleX

| Capability | Communitas | SimpleX |
|------------|------------|---------|
| Latency | Direct P2P, sub-50ms typical | All via relay, higher latency |
| Identity | Human-readable four-words | No identity, harder to find contacts |
| Discovery | FOAF, presence in shared groups | Must share queue addresses OOB |
| Offline sync | CRDT anti-entropy, peer-held | Relay-held, time-limited |
| Collaboration | Virtual disks, Kanban, channels | Chat-focused only |

## Summary

| SimpleX Innovation | Communitas Status | Action |
|--------------------|-------------------|--------|
| 2-hop onion routing | MASQUE + peer cache covers this | None needed |
| No user identifiers | Four-word networking better for UX | Keep current |
| Unidirectional queues | Topic isolation provides similar benefit | None needed |
| Relay message storage | Prefer P2P purity with CRDT sync | None needed |
| XFTP chunking | Could enhance Sites for redundancy | Future optional |
| Super-peers | Consider when >200 member groups | Future optional |
| Post-quantum crypto | Already using ML-DSA/ML-KEM | None needed |

**Conclusion**: Communitas's architecture is well-designed for its purpose. SimpleX optimizes for maximum metadata privacy by sacrificing P2P directness. Communitas makes the opposite trade-off - true P2P with human-readable identity - which is better suited for a local-first collaboration platform.

## References

- [SimpleX GitHub](https://github.com/simplex-chat/simplex-chat)
- [SimpleX Platform Docs](https://simplex.chat/docs/simplex.html)
- [SimpleX Messaging Protocol](https://github.com/simplex-chat/simplexmq/blob/stable/protocol/simplex-messaging.md)
- [SimpleX Private Message Routing v5.8](https://simplex.chat/blog/20240604-simplex-chat-v5.8-private-message-routing-chat-themes.html)
- [SimpleX PQ Double Ratchet](https://simplex.chat/blog/20240314-simplex-chat-v5-6-quantum-resistance-signal-double-ratchet-algorithm.html)
- [SimpleX Large Groups 2025](https://simplex.chat/blog/20250114-simplex-network-large-groups-privacy-preserving-content-moderation.html)
- [SimpleX XFTP Protocol](https://simplex.chat/blog/20230301-simplex-file-transfer-protocol.html)

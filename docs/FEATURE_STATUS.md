# Feature Status

> What communitas actually ships vs what's designed but not yet implemented.
> Last updated: 2026-04-03

## Shipped (Production)

- **Messaging**: send, edit, delete, react, thread, quote, mention, search
- **Presence**: online/offline badges, typing indicators
- **Groups**: create, add members, manage roles, threshold-ready ML-DSA identities
- **Kanban**: CRDT-based boards, drag-drop cards, virtualized columns
- **File Drive**: virtual disks (Private/Public/Shared), chunked uploads, preview
- **Auth**: identity creation (ML-DSA-65), BIP-39 mnemonic recovery
- **Onboarding**: auto-install x0xd daemon on first run
- **Offline**: local-first architecture, syncs when reconnected via CRDTs (Yrs)
- **SWIM failure detection**: K-peer probing, indirect probes, suspect-to-dead
- **Anti-entropy reconciliation**: set-difference based partition recovery
- **Signed presence beacons**: ML-DSA signed, per-peer rate limiting

## Scaffolding (Components exist, not complete)

- **Calls/WebRTC**: device enumeration works, multimedia stack incomplete
- **Canvas**: canvas-core integrated, advanced collaboration features incomplete

## Designed (ADRs accepted, not yet implemented)

- ADR-022: MCP apps integration
- ADR-023: Capability tokens / unlock grants
- ADR-024: Policy kernel architecture
- ADR-025: Capability registry
- ADR-026: Principal hierarchy
- ADR-027: Saorsa Canvas client strategy

## Note on Dependencies

Communitas connects to x0x via HTTP REST + WebSocket (`communitas-x0x-client` crate).
It does **NOT** directly embed `saorsa-pqc`, `saorsa-gossip`, `saorsa-mls`, or `ant-quic`.
All P2P networking is delegated to the `x0xd` daemon.

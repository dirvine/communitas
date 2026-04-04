# ADR-028: x0x Daemon Networking Delegation

## Status
Accepted (2026-03-26)

## Context
Communitas originally implemented peer-to-peer networking directly via saorsa-gossip (ADR-007), with its own QUIC connection management (ADR-013), presence system (ADR-014), and bootstrap process (ADR-015). Managing a full P2P stack alongside application logic created complexity and slowed feature delivery.

The x0x project consolidates all P2P networking into a single daemon (`x0xd`) with a REST + WebSocket API, handling gossip, presence, groups, stores, messaging, file transfer, and trust management.

## Decision
Delegate ALL networking to a local `x0xd` daemon. Communitas communicates with x0x exclusively via:
- HTTP REST API at `127.0.0.1:<port>` (read from `api.port` file)
- WebSocket for real-time events
- Bearer token authentication (read from `api-token` file)

The `communitas-x0x-client` crate provides typed Rust bindings over the REST/WS surface.

## Consequences

### Benefits
- Simpler communitas codebase — no P2P protocol implementation
- Faster feature iteration — network layer maintained by x0x team
- Shared networking across all clients (Dioxus, Swift, CLI, MCP)
- Single daemon instance serves all local applications

### Trade-offs
- Hard dependency on x0xd daemon being running
- No pure peer-to-peer mode — always requires x0x daemon
- Network feature velocity tied to x0x release cycle

## Supersedes
- ADR-007: Gossip Overlay Networking (now handled by x0x)
- ADR-013: Connection System (now handled by x0x)
- ADR-014: Peer Discovery Presence System (now handled by x0x)
- ADR-015: Bootstrap Process (now handled by x0x)

## References
- [x0x Integration Contract](../x0x-integration-contract.md)
- [x0x Convergence Report](../x0x-convergence-recheck-report-20260326.md)

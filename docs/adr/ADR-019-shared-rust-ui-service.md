# ADR-019: Shared Rust UI Service Layer

## Status
Accepted (2026-01-18)

## Context
- Communitas now ships a pure-Rust Dioxus client plus a comprehensive MCP surface for AI agents (ADR-018). The prior multi-language thin-client code was archived on January 18, 2026 per ADR-020.  
- Even after the archive, Dioxus components and MCP tool handlers independently wrapped the same `CommunitasApp` commands/queries, duplicating conversion logic, navigation prefs, session state, and gossip lifecycle code.  
- This duplication slows feature delivery, makes it easy for UI stacks to drift from MCP semantics, and blocks the “AI can operate every screen” goal because some state (recents/starred, overrides, quick actions) only lived inside front-end scaffolding.

## Decision
Create a shared Rust service layer (`communitas-ui-service`, name TBD) that sits between `communitas-core` and every presentation technology (Dioxus native, future mobile shells built in Rust, MCP):

1. **Crate structure**
   - Lives alongside `communitas-ui-api`, re-exporting typed DTOs already shared by Dioxus and MCP.  
   - Provides async traits per domain (`AuthService`, `DirectoryService`, `MessagingService`, `KanbanService`, `FilesService`, `ContactsService`, `NavigationService`, `PresenceService`).  
   - Depends solely on stable `communitas-core` APIs and exposes sendable DTOs plus `tokio::sync::broadcast`/`watch` streams for live updates.
2. **State + persistence**
   - Moves UI-specific persisted state (recents, starred items, organization overrides, demo-mode flags) from ad-hoc Dioxus code into Rust, storing data under the Communitas application directory alongside vault metadata.  
   - Emits change events so UI shells and MCP stay in sync without duplicating persistence logic.
3. **Session + gossip orchestration**
   - Consolidates login/create/recover flows, gossip start/stop, delegate-token issuance, and network diagnostics so Dioxus components and MCP authentication code all call the same routines.  
   - Provides structured errors (`thiserror`) to satisfy workspace lint rules and keep UX messaging consistent.
4. **MCP alignment**
   - MCP tool handlers in `communitas-mcp` delegate to the same service traits, guaranteeing tool semantics match GUI behavior.  
   - Adds optional MCP “navigation state” tools/resources so AI agents can inspect or manipulate recents/starred lists, matching the new desktop UX requirements.
5. **Adoption plan**
   - Phase 1: stand up the crate with auth/navigation services, switch the Dioxus client first (nav shell + auth).  
   - Phase 2: migrate MCP tool handlers and any residual helper crates incrementally, retiring duplicate conversion code.  
   - Phase 3: extend services for remaining feature clusters (messaging, Kanban, drive, canvas, calls) and enforce parity tests that hit the shared service.

## Consequences
### Benefits
- **Parity & maintenance**: One code path feeds Dioxus and MCP, preventing feature drift and ensuring AI agents can execute the same flows users see in the GUI.  
- **Pure Rust UX**: Dioxus consumes services directly without intermediate bindings, keeping the promised all-Rust experience.  
- **Faster MCP evolution**: New UI capabilities (e.g., navigation state) are instantly available to MCP because the tools simply wrap the shared service.  
- **Testability**: Service traits run in isolation, enabling headless integration tests that cover session flows, gossip management, and data projections once rather than per client.

### Trade-offs
- **Up-front refactor**: Existing Dioxus components and MCP handlers must migrate in stages, temporarily increasing complexity while both pathways exist.  
- **API discipline**: The service layer becomes part of our public surface; breaking changes require coordination across all UIs and MCP clients.  
- **Binary size**: Dioxus bundles the new crate, slightly increasing binary size versus hand-rolled hooks (mitigated by deduping duplicated logic).

## Alternatives Considered
1. **Status quo (per-client wrappers)**  
   - Rejected: Continuous duplication, inconsistent UX/messaging, and no straightforward way to expose UI state to MCP.
2. **“MCP-only” UI access** (UIs talk to Communitas via MCP)  
   - Rejected: Adds IPC latency, sacrifices offline mode, and contradicts native UX requirements.  
3. **Code generation from core protobufs**  
   - Considered but rejected for now; still leaves navigation/session orchestration duplicated and does not address persistence/state concerns.

## References
- ADR-018: MCP External Integration Architecture  
- ADR-020: Dioxus Desktop Adoption  
- `docs/architecture/dioxus_desktop_prototype_plan.md`  
- `docs/MCP_THIN_GUI_ARCHITECTURE.md`

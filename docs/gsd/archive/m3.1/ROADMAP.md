# Communitas Dioxus Prototype Roadmap

_Last updated: 2026-01-19_

## Milestone Overview

| Milestone | Description | Status |
|-----------|-------------|--------|
| **M0** | Discovery & Inventory | ✅ Complete |
| **M1** | Bootstrap Node Enhancement | ✅ Complete |
| **M2** | Messaging & Entities | ✅ Complete |
| **M3** | Advanced Surfaces (UI only) | ⚠️ UI Complete, Backend Stubbed |
| **M4** | Polish & Performance | ⚠️ UI Complete, Backend Stubbed |
| **M3.1** | Remediation - Wire Services | ⏳ In Progress |
| **M5** | Stabilization | ☐ Not started |

---

## M3.1: Remediation - Wire Stubbed Services

Wiring mock UI services to real `CommunitasApp` backends.

### Phases

| Phase | Description | Status |
|-------|-------------|--------|
| **1** | MessagingService → CommunitasApp (CRUD + reactions) | ⏳ Planning |
| **2** | MCP parity harness fix + docs downgrade | ☐ Pending |
| **3** | KanbanService user binding + CRDT propagation | ☐ Pending |
| **4** | DriveService → storage APIs (uploads/downloads) | ☐ Pending |
| **5** | CanvasService → saorsa-canvas (CRDT sync) | ☐ Pending |
| **6** | CallService → saorsa-webrtc (device discovery) | ☐ Pending |

### Phase 1: MessagingService (Current)

**Goal**: Wire MessagingService to CommunitasApp with full CRUD support.

**Tasks**:
1. Identify CommunitasApp chat_manager query/command interface
2. Implement `list_threads()` using core queries
3. Implement `get_messages()` with pagination
4. Implement `send_message()` command
5. Implement `edit_message()` command
6. Implement `delete_message()` command
7. Implement `add_reaction()` / `remove_reaction()`
8. Add integration tests
9. Verify MCP tools use same paths

---

## Historical Milestones

### M1: Bootstrap Node Enhancement (Complete)

Enabled communitas-headless to serve as effective bootstrap nodes.

**Key deliverables:**
- Peer cache integration (ant-quic BootstrapCache)
- Gossip protocol peer sharing
- External address reflection
- Friend connection feature (4-word connection IDs)

### M2: Messaging & Entities (Complete)

Core messaging flows with MCP parity.

**Key deliverables:**
- MessagingService + PresenceService (wiring incomplete)
- Thread list, composer, message list components
- Entity & contact detail views
- MCP tools: list_threads, list_messages, list_contacts

### M3: Advanced Surfaces (UI Complete, Backend Stubbed)

Full collaboration surface UIs - backend wiring deferred.

**Key deliverables:**
- KanbanService + Dioxus Kanban UI (CRDT local only)
- DriveService + Dioxus Drive UI (mock data)
- CallService + Dioxus Call UI (mock devices)
- CanvasService + Dioxus Canvas UI (local scene)

### M4: Polish & Performance (UI Complete, Backend Stubbed)

Design system and accessibility.

**Key deliverables:**
- Accessibility remediation (P1 issues)
- Design token system (CSS variables)
- Token migration (core + advanced components)
- MCP tool surface area (tools exist but hit mocks)

---

## Reference

- Milestone tracker: `docs/architecture/dioxus_milestones.md`
- Archived plans: `docs/gsd/archive/PLAN-*.md`
- Main plan: `docs/architecture/dioxus_desktop_prototype_plan.md`

# GSD-Hybrid State

## Current Position

| Field | Value |
|-------|-------|
| **Project** | M3.1 Remediation - Wire Stubbed Services |
| **Milestone** | M3.1: Service Backend Wiring |
| **Phase** | 1.1 - MessagingService Foundation |
| **Status** | READY |
| **Last Updated** | 2026-01-19 |

---

## Phase 1 Structure

Phase 1 (Messaging) is broken into 4 sub-phases for context isolation:

| Plan | Sub-Phase | Tasks | Status |
|------|-----------|-------|--------|
| PLAN-23 | 1.1 Foundation | 3 | Ready |
| PLAN-24 | 1.2 Read Operations | 3 | Pending |
| PLAN-25 | 1.3 Write Operations | 3 | Pending |
| PLAN-26 | 1.4 Reactions + Events + Tests | 3 | Pending |

**Total Phase 1 Tasks**: 12

---

## Context

Code review identified that UI services are mock implementations despite M3/M4 being marked complete. This project wires those services to real `CommunitasApp` backends.

### Key Findings (from review)

| Service | Issue |
|---------|-------|
| **MessagingService** | Returns placeholder data or `Internal("not yet implemented")` |
| **DriveService** | Mock disks/directories, fabricated entries |
| **CallService** | Hardcoded device lists, fake participants |
| **CanvasService** | Local scene only, no persistence/CRDT/gossip |
| **KanbanService** | Uses "anonymous" user, no CRDT propagation |
| **MCP parity** | Compares fabricated datasets |
| **Docs** | M3/M4 marked complete but code is stubbed |

---

## Interview Decisions

| Question | Decision | Rationale |
|----------|----------|-----------|
| Priority order | Messaging first | Foundational - unlocks real data for testing |
| Scope | All 6 items (Full) | Complete remediation needed |
| Testing approach | Integration tests per service | Each service tests real CommunitasApp paths |
| Doc handling | Downgrade to partial | Honest status: "UI Complete, Backend Stubbed" |
| First deliverable | Messaging CRUD E2E | list/get/send/edit/delete/react all working |

---

## All Phases

| Phase | Description | Plans | Status |
|-------|-------------|-------|--------|
| **1** | MessagingService → CommunitasApp | 23-26 | Ready |
| **2** | MCP parity harness + docs update | TBD | Pending |
| **3** | KanbanService user binding | TBD | Pending |
| **4** | DriveService → storage APIs | TBD | Pending |
| **5** | CanvasService → saorsa-canvas | TBD | Pending |
| **6** | CallService → saorsa-webrtc | TBD | Pending |

---

## Prior Work Reference

Migrated from `.planning/` - 22 archived plans (M1-M4) preserved in `archive/`.

---

## Blockers

- None currently

---

## Handoff Context

Phase 1 is fully planned with XML task structure:
- **PLAN-23**: Add CommunitasApp + type conversions (3 tasks)
- **PLAN-24**: list_threads + get_messages (3 tasks)
- **PLAN-25**: send + edit + delete (3 tasks)
- **PLAN-26**: reactions + events + integration tests (3 tasks)

Execute with `/gsd:execute-phase 1` to begin implementation.

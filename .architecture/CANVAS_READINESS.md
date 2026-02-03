# Canvas Readiness Checklist

## Overview

Saorsa Canvas is a **future UI surface** that will provide chat/canvas-first interaction. This document identifies what infrastructure must exist **before** Canvas can be safely introduced as a first-class client.

**Key Principle**: Canvas is another client of the same capability layer, not a replacement for Dioxus.

---

## Current Canvas State

### What EXISTS Today

| Component | Location | Status |
|-----------|----------|--------|
| CanvasService | `communitas-ui-service/src/canvas.rs` | **EXISTS** |
| Canvas view models | `communitas-ui-api/src/canvas.rs` | **EXISTS** |
| Canvas components | `communitas-dioxus/src/components/canvas/` | **EXISTS** |
| Canvas MCP tools | `communitas-mcp/src/tools.rs` (canvas section) | **EXISTS** |
| CanvasSnapshot type | `communitas-ui-api` | **EXISTS** |
| CRDT operations | Via Yrs | **EXISTS** |

### What Canvas Does Today

The existing CanvasService supports:
- Add/update/delete elements (shapes, text, images)
- Transform elements (move, resize, rotate)
- Undo/redo history
- Zoom and pan
- Export/import
- Remote cursor display
- Offline queue with sync

**This is the drawing canvas within Communitas, NOT Saorsa Canvas.**

---

## Saorsa Canvas: What It Is

Saorsa Canvas is a **different product** that:
- Provides a chat/canvas hybrid interface
- Renders conversation as a spatial workspace
- Allows agents to propose actions visually
- Integrates with Communitas as a capability provider

**Canvas is NOT**:
- A replacement for the Dioxus app
- A fork of Communitas
- A separate backend

---

## Infrastructure Prerequisites

### P0: MUST HAVE (Before any Canvas work)

#### 1. Policy Kernel Implementation

| Requirement | Status | Why |
|-------------|--------|-----|
| `PolicyKernel.evaluate()` | **MISSING** | Canvas must use same gate as Dioxus |
| Principal types for Canvas | **MISSING** | Canvas needs its own principal type |
| Receipt generation | **MISSING** | All Canvas operations need audit trail |
| Rule evaluation | **MISSING** | Deterministic policy enforcement |

**DO NOT BUILD Canvas UI until Policy Kernel exists.**

#### 2. Capability Registry

| Requirement | Status | Why |
|-------------|--------|-----|
| Formal capability definitions | **MISSING** | Canvas needs to discover operations |
| JSON schemas for all capabilities | **PARTIAL** (MCP has schemas) | Validation and UI generation |
| Capability versioning | **MISSING** | Cross-version compatibility |

**DO NOT BUILD Canvas capability discovery until registry exists.**

#### 3. Approval Queue

| Requirement | Status | Why |
|-------------|--------|-----|
| Proposal submission | **MISSING** | Agents propose, users approve |
| Approval UI (Dioxus) | **MISSING** | Users need to see pending proposals |
| Proposal expiration | **MISSING** | Time-limited approvals |

**DO NOT BUILD Canvas agent features until approval queue exists.**

---

### P1: SHOULD HAVE (Before Canvas beta)

#### 4. Enhanced Audit Log

| Requirement | Status | Why |
|-------------|--------|-----|
| Signed receipts | **MISSING** | Tamper-proof audit trail |
| Receipt export | **PARTIAL** (AuditService exists) | Compliance requirements |
| Query by principal | **PARTIAL** | Debugging and investigation |

#### 5. Principal Trust Levels

| Requirement | Status | Why |
|-------------|--------|-----|
| TrustLevel enum | **MISSING** | Different principals, different defaults |
| Trust escalation | **MISSING** | User can promote agent trust |
| Trust revocation | **MISSING** | User can demote agent trust |

#### 6. Rate Limiting for Canvas

| Requirement | Status | Why |
|-------------|--------|-----|
| Per-principal rate limits | **EXISTS** (in MCP) | Prevent abuse |
| Adaptive limits | **MISSING** | Adjust based on behavior |
| Canvas-specific limits | **MISSING** | Canvas may have different patterns |

---

### P2: NICE TO HAVE (Before Canvas GA)

#### 7. Quarantine Layer

| Requirement | Status | Why |
|-------------|--------|-----|
| Sandboxed execution | **MISSING** | Untrusted agent code isolation |
| Resource limits per agent | **MISSING** | Memory/CPU/network caps |
| Capability firewall | **MISSING** | Block dangerous capabilities |

#### 8. Agent Collaboration Protocol

| Requirement | Status | Why |
|-------------|--------|-----|
| Multi-agent coordination | **MISSING** | Multiple agents working together |
| Conflict resolution | **EXISTS** (CRDT) | CRDT handles data conflicts |
| Turn-taking | **MISSING** | Prevent agent races |

---

## DO NOT BUILD YET

### Canvas Client Adapter

**Why not yet**: Policy Kernel must exist first.

```rust
// DO NOT BUILD until PolicyKernel exists

impl CanvasClient {
    fn invoke_capability(&self, ...) {
        // This needs PolicyKernel
    }
}
```

### Canvas-Specific Authentication

**Why not yet**: Canvas should use existing auth infrastructure.

```rust
// DO NOT BUILD - use existing auth

// WRONG: Canvas-specific auth
impl CanvasAuth {
    fn login(&self, ...) { ... }
}

// RIGHT: Canvas uses Principal::Canvas with existing AuthController
let principal = Principal::Canvas {
    identity: four_words,
    session_id: uuid,
};
```

### Canvas Native UI

**Why not yet**: Infrastructure prerequisites not met.

```
// DO NOT BUILD until:
// 1. Policy Kernel exists
// 2. Capability Registry exists
// 3. Approval Queue exists
// 4. Dioxus approval UI exists

// Canvas native UI can start when infrastructure is ready
```

### Canvas-to-Canvas Communication

**Why not yet**: Single-user Canvas first.

```rust
// DO NOT BUILD - multi-user Canvas later

// WRONG: Direct Canvas-to-Canvas
impl CanvasNetwork {
    fn sync_with_other_canvas(&self, ...) { ... }
}

// RIGHT: Canvas uses existing gossip infrastructure
// through UiServices → CommunitasApp → GossipContext
```

---

## Safe to Build Now

### Canvas Integration Tests

**Why safe**: Tests prepare for future without building features.

```rust
#[tokio::test]
async fn canvas_principal_would_be_allowed() {
    // Mock test for when PolicyKernel exists
    let principal = Principal::Canvas { ... };
    let capability = CapabilityId::from("messaging.send");

    // When PolicyKernel is implemented, Canvas should be treated like TrustedUi
    // assert!(matches!(decision, Decision::Allow(_)));
}
```

### Canvas Type Definitions

**Why safe**: Types are infrastructure, not features.

```rust
// communitas-core/src/policy/principal.rs

pub enum Principal {
    // ... existing variants ...

    /// Future: Saorsa Canvas client
    Canvas {
        identity: FourWords,
        session_id: Uuid,
    },
}
```

### Canvas Documentation

**Why safe**: Documentation prepares for future.

- Architecture documents (this file)
- Integration guides
- Security model documentation

---

## Readiness Checklist

### Before Canvas Alpha (Internal Testing)

- [ ] Policy Kernel implemented and tested
- [ ] Capability Registry with all current capabilities
- [ ] Principal::Canvas type defined
- [ ] Canvas treated same as TrustedUi in default rules
- [ ] Basic audit logging working

### Before Canvas Beta (Limited External)

- [ ] Approval Queue for agent actions
- [ ] Dioxus UI for viewing/approving proposals
- [ ] Rate limiting for Canvas clients
- [ ] Receipt verification working
- [ ] 90%+ test coverage on policy code

### Before Canvas GA (General Availability)

- [ ] Quarantine layer for untrusted agents
- [ ] Performance benchmarks met
- [ ] Security audit completed
- [ ] Documentation finalized
- [ ] Migration guide for Dioxus users

---

## Integration Architecture

### Target State

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLIENTS                                   │
│                                                                  │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────────┐ │
│  │  Dioxus  │   │   MCP    │   │ Headless │   │Saorsa Canvas │ │
│  │ Desktop  │   │ Agents   │   │  Daemon  │   │  (Future)    │ │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └──────┬───────┘ │
│       │              │              │                 │         │
│       │     Principal::TrustedUi    │    Principal::Canvas      │
│       │              │              │                 │         │
└───────┼──────────────┼──────────────┼─────────────────┼─────────┘
        │              │              │                 │
        ▼              ▼              ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    POLICY KERNEL                                 │
│                                                                  │
│    evaluate(principal, capability, context) → Decision           │
│                                                                  │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │  TrustedUi → Allow                                      │  │
│    │  Canvas    → Allow (same as TrustedUi)                  │  │
│    │  Agent     → Check scope, maybe RequireApproval         │  │
│    └─────────────────────────────────────────────────────────┘  │
│                                                                  │
│    Receipts signed with ML-DSA-87                               │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    UI SERVICES                                   │
│                                                                  │
│    Same for all clients: Auth, Messaging, Kanban, etc.          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CORE LAYER                                    │
│                                                                  │
│    CommunitasApp, CRDT, Gossip - unchanged                      │
└─────────────────────────────────────────────────────────────────┘
```

### Canvas Integration Path

1. **Phase 1**: Define `Principal::Canvas` type
2. **Phase 2**: Add Canvas rules to PolicyKernel (same as TrustedUi)
3. **Phase 3**: Canvas client implements capability invoker
4. **Phase 4**: Canvas connects to same UiServices instance
5. **Phase 5**: Canvas UI renders state from UiServices snapshots

---

## Risk Assessment

### High Risk (Must Mitigate)

| Risk | Impact | Mitigation |
|------|--------|------------|
| Canvas bypasses Policy Kernel | Authority without audit | Canvas MUST go through kernel |
| Canvas has different auth model | Security fragmentation | Canvas uses existing AuthController |
| Canvas modifies CRDT differently | Data corruption | Canvas uses same UiServices |

### Medium Risk (Should Mitigate)

| Risk | Impact | Mitigation |
|------|--------|------------|
| Canvas performance issues | User frustration | Benchmark early |
| Canvas-Dioxus feature divergence | Maintenance burden | Shared capability layer |
| Canvas-specific bugs | Quality issues | Shared test suite |

### Low Risk (Monitor)

| Risk | Impact | Mitigation |
|------|--------|------------|
| Canvas adoption low | Wasted effort | Feature flags, incremental rollout |
| Users confused by two UIs | Support burden | Clear documentation |

---

## Summary

**Canvas is not ready to build until**:

1. Policy Kernel exists
2. Capability Registry exists
3. Approval Queue exists

**Canvas will be**:

- Another client using UiServices
- Same trust level as Dioxus (TrustedUi)
- Subject to same audit requirements
- Using same CRDT sync infrastructure

**Canvas will NOT be**:

- A replacement for Dioxus
- A separate backend
- A special-cased principal
- Exempt from policy enforcement

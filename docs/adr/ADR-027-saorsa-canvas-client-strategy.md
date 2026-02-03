# ADR-027: Saorsa Canvas Client Strategy

## Status

Accepted

## Context

### Two Different "Canvas" Concepts

Communitas has two distinct canvas-related concepts that must not be confused:

| Concept | ADR | Status | Description |
|---------|-----|--------|-------------|
| **Internal Canvas** | ADR-021 | Implemented | saorsa-canvas library crates (canvas-core, canvas-renderer) integrated for whiteboard features within Communitas |
| **Saorsa Canvas Client** | This ADR | Proposed | Saorsa Canvas as a separate UI product that consumes Communitas capabilities |

This ADR addresses the **second concept**: Saorsa Canvas as a future alternative UI surface, similar to how Dioxus is the current primary UI.

### What is Saorsa Canvas (Product)?

Saorsa Canvas is a **chat/canvas hybrid interface** that:
- Provides a spatial workspace for conversations
- Renders data visually (org charts, message threads, kanban boards)
- Enables AI agents to propose and execute actions
- Integrates with Communitas as a capability provider

### Why Canvas as a Client?

1. **Alternative UX Paradigm**: Some users prefer visual/spatial interaction over traditional UI
2. **AI-Native Interface**: Canvas is designed for AI conversation from the start
3. **Progressive Enhancement**: Users can start with Dioxus, add Canvas later
4. **Separation of Concerns**: Canvas focuses on rendering, Communitas on capabilities

### Prerequisites

Before Canvas can be built, Communitas must have:

| Prerequisite | ADR | Status |
|--------------|-----|--------|
| Policy Kernel | ADR-024 | Proposed |
| Capability Registry | ADR-025 | Proposed |
| Principal Hierarchy | ADR-026 | Proposed |
| Principal::Canvas type | ADR-026 | Proposed |

See [.architecture/CANVAS_READINESS.md](../../.architecture/CANVAS_READINESS.md) for the full checklist.

## Decision

### Canvas as UiServices Consumer

Canvas will connect to Communitas as another client of the UiServices layer, just like Dioxus:

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLIENTS                                   │
│                                                                  │
│  ┌──────────────┐   ┌──────────────┐   ┌──────────────────────┐│
│  │    Dioxus    │   │     MCP      │   │   Saorsa Canvas      ││
│  │   Desktop    │   │   Agents     │   │   (Future Client)    ││
│  └──────┬───────┘   └──────┬───────┘   └──────────┬───────────┘│
│         │                  │                      │             │
│  Principal::TrustedUi      │ Principal::Agent     │ Principal::Canvas
│         │                  │                      │             │
└─────────┼──────────────────┼──────────────────────┼─────────────┘
          │                  │                      │
          ▼                  ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    POLICY KERNEL (ADR-024)                       │
│                                                                  │
│    evaluate(principal, capability, context) → Decision           │
│                                                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                       UI SERVICES (ADR-019)                      │
│                                                                  │
│    Auth │ Messaging │ Directory │ Kanban │ Canvas │ Drive       │
│                                                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      COMMUNITAS CORE                             │
└─────────────────────────────────────────────────────────────────┘
```

### Canvas Principal

Canvas uses the same trust level as TrustedUi:

```rust
// Principal for Saorsa Canvas client
Principal::Canvas {
    identity: FourWords,    // User's identity
    session_id: Uuid,       // Canvas session
}

// Policy treats Canvas same as TrustedUi
PolicyRule {
    id: "canvas-allow".into(),
    conditions: vec![Condition::PrincipalType(PrincipalType::Canvas)],
    effect: Effect::Allow,
    priority: 100,
}
```

### Connection Options

Canvas can connect to Communitas via:

| Option | Protocol | Use Case |
|--------|----------|----------|
| **Embedded** | Direct Rust API | Canvas bundles communitas-core |
| **MCP** | JSON-RPC (HTTP/HTTPS) | Canvas as separate process |
| **Hybrid** | Both | Canvas uses MCP, embeds core for offline |

Recommended: **MCP connection** for separation, with offline queue for resilience.

### Canvas Authentication

Canvas authenticates the same way as Dioxus:

1. User unlocks vault in Canvas UI
2. Canvas creates `Principal::Canvas` with identity
3. Canvas passes principal to UiServices via MCP
4. Policy Kernel evaluates (always Allow for Canvas)

Canvas does **not** use delegate tokens (those are for agents). Canvas has full trust.

### Capability Invocation

Canvas invokes capabilities through UiServices:

```rust
// Canvas client implementation
pub struct CanvasClient {
    services: Arc<UiServices>,
    principal: Principal,
}

impl CanvasClient {
    /// Handle user action in Canvas UI
    pub async fn handle_action(&self, action: CanvasAction) -> Result<()> {
        // 1. Translate Canvas gesture to capability
        let capability = action.to_capability();
        let args = action.to_args();

        // 2. Invoke via UiServices (Policy Kernel evaluates internally)
        let result = self.services.invoke(&self.principal, &capability, args).await?;

        // 3. Render result in Canvas
        self.render_result(result);

        Ok(())
    }
}
```

### State Synchronization

Canvas subscribes to UiServices state changes:

```rust
impl CanvasClient {
    pub async fn subscribe_to_state(&self) {
        // Subscribe to messaging updates
        let mut messaging_rx = self.services.messaging().subscribe();
        tokio::spawn(async move {
            while let Ok(snapshot) = messaging_rx.recv().await {
                self.render_messages(snapshot);
            }
        });

        // Subscribe to directory updates
        let mut directory_rx = self.services.directory().subscribe();
        tokio::spawn(async move {
            while let Ok(snapshot) = directory_rx.recv().await {
                self.render_entities(snapshot);
            }
        });

        // ... other subscriptions
    }
}
```

### Canvas-Specific Features

Canvas may have unique features that Dioxus doesn't:

| Feature | Description | Implementation |
|---------|-------------|----------------|
| Spatial Layout | Arrange data in 2D space | Canvas rendering |
| Voice Input | Speech-to-action | Canvas input fusion |
| AI Proposals | Agent actions shown visually | Canvas UI + Approval Queue |
| Collaborative Cursors | See other users' focus | Canvas presence |

These features use the same underlying capabilities but render differently.

## Consequences

### Benefits

1. **Same Backend**: Canvas uses same UiServices, ensuring parity
2. **Same Security**: Policy Kernel applies equally to Canvas
3. **Same Data**: CRDT sync works identically
4. **Flexible Deployment**: Canvas can be bundled or separate
5. **Progressive Adoption**: Users can try Canvas without abandoning Dioxus

### Trade-offs

1. **Development Cost**: Canvas is a significant new codebase
2. **Testing Surface**: Must test same features in two UIs
3. **User Confusion**: Two UIs may confuse some users

### Risks Mitigated

1. **Feature Drift**: Shared UiServices prevents drift
2. **Security Gaps**: Policy Kernel is single enforcement point
3. **Data Inconsistency**: Same CRDT layer for both

## Implementation Plan

### Phase 0: Prerequisites (Before Canvas)
1. Implement Policy Kernel (ADR-024)
2. Implement Capability Registry (ADR-025)
3. Define Principal::Canvas (ADR-026)
4. Build Dioxus approval UI

### Phase 1: Canvas Preview (Read-Only)
1. Canvas connects via MCP
2. Canvas can view data (read-only)
3. Canvas cannot modify anything
4. Validate UiServices integration

### Phase 2: Canvas Proposals
1. Canvas can propose actions
2. Actions require approval in Dioxus
3. Approved actions execute
4. Receipt audit trail

### Phase 3: Full Canvas Client
1. Canvas has full read/write
2. Same trust as Dioxus
3. Canvas-specific features enabled
4. Collaborative cursors working

## Alternatives Considered

1. **Canvas as Fork**: Fork Communitas for Canvas
   - Rejected: Massive duplication, maintenance nightmare

2. **Canvas-Only**: Replace Dioxus with Canvas
   - Rejected: Canvas is additive, not replacement

3. **Canvas via MCP Only**: No direct integration
   - Rejected: Loses offline capability, adds latency

4. **Canvas as Thin Wrapper**: Just render MCP responses
   - Rejected: Doesn't leverage UiServices architecture

## References

- [.architecture/CANVAS_READINESS.md](../../.architecture/CANVAS_READINESS.md) - Prerequisites checklist
- [.architecture/ARCHITECTURE.md](../../.architecture/ARCHITECTURE.md) - Target architecture
- [.architecture/ROLLOUT_PLAN.md](../../.architecture/ROLLOUT_PLAN.md) - Phased rollout
- [ADR-021](ADR-021-canvas-integration-strategy.md) - Internal canvas library (different)
- [ADR-024](ADR-024-policy-kernel-architecture.md) - Policy Kernel (prerequisite)
- [ADR-025](ADR-025-capability-registry.md) - Capability Registry (prerequisite)
- [ADR-026](ADR-026-principal-hierarchy.md) - Principal::Canvas type (prerequisite)

# Communitas Architecture: Future-Proof Design

## Executive Summary

This document describes the architectural layers that enable Communitas to:
1. **Ship the Dioxus app unchanged** - First-class desktop experience
2. **Layer in Canvas later** - As an alternative UI surface
3. **Enable agent collaboration** - Without granting authority
4. **Avoid future rewrites** - Structural steel before glass

**Principle**: Authority flows from capabilities, not conversations.

---

## 1. Current Architecture (EXISTS TODAY)

### 1.1 Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        UI LAYER                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │  Dioxus Desktop  │  │   MCP Server     │  │   Headless    │ │
│  │  (Tauri WebView) │  │  (AI Agents)     │  │   (Daemon)    │ │
│  └────────┬─────────┘  └────────┬─────────┘  └───────┬───────┘ │
└───────────┼─────────────────────┼────────────────────┼─────────┘
            │                     │                    │
            ▼                     ▼                    ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SERVICE LAYER (ADR-019)                       │
│                                                                  │
│    UiServices: Shared orchestration for all UI surfaces          │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │ AuthController │ MessagingService │ DirectoryService    │  │
│    │ KanbanService  │ CanvasService    │ DriveService        │  │
│    │ CallService    │ PresenceService  │ AuditService        │  │
│    └─────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      CORE LAYER                                  │
│                                                                  │
│    CommunitasApp: Execute/Query/Subscribe interface              │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │ CoreContext │ EntityService │ MessageService            │  │
│    │ CrdtManager │ GossipContext │ SecurityModule            │  │
│    └─────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                   INFRASTRUCTURE LAYER                           │
│                                                                  │
│    ┌────────────┐ ┌──────────────┐ ┌───────────┐ ┌───────────┐ │
│    │ saorsa-    │ │  saorsa-pqc  │ │    yrs    │ │ Platform  │ │
│    │ gossip-*   │ │  (ML-DSA/    │ │  (CRDT)   │ │ Keyring   │ │
│    │            │ │   ML-KEM)    │ │           │ │           │ │
│    └────────────┘ └──────────────┘ └───────────┘ └───────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Current Security Model

**Authentication Flow (MCP)**:
```
1. Client connects (stdio/HTTPS)
2. MCP checks: is tool pre-auth?
   ├─ Yes → execute immediately
   └─ No  → check auth_state
           ├─ Unauthenticated → reject
           └─ Authenticated → check unlock_lease
                              ├─ Expired → reject
                              └─ Valid → check scope
                                        ├─ Insufficient → reject
                                        └─ Sufficient → execute
```

**Key Properties (EXISTS)**:
- Pre-auth tools: 8 (authenticate, create_vault, etc.)
- Post-auth tools: 100+ (all CRDT operations)
- Unlock lease: 10-minute sliding window
- Scopes: 10 (ReadMessages, WriteFiles, ManageEntities, etc.)

### 1.3 What Works Well

| Aspect | Implementation | Status |
|--------|----------------|--------|
| UI Parity | UiServices consumed by both Dioxus and MCP | **STABLE** |
| CRDT Sync | Yrs documents with anti-entropy | **STABLE** |
| Offline-First | OfflineQueue + retry on reconnect | **STABLE** |
| PQC Identity | ML-DSA-87 (NIST Level 5) | **STABLE** |
| P2P Networking | Gossip overlay (HyParView+SWIM+Plumtree) | **STABLE** |

---

## 2. Future Architecture (REQUIRED FOR CANVAS/AGENTS)

### 2.1 Target Layer Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        UI LAYER                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────────┐ │
│  │  Dioxus  │  │   MCP    │  │ Headless │  │  Saorsa Canvas  │ │
│  │ Desktop  │  │ Agents   │  │  Daemon  │  │  (Future UI)    │ │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └───────┬─────────┘ │
└───────┼─────────────┼────────────┼─────────────────┼───────────┘
        │             │            │                 │
        ▼             ▼            ▼                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CAPABILITY LAYER (NEW)                        │
│                                                                  │
│    CapabilityRegistry: Formal operation definitions              │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │  Capability │ InputSchema │ OutputSchema │ RequiredRole │  │
│    └─────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    POLICY KERNEL (NEW)                           │
│                                                                  │
│    Deterministic gate for ALL privileged operations              │
│    ┌─────────────────────────────────────────────────────────┐  │
│    │  evaluate(principal, capability, context) → Decision    │  │
│    │  Decision: Allow | Deny(reason) | RequireApproval       │  │
│    │  Receipt: Signed audit trail of every decision          │  │
│    └─────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SERVICE LAYER (UNCHANGED)                     │
│    UiServices: Same as today, now gated by Policy Kernel         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    CORE + INFRASTRUCTURE (UNCHANGED)             │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 New Components Required

#### A. Capability Registry

**Purpose**: Formalize all operations as typed capabilities with schemas.

```rust
// FUTURE: communitas-core/src/capabilities/registry.rs

pub struct Capability {
    pub id: CapabilityId,           // e.g., "messaging.send"
    pub name: String,               // Human-readable
    pub input_schema: JsonSchema,   // Validated input
    pub output_schema: JsonSchema,  // Typed output
    pub required_role: Role,        // Minimum permission
    pub audit_level: AuditLevel,    // Logging requirement
    pub reversible: bool,           // Can be undone?
}

pub struct CapabilityRegistry {
    capabilities: HashMap<CapabilityId, Capability>,
}

impl CapabilityRegistry {
    pub fn get(&self, id: &CapabilityId) -> Option<&Capability>;
    pub fn validate_input(&self, id: &CapabilityId, input: &Value) -> Result<()>;
}
```

**Why Required**: Canvas and agents need to discover available operations without hardcoding.

#### B. Policy Kernel

**Purpose**: Centralized, deterministic gate for all privileged operations.

```rust
// FUTURE: communitas-core/src/policy/kernel.rs

pub struct PolicyKernel {
    rules: Vec<PolicyRule>,
    audit_log: AuditLog,
}

pub enum Decision {
    Allow(Receipt),
    Deny(DenyReason, Receipt),
    RequireApproval(ApprovalRequest, Receipt),
}

impl PolicyKernel {
    /// Evaluate whether a principal can invoke a capability in context
    pub fn evaluate(
        &self,
        principal: &Principal,       // Who is asking
        capability: &CapabilityId,   // What they want to do
        context: &EvaluationContext, // Current state
    ) -> Decision;
}
```

**Why Required**: Agents must go through the same gate as users. No backdoors.

#### C. Receipt System

**Purpose**: Tamper-proof audit trail of every privileged operation.

```rust
// FUTURE: communitas-core/src/policy/receipt.rs

pub struct Receipt {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub principal: Principal,
    pub capability: CapabilityId,
    pub input_hash: Blake3Hash,
    pub decision: DecisionSummary,
    pub signature: MlDsa87Signature, // Signed by policy kernel
}

impl Receipt {
    pub fn verify(&self, kernel_pubkey: &PublicKey) -> bool;
}
```

**Why Required**: Accountability for agent actions. Audit trails for compliance.

#### D. Proposal Queue

**Purpose**: Agent actions that require human approval before execution.

```rust
// FUTURE: communitas-core/src/policy/proposal.rs

pub struct Proposal {
    pub id: Uuid,
    pub capability: CapabilityId,
    pub input: Value,
    pub principal: Principal,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ProposalStatus,
}

pub enum ProposalStatus {
    Pending,
    Approved(Receipt),
    Rejected(String),
    Expired,
}
```

**Why Required**: Agents can suggest actions without executing them.

---

## 3. Client Separation

### 3.1 Dioxus Desktop (EXISTS)

**Role**: First-class native client for desktop users.

**Integration**:
```rust
// TODAY: Direct UiServices access
let services = use_context::<Arc<UiServices>>();
let result = services.messaging().send_message(entity_id, content).await;
```

**FUTURE (with Policy Kernel)**:
```rust
// FUTURE: Same API, kernel gating transparent
let services = use_context::<Arc<UiServices>>();
// UiServices internally calls PolicyKernel.evaluate()
// User always gets Allow (trusted principal)
let result = services.messaging().send_message(entity_id, content).await;
```

**Key Point**: Dioxus code does NOT change. Policy Kernel is transparent for trusted UI.

### 3.2 MCP Agents (EXISTS)

**Role**: AI agent interface via JSON-RPC tools.

**Integration**:
```rust
// TODAY: Auth + Scope checking
if !requires_auth(tool_name) || is_authenticated() {
    if !requires_unlock(tool_name) || is_unlocked() {
        if let Some(scope) = required_scope(tool_name) {
            if has_scope(scope) {
                execute_tool(tool_name, args).await
            }
        }
    }
}
```

**FUTURE (with Policy Kernel)**:
```rust
// FUTURE: Unified with Policy Kernel
let capability = CapabilityId::from_tool(tool_name);
let decision = kernel.evaluate(&agent_principal, &capability, &context);
match decision {
    Decision::Allow(receipt) => execute_tool(tool_name, args).await,
    Decision::Deny(reason, _) => Err(reason),
    Decision::RequireApproval(req, _) => queue_proposal(req),
}
```

### 3.3 Saorsa Canvas (FUTURE)

**Role**: Alternative UI surface with richer interaction model.

**Integration (FUTURE)**:
```rust
// FUTURE: Canvas as another UiServices consumer
impl CanvasClient {
    fn new(services: Arc<UiServices>) -> Self {
        Self { services }
    }

    async fn handle_user_action(&self, action: CanvasAction) -> Result<()> {
        // Canvas translates UI gestures to capabilities
        let capability = action.to_capability();
        // Goes through same Policy Kernel as Dioxus/MCP
        self.services.invoke(capability, action.args()).await
    }
}
```

**Key Point**: Canvas is NOT special. Same capabilities, same policy, same audit.

---

## 4. Principal Hierarchy

### 4.1 Principal Types

```rust
pub enum Principal {
    /// Human user authenticated via vault
    User {
        identity: FourWords,
        session_id: Uuid,
    },

    /// Dioxus desktop (trusted, owned by user)
    TrustedUi {
        identity: FourWords,
        device_id: DeviceId,
    },

    /// AI agent with delegated access
    Agent {
        identity: FourWords,      // Issuing user
        delegate_name: String,    // e.g., "claude-code"
        scopes: Vec<Scope>,       // Granted permissions
        token_id: Uuid,           // Delegate token
    },

    /// Network peer (limited trust)
    Peer {
        peer_id: PeerId,
        reputation: ReputationScore,
    },

    /// System (internal operations)
    System,
}
```

### 4.2 Trust Levels

| Principal | Trust Level | Default Policy |
|-----------|-------------|----------------|
| TrustedUi | Full | Allow all user-scoped operations |
| User | Full | Allow all user-scoped operations |
| Agent | Scoped | Allow within granted scopes |
| Peer | Limited | Allow read, require approval for write |
| System | Internal | Allow infrastructure operations |

---

## 5. Migration Path

### 5.1 Phase 0: Ship Dioxus (NOW)

**Changes**: None to Dioxus.
**New**: Harden infrastructure, add capability schemas.

### 5.2 Phase 1: Add Policy Kernel (NEXT)

**Changes**:
- Insert PolicyKernel between MCP and UiServices
- Dioxus calls bypass kernel (TrustedUi always Allow)

**Rollback**: Remove kernel, revert to direct calls.

### 5.3 Phase 2: Unify All Clients

**Changes**:
- Dioxus calls flow through kernel (still Allow)
- Receipts generated for all operations
- Audit log always populated

**Rollback**: Disable receipt generation.

### 5.4 Phase 3: Add Canvas

**Changes**:
- Canvas client implements CapabilityInvoker
- Canvas uses same UiServices, same kernel

**Rollback**: Disable Canvas feature flag.

---

## 6. Non-Negotiables Preserved

| Constraint | How Preserved |
|------------|---------------|
| Dioxus first-class | Policy Kernel transparent for TrustedUi |
| Canvas additive | Canvas is another client, not a replacement |
| No secrets in model | Agent principals never hold keys |
| All input hostile | Validation at capability layer |
| No authority by conversation | Policy Kernel evaluates principal, not content |
| Local-first | CRDT + offline queue unchanged |
| PQC intact | Same ML-DSA-87 identity model |

---

## 7. Files to Create

| Path | Purpose | Priority |
|------|---------|----------|
| `communitas-core/src/capabilities/mod.rs` | Capability types | P1 |
| `communitas-core/src/capabilities/registry.rs` | Registry impl | P1 |
| `communitas-core/src/policy/mod.rs` | Policy kernel | P1 |
| `communitas-core/src/policy/kernel.rs` | Kernel impl | P1 |
| `communitas-core/src/policy/receipt.rs` | Receipt types | P1 |
| `communitas-core/src/policy/rules.rs` | Rule definitions | P2 |
| `communitas-core/src/policy/proposal.rs` | Proposal queue | P2 |

---

## 8. Summary

**Today**: Dioxus and MCP both work through UiServices. Auth is ad-hoc.

**Tomorrow**: All clients work through Policy Kernel → UiServices. Auth is formal.

**Key Insight**: The kernel is a **gate**, not a **replacement**. Dioxus code stays the same. Canvas becomes possible. Agents become auditable.

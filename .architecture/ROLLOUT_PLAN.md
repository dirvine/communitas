# Communitas Rollout Plan

## Overview

This document defines incremental, non-breaking phases for introducing Canvas and Agent collaboration to Communitas. Each phase has explicit acceptance criteria and reversibility guarantees.

**Principle**: Every phase can be rolled back without data loss or breaking existing users.

---

## Rollout Phases Summary

| Phase | Focus | Dioxus Impact | Canvas State | Agent State |
|-------|-------|---------------|--------------|-------------|
| **Phase 0** | Ship Dioxus + Harden | None | Not started | MCP (current) |
| **Phase 1** | Canvas Preview | None | Read-only | MCP (current) |
| **Phase 2** | Agent Proposals | None | Read + Propose | MCP + Approvals |
| **Phase 3** | Full Collaboration | None | Full access | Quarantined agents |

---

## Phase 0: Ship Dioxus + Harden Infrastructure

### Duration
2-4 weeks

### Objective
Ship the Dioxus desktop application while adding infrastructure for future expansion.

### What Changes

| Component | Change | Risk |
|-----------|--------|------|
| Dioxus App | No code changes | None |
| MCP Server | No code changes | None |
| Core Library | Add capability type definitions | Low |
| Documentation | Add architecture documents | None |

### Deliverables

1. **Capability Type Definitions**
   ```rust
   // communitas-core/src/capabilities/mod.rs

   pub struct CapabilityId(pub String);

   pub struct Capability {
       pub id: CapabilityId,
       pub name: String,
       pub input_schema: serde_json::Value,
       pub output_schema: serde_json::Value,
       pub required_role: Role,
       pub audit_level: AuditLevel,
       pub reversible: bool,
       pub offline_capable: bool,
   }
   ```

2. **Principal Type Definitions**
   ```rust
   // communitas-core/src/policy/principal.rs

   pub enum Principal {
       User { identity: FourWords, session_id: Uuid },
       TrustedUi { identity: FourWords, device_id: DeviceId },
       Agent { identity: FourWords, delegate_name: String, scopes: Vec<Scope>, token_id: Uuid },
       Peer { peer_id: PeerId, reputation: ReputationScore },
       System,
       Canvas { identity: FourWords, session_id: Uuid }, // Future
   }
   ```

3. **Architecture Documentation**
   - `.architecture/ARCHITECTURE.md` ✓
   - `.architecture/CAPABILITIES.md` ✓
   - `.architecture/POLICY_KERNEL.md` ✓
   - `.architecture/CANVAS_READINESS.md` ✓
   - `.architecture/THREAT_MODEL.md` ✓
   - `.architecture/ROLLOUT_PLAN.md` ✓
   - `.architecture/MODULE_MAP.md` ✓

### Acceptance Criteria

- [ ] Dioxus app ships and works unchanged
- [ ] MCP server operates unchanged
- [ ] Capability types compile and have tests
- [ ] Principal types compile and have tests
- [ ] All architecture documents complete
- [ ] No regressions in existing functionality

### Reversibility

**Fully Reversible**: Type definitions can be removed without affecting runtime behavior.

### Feature Flags

```toml
# Cargo.toml
[features]
default = []
capability-types = []  # Enable capability infrastructure
```

---

## Phase 1: Canvas Preview (Read-Only)

### Duration
4-6 weeks

### Objective
Introduce Canvas as a read-only view of Communitas data. Canvas cannot modify anything.

### Prerequisites

- Phase 0 complete
- Canvas UI prototype ready
- Principal::Canvas type tested

### What Changes

| Component | Change | Risk |
|-----------|--------|------|
| Dioxus App | None | None |
| MCP Server | None | None |
| Core Library | Add PolicyKernel (stub) | Low |
| UiServices | Add read-only Canvas path | Low |
| Canvas | New client (read-only) | Medium |

### Deliverables

1. **PolicyKernel Stub**
   ```rust
   // communitas-core/src/policy/kernel.rs

   pub struct PolicyKernel {
       // Stub implementation for Phase 1
   }

   impl PolicyKernel {
       pub fn evaluate(
           &self,
           principal: &Principal,
           capability: &CapabilityId,
           _context: &EvaluationContext,
       ) -> Decision {
           match principal {
               Principal::TrustedUi { .. } => Decision::Allow(Receipt::stub()),
               Principal::Canvas { .. } => {
                   if capability.is_read_only() {
                       Decision::Allow(Receipt::stub())
                   } else {
                       Decision::Deny(DenyReason::ReadOnlyMode, Receipt::stub())
                   }
               }
               _ => Decision::Deny(DenyReason::NotImplemented, Receipt::stub()),
           }
       }
   }
   ```

2. **Canvas Client Adapter (Read-Only)**
   ```rust
   // communitas-canvas/src/client.rs

   pub struct CanvasClient {
       services: Arc<UiServices>,
       principal: Principal,
       kernel: Arc<PolicyKernel>,
   }

   impl CanvasClient {
       pub async fn list_entities(&self) -> Result<Vec<EntitySnapshot>> {
           let cap = CapabilityId::from("entity.list");
           match self.kernel.evaluate(&self.principal, &cap, &ctx) {
               Decision::Allow(_) => self.services.directory().list_entities().await,
               Decision::Deny(reason, _) => Err(Error::AccessDenied(reason)),
               _ => unreachable!(),
           }
       }

       // All write operations return ReadOnlyMode error
       pub async fn send_message(&self, _: Uuid, _: String) -> Result<()> {
           Err(Error::AccessDenied(DenyReason::ReadOnlyMode))
       }
   }
   ```

3. **Read-Only Capability List**
   ```rust
   impl CapabilityId {
       pub fn is_read_only(&self) -> bool {
           matches!(self.0.as_str(),
               "entity.list" | "entity.get" |
               "messaging.list" | "messaging.search" |
               "member.list" | "contact.list" |
               "kanban.list_boards" | "kanban.list_cards" |
               "drive.list" | "drive.preview" |
               "presence.get" | "network.status" |
               "settings.get_preferences" | "audit.list_events"
           )
       }
   }
   ```

### Acceptance Criteria

- [ ] Dioxus app unchanged
- [ ] Canvas can authenticate with Principal::Canvas
- [ ] Canvas can view all read-only data
- [ ] Canvas write attempts return clear errors
- [ ] PolicyKernel logs all decisions
- [ ] No data corruption possible from Canvas

### Reversibility

**Fully Reversible**:
- Canvas client can be disabled via feature flag
- PolicyKernel stub can be removed
- No data format changes

### Feature Flags

```toml
[features]
default = ["capability-types"]
canvas-preview = ["capability-types"]  # Enable Canvas read-only mode
```

### Rollback Procedure

1. Disable `canvas-preview` feature flag
2. Rebuild and deploy
3. Canvas connections rejected at auth layer

---

## Phase 2: Agent Proposals (Approval Required)

### Duration
6-8 weeks

### Objective
Enable agents to propose actions that require user approval before execution.

### Prerequisites

- Phase 1 complete
- PolicyKernel fully implemented
- Approval Queue implemented
- Dioxus approval UI implemented

### What Changes

| Component | Change | Risk |
|-----------|--------|------|
| Dioxus App | Add approval UI component | Low |
| MCP Server | Route through PolicyKernel | Medium |
| Core Library | Full PolicyKernel + Approval Queue | Medium |
| Canvas | Enable proposal submission | Medium |

### Deliverables

1. **Full PolicyKernel Implementation**
   ```rust
   impl PolicyKernel {
       pub fn evaluate(
           &self,
           principal: &Principal,
           capability: &CapabilityId,
           context: &EvaluationContext,
       ) -> Decision {
           // Load rules for this capability
           let rules = self.get_rules(capability);

           for rule in rules {
               if rule.matches(principal, context) {
                   return match rule.effect {
                       Effect::Allow => Decision::Allow(self.sign_receipt(principal, capability)),
                       Effect::Deny => Decision::Deny(rule.reason.clone(), self.sign_receipt(...)),
                       Effect::RequireApproval => {
                           let request = ApprovalRequest::new(principal, capability, context);
                           Decision::RequireApproval(request, self.sign_receipt(...))
                       }
                   };
               }
           }

           // Default deny
           Decision::Deny(DenyReason::NoMatchingRule, self.sign_receipt(...))
       }
   }
   ```

2. **Approval Queue**
   ```rust
   pub struct ApprovalQueue {
       pending: HashMap<Uuid, Proposal>,
       approved: Vec<ApprovedAction>,
       rejected: Vec<RejectedAction>,
   }

   impl ApprovalQueue {
       pub fn submit(&mut self, request: ApprovalRequest) -> Proposal {
           let proposal = Proposal {
               id: Uuid::new_v4(),
               capability: request.capability,
               input: request.input,
               principal: request.principal,
               created_at: Utc::now(),
               expires_at: Utc::now() + Duration::minutes(5),
               status: ProposalStatus::Pending,
           };
           self.pending.insert(proposal.id, proposal.clone());
           proposal
       }

       pub fn approve(&mut self, proposal_id: Uuid, approver: &Principal) -> Result<Receipt> {
           // Verify approver is the owning user
           // Execute the proposed action
           // Generate receipt
       }

       pub fn reject(&mut self, proposal_id: Uuid, reason: String) -> Result<()> {
           // Mark proposal as rejected
       }
   }
   ```

3. **Dioxus Approval UI**
   ```rust
   #[component]
   pub fn ApprovalQueue(cx: Scope) -> Element {
       let queue = use_context::<ApprovalQueueState>(cx);

       rsx! {
           div { class: "approval-queue",
               for proposal in queue.pending.iter() {
                   ProposalCard {
                       proposal: proposal.clone(),
                       on_approve: move |_| queue.approve(proposal.id),
                       on_reject: move |reason| queue.reject(proposal.id, reason),
                   }
               }
           }
       }
   }

   #[component]
   fn ProposalCard(cx: Scope, proposal: Proposal, on_approve: EventHandler<()>, on_reject: EventHandler<String>) -> Element {
       rsx! {
           div { class: "proposal-card",
               h3 { "{proposal.capability}" }
               p { class: "requester", "Requested by: {proposal.principal}" }
               pre { "{proposal.input}" }
               div { class: "actions",
                   button { onclick: on_approve, "Approve" }
                   button { onclick: move |_| on_reject.call("User rejected".into()), "Reject" }
               }
               p { class: "expires", "Expires: {proposal.expires_at}" }
           }
       }
   }
   ```

4. **Agent Proposal Flow**
   ```rust
   // MCP tool execution with approval
   async fn execute_tool(&self, tool: &str, args: Value) -> Result<Value> {
       let capability = CapabilityId::from_tool(tool);
       let decision = self.kernel.evaluate(&self.agent_principal, &capability, &ctx);

       match decision {
           Decision::Allow(receipt) => {
               let result = self.services.invoke(&capability, args).await?;
               Ok(json!({ "result": result, "receipt": receipt }))
           }
           Decision::Deny(reason, receipt) => {
               Err(McpError::AccessDenied { reason, receipt })
           }
           Decision::RequireApproval(request, receipt) => {
               let proposal = self.approval_queue.submit(request);
               Ok(json!({
                   "status": "pending_approval",
                   "proposal_id": proposal.id,
                   "expires_at": proposal.expires_at,
                   "receipt": receipt
               }))
           }
       }
   }
   ```

### Acceptance Criteria

- [ ] Dioxus app shows pending proposals
- [ ] User can approve/reject proposals
- [ ] Approved actions execute correctly
- [ ] Rejected actions do not execute
- [ ] Expired proposals auto-reject
- [ ] All decisions generate signed receipts
- [ ] Receipts are exportable for audit
- [ ] Canvas can submit proposals (not just read)

### Reversibility

**Partially Reversible**:
- Approval queue can be disabled (agents get Deny instead of RequireApproval)
- Receipts are append-only (cannot remove)
- No data format changes to CRDT documents

### Feature Flags

```toml
[features]
default = ["capability-types", "canvas-preview"]
agent-proposals = ["canvas-preview"]  # Enable proposal workflow
```

### Rollback Procedure

1. Disable `agent-proposals` feature flag
2. All pending proposals auto-expire
3. Agents receive Deny for all write operations
4. Canvas returns to read-only mode

---

## Phase 3: Full Agent Collaboration

### Duration
8-12 weeks

### Objective
Enable full agent collaboration with quarantine for untrusted agents.

### Prerequisites

- Phase 2 complete
- Quarantine layer implemented
- Trust escalation UI implemented
- Security audit completed

### What Changes

| Component | Change | Risk |
|-----------|--------|------|
| Dioxus App | Add trust management UI | Low |
| MCP Server | Support quarantined agents | Medium |
| Core Library | Add quarantine layer | High |
| Canvas | Full read/write access | Medium |
| Agents | Tier-based trust levels | Medium |

### Deliverables

1. **Trust Level System**
   ```rust
   pub enum TrustLevel {
       /// User's own UI (Dioxus, Canvas)
       Full,

       /// Agents with proven track record
       Established {
           successful_actions: u64,
           failed_actions: u64,
       },

       /// New agents, require approval for writes
       New,

       /// Untrusted agents, run in quarantine
       Quarantined,

       /// Blocked agents, no access
       Blocked,
   }

   impl TrustLevel {
       pub fn default_for(principal: &Principal) -> Self {
           match principal {
               Principal::TrustedUi { .. } => TrustLevel::Full,
               Principal::Canvas { .. } => TrustLevel::Full,
               Principal::Agent { .. } => TrustLevel::New,
               Principal::Peer { .. } => TrustLevel::Quarantined,
               Principal::System => TrustLevel::Full,
           }
       }
   }
   ```

2. **Quarantine Layer**
   ```rust
   pub struct QuarantineContext {
       resource_limits: ResourceLimits,
       capability_firewall: CapabilityFirewall,
       execution_sandbox: Sandbox,
   }

   impl QuarantineContext {
       pub async fn execute(
           &self,
           principal: &Principal,
           capability: &CapabilityId,
           args: Value,
       ) -> Result<Value> {
           // Check resource limits
           self.resource_limits.check(principal)?;

           // Check capability firewall
           self.capability_firewall.check(capability)?;

           // Execute in sandbox
           self.execution_sandbox.run(async {
               // Actual execution with timeout
           }).await
       }
   }

   pub struct ResourceLimits {
       max_memory_bytes: u64,
       max_cpu_seconds: u64,
       max_network_requests: u64,
       max_storage_bytes: u64,
   }

   pub struct CapabilityFirewall {
       blocked: HashSet<CapabilityId>,
       rate_limited: HashMap<CapabilityId, RateLimit>,
   }
   ```

3. **Trust Escalation UI**
   ```rust
   #[component]
   pub fn TrustManagement(cx: Scope) -> Element {
       let agents = use_context::<AgentRegistry>(cx);

       rsx! {
           div { class: "trust-management",
               h2 { "Agent Trust Levels" }
               for agent in agents.iter() {
                   AgentTrustCard {
                       agent: agent.clone(),
                       on_escalate: move |level| agents.set_trust(agent.id, level),
                       on_block: move |_| agents.block(agent.id),
                   }
               }
           }
       }
   }

   #[component]
   fn AgentTrustCard(cx: Scope, agent: AgentInfo, ...) -> Element {
       rsx! {
           div { class: "agent-card",
               h3 { "{agent.delegate_name}" }
               p { "Trust: {agent.trust_level:?}" }
               p { "Actions: {agent.successful_actions} / {agent.total_actions}" }
               div { class: "trust-controls",
                   button { onclick: move |_| on_escalate(TrustLevel::Established { ... }), "Promote" }
                   button { onclick: on_block, class: "danger", "Block" }
               }
           }
       }
   }
   ```

4. **Multi-Agent Coordination**
   ```rust
   pub struct AgentCoordinator {
       active_agents: HashMap<Uuid, AgentSession>,
       operation_locks: Mutex<HashMap<ResourceId, Uuid>>,
   }

   impl AgentCoordinator {
       pub async fn acquire_lock(&self, agent: Uuid, resource: ResourceId) -> Result<LockGuard> {
           // Prevent conflicting operations
       }

       pub async fn resolve_conflict(&self, agents: Vec<Uuid>, resource: ResourceId) -> Resolution {
           // Turn-taking or merge strategies
       }
   }
   ```

### Acceptance Criteria

- [ ] Quarantine layer isolates untrusted agents
- [ ] Resource limits enforced per agent
- [ ] Capability firewall blocks dangerous operations
- [ ] Trust escalation works via UI
- [ ] Blocked agents cannot access system
- [ ] Multi-agent conflicts resolved gracefully
- [ ] Performance within SLA targets
- [ ] Security audit passed

### Reversibility

**Partially Reversible**:
- Can downgrade to Phase 2 (approval-only mode)
- Cannot remove receipts (audit requirement)
- Trust levels can be reset to New

### Feature Flags

```toml
[features]
default = ["capability-types", "canvas-preview", "agent-proposals"]
full-collaboration = ["agent-proposals"]  # Enable quarantine and trust
```

### Rollback Procedure

1. Disable `full-collaboration` feature flag
2. All agents revert to Phase 2 behavior (proposal mode)
3. Quarantine layer bypassed (agents get Deny)
4. Trust levels preserved but not enforced

---

## Phase Comparison Matrix

| Aspect | Phase 0 | Phase 1 | Phase 2 | Phase 3 |
|--------|---------|---------|---------|---------|
| Dioxus changes | None | None | Approval UI | Trust UI |
| Canvas access | None | Read-only | Read + Propose | Full |
| Agent access | MCP (current) | MCP (current) | MCP + Approvals | MCP + Trust tiers |
| PolicyKernel | Types only | Stub | Full | Full + Quarantine |
| Receipts | None | Stub | Signed | Signed |
| Rollback risk | None | Low | Low | Medium |

---

## Risk Mitigation

### Phase 1 Risks

| Risk | Mitigation |
|------|------------|
| Canvas shows stale data | Same UiServices refresh cycle as Dioxus |
| Canvas auth confusion | Clear Principal::Canvas separation |
| Performance impact | Kernel is minimal overhead for read-only |

### Phase 2 Risks

| Risk | Mitigation |
|------|------------|
| Approval fatigue | Smart defaults, batching, clear previews |
| Proposal spam | Rate limiting per agent |
| Receipt storage growth | Configurable retention, compression |

### Phase 3 Risks

| Risk | Mitigation |
|------|------------|
| Quarantine escape | Defense in depth, OS-level isolation |
| Trust gaming | Reputation based on multiple signals |
| Multi-agent deadlock | Timeout-based lock release |

---

## Success Metrics

### Phase 0
- Dioxus app stable (no regressions)
- Type definitions compile (zero warnings)
- Documentation complete (7 documents)

### Phase 1
- Canvas read latency <200ms
- Zero write operations from Canvas
- PolicyKernel logging 100% of decisions

### Phase 2
- Approval response time <5 seconds (p95)
- Proposal expiration rate <10%
- Receipt verification success rate 100%

### Phase 3
- Quarantine breach attempts: 0
- Trust escalation rate: monitored
- Multi-agent conflict resolution success: >99%

---

## Timeline Summary

```
Phase 0: Ship Dioxus + Harden          [NOW → +4 weeks]
├── Capability type definitions
├── Principal type definitions
└── Architecture documentation

Phase 1: Canvas Preview                [+4 weeks → +10 weeks]
├── PolicyKernel stub
├── Canvas client (read-only)
└── Integration testing

Phase 2: Agent Proposals               [+10 weeks → +18 weeks]
├── Full PolicyKernel
├── Approval Queue
├── Dioxus approval UI
└── Receipt signing

Phase 3: Full Collaboration            [+18 weeks → +30 weeks]
├── Quarantine layer
├── Trust escalation
├── Multi-agent coordination
└── Security audit
```

---

## Appendix: Feature Flag Reference

```toml
# Cargo.toml
[features]
default = []

# Phase 0: Type definitions only
capability-types = []

# Phase 1: Canvas read-only
canvas-preview = ["capability-types"]

# Phase 2: Agent proposals with approval
agent-proposals = ["canvas-preview"]

# Phase 3: Full agent collaboration
full-collaboration = ["agent-proposals"]
```

Each feature flag enables everything from previous phases, ensuring forward compatibility and clean rollback paths.


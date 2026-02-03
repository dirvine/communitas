# Policy Kernel Design

## Overview

The Policy Kernel is a **deterministic gate** that sits below all UI surfaces (Dioxus, Canvas, MCP) and arbitrates every privileged operation. It ensures:

1. **Uniform enforcement** - Same rules for users, agents, and Canvas
2. **Auditability** - Every decision produces a signed receipt
3. **Testability** - Deterministic evaluation, no hidden state
4. **Reversibility** - Can disable without breaking clients

**Principle**: The kernel evaluates principals and capabilities, never content.

---

## 1. Kernel Interface

### 1.1 Core Evaluation Function

```rust
// communitas-core/src/policy/kernel.rs

pub struct PolicyKernel {
    rules: PolicyRuleSet,
    signing_key: MlDsa87PrivateKey,  // For receipts
    audit_log: Arc<AuditLog>,
    config: PolicyConfig,
}

impl PolicyKernel {
    /// Evaluate whether a principal can invoke a capability
    ///
    /// This is the ONLY entry point for privileged operations.
    /// All clients (Dioxus, Canvas, MCP) must call this.
    pub fn evaluate(
        &self,
        principal: &Principal,
        capability: &CapabilityId,
        context: &EvaluationContext,
    ) -> Decision {
        // 1. Check if capability exists
        let cap = match self.capabilities.get(capability) {
            Some(c) => c,
            None => return Decision::Deny(
                DenyReason::UnknownCapability,
                self.sign_receipt(principal, capability, "unknown"),
            ),
        };

        // 2. Check principal authentication
        if cap.required_role != Role::None && !principal.is_authenticated() {
            return Decision::Deny(
                DenyReason::NotAuthenticated,
                self.sign_receipt(principal, capability, "not_authenticated"),
            );
        }

        // 3. Evaluate rules in order
        for rule in &self.rules {
            if rule.matches(principal, capability, context) {
                match rule.effect {
                    Effect::Allow => {
                        return Decision::Allow(
                            self.sign_receipt(principal, capability, "allowed"),
                        );
                    }
                    Effect::Deny(reason) => {
                        return Decision::Deny(
                            reason,
                            self.sign_receipt(principal, capability, "denied"),
                        );
                    }
                    Effect::RequireApproval => {
                        return Decision::RequireApproval(
                            ApprovalRequest::new(principal, capability, context),
                            self.sign_receipt(principal, capability, "pending_approval"),
                        );
                    }
                }
            }
        }

        // 4. Default deny if no rule matched
        Decision::Deny(
            DenyReason::NoMatchingRule,
            self.sign_receipt(principal, capability, "default_deny"),
        )
    }
}
```

### 1.2 Decision Types

```rust
pub enum Decision {
    /// Operation is allowed, proceed immediately
    Allow(Receipt),

    /// Operation is denied, do not proceed
    Deny(DenyReason, Receipt),

    /// Operation requires human approval before proceeding
    RequireApproval(ApprovalRequest, Receipt),
}

pub enum DenyReason {
    NotAuthenticated,
    UnknownCapability,
    InsufficientRole,
    InsufficientScope,
    RateLimited,
    ResourceExhausted,
    PolicyViolation(String),
    NoMatchingRule,
}
```

---

## 2. Principal Types

### 2.1 Principal Enum

```rust
pub enum Principal {
    /// Human user authenticated via vault unlock
    User {
        identity: FourWords,
        session_id: Uuid,
        device_id: DeviceId,
    },

    /// Trusted UI (Dioxus desktop, native app)
    /// Always treated as the owning user
    TrustedUi {
        identity: FourWords,
        device_id: DeviceId,
        app_version: Version,
    },

    /// AI agent with delegated access
    Agent {
        issuing_user: FourWords,
        delegate_name: String,      // e.g., "claude-code"
        scopes: Vec<Scope>,
        token_id: Uuid,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    },

    /// Future: Saorsa Canvas client
    Canvas {
        identity: FourWords,
        session_id: Uuid,
    },

    /// Network peer (gossip protocol)
    Peer {
        peer_id: PeerId,
        reputation: Option<ReputationScore>,
    },

    /// System-internal operations (no user context)
    System {
        subsystem: String,  // e.g., "crdt_sync", "anti_entropy"
    },
}
```

### 2.2 Principal Trust Levels

```rust
impl Principal {
    pub fn trust_level(&self) -> TrustLevel {
        match self {
            Principal::TrustedUi { .. } => TrustLevel::Full,
            Principal::User { .. } => TrustLevel::Full,
            Principal::Canvas { .. } => TrustLevel::Full,  // Same as TrustedUi
            Principal::Agent { .. } => TrustLevel::Scoped,
            Principal::Peer { .. } => TrustLevel::Limited,
            Principal::System { .. } => TrustLevel::Internal,
        }
    }

    pub fn can_act_as_user(&self) -> bool {
        matches!(
            self,
            Principal::TrustedUi { .. } |
            Principal::User { .. } |
            Principal::Canvas { .. }
        )
    }
}
```

---

## 3. Policy Rules

### 3.1 Rule Structure

```rust
pub struct PolicyRule {
    pub id: String,
    pub description: String,
    pub priority: i32,           // Lower = evaluated first
    pub conditions: Vec<Condition>,
    pub effect: Effect,
}

pub enum Condition {
    /// Principal type matches
    PrincipalType(PrincipalTypePattern),

    /// Principal has specific identity
    PrincipalIdentity(FourWords),

    /// Capability matches pattern
    CapabilityPattern(String),  // e.g., "messaging.*"

    /// Principal has required scope (for agents)
    HasScope(Scope),

    /// Principal has required role in entity
    HasRole(EntityId, Role),

    /// Time-based condition
    TimeWindow { start: Time, end: Time },

    /// Rate limit not exceeded
    WithinRateLimit { key: String, limit: RateLimit },

    /// Custom predicate
    Custom(Box<dyn Fn(&Principal, &CapabilityId, &EvaluationContext) -> bool>),
}

pub enum Effect {
    Allow,
    Deny(DenyReason),
    RequireApproval,
}
```

### 3.2 Default Rule Set

```rust
pub fn default_rules() -> Vec<PolicyRule> {
    vec![
        // Rule 1: TrustedUi always allowed (Dioxus)
        PolicyRule {
            id: "trusted_ui_allow".into(),
            description: "Trusted UI clients have full access".into(),
            priority: 0,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::TrustedUi),
            ],
            effect: Effect::Allow,
        },

        // Rule 2: Canvas treated same as TrustedUi
        PolicyRule {
            id: "canvas_allow".into(),
            description: "Canvas clients have full access".into(),
            priority: 1,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::Canvas),
            ],
            effect: Effect::Allow,
        },

        // Rule 3: Agents need scope for messaging
        PolicyRule {
            id: "agent_messaging_scope".into(),
            description: "Agents need messaging scope".into(),
            priority: 10,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::Agent),
                Condition::CapabilityPattern("messaging.*".into()),
                Condition::HasScope(Scope::SendMessages),
            ],
            effect: Effect::Allow,
        },

        // Rule 4: Agents need explicit scope for writes
        PolicyRule {
            id: "agent_write_scope".into(),
            description: "Agents need write scope for file operations".into(),
            priority: 10,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::Agent),
                Condition::CapabilityPattern("drive.upload".into()),
                Condition::HasScope(Scope::WriteFiles),
            ],
            effect: Effect::Allow,
        },

        // Rule 5: Agents without scope get denied
        PolicyRule {
            id: "agent_default_deny".into(),
            description: "Agents without required scope are denied".into(),
            priority: 100,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::Agent),
            ],
            effect: Effect::Deny(DenyReason::InsufficientScope),
        },

        // Rule 6: Peers can only read (no writes)
        PolicyRule {
            id: "peer_read_only".into(),
            description: "Network peers can only read".into(),
            priority: 10,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::Peer),
                Condition::CapabilityPattern("*.list".into()),
            ],
            effect: Effect::Allow,
        },

        // Rule 7: Peer writes require approval
        PolicyRule {
            id: "peer_write_approval".into(),
            description: "Peer writes require human approval".into(),
            priority: 20,
            conditions: vec![
                Condition::PrincipalType(PrincipalTypePattern::Peer),
            ],
            effect: Effect::RequireApproval,
        },

        // Rule 8: Rate limiting for all principals
        PolicyRule {
            id: "rate_limit".into(),
            description: "Enforce rate limits".into(),
            priority: -10,  // Evaluated early
            conditions: vec![
                Condition::WithinRateLimit {
                    key: "global".into(),
                    limit: RateLimit::new(1000, Duration::from_secs(60)),
                },
            ],
            effect: Effect::Deny(DenyReason::RateLimited),
        },
    ]
}
```

---

## 4. Receipt System

### 4.1 Receipt Structure

```rust
pub struct Receipt {
    /// Unique receipt ID
    pub id: Uuid,

    /// When the decision was made
    pub timestamp: DateTime<Utc>,

    /// Who requested the operation
    pub principal: PrincipalSummary,

    /// What operation was requested
    pub capability: CapabilityId,

    /// Input parameters (hashed, not stored)
    pub input_hash: Blake3Hash,

    /// The decision made
    pub decision: DecisionSummary,

    /// Which rule triggered the decision
    pub matched_rule: Option<String>,

    /// ML-DSA-87 signature over all fields
    pub signature: Vec<u8>,
}

impl Receipt {
    /// Verify receipt was signed by the policy kernel
    pub fn verify(&self, kernel_pubkey: &MlDsa87PublicKey) -> bool {
        let message = self.canonical_bytes();
        kernel_pubkey.verify(&message, &self.signature).is_ok()
    }

    /// Get canonical bytes for signing/verification
    fn canonical_bytes(&self) -> Vec<u8> {
        // Deterministic serialization
        let mut hasher = Blake3::new();
        hasher.update(self.id.as_bytes());
        hasher.update(&self.timestamp.timestamp().to_le_bytes());
        hasher.update(&self.principal.to_bytes());
        hasher.update(self.capability.as_bytes());
        hasher.update(self.input_hash.as_bytes());
        hasher.update(&self.decision.to_bytes());
        hasher.finalize().as_bytes().to_vec()
    }
}
```

### 4.2 Receipt Storage

```rust
pub struct AuditLog {
    storage: Box<dyn ReceiptStorage>,
    retention_policy: RetentionPolicy,
}

impl AuditLog {
    /// Store a receipt (called by PolicyKernel)
    pub async fn store(&self, receipt: Receipt) -> Result<()>;

    /// Query receipts by principal
    pub async fn by_principal(
        &self,
        principal: &PrincipalSummary,
        range: TimeRange,
    ) -> Result<Vec<Receipt>>;

    /// Query receipts by capability
    pub async fn by_capability(
        &self,
        capability: &CapabilityId,
        range: TimeRange,
    ) -> Result<Vec<Receipt>>;

    /// Export receipts for compliance
    pub async fn export(&self, range: TimeRange) -> Result<Vec<Receipt>>;
}
```

---

## 5. Approval System

### 5.1 Approval Request

```rust
pub struct ApprovalRequest {
    pub id: Uuid,
    pub principal: Principal,
    pub capability: CapabilityId,
    pub input: Value,  // Stored for execution on approval
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ApprovalStatus,
}

pub enum ApprovalStatus {
    Pending,
    Approved { by: FourWords, at: DateTime<Utc> },
    Rejected { by: FourWords, reason: String, at: DateTime<Utc> },
    Expired,
    Cancelled,
}
```

### 5.2 Approval Workflow

```rust
pub struct ApprovalQueue {
    pending: Vec<ApprovalRequest>,
    approvers: Vec<FourWords>,  // Users who can approve
}

impl ApprovalQueue {
    /// Submit a request for approval
    pub fn submit(&mut self, request: ApprovalRequest);

    /// List pending requests for an approver
    pub fn pending_for(&self, approver: &FourWords) -> Vec<&ApprovalRequest>;

    /// Approve a request (executes the operation)
    pub async fn approve(
        &mut self,
        request_id: Uuid,
        approver: &FourWords,
        kernel: &PolicyKernel,
    ) -> Result<Receipt>;

    /// Reject a request
    pub fn reject(
        &mut self,
        request_id: Uuid,
        approver: &FourWords,
        reason: String,
    ) -> Result<()>;
}
```

---

## 6. Integration with Existing Code

### 6.1 UiServices Integration

```rust
// communitas-ui-service/src/lib.rs

impl UiServices {
    /// Get the policy kernel (shared across all services)
    pub fn kernel(&self) -> &PolicyKernel {
        &self.policy_kernel
    }

    /// Invoke a capability through the policy kernel
    pub async fn invoke(
        &self,
        capability: CapabilityId,
        input: Value,
    ) -> Result<Value, PolicyError> {
        // 1. Get current principal (from auth state)
        let principal = self.current_principal();

        // 2. Build evaluation context
        let context = EvaluationContext {
            timestamp: Utc::now(),
            entity_context: self.current_entity_context(),
            rate_limit_state: self.rate_limiter.state(&principal),
        };

        // 3. Evaluate policy
        let decision = self.kernel().evaluate(&principal, &capability, &context);

        // 4. Act on decision
        match decision {
            Decision::Allow(receipt) => {
                // Execute the operation
                let result = self.execute_capability(&capability, input).await?;
                Ok(result)
            }
            Decision::Deny(reason, receipt) => {
                Err(PolicyError::Denied(reason, receipt))
            }
            Decision::RequireApproval(request, receipt) => {
                // Queue for approval
                self.approval_queue.submit(request);
                Err(PolicyError::PendingApproval(receipt))
            }
        }
    }
}
```

### 6.2 MCP Integration

```rust
// communitas-mcp/src/server.rs

impl McpServer {
    async fn handle_tool_call(
        &self,
        tool_name: &str,
        args: Value,
    ) -> ToolCallResult {
        // Convert tool name to capability
        let capability = CapabilityId::from_tool(tool_name);

        // Invoke through UiServices (which uses PolicyKernel)
        match self.services.invoke(capability, args).await {
            Ok(result) => ToolCallResult::success(result),
            Err(PolicyError::Denied(reason, _)) => {
                ToolCallResult::error(format!("Denied: {:?}", reason))
            }
            Err(PolicyError::PendingApproval(_)) => {
                ToolCallResult::success(json!({
                    "status": "pending_approval",
                    "message": "This operation requires human approval"
                }))
            }
            Err(e) => ToolCallResult::error(e.to_string()),
        }
    }
}
```

### 6.3 Dioxus Integration (Transparent)

```rust
// communitas-dioxus/src/hooks/use_messaging.rs

// NO CHANGES REQUIRED
// Dioxus components continue to call UiServices directly
// PolicyKernel is transparent for TrustedUi principals

pub fn use_send_message() -> impl Fn(EntityId, String) {
    let services = use_context::<Arc<UiServices>>();

    move |entity_id, content| {
        spawn(async move {
            // This internally goes through PolicyKernel
            // TrustedUi always gets Allow, so user sees no difference
            services.messaging().send_message(entity_id, content).await
        });
    }
}
```

---

## 7. Policy Tiers

### Tier 0: Pre-Auth (No Principal Required)

| Capability | Effect |
|------------|--------|
| `auth.login` | Allow |
| `auth.create_vault` | Allow |
| `auth.recover_vault` | Allow |
| `auth.list_vaults` | Allow |

### Tier 1: Full Trust (TrustedUi, User, Canvas)

| Capability | Effect |
|------------|--------|
| All capabilities | Allow |

### Tier 2: Scoped Trust (Agents)

| Capability | Effect |
|------------|--------|
| Matches granted scope | Allow |
| Outside granted scope | Deny |
| Destructive operations | RequireApproval |

### Tier 3: Limited Trust (Peers)

| Capability | Effect |
|------------|--------|
| Read operations | Allow |
| Write operations | RequireApproval |
| Admin operations | Deny |

---

## 8. File Locations

| File | Purpose |
|------|---------|
| `communitas-core/src/policy/mod.rs` | Module exports |
| `communitas-core/src/policy/kernel.rs` | PolicyKernel implementation |
| `communitas-core/src/policy/principal.rs` | Principal types |
| `communitas-core/src/policy/rules.rs` | Rule definitions |
| `communitas-core/src/policy/receipt.rs` | Receipt types |
| `communitas-core/src/policy/approval.rs` | Approval queue |
| `communitas-core/src/policy/audit.rs` | Audit log |

---

## 9. Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trusted_ui_always_allowed() {
        let kernel = PolicyKernel::with_default_rules();
        let principal = Principal::TrustedUi { ... };
        let capability = CapabilityId::from("messaging.send");

        let decision = kernel.evaluate(&principal, &capability, &context);

        assert!(matches!(decision, Decision::Allow(_)));
    }

    #[test]
    fn agent_without_scope_denied() {
        let kernel = PolicyKernel::with_default_rules();
        let principal = Principal::Agent {
            scopes: vec![Scope::ReadMessages],  // No SendMessages
            ..
        };
        let capability = CapabilityId::from("messaging.send");

        let decision = kernel.evaluate(&principal, &capability, &context);

        assert!(matches!(decision, Decision::Deny(DenyReason::InsufficientScope, _)));
    }

    #[test]
    fn receipts_are_verifiable() {
        let kernel = PolicyKernel::with_default_rules();
        let decision = kernel.evaluate(&principal, &capability, &context);

        let receipt = match decision {
            Decision::Allow(r) | Decision::Deny(_, r) => r,
            _ => panic!("Expected receipt"),
        };

        assert!(receipt.verify(&kernel.public_key()));
    }
}
```

---

## 10. Summary

The Policy Kernel:

1. **Sits below all clients** - Dioxus, Canvas, MCP all go through it
2. **Evaluates principals, not content** - No prompt injection attacks
3. **Produces signed receipts** - Every decision is auditable
4. **Supports approval workflows** - Agents can propose, humans approve
5. **Is transparent for trusted UI** - Dioxus code doesn't change
6. **Is deterministic and testable** - Same inputs → same outputs

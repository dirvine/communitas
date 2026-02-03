# ADR-024: Policy Kernel Architecture

## Status

Accepted

## Context

Communitas currently has multiple authorization checkpoints:
- MCP server: auth state, unlock leases, scope validation
- UiServices: implicit trust for Dioxus
- Individual tools: ad-hoc permission checks

This fragmented approach creates several problems:

1. **No Single Enforcement Point**: Authorization logic is scattered across layers
2. **No Audit Trail**: Decisions aren't systematically logged with cryptographic proof
3. **Agent Limitations**: Agents can't propose actions for user approval
4. **Canvas Blocked**: A future Saorsa Canvas client has no defined authorization path

For Communitas to safely support Canvas and advanced agent collaboration, we need a centralized, deterministic authorization gate that all clients pass through.

## Decision

Implement a **Policy Kernel** as the single enforcement point for all privileged operations.

### Core Interface

```rust
// communitas-core/src/policy/kernel.rs

pub struct PolicyKernel {
    rules: Vec<PolicyRule>,
    signing_key: MlDsa87PrivateKey,
    audit_log: AuditLog,
}

pub enum Decision {
    /// Operation allowed, receipt attached
    Allow(Receipt),

    /// Operation denied with reason, receipt attached
    Deny(DenyReason, Receipt),

    /// Operation requires user approval, proposal created
    RequireApproval(ApprovalRequest, Receipt),
}

impl PolicyKernel {
    /// Evaluate whether a principal can invoke a capability
    pub fn evaluate(
        &self,
        principal: &Principal,
        capability: &CapabilityId,
        context: &EvaluationContext,
    ) -> Decision {
        // 1. Find matching rules
        let rules = self.get_rules_for(capability);

        // 2. Evaluate rules in order
        for rule in rules {
            if rule.matches(principal, context) {
                let receipt = self.sign_receipt(principal, capability, &rule.effect);
                return match rule.effect {
                    Effect::Allow => Decision::Allow(receipt),
                    Effect::Deny(reason) => Decision::Deny(reason, receipt),
                    Effect::RequireApproval => {
                        let request = ApprovalRequest::new(principal, capability, context);
                        Decision::RequireApproval(request, receipt)
                    }
                };
            }
        }

        // 3. Default deny
        let receipt = self.sign_receipt(principal, capability, &Effect::Deny(DenyReason::NoMatchingRule));
        Decision::Deny(DenyReason::NoMatchingRule, receipt)
    }
}
```

### Policy Rules

```rust
pub struct PolicyRule {
    pub id: String,
    pub description: String,
    pub conditions: Vec<Condition>,
    pub effect: Effect,
    pub priority: i32,
}

pub enum Condition {
    /// Principal must be of this type
    PrincipalType(PrincipalType),

    /// Principal must have this trust level
    TrustLevel(TrustLevel),

    /// Principal must have this scope
    HasScope(Scope),

    /// Capability must match pattern
    CapabilityPattern(String),

    /// Context-specific conditions
    ContextMatch(ContextCondition),
}

pub enum Effect {
    Allow,
    Deny(DenyReason),
    RequireApproval,
}
```

### Default Rules

```rust
// Built-in rules (can be extended by user)
let default_rules = vec![
    // TrustedUi (Dioxus) always allowed
    PolicyRule {
        id: "trusted-ui-allow".into(),
        conditions: vec![Condition::PrincipalType(PrincipalType::TrustedUi)],
        effect: Effect::Allow,
        priority: 100,
    },

    // Canvas treated same as TrustedUi
    PolicyRule {
        id: "canvas-allow".into(),
        conditions: vec![Condition::PrincipalType(PrincipalType::Canvas)],
        effect: Effect::Allow,
        priority: 100,
    },

    // Agents with scope allowed for matching capabilities
    PolicyRule {
        id: "agent-scoped-allow".into(),
        conditions: vec![
            Condition::PrincipalType(PrincipalType::Agent),
            Condition::HasScope(Scope::Dynamic), // Scope must match capability
        ],
        effect: Effect::Allow,
        priority: 50,
    },

    // Agents without scope require approval
    PolicyRule {
        id: "agent-require-approval".into(),
        conditions: vec![Condition::PrincipalType(PrincipalType::Agent)],
        effect: Effect::RequireApproval,
        priority: 10,
    },

    // Default deny
    PolicyRule {
        id: "default-deny".into(),
        conditions: vec![],
        effect: Effect::Deny(DenyReason::NoMatchingRule),
        priority: 0,
    },
];
```

### Receipt System

Every decision generates a signed receipt for audit:

```rust
pub struct Receipt {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub principal: PrincipalSummary,
    pub capability: CapabilityId,
    pub input_hash: Blake3Hash,
    pub decision: DecisionSummary,
    pub rule_id: String,
    pub signature: MlDsa87Signature,
}

impl Receipt {
    /// Verify receipt signature
    pub fn verify(&self, kernel_pubkey: &MlDsa87PublicKey) -> bool {
        let message = self.canonical_bytes();
        kernel_pubkey.verify(&message, &self.signature).is_ok()
    }
}
```

### Integration with UiServices

UiServices will call the Policy Kernel for all operations:

```rust
impl MessagingService {
    pub async fn send_message(
        &self,
        principal: &Principal,
        entity_id: Uuid,
        content: String,
    ) -> Result<MessageSnapshot, ServiceError> {
        // 1. Evaluate policy
        let capability = CapabilityId::from("messaging.send");
        let context = EvaluationContext::new()
            .with_entity_id(entity_id)
            .with_content_hash(hash(&content));

        match self.kernel.evaluate(principal, &capability, &context) {
            Decision::Allow(receipt) => {
                // 2. Execute operation
                let result = self.core.send_message(entity_id, content).await?;

                // 3. Log receipt
                self.audit.log_receipt(receipt);

                Ok(result)
            }
            Decision::Deny(reason, receipt) => {
                self.audit.log_receipt(receipt);
                Err(ServiceError::AccessDenied(reason))
            }
            Decision::RequireApproval(request, receipt) => {
                self.audit.log_receipt(receipt);
                self.approval_queue.submit(request);
                Err(ServiceError::ApprovalRequired(request.id))
            }
        }
    }
}
```

### MCP Integration

MCP tools delegate to UiServices, which handles policy:

```rust
// MCP tool handler
async fn handle_send_message(&self, args: Value) -> McpResult {
    let principal = self.get_agent_principal();

    match self.services.messaging().send_message(&principal, entity_id, content).await {
        Ok(snapshot) => McpResult::success(snapshot),
        Err(ServiceError::AccessDenied(reason)) => McpResult::error(reason),
        Err(ServiceError::ApprovalRequired(proposal_id)) => {
            McpResult::pending_approval(proposal_id)
        }
        Err(e) => McpResult::error(e),
    }
}
```

## Consequences

### Benefits

1. **Single Enforcement Point**: All authorization flows through one gate
2. **Audit Trail**: Signed receipts provide tamper-proof history
3. **Agent Collaboration**: Agents can propose actions for approval
4. **Canvas Ready**: Clear authorization path for future Canvas client
5. **Extensible Rules**: Users can add custom rules for their needs
6. **Testable**: Policy logic is isolated and deterministic

### Trade-offs

1. **Added Latency**: Every operation goes through kernel (mitigated: kernel is fast)
2. **Complexity**: New subsystem to maintain
3. **Migration**: Existing code must be updated to pass principal

### Risks Mitigated

1. **Rule Misconfiguration**: Default-deny ensures safety
2. **Receipt Forgery**: ML-DSA-87 signatures prevent tampering
3. **Bypass Attacks**: All paths converge at kernel

## Implementation Plan

### Phase 1: Core Types (Week 1-2)
- Define `PolicyKernel`, `PolicyRule`, `Decision` types
- Define `Receipt` with signing
- Add to `communitas-core/src/policy/`

### Phase 2: Basic Rules (Week 3-4)
- Implement rule evaluation
- Add default rules
- Wire to UiServices (TrustedUi only)

### Phase 3: Agent Integration (Week 5-6)
- Wire MCP to pass principal through UiServices
- Implement scope checking
- Add approval queue stub

### Phase 4: Approval Queue (Week 7-8)
- Implement full approval queue
- Add Dioxus approval UI
- Enable agent proposals

## Alternatives Considered

1. **Extend Current Auth Layer**: Add features to existing auth state
   - Rejected: Would further fragment authorization logic

2. **RBAC System**: Traditional role-based access control
   - Rejected: Doesn't support approval workflows or receipts

3. **External Policy Engine (OPA)**: Use Open Policy Agent
   - Rejected: Adds deployment complexity, latency

4. **Per-Service Authorization**: Each service handles its own auth
   - Rejected: Current state, causes fragmentation

## References

- [.architecture/POLICY_KERNEL.md](../../.architecture/POLICY_KERNEL.md) - Detailed design
- [.architecture/CAPABILITIES.md](../../.architecture/CAPABILITIES.md) - Capability definitions
- [.architecture/THREAT_MODEL.md](../../.architecture/THREAT_MODEL.md) - Security analysis
- [ADR-023](ADR-023-unlock-grants-capability-tokens.md) - Current token system (foundation)
- [ADR-025](ADR-025-capability-registry.md) - Capability registry (companion)
- [ADR-026](ADR-026-principal-hierarchy.md) - Principal types (companion)

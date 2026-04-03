# ADR-026: Principal Hierarchy

## Status

Accepted — Not Yet Implemented

## Context

Communitas currently identifies callers implicitly:
- Dioxus: trusted, no identity passed
- MCP: auth state with optional delegate token
- Headless: system operations

For the Policy Kernel (ADR-024) to make authorization decisions, we need explicit principal types with defined trust levels. This enables:
1. Different default policies per principal type
2. Trust escalation for agents
3. Clear Canvas integration path
4. Audit trails with principal attribution

## Decision

Define a **Principal** enum that explicitly identifies all callers, with associated **TrustLevel** for policy decisions.

### Principal Types

```rust
// communitas-core/src/policy/principal.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Principal {
    /// Human user authenticated via vault
    User {
        identity: FourWords,
        session_id: Uuid,
    },

    /// Dioxus desktop or mobile app (trusted, owned by user)
    TrustedUi {
        identity: FourWords,
        device_id: DeviceId,
    },

    /// Saorsa Canvas client (trusted, same as Dioxus)
    Canvas {
        identity: FourWords,
        session_id: Uuid,
    },

    /// AI agent with delegated access
    Agent {
        /// Identity of the user who granted access
        identity: FourWords,

        /// Name of the agent (e.g., "claude-code", "custom-bot")
        delegate_name: String,

        /// Granted permission scopes
        scopes: Vec<Scope>,

        /// Unique token identifier
        token_id: Uuid,

        /// Trust level (can be escalated)
        trust_level: TrustLevel,
    },

    /// Network peer (limited trust)
    Peer {
        peer_id: PeerId,
        reputation: ReputationScore,
    },

    /// Internal system operations
    System,
}

impl Principal {
    /// Get the identity associated with this principal
    pub fn identity(&self) -> Option<&FourWords> {
        match self {
            Principal::User { identity, .. } => Some(identity),
            Principal::TrustedUi { identity, .. } => Some(identity),
            Principal::Canvas { identity, .. } => Some(identity),
            Principal::Agent { identity, .. } => Some(identity),
            Principal::Peer { .. } => None,
            Principal::System => None,
        }
    }

    /// Get the principal type for policy matching
    pub fn principal_type(&self) -> PrincipalType {
        match self {
            Principal::User { .. } => PrincipalType::User,
            Principal::TrustedUi { .. } => PrincipalType::TrustedUi,
            Principal::Canvas { .. } => PrincipalType::Canvas,
            Principal::Agent { .. } => PrincipalType::Agent,
            Principal::Peer { .. } => PrincipalType::Peer,
            Principal::System => PrincipalType::System,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrincipalType {
    User,
    TrustedUi,
    Canvas,
    Agent,
    Peer,
    System,
}
```

### Trust Levels

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// Fully trusted (user's own UI)
    Full,

    /// Established trust (proven track record)
    Established {
        successful_actions: u64,
        failed_actions: u64,
    },

    /// New/unknown (requires approval for sensitive operations)
    New,

    /// Quarantined (sandboxed execution, strict limits)
    Quarantined,

    /// Blocked (no access)
    Blocked,
}

impl TrustLevel {
    /// Get default trust level for a principal type
    pub fn default_for(principal_type: PrincipalType) -> Self {
        match principal_type {
            PrincipalType::User => TrustLevel::Full,
            PrincipalType::TrustedUi => TrustLevel::Full,
            PrincipalType::Canvas => TrustLevel::Full,
            PrincipalType::Agent => TrustLevel::New,
            PrincipalType::Peer => TrustLevel::Quarantined,
            PrincipalType::System => TrustLevel::Full,
        }
    }

    /// Can this trust level perform the operation without approval?
    pub fn allows_without_approval(&self, capability: &CapabilityId) -> bool {
        match self {
            TrustLevel::Full => true,
            TrustLevel::Established { .. } => true,
            TrustLevel::New => capability.is_read_only(),
            TrustLevel::Quarantined => false,
            TrustLevel::Blocked => false,
        }
    }
}
```

### Policy Matrix

| Principal Type | Default Trust | Read Operations | Write Operations | Admin Operations |
|----------------|---------------|-----------------|------------------|------------------|
| TrustedUi | Full | Allow | Allow | Allow |
| Canvas | Full | Allow | Allow | Allow |
| User | Full | Allow | Allow | Allow |
| Agent (scoped) | New | Allow | Check scope | Require approval |
| Agent (established) | Established | Allow | Allow (in scope) | Check scope |
| Peer | Quarantined | Limited | Require approval | Deny |
| System | Full | Allow | Allow | Allow |

### Scope System

Agents have explicit scopes that limit their capabilities:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Full access (dangerous, avoid)
    Full,

    /// Read message history
    ReadMessages,

    /// Send new messages
    SendMessages,

    /// Read entity information
    ReadEntities,

    /// Create/modify entities
    WriteEntities,

    /// Read files from virtual disk
    ReadFiles,

    /// Upload/modify files
    WriteFiles,

    /// Add/remove entity members
    ManageMembers,

    /// Create/accept/reject invites
    ManageInvites,

    /// Kanban board operations
    ManageKanban,

    /// Canvas operations
    ManageCanvas,

    /// Start/join calls
    ManageCalls,
}

impl Scope {
    /// Check if scope covers a capability
    pub fn covers(&self, capability: &CapabilityId) -> bool {
        match self {
            Scope::Full => true,
            Scope::ReadMessages => capability.0.starts_with("messaging.") && capability.is_read_only(),
            Scope::SendMessages => capability.0 == "messaging.send",
            Scope::ReadEntities => capability.0.starts_with("entity.") && capability.is_read_only(),
            Scope::WriteEntities => capability.0.starts_with("entity."),
            Scope::ReadFiles => capability.0.starts_with("drive.") && capability.is_read_only(),
            Scope::WriteFiles => capability.0.starts_with("drive."),
            Scope::ManageMembers => capability.0.starts_with("member."),
            Scope::ManageInvites => capability.0.starts_with("invite."),
            Scope::ManageKanban => capability.0.starts_with("kanban."),
            Scope::ManageCanvas => capability.0.starts_with("canvas."),
            Scope::ManageCalls => capability.0.starts_with("call."),
        }
    }
}
```

### Trust Escalation

Users can escalate agent trust levels:

```rust
pub struct TrustManager {
    agent_trust: HashMap<Uuid, TrustLevel>,
}

impl TrustManager {
    /// Escalate agent trust (user action)
    pub fn escalate(&mut self, token_id: Uuid, new_level: TrustLevel) -> Result<()> {
        // Validate escalation is allowed
        let current = self.agent_trust.get(&token_id).unwrap_or(&TrustLevel::New);
        if new_level < *current {
            return Err(Error::CannotDemoteViaEscalate);
        }
        self.agent_trust.insert(token_id, new_level);
        Ok(())
    }

    /// Demote agent trust (user action or automatic)
    pub fn demote(&mut self, token_id: Uuid, reason: DemotionReason) {
        self.agent_trust.insert(token_id, TrustLevel::Quarantined);
        // Log demotion reason for audit
    }

    /// Block agent permanently
    pub fn block(&mut self, token_id: Uuid) {
        self.agent_trust.insert(token_id, TrustLevel::Blocked);
    }

    /// Update trust based on action outcomes
    pub fn record_outcome(&mut self, token_id: Uuid, success: bool) {
        if let Some(TrustLevel::Established { successful_actions, failed_actions }) =
            self.agent_trust.get_mut(&token_id)
        {
            if success {
                *successful_actions += 1;
            } else {
                *failed_actions += 1;
            }
        }
    }
}
```

### Creating Principals

```rust
// Dioxus creates TrustedUi principal
impl DioxusApp {
    fn get_principal(&self) -> Principal {
        Principal::TrustedUi {
            identity: self.current_identity().clone(),
            device_id: self.device_id().clone(),
        }
    }
}

// MCP creates Agent principal from delegate token
impl McpServer {
    fn get_principal(&self, token: &DelegateToken) -> Principal {
        Principal::Agent {
            identity: token.issuer.clone(),
            delegate_name: token.delegate_name.clone(),
            scopes: token.scopes.clone(),
            token_id: token.id,
            trust_level: self.trust_manager.get_level(token.id),
        }
    }
}

// Canvas creates Canvas principal
impl CanvasClient {
    fn get_principal(&self) -> Principal {
        Principal::Canvas {
            identity: self.current_identity().clone(),
            session_id: self.session_id,
        }
    }
}
```

## Consequences

### Benefits

1. **Explicit Identity**: Every operation has a clear caller
2. **Policy Foundation**: Policy Kernel can make principal-based decisions
3. **Trust Gradation**: Agents can earn trust over time
4. **Canvas Ready**: Canvas has defined principal type
5. **Audit Attribution**: Every receipt identifies the principal

### Trade-offs

1. **Breaking Change**: All service methods need principal parameter
2. **Migration Work**: Existing code must be updated
3. **Complexity**: More types to understand and maintain

### Risks Mitigated

1. **Privilege Confusion**: Clear principal types prevent confusion
2. **Agent Abuse**: Trust levels limit damage from compromised agents
3. **Audit Gaps**: Principal attribution ensures accountability

## Implementation Plan

### Phase 1: Type Definitions (Week 1)
- Define `Principal`, `PrincipalType`, `TrustLevel`
- Define `Scope` enum
- Add to `communitas-core/src/policy/`

### Phase 2: Service Integration (Week 2-3)
- Add `principal` parameter to UiServices methods
- Update Dioxus to pass `TrustedUi` principal
- Update MCP to pass `Agent` principal

### Phase 3: Trust Management (Week 4)
- Implement `TrustManager`
- Add trust escalation UI to Dioxus
- Wire to Policy Kernel

### Phase 4: Canvas Principal (Week 5)
- Define `Principal::Canvas` handling
- Wire to future Canvas client
- Verify same treatment as TrustedUi

## Alternatives Considered

1. **Single Principal Type**: Use one type with role field
   - Rejected: Loses type safety, conflates different concepts

2. **String-Based Principals**: Use string identifiers
   - Rejected: No type safety, easy to confuse

3. **RBAC Roles Only**: Use roles without principal types
   - Rejected: Doesn't capture agent/Canvas distinctions

4. **No Trust Levels**: Binary allow/deny
   - Rejected: Loses ability for agents to earn trust

## References

- [.architecture/ARCHITECTURE.md](../../.architecture/ARCHITECTURE.md) - Principal hierarchy overview
- [.architecture/THREAT_MODEL.md](../../.architecture/THREAT_MODEL.md) - Trust boundary analysis
- [ADR-024](ADR-024-policy-kernel-architecture.md) - Policy Kernel (consumer)
- [ADR-023](ADR-023-unlock-grants-capability-tokens.md) - Current token system

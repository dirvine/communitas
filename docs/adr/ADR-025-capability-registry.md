# ADR-025: Capability Registry

## Status

Accepted — Not Yet Implemented

## Context

Communitas currently has 197+ MCP tools that expose platform functionality. These tools have:
- Implicit input/output expectations
- Ad-hoc permission requirements
- No formal schema definitions
- No discoverability mechanism

For Canvas and third-party integrations to work reliably, we need:
1. Formal capability definitions with schemas
2. Runtime discoverability
3. Consistent metadata (roles, audit levels, offline support)
4. Cross-version compatibility

## Decision

Implement a **Capability Registry** that formalizes all operations as typed capabilities.

### Capability Definition

```rust
// communitas-core/src/capabilities/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Unique identifier: "domain.action" (e.g., "messaging.send")
    pub id: CapabilityId,

    /// Human-readable name
    pub name: String,

    /// Description of what this capability does
    pub description: String,

    /// JSON Schema for input validation
    pub input_schema: JsonSchema,

    /// JSON Schema for output type
    pub output_schema: JsonSchema,

    /// Minimum role required
    pub required_role: Role,

    /// Audit logging level
    pub audit_level: AuditLevel,

    /// Can this operation be undone?
    pub reversible: bool,

    /// Works offline (queues for sync)?
    pub offline_capable: bool,

    /// Modifies CRDT state?
    pub crdt_operation: bool,

    /// MCP tool name (if mapped)
    pub mcp_tool: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId(pub String);

impl CapabilityId {
    pub fn domain(&self) -> &str {
        self.0.split('.').next().unwrap_or("")
    }

    pub fn action(&self) -> &str {
        self.0.split('.').nth(1).unwrap_or("")
    }
}
```

### Registry Implementation

```rust
pub struct CapabilityRegistry {
    capabilities: HashMap<CapabilityId, Capability>,
    by_domain: HashMap<String, Vec<CapabilityId>>,
    by_mcp_tool: HashMap<String, CapabilityId>,
}

impl CapabilityRegistry {
    /// Get capability by ID
    pub fn get(&self, id: &CapabilityId) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    /// List all capabilities
    pub fn list(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.values()
    }

    /// List capabilities in a domain
    pub fn list_domain(&self, domain: &str) -> Vec<&Capability> {
        self.by_domain
            .get(domain)
            .map(|ids| ids.iter().filter_map(|id| self.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get capability for MCP tool
    pub fn from_mcp_tool(&self, tool_name: &str) -> Option<&Capability> {
        self.by_mcp_tool
            .get(tool_name)
            .and_then(|id| self.get(id))
    }

    /// Validate input against capability schema
    pub fn validate_input(&self, id: &CapabilityId, input: &Value) -> Result<(), ValidationError> {
        let cap = self.get(id).ok_or(ValidationError::UnknownCapability)?;
        validate_json_schema(&cap.input_schema, input)
    }

    /// List capabilities available to a principal
    pub fn list_for_principal(&self, principal: &Principal) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|cap| self.principal_can_invoke(principal, cap))
            .collect()
    }
}
```

### Capability Domains

| Domain | Description | Example Capabilities |
|--------|-------------|---------------------|
| `auth` | Authentication | `auth.login`, `auth.logout`, `auth.create_vault` |
| `messaging` | Messages | `messaging.send`, `messaging.edit`, `messaging.delete` |
| `entity` | Organizations/Projects | `entity.create_org`, `entity.update`, `entity.delete` |
| `member` | Membership | `member.invite`, `member.remove`, `member.set_role` |
| `contact` | Contacts | `contact.add`, `contact.block`, `contact.list` |
| `kanban` | Project boards | `kanban.create_card`, `kanban.move_card` |
| `drive` | File storage | `drive.upload`, `drive.download`, `drive.share` |
| `canvas` | Whiteboard | `canvas.add_element`, `canvas.transform` |
| `call` | Video calls | `call.start`, `call.join`, `call.end` |
| `presence` | Online status | `presence.update`, `presence.get` |
| `network` | P2P networking | `network.start`, `network.connect` |
| `settings` | Preferences | `settings.update_profile`, `settings.set_preferences` |
| `audit` | Audit logs | `audit.list_events`, `audit.export` |

### Role Hierarchy

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    None,           // Pre-authentication
    Authenticated,  // Logged in
    Member,         // Entity member
    Admin,          // Entity admin
    Owner,          // Entity owner
}

// Special contextual roles
pub enum ContextualRole {
    Author,     // Message author
    Assignee,   // Card assignee
    Host,       // Call host
    Invitee,    // Invitation recipient
}
```

### Audit Levels

```rust
#[derive(Debug, Clone)]
pub enum AuditLevel {
    /// Always log (auth, admin actions)
    Always,

    /// Log success and failure (standard operations)
    Standard,

    /// Log failures only (high-frequency reads)
    Minimal,

    /// Never log (presence heartbeats)
    None,
}
```

### Static Registry Initialization

The registry is built at compile time from capability definitions:

```rust
lazy_static! {
    pub static ref CAPABILITY_REGISTRY: CapabilityRegistry = {
        let mut registry = CapabilityRegistry::new();

        // Auth capabilities
        registry.register(Capability {
            id: CapabilityId::from("auth.login"),
            name: "Login".into(),
            description: "Authenticate with vault".into(),
            input_schema: json_schema!({
                "type": "object",
                "required": ["four_words", "password"],
                "properties": {
                    "four_words": { "type": "string", "pattern": "^[a-z]+-[a-z]+-[a-z]+-[a-z]+$" },
                    "password": { "type": "string", "minLength": 1 }
                }
            }),
            output_schema: json_schema!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "format": "uuid" },
                    "display_name": { "type": "string" }
                }
            }),
            required_role: Role::None,
            audit_level: AuditLevel::Always,
            reversible: false,
            offline_capable: true,
            crdt_operation: false,
            mcp_tool: Some("authenticate".into()),
        });

        // ... 120+ more capabilities

        registry
    };
}
```

### Canvas/Agent Discovery

Canvas and agents can discover available capabilities:

```rust
// Agent capability discovery
let registry = CapabilityRegistry::global();

// List all capabilities I can invoke
let available = registry.list_for_principal(&my_principal);

// Get schema for a specific capability
let cap = registry.get(&CapabilityId::from("messaging.send"))?;
let input_schema = &cap.input_schema;

// Validate before invocation
registry.validate_input(&cap.id, &my_args)?;

// Invoke via UiServices
services.invoke(&cap.id, my_args).await?;
```

## Consequences

### Benefits

1. **Formal Definitions**: Every operation has a schema
2. **Discoverability**: Canvas/agents can list available capabilities
3. **Validation**: Input validated before execution
4. **Consistency**: Same metadata across Dioxus, MCP, Canvas
5. **Documentation**: Schemas are self-documenting
6. **Versioning**: Schema evolution with compatibility checks

### Trade-offs

1. **Upfront Work**: Must define 120+ capability schemas
2. **Maintenance**: Schemas must stay in sync with implementations
3. **Binary Size**: Schema data adds to binary

### Risks Mitigated

1. **Schema Drift**: Generated from single source
2. **Invalid Input**: Validated at registry level
3. **Missing Capabilities**: Compile-time registry ensures completeness

## Implementation Plan

### Phase 1: Type Definitions (Week 1)
- Define `Capability`, `CapabilityId`, `Role`, `AuditLevel`
- Create `CapabilityRegistry` struct
- Add to `communitas-core/src/capabilities/`

### Phase 2: Core Capabilities (Week 2-3)
- Define auth capabilities (7)
- Define messaging capabilities (11)
- Define entity capabilities (9)
- Define member capabilities (6)

### Phase 3: Remaining Capabilities (Week 4-5)
- Define kanban capabilities (17)
- Define drive capabilities (10)
- Define canvas capabilities (10)
- Define call/presence/network capabilities

### Phase 4: Integration (Week 6)
- Wire registry to Policy Kernel
- Add MCP tool mapping
- Enable capability discovery endpoint

## Alternatives Considered

1. **OpenAPI/Swagger**: Use existing API spec format
   - Rejected: Designed for REST, not capability-based systems

2. **Protocol Buffers**: Use protobuf for schemas
   - Rejected: Adds build dependency, less flexible

3. **Runtime Definitions**: Load capabilities from config
   - Rejected: Lose compile-time guarantees

4. **No Registry**: Keep implicit tool definitions
   - Rejected: Current state, blocks Canvas/agent discoverability

## References

- [.architecture/CAPABILITIES.md](../../.architecture/CAPABILITIES.md) - Full capability catalog
- [ADR-024](ADR-024-policy-kernel-architecture.md) - Policy Kernel (consumer)
- [ADR-018](ADR-018-mcp-external-integration.md) - MCP tool mapping
- [docs/api/mcp-api.md](../api/mcp-api.md) - Current MCP tool definitions

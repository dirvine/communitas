# ADR-010: Cross-Organization Invites

## Status

Accepted (2025-12-24)

## Context

### The Problem

Collaboration across organizational boundaries is essential for modern work:

- **Partnerships**: Company A collaborates with Company B
- **Contractors**: External consultants join project teams
- **Open source**: Contributors from different organizations
- **Events**: Temporary access for conference attendees

Traditional invite systems have limitations:

| Approach | Problem |
|----------|---------|
| Email invites | Requires email infrastructure, spam issues |
| Link sharing | Can be forwarded, no recipient verification |
| Admin-only | Bottleneck, slow for large organizations |
| Central directory | Vendor lock-in, privacy concerns |

### Requirements

- Identity-based (pubkey_hex or invite token; no email required)
- Recipient verification
- Time-limited validity
- Revocable by issuer
- Role-based access
- Works offline (queue for sync)
- Cross-organization capability

## Decision

Implement an **identity-based invite system** with shareable invite tokens for cross-organization collaboration:

### Invite Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Invite Lifecycle                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. CREATE                  2. SHARE                 3. ACCEPT      │
│  ─────────                  ────────                 ────────       │
│                                                                     │
│  Creator (Org A)           Out-of-band              Recipient       │
│  ┌──────────────┐         ┌──────────────┐        ┌──────────────┐ │
│  │ Create       │         │ Share invite │        │ Enter invite │ │
│  │ invite for   │ ───────►│ code via:    │───────►│ code:        │ │
│  │ specific     │         │ - Chat       │        │              │ │
│  │ recipient    │         │ - Voice      │        │ "brave-      │ │
│  │              │         │ - QR code    │        │  echo-nova-  │ │
│  │ Parameters:  │         │              │        │  frost"      │ │
│  │ - entity_id  │         └──────────────┘        └──────────────┘ │
│  │ - recipient  │                                         │        │
│  │ - role       │                                         ▼        │
│  │ - expires    │                                 ┌──────────────┐ │
│  └──────────────┘                                 │ System       │ │
│         │                                         │ verifies:    │ │
│         ▼                                         │ - recipient  │ │
│  ┌──────────────┐                                 │ - expiration │ │
│  │ Invite ID:   │                                 │ - status     │ │
│  │ brave-echo-  │                                 │              │ │
│  │ nova-frost   │                                 │ If valid:    │ │
│  │              │◄────────────────────────────────│ Add member   │ │
│  │ Stored in    │                                 └──────────────┘ │
│  │ CRDT         │                                                  │
│  └──────────────┘                                                  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Invite Structure

```rust
pub struct Invite {
    pub id: String,                // Invite token (opaque)
    pub entity_type: EntityType,   // Organization, Group, Channel, Project
    pub entity_id: String,         // Target entity identity (pubkey_hex)
    pub recipient_id: String,      // Intended recipient identity (pubkey_hex)
    pub created_by: String,        // Creator identity (pubkey_hex)
    pub created_at: i64,
    pub expires_at: Option<i64>,   // Optional expiration
    pub role: String,              // "admin" | "member" | "guest"
    pub status: InviteStatus,
    pub message: Option<String>,   // Optional personal message
}

pub enum InviteStatus {
    Pending,    // Awaiting acceptance
    Accepted,   // Recipient joined
    Rejected,   // Recipient declined
    Revoked,    // Creator cancelled
    Expired,    // Past expiration time
}
```

### Invite Service API

```rust
impl InviteService {
    /// Create a new invite for a specific recipient
    pub async fn create_invite(
        &self,
        creator_id: &str,
        request: InviteRequest,
    ) -> Result<Invite>;

    /// Accept an invite (recipient only)
    pub async fn accept_invite(
        &self,
        recipient_id: &str,
        invite_id: &str,
    ) -> Result<()>;

    /// Reject an invite (recipient only)
    pub async fn reject_invite(
        &self,
        recipient_id: &str,
        invite_id: &str,
    ) -> Result<()>;

    /// Revoke an invite (creator/admin only)
    pub async fn revoke_invite(
        &self,
        revoker_id: &str,
        invite_id: &str,
    ) -> Result<()>;

    /// List invites for an entity
    pub async fn list_entity_invites(
        &self,
        requester_id: &str,
        entity_type: EntityType,
        entity_id: &str,
    ) -> Result<Vec<Invite>>;

    /// List pending invites for a user
    pub async fn list_pending_invites(
        &self,
        recipient_id: &str,
    ) -> Result<Vec<Invite>>;
}
```

### Request Builder

```rust
pub struct InviteRequest {
    recipient_id: String,
    entity_type: EntityType,
    entity_id: String,
    role: String,
    expires_in_hours: Option<u64>,
    message: Option<String>,
}

impl InviteRequest {
    pub fn new(
        recipient_id: &str,
        entity_type: EntityType,
        entity_id: &str,
        role: &str,
    ) -> Self;

    pub fn with_expiration(self, hours: u64) -> Self;
    pub fn with_message(self, message: String) -> Self;
}
```

### Security Model

**Recipient Verification**:
```rust
fn verify_recipient(invite: &Invite, claimer_id: &str) -> bool {
    // Only the intended recipient can accept
    invite.recipient_id == claimer_id
}
```

**Permission Checks**:
```rust
fn can_create_invite(creator: &Member, entity: &Entity) -> bool {
    // Owners and admins can create invites
    matches!(creator.role.as_str(), "owner" | "admin")
}

fn can_revoke_invite(revoker: &Member, invite: &Invite) -> bool {
    // Creator or entity admin can revoke
    revoker.member_id == invite.created_by ||
    matches!(revoker.role.as_str(), "owner" | "admin")
}
```

### Invite Code Format

Invite codes are opaque tokens (not derived from four-word networking):

```rust
// Generate random invite token
let invite_id = generate_invite_token()?; // "inv_7w3R9kQGm1..."

// Validate invite token
assert!(validate_invite_token(&invite_id));
```

**Why opaque tokens?**:
- Shareable via chat or QR
- Avoids identity coupling
- Independent of connection-word encoding

### Cross-Organization Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│              Cross-Organization Invite                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Organization A                        Organization B               │
│  (acme-corp-river-stone)              (beta-inc-moon-star)         │
│                                                                     │
│  ┌─────────────────────┐              ┌─────────────────────┐      │
│  │ Alice (admin)       │              │ Bob (member)        │      │
│  │                     │              │                     │      │
│  │ Creates invite to   │   Invite    │ Receives invite for │      │
│  │ project "mobile-    │───────────►│ "mobile-app-        │      │
│  │ app-forest-rain"    │   Code     │  forest-rain"       │      │
│  │                     │              │                     │      │
│  │ Recipient: Bob      │              │ Accepts → becomes   │      │
│  │ Role: member        │              │ project member      │      │
│  └─────────────────────┘              └─────────────────────┘      │
│                                                                     │
│  Result: Bob can now access the project while remaining in Org B   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### CRDT Storage

Invites are stored in entity CRDT documents:

```rust
// In entity:core document
Doc {
    "metadata": {...},
    "members": {...},
    "invites": Map<invite_id, InviteData>,  // Active invites
}

InviteData = Map {
    "id": String,
    "recipient_id": String,
    "created_by": String,
    "role": String,
    "status": String,
    "created_at": i64,
    "expires_at": i64,
}
```

## Consequences

### Benefits

- **No email required**: Works with identity IDs or invite tokens
- **Verifiable recipient**: Only intended recipient can accept
- **Revocable**: Creator can cancel before acceptance
- **Time-limited**: Optional expiration prevents stale invites
- **Auditable**: Full history in CRDT
- **Cross-org**: Works across organizational boundaries

### Trade-offs

- **Requires recipient identity**: Must know recipient identity or share token
- **Single use**: Each invite for one recipient
- **No discovery**: Can't browse available invites

### Role Mapping

| Invite Role | Capabilities |
|-------------|--------------|
| owner | Full control (rare for invites) |
| admin | Manage members, create invites |
| member | Read/write access |
| guest | Read-only access (time-limited) |

## Alternatives Considered

1. **Email-based invites**: Send link via email
   - Rejected: Requires email infrastructure, no recipient verification

2. **Public invite links**: Anyone with link can join
   - Rejected: No control over who joins

3. **Admin approval**: All membership requests require approval
   - Rejected: Bottleneck, doesn't scale

4. **Central directory**: Look up users in central system
   - Rejected: Privacy concerns, vendor lock-in

5. **Blockchain tokens**: NFT-style membership tokens
   - Rejected: Overkill, expensive

## References

- Implementation: `communitas-core/src/invite.rs`
- Service: `communitas-core/src/invite_service.rs`
- Bindings: `communitas-bindings/src/lib.rs`
- Related ADR: [ADR-001 Four-Word Identity](ADR-001-four-word-identity-system.md) (superseded)
- Related ADR: [ADR-004 Entity Hierarchy](ADR-004-entity-hierarchy-model.md)

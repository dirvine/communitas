# ADR-005: Virtual Disk Architecture

## Status

Accepted (2025-12-24)

## Context

### The Problem

Collaboration platforms need file storage with different access patterns:

- **Cloud storage**: Vendor lock-in, privacy concerns, requires internet
- **Local-only**: No sharing, no collaboration
- **Shared drives**: All-or-nothing access, no granularity

Communitas needs storage that:
- Works offline (local-first)
- Supports different access policies
- Syncs with peers when connected
- Provides per-entity isolation
- Enables DNS-free website publishing

### Requirements

- Three access levels: private, public, shared
- Per-entity storage isolation
- Content-addressed for deduplication
- Encrypted where appropriate
- Supports website publishing

## Decision

Implement a **Virtual Disk System** where each entity has three disk types with different access and replication policies:

### Disk Types

| Disk | Access | Encryption | Replication | Use Case |
|------|--------|------------|-------------|----------|
| **Private** | Owner only | User key | None | Personal files, credentials |
| **Public** | Anyone | None | Full | Websites, public docs |
| **Shared** | Entity members | Group key | Members | Team documents |

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│            Virtual Disk Architecture                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Entity: ocean-forest-moon-star                                    │
│                                                                     │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐ │
│  │   Private Disk   │  │   Public Disk    │  │   Shared Disk    │ │
│  │                  │  │                  │  │                  │ │
│  │ /credentials/    │  │ /website/        │  │ /docs/           │ │
│  │ /drafts/         │  │   index.html     │  │ /images/         │ │
│  │ /personal/       │  │   style.css      │  │ /attachments/    │ │
│  │                  │  │ /public-docs/    │  │                  │ │
│  │ 🔒 User-key      │  │ 🌐 Unencrypted   │  │ 🔐 Group-key     │ │
│  │    encrypted     │  │    replicated    │  │    encrypted     │ │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘ │
│         │                      │                      │            │
│         ▼                      ▼                      ▼            │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │              libSQL Database (Content Store)                 │  │
│  │                                                              │  │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐ │  │
│  │  │ disk_files     │  │ disk_chunks    │  │ disk_metadata  │ │  │
│  │  │ (file index)   │  │ (content)      │  │ (entity info)  │ │  │
│  │  └────────────────┘  └────────────────┘  └────────────────┘ │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Disk Service API

```rust
pub enum DiskType {
    Private,  // Owner-only, encrypted
    Public,   // Anyone can read, replicated
    Shared,   // Members only, group-encrypted
}

impl DiskService {
    /// Write file to virtual disk
    pub async fn write_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        content: &[u8],
    ) -> Result<FileHash>;

    /// Read file from virtual disk
    pub async fn read_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<u8>>;

    /// List files in directory
    pub async fn list_files(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<FileEntry>>;

    /// Delete file
    pub async fn delete_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<()>;
}
```

### Content Addressing

Files are stored content-addressed using BLAKE3:

```
┌─────────────────────────────────────────────────────────────────────┐
│                Content-Addressed Storage                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  File: /shared/docs/report.pdf (1.5MB)                             │
│                                                                     │
│  1. Split into chunks (1MB each)                                   │
│     ┌────────────────┐  ┌────────────────┐                         │
│     │ Chunk 0 (1MB)  │  │ Chunk 1 (0.5MB)│                         │
│     │                │  │                │                         │
│     │ hash: abc123.. │  │ hash: def456.. │                         │
│     └────────────────┘  └────────────────┘                         │
│                                                                     │
│  2. Store chunks by hash                                           │
│     disk_chunks: { "abc123..": [chunk0], "def456..": [chunk1] }   │
│                                                                     │
│  3. Store file manifest                                            │
│     disk_files: {                                                   │
│       path: "/shared/docs/report.pdf",                             │
│       chunks: ["abc123..", "def456.."],                            │
│       size: 1572864,                                               │
│       mime: "application/pdf"                                      │
│     }                                                              │
│                                                                     │
│  4. Deduplication                                                  │
│     If same chunk exists elsewhere → reuse (no duplicate storage)  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Encryption Policies

| Disk | Key Source | Algorithm |
|------|------------|-----------|
| Private | User's master key (from passkey/password) | ChaCha20-Poly1305 |
| Public | None (plaintext) | None |
| Shared | Group key (distributed to members) | ChaCha20-Poly1305 |

**Private disk encryption**:
```rust
fn encrypt_private(content: &[u8], user_key: &[u8; 32]) -> Vec<u8> {
    let nonce = generate_nonce();
    let cipher = ChaCha20Poly1305::new(user_key.into());
    cipher.encrypt(&nonce, content).unwrap()
}
```

**Shared disk key distribution**:
```rust
// Group key encrypted for each member's public key
struct GroupKeyDistribution {
    group_id: String,
    encrypted_keys: HashMap<MemberId, EncryptedGroupKey>,
}
```

### Website Publishing

Public disks can serve as website roots:

```rust
// Set entity's website root
pub async fn set_website_root(
    &self,
    entity_id: &str,
    root_hash: &str,  // BLAKE3 hash of /public/website/
) -> Result<()>;

// Resolve website content
pub async fn resolve_website(
    &self,
    four_words: &str,
    path: &str,
) -> Result<Vec<u8>>;
```

**DNS-free access**:
```
ocean-forest-moon-star.communitas/index.html
→ Resolves to: /public/website/index.html
→ No DNS required, P2P resolution
```

### Replication Strategy

| Disk | Replication | Sync |
|------|-------------|------|
| Private | None | Never (local only) |
| Public | Full (all peers) | Gossip broadcast |
| Shared | Members only | Gossip with group filter |

## Consequences

### Benefits

- **Access control**: Three tiers cover all use cases
- **Deduplication**: Content-addressing saves space
- **Encryption**: Private and shared data protected
- **Website publishing**: DNS-free publishing built-in
- **Offline-first**: All operations work locally
- **Per-entity isolation**: Clean data boundaries

### Trade-offs

- **Storage overhead**: Multiple copies for redundancy
- **Key management**: Group keys require distribution
- **No fine-grained ACL**: Only three access levels
- **Website size limits**: Constrained by P2P bandwidth

### Performance

| Operation | Latency | Notes |
|-----------|---------|-------|
| Read (local) | <10ms | From local libSQL |
| Write (local) | <50ms | Chunk + store |
| Sync (remote) | Variable | Depends on file size |
| Dedup check | <5ms | Hash lookup |

## Alternatives Considered

1. **Single disk per entity**: One storage space
   - Rejected: Can't separate access levels

2. **File-level ACL**: Per-file permissions
   - Rejected: Complex, hard to reason about

3. **External storage**: S3, IPFS integration
   - Rejected: Dependency, not local-first

4. **No content addressing**: Store files directly
   - Rejected: No deduplication, inefficient

## References

- Implementation: `communitas-core/src/disk_service.rs`
- Storage: `communitas-core/src/storage/`
- CRDT Architecture: `docs/architecture/crdt-system.md`
- Related ADR: [ADR-004 Entity Hierarchy](ADR-004-entity-hierarchy-model.md)
- Related ADR: [ADR-011 Encrypted Vault Storage](ADR-011-encrypted-vault-storage.md)

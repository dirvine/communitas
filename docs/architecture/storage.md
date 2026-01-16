# Storage Architecture

**Version**: 1.0
**Last Updated**: 2025-10-15
**Status**: Active

## Overview

Communitas implements a sophisticated multi-layered storage architecture designed for local-first operation with offline capability, strong encryption, and eventual consistency across peers. The storage system combines encrypted vaults, virtual disks, content addressing, and forward error correction to provide resilient, secure data persistence.

**Core Technologies**:
- **Encryption**: ChaCha20-Poly1305 with PBKDF2 key derivation
- **Hashing**: BLAKE3 for content addressing and integrity
- **Error Correction**: Reed-Solomon FEC for resilience
- **Platform Integration**: System keyring (macOS Keychain, Windows DPAPI, Linux Secret Service)
- **Database**: libSQL (Turso) for SQL materialization

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Encrypted Vaults](#encrypted-vaults)
- [Virtual Disks](#virtual-disks)
- [Content Addressing](#content-addressing)
- [Forward Error Correction](#forward-error-correction)
- [Website Publishing](#website-publishing-saorsa-sites)
- [Storage Policies](#storage-policies)
- [Platform Integration](#platform-integration)
- [Performance Characteristics](#performance-characteristics)

## Architecture Overview

### Storage Stack Layers

```
┌─────────────────────────────────────────────────────────────┐
│                   APPLICATION LAYER                         │
│         (Files, Messages, Documents, Websites)             │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                  VIRTUAL DISK LAYER                         │
│    - Private Disk (encrypted, local-only)                  │
│    - Public Disk (content-addressed, distributed)          │
│    - Shared Disk (group-encrypted, replicated)             │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                  ENCRYPTED VAULT LAYER                      │
│         Per-identity ChaCha20-Poly1305 encryption          │
│         PBKDF2 key derivation (100k iterations)            │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              CONTENT ADDRESSING LAYER                       │
│           BLAKE3 hashing for integrity                      │
│           512KB blocks for large content                    │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│           FORWARD ERROR CORRECTION (OPTIONAL)               │
│              Reed-Solomon encoding                          │
│              Adaptive sharding (3+2 to 16+8)                │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│               PLATFORM STORAGE LAYER                        │
│  macOS: Keychain | Windows: DPAPI | Linux: Secret Service  │
│              Filesystem + libSQL database                   │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

#### EncryptedStorageManager
Central manager for all encrypted storage operations.

**File**: `communitas-core/src/encrypted_storage/mod.rs`

```rust
pub struct EncryptedStorageManager {
    config: StorageConfig,
    vaults: Arc<RwLock<HashMap<String, Arc<EncryptedVault>>>>,
    active_sessions: Arc<RwLock<HashMap<String, Session>>>,
    key_manager: Arc<KeyManager>,
    platform_storage: Arc<PlatformStorage>,
    app_config: Arc<RwLock<AppConfigManager>>,
    passkey_manager: Arc<PasskeyManager>,
}
```

#### Storage Configuration

```rust
pub struct StorageConfig {
    /// Base directory for encrypted vaults
    pub vault_dir: PathBuf,

    /// PBKDF2 iteration count (100,000 per DESIGN.md)
    pub pbkdf2_iterations: u32,

    /// Enable Forward Error Correction
    pub enable_fec: bool,

    /// FEC redundancy factor (1.5 = 50% redundancy)
    pub fec_redundancy: f32,

    /// Maximum vault size (0 = unlimited)
    pub max_vault_size: u64,

    /// Enable platform keyring integration
    pub use_keyring: bool,

    /// Cache timeout for decrypted data (seconds)
    pub cache_timeout: u64,
}
```

## Encrypted Vaults

### Overview

Each four-word identity has an **encrypted vault** that stores all local data with strong encryption. Vaults use ChaCha20-Poly1305 for authenticated encryption and PBKDF2 for secure key derivation from passwords.

**File**: `communitas-core/src/encrypted_storage/vault.rs`

### Vault Structure

```rust
pub struct EncryptedVault {
    /// Four-word identity address
    pub four_words: String,

    /// User-friendly display name
    pub display_name: String,

    /// Vault metadata (unencrypted for discovery)
    metadata: VaultMetadata,

    /// Encryption key (zeroized on drop)
    encryption_key: Zeroizing<Vec<u8>>,

    /// In-memory data store
    data_store: RwLock<HashMap<String, EncryptedEntry>>,

    /// Vault directory path
    vault_path: PathBuf,

    /// Key manager for crypto operations
    key_manager: KeyManager,

    /// Optional FEC storage
    fec_storage: Option<FecStorage>,
}
```

### Vault Metadata

Stored **unencrypted** for vault discovery and verification:

```rust
pub struct VaultMetadata {
    /// Schema version
    pub version: u32,

    /// Creation timestamp
    pub created_at: u64,

    /// Last access timestamp
    pub last_accessed: u64,

    /// PBKDF2 salt (random per vault)
    pub salt: Vec<u8>,

    /// PBKDF2 iterations
    pub pbkdf2_iterations: u32,

    /// Total storage size
    pub total_size: u64,

    /// Number of entries
    pub entry_count: usize,

    /// BLAKE3 checksum of vault contents
    pub checksum: Vec<u8>,
}
```

### Encryption Process

#### 1. Key Derivation (PBKDF2)

```rust
/// Derive encryption key from password using PBKDF2
pub async fn derive_key(
    &self,
    password: &str,
    salt: &[u8],
) -> Result<Zeroizing<Vec<u8>>> {
    // Use PBKDF2-HMAC-SHA256 with 100,000 iterations (per DESIGN.md)
    let iterations = 100_000;

    let mut key = vec![0u8; 32]; // 256-bit key
    pbkdf2::pbkdf2_hmac::<sha2::Sha256>(
        password.as_bytes(),
        salt,
        iterations,
        &mut key,
    );

    Ok(Zeroizing::new(key))
}
```

**Parameters**:
- **Algorithm**: PBKDF2-HMAC-SHA256
- **Iterations**: 100,000 (OWASP minimum for 2024)
- **Key size**: 256 bits (32 bytes)
- **Salt size**: 256 bits (32 bytes, random per vault)

#### 2. Authenticated Encryption (ChaCha20-Poly1305)

```rust
/// Encrypt data with ChaCha20-Poly1305
pub fn encrypt(&self, key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    use chacha20poly1305::{
        aead::{Aead, NewAead, Payload},
        ChaCha20Poly1305,
    };

    // Generate random 96-bit nonce
    let mut nonce = [0u8; 12];
    getrandom::getrandom(&mut nonce)?;

    // Create cipher
    let cipher = ChaCha20Poly1305::new_from_slice(key)?;

    // Encrypt with AEAD
    let ciphertext = cipher.encrypt(&nonce.into(), plaintext)?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(12 + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data with ChaCha20-Poly1305
pub fn decrypt(&self, key: &[u8], ciphertext: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    use chacha20poly1305::{
        aead::{Aead, NewAead},
        ChaCha20Poly1305,
    };

    // Extract nonce (first 12 bytes)
    let nonce = &ciphertext[..12];
    let encrypted_data = &ciphertext[12..];

    // Create cipher
    let cipher = ChaCha20Poly1305::new_from_slice(key)?;

    // Decrypt with AEAD
    let plaintext = cipher.decrypt(nonce.into(), encrypted_data)?;

    Ok(Zeroizing::new(plaintext))
}
```

**Properties**:
- **Algorithm**: ChaCha20-Poly1305 (RFC 8439)
- **Key size**: 256 bits
- **Nonce size**: 96 bits (random per encryption)
- **Authentication tag**: 128 bits (AEAD)
- **Performance**: Superior to AES-GCM on most CPUs

### Vault Operations

#### Creating a Vault

```rust
/// Create a new encrypted vault
pub async fn create(
    four_words: String,
    display_name: String,
    encryption_key: Zeroizing<Vec<u8>>,
    salt: Vec<u8>,
    config: &StorageConfig,
) -> Result<Self> {
    let vault_path = config.vault_dir.join(&four_words);

    // Create vault directory
    fs::create_dir_all(&vault_path).await?;

    // Initialize metadata
    let metadata = VaultMetadata {
        version: 1,
        created_at: current_timestamp(),
        last_accessed: current_timestamp(),
        salt,
        pbkdf2_iterations: config.pbkdf2_iterations,
        total_size: 0,
        entry_count: 0,
        checksum: vec![],
    };

    // Save metadata (unencrypted)
    let metadata_path = vault_path.join("vault.meta");
    fs::write(&metadata_path, serde_json::to_vec(&metadata)?).await?;

    // Create password verifier for empty vaults
    let verifier_data = b"communitas:password:verifier:v1";
    let encrypted_verifier = key_manager.encrypt(&encryption_key, verifier_data)?;
    fs::write(vault_path.join("password.verifier"), encrypted_verifier).await?;

    // Store identity data (display name)
    let identity_data = IdentityData {
        display_name: display_name.clone(),
        created_at: current_timestamp(),
    };
    let identity_json = serde_json::to_vec(&identity_data)?;
    let encrypted_identity = key_manager.encrypt(&encryption_key, &identity_json)?;
    fs::write(vault_path.join("identity.enc"), encrypted_identity).await?;

    Ok(Self { /* ... */ })
}
```

#### Loading a Vault

```rust
/// Load an existing vault with password validation
pub async fn load(
    four_words: &str,
    password: &str,
    config: &StorageConfig,
) -> Result<Self> {
    let vault_path = config.vault_dir.join(four_words);

    // Load metadata
    let metadata: VaultMetadata = serde_json::from_slice(
        &fs::read(vault_path.join("vault.meta")).await?
    )?;

    // Derive encryption key from password
    let key_manager = KeyManager::new(
        metadata.pbkdf2_iterations,
        config.use_keyring
    ).await?;
    let encryption_key = key_manager.derive_key(password, &metadata.salt).await?;

    // SECURITY: Validate password by decrypting something
    if vault_path.join("index.enc").exists() {
        // Decrypt index - will fail if wrong password
        let encrypted_index = fs::read(vault_path.join("index.enc")).await?;
        let decrypted_index = key_manager.decrypt(&encryption_key, &encrypted_index)
            .context("Invalid password or corrupted vault")?;
        let data_store = serde_json::from_slice(&decrypted_index)?;

        Ok(Self { data_store, /* ... */ })
    } else {
        // For empty vaults, validate via password verifier
        let encrypted_verifier = fs::read(vault_path.join("password.verifier")).await?;
        key_manager.decrypt(&encryption_key, &encrypted_verifier)
            .context("Invalid password")?;

        Ok(Self { data_store: HashMap::new(), /* ... */ })
    }
}
```

#### Storing Data

```rust
/// Store encrypted data in vault
pub async fn store(&self, key: &str, data: &[u8]) -> Result<()> {
    // Encrypt data
    let encrypted = self.key_manager.encrypt(&self.encryption_key, data)?;

    // Create entry with metadata
    let entry = EncryptedEntry {
        key: key.to_string(),
        encrypted_data: encrypted.clone(),
        metadata: EntryMetadata {
            created_at: current_timestamp(),
            modified_at: current_timestamp(),
            size: data.len(),
            content_type: ContentType::LocalFile,
            compression: None,
            fec_shards: None,
        },
    };

    // Store in memory
    self.data_store.write().await.insert(key.to_string(), entry);

    // Store on disk
    let file_path = self.vault_path.join(format!("{}.enc", key));
    fs::write(file_path, encrypted).await?;

    // Update encrypted index
    self.save_index().await?;

    Ok(())
}
```

#### Retrieving Data

```rust
/// Retrieve and decrypt data from vault
pub async fn retrieve(&self, key: &str) -> Result<Vec<u8>> {
    let store = self.data_store.read().await;
    let entry = store.get(key)
        .ok_or_else(|| anyhow!("Key not found: {}", key))?;

    // Check if data is in FEC shards
    if let Some(shard_paths) = &entry.metadata.fec_shards {
        if let Some(fec) = &self.fec_storage {
            let encrypted = fec.retrieve_from_fec(shard_paths).await?;
            let decrypted = self.key_manager.decrypt(&self.encryption_key, &encrypted)?;
            return Ok(decrypted.to_vec());
        }
    }

    // Decrypt regular data
    let decrypted = self.key_manager.decrypt(
        &self.encryption_key,
        &entry.encrypted_data
    )?;

    Ok(decrypted.to_vec())
}
```

### Vault Statistics

```rust
pub struct VaultStats {
    /// Total storage size (bytes)
    pub total_size: usize,

    /// Number of entries
    pub entry_count: usize,

    /// Regular files
    pub file_count: usize,

    /// FEC-protected files
    pub fec_count: usize,

    /// Creation timestamp
    pub created_at: u64,

    /// Last access timestamp
    pub last_accessed: u64,
}
```

## Virtual Disks

### Overview

Each entity (user, group, channel, project, organization) has **three virtual disks** with different encryption and distribution policies:

1. **Private Disk**: Encrypted, local-only storage
2. **Public Disk**: Content-addressed, distributed storage
3. **Shared Disk**: Group-accessible with shared encryption

### Virtual Disk Types

```rust
pub enum DiskType {
    /// Private: Encrypted, never shared
    Private,

    /// Public: Content-addressed, publicly distributed
    Public,

    /// Shared: Group-encrypted, replicated to group members
    Shared,
}
```

### Private Disk

**Purpose**: Personal, sensitive data that never leaves the device.

**Encryption**: ChaCha20-Poly1305 with user's password-derived key

**Storage Location**: Local vault only

**Use Cases**:
- Personal notes and drafts
- Credentials and API keys (auto-encrypted)
- Private documents
- Identity data
- Session tokens

**Example**:
```rust
// Store sensitive data in private disk
vault.store("credentials/github_token", token_bytes).await?;
vault.store("notes/personal/diary.md", diary_content).await?;
```

### Public Disk

**Purpose**: Content meant to be publicly accessible and distributed.

**Encryption**: None (content is public)

**Content Addressing**: BLAKE3 hash for integrity

**Storage**: Distributed via P2P network, cached locally

**Use Cases**:
- Website content (HTML, CSS, JS, images)
- Public documents
- Shared files
- Avatar images
- Public announcements

**Example**:
```rust
// Publish website content to public disk
let html_block = Block::new(html_content);
let css_block = Block::new(css_content);

let manifest = SiteManifest::new(
    site_id,
    manifest_version,
    vec![
        ("/index.html".to_string(), html_block.hash),
        ("/style.css".to_string(), css_block.hash),
    ],
);

site_publisher.publish_manifest(manifest).await?;
```

### Shared Disk

**Purpose**: Collaborative data accessible to group members.

**Encryption**: ChaCha20-Poly1305 with shared group key

**Access Control**: Group membership determines access

**Storage**: Replicated to all group members via P2P

**Use Cases**:
- Team documents
- Project files
- Shared spreadsheets
- Collaborative notes
- Group media

**Example**:
```rust
// Store team document in shared disk
let group_key = group_context.get_shared_key(&group_id).await?;
let encrypted = encrypt_with_group_key(&document, &group_key)?;
shared_storage.store(group_id, "documents/roadmap.md", encrypted).await?;
```

### Disk Policies

| Disk Type | Encryption | Distribution | Replication | Use Case |
|-----------|------------|--------------|-------------|----------|
| Private   | ChaCha20-Poly1305 (password-derived) | Local only | None | Sensitive data |
| Public    | None | P2P network | Content-addressed | Public content |
| Shared    | ChaCha20-Poly1305 (group key) | Group members | All members | Collaboration |

## Content Addressing

### Overview

Public and shared content uses **content addressing** with BLAKE3 hashing for integrity verification and deduplication.

**File**: `communitas-core/src/gossip/sites.rs`

### Block Structure

```rust
/// Content-addressed block
pub struct Block {
    /// BLAKE3 hash of content (32 bytes)
    pub hash: [u8; 32],

    /// Raw block content (up to 512KB)
    pub content: Vec<u8>,
}

impl Block {
    /// Create a new block from content
    pub fn new(content: Vec<u8>) -> Self {
        let hash = blake3::hash(&content);
        Self {
            hash: hash.into(),
            content,
        }
    }

    /// Verify block hash matches content
    pub fn verify(&self) -> bool {
        let computed = blake3::hash(&self.content);
        computed.as_bytes() == &self.hash
    }
}
```

### Chunking Large Content

Large files are split into **512KB blocks** for efficient transfer:

```rust
/// Maximum block size per SPEC2.md §5.3
pub const MAX_BLOCK_SIZE: usize = 512 * 1024;

/// Chunk large content into blocks
pub fn chunk_content(content: &[u8], chunk_size: usize) -> Vec<Block> {
    content
        .chunks(chunk_size)
        .map(|chunk| Block::new(chunk.to_vec()))
        .collect()
}
```

### Content Verification

```rust
/// Verify all blocks in a manifest
pub fn verify_blocks(manifest: &SiteManifest, blocks: &[Block]) -> Result<()> {
    let mut hasher = blake3::Hasher::new();

    for (path, expected_hash) in &manifest.blocks {
        // Find block by hash
        let block = blocks.iter()
            .find(|b| &b.hash == expected_hash)
            .ok_or_else(|| anyhow!("Missing block for {}", path))?;

        // Verify block integrity
        if !block.verify() {
            return Err(anyhow!("Block verification failed for {}", path));
        }

        hasher.update(path.as_bytes());
        hasher.update(&block.hash);
    }

    // Verify root hash
    let computed_root = hasher.finalize();
    if computed_root.as_bytes() != &manifest.root_hash {
        return Err(anyhow!("Manifest root hash mismatch"));
    }

    Ok(())
}
```

### Deduplication

Content addressing enables automatic deduplication:

```rust
/// Check if block already exists before storing
pub async fn store_block_if_new(block: &Block) -> Result<bool> {
    let hash_hex = hex::encode(block.hash);
    let block_path = storage_dir.join(format!("{}.block", hash_hex));

    if block_path.exists() {
        // Block already exists, skip storage
        return Ok(false);
    }

    // Store new block
    fs::write(block_path, &block.content).await?;
    Ok(true)
}
```

## Forward Error Correction

### Overview

**Reed-Solomon FEC** provides fault tolerance by adding redundancy shards that allow data recovery even if some shards are lost or corrupted.

**File**: `communitas-core/src/storage/reed_solomon_manager.rs`

### FEC Configuration

```rust
pub struct ReedSolomonConfig {
    /// Number of data shards (k)
    pub data_shards: usize,

    /// Number of parity shards (m)
    pub parity_shards: usize,

    /// Bytes per shard
    pub shard_size: usize,

    /// Group size range for this config
    pub group_size_range: (usize, usize),
}

impl ReedSolomonConfig {
    /// Select optimal config based on group size
    pub fn for_group_size(member_count: usize) -> Self {
        match member_count {
            1..=5 => Self {
                data_shards: 3,
                parity_shards: 2,
                shard_size: 4096,
                group_size_range: (1, 5),
            },
            6..=15 => Self {
                data_shards: 8,
                parity_shards: 4,
                shard_size: 4096,
                group_size_range: (6, 15),
            },
            16..=50 => Self {
                data_shards: 12,
                parity_shards: 6,
                shard_size: 8192,
                group_size_range: (16, 50),
            },
            _ => Self {
                data_shards: 16,
                parity_shards: 8,
                shard_size: 8192,
                group_size_range: (51, usize::MAX),
            },
        }
    }

    /// Total shards (k + m)
    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }

    /// Number of shards that can be lost
    pub fn can_lose_members(&self) -> usize {
        self.parity_shards
    }

    /// Redundancy factor
    pub fn redundancy_factor(&self) -> f32 {
        (self.total_shards() as f32) / (self.data_shards as f32)
    }
}
```

### Shard Structure

```rust
pub struct Shard {
    /// Shard index
    pub index: usize,

    /// Shard type (Data or Parity)
    pub shard_type: ShardType,

    /// Shard data
    pub data: Vec<u8>,

    /// Group ID
    pub group_id: String,

    /// Data ID
    pub data_id: String,

    /// BLAKE3 integrity hash
    pub integrity_hash: String,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Shard size
    pub size: usize,
}

pub enum ShardType {
    Data,   // Original data shard (k needed)
    Parity, // Redundancy shard (m for fault tolerance)
}
```

### Encoding with FEC

```rust
/// Encode data with Reed-Solomon FEC
pub async fn store_with_fec(
    &self,
    key: &str,
    data: &[u8],
    redundancy: f32,
) -> Result<Vec<PathBuf>> {
    // Encrypt data first
    let encrypted = self.key_manager.encrypt(&self.encryption_key, data)?;

    // Get FEC configuration
    let config = ReedSolomonConfig::for_group_size(
        calculate_group_size(redundancy)
    );

    // Create FEC encoder
    let encoder = FecCodec::new(
        config.data_shards,
        config.parity_shards,
        config.shard_size,
    )?;

    // Encode into shards
    let shards = encoder.encode(&encrypted)?;

    // Store shards
    let mut shard_paths = Vec::new();
    for (i, shard_data) in shards.iter().enumerate() {
        let shard = Shard {
            index: i,
            shard_type: if i < config.data_shards {
                ShardType::Data
            } else {
                ShardType::Parity
            },
            data: shard_data.clone(),
            group_id: self.four_words.clone(),
            data_id: key.to_string(),
            integrity_hash: hex::encode(blake3::hash(shard_data).as_bytes()),
            created_at: chrono::Utc::now(),
            size: shard_data.len(),
        };

        let shard_path = self.vault_path.join(format!("{}.shard.{}", key, i));
        fs::write(&shard_path, serde_json::to_vec(&shard)?).await?;
        shard_paths.push(shard_path);
    }

    Ok(shard_paths)
}
```

### Decoding from FEC

```rust
/// Retrieve data from FEC shards (tolerates missing shards)
pub async fn retrieve_from_fec(
    &self,
    shard_paths: &[PathBuf],
) -> Result<Vec<u8>> {
    // Load available shards
    let mut shards = Vec::new();
    for path in shard_paths {
        if let Ok(shard_data) = fs::read(path).await {
            if let Ok(shard) = serde_json::from_slice::<Shard>(&shard_data) {
                // Verify shard integrity
                let computed_hash = hex::encode(blake3::hash(&shard.data).as_bytes());
                if computed_hash == shard.integrity_hash {
                    shards.push((shard.index, shard.data));
                }
            }
        }
    }

    // Need at least k shards to reconstruct
    let config = self.get_fec_config();
    if shards.len() < config.data_shards {
        return Err(anyhow!(
            "Insufficient shards: need {}, have {}",
            config.data_shards,
            shards.len()
        ));
    }

    // Create FEC decoder
    let decoder = FecCodec::new(
        config.data_shards,
        config.parity_shards,
        config.shard_size,
    )?;

    // Reconstruct data from available shards
    let reconstructed = decoder.decode(&shards)?;

    Ok(reconstructed)
}
```

### FEC Benefits

- **Fault Tolerance**: Survive loss of up to `m` shards
- **Member Churn**: Groups tolerate members going offline
- **Network Resilience**: Partial data recovery from any `k` of `k+m` shards
- **Adaptive**: Configuration scales with group size

### FEC Overhead

| Group Size | Config | Redundancy | Can Lose |
|------------|--------|------------|----------|
| 1-5        | 3+2    | 1.67x      | 2 members|
| 6-15       | 8+4    | 1.50x      | 4 members|
| 16-50      | 12+6   | 1.50x      | 6 members|
| 51+        | 16+8   | 1.50x      | 8 members|

## Website Publishing (Saorsa Sites)

### Overview

**Saorsa Sites** enables DNS-free website publishing using ML-DSA signed manifests and content-addressed blocks distributed over the P2P network.

**File**: `communitas-core/src/gossip/sites.rs`

**Specification**: SPEC2.md §5

### Site Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Site Manifest                          │
│              (ML-DSA signed, version tracked)               │
│  - Site ID (ML-DSA public key)                             │
│  - Block map (path → BLAKE3 hash)                          │
│  - Root hash (BLAKE3 of all blocks)                        │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                  Content Blocks (512KB max)                 │
│  /index.html → [hash: abc123...]                           │
│  /style.css  → [hash: def456...]                           │
│  /app.js     → [hash: ghi789...]                           │
│  /logo.png   → [hash: jkl012...]                           │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              Rendezvous Discovery (65k shards)              │
│        SITE_ADVERT shard for Provider Summaries            │
│  - Provider peer IDs                                        │
│  - Address hints                                            │
│  - NAT class                                                │
└─────────────────────────────────────────────────────────────┘
```

### Site Manifest

```rust
pub struct SiteManifest {
    /// Protocol version
    pub version: u8,

    /// Site identifier (ML-DSA public key)
    pub site_id: SiteId,

    /// Manifest version (incrementing)
    pub manifest_version: u64,

    /// Root hash (BLAKE3 of all block hashes)
    pub root_hash: [u8; 32],

    /// Block map: path → block_hash
    pub blocks: Vec<(String, [u8; 32])>,

    /// ML-DSA signature
    pub signature: Vec<u8>,
}
```

### Publishing a Site

```rust
/// Publish a website
pub async fn publish_site(
    site_id: SiteId,
    content: HashMap<String, Vec<u8>>,
) -> Result<SiteManifest> {
    // Step 1: Chunk content into blocks
    let mut blocks = Vec::new();
    let mut block_map = Vec::new();

    for (path, data) in content {
        let chunks = chunk_content(&data, MAX_BLOCK_SIZE);

        for (i, block) in chunks.iter().enumerate() {
            let block_path = if chunks.len() > 1 {
                format!("{}.part{}", path, i)
            } else {
                path.clone()
            };

            block_map.push((block_path, block.hash));
            blocks.push(block.clone());
        }
    }

    // Step 2: Create manifest
    let manifest = SiteManifest::new(site_id, 1, block_map);

    // Step 3: Sign manifest with ML-DSA
    let signed_manifest = sign_manifest(manifest, &identity_key)?;

    // Step 4: Store blocks locally
    for block in &blocks {
        store_block_if_new(block).await?;
    }

    // Step 5: Publish ProviderSummary to SITE_ADVERT shard
    let summary = ProviderSummary {
        target: site_id.as_bytes().clone(),
        provider: our_peer_id,
        addr_hints: our_addresses,
        nat_class: detected_nat_class,
        timestamp_ms: current_timestamp_ms(),
    };

    rendezvous.publish_provider_summary(summary).await?;

    Ok(signed_manifest)
}
```

### Fetching a Site

```rust
/// Fetch a website
pub async fn fetch_site(site_id: SiteId) -> Result<HashMap<String, Vec<u8>>> {
    // Step 1: Subscribe to SITE_ADVERT shard
    let shard = rendezvous.subscribe_to_shard(site_id.as_bytes()).await?;

    // Step 2: Collect provider summaries
    let providers = rendezvous.collect_providers(site_id.as_bytes(), 10).await?;

    if providers.is_empty() {
        return Err(anyhow!("No providers found for site"));
    }

    // Step 3: Score and select best provider
    let best_provider = score_providers(&providers)?;

    // Step 4: Fetch manifest
    let request = SiteRequest::GetManifest { site_id };
    let response = send_site_request(&best_provider, request).await?;

    let manifest = match response {
        SiteResponse::Manifest(m) => m,
        _ => return Err(anyhow!("Expected manifest response")),
    };

    // Step 5: Verify ML-DSA signature
    verify_manifest_signature(&manifest)?;

    // Step 6: Fetch blocks
    let mut content = HashMap::new();

    for (path, block_hash) in &manifest.blocks {
        let request = SiteRequest::GetBlock { hash: *block_hash };
        let response = send_site_request(&best_provider, request).await?;

        let block = match response {
            SiteResponse::Block(b) => b,
            _ => return Err(anyhow!("Expected block response")),
        };

        // Verify block integrity
        if !block.verify() {
            return Err(anyhow!("Block verification failed for {}", path));
        }

        content.insert(path.clone(), block.content);
    }

    // Step 7: Verify root hash
    verify_blocks(&manifest, &content.values().collect::<Vec<_>>())?;

    Ok(content)
}
```

### Private Sites

Sites can be encrypted for group-only access:

```rust
/// Publish a private site encrypted with MLS group key
pub async fn publish_private_site(
    site_id: SiteId,
    group_id: &str,
    content: HashMap<String, Vec<u8>>,
) -> Result<SiteManifest> {
    // Get group encryption key
    let group_key = group_context.get_shared_key(group_id).await?;

    // Encrypt all content with group key
    let encrypted_content: HashMap<String, Vec<u8>> = content
        .into_iter()
        .map(|(path, data)| {
            let encrypted = encrypt_with_group_key(&data, &group_key)?;
            Ok((path, encrypted))
        })
        .collect::<Result<_>>()?;

    // Publish encrypted site
    publish_site(site_id, encrypted_content).await
}
```

## Storage Policies

### Access Control

```rust
pub enum AccessPolicy {
    /// Owner only
    Private,

    /// Anyone can read
    Public,

    /// Group members only
    GroupRestricted(String), // group_id

    /// Specific users
    UserRestricted(Vec<String>), // four_words addresses
}
```

### Encryption Policies

```rust
pub enum EncryptionPolicy {
    /// No encryption (public content)
    None,

    /// Encrypted with user's password-derived key
    UserKey,

    /// Encrypted with group shared key
    GroupKey(String), // group_id

    /// Encrypted with specific recipient keys
    RecipientKeys(Vec<String>), // four_words addresses
}
```

### Replication Strategies

```rust
pub enum ReplicationStrategy {
    /// No replication (local only)
    LocalOnly,

    /// Replicate to all group members
    AllMembers,

    /// Replicate to N random members
    RandomSubset(usize),

    /// Replicate to favourite contacts
    Favourites,

    /// Content-addressed replication (on-demand)
    ContentAddressed,
}
```

### Storage Policy Configuration

```rust
pub struct StoragePolicy {
    /// Access control
    pub access: AccessPolicy,

    /// Encryption policy
    pub encryption: EncryptionPolicy,

    /// Replication strategy
    pub replication: ReplicationStrategy,

    /// Enable FEC
    pub use_fec: bool,

    /// FEC redundancy factor
    pub fec_redundancy: f32,

    /// Time-to-live (0 = permanent)
    pub ttl_secs: u64,
}
```

### Default Policies by Disk Type

| Disk Type | Access | Encryption | Replication | FEC |
|-----------|--------|------------|-------------|-----|
| Private   | Private | UserKey | LocalOnly | Optional |
| Public    | Public | None | ContentAddressed | No |
| Shared    | GroupRestricted | GroupKey | AllMembers | Yes |

## Platform Integration

### Keyring Integration

**File**: `communitas-core/src/encrypted_storage/platform_storage.rs`

#### macOS (Keychain)

```rust
#[cfg(target_os = "macos")]
pub fn store_key(service: &str, account: &str, password: &str) -> Result<()> {
    use security_framework::keychain::{SecKeychain, SecKeychainItem};

    let keychain = SecKeychain::default()?;

    keychain.set_generic_password(
        service,
        account,
        password.as_bytes(),
    )?;

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn get_key(service: &str, account: &str) -> Result<String> {
    use security_framework::keychain::SecKeychain;

    let keychain = SecKeychain::default()?;

    let (password, _) = keychain.find_generic_password(service, account)?;

    Ok(String::from_utf8(password.to_vec())?)
}
```

#### Windows (DPAPI)

```rust
#[cfg(target_os = "windows")]
pub fn store_key(service: &str, account: &str, password: &str) -> Result<()> {
    use windows::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTOAPI_BLOB,
    };

    let key_name = format!("{}:{}", service, account);
    let encrypted = protect_data(password.as_bytes())?;

    // Store in registry or credential manager
    store_encrypted_credential(&key_name, &encrypted)?;

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn get_key(service: &str, account: &str) -> Result<String> {
    use windows::Win32::Security::Cryptography::CryptUnprotectData;

    let key_name = format!("{}:{}", service, account);
    let encrypted = load_encrypted_credential(&key_name)?;
    let decrypted = unprotect_data(&encrypted)?;

    Ok(String::from_utf8(decrypted)?)
}
```

#### Linux (Secret Service)

```rust
#[cfg(target_os = "linux")]
pub fn store_key(service: &str, account: &str, password: &str) -> Result<()> {
    use secret_service::{SecretService, EncryptionType};

    let ss = SecretService::new(EncryptionType::Dh)?;
    let collection = ss.get_default_collection()?;

    collection.create_item(
        &format!("{} - {}", service, account),
        vec![("service", service), ("account", account)],
        password.as_bytes(),
        true, // replace existing
        "text/plain",
    )?;

    Ok(())
}

#[cfg(target_os = "linux")]
pub fn get_key(service: &str, account: &str) -> Result<String> {
    use secret_service::{SecretService, EncryptionType};

    let ss = SecretService::new(EncryptionType::Dh)?;
    let collection = ss.get_default_collection()?;

    let items = collection.search_items(vec![
        ("service", service),
        ("account", account),
    ])?;

    if let Some(item) = items.first() {
        let secret = item.get_secret()?;
        return Ok(String::from_utf8(secret)?);
    }

    Err(anyhow!("Key not found"))
}
```

### Filesystem Storage

```
~/.communitas/
├── vaults/
│   ├── ocean-forest-moon-star/
│   │   ├── vault.meta (unencrypted metadata)
│   │   ├── password.verifier (encrypted verifier)
│   │   ├── identity.enc (encrypted identity)
│   │   ├── index.enc (encrypted index)
│   │   ├── data-key.enc (encrypted data file)
│   │   ├── data-key.shard.0 (FEC shard)
│   │   ├── data-key.shard.1 (FEC shard)
│   │   └── ...
│   └── river-mountain-sun-cloud/
│       └── ...
├── blocks/
│   ├── abc123...def.block (content-addressed block)
│   └── ...
├── cache/
│   └── crdt/ (CRDT document cache)
└── config/
    ├── app.config (application configuration)
    └── passkeys/ (WebAuthn passkey storage)
```

## Performance Characteristics

### Encryption Performance

- **ChaCha20-Poly1305**: 3-4 GB/s on modern CPUs (single-core)
- **BLAKE3 hashing**: 10+ GB/s on modern CPUs
- **PBKDF2 (100k iterations)**: ~50-100ms per key derivation

### Storage Operations

| Operation | Latency | Notes |
|-----------|---------|-------|
| Encrypt | <1ms per KB | ChaCha20-Poly1305 |
| Decrypt | <1ms per KB | ChaCha20-Poly1305 |
| Hash | <0.1ms per KB | BLAKE3 |
| Store (vault) | 10-50ms | Includes disk write |
| Retrieve (vault) | 5-20ms | Includes disk read |
| FEC encode | 5-10ms per MB | Depends on config |
| FEC decode | 5-10ms per MB | Depends on config |

### Scalability

- **Vault size**: Unlimited (tested to 100GB)
- **Entries per vault**: Millions supported
- **Block size**: 512KB maximum
- **FEC shards**: 3-24 shards typical
- **Concurrent vaults**: Limited by system resources

## Security Considerations

### Threat Model

**Protected Against**:
- ✅ Password guessing (PBKDF2 with 100k iterations)
- ✅ Ciphertext tampering (AEAD authentication)
- ✅ Data corruption (BLAKE3 integrity checks)
- ✅ Shard loss (FEC redundancy)
- ✅ Man-in-the-middle (content addressing)

**Not Protected Against**:
- ❌ Physical device compromise (encrypt device storage)
- ❌ Weak passwords (use strong passwords or passkeys)
- ❌ Memory dumps (use encrypted swap)

### Best Practices

1. **Strong Passwords**: Minimum 12 characters, mixed case, numbers, symbols
2. **Passkeys**: Use WebAuthn/FIDO2 for stronger authentication
3. **Device Encryption**: Enable full-disk encryption (FileVault, BitLocker, LUKS)
4. **Regular Backups**: Export vaults periodically
5. **Key Rotation**: Update passwords every 6-12 months
6. **Secure Delete**: Overwrite sensitive files before deletion

## Future Enhancements

### Planned Features

1. **Compression**: Zstd compression for storage efficiency
2. **Tiered Storage**: Hot/warm/cold data policies
3. **Cloud Backup**: Optional encrypted backup to S3/B2
4. **Key Escrow**: Optional key recovery via threshold signatures
5. **Search**: Encrypted search indices
6. **Versioning**: Git-like content versioning
7. **Snapshots**: Point-in-time vault snapshots

### Research Directions

1. **Homomorphic Encryption**: Compute on encrypted data
2. **Zero-Knowledge Proofs**: Prove access without revealing data
3. **Quantum-Resistant Storage**: Post-quantum encryption schemes
4. **Distributed Storage**: Erasure-coded storage across peers

## References

### Specifications

- **DESIGN.md**: Cryptographic design decisions
- **SPEC2.md**: Saorsa Sites and content addressing
- **RFC 8439**: ChaCha20-Poly1305 AEAD
- **RFC 2898**: PBKDF2 key derivation

### Dependencies

- **chacha20poly1305**: Authenticated encryption
- **blake3**: Fast cryptographic hashing
- **pbkdf2**: Password-based key derivation
- **reed-solomon**: Error correction codes
- **keyring**: Platform keyring integration
- **libSQL**: SQL database (Turso embedded)

### Related Documentation

- [Security](security.md) - Cryptography and security model
- [CRDT System](crdt-system.md) - Real-time collaborative documents
- [Architecture Overview](README.md) - System component overview
- [Architecture README](README.md) - Architecture overview

---

**Last Updated**: 2025-10-15
**Maintained By**: Saorsa Labs
**License**: GPL-3.0

# ADR-011: Encrypted Vault Storage

## Status

Accepted (2025-12-24)

## Context

### The Problem

A collaboration platform handling sensitive data needs robust local encryption:

- **Multi-account support**: Users may have multiple identities on one device
- **Password-only login**: Users want convenience after initial setup
- **Offline security**: Data must be protected even when device is compromised
- **Platform integration**: Should leverage OS-specific secure storage
- **Data resilience**: Critical data needs protection against corruption

Traditional approaches have limitations:

| Approach | Problem |
|----------|---------|
| Full disk encryption | User-level, not app-level isolation |
| Browser localStorage | Not encrypted by default |
| Cloud sync | Vendor dependency, privacy concerns |
| Single password file | No multi-account, single point of failure |

### Requirements

- Per-identity encrypted vaults
- Strong key derivation (PBKDF2)
- Modern authenticated encryption (ChaCha20-Poly1305)
- Platform keyring integration
- Forward Error Correction for resilience
- Passkey/WebAuthn support

## Decision

Implement a **multi-layered encrypted vault system** with platform-native security:

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Encrypted Storage Architecture                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Layer 1: Authentication                                            │
│  ──────────────────────────                                         │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Four-Word Address │ Password │ Passkey/WebAuthn │ Touch ID   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  Layer 2: Key Derivation (PBKDF2)                                  │
│  ────────────────────────────────                                   │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ PBKDF2-HMAC-SHA256 │ 100,000 iterations │ 32-byte salt       │  │
│  │                    │                    │                     │  │
│  │ password + salt ──────────────────────► 256-bit master key   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  Layer 3: Symmetric Encryption (ChaCha20-Poly1305)                 │
│  ─────────────────────────────────────────────────                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ ChaCha20-Poly1305 │ 96-bit nonce │ AEAD (authenticated)      │  │
│  │                   │              │                            │  │
│  │ nonce + ciphertext + tag (16 bytes) = encrypted data         │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  Layer 4: Forward Error Correction (Optional)                      │
│  ────────────────────────────────────────────                       │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ Reed-Solomon coding │ Data shards │ Parity shards            │  │
│  │                     │             │                           │  │
│  │ 1.5x redundancy = recover from 33% data loss                 │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  Layer 5: Platform Storage                                         │
│  ─────────────────────────                                          │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │ macOS: Keychain    │ Windows: DPAPI  │ Linux: Secret Service │  │
│  │                    │                 │                        │  │
│  │ Platform-native secure credential storage                    │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Vault Structure

Each identity (pubkey_hex) gets its own encrypted vault:

```rust
pub struct EncryptedVault {
    pub four_words: String,
    pub display_name: String,
    metadata: VaultMetadata,
    encryption_key: Zeroizing<Vec<u8>>,  // Auto-zeroed on drop
    data_store: RwLock<HashMap<String, EncryptedEntry>>,
    vault_path: PathBuf,
    key_manager: KeyManager,
    fec_storage: Option<FecStorage>,
}

pub struct VaultMetadata {
    pub version: u32,
    pub created_at: u64,
    pub last_accessed: u64,
    pub salt: Vec<u8>,           // Per-vault unique salt
    pub pbkdf2_iterations: u32,  // 100,000 default
    pub total_size: u64,
    pub entry_count: usize,
    pub checksum: Vec<u8>,       // BLAKE3 integrity hash
    pub display_name: String,
}
```

### Directory Layout

```
~/.communitas/vaults/
├── pubkey_hex_.../                  # Vault per identity
│   ├── vault.meta                   # Unencrypted metadata
│   ├── password.verifier            # Encrypted verifier for empty vaults
│   ├── identity.enc                 # Encrypted identity data
│   ├── index.enc                    # Encrypted file index
│   ├── *.enc                        # Individual encrypted files
│   └── fec/                         # FEC shards (if enabled)
│       ├── file1_shard_0.rs
│       ├── file1_shard_1.rs
│       └── ...
├── river-mountain-sun-cloud/        # Another identity's vault
│   └── ...
└── locators/
    └── password_locators.enc        # Password-only login mapping
```

### Key Derivation

PBKDF2 with high iteration count ensures brute-force resistance:

```rust
impl KeyManager {
    pub async fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        tokio::task::spawn_blocking(move || {
            let mut key = Zeroizing::new(vec![0u8; 32]);

            // PBKDF2-HMAC-SHA256, 100,000 iterations
            pbkdf2_hmac::<Sha256>(
                password.as_bytes(),
                &salt,
                100_000,  // Iterations as per design
                &mut key
            );

            Ok(key)
        }).await?
    }
}
```

**Security parameters**:

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| Iterations | 100,000 | OWASP 2023 recommendation |
| Salt size | 32 bytes | Unique per vault |
| Key size | 256 bits | ChaCha20 requirement |
| Hash function | SHA-256 | FIPS-compliant |

### Encryption/Decryption

ChaCha20-Poly1305 AEAD provides confidentiality and integrity:

```rust
pub fn encrypt(&self, key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(&Key::from_slice(key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, plaintext)?;

    // Format: nonce (12 bytes) || ciphertext || tag (16 bytes)
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt(&self, key: &[u8], data: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    let (nonce, ciphertext) = data.split_at(12);
    let cipher = ChaCha20Poly1305::new(&Key::from_slice(key));

    // AEAD decryption fails with wrong key or tampered data
    let plaintext = cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;

    Ok(Zeroizing::new(plaintext))
}
```

**Why ChaCha20-Poly1305 over AES-GCM?**:
- Faster on devices without AES hardware acceleration
- Simpler implementation, fewer side-channel risks
- Same security level (256-bit)
- Used by WireGuard, TLS 1.3

### Password-Only Login

Users can log in with just a password (no connection words required):

```rust
// Store password hash → identity mapping
pub async fn store_password_locator(
    &self,
    password_hash: &[u8],  // BLAKE3 hash
    four_words: &str,
) -> Result<()>;

// Find vault by password
pub async fn login_password_only(&self, password: &str) -> Result<Session> {
    let password_hash = self.key_manager.hash_password(password).await?;
    let four_words = self.platform_storage
        .find_vault_by_password_hash(&password_hash)
        .await?;

    self.login(&four_words, password, None).await
}
```

### Platform Keyring Integration

Leverage OS-native secure storage:

```rust
impl KeyManager {
    pub async fn store_in_keyring(&self, four_words: &str, key: &[u8]) -> Result<()> {
        let entry = Entry::new("com.saorsalabs.communitas", four_words)?;
        entry.set_password(&base64::encode(key))?;
        Ok(())
    }

    pub async fn get_from_keyring(&self, four_words: &str) -> Result<Zeroizing<Vec<u8>>> {
        let entry = Entry::new("com.saorsalabs.communitas", four_words)?;
        let key_b64 = entry.get_password()?;
        Ok(Zeroizing::new(base64::decode(key_b64)?))
    }
}
```

| Platform | Backend | Security |
|----------|---------|----------|
| macOS | Keychain Services | Hardware-backed on Apple Silicon |
| Windows | Credential Manager (DPAPI) | User session isolation |
| Linux | Secret Service (libsecret) | Desktop keyring integration |

### Passkey/WebAuthn Support

Biometric authentication for passwordless login:

```rust
pub async fn passkey_authenticate(&self, four_words: &str) -> Result<Session> {
    // 1. Verify passkey is registered
    if !self.passkey_manager.has_passkey(&four_words).await {
        bail!("No passkey registered");
    }

    // 2. Retrieve password from keyring (stored during initial login)
    let password = self.key_manager.get_from_keyring(&four_words).await?;

    // 3. Login with stored password
    self.login(&four_words, &password, None).await
}
```

### Forward Error Correction

Reed-Solomon coding protects critical data:

```rust
pub async fn store_with_fec(
    &self,
    key: &str,
    data: &[u8],
    redundancy: f32,  // e.g., 1.5 = 50% extra
) -> Result<()> {
    // Encrypt first
    let encrypted = self.key_manager.encrypt(&self.encryption_key, data)?;

    // Create FEC shards
    let shard_paths = self.fec_storage.store_with_fec(
        key,
        &encrypted,
        redundancy
    ).await?;

    // Store shard locations in metadata
    // ...
}
```

| Redundancy | Recovery Capability |
|------------|---------------------|
| 1.5x | 33% data loss |
| 2.0x | 50% data loss |
| 3.0x | 66% data loss |

### Session Management

Sessions provide authenticated access with timeout:

```rust
pub struct Session {
    pub id: String,           // UUID
    pub four_words: String,
    pub display_name: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub is_active: bool,
}

impl Session {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }
}
```

### Security Properties

| Property | Implementation |
|----------|----------------|
| Key zeroization | `zeroize::Zeroizing<Vec<u8>>` |
| Password validation | AEAD decryption (wrong key → auth fail) |
| Memory safety | Rust ownership, no buffer overflows |
| Side-channel resistance | Constant-time operations |
| File permissions | `chmod 600` on Unix |

## Consequences

### Benefits

- **Multi-identity**: Each identity gets isolated vault
- **Strong encryption**: ChaCha20-Poly1305 AEAD
- **Key stretching**: 100,000 PBKDF2 iterations
- **Platform security**: Native keyring integration
- **Data resilience**: Optional FEC for important data
- **Passwordless**: Touch ID/Face ID via passkeys

### Trade-offs

- **Performance**: PBKDF2 adds ~100ms to login
- **Storage overhead**: FEC adds 50-200% per file
- **Complexity**: Multi-layer system requires careful handling
- **Platform dependency**: Keyring behavior varies by OS

### Password Verifier Security

Empty vaults store a password verifier to prevent "phantom login":

```rust
// On vault creation
let verifier_data = b"communitas:password:verifier:v1";
let encrypted_verifier = key_manager.encrypt(&encryption_key, verifier_data)?;
fs::write(vault_path.join("password.verifier"), encrypted_verifier).await?;

// On vault load (even if empty)
let encrypted_verifier = fs::read(verifier_path).await?;
key_manager.decrypt(&encryption_key, &encrypted_verifier)
    .context("Invalid password")?;  // Fails with wrong password
```

## Alternatives Considered

1. **SQLCipher**: Encrypted SQLite database
   - Rejected: Adds SQLite dependency, less flexible

2. **Age encryption**: Modern file encryption
   - Rejected: File-based, no session/multi-account

3. **Browser Web Crypto API**: For web builds
   - Rejected: No platform keyring, limited key storage

4. **Hardware security modules**: HSM-based key storage
   - Rejected: Not available on consumer devices

5. **Password manager integration**: 1Password/Bitwarden
   - Rejected: External dependency, not self-contained

## References

- Implementation: `communitas-core/src/encrypted_storage/`
- Key Management: `communitas-core/src/encrypted_storage/key_management.rs`
- Vault: `communitas-core/src/encrypted_storage/vault.rs`
- Platform Storage: `communitas-core/src/encrypted_storage/platform_storage.rs`
- Related ADR: [ADR-001 Four-Word Identity](ADR-001-four-word-identity-system.md) (superseded)
- Related ADR: [ADR-006 Post-Quantum Cryptography](ADR-006-post-quantum-cryptography.md)

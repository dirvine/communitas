# ADR-016: Identity Recovery System

## Status

Proposed (2025-01-15)

## Context

### The Problem

In Communitas, a user's identity is anchored to their ML-DSA-65 public key. The corresponding secret key (4,032 bytes) is required to:

- Sign messages proving identity ownership
- Decrypt messages sent to the user
- Authenticate to contacts in the F2F network
- Retrieve distributed data from the network

**If a user loses their secret key, they permanently lose:**
- Their identity and reputation
- Access to all their data distributed across contacts
- All contact relationships and group memberships
- Any value or reputation accrued to that identity

This is catastrophic for users and unacceptable for mainstream adoption.

### Current State

The existing system (ADR-011, ADR-012) provides:
- Local encrypted vault storage with PBKDF2 key derivation
- Password-based daily login
- Platform keyring integration for convenience
- Passkey/biometric authentication

**However, there is no recovery mechanism if:**
- The device is lost, stolen, or destroyed
- The vault becomes corrupted
- The user forgets their password on a device without passkey

### Requirements

1. **Self-Sovereign Recovery**: Users must be able to recover without depending on any central authority
2. **Deterministic Key Generation**: Same recovery input must always produce same keys
3. **Post-Quantum Safety**: Recovery mechanism must not weaken PQC security
4. **User-Friendly**: Non-technical users must be able to use it
5. **Offline Capable**: Primary recovery must work without network access
6. **Resistant to AI Attacks**: Must consider deepfake and social engineering vectors
7. **Optional Social Recovery**: For users who cannot manage their own backups

### Threat Model

| Threat | Severity | Mitigation Required |
|--------|----------|---------------------|
| Device loss/theft | High | Backup recovery method |
| Forgotten password | Medium | Recovery bypasses password |
| Physical backup theft | High | Optional passphrase encryption |
| AI-generated deepfakes | Critical | Multi-factor verification |
| Social engineering | High | Time-locks, cancellation |
| Coercion/extortion | Medium | Plausible deniability options |
| Backup destruction (fire, flood) | Medium | Geographic distribution |

## Decision

Implement a **hybrid recovery system** with BIP39 mnemonic as the primary method and optional encrypted social recovery as a secondary backup.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     IDENTITY RECOVERY ARCHITECTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  IDENTITY CREATION                                                          │
│  ═════════════════                                                          │
│                                                                             │
│  ┌────────────────┐                                                         │
│  │ FIPS-compliant │                                                         │
│  │ RNG (256-bit)  │                                                         │
│  └───────┬────────┘                                                         │
│          │                                                                  │
│          ▼                                                                  │
│  ┌────────────────┐     ┌──────────────────────────────────────────────┐   │
│  │ 256-bit        │────►│ BIP39 Mnemonic (24 words)                    │   │
│  │ Entropy        │     │ "witch collapse practice feed shame open..." │   │
│  └────────────────┘     └──────────────────────────────────────────────┘   │
│                                        │                                    │
│                                        ▼                                    │
│                         ┌──────────────────────────────┐                    │
│                         │ PBKDF2-HMAC-SHA512           │                    │
│                         │ 2048 iterations              │                    │
│                         │ salt: "mnemonic" + passphrase│                    │
│                         └──────────────────────────────┘                    │
│                                        │                                    │
│                                        ▼                                    │
│                         ┌──────────────────────────────┐                    │
│                         │ 64-byte Master Seed          │                    │
│                         └──────────────────────────────┘                    │
│                                        │                                    │
│                    ┌───────────────────┴───────────────────┐                │
│                    ▼                                       ▼                │
│           ┌──────────────┐                       ┌──────────────┐           │
│           │ ML-DSA-65    │                       │ ML-KEM-768   │           │
│           │ Signing Key  │                       │ KEM Key      │           │
│           └──────────────┘                       └──────────────┘           │
│                                                                             │
│  RECOVERY PATHS                                                             │
│  ═════════════                                                              │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ PRIMARY: BIP39 Mnemonic Recovery                                    │   │
│  │ ────────────────────────────────                                    │   │
│  │ • User enters 24 words on new device                                │   │
│  │ • Deterministically regenerates all keys                            │   │
│  │ • No network required                                               │   │
│  │ • Full self-sovereignty                                             │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ SECONDARY: Encrypted Social Recovery (Optional)                     │   │
│  │ ───────────────────────────────────────────────                     │   │
│  │ • Mnemonic encrypted with recovery password                         │   │
│  │ • Split via Shamir's Secret Sharing (k-of-n)                        │   │
│  │ • Shards distributed to trusted contacts                            │   │
│  │ • 7-day time-lock with cancellation                                 │   │
│  │ • Multi-factor verification required                                │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Part 1: BIP39 Mnemonic Recovery (Primary)

#### Why BIP39?

| Consideration | BIP39 Advantage |
|---------------|-----------------|
| **Proven Security** | Secures hundreds of billions in cryptocurrency |
| **Standardized** | BIP39 is a well-audited, widely-implemented standard |
| **Human-Readable** | 24 English words are memorable and writable |
| **Error Detection** | Built-in checksum catches transcription errors |
| **Language Support** | Standard supports multiple languages |
| **Ecosystem** | Compatible with hardware wallets, backup tools |

#### Mnemonic Generation

```rust
/// Recovery phrase configuration
pub struct RecoveryConfig {
    /// Number of mnemonic words (24 for 256-bit security)
    pub word_count: usize,
    /// BIP39 language
    pub language: Language,
    /// Optional additional passphrase (25th word)
    pub use_passphrase: bool,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            word_count: 24,  // 256 bits of entropy
            language: Language::English,
            use_passphrase: false,
        }
    }
}

/// Generate a new BIP39 mnemonic for identity creation
pub fn generate_recovery_mnemonic(
    config: &RecoveryConfig
) -> Result<Mnemonic, RecoveryError> {
    // Calculate required entropy bytes
    // 24 words = 256 bits = 32 bytes
    let entropy_bytes = (config.word_count * 11 - config.word_count / 3) / 8;
    
    // Generate entropy using FIPS-compliant RNG
    let mut entropy = vec![0u8; entropy_bytes];
    let mut rng = FipsRng::new(SecurityStrength::Bits256)
        .map_err(|e| RecoveryError::EntropyGenerationFailed(e.to_string()))?;
    rng.fill_bytes(&mut entropy);
    
    // Create BIP39 mnemonic with checksum
    let mnemonic = Mnemonic::from_entropy_in(config.language, &entropy)
        .map_err(|e| RecoveryError::MnemonicGenerationFailed(e.to_string()))?;
    
    // Zeroize entropy immediately
    entropy.zeroize();
    
    Ok(mnemonic)
}
```

#### Deterministic Key Derivation

```rust
/// Key derivation paths for different key types
/// Using BIP44-style derivation adapted for PQC
pub mod derivation_paths {
    /// Purpose: 44' (BIP44 standard)
    pub const PURPOSE: u32 = 44;
    /// Coin type: 0x434F4D (ASCII "COM" for Communitas)
    pub const COIN_TYPE: u32 = 0x434F4D;
    /// ML-DSA signing key account
    pub const MLDSA_ACCOUNT: u32 = 0;
    /// ML-KEM encryption key account
    pub const MLKEM_ACCOUNT: u32 = 1;
}

/// Derive all identity keys from a BIP39 mnemonic
pub fn derive_identity_keys(
    mnemonic: &Mnemonic,
    passphrase: Option<&str>,
) -> Result<IdentityKeys, RecoveryError> {
    // BIP39 seed derivation
    // PBKDF2-HMAC-SHA512, 2048 iterations
    // Salt: "mnemonic" + passphrase
    let seed = mnemonic.to_seed(passphrase.unwrap_or(""));
    
    // Derive master key using BLAKE3 with domain separation
    let master_key = blake3::derive_key(
        "communitas:identity:master:v1",
        &seed
    );
    
    // Derive ML-DSA-65 signing keypair
    let mldsa_seed = blake3::derive_key(
        "communitas:mldsa65:v1",
        &master_key
    );
    let mut mldsa_rng = ChaCha20Rng::from_seed(mldsa_seed);
    let (mldsa_pk, mldsa_sk) = ml_dsa_65::try_keygen_with_rng(&mut mldsa_rng)
        .map_err(|e| RecoveryError::KeyDerivationFailed(e.to_string()))?;
    
    // Derive ML-KEM-768 encryption keypair
    let mlkem_seed = blake3::derive_key(
        "communitas:mlkem768:v1", 
        &master_key
    );
    let mut mlkem_rng = ChaCha20Rng::from_seed(mlkem_seed);
    let (mlkem_pk, mlkem_sk) = ml_kem_768::try_keygen_with_rng(&mut mlkem_rng)
        .map_err(|e| RecoveryError::KeyDerivationFailed(e.to_string()))?;
    
    // Derive four-word identity from public key for human-friendly display
    let four_words = derive_four_words_from_pubkey(&mldsa_pk)?;

    Ok(IdentityKeys {
        four_words,
        mldsa_public: MlDsaPublicKey::from_bytes(&mldsa_pk.into_bytes())?,
        mldsa_secret: MlDsaSecretKey::from_bytes(&mldsa_sk.into_bytes())?,
        mlkem_public: MlKemPublicKey::from_bytes(&mlkem_pk.into_bytes())?,
        mlkem_secret: MlKemSecretKey::from_bytes(&mlkem_sk.into_bytes())?,
    })
}
```

#### Recovery Flow

```rust
/// Recover identity from BIP39 mnemonic
pub async fn recover_from_mnemonic(
    mnemonic_words: &str,
    passphrase: Option<&str>,
    new_vault_password: &str,
    config: &StorageConfig,
) -> Result<RecoveryResult, RecoveryError> {
    // Parse and validate mnemonic
    let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_words)
        .map_err(|e| RecoveryError::InvalidMnemonic(e.to_string()))?;
    
    // Verify checksum (BIP39 includes 8-bit checksum for 24 words)
    // This catches most transcription errors
    if !mnemonic.verify_checksum() {
        return Err(RecoveryError::ChecksumFailed);
    }
    
    // Derive identity keys
    let identity_keys = derive_identity_keys(&mnemonic, passphrase)?;
    
    // Check if vault already exists for this identity
    let vault_path = config.vault_dir.join(&identity_keys.four_words);
    if vault_path.exists() {
        return Err(RecoveryError::VaultAlreadyExists {
            four_words: identity_keys.four_words.clone(),
        });
    }
    
    // Create new encrypted vault
    let salt = KeyManager::generate_salt();
    let key_manager = KeyManager::new(config.pbkdf2_iterations, config.use_keyring).await?;
    let encryption_key = key_manager.derive_key(new_vault_password, &salt).await?;
    
    // Store identity keys in vault
    let vault = EncryptedVault::create(
        identity_keys.four_words.clone(),
        identity_keys.four_words.clone(), // Use four-words as initial display name
        encryption_key,
        salt,
        config,
    ).await?;
    
    // Store all key material
    vault.store("mldsa_secret", identity_keys.mldsa_secret.as_bytes()).await?;
    vault.store("mldsa_public", identity_keys.mldsa_public.as_bytes()).await?;
    vault.store("mlkem_secret", identity_keys.mlkem_secret.as_bytes()).await?;
    vault.store("mlkem_public", identity_keys.mlkem_public.as_bytes()).await?;
    
    Ok(RecoveryResult {
        four_words: identity_keys.four_words,
        public_key: identity_keys.mldsa_public,
        recovery_method: RecoveryMethod::Mnemonic,
    })
}
```

### Part 2: Social Recovery (Secondary/Optional)

#### Design Philosophy

Social recovery is designed as an **emergency backup** for users who:
- Cannot reliably store a physical backup
- Want redundancy in case of natural disaster
- Accept the trade-off of trusting contacts

**Critical Principle**: Social recovery is layered encryption—contacts never see the mnemonic.

#### Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     SOCIAL RECOVERY ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  SETUP PHASE                                                                │
│  ═══════════                                                                │
│                                                                             │
│  ┌────────────────┐                                                         │
│  │ BIP39 Mnemonic │                                                         │
│  │ (24 words)     │                                                         │
│  └───────┬────────┘                                                         │
│          │                                                                  │
│          ▼                                                                  │
│  ┌────────────────────────────────────────┐                                 │
│  │ Encrypt with Recovery Password         │                                 │
│  │ ─────────────────────────────────────  │                                 │
│  │ • PBKDF2-SHA256 (100,000 iterations)   │                                 │
│  │ • ChaCha20-Poly1305 AEAD               │                                 │
│  │ • Random 256-bit salt                  │                                 │
│  └────────────────────────────────────────┘                                 │
│          │                                                                  │
│          ▼                                                                  │
│  ┌────────────────────────────────────────┐                                 │
│  │ Shamir's Secret Sharing                │                                 │
│  │ ─────────────────────────────────────  │                                 │
│  │ • k-of-n threshold (e.g., 3-of-5)      │                                 │
│  │ • GF(256) polynomial interpolation     │                                 │
│  │ • Each shard ~same size as secret      │                                 │
│  └────────────────────────────────────────┘                                 │
│          │                                                                  │
│    ┌─────┴─────┬─────────┬─────────┬─────────┐                              │
│    ▼           ▼         ▼         ▼         ▼                              │
│ ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐                            │
│ │Shard │  │Shard │  │Shard │  │Shard │  │Shard │                            │
│ │  1   │  │  2   │  │  3   │  │  4   │  │  5   │                            │
│ └──┬───┘  └──┬───┘  └──┬───┘  └──┬───┘  └──┬───┘                            │
│    │         │         │         │         │                                │
│    ▼         ▼         ▼         ▼         ▼                                │
│ ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐                            │
│ │Encrypt│ │Encrypt│ │Encrypt│ │Encrypt│ │Encrypt│                           │
│ │for    │ │for    │ │for    │ │for    │ │for    │                           │
│ │Alice  │ │Bob    │ │Carol  │ │Dave   │ │Eve    │                           │
│ └──┬───┘  └──┬───┘  └──┬───┘  └──┬───┘  └──┬───┘                            │
│    │         │         │         │         │                                │
│    ▼         ▼         ▼         ▼         ▼                                │
│ ┌──────────────────────────────────────────────────────────────────────┐    │
│ │ Guardian Storage (in their local vaults)                             │    │
│ │ ──────────────────────────────────────────                           │    │
│ │ • Encrypted shard stored locally by each guardian                    │    │
│ │ • Guardian cannot decrypt (encrypted to owner's recovery key)        │    │
│ │ • Shard includes: encrypted_data, owner_pubkey, metadata             │    │
│ └──────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  RECOVERY PHASE                                                             │
│  ══════════════                                                             │
│                                                                             │
│  ┌────────────────────────────────────────┐                                 │
│  │ 1. User creates temporary identity     │                                 │
│  │    (24-hour expiration)                │                                 │
│  └────────────────────────────────────────┘                                 │
│                    │                                                        │
│                    ▼                                                        │
│  ┌────────────────────────────────────────┐                                 │
│  │ 2. Contact k guardians out-of-band     │                                 │
│  │    (phone call, in-person, etc.)       │                                 │
│  └────────────────────────────────────────┘                                 │
│                    │                                                        │
│                    ▼                                                        │
│  ┌────────────────────────────────────────┐                                 │
│  │ 3. Multi-factor verification           │                                 │
│  │    • Security questions                │                                 │
│  │    • Voice verification (known number) │                                 │
│  │    • In-person if possible             │                                 │
│  └────────────────────────────────────────┘                                 │
│                    │                                                        │
│                    ▼                                                        │
│  ┌────────────────────────────────────────┐                                 │
│  │ 4. Guardian initiates shard release    │                                 │
│  │    → 7-day time-lock begins            │                                 │
│  │    → Owner notified on all devices     │                                 │
│  │    → Can cancel if fraudulent          │                                 │
│  └────────────────────────────────────────┘                                 │
│                    │                                                        │
│                    ▼                                                        │
│  ┌────────────────────────────────────────┐                                 │
│  │ 5. After time-lock, collect k shards   │                                 │
│  │    → Reconstruct encrypted mnemonic    │                                 │
│  └────────────────────────────────────────┘                                 │
│                    │                                                        │
│                    ▼                                                        │
│  ┌────────────────────────────────────────┐                                 │
│  │ 6. User enters recovery password       │                                 │
│  │    → Decrypts mnemonic                 │                                 │
│  │    → Derives all keys                  │                                 │
│  └────────────────────────────────────────┘                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Social Recovery Data Structures

```rust
/// Social recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialRecoveryConfig {
    /// Minimum guardians required for recovery (k)
    pub threshold: usize,
    /// Total number of guardians (n)
    pub total_guardians: usize,
    /// Time-lock duration in seconds (default: 7 days)
    pub timelock_seconds: u64,
    /// Maximum recovery attempts before lockout
    pub max_attempts: usize,
    /// Lockout duration after max attempts (default: 30 days)
    pub lockout_seconds: u64,
}

impl Default for SocialRecoveryConfig {
    fn default() -> Self {
        Self {
            threshold: 3,
            total_guardians: 5,
            timelock_seconds: 7 * 24 * 60 * 60, // 7 days
            max_attempts: 3,
            lockout_seconds: 30 * 24 * 60 * 60, // 30 days
        }
    }
}

/// Guardian information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guardian {
    /// Guardian's public key (their Communitas identity)
    pub pubkey: Vec<u8>,
    /// Human-readable name
    pub display_name: String,
    /// Shard index (1..=n)
    pub shard_index: usize,
    /// Pre-shared security questions (hashed)
    pub security_question_hashes: Vec<Vec<u8>>,
    /// Guardian's known contact methods (phone, etc.)
    pub contact_methods: Vec<ContactMethod>,
    /// When this guardian was added
    pub added_at: u64,
}

/// Encrypted shard stored by guardian
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedShard {
    /// Version for forward compatibility
    pub version: u32,
    /// Owner's public key (who this shard belongs to)
    pub owner_pubkey: Vec<u8>,
    /// Owner's four-word identity
    pub owner_four_words: String,
    /// Shard index
    pub shard_index: usize,
    /// Total shards
    pub total_shards: usize,
    /// Threshold required
    pub threshold: usize,
    /// Encrypted shard data (ChaCha20-Poly1305)
    /// Encrypted with key derived from owner's recovery password
    pub encrypted_data: Vec<u8>,
    /// Salt used for shard encryption key derivation
    pub salt: Vec<u8>,
    /// When this shard was created
    pub created_at: u64,
    /// Signature by owner proving shard authenticity
    pub owner_signature: Vec<u8>,
}

/// Recovery attempt record (for rate limiting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    /// Temporary identity that initiated recovery
    pub temp_identity: Vec<u8>,
    /// Timestamp of attempt
    pub timestamp: u64,
    /// Guardian who received request
    pub guardian_pubkey: Vec<u8>,
    /// Whether guardian approved (for audit)
    pub approved: bool,
    /// Verification methods used
    pub verification_methods: Vec<String>,
}

/// Active recovery request (time-locked)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveRecoveryRequest {
    /// Unique recovery request ID
    pub request_id: String,
    /// Temporary identity requesting recovery
    pub requestor_temp_id: Vec<u8>,
    /// Owner's permanent identity being recovered
    pub owner_pubkey: Vec<u8>,
    /// Guardian who approved this shard
    pub guardian_pubkey: Vec<u8>,
    /// When approval was given
    pub approved_at: u64,
    /// When time-lock expires
    pub release_at: u64,
    /// Whether owner has cancelled
    pub cancelled: bool,
    /// Cancellation timestamp if cancelled
    pub cancelled_at: Option<u64>,
}
```

#### Social Recovery Implementation

```rust
/// Setup social recovery for an identity
pub async fn setup_social_recovery(
    identity: &IdentityKeys,
    mnemonic: &Mnemonic,
    recovery_password: &str,
    guardians: &[Guardian],
    config: &SocialRecoveryConfig,
) -> Result<Vec<EncryptedShard>, RecoveryError> {
    // Validate configuration
    if guardians.len() != config.total_guardians {
        return Err(RecoveryError::InvalidGuardianCount {
            expected: config.total_guardians,
            actual: guardians.len(),
        });
    }
    if config.threshold > config.total_guardians {
        return Err(RecoveryError::InvalidThreshold);
    }
    if config.threshold < 2 {
        return Err(RecoveryError::ThresholdTooLow);
    }
    
    // Step 1: Serialize mnemonic
    let mnemonic_bytes = mnemonic.to_string().into_bytes();
    
    // Step 2: Encrypt mnemonic with recovery password
    let salt = KeyManager::generate_salt();
    let key_manager = KeyManager::new(100_000, false).await?;
    let encryption_key = key_manager.derive_key(recovery_password, &salt).await?;
    let encrypted_mnemonic = key_manager.encrypt(&encryption_key, &mnemonic_bytes)?;
    
    // Step 3: Split using Shamir's Secret Sharing
    let shards = shamir_split(
        &encrypted_mnemonic,
        config.threshold,
        config.total_guardians,
    )?;
    
    // Step 4: Create encrypted shards for each guardian
    let mut encrypted_shards = Vec::with_capacity(guardians.len());
    
    for (guardian, shard_data) in guardians.iter().zip(shards.iter()) {
        // Create shard metadata
        let shard = EncryptedShard {
            version: 1,
            owner_pubkey: identity.mldsa_public.as_bytes().to_vec(),
            owner_four_words: identity.four_words.clone(),
            shard_index: guardian.shard_index,
            total_shards: config.total_guardians,
            threshold: config.threshold,
            encrypted_data: shard_data.clone(),
            salt: salt.clone(),
            created_at: current_timestamp(),
            owner_signature: vec![], // Will be set below
        };
        
        // Sign the shard to prove authenticity
        let shard_bytes = bincode::serialize(&shard)?;
        let signature = ml_dsa_sign(&identity.mldsa_secret, &shard_bytes)?;
        
        let mut signed_shard = shard;
        signed_shard.owner_signature = signature.as_bytes().to_vec();
        
        encrypted_shards.push(signed_shard);
    }
    
    Ok(encrypted_shards)
}

/// Initiate social recovery (guardian side)
pub async fn guardian_approve_recovery(
    guardian_identity: &IdentityKeys,
    stored_shard: &EncryptedShard,
    requestor_temp_id: &[u8],
    verification_result: &VerificationResult,
    config: &SocialRecoveryConfig,
) -> Result<ActiveRecoveryRequest, RecoveryError> {
    // Verify shard authenticity
    let owner_pubkey = MlDsaPublicKey::from_bytes(&stored_shard.owner_pubkey)?;
    let mut shard_for_verify = stored_shard.clone();
    shard_for_verify.owner_signature = vec![];
    let shard_bytes = bincode::serialize(&shard_for_verify)?;
    
    if !ml_dsa_verify(&owner_pubkey, &shard_bytes, &stored_shard.owner_signature)? {
        return Err(RecoveryError::InvalidShardSignature);
    }
    
    // Verify requestor passed verification
    if !verification_result.passed {
        return Err(RecoveryError::VerificationFailed {
            reason: verification_result.failure_reason.clone().unwrap_or_default(),
        });
    }
    
    // Check rate limiting
    check_rate_limit(&stored_shard.owner_pubkey)?;
    
    // Create time-locked recovery request
    let request = ActiveRecoveryRequest {
        request_id: generate_request_id(),
        requestor_temp_id: requestor_temp_id.to_vec(),
        owner_pubkey: stored_shard.owner_pubkey.clone(),
        guardian_pubkey: guardian_identity.mldsa_public.as_bytes().to_vec(),
        approved_at: current_timestamp(),
        release_at: current_timestamp() + config.timelock_seconds,
        cancelled: false,
        cancelled_at: None,
    };
    
    // Broadcast recovery notification to owner's known devices
    broadcast_recovery_notification(&stored_shard.owner_pubkey, &request).await?;
    
    Ok(request)
}

/// Cancel an active recovery request (owner side)
pub async fn cancel_recovery(
    owner_identity: &IdentityKeys,
    request_id: &str,
) -> Result<(), RecoveryError> {
    // Sign cancellation with owner's key
    let cancellation = RecoveryCancellation {
        request_id: request_id.to_string(),
        cancelled_at: current_timestamp(),
        owner_signature: vec![],
    };
    
    let cancel_bytes = bincode::serialize(&cancellation)?;
    let signature = ml_dsa_sign(&owner_identity.mldsa_secret, &cancel_bytes)?;
    
    let signed_cancellation = RecoveryCancellation {
        owner_signature: signature.as_bytes().to_vec(),
        ..cancellation
    };
    
    // Broadcast cancellation to all guardians
    broadcast_cancellation(&signed_cancellation).await?;
    
    Ok(())
}

/// Complete social recovery (requestor side)
pub async fn complete_social_recovery(
    collected_shards: &[EncryptedShard],
    recovery_password: &str,
    new_vault_password: &str,
    config: &StorageConfig,
) -> Result<RecoveryResult, RecoveryError> {
    // Verify we have enough shards
    if collected_shards.is_empty() {
        return Err(RecoveryError::NoShardsProvided);
    }
    
    let threshold = collected_shards[0].threshold;
    if collected_shards.len() < threshold {
        return Err(RecoveryError::InsufficientShards {
            required: threshold,
            provided: collected_shards.len(),
        });
    }
    
    // Extract shard data for reconstruction
    let shard_data: Vec<(usize, Vec<u8>)> = collected_shards
        .iter()
        .map(|s| (s.shard_index, s.encrypted_data.clone()))
        .collect();
    
    // Reconstruct encrypted mnemonic using Shamir
    let encrypted_mnemonic = shamir_combine(&shard_data, threshold)?;
    
    // Decrypt mnemonic with recovery password
    let salt = &collected_shards[0].salt;
    let key_manager = KeyManager::new(100_000, false).await?;
    let decryption_key = key_manager.derive_key(recovery_password, salt).await?;
    
    let mnemonic_bytes = key_manager.decrypt(&decryption_key, &encrypted_mnemonic)
        .map_err(|_| RecoveryError::WrongRecoveryPassword)?;
    
    let mnemonic_string = String::from_utf8(mnemonic_bytes.to_vec())
        .map_err(|_| RecoveryError::CorruptedMnemonic)?;
    
    // Now we have the mnemonic, use standard recovery
    recover_from_mnemonic(&mnemonic_string, None, new_vault_password, config).await
}
```

### Part 3: Verification for Social Recovery

#### The AI Deepfake Problem

Modern AI can generate convincing:
- Real-time face swaps in video calls
- Voice clones from ~30 seconds of audio
- Synthetic video avatars

**This makes video/voice verification unreliable for high-security scenarios.**

#### Multi-Factor Verification Design

```rust
/// Verification methods available for social recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Pre-shared security questions (set at guardian setup)
    SecurityQuestions {
        /// Number of questions that must be answered correctly
        required_correct: usize,
        /// Total questions available
        total_questions: usize,
    },
    
    /// Out-of-band phone verification (guardian calls known number)
    PhoneVerification {
        /// Phone number registered at guardian setup
        registered_number: String,
        /// Verification code spoken by requestor
        verification_code: String,
    },
    
    /// In-person verification (highest security)
    InPerson {
        /// Location agreed upon
        location: String,
        /// Additional notes
        notes: String,
    },
    
    /// Hardware token verification (if user has backup token)
    HardwareToken {
        /// Token type (YubiKey, etc.)
        token_type: String,
        /// Challenge-response verification
        challenge: Vec<u8>,
        response: Vec<u8>,
    },
    
    /// Pre-shared secret phrase (set at guardian setup)
    SharedSecret {
        /// Hash of the secret phrase
        secret_hash: Vec<u8>,
    },
    
    /// Delayed verification (reduces AI attack window)
    /// Guardian waits 24-48 hours before approving
    DelayedApproval {
        /// When request was received
        received_at: u64,
        /// Minimum delay before approval
        min_delay_seconds: u64,
    },
}

/// Result of verification process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification passed
    pub passed: bool,
    /// Methods used
    pub methods_used: Vec<VerificationMethod>,
    /// Methods that passed
    pub methods_passed: Vec<VerificationMethod>,
    /// Failure reason if any
    pub failure_reason: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Guardian's notes
    pub guardian_notes: Option<String>,
}

/// Verification requirements for different security levels
#[derive(Debug, Clone)]
pub struct VerificationRequirements {
    /// Minimum number of verification methods required
    pub min_methods: usize,
    /// Required method types (at least one from each category)
    pub required_categories: Vec<VerificationCategory>,
    /// Minimum confidence score
    pub min_confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VerificationCategory {
    /// Something you know (security questions, shared secrets)
    Knowledge,
    /// Something you have (phone, hardware token)
    Possession,
    /// Something you are (in-person, delayed approval)
    Presence,
}

impl Default for VerificationRequirements {
    fn default() -> Self {
        Self {
            min_methods: 2,
            required_categories: vec![
                VerificationCategory::Knowledge,
                VerificationCategory::Possession,
            ],
            min_confidence: 0.8,
        }
    }
}
```

#### Security Questions Best Practices

```rust
/// Security question configuration
/// Questions should be:
/// - Stable over time (not "current" anything)
/// - Specific enough to be unique
/// - Not easily discoverable via social media
/// - Memorable to the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityQuestion {
    /// Question text (set by user at guardian setup)
    pub question: String,
    /// Hash of lowercase, normalized answer
    pub answer_hash: Vec<u8>,
    /// Salt for answer hashing
    pub salt: Vec<u8>,
    /// Category for diversity requirements
    pub category: QuestionCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionCategory {
    /// Childhood memories
    Childhood,
    /// Family history
    Family,
    /// Personal preferences (stable ones)
    Preferences,
    /// Shared experiences with this guardian
    SharedExperience,
    /// Custom category
    Custom(String),
}

/// Example good security questions:
/// - "What was the name of your first pet?" (Childhood)
/// - "In what city did your parents meet?" (Family)
/// - "What was our secret codeword in college?" (SharedExperience)
/// - "What is the registration number of your first car?" (Preferences - stable)
///
/// Example BAD security questions (easily discovered):
/// - "What is your mother's maiden name?" (public records)
/// - "Where were you born?" (public records)
/// - "What is your favorite movie?" (social media)
/// - "What high school did you attend?" (LinkedIn)

pub fn hash_security_answer(answer: &str, salt: &[u8]) -> Vec<u8> {
    // Normalize answer: lowercase, trim whitespace, remove punctuation
    let normalized = answer
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    
    // Use PBKDF2 with high iteration count for answer hashing
    let mut hash = vec![0u8; 32];
    pbkdf2_hmac::<Sha256>(
        normalized.as_bytes(),
        salt,
        100_000,
        &mut hash,
    );
    
    hash
}
```

### Part 4: User Experience Design

#### First-Time Identity Creation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     IDENTITY CREATION USER FLOW                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  SCREEN 1: Welcome                                                          │
│  ═══════════════════                                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  🌟 Welcome to Communitas                                          │   │
│  │                                                                     │   │
│  │  You're about to create your decentralized identity.               │   │
│  │  This identity is yours alone - no company controls it.            │   │
│  │                                                                     │   │
│  │  You'll receive a recovery phrase that lets you restore            │   │
│  │  your identity if you ever lose access to your device.             │   │
│  │                                                                     │   │
│  │  ⚠️  This phrase is the ONLY way to recover your identity.         │   │
│  │     Keep it safe and never share it online.                        │   │
│  │                                                                     │   │
│  │  [Create My Identity]                                               │   │
│  │                                                                     │   │
│  │  Already have a recovery phrase? [Recover Identity]                │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 2: Recovery Phrase                                                  │
│  ═══════════════════════════                                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  📝 Your Recovery Phrase                                           │   │
│  │                                                                     │   │
│  │  Write down these 24 words in order. Store them safely.            │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │                                                             │   │   │
│  │  │  1. witch      7. despair    13. ribbon   19. cruel        │   │   │
│  │  │  2. collapse   8. creek      14. health   20. parade       │   │   │
│  │  │  3. practice   9. road       15. lawn     21. dumb         │   │   │
│  │  │  4. feed      10. again      16. witness  22. violin       │   │   │
│  │  │  5. shame     11. ice        17. dizzy    23. ocean        │   │   │
│  │  │  6. open      12. lung       18. merit    24. forest       │   │   │
│  │  │                                                             │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  │  ✅ I have written down my recovery phrase                         │   │
│  │                                                                     │   │
│  │  [Continue]                                                         │   │
│  │                                                                     │   │
│  │  💡 Tips:                                                          │   │
│  │  • Write on paper, not digitally                                   │   │
│  │  • Store in a fireproof safe or bank deposit box                   │   │
│  │  • Consider making 2 copies in different locations                 │   │
│  │  • Never take a screenshot or store in cloud                       │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 3: Verify Backup                                                    │
│  ═══════════════════════                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  ✓ Verify Your Backup                                              │   │
│  │                                                                     │   │
│  │  Please enter the following words from your recovery phrase:       │   │
│  │                                                                     │   │
│  │  Word #3:  [__________]                                            │   │
│  │  Word #11: [__________]                                            │   │
│  │  Word #19: [__________]                                            │   │
│  │                                                                     │   │
│  │  [Verify]                                                           │   │
│  │                                                                     │   │
│  │  [← Show phrase again]                                             │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 4: Create Password                                                  │
│  ═══════════════════════════                                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  🔐 Create Device Password                                         │   │
│  │                                                                     │   │
│  │  This password unlocks your identity on THIS device.               │   │
│  │  Your recovery phrase works on ANY device.                         │   │
│  │                                                                     │   │
│  │  Password:        [________________]                                │   │
│  │  Confirm:         [________________]                                │   │
│  │                                                                     │   │
│  │  Strength: ████████░░ Strong                                       │   │
│  │                                                                     │   │
│  │  [Create Password]                                                  │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 5: Optional Biometric                                               │
│  ═══════════════════════════                                                │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  👆 Enable Quick Access                                            │   │
│  │                                                                     │   │
│  │  Use Face ID / Touch ID to unlock quickly?                         │   │
│  │                                                                     │   │
│  │  [Enable Face ID]                                                   │   │
│  │                                                                     │   │
│  │  [Skip for now]                                                     │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 6: Display Name                                                     │
│  ═══════════════════════                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  👤 What should we call you?                                       │   │
│  │                                                                     │   │
│  │  Display Name: [________________]                                   │   │
│  │                                                                     │   │
│  │  This is what your contacts will see.                              │   │
│  │  You can change it anytime.                                        │   │
│  │                                                                     │   │
│  │  [Complete Setup]                                                   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 7: Success                                                          │
│  ════════════════                                                           │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  🎉 Welcome, Alice!                                                │   │
│  │                                                                     │   │
│  │  Your identity: ocean-forest-moon-star                             │   │
│  │                                                                     │   │
│  │  You're all set! Your identity is secured by:                      │   │
│  │                                                                     │   │
│  │  ✅ 24-word recovery phrase (stored safely by you)                 │   │
│  │  ✅ Device password                                                │   │
│  │  ✅ Face ID enabled                                                │   │
│  │                                                                     │   │
│  │  Optional: Add recovery guardians for extra protection             │   │
│  │  [Setup Social Recovery]  [Maybe Later]                            │   │
│  │                                                                     │   │
│  │  [Start Using Communitas]                                           │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Recovery Flow UX

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        RECOVERY USER FLOW                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  SCREEN 1: Recovery Method Selection                                        │
│  ═══════════════════════════════════                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  🔄 Recover Your Identity                                          │   │
│  │                                                                     │   │
│  │  How would you like to recover?                                    │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │ 📝 Recovery Phrase                              [Recommended] │   │   │
│  │  │    I have my 24-word recovery phrase                        │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  │  ┌─────────────────────────────────────────────────────────────┐   │   │
│  │  │ 👥 Social Recovery                                           │   │   │
│  │  │    Contact my recovery guardians                            │   │   │
│  │  │    (Requires 7-day waiting period)                          │   │   │
│  │  └─────────────────────────────────────────────────────────────┘   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 2A: Mnemonic Entry                                                  │
│  ═════════════════════════                                                  │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  📝 Enter Recovery Phrase                                          │   │
│  │                                                                     │   │
│  │  Enter your 24 words in order:                                     │   │
│  │                                                                     │   │
│  │   1. [________]  7. [________] 13. [________] 19. [________]      │   │
│  │   2. [________]  8. [________] 14. [________] 20. [________]      │   │
│  │   3. [________]  9. [________] 15. [________] 21. [________]      │   │
│  │   4. [________] 10. [________] 16. [________] 22. [________]      │   │
│  │   5. [________] 11. [________] 17. [________] 23. [________]      │   │
│  │   6. [________] 12. [________] 18. [________] 24. [________]      │   │
│  │                                                                     │   │
│  │  ✅ Checksum valid                                                 │   │
│  │                                                                     │   │
│  │  [Recover Identity]                                                 │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  SCREEN 2B: Social Recovery (if selected)                                   │
│  ═══════════════════════════════════════                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │  👥 Social Recovery                                                │   │
│  │                                                                     │   │
│  │  ⚠️  This requires contacting your recovery guardians.             │   │
│  │     There will be a 7-day waiting period.                          │   │
│  │                                                                     │   │
│  │  1. We'll create a temporary identity for you                      │   │
│  │  2. Contact your guardians using their known phone numbers         │   │
│  │  3. They will verify your identity and approve recovery            │   │
│  │  4. After 7 days, you can complete recovery                        │   │
│  │                                                                     │   │
│  │  Enter your four-word identity:                                    │   │
│  │  [________]-[________]-[________]-[________]                       │   │
│  │                                                                     │   │
│  │  [Begin Social Recovery]                                            │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Part 5: Security Considerations

#### Threat Analysis

| Threat | Attack Vector | Mitigation |
|--------|---------------|------------|
| **Mnemonic Theft** | Physical access to written backup | Passphrase (25th word), geographic distribution |
| **Mnemonic Loss** | Fire, flood, forgetting location | Social recovery backup, multiple copies |
| **Deepfake Attack** | AI impersonation to guardians | Multi-factor verification, time-locks, out-of-band |
| **Guardian Collusion** | k guardians conspire | Encrypted shards (need password too), diverse guardian selection |
| **Social Engineering** | Manipulating guardians | Security questions, shared secrets, delayed approval |
| **Coercion** | Forced recovery | Duress passphrase (reveals decoy identity) |
| **Replay Attack** | Reusing old recovery attempts | One-time request IDs, nonces |
| **Time-Lock Bypass** | Attacking before lock expires | Cryptographic time-locks, distributed enforcement |

#### Passphrase (25th Word) Option

For additional security, users can set an optional passphrase:

```rust
/// BIP39 passphrase provides:
/// 1. Additional entropy (unlimited length)
/// 2. Plausible deniability (different passphrase = different identity)
/// 3. Protection against mnemonic theft alone
///
/// WARNING: Forgotten passphrase = permanent identity loss
pub struct PassphraseConfig {
    /// Whether passphrase is required
    pub required: bool,
    /// Minimum passphrase length
    pub min_length: usize,
    /// Whether to allow duress passphrase
    pub allow_duress: bool,
}

/// Duress passphrase: if coerced, user enters duress passphrase
/// This derives a different (decoy) identity with minimal data
/// Attacker cannot verify if this is the "real" identity
pub fn handle_potential_duress(
    mnemonic: &Mnemonic,
    entered_passphrase: &str,
    duress_passphrase: &str,
) -> IdentityType {
    if entered_passphrase == duress_passphrase {
        IdentityType::Duress  // Derive decoy identity
    } else {
        IdentityType::Primary  // Derive real identity
    }
}
```

#### Rate Limiting and Abuse Prevention

```rust
/// Rate limiting configuration for recovery attempts
pub struct RateLimitConfig {
    /// Maximum recovery attempts per identity per day
    pub max_daily_attempts: usize,
    /// Lockout duration after max attempts (seconds)
    pub lockout_duration: u64,
    /// Exponential backoff base (seconds)
    pub backoff_base: u64,
    /// Maximum backoff (seconds)
    pub max_backoff: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_daily_attempts: 3,
            lockout_duration: 30 * 24 * 60 * 60,  // 30 days
            backoff_base: 60,  // 1 minute
            max_backoff: 24 * 60 * 60,  // 24 hours
        }
    }
}

/// Calculate backoff for recovery attempt
pub fn calculate_backoff(attempt_number: usize, config: &RateLimitConfig) -> u64 {
    let backoff = config.backoff_base * 2u64.pow(attempt_number as u32);
    backoff.min(config.max_backoff)
}
```

### Part 6: Implementation Plan

#### Phase 1: BIP39 Core (Required)

1. Add `bip39` crate dependency
2. Implement `generate_recovery_mnemonic()`
3. Implement `derive_identity_keys()` with deterministic PQC generation
4. Implement `recover_from_mnemonic()`
5. Update identity creation flow to generate and display mnemonic
6. Add verification step (confirm random words)
7. Update vault creation to store derived keys

#### Phase 2: Recovery UI (Required)

1. Add mnemonic display screen with word grid
2. Add mnemonic entry screen with validation
3. Add recovery flow screens
4. Add checksum validation feedback

#### Phase 3: Social Recovery (Optional)

1. Implement Shamir's Secret Sharing
2. Add guardian management UI
3. Implement shard encryption and distribution
4. Implement time-locked recovery requests
5. Add cancellation mechanism
6. Add verification methods (security questions, etc.)

#### Phase 4: Advanced Features (Future)

1. Passphrase (25th word) support
2. Duress identity support
3. Hardware wallet integration
4. Multi-signature guardian schemes

## Consequences

### Positive

- **Self-Sovereign Recovery**: Users control their own backup, no central authority
- **Proven Security**: BIP39 is battle-tested with billions in assets
- **Offline Recovery**: Primary recovery works without network
- **Deterministic**: Same mnemonic always produces same identity
- **Quantum-Safe**: PBKDF2-SHA512 with 512-bit output resists Grover's algorithm
- **Optional Social Backup**: Users who can't manage physical backup have an option
- **AI-Resistant Design**: Multi-factor verification, time-locks mitigate deepfake risk

### Negative

- **User Responsibility**: Users must secure their mnemonic (can't "reset password")
- **Complexity**: 24 words are more complex than traditional passwords
- **Social Recovery Trust**: Requires trusting k guardians not to collude
- **Time-Lock Delays**: Social recovery takes 7 days minimum

### Trade-offs

| Aspect | Trade-off |
|--------|-----------|
| **Security vs Convenience** | 24 words harder to remember than password, but more secure |
| **Self-Sovereignty vs Safety Net** | No central recovery, but social recovery backup |
| **Simplicity vs Features** | More complex than centralized auth, but no single point of failure |

## Alternatives Considered

### 1. Hardware Security Module (HSM) Backup

Store encrypted keys in cloud HSM (AWS CloudHSM, Azure Key Vault).

**Rejected because:**
- Introduces centralized dependency
- Conflicts with self-sovereignty principle
- Subject to government access
- Not offline-capable

### 2. Pure Social Recovery (No Mnemonic)

Only use distributed key shards with contacts.

**Rejected because:**
- Vulnerable to AI deepfake attacks
- Requires network for all recovery
- Guardian availability issues
- Collusion risk without additional factor

### 3. Email/Phone Recovery

Traditional "forgot password" flow with email/SMS.

**Rejected because:**
- Email/phone providers become identity authorities
- SIM swapping attacks
- Account takeover via provider compromise
- Centralized dependency

### 4. Biometric-Only Recovery

Use biometric data as recovery factor.

**Rejected because:**
- Biometrics can't be changed if compromised
- AI can generate synthetic biometrics
- Not universally available
- Privacy concerns with biometric storage

## References

- BIP39 Specification: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- Shamir's Secret Sharing: https://en.wikipedia.org/wiki/Shamir%27s_Secret_Sharing
- NIST SP 800-132 (PBKDF): https://csrc.nist.gov/publications/detail/sp/800-132/final
- Related ADR: [ADR-006 Post-Quantum Cryptography](ADR-006-post-quantum-cryptography.md)
- Related ADR: [ADR-011 Encrypted Vault Storage](ADR-011-encrypted-vault-storage.md)
- Related ADR: [ADR-012 Identity Packet System](ADR-012-identity-packet-system.md)

## Appendix A: BIP39 Word Lists

The BIP39 specification includes standardized word lists for multiple languages:
- English (2048 words)
- Japanese, Korean, Chinese (Simplified/Traditional)
- French, Italian, Spanish, Portuguese
- Czech

Each word is carefully chosen to:
- Be unambiguous (no similar words)
- Have consistent length (3-8 characters typical)
- Be common enough to be recognizable

## Appendix B: Security Parameter Justification

| Parameter | Value | Justification |
|-----------|-------|---------------|
| Mnemonic words | 24 | 256-bit entropy (NIST Level 5 equivalent) |
| PBKDF2 iterations (BIP39) | 2048 | BIP39 standard |
| PBKDF2 iterations (vault) | 100,000 | OWASP 2023 recommendation |
| Time-lock duration | 7 days | Balance between security and usability |
| Guardian threshold | 3-of-5 | Resilient to 2 compromised guardians |
| Max recovery attempts | 3/day | Prevents brute force on security questions |
| Lockout duration | 30 days | Severe penalty for repeated failures |

## Appendix C: Cryptographic Details

### Key Derivation Chain

```
Mnemonic (24 words, 256 bits entropy)
    │
    ├──► PBKDF2-HMAC-SHA512(mnemonic, "mnemonic" + passphrase, 2048)
    │    │
    │    └──► 512-bit Master Seed
    │         │
    │         ├──► BLAKE3-KDF("communitas:mldsa65:v1") → ML-DSA-65 seed
    │         │    │
    │         │    └──► ChaCha20Rng → ML-DSA-65 keypair (signing/identity)
    │         │
    │         └──► BLAKE3-KDF("communitas:mlkem768:v1") → ML-KEM-768 seed
    │              │
    │              └──► ChaCha20Rng → ML-KEM-768 keypair (encryption)
    │
    └──► Four-word identity derived from ML-DSA public key (for display)
```

### Shard Encryption Chain

```
Mnemonic String
    │
    ├──► ChaCha20-Poly1305-Encrypt(recovery_password_key) → Encrypted Mnemonic
    │    │
    │    └──► Shamir Split (k-of-n) → Shards[1..n]
    │         │
    │         └──► Each shard signed by owner → Authenticated Shards
    │
    └──► Distributed to guardians (stored encrypted in their vaults)
```

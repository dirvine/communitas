# Authentication System - Communitas

## Overview

Communitas implements a **four-word identity + password** authentication system with optional **passkey/Touch ID** support for seamless re-authentication. The system provides cryptographic isolation between identities while enabling convenient biometric login on supported platforms.

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────────┐
│                     Authentication Flow                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  User Input                                                     │
│  ├─ Four-word address (e.g., "ocean-forest-moon-star")         │
│  └─ Password                                                    │
│         │                                                       │
│         ▼                                                       │
│  Password Hashing (BLAKE3)                                      │
│         │                                                       │
│         ▼                                                       │
│  Vault Lookup                                                   │
│  ├─ Find vault directory by four-word address                  │
│  └─ Load encrypted vault metadata                              │
│         │                                                       │
│         ▼                                                       │
│  Key Derivation (PBKDF2)                                        │
│  ├─ 100,000 iterations                                          │
│  ├─ SHA-256 hash                                                │
│  └─ 32-byte salt (unique per vault)                            │
│         │                                                       │
│         ▼                                                       │
│  Password Verification (AEAD)                                   │
│  ├─ ChaCha20-Poly1305 authentication                           │
│  └─ Constant-time comparison                                   │
│         │                                                       │
│         ▼                                                       │
│  Session Creation                                               │
│  ├─ Generate session ID (UUID)                                 │
│  ├─ Store vault key (zeroized)                                 │
│  └─ Set session expiry (configurable)                          │
│         │                                                       │
│         ▼                                                       │
│  Optional: Store in Keyring                                     │
│  └─ macOS Keychain / Windows Credential Manager                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Storage Locations

**macOS:**
```
~/Library/Application Support/com.saorsalabs.communitas/
├── vaults/
│   └── {four-word-address}/
│       ├── metadata.json          # Unencrypted (created_at, last_accessed)
│       ├── identity.enc           # Encrypted identity data
│       ├── password.verifier      # AEAD-encrypted password verification
│       ├── salt                   # PBKDF2 salt (32 bytes)
│       └── data/                  # Encrypted data files
└── app_config.json                # App-level config (last_identity, etc.)
```

**Windows:**
```
%APPDATA%/communitas/vaults/{four-word-address}/
```

**Linux:**
```
~/.config/communitas/vaults/{four-word-address}/
```

## Authentication Methods

### 1. Standard Login (Four-words + Password)

**Frontend (`src/components/auth/UnifiedAuthFlow.tsx`):**

```typescript
const handleLogin = async () => {
  try {
    const result = await invoke<AuthResult>('auth_login', {
      fourWords: fourWords,
      password: password,
      remember: rememberMe  // Store in keyring if true
    });

    // Session established
    setAuthState({
      isAuthenticated: true,
      identity: result.identity
    });
  } catch (error) {
    // Handle authentication failure
  }
};
```

**Backend (`communitas-core/src/encrypted_storage/mod.rs`):**

```rust
pub async fn login(
    &self,
    four_words: &str,
    password: &str,
    app_config: Option<&AppConfig>,
) -> Result<SessionInfo> {
    // 1. Normalize four-word address
    let normalized = normalize_four_words(four_words)?;

    // 2. Hash password for vault lookup
    let password_hash = self.key_manager.hash_password(password).await?;

    // 3. Load vault
    let vault_path = self.get_vault_path(&normalized);
    let vault = self.load_vault(&vault_path).await?;

    // 4. Derive encryption key from password
    let key = self.key_manager
        .derive_key(password, &vault.salt)
        .await?;

    // 5. Verify password (constant-time AEAD check)
    self.verify_password(&key, &vault.password_verifier)?;

    // 6. Create session
    let session = Session {
        id: Uuid::new_v4(),
        four_words: normalized.clone(),
        vault_key: key,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
    };

    // 7. Optional: Store password in platform keyring
    if self.config.use_keyring && remember_me {
        self.key_manager
            .store_in_keyring(&normalized, password.as_bytes())
            .await
            .ok(); // Silent failure - see Known Issues
    }

    Ok(SessionInfo::from(session))
}
```

### 2. Passkey/Touch ID Login

**Overview:**
Passkeys use **WebAuthn** standard with platform authenticators (Touch ID, Face ID, Windows Hello) for biometric authentication. The passkey workflow requires the password to be stored in the platform keyring.

**Registration Flow:**

```typescript
// Frontend - Register Passkey
async function registerPasskey() {
  // 1. Get challenge from backend
  const challenge = await invoke('auth_generate_challenge');

  // 2. Create WebAuthn credential
  const credential = await navigator.credentials.create({
    publicKey: {
      challenge: base64ToArrayBuffer(challenge),
      rp: {
        name: 'Communitas',
        id: window.location.hostname
      },
      user: {
        id: crypto.randomUUID(),
        name: currentIdentity.four_words,
        displayName: currentIdentity.display_name
      },
      pubKeyCredParams: [
        { type: 'public-key', alg: -7 }  // ES256 (ECDSA with SHA-256)
      ],
      authenticatorSelection: {
        authenticatorAttachment: 'platform',  // Touch ID, Face ID, Windows Hello
        userVerification: 'required',
        requireResidentKey: false
      },
      timeout: 60000,
      attestation: 'none'
    }
  });

  // 3. Store credential in vault
  await invoke('auth_store_passkey', {
    identityId: currentIdentity.four_words,
    credential: serializeCredential(credential)
  });

  // 4. Enable keyring storage for this identity
  await invoke('auth_enable_keyring', {
    fourWords: currentIdentity.four_words
  });
}
```

**Authentication Flow:**

```typescript
// Frontend - Login with Passkey
async function loginWithPasskey(fourWords: string) {
  // 1. Get challenge from backend
  const challenge = await invoke('auth_generate_passkey_challenge', {
    fourWords: fourWords
  });

  // 2. Get WebAuthn assertion (triggers Touch ID prompt)
  const assertion = await navigator.credentials.get({
    publicKey: {
      challenge: base64ToArrayBuffer(challenge),
      allowCredentials: [{
        type: 'public-key',
        id: base64ToArrayBuffer(credentialId)
      }],
      userVerification: 'required',
      timeout: 60000
    }
  });

  // 3. Verify and login
  const result = await invoke('auth_login_with_passkey', {
    fourWords: fourWords,
    assertion: serializeAssertion(assertion)
  });

  // Session established
  setAuthState({ isAuthenticated: true, identity: result.identity });
}
```

**Backend Implementation:**

```rust
#[tauri::command]
pub async fn auth_login_with_passkey(
    state: State<'_, AppState>,
    four_words: String,
    assertion_json: String,
) -> Result<SessionInfo, String> {
    let storage = state.storage_manager.read().await;
    let manager = storage.as_ref().ok_or("Storage not initialized")?;

    // 1. Parse WebAuthn assertion
    let assertion: PasskeyAssertion = serde_json::from_str(&assertion_json)
        .map_err(|e| format!("Invalid assertion: {}", e))?;

    // 2. Verify passkey signature
    let vault = manager.load_vault_by_four_words(&four_words).await
        .map_err(|e| format!("Vault not found: {}", e))?;

    vault.verify_passkey_assertion(&assertion)
        .map_err(|e| format!("Passkey verification failed: {}", e))?;

    // 3. Retrieve password from keyring
    let password = manager.key_manager
        .get_from_keyring(&four_words)
        .await
        .map_err(|e| format!("Keyring access failed: {}", e))?;

    // 4. Standard login with retrieved password
    manager.login(&four_words, &String::from_utf8_lossy(&password), None).await
        .map_err(|e| format!("Login failed: {}", e))
}
```

### 3. Quick Re-authentication

**Last Used Identity:**

```rust
// App config tracks last successfully authenticated identity
pub struct AppConfig {
    pub last_identity: Option<String>,  // Four-word address
    pub auto_login_enabled: bool,
    pub keyring_enabled: bool,
}

// On app startup
pub async fn try_auto_login(app_config: &AppConfig) -> Result<Option<SessionInfo>> {
    if !app_config.auto_login_enabled {
        return Ok(None);
    }

    let Some(four_words) = &app_config.last_identity else {
        return Ok(None);
    };

    // Try to retrieve password from keyring
    match key_manager.get_from_keyring(four_words).await {
        Ok(password) => {
            // Auto-login
            Ok(Some(login(four_words, &password, Some(app_config)).await?))
        }
        Err(_) => {
            // Keyring unavailable, show login screen
            Ok(None)
        }
    }
}
```

## Security Model

### Cryptographic Properties

**Password Hashing:**
- Algorithm: BLAKE3
- Purpose: Fast vault lookup (not for key derivation)
- Context: `communitas:password:v1:`

**Key Derivation:**
- Algorithm: PBKDF2-HMAC-SHA256
- Iterations: 100,000 (as per DESIGN.md)
- Salt: 32 bytes random (unique per vault)
- Output: 256-bit key for ChaCha20-Poly1305

**Password Verification:**
- Algorithm: ChaCha20-Poly1305 AEAD
- Purpose: Constant-time password verification
- Prevents timing attacks on password comparison

**Session Keys:**
- Stored in memory using `Zeroizing<Vec<u8>>`
- Automatically wiped on drop
- Never written to disk unencrypted

### Platform Keyring Integration

**macOS Keychain:**
- Service: `com.saorsalabs.communitas`
- Account: Four-word address
- Access: Requires keychain-access-groups entitlement
- Protection: `kSecAttrAccessibleAfterFirstUnlock`

**Windows Credential Manager:**
- Target: `com.saorsalabs.communitas:{four-words}`
- Type: Generic credential
- Protection: User-level DPAPI encryption

**Linux Secret Service:**
- Schema: `com.saorsalabs.communitas`
- Collection: Default keyring
- Protection: User login password

### Threat Model

**Protected Against:**
- ✅ Password guessing (100k PBKDF2 iterations)
- ✅ Timing attacks (constant-time AEAD verification)
- ✅ Data tampering (AEAD authentication tags)
- ✅ Cross-identity access (separate vault keys)
- ✅ Memory disclosure (zeroized keys)
- ✅ Replay attacks (session expiry + UUIDs)

**Not Protected Against:**
- ❌ OS-level malware with keylogging
- ❌ Root/admin access to keyring
- ❌ Physical access to unlocked computer
- ❌ Memory dumps while session active

## Known Issues and Limitations

### Issue 1: Rust `keyring` Crate Incompatibility with macOS Sandbox

**Problem:**
The Rust `keyring` crate (v2.x) is **incompatible with sandboxed macOS applications**. When called from a sandboxed Tauri app:
- `entry.set_password()` returns `Ok(())` but **does not actually store** the password in Keychain
- `entry.get_password()` returns an error because no credential exists
- This occurs even with proper `keychain-access-groups` entitlement

**Root Cause:**
The `keyring` crate uses `security-framework` which doesn't properly handle the sandboxed app's keychain access group. The crate assumes unrestricted Keychain access which sandboxed apps don't have.

**Impact:**
- Touch ID/Passkey login **cannot work** on macOS (requires keyring to retrieve password)
- "Remember Me" functionality **silently fails**
- Users must enter password on every login

**Current Workaround:**
The code silently ignores keyring storage failures:
```rust
// Store in keyring (silently fails on sandboxed macOS)
self.key_manager
    .store_in_keyring(&normalized, password.as_bytes())
    .await
    .ok(); // Ignores error
```

**Proposed Solutions:**

1. **Tauri Store Plugin** (Recommended)
   - Use `@tauri-apps/plugin-store` for encrypted local storage
   - Encrypted with OS-level protection
   - Works reliably in sandboxed environment
   - Implementation: Store encrypted password in Tauri Store instead of keyring

2. **Custom Keychain Integration**
   - Direct `Security.framework` calls via Tauri commands
   - Properly handle sandbox keychain access groups
   - More complex but full control

3. **Remove Keyring Dependency**
   - Always require password on login
   - Use session tokens with longer expiry
   - Simpler but less convenient UX

**References:**
- Issue: `PASSKEY_DEBUG_SUMMARY.md`
- Crate: https://docs.rs/keyring/latest/keyring/
- Security Framework: https://developer.apple.com/documentation/security

### Issue 2: WebAuthn Requires HTTPS or localhost

**Problem:**
WebAuthn (passkeys) only work on HTTPS sites or `localhost`. If the Tauri app serves content from a non-localhost domain, passkey registration will fail.

**Solution:**
Tauri apps use `tauri://localhost` scheme which is treated as localhost by browsers, so this works correctly.

### Issue 3: TypeScript Type Error in UnifiedAuthFlow.tsx

**Problem:**
TypeScript error on line 373: `Property 'type' does not exist on type '{}'`

**Status:**
Pre-existing error, unrelated to authentication system. Does not affect runtime behavior.

## Testing

### Manual Testing Checklist

**Standard Login:**
- [ ] Login with correct four-words + password succeeds
- [ ] Login with incorrect password fails with clear error
- [ ] Login with non-existent four-words fails
- [ ] Session persists across page refreshes (if remember enabled)
- [ ] Session expires after configured timeout

**Passkey Registration:**
- [ ] Touch ID prompt appears on registration
- [ ] Biometric authentication required
- [ ] Passkey stored in vault
- [ ] Password stored in keyring (fails on sandboxed macOS)

**Passkey Login:**
- [ ] Touch ID prompt appears on login
- [ ] Successful biometric auth logs in user
- [ ] Failed biometric auth shows error
- [ ] Fallback to password works if passkey unavailable

**Multi-Identity:**
- [ ] Can create multiple vaults with different passwords
- [ ] Can switch between identities
- [ ] Each identity has isolated data
- [ ] Last used identity remembered

### Automated Tests

**Unit Tests:**
```bash
# Rust backend tests
cargo test --package communitas-core storage_tests
cargo test --package communitas-core key_management_tests

# Frontend tests
npm test -- auth
```

**Integration Tests:**
```bash
# Full login flow
cargo test --package communitas-desktop login_flow_integration_test

# Passkey flow (uses mock keyring in tests)
cargo test --package communitas-desktop passkey_flow_integration_test
```

## Future Enhancements

### Planned Features

1. **Multi-Device Sync**
   - Sync encrypted credentials across devices via gossip overlay
   - Secure device pairing with QR codes
   - Backup codes for recovery

2. **Passphrase Recovery**
   - Split-key backup with threshold recovery
   - Social recovery (trusted contacts)
   - Encrypted cloud backup option

3. **Advanced Biometrics**
   - Multiple passkeys per identity
   - Device-specific passkeys
   - Conditional authentication (location, time)

4. **Session Management**
   - Active session listing
   - Remote session revocation
   - Device fingerprinting

### Research Areas

1. **Post-Quantum Authentication**
   - ML-DSA for passkey signatures
   - ML-KEM for session key exchange
   - Hybrid classical/PQ schemes

2. **Zero-Knowledge Proofs**
   - ZK password verification
   - Anonymous authentication
   - Privacy-preserving session tokens

3. **Decentralized Identity**
   - DID integration
   - Verifiable credentials
   - Self-sovereign identity

## References

### Specifications
- **WebAuthn**: https://www.w3.org/TR/webauthn-2/
- **PBKDF2**: RFC 2898
- **ChaCha20-Poly1305**: RFC 8439
- **BLAKE3**: https://github.com/BLAKE3-team/BLAKE3-specs

### Documentation
- **Design Document**: `DESIGN.md`
- **User Isolation**: `docs/USER_ISOLATION_MODEL.md`
- **Debug Summary**: `PASSKEY_DEBUG_SUMMARY.md`

### Dependencies
- **chacha20poly1305**: `^0.10` - AEAD encryption
- **pbkdf2**: `^0.12` - Key derivation
- **blake3**: `^1.5` - Fast hashing
- **keyring**: `^2.3` - Platform keyring (with known issues)
- **zeroize**: `^1.7` - Memory wiping

## Conclusion

The Communitas authentication system provides **strong cryptographic isolation** between identities while supporting modern **biometric authentication** via WebAuthn. The primary limitation is the Rust `keyring` crate's incompatibility with macOS sandboxed apps, which prevents Touch ID login from working reliably.

The recommended path forward is to replace keyring with **Tauri Store plugin** for encrypted local storage, which will enable full passkey functionality on all platforms while maintaining security and user convenience.

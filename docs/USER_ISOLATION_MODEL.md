# User Isolation Model - Communitas

## Overview

Communitas implements a **multi-identity, single OS user** model where one operating system user can manage multiple Communitas identities with complete data isolation.

## Architecture

### Current Implementation ✅

#### Storage Structure
```
macOS:   ~/Library/Application Support/com.saorsalabs.communitas/vaults/
Windows: %APPDATA%/communitas/vaults/
Linux:   ~/.config/communitas/vaults/

vaults/
├── ocean-forest-moon-star/          # Identity 1 vault
│   ├── metadata.json                # Unencrypted metadata (created_at, last_accessed)
│   ├── identity.enc                 # Encrypted identity data (display name, profile)
│   ├── password.verifier            # AEAD-encrypted password verification
│   ├── index.enc                    # Encrypted file index
│   └── data/                        # Encrypted data files
│       └── *.enc
└── river-mountain-sun-cloud/        # Identity 2 vault
    ├── metadata.json
    ├── identity.enc
    ├── password.verifier
    ├── index.enc
    └── data/
```

#### Security Properties

1. **Cryptographic Isolation**:
   - Each vault has unique PBKDF2 salt (32 bytes random)
   - 100,000 PBKDF2 iterations per DESIGN.md
   - ChaCha20-Poly1305 AEAD encryption
   - Keys derived independently per identity
   - No shared secrets between vaults

2. **Authentication Methods**:
   - Four-word address + password
   - Password-only login (searches all vaults)
   - Session management with expiry

3. **Data Isolation**:
   - Each identity has separate encrypted storage
   - No cross-identity data access
   - Vault switching requires re-authentication

### User Experience Flows

#### Scenario 1: Single OS User, Multiple Identities
```
Alice (OS user) has:
├── Work Identity: "ocean-forest-moon-star"
│   └── Used for professional collaboration
└── Personal Identity: "river-mountain-sun-cloud"
    └── Used for personal projects
```

**Current Flow**:
1. Launch app → Show identity list or login screen
2. Select identity OR enter four-words
3. Enter password
4. Access identity-specific data

**Proposed Enhancement** (see below):
1. Launch app → Auto-login to last used identity (if keyring enabled)
2. OR Touch ID/Face ID for default identity
3. Switch identities from user menu (requires password)

#### Scenario 2: Shared Computer
```
Alice (OS user) wants to:
├── Create temporary identity for guest access
└── Keep her main identities separate
```

**Current Flow**:
1. Create new vault with unique four-words
2. Guest uses that identity + password
3. Alice deletes vault when done

**Security**: Each vault is cryptographically isolated - guest cannot access Alice's other identities.

## Proposed Enhancements

### 1. Default Identity & Quick Login

**Goal**: Seamless experience for primary identity while maintaining security.

**Implementation**:
```rust
// Store last used identity in unencrypted config
~/.config/communitas/app_config.json:
{
  "last_identity": "ocean-forest-moon-star",
  "auto_login_enabled": true,
  "keyring_enabled": true
}

// On startup:
1. Check for last_identity
2. If keyring_enabled && password in keyring:
   → Auto-login
3. Else:
   → Show login screen with last_identity pre-filled
```

**Platform Keyring Storage**:
- macOS: Keychain with `kSecAttrAccessibleAfterFirstUnlock`
- Windows: DPAPI with user-level protection
- Linux: Secret Service (GNOME Keyring, KWallet)

**Security Considerations**:
- Keyring password encrypted with OS-level protection
- Requires OS user authentication to access keyring
- User can disable auto-login in settings
- Session timeout still applies after login

### 2. Passkey/WebAuthn Support

**Goal**: Biometric authentication for default identity.

**Implementation**:
```typescript
// Frontend - WebAuthn Registration
async function registerPasskey() {
  const credential = await navigator.credentials.create({
    publicKey: {
      challenge: await invoke('auth_generate_challenge'),
      rp: { name: 'Communitas' },
      user: {
        id: crypto.randomUUID(),
        name: current_identity.four_words,
        displayName: current_identity.display_name
      },
      pubKeyCredParams: [
        { type: 'public-key', alg: -7 }  // ES256
      ],
      authenticatorSelection: {
        authenticatorAttachment: 'platform', // Touch ID, Face ID, Windows Hello
        userVerification: 'required'
      }
    }
  });

  // Store credential in vault
  await invoke('auth_store_passkey', {
    identityId: current_identity.four_words,
    credential: serialize(credential)
  });
}

// Login with Passkey
async function loginWithPasskey() {
  const credential = await navigator.credentials.get({
    publicKey: {
      challenge: await invoke('auth_generate_challenge'),
      userVerification: 'required'
    }
  });

  // Verify and login
  await invoke('auth_login_with_passkey', { credential });
}
```

**Rust Backend**:
```rust
// communitas-desktop/src/commands/auth.rs

#[tauri::command]
pub async fn auth_register_passkey(
    state: State<'_, AppState>,
    credential_json: String,
) -> Result<(), String> {
    // Verify user is logged in
    let session = state.active_session.read().await;
    let session = session.as_ref().ok_or("Not authenticated")?;

    // Parse WebAuthn credential
    let credential: PasskeyCredential = serde_json::from_str(&credential_json)?;

    // Store in vault with encryption
    let storage = state.storage_manager.read().await;
    let manager = storage.as_ref().ok_or("Storage not initialized")?;

    manager.store_passkey(&session.id, &credential).await?;

    // Enable keyring storage for password
    if manager.config.use_keyring {
        manager.key_manager
            .store_in_keyring(&session.four_words, &session.vault_key)
            .await?;
    }

    Ok(())
}

#[tauri::command]
pub async fn auth_login_with_passkey(
    state: State<'_, AppState>,
    credential_json: String,
) -> Result<SessionInfo, String> {
    // Parse WebAuthn assertion
    let assertion: PasskeyAssertion = serde_json::from_str(&credential_json)?;

    // Find vault by credential ID
    let storage = state.storage_manager.read().await;
    let manager = storage.as_ref().ok_or("Storage not initialized")?;

    let four_words = manager.find_vault_by_passkey(&assertion.credential_id).await?;

    // Get password from keyring (passkey requires keyring enabled)
    let password = manager.key_manager
        .get_from_keyring(&four_words)
        .await
        .ok_or("Passkey authentication requires keyring")?;

    // Login with retrieved password
    manager.login(&four_words, &password, None).await
}
```

**Security Properties**:
- Biometric verification required for passkey use
- Private key never leaves secure enclave (iOS/macOS) or TPM (Windows)
- Passkey only works for identity that registered it
- Falls back to password if passkey unavailable

### 3. Enhanced Identity Switching

**Goal**: Quick switch between identities with minimal friction.

**UI Flow**:
```
User Menu (click avatar) →
├── Alice (Work) ← Currently logged in
├── Switch Identity →
│   ├── Bob (Personal) → Enter password → Switch
│   ├── Carol (Gaming) → Touch ID → Switch
│   └── Add New Identity...
└── Logout
```

**Implementation**:
```rust
#[tauri::command]
pub async fn auth_quick_switch(
    state: State<'_, AppState>,
    target_four_words: String,
    password: Option<String>,
    use_passkey: bool,
) -> Result<SessionInfo, String> {
    // End current session
    let current_session = state.active_session.read().await;
    if let Some(session) = current_session.as_ref() {
        let storage = state.storage_manager.read().await;
        if let Some(manager) = storage.as_ref() {
            manager.logout(&session.id).await?;
        }
    }
    drop(current_session);

    // Switch to target identity
    if use_passkey {
        // Try passkey authentication with keyring password
        auth_login_with_passkey_by_identity(state, target_four_words).await
    } else {
        // Standard password login
        let password = password.ok_or("Password required")?;
        auth_login(state, target_four_words, password).await
    }
}
```

## Implementation Checklist

### Phase 1: Core Isolation (✅ Complete)
- [x] Per-identity vault directories
- [x] Cryptographic isolation (separate keys)
- [x] Password validation via AEAD
- [x] Multi-vault support
- [x] Vault listing and discovery

### Phase 2: UX Enhancements (Proposed)
- [ ] Last used identity tracking
- [ ] Auto-login with keyring
- [ ] Touch ID / Face ID support (WebAuthn)
- [ ] Quick identity switching UI
- [ ] Identity management UI (rename, delete)

### Phase 3: Advanced Features (Future)
- [ ] Vault export/import with password
- [ ] Vault sharing (encrypted backup codes)
- [ ] Multi-device sync (DHT-based)
- [ ] Family/team identity management

## Security Considerations

### Threat Model

**Protected Against**:
- ✅ OS user accessing another OS user's data (OS-level protection)
- ✅ One identity accessing another identity's data (cryptographic isolation)
- ✅ Password guessing (PBKDF2 100k iterations + AEAD verification)
- ✅ Tampering with vault data (AEAD authentication tags)

**Not Protected Against** (by design):
- ❌ OS user with physical access viewing their own vault directory structure
  - Mitigation: Data is encrypted, only metadata visible
- ❌ OS root/admin accessing keyring
  - Mitigation: Keyring requires OS authentication
- ❌ Malware running as OS user
  - Mitigation: Standard OS security applies

### Best Practices

1. **Password Requirements**:
   - Minimum 12 characters recommended
   - Mix of uppercase, lowercase, numbers, symbols
   - Unique per identity

2. **Keyring Usage**:
   - Enable only on trusted devices
   - Requires OS authentication (login password, biometrics)
   - Can be disabled per identity

3. **Passkey Usage**:
   - Requires platform authenticator (Touch ID, Face ID, Windows Hello)
   - Strongest authentication method
   - Recommended for primary identity on personal devices

4. **Multi-Identity Best Practices**:
   - Use different passwords per identity
   - Enable passkey for frequently used identities
   - Disable auto-login on shared computers
   - Regular vault backups

## Conclusion

The current implementation provides **strong cryptographic isolation** between identities while allowing a single OS user to manage multiple Communitas identities. The proposed enhancements maintain this security while improving user experience through platform keyring integration and biometric authentication.

The model supports both **personal use** (multiple identities per person) and **shared computer scenarios** (temporary identities) without compromising security or usability.

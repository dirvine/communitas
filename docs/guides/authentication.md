# Authentication Guide

Comprehensive guide to authentication in Communitas - from basic password auth to advanced passkey integration.

## Authentication Overview

Communitas provides multiple authentication methods to balance security and convenience:

1. **Password Authentication** - Traditional password-based login
2. **Passkey Authentication** - Modern biometric authentication (WebAuthn)
3. **Platform Integration** - Touch ID (macOS), Windows Hello, fingerprint readers
4. **Session Management** - Secure session handling with automatic expiry

## Identity System

Every user in Communitas has a unique identity consisting of:

- **Four-Word Address**: Human-readable identifier (e.g., "ocean-forest-moon-star")
- **Display Name**: Your chosen name (e.g., "Alice")
- **Device Name**: Identifies this device (e.g., "MacBook Pro")
- **ML-DSA Key Pair**: Post-quantum signature keys
- **ML-KEM Key Pair**: Post-quantum encryption keys

## Password Authentication

### Registration

Creating a new account with password:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke('register_user', {
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Alice',
  deviceName: 'MacBook Pro',
  password: 'secure-password-here'
});
```

#### Password Requirements

- **Minimum Length**: 12 characters
- **Complexity**: Mix of letters, numbers, symbols
- **Strength Meter**: Real-time feedback during entry
- **No Common Passwords**: Checked against breach databases

**Good Passwords**:
- ✅ `MyDog@Home2024!Secure`
- ✅ `Tr0ub4dor&3-Extended`
- ✅ `correct-horse-battery-staple-2024`

**Bad Passwords**:
- ❌ `password123`
- ❌ `qwerty`
- ❌ `123456789`

### Login

Authenticating with existing credentials:

```typescript
const session = await invoke('login_user', {
  fourWords: 'ocean-forest-moon-star',
  password: 'secure-password-here'
});

console.log('Session token:', session.token);
console.log('Expires:', session.expiresAt);
```

### Changing Password

```typescript
await invoke('change_password', {
  currentPassword: 'old-password',
  newPassword: 'new-secure-password'
});
```

**Security Notes**:
- Current password required for verification
- New password must meet strength requirements
- All active sessions invalidated (except current)
- Automatic re-encryption of stored data

### Password Recovery

⚠️ **Important**: Communitas is decentralized with no password recovery service. If you lose your password, you lose access to your encrypted data.

**Backup Options**:
1. **Export Identity**: Encrypted backup file
2. **Write Down Recovery Phrase**: 24-word mnemonic
3. **Store in Password Manager**: Use 1Password, Bitwarden, etc.

```typescript
// Export encrypted identity backup
const backup = await invoke('export_identity', {
  password: 'secure-password'
});

// Save backup.json file securely
```

## Passkey Authentication

### What is a Passkey?

Passkeys are a modern authentication method using public-key cryptography and biometrics:

- **No passwords to remember** - Biometrics or device PIN
- **Phishing resistant** - Cryptographically bound to domain
- **Fast and convenient** - Touch ID, Face ID, fingerprint
- **Post-quantum secure** - Uses ML-DSA signatures

### Enabling Passkey

#### During Registration

```typescript
const result = await invoke('register_user_with_passkey', {
  fourWords: 'ocean-forest-moon-star',
  displayName: 'Alice',
  deviceName: 'MacBook Pro'
});
```

This will:
1. Generate post-quantum key pairs
2. Create passkey with platform authenticator
3. Store encrypted credentials in system keyring
4. Enable biometric authentication

#### For Existing Account

```typescript
await invoke('enable_passkey_authentication');
```

Platform-specific prompts:
- **macOS**: Touch ID or password
- **Windows**: Windows Hello PIN, fingerprint, or face
- **Linux**: Fingerprint reader or system password

### Authenticating with Passkey

```typescript
const session = await invoke('authenticate_with_passkey', {
  fourWords: 'ocean-forest-moon-star'
});
```

**User Experience**:
```
┌────────────────────────────────────┐
│  Authenticate with Touch ID        │
│                                    │
│  Place your finger on the          │
│  Touch ID sensor                   │
│                                    │
│  [Touch ID Icon]                   │
│                                    │
│  ocean-forest-moon-star            │
└────────────────────────────────────┘
```

### Managing Passkeys

```typescript
// List registered passkeys
const passkeys = await invoke('list_passkeys');

// Remove passkey
await invoke('remove_passkey', {
  passkeyId: 'passkey-id-here'
});

// Update passkey (re-register)
await invoke('update_passkey');
```

### Passkey Storage

Passkeys are stored securely using platform-specific credential managers:

**macOS**: Keychain
```bash
# View stored passkeys
security find-generic-password -s "communitas"
```

**Windows**: Windows Credential Manager
```powershell
# View stored credentials
cmdkey /list | findstr communitas
```

**Linux**: Secret Service API (libsecret)
```bash
# View stored secrets
secret-tool search service communitas
```

## Platform-Specific Authentication

### macOS: Touch ID

#### Setup

1. Ensure Touch ID is configured in System Preferences
2. Enable in Communitas: Settings → Security → Touch ID
3. Authenticate once with password to bind

#### Code Example

```rust
use security_framework::item::*;

pub async fn authenticate_touch_id() -> Result<bool> {
    let context = LAContext::new()?;

    context.can_evaluate_policy(
        LAPolicy::DeviceOwnerAuthenticationWithBiometrics
    )?;

    let result = context.evaluate_policy(
        LAPolicy::DeviceOwnerAuthenticationWithBiometrics,
        "Authenticate to access Communitas"
    ).await?;

    Ok(result)
}
```

#### Troubleshooting

**Touch ID not available**:
- Check System Preferences → Touch ID
- Ensure at least one fingerprint enrolled
- Restart Communitas if just enabled

**Authentication fails**:
- Try different finger
- Clean Touch ID sensor
- Verify in System Preferences → Touch ID

### Windows: Windows Hello

#### Setup

1. Configure Windows Hello in Settings
2. Choose: PIN, Fingerprint, or Face recognition
3. Enable in Communitas: Settings → Security → Windows Hello

#### Code Example

```rust
use windows::Security::Credentials::UI::*;

pub async fn authenticate_windows_hello() -> Result<bool> {
    let options = UserConsentVerifierAvailability::Available;

    let result = UserConsentVerifier::RequestVerificationAsync(
        "Authenticate to access Communitas"
    )?.await?;

    Ok(result.status() == UserConsentVerificationResult::Verified)
}
```

### Linux: Fingerprint Reader

#### Setup

1. Install fprintd: `sudo apt install fprintd`
2. Enroll fingerprints: `fprintd-enroll`
3. Enable in Communitas: Settings → Security → Fingerprint

#### Code Example

```rust
use libsecret::*;

pub async fn store_credential(
    four_words: &str,
    password: &str
) -> Result<()> {
    let schema = Schema::new(
        "com.saorsalabs.communitas",
        SchemaFlags::NONE,
        vec![("four_words", SchemaAttributeType::String)]
    );

    password_store_sync(
        Some(&schema),
        vec![("four_words", four_words)],
        Some("Communitas"),
        password,
        None
    )?;

    Ok(())
}
```

## Session Management

### Session Lifecycle

```
Registration/Login
       ↓
Create Session (Token + Expiry)
       ↓
Active Session (24 hours default)
       ↓
Session Refresh (automatic)
       ↓
Session Expiry → Re-authenticate
```

### Session Tokens

```typescript
interface SessionInfo {
  token: string;           // Session token
  fourWords: string;       // User's four-word address
  displayName: string;     // Display name
  entityId: string;        // Entity ID
  expiresAt: number;       // Unix timestamp
  refreshToken: string;    // Refresh token
}
```

### Automatic Session Refresh

Sessions automatically refresh before expiry:

```typescript
// Configure auto-refresh
await invoke('configure_session', {
  autoRefresh: true,
  refreshBeforeExpiry: 300 // 5 minutes
});
```

### Manual Session Management

```typescript
// Get current session
const session = await invoke('get_current_session');

// Refresh session
const newSession = await invoke('refresh_session', {
  refreshToken: session.refreshToken
});

// End session (logout)
await invoke('logout');
```

### Multiple Sessions

Each device/browser gets its own session:

```typescript
// List active sessions
const sessions = await invoke('list_sessions');

// Revoke specific session
await invoke('revoke_session', {
  sessionId: 'session-id-here'
});

// Revoke all other sessions
await invoke('revoke_other_sessions');
```

## Two-Factor Authentication (Coming Soon)

Future support for additional authentication factors:

### Time-Based OTP (TOTP)

```typescript
// Enable TOTP
const secret = await invoke('enable_totp');
// → Returns QR code for authenticator app

// Verify TOTP
await invoke('verify_totp', {
  code: '123456'
});
```

### Hardware Security Keys

```typescript
// Register FIDO2 key
await invoke('register_security_key');

// Authenticate with key
await invoke('authenticate_security_key');
```

## Security Best Practices

### For Users

1. **Use Strong Passwords**
   - Minimum 12 characters
   - Mix of letters, numbers, symbols
   - Unique to Communitas

2. **Enable Passkeys**
   - More secure than passwords
   - Faster authentication
   - Phishing resistant

3. **Regular Backups**
   - Export identity monthly
   - Store backup securely
   - Test recovery process

4. **Monitor Sessions**
   - Review active sessions regularly
   - Revoke unknown sessions
   - Log out when done

5. **Physical Security**
   - Lock screen when away
   - Encrypt device storage
   - Use secure devices

### For Developers

1. **Never Store Passwords**
   - Hash with Argon2id
   - Use platform keyring
   - Zero passwords in logs

2. **Validate Input**
   - Check password strength
   - Sanitize four-word addresses
   - Rate limit login attempts

3. **Secure Sessions**
   - Use secure random tokens
   - Set appropriate expiry
   - Implement refresh flow

4. **Audit Authentication**
   - Log auth attempts
   - Monitor for anomalies
   - Alert on suspicious activity

## API Reference

### Registration

```typescript
// Password-based registration
await invoke('register_user', {
  fourWords: string,
  displayName: string,
  deviceName: string,
  password: string
});

// Passkey-based registration
await invoke('register_user_with_passkey', {
  fourWords: string,
  displayName: string,
  deviceName: string
});
```

### Login

```typescript
// Password login
await invoke('login_user', {
  fourWords: string,
  password: string
});

// Passkey login
await invoke('authenticate_with_passkey', {
  fourWords: string
});

// Platform biometric login (macOS)
await invoke('authenticate_with_touchid');
```

### Session Management

```typescript
// Get current session
await invoke('get_current_session');

// Refresh session
await invoke('refresh_session', {
  refreshToken: string
});

// Logout
await invoke('logout');

// List all sessions
await invoke('list_sessions');

// Revoke session
await invoke('revoke_session', {
  sessionId: string
});
```

### Password Management

```typescript
// Change password
await invoke('change_password', {
  currentPassword: string,
  newPassword: string
});

// Reset password (with recovery phrase)
await invoke('reset_password', {
  recoveryPhrase: string[],
  newPassword: string
});
```

### Passkey Management

```typescript
// Enable passkey
await invoke('enable_passkey_authentication');

// Disable passkey
await invoke('disable_passkey_authentication');

// List passkeys
await invoke('list_passkeys');

// Remove passkey
await invoke('remove_passkey', {
  passkeyId: string
});
```

## Troubleshooting

### Common Issues

#### "Invalid Credentials"

- Verify four-word address is correct
- Check password for typos
- Ensure Caps Lock is off
- Try password recovery if available

#### "Passkey Not Available"

- Check biometric sensor is working
- Verify in system settings
- Re-enroll biometrics if needed
- Fall back to password

#### "Session Expired"

- Normal after 24 hours inactive
- Simply log in again
- Enable "Remember Me" for longer sessions
- Check system clock is correct

#### "Too Many Login Attempts"

- Rate limiting active (security feature)
- Wait 5 minutes and retry
- Check for keyloggers if persistent
- Contact support if locked out

### Debug Logging

Enable authentication debug logs:

```bash
RUST_LOG=communitas_core::auth_service=debug \
  npm run tauri dev
```

Check logs:
```bash
tail -f ~/Library/Application\ Support/communitas/logs/auth.log
```

## Migration Guide

### From Password to Passkey

```typescript
// 1. Verify current password
const session = await invoke('login_user', {
  fourWords: 'ocean-forest-moon-star',
  password: 'current-password'
});

// 2. Enable passkey
await invoke('enable_passkey_authentication');

// 3. Test passkey login
await invoke('logout');
await invoke('authenticate_with_passkey', {
  fourWords: 'ocean-forest-moon-star'
});

// 4. Optional: Remove password requirement
await invoke('disable_password_authentication');
```

### Importing from Another Device

```typescript
// 1. Export from original device
const backup = await invoke('export_identity', {
  password: 'secure-password'
});

// 2. Save backup.json

// 3. Import on new device
await invoke('import_identity', {
  backupData: backupJson,
  password: 'secure-password',
  deviceName: 'New Device'
});

// 4. Re-enable passkey on new device
await invoke('enable_passkey_authentication');
```

## See Also

- [Getting Started](getting-started.md) - Initial setup
- [Four-Word Addresses](four-word-addresses.md) - Identity system deep dive
- [Security Architecture](../architecture/security.md) - How security works
- [API Documentation](../api/) - Complete API reference
- [Desktop README](../../communitas-desktop/README.md) - Platform-specific features

---

**Stay secure! Use strong authentication and keep your credentials safe! 🔐**

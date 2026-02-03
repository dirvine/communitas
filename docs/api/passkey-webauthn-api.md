# WebAuthn/Passkey Authentication API

**Status:** Deferred (passkeys removed from app as of 2026-02-02)
**Last Updated:** February 2, 2026
**Module:** `communitas-core::encrypted_storage`

## Overview

The WebAuthn/Passkey API enables biometric authentication (Touch ID, Face ID, Windows Hello) for Communitas identities. It implements the W3C WebAuthn standard adapted for local-first, offline-capable applications.

### Key Features

- **Biometric Authentication:** Touch ID, Face ID, Windows Hello
- **Offline-First:** Works without internet connection
- **Platform Integration:** Uses native platform keyrings (Keychain, Credential Manager, Secret Service)
- **Zero-Knowledge:** Server never sees credentials
- **Multi-Device:** Register different passkeys on different devices

## Architecture

```
┌─────────────────────────────────────────────────────┐
│            User Interface Layer (Dioxus)            │
│  PasskeyPrompt / IdentitySwitcher Components       │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│           AuthService (Public API)                  │
│  - passkey_start_registration()                     │
│  - passkey_finish_registration()                    │
│  - passkey_start_authentication()                   │
│  - passkey_finish_authentication()                  │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│      WebAuthnHandler (Protocol Layer)               │
│  - Uses webauthn-rs for W3C compliance              │
│  - Local RP ID: "communitas.local"                  │
│  - User verification required                       │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│     PasskeyManager + Platform Keyring              │
│  - Metadata: JSON files in vault                    │
│  - Credentials: Platform keyring                    │
└─────────────────────────────────────────────────────┘
```

## Core Types

### WebAuthnHandler

Main entry point for WebAuthn operations.

```rust
pub struct WebAuthnHandler {
    // Relying Party configuration
    config: WebAuthnConfig,
}

impl WebAuthnHandler {
    /// Create handler with default configuration
    pub fn new() -> Result<Self>

    /// Create handler with custom configuration
    pub fn with_config(config: WebAuthnConfig) -> Result<Self>

    /// Get Relying Party ID
    pub fn rp_id(&self) -> &str

    /// Start passkey registration ceremony
    pub fn start_registration(
        &self,
        user_id: &str,
        display_name: &str,
        resident_keys: &[PasskeyRegistration],
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration)>

    /// Complete passkey registration
    pub fn finish_registration(
        &self,
        response: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<Passkey>

    /// Start passkey authentication ceremony
    pub fn start_authentication(
        &self,
        passkeys: &[Passkey],
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication)>

    /// Complete passkey authentication
    pub fn finish_authentication(
        &self,
        response: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> Result<FinishAuthenticationResponse>
}
```

### WebAuthnConfig

Configuration for the WebAuthn handler.

```rust
pub struct WebAuthnConfig {
    /// Relying Party ID (must match origin domain)
    pub rp_id: String,

    /// Relying Party display name
    pub rp_name: String,

    /// Origin URL for credential binding
    pub rp_origin: Url,
}

impl WebAuthnConfig {
    /// Create default config for Communitas
    /// - rp_id: "communitas.local"
    /// - rp_name: "Communitas"
    /// - rp_origin: "https://communitas.local"
    pub fn new() -> Result<Self>
}
```

### PasskeyInfo

Metadata about a registered passkey.

```rust
pub struct PasskeyInfo {
    /// Four-word identity this passkey is for
    pub four_words: String,

    /// When the passkey was registered (Unix timestamp)
    pub registered_at: u64,

    /// Last successful authentication (Unix timestamp)
    pub last_used: Option<u64>,

    /// Device name (e.g., "MacBook Pro Touch ID")
    pub device_name: String,

    /// Actual WebAuthn credential (stored in keyring)
    pub webauthn_credential: Option<WebAuthnCredential>,
}
```

### WebAuthnCredential

Serializable WebAuthn credential data.

```rust
pub struct WebAuthnCredential {
    /// Unique credential ID
    pub id: String,

    /// Raw credential ID bytes
    pub raw_id: Vec<u8>,

    /// Credential type (always "public-key")
    pub credential_type: String,

    /// Attestation object (for verification if needed)
    pub attestation_object: Vec<u8>,

    /// Client data JSON (for verification if needed)
    pub client_data_json: Vec<u8>,
}
```

## AuthService API

The high-level authentication API used by frontends.

### Registration Flow

```rust
pub async fn passkey_start_registration(
    &self,
    four_words: &str,
    display_name: &str,
) -> Result<webauthn_rs::prelude::CreationChallengeResponse>
```

Returns a challenge that should be sent to the client's authenticator (browser WebAuthn API or platform biometric).

```rust
pub async fn passkey_finish_registration(
    &mut self,
    four_words: &str,
    device_name: &str,
    response: &webauthn_rs::prelude::RegisterPublicKeyCredential,
    state: &webauthn_rs::prelude::PasskeyRegistration,
) -> Result<PasskeyInfo>
```

Completes registration with the authenticator's response. Returns `PasskeyInfo` with the registered credential.

**Error Cases:**
- `"WebAuthn not available"` - WebAuthn handler failed to initialize
- `"Credential already exists for this device"` - Passkey already registered for this identity
- Keyring errors - Platform keyring unavailable
- Serialization errors - Corrupted credential data

### Authentication Flow

```rust
pub async fn passkey_start_authentication(
    &self,
    four_words: &str,
) -> Result<(
    webauthn_rs::prelude::RequestChallengeResponse,
    webauthn_rs::prelude::PasskeyAuthentication,
)>
```

Returns a challenge for the authenticator. State must be passed to `passkey_finish_authentication`.

**Important:** The state object must be preserved (e.g., stored in session) until authentication is complete. It's needed to verify the authenticator's response.

```rust
pub async fn passkey_finish_authentication(
    &mut self,
    four_words: &str,
    response: &webauthn_rs::prelude::PublicKeyCredential,
    state: &webauthn_rs::prelude::PasskeyAuthentication,
) -> Result<SessionInfo>
```

Completes authentication with the authenticator's response. Returns a new session.

**Error Cases:**
- `"WebAuthn not available"` - WebAuthn handler failed to initialize
- `"No WebAuthn credential found for this identity"` - No passkey registered
- `"Authentication failed"` - Authenticator verification failed
- `"Invalid state"` - State doesn't match stored state

### Utility Methods

```rust
pub fn webauthn_available(&self) -> bool
```

Check if WebAuthn is available (handler initialized successfully).

```rust
pub async fn passkey_has_passkey(&self, four_words: &str) -> Result<bool>
```

Check if an identity has a registered passkey.

```rust
pub async fn passkey_delete(&self, four_words: &str) -> Result<()>
```

Delete a passkey for an identity. This removes it from both file storage and platform keyring.

## State Management

### Challenge States

During registration and authentication, the client must preserve state objects:

```
Registration:
  1. Client calls passkey_start_registration()
  2. Server returns CreationChallengeResponse + PasskeyRegistration state
  3. State must be stored (in session, cookie, etc.)
  4. Authenticator challenges user for biometric
  5. Authenticator returns signed response
  6. Client calls passkey_finish_registration() with state

Authentication:
  1. Client calls passkey_start_authentication()
  2. Server returns RequestChallengeResponse + PasskeyAuthentication state
  3. State must be stored (in session, cookie, etc.)
  4. Authenticator challenges user for biometric
  5. Authenticator returns signed response
  6. Client calls passkey_finish_authentication() with state
```

**Security Note:** State is NOT sensitive - it contains public challenge data. It's safe to transmit over HTTP or store in cookies.

## Platform Keyring Storage

Credentials are stored in the platform's native keyring for maximum security:

| Platform | Keyring | Benefits |
|----------|---------|----------|
| macOS | Keychain | Hardware-protected, biometric integrated |
| Windows | Credential Manager | Hardware-protected (TPM when available) |
| Linux | Secret Service / Pass | Software-protected, standard interface |

### Storage Layout

```
File Storage (encrypted vault):
  ~/.communitas/[identity]/passkeys/
    - passkey_info.json          # Metadata
    - passkey_events.json        # Audit log

Platform Keyring:
  Service: "com.saorsalabs.communitas.passkey"
  Account: "[four-words]"
  Password: base64(json(WebAuthnCredential))
```

## Error Handling

All APIs return `Result<T>` using `anyhow::Error` with context:

```rust
// Recommended pattern
match auth_service.passkey_start_registration(four_words, name).await {
    Ok(challenge) => {
        // Send challenge to authenticator
    }
    Err(e) => {
        // Log error
        tracing::error!("Passkey registration failed: {}", e);

        // Show user-friendly message
        let user_msg = match e.kind {
            ErrorKind::WebAuthnNotAvailable =>
                "This device doesn't support passkey registration",
            ErrorKind::Keyring =>
                "Failed to access device keyring",
            _ => "Registration failed, please try again"
        };
    }
}
```

## Usage Examples

### Basic Registration (Dioxus Component)

```rust
use communitas_ui_service::auth::AuthService;

#[component]
fn PasskeyRegistration() -> Element {
    let mut auth = use_auth();
    let mut challenge_state = use_signal(None);

    let on_register = move |_| {
        spawn({
            let auth = auth.clone();
            async move {
                match auth.passkey_start_registration(
                    "ocean-forest-moon-star",
                    "My MacBook"
                ).await {
                    Ok(challenge) => {
                        challenge_state.set(Some(challenge));
                        // Show passkey prompt to user
                    }
                    Err(e) => {
                        tracing::error!("Registration failed: {}", e);
                    }
                }
            }
        });
    };

    rsx! {
        button {
            onclick: on_register,
            "Register Passkey"
        }
    }
}
```

### Basic Authentication

```rust
match auth_service.passkey_start_authentication(four_words).await {
    Ok((challenge, state)) => {
        // Send challenge to platform authenticator
        // On success, call passkey_finish_authentication
        match auth_service.passkey_finish_authentication(
            four_words,
            &response,
            &state
        ).await {
            Ok(session) => {
                // User authenticated, create session
            }
            Err(e) => {
                // Authentication failed
            }
        }
    }
    Err(e) => {
        // Start authentication failed
    }
}
```

## Security Considerations

### What Passkeys Protect

- ✅ Biometric authentication (Touch ID, Face ID, etc.)
- ✅ Device compromise - passkey is bound to specific device/platform
- ✅ Phishing - authenticator verifies RP ID, not URL
- ✅ Shoulder surfing - biometric required even with stolen device

### What Passkeys Don't Protect

- ❌ Compromised platform keyring - if attacker accesses keyring, they can use passkey
- ❌ Device theft + unlocked device - passkey accessible without biometric
- ❌ Logical compromise - if device malware proxies to authenticator, could capture auth

### Best Practices

1. **Enable Device Lock:** Always lock device when not in use
2. **Regular Backups:** Back up recovery codes if available
3. **Multiple Devices:** Register passkeys on multiple devices for redundancy
4. **Monitor Sessions:** Check active sessions regularly
5. **Audit Logs:** Review passkey usage logs for suspicious activity

## Limitations

### Current Implementation

- **No Cloned Device Detection:** Counter not currently tracked
- **No Resident Keys:** Credentials stored server-side only (expected for local-first)
- **No Attestation Validation:** Assumes all authenticators are trusted
- **Single RP:** All Communitas instances use "communitas.local" RP ID

### Future Enhancements

- Counter tracking for cloned device detection
- Conditional UI for username-less flows
- Support for cross-origin authentication
- Backup passkey restoration

## Testing

### Unit Tests

```bash
# Test core WebAuthn functionality
cargo test -p communitas-core encrypted_storage::webauthn::tests

# Test passkey manager
cargo test -p communitas-core encrypted_storage::passkey::tests
```

### Integration Tests

```bash
# Test complete registration/authentication flows
cargo test --test passkey_integration

# Test UI service layer
cargo test --test auth_passkey_integration
```

### Manual Testing

For complete authentication flow testing with actual authenticators:

```bash
# Enable platform keyring in tests
cargo test --test passkey_integration -- --ignored

# UI testing with real device authenticators
cd communitas-dioxus
dx serve --platform desktop
# Use PasskeyPrompt component with real Touch ID/Face ID
```

## Migration from Legacy Passkeys

If using the legacy passkey implementation (without WebAuthn):

1. **Backward Compatible:** Legacy credentials still work
2. **Migration Path:** Users can register new WebAuthn passkeys alongside legacy ones
3. **Deprecation:** Plan to deprecate legacy passkeys in Phase 8.4

## Related Documentation

- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/)
- [User Guide: Passkey Registration](./guides/passkey-registration.md)
- [User Guide: Passkey Authentication](./guides/passkey-authentication.md)
- [Troubleshooting Guide](./guides/passkey-troubleshooting.md)
- [Security Best Practices](./guides/passkey-security.md)

## Support & Issues

For bugs or issues:
- GitHub Issues: https://github.com/saorsa-labs/communitas/issues
- Security Issues: security@saorsalabs.com
- Documentation Issues: docs@saorsalabs.com

---

**Last Updated:** January 25, 2026
**Phase:** 8.3 (Documentation)
**Status:** Ready for Production

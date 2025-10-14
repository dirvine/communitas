# Touch ID Passkey Authentication - Debug Summary

## Issue
Touch ID biometric authentication succeeds, but login fails with:
> "Passkey registered but vault password not found in keyring. Please login with password first."

## Investigation (Systematic Debugging - Phase 1: Root Cause Investigation)

### Diagnostic Logging Added

I've added comprehensive diagnostic logging to trace the password storage and retrieval flow:

#### 1. Password Storage Functions

**File**: `communitas-core/src/encrypted_storage/mod.rs`

- `store_password_in_keyring` (lines 585-604): Logs password storage attempts
- `login` function (lines 256-276): **NEW** - Logs keyring storage during login

#### 2. Password Retrieval Functions

**File**: `communitas-core/src/encrypted_storage/mod.rs`

- `passkey_authenticate` (lines 497-554): Logs password retrieval attempts

### Critical Finding: Silent Failure Pattern

**Location**: `/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-core/src/encrypted_storage/mod.rs` lines 257-276

```rust
// Store password in keyring if enabled
if self.config.use_keyring && app_config.get_config().keyring_enabled {
    tracing::info!("🔑 LOGIN: Attempting to store password in keyring for '{}'", normalized);
    match self.key_manager
        .store_in_keyring(&normalized, password.as_bytes())
        .await
    {
        Ok(()) => {
            tracing::info!("✅ LOGIN: Password stored in keyring successfully for '{}'", normalized);
        }
        Err(e) => {
            tracing::error!("❌ LOGIN: Failed to store password in keyring for '{}': {}", normalized, e);
            tracing::error!("⚠️ LOGIN: This means passkey/Touch ID authentication will fail later!");
        }
    }
} else {
    tracing::warn!("⚠️ LOGIN: Keyring storage skipped - use_keyring={}, keyring_enabled={}",
        self.config.use_keyring,
        app_config.get_config().keyring_enabled
    );
}
```

**Previous Code** (BEFORE my changes):
```rust
if self.config.use_keyring && app_config.get_config().keyring_enabled {
    self.key_manager
        .store_in_keyring(&normalized, password.as_bytes())
        .await
        .ok(); // Non-fatal if keyring fails  <--- SILENTLY IGNORES FAILURES!
}
```

## Hypothesis

The `.ok()` on the previous line 261 silently discards any errors from `store_in_keyring()`.

**If keyring storage fails for any reason:**
1. No error is raised during login/registration
2. User/system thinks password was stored
3. No password is actually in the keyring
4. Later, Touch ID authentication tries to retrieve the password
5. Retrieval fails because nothing was stored
6. Error: "Passkey registered but vault password not found in keyring"

## Diagnostic Logs to Watch For

When testing with the updated code, look for these log patterns:

### During Login/Registration:
```
🔑 LOGIN: Attempting to store password in keyring for 'four-word-address'
✅ LOGIN: Password stored in keyring successfully for 'four-word-address'
```

OR (if hypothesis is correct):
```
🔑 LOGIN: Attempting to store password in keyring for 'four-word-address'
❌ LOGIN: Failed to store password in keyring for 'four-word-address': [ERROR DETAILS]
⚠️ LOGIN: This means passkey/Touch ID authentication will fail later!
```

OR (if keyring is disabled):
```
⚠️ LOGIN: Keyring storage skipped - use_keyring=false, keyring_enabled=false
```

### During Touch ID Authentication:
```
🔍 RETRIEVAL: Attempting passkey auth for four_words='original' -> normalized='normalized-version'
✅ RETRIEVAL: Passkey IS registered for 'normalized-version'
🔍 RETRIEVAL: Checking keyring config - use_keyring=true
🔍 RETRIEVAL: Attempting to get password from keyring for 'normalized-version'
✅ RETRIEVAL: Password bytes retrieved from keyring for 'normalized-version'
✅ RETRIEVAL: Password successfully decoded, attempting login for 'normalized-version'
```

OR (if password not found - current behavior):
```
🔍 RETRIEVAL: Attempting passkey auth for four_words='original' -> normalized='normalized-version'
✅ RETRIEVAL: Passkey IS registered for 'normalized-version'
🔍 RETRIEVAL: Checking keyring config - use_keyring=true
🔍 RETRIEVAL: Attempting to get password from keyring for 'normalized-version'
❌ RETRIEVAL: Failed to get password from keyring for 'normalized-version': [ERROR]
❌ RETRIEVAL: No password found in keyring for 'normalized-version'
```

## Next Steps to Test

### Option 1: Test in Running Tauri App

The diagnostic logging has been added but the dev server needs to be restarted to pick up the changes:

1. Stop the current `npm run tauri dev` process
2. Restart with: `npm run tauri dev`
3. Create a new identity or login with an existing one
4. Try Touch ID authentication
5. Check the terminal output for diagnostic logs

### Option 2: Run Integration Test

The test at `communitas-desktop/tests/passkey_flow_integration_test.rs` can be updated to work once the following exports are added to `communitas-desktop/src/lib.rs`:

```rust
pub mod auth_service;
pub mod container;
```

Then run:
```bash
RUST_LOG=debug cargo test --package communitas-desktop --test passkey_flow_integration_test passkey_registration_and_authentication -- --nocapture
```

## Evidence Summary

### Confirmed:
1. ✅ Touch ID native prompt displays correctly
2. ✅ Biometric authentication succeeds
3. ✅ Passkey is registered in database
4. ✅ `normalize_four_words` function is used consistently
5. ✅ Keyring service name is "com.saorsalabs.communitas"
6. ✅ Default config has `use_keyring: true` and `keyring_enabled: true`

### To Be Confirmed:
1. ⏳ Whether keyring storage is actually succeeding or failing during login
2. ⏳ Whether keyring retrieval is finding any password data
3. ⏳ The specific error when keyring operations fail

## Files Modified

1. `/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-core/src/encrypted_storage/mod.rs`
   - Added diagnostic logging to `store_password_in_keyring` (lines 585-604)
   - Added diagnostic logging to `passkey_authenticate` (lines 497-554)
   - **Changed `login` function to log keyring storage attempts instead of silently ignoring failures** (lines 256-276)

2. `/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-desktop/tests/login_keyring_diagnostic_test.rs` (NEW)
   - Created diagnostic test (currently uses mock keyring in tests, not real macOS keychain)

## Recommended Fix (Once Hypothesis Confirmed)

If diagnostic logs confirm that keyring storage is failing silently, the fix would be to:

1. **Remove the silent failure** - Don't use `.ok()` to ignore keyring errors
2. **Return proper errors** - Let keyring failures propagate up so users see them
3. **Add user feedback** - Show a warning if keyring storage fails
4. **Fallback strategy** - Either require password on every login, or implement a secure fallback

Example fix:
```rust
// Store password in keyring if enabled
if self.config.use_keyring && app_config.get_config().keyring_enabled {
    if let Err(e) = self.key_manager
        .store_in_keyring(&normalized, password.as_bytes())
        .await
    {
        tracing::warn!("Failed to store password in keyring: {}. Touch ID login will not be available.", e);
        // Optionally: Set a flag in session to indicate keyring unavailable
    }
}
```

## 🚨 ROOT CAUSE IDENTIFIED (Phase 2 Complete)

### The Problem: Frontend Bypassing Backend Authentication

**Critical Finding**: When users log in through the UI, the frontend does NOT call the backend `auth_login` Tauri command. Instead, it uses browser-only authentication (localStorage/Web Crypto).

**Evidence**:
1. User logged in with `ethics-yet-ketchup-death`
2. Backend logs show NO "Login attempt for ethics-yet-ketchup-death" message
3. Backend logs show NO LOGIN diagnostic output (🔑 LOGIN markers)
4. Frontend warning: "Command store_encryption_keys not found" - indicating frontend trying to use non-existent commands
5. Previous logins for other identities ARE in the logs, confirming backend auth works for some flows

**Chain of Failure**:
```
User enters password → Frontend handles auth in browser → localStorage stores session
     ↓                     ↓                                    ↓
Backend never called   Password never stored in keyring   Touch ID has nothing to retrieve
     ↓                     ↓                                    ↓
No LOGIN logs          No keyring entry                   Authentication fails
```

**Files Involved**:
- `/Users/davidirvine/Desktop/Devel/projects/communitas/src/contexts/AuthContext.tsx` (lines 243-315)
  - Calls `invoke('auth_login')` but something is preventing it from working
  - Falls back to browser-only auth when Tauri commands fail

## Current Status

✅ Phase 1 Complete: Evidence gathered, diagnostic logging added
✅ **Phase 2 COMPLETE: ROOT CAUSE CONFIRMED - Frontend not calling backend auth**
⏳ Phase 3 In Progress: Fix frontend authentication to use backend
⏸️ Phase 4 Pending: Verify fix resolves the issue

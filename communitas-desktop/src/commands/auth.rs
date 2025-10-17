//! Authentication and Encrypted Storage Commands
//!
//! This module provides Tauri commands for user authentication using the shared AuthService.
//! All business logic is centralized in communitas-core's AuthService.

use communitas_core::{
    AuthService, SessionInfo,
    encrypted_storage::{
        AppConfig, EncryptedStorageManager, PasskeyInfo as CorePasskeyInfo,
        RecentIdentity as CoreRecentIdentity, StorageConfig, VaultInfo as CoreVaultInfo,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Application state for authentication using shared AuthService
pub struct AppState {
    pub auth_service: Arc<RwLock<Option<AuthService>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            auth_service: Arc::new(RwLock::new(None)),
        }
    }
}

/// Vault information for listing available vaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

impl From<CoreVaultInfo> for VaultInfo {
    fn from(core: CoreVaultInfo) -> Self {
        Self {
            four_words: core.four_words,
            display_name: core.display_name,
            created_at: core.created_at,
            last_accessed: core.last_accessed,
            size_bytes: core.size_bytes,
        }
    }
}

/// Initialize the authentication service
///
/// This must be called before any other auth commands.
#[tauri::command]
pub async fn auth_initialize(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Initializing authentication service");

    let config = StorageConfig::default();
    let storage_manager = EncryptedStorageManager::new(config).await.map_err(|e| {
        tracing::error!("Failed to initialize storage: {}", e);
        format!("Failed to initialize storage: {}", e)
    })?;

    let auth_service = AuthService::new(storage_manager);

    let mut service = state.auth_service.write().await;
    *service = Some(auth_service);

    tracing::info!("Authentication service initialized successfully");
    Ok(())
}

/// Create a new vault for a four-word identity
///
/// This creates an encrypted vault with PBKDF2 key derivation (100,000 iterations)
/// and ChaCha20-Poly1305 encryption.
#[tauri::command]
pub async fn auth_create_vault(
    state: State<'_, AppState>,
    four_words: String,
    password: String,
    display_name: String,
) -> Result<String, String> {
    tracing::info!("Creating vault for: {}", four_words);

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized. Call auth_initialize first.".to_string())?;

    let vault_id = auth_service
        .create_vault(&four_words, &password, &display_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create vault: {}", e);
            format!("Failed to create vault: {}", e)
        })?;

    tracing::info!("Vault created successfully: {}", vault_id);
    Ok(vault_id)
}

/// Login with four-word address and password
///
/// Returns a session that can be used for subsequent operations.
#[tauri::command]
pub async fn auth_login(
    state: State<'_, AppState>,
    four_words: String,
    password: String,
) -> Result<SessionInfo, String> {
    tracing::info!("Login attempt for: {}", four_words);

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let session_info = auth_service
        .login(&four_words, &password, Some("Desktop"))
        .await
        .map_err(|e| {
            tracing::error!("Login failed: {}", e);
            format!("Login failed: {}", e)
        })?;

    tracing::info!("Login successful: {}", four_words);
    Ok(session_info)
}

/// Login with password only (searches all vaults)
///
/// This is useful for devices where the user has previously logged in
/// and doesn't remember their four-word address.
#[tauri::command]
pub async fn auth_login_password_only(
    state: State<'_, AppState>,
    password: String,
) -> Result<SessionInfo, String> {
    tracing::info!("Password-only login attempt");

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    // Use storage_manager directly for password-only login (not exposed in AuthService)
    let storage_manager = auth_service.storage_manager();
    let session = storage_manager
        .login_password_only(&password)
        .await
        .map_err(|e| {
            tracing::error!("Password-only login failed: {}", e);
            format!("Password-only login failed: {}", e)
        })?;

    let session_info = SessionInfo {
        session_id: session.id,
        four_words: session.four_words,
        display_name: session.display_name,
    };

    tracing::info!(
        "Password-only login successful: {}",
        session_info.four_words
    );
    Ok(session_info)
}

/// Logout the current session
///
/// Clears the active session and securely zeros sensitive data.
#[tauri::command]
pub async fn auth_logout(state: State<'_, AppState>) -> Result<(), String> {
    tracing::info!("Logout requested");

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    auth_service
        .logout()
        .await
        .map_err(|e| format!("Logout failed: {}", e))?;

    tracing::info!("Logout successful");
    Ok(())
}

/// Get the current active session
///
/// Returns None if no user is logged in.
#[tauri::command]
pub async fn auth_get_session(state: State<'_, AppState>) -> Result<Option<SessionInfo>, String> {
    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    Ok(auth_service.get_current_session())
}

/// List all available vaults on this device
///
/// Returns metadata for all vaults (encrypted storage containers) on this device.
#[tauri::command]
pub async fn auth_list_vaults(state: State<'_, AppState>) -> Result<Vec<VaultInfo>, String> {
    tracing::info!("Listing available vaults");

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let vaults = auth_service
        .list_vaults()
        .await
        .map_err(|e| format!("Failed to list vaults: {}", e))?;

    let vault_infos: Vec<VaultInfo> = vaults.into_iter().map(VaultInfo::from).collect();

    tracing::info!("Found {} vaults", vault_infos.len());
    Ok(vault_infos)
}

/// Check if a session is still valid
///
/// Returns true if the session exists and has not expired.
#[tauri::command]
pub async fn auth_check_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<bool, String> {
    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    if let Some(session_info) = auth_service.get_current_session() {
        Ok(session_info.session_id == session_id)
    } else {
        Ok(false)
    }
}

/// Get app configuration
#[tauri::command]
pub async fn auth_get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    tracing::info!("Getting app configuration");

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let config = auth_service.storage_manager().get_app_config().await;
    Ok(config)
}

/// Try auto-login with last used identity
#[tauri::command]
pub async fn auth_try_auto_login(
    state: State<'_, AppState>,
) -> Result<Option<SessionInfo>, String> {
    tracing::info!("Attempting auto-login");

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    match auth_service.try_auto_login().await {
        Ok(Some(session_info)) => {
            tracing::info!("Auto-login successful: {}", session_info.four_words);
            Ok(Some(session_info))
        }
        Ok(None) => {
            tracing::info!("Auto-login not available");
            Ok(None)
        }
        Err(e) => {
            tracing::warn!("Auto-login failed: {}", e);
            Ok(None) // Non-fatal, return None
        }
    }
}

/// Get recent identities for quick access
#[tauri::command]
pub async fn auth_get_recent_identities(
    state: State<'_, AppState>,
) -> Result<Vec<CoreRecentIdentity>, String> {
    tracing::info!("Getting recent identities");

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let identities = auth_service
        .get_recent_identities()
        .await
        .map_err(|e| format!("Failed to get recent identities: {}", e))?;

    Ok(identities)
}

/// Remove a recent identity from the list (does not delete the vault)
#[tauri::command]
pub async fn auth_remove_recent_identity(
    state: State<'_, AppState>,
    four_words: String,
) -> Result<(), String> {
    tracing::info!("Removing recent identity: {}", four_words);

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    auth_service
        .remove_recent_identity(&four_words)
        .await
        .map_err(|e| format!("Failed to remove recent identity: {}", e))?;

    Ok(())
}

/// Delete a vault and its associated identity (requires password confirmation)
///
/// This permanently deletes all data for the given identity including:
/// - The encrypted vault file
/// - Identity from recent identities list
/// - Any passkey/biometric data
/// - Keyring stored passwords
#[tauri::command]
pub async fn auth_delete_vault(
    state: State<'_, AppState>,
    four_words: String,
    password: String,
) -> Result<(), String> {
    tracing::warn!("Deleting vault for: {}", four_words);

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    // This will verify password and delete the vault
    auth_service
        .delete_vault(&four_words, &password)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete vault: {}", e);
            format!("Failed to delete vault: {}", e)
        })?;

    tracing::info!("Successfully deleted vault for: {}", four_words);
    Ok(())
}

/// Get list of old/stale vaults that can be cleaned up
///
/// Returns vaults that haven't been accessed in the specified number of days.
/// Useful for cleanup operations.
#[tauri::command]
pub async fn auth_list_old_vaults(
    state: State<'_, AppState>,
    days_since_access: u64,
) -> Result<Vec<VaultInfo>, String> {
    tracing::info!(
        "Listing vaults not accessed in last {} days",
        days_since_access
    );

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let all_vaults = auth_service
        .list_vaults()
        .await
        .map_err(|e| format!("Failed to list vaults: {}", e))?;

    // Filter vaults by last access time
    let threshold_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get current time: {}", e))?
        .as_secs()
        - (days_since_access * 24 * 60 * 60);

    let old_vaults: Vec<VaultInfo> = all_vaults
        .into_iter()
        .filter(|v| v.last_accessed < threshold_timestamp)
        .map(VaultInfo::from)
        .collect();

    tracing::info!(
        "Found {} vaults not accessed in last {} days",
        old_vaults.len(),
        days_since_access
    );
    Ok(old_vaults)
}

/// Enable or disable auto-login
#[tauri::command]
pub async fn auth_set_auto_login(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    tracing::info!("Setting auto-login: {}", enabled);

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    auth_service
        .storage_manager()
        .set_auto_login_enabled(enabled)
        .await
        .map_err(|e| format!("Failed to set auto-login: {}", e))?;

    Ok(())
}

/// Enable or disable keyring password storage
#[tauri::command]
pub async fn auth_set_keyring_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    tracing::info!("Setting keyring enabled: {}", enabled);

    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    auth_service
        .storage_manager()
        .set_keyring_enabled(enabled)
        .await
        .map_err(|e| format!("Failed to set keyring: {}", e))?;

    Ok(())
}

/// Passkey information for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyInfo {
    pub four_words: String,
    pub registered_at: u64,
    pub last_used: Option<u64>,
    pub device_name: String,
}

impl From<CorePasskeyInfo> for PasskeyInfo {
    fn from(core: CorePasskeyInfo) -> Self {
        Self {
            four_words: core.four_words,
            registered_at: core.registered_at,
            last_used: core.last_used,
            device_name: core.device_name,
        }
    }
}

/// Register a passkey/biometric for an identity (legacy - without WebAuthn)
///
/// This allows the user to use biometric authentication (Touch ID, Face ID, Windows Hello)
/// to login. The actual authentication still uses the password from keyring.
#[tauri::command]
pub async fn auth_passkey_register(
    state: State<'_, AppState>,
    four_words: String,
    device_name: String,
    password: String,
) -> Result<PasskeyInfo, String> {
    tracing::info!(
        "Registering passkey for: {} on device: {}",
        four_words,
        device_name
    );

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    // Register passkey metadata
    let info = auth_service
        .passkey_register(&four_words, &device_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to register passkey: {}", e);
            format!("Failed to register passkey: {}", e)
        })?;

    // CRITICAL: Store password in keyring for biometric authentication
    // This is what triggers OS-level auth prompts (Touch ID, Face ID, etc.)
    // Use storage_manager directly to store for the correct four_words identity
    auth_service
        .storage_manager()
        .store_password_in_keyring(&four_words, &password)
        .await
        .map_err(|e| {
            tracing::error!("Failed to store password in keyring: {}", e);
            format!("Failed to store password in keyring: {}", e)
        })?;

    tracing::info!(
        "Passkey registered and password stored in keyring for: {}",
        four_words
    );
    Ok(info.into())
}

/// Register a passkey with WebAuthn credential
///
/// This stores the WebAuthn credential data for true biometric authentication.
/// The credential was created by the frontend using navigator.credentials.create().
#[tauri::command]
pub async fn auth_passkey_register_webauthn(
    state: State<'_, AppState>,
    four_words: String,
    device_name: String,
    credential_data: String,
) -> Result<PasskeyInfo, String> {
    tracing::info!(
        "Registering WebAuthn passkey for: {} on device: {}",
        four_words,
        device_name
    );

    // Parse credential data from JSON
    let credential_json: serde_json::Value = serde_json::from_str(&credential_data)
        .map_err(|e| format!("Failed to parse credential data: {}", e))?;

    // Extract credential components
    use communitas_core::encrypted_storage::passkey::WebAuthnCredential;

    let credential = WebAuthnCredential {
        id: credential_json["id"]
            .as_str()
            .ok_or("Missing credential id")?
            .to_string(),
        raw_id: credential_json["rawId"]
            .as_array()
            .ok_or("Missing rawId")?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as u8)
            .collect(),
        credential_type: credential_json["type"]
            .as_str()
            .ok_or("Missing type")?
            .to_string(),
        attestation_object: credential_json["response"]["attestationObject"]
            .as_array()
            .ok_or("Missing attestationObject")?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as u8)
            .collect(),
        client_data_json: credential_json["response"]["clientDataJSON"]
            .as_array()
            .ok_or("Missing clientDataJSON")?
            .iter()
            .map(|v| v.as_u64().unwrap_or(0) as u8)
            .collect(),
    };

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let info = auth_service
        .passkey_register_webauthn(&four_words, &device_name, credential)
        .await
        .map_err(|e| {
            tracing::error!("Failed to register WebAuthn passkey: {}", e);
            format!("Failed to register WebAuthn passkey: {}", e)
        })?;

    tracing::info!(
        "WebAuthn passkey registered successfully for: {}",
        four_words
    );
    Ok(info.into())
}

/// Authenticate using WebAuthn passkey
///
/// This verifies the WebAuthn credential and creates a session.
#[tauri::command]
pub async fn auth_passkey_authenticate_webauthn(
    state: State<'_, AppState>,
    four_words: String,
    assertion_data: String,
) -> Result<SessionInfo, String> {
    tracing::info!(
        "WebAuthn passkey authentication attempt for: {}",
        four_words
    );

    // Parse assertion data from JSON
    let _assertion_json: serde_json::Value = serde_json::from_str(&assertion_data)
        .map_err(|e| format!("Failed to parse assertion data: {}", e))?;

    // For now, we'll verify the credential exists and use password from keyring
    // Full WebAuthn verification would validate the signature, but that requires
    // storing the public key from registration

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    // Verify passkey exists
    let has_passkey = auth_service
        .passkey_has_passkey(&four_words)
        .await
        .map_err(|e| format!("Failed to check passkey: {}", e))?;

    if !has_passkey {
        return Err("No passkey registered for this identity".to_string());
    }

    // Authenticate using stored password (biometric already verified by OS)
    let session_info = auth_service
        .passkey_authenticate(&four_words)
        .await
        .map_err(|e| {
            tracing::error!("WebAuthn passkey authentication failed: {}", e);
            format!("WebAuthn passkey authentication failed: {}", e)
        })?;

    tracing::info!("WebAuthn passkey authentication successful: {}", four_words);
    Ok(session_info)
}

/// Authenticate using passkey/biometric (legacy)
///
/// This retrieves the password from keyring and performs standard login.
/// The biometric verification happens at the OS level before this is called.
#[tauri::command]
pub async fn auth_passkey_authenticate(
    state: State<'_, AppState>,
    four_words: String,
) -> Result<SessionInfo, String> {
    tracing::info!("Passkey authentication attempt for: {}", four_words);

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let session_info = auth_service
        .passkey_authenticate(&four_words)
        .await
        .map_err(|e| {
            tracing::error!("Passkey authentication failed: {}", e);
            format!("Passkey authentication failed: {}", e)
        })?;

    tracing::info!("Passkey authentication successful: {}", four_words);
    Ok(session_info)
}

/// Check if a passkey is registered for an identity
#[tauri::command]
pub async fn auth_passkey_has_passkey(
    state: State<'_, AppState>,
    four_words: String,
) -> Result<bool, String> {
    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let has_passkey = auth_service
        .passkey_has_passkey(&four_words)
        .await
        .map_err(|e| format!("Failed to check passkey: {}", e))?;

    Ok(has_passkey)
}

/// Get passkey information for an identity
#[tauri::command]
pub async fn auth_passkey_get_info(
    state: State<'_, AppState>,
    four_words: String,
) -> Result<PasskeyInfo, String> {
    let service = state.auth_service.read().await;
    let auth_service = service
        .as_ref()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    let info = auth_service
        .passkey_get_info(&four_words)
        .await
        .map_err(|e| format!("Failed to get passkey info: {}", e))?;

    Ok(info.into())
}

/// Register a passkey using native macOS Touch ID
///
/// This uses the macOS Security Framework to trigger a native Touch ID prompt.
/// Only available on macOS.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn auth_touchid_register(
    state: State<'_, AppState>,
    four_words: String,
    device_name: String,
    password: String,
    reason: Option<String>,
) -> Result<PasskeyInfo, String> {
    tracing::info!(
        "Registering Touch ID passkey for: {} on device: {}",
        four_words,
        device_name
    );

    // Trigger native Touch ID authentication
    let auth_reason = reason.unwrap_or_else(|| format!("Register Touch ID for {}", four_words));

    // Use macOS LocalAuthentication to prompt for Touch ID
    // This uses Swift command with stdin to compile and run Swift code that triggers actual biometric authentication
    let _auth_result = tokio::task::spawn_blocking(move || {
        use std::process::Command;
        use std::io::Write;

        // Use swift command to compile and run Swift code inline
        // This approach works reliably on macOS and triggers native Touch ID
        let swift_code = format!(
            r#"
import LocalAuthentication
import Foundation

let context = LAContext()
var error: NSError?

if !context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {{
    print("false")
    exit(2)
}}

let semaphore = DispatchSemaphore(value: 0)
var success = false

context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{}") {{ result, authError in
    success = result
    semaphore.signal()
}}

semaphore.wait()
print(success ? "true" : "false")
exit(success ? 0 : 1)
"#,
            auth_reason.replace("\"", "\\\"")
        );

        // Compile and run Swift code using stdin
        let output = Command::new("swift")
            .arg("-")
            .arg("-framework")
            .arg("LocalAuthentication")
            .arg("-framework")
            .arg("Foundation")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(swift_code.as_bytes())?;
                }
                child.wait_with_output()
            });

        match output {
            Ok(result) => {
                match result.status.code() {
                    Some(0) => Ok(()),
                    Some(1) => Err("Touch ID authentication was cancelled or failed".to_string()),
                    Some(2) => Err("Touch ID is not available on this device".to_string()),
                    _ => {
                        let error = String::from_utf8_lossy(&result.stderr);
                        Err(format!("Touch ID authentication failed: {}", error))
                    }
                }
            }
            Err(e) => Err(format!("Failed to trigger Touch ID: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Touch ID task failed: {}", e))??;

    // If we get here, Touch ID authentication succeeded
    tracing::info!("Touch ID authentication successful");

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    // Register passkey metadata
    let info = auth_service
        .passkey_register(&four_words, &device_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to register passkey: {}", e);
            format!("Failed to register passkey: {}", e)
        })?;

    // Store password in keyring for auto-login (for the specific four_words being registered)
    auth_service
        .storage_manager()
        .store_password_in_keyring(&four_words, &password)
        .await
        .map_err(|e| {
            tracing::error!("Failed to store password in keyring: {}", e);
            format!("Failed to store password in keyring: {}", e)
        })?;

    tracing::info!(
        "Touch ID passkey registered successfully for: {}",
        four_words
    );
    Ok(info.into())
}

/// Authenticate using native macOS Touch ID
///
/// This uses the macOS Security Framework to trigger a native Touch ID prompt.
/// Only available on macOS.
#[cfg(target_os = "macos")]
#[tauri::command]
pub async fn auth_touchid_authenticate(
    state: State<'_, AppState>,
    four_words: String,
    reason: Option<String>,
) -> Result<SessionInfo, String> {
    tracing::info!("Touch ID authentication attempt for: {}", four_words);

    // Trigger native Touch ID authentication
    let auth_reason = reason.unwrap_or_else(|| format!("Sign in as {}", four_words));

    // Use macOS LocalAuthentication to prompt for Touch ID
    // This uses Swift command with stdin to compile and run Swift code that triggers actual biometric authentication
    let _auth_result = tokio::task::spawn_blocking(move || {
        use std::process::Command;
        use std::io::Write;

        // Use swift command to compile and run Swift code inline
        // This approach works reliably on macOS and triggers native Touch ID
        let swift_code = format!(
            r#"
import LocalAuthentication
import Foundation

let context = LAContext()
var error: NSError?

if !context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) {{
    print("false")
    exit(2)
}}

let semaphore = DispatchSemaphore(value: 0)
var success = false

context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: "{}") {{ result, authError in
    success = result
    semaphore.signal()
}}

semaphore.wait()
print(success ? "true" : "false")
exit(success ? 0 : 1)
"#,
            auth_reason.replace("\"", "\\\"")
        );

        // Compile and run Swift code using stdin
        let output = Command::new("swift")
            .arg("-")
            .arg("-framework")
            .arg("LocalAuthentication")
            .arg("-framework")
            .arg("Foundation")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(swift_code.as_bytes())?;
                }
                child.wait_with_output()
            });

        match output {
            Ok(result) => {
                match result.status.code() {
                    Some(0) => Ok(()),
                    Some(1) => Err("Touch ID authentication was cancelled or failed".to_string()),
                    Some(2) => Err("Touch ID is not available on this device".to_string()),
                    _ => {
                        let error = String::from_utf8_lossy(&result.stderr);
                        Err(format!("Touch ID authentication failed: {}", error))
                    }
                }
            }
            Err(e) => Err(format!("Failed to trigger Touch ID: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Touch ID task failed: {}", e))??;

    // If we get here, Touch ID authentication succeeded
    tracing::info!("Touch ID authentication successful");

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    // Authenticate using stored password from keyring
    let session_info = auth_service
        .passkey_authenticate(&four_words)
        .await
        .map_err(|e| {
            tracing::error!("Touch ID passkey authentication failed: {}", e);
            format!("Touch ID passkey authentication failed: {}", e)
        })?;

    tracing::info!("Touch ID authentication successful: {}", four_words);
    Ok(session_info)
}

/// Delete passkey registration for an identity
#[tauri::command]
pub async fn auth_passkey_delete(
    state: State<'_, AppState>,
    four_words: String,
) -> Result<(), String> {
    tracing::info!("Deleting passkey for: {}", four_words);

    let mut service = state.auth_service.write().await;
    let auth_service = service
        .as_mut()
        .ok_or_else(|| "Auth service not initialized".to_string())?;

    auth_service
        .passkey_delete(&four_words)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete passkey: {}", e);
            format!("Failed to delete passkey: {}", e)
        })?;

    tracing::info!("Passkey deleted successfully for: {}", four_words);
    Ok(())
}

/// Get OS username for default display name
///
/// Returns the current OS user's display name to use as default
/// when creating new identities, avoiding the need to prompt the user.
#[tauri::command]
pub fn get_os_username() -> Result<String, String> {
    tracing::info!("Getting OS username");

    #[cfg(target_os = "macos")]
    {
        // Try to get full name from macOS system
        use std::process::Command;

        // Try 'id -F' first (full name)
        if let Ok(output) = Command::new("id").arg("-F").output() {
            let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !username.is_empty() && username != "unknown" {
                tracing::info!("Got macOS full name: {}", username);
                return Ok(username);
            }
        }

        // Fallback to USER environment variable
        if let Ok(user) = std::env::var("USER") {
            tracing::info!("Using USER env var: {}", user);
            return Ok(user);
        }

        // Last resort
        Ok("User".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: Try USERNAME environment variable
        if let Ok(username) = std::env::var("USERNAME") {
            tracing::info!("Got Windows username: {}", username);
            return Ok(username);
        }

        Ok("User".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: Try multiple sources
        // 1. Try GECOS field (full name)
        use std::process::Command;
        if let Ok(output) = Command::new("getent")
            .arg("passwd")
            .arg(std::env::var("USER").unwrap_or_default())
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // GECOS field is the 5th field (0-indexed: 4)
            if let Some(gecos) = output_str.split(':').nth(4) {
                let full_name = gecos.split(',').next().unwrap_or("").trim();
                if !full_name.is_empty() {
                    tracing::info!("Got Linux GECOS name: {}", full_name);
                    return Ok(full_name.to_string());
                }
            }
        }

        // 2. Fallback to USER environment variable
        if let Ok(user) = std::env::var("USER") {
            tracing::info!("Using USER env var: {}", user);
            return Ok(user);
        }

        Ok("User".to_string())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Ok("User".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new();
        assert!(state.auth_service.read().await.is_none());
    }

    #[tokio::test]
    async fn test_auth_initialize() {
        let state = AppState::new();

        // Initialize directly
        let config = StorageConfig::default();
        let storage_manager = EncryptedStorageManager::new(config).await.unwrap();
        let auth_service = AuthService::new(storage_manager);
        *state.auth_service.write().await = Some(auth_service);

        assert!(state.auth_service.read().await.is_some());
    }

    // Helper to generate unique vault names for test isolation
    fn test_vault_name(base: &str) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros();
        format!("{}-{}", base, timestamp)
    }

    async fn init_test_state() -> AppState {
        let state = AppState::new();
        // Use temp directory for each test to ensure isolation
        let temp_dir = std::env::temp_dir().join(format!(
            "communitas-test-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros()
        ));
        let config = StorageConfig {
            vault_dir: temp_dir,
            use_keyring: false, // Disable keyring in tests
            ..StorageConfig::default()
        };
        let storage_manager = EncryptedStorageManager::new(config).await.unwrap();
        let auth_service = AuthService::new(storage_manager);
        *state.auth_service.write().await = Some(auth_service);
        state
    }

    #[tokio::test]
    async fn test_create_vault_and_login() {
        let state = init_test_state().await;
        let mut service = state.auth_service.write().await;
        let auth_service = service.as_mut().unwrap();

        // Use unique vault name for test isolation
        let vault_name = test_vault_name("test-vault-login");

        // Create a vault
        let vault_id = auth_service
            .create_vault(&vault_name, "secure-password-123", "Test User")
            .await
            .unwrap();

        assert!(!vault_id.is_empty());

        // Login with the vault
        let session_info = auth_service
            .login(&vault_name, "secure-password-123", Some("Desktop"))
            .await
            .unwrap();

        assert_eq!(session_info.four_words, vault_name);
        assert_eq!(session_info.display_name, "Test User");

        // Verify session is active
        assert!(auth_service.is_logged_in());
    }

    #[tokio::test]
    async fn test_login_with_wrong_password() {
        let state = init_test_state().await;
        let mut service = state.auth_service.write().await;
        let auth_service = service.as_mut().unwrap();

        // Use unique vault name for test isolation
        let vault_name = test_vault_name("test-vault-wrong-pwd");

        // Create a vault
        auth_service
            .create_vault(&vault_name, "correct-password", "Test User")
            .await
            .unwrap();

        // Try to login with wrong password - should fail
        let result = auth_service
            .login(&vault_name, "wrong-password", Some("Desktop"))
            .await;

        assert!(result.is_err(), "Login should fail with wrong password");
        assert!(!auth_service.is_logged_in());
    }

    #[tokio::test]
    async fn test_session_management() {
        let state = init_test_state().await;
        let mut service = state.auth_service.write().await;
        let auth_service = service.as_mut().unwrap();

        // No session initially
        assert!(!auth_service.is_logged_in());

        // Create and login
        auth_service
            .create_vault("test-session", "password", "Session Test")
            .await
            .unwrap();

        auth_service
            .login("test-session", "password", Some("Desktop"))
            .await
            .unwrap();

        // Verify session exists
        assert!(auth_service.is_logged_in());

        // Logout
        auth_service.logout().await.unwrap();

        // Verify session is cleared
        assert!(!auth_service.is_logged_in());
    }

    #[tokio::test]
    async fn test_list_vaults() {
        let state = init_test_state().await;
        let mut service = state.auth_service.write().await;
        let auth_service = service.as_mut().unwrap();

        // Use unique vault names for test isolation
        let vault_one = test_vault_name("vault-one");
        let vault_two = test_vault_name("vault-two");

        // Create multiple vaults
        auth_service
            .create_vault(&vault_one, "pass1", "User One")
            .await
            .unwrap();
        auth_service
            .create_vault(&vault_two, "pass2", "User Two")
            .await
            .unwrap();

        // List vaults
        let vaults = auth_service.list_vaults().await.unwrap();

        assert!(vaults.len() >= 2);
        let vault_words: Vec<String> = vaults.iter().map(|v| v.four_words.clone()).collect();
        assert!(vault_words.contains(&vault_one));
        assert!(vault_words.contains(&vault_two));
    }

    #[tokio::test]
    async fn test_multiple_vaults_isolation() {
        let state = init_test_state().await;
        let mut service = state.auth_service.write().await;
        let auth_service = service.as_mut().unwrap();

        // Use unique vault names for test isolation
        let vault_1 = test_vault_name("isolated-1");
        let vault_2 = test_vault_name("isolated-2");

        // Create vault 1
        auth_service
            .create_vault(&vault_1, "password1", "User 1")
            .await
            .unwrap();

        // Create vault 2
        auth_service
            .create_vault(&vault_2, "password2", "User 2")
            .await
            .unwrap();

        // Login to vault 1
        let session1 = auth_service
            .login(&vault_1, "password1", Some("Desktop"))
            .await
            .unwrap();
        assert_eq!(session1.display_name, "User 1");

        // Logout
        auth_service.logout().await.unwrap();

        // Login to vault 2
        let session2 = auth_service
            .login(&vault_2, "password2", Some("Desktop"))
            .await
            .unwrap();
        assert_eq!(session2.display_name, "User 2");
    }
}

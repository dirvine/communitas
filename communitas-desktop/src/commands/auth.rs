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

/// Register a passkey/biometric for an identity
///
/// This allows the user to use biometric authentication (Touch ID, Face ID, Windows Hello)
/// to login. The actual authentication still uses the password from keyring.
#[tauri::command]
pub async fn auth_passkey_register(
    state: State<'_, AppState>,
    four_words: String,
    device_name: String,
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

    let info = auth_service
        .passkey_register(&four_words, &device_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to register passkey: {}", e);
            format!("Failed to register passkey: {}", e)
        })?;

    tracing::info!("Passkey registered successfully for: {}", four_words);
    Ok(info.into())
}

/// Authenticate using passkey/biometric
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

/// Generate a valid four-word identity using the four-word-networking dictionary
///
/// Returns a random four-word identity with valid dictionary words
/// (e.g., "ocean-forest-moon-star")
#[tauri::command]
pub fn generate_four_word_identity() -> Result<String, String> {
    tracing::info!("Generating four-word identity");

    communitas_core::identity::generate_id_words()
        .map_err(|e| format!("Failed to generate identity: {}", e))
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

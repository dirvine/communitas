// Copyright (c) 2025 Saorsa Labs Limited
//
// Encrypted storage Tauri commands

use communitas_core::{
    core_context::CoreContext,
    encrypted_storage::{
        EncryptedStorageManager, StorageConfig, VaultInfo, AuthInfo, AuthResult,
    },
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use base64::{Engine as _, engine::general_purpose};

/// Initialize encrypted storage for the current identity
#[tauri::command]
pub async fn core_storage_initialize(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    password: String,
    display_name: String,
) -> Result<AuthResult, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    // Get the current identity's four words
    let identity = context.get_current_identity()
        .await
        .ok_or("No current identity")?;

    // Get or create storage manager
    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    // Initialize vault for current identity
    let auth_info = AuthInfo {
        four_words: identity.four_words.clone(),
        password,
        display_name,
        device_id: context.device_name.clone(),
        require_passkey: false,
    };

    storage_manager.authenticate(auth_info)
        .await
        .map_err(|e| e.to_string())
}

/// Login to an existing vault
#[tauri::command]
pub async fn core_storage_login(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    four_words: String,
    password: String,
) -> Result<AuthResult, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    let auth_info = AuthInfo {
        four_words,
        password,
        display_name: String::new(), // Not needed for login
        device_id: context.device_name.clone(),
        require_passkey: false,
    };

    storage_manager.authenticate(auth_info)
        .await
        .map_err(|e| e.to_string())
}

/// Password-only login for familiar devices
#[tauri::command]
pub async fn core_storage_password_login(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    password: String,
) -> Result<AuthResult, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    storage_manager.password_only_login(&password)
        .await
        .map_err(|e| e.to_string())
}

/// Store data in vault
#[tauri::command]
pub async fn core_storage_store(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
    data_base64: String,
    use_fec: bool,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    // Decode base64 data
    let data = general_purpose::STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    storage_manager.store(&key, &data, use_fec)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Retrieve data from vault
#[tauri::command]
pub async fn core_storage_retrieve(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
) -> Result<String, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    let data = storage_manager.retrieve(&key)
        .await
        .map_err(|e| e.to_string())?;

    // Encode to base64 for JavaScript
    Ok(general_purpose::STANDARD.encode(data))
}

/// Delete data from vault
#[tauri::command]
pub async fn core_storage_delete(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    key: String,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    storage_manager.delete(&key)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// List all keys in current vault
#[tauri::command]
pub async fn core_storage_list_keys(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<String>, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    storage_manager.list_keys()
        .await
        .map_err(|e| e.to_string())
}

/// List all available vaults
#[tauri::command]
pub async fn core_storage_list_vaults(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<VaultInfo>, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    storage_manager.list_vaults()
        .await
        .map_err(|e| e.to_string())
}

/// Get active sessions
#[tauri::command]
pub async fn core_storage_get_sessions(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<String>, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    let sessions = storage_manager.get_active_sessions()
        .await
        .map_err(|e| e.to_string())?;

    // Return just four-words for simplicity
    Ok(sessions.into_iter().map(|s| s.four_words).collect())
}

/// Switch to a different vault
#[tauri::command]
pub async fn core_storage_switch_vault(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    four_words: String,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    storage_manager.switch_vault(&four_words)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Logout from current vault
#[tauri::command]
pub async fn core_storage_logout(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    storage_manager.logout()
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Export vault for backup
#[tauri::command]
pub async fn core_storage_export_vault(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    include_data: bool,
) -> Result<String, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    let export_data = storage_manager.export_vault(include_data)
        .await
        .map_err(|e| e.to_string())?;

    // Return as base64
    Ok(general_purpose::STANDARD.encode(export_data))
}

/// Import vault from backup
#[tauri::command]
pub async fn core_storage_import_vault(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    backup_base64: String,
    password: String,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    // Decode base64
    let backup_data = general_purpose::STANDARD
        .decode(&backup_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    storage_manager.import_vault(&backup_data, &password)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Store identity data in vault
#[tauri::command]
pub async fn core_storage_store_identity(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    identity_data_base64: String,
) -> Result<bool, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    // Decode base64
    let identity_data = general_purpose::STANDARD
        .decode(&identity_data_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    // Store with FEC for critical data
    storage_manager.store("identity", &identity_data, true)
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

/// Get vault statistics
#[tauri::command]
pub async fn core_storage_get_stats(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<serde_json::Value, String> {
    let guard = shared.read().await;
    let context = guard.as_ref().ok_or("Core context not initialized")?;

    let storage_manager = context.get_storage_manager()
        .await
        .map_err(|e| e.to_string())?;

    let stats = storage_manager.get_stats()
        .await
        .map_err(|e| e.to_string())?;

    // Convert to JSON for frontend
    serde_json::to_value(stats)
        .map_err(|e| e.to_string())
}
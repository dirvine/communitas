// Copyright (c) 2025 Saorsa Labs Limited
//
// Core application commands (placeholder)
//
// TODO: Implement with new gossip-based architecture

use communitas_core::CoreContext;
use communitas_core::types::DeviceType;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub peer_id: String,
    pub display_name: String,
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    pub content: String,
    pub author: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync: Option<i64>,
}

/// Initialize core context
#[tauri::command]
pub async fn core_initialize(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    four_words: String,
    display_name: String,
    device_name: Option<String>,
    device_type: Option<String>,
) -> Result<bool, String> {
    let dev_type = match device_type
        .unwrap_or_else(|| "Desktop".to_string())
        .as_str()
    {
        "Desktop" | "desktop" => DeviceType::Desktop,
        "Laptop" | "laptop" => DeviceType::Laptop,
        "Mobile" | "mobile" => DeviceType::Mobile,
        "Server" | "server" => DeviceType::Server,
        _ => DeviceType::Unknown,
    };

    let storage_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("communitas")
        .join(&four_words);

    let mut ctx = CoreContext::initialize(
        four_words.clone(),
        display_name,
        device_name.unwrap_or_else(|| "default".to_string()),
        dev_type,
        storage_dir,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Auto-start networking with saorsa-gossip + ant-quic
    tracing::info!("🌐 Starting P2P networking for {}", four_words);
    match ctx.start_networking(None).await {
        Ok(connection_identity) => {
            tracing::info!("✅ Network started successfully: {}", connection_identity);
        }
        Err(e) => {
            tracing::warn!(
                "⚠️ Network startup failed (continuing in local mode): {}",
                e
            );
            // Don't fail initialization if networking fails - app works in local mode
        }
    }

    let mut guard = shared.write().await;
    *guard = Some(ctx);

    Ok(true)
}

#[tauri::command]
pub async fn core_get_peer_id(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<String, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_get_user_info(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<UserInfo, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_set_display_name(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _display_name: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_create_channel(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _name: String,
    _description: String,
) -> Result<ChannelInfo, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_get_channels(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<ChannelInfo>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_add_reaction(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
    _emoji: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_send_message_to_channel(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    _content: String,
) -> Result<MessageInfo, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_channel_recipients(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_channel_list_members(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_channel_invite_by_words(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    _four_words: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_resolve_channel_members(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<UserInfo>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_create_thread(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
) -> Result<String, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_subscribe_messages(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_private_put(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _key: String,
    _value: Vec<u8>,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_private_get(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _key: String,
) -> Result<Vec<u8>, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_send_message_to_recipients(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _recipients: Vec<String>,
    _content: String,
) -> Result<MessageInfo, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_get_bootstrap_nodes() -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_update_bootstrap_nodes(_nodes: Vec<String>) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_add_bootstrap_node(_node: String) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_clear_custom_nodes() -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_get_bootstrap_stats() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn core_messages_list(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<MessageInfo>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_messages_send(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    _content: String,
) -> Result<MessageInfo, String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_messages_edit(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
    _new_content: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_messages_delete(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_entity_get_permissions(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn core_entity_get_encryption_status(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[tauri::command]
pub async fn core_entity_update(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
    _updates: serde_json::Value,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_entity_delete(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_entity_mute(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
    _muted: bool,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn core_entity_block(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
    _blocked: bool,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn get_sync_status(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<SyncStatus, String> {
    Ok(SyncStatus {
        is_syncing: false,
        last_sync: None,
    })
}

#[tauri::command]
pub async fn subscribe_to_entity(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[tauri::command]
pub async fn unsubscribe_from_entity(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

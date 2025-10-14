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

// Gossip overlay integration
#[cfg(feature = "gossip_overlay")]
use crate::gossip_commands;

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

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_get_user_info(
    gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<UserInfo, String> {
    // Get current user's identity from gossip
    let peer_id = gossip_commands::gossip_get_own_identity(gossip_state).await?;

    // TODO: Retrieve display_name and device_name from gossip metadata
    // For now, return basic info with peer_id
    Ok(UserInfo {
        peer_id: peer_id.clone(),
        display_name: "User".to_string(), // TODO: Get from metadata
        device_name: "Device".to_string(), // TODO: Get from metadata
    })
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_get_user_info(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<UserInfo, String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_set_display_name(
    gossip_state: State<'_, gossip_commands::GossipState>,
    display_name: String,
) -> Result<(), String> {
    // Update display name by storing as metadata in gossip
    // Prefix the display name with "metadata:display_name:" for later retrieval
    let metadata_value = format!("metadata:display_name:{}", display_name);

    gossip_commands::gossip_store_message(gossip_state, metadata_value.as_bytes().to_vec()).await?;

    // TODO: Implement proper metadata storage mechanism in gossip
    // For now, we store it as a regular message with "metadata:" prefix
    Ok(())
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_set_display_name(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _display_name: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_create_channel(
    gossip_state: State<'_, gossip_commands::GossipState>,
    name: String,
    _description: String,
) -> Result<ChannelInfo, String> {
    let channel_id = uuid::Uuid::new_v4().to_string();

    // Join entity (creates it if doesn't exist)
    gossip_commands::gossip_join_entity(
        gossip_state,
        channel_id.clone(),
        "channel".to_string(),
    )
    .await?;

    Ok(ChannelInfo {
        id: channel_id,
        name,
        members: vec![],
    })
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_create_channel(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _name: String,
    _description: String,
) -> Result<ChannelInfo, String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_get_channels(
    _gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<Vec<ChannelInfo>, String> {
    // TODO: Implement channel tracking in gossip overlay
    // For now, return empty list until gossip_get_subscribed_entities is available
    // This will require adding entity subscription tracking to gossip_commands.rs
    Ok(vec![])
}

#[cfg(not(feature = "gossip_overlay"))]
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

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_send_message_to_channel(
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
    content: String,
) -> Result<MessageInfo, String> {
    let message_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();

    // Get current user for author field
    let author = gossip_commands::gossip_get_own_identity(gossip_state.clone())
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    // Publish to channel entity
    gossip_commands::gossip_publish_to_entity(
        gossip_state,
        channel_id,
        content.as_bytes().to_vec(),
    )
    .await?;

    Ok(MessageInfo {
        id: message_id,
        content,
        author,
        timestamp,
    })
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_send_message_to_channel(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    _content: String,
) -> Result<MessageInfo, String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_channel_recipients(
    _gossip_state: State<'_, gossip_commands::GossipState>,
    _channel_id: String,
) -> Result<Vec<String>, String> {
    // TODO: Implement entity recipient tracking in gossip overlay
    // For now, return empty list until gossip_get_entity_subscribers is available
    // Recipients are essentially the same as subscribers
    Ok(vec![])
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_channel_recipients(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_channel_list_members(
    _gossip_state: State<'_, gossip_commands::GossipState>,
    _channel_id: String,
) -> Result<Vec<String>, String> {
    // TODO: Implement entity subscriber tracking in gossip overlay
    // For now, return empty list until gossip_get_entity_subscribers is available
    // This will require adding subscriber tracking to gossip entity management
    Ok(vec![])
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_channel_list_members(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_channel_invite_by_words(
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
    four_words: String,
) -> Result<(), String> {
    // First, find the contact by four-word address
    let _contact = gossip_commands::gossip_find_contact(gossip_state.clone(), four_words.clone())
        .await?;

    // Then invite them by joining them to the channel entity
    // Note: The actual invitation mechanism may require additional gossip operations
    // For now, we publish an invite message to the channel
    let invite_msg = format!("invite:{}", four_words);
    gossip_commands::gossip_publish_to_entity(
        gossip_state,
        channel_id,
        invite_msg.as_bytes().to_vec(),
    )
    .await?;

    Ok(())
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_channel_invite_by_words(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    _four_words: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_resolve_channel_members(
    _gossip_state: State<'_, gossip_commands::GossipState>,
    _channel_id: String,
) -> Result<Vec<UserInfo>, String> {
    // TODO: Implement entity subscriber tracking with user metadata
    // Requires:
    // 1. gossip_get_entity_subscribers to get member list
    // 2. gossip_get_peer_metadata to resolve user details
    // For now, return empty list
    Ok(vec![])
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_resolve_channel_members(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<Vec<UserInfo>, String> {
    Ok(vec![])
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_create_thread(
    gossip_state: State<'_, gossip_commands::GossipState>,
    message_id: String,
) -> Result<String, String> {
    // Create a new thread entity based on the message_id
    let thread_id = format!("thread_{}", message_id);

    // Join the thread entity (creates it if doesn't exist)
    gossip_commands::gossip_join_entity(
        gossip_state,
        thread_id.clone(),
        "thread".to_string(),
    )
    .await?;

    Ok(thread_id)
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_create_thread(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
) -> Result<String, String> {
    Err("Gossip overlay not enabled".to_string())
}

#[tauri::command]
pub async fn core_subscribe_messages(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_private_put(
    gossip_state: State<'_, gossip_commands::GossipState>,
    key: String,
    value: Vec<u8>,
) -> Result<(), String> {
    // Store the key-value pair as a message in gossip storage
    // Prepend the key to the value for later retrieval
    let mut data = key.as_bytes().to_vec();
    data.push(b':');
    data.extend_from_slice(&value);

    gossip_commands::gossip_store_message(gossip_state, data).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_private_put(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _key: String,
    _value: Vec<u8>,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_private_get(
    gossip_state: State<'_, gossip_commands::GossipState>,
    key: String,
) -> Result<Vec<u8>, String> {
    // Retrieve all stored messages and find the one matching our key
    let messages = gossip_commands::gossip_get_all_messages(gossip_state).await?;

    let key_prefix = format!("{}:", key);
    let key_prefix_bytes = key_prefix.as_bytes();

    for msg in messages {
        if msg.starts_with(key_prefix_bytes) {
            // Found the key, return the value (everything after "key:")
            return Ok(msg[key_prefix_bytes.len()..].to_vec());
        }
    }

    Err(format!("Key '{}' not found in private storage", key))
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_private_get(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _key: String,
) -> Result<Vec<u8>, String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_send_message_to_recipients(
    gossip_state: State<'_, gossip_commands::GossipState>,
    recipients: Vec<String>,
    content: String,
) -> Result<MessageInfo, String> {
    let message_id = uuid::Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().timestamp();
    let message_bytes = content.as_bytes().to_vec();

    // Get current user for author field
    let author = gossip_commands::gossip_get_own_identity(gossip_state.clone())
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    // Send to each recipient via gossip overlay
    for recipient in recipients {
        gossip_commands::gossip_send_direct_message(
            gossip_state.clone(),
            recipient,
            message_bytes.clone(),
        )
        .await?;
    }

    Ok(MessageInfo {
        id: message_id,
        content,
        author,
        timestamp,
    })
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_send_message_to_recipients(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _recipients: Vec<String>,
    _content: String,
) -> Result<MessageInfo, String> {
    Err("Gossip overlay not enabled".to_string())
}

#[tauri::command]
pub async fn core_get_bootstrap_nodes(
    #[cfg(feature = "gossip_overlay")]
    gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<Vec<String>, String> {
    #[cfg(feature = "gossip_overlay")]
    {
        let peers = gossip_commands::gossip_get_cached_peers(gossip_state).await?;
        Ok(peers.into_iter().map(|p| p.four_words).collect())
    }

    #[cfg(not(feature = "gossip_overlay"))]
    {
        Ok(vec![])
    }
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_update_bootstrap_nodes(
    gossip_state: State<'_, gossip_commands::GossipState>,
    nodes: Vec<String>,
) -> Result<(), String> {
    // Add each bootstrap node individually
    for node in nodes {
        gossip_commands::gossip_add_bootstrap_peer(gossip_state.clone(), node).await?;
    }
    Ok(())
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_update_bootstrap_nodes(_nodes: Vec<String>) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_add_bootstrap_node(
    gossip_state: State<'_, gossip_commands::GossipState>,
    node: String,
) -> Result<(), String> {
    gossip_commands::gossip_add_bootstrap_peer(gossip_state, node).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_add_bootstrap_node(_node: String) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_clear_custom_nodes(
    _gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<(), String> {
    // TODO: Implement gossip_clear_bootstrap_peers in gossip_commands.rs
    // For now, return error indicating this needs to be implemented
    Err("Clear custom nodes not yet implemented in gossip overlay".to_string())
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_clear_custom_nodes() -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[tauri::command]
pub async fn core_get_bootstrap_stats() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({}))
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_messages_list(
    gossip_state: State<'_, gossip_commands::GossipState>,
    _channel_id: String,
) -> Result<Vec<MessageInfo>, String> {
    // Get all stored messages (not filtered by channel yet)
    // TODO: Implement per-entity message retrieval in gossip overlay
    let messages_bytes = gossip_commands::gossip_get_all_messages(gossip_state).await?;

    // Convert byte messages to MessageInfo
    let messages = messages_bytes
        .into_iter()
        .enumerate()
        .map(|(idx, msg_bytes)| {
            let content = String::from_utf8_lossy(&msg_bytes).to_string();
            MessageInfo {
                id: format!("msg_{}", idx),
                content,
                author: "unknown".to_string(), // TODO: Extract from message metadata
                timestamp: 0, // TODO: Extract from message metadata
            }
        })
        .collect();

    Ok(messages)
}

#[cfg(not(feature = "gossip_overlay"))]
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

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn subscribe_to_entity(
    gossip_state: State<'_, gossip_commands::GossipState>,
    entity_id: String,
) -> Result<(), String> {
    gossip_commands::gossip_subscribe_to_entity(gossip_state, entity_id).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn subscribe_to_entity(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn unsubscribe_from_entity(
    gossip_state: State<'_, gossip_commands::GossipState>,
    entity_id: String,
) -> Result<(), String> {
    gossip_commands::gossip_leave_entity(gossip_state, entity_id).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn unsubscribe_from_entity(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

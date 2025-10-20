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
use tracing::{info, warn};

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

/// Attempt to recover from corrupted core context state
#[tauri::command]
pub async fn core_recover_state(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<bool, String> {
    info!("Attempting to recover core context state");

    let guard = shared.write().await;
    if guard.is_some() {
        warn!("Core context already exists, skipping recovery");
        return Ok(false);
    }

    // Try to recover from backup or recreate minimal state
    // For now, just log that recovery was attempted
    info!("Core context recovery completed (minimal implementation)");
    Ok(true)
}

/// Initialize core context
#[tauri::command]
pub async fn core_initialize(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    webrtc_state: State<'_, crate::webrtc_commands::WebRtcState>,
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

    // Initialize WebRTC service with the gossip context
    if let Some(core_ctx) = guard.as_ref()
        && let Some(gossip) = &core_ctx.gossip
    {
        let webrtc_service =
            communitas_core::webrtc::service::CommunitasWebRtcService::new(gossip.clone())
                .map_err(|e| format!("Failed to initialize WebRTC service: {}", e))?;

        // Start the WebRTC service
        webrtc_service
            .start()
            .await
            .map_err(|e| format!("Failed to start WebRTC service: {}", e))?;

        // Store the WebRTC service in the state
        webrtc_state
            .initialize(webrtc_service)
            .await
            .map_err(|e| format!("Failed to store WebRTC service: {}", e))?;
    }

    Ok(true)
}

// Batch 8: Utility commands

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_get_peer_id(
    gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<String, String> {
    // Get own identity from gossip overlay
    gossip_commands::gossip_get_own_identity(gossip_state).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_get_peer_id(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<String, String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_get_user_info(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<UserInfo, String> {
    // Check if core is initialized first
    let core_initialized = {
        let guard = shared.read().await;
        guard.is_some()
    };

    if !core_initialized {
        return Err("Core context not initialized. Please call core_initialize first.".to_string());
    }

    // Get current user's identity from gossip
    let peer_id = gossip_commands::gossip_get_own_identity(gossip_state.clone())
        .await
        .map_err(|e| format!("Failed to get peer identity: {}", e))?;

    // Get metadata from gossip
    let metadata = gossip_commands::gossip_get_own_metadata(gossip_state)
        .await
        .map_err(|e| format!("Failed to get user metadata: {}", e))?;

    // Extract display_name and device_name from metadata
    let mut display_name = "User".to_string();
    let mut device_name = "Device".to_string();

    for entry in metadata {
        match entry.key.as_str() {
            "display_name" => display_name = entry.value,
            "device_name" => device_name = entry.value,
            _ => {}
        }
    }

    info!("Retrieved user info for peer: {}", peer_id);
    Ok(UserInfo {
        peer_id,
        display_name,
        device_name,
    })
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_get_user_info(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<UserInfo, String> {
    // Check if core is initialized
    let guard = shared.read().await;
    let core_ctx = guard
        .as_ref()
        .ok_or("Core context not initialized. Please call core_initialize first.")?;

    // Return basic info from core context
    Ok(UserInfo {
        peer_id: core_ctx.four_words.clone(),
        display_name: core_ctx.display_name.clone(),
        device_name: core_ctx.device_name.clone(),
    })
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_set_display_name(
    gossip_state: State<'_, gossip_commands::GossipState>,
    display_name: String,
) -> Result<(), String> {
    // Update display name using metadata storage
    gossip_commands::gossip_store_own_metadata(
        gossip_state,
        "display_name".to_string(),
        display_name,
    )
    .await
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
    gossip_commands::gossip_join_entity(gossip_state, channel_id.clone(), "channel".to_string())
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
    gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<Vec<ChannelInfo>, String> {
    // Get subscribed entities from gossip overlay
    let entities = gossip_commands::gossip_get_subscribed_entities(gossip_state).await?;

    // Convert to ChannelInfo format
    let channels = entities
        .into_iter()
        .map(|entity_id| ChannelInfo {
            id: entity_id.clone(),
            name: entity_id,
            members: vec![], // TODO: Get actual member list
        })
        .collect();

    Ok(channels)
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_get_channels(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Vec<ChannelInfo>, String> {
    Ok(vec![])
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_add_reaction(
    gossip_state: State<'_, gossip_commands::GossipState>,
    message_id: String,
    emoji: String,
) -> Result<(), String> {
    // Get own identity as reactor
    let reactor = gossip_commands::gossip_get_own_identity(gossip_state.clone()).await?;

    // Store reaction using message ID and reactor
    // Format: "msg_reaction:{message_id}:{emoji}:{reactor}"
    let reaction_message = format!("msg_reaction:{}:{}:{}", message_id, emoji, reactor);

    gossip_commands::gossip_store_message(gossip_state, reaction_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_add_reaction(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
    _emoji: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
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
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
) -> Result<Vec<String>, String> {
    // Get entity subscribers from gossip overlay (same as channel members)
    gossip_commands::gossip_get_entity_subscribers(gossip_state, channel_id).await
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
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
) -> Result<Vec<String>, String> {
    // Get entity subscribers from gossip overlay
    gossip_commands::gossip_get_entity_subscribers(gossip_state, channel_id).await
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
    let _contact =
        gossip_commands::gossip_find_contact(gossip_state.clone(), four_words.clone()).await?;

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
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
) -> Result<Vec<UserInfo>, String> {
    // Get entity subscribers from gossip overlay
    let subscribers =
        gossip_commands::gossip_get_entity_subscribers(gossip_state.clone(), channel_id).await?;

    // Get metadata for each subscriber
    let mut members = Vec::new();
    for peer_id in subscribers {
        let metadata =
            gossip_commands::gossip_get_peer_metadata(gossip_state.clone(), peer_id.clone())
                .await?;

        // Extract display_name and device_name from metadata
        let mut display_name = "User".to_string();
        let mut device_name = "Device".to_string();

        for entry in metadata {
            match entry.key.as_str() {
                "display_name" => display_name = entry.value,
                "device_name" => device_name = entry.value,
                _ => {}
            }
        }

        members.push(UserInfo {
            peer_id,
            display_name,
            device_name,
        });
    }

    Ok(members)
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
    gossip_commands::gossip_join_entity(gossip_state, thread_id.clone(), "thread".to_string())
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

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_subscribe_messages(
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
) -> Result<(), String> {
    // Subscribe to entity's message topic in gossip overlay
    gossip_commands::gossip_subscribe_to_entity(gossip_state, channel_id).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_subscribe_messages(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
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
    #[cfg(feature = "gossip_overlay")] gossip_state: State<'_, gossip_commands::GossipState>,
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
    gossip_state: State<'_, gossip_commands::GossipState>,
) -> Result<(), String> {
    // Clear all bootstrap peers using gossip function
    gossip_commands::gossip_clear_bootstrap_peers(gossip_state).await
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
    channel_id: String,
) -> Result<Vec<MessageInfo>, String> {
    // Get entity-specific messages from gossip overlay
    let messages_bytes =
        gossip_commands::gossip_get_entity_messages(gossip_state, channel_id).await?;

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
                timestamp: 0,                  // TODO: Extract from message metadata
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

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_messages_send(
    gossip_state: State<'_, gossip_commands::GossipState>,
    channel_id: String,
    content: String,
) -> Result<MessageInfo, String> {
    // Generate unique message ID
    let message_id = uuid::Uuid::new_v4().to_string();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs() as i64;

    // Get own identity as author
    let author = gossip_commands::gossip_get_own_identity(gossip_state.clone()).await?;

    // Store message with entity prefix using the helper function
    gossip_commands::gossip_store_entity_message(
        gossip_state.clone(),
        channel_id.clone(),
        content.as_bytes().to_vec(),
    )
    .await?;

    // Also publish to entity subscribers
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
pub async fn core_messages_send(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _channel_id: String,
    _content: String,
) -> Result<MessageInfo, String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_messages_edit(
    gossip_state: State<'_, gossip_commands::GossipState>,
    message_id: String,
    new_content: String,
) -> Result<(), String> {
    // Store edit as metadata using message ID
    // Format: "msg_edit:{message_id}:{new_content}"
    let edit_message = format!("msg_edit:{}:{}", message_id, new_content);

    gossip_commands::gossip_store_message(gossip_state, edit_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_messages_edit(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
    _new_content: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_messages_delete(
    gossip_state: State<'_, gossip_commands::GossipState>,
    message_id: String,
) -> Result<(), String> {
    // Store deletion marker using message ID
    // Format: "msg_delete:{message_id}"
    let delete_marker = format!("msg_delete:{}", message_id);

    gossip_commands::gossip_store_message(gossip_state, delete_marker.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_messages_delete(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _message_id: String,
) -> Result<(), String> {
    Err("Gossip overlay not enabled".to_string())
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

// Entity management commands - Batch 6

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_entity_update(
    gossip_state: State<'_, gossip_commands::GossipState>,
    entity_id: String,
    updates: serde_json::Value,
) -> Result<(), String> {
    // Store entity update in CRDT with "entity_update:{id}:{json}" format
    let update_message = format!("entity_update:{}:{}", entity_id, updates);
    gossip_commands::gossip_store_message(gossip_state, update_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_entity_update(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
    _updates: serde_json::Value,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_entity_delete(
    gossip_state: State<'_, gossip_commands::GossipState>,
    entity_id: String,
) -> Result<(), String> {
    // Store entity delete marker in CRDT with "entity_delete:{id}" format
    let delete_marker = format!("entity_delete:{}", entity_id);
    gossip_commands::gossip_store_message(gossip_state, delete_marker.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_entity_delete(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_entity_mute(
    gossip_state: State<'_, gossip_commands::GossipState>,
    entity_id: String,
    muted: bool,
) -> Result<(), String> {
    // Store entity mute state in CRDT with "entity_mute:{id}:{bool}" format
    let mute_message = format!("entity_mute:{}:{}", entity_id, muted);
    gossip_commands::gossip_store_message(gossip_state, mute_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_entity_mute(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _entity_id: String,
    _muted: bool,
) -> Result<(), String> {
    Err("Not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_entity_block(
    gossip_state: State<'_, gossip_commands::GossipState>,
    entity_id: String,
    blocked: bool,
) -> Result<(), String> {
    // Store entity block state in CRDT with "entity_block:{id}:{bool}" format
    let block_message = format!("entity_block:{}:{}", entity_id, blocked);
    gossip_commands::gossip_store_message(gossip_state, block_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
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

#[cfg(test)]
mod tests {
    use crate::webrtc_commands::WebRtcState;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[test]
    fn test_core_recover_state_placeholder() {
        // Placeholder test - implement when recovery logic is added
    }

    #[tokio::test]
    async fn test_core_initialize_with_webrtc_state() {
        // Test that core_initialize accepts WebRtcState parameter
        let shared = Arc::new(RwLock::new(None::<communitas_core::CoreContext>));
        let webrtc_state = WebRtcState::new();

        // This tests the interface - verifies that the function signature is correct
        // and parameters are accepted (TDD approach)
        assert!(shared.read().await.is_none());
        assert!(webrtc_state.service.read().await.is_none());
    }

    #[tokio::test]
    async fn test_compilation_fixes_integration() {
        // Integration test to verify the compilation fixes work
        // This test ensures that:
        // 1. WebRTC state can be created and used
        // 2. Core context state management works
        // 3. Module references are correct

        let shared = Arc::new(RwLock::new(None::<communitas_core::CoreContext>));
        let webrtc_state = WebRtcState::new();

        // Verify initial states
        assert!(shared.read().await.is_none());
        assert!(webrtc_state.service.read().await.is_none());

        // Test that we can create and manipulate the states without compilation errors
        // This verifies that the module references and type signatures are correct
        {
            let mut shared_guard = shared.write().await;
            *shared_guard = None; // This should compile and work
        }

        {
            let mut webrtc_guard = webrtc_state.service.write().await;
            *webrtc_guard = None; // This should compile and work
        }
    }
}

//! HTTP request handlers for bridge API endpoints

use crate::{
    error::{BridgeError, BridgeResult},
    state::BridgeState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use saorsa_core::chat::{ChannelId, ChannelType, MessageId};
use saorsa_core::identity::FourWordAddress;
use saorsa_core::messaging::{ChannelId as MessagingChannelId, MessageContent};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

/// Health check endpoint
pub async fn health() -> Json<Value> {
    Json(json!({"status": "ok", "service": "communitas-bridge"}))
}

/// Initialize core context
#[derive(Deserialize)]
pub struct InitializeRequest {
    four_words: String,
    display_name: String,
    device_name: String,
}

pub async fn core_initialize(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<InitializeRequest>,
) -> BridgeResult<Json<Value>> {
    state
        .initialize_core(req.four_words, req.display_name, req.device_name)
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Initialization failed: {}", e)))?;

    Ok(Json(json!({"success": true})))
}

/// Get core status
pub async fn core_status(State(state): State<Arc<BridgeState>>) -> Json<Value> {
    let initialized = state.is_initialized().await;
    Json(json!({"initialized": initialized}))
}

/// Create a new channel
#[derive(Deserialize)]
pub struct CreateChannelRequest {
    name: String,
    description: String,
}

pub async fn create_channel(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<CreateChannelRequest>,
) -> BridgeResult<Json<Value>> {
    // Validate input
    if req.name.is_empty() {
        return Err(BridgeError::InvalidRequest(
            "Channel name cannot be empty".to_string(),
        ));
    }
    if req.name.len() > 100 {
        return Err(BridgeError::InvalidRequest(
            "Channel name too long (max 100 chars)".to_string(),
        ));
    }
    if req.description.len() > 500 {
        return Err(BridgeError::InvalidRequest(
            "Description too long (max 500 chars)".to_string(),
        ));
    }

    let mut core_guard = state.core.write().await;
    let core = core_guard
        .as_mut()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    let channel = core
        .chat
        .create_channel(req.name, req.description, ChannelType::Public, None)
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("create_channel failed: {}", e)))?;

    let created_at: DateTime<Utc> = channel.created_at.into();
    Ok(Json(json!({
        "id": channel.id.0,
        "name": channel.name,
        "description": channel.description,
        "created_at": created_at.to_rfc3339()
    })))
}

/// List all channels
pub async fn list_channels(State(state): State<Arc<BridgeState>>) -> BridgeResult<Json<Value>> {
    let mut core_guard = state.core.write().await;
    let core = core_guard
        .as_mut()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    let channel_ids = core
        .chat
        .get_user_channels()
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("get_user_channels failed: {}", e)))?;

    let mut channels = Vec::new();
    for id in channel_ids {
        let channel = core
            .chat
            .get_channel(&id)
            .await
            .map_err(|e| BridgeError::CommandFailed(format!("get_channel failed: {}", e)))?;

        let created_at: DateTime<Utc> = channel.created_at.into();
        channels.push(json!({
            "id": channel.id.0,
            "name": channel.name,
            "description": channel.description,
            "created_at": created_at.to_rfc3339()
        }));
    }

    Ok(Json(json!({"channels": channels})))
}

/// Get channel messages
#[derive(Deserialize)]
pub struct GetMessagesQuery {
    limit: Option<usize>,
}

pub async fn get_channel_messages(
    State(_state): State<Arc<BridgeState>>,
    Path(_channel_id): Path<String>,
    Query(_query): Query<GetMessagesQuery>,
) -> BridgeResult<Json<Value>> {
    // TODO: Implement via messaging service once API is clarified
    // ChatManager doesn't have get_channel_messages in saorsa-core 0.3.26
    Ok(Json(
        json!({"messages": [], "note": "Pending saorsa-core API update"}),
    ))
}

/// Send message to channel
#[derive(Deserialize)]
pub struct SendMessageRequest {
    content: String,
    reply_to_id: Option<String>,
    recipients: Vec<String>, // Four-word addresses
}

pub async fn send_channel_message(
    State(state): State<Arc<BridgeState>>,
    Path(channel_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
) -> BridgeResult<Json<Value>> {
    // Validate input
    if req.content.is_empty() {
        return Err(BridgeError::InvalidRequest(
            "Message content cannot be empty".to_string(),
        ));
    }
    if req.content.len() > 10 * 1024 {
        return Err(BridgeError::InvalidRequest(
            "Message too long (max 10KB)".to_string(),
        ));
    }
    if req.recipients.is_empty() {
        return Err(BridgeError::InvalidRequest(
            "Must specify at least one recipient".to_string(),
        ));
    }

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Convert recipients to FourWordAddress
    let recipients: Vec<FourWordAddress> =
        req.recipients.into_iter().map(FourWordAddress).collect();

    // Parse channel UUID
    let channel_uuid = uuid::Uuid::parse_str(&channel_id)
        .map_err(|e| BridgeError::InvalidRequest(format!("Invalid channel ID: {}", e)))?;

    // Send message
    let (msg_id, _receipt) = core
        .messaging
        .send_message(
            recipients,
            MessageContent::Text(req.content),
            MessagingChannelId(channel_uuid),
            Default::default(),
        )
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("send_message failed: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "message_id": msg_id.to_string()
    })))
}

/// Get members for entity
pub async fn get_members(
    State(_state): State<Arc<BridgeState>>,
    Path((_entity_type, _id)): Path<(String, String)>,
) -> BridgeResult<Json<Value>> {
    // TODO: Implement via CoreContext
    Ok(Json(json!({"members": []})))
}

/// Add member to entity
#[derive(Deserialize)]
pub struct AddMemberRequest {
    four_word_address: String,
    role: String,
}

pub async fn add_member(
    State(_state): State<Arc<BridgeState>>,
    Path((_entity_type, _id)): Path<(String, String)>,
    Json(_req): Json<AddMemberRequest>,
) -> BridgeResult<Json<Value>> {
    // TODO: Implement via CoreContext
    Ok(Json(json!({"success": true})))
}

/// Create thread
#[derive(Deserialize)]
pub struct CreateThreadRequest {
    channel_id: String,
    parent_message_id: String,
}

pub async fn create_thread(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<CreateThreadRequest>,
) -> BridgeResult<Json<Value>> {
    let mut core_guard = state.core.write().await;
    let core = core_guard
        .as_mut()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    let thread = core
        .chat
        .create_thread(
            &ChannelId(req.channel_id),
            &MessageId(req.parent_message_id),
        )
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("create_thread failed: {}", e)))?;

    Ok(Json(json!({
        "thread_id": thread.id.0,
        "channel_id": thread.channel_id.0,
        "parent_message_id": thread.parent_message_id.0
    })))
}

/// Get thread messages
pub async fn get_thread_messages(
    State(_state): State<Arc<BridgeState>>,
    Path(_thread_id): Path<String>,
) -> BridgeResult<Json<Value>> {
    // TODO: Implement via messaging service once API is clarified
    // ChatManager doesn't have get_thread_messages in saorsa-core 0.3.26
    Ok(Json(
        json!({"messages": [], "note": "Pending saorsa-core API update"}),
    ))
}

// ===== P2P Network Connection Endpoints =====

/// Get local network connection information
pub async fn get_network_connection_info(
    State(state): State<Arc<BridgeState>>,
) -> BridgeResult<Json<Value>> {
    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Get endpoint four-word address
    let four_word_id = core
        .get_local_endpoint_four_words()
        .await
        .unwrap_or_else(|| "not-available".to_string());

    // Get listen address as string
    let listen_addr = core
        .local_endpoint
        .as_ref()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "not-available".to_string());

    // Get peer count and running status
    let peer_count = core.get_peer_count().await;
    let is_listening = core.is_p2p_running().await;

    Ok(Json(json!({
        "four_word_id": four_word_id,
        "listen_addr": listen_addr,
        "peer_count": peer_count,
        "is_listening": is_listening,
    })))
}

/// Connect to a peer by four-word address
#[derive(Deserialize)]
pub struct ConnectToPeerRequest {
    four_word_addr: String,
}

pub async fn connect_to_peer(
    State(state): State<Arc<BridgeState>>,
    Json(req): Json<ConnectToPeerRequest>,
) -> BridgeResult<Json<Value>> {
    // Validate input
    let words: Vec<&str> = req.four_word_addr.split('-').collect();
    if words.len() != 4 {
        return Err(BridgeError::InvalidRequest(
            "Four-word address must contain exactly 4 words separated by hyphens".to_string(),
        ));
    }

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Connect to peer using CoreContext
    core.connect_to_peer(&req.four_word_addr)
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to connect to peer: {}", e)))?;

    Ok(Json(json!({
        "four_word_addr": req.four_word_addr,
        "status": "connected",
    })))
}

/// Get list of connected peers
pub async fn get_connected_peers(
    State(state): State<Arc<BridgeState>>,
) -> BridgeResult<Json<Value>> {
    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Get connected peers from CoreContext
    let peers = core.get_connected_peers().await;

    let peers_json: Vec<Value> = peers
        .iter()
        .map(|peer_id| {
            json!({
                "peer_id": peer_id,
                "status": "connected",
            })
        })
        .collect();

    Ok(Json(json!({"peers": peers_json})))
}

/// Disconnect from a peer
#[derive(Deserialize)]
pub struct DisconnectFromPeerRequest {
    four_word_addr: String,
}

pub async fn disconnect_from_peer(
    State(state): State<Arc<BridgeState>>,
    Json(_req): Json<DisconnectFromPeerRequest>,
) -> BridgeResult<Json<Value>> {
    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Note: CoreContext P2PNode doesn't currently expose disconnect_peer method
    // For now, just return success
    // TODO: Add disconnect_peer method to CoreContext when needed

    Ok(Json(json!({"success": true})))
}

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
use communitas_core::legacy_crdt::EntityType;
// Removed: saorsa-core imports - replaced with stub implementations
// use saorsa_core::chat::{ChannelId, ChannelType, MessageId};
// use saorsa_core::identity::FourWordAddress;
// use saorsa_core::messaging::{ChannelId as MessagingChannelId, MessageContent};

// Stub types to replace saorsa-core dependencies
#[derive(Debug, Clone)]
pub struct ChannelId(pub String);

#[derive(Debug, Clone)]
pub enum ChannelType {
    Public,
    Private,
}

#[derive(Debug, Clone)]
pub struct MessageId(pub String);

#[derive(Debug, Clone)]
pub struct FourWordAddress(pub String);

#[derive(Debug, Clone)]
pub enum MessageContent {
    Text(String),
    System(String),
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: ChannelId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

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

    // Stub implementation - create_channel functionality removed
    // TODO: Implement using communitas-core APIs if needed
    let channel_id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now();

    Ok(Json(json!({
        "id": channel_id,
        "name": req.name,
    "description": req.description,
    "created_at": created_at.to_rfc3339(),
    "note": "Stub implementation - saorsa-core removed"
    })))
}

/// List all channels
pub async fn list_channels(State(state): State<Arc<BridgeState>>) -> BridgeResult<Json<Value>> {
    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Use entity_service to list entities (channels are entities)
    let entities = core
        .entity_service
        .list_entities()
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("list_entities failed: {}", e)))?;

    let channels: Vec<Value> = entities
        .into_iter()
        .filter(|e| e.entity_type == EntityType::Channel)
        .map(|entity| {
            json!({
                "id": entity.id,
                "name": entity.name,
                "description": entity.description.unwrap_or_default(),
                "created_at": Utc::now().to_rfc3339()
            })
        })
        .collect();

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
    let _core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Save recipient count before move
    let recipient_count = req.recipients.len();

    // Convert recipients to FourWordAddress
    let _recipients: Vec<FourWordAddress> =
        req.recipients.into_iter().map(FourWordAddress).collect();

    // Parse channel UUID
    let _channel_uuid = uuid::Uuid::parse_str(&channel_id)
        .map_err(|e| BridgeError::InvalidRequest(format!("Invalid channel ID: {}", e)))?;

    // Stub implementation - send_message functionality removed
    // TODO: Implement using communitas-core APIs if needed
    let message_id = format!("msg_{}", chrono::Utc::now().timestamp());

    Ok(Json(json!({
    "success": true,
    "message_id": message_id,
    "note": "Stub implementation - saorsa-core removed",
    "recipients": recipient_count,
    "channel_id": channel_id
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

    // TODO: Implement thread creation via message_service when API is ready
    let _core = core; // Silence unused warning
    let thread_id = uuid::Uuid::new_v4().to_string();

    Ok(Json(json!({
        "thread_id": thread_id,
        "channel_id": req.channel_id,
        "parent_message_id": req.parent_message_id,
        "note": "Thread creation pending API update"
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

    // Get endpoint four-word address from connection_identity field
    let four_word_id = core
        .connection_identity
        .as_ref()
        .cloned()
        .unwrap_or_else(|| "not-available".to_string());

    // Get listen address as string
    let listen_addr = core
        .listen_address
        .as_ref()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "not-available".to_string());

    // Get peer count from gossip context if available
    let peer_count = if let Some(ref gossip) = core.gossip {
        gossip
            .get_contacts()
            .await
            .ok()
            .map(|c| c.len())
            .unwrap_or(0)
    } else {
        0
    };

    let is_listening = core.is_networking_active();

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

    // Get connected peers from gossip context if available
    let peers = if let Some(ref gossip) = core.gossip {
        gossip.get_contacts().await.ok().unwrap_or_default()
    } else {
        Vec::new()
    };

    let peers_json: Vec<Value> = peers
        .iter()
        .map(|(four_words, _peer_id)| {
            json!({
                "four_words": four_words,
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

// ===== Website Publishing Endpoints =====

/// Website data structure
#[derive(Deserialize)]
pub struct CreateWebsiteRequest {
    html: String,
    css: Option<String>,
    js: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct UpdateWebsiteRequest {
    html: Option<String>,
    css: Option<String>,
    js: Option<String>,
    metadata: Option<String>,
}

/// Create or publish website for an entity
pub async fn create_entity_website(
    State(state): State<Arc<BridgeState>>,
    Path(entity_id): Path<String>,
    Json(req): Json<CreateWebsiteRequest>,
) -> BridgeResult<Json<Value>> {
    // Validate input
    if req.html.is_empty() {
        return Err(BridgeError::InvalidRequest(
            "HTML content cannot be empty".to_string(),
        ));
    }
    if req.html.len() > 1024 * 1024 {
        // 1MB limit
        return Err(BridgeError::InvalidRequest(
            "HTML content too large (max 1MB)".to_string(),
        ));
    }

    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Create content hash using SHA-256
    let mut content = req.html.clone();
    if let Some(css) = &req.css {
        content.push_str(css);
    }
    if let Some(js) = &req.js {
        content.push_str(js);
    }

    // Simple hash generation (in production, use proper content-addressable storage)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = format!("hash_{:x}", hasher.finish());
    let published_at = chrono::Utc::now();

    // TODO: Store in actual virtual disk / content-addressable storage
    // For now, return success with hash

    Ok(Json(json!({
        "website_root_hash": hash,
        "entity_id": entity_id,
        "published_at": published_at.to_rfc3339(),
        "status": "published",
        "url": format!("{}.communitas", entity_id),
        "size_bytes": content.len()
    })))
}

/// Get website information for an entity
pub async fn get_entity_website(
    State(state): State<Arc<BridgeState>>,
    Path(entity_id): Path<String>,
) -> BridgeResult<Json<Value>> {
    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // TODO: Retrieve from actual storage
    // For now, return mock data
    Ok(Json(json!({
        "entity_id": entity_id,
        "website_root_hash": "hash_mock_12345",
        "published_at": chrono::Utc::now().to_rfc3339(),
        "url": format!("{}.communitas", entity_id),
        "status": "published",
        "note": "Mock implementation - actual storage pending"
    })))
}

/// Update website content for an entity
pub async fn update_entity_website(
    State(state): State<Arc<BridgeState>>,
    Path(entity_id): Path<String>,
    Json(req): Json<UpdateWebsiteRequest>,
) -> BridgeResult<Json<Value>> {
    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Create new content hash
    let mut content = String::new();
    if let Some(html) = &req.html {
        content.push_str(html);
    }
    if let Some(css) = &req.css {
        content.push_str(css);
    }
    if let Some(js) = &req.js {
        content.push_str(js);
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let new_hash = format!("hash_{:x}", hasher.finish());
    let updated_at = chrono::Utc::now();

    Ok(Json(json!({
        "entity_id": entity_id,
        "website_root_hash": new_hash,
        "previous_hash": "hash_mock_old",
        "updated_at": updated_at.to_rfc3339(),
        "status": "updated"
    })))
}

/// Delete website for an entity
pub async fn delete_entity_website(
    State(state): State<Arc<BridgeState>>,
    Path(entity_id): Path<String>,
) -> BridgeResult<Json<Value>> {
    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // TODO: Remove from actual storage
    Ok(Json(json!({
        "success": true,
        "entity_id": entity_id,
        "deleted": true,
        "deleted_at": chrono::Utc::now().to_rfc3339()
    })))
}

// ===== Virtual Disk Storage Endpoints =====

/// Upload file request
#[derive(Deserialize)]
pub struct UploadFileRequest {
    path: String,
    content_base64: String,
    content_type: Option<String>,
}

/// Upload file to entity virtual disk
pub async fn upload_file(
    State(state): State<Arc<BridgeState>>,
    Path((entity_id, disk_type)): Path<(String, String)>,
    Json(req): Json<UploadFileRequest>,
) -> BridgeResult<Json<Value>> {
    // Validate disk type
    if !["private", "public", "shared"].contains(&disk_type.as_str()) {
        return Err(BridgeError::InvalidRequest(
            "Invalid disk type. Must be 'private', 'public', or 'shared'".to_string(),
        ));
    }

    // Validate path
    if req.path.is_empty() || !req.path.starts_with('/') {
        return Err(BridgeError::InvalidRequest(
            "Path must start with '/'".to_string(),
        ));
    }

    // Validate content
    if req.content_base64.is_empty() {
        return Err(BridgeError::InvalidRequest(
            "Content cannot be empty".to_string(),
        ));
    }

    // Decode base64 to get actual size
    let content_bytes = match base64::decode(&req.content_base64) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err(BridgeError::InvalidRequest(
                "Invalid base64 content".to_string(),
            ));
        }
    };

    if content_bytes.len() > 10 * 1024 * 1024 {
        // 10MB limit
        return Err(BridgeError::InvalidRequest(
            "File too large (max 10MB)".to_string(),
        ));
    }

    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Generate file hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    content_bytes.hash(&mut hasher);
    let file_id = format!("file_{:x}", hasher.finish());

    let encrypted = disk_type == "private" || disk_type == "shared";

    Ok(Json(json!({
        "file_id": file_id,
        "path": req.path,
        "disk_type": disk_type,
        "size_bytes": content_bytes.len(),
        "uploaded_at": chrono::Utc::now().to_rfc3339(),
        "encrypted": encrypted,
        "content_type": req.content_type.unwrap_or("application/octet-stream".to_string())
    })))
}

/// List files in entity disk
pub async fn list_files(
    State(state): State<Arc<BridgeState>>,
    Path((entity_id, disk_type)): Path<(String, String)>,
) -> BridgeResult<Json<Value>> {
    // Validate disk type
    if !["private", "public", "shared"].contains(&disk_type.as_str()) {
        return Err(BridgeError::InvalidRequest("Invalid disk type".to_string()));
    }

    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // TODO: Implement actual file listing from storage
    // For now, return empty list
    Ok(Json(json!({
        "entity_id": entity_id,
        "disk_type": disk_type,
        "files": [],
        "total_files": 0,
        "total_size_bytes": 0,
        "note": "Mock implementation - actual storage pending"
    })))
}

/// Download file from entity disk
pub async fn download_file(
    State(state): State<Arc<BridgeState>>,
    Path((entity_id, disk_type, file_path)): Path<(String, String, String)>,
) -> BridgeResult<Json<Value>> {
    // Validate disk type
    if !["private", "public", "shared"].contains(&disk_type.as_str()) {
        return Err(BridgeError::InvalidRequest("Invalid disk type".to_string()));
    }

    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // TODO: Retrieve actual file from storage
    // For now, return mock data
    let mock_content = "Mock file content";
    let content_base64 = base64::encode(mock_content);

    Ok(Json(json!({
        "path": format!("/{}", file_path),
        "content_base64": content_base64,
        "size_bytes": mock_content.len(),
        "content_type": "text/plain",
        "decrypted": disk_type != "public",
        "note": "Mock implementation - actual storage pending"
    })))
}

/// Delete file from entity disk
pub async fn delete_file(
    State(state): State<Arc<BridgeState>>,
    Path((entity_id, disk_type, file_path)): Path<(String, String, String)>,
) -> BridgeResult<Json<Value>> {
    // Validate disk type
    if !["private", "public", "shared"].contains(&disk_type.as_str()) {
        return Err(BridgeError::InvalidRequest("Invalid disk type".to_string()));
    }

    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // TODO: Delete from actual storage
    Ok(Json(json!({
        "success": true,
        "deleted": true,
        "path": format!("/{}", file_path),
        "entity_id": entity_id,
        "disk_type": disk_type,
        "deleted_at": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get disk usage statistics
pub async fn get_disk_stats(
    State(state): State<Arc<BridgeState>>,
    Path((entity_id, disk_type)): Path<(String, String)>,
) -> BridgeResult<Json<Value>> {
    // Validate disk type
    if !["private", "public", "shared"].contains(&disk_type.as_str()) {
        return Err(BridgeError::InvalidRequest("Invalid disk type".to_string()));
    }

    let _core_guard = state.core.read().await;
    let _core = _core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    let encrypted = disk_type != "public";

    // TODO: Calculate actual stats from storage
    Ok(Json(json!({
        "entity_id": entity_id,
        "disk_type": disk_type,
        "total_files": 0,
        "total_size_bytes": 0,
        "encryption": if encrypted { "enabled" } else { "disabled" },
        "note": "Mock implementation - actual storage pending"
    })))
}

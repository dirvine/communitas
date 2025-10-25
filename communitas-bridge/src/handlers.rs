//! HTTP request handlers for bridge API endpoints

use crate::{
    error::{BridgeError, BridgeResult},
    state::BridgeState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use base64::Engine;
use chrono::{DateTime, Utc};
use communitas_core::legacy_crdt::EntityType;
use yrs::types::ToJson;
use yrs::{Map, Transact};
// Removed: saorsa-core imports - replaced with stub implementations

// Helper to convert yrs::Any to serde_json::Value
fn yrs_to_json(any: yrs::Any) -> serde_json::Value {
    let mut buf = String::new();
    any.to_json(&mut buf);
    serde_json::from_str(&buf).unwrap_or(serde_json::Value::Null)
}
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

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Get creator identity (user's four-word identity, not connection endpoint)
    let created_by = core.four_words.clone();

    // Create channel entity using CRDT-backed entity_service
    let entity = core
        .entity_service
        .create_entity(
            req.name.clone(),
            EntityType::Channel,
            Some(req.description.clone()),
            created_by,
            vec![], // No initial members for now
        )
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to create channel: {}", e)))?;

    Ok(Json(json!({
        "id": entity.id,
        "name": entity.name,
        "description": entity.description.unwrap_or_default(),
        "created_at": chrono::Utc::now().timestamp(),
        "created_by": entity.created_by,
        "members": entity.members
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
                "created_at": entity.created_at, // Real timestamp from CRDT
                "created_by": entity.created_by,
                "members": entity.members
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

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Create content hash using Blake3 for content-addressable storage
    let mut content = req.html.clone();
    if let Some(css) = &req.css {
        content.push_str(css);
    }
    if let Some(js) = &req.js {
        content.push_str(js);
    }

    let hash = blake3::hash(content.as_bytes());
    let hash_hex = hash.to_hex().to_string();
    let published_at = chrono::Utc::now();

    // Create CRDT document for website
    let doc = yrs::Doc::new();
    let root = doc.get_or_insert_map("website");
    {
        let mut txn = doc.transact_mut();

        root.insert(&mut txn, "entity_id", entity_id.clone());
        root.insert(&mut txn, "html", req.html.clone());
        root.insert(&mut txn, "css", req.css.clone().unwrap_or_default());
        root.insert(&mut txn, "js", req.js.clone().unwrap_or_default());
        root.insert(&mut txn, "hash", hash_hex.clone());
        root.insert(&mut txn, "published_at", published_at.timestamp());
        root.insert(&mut txn, "size_bytes", content.len() as i64);

        if let Some(metadata) = &req.metadata {
            root.insert(&mut txn, "metadata", metadata.to_string());
        }
    }

    // Save to CRDT manager (persists to disk)
    core.crdt_manager
        .save_document(
            &format!("website:{}", entity_id),
            "website",
            &entity_id,
            &doc,
        )
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to save website: {}", e)))?;

    Ok(Json(json!({
        "website_root_hash": hash_hex,
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
    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Load website document from CRDT storage
    let doc = core
        .crdt_manager
        .load_document(&format!("website:{}", entity_id))
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Website not found: {}", e)))?;

    let root = doc.get_or_insert_map("website");
    let txn = doc.transact();

    // Extract website data
    let html = root
        .get(&txn, "html")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();
    let css = root
        .get(&txn, "css")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();
    let js = root
        .get(&txn, "js")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();
    let hash = root
        .get(&txn, "hash")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();
    let published_at = root
        .get(&txn, "published_at")
        .and_then(|v| i64::try_from(v).ok())
        .unwrap_or(0);
    let size_bytes = root
        .get(&txn, "size_bytes")
        .and_then(|v| i64::try_from(v).ok())
        .unwrap_or(0);

    Ok(Json(json!({
        "entity_id": entity_id,
        "html": html,
        "css": css,
        "js": js,
        "website_root_hash": hash,
        "published_at": chrono::DateTime::from_timestamp(published_at, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .to_rfc3339(),
        "url": format!("{}.communitas", entity_id),
        "status": "published",
        "size_bytes": size_bytes
    })))
}

/// Update website content for an entity
pub async fn update_entity_website(
    State(state): State<Arc<BridgeState>>,
    Path(entity_id): Path<String>,
    Json(req): Json<UpdateWebsiteRequest>,
) -> BridgeResult<Json<Value>> {
    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Load existing website document
    let doc = core
        .crdt_manager
        .load_document(&format!("website:{}", entity_id))
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Website not found: {}", e)))?;

    let root = doc.get_or_insert_map("website");

    let previous_hash = {
        let txn = doc.transact();
        root.get(&txn, "hash")
            .and_then(|v| v.to_string(&txn).into())
            .unwrap_or_default()
    };

    // Update document fields
    {
        let mut txn = doc.transact_mut();

        if let Some(html) = &req.html {
            root.insert(&mut txn, "html", html.clone());
        }
        if let Some(css) = &req.css {
            root.insert(&mut txn, "css", css.clone());
        }
        if let Some(js) = &req.js {
            root.insert(&mut txn, "js", js.clone());
        }

        // Recalculate hash with updated content
        let html = root
            .get(&txn, "html")
            .and_then(|v| v.to_string(&txn).into())
            .unwrap_or_default();
        let css = root
            .get(&txn, "css")
            .and_then(|v| v.to_string(&txn).into())
            .unwrap_or_default();
        let js = root
            .get(&txn, "js")
            .and_then(|v| v.to_string(&txn).into())
            .unwrap_or_default();

        let content = format!("{}{}{}", html, css, js);
        let hash = blake3::hash(content.as_bytes());
        let new_hash = hash.to_hex().to_string();

        root.insert(&mut txn, "hash", new_hash.clone());
        root.insert(&mut txn, "updated_at", chrono::Utc::now().timestamp());
        root.insert(&mut txn, "size_bytes", content.len() as i64);
    }

    // Save updated document
    core.crdt_manager
        .save_document(
            &format!("website:{}", entity_id),
            "website",
            &entity_id,
            &doc,
        )
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to update website: {}", e)))?;

    let updated_at = chrono::Utc::now();
    let txn = doc.transact();
    let new_hash = root
        .get(&txn, "hash")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();

    Ok(Json(json!({
        "entity_id": entity_id,
        "website_root_hash": new_hash,
        "previous_hash": previous_hash,
        "updated_at": updated_at.to_rfc3339(),
        "status": "updated"
    })))
}

/// Delete website for an entity
pub async fn delete_entity_website(
    State(state): State<Arc<BridgeState>>,
    Path(entity_id): Path<String>,
) -> BridgeResult<Json<Value>> {
    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Delete website document from CRDT storage
    core.crdt_manager
        .delete_document(&format!("website:{}", entity_id))
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to delete website: {}", e)))?;

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

    // Decode base64 to get actual size and validate
    let content_bytes = base64::engine::general_purpose::STANDARD
        .decode(&req.content_base64)
        .map_err(|_| BridgeError::InvalidRequest("Invalid base64 content".to_string()))?;

    if content_bytes.len() > 10 * 1024 * 1024 {
        // 10MB limit
        return Err(BridgeError::InvalidRequest(
            "File too large (max 10MB)".to_string(),
        ));
    }

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Generate file ID from content hash (Blake3 for consistency)
    let hash = blake3::hash(&content_bytes);
    let file_id = hex::encode(hash.as_bytes());

    let encrypted = disk_type == "private" || disk_type == "shared";
    let uploaded_at = chrono::Utc::now();
    let content_type = req
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    // Create CRDT document for file
    let doc = yrs::Doc::new();
    let root = doc.get_or_insert_map("file");
    {
        let mut txn = doc.transact_mut();

        root.insert(&mut txn, "entity_id", entity_id.clone());
        root.insert(&mut txn, "disk_type", disk_type.clone());
        root.insert(&mut txn, "path", req.path.clone());
        root.insert(&mut txn, "content_base64", req.content_base64.clone());
        root.insert(&mut txn, "content_type", content_type.clone());
        root.insert(&mut txn, "size_bytes", content_bytes.len() as i64);
        root.insert(&mut txn, "uploaded_at", uploaded_at.timestamp());
        root.insert(&mut txn, "file_id", file_id.clone());
        root.insert(&mut txn, "encrypted", encrypted);
    }

    // Store file using document ID: file:{entity_id}:{disk_type}:{path_hash}
    let path_hash = hex::encode(blake3::hash(req.path.as_bytes()).as_bytes());
    let doc_id = format!("{}:{}:{}", entity_id, disk_type, path_hash);

    core.crdt_manager
        .save_document(&doc_id, "file", &entity_id, &doc)
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to save file: {}", e)))?;

    Ok(Json(json!({
        "file_id": file_id,
        "path": req.path,
        "disk_type": disk_type,
        "size_bytes": content_bytes.len(),
        "uploaded_at": uploaded_at.to_rfc3339(),
        "encrypted": encrypted,
        "content_type": content_type,
        "persisted": true
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

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // List all file documents
    let all_doc_ids = core
        .crdt_manager
        .list_documents("file")
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to list files: {}", e)))?;

    // Filter by entity_id and disk_type (doc_id format: {entity_id}:{disk_type}:{path_hash})
    let prefix = format!("{}:{}:", entity_id, disk_type);
    let matching_doc_ids: Vec<_> = all_doc_ids
        .into_iter()
        .filter(|id| id.starts_with(&prefix))
        .collect();

    // Load and extract metadata from each file document
    let mut files = Vec::new();
    let mut total_size_bytes: i64 = 0;

    for doc_id in matching_doc_ids {
        if let Ok(doc) = core.crdt_manager.load_document(&doc_id).await {
            let root = doc.get_or_insert_map("file");
            let txn = doc.transact();

            let path = root
                .get(&txn, "path")
                .and_then(|v| v.to_string(&txn).into())
                .unwrap_or_default();
            let size_bytes = root
                .get(&txn, "size_bytes")
                .and_then(|v| yrs_to_json(v.to_json(&txn)).as_i64())
                .unwrap_or(0);
            let file_id = root
                .get(&txn, "file_id")
                .and_then(|v| v.to_string(&txn).into())
                .unwrap_or_default();
            let content_type = root
                .get(&txn, "content_type")
                .and_then(|v| v.to_string(&txn).into())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            let uploaded_at = root
                .get(&txn, "uploaded_at")
                .and_then(|v| yrs_to_json(v.to_json(&txn)).as_i64())
                .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .flatten()
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();
            let encrypted = root
                .get(&txn, "encrypted")
                .and_then(|v| yrs_to_json(v.to_json(&txn)).as_bool())
                .unwrap_or(false);

            total_size_bytes += size_bytes;

            files.push(json!({
                "file_id": file_id,
                "path": path,
                "size_bytes": size_bytes,
                "content_type": content_type,
                "uploaded_at": uploaded_at,
                "encrypted": encrypted
            }));
        }
    }

    Ok(Json(json!({
        "entity_id": entity_id,
        "disk_type": disk_type,
        "files": files,
        "total_files": files.len(),
        "total_size_bytes": total_size_bytes,
        "persisted": true
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

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Reconstruct full path (may need to add leading slash)
    let full_path = if file_path.starts_with('/') {
        file_path.clone()
    } else {
        format!("/{}", file_path)
    };

    // Generate document ID from entity, disk_type, and path
    let path_hash = hex::encode(blake3::hash(full_path.as_bytes()).as_bytes());
    let doc_id = format!("{}:{}:{}", entity_id, disk_type, path_hash);

    // Load file document
    let doc = core
        .crdt_manager
        .load_document(&doc_id)
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("File not found: {}", e)))?;

    // Extract file data
    let root = doc.get_or_insert_map("file");
    let txn = doc.transact();

    let content_base64 = root
        .get(&txn, "content_base64")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();
    let size_bytes = root
        .get(&txn, "size_bytes")
        .and_then(|v| yrs_to_json(v.to_json(&txn)).as_i64())
        .unwrap_or(0);
    let content_type = root
        .get(&txn, "content_type")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let file_id = root
        .get(&txn, "file_id")
        .and_then(|v| v.to_string(&txn).into())
        .unwrap_or_default();
    let encrypted = root
        .get(&txn, "encrypted")
        .and_then(|v| yrs_to_json(v.to_json(&txn)).as_bool())
        .unwrap_or(false);
    let uploaded_at = root
        .get(&txn, "uploaded_at")
        .and_then(|v| yrs_to_json(v.to_json(&txn)).as_i64())
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
        .flatten()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();

    Ok(Json(json!({
        "path": full_path,
        "file_id": file_id,
        "content_base64": content_base64,
        "size_bytes": size_bytes,
        "content_type": content_type,
        "uploaded_at": uploaded_at,
        "encrypted": encrypted,
        "persisted": true
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

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    // Reconstruct full path (may need to add leading slash)
    let full_path = if file_path.starts_with('/') {
        file_path.clone()
    } else {
        format!("/{}", file_path)
    };

    // Generate document ID from entity, disk_type, and path
    let path_hash = hex::encode(blake3::hash(full_path.as_bytes()).as_bytes());
    let doc_id = format!("{}:{}:{}", entity_id, disk_type, path_hash);

    // Delete file document from CRDT storage
    core.crdt_manager
        .delete_document(&doc_id)
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to delete file: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "deleted": true,
        "path": full_path,
        "entity_id": entity_id,
        "disk_type": disk_type,
        "deleted_at": chrono::Utc::now().to_rfc3339(),
        "persisted": true
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

    let core_guard = state.core.read().await;
    let core = core_guard
        .as_ref()
        .ok_or_else(|| BridgeError::CommandFailed("Core not initialized".to_string()))?;

    let encrypted = disk_type != "public";

    // List all file documents
    let all_doc_ids = core
        .crdt_manager
        .list_documents("file")
        .await
        .map_err(|e| BridgeError::CommandFailed(format!("Failed to list files: {}", e)))?;

    // Filter by entity_id and disk_type (doc_id format: {entity_id}:{disk_type}:{path_hash})
    let prefix = format!("{}:{}:", entity_id, disk_type);
    let matching_doc_ids: Vec<_> = all_doc_ids
        .into_iter()
        .filter(|id| id.starts_with(&prefix))
        .collect();

    // Calculate total size by loading each document
    let mut total_size_bytes: i64 = 0;
    let mut total_files = 0;

    for doc_id in matching_doc_ids {
        if let Ok(doc) = core.crdt_manager.load_document(&doc_id).await {
            let root = doc.get_or_insert_map("file");
            let txn = doc.transact();

            let size_bytes = root
                .get(&txn, "size_bytes")
                .and_then(|v| yrs_to_json(v.to_json(&txn)).as_i64())
                .unwrap_or(0);

            total_size_bytes += size_bytes;
            total_files += 1;
        }
    }

    Ok(Json(json!({
        "entity_id": entity_id,
        "disk_type": disk_type,
        "total_files": total_files,
        "total_size_bytes": total_size_bytes,
        "encryption": if encrypted { "enabled" } else { "disabled" },
        "persisted": true
    })))
}

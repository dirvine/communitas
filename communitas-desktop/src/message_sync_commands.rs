//! Tauri Commands for Message Synchronization
//!
//! Exposes CRDT-based message sync functionality to the frontend.
//! Supports get_all_messages(), out-of-order detection, and missing message sync.

use communitas_core::crdt::{
    CRDTMessage, EntitySyncState, EntityType, MessageContent, MissingRange,
    SyncRequest, SyncResponse, VectorClock,
};
use communitas_core::message_sync::MessageSyncService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global message sync service (one per app instance)
pub type MessageSyncState = Arc<RwLock<Option<MessageSyncService>>>;

/// Initialize the message sync service with our peer ID
#[tauri::command]
pub async fn message_sync_initialize(
    state: tauri::State<'_, MessageSyncState>,
    peer_id: String,
) -> Result<(), String> {
    let service = MessageSyncService::new(peer_id);
    let mut guard = state.write().await;
    *guard = Some(service);
    Ok(())
}

/// Get all messages for an entity (contact, group, project, org, channel)
/// This is called when another peer requests sync from us
#[tauri::command]
pub async fn message_sync_get_all_messages(
    state: tauri::State<'_, MessageSyncState>,
    entity_id: String,
) -> Result<SyncResponse, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    service
        .get_all_messages(&entity_id)
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))
}

/// Receive an incoming message - detects out-of-order and missing dependencies
#[tauri::command]
pub async fn message_sync_receive_message(
    state: tauri::State<'_, MessageSyncState>,
    message: CRDTMessage,
) -> Result<ReceiveResultDto, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    let result = service
        .receive_message(message)
        .await
        .map_err(|e| format!("Failed to receive message: {}", e))?;

    Ok(ReceiveResultDto {
        accepted: result.accepted,
        out_of_order: result.out_of_order,
        missing_ranges: result.missing_ranges,
    })
}

/// Send a new message - assigns vector clock and Lamport timestamp
#[tauri::command]
pub async fn message_sync_send_message(
    state: tauri::State<'_, MessageSyncState>,
    entity_id: String,
    entity_type: String,
    text: String,
    author: String,
    reply_to_id: Option<String>,
) -> Result<CRDTMessage, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    let entity_type_enum = parse_entity_type(&entity_type)?;

    let content = MessageContent {
        text,
        author,
        attachments: None,
    };

    service
        .send_message(entity_id, entity_type_enum, content, reply_to_id)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))
}

/// Request sync from a peer - send our vector clock and get missing messages
#[tauri::command]
pub async fn message_sync_request_sync(
    state: tauri::State<'_, MessageSyncState>,
    entity_id: String,
    from_peer_id: String,
) -> Result<SyncRequest, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    service
        .request_sync(&entity_id, &from_peer_id)
        .await
        .map_err(|e| format!("Failed to request sync: {}", e))
}

/// Handle sync response - integrate received messages
#[tauri::command]
pub async fn message_sync_handle_sync_response(
    state: tauri::State<'_, MessageSyncState>,
    response: SyncResponse,
) -> Result<SyncResultDto, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    let result = service
        .handle_sync_response(response)
        .await
        .map_err(|e| format!("Failed to handle sync response: {}", e))?;

    Ok(SyncResultDto {
        messages_added: result.messages_added,
        messages_rejected: result.messages_rejected,
    })
}

/// Get sync state for an entity
#[tauri::command]
pub async fn message_sync_get_sync_state(
    state: tauri::State<'_, MessageSyncState>,
    entity_id: String,
) -> Result<EntitySyncState, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    service
        .get_sync_state(&entity_id)
        .await
        .map_err(|e| format!("Failed to get sync state: {}", e))
}

/// Get all messages for an entity in causal order
#[tauri::command]
pub async fn message_sync_get_messages(
    state: tauri::State<'_, MessageSyncState>,
    entity_id: String,
) -> Result<Vec<CRDTMessage>, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    service
        .get_messages(&entity_id)
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))
}

/// Check if we need to request a sync (missing messages)
#[tauri::command]
pub async fn message_sync_needs_sync(
    state: tauri::State<'_, MessageSyncState>,
    entity_id: String,
    remote_clock: VectorClock,
) -> Result<bool, String> {
    let guard = state.read().await;
    let service = guard.as_ref().ok_or("MessageSyncService not initialized")?;

    Ok(service.needs_sync(&entity_id, &remote_clock).await)
}

// Helper DTOs for serialization

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiveResultDto {
    pub accepted: bool,
    pub out_of_order: bool,
    pub missing_ranges: Option<Vec<MissingRange>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResultDto {
    pub messages_added: usize,
    pub messages_rejected: usize,
}

// Helper function to parse entity type string
fn parse_entity_type(entity_type: &str) -> Result<EntityType, String> {
    match entity_type.to_lowercase().as_str() {
        "person" => Ok(EntityType::Person),
        "group" => Ok(EntityType::Group),
        "project" => Ok(EntityType::Project),
        "channel" => Ok(EntityType::Channel),
        "organisation" | "organization" => Ok(EntityType::Organisation),
        _ => Err(format!("Invalid entity type: {}", entity_type)),
    }
}

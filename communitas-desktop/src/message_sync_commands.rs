//! Tauri Commands for Message Synchronization
//!
//! Thin wrappers around CoreContext MessageService.
//! These commands now delegate to the unified MessageService in communitas-core,
//! eliminating code duplication between desktop and TUI applications.

use communitas_core::crdt::{
    CRDTMessage, EntitySyncState, EntityType, MessageContent, MissingRange, SyncRequest,
    SyncResponse, VectorClock,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Initialize the message sync service with our peer ID
/// Note: MessageService is now initialized as part of CoreContext
#[tauri::command]
pub async fn message_sync_initialize(
    _core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    _peer_id: String,
) -> Result<(), String> {
    // MessageService is now automatically initialized with CoreContext
    // This command is kept for backward compatibility
    Ok(())
}

/// Get all messages for an entity (contact, group, project, org, channel)
/// This is called when another peer requests sync from us
#[tauri::command]
pub async fn message_sync_get_all_messages(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    entity_id: String,
) -> Result<SyncResponse, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    core_ctx
        .message_service
        .get_entity_messages(entity_id)
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))
}

/// Receive an incoming message - detects out-of-order and missing dependencies
#[tauri::command]
pub async fn message_sync_receive_message(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    message: CRDTMessage,
) -> Result<ReceiveResultDto, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    let result = core_ctx
        .message_service
        .receive_message(message)
        .await
        .map_err(|e| format!("Failed to receive message: {}", e))?;

    Ok(ReceiveResultDto {
        accepted: result.accepted,
        out_of_order: result.out_of_order,
        missing_ranges: Some(result.missing_ranges),
    })
}

/// Send a new message - assigns vector clock and Lamport timestamp
#[tauri::command]
pub async fn message_sync_send_message(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    entity_id: String,
    entity_type: String,
    text: String,
    author: String,
    reply_to_id: Option<String>,
) -> Result<CRDTMessage, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    let entity_type_enum = parse_entity_type(&entity_type)?;

    let content = MessageContent {
        text,
        author,
        attachments: None,
    };

    core_ctx
        .message_service
        .send_message(entity_id, entity_type_enum, content, reply_to_id)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))
}

/// Request sync from a peer - send our vector clock and get missing messages
#[tauri::command]
pub async fn message_sync_request_sync(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    entity_id: String,
    from_peer_id: String,
) -> Result<SyncRequest, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    // Create a sync request with our current state
    let sync_state = core_ctx
        .message_service
        .get_entity_sync_state(entity_id.clone(), EntityType::Channel)
        .await
        .map_err(|e| format!("Failed to get sync state: {}", e))?;

    Ok(SyncRequest {
        entity_id,
        entity_type: EntityType::Channel,
        requester_peer_id: from_peer_id,
        vector_clock: sync_state.vector_clock,
        missing_message_ids: Some(sync_state.missing_messages),
    })
}

/// Handle sync response - integrate received messages
#[tauri::command]
pub async fn message_sync_handle_sync_response(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    response: SyncResponse,
) -> Result<SyncResultDto, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    // For sync response handling, we need to process each message individually
    // This is a simplified implementation
    let total_messages = response.messages.len();
    let mut messages_added = 0;
    for message in response.messages {
        match core_ctx.message_service.receive_message(message).await {
            Ok(result) if result.accepted => messages_added += 1,
            _ => {} // Message rejected or error
        }
    }

    Ok(SyncResultDto {
        messages_added,
        messages_rejected: total_messages - messages_added,
    })
}

/// Get sync state for an entity
#[tauri::command]
pub async fn message_sync_get_sync_state(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    entity_id: String,
) -> Result<EntitySyncState, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    // Use default entity type - this could be made more sophisticated
    let entity_type = EntityType::Channel;

    core_ctx
        .message_service
        .get_entity_sync_state(entity_id, entity_type)
        .await
        .map_err(|e| format!("Failed to get sync state: {}", e))
}

/// Get all messages for an entity in causal order
#[tauri::command]
pub async fn message_sync_get_messages(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    entity_id: String,
) -> Result<Vec<CRDTMessage>, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    let response = core_ctx
        .message_service
        .get_entity_messages(entity_id)
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    Ok(response.messages)
}

/// Check if we need to request a sync (missing messages)
#[tauri::command]
pub async fn message_sync_needs_sync(
    core_state: State<'_, Arc<RwLock<Option<communitas_core::CoreContext>>>>,
    entity_id: String,
    remote_clock: VectorClock,
) -> Result<bool, String> {
    let guard = core_state.read().await;
    let core_ctx = guard.as_ref().ok_or("CoreContext not initialized")?;

    // Simplified check - compare our sync state with remote clock
    let entity_type = EntityType::Channel; // Default assumption
    let our_state = core_ctx
        .message_service
        .get_entity_sync_state(entity_id.clone(), entity_type)
        .await
        .map_err(|e| format!("Failed to get sync state: {}", e))?;

    // Check if remote has messages we don't have
    Ok(
        remote_clock.compare(&our_state.vector_clock)
            == communitas_core::crdt::ClockOrdering::After,
    )
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

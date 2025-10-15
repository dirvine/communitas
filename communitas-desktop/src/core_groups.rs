// Copyright (c) 2025 Saorsa Labs Limited
//
// Group management commands
//
// Implements ML-DSA signed group management using GossipContext

use communitas_core::CoreContext;
use communitas_core::gossip::GossipContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{info, error, debug};

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupCreateResult {
    pub id_hex: String,
    pub words: [String; 4],
}

/// Create a group identity with current user as initial member and publish it
#[tauri::command]
pub async fn core_group_create(
    core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
    words: [String; 4],
) -> Result<GroupCreateResult, String> {
    info!("Creating group with four-word identity: {:?}", words);
    
    // Get core context and extract needed data
    let user_id_fw = {
        let guard = core_state.read().await;
        let core_ctx = guard.as_ref()
            .ok_or("CoreContext not initialized - call core_initialize first")?;
        core_ctx.profile.id_fw.clone()
    };
    
    // Validate four-word address
    let group_id = words.join("-");
    
    // All gossip operations must be within the guard scope
    {
        let guard = gossip_state.read().await;
        let gossip_ctx = guard.as_ref()
            .ok_or("GossipContext not initialized - call gossip_initialize first")?;
        
        // Create MLS group for the entity
        if let Err(e) = gossip_ctx.join_entity(&group_id, "group").await {
            error!("Failed to create MLS group for {}: {}", group_id, e);
            return Err(format!("Failed to create group: {}", e));
        }
        
        // Store group metadata in CRDT
        let group_metadata = serde_json::json!({
            "type": "group",
            "id": group_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "created_by": user_id_fw.clone(),
            "members": vec![user_id_fw.clone()],
        });
        
        let metadata_bytes = serde_json::to_vec(&group_metadata)
            .map_err(|e| format!("Failed to serialize group metadata: {}", e))?;
        
        // Store with entity prefix for filtering
        let entity_message = format!("group:{}:{}", group_id, String::from_utf8_lossy(&metadata_bytes));
        if let Err(e) = gossip_ctx.store_message(entity_message.as_bytes().to_vec()).await {
            error!("Failed to store group metadata: {}", e);
            return Err(format!("Failed to store group metadata: {}", e));
        }
    }
    
    info!("Successfully created group: {}", group_id);
    
    Ok(GroupCreateResult {
        id_hex: hex::encode(group_id.as_bytes()),
        words,
    })
}

#[tauri::command]
pub async fn core_group_add_member(
    _core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
    group_words: [String; 4],
    member_words: [String; 4],
) -> Result<bool, String> {
    let group_id = group_words.join("-");
    let member_id = member_words.join("-");
    
    info!("Adding member {} to group {}", member_id, group_id);
    
    // All operations must be within the guard scope
    let result = {
        let guard = gossip_state.read().await;
        let gossip_ctx = guard.as_ref()
            .ok_or("GossipContext not initialized - call gossip_initialize first")?;
        
        // Find the member via FOAF discovery
        let peer_id = match gossip_ctx.find_contact(&member_id).await {
            Ok(peer_id) => {
                debug!("Found member {} via FOAF discovery", member_id);
                peer_id
            }
            Err(e) => {
                error!("Failed to find member {}: {}", member_id, e);
                return Err(format!("Member not found: {}", e));
            }
        };
        
        // Add member to contact cache
        if let Err(e) = gossip_ctx.add_contact(member_id.clone(), peer_id).await {
            error!("Failed to add member to contact cache: {}", e);
            return Err(format!("Failed to add member to contacts: {}", e));
        }
        
        // Get current group metadata
        let current_metadata = {
            let messages = gossip_ctx.get_all_messages().await
                .map_err(|e| format!("Failed to get group metadata: {}", e))?;
            
            // Filter for group-specific messages
            let group_prefix = format!("group:{}:", group_id);
            let group_messages: Vec<Vec<u8>> = messages.iter()
                .filter(|msg| {
                    if let Ok(msg_str) = String::from_utf8(msg.to_vec()) {
                        msg_str.starts_with(&group_prefix)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            
            if group_messages.is_empty() {
                return Err("Group not found".to_string());
            }
            
            // Get the latest metadata message (parse after the prefix)
            let latest_message = group_messages.last().unwrap();
            let msg_str = String::from_utf8(latest_message.to_vec())
                .map_err(|e| format!("Failed to parse group message: {}", e))?;
            
            // Extract JSON part after the prefix
            let json_part = msg_str.strip_prefix(&group_prefix)
                .ok_or("Invalid group message format")?;
            
            serde_json::from_str::<serde_json::Value>(json_part)
                .map_err(|e| format!("Failed to parse group metadata: {}", e))?
        };
        
        // Update members list
        let mut members = current_metadata["members"].as_array()
            .ok_or("Invalid group metadata: members not an array")?
            .clone();
        
        if !members.iter().any(|m| m.as_str() == Some(&member_id)) {
            members.push(serde_json::Value::String(member_id.clone()));
        }
        
        // Create updated metadata
        let updated_metadata = serde_json::json!({
            "type": "group",
            "id": group_id,
            "created_at": current_metadata["created_at"],
            "created_by": current_metadata["created_by"],
            "updated_at": chrono::Utc::now().to_rfc3339(),
            "members": members,
        });
        
        // Store updated metadata
        let metadata_bytes = serde_json::to_vec(&updated_metadata)
            .map_err(|e| format!("Failed to serialize updated group metadata: {}", e))?;
        
        // Store with entity prefix for filtering
        let entity_message = format!("group:{}:{}", group_id, String::from_utf8_lossy(&metadata_bytes));
        if let Err(e) = gossip_ctx.store_message(entity_message.as_bytes().to_vec()).await {
            error!("Failed to store updated group metadata: {}", e);
            return Err(format!("Failed to update group metadata: {}", e));
        }
        
        // Publish member addition event
        let event = serde_json::json!({
            "type": "member_added",
            "group_id": group_id,
            "member_id": member_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| format!("Failed to serialize member addition event: {}", e))?;
        
        if let Err(e) = gossip_ctx.publish_to_entity(&format!("group_events:{}", group_id), event_bytes).await {
            error!("Failed to publish member addition event: {}", e);
            return Err(format!("Failed to publish member addition: {}", e));
        }
        
        true
    };
    
    info!("Successfully added member {} to group {}", member_id, group_id);
    Ok(result)
}

#[tauri::command]
pub async fn core_group_remove_member(
    _core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
    group_words: [String; 4],
    member_words: [String; 4],
) -> Result<bool, String> {
    let group_id = group_words.join("-");
    let member_id = member_words.join("-");
    
    info!("Removing member {} from group {}", member_id, group_id);
    
    // All operations must be within the guard scope
    let result = {
        let guard = gossip_state.read().await;
        let gossip_ctx = guard.as_ref()
            .ok_or("GossipContext not initialized - call gossip_initialize first")?;
        
        // Get current group metadata
        let current_metadata = {
            let messages = gossip_ctx.get_all_messages().await
                .map_err(|e| format!("Failed to get group metadata: {}", e))?;
            
            // Filter for group-specific messages
            let group_prefix = format!("group:{}:", group_id);
            let group_messages: Vec<Vec<u8>> = messages.iter()
                .filter(|msg| {
                    if let Ok(msg_str) = String::from_utf8(msg.to_vec()) {
                        msg_str.starts_with(&group_prefix)
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();
            
            if group_messages.is_empty() {
                return Err("Group not found".to_string());
            }
            
            // Get the latest metadata message (parse after the prefix)
            let latest_message = group_messages.last().unwrap();
            let msg_str = String::from_utf8(latest_message.to_vec())
                .map_err(|e| format!("Failed to parse group message: {}", e))?;
            
            // Extract JSON part after the prefix
            let json_part = msg_str.strip_prefix(&group_prefix)
                .ok_or("Invalid group message format")?;
            
            serde_json::from_str::<serde_json::Value>(json_part)
                .map_err(|e| format!("Failed to parse group metadata: {}", e))?
        };
        
        // Update members list
        let mut members = current_metadata["members"].as_array()
            .ok_or("Invalid group metadata: members not an array")?
            .clone();
        
        members.retain(|m| m.as_str() != Some(&member_id));
        
        // Create updated metadata
        let updated_metadata = serde_json::json!({
            "type": "group",
            "id": group_id,
            "created_at": current_metadata["created_at"],
            "created_by": current_metadata["created_by"],
            "updated_at": chrono::Utc::now().to_rfc3339(),
            "members": members,
        });
        
        // Store updated metadata
        let metadata_bytes = serde_json::to_vec(&updated_metadata)
            .map_err(|e| format!("Failed to serialize updated group metadata: {}", e))?;
        
        // Store with entity prefix for filtering
        let entity_message = format!("group:{}:{}", group_id, String::from_utf8_lossy(&metadata_bytes));
        if let Err(e) = gossip_ctx.store_message(entity_message.as_bytes().to_vec()).await {
            error!("Failed to store updated group metadata: {}", e);
            return Err(format!("Failed to update group metadata: {}", e));
        }
        
        // Publish member removal event
        let event = serde_json::json!({
            "type": "member_removed",
            "group_id": group_id,
            "member_id": member_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        let event_bytes = serde_json::to_vec(&event)
            .map_err(|e| format!("Failed to serialize member removal event: {}", e))?;
        
        if let Err(e) = gossip_ctx.publish_to_entity(&format!("group_events:{}", group_id), event_bytes).await {
            error!("Failed to publish member removal event: {}", e);
            return Err(format!("Failed to publish member removal: {}", e));
        }
        
        true
    };
    
    info!("Successfully removed member {} from group {}", member_id, group_id);
    Ok(result)
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Core gossip overlay commands
//
// Batch 7: Core storage and identity management using gossip mesh network

use communitas_core::CoreContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

#[cfg(feature = "gossip_overlay")]
use crate::gossip_commands;

#[derive(Debug, Serialize, Deserialize)]
pub struct GossipStatus {
    pub connected: bool,
    pub message: String,
}

// Batch 7: Core storage commands

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_claim(
    gossip_state: State<'_, gossip_commands::GossipState>,
    words: [String; 4],
) -> Result<String, String> {
    // Store identity claim in gossip mesh via CRDT
    let words_joined = words.join("-");
    let claim_message = format!("identity_claim:{}", words_joined);

    gossip_commands::gossip_store_message(gossip_state, claim_message.as_bytes().to_vec()).await?;

    Ok(format!("Claimed identity: {}", words_joined))
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_claim(_words: [String; 4]) -> Result<String, String> {
    Err("Core claim not yet implemented with new architecture".to_string())
}

/// Generate a valid four-word identity using the four-word-networking dictionary
///
/// Returns a random four-word identity with valid dictionary words
/// (e.g., "ocean-forest-moon-star")
#[tauri::command]
pub async fn generate_four_word_identity() -> Result<String, String> {
    tracing::info!("Generating four-word identity");

    communitas_core::identity::generate_id_words()
        .map_err(|e| format!("Failed to generate identity: {}", e))
}

#[tauri::command]
pub async fn check_gossip_connection(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<GossipStatus, String> {
    Ok(GossipStatus {
        connected: false,
        message: "Gossip overlay not yet implemented with new architecture".to_string(),
    })
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn core_advertise(
    gossip_state: State<'_, gossip_commands::GossipState>,
    key_hex: String,
    value_hex: String,
) -> Result<(), String> {
    // Store key-value pair in gossip mesh via CRDT
    let advertise_message = format!("advertise:{}:{}", key_hex, value_hex);
    gossip_commands::gossip_store_message(gossip_state, advertise_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn core_advertise(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _key_hex: String,
    _value_hex: String,
) -> Result<(), String> {
    Err("Gossip overlay advertising not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn find_group_storage_disk(
    gossip_state: State<'_, gossip_commands::GossipState>,
    group_id_hex: String,
) -> Result<String, String> {
    // Query gossip mesh for group storage location via CRDT
    let messages = gossip_commands::gossip_get_all_messages(gossip_state).await?;
    let prefix = format!("group_storage:{}:", group_id_hex);

    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg) {
            if let Some(storage_location) = msg_str.strip_prefix(&prefix) {
                return Ok(storage_location.to_string());
            }
        }
    }

    // Default storage location if not found
    Ok(format!("/storage/groups/{}", group_id_hex))
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn find_group_storage_disk(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _group_id_hex: String,
) -> Result<String, String> {
    Err("Group storage not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn store_user_identity(
    gossip_state: State<'_, gossip_commands::GossipState>,
    identity_data: String,
) -> Result<(), String> {
    // Store user identity in gossip mesh via CRDT
    let identity_message = format!("user_identity:{}", identity_data);
    gossip_commands::gossip_store_message(gossip_state, identity_message.as_bytes().to_vec()).await
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn store_user_identity(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _identity_data: String,
) -> Result<(), String> {
    Err("User identity storage not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn find_user_current_address(
    gossip_state: State<'_, gossip_commands::GossipState>,
    user_id: String,
) -> Result<String, String> {
    // Query gossip mesh for user address mapping via CRDT
    let messages = gossip_commands::gossip_get_all_messages(gossip_state).await?;
    let prefix = format!("user_address:{}:", user_id);

    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg) {
            if let Some(address) = msg_str.strip_prefix(&prefix) {
                return Ok(address.to_string());
            }
        }
    }

    Err(format!("User address not found: {}", user_id))
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn find_user_current_address(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _user_id: String,
) -> Result<String, String> {
    Err("User address lookup not yet implemented".to_string())
}

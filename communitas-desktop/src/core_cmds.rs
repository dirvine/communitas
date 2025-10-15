// Copyright (c) 2025 Saorsa Labs Limited
//
// Core gossip overlay commands
//
// Batch 7 implementation - Phase 1: CRDT storage
// TODO Phase 2: Full DHT integration with saorsa-gossip

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
    // Phase 1: Store identity claim in CRDT
    // TODO Phase 2: Integrate with actual identity claiming via DHT
    let words_joined = words.join("-");
    let claim_message = format!("identity_claim:{}", words_joined);

    gossip_commands::gossip_store_message(gossip_state, claim_message.as_bytes().to_vec())
        .await?;

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
    // Phase 1: Store key-value in CRDT
    // TODO Phase 2: Integrate with actual DHT advertising
    let advertise_message = format!("dht_advertise:{}:{}", key_hex, value_hex);
    gossip_commands::gossip_store_message(gossip_state, advertise_message.as_bytes().to_vec())
        .await
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
pub async fn container_put(
    gossip_state: State<'_, gossip_commands::GossipState>,
    data: Vec<u8>,
) -> Result<String, String> {
    // Phase 1: Compute simple OID and store in CRDT
    // TODO Phase 2: Use proper content-addressing (BLAKE3) and DHT storage
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let oid = format!("{:x}", hasher.finish());

    let container_message = format!("container:{}:", oid);
    let mut full_message = container_message.as_bytes().to_vec();
    full_message.extend_from_slice(&data);

    gossip_commands::gossip_store_message(gossip_state, full_message).await?;

    Ok(oid)
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn container_put(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _data: Vec<u8>,
) -> Result<String, String> {
    Err("Container put not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn container_get(
    gossip_state: State<'_, gossip_commands::GossipState>,
    oid_hex: String,
) -> Result<Vec<u8>, String> {
    // Phase 1: Search CRDT for container by OID
    // TODO Phase 2: Query DHT for content-addressed data
    let messages = gossip_commands::gossip_get_all_messages(gossip_state).await?;
    let prefix = format!("container:{}:", oid_hex);

    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg.clone()) {
            if let Some(data_start) = msg_str.find(&prefix) {
                let data_offset = data_start + prefix.len();
                if data_offset < msg.len() {
                    return Ok(msg[data_offset..].to_vec());
                }
            }
        }
    }

    Err(format!("Container not found: {}", oid_hex))
}

#[cfg(not(feature = "gossip_overlay"))]
#[tauri::command]
pub async fn container_get(
    _shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    _oid_hex: String,
) -> Result<Vec<u8>, String> {
    Err("Container get not yet implemented".to_string())
}

#[cfg(feature = "gossip_overlay")]
#[tauri::command]
pub async fn find_group_storage_disk(
    gossip_state: State<'_, gossip_commands::GossipState>,
    group_id_hex: String,
) -> Result<String, String> {
    // Phase 1: Query CRDT for group storage location
    // TODO Phase 2: Integrate with actual group storage management
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
    // Phase 1: Store identity in CRDT
    // TODO Phase 2: Integrate with proper identity management
    let identity_message = format!("user_identity:{}", identity_data);
    gossip_commands::gossip_store_message(gossip_state, identity_message.as_bytes().to_vec())
        .await
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
    // Phase 1: Query CRDT for user address mapping
    // TODO Phase 2: Integrate with DHT-based user directory
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

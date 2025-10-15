//! Tauri Commands for Gossip Overlay Integration
//!
//! Provides GossipContext-based APIs to replace DHT functionality.
//! Supports FOAF discovery, presence beacons, Plumtree pub/sub, and CRDT storage.

use base64::Engine;
use communitas_core::gossip::GossipContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global gossip context (one per app instance)
pub type GossipState = Arc<RwLock<Option<GossipContext>>>;

// ===== Initialization Commands =====

/// Initialize GossipContext with four-word identity
#[tauri::command]
pub async fn gossip_initialize(
    state: tauri::State<'_, GossipState>,
    four_words: String,
    display_name: String,
    device_name: String,
) -> Result<bool, String> {
    let ctx = GossipContext::initialize(four_words, display_name, device_name)
        .await
        .map_err(|e| format!("Failed to initialize GossipContext: {}", e))?;

    let mut guard = state.write().await;
    *guard = Some(ctx);
    Ok(true)
}

// ===== Storage Commands (CRDT-based) =====

/// Store a message in local CRDT set

#[tauri::command]
pub async fn gossip_store_message(
    state: tauri::State<'_, GossipState>,
    message: Vec<u8>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.store_message(message)
        .await
        .map_err(|e| format!("Failed to store message: {}", e))
}

/// Retrieve all messages from local CRDT

#[tauri::command]
pub async fn gossip_get_all_messages(
    state: tauri::State<'_, GossipState>,
) -> Result<Vec<Vec<u8>>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.get_all_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))
}

/// Check if a message exists in local CRDT

#[tauri::command]
pub async fn gossip_contains_message(
    state: tauri::State<'_, GossipState>,
    message: Vec<u8>,
) -> Result<bool, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.contains_message(&message)
        .await
        .map_err(|e| format!("Failed to check message: {}", e))
}

/// Remove a message from local CRDT

#[tauri::command]
pub async fn gossip_remove_message(
    state: tauri::State<'_, GossipState>,
    message: Vec<u8>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.remove_message(&message)
        .await
        .map_err(|e| format!("Failed to remove message: {}", e))
}

// ===== Contact Discovery Commands (FOAF + Presence) =====

/// Find contact via FOAF discovery + presence
/// Returns peer_id as hex-encoded bytes for use in other commands

#[tauri::command]
pub async fn gossip_find_contact(
    state: tauri::State<'_, GossipState>,
    four_words: String,
) -> Result<ContactEntry, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let peer_id = ctx
        .find_contact(&four_words)
        .await
        .map_err(|e| format!("Failed to find contact: {}", e))?;

    // Return both four_words and peer_id for consistency
    Ok(ContactEntry {
        four_words: four_words.clone(),
        peer_id: hex::encode(peer_id.as_bytes()),
    })
}

/// Add known contact to local cache
/// For internal use - typically contacts are added automatically via find_contact

#[tauri::command]
pub async fn gossip_add_contact(
    state: tauri::State<'_, GossipState>,
    _four_words: String,
    four_words_to_add: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // First find the contact to get their PeerId
    let peer_id = ctx
        .find_contact(&four_words_to_add)
        .await
        .map_err(|e| format!("Failed to find contact {}: {}", four_words_to_add, e))?;

    // Then add to cache
    ctx.add_contact(four_words_to_add, peer_id)
        .await
        .map_err(|e| format!("Failed to add contact: {}", e))
}

/// Get all cached contacts

#[tauri::command]
pub async fn gossip_get_contacts(
    state: tauri::State<'_, GossipState>,
) -> Result<Vec<ContactEntry>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let contacts = ctx
        .get_contacts()
        .await
        .map_err(|e| format!("Failed to get contacts: {}", e))?;

    Ok(contacts
        .into_iter()
        .map(|(four_words, peer_id)| ContactEntry {
            four_words,
            peer_id: peer_id.to_string(),
        })
        .collect())
}

/// Remove contact from cache

#[tauri::command]
pub async fn gossip_remove_contact(
    state: tauri::State<'_, GossipState>,
    four_words: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.remove_contact(&four_words)
        .await
        .map_err(|e| format!("Failed to remove contact: {}", e))
}

// ===== Messaging Commands (Plumtree Pub/Sub) =====

/// Send direct message to peer by their four-word address
/// This will look up the peer via FOAF/presence first

#[tauri::command]
pub async fn gossip_send_direct_message(
    state: tauri::State<'_, GossipState>,
    four_words: String,
    message: Vec<u8>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Find the peer first to get their PeerId
    let peer_id = ctx
        .find_contact(&four_words)
        .await
        .map_err(|e| format!("Failed to find contact {}: {}", four_words, e))?;

    ctx.send_direct_message(peer_id, message)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))
}

/// Subscribe to entity's topic

#[tauri::command]
pub async fn gossip_subscribe_to_entity(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Subscribe and start background task to forward messages to frontend
    let _rx = ctx
        .subscribe_to_entity(&entity_id)
        .await
        .map_err(|e| format!("Failed to subscribe: {}", e))?;

    // TODO: Forward messages to frontend via Tauri events
    // tokio::spawn(async move {
    //     while let Some((sender, msg)) = rx.recv().await {
    //         // emit("gossip-message-received", { entity_id, sender, message: msg })
    //     }
    // });

    Ok(())
}

/// Publish message to entity's topic

#[tauri::command]
pub async fn gossip_publish_to_entity(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
    message: Vec<u8>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.publish_to_entity(&entity_id, message)
        .await
        .map_err(|e| format!("Failed to publish: {}", e))
}

// ===== Group Management Commands =====

/// Join entity (creates MLS group + subscribes to topic)

#[tauri::command]
pub async fn gossip_join_entity(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
    entity_type: String, // "channel", "project", "org"
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.join_entity(&entity_id, &entity_type)
        .await
        .map_err(|e| format!("Failed to join entity: {}", e))
}

/// Leave entity (unsubscribe + leave MLS group)

#[tauri::command]
pub async fn gossip_leave_entity(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.leave_entity(&entity_id)
        .await
        .map_err(|e| format!("Failed to leave entity: {}", e))
}

// ===== Presence Commands =====

/// Start sending presence beacons (5min interval)

#[tauri::command]
pub async fn gossip_start_presence_beacons(
    state: tauri::State<'_, GossipState>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.start_presence_beacons()
        .await
        .map_err(|e| format!("Failed to start presence: {}", e))
}

/// Stop presence beacons

#[tauri::command]
pub async fn gossip_stop_presence_beacons(
    state: tauri::State<'_, GossipState>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.stop_presence_beacons()
        .await
        .map_err(|e| format!("Failed to stop presence: {}", e))
}

/// Check if peer is online in any shared group by their four-word address

#[tauri::command]
pub async fn gossip_is_peer_online(
    state: tauri::State<'_, GossipState>,
    four_words: String,
) -> Result<bool, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Find the peer first to get their PeerId
    let peer_id = ctx
        .find_contact(&four_words)
        .await
        .map_err(|e| format!("Failed to find contact {}: {}", four_words, e))?;

    ctx.is_peer_online(peer_id)
        .await
        .map_err(|e| format!("Failed to check presence: {}", e))
}

/// Get online peers in entity

#[tauri::command]
pub async fn gossip_get_online_peers(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
) -> Result<Vec<String>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let peers = ctx
        .get_online_peers(&entity_id)
        .await
        .map_err(|e| format!("Failed to get online peers: {}", e))?;

    Ok(peers.into_iter().map(|p| p.to_string()).collect())
}

// ===== Backup & Recovery Commands =====

/// Add favourite contact for encrypted backups

#[tauri::command]
pub async fn gossip_add_favourite_contact(
    state: tauri::State<'_, GossipState>,
    four_words: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.add_favourite_contact(four_words)
        .await
        .map_err(|e| format!("Failed to add favourite: {}", e))
}

/// Get list of favourite contacts

#[tauri::command]
pub async fn gossip_get_favourite_contacts(
    state: tauri::State<'_, GossipState>,
) -> Result<Vec<String>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    Ok(ctx.get_favourite_contacts().await)
}

/// Replicate state to all favourite contacts

#[tauri::command]
pub async fn gossip_replicate_to_favourites(
    state: tauri::State<'_, GossipState>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.replicate_to_favourites()
        .await
        .map_err(|e| format!("Failed to replicate: {}", e))
}

/// Recover state from a favourite contact
///
/// Decrypts backup using ChaCha20Poly1305 and merges into local CRDT

#[tauri::command]
pub async fn gossip_recover_from_favourite(
    state: tauri::State<'_, GossipState>,
    four_words: String,
    encrypted_package: Vec<u8>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    ctx.recover_from_favourite(&four_words, encrypted_package)
        .await
        .map_err(|e| format!("Failed to recover: {}", e))
}

// ===== Saorsa Sites Commands (SPEC2.md §5 - Rendezvous Protocol) =====

/// Publish a site with assets

#[tauri::command]
pub async fn gossip_site_publish(
    state: tauri::State<'_, GossipState>,
    assets: Vec<AssetData>,
) -> Result<String, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let publisher = ctx
        .site_publisher
        .as_ref()
        .ok_or("SitePublisher not initialized")?;

    // Add all assets and collect their hashes
    let mut asset_paths = Vec::new();
    for asset in assets {
        let content = base64::engine::general_purpose::STANDARD
            .decode(&asset.content_base64)
            .map_err(|e| format!("Failed to decode asset: {}", e))?;

        let hash = publisher
            .add_asset(asset.path.clone(), content)
            .await
            .map_err(|e| format!("Failed to add asset: {}", e))?;

        asset_paths.push((asset.path, hash));
    }

    // Build manifest with version 1 and collected asset paths
    let manifest = publisher
        .build_manifest(1, asset_paths)
        .await
        .map_err(|e| format!("Failed to build manifest: {}", e))?;

    // Return site_id as hex
    Ok(hex::encode(manifest.site_id.as_bytes()))
}

/// Fetch a site by ID

#[tauri::command]
pub async fn gossip_site_fetch(
    state: tauri::State<'_, GossipState>,
    site_id_hex: String,
) -> Result<SiteData, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let fetcher = ctx
        .site_fetcher
        .as_ref()
        .ok_or("SiteFetcher not initialized")?;

    // Parse site_id
    let site_id_bytes =
        hex::decode(&site_id_hex).map_err(|e| format!("Invalid site_id hex: {}", e))?;
    let site_id_array: [u8; 32] = site_id_bytes
        .try_into()
        .map_err(|_| "site_id must be 32 bytes".to_string())?;
    let site_id = communitas_core::gossip::SiteId::new(site_id_array);

    // Start discovery
    fetcher
        .start_discovery(&site_id)
        .await
        .map_err(|e| format!("Discovery failed: {}", e))?;

    // Get providers
    let providers = fetcher.get_providers(&site_id).await;

    if providers.is_empty() {
        return Err("No providers found for site".to_string());
    }

    // Fetch manifest from first provider
    let provider_summary = &providers[0];
    let provider_peer_id = provider_summary.provider;
    let manifest = fetcher
        .fetch_manifest(&site_id, provider_peer_id)
        .await
        .map_err(|e| format!("Failed to fetch manifest: {}", e))?;

    // Fetch all blocks
    let mut assets = Vec::new();
    for (path, hash) in &manifest.blocks {
        let block = fetcher
            .fetch_block(hash, provider_peer_id)
            .await
            .map_err(|e| format!("Failed to fetch block: {}", e))?;

        assets.push(AssetData {
            path: path.clone(),
            content_base64: base64::engine::general_purpose::STANDARD.encode(&block.content),
        });
    }

    Ok(SiteData {
        site_id: site_id_hex,
        assets,
    })
}

/// List published sites

#[tauri::command]
pub async fn gossip_site_list(state: tauri::State<'_, GossipState>) -> Result<Vec<String>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let publisher = ctx
        .site_publisher
        .as_ref()
        .ok_or("SitePublisher not initialized")?;

    // Get site_id from manifest
    let manifest = publisher
        .get_manifest()
        .await
        .ok_or("No manifest available - publish a site first")?;

    let site_id = hex::encode(manifest.site_id.as_bytes());

    // For now, just return our own published site
    // In future, this could return a list of discovered sites
    Ok(vec![site_id])
}

/// Get providers for a site

#[tauri::command]
pub async fn gossip_site_providers(
    state: tauri::State<'_, GossipState>,
    site_id_hex: String,
) -> Result<Vec<String>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    let fetcher = ctx
        .site_fetcher
        .as_ref()
        .ok_or("SiteFetcher not initialized")?;

    // Parse site_id
    let site_id_bytes =
        hex::decode(&site_id_hex).map_err(|e| format!("Invalid site_id hex: {}", e))?;
    let site_id_array: [u8; 32] = site_id_bytes
        .try_into()
        .map_err(|_| "site_id must be 32 bytes".to_string())?;
    let site_id = communitas_core::gossip::SiteId::new(site_id_array);

    // Get providers
    let providers = fetcher.get_providers(&site_id).await;

    // Convert PeerIds to hex strings
    let provider_hexes = providers
        .into_iter()
        .map(|p| hex::encode(p.provider.as_bytes()))
        .collect();

    Ok(provider_hexes)
}

// ===== Connection Status Commands =====

/// Get own four-word identity

#[tauri::command]
pub async fn gossip_get_own_identity(
    state: tauri::State<'_, GossipState>,
) -> Result<String, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get four-word identity from context
    Ok(ctx.four_words().to_string())
}

/// Get connection status (online/offline, peer count, etc.)

#[tauri::command]
pub async fn gossip_get_connection_status(
    state: tauri::State<'_, GossipState>,
) -> Result<ConnectionStatus, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get peer count from contacts as proxy for connection status
    let contacts = ctx
        .get_contacts()
        .await
        .map_err(|e| format!("Failed to get contacts: {}", e))?;

    let peer_count = contacts.len();
    let online = peer_count > 0;
    let four_words = ctx.four_words().to_string();

    Ok(ConnectionStatus {
        online,
        four_words,
        peer_count,
    })
}

/// Add bootstrap peer via four-word address
/// This peer will be used to bootstrap into the network

#[tauri::command]
pub async fn gossip_add_bootstrap_peer(
    state: tauri::State<'_, GossipState>,
    four_words: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Try to find the peer (may fail if offline or not discovered yet)
    // Even if it fails, we can add to contacts for future discovery
    match ctx.find_contact(&four_words).await {
        Ok(peer_id) => {
            // Successfully found peer - add to contacts
            ctx.add_contact(four_words, peer_id)
                .await
                .map_err(|e| format!("Failed to add contact: {}", e))?;
            Ok(())
        }
        Err(e) => {
            // Not found yet - that's OK, they'll be discovered later
            Err(format!(
                "Peer not found (will retry on next connection): {}",
                e
            ))
        }
    }
}

/// Get known contacts (for bootstrap/connection info)

#[tauri::command]
pub async fn gossip_get_cached_peers(
    state: tauri::State<'_, GossipState>,
) -> Result<Vec<BootstrapPeer>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get contacts from context
    let contacts = ctx
        .get_contacts()
        .await
        .map_err(|e| format!("Failed to get contacts: {}", e))?;

    Ok(contacts
        .into_iter()
        .map(|(four_words, peer_id)| {
            BootstrapPeer {
                four_words,
                peer_id: peer_id.to_string(),
                last_seen: 0,      // TODO: Track last seen timestamp
                success_rate: 1.0, // Assume success for known contacts
            }
        })
        .collect())
}

// ===== Entity Tracking Commands =====

/// Get list of entities the user has subscribed to
///
/// Uses CRDT storage with "entity_sub:{entity_id}" prefix to track subscriptions

#[tauri::command]
pub async fn gossip_get_subscribed_entities(
    state: tauri::State<'_, GossipState>,
) -> Result<Vec<String>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get all messages and filter for entity subscription markers
    let messages = ctx
        .get_all_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let mut entities = Vec::new();
    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg) {
            if let Some(entity_id) = msg_str.strip_prefix("entity_sub:") {
                entities.push(entity_id.to_string());
            }
        }
    }

    Ok(entities)
}

/// Get list of subscribers for a specific entity
///
/// Note: This requires network-wide queries which are not yet implemented.
/// For now, returns empty list. Future implementation should query the gossip
/// network for all peers subscribed to the entity topic.

#[tauri::command]
pub async fn gossip_get_entity_subscribers(
    _state: tauri::State<'_, GossipState>,
    _entity_id: String,
) -> Result<Vec<String>, String> {
    // TODO: Implement network-wide query for entity subscribers
    // This requires extending GossipContext with subscriber tracking
    // For now, return empty list
    Ok(vec![])
}

/// Get messages filtered by entity ID
///
/// Uses CRDT storage with "entity_msg:{entity_id}:" prefix for entity-specific messages

#[tauri::command]
pub async fn gossip_get_entity_messages(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
) -> Result<Vec<Vec<u8>>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get all messages and filter for entity-specific ones
    let messages = ctx
        .get_all_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let prefix = format!("entity_msg:{}:", entity_id);
    let mut entity_messages = Vec::new();

    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg.clone()) {
            if let Some(content) = msg_str.strip_prefix(&prefix) {
                entity_messages.push(content.as_bytes().to_vec());
            }
        }
    }

    Ok(entity_messages)
}

/// Store a message tagged with entity ID
///
/// Helper function to store messages with entity prefix for filtering

#[tauri::command]
pub async fn gossip_store_entity_message(
    state: tauri::State<'_, GossipState>,
    entity_id: String,
    message: Vec<u8>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Store with entity prefix
    let content = String::from_utf8(message)
        .map_err(|e| format!("Message must be valid UTF-8: {}", e))?;
    let prefixed_message = format!("entity_msg:{}:{}", entity_id, content);

    ctx.store_message(prefixed_message.as_bytes().to_vec())
        .await
        .map_err(|e| format!("Failed to store entity message: {}", e))
}

// ===== Bootstrap Management Commands =====

/// Clear all bootstrap peers from cache
///
/// Removes all contacts which effectively clears bootstrap peer cache

#[tauri::command]
pub async fn gossip_clear_bootstrap_peers(
    state: tauri::State<'_, GossipState>,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get all contacts
    let contacts = ctx
        .get_contacts()
        .await
        .map_err(|e| format!("Failed to get contacts: {}", e))?;

    // Remove each contact
    for (four_words, _) in contacts {
        ctx.remove_contact(&four_words)
            .await
            .map_err(|e| format!("Failed to remove contact {}: {}", four_words, e))?;
    }

    Ok(())
}

// ===== Metadata Storage Commands =====

/// Get metadata for a specific peer
///
/// Uses CRDT storage with "peer_meta:{peer_id}:" prefix to store peer metadata
/// Returns a map of key-value pairs for the peer

#[tauri::command]
pub async fn gossip_get_peer_metadata(
    state: tauri::State<'_, GossipState>,
    peer_id: String,
) -> Result<Vec<MetadataEntry>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get all messages and filter for peer metadata
    let messages = ctx
        .get_all_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let prefix = format!("peer_meta:{}:", peer_id);
    let mut metadata = Vec::new();

    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg) {
            if let Some(rest) = msg_str.strip_prefix(&prefix) {
                // Parse "key:value" format
                if let Some((key, value)) = rest.split_once(':') {
                    metadata.push(MetadataEntry {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
            }
        }
    }

    Ok(metadata)
}

/// Store metadata for a peer
///
/// Uses CRDT storage with "peer_meta:{peer_id}:{key}:{value}" format

#[tauri::command]
pub async fn gossip_store_peer_metadata(
    state: tauri::State<'_, GossipState>,
    peer_id: String,
    key: String,
    value: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Store with peer metadata prefix
    let metadata_message = format!("peer_meta:{}:{}:{}", peer_id, key, value);

    ctx.store_message(metadata_message.as_bytes().to_vec())
        .await
        .map_err(|e| format!("Failed to store peer metadata: {}", e))
}

/// Get own metadata (display name, device name, etc.)
///
/// Uses CRDT storage with "self_meta:{key}:{value}" format

#[tauri::command]
pub async fn gossip_get_own_metadata(
    state: tauri::State<'_, GossipState>,
) -> Result<Vec<MetadataEntry>, String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Get all messages and filter for own metadata
    let messages = ctx
        .get_all_messages()
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))?;

    let prefix = "self_meta:";
    let mut metadata = Vec::new();

    for msg in messages {
        if let Ok(msg_str) = String::from_utf8(msg) {
            if let Some(rest) = msg_str.strip_prefix(prefix) {
                // Parse "key:value" format
                if let Some((key, value)) = rest.split_once(':') {
                    metadata.push(MetadataEntry {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
            }
        }
    }

    Ok(metadata)
}

/// Store own metadata (display name, device name, etc.)
///
/// Uses CRDT storage with "self_meta:{key}:{value}" format

#[tauri::command]
pub async fn gossip_store_own_metadata(
    state: tauri::State<'_, GossipState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let guard = state.read().await;
    let ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

    // Store with self metadata prefix
    let metadata_message = format!("self_meta:{}:{}", key, value);

    ctx.store_message(metadata_message.as_bytes().to_vec())
        .await
        .map_err(|e| format!("Failed to store own metadata: {}", e))
}

// ===== DTOs for Serialization =====

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContactEntry {
    pub four_words: String,
    pub peer_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetData {
    pub path: String,
    pub content_base64: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SiteData {
    pub site_id: String,
    pub assets: Vec<AssetData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub online: bool,
    pub four_words: String,
    pub peer_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapPeer {
    pub four_words: String,
    pub peer_id: String,
    pub last_seen: u64,
    pub success_rate: f32,
}

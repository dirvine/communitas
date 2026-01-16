// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Message Synchronization Service
//!
//! Handles CRDT-based message synchronization:
//! - get_all_messages() for full sync requests
//! - Out-of-order message detection and queuing
//! - Missing message reply mechanism
//! - Causal consistency enforcement

use crate::crdt::*;
use crate::error::AppResult;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Message synchronization service
#[derive(Debug)]
pub struct MessageSyncService {
    /// Our peer ID (four-word address)
    peer_id: String,

    /// Vector clock per entity
    entity_clocks: Arc<RwLock<HashMap<String, VectorClock>>>,

    /// Message storage per entity
    entity_messages: Arc<RwLock<HashMap<String, Vec<CRDTMessage>>>>,

    /// Out-of-order messages waiting for dependencies
    pending_messages: Arc<RwLock<HashMap<String, Vec<CRDTMessage>>>>,

    /// Global Lamport clock for total ordering
    lamport_clock: Arc<RwLock<u64>>,
}

impl MessageSyncService {
    /// Create a new message sync service
    pub fn new(peer_id: String) -> Self {
        info!("🔄 MessageSyncService initialized for peer: {}", peer_id);

        Self {
            peer_id,
            entity_clocks: Arc::new(RwLock::new(HashMap::new())),
            entity_messages: Arc::new(RwLock::new(HashMap::new())),
            pending_messages: Arc::new(RwLock::new(HashMap::new())),
            lamport_clock: Arc::new(RwLock::new(0)),
        }
    }

    /// Get all messages for an entity (contact, group, project, org, channel)
    /// This is the entry point for sync requests from other peers
    pub async fn get_all_messages(&self, entity_id: &str) -> AppResult<SyncResponse> {
        let messages_map = self.entity_messages.read().await;
        let clocks_map = self.entity_clocks.read().await;

        let mut messages = messages_map.get(entity_id).cloned().unwrap_or_default();

        let vector_clock = clocks_map.get(entity_id).cloned().unwrap_or_default();

        // Sort in causal order before returning
        sort_messages_causally(&mut messages);

        info!(
            "📤 get_all_messages for {}: {} messages",
            entity_id,
            messages.len()
        );

        Ok(SyncResponse {
            entity_id: entity_id.to_string(),
            entity_type: self.infer_entity_type(entity_id),
            messages,
            vector_clock,
        })
    }

    /// Handle incoming message - detect out-of-order and missing dependencies
    pub async fn receive_message(&self, message: CRDTMessage) -> AppResult<ReceiveResult> {
        let entity_id = message.metadata.entity_id.clone();
        let clocks_map = self.entity_clocks.read().await;

        let local_clock = clocks_map.get(&entity_id).cloned().unwrap_or_default();

        // Release read lock before potential writes
        drop(clocks_map);

        // Check if we have all causal dependencies
        let has_deps = local_clock.has_dependencies(&message.metadata.vector_clock);

        if !has_deps {
            // Message is out of order - store in pending queue
            warn!("⚠️  Out-of-order message detected: {}", message.metadata.id);

            let mut pending_map = self.pending_messages.write().await;
            let pending = pending_map.entry(entity_id.clone()).or_default();
            pending.push(message.clone());

            // Calculate what we're missing
            let missing = local_clock.get_missing_ranges(&message.metadata.vector_clock);

            return Ok(ReceiveResult {
                accepted: false,
                out_of_order: true,
                missing_ranges: Some(missing),
            });
        }

        // Message has all dependencies - accept it
        self.add_message(message).await?;

        // Try to process pending messages
        self.process_pending_messages(&entity_id).await?;

        Ok(ReceiveResult {
            accepted: true,
            out_of_order: false,
            missing_ranges: None,
        })
    }

    /// Send a new message - assigns vector clock and Lamport timestamp
    pub async fn send_message(
        &self,
        entity_id: String,
        entity_type: EntityType,
        content: MessageContent,
        reply_to_id: Option<String>,
    ) -> AppResult<CRDTMessage> {
        // Increment our vector clock
        let mut clocks_map = self.entity_clocks.write().await;
        let clock = clocks_map.entry(entity_id.clone()).or_default();
        clock.increment(&self.peer_id);
        let new_clock = clock.clone();
        drop(clocks_map);

        // Increment Lamport clock
        let mut lamport = self.lamport_clock.write().await;
        *lamport += 1;
        let lamport_value = *lamport;
        drop(lamport);

        // Get previous message for causal chain
        let messages_map = self.entity_messages.read().await;
        let previous_id = messages_map
            .get(&entity_id)
            .and_then(|msgs| msgs.last())
            .map(|msg| msg.metadata.id.clone());
        drop(messages_map);

        let metadata = MessageMetadata {
            id: format!(
                "{}-{}-{}",
                self.peer_id,
                new_clock.0.get(&self.peer_id).copied().unwrap_or(0),
                chrono::Utc::now().timestamp_millis()
            ),
            entity_id: entity_id.clone(),
            entity_type,
            author_peer_id: self.peer_id.clone(),
            vector_clock: new_clock,
            lamport_clock: lamport_value,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            previous_message_id: previous_id,
            reply_to_id,
        };

        let message = CRDTMessage {
            content,
            metadata,
            local_state: Some(LocalMessageState {
                status: Some(MessageStatus::Sent),
                reactions: Vec::new(),
                edited_at: None,
                thread_count: None,
                latest_reply_by: None,
            }),
        };

        self.add_message(message.clone()).await?;

        Ok(message)
    }

    /// Request sync from a peer - send our vector clock and get missing messages
    pub async fn request_sync(
        &self,
        entity_id: &str,
        from_peer_id: &str,
    ) -> AppResult<SyncRequest> {
        let clocks_map = self.entity_clocks.read().await;
        let local_clock = clocks_map.get(entity_id).cloned().unwrap_or_default();
        drop(clocks_map);

        // Check for specific missing messages
        let pending_map = self.pending_messages.read().await;
        let missing_ids = pending_map
            .get(entity_id)
            .map(|pending| pending.iter().map(|m| m.metadata.id.clone()).collect());
        drop(pending_map);

        debug!("🔄 Requesting sync for {} from {}", entity_id, from_peer_id);
        debug!("   Local clock: {:?}", local_clock);
        debug!("   Missing messages: {:?}", missing_ids);

        Ok(SyncRequest {
            entity_id: entity_id.to_string(),
            entity_type: self.infer_entity_type(entity_id),
            requester_peer_id: self.peer_id.clone(),
            vector_clock: local_clock,
            missing_message_ids: missing_ids,
        })
    }

    /// Handle sync response - integrate received messages
    pub async fn handle_sync_response(&self, response: SyncResponse) -> AppResult<SyncResult> {
        let entity_id = &response.entity_id;
        let mut added = 0;
        let mut rejected = 0;

        info!(
            "📥 Handling sync response for {}: {} messages",
            entity_id,
            response.messages.len()
        );

        // Process each message
        for message in response.messages {
            let result = self.receive_message(message).await?;
            if result.accepted {
                added += 1;
            } else {
                rejected += 1;
            }
        }

        // Merge remote clock into ours
        let mut clocks_map = self.entity_clocks.write().await;
        let local_clock = clocks_map.entry(entity_id.clone()).or_default();
        local_clock.merge(&response.vector_clock);
        let merged_clock = local_clock.clone();
        drop(clocks_map);

        info!("✅ Sync complete: {} added, {} rejected", added, rejected);
        debug!("   Updated clock: {:?}", merged_clock);

        Ok(SyncResult {
            messages_added: added,
            messages_rejected: rejected,
        })
    }

    /// Get sync state for an entity
    pub async fn get_sync_state(&self, entity_id: &str) -> AppResult<EntitySyncState> {
        let messages_map = self.entity_messages.read().await;
        let pending_map = self.pending_messages.read().await;
        let clocks_map = self.entity_clocks.read().await;

        let messages = messages_map.get(entity_id).cloned().unwrap_or_default();
        let pending = pending_map.get(entity_id).cloned().unwrap_or_default();
        let clock = clocks_map.get(entity_id).cloned().unwrap_or_default();

        Ok(EntitySyncState {
            entity_id: entity_id.to_string(),
            entity_type: self.infer_entity_type(entity_id),
            vector_clock: clock,
            last_sync_time: chrono::Utc::now().timestamp_millis() as u64,
            message_count: messages.len(),
            missing_messages: pending.iter().map(|m| m.metadata.id.clone()).collect(),
            out_of_order_messages: pending.iter().map(|m| m.metadata.id.clone()).collect(),
        })
    }

    /// Get all messages in causal order for an entity
    pub async fn get_messages(&self, entity_id: &str) -> AppResult<Vec<CRDTMessage>> {
        let messages_map = self.entity_messages.read().await;
        let mut messages = messages_map.get(entity_id).cloned().unwrap_or_default();

        sort_messages_causally(&mut messages);

        Ok(messages)
    }

    /// Check if we need to request a sync (missing messages)
    pub async fn needs_sync(&self, entity_id: &str, remote_clock: &VectorClock) -> bool {
        let clocks_map = self.entity_clocks.read().await;
        let local_clock = clocks_map.get(entity_id).cloned().unwrap_or_default();

        let missing = local_clock.get_missing_ranges(remote_clock);
        !missing.is_empty()
    }

    pub async fn delete_message(&self, entity_id: &str, message_id: &str) -> AppResult<bool> {
        let mut messages_map = self.entity_messages.write().await;
        if let Some(messages) = messages_map.get_mut(entity_id) {
            let original_len = messages.len();
            messages.retain(|m| m.metadata.id != message_id);
            let deleted = messages.len() < original_len;
            if deleted {
                info!("🗑️ Message deleted: {} (entity: {})", message_id, entity_id);
            }
            return Ok(deleted);
        }
        Ok(false)
    }

    pub async fn edit_message(
        &self,
        entity_id: &str,
        message_id: &str,
        new_text: String,
    ) -> AppResult<u64> {
        let mut messages_map = self.entity_messages.write().await;
        if let Some(messages) = messages_map.get_mut(entity_id) {
            for message in messages.iter_mut() {
                if message.metadata.id == message_id {
                    message.content.text = new_text;
                    let edited_at = chrono::Utc::now().timestamp_millis() as u64;
                    let local_state =
                        message
                            .local_state
                            .get_or_insert_with(|| LocalMessageState {
                                status: None,
                                reactions: Vec::new(),
                                edited_at: None,
                                thread_count: None,
                                latest_reply_by: None,
                            });
                    local_state.edited_at = Some(edited_at);
                    info!("✏️ Message edited: {} (entity: {})", message_id, entity_id);
                    return Ok(edited_at);
                }
            }
        }
        Err(crate::error::AppError::NotFound(format!(
            "Message not found: {}",
            message_id
        )))
    }

    pub async fn add_reaction(
        &self,
        entity_id: &str,
        message_id: &str,
        emoji: String,
        peer_id: String,
    ) -> AppResult<()> {
        let mut messages_map = self.entity_messages.write().await;
        if let Some(messages) = messages_map.get_mut(entity_id) {
            for message in messages.iter_mut() {
                if message.metadata.id == message_id {
                    let local_state =
                        message
                            .local_state
                            .get_or_insert_with(|| LocalMessageState {
                                status: None,
                                reactions: Vec::new(),
                                edited_at: None,
                                thread_count: None,
                                latest_reply_by: None,
                            });

                    if let Some(reaction) =
                        local_state.reactions.iter_mut().find(|r| r.emoji == emoji)
                    {
                        if !reaction.peer_ids.contains(&peer_id) {
                            reaction.peer_ids.push(peer_id.clone());
                            reaction.count += 1;
                        }
                    } else {
                        local_state.reactions.push(Reaction {
                            emoji: emoji.clone(),
                            count: 1,
                            user_reacted: Some(true),
                            peer_ids: vec![peer_id.clone()],
                        });
                    }

                    info!(
                        "👍 Reaction added: {} to {} (entity: {})",
                        emoji, message_id, entity_id
                    );
                    return Ok(());
                }
            }
        }
        Err(crate::error::AppError::NotFound(format!(
            "Message not found: {}",
            message_id
        )))
    }

    pub async fn remove_reaction(
        &self,
        entity_id: &str,
        message_id: &str,
        emoji: String,
        peer_id: String,
    ) -> AppResult<()> {
        let mut messages_map = self.entity_messages.write().await;
        if let Some(messages) = messages_map.get_mut(entity_id) {
            for message in messages.iter_mut() {
                if message.metadata.id == message_id {
                    if let Some(ref mut local_state) = message.local_state
                        && let Some(reaction) =
                            local_state.reactions.iter_mut().find(|r| r.emoji == emoji)
                    {
                        reaction.peer_ids.retain(|p| p != &peer_id);
                        reaction.count = reaction.count.saturating_sub(1);

                        if reaction.count == 0 {
                            local_state.reactions.retain(|r| r.emoji != emoji);
                        }

                        info!(
                            "👎 Reaction removed: {} from {} (entity: {})",
                            emoji, message_id, entity_id
                        );
                        return Ok(());
                    }
                    return Err(crate::error::AppError::NotFound(format!(
                        "Reaction not found: {} on {}",
                        emoji, message_id
                    )));
                }
            }
        }
        Err(crate::error::AppError::NotFound(format!(
            "Message not found: {}",
            message_id
        )))
    }

    pub async fn get_reactions(
        &self,
        entity_id: &str,
        message_id: &str,
    ) -> AppResult<Vec<Reaction>> {
        let messages_map = self.entity_messages.read().await;
        if let Some(messages) = messages_map.get(entity_id) {
            for message in messages.iter() {
                if message.metadata.id == message_id {
                    return Ok(message
                        .local_state
                        .as_ref()
                        .map(|ls| ls.reactions.clone())
                        .unwrap_or_default());
                }
            }
        }
        Err(crate::error::AppError::NotFound(format!(
            "Message not found: {}",
            message_id
        )))
    }

    // Private methods

    async fn add_message(&self, message: CRDTMessage) -> AppResult<()> {
        let entity_id = &message.metadata.entity_id;

        let mut messages_map = self.entity_messages.write().await;
        let messages = messages_map.entry(entity_id.clone()).or_default();

        // Check for duplicates
        if messages
            .iter()
            .any(|m| m.metadata.id == message.metadata.id)
        {
            warn!("⚠️  Duplicate message ignored: {}", message.metadata.id);
            return Ok(());
        }

        messages.push(message.clone());
        drop(messages_map);

        // Update our vector clock
        let mut clocks_map = self.entity_clocks.write().await;
        let local_clock = clocks_map.entry(entity_id.clone()).or_default();
        local_clock.merge(&message.metadata.vector_clock);
        drop(clocks_map);

        // Update Lamport clock
        let mut lamport = self.lamport_clock.write().await;
        *lamport = (*lamport).max(message.metadata.lamport_clock) + 1;
        drop(lamport);

        info!(
            "📨 Message added: {} (entity: {})",
            message.metadata.id, entity_id
        );

        Ok(())
    }

    async fn process_pending_messages(&self, entity_id: &str) -> AppResult<()> {
        // Extract pending messages first
        let mut pending_map = self.pending_messages.write().await;
        let pending_messages = match pending_map.get_mut(entity_id) {
            Some(p) if !p.is_empty() => std::mem::take(p),
            _ => return Ok(()),
        };
        drop(pending_map);

        let clocks_map = self.entity_clocks.read().await;
        let local_clock = clocks_map.get(entity_id).cloned().unwrap_or_default();
        drop(clocks_map);

        let mut still_pending = Vec::new();

        for message in pending_messages {
            if local_clock.has_dependencies(&message.metadata.vector_clock) {
                info!(
                    "✅ Pending message now has dependencies: {}",
                    message.metadata.id
                );
                self.add_message(message).await?;
            } else {
                still_pending.push(message);
            }
        }

        // Update pending map with still-pending messages
        let mut pending_map = self.pending_messages.write().await;
        if !still_pending.is_empty() {
            info!(
                "⏳ Still pending: {} messages for {}",
                still_pending.len(),
                entity_id
            );
            pending_map.insert(entity_id.to_string(), still_pending);
        } else {
            pending_map.remove(entity_id);
            info!("✨ All pending messages processed for {}", entity_id);
        }

        Ok(())
    }

    fn infer_entity_type(&self, entity_id: &str) -> EntityType {
        // Infer from ID prefix
        if entity_id.starts_with("contact-")
            || entity_id.starts_with("ben-")
            || entity_id.starts_with("lauren")
        {
            EntityType::Person
        } else if entity_id.contains("-org") {
            EntityType::Organisation
        } else if entity_id.starts_with("project-") {
            EntityType::Project
        } else if entity_id.contains("general") || entity_id.contains("channel") {
            EntityType::Channel
        } else {
            EntityType::Group
        }
    }
}

/// Result of receiving a message
#[derive(Debug)]
pub struct ReceiveResult {
    pub accepted: bool,
    pub out_of_order: bool,
    pub missing_ranges: Option<Vec<MissingRange>>,
}

/// Result of syncing messages
#[derive(Debug)]
pub struct SyncResult {
    pub messages_added: usize,
    pub messages_rejected: usize,
}

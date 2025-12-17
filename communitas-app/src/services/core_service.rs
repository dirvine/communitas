// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Core Service Wrapper
//!
//! Provides async access to communitas-core functionality.
//! This is the direct Rust integration that avoids FFI overhead.

use communitas_core::{
    CoreContext, crdt::EntityType, entity_service::Entity, generate_id_words, types::DeviceType,
    validate_id_words,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Service wrapper for CoreContext
///
/// Provides async methods for all core operations.
/// Thread-safe and clonable for use with Dioxus signals.
#[derive(Clone)]
pub struct CoreService {
    context: Arc<RwLock<Option<CoreContext>>>,
}

impl Default for CoreService {
    fn default() -> Self {
        Self::new()
    }
}

// Allow dead code for methods that are part of the public API
// but not yet used by the TUI application
#[allow(dead_code)]
impl CoreService {
    /// Create a new uninitialized CoreService
    pub fn new() -> Self {
        Self {
            context: Arc::new(RwLock::new(None)),
        }
    }

    /// Generate a new four-word identity using the official dictionary
    ///
    /// Uses cryptographically secure randomness to generate
    /// a unique four-word identity from the four-word-networking dictionary.
    ///
    /// # Returns
    /// A four-word identity string (e.g., "ocean-forest-moon-star")
    pub fn generate_four_words() -> Result<String, String> {
        generate_id_words().map_err(|e| format!("Failed to generate identity: {}", e))
    }

    /// Validate a four-word identity against the dictionary
    ///
    /// Checks that the identity has exactly 4 words,
    /// separated by dashes, and all words are in the dictionary.
    ///
    /// # Arguments
    /// * `identity` - Four-word identity to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_four_words(identity: &str) -> bool {
        validate_id_words(identity)
    }

    /// Initialize the core context with user credentials
    ///
    /// # Arguments
    /// * `four_words` - Four-word identity (e.g., "ocean-forest-moon-star")
    /// * `display_name` - User's display name
    /// * `device_name` - Device identifier
    ///
    /// # Returns
    /// Ok if initialization succeeds
    pub async fn initialize(
        &self,
        four_words: String,
        display_name: String,
        device_name: String,
    ) -> Result<(), String> {
        info!("Initializing CoreService for {}", four_words);

        // Determine storage directory
        let storage_dir = Self::get_storage_dir()?;

        // Initialize CoreContext
        let ctx = CoreContext::initialize(
            four_words.clone(),
            display_name.clone(),
            device_name,
            DeviceType::Desktop,
            storage_dir,
        )
        .await
        .map_err(|e| {
            error!("Failed to initialize CoreContext: {}", e);
            e
        })?;

        // Store the context
        *self.context.write().await = Some(ctx);

        info!("CoreService initialized successfully for {}", four_words);
        Ok(())
    }

    /// Check if the service is initialized
    #[allow(dead_code)]
    pub async fn is_initialized(&self) -> bool {
        self.context.read().await.is_some()
    }

    /// Get the current four-word identity
    #[allow(dead_code)]
    pub async fn four_words(&self) -> Option<String> {
        self.context
            .read()
            .await
            .as_ref()
            .map(|ctx| ctx.four_words.clone())
    }

    /// Get the current display name
    #[allow(dead_code)]
    pub async fn display_name(&self) -> Option<String> {
        self.context
            .read()
            .await
            .as_ref()
            .map(|ctx| ctx.display_name.clone())
    }

    /// Start P2P networking
    ///
    /// # Arguments
    /// * `port` - Optional specific port to listen on
    ///
    /// # Returns
    /// Connection identity (four-word encoded address)
    #[allow(dead_code)]
    pub async fn start_networking(&self, port: Option<u16>) -> Result<String, String> {
        let mut guard = self.context.write().await;
        let ctx = guard.as_mut().ok_or("CoreService not initialized")?;

        ctx.start_networking(port).await
    }

    /// Check if networking is active
    #[allow(dead_code)]
    pub async fn is_networking_active(&self) -> bool {
        self.context
            .read()
            .await
            .as_ref()
            .map(|ctx| ctx.is_networking_active())
            .unwrap_or(false)
    }

    /// Get platform-specific storage directory
    fn get_storage_dir() -> Result<PathBuf, String> {
        // Use dirs crate for cross-platform directories
        let base_dir = dirs::data_local_dir().ok_or("Could not determine local data directory")?;

        let storage_dir = base_dir.join("communitas");

        // Create if it doesn't exist
        if !storage_dir.exists() {
            std::fs::create_dir_all(&storage_dir)
                .map_err(|e| format!("Failed to create storage directory: {}", e))?;
        }

        Ok(storage_dir)
    }

    /// Shutdown the service gracefully
    #[allow(dead_code)]
    pub async fn shutdown(&self) -> Result<(), String> {
        let mut guard = self.context.write().await;
        if let Some(ctx) = guard.as_mut() {
            ctx.stop_networking().await?;
        }
        *guard = None;
        info!("CoreService shut down");
        Ok(())
    }

    // ===========================================
    // Entity Management Methods
    // ===========================================

    /// Create a new entity (group, channel, project, or organization)
    ///
    /// # Arguments
    /// * `name` - Entity name
    /// * `entity_type` - Type of entity to create
    /// * `description` - Optional description
    ///
    /// # Returns
    /// The created Entity
    pub async fn create_entity(
        &self,
        name: String,
        entity_type: EntityType,
        description: Option<String>,
    ) -> Result<Entity, String> {
        let guard = self.context.read().await;
        let ctx = guard.as_ref().ok_or("CoreService not initialized")?;

        info!("Creating entity '{}' of type {:?}", name, entity_type);

        ctx.entity_service
            .create_entity(
                name,
                entity_type,
                description,
                ctx.four_words.clone(),
                vec![], // Creator will be added automatically
            )
            .await
            .map_err(|e| {
                error!("Failed to create entity: {}", e);
                format!("Failed to create entity: {}", e)
            })
    }

    /// List all entities
    ///
    /// # Returns
    /// Vector of all entities
    pub async fn list_entities(&self) -> Result<Vec<Entity>, String> {
        let guard = self.context.read().await;
        let ctx = guard.as_ref().ok_or("CoreService not initialized")?;

        ctx.entity_service.list_entities().await.map_err(|e| {
            warn!("Failed to list entities: {}", e);
            format!("Failed to list entities: {}", e)
        })
    }

    /// Get entity by ID
    ///
    /// # Arguments
    /// * `entity_id` - Entity UUID
    ///
    /// # Returns
    /// The Entity if found
    #[allow(dead_code)]
    pub async fn get_entity(&self, entity_id: &str) -> Result<Entity, String> {
        let guard = self.context.read().await;
        let ctx = guard.as_ref().ok_or("CoreService not initialized")?;

        ctx.entity_service
            .get_entity(entity_id)
            .await
            .map_err(|e| format!("Failed to get entity: {}", e))
    }

    /// List entities filtered by type
    #[allow(dead_code)]
    pub async fn list_entities_by_type(
        &self,
        entity_type: EntityType,
    ) -> Result<Vec<Entity>, String> {
        let entities = self.list_entities().await?;
        Ok(entities
            .into_iter()
            .filter(|e| e.entity_type == entity_type)
            .collect())
    }

    // ===========================================
    // Messaging Methods (via Gossip Overlay)
    // ===========================================

    /// Store a message in the local CRDT and broadcast via gossip network
    ///
    /// # Arguments
    /// * `content` - Message content as bytes
    ///
    /// # Returns
    /// Ok if message stored successfully
    pub async fn store_message(&self, content: Vec<u8>) -> Result<(), String> {
        let guard = self.context.read().await;
        let ctx = guard.as_ref().ok_or("CoreService not initialized")?;

        let gossip = ctx
            .gossip
            .as_ref()
            .ok_or("Gossip networking not started. Call start_networking() first")?;

        gossip.store_message(content).await.map_err(|e| {
            error!("Failed to store message: {}", e);
            format!("Failed to store message: {}", e)
        })
    }

    /// Get all messages from the local CRDT store
    ///
    /// # Returns
    /// Vector of all messages as byte vectors
    pub async fn get_all_messages(&self) -> Result<Vec<Vec<u8>>, String> {
        let guard = self.context.read().await;
        let ctx = guard.as_ref().ok_or("CoreService not initialized")?;

        let gossip = ctx
            .gossip
            .as_ref()
            .ok_or("Gossip networking not started. Call start_networking() first")?;

        gossip.get_all_messages().await.map_err(|e| {
            warn!("Failed to get messages: {}", e);
            format!("Failed to get messages: {}", e)
        })
    }

    /// Get the number of connected peers in the gossip network
    ///
    /// # Returns
    /// Number of active peers (0 if networking not started)
    pub async fn peer_count(&self) -> usize {
        let guard = self.context.read().await;
        if let Some(ctx) = guard.as_ref()
            && let Some(gossip) = &ctx.gossip
        {
            let membership = gossip.membership.read().await;
            return membership.active_view().len();
        }
        0
    }

    /// Check if gossip networking is active
    pub async fn is_gossip_active(&self) -> bool {
        let guard = self.context.read().await;
        guard
            .as_ref()
            .map(|ctx| ctx.gossip.is_some())
            .unwrap_or(false)
    }

    /// Connect to a peer using their four-word connection identity
    ///
    /// # Arguments
    /// * `peer_connection_id` - The peer's four-word encoded address
    ///
    /// # Returns
    /// Ok if connection initiated successfully
    pub async fn connect_to_peer(&self, peer_connection_id: &str) -> Result<(), String> {
        let guard = self.context.read().await;
        let ctx = guard.as_ref().ok_or("CoreService not initialized")?;

        ctx.connect_to_peer(peer_connection_id).await
    }

    // ===========================================
    // Presence & Peer Discovery Methods
    // ===========================================

    /// Get list of active peer IDs from the gossip membership
    ///
    /// Returns the 32-byte peer IDs of all currently connected peers.
    /// Use this to check who is currently reachable in the network.
    ///
    /// # Returns
    /// Vector of peer ID bytes (empty if networking not started)
    pub async fn get_active_peer_ids(&self) -> Vec<[u8; 32]> {
        let guard = self.context.read().await;
        if let Some(ctx) = guard.as_ref()
            && let Some(gossip) = &ctx.gossip
        {
            let membership = gossip.membership.read().await;
            return membership
                .active_view()
                .iter()
                .map(|peer_id| peer_id.to_bytes())
                .collect();
        }
        Vec::new()
    }

    /// Get connection information for display
    ///
    /// Returns a simplified summary of network status for the UI.
    ///
    /// # Returns
    /// (peer_count, is_connected, connection_identity)
    pub async fn get_connection_info(&self) -> (usize, bool, Option<String>) {
        let guard = self.context.read().await;
        if let Some(ctx) = guard.as_ref()
            && let Some(gossip) = &ctx.gossip
        {
            let membership = gossip.membership.read().await;
            let peer_count = membership.active_view().len();
            let is_connected = peer_count > 0;
            // Use the four-word identity as connection identifier
            let conn_id = Some(gossip.four_words.clone());
            return (peer_count, is_connected, conn_id);
        }
        (0, false, None)
    }

    /// Get our connection identity (four-word identity)
    ///
    /// This is the identity other peers use to connect to us.
    ///
    /// # Returns
    /// Our four-word identity if networking is active
    pub async fn get_our_connection_id(&self) -> Option<String> {
        let guard = self.context.read().await;
        if let Some(ctx) = guard.as_ref()
            && let Some(gossip) = &ctx.gossip
        {
            return Some(gossip.four_words.clone());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_service_new() {
        let service = CoreService::new();
        assert!(service.context.try_read().is_ok());
    }
}

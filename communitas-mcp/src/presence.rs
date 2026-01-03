// Licensed under the AGPL-3.0 license - see LICENSE file for details

//! Presence and status system for real-time collaboration
//! 
//! This module provides user presence, typing indicators, and status management
//! across all entities. It enables features equivalent to WhatsApp, Slack,
//! and Linear's presence systems while maintaining Communitas' P2P, offline-first
//! architecture.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// User presence status within entities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    /// User is actively online and available
    #[serde(rename = "online")]
    Online,
    /// User is away from keyboard
    #[serde(rename = "away")]
    Away,
    /// User is busy/in a meeting
    #[serde(rename = "busy")]
    Busy,
    /// User is invisible (appears offline)
    #[serde(rename = "invisible")]
    Invisible,
    /// User is currently typing a message
    #[serde(rename = "typing")]
    Typing {
        /// ID of the entity where user is typing
        entity_id: String,
        /// Optional message preview
        message_preview: Option<String>,
    },
}

/// User presence information for contacts and entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    /// User's unique identifier
    pub user_id: String,
    /// Current presence status
    pub status: PresenceStatus,
    /// Last time user was seen
    pub last_seen: SystemTime,
    /// Current entity user is active in (if any)
    pub current_entity: Option<String>,
    /// Typing indicators for this user
    pub typing_in: Vec<TypingIndicator>,
}

/// Typing indicator for a user in an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    /// ID of the entity
    pub entity_id: String,
    /// Optional preview of message being typed
    pub message_preview: Option<String>,
    /// Timestamp when typing started
    pub started_at: SystemTime,
}

/// Presence update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    /// New presence status (if changing status)
    pub status: Option<PresenceStatus>,
    /// Current entity (if joining/leaving)
    pub current_entity: Option<String>,
    /// Typing indicator (if starting/stopping typing)
    pub typing: Option<TypingIndicator>,
}

/// Presence subscription configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceSubscription {
    /// Entity IDs to subscribe to for presence updates
    pub entity_ids: Vec<String>,
    /// Include self in presence updates
    pub include_self: bool,
}

impl PresenceUpdate {
    /// Create a simple status update
    pub fn status_only(status: PresenceStatus) -> Self {
        Self {
            status: Some(status),
            current_entity: None,
            typing: None,
        }
    }
}

/// CRDT operations for presence data
pub struct PresenceOperations;

impl PresenceOperations {
    /// Update user presence across all subscribed entities
    pub fn update_presence(
        _app: &communitas_core::app::CommunitasApp,
        user_id: String,
        _update: PresenceUpdate,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Implementation would update presence in CRDT
        // This triggers real-time sync to all subscribers
        // TODO: Implement actual CRDT storage
        tracing::info!("Updating presence for user: {}", user_id);
        Ok(())
    }

    /// Get presence information for multiple users
    pub fn get_users_presence(
        _app: &communitas_core::app::CommunitasApp,
        user_ids: Vec<String>,
    ) -> Result<Vec<Presence>, Box<dyn std::error::Error>> {
        // Implementation would query CRDT for user presence
        let mut presences = Vec::new();
        for user_id in user_ids {
            // TODO: Query CRDT for user presence data
            presences.push(Presence {
                user_id,
                status: PresenceStatus::Online, // Default to online
                last_seen: SystemTime::now(),
                current_entity: None,
                typing_in: Vec::new(),
            });
        }
        Ok(presences)
    }

    /// Subscribe to presence updates for entities
    pub fn subscribe_to_presence(
        _app: &communitas_core::app::CommunitasApp,
        subscription: PresenceSubscription,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Implementation would create CRDT subscription
        // Returns subscription ID for management
        let subscription_id = format!("presence_sub_{}", uuid::Uuid::new_v4());
        tracing::info!("Subscribing to presence for entities: {:?}", subscription.entity_ids);
        Ok(subscription_id)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_status_serialization() {
        let status = PresenceStatus::Online;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"online\""));
    }

    #[test]
    fn test_presence_update_creation() {
        let update = PresenceUpdate::status_only(PresenceStatus::Online);
        assert!(update.status.is_some());
        assert!(update.current_entity.is_none());
        assert!(update.typing.is_none());
    }

    #[test]
    fn test_typing_indicator() {
        let typing = TypingIndicator {
            entity_id: "test-entity".to_string(),
            message_preview: Some("Hello".to_string()),
            started_at: SystemTime::now(),
        };
        assert_eq!(typing.entity_id, "test-entity");
    }
}
// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Test Helper Functions
//!
//! Provides reusable helper functions for common test operations.

use communitas_core::CoreContext;
use communitas_core::crdt::{CRDTMessage, EntityType, MessageContent};
use communitas_core::entity_service::Entity;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};

/// Error type for wait/timeout operations
#[derive(Debug)]
pub struct TimeoutError {
    pub message: String,
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Timeout: {}", self.message)
    }
}

impl std::error::Error for TimeoutError {}

//==============================================================================
// Test Data Generators
//==============================================================================

/// Generate default test four-word identity
pub fn test_four_words() -> [String; 4] {
    [
        "ocean".to_string(),
        "forest".to_string(),
        "moon".to_string(),
        "star".to_string(),
    ]
}

/// Generate alternative test four-word identity
pub fn alt_test_four_words() -> [String; 4] {
    [
        "river".to_string(),
        "mountain".to_string(),
        "cloud".to_string(),
        "tree".to_string(),
    ]
}

/// Generate unique test identifier with timestamp
pub fn test_id(base: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    format!("{}-{}", base, timestamp)
}

/// Generate random test channel name
pub fn test_channel_name() -> String {
    test_id("test-channel")
}

/// Generate random test group name
pub fn test_group_name() -> String {
    test_id("test-group")
}

//==============================================================================
// Channel Operations
//==============================================================================

/// Create a test channel and return its ID
///
/// # Arguments
/// * `ctx` - CoreContext reference
/// * `name` - Channel name
///
/// # Returns
/// Channel ID string
///
/// # Example
/// ```no_run
/// let channel_id = create_test_channel(&ctx, "test-channel").await?;
/// ```
pub async fn create_test_channel(
    ctx: &Arc<RwLock<CoreContext>>,
    name: &str,
) -> Result<String, String> {
    let ctx_guard = ctx.write().await;

    // Use EntityService to create channel
    let entity = ctx_guard
        .entity_service
        .create_entity(
            name.to_string(),
            EntityType::Channel,
            Some(format!("Test channel: {}", name)),
            ctx_guard.four_words.clone(),
            vec![],
        )
        .await
        .map_err(|e| format!("Failed to create channel: {}", e))?;

    tracing::info!("Created test channel '{}' with ID: {}", name, entity.id);
    Ok(entity.id)
}

/// Get all channels for an entity
pub async fn get_channels(ctx: &Arc<RwLock<CoreContext>>) -> Result<Vec<Entity>, String> {
    let ctx_guard = ctx.read().await;

    ctx_guard
        .entity_service
        .list_entities()
        .await
        .map_err(|e| format!("Failed to list entities: {}", e))
        .map(|entities| {
            entities
                .into_iter()
                .filter(|e| matches!(e.entity_type, EntityType::Channel))
                .collect()
        })
}

//==============================================================================
// Message Operations
//==============================================================================

/// Send a test message to a channel
///
/// # Arguments
/// * `ctx` - CoreContext reference
/// * `channel_id` - Target channel ID
/// * `content` - Message content
///
/// # Returns
/// Message ID string
pub async fn send_test_message(
    ctx: &Arc<RwLock<CoreContext>>,
    channel_id: &str,
    content: &str,
) -> Result<String, String> {
    let ctx_guard = ctx.read().await;

    // Create MessageContent struct
    let message_content = MessageContent {
        text: content.to_string(),
        author: ctx_guard.display_name.clone(),
        attachments: None,
    };

    // Use MessageService to send message
    let message = ctx_guard
        .message_service
        .send_message(
            channel_id.to_string(),
            EntityType::Channel,
            message_content,
            None,
        )
        .await
        .map_err(|e| format!("Failed to send message: {}", e))?;

    tracing::info!(
        "Sent test message to channel '{}': '{}'",
        channel_id,
        content
    );
    Ok(message.metadata.id)
}

/// Get all messages in a channel
pub async fn get_channel_messages(
    ctx: &Arc<RwLock<CoreContext>>,
    channel_id: &str,
) -> Result<Vec<CRDTMessage>, String> {
    let ctx_guard = ctx.read().await;

    ctx_guard
        .message_service
        .get_channel_messages(channel_id.to_string())
        .await
        .map_err(|e| format!("Failed to get messages: {}", e))
}

//==============================================================================
// Group Operations
//==============================================================================

/// Create a test group and return its ID
///
/// # Arguments
/// * `ctx` - CoreContext reference
/// * `name` - Group name
///
/// # Returns
/// Group ID string
pub async fn create_test_group(
    ctx: &Arc<RwLock<CoreContext>>,
    name: &str,
) -> Result<String, String> {
    let ctx_guard = ctx.write().await;

    // Use EntityService to create group
    let entity = ctx_guard
        .entity_service
        .create_entity(
            name.to_string(),
            EntityType::Group,
            Some(format!("Test group: {}", name)),
            ctx_guard.four_words.clone(),
            vec![],
        )
        .await
        .map_err(|e| format!("Failed to create group: {}", e))?;

    tracing::info!("Created test group '{}' with ID: {}", name, entity.id);
    Ok(entity.id)
}

/// Add a member to a group
pub async fn add_group_member(
    ctx: &Arc<RwLock<CoreContext>>,
    group_id: &str,
    member_id: &str,
    role: &str,
) -> Result<(), String> {
    let ctx_guard = ctx.write().await;

    ctx_guard
        .entity_service
        .add_member(EntityType::Group, group_id, member_id, role)
        .await
        .map_err(|e| format!("Failed to add member: {}", e))
}

/// Remove a member from a group
pub async fn remove_group_member(
    ctx: &Arc<RwLock<CoreContext>>,
    group_id: &str,
    member_id: &str,
) -> Result<(), String> {
    let ctx_guard = ctx.write().await;
    let deleted_by = ctx_guard.four_words.clone();

    ctx_guard
        .entity_service
        .remove_member(EntityType::Group, group_id, member_id, &deleted_by)
        .await
        .map_err(|e| format!("Failed to remove member: {}", e))
}

/// Get all members of a group
pub async fn get_group_members(
    ctx: &Arc<RwLock<CoreContext>>,
    group_id: &str,
) -> Result<Vec<String>, String> {
    let ctx_guard = ctx.read().await;

    let members = ctx_guard
        .entity_service
        .list_members(EntityType::Group, group_id)
        .await
        .map_err(|e| format!("Failed to list members: {}", e))?;

    // Extract member IDs from MemberInfo structs, filtering out deleted members
    Ok(members
        .into_iter()
        .filter(|m| !m.deleted)
        .map(|m| m.member_id)
        .collect())
}

//==============================================================================
// Wait/Polling Utilities
//==============================================================================

/// Wait for a condition to become true with timeout
///
/// # Arguments
/// * `condition` - Function that returns true when condition is met
/// * `max_duration` - Maximum time to wait
///
/// # Returns
/// Ok if condition met within timeout, TimeoutError otherwise
///
/// # Example
/// ```no_run
/// wait_for(|| channel_exists(&ctx, "test"), Duration::from_secs(5)).await?;
/// ```
pub async fn wait_for<F>(mut condition: F, max_duration: Duration) -> Result<(), TimeoutError>
where
    F: FnMut() -> bool,
{
    let result = timeout(max_duration, async {
        while !condition() {
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    result.map_err(|_| TimeoutError {
        message: format!("Condition not met within {:?}", max_duration),
    })
}

/// Wait for an async condition with timeout
pub async fn wait_for_async<F, Fut>(
    mut condition: F,
    max_duration: Duration,
) -> Result<(), TimeoutError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let result = timeout(max_duration, async {
        while !condition().await {
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    result.map_err(|_| TimeoutError {
        message: format!("Async condition not met within {:?}", max_duration),
    })
}

//==============================================================================
// Test Assertions
//==============================================================================

/// Assert that a channel exists
pub async fn assert_channel_exists(
    ctx: &Arc<RwLock<CoreContext>>,
    channel_id: &str,
) -> Result<(), String> {
    let channels = get_channels(ctx).await?;

    let exists = channels.iter().any(|ch| ch.id == channel_id);

    if exists {
        Ok(())
    } else {
        Err(format!("Channel '{}' does not exist", channel_id))
    }
}

/// Assert that a message exists in a channel
pub async fn assert_message_in_channel(
    ctx: &Arc<RwLock<CoreContext>>,
    channel_id: &str,
    message_content: &str,
) -> Result<(), String> {
    let messages = get_channel_messages(ctx, channel_id).await?;

    let exists = messages
        .iter()
        .any(|msg| msg.content.text == message_content);

    if exists {
        Ok(())
    } else {
        Err(format!(
            "Message '{}' not found in channel '{}'",
            message_content, channel_id
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_four_words_generation() {
        let words = test_four_words();
        assert_eq!(words.len(), 4);
        assert_eq!(words[0], "ocean");
        assert_eq!(words[3], "star");

        let alt_words = alt_test_four_words();
        assert_eq!(alt_words.len(), 4);
        assert_eq!(alt_words[0], "river");
    }

    #[test]
    fn test_id_generation() {
        let id1 = test_id("test");
        let id2 = test_id("test");

        // Should be different due to timestamp
        assert_ne!(id1, id2);
        assert!(id1.starts_with("test-"));
        assert!(id2.starts_with("test-"));
    }

    #[tokio::test]
    async fn test_wait_for_immediate() {
        let result = wait_for(|| true, Duration::from_secs(1)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(|| false, Duration::from_millis(100)).await;
        assert!(result.is_err());
    }
}

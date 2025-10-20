//! Automatic offline handling for Backend operations
//!
//! This module provides "smart" operation wrappers that automatically handle network
//! failures by queueing operations when the network is unavailable and executing them
//! immediately when the network is available.
//!
//! ## Architecture
//!
//! The application never exposes a manual "offline mode" to users. Instead:
//! 1. Operations try immediate execution
//! 2. If network is unavailable (CoreContext not initialized), operation is queued
//! 3. When network returns, queued operations auto-sync
//! 4. Users experience transparent offline->online transitions
//!
//! ## Usage
//!
//! ```rust
//! // Try to create entity - executes immediately if online, queues if offline
//! let result = backend.create_entity_auto(
//!     "My Channel".to_string(),
//!     EntityType::Channel,
//!     vec!["alice-test-one".to_string()],
//! ).await?;
//!
//! match result {
//!     EntityOrQueued::Executed(entity) => {
//!         // Operation succeeded immediately
//!         println!("Created entity: {}", entity.id);
//!     }
//!     EntityOrQueued::Queued(op_id) => {
//!         // Network unavailable - queued for later sync
//!         println!("Operation queued: {}", op_id);
//!     }
//! }
//! ```

use super::Backend;
use super::channels::Entity;
use anyhow::Result;
use communitas_core::crdt::EntityType;

/// Result of an automatic operation - either executed immediately or queued for later
#[derive(Debug, Clone)]
pub enum EntityOrQueued {
    /// Operation executed immediately (network available)
    Executed(Entity),
    /// Operation queued for later sync (network unavailable)
    Queued(String), // operation_id
}

/// Result of a message send operation
#[derive(Debug, Clone)]
pub enum MessageOrQueued {
    /// Message sent immediately (network available)
    Sent(String), // message_id
    /// Message queued for later sync (network unavailable)
    Queued(String), // operation_id
}

/// Result of a member operation
#[derive(Debug, Clone)]
pub enum MemberOperationResult {
    /// Operation completed immediately (network available)
    Completed,
    /// Operation queued for later sync (network unavailable)
    Queued(String), // operation_id
}

impl Backend {
    // ========================================================================
    // Smart Entity Operations
    // ========================================================================

    /// Create entity - executes immediately if online, queues if offline
    ///
    /// This method automatically handles network failures by queueing the operation
    /// when CoreContext is unavailable (indicating network failure or not yet initialized).
    ///
    /// # Arguments
    ///
    /// * `name` - Entity name
    /// * `entity_type` - Type of entity (Channel, Group, Project, etc.)
    /// * `members` - Initial members (four-word addresses)
    ///
    /// # Returns
    ///
    /// * `EntityOrQueued::Executed(entity)` - Entity created immediately
    /// * `EntityOrQueued::Queued(op_id)` - Operation queued for later sync
    ///
    /// # Errors
    ///
    /// Returns error for validation failures, authentication failures, or other
    /// non-network errors. Network failures result in automatic queueing.
    pub async fn create_entity_auto(
        &mut self,
        name: String,
        entity_type: EntityType,
        members: Vec<String>,
    ) -> Result<EntityOrQueued> {
        // Validate and sanitize input BEFORE attempting to create or queue
        // This ensures invalid data never enters the queue
        let sanitized_name = self
            .validator
            .validate_and_sanitize(&name, communitas_core::validation::InputType::EntityName)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Try immediate execution with sanitized name
        match self
            .create_entity(sanitized_name.clone(), entity_type, members.clone())
            .await
        {
            Ok(entity) => Ok(EntityOrQueued::Executed(entity)),
            Err(e) if is_network_error(&e) => {
                // Network failure - queue for later with sanitized name
                let op_id = self
                    .queue_create_entity(sanitized_name, entity_type, members)
                    .await?;
                Ok(EntityOrQueued::Queued(op_id))
            }
            Err(e) => Err(e), // Other errors (validation, auth, etc.) propagate
        }
    }

    // ========================================================================
    // Smart Message Operations
    // ========================================================================

    /// Send message - executes immediately if online, queues if offline
    ///
    /// This method automatically handles network failures by queueing the message
    /// when CoreContext is unavailable.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Target entity ID
    /// * `entity_type` - Type of entity
    /// * `text` - Message text
    ///
    /// # Returns
    ///
    /// * `MessageOrQueued::Sent(message_id)` - Message sent immediately
    /// * `MessageOrQueued::Queued(op_id)` - Message queued for later sync
    ///
    /// # Errors
    ///
    /// Returns error for validation failures or authentication failures.
    /// Network failures result in automatic queueing.
    pub async fn send_message_auto(
        &mut self,
        entity_id: String,
        entity_type: EntityType,
        text: String,
    ) -> Result<MessageOrQueued> {
        // Validate message text BEFORE attempting to send or queue
        // This ensures invalid data never enters the queue
        self.validator
            .validate(&text, communitas_core::validation::InputType::Message)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // Try immediate execution with validated text
        match self
            .send_message(entity_id.clone(), entity_type, text.clone())
            .await
        {
            Ok(message_id) => Ok(MessageOrQueued::Sent(message_id)),
            Err(e) if is_network_error(&e) => {
                // Network failure - queue for later with validated text
                let op_id = self
                    .queue_send_message(entity_id, entity_type, text)
                    .await?;
                Ok(MessageOrQueued::Queued(op_id))
            }
            Err(e) => Err(e), // Other errors propagate
        }
    }

    // ========================================================================
    // Smart Member Operations
    // ========================================================================

    /// Add member - executes immediately if online, queues if offline
    ///
    /// This method automatically handles network failures by queueing the operation
    /// when CoreContext is unavailable.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - Entity ID
    /// * `member_id` - Member four-word address
    ///
    /// # Returns
    ///
    /// * `MemberOperationResult::Completed` - Member added immediately
    /// * `MemberOperationResult::Queued(op_id)` - Operation queued for later sync
    ///
    /// # Errors
    ///
    /// Returns error for validation failures or authentication failures.
    /// Network failures result in automatic queueing.
    pub async fn add_member_auto(
        &mut self,
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
    ) -> Result<MemberOperationResult> {
        // Try immediate execution
        match self
            .add_entity_member(entity_type, &entity_id, member_id.clone())
            .await
        {
            Ok(()) => Ok(MemberOperationResult::Completed),
            Err(e) if is_network_error(&e) => {
                // Network failure - queue for later
                let op_id = self
                    .queue_add_member(entity_id, entity_type, member_id)
                    .await?;
                Ok(MemberOperationResult::Queued(op_id))
            }
            Err(e) => Err(e), // Other errors propagate
        }
    }

    /// Remove member - executes immediately if online, queues if offline
    ///
    /// This method automatically handles network failures by queueing the operation
    /// when CoreContext is unavailable.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - Entity ID
    /// * `member_id` - Member four-word address
    ///
    /// # Returns
    ///
    /// * `MemberOperationResult::Completed` - Member removed immediately
    /// * `MemberOperationResult::Queued(op_id)` - Operation queued for later sync
    ///
    /// # Errors
    ///
    /// Returns error for validation failures or authentication failures.
    /// Network failures result in automatic queueing.
    pub async fn remove_member_auto(
        &mut self,
        entity_type: EntityType,
        entity_id: String,
        member_id: String,
    ) -> Result<MemberOperationResult> {
        // Try immediate execution
        match self
            .remove_entity_member(entity_type, &entity_id, member_id.clone())
            .await
        {
            Ok(()) => Ok(MemberOperationResult::Completed),
            Err(e) if is_network_error(&e) => {
                // Network failure - queue for later
                // Note: We'll need to add queue_remove_member to core.rs
                // For now, return error
                Err(anyhow::anyhow!(
                    "Remove member operation queueing not yet implemented: {}",
                    e
                ))
            }
            Err(e) => Err(e), // Other errors propagate
        }
    }
}

// ============================================================================
// Network Error Detection
// ============================================================================

/// Detect if an error is due to network unavailability
///
/// Network errors indicate that the operation should be queued for later retry.
/// Non-network errors (validation, authentication, etc.) should propagate to caller.
///
/// # Detection Strategy
///
/// We detect network errors by looking for:
/// - "CoreContext not initialized" - Indicates network services unavailable
/// - "context()" errors - CoreContext access failures
/// - Future: Could add more sophisticated network detection
///
/// # Arguments
///
/// * `e` - The error to check
///
/// # Returns
///
/// `true` if this is a network-related error, `false` otherwise
fn is_network_error(e: &anyhow::Error) -> bool {
    let error_str = e.to_string();

    // CoreContext not initialized is the primary indicator of network unavailability
    error_str.contains("CoreContext not initialized")
        // Also catch errors from context() method
        || error_str.contains("CoreContext")
        // Future: Add more network error patterns
        || error_str.contains("network unavailable")
        || error_str.contains("connection failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_error_detection() {
        // Network errors should be detected
        let network_err = anyhow::anyhow!("CoreContext not initialized");
        assert!(is_network_error(&network_err));

        let network_err2 = anyhow::anyhow!("Failed to get CoreContext");
        assert!(is_network_error(&network_err2));

        // Validation errors should NOT be detected as network errors
        let validation_err = anyhow::anyhow!("Invalid entity name");
        assert!(!is_network_error(&validation_err));

        let auth_err = anyhow::anyhow!("Authentication failed");
        assert!(!is_network_error(&auth_err));
    }
}

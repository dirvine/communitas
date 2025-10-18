//! Integration tests for Backend offline queue management
//!
//! These tests verify that the Backend properly queues operations when offline
//! and synchronizes them when back online with proper persistence.
//!
//! Test Strategy:
//! - Test operation queueing (create entity, send message, add member)
//! - Test queue persistence across app restarts
//! - Test sync when coming back online
//! - Test conflict resolution
//! - Test queue size limits and priority
//! - Test error handling during sync
//!
//! Note: Tests currently blocked by CoreContext stack overflow issue.
//! Tests are written following TDD RED-GREEN-REFACTOR methodology.

use anyhow::Result;
use communitas_core::crdt::EntityType;
use communitas_tui::backend::Backend;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Operation that can be queued for offline execution
#[derive(Debug, Clone, PartialEq)]
pub enum QueuedOperation {
    /// Create entity operation
    CreateEntity {
        name: String,
        entity_type: EntityType,
        members: Vec<String>,
    },
    /// Send message operation
    SendMessage {
        entity_id: String,
        entity_type: EntityType,
        text: String,
    },
    /// Add member operation
    AddMember {
        entity_id: String,
        entity_type: EntityType,
        member_id: String,
    },
    /// Remove member operation
    RemoveMember {
        entity_id: String,
        entity_type: EntityType,
        member_id: String,
    },
}

/// Result of a sync operation
#[derive(Debug, Clone, PartialEq)]
pub enum SyncResult {
    /// Operation completed successfully
    Success { operation_id: String },
    /// Operation failed with error
    Failed { operation_id: String, error: String },
    /// Operation skipped (e.g., duplicate)
    Skipped { operation_id: String, reason: String },
}

/// Create a test backend with authenticated CoreContext
async fn create_test_backend() -> Result<(Backend, TempDir)> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let mut backend = Backend::new(data_dir.clone(), false).await?;

    // Create and login to a test vault
    backend
        .create_vault("ocean-forest-moon-star", "test-password-123", "Test User")
        .await?;

    // Initialize CoreContext
    backend.initialize_core_context().await?;

    Ok((backend, temp_dir))
}

// =============================================================================
// Queue Management Tests
// =============================================================================

#[tokio::test]
async fn test_queue_operation_when_offline() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Mark backend as offline
    backend.set_offline_mode(true).await?;

    // Try to create entity while offline - should be queued
    let operation_id = backend
        .queue_create_entity(
            "Offline Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify operation was queued
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 1);
    assert_eq!(queued_ops[0].id, operation_id);

    Ok(())
}

#[tokio::test]
async fn test_queue_multiple_operations() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity first (while online)
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Go offline
    backend.set_offline_mode(true).await?;

    // Queue multiple operations
    let op1 = backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Message 1".to_string(),
        )
        .await?;

    let op2 = backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Message 2".to_string(),
        )
        .await?;

    let op3 = backend
        .queue_add_member(
            entity.id.clone(),
            EntityType::Channel,
            "alice-test-one".to_string(),
        )
        .await?;

    // Verify all operations queued in order
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 3);
    assert_eq!(queued_ops[0].id, op1);
    assert_eq!(queued_ops[1].id, op2);
    assert_eq!(queued_ops[2].id, op3);

    Ok(())
}

#[tokio::test]
async fn test_queue_size_limit() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Set small queue limit
    backend.set_queue_size_limit(5).await?;

    // Create entity first
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Go offline
    backend.set_offline_mode(true).await?;

    // Queue more operations than limit
    for i in 0..10 {
        backend
            .queue_send_message(
                entity.id.clone(),
                EntityType::Channel,
                format!("Message {}", i),
            )
            .await?;
    }

    // Should only have 5 queued (oldest dropped)
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 5);

    // Verify it's the last 5 messages
    for (idx, op) in queued_ops.iter().enumerate() {
        if let QueuedOperation::SendMessage { text, .. } = &op.operation {
            assert_eq!(text, &format!("Message {}", idx + 5));
        }
    }

    Ok(())
}

// =============================================================================
// Queue Persistence Tests
// =============================================================================

#[tokio::test]
async fn test_queue_persists_across_restarts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Create entity and queue operations
    let (operation_ids, entity_id) = {
        let mut backend = Backend::new(data_dir.clone(), false).await?;
        backend
            .create_vault("test-one-two-three", "password123", "Test User")
            .await?;
        backend.initialize_core_context().await?;

        // Create entity while online
        let entity = backend
            .create_entity(
                "Persistent Channel".to_string(),
                EntityType::Channel,
                vec!["test-one-two-three".to_string()],
            )
            .await?;

        // Go offline and queue operations
        backend.set_offline_mode(true).await?;

        let op1 = backend
            .queue_send_message(
                entity.id.clone(),
                EntityType::Channel,
                "Offline Message 1".to_string(),
            )
            .await?;

        let op2 = backend
            .queue_send_message(
                entity.id.clone(),
                EntityType::Channel,
                "Offline Message 2".to_string(),
            )
            .await?;

        (vec![op1, op2], entity.id)
    }; // Backend dropped here

    // Create new backend instance with same data directory
    let mut backend = Backend::new(data_dir.clone(), false).await?;
    backend.login("test-one-two-three", "password123").await?;
    backend.initialize_core_context().await?;

    // Verify queued operations persisted
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 2);
    assert_eq!(queued_ops[0].id, operation_ids[0]);
    assert_eq!(queued_ops[1].id, operation_ids[1]);

    // Verify operation details
    if let QueuedOperation::SendMessage { text, entity_id: eid, .. } = &queued_ops[0].operation {
        assert_eq!(text, "Offline Message 1");
        assert_eq!(eid, &entity_id);
    } else {
        panic!("Expected SendMessage operation");
    }

    Ok(())
}

#[tokio::test]
async fn test_clear_queue_after_successful_sync() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let mut backend = Backend::new(data_dir.clone(), false).await?;
    backend
        .create_vault("test-sync", "password123", "Test User")
        .await?;
    backend.initialize_core_context().await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Sync Channel".to_string(),
            EntityType::Channel,
            vec!["test-sync".to_string()],
        )
        .await?;

    // Queue operations while offline
    backend.set_offline_mode(true).await?;
    backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Queued Message".to_string(),
        )
        .await?;

    // Verify queued
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 1);

    // Go back online and sync
    backend.set_offline_mode(false).await?;
    let results = backend.sync_queued_operations().await?;

    // Verify sync successful
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], SyncResult::Success { .. }));

    // Verify queue cleared
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 0);

    Ok(())
}

// =============================================================================
// Sync Operation Tests
// =============================================================================

#[tokio::test]
async fn test_sync_queued_operations_in_order() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Order Test".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Queue operations in specific order while offline
    backend.set_offline_mode(true).await?;

    backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "First".to_string(),
        )
        .await?;

    backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Second".to_string(),
        )
        .await?;

    backend
        .queue_add_member(
            entity.id.clone(),
            EntityType::Channel,
            "alice-test-one".to_string(),
        )
        .await?;

    // Sync operations
    backend.set_offline_mode(false).await?;
    let results = backend.sync_queued_operations().await?;

    // All should succeed
    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(matches!(result, SyncResult::Success { .. }));
    }

    // Verify messages in correct order
    let messages = backend.get_entity_messages(entity.id.clone()).await?;
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].content.text, "First");
    assert_eq!(messages[1].content.text, "Second");

    Ok(())
}

#[tokio::test]
async fn test_sync_with_partial_failures() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Queue operations while offline (some will fail)
    backend.set_offline_mode(true).await?;

    // Valid operation
    backend
        .queue_create_entity(
            "Valid Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Invalid operation (non-existent entity)
    backend
        .queue_send_message(
            "invalid-entity-id".to_string(),
            EntityType::Channel,
            "Invalid Message".to_string(),
        )
        .await?;

    // Another valid operation
    backend
        .queue_create_entity(
            "Another Valid".to_string(),
            EntityType::Group,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Sync operations
    backend.set_offline_mode(false).await?;
    let results = backend.sync_queued_operations().await?;

    // Check results
    assert_eq!(results.len(), 3);
    assert!(matches!(results[0], SyncResult::Success { .. }));
    assert!(matches!(results[1], SyncResult::Failed { .. }));
    assert!(matches!(results[2], SyncResult::Success { .. }));

    // Failed operation should remain in queue for retry
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_sync_progress_reporting() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Progress Test".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Queue multiple operations
    backend.set_offline_mode(true).await?;
    for i in 0..5 {
        backend
            .queue_send_message(
                entity.id.clone(),
                EntityType::Channel,
                format!("Message {}", i),
            )
            .await?;
    }

    // Subscribe to sync progress
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);
    backend.subscribe_sync_progress(tx).await?;

    // Start sync
    backend.set_offline_mode(false).await?;
    let sync_task = tokio::spawn(async move {
        backend.sync_queued_operations().await
    });

    // Monitor progress
    let mut progress_updates = Vec::new();
    while let Ok(Some(progress)) = timeout(Duration::from_millis(100), rx.recv()).await {
        progress_updates.push(progress);
        if progress_updates.len() >= 5 {
            break;
        }
    }

    // Wait for sync to complete
    let results = sync_task.await??;
    assert_eq!(results.len(), 5);

    // Verify we got progress updates
    assert!(!progress_updates.is_empty());

    Ok(())
}

// =============================================================================
// Priority Queue Tests
// =============================================================================

#[tokio::test]
async fn test_priority_queue_operations() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Priority Test".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Queue operations with different priorities while offline
    backend.set_offline_mode(true).await?;

    // Low priority
    backend
        .queue_send_message_with_priority(
            entity.id.clone(),
            EntityType::Channel,
            "Low Priority".to_string(),
            0,
        )
        .await?;

    // High priority (should be synced first)
    backend
        .queue_send_message_with_priority(
            entity.id.clone(),
            EntityType::Channel,
            "High Priority".to_string(),
            10,
        )
        .await?;

    // Medium priority
    backend
        .queue_send_message_with_priority(
            entity.id.clone(),
            EntityType::Channel,
            "Medium Priority".to_string(),
            5,
        )
        .await?;

    // Sync operations
    backend.set_offline_mode(false).await?;
    let results = backend.sync_queued_operations().await?;

    assert_eq!(results.len(), 3);

    // Verify messages synced in priority order (high to low)
    let messages = backend.get_entity_messages(entity.id).await?;
    assert_eq!(messages.len(), 3);
    assert_eq!(messages[0].content.text, "High Priority");
    assert_eq!(messages[1].content.text, "Medium Priority");
    assert_eq!(messages[2].content.text, "Low Priority");

    Ok(())
}

// =============================================================================
// Conflict Resolution Tests
// =============================================================================

#[tokio::test]
async fn test_conflict_resolution_duplicate_operation() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Queue duplicate operations while offline
    backend.set_offline_mode(true).await?;

    let op1 = backend
        .queue_create_entity(
            "Duplicate Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Queue same operation again (duplicate)
    let op2 = backend
        .queue_create_entity(
            "Duplicate Entity".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Sync operations
    backend.set_offline_mode(false).await?;
    let results = backend.sync_queued_operations().await?;

    // First should succeed, second should be skipped
    assert_eq!(results.len(), 2);
    assert!(matches!(results[0], SyncResult::Success { .. }));
    assert!(matches!(results[1], SyncResult::Skipped { .. }));

    Ok(())
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_queue_without_authentication_fails() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let mut backend = Backend::new(temp_dir.path().to_path_buf(), false).await?;

    // Try to queue operation without authentication - should fail
    let result = backend
        .queue_create_entity(
            "Test".to_string(),
            EntityType::Person,
            vec!["test-one-two-three".to_string()],
        )
        .await;

    assert!(result.is_err(), "Expected error when queueing without authentication");

    Ok(())
}

#[tokio::test]
async fn test_sync_retry_on_network_error() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Retry Test".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Queue operation while offline
    backend.set_offline_mode(true).await?;
    backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Retry Message".to_string(),
        )
        .await?;

    // Simulate network error during sync
    backend.set_offline_mode(false).await?;
    backend.simulate_network_error(true).await?;

    let results = backend.sync_queued_operations().await?;

    // Should fail due to network error
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], SyncResult::Failed { .. }));

    // Operation should remain in queue for retry
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 1);

    // Retry without network error
    backend.simulate_network_error(false).await?;
    let retry_results = backend.sync_queued_operations().await?;

    // Should succeed on retry
    assert_eq!(retry_results.len(), 1);
    assert!(matches!(retry_results[0], SyncResult::Success { .. }));

    // Queue should be empty
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 0);

    Ok(())
}

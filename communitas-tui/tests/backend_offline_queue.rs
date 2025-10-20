//! Integration tests for Backend automatic offline queue management
//!
//! These tests verify that the Backend properly handles network failures by:
//! - Automatically queueing operations when network is unavailable
//! - Executing operations immediately when network is available
//! - Auto-syncing queued operations when network returns
//! - Persisting queue across app restarts
//!
//! ## Testing Philosophy
//!
//! The application never exposes manual "offline mode" to users. Network state is
//! automatically detected and handled transparently. Tests simulate network failures
//! by clearing CoreContext, which triggers the automatic queueing behavior.

use anyhow::Result;
use communitas_core::crdt::EntityType;
use communitas_tui::backend::offline_queue::SyncResult;
use communitas_tui::backend::{Backend, EntityOrQueued, MemberOperationResult, MessageOrQueued};
use tempfile::TempDir;

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
// Automatic Offline Handling Tests
// =============================================================================

#[tokio::test]
async fn test_auto_execute_when_network_available() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // When network is available, operations execute immediately
    let result = backend
        .create_entity_auto(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify it was executed (not queued)
    match result {
        EntityOrQueued::Executed(entity) => {
            assert_eq!(entity.name, "Test Channel");
            assert_eq!(entity.entity_type, EntityType::Channel);
        }
        EntityOrQueued::Queued(_) => {
            panic!("Expected immediate execution, but operation was queued");
        }
    }

    // Queue should be empty
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_auto_queue_when_network_unavailable() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Simulate network failure (CoreContext becomes unavailable)
    backend.simulate_network_unavailable();

    // Try to create entity - should automatically queue
    let result = backend
        .create_entity_auto(
            "Offline Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Verify it was queued (not executed)
    let operation_id = match result {
        EntityOrQueued::Executed(_) => {
            panic!("Expected queued operation, but it was executed immediately");
        }
        EntityOrQueued::Queued(op_id) => op_id,
    };

    // Verify operation is in queue
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 1);
    assert_eq!(queued_ops[0].id, operation_id);

    Ok(())
}

#[tokio::test]
async fn test_auto_queue_multiple_operations() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity while online
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Simulate network failure
    backend.simulate_network_unavailable();

    // Try multiple operations - all should automatically queue
    let msg1_result = backend
        .send_message_auto(
            entity.id.clone(),
            EntityType::Channel,
            "Message 1".to_string(),
        )
        .await?;

    let msg2_result = backend
        .send_message_auto(
            entity.id.clone(),
            EntityType::Channel,
            "Message 2".to_string(),
        )
        .await?;

    let member_result = backend
        .add_member_auto(
            EntityType::Channel,
            entity.id.clone(),
            "alice-test-one".to_string(),
        )
        .await?;

    // All should be queued
    let op1 = match msg1_result {
        MessageOrQueued::Queued(id) => id,
        MessageOrQueued::Sent(_) => panic!("Expected queued, got sent"),
    };

    let op2 = match msg2_result {
        MessageOrQueued::Queued(id) => id,
        MessageOrQueued::Sent(_) => panic!("Expected queued, got sent"),
    };

    let op3 = match member_result {
        MemberOperationResult::Queued(id) => id,
        MemberOperationResult::Completed => panic!("Expected queued, got completed"),
    };

    // Verify all operations queued in order
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 3);
    assert_eq!(queued_ops[0].id, op1);
    assert_eq!(queued_ops[1].id, op2);
    assert_eq!(queued_ops[2].id, op3);

    Ok(())
}

#[tokio::test]
async fn test_auto_sync_when_network_returns() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity while online
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Simulate network failure and queue operations
    backend.simulate_network_unavailable();

    backend
        .send_message_auto(
            entity.id.clone(),
            EntityType::Channel,
            "Queued Message".to_string(),
        )
        .await?;

    // Verify operation is queued
    assert_eq!(backend.get_queued_operations().await?.len(), 1);

    // Network returns - reinitialize CoreContext
    backend.simulate_network_available();
    backend.initialize_core_context().await?;

    // Sync queued operations
    let results = backend.sync_queued_operations().await?;

    // Verify sync succeeded
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], SyncResult::Success { .. }));

    // Queue should be empty after successful sync
    assert_eq!(backend.get_queued_operations().await?.len(), 0);

    Ok(())
}

// =============================================================================
// Queue Management Tests
// =============================================================================

#[tokio::test]
async fn test_queue_size_limit() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Set small queue limit
    backend.set_queue_size_limit(5).await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Simulate network failure
    backend.simulate_network_unavailable();

    // Queue more operations than limit
    for i in 0..10 {
        backend
            .send_message_auto(
                entity.id.clone(),
                EntityType::Channel,
                format!("Message {}", i),
            )
            .await?;
    }

    // Should only have 5 queued (oldest dropped)
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 5);

    Ok(())
}

// =============================================================================
// Queue Persistence Tests
// =============================================================================

#[tokio::test]
async fn test_queue_persists_across_restarts() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // First session: queue an operation
    {
        let mut backend = Backend::new(data_dir.clone(), false).await?;
        backend
            .create_vault("ocean-forest-moon-star", "test-password-123", "Test User")
            .await?;
        backend.initialize_core_context().await?;

        // Create entity while online
        let entity = backend
            .create_entity(
                "Test Channel".to_string(),
                EntityType::Channel,
                vec!["ocean-forest-moon-star".to_string()],
            )
            .await?;

        // Simulate network failure and queue message
        backend.simulate_network_unavailable();
        backend
            .send_message_auto(
                entity.id.clone(),
                EntityType::Channel,
                "Persistent Message".to_string(),
            )
            .await?;

        // Verify queued
        assert_eq!(backend.get_queued_operations().await?.len(), 1);
    }
    // Backend dropped - simulates app restart

    // Second session: verify queue persisted
    {
        let mut backend = Backend::new(data_dir.clone(), false).await?;
        backend
            .login("ocean-forest-moon-star", "test-password-123")
            .await?;
        backend.initialize_core_context().await?;

        // Queue should still have the operation
        let queued_ops = backend.get_queued_operations().await?;
        assert_eq!(queued_ops.len(), 1);

        // Sync should succeed
        let results = backend.sync_queued_operations().await?;
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], SyncResult::Success { .. }));
    }

    Ok(())
}

// =============================================================================
// Priority and Ordering Tests
// =============================================================================

#[tokio::test]
async fn test_queue_priority_ordering() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Simulate network failure
    backend.simulate_network_unavailable();

    // Queue messages with different priorities
    let low_priority_op = backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Low priority".to_string(),
        )
        .await?;

    let high_priority_op = backend
        .queue_send_message_with_priority(
            entity.id.clone(),
            EntityType::Channel,
            "High priority".to_string(),
            10, // Higher priority
        )
        .await?;

    // Verify high priority is first in queue
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 2);
    assert_eq!(queued_ops[0].id, high_priority_op);
    assert_eq!(queued_ops[1].id, low_priority_op);

    Ok(())
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_validation_errors_not_queued() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Simulate network unavailable
    backend.simulate_network_unavailable();

    // Try to create entity with empty name (validation error)
    let result = backend
        .create_entity_auto(
            "".to_string(), // Invalid - empty name
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await;

    // Should get validation error (not queued)
    assert!(result.is_err());

    // Queue should be empty (validation errors don't queue)
    let queued_ops = backend.get_queued_operations().await?;
    assert_eq!(queued_ops.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_duplicate_detection_during_sync() -> Result<()> {
    let (mut backend, _temp) = create_test_backend().await?;

    // Create entity while online
    let entity = backend
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            vec!["ocean-forest-moon-star".to_string()],
        )
        .await?;

    // Simulate network failure
    backend.simulate_network_unavailable();

    // Queue duplicate operations
    backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Duplicate Message".to_string(),
        )
        .await?;

    backend
        .queue_send_message(
            entity.id.clone(),
            EntityType::Channel,
            "Duplicate Message".to_string(),
        )
        .await?;

    assert_eq!(backend.get_queued_operations().await?.len(), 2);

    // Network returns
    backend.simulate_network_available();
    backend.initialize_core_context().await?;

    // Sync should detect duplicates
    let results = backend.sync_queued_operations().await?;

    // Should have results for both operations
    assert_eq!(results.len(), 2);

    // At least one should be marked as skipped (duplicate)
    let has_skipped = results
        .iter()
        .any(|r| matches!(r, SyncResult::Skipped { .. }));
    assert!(has_skipped);

    Ok(())
}

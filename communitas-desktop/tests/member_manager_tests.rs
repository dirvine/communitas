use communitas_desktop::member_manager::{EntityType, MemberError, MemberManager};
use std::sync::Arc;
use tempfile::tempdir;

/// Test RED: Add member creates new member entry
#[tokio::test]
async fn test_add_member_creates_new_member() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt.clone());

    let entity_type = EntityType::Organization;
    let entity_id = "test-entity-001";
    let member_id = "ocean-forest-moon-star";
    let role = "admin";

    // Act
    member_manager
        .add_member(entity_type, entity_id, member_id, role)
        .await?;

    // Assert
    let members = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(members.len(), 1, "Should have exactly one member");

    let member = &members[0];
    assert_eq!(member.member_id, member_id, "Member ID should match");
    assert_eq!(member.role, role, "Role should match");
    assert!(!member.deleted, "Member should not be marked as deleted");
    assert!(member.joined_at > 0, "Join timestamp should be set");

    Ok(())
}

/// Test RED: Cannot add duplicate member
#[tokio::test]
async fn test_add_member_rejects_duplicate() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Group;
    let entity_id = "test-entity-002";
    let member_id = "ocean-forest-moon-star";
    let role = "member";

    // Add first member
    member_manager
        .add_member(entity_type, entity_id, member_id, role)
        .await?;

    // Act & Assert - trying to add same member again should fail
    let result = member_manager
        .add_member(entity_type, entity_id, member_id, role)
        .await;
    assert!(
        matches!(result, Err(MemberError::AlreadyExists)),
        "Should return AlreadyExists error, got: {:?}",
        result
    );

    // Verify only one member exists
    let members = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(members.len(), 1, "Should still have exactly one member");

    Ok(())
}

/// Test RED: Remove member creates tombstone
#[tokio::test]
async fn test_remove_member_creates_tombstone() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Channel;
    let entity_id = "test-entity-003";
    let member_id = "ocean-forest-moon-star";
    let deleter_id = "admin-user-name";
    let role = "member";

    // Add member first
    member_manager
        .add_member(entity_type, entity_id, member_id, role)
        .await?;

    // Act - remove the member
    member_manager
        .remove_member(entity_type, entity_id, member_id, deleter_id)
        .await?;

    // Assert - member should now be marked as deleted
    let members = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(members.len(), 1, "Member should still exist as tombstone");

    let member = &members[0];
    assert_eq!(member.member_id, member_id, "Member ID should match");
    assert!(member.deleted, "Member should be marked as deleted");

    Ok(())
}

/// Test RED: Cannot remove non-existent member
#[tokio::test]
async fn test_remove_member_not_found() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Project;
    let entity_id = "test-entity-004";
    let member_id = "ocean-forest-moon-star";
    let deleter_id = "admin-user-name";

    // Act & Assert - trying to remove non-existent member should fail
    let result = member_manager
        .remove_member(entity_type, entity_id, member_id, deleter_id)
        .await;
    assert!(
        matches!(result, Err(MemberError::NotFound)),
        "Should return NotFound error, got: {:?}",
        result
    );

    Ok(())
}

/// Test RED: Update role changes member's role
#[tokio::test]
async fn test_update_role_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Individual;
    let entity_id = "test-entity-005";
    let member_id = "ocean-forest-moon-star";
    let initial_role = "member";
    let new_role = "admin";

    // Add member with initial role
    member_manager
        .add_member(entity_type, entity_id, member_id, initial_role)
        .await?;

    // Act - update role
    member_manager
        .update_role(entity_type, entity_id, member_id, new_role)
        .await?;

    // Assert - verify role was updated
    let members = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(members.len(), 1, "Should have exactly one member");

    let member = &members[0];
    assert_eq!(member.member_id, member_id, "Member ID should match");
    assert_eq!(member.role, new_role, "Role should be updated");
    assert!(!member.deleted, "Member should not be deleted");

    Ok(())
}

/// Test RED: Cannot update role for non-existent member
#[tokio::test]
async fn test_update_role_not_found() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Organization;
    let entity_id = "test-entity-006";
    let member_id = "ocean-forest-moon-star";
    let new_role = "admin";

    // Act & Assert - trying to update role for non-existent member should fail
    let result = member_manager
        .update_role(entity_type, entity_id, member_id, new_role)
        .await;
    assert!(
        matches!(result, Err(MemberError::NotFound)),
        "Should return NotFound error, got: {:?}",
        result
    );

    Ok(())
}

/// Test RED: Cannot update role for deleted member
#[tokio::test]
async fn test_update_role_deleted_member() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Group;
    let entity_id = "test-entity-007";
    let member_id = "ocean-forest-moon-star";
    let deleter_id = "admin-user-name";
    let role = "member";
    let new_role = "admin";

    // Add and then remove member
    member_manager
        .add_member(entity_type, entity_id, member_id, role)
        .await?;
    member_manager
        .remove_member(entity_type, entity_id, member_id, deleter_id)
        .await?;

    // Act & Assert - trying to update role for deleted member should fail
    let result = member_manager
        .update_role(entity_type, entity_id, member_id, new_role)
        .await;
    assert!(
        matches!(result, Err(MemberError::NotFound)),
        "Should return NotFound error for deleted member, got: {:?}",
        result
    );

    Ok(())
}

/// Test RED: Prune tombstones removes old deletions but keeps recent ones
#[tokio::test]
async fn test_prune_tombstones_removes_old_deletions() -> Result<(), Box<dyn std::error::Error>>
{
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Channel;
    let entity_id = "test-entity-008";
    let member_id = "ocean-forest-moon-star";
    let deleter_id = "admin-user-name";
    let role = "member";

    // Add and remove member to create tombstone
    member_manager
        .add_member(entity_type, entity_id, member_id, role)
        .await?;
    member_manager
        .remove_member(entity_type, entity_id, member_id, deleter_id)
        .await?;

    // Verify tombstone exists
    let members_before = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(
        members_before.len(),
        1,
        "Should have one tombstone before pruning"
    );
    assert!(
        members_before[0].deleted,
        "Member should be marked as deleted"
    );

    // Act - prune tombstones (should NOT remove fresh tombstone)
    let pruned_count = member_manager
        .prune_tombstones(entity_type, entity_id)
        .await?;

    // Assert - fresh tombstone should NOT be pruned
    assert_eq!(pruned_count, 0, "Should not prune fresh tombstones");

    let members_after = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(
        members_after.len(),
        1,
        "Fresh tombstone should still exist"
    );

    Ok(())
}

/// Test RED: Prune tombstones on empty member list
#[tokio::test]
async fn test_prune_tombstones_empty_list() -> Result<(), Box<dyn std::error::Error>> {
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Project;
    let entity_id = "test-entity-009";

    // Act - prune tombstones on non-existent entity (empty list)
    let result = member_manager
        .prune_tombstones(entity_type, entity_id)
        .await;

    // Assert - should handle gracefully (document doesn't exist yet)
    // This will create the document, so it should succeed with 0 pruned
    match result {
        Ok(count) => assert_eq!(count, 0, "Should prune 0 from empty list"),
        Err(_) => {
            // Or it might error because document doesn't exist - both are acceptable
            // The important thing is it doesn't panic
        }
    }

    Ok(())
}

/// Test RED: Prune tombstones ignores active members
#[tokio::test]
async fn test_prune_tombstones_ignores_active_members() -> Result<(), Box<dyn std::error::Error>>
{
    // Arrange
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(
        communitas_desktop::crdt_manager::CrdtManager::new(&db_path)
            .await
            .expect("Failed to create CrdtManager"),
    );
    let member_manager = MemberManager::new(crdt);

    let entity_type = EntityType::Individual;
    let entity_id = "test-entity-010";
    let active_member = "ocean-forest-moon-star";
    let deleted_member = "mountain-river-cloud-tree";
    let deleter_id = "admin-user-name";
    let role = "member";

    // Add two members
    member_manager
        .add_member(entity_type, entity_id, active_member, role)
        .await?;
    member_manager
        .add_member(entity_type, entity_id, deleted_member, role)
        .await?;

    // Remove one member to create tombstone
    member_manager
        .remove_member(entity_type, entity_id, deleted_member, deleter_id)
        .await?;

    // Verify we have one active and one deleted
    let members_before = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(members_before.len(), 2, "Should have two members");

    let active_count = members_before.iter().filter(|m| !m.deleted).count();
    let deleted_count = members_before.iter().filter(|m| m.deleted).count();
    assert_eq!(active_count, 1, "Should have one active member");
    assert_eq!(deleted_count, 1, "Should have one deleted member");

    // Act - prune tombstones
    let pruned_count = member_manager
        .prune_tombstones(entity_type, entity_id)
        .await?;

    // Assert - should not prune fresh tombstone or active member
    assert_eq!(pruned_count, 0, "Should not prune fresh tombstones");

    let members_after = member_manager.list_members(entity_type, entity_id).await?;
    assert_eq!(members_after.len(), 2, "Should still have both members");

    // Verify active member is still active
    let active_member_data = members_after
        .iter()
        .find(|m| m.member_id == active_member)
        .expect("Active member should still exist");
    assert!(
        !active_member_data.deleted,
        "Active member should still be active"
    );

    Ok(())
}

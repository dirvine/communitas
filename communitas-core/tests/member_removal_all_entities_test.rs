// Copyright (c) 2025 Saorsa Labs Limited
//
// Integration tests for member removal across all entity types
//
// Verifies CRDT-based member management works for:
// - Groups
// - Organizations  
// - Channels
// - Projects
// - Individuals

use communitas_core::crdt::EntityType;
use communitas_core::entity_service::EntityService;
use communitas_core::crdt_manager::CrdtManager;
use std::path::PathBuf;
use tempfile::TempDir;

async fn setup_test_environment() -> (TempDir, EntityService) {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().to_path_buf();
    
    let crdt_manager = CrdtManager::new(storage_path);
    let entity_service = EntityService::new(crdt_manager);
    
    (temp_dir, entity_service)
}

#[tokio::test]
async fn test_remove_member_from_group() {
    let (_temp, service) = setup_test_environment().await;
    
    // Create group
    let group_id = "test-group-123";
    service.create_entity(EntityType::Group, group_id, "Test Group").await.unwrap();
    
    // Add member
    service.add_member(EntityType::Group, group_id, "member-four-words", "member").await.unwrap();
    
    // Verify member exists
    let members_before = service.list_members(EntityType::Group, group_id).await.unwrap();
    assert_eq!(members_before.len(), 1);
    assert!(!members_before[0].deleted);
    
    // Remove member
    service.remove_member(EntityType::Group, group_id, "member-four-words", "admin-user").await.unwrap();
    
    // Verify member marked as deleted (tombstone)
    let members_after = service.list_members(EntityType::Group, group_id).await.unwrap();
    assert_eq!(members_after.len(), 1);
    assert!(members_after[0].deleted, "Member should be marked as deleted (tombstone)");
}

#[tokio::test]
async fn test_remove_member_from_organization() {
    let (_temp, service) = setup_test_environment().await;
    
    let org_id = "test-org-456";
    service.create_entity(EntityType::Organisation, org_id, "Test Org").await.unwrap();
    service.add_member(EntityType::Organisation, org_id, "member-four-words", "member").await.unwrap();
    
    let members_before = service.list_members(EntityType::Organisation, org_id).await.unwrap();
    assert_eq!(members_before.len(), 1);
    
    service.remove_member(EntityType::Organisation, org_id, "member-four-words", "admin-user").await.unwrap();
    
    let members_after = service.list_members(EntityType::Organisation, org_id).await.unwrap();
    assert!(members_after[0].deleted);
}

#[tokio::test]
async fn test_remove_member_from_channel() {
    let (_temp, service) = setup_test_environment().await;
    
    let channel_id = "test-channel-789";
    service.create_entity(EntityType::Channel, channel_id, "Test Channel").await.unwrap();
    service.add_member(EntityType::Channel, channel_id, "member-four-words", "member").await.unwrap();
    
    let members_before = service.list_members(EntityType::Channel, channel_id).await.unwrap();
    assert_eq!(members_before.len(), 1);
    
    service.remove_member(EntityType::Channel, channel_id, "member-four-words", "admin-user").await.unwrap();
    
    let members_after = service.list_members(EntityType::Channel, channel_id).await.unwrap();
    assert!(members_after[0].deleted);
}

#[tokio::test]
async fn test_remove_member_from_project() {
    let (_temp, service) = setup_test_environment().await;
    
    let project_id = "test-project-101";
    service.create_entity(EntityType::Project, project_id, "Test Project").await.unwrap();
    service.add_member(EntityType::Project, project_id, "member-four-words", "member").await.unwrap();
    
    let members_before = service.list_members(EntityType::Project, project_id).await.unwrap();
    assert_eq!(members_before.len(), 1);
    
    service.remove_member(EntityType::Project, project_id, "member-four-words", "admin-user").await.unwrap();
    
    let members_after = service.list_members(EntityType::Project, project_id).await.unwrap();
    assert!(members_after[0].deleted);
}

#[tokio::test]
async fn test_remove_multiple_members_from_group() {
    let (_temp, service) = setup_test_environment().await;
    
    let group_id = "multi-member-group";
    service.create_entity(EntityType::Group, group_id, "Multi Member Group").await.unwrap();
    
    // Add multiple members
    service.add_member(EntityType::Group, group_id, "member-1", "member").await.unwrap();
    service.add_member(EntityType::Group, group_id, "member-2", "admin").await.unwrap();
    service.add_member(EntityType::Group, group_id, "member-3", "member").await.unwrap();
    
    let members_before = service.list_members(EntityType::Group, group_id).await.unwrap();
    assert_eq!(members_before.len(), 3);
    
    // Remove one member
    service.remove_member(EntityType::Group, group_id, "member-2", "owner").await.unwrap();
    
    let members_after = service.list_members(EntityType::Group, group_id).await.unwrap();
    assert_eq!(members_after.len(), 3); // Tombstone still in list
    
    let deleted_count = members_after.iter().filter(|m| m.deleted).count();
    assert_eq!(deleted_count, 1);
    
    let active_count = members_after.iter().filter(|m| !m.deleted).count();
    assert_eq!(active_count, 2);
}

#[tokio::test]
async fn test_cannot_remove_nonexistent_member() {
    let (_temp, service) = setup_test_environment().await;
    
    let group_id = "empty-group";
    service.create_entity(EntityType::Group, group_id, "Empty Group").await.unwrap();
    
    // Try to remove member that doesn't exist
    let result = service.remove_member(EntityType::Group, group_id, "nonexistent-member", "admin").await;
    
    assert!(result.is_err(), "Should fail when removing nonexistent member");
}

#[tokio::test]
async fn test_remove_preserves_tombstone_for_sync() {
    let (_temp, service) = setup_test_environment().await;
    
    let group_id = "tombstone-test";
    service.create_entity(EntityType::Group, group_id, "Tombstone Test").await.unwrap();
    service.add_member(EntityType::Group, group_id, "member-to-remove", "member").await.unwrap();
    
    // Remove member
    service.remove_member(EntityType::Group, group_id, "member-to-remove", "admin").await.unwrap();
    
    // Member should still exist as tombstone for CRDT sync
    let members = service.list_members(EntityType::Group, group_id).await.unwrap();
    assert_eq!(members.len(), 1, "Tombstone should remain for CRDT sync");
    assert_eq!(members[0].member_id, "member-to-remove");
    assert!(members[0].deleted, "Should be marked as deleted");
}

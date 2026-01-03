// Copyright (c) 2025 Saorsa Labs Limited
//
// CRITICAL SECURITY TEST: Cascading Member Removal from Organizations
//
// When a member is removed from an organization, they MUST be removed from:
// - All channels in the organization
// - All groups in the organization
// - All projects in the organization
//
// Without this, removed members retain access to org resources (SECURITY VULNERABILITY)

use communitas_core::crdt::EntityType;
use communitas_core::crdt_manager::CrdtManager;
use communitas_core::entity_service::EntityService;
use std::sync::Arc;
use tempfile::TempDir;

async fn setup_test_environment() -> (TempDir, EntityService) {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().to_path_buf();

    let crdt_manager = CrdtManager::new(storage_path).await.unwrap();
    let entity_service = EntityService::new(Arc::new(crdt_manager));

    (temp_dir, entity_service)
}

/// CRITICAL: When removing member from organization, they should be removed from all child channels
#[tokio::test]
async fn test_remove_from_org_cascades_to_channels() {
    let (_temp, service) = setup_test_environment().await;

    // Create organization
    let org = service
        .create_entity(
            "Test Organization".to_string(),
            EntityType::Organisation,
            None,
            "owner-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let org_id = &org.id;

    // Add member to organization
    service
        .add_member(
            EntityType::Organisation,
            org_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Create channel within organization
    let channel = service
        .create_entity(
            "Org Channel 1".to_string(),
            EntityType::Channel,
            None,
            "owner-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let channel_id = &channel.id;

    // Link channel to org via parent_org_id
    service
        .set_parent_organization(channel_id, org_id)
        .await
        .unwrap();

    // Add same member to channel
    service
        .add_member(
            EntityType::Channel,
            channel_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Verify member is in both org and channel
    let org_members_before = service
        .list_members(EntityType::Organisation, org_id)
        .await
        .unwrap();
    let channel_members_before = service
        .list_members(EntityType::Channel, channel_id)
        .await
        .unwrap();

    assert_eq!(org_members_before.len(), 2); // owner + member-to-remove
    assert_eq!(channel_members_before.len(), 2);

    // CRITICAL TEST: Remove member from organization
    // This MUST cascade to remove from channel
    service
        .remove_organization_member(org_id, "member-to-remove", "admin-user")
        .await
        .unwrap();

    // Verify member removed from organization (list_members filters deleted members)
    let org_members_after = service
        .list_members(EntityType::Organisation, org_id)
        .await
        .unwrap();
    assert!(
        !org_members_after
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "Member should be removed from organization"
    );

    // CRITICAL ASSERTION: Member should also be removed from channel
    let channel_members_after = service
        .list_members(EntityType::Channel, channel_id)
        .await
        .unwrap();
    assert!(
        !channel_members_after
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "SECURITY: Member MUST be removed from org channel when removed from org"
    );
}

/// Test cascade to groups
#[tokio::test]
async fn test_remove_from_org_cascades_to_groups() {
    let (_temp, service) = setup_test_environment().await;

    let org_id = "test-org-2";
    let _org = service
        .create_entity(
            "Test Org 2".to_string(),
            EntityType::Organisation,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();

    service
        .add_member(
            EntityType::Organisation,
            org_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Create group within organization
    let group = service
        .create_entity(
            "Org Group".to_string(),
            EntityType::Group,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let group_id = &group.id;

    service
        .set_parent_organization(group_id, org_id)
        .await
        .unwrap();
    service
        .add_member(EntityType::Group, group_id, "member-to-remove", "member")
        .await
        .unwrap();

    // Remove from org (should cascade)
    service
        .remove_organization_member(org_id, "member-to-remove", "admin")
        .await
        .unwrap();

    // Verify removed from group (list_members filters deleted members)
    let group_members = service
        .list_members(EntityType::Group, group_id)
        .await
        .unwrap();
    assert!(
        !group_members
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "SECURITY: Member MUST be removed from org group"
    );
}

/// Test cascade to projects
#[tokio::test]
async fn test_remove_from_org_cascades_to_projects() {
    let (_temp, service) = setup_test_environment().await;

    let org_id = "test-org-3";
    let _org = service
        .create_entity(
            "Test Org 3".to_string(),
            EntityType::Organisation,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();

    service
        .add_member(
            EntityType::Organisation,
            org_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Create project within organization
    let project = service
        .create_entity(
            "Org Project".to_string(),
            EntityType::Project,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let project_id = &project.id;

    service
        .set_parent_organization(project_id, org_id)
        .await
        .unwrap();
    service
        .add_member(
            EntityType::Project,
            project_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Remove from org (should cascade)
    service
        .remove_organization_member(org_id, "member-to-remove", "admin")
        .await
        .unwrap();

    // Verify removed from project (list_members filters deleted members)
    let project_members = service
        .list_members(EntityType::Project, project_id)
        .await
        .unwrap();
    assert!(
        !project_members
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "SECURITY: Member MUST be removed from org project"
    );
}

/// Test cascade to ALL child types simultaneously
#[tokio::test]
async fn test_remove_from_org_cascades_to_all_child_types() {
    let (_temp, service) = setup_test_environment().await;

    let org_id = "big-org";
    let _org = service
        .create_entity(
            "Big Org".to_string(),
            EntityType::Organisation,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();

    service
        .add_member(
            EntityType::Organisation,
            org_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Create multiple child entities of different types
    let channel = service
        .create_entity(
            "Channel".to_string(),
            EntityType::Channel,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let group = service
        .create_entity(
            "Group".to_string(),
            EntityType::Group,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let project = service
        .create_entity(
            "Project".to_string(),
            EntityType::Project,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();

    let channel_id = &channel.id;
    let group_id = &group.id;
    let project_id = &project.id;

    // Link all to organization
    service
        .set_parent_organization(channel_id, org_id)
        .await
        .unwrap();
    service
        .set_parent_organization(group_id, org_id)
        .await
        .unwrap();
    service
        .set_parent_organization(project_id, org_id)
        .await
        .unwrap();

    // Add member to all child entities
    service
        .add_member(
            EntityType::Channel,
            channel_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();
    service
        .add_member(EntityType::Group, group_id, "member-to-remove", "member")
        .await
        .unwrap();
    service
        .add_member(
            EntityType::Project,
            project_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // CRITICAL: Remove from organization
    let result = service
        .remove_organization_member(org_id, "member-to-remove", "admin")
        .await
        .unwrap();

    // Verify cascade happened
    assert!(
        result.removed_in.len() >= 4,
        "Should remove from org + 3 children = 4 entities"
    );

    // Verify each child has member removed (list_members filters deleted members)
    let channel_members = service
        .list_members(EntityType::Channel, channel_id)
        .await
        .unwrap();
    assert!(
        !channel_members
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "Member should be removed from channel"
    );

    let group_members = service
        .list_members(EntityType::Group, group_id)
        .await
        .unwrap();
    assert!(
        !group_members
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "Member should be removed from group"
    );

    let project_members = service
        .list_members(EntityType::Project, project_id)
        .await
        .unwrap();
    assert!(
        !project_members
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "Member should be removed from project"
    );
}

/// Test idempotency: Member not in some children (should not fail)
#[tokio::test]
async fn test_cascade_handles_member_not_in_all_children() {
    let (_temp, service) = setup_test_environment().await;

    let org_id = "sparse-org";
    let _org = service
        .create_entity(
            "Sparse Org".to_string(),
            EntityType::Organisation,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();

    service
        .add_member(
            EntityType::Organisation,
            org_id,
            "member-to-remove",
            "member",
        )
        .await
        .unwrap();

    // Create channel but DON'T add member to it
    let channel = service
        .create_entity(
            "Channel".to_string(),
            EntityType::Channel,
            None,
            "owner".to_string(),
            vec![],
        )
        .await
        .unwrap();
    service
        .set_parent_organization(&channel.id, org_id)
        .await
        .unwrap();

    // Should succeed even though member not in channel
    let result = service
        .remove_organization_member(org_id, "member-to-remove", "admin")
        .await
        .unwrap();

    // Should have skipped channel (member not found)
    assert!(
        result
            .skipped_not_member
            .iter()
            .any(|(t, _)| *t == EntityType::Channel),
        "Should skip child entities where member doesn't exist"
    );
}

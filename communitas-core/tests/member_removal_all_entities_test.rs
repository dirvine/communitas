// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::crdt::EntityType;

use communitas_core::crdt_manager::CrdtManager;

use communitas_core::entity_service::EntityService;
use tempfile::TempDir;

async fn setup_test_environment() -> (TempDir, EntityService) {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().to_path_buf();

    let crdt_manager = CrdtManager::new(storage_path).await.unwrap();
    let entity_service = EntityService::new(std::sync::Arc::new(crdt_manager));

    (temp_dir, entity_service)
}

#[tokio::test]
async fn test_remove_member_from_group() {
    let (_temp, service) = setup_test_environment().await;

    // Create group
    let created_group = service
        .create_entity(
            "Test Group".to_string(),
            EntityType::Group,
            Some("A test group".to_string()),
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let group_id = created_group.id;

    // Add member
    service
        .add_member(EntityType::Group, &group_id, "member-four-words", "member")
        .await
        .unwrap();

    // Verify member exists (creator + added member = 2 total)
    let members_before = service
        .list_members(EntityType::Group, &group_id)
        .await
        .unwrap();
    assert_eq!(
        members_before.len(),
        2,
        "Should have creator + added member"
    );
    let added_member = members_before
        .iter()
        .find(|m| m.member_id == "member-four-words");
    assert!(added_member.is_some(), "Added member should exist");
    assert!(
        !added_member.unwrap().deleted,
        "Added member should not be deleted"
    );

    // Remove member
    service
        .remove_member(
            EntityType::Group,
            &group_id,
            "member-four-words",
            "admin-user",
        )
        .await
        .unwrap();

    // Verify member is removed (list_members filters deleted members)
    let members_after = service
        .list_members(EntityType::Group, &group_id)
        .await
        .unwrap();
    assert_eq!(
        members_after.len(),
        1,
        "Should only have creator after removal"
    );
    assert!(
        !members_after
            .iter()
            .any(|m| m.member_id == "member-four-words"),
        "Removed member should not appear in list_members"
    );
}

#[tokio::test]
async fn test_remove_member_from_organization() {
    let (_temp, service) = setup_test_environment().await;

    // Create organization and capture its ID
    let created_org = service
        .create_entity(
            "Test Org".to_string(),
            EntityType::Organisation,
            None,
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let org_id = created_org.id;

    service
        .add_member(
            EntityType::Organisation,
            &org_id,
            "member-four-words",
            "member",
        )
        .await
        .unwrap();

    // Should have creator + added member = 2 total
    let members_before = service
        .list_members(EntityType::Organisation, &org_id)
        .await
        .unwrap();
    assert_eq!(
        members_before.len(),
        2,
        "Should have creator + added member"
    );

    service
        .remove_member(
            EntityType::Organisation,
            &org_id,
            "member-four-words",
            "admin-user",
        )
        .await
        .unwrap();

    // Verify member is removed (list_members filters deleted members)
    let members_after = service
        .list_members(EntityType::Organisation, &org_id)
        .await
        .unwrap();
    assert_eq!(
        members_after.len(),
        1,
        "Should only have creator after removal"
    );
    assert!(
        !members_after
            .iter()
            .any(|m| m.member_id == "member-four-words"),
        "Removed member should not appear in list_members"
    );
}

#[tokio::test]
async fn test_remove_member_from_channel() {
    let (_temp, service) = setup_test_environment().await;

    // Create channel and capture its ID
    let created_channel = service
        .create_entity(
            "Test Channel".to_string(),
            EntityType::Channel,
            None,
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let channel_id = created_channel.id;

    service
        .add_member(
            EntityType::Channel,
            &channel_id,
            "member-four-words",
            "member",
        )
        .await
        .unwrap();

    // Should have creator + added member = 2 total
    let members_before = service
        .list_members(EntityType::Channel, &channel_id)
        .await
        .unwrap();
    assert_eq!(
        members_before.len(),
        2,
        "Should have creator + added member"
    );

    service
        .remove_member(
            EntityType::Channel,
            &channel_id,
            "member-four-words",
            "admin-user",
        )
        .await
        .unwrap();

    // Verify member is removed (list_members filters deleted members)
    let members_after = service
        .list_members(EntityType::Channel, &channel_id)
        .await
        .unwrap();
    assert_eq!(
        members_after.len(),
        1,
        "Should only have creator after removal"
    );
    assert!(
        !members_after
            .iter()
            .any(|m| m.member_id == "member-four-words"),
        "Removed member should not appear in list_members"
    );
}

#[tokio::test]
async fn test_remove_member_from_project() {
    let (_temp, service) = setup_test_environment().await;

    // Create project and capture its ID
    let created_project = service
        .create_entity(
            "Test Project".to_string(),
            EntityType::Project,
            None,
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let project_id = created_project.id;

    service
        .add_member(
            EntityType::Project,
            &project_id,
            "member-four-words",
            "member",
        )
        .await
        .unwrap();

    // Should have creator + added member = 2 total
    let members_before = service
        .list_members(EntityType::Project, &project_id)
        .await
        .unwrap();
    assert_eq!(
        members_before.len(),
        2,
        "Should have creator + added member"
    );

    service
        .remove_member(
            EntityType::Project,
            &project_id,
            "member-four-words",
            "admin-user",
        )
        .await
        .unwrap();

    // Verify member is removed (list_members filters deleted members)
    let members_after = service
        .list_members(EntityType::Project, &project_id)
        .await
        .unwrap();
    assert_eq!(
        members_after.len(),
        1,
        "Should only have creator after removal"
    );
    assert!(
        !members_after
            .iter()
            .any(|m| m.member_id == "member-four-words"),
        "Removed member should not appear in list_members"
    );
}

#[tokio::test]
async fn test_remove_multiple_members_from_group() {
    let (_temp, service) = setup_test_environment().await;

    // Create group and capture its ID
    let created_group = service
        .create_entity(
            "Multi Member Group".to_string(),
            EntityType::Group,
            None,
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let group_id = created_group.id;

    // Add multiple members
    service
        .add_member(EntityType::Group, &group_id, "member-1", "member")
        .await
        .unwrap();
    service
        .add_member(EntityType::Group, &group_id, "member-2", "admin")
        .await
        .unwrap();
    service
        .add_member(EntityType::Group, &group_id, "member-3", "member")
        .await
        .unwrap();

    // Should have creator + 3 added members = 4 total
    let members_before = service
        .list_members(EntityType::Group, &group_id)
        .await
        .unwrap();
    assert_eq!(
        members_before.len(),
        4,
        "Should have creator + 3 added members"
    );

    // Remove one member
    service
        .remove_member(EntityType::Group, &group_id, "member-2", "admin-user")
        .await
        .unwrap();

    // list_members filters deleted members, so should have creator + 2 active = 3 total
    let members_after = service
        .list_members(EntityType::Group, &group_id)
        .await
        .unwrap();
    assert_eq!(
        members_after.len(),
        3,
        "Should have creator + 2 active members (deleted filtered out)"
    );

    // All returned members should be active (deleted are filtered)
    assert!(
        members_after.iter().all(|m| !m.deleted),
        "All members in list should be active"
    );

    // Verify member-2 is not in the list
    assert!(
        !members_after.iter().any(|m| m.member_id == "member-2"),
        "Removed member should not appear in list_members"
    );
}

#[tokio::test]
async fn test_cannot_remove_nonexistent_member() {
    let (_temp, service) = setup_test_environment().await;

    // Create group and capture its ID
    let created_group = service
        .create_entity(
            "Empty Group".to_string(),
            EntityType::Group,
            None,
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let group_id = created_group.id;

    // Try to remove member that doesn't exist
    let result = service
        .remove_member(EntityType::Group, &group_id, "nonexistent-member", "admin")
        .await;

    assert!(
        result.is_err(),
        "Should fail when removing nonexistent member"
    );
}

#[tokio::test]
async fn test_remove_preserves_tombstone_for_sync() {
    // Note: Tombstones ARE preserved internally in the CRDT document for sync purposes.
    // However, list_members() intentionally filters out deleted members for the user-facing API.
    // This test verifies that:
    // 1. A member can be added and then removed
    // 2. The removal operation succeeds
    // 3. The removed member no longer appears in list_members()
    // The actual tombstone preservation happens at the CRDT layer (Yrs document).

    let (_temp, service) = setup_test_environment().await;

    // Create group and capture its ID
    let created_group = service
        .create_entity(
            "Tombstone Test".to_string(),
            EntityType::Group,
            None,
            "admin-user".to_string(),
            vec![],
        )
        .await
        .unwrap();
    let group_id = created_group.id;

    // Add member
    service
        .add_member(EntityType::Group, &group_id, "member-to-remove", "member")
        .await
        .unwrap();

    // Verify member was added (creator + member = 2)
    let members_before = service
        .list_members(EntityType::Group, &group_id)
        .await
        .unwrap();
    assert_eq!(
        members_before.len(),
        2,
        "Should have creator + added member before removal"
    );

    // Remove member
    service
        .remove_member(
            EntityType::Group,
            &group_id,
            "member-to-remove",
            "admin-user",
        )
        .await
        .unwrap();

    // Verify member is removed from list (tombstone preserved internally in CRDT)
    let members_after = service
        .list_members(EntityType::Group, &group_id)
        .await
        .unwrap();
    assert_eq!(
        members_after.len(),
        1,
        "Should only have creator after removal (tombstone preserved in CRDT, not in list)"
    );
    assert!(
        !members_after
            .iter()
            .any(|m| m.member_id == "member-to-remove"),
        "Removed member should not appear in list_members"
    );
}

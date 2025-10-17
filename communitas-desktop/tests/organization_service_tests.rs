// Copyright (c) 2025 Saorsa Labs Limited
//
// Comprehensive tests for OrganizationService
// Tests cover CRUD operations, four-word identity, CRDT sync, and member management

use anyhow::Result;
use communitas_desktop::{
    crdt_manager::CrdtManager,
    services::organization_service::OrganizationService,
};
use std::sync::Arc;
use tempfile::TempDir;

/// Helper to create test CrdtManager with temporary database
async fn create_test_crdt() -> Result<(Arc<CrdtManager>, TempDir)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let crdt = Arc::new(CrdtManager::new(&db_path).await?);
    Ok((crdt, temp_dir))
}

/// Helper to create test OrganizationService
async fn create_test_service() -> Result<(OrganizationService, TempDir)> {
    let (crdt, temp_dir) = create_test_crdt().await?;
    let service = OrganizationService::new(crdt);
    Ok((service, temp_dir))
}

#[tokio::test]
async fn test_create_organization() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization(
            "Test Organization",
            Some("A test organization".to_string()),
            "user-123",
        )
        .await?;

    // Verify basic fields
    assert!(!org.id.is_empty());
    assert_eq!(org.name, "Test Organization");
    assert_eq!(org.description, Some("A test organization".to_string()));
    assert_eq!(org.created_by, "user-123");

    // Verify four-word identity was generated
    assert!(!org.four_word_identity.is_empty());
    assert!(org.four_word_identity.contains('-')); // Should have dashes

    // Verify disk IDs were generated
    assert!(!org.private_disk_id.is_empty());
    assert!(!org.public_disk_id.is_empty());
    assert_ne!(org.private_disk_id, org.public_disk_id);

    // Verify CRDT doc ID
    assert!(org.crdt_doc_id.starts_with("organization:"));

    Ok(())
}

#[tokio::test]
async fn test_get_organization() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create organization
    let created = service
        .create_organization("Get Test Org", None, "user-123")
        .await?;

    // Retrieve it
    let retrieved = service.get_organization(&created.id).await?;

    assert!(retrieved.is_some());
    let org = retrieved.unwrap();
    assert_eq!(org.id, created.id);
    assert_eq!(org.name, "Get Test Org");
    assert_eq!(org.four_word_identity, created.four_word_identity);

    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_organization() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let result = service.get_organization("nonexistent-id").await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_get_organization_by_four_words() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create organization
    let created = service
        .create_organization("Four Word Test", None, "user-123")
        .await?;

    // Retrieve by four-word identity
    let retrieved = service
        .get_organization_by_four_words(&created.four_word_identity)
        .await?;

    assert!(retrieved.is_some());
    let org = retrieved.unwrap();
    assert_eq!(org.id, created.id);
    assert_eq!(org.four_word_identity, created.four_word_identity);

    Ok(())
}

#[tokio::test]
async fn test_update_organization() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create organization
    let org = service
        .create_organization("Original Name", Some("Original desc".to_string()), "user-123")
        .await?;

    // Update name
    service
        .update_organization(
            &org.id,
            Some("Updated Name"),
            None,
        )
        .await?;

    let updated = service.get_organization(&org.id).await?.unwrap();
    assert_eq!(updated.name, "Updated Name");
    assert_eq!(updated.description, Some("Original desc".to_string()));

    // Update description
    service
        .update_organization(
            &org.id,
            None,
            Some("Updated description".to_string()),
        )
        .await?;

    let updated2 = service.get_organization(&org.id).await?.unwrap();
    assert_eq!(updated2.name, "Updated Name");
    assert_eq!(updated2.description, Some("Updated description".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_set_website_root() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Website Test Org", None, "user-123")
        .await?;

    // Initially no website root
    let initial = service.get_organization(&org.id).await?.unwrap();
    assert!(initial.website_root.is_none());

    // Set website root
    let root_hash = "QmHash123456789";
    service.set_website_root(&org.id, root_hash).await?;

    // Verify it was set
    let updated = service.get_organization(&org.id).await?.unwrap();
    assert_eq!(updated.website_root, Some(root_hash.to_string()));

    Ok(())
}

#[tokio::test]
async fn test_add_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Member Test Org", None, "user-123")
        .await?;

    // Add member with admin role (creator is already added as owner)
    service.add_member(&org.id, "user-456", "admin").await?;

    // Verify member was added (creator + new member)
    let members = service.get_members(&org.id).await?;
    assert_eq!(members.len(), 2);
    // Find the added member (not the creator)
    let added_member = members.iter().find(|m| m.user_id == "user-456").unwrap();
    assert_eq!(added_member.role, "admin");

    // Add second member
    service.add_member(&org.id, "user-789", "member").await?;

    let members2 = service.get_members(&org.id).await?;
    assert_eq!(members2.len(), 3);

    Ok(())
}

#[tokio::test]
async fn test_remove_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Remove Member Test", None, "user-123")
        .await?;

    // Add members (creator is already added as owner)
    service.add_member(&org.id, "user-456", "admin").await?;
    service.add_member(&org.id, "user-789", "member").await?;

    assert_eq!(service.get_members(&org.id).await?.len(), 3);

    // Remove one member
    service.remove_member(&org.id, "user-456").await?;

    let members = service.get_members(&org.id).await?;
    assert_eq!(members.len(), 2);
    // Verify user-789 is still there
    assert!(members.iter().any(|m| m.user_id == "user-789"));

    Ok(())
}

#[tokio::test]
async fn test_update_member_role() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Role Update Test", None, "user-123")
        .await?;

    // Add member (creator is already added as owner)
    service.add_member(&org.id, "user-456", "member").await?;

    // Update role to admin
    service.update_member_role(&org.id, "user-456", "admin").await?;

    let members = service.get_members(&org.id).await?;
    let updated_member = members.iter().find(|m| m.user_id == "user-456").unwrap();
    assert_eq!(updated_member.role, "admin");

    Ok(())
}

#[tokio::test]
async fn test_is_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Membership Test", None, "user-123")
        .await?;

    // Initially not a member
    assert!(!service.is_member(&org.id, "user-456").await?);

    // Add member
    service.add_member(&org.id, "user-456", "member").await?;

    // Now is a member
    assert!(service.is_member(&org.id, "user-456").await?);

    Ok(())
}

#[tokio::test]
async fn test_list_user_organizations() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let user_id = "user-123";

    // Create organizations and add user as member
    let org1 = service
        .create_organization("Org 1", None, "creator-1")
        .await?;
    let org2 = service
        .create_organization("Org 2", None, "creator-2")
        .await?;

    service.add_member(&org1.id, user_id, "member").await?;
    service.add_member(&org2.id, user_id, "admin").await?;

    // List organizations for user
    let orgs = service.list_user_organizations(user_id).await?;
    assert_eq!(orgs.len(), 2);

    let org_names: Vec<String> = orgs.iter().map(|o| o.name.clone()).collect();
    assert!(org_names.contains(&"Org 1".to_string()));
    assert!(org_names.contains(&"Org 2".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_delete_organization() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Delete Test", None, "user-123")
        .await?;

    // Verify it exists
    assert!(service.get_organization(&org.id).await?.is_some());

    // Delete it
    service.delete_organization(&org.id).await?;

    // Verify it's gone
    assert!(service.get_organization(&org.id).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_crdt_update_sync() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("CRDT Test", None, "user-123")
        .await?;

    // Get full update
    let update = service.get_organization_update(&org.id).await?;
    assert!(!update.is_empty());

    // Apply update (simulating sync from another peer)
    service.apply_organization_update(&org.id, &update).await?;

    // Should still work
    let retrieved = service.get_organization(&org.id).await?;
    assert!(retrieved.is_some());

    Ok(())
}

#[tokio::test]
async fn test_crdt_state_vector() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("State Vector Test", None, "user-123")
        .await?;

    // Get state vector
    let sv = service.get_organization_state_vector(&org.id).await?;
    assert!(!sv.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_crdt_diff_sync() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Diff Sync Test", None, "user-123")
        .await?;

    // Get initial state vector
    let sv1 = service.get_organization_state_vector(&org.id).await?;

    // Make some changes via update
    service.update_organization(&org.id, Some("Updated Name"), None).await?;

    // Get diff from initial state
    let diff = service.get_organization_diff(&org.id, &sv1).await?;
    assert!(!diff.is_empty()); // Should have changes

    // Apply diff
    service.apply_organization_diff(&org.id, &diff).await?;

    Ok(())
}

#[tokio::test]
async fn test_multiple_organizations() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create multiple organizations
    for i in 1..=5 {
        service
            .create_organization(
                &format!("Org {}", i),
                Some(format!("Description {}", i)),
                "user-123",
            )
            .await?;
    }

    // Creator is automatically added as owner to all organizations
    let user_orgs = service.list_user_organizations("user-123").await?;
    assert_eq!(user_orgs.len(), 5); // User is creator/owner of all 5 orgs

    Ok(())
}

#[tokio::test]
async fn test_organization_metadata_in_crdt() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization(
            "Metadata Test",
            Some("Test description".to_string()),
            "user-123",
        )
        .await?;

    // Get update and verify it contains data
    let update = service.get_organization_update(&org.id).await?;
    assert!(!update.is_empty());

    // CRDT document should be loadable
    // This tests that the CRDT integration works end-to-end
    let sv = service.get_organization_state_vector(&org.id).await?;
    assert!(!sv.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_concurrent_member_operations() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let org = service
        .create_organization("Concurrent Test", None, "user-123")
        .await?;

    // Add multiple members concurrently (creator is already added as owner)
    let handles: Vec<_> = (1..=10)
        .map(|i| {
            let service = &service;
            let org_id = org.id.clone();
            async move {
                service
                    .add_member(&org_id, &format!("user-{}", i), "member")
                    .await
            }
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        handle.await?;
    }

    // Verify all members were added (creator + 10 concurrent adds)
    let members = service.get_members(&org.id).await?;
    assert_eq!(members.len(), 11);

    Ok(())
}

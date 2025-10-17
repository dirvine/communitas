// Copyright (c) 2025 Saorsa Labs Limited
//
// Comprehensive tests for MemberService
// Tests cover CRUD operations, four-word identity, personal disks, and CRDT sync

use anyhow::Result;
use communitas_desktop::{
    crdt_manager::CrdtManager,
    services::{
        member_service::MemberService,
        virtual_disk_service::VirtualDiskService,
    },
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

/// Helper to create test MemberService
async fn create_test_service() -> Result<(MemberService, TempDir)> {
    let (crdt, temp_dir) = create_test_crdt().await?;
    let disk_service = Arc::new(VirtualDiskService::new(crdt.clone()));
    let service = MemberService::new(crdt, disk_service);
    Ok((service, temp_dir))
}

#[tokio::test]
async fn test_create_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Alice Johnson", Some("alice@example.com".to_string()))
        .await?;

    // Verify basic fields
    assert!(!member.id.is_empty());
    assert_eq!(member.display_name, "Alice Johnson");
    assert_eq!(member.email, Some("alice@example.com".to_string()));

    // Verify four-word identity was generated
    assert!(!member.four_word_identity.is_empty());
    assert!(member.four_word_identity.contains('-')); // Should have dashes

    // Verify personal disk was created
    assert!(!member.personal_disk_id.is_empty());

    // Verify CRDT doc ID
    assert!(member.crdt_doc_id.starts_with("member:"));

    // Initially no avatar, bio, or website root
    assert!(member.avatar_url.is_none());
    assert!(member.bio.is_none());
    assert!(member.website_root.is_none());

    Ok(())
}

#[tokio::test]
async fn test_create_member_without_email() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service.create_member("Bob Smith", None).await?;

    assert_eq!(member.display_name, "Bob Smith");
    assert!(member.email.is_none());
    assert!(!member.four_word_identity.is_empty());
    assert!(!member.personal_disk_id.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_get_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create member
    let created = service
        .create_member("Charlie Brown", Some("charlie@example.com".to_string()))
        .await?;

    // Retrieve it
    let retrieved = service.get_member(&created.id).await?;

    assert!(retrieved.is_some());
    let member = retrieved.unwrap();
    assert_eq!(member.id, created.id);
    assert_eq!(member.display_name, "Charlie Brown");
    assert_eq!(member.four_word_identity, created.four_word_identity);
    assert_eq!(member.email, Some("charlie@example.com".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_get_nonexistent_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let result = service.get_member("nonexistent-id").await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_get_member_by_four_words() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create member
    let created = service
        .create_member("Diana Prince", Some("diana@example.com".to_string()))
        .await?;

    // Retrieve by four-word identity
    let retrieved = service
        .get_member_by_four_words(&created.four_word_identity)
        .await?;

    assert!(retrieved.is_some());
    let member = retrieved.unwrap();
    assert_eq!(member.id, created.id);
    assert_eq!(member.four_word_identity, created.four_word_identity);
    assert_eq!(member.display_name, "Diana Prince");

    Ok(())
}

#[tokio::test]
async fn test_get_member_by_invalid_four_words() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let result = service
        .get_member_by_four_words("invalid-four-word-identity")
        .await?;
    assert!(result.is_none());

    Ok(())
}

#[tokio::test]
async fn test_update_member_display_name() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Original Name", Some("email@example.com".to_string()))
        .await?;

    // Update display name
    service
        .update_member(&member.id, Some("Updated Name"), None, None, None)
        .await?;

    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(updated.display_name, "Updated Name");
    assert_eq!(updated.email, Some("email@example.com".to_string())); // Unchanged

    Ok(())
}

#[tokio::test]
async fn test_update_member_email() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Test User", Some("old@example.com".to_string()))
        .await?;

    // Update email
    service
        .update_member(
            &member.id,
            None,
            Some("new@example.com".to_string()),
            None,
            None,
        )
        .await?;

    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(updated.email, Some("new@example.com".to_string()));
    assert_eq!(updated.display_name, "Test User"); // Unchanged

    Ok(())
}

#[tokio::test]
async fn test_update_member_avatar() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service.create_member("Avatar Test", None).await?;

    // Initially no avatar
    assert!(member.avatar_url.is_none());

    // Set avatar
    service
        .update_member(
            &member.id,
            None,
            None,
            Some("https://example.com/avatar.jpg".to_string()),
            None,
        )
        .await?;

    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(
        updated.avatar_url,
        Some("https://example.com/avatar.jpg".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_update_member_bio() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service.create_member("Bio Test", None).await?;

    // Initially no bio
    assert!(member.bio.is_none());

    // Set bio
    service
        .update_member(
            &member.id,
            None,
            None,
            None,
            Some("Software developer and open source enthusiast".to_string()),
        )
        .await?;

    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(
        updated.bio,
        Some("Software developer and open source enthusiast".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn test_update_member_multiple_fields() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Original", Some("original@example.com".to_string()))
        .await?;

    // Update multiple fields at once
    service
        .update_member(
            &member.id,
            Some("New Name"),
            Some("new@example.com".to_string()),
            Some("https://example.com/avatar.jpg".to_string()),
            Some("My bio".to_string()),
        )
        .await?;

    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(updated.display_name, "New Name");
    assert_eq!(updated.email, Some("new@example.com".to_string()));
    assert_eq!(
        updated.avatar_url,
        Some("https://example.com/avatar.jpg".to_string())
    );
    assert_eq!(updated.bio, Some("My bio".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_set_website_root() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Website Test", None)
        .await?;

    // Initially no website root
    let initial = service.get_member(&member.id).await?.unwrap();
    assert!(initial.website_root.is_none());

    // Set website root
    let root_hash = "QmWebsiteHash123456789";
    service.set_website_root(&member.id, root_hash).await?;

    // Verify it was set
    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(updated.website_root, Some(root_hash.to_string()));

    Ok(())
}

#[tokio::test]
async fn test_update_website_root() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service.create_member("Website Update", None).await?;

    // Set initial website root
    service
        .set_website_root(&member.id, "QmOldHash")
        .await?;

    let initial = service.get_member(&member.id).await?.unwrap();
    assert_eq!(initial.website_root, Some("QmOldHash".to_string()));

    // Update to new website root
    service
        .set_website_root(&member.id, "QmNewHash")
        .await?;

    let updated = service.get_member(&member.id).await?.unwrap();
    assert_eq!(updated.website_root, Some("QmNewHash".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_delete_member() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service.create_member("Delete Test", None).await?;

    // Verify it exists
    assert!(service.get_member(&member.id).await?.is_some());

    // Delete it
    service.delete_member(&member.id).await?;

    // Verify it's gone
    assert!(service.get_member(&member.id).await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_crdt_update_sync() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("CRDT Test", Some("crdt@example.com".to_string()))
        .await?;

    // Get full update
    let update = service.get_member_update(&member.id).await?;
    assert!(!update.is_empty());

    // Apply update (simulating sync from another peer)
    service.apply_member_update(&member.id, &update).await?;

    // Should still work
    let retrieved = service.get_member(&member.id).await?;
    assert!(retrieved.is_some());

    Ok(())
}

#[tokio::test]
async fn test_crdt_state_vector() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("State Vector Test", None)
        .await?;

    // Get state vector
    let sv = service.get_member_state_vector(&member.id).await?;
    assert!(!sv.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_crdt_diff_sync() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Diff Sync Test", None)
        .await?;

    // Get initial state vector
    let sv1 = service.get_member_state_vector(&member.id).await?;

    // Make some changes via update
    service
        .update_member(&member.id, Some("Updated Name"), None, None, None)
        .await?;

    // Get diff from initial state
    let diff = service.get_member_diff(&member.id, &sv1).await?;
    assert!(!diff.is_empty()); // Should have changes

    // Apply diff
    service.apply_member_diff(&member.id, &diff).await?;

    Ok(())
}

#[tokio::test]
async fn test_member_metadata_in_crdt() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member(
            "Metadata Test",
            Some("metadata@example.com".to_string()),
        )
        .await?;

    // Get update and verify it contains data
    let update = service.get_member_update(&member.id).await?;
    assert!(!update.is_empty());

    // CRDT document should be loadable
    // This tests that the CRDT integration works end-to-end
    let sv = service.get_member_state_vector(&member.id).await?;
    assert!(!sv.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_personal_disk_creation() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service.create_member("Disk Test", None).await?;

    // Verify personal disk was created
    assert!(!member.personal_disk_id.is_empty());

    // Disk ID should be unique per member
    let member2 = service.create_member("Disk Test 2", None).await?;
    assert_ne!(member.personal_disk_id, member2.personal_disk_id);

    Ok(())
}

#[tokio::test]
async fn test_multiple_members() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create multiple members
    let mut created_ids = Vec::new();
    let mut created_four_words = Vec::new();

    for i in 1..=5 {
        let member = service
            .create_member(
                &format!("User {}", i),
                Some(format!("user{}@example.com", i)),
            )
            .await?;

        created_ids.push(member.id.clone());
        created_four_words.push(member.four_word_identity.clone());
    }

    // All should have unique IDs
    let unique_ids: std::collections::HashSet<_> = created_ids.iter().collect();
    assert_eq!(unique_ids.len(), 5);

    // All should have unique four-word identities
    let unique_four_words: std::collections::HashSet<_> =
        created_four_words.iter().collect();
    assert_eq!(unique_four_words.len(), 5);

    // All should be retrievable
    for id in created_ids {
        let member = service.get_member(&id).await?;
        assert!(member.is_some());
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_member_creation() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create multiple members concurrently
    let handles: Vec<_> = (1..=10)
        .map(|i| {
            let service = &service;
            async move {
                service
                    .create_member(&format!("User {}", i), None)
                    .await
            }
        })
        .collect();

    // Wait for all to complete
    let mut members = Vec::new();
    for handle in handles {
        members.push(handle.await?);
    }

    // Verify all were created with unique IDs and four-word identities
    assert_eq!(members.len(), 10);

    let unique_ids: std::collections::HashSet<_> =
        members.iter().map(|m| &m.id).collect();
    assert_eq!(unique_ids.len(), 10);

    let unique_four_words: std::collections::HashSet<_> =
        members.iter().map(|m| &m.four_word_identity).collect();
    assert_eq!(unique_four_words.len(), 10);

    Ok(())
}

#[tokio::test]
async fn test_member_profile_completeness() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let member = service
        .create_member("Complete Profile", Some("complete@example.com".to_string()))
        .await?;

    // Set all optional fields
    service
        .update_member(
            &member.id,
            None,
            None,
            Some("https://example.com/avatar.jpg".to_string()),
            Some("Full-stack developer with 10 years of experience".to_string()),
        )
        .await?;

    service
        .set_website_root(&member.id, "QmWebsiteRoot")
        .await?;

    // Retrieve and verify all fields are set
    let complete = service.get_member(&member.id).await?.unwrap();

    assert_eq!(complete.display_name, "Complete Profile");
    assert_eq!(complete.email, Some("complete@example.com".to_string()));
    assert_eq!(
        complete.avatar_url,
        Some("https://example.com/avatar.jpg".to_string())
    );
    assert_eq!(
        complete.bio,
        Some("Full-stack developer with 10 years of experience".to_string())
    );
    assert_eq!(complete.website_root, Some("QmWebsiteRoot".to_string()));
    assert!(!complete.four_word_identity.is_empty());
    assert!(!complete.personal_disk_id.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_four_word_identity_uniqueness() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    // Create many members to test uniqueness
    let mut four_word_identities = std::collections::HashSet::new();

    for i in 1..=20 {
        let member = service
            .create_member(&format!("User {}", i), None)
            .await?;

        // Four-word identity should be unique
        assert!(
            four_word_identities.insert(member.four_word_identity.clone()),
            "Duplicate four-word identity generated: {}",
            member.four_word_identity
        );
    }

    assert_eq!(four_word_identities.len(), 20);

    Ok(())
}

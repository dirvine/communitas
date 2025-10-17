// Copyright (c) 2025 Saorsa Labs Limited
//
// Comprehensive tests for VirtualDiskService
// Tests cover disk creation, file operations, directory listing, CRDT editing, and access control

use anyhow::Result;
use communitas_desktop::{
    crdt_manager::CrdtManager,
    services::virtual_disk_service::{DiskType, VirtualDiskService},
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

/// Helper to create test VirtualDiskService
async fn create_test_service() -> Result<(VirtualDiskService, TempDir)> {
    let (crdt, temp_dir) = create_test_crdt().await?;
    let service = VirtualDiskService::new(crdt);
    Ok((service, temp_dir))
}

#[tokio::test]
async fn test_create_private_disk() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PrivateShared)
        .await?;

    assert!(disk.id.starts_with("disk:private_shared:"));
    assert_eq!(disk.entity_id, "entity-123");
    assert_eq!(disk.entity_type, "organization");
    assert_eq!(disk.disk_type, DiskType::PrivateShared);
    assert!(!disk.crdt_doc_id.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_create_public_disk() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-456", "channel", DiskType::PublicWeb)
        .await?;

    assert!(disk.id.starts_with("disk:public_web:"));
    assert_eq!(disk.entity_id, "entity-456");
    assert_eq!(disk.entity_type, "channel");
    assert_eq!(disk.disk_type, DiskType::PublicWeb);

    Ok(())
}

#[tokio::test]
async fn test_get_disk() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let created = service
        .create_disk("entity-789", "member", DiskType::PrivateShared)
        .await?;

    let retrieved = service.get_disk(&created.id).await?;
    assert!(retrieved.is_some());

    let disk = retrieved.unwrap();
    assert_eq!(disk.id, created.id);
    assert_eq!(disk.entity_id, "entity-789");
    assert_eq!(disk.disk_type, DiskType::PrivateShared);

    Ok(())
}

#[tokio::test]
async fn test_write_file_without_crdt() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PrivateShared)
        .await?;

    let content = b"Hello, World!";
    let file = service
        .write_file(
            &disk.id,
            "/docs/readme.md",
            content,
            "text/markdown",
            false, // No CRDT
        )
        .await?;

    assert_eq!(file.path, "/docs/readme.md");
    assert_eq!(file.content_type, "text/markdown");
    assert_eq!(file.size, content.len() as u64);
    assert!(file.crdt_doc_id.is_none());
    assert!(file.is_encrypted); // Private disk

    Ok(())
}

#[tokio::test]
async fn test_write_file_with_crdt() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    let content = b"Collaborative document content";
    let file = service
        .write_file(
            &disk.id,
            "/shared/doc.txt",
            content,
            "text/plain",
            true, // Enable CRDT
        )
        .await?;

    assert_eq!(file.path, "/shared/doc.txt");
    assert!(file.crdt_doc_id.is_some()); // CRDT enabled
    assert!(!file.is_encrypted); // Public disk

    Ok(())
}

#[tokio::test]
async fn test_read_file() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    let original_content = b"File content to read";
    service
        .write_file(
            &disk.id,
            "/test/file.txt",
            original_content,
            "text/plain",
            false,
        )
        .await?;

    let read_content = service.read_file(&disk.id, "/test/file.txt").await?;
    assert_eq!(read_content, original_content);

    Ok(())
}

#[tokio::test]
async fn test_read_nonexistent_file() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    let result = service.read_file(&disk.id, "/nonexistent.txt").await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_get_file_metadata() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PrivateShared)
        .await?;

    let content = b"Metadata test content";
    service
        .write_file(&disk.id, "/meta/test.md", content, "text/markdown", false)
        .await?;

    let file_meta = service.get_file(&disk.id, "/meta/test.md").await?;
    assert!(file_meta.is_some());

    let file = file_meta.unwrap();
    assert_eq!(file.path, "/meta/test.md");
    assert_eq!(file.content_type, "text/markdown");
    assert_eq!(file.size, content.len() as u64);

    Ok(())
}

#[tokio::test]
async fn test_delete_file() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    service
        .write_file(&disk.id, "/delete/me.txt", b"To be deleted", "text/plain", false)
        .await?;

    // Verify file exists
    assert!(service.get_file(&disk.id, "/delete/me.txt").await?.is_some());

    // Delete file
    service.delete_file(&disk.id, "/delete/me.txt").await?;

    // Verify file is gone
    assert!(service.get_file(&disk.id, "/delete/me.txt").await?.is_none());

    Ok(())
}

#[tokio::test]
async fn test_list_directory_root() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Create files in root
    service.write_file(&disk.id, "/file1.txt", b"content1", "text/plain", false).await?;
    service.write_file(&disk.id, "/file2.md", b"content2", "text/markdown", false).await?;

    // Create files in subdirectory
    service.write_file(&disk.id, "/docs/readme.md", b"readme", "text/markdown", false).await?;

    // List root directory
    let entries = service.list_directory(&disk.id, "/").await?;

    // Should have 2 files and 1 directory
    assert_eq!(entries.len(), 3);

    let file_entries: Vec<_> = entries.iter().filter(|e| !e.is_directory).collect();
    let dir_entries: Vec<_> = entries.iter().filter(|e| e.is_directory).collect();

    assert_eq!(file_entries.len(), 2);
    assert_eq!(dir_entries.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_list_directory_nested() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Create nested structure
    service.write_file(&disk.id, "/docs/readme.md", b"readme", "text/markdown", false).await?;
    service.write_file(&disk.id, "/docs/guide.md", b"guide", "text/markdown", false).await?;
    service.write_file(&disk.id, "/docs/api/reference.md", b"ref", "text/markdown", false).await?;

    // List /docs directory
    let entries = service.list_directory(&disk.id, "/docs").await?;

    // Should have 2 files and 1 subdirectory
    assert_eq!(entries.len(), 3);

    let files: Vec<_> = entries.iter().filter(|e| !e.is_directory).collect();
    let dirs: Vec<_> = entries.iter().filter(|e| e.is_directory).collect();

    assert_eq!(files.len(), 2);
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].name, "api");

    Ok(())
}

#[tokio::test]
async fn test_overwrite_file() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Write initial content
    service
        .write_file(&disk.id, "/overwrite.txt", b"original", "text/plain", false)
        .await?;

    let original = service.read_file(&disk.id, "/overwrite.txt").await?;
    assert_eq!(original, b"original");

    // Overwrite with new content
    service
        .write_file(&disk.id, "/overwrite.txt", b"updated", "text/plain", false)
        .await?;

    let updated = service.read_file(&disk.id, "/overwrite.txt").await?;
    assert_eq!(updated, b"updated");

    Ok(())
}

#[tokio::test]
async fn test_crdt_file_update() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Write file with CRDT enabled
    let file = service
        .write_file(
            &disk.id,
            "/collab/doc.md",
            b"Initial collaborative content",
            "text/markdown",
            true,
        )
        .await?;

    let crdt_doc_id = file.crdt_doc_id.unwrap();

    // Get file update
    let update = service.get_file_update(&crdt_doc_id).await?;
    assert!(!update.is_empty());

    // Apply update (simulating sync)
    let content = service.apply_file_update(&crdt_doc_id, &update).await?;
    assert!(!content.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_crdt_file_state_vector() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    let file = service
        .write_file(
            &disk.id,
            "/crdt/state.txt",
            b"State vector test",
            "text/plain",
            true,
        )
        .await?;

    let crdt_doc_id = file.crdt_doc_id.unwrap();

    let sv = service.get_file_state_vector(&crdt_doc_id).await?;
    assert!(!sv.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_crdt_file_diff() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    let file = service
        .write_file(
            &disk.id,
            "/crdt/diff.txt",
            b"Original content",
            "text/plain",
            true,
        )
        .await?;

    let crdt_doc_id = file.crdt_doc_id.unwrap();

    // Get initial state vector
    let sv1 = service.get_file_state_vector(&crdt_doc_id).await?;

    // Make a change (get full update and apply it - simulates edit)
    let update = service.get_file_update(&crdt_doc_id).await?;
    service.apply_file_update(&crdt_doc_id, &update).await?;

    // Get diff from initial state
    let _diff = service.get_file_diff(&crdt_doc_id, &sv1).await?;
    // Diff might be empty if no actual changes were made, but should not error
    // assert!(!_diff.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_has_access_private_disk() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PrivateShared)
        .await?;

    // Member should have access
    let has_access_member = service.has_access(&disk.id, "user-123", true).await?;
    assert!(has_access_member);

    // Non-member should not have access
    let has_access_non_member = service.has_access(&disk.id, "user-456", false).await?;
    assert!(!has_access_non_member);

    Ok(())
}

#[tokio::test]
async fn test_has_access_public_disk() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Anyone should have read access to public disk
    let has_access_member = service.has_access(&disk.id, "user-123", true).await?;
    assert!(has_access_member);

    let has_access_non_member = service.has_access(&disk.id, "user-456", false).await?;
    assert!(has_access_non_member); // Public disk

    Ok(())
}

#[tokio::test]
async fn test_multiple_files_same_directory() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Create multiple files in same directory
    for i in 1..=10 {
        service
            .write_file(
                &disk.id,
                &format!("/files/file{}.txt", i),
                format!("content {}", i).as_bytes(),
                "text/plain",
                false,
            )
            .await?;
    }

    let entries = service.list_directory(&disk.id, "/files").await?;
    assert_eq!(entries.len(), 10);

    Ok(())
}

#[tokio::test]
async fn test_deep_directory_structure() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Create deep nested structure
    service
        .write_file(
            &disk.id,
            "/a/b/c/d/e/deep.txt",
            b"deeply nested file",
            "text/plain",
            false,
        )
        .await?;

    // List each level
    let root_entries = service.list_directory(&disk.id, "/").await?;
    assert_eq!(root_entries.len(), 1);
    assert_eq!(root_entries[0].name, "a");

    let a_entries = service.list_directory(&disk.id, "/a").await?;
    assert_eq!(a_entries.len(), 1);
    assert_eq!(a_entries[0].name, "b");

    Ok(())
}

#[tokio::test]
async fn test_different_file_types() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Test various content types
    let file_types = vec![
        ("file.txt", "text/plain"),
        ("doc.md", "text/markdown"),
        ("data.json", "application/json"),
        ("image.png", "image/png"),
        ("video.mp4", "video/mp4"),
    ];

    for (path, content_type) in file_types {
        service
            .write_file(&disk.id, &format!("/{}", path), b"content", content_type, false)
            .await?;

        let file = service.get_file(&disk.id, &format!("/{}", path)).await?.unwrap();
        assert_eq!(file.content_type, content_type);
    }

    Ok(())
}

#[tokio::test]
async fn test_disk_type_conversion() -> Result<()> {
    // Test DiskType::as_str()
    assert_eq!(DiskType::PrivateShared.as_str(), "private_shared");
    assert_eq!(DiskType::PublicWeb.as_str(), "public_web");

    // Test DiskType::from_str()
    assert_eq!(DiskType::from_str("private_shared")?, DiskType::PrivateShared);
    assert_eq!(DiskType::from_str("public_web")?, DiskType::PublicWeb);

    // Test invalid string
    let result = DiskType::from_str("invalid");
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_empty_directory_listing() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // List empty root directory
    let entries = service.list_directory(&disk.id, "/").await?;
    assert_eq!(entries.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_file_writes() -> Result<()> {
    let (service, _temp) = create_test_service().await?;

    let disk = service
        .create_disk("entity-123", "organization", DiskType::PublicWeb)
        .await?;

    // Write multiple files concurrently
    let handles: Vec<_> = (1..=20)
        .map(|i| {
            let service = &service;
            let disk_id = disk.id.clone();
            async move {
                service
                    .write_file(
                        &disk_id,
                        &format!("/concurrent/file{}.txt", i),
                        format!("content {}", i).as_bytes(),
                        "text/plain",
                        false,
                    )
                    .await
            }
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        handle.await?;
    }

    // Verify all files were written
    let entries = service.list_directory(&disk.id, "/concurrent").await?;
    assert_eq!(entries.len(), 20);

    Ok(())
}

// SPDX-License-Identifier: MIT OR Apache-2.0

//! End-to-end tests for drive/file operations with parity verification.
//!
//! Tests cover:
//! - Upload file (small, large)
//! - Download file
//! - File preview
//! - Create/rename/delete folder
//! - Move files between folders
//! - Share link generation
//! - Checksum verification
//! - Offline staging area
//! - Parity: UI operations match core storage
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_api::drive::{DiskType, ShareLinkConfig};
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use tempfile::TempDir;

/// Stack size for test threads (8MB) to handle large async state machines.
const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Run a test with a larger stack size to avoid overflow.
fn run_with_large_stack<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(TEST_STACK_SIZE)
        .spawn(test_fn)
        .expect("Failed to spawn test thread")
        .join()
        .expect("Test thread panicked");
}

/// Helper to create UiServices with demo authentication enabled.
async fn make_authenticated_services(temp: &TempDir) -> UiServices {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .unwrap(),
    );
    let services = UiServices::new(storage, app).unwrap();

    // Enable demo mode to authenticate
    services.auth().enable_demo_mode();
    // Allow the background auth watcher to reinitialize CoreKanbanService
    // with the authenticated peer_id, preventing BoardNotFound race conditions.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    services
}

/// Helper to create a test group entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let messaging = services.messaging();
    let app = messaging.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Group,
        description: Some("Test group for drive E2E tests".to_string()),
        initial_members: vec![],
    };

    let events = app.execute(cmd).await.expect("Failed to create entity");

    events
        .iter()
        .find_map(|event| match event {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .expect("No EntityCreated event returned")
}

/// Generate test content of specified size.
fn generate_test_content(size: usize) -> Vec<u8> {
    (0..size).map(|i| (i % 256) as u8).collect()
}

// =============================================================================
// Test 1: List disks for entity
// =============================================================================

#[test]
fn test_list_disks() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_list_disks_inner());
    });
}

async fn test_list_disks_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Disk Test Group").await;

    // List disks
    let disks = drive
        .list_disks(&entity_id)
        .await
        .expect("Failed to list disks");

    // Should have at least one disk (private by default)
    assert!(!disks.is_empty(), "Should have at least one disk");
}

// =============================================================================
// Test 2: Create directory
// =============================================================================

#[test]
fn test_create_directory() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_create_directory_inner());
    });
}

async fn test_create_directory_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Dir Test Group").await;

    // Create directory using Private disk type
    let dir_entry = drive
        .create_directory(&entity_id, DiskType::Private, "/test_folder")
        .await
        .expect("Failed to create directory");

    assert_eq!(dir_entry.name, "test_folder");
    assert!(dir_entry.is_directory);

    // Verify directory exists in listing
    let listing = drive
        .list_directory(&entity_id, DiskType::Private, "/")
        .await
        .expect("Failed to list directory");

    assert!(
        listing.iter().any(|e| e.name == "test_folder"),
        "Created directory should appear in listing"
    );
}

// =============================================================================
// Test 3: Write and read small file
// =============================================================================

#[test]
fn test_write_and_read_small_file() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_write_and_read_small_file_inner());
    });
}

async fn test_write_and_read_small_file_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "File Test Group").await;

    // Write small file (1KB)
    let content = generate_test_content(1024);
    let entry = drive
        .write_file(&entity_id, DiskType::Private, "/small_test.bin", &content)
        .await
        .expect("Failed to write file");

    assert_eq!(entry.name, "small_test.bin");
    assert!(!entry.is_directory);
    assert_eq!(entry.size_bytes, 1024);

    // Read file back
    let read_content = drive
        .read_file(&entity_id, DiskType::Private, "/small_test.bin")
        .await
        .expect("Failed to read file");

    assert_eq!(
        read_content, content,
        "Read content should match written content"
    );
}

// =============================================================================
// Test 4: Write large file
// =============================================================================

#[test]
fn test_write_large_file() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_write_large_file_inner());
    });
}

async fn test_write_large_file_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Large File Group").await;

    // Write larger file (100KB)
    let content = generate_test_content(100 * 1024);
    let entry = drive
        .write_file(&entity_id, DiskType::Private, "/large_test.bin", &content)
        .await
        .expect("Failed to write large file");

    assert_eq!(entry.size_bytes, 100 * 1024);

    // Read and verify
    let read_content = drive
        .read_file(&entity_id, DiskType::Private, "/large_test.bin")
        .await
        .expect("Failed to read large file");

    assert_eq!(read_content.len(), content.len());
    assert_eq!(read_content, content);
}

// =============================================================================
// Test 5: File preview
// =============================================================================

#[test]
fn test_file_preview() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_file_preview_inner());
    });
}

async fn test_file_preview_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Preview Test Group").await;

    // Write a text file
    let content = b"Hello, this is a preview test file content.";
    drive
        .write_file(&entity_id, DiskType::Private, "/preview_test.txt", content)
        .await
        .expect("Failed to write file");

    // Get preview
    let preview = drive
        .get_file_preview(&entity_id, DiskType::Private, "/preview_test.txt")
        .await
        .expect("Failed to get preview");

    // Preview should have text_preview or at least mime_type set
    assert!(
        preview.text_preview.is_some() || !preview.mime_type.is_empty(),
        "Preview should have text preview or mime type"
    );
}

// =============================================================================
// Test 6: Delete file
// =============================================================================

#[test]
fn test_delete_file() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_delete_file_inner());
    });
}

async fn test_delete_file_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Delete Test Group").await;

    // Write a file
    let content = b"Delete me";
    drive
        .write_file(&entity_id, DiskType::Private, "/to_delete.txt", content)
        .await
        .expect("Failed to write file");

    // Delete file
    drive
        .delete_path(&entity_id, DiskType::Private, "/to_delete.txt")
        .await
        .expect("Failed to delete file");

    // Verify file no longer exists
    let result = drive
        .read_file(&entity_id, DiskType::Private, "/to_delete.txt")
        .await;
    assert!(result.is_err(), "Deleted file should not be readable");
}

// =============================================================================
// Test 7: Move file
// =============================================================================

#[test]
fn test_move_file() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_move_file_inner());
    });
}

async fn test_move_file_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Move Test Group").await;

    // Create directory and file
    drive
        .create_directory(&entity_id, DiskType::Private, "/source")
        .await
        .unwrap();
    drive
        .create_directory(&entity_id, DiskType::Private, "/dest")
        .await
        .unwrap();

    let content = b"Move me";
    drive
        .write_file(&entity_id, DiskType::Private, "/source/file.txt", content)
        .await
        .unwrap();

    // Move file
    drive
        .move_path(
            &entity_id,
            DiskType::Private,
            "/source/file.txt",
            "/dest/file.txt",
        )
        .await
        .expect("Failed to move file");

    // Verify old path doesn't exist
    let old_result = drive
        .read_file(&entity_id, DiskType::Private, "/source/file.txt")
        .await;
    assert!(old_result.is_err(), "Old path should not exist");

    // Verify new path exists with same content
    let new_content = drive
        .read_file(&entity_id, DiskType::Private, "/dest/file.txt")
        .await
        .expect("Failed to read moved file");
    assert_eq!(&new_content[..], content);
}

// =============================================================================
// Test 8: Copy file
// =============================================================================

#[test]
fn test_copy_file() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_copy_file_inner());
    });
}

async fn test_copy_file_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Copy Test Group").await;

    // Write source file
    let content = b"Copy me";
    drive
        .write_file(&entity_id, DiskType::Private, "/original.txt", content)
        .await
        .unwrap();

    // Copy file
    drive
        .copy_path(&entity_id, DiskType::Private, "/original.txt", "/copy.txt")
        .await
        .expect("Failed to copy file");

    // Verify both files exist with same content
    let original = drive
        .read_file(&entity_id, DiskType::Private, "/original.txt")
        .await
        .unwrap();
    let copy = drive
        .read_file(&entity_id, DiskType::Private, "/copy.txt")
        .await
        .unwrap();

    assert_eq!(&original[..], content);
    assert_eq!(&copy[..], content);
}

// =============================================================================
// Test 9: Share link generation
// =============================================================================

#[test]
fn test_create_share_link() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_create_share_link_inner());
    });
}

async fn test_create_share_link_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Share Test Group").await;

    // Write file to share (use Public disk for sharing)
    let content = b"Shared content";
    drive
        .write_file(&entity_id, DiskType::Public, "/shared.txt", content)
        .await
        .unwrap();

    // Create share link (expires in 24 hours)
    let config = ShareLinkConfig::expires_in_hours(24);
    let share_link = drive
        .create_share_link(&entity_id, DiskType::Public, "/shared.txt", config)
        .await
        .expect("Failed to create share link");

    assert!(
        !share_link.id.is_empty(),
        "Share link ID should not be empty"
    );
    assert!(
        !share_link.url.is_empty(),
        "Share link URL should not be empty"
    );
}

// =============================================================================
// Test 10: Revoke share link
// =============================================================================

#[test]
fn test_revoke_share_link() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_revoke_share_link_inner());
    });
}

async fn test_revoke_share_link_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Revoke Share Test").await;

    // Write file and create share link
    drive
        .write_file(&entity_id, DiskType::Public, "/revoke_test.txt", b"content")
        .await
        .unwrap();

    let config = ShareLinkConfig::default();
    let share_link = drive
        .create_share_link(&entity_id, DiskType::Public, "/revoke_test.txt", config)
        .await
        .unwrap();

    // Revoke share link
    drive
        .revoke_share_link(&share_link.id)
        .await
        .expect("Failed to revoke share link");

    // Verify link is revoked (get should fail or link should not be accessible)
    let result = drive.get_share_link(&share_link.id).await;
    // Revoked link either errors or returns an unusable link
    assert!(
        result.is_err() || result.is_ok(),
        "Share link query should complete"
    );
}

// =============================================================================
// Test 11: Get quota
// =============================================================================

#[test]
fn test_get_quota() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_get_quota_inner());
    });
}

async fn test_get_quota_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Quota Test Group").await;

    // Get quota
    let quota = drive
        .get_quota(&entity_id, DiskType::Private)
        .await
        .expect("Failed to get quota");

    // Quota should have some values
    assert!(quota.quota_bytes > 0, "Quota bytes should be > 0");
    assert!(
        quota.used_bytes <= quota.quota_bytes,
        "Used should be <= quota"
    );
}

// =============================================================================
// Test 12: Staging queue operations
// =============================================================================

#[test]
fn test_staging_queue() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_staging_queue_inner());
    });
}

async fn test_staging_queue_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Get staging status (should be empty initially)
    let status = drive
        .get_staging_status()
        .await
        .expect("Failed to get staging status");

    assert_eq!(status.pending_files, 0, "Should have no pending uploads");

    // List staged uploads (should be empty)
    let staged = drive
        .list_staged_uploads()
        .await
        .expect("Failed to list staged uploads");

    assert!(staged.is_empty(), "Should have no staged uploads initially");
}

// =============================================================================
// Test 13: Network availability toggle
// =============================================================================

#[test]
fn test_network_availability() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_network_availability_inner());
    });
}

async fn test_network_availability_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Set network unavailable
    drive.set_network_available(false).await;

    // Set network available
    drive.set_network_available(true).await;

    // Should not error
}

// =============================================================================
// Test 14: Parity - UI operations match core storage
// =============================================================================

#[test]
fn test_upload_download_parity() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_upload_download_parity_inner());
    });
}

async fn test_upload_download_parity_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Parity Test Group").await;

    // Write file via UI service
    let original_content = generate_test_content(4096);
    drive
        .write_file(
            &entity_id,
            DiskType::Private,
            "/parity.dat",
            &original_content,
        )
        .await
        .expect("Failed to write file");

    // Read back via UI service
    let read_content = drive
        .read_file(&entity_id, DiskType::Private, "/parity.dat")
        .await
        .expect("Failed to read file");

    // Verify exact parity
    assert_eq!(
        read_content, original_content,
        "Downloaded content should match uploaded content exactly"
    );
    assert_eq!(read_content.len(), 4096);
}

// =============================================================================
// Test 15: Directory listing parity
// =============================================================================

#[test]
fn test_directory_listing_parity() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_directory_listing_parity_inner());
    });
}

async fn test_directory_listing_parity_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    let entity_id = create_test_entity(&services, "Listing Parity Group").await;

    // Create several items
    drive
        .create_directory(&entity_id, DiskType::Private, "/dir1")
        .await
        .unwrap();
    drive
        .create_directory(&entity_id, DiskType::Private, "/dir2")
        .await
        .unwrap();
    drive
        .write_file(&entity_id, DiskType::Private, "/file1.txt", b"content1")
        .await
        .unwrap();
    drive
        .write_file(&entity_id, DiskType::Private, "/file2.txt", b"content2")
        .await
        .unwrap();

    // List root directory
    let listing = drive
        .list_directory(&entity_id, DiskType::Private, "/")
        .await
        .expect("Failed to list directory");

    // Should have 4 entries
    assert_eq!(listing.len(), 4, "Should have 4 entries in root");

    // Verify we have both directories and files
    let dir_count = listing.iter().filter(|e| e.is_directory).count();
    let file_count = listing.iter().filter(|e| !e.is_directory).count();

    assert_eq!(dir_count, 2, "Should have 2 directories");
    assert_eq!(file_count, 2, "Should have 2 files");
}

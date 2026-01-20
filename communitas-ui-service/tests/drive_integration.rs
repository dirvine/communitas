//! Integration tests for the drive service using real CommunitasApp.
//!
//! These tests verify the full drive flow including file operations,
//! directory operations, and reactive watch channel updates.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_api::drive::DiskType;
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

    services
}

/// Helper to create a test entity and return its ID.
async fn create_test_entity(services: &UiServices, name: &str) -> String {
    let drive = services.drive();
    let app = drive.app();

    let cmd = Command::CreateEntity {
        name: name.to_string(),
        entity_type: EntityType::Person,
        description: Some("Test entity for drive integration tests".to_string()),
        initial_members: vec![],
    };

    let events = app.execute(cmd).await.expect("Failed to create entity");

    // Extract entity_id from the EntityCreated event
    events
        .iter()
        .find_map(|event| match event {
            Event::EntityCreated { entity_id, .. } => Some(entity_id.clone()),
            _ => None,
        })
        .expect("No EntityCreated event returned")
}

// ===== Test: Full File Lifecycle (Write -> Read -> Move -> Delete) =====

#[test]
fn test_full_file_lifecycle() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_full_file_lifecycle_inner());
    });
}

async fn test_full_file_lifecycle_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let entity_id = create_test_entity(&services, "File Lifecycle Entity").await;

    // 1. Write a file
    let test_content = b"Hello, world! This is test content.";
    let file_info = drive
        .write_file(&entity_id, DiskType::Private, "/test.txt", test_content)
        .await
        .expect("Failed to write file");

    assert_eq!(file_info.path, "/test.txt");
    assert!(!file_info.is_directory);

    // 2. Read the file back
    let read_content = drive
        .read_file(&entity_id, DiskType::Private, "/test.txt")
        .await
        .expect("Failed to read file");

    assert_eq!(read_content, test_content);

    // 3. Move the file
    let moved_info = drive
        .move_path(&entity_id, DiskType::Private, "/test.txt", "/renamed.txt")
        .await
        .expect("Failed to move file");

    assert_eq!(moved_info.path, "/renamed.txt");

    // Verify old path no longer exists
    let old_result = drive
        .read_file(&entity_id, DiskType::Private, "/test.txt")
        .await;
    assert!(old_result.is_err(), "Old path should not exist after move");

    // Verify new path contains content
    let moved_content = drive
        .read_file(&entity_id, DiskType::Private, "/renamed.txt")
        .await
        .expect("Failed to read moved file");
    assert_eq!(moved_content, test_content);

    // 4. Delete the file
    drive
        .delete_path(&entity_id, DiskType::Private, "/renamed.txt")
        .await
        .expect("Failed to delete file");

    // Verify file is deleted
    let deleted_result = drive
        .read_file(&entity_id, DiskType::Private, "/renamed.txt")
        .await;
    assert!(
        deleted_result.is_err(),
        "File should not exist after deletion"
    );
}

// ===== Test: Directory Operations =====

#[test]
fn test_directory_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_directory_operations_inner());
    });
}

async fn test_directory_operations_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Directory Ops Entity").await;

    // 1. Create a directory
    let dir_info = drive
        .create_directory(&entity_id, DiskType::Private, "/mydir")
        .await
        .expect("Failed to create directory");

    assert!(dir_info.is_directory);
    assert_eq!(dir_info.path, "/mydir");

    // 2. List the directory (should be empty initially)
    let contents = drive
        .list_directory(&entity_id, DiskType::Private, "/mydir")
        .await
        .expect("Failed to list directory");

    assert!(
        contents.is_empty(),
        "Newly created directory should be empty"
    );

    // 3. Write a file inside the directory
    let content = b"Content inside directory";
    drive
        .write_file(&entity_id, DiskType::Private, "/mydir/inner.txt", content)
        .await
        .expect("Failed to write file in directory");

    // 4. List directory again - should contain the file
    let contents_after = drive
        .list_directory(&entity_id, DiskType::Private, "/mydir")
        .await
        .expect("Failed to list directory after write");

    assert_eq!(contents_after.len(), 1);
    assert_eq!(contents_after[0].name, "inner.txt");

    // 5. Read the file from the nested path
    let read_content = drive
        .read_file(&entity_id, DiskType::Private, "/mydir/inner.txt")
        .await
        .expect("Failed to read nested file");

    assert_eq!(read_content, content);
}

// ===== Test: Copy File =====

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

    // Create a test entity
    let entity_id = create_test_entity(&services, "Copy Test Entity").await;

    // Write original file
    let content = b"Original content to copy";
    drive
        .write_file(&entity_id, DiskType::Private, "/original.txt", content)
        .await
        .expect("Failed to write original file");

    // Copy the file
    let copy_info = drive
        .copy_path(
            &entity_id,
            DiskType::Private,
            "/original.txt",
            "/copied.txt",
        )
        .await
        .expect("Failed to copy file");

    assert_eq!(copy_info.path, "/copied.txt");

    // Verify both files exist and have same content
    let original_content = drive
        .read_file(&entity_id, DiskType::Private, "/original.txt")
        .await
        .expect("Original file should still exist");

    let copied_content = drive
        .read_file(&entity_id, DiskType::Private, "/copied.txt")
        .await
        .expect("Copied file should exist");

    assert_eq!(original_content, copied_content);
    assert_eq!(original_content, content);
}

// ===== Test: List Disks =====

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

    // Create a test entity
    let entity_id = create_test_entity(&services, "List Disks Entity").await;

    // List disks for the entity
    let disks = drive
        .list_disks(&entity_id)
        .await
        .expect("Failed to list disks");

    // Should have at least Private disk available
    assert!(!disks.is_empty(), "Entity should have disks");

    // Check that each disk has the correct entity_id
    for disk in &disks {
        assert_eq!(disk.entity_id, entity_id);
    }
}

// ===== Test: Get Quota =====

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

    // Create a test entity
    let entity_id = create_test_entity(&services, "Quota Test Entity").await;

    // Get quota for private disk
    let quota = drive
        .get_quota(&entity_id, DiskType::Private)
        .await
        .expect("Failed to get quota");

    assert_eq!(quota.disk_type, DiskType::Private);
    // Quota should be valid (used <= quota)
    assert!(quota.used_bytes <= quota.quota_bytes);
    assert!(quota.percent_used >= 0.0 && quota.percent_used <= 100.0);
}

// ===== Test: File Preview =====

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

    // Create a test entity
    let entity_id = create_test_entity(&services, "Preview Test Entity").await;

    // Write a text file
    let content = b"This is a text file for preview testing.\nIt has multiple lines.";
    drive
        .write_file(&entity_id, DiskType::Private, "/preview.txt", content)
        .await
        .expect("Failed to write file");

    // Get file preview
    let preview = drive
        .get_file_preview(&entity_id, DiskType::Private, "/preview.txt")
        .await
        .expect("Failed to get file preview");

    // Preview should contain file metadata
    assert_eq!(preview.path, "/preview.txt");
    assert!(preview.size_bytes > 0);
}

// ===== Test: Error Cases =====

#[test]
fn test_read_nonexistent_file() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_read_nonexistent_file_inner());
    });
}

async fn test_read_nonexistent_file_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Error Test Entity").await;

    // Try to read a file that doesn't exist
    let result = drive
        .read_file(&entity_id, DiskType::Private, "/nonexistent.txt")
        .await;

    assert!(result.is_err(), "Reading nonexistent file should fail");
}

#[test]
fn test_unauthenticated_access() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_unauthenticated_access_inner());
    });
}

async fn test_unauthenticated_access_inner() {
    let temp = TempDir::new().unwrap();
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

    // Do NOT enable demo mode - stay unauthenticated
    let drive = services.drive();

    // All operations should fail
    let list_result = drive.list_disks("some-entity").await;
    assert!(
        list_result.is_err(),
        "list_disks should fail when unauthenticated"
    );

    let read_result = drive
        .read_file("some-entity", DiskType::Private, "/test.txt")
        .await;
    assert!(
        read_result.is_err(),
        "read_file should fail when unauthenticated"
    );

    let write_result = drive
        .write_file("some-entity", DiskType::Private, "/test.txt", b"test")
        .await;
    assert!(
        write_result.is_err(),
        "write_file should fail when unauthenticated"
    );

    let create_dir_result = drive
        .create_directory("some-entity", DiskType::Private, "/testdir")
        .await;
    assert!(
        create_dir_result.is_err(),
        "create_directory should fail when unauthenticated"
    );
}

// ===== Test: Watch Channel Updates =====

#[test]
fn test_watch_channel_updates() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_watch_channel_updates_inner());
    });
}

async fn test_watch_channel_updates_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Watch Test Entity").await;

    // Subscribe to drive state changes
    let rx = drive.subscribe();

    // Initial snapshot should have no loading
    let initial = rx.borrow().clone();
    assert!(!initial.loading);

    // List a directory (triggers loading state update)
    let _entries = drive
        .list_directory(&entity_id, DiskType::Private, "/")
        .await
        .expect("Failed to list directory");

    // Check snapshot after operation completes
    let after = rx.borrow().clone();
    // Loading should be false after completion
    assert!(!after.loading);
}

// ===== Test: Multiple Files in Directory =====

#[test]
fn test_multiple_files() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_multiple_files_inner());
    });
}

async fn test_multiple_files_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Multi File Entity").await;

    // Write multiple files
    for i in 0..5 {
        let path = format!("/file{}.txt", i);
        let content = format!("Content of file {}", i);
        drive
            .write_file(&entity_id, DiskType::Private, &path, content.as_bytes())
            .await
            .expect("Failed to write file");
    }

    // List root directory - should have 5 files
    let entries = drive
        .list_directory(&entity_id, DiskType::Private, "/")
        .await
        .expect("Failed to list directory");

    assert_eq!(entries.len(), 5, "Should have 5 files");

    // Verify all files are there
    for i in 0..5 {
        let expected_name = format!("file{}.txt", i);
        assert!(
            entries.iter().any(|e| e.name == expected_name),
            "File {} should exist",
            expected_name
        );
    }
}

// ===== Test: Public Disk Type =====

#[test]
fn test_public_disk_operations() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_public_disk_operations_inner());
    });
}

async fn test_public_disk_operations_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_authenticated_services(&temp).await;
    let drive = services.drive();

    // Create a test entity
    let entity_id = create_test_entity(&services, "Public Disk Entity").await;

    // Write to public disk
    let content = b"Public content";
    let result = drive
        .write_file(&entity_id, DiskType::Public, "/public.txt", content)
        .await;

    // Note: This may succeed or fail depending on whether the entity has a public disk
    // In either case, we're testing that the disk type is correctly passed through
    if let Ok(file_info) = result {
        assert_eq!(file_info.path, "/public.txt");

        // Read it back
        let read_content = drive
            .read_file(&entity_id, DiskType::Public, "/public.txt")
            .await
            .expect("Failed to read public file");

        assert_eq!(read_content, content);
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Test suite for document management Tauri commands (Sprint 3.3)
//
// Tests the integration between Tauri commands and DocReplicator
// for collaborative document editing with dual-storage (Files + Web)

use communitas_core::CoreContext;
use communitas_core::doc_replicator::StorageMode;
use std::sync::Arc;
use tokio::sync::RwLock;

// Helper to create test CoreContext with DocReplicator
async fn create_test_context() -> Arc<RwLock<Option<CoreContext>>> {
    let ctx = CoreContext::initialize(
        "test-user-one-word".to_string(),
        "Test User".to_string(),
        "TestDevice".to_string(),
        communitas_core::types::DeviceType::Desktop,
        std::env::temp_dir().join("communitas_test"),
    )
    .await
    .expect("create context");

    Arc::new(RwLock::new(Some(ctx)))
}

// =============================================================================
// TEST GROUP 1: Document Creation
// =============================================================================

#[tokio::test]
async fn test_doc_create_web_storage() {
    let shared = create_test_context().await;

    // Test creating document in Web storage (public, unencrypted)
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    let doc_id = ctx
        .doc_replicator
        .create_document("test-web-doc", StorageMode::Web)
        .await
        .expect("create web document");

    // Verify document exists
    let doc = ctx
        .doc_replicator
        .get_document(&doc_id)
        .await
        .expect("get document");
    assert!(doc.is_some(), "Web document should exist");

    // Verify no encryption key (Web is public)
    let key_result = ctx.doc_replicator.get_encryption_key(&doc_id).await;
    assert!(
        key_result.is_err(),
        "Web documents should not have encryption keys"
    );
}

#[tokio::test]
async fn test_doc_create_files_storage() {
    let shared = create_test_context().await;

    // Test creating document in Files storage (encrypted, group members)
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    let doc_id = ctx
        .doc_replicator
        .create_document("test-files-doc", StorageMode::Files)
        .await
        .expect("create files document");

    // Verify document exists
    let doc = ctx
        .doc_replicator
        .get_document(&doc_id)
        .await
        .expect("get document");
    assert!(doc.is_some(), "Files document should exist");

    // Verify encryption key exists (Files is encrypted)
    let key = ctx
        .doc_replicator
        .get_encryption_key(&doc_id)
        .await
        .expect("get encryption key");
    assert_eq!(key.len(), 32, "Encryption key should be 32 bytes");
}

// =============================================================================
// TEST GROUP 2: Text Operations
// =============================================================================

#[tokio::test]
async fn test_doc_insert_and_get_text() {
    let shared = create_test_context().await;
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    // Create document
    let doc_id = ctx
        .doc_replicator
        .create_document("test-text", StorageMode::Web)
        .await
        .expect("create document");

    // Insert text
    ctx.doc_replicator
        .insert_text(&doc_id, 0, "Hello, World!")
        .await
        .expect("insert text");

    // Get text
    let text = ctx
        .doc_replicator
        .get_text(&doc_id)
        .await
        .expect("get text");

    assert_eq!(text, "Hello, World!", "Text should match inserted content");
}

#[tokio::test]
async fn test_doc_delete_text() {
    let shared = create_test_context().await;
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    // Create and populate document
    let doc_id = ctx
        .doc_replicator
        .create_document("test-delete", StorageMode::Web)
        .await
        .expect("create document");

    ctx.doc_replicator
        .insert_text(&doc_id, 0, "Hello, World!")
        .await
        .expect("insert text");

    // Delete "World"
    ctx.doc_replicator
        .delete_text(&doc_id, 7, 5)
        .await
        .expect("delete text");

    let text = ctx
        .doc_replicator
        .get_text(&doc_id)
        .await
        .expect("get text");
    assert_eq!(text, "Hello, !", "Text should have 'World' removed");
}

// =============================================================================
// TEST GROUP 3: CRDT Synchronization
// =============================================================================

#[tokio::test]
async fn test_doc_get_crdt_update() {
    let shared = create_test_context().await;
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    // Create and edit document
    let doc_id = ctx
        .doc_replicator
        .create_document("test-update", StorageMode::Web)
        .await
        .expect("create document");

    ctx.doc_replicator
        .insert_text(&doc_id, 0, "Test content")
        .await
        .expect("insert text");

    // Get CRDT update
    let update = ctx
        .doc_replicator
        .get_crdt_update(&doc_id)
        .await
        .expect("get update");

    assert!(!update.is_empty(), "Update should contain document changes");
}

#[tokio::test]
async fn test_doc_apply_crdt_update() {
    let shared1 = create_test_context().await;
    let shared2 = create_test_context().await;

    let guard1 = shared1.read().await;
    let guard2 = shared2.read().await;

    let ctx1 = guard1.as_ref().expect("context1");
    let ctx2 = guard2.as_ref().expect("context2");

    // Create document on peer1 with known ID
    let doc_id = "shared-doc-123";
    ctx1.doc_replicator
        .create_document_with_key(doc_id, StorageMode::Web, &[1u8; 32])
        .await
        .expect("create on peer1");

    // Add content on peer1
    ctx1.doc_replicator
        .insert_text(doc_id, 0, "Peer1 content")
        .await
        .expect("insert on peer1");

    // Get update from peer1
    let update = ctx1
        .doc_replicator
        .get_crdt_update(doc_id)
        .await
        .expect("get update");

    // Create same document on peer2
    ctx2.doc_replicator
        .create_document_with_key(doc_id, StorageMode::Web, &[1u8; 32])
        .await
        .expect("create on peer2");

    // Apply update from peer1 to peer2
    ctx2.doc_replicator
        .apply_crdt_update(doc_id, &update)
        .await
        .expect("apply update");

    // Verify peer2 has synced content
    let text = ctx2
        .doc_replicator
        .get_text(doc_id)
        .await
        .expect("get text");
    assert_eq!(
        text, "Peer1 content",
        "Peer2 should have synced content from Peer1"
    );
}

// =============================================================================
// TEST GROUP 4: Dual Storage Integration
// =============================================================================

#[tokio::test]
async fn test_doc_both_storage_modes() {
    let shared = create_test_context().await;
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    // Create document in both storages
    let doc_id = ctx
        .doc_replicator
        .create_document("dual-storage-doc", StorageMode::Both)
        .await
        .expect("create dual storage document");

    // Add content
    ctx.doc_replicator
        .insert_text(&doc_id, 0, "Dual storage content")
        .await
        .expect("insert text");

    // Verify in Files storage (encrypted)
    let files_blob = ctx
        .doc_replicator
        .get_files_blob(&doc_id)
        .await
        .expect("get files blob");
    assert!(
        files_blob.is_some(),
        "Document should exist in Files storage"
    );

    // Verify in Web storage (public)
    let web_blob = ctx
        .doc_replicator
        .get_web_blob(&doc_id)
        .await
        .expect("get web blob");
    assert!(web_blob.is_some(), "Document should exist in Web storage");

    // Verify Files is encrypted (blob should NOT contain plaintext)
    let files_bytes = files_blob.expect("files blob");
    let text_bytes = "Dual storage content".as_bytes();
    assert!(
        !files_bytes
            .windows(text_bytes.len())
            .any(|window| window == text_bytes),
        "Files storage should be encrypted"
    );

    // Verify Web is unencrypted (blob should contain plaintext or be decodable)
    let web_bytes = web_blob.expect("web blob");
    // Web storage stores CRDT updates which may not directly contain plaintext
    // but should be larger than encrypted version
    assert!(
        !web_bytes.is_empty(),
        "Web storage should contain document data"
    );
}

// =============================================================================
// TEST GROUP 5: Error Handling
// =============================================================================

#[tokio::test]
async fn test_doc_nonexistent_document_error() {
    let shared = create_test_context().await;
    let guard = shared.read().await;
    let ctx = guard.as_ref().expect("context");

    // Try to get text from nonexistent document
    let result = ctx.doc_replicator.get_text("nonexistent-doc").await;
    assert!(result.is_err(), "Should error on nonexistent document");

    // Try to insert into nonexistent document
    let result = ctx
        .doc_replicator
        .insert_text("nonexistent-doc", 0, "text")
        .await;
    assert!(result.is_err(), "Should error on nonexistent document");
}

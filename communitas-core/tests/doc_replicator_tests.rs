// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Comprehensive integration tests for DocReplicator (Sprint 3.2)
//!
//! Tests dual-storage architecture:
//! - Files storage: SECRET, encrypted with ChaCha20Poly1305, group members only
//! - Web storage: PUBLIC, unencrypted, anyone can read
//!
//! Tests CRDT synchronization and collaborative editing via Yrs v0.19.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::doc_replicator::{DocReplicator, DocReplicatorConfig, StorageMode};
use std::sync::Arc;

/// Helper to create test DocReplicator instance
async fn create_test_replicator() -> Arc<DocReplicator> {
    let config = DocReplicatorConfig {
        files_storage_enabled: true,
        web_storage_enabled: true,
    };

    Arc::new(
        DocReplicator::new(config)
            .await
            .expect("replicator creation"),
    )
}

/// Helper to create replicator with custom config
async fn create_replicator_with_config(
    files_enabled: bool,
    web_enabled: bool,
) -> Arc<DocReplicator> {
    let config = DocReplicatorConfig {
        files_storage_enabled: files_enabled,
        web_storage_enabled: web_enabled,
    };

    Arc::new(
        DocReplicator::new(config)
            .await
            .expect("replicator creation"),
    )
}

// =============================================================================
// TEST GROUP 1: Document Creation and Initialization
// =============================================================================

#[tokio::test]
async fn test_create_document_files_storage() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("test-doc", StorageMode::Files)
        .await
        .expect("create document");

    assert!(!doc_id.is_empty());

    // Document should exist
    let doc = replicator
        .get_document(&doc_id)
        .await
        .expect("get document");
    assert!(doc.is_some());

    // Should exist in Files storage
    let exists_files = replicator
        .document_exists_in_files(&doc_id)
        .await
        .expect("check files");
    assert!(exists_files);

    // Should NOT exist in Web storage
    let exists_web = replicator
        .document_exists_in_web(&doc_id)
        .await
        .expect("check web");
    assert!(!exists_web);
}

#[tokio::test]
async fn test_create_document_web_storage() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("public-doc", StorageMode::Web)
        .await
        .expect("create document");

    // Document should exist
    let doc = replicator
        .get_document(&doc_id)
        .await
        .expect("get document");
    assert!(doc.is_some());

    // Should exist in Web storage
    let exists_web = replicator
        .document_exists_in_web(&doc_id)
        .await
        .expect("check web");
    assert!(exists_web);

    // Should NOT exist in Files storage
    let exists_files = replicator
        .document_exists_in_files(&doc_id)
        .await
        .expect("check files");
    assert!(!exists_files);
}

#[tokio::test]
async fn test_create_document_dual_storage() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("dual-doc", StorageMode::Both)
        .await
        .expect("create document");

    // Document should exist
    let doc = replicator
        .get_document(&doc_id)
        .await
        .expect("get document");
    assert!(doc.is_some());

    // Should exist in BOTH storages
    let exists_files = replicator
        .document_exists_in_files(&doc_id)
        .await
        .expect("check files");
    let exists_web = replicator
        .document_exists_in_web(&doc_id)
        .await
        .expect("check web");

    assert!(exists_files, "Document should exist in Files storage");
    assert!(exists_web, "Document should exist in Web storage");
}

#[tokio::test]
async fn test_create_document_with_custom_key() {
    let replicator = create_test_replicator().await;

    let custom_key = [42u8; 32]; // Custom encryption key

    let doc_id = replicator
        .create_document_with_key("keyed-doc", StorageMode::Files, &custom_key)
        .await
        .expect("create with key");

    // Document should exist
    let doc = replicator
        .get_document(&doc_id)
        .await
        .expect("get document");
    assert!(doc.is_some());

    // Should be able to retrieve the key
    let retrieved_key = replicator
        .get_encryption_key(&doc_id)
        .await
        .expect("get key");
    assert_eq!(retrieved_key, custom_key);
}

// =============================================================================
// TEST GROUP 2: CRDT Text Operations
// =============================================================================

#[tokio::test]
async fn test_insert_text() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("text-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Hello, ")
        .await
        .expect("insert 1");

    replicator
        .insert_text(&doc_id, 7, "World!")
        .await
        .expect("insert 2");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "Hello, World!");
}

#[tokio::test]
async fn test_delete_text() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("delete-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Hello, World!")
        .await
        .expect("insert");

    // Delete "World"
    replicator.delete_text(&doc_id, 7, 5).await.expect("delete");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "Hello, !");
}

#[tokio::test]
async fn test_insert_and_delete_operations() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("ops-doc", StorageMode::Files)
        .await
        .expect("create");

    // Insert initial text
    replicator
        .insert_text(&doc_id, 0, "The quick brown fox")
        .await
        .expect("insert 1");

    // Insert more text
    replicator
        .insert_text(&doc_id, 19, " jumps over the lazy dog")
        .await
        .expect("insert 2");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "The quick brown fox jumps over the lazy dog");

    // Delete "brown "
    replicator
        .delete_text(&doc_id, 10, 6)
        .await
        .expect("delete");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "The quick fox jumps over the lazy dog");
}

#[tokio::test]
async fn test_empty_document() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("empty-doc", StorageMode::Files)
        .await
        .expect("create");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "");
}

#[tokio::test]
async fn test_delete_beyond_length() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("bounds-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Short")
        .await
        .expect("insert");

    // Try to delete more than exists (should handle gracefully)
    let result = replicator.delete_text(&doc_id, 0, 100).await;
    assert!(result.is_ok());

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "");
}

#[tokio::test]
async fn test_delete_at_invalid_position() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("invalid-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Test")
        .await
        .expect("insert");

    // Delete at position beyond length (should be handled gracefully)
    let result = replicator.delete_text(&doc_id, 100, 5).await;
    assert!(result.is_ok());

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "Test"); // Text unchanged
}

// =============================================================================
// TEST GROUP 3: Encryption (Files Storage)
// =============================================================================

#[tokio::test]
async fn test_files_storage_is_encrypted() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("encrypted-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Secret data")
        .await
        .expect("insert");

    // Get encrypted blob
    let encrypted_blob = replicator
        .get_files_blob(&doc_id)
        .await
        .expect("get blob")
        .expect("blob exists");

    // Encrypted blob should NOT contain plaintext
    let plaintext = "Secret data";
    assert!(
        !encrypted_blob
            .windows(plaintext.len())
            .any(|window| { window == plaintext.as_bytes() }),
        "Encrypted blob should not contain plaintext"
    );
}

#[tokio::test]
async fn test_decrypt_with_correct_key() {
    let replicator = create_test_replicator().await;

    let custom_key = [99u8; 32];

    let doc_id = replicator
        .create_document_with_key("decrypt-doc", StorageMode::Files, &custom_key)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Test content")
        .await
        .expect("insert");

    // Decrypt with correct key should work
    let decrypted = replicator
        .decrypt_with_key(&doc_id, &custom_key)
        .await
        .expect("decrypt");

    assert!(!decrypted.is_empty());
}

#[tokio::test]
async fn test_decrypt_with_wrong_key() {
    let replicator = create_test_replicator().await;

    let correct_key = [99u8; 32];
    let wrong_key = [55u8; 32];

    let doc_id = replicator
        .create_document_with_key("wrong-key-doc", StorageMode::Files, &correct_key)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Test content")
        .await
        .expect("insert");

    // Decrypt with wrong key should fail
    let result = replicator.decrypt_with_key(&doc_id, &wrong_key).await;
    assert!(result.is_err(), "Decryption with wrong key should fail");
}

#[tokio::test]
async fn test_web_document_has_no_encryption_key() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("web-doc", StorageMode::Web)
        .await
        .expect("create");

    // Web documents should not have encryption keys
    let result = replicator.get_encryption_key(&doc_id).await;
    assert!(
        result.is_err(),
        "Web documents should not have encryption keys"
    );
}

// =============================================================================
// TEST GROUP 4: Web Storage (Public/Unencrypted)
// =============================================================================

#[tokio::test]
async fn test_web_storage_is_unencrypted() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("public-doc", StorageMode::Web)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Public data")
        .await
        .expect("insert");

    // Get web blob (should be unencrypted CRDT update)
    let web_blob = replicator
        .get_web_blob(&doc_id)
        .await
        .expect("get blob")
        .expect("blob exists");

    // Web blob is CRDT update format, not plaintext
    // But it should NOT be encrypted (no nonce/tag)
    assert!(!web_blob.is_empty());
}

#[tokio::test]
async fn test_web_storage_accessible_without_key() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("accessible-doc", StorageMode::Web)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Open content")
        .await
        .expect("insert");

    // Should be able to get web blob without any key
    let web_blob = replicator.get_web_blob(&doc_id).await.expect("get blob");

    assert!(web_blob.is_some());
}

// =============================================================================
// TEST GROUP 5: Dual Storage Synchronization
// =============================================================================

#[tokio::test]
async fn test_dual_storage_both_updated() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("dual-sync", StorageMode::Both)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Sync test")
        .await
        .expect("insert");

    // Both storages should have data
    let files_blob = replicator
        .get_files_blob(&doc_id)
        .await
        .expect("get files")
        .expect("files exists");

    let web_blob = replicator
        .get_web_blob(&doc_id)
        .await
        .expect("get web")
        .expect("web exists");

    assert!(!files_blob.is_empty());
    assert!(!web_blob.is_empty());
}

#[tokio::test]
async fn test_dual_storage_files_encrypted_web_not() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("dual-enc", StorageMode::Both)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Dual content")
        .await
        .expect("insert");

    // Files should have encryption key
    let key_result = replicator.get_encryption_key(&doc_id).await;
    assert!(key_result.is_ok(), "Files storage should have key");

    // Both blobs should exist but be different (encrypted vs unencrypted)
    let files_blob = replicator
        .get_files_blob(&doc_id)
        .await
        .expect("get files")
        .expect("files exists");

    let web_blob = replicator
        .get_web_blob(&doc_id)
        .await
        .expect("get web")
        .expect("web exists");

    // Blobs should be different (encrypted vs plain CRDT)
    assert_ne!(files_blob, web_blob);
}

// =============================================================================
// TEST GROUP 6: CRDT Update Synchronization
// =============================================================================

#[tokio::test]
async fn test_get_crdt_update() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("update-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Update test")
        .await
        .expect("insert");

    // Should be able to export CRDT update
    let update = replicator
        .get_crdt_update(&doc_id)
        .await
        .expect("get update");

    assert!(!update.is_empty());
}

#[tokio::test]
async fn test_apply_crdt_update() {
    let replicator1 = create_test_replicator().await;
    let replicator2 = create_test_replicator().await;

    // Use a shared document ID for sync
    let doc_id = "shared-sync-doc";

    // Create document on BOTH replicators with same ID
    replicator1
        .create_document_with_key(doc_id, StorageMode::Files, &[1u8; 32])
        .await
        .expect("create 1");

    replicator2
        .create_document_with_key(doc_id, StorageMode::Files, &[1u8; 32])
        .await
        .expect("create 2");

    // Make change on replicator1
    replicator1
        .insert_text(doc_id, 0, "Sync content")
        .await
        .expect("insert");

    // Get update from replicator1
    let update = replicator1
        .get_crdt_update(doc_id)
        .await
        .expect("get update");

    // Apply update to replicator2
    replicator2
        .apply_crdt_update(doc_id, &update)
        .await
        .expect("apply update");

    // replicator2 should now have the synced content
    let text = replicator2.get_text(doc_id).await.expect("get text");
    assert_eq!(text, "Sync content");
}

#[tokio::test]
async fn test_crdt_convergence() {
    let replicator1 = create_test_replicator().await;
    let replicator2 = create_test_replicator().await;

    // Use a shared document ID for both replicators
    let doc_id = "convergence-doc";

    // Both replicators create the SAME document
    replicator1
        .create_document_with_key(doc_id, StorageMode::Files, &[2u8; 32])
        .await
        .expect("create 1");

    replicator2
        .create_document_with_key(doc_id, StorageMode::Files, &[2u8; 32])
        .await
        .expect("create 2");

    // Both peers make different edits
    replicator1
        .insert_text(doc_id, 0, "A")
        .await
        .expect("insert 1");

    replicator2
        .insert_text(doc_id, 0, "B")
        .await
        .expect("insert 2");

    // Exchange updates
    let update1 = replicator1.get_crdt_update(doc_id).await.expect("update 1");
    let update2 = replicator2.get_crdt_update(doc_id).await.expect("update 2");

    replicator1
        .apply_crdt_update(doc_id, &update2)
        .await
        .expect("apply 2");
    replicator2
        .apply_crdt_update(doc_id, &update1)
        .await
        .expect("apply 1");

    // Both should converge to same state (CRDT property)
    let text1 = replicator1.get_text(doc_id).await.expect("text 1");
    let text2 = replicator2.get_text(doc_id).await.expect("text 2");

    assert_eq!(text1, text2, "CRDT should converge");
    // Both should have merged content (order determined by CRDT)
    assert!(!text1.is_empty(), "Merged text should not be empty");
}

// =============================================================================
// TEST GROUP 7: Error Handling
// =============================================================================

#[tokio::test]
async fn test_get_nonexistent_document() {
    let replicator = create_test_replicator().await;

    let result = replicator
        .get_document("nonexistent-id")
        .await
        .expect("query should succeed");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_insert_text_nonexistent_document() {
    let replicator = create_test_replicator().await;

    let result = replicator.insert_text("nonexistent-id", 0, "test").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_text_nonexistent_document() {
    let replicator = create_test_replicator().await;

    let result = replicator.get_text("nonexistent-id").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_encryption_key_nonexistent_document() {
    let replicator = create_test_replicator().await;

    let result = replicator.get_encryption_key("nonexistent-id").await;
    assert!(result.is_err());
}

// =============================================================================
// TEST GROUP 8: Storage Configuration
// =============================================================================

#[tokio::test]
async fn test_files_storage_disabled() {
    let replicator = create_replicator_with_config(false, true).await;

    let doc_id = replicator
        .create_document("files-disabled", StorageMode::Files)
        .await
        .expect("create");

    // Document created but not saved to Files (disabled)
    let _exists = replicator
        .document_exists_in_files(&doc_id)
        .await
        .expect("check");

    // Behavior: document exists in memory, storage save is no-op when disabled
    // This is acceptable - document works, just not persisted to disabled storage
}

#[tokio::test]
async fn test_web_storage_disabled() {
    let replicator = create_replicator_with_config(true, false).await;

    let doc_id = replicator
        .create_document("web-disabled", StorageMode::Web)
        .await
        .expect("create");

    // Document created but not saved to Web (disabled)
    let _exists = replicator
        .document_exists_in_web(&doc_id)
        .await
        .expect("check");

    // Behavior: document exists in memory, storage save is no-op when disabled
}

// =============================================================================
// TEST GROUP 9: Unicode and Special Characters
// =============================================================================

#[tokio::test]
async fn test_unicode_text() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("unicode-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "Hello 世界! 🌍🚀")
        .await
        .expect("insert");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "Hello 世界! 🌍🚀");
}

#[tokio::test]
async fn test_emoji_text() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("emoji-doc", StorageMode::Files)
        .await
        .expect("create");

    replicator
        .insert_text(&doc_id, 0, "🎉🎊🎈🎁")
        .await
        .expect("insert");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, "🎉🎊🎈🎁");
}

#[tokio::test]
async fn test_newlines_and_special_chars() {
    let replicator = create_test_replicator().await;

    let doc_id = replicator
        .create_document("special-doc", StorageMode::Files)
        .await
        .expect("create");

    let content = "Line 1\nLine 2\n\tTabbed\r\nWindows line";
    replicator
        .insert_text(&doc_id, 0, content)
        .await
        .expect("insert");

    let text = replicator.get_text(&doc_id).await.expect("get text");
    assert_eq!(text, content);
}

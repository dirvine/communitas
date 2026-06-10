// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Error Handling Tests
//!
//! Tests to ensure errors are properly propagated and we don't silently
//! lose data when filesystem or deserialization errors occur.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::crdt_manager::{CrdtError, CrdtManager};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use yrs::{Doc, ReadTxn, Transact};

/// Test that apply_update propagates non-DocumentNotFound errors
#[tokio::test]
async fn test_apply_update_propagates_read_errors() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    let doc_id = "channel:test:metadata";

    // Create a document
    let doc = Doc::new();
    CrdtManager::set_map_value(&doc, "name", "Test").expect("set name");
    manager
        .save_document(doc_id, "channel", "test", &doc)
        .await
        .expect("save document");

    // Make the file unreadable (simulate permission error)
    let entity_dir = temp_dir.path().join("crdt/channel");
    let hex_doc_id = hex::encode(doc_id.as_bytes());
    let yrs_path = entity_dir.join(format!("{}.yrs", hex_doc_id));

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&yrs_path).expect("get metadata").permissions();
        perms.set_mode(0o000); // No read permissions
        fs::set_permissions(&yrs_path, perms).expect("set permissions");

        // Create an update
        let update_doc = Doc::new();
        CrdtManager::set_map_value(&update_doc, "name", "Updated").expect("set name");
        let update_bytes = update_doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Apply should fail with FileSystem error, NOT create new doc
        let result = manager.apply_update(doc_id, &update_bytes).await;

        // Restore permissions for cleanup
        let mut perms = fs::metadata(&yrs_path).expect("get metadata").permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&yrs_path, perms).expect("restore permissions");

        assert!(result.is_err(), "Should propagate read error");
        assert!(
            matches!(result, Err(CrdtError::FileSystem(_))),
            "Should be FileSystem error, not silent success"
        );
    }
}

/// Test that apply_update creates new doc ONLY for DocumentNotFound
#[tokio::test]
async fn test_apply_update_creates_doc_only_for_not_found() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    let doc_id = "channel:newdoc:metadata";

    // Document doesn't exist yet
    let update_doc = Doc::new();
    CrdtManager::set_map_value(&update_doc, "name", "New").expect("set name");
    let update_bytes = update_doc
        .transact()
        .encode_state_as_update_v1(&yrs::StateVector::default());

    // Should create new doc since it doesn't exist
    let result = manager.apply_update(doc_id, &update_bytes).await;
    assert!(result.is_ok(), "Should succeed for non-existent doc");

    // Verify it was created
    let loaded = manager
        .load_document(doc_id)
        .await
        .expect("load created doc");
    let name: String = CrdtManager::get_map_value(&loaded, "name")
        .expect("get value")
        .expect("name exists");
    assert_eq!(name, "New");
}

/// Test that merge_updates propagates errors (not just apply_update)
#[tokio::test]
async fn test_merge_updates_propagates_errors() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    let doc_id = "channel:test:metadata";

    // Create a document
    let doc = Doc::new();
    CrdtManager::set_map_value(&doc, "name", "Original").expect("set name");
    manager
        .save_document(doc_id, "channel", "test", &doc)
        .await
        .expect("save document");

    // Corrupt the metadata file
    let entity_dir = temp_dir.path().join("crdt/channel");
    let hex_doc_id = hex::encode(doc_id.as_bytes());
    let meta_path = entity_dir.join(format!("{}.meta", hex_doc_id));

    fs::write(&meta_path, "INVALID JSON{{{").expect("corrupt metadata");

    // Create updates
    let update1 = Doc::new();
    CrdtManager::set_map_value(&update1, "field1", "value1").expect("set");
    let bytes1 = update1
        .transact()
        .encode_state_as_update_v1(&yrs::StateVector::default());

    // Merge should fail due to corrupted metadata, not silently create new doc
    let result = manager.merge_updates(doc_id, vec![bytes1]).await;

    assert!(result.is_err(), "Should propagate deserialization error");
}

/// Test that we don't silently overwrite good data with empty doc
#[tokio::test]
async fn test_no_silent_data_loss_on_transient_error() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    let doc_id = "channel:important:metadata";

    // Create document with important data
    let doc = Doc::new();
    CrdtManager::set_map_value(&doc, "important_data", "DO NOT LOSE THIS").expect("set");
    manager
        .save_document(doc_id, "channel", "important", &doc)
        .await
        .expect("save document");

    // Simulate transient error by making directory unreadable
    #[cfg(unix)]
    {
        let entity_dir = temp_dir.path().join("crdt/channel");
        let mut perms = fs::metadata(&entity_dir)
            .expect("get metadata")
            .permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&entity_dir, perms).expect("set permissions");

        // Create an update
        let update_doc = Doc::new();
        CrdtManager::set_map_value(&update_doc, "new_field", "value").expect("set");
        let update_bytes = update_doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Apply should FAIL, not silently create new doc
        let result = manager.apply_update(doc_id, &update_bytes).await;

        // Restore permissions
        let mut perms = fs::metadata(&entity_dir)
            .expect("get metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&entity_dir, perms).expect("restore permissions");

        assert!(result.is_err(), "Should fail, not silently lose data");

        // Verify original data still intact after error
        let loaded = manager.load_document(doc_id).await.expect("load original");
        let data: String = CrdtManager::get_map_value(&loaded, "important_data")
            .expect("get value")
            .expect("data exists");
        assert_eq!(data, "DO NOT LOSE THIS", "Original data must be preserved");
    }
}

/// Test update size limit prevents DOS before attempting decode
#[tokio::test]
async fn test_update_size_limit_before_decode() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    let doc_id = "channel:test:metadata";

    // Create a huge "update" (11MB of garbage)
    let huge_update = vec![0u8; 11 * 1024 * 1024];

    // Should reject before even trying to decode
    let result = manager.apply_update(doc_id, &huge_update).await;

    assert!(result.is_err());
    assert!(
        matches!(result, Err(CrdtError::Encoding(_))),
        "Should reject oversized update"
    );
}

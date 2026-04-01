// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Document Lifecycle Tests
//!
//! Tests for document creation, concurrent editing, merging, and removal
//! with tombstone support. These tests verify that CRDT operations work
//! correctly across simulated peer scenarios.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::crdt::documents::{ChannelDocument, CrdtDocument};

use communitas_core::crdt_manager::{CrdtError, CrdtManager};
use tempfile::tempdir;
use yrs::updates::decoder::Decode;
use yrs::{Doc, Map, ReadTxn, Transact, Update};

/// Helper to encode a document's state as an update
fn encode_state(doc: &Doc) -> Vec<u8> {
    doc.transact()
        .encode_state_as_update_v1(&yrs::StateVector::default())
}

/// Helper to apply an update to a document
fn apply_update_to_doc(doc: &Doc, update_bytes: &[u8]) -> Result<(), String> {
    let update =
        Update::decode_v1(update_bytes).map_err(|e| format!("Failed to decode update: {}", e))?;
    let mut txn = doc.transact_mut();
    txn.apply_update(update);
    Ok(())
}

/// Test basic document creation and persistence
#[tokio::test]
async fn test_create_and_load_document() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create a channel document
    let channel = ChannelDocument {
        id: "channel-123".to_string(),
        pubkey_hex: "test-channel-main-room".to_string(),
        org_id: "org-456".to_string(),
        name: "Test Channel".to_string(),
        description: Some("A test channel for CRDT operations".to_string()),
        created_by: "creator-789".to_string(),
        created_at: 1234567890,
        private_disk_id: "disk-111".to_string(),
        public_disk_id: "disk-222".to_string(),
        website_root: None,
    };

    // Create and save the document
    let doc = ChannelDocument::create_document(&channel.id).expect("Failed to create document");
    channel
        .update_document(&doc)
        .expect("Failed to update document");

    manager
        .save_document(
            &format!("channel:{}:metadata", channel.id),
            "channel",
            &channel.id,
            &doc,
        )
        .await
        .expect("Failed to save document");

    // Load the document
    let loaded_doc = manager
        .load_document(&format!("channel:{}:metadata", channel.id))
        .await
        .expect("Failed to load document");

    // Verify the loaded data
    let loaded_channel =
        ChannelDocument::from_document(&loaded_doc).expect("Failed to parse document");

    assert_eq!(loaded_channel.id, channel.id);
    assert_eq!(loaded_channel.name, channel.name);
    assert_eq!(loaded_channel.org_id, channel.org_id);
    assert_eq!(loaded_channel.description, channel.description);
}

/// Test concurrent edits from two simulated peers
#[tokio::test]
async fn test_concurrent_edits_merge_correctly() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create initial document (Peer A)
    let channel = ChannelDocument {
        id: "channel-concurrent".to_string(),
        pubkey_hex: "test-concurrent-edit".to_string(),
        org_id: "org-123".to_string(),
        name: "Original Name".to_string(),
        description: Some("Original description".to_string()),
        created_by: "peer-a".to_string(),
        created_at: 1000,
        private_disk_id: "disk-a".to_string(),
        public_disk_id: "disk-b".to_string(),
        website_root: None,
    };

    let doc_id = format!("channel:{}:metadata", channel.id);

    // Peer A creates and saves initial document
    let doc_peer_a = ChannelDocument::create_document(&channel.id).expect("Create doc");
    channel.update_document(&doc_peer_a).expect("Update doc");

    manager
        .save_document(&doc_id, "channel", &channel.id, &doc_peer_a)
        .await
        .expect("Save initial document");

    // Peer B loads the document
    let doc_peer_b = manager
        .load_document(&doc_id)
        .await
        .expect("Peer B load document");

    // Peer A makes an edit (changes name)
    {
        let root = doc_peer_a.get_or_insert_map("root");
        let mut txn = doc_peer_a.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "name", "Name by Peer A");
    }

    // Peer B makes a different edit (changes description)
    {
        let root = doc_peer_b.get_or_insert_map("root");
        let mut txn = doc_peer_b.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "description", "Description by Peer B");
    }

    // Simulate sync: exchange updates
    let update_a = encode_state(&doc_peer_a);
    let update_b = encode_state(&doc_peer_b);

    // Peer A applies Peer B's update
    apply_update_to_doc(&doc_peer_a, &update_b).expect("Peer A apply update");

    // Peer B applies Peer A's update
    apply_update_to_doc(&doc_peer_b, &update_a).expect("Peer B apply update");

    // Both peers should now have the same state
    let channel_a = ChannelDocument::from_document(&doc_peer_a).expect("Parse A");
    let channel_b = ChannelDocument::from_document(&doc_peer_b).expect("Parse B");

    // Both should have Peer A's name (Last-Write-Wins)
    assert_eq!(channel_a.name, "Name by Peer A");
    assert_eq!(channel_b.name, "Name by Peer A");

    // Both should have Peer B's description (Last-Write-Wins)
    assert_eq!(
        channel_a.description,
        Some("Description by Peer B".to_string())
    );
    assert_eq!(
        channel_b.description,
        Some("Description by Peer B".to_string())
    );

    // Save final merged state
    manager
        .save_document(&doc_id, "channel", &channel.id, &doc_peer_a)
        .await
        .expect("Save merged document");
}

/// Test three-way concurrent merge scenario
#[tokio::test]
async fn test_three_way_concurrent_merge() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create initial document
    let channel = ChannelDocument {
        id: "channel-three-way".to_string(),
        pubkey_hex: "test-three-way".to_string(),
        org_id: "org-999".to_string(),
        name: "Initial".to_string(),
        description: Some("Initial".to_string()),
        created_by: "creator".to_string(),
        created_at: 1000,
        private_disk_id: "disk-1".to_string(),
        public_disk_id: "disk-2".to_string(),
        website_root: Some("https://initial.com".to_string()),
    };

    let doc_id = format!("channel:{}:metadata", channel.id);

    // Create initial document and save
    let doc_initial = ChannelDocument::create_document(&channel.id).expect("Create doc");
    channel.update_document(&doc_initial).expect("Update doc");
    manager
        .save_document(&doc_id, "channel", &channel.id, &doc_initial)
        .await
        .expect("Save initial");

    // Three peers load the document
    let doc_peer_a = manager.load_document(&doc_id).await.expect("Load A");
    let doc_peer_b = manager.load_document(&doc_id).await.expect("Load B");
    let doc_peer_c = manager.load_document(&doc_id).await.expect("Load C");

    // Peer A edits name
    {
        let root = doc_peer_a.get_or_insert_map("root");
        let mut txn = doc_peer_a.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "name", "Edited by A");
    }

    // Peer B edits description
    {
        let root = doc_peer_b.get_or_insert_map("root");
        let mut txn = doc_peer_b.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "description", "Edited by B");
    }

    // Peer C edits website_root
    {
        let root = doc_peer_c.get_or_insert_map("root");
        let mut txn = doc_peer_c.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "website_root", "https://peer-c.com");
    }

    // Simulate gossip sync: everyone applies everyone else's updates
    let update_a = encode_state(&doc_peer_a);
    let update_b = encode_state(&doc_peer_b);
    let update_c = encode_state(&doc_peer_c);

    // A applies B and C
    apply_update_to_doc(&doc_peer_a, &update_b).expect("A apply B");
    apply_update_to_doc(&doc_peer_a, &update_c).expect("A apply C");

    // B applies A and C
    apply_update_to_doc(&doc_peer_b, &update_a).expect("B apply A");
    apply_update_to_doc(&doc_peer_b, &update_c).expect("B apply C");

    // C applies A and B
    apply_update_to_doc(&doc_peer_c, &update_a).expect("C apply A");
    apply_update_to_doc(&doc_peer_c, &update_b).expect("C apply B");

    // All peers should converge to the same state
    let channel_a = ChannelDocument::from_document(&doc_peer_a).expect("Parse A");
    let channel_b = ChannelDocument::from_document(&doc_peer_b).expect("Parse B");
    let channel_c = ChannelDocument::from_document(&doc_peer_c).expect("Parse C");

    assert_eq!(channel_a.name, channel_b.name);
    assert_eq!(channel_b.name, channel_c.name);
    assert_eq!(channel_a.description, channel_b.description);
    assert_eq!(channel_b.description, channel_c.description);
    assert_eq!(channel_a.website_root, channel_b.website_root);
    assert_eq!(channel_b.website_root, channel_c.website_root);

    // Verify expected values (LWW semantics)
    assert_eq!(channel_a.name, "Edited by A");
    assert_eq!(channel_a.description, Some("Edited by B".to_string()));
    assert_eq!(
        channel_a.website_root,
        Some("https://peer-c.com".to_string())
    );
}

/// Test document deletion with tombstone
#[tokio::test]
async fn test_document_deletion_with_tombstone() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create a document
    let channel = ChannelDocument {
        id: "channel-to-delete".to_string(),
        pubkey_hex: "test-delete".to_string(),
        org_id: "org-123".to_string(),
        name: "To Be Deleted".to_string(),
        description: None,
        created_by: "creator".to_string(),
        created_at: 1000,
        private_disk_id: "disk-1".to_string(),
        public_disk_id: "disk-2".to_string(),
        website_root: None,
    };

    let doc_id = format!("channel:{}:metadata", channel.id);

    let doc = ChannelDocument::create_document(&channel.id).expect("Create doc");
    channel.update_document(&doc).expect("Update doc");

    manager
        .save_document(&doc_id, "channel", &channel.id, &doc)
        .await
        .expect("Save document");

    // Mark document as deleted with tombstone
    {
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "deleted", true);
        metadata_map.insert(&mut txn, "deleted_at", 2000i64);
        metadata_map.insert(&mut txn, "deleted_by", "deleter-id");
    }

    // Save tombstoned document
    manager
        .save_document(&doc_id, "channel", &channel.id, &doc)
        .await
        .expect("Save tombstoned document");

    // Load and verify tombstone
    let loaded_doc = manager.load_document(&doc_id).await.expect("Load doc");

    let root = loaded_doc.get_or_insert_map("root");
    let txn = loaded_doc.transact();
    let metadata = root.get(&txn, "metadata").unwrap();
    let metadata_map = yrs::MapRef::try_from(metadata).unwrap();

    let deleted = metadata_map
        .get(&txn, "deleted")
        .and_then(|v| bool::try_from(v).ok())
        .unwrap_or(false);
    assert!(deleted, "Document should be marked as deleted");

    let deleted_at = metadata_map
        .get(&txn, "deleted_at")
        .and_then(|v| i64::try_from(v).ok());
    assert_eq!(deleted_at, Some(2000));
}

/// Test that tombstone propagates across peers
#[tokio::test]
async fn test_tombstone_propagation() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create initial document
    let channel = ChannelDocument {
        id: "channel-tombstone-prop".to_string(),
        pubkey_hex: "test-tombstone".to_string(),
        org_id: "org-123".to_string(),
        name: "Test Channel".to_string(),
        description: None,
        created_by: "creator".to_string(),
        created_at: 1000,
        private_disk_id: "disk-1".to_string(),
        public_disk_id: "disk-2".to_string(),
        website_root: None,
    };

    let doc_id = format!("channel:{}:metadata", channel.id);

    // Peer A creates document
    let doc_peer_a = ChannelDocument::create_document(&channel.id).expect("Create doc");
    channel.update_document(&doc_peer_a).expect("Update doc");
    manager
        .save_document(&doc_id, "channel", &channel.id, &doc_peer_a)
        .await
        .expect("Save initial");

    // Peer B loads document
    let doc_peer_b = manager.load_document(&doc_id).await.expect("Load B");

    // Peer A marks document as deleted
    {
        let root = doc_peer_a.get_or_insert_map("root");
        let mut txn = doc_peer_a.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "deleted", true);
        metadata_map.insert(&mut txn, "deleted_at", 5000i64);
    }

    // Sync: Peer B applies Peer A's update
    let update_a = encode_state(&doc_peer_a);
    apply_update_to_doc(&doc_peer_b, &update_a).expect("B apply A's tombstone");

    // Verify Peer B sees the tombstone
    let root = doc_peer_b.get_or_insert_map("root");
    let txn = doc_peer_b.transact();
    let metadata = root.get(&txn, "metadata").unwrap();
    let metadata_map = yrs::MapRef::try_from(metadata).unwrap();

    let deleted = metadata_map
        .get(&txn, "deleted")
        .and_then(|v| bool::try_from(v).ok())
        .unwrap_or(false);
    assert!(deleted, "Peer B should see tombstone from Peer A");
}

/// Test offline-online scenario with queued updates
#[tokio::test]
async fn test_offline_online_sync() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create initial document
    let channel = ChannelDocument {
        id: "channel-offline".to_string(),
        pubkey_hex: "test-offline".to_string(),
        org_id: "org-123".to_string(),
        name: "Initial".to_string(),
        description: Some("Initial".to_string()),
        created_by: "creator".to_string(),
        created_at: 1000,
        private_disk_id: "disk-1".to_string(),
        public_disk_id: "disk-2".to_string(),
        website_root: None,
    };

    let doc_id = format!("channel:{}:metadata", channel.id);

    let doc = ChannelDocument::create_document(&channel.id).expect("Create doc");
    channel.update_document(&doc).expect("Update doc");
    manager
        .save_document(&doc_id, "channel", &channel.id, &doc)
        .await
        .expect("Save initial");

    // Peer A goes offline and makes multiple edits
    let doc_peer_a = manager.load_document(&doc_id).await.expect("Load A");

    // Edit 1 while offline
    {
        let root = doc_peer_a.get_or_insert_map("root");
        let mut txn = doc_peer_a.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "name", "Offline Edit 1");
    }

    // Edit 2 while offline
    {
        let root = doc_peer_a.get_or_insert_map("root");
        let mut txn = doc_peer_a.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "description", "Offline Edit 2");
    }

    // Peer B makes an edit while A is offline
    let doc_peer_b = manager.load_document(&doc_id).await.expect("Load B");
    {
        let root = doc_peer_b.get_or_insert_map("root");
        let mut txn = doc_peer_b.transact_mut();
        let metadata = root.get(&txn, "metadata").unwrap();
        let metadata_map = yrs::MapRef::try_from(metadata).unwrap();
        metadata_map.insert(&mut txn, "website_root", "https://peer-b.com");
    }

    // Peer A comes back online and syncs
    let update_a = encode_state(&doc_peer_a);
    let update_b = encode_state(&doc_peer_b);

    apply_update_to_doc(&doc_peer_a, &update_b).expect("A apply B");
    apply_update_to_doc(&doc_peer_b, &update_a).expect("B apply A");

    // Both should converge
    let channel_a = ChannelDocument::from_document(&doc_peer_a).expect("Parse A");
    let channel_b = ChannelDocument::from_document(&doc_peer_b).expect("Parse B");

    assert_eq!(channel_a.name, channel_b.name);
    assert_eq!(channel_a.description, channel_b.description);
    assert_eq!(channel_a.website_root, channel_b.website_root);

    // All edits should be present
    assert_eq!(channel_a.name, "Offline Edit 1");
    assert_eq!(channel_a.description, Some("Offline Edit 2".to_string()));
    assert_eq!(
        channel_a.website_root,
        Some("https://peer-b.com".to_string())
    );
}

/// Test that document size limits are enforced
#[tokio::test]
async fn test_document_size_limit() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create a document with a very large description
    let large_desc = "x".repeat(11 * 1024 * 1024); // 11 MB (exceeds 10MB limit)

    let channel = ChannelDocument {
        id: "channel-large".to_string(),
        pubkey_hex: "test-large".to_string(),
        org_id: "org-123".to_string(),
        name: "Test".to_string(),
        description: Some(large_desc),
        created_by: "creator".to_string(),
        created_at: 1000,
        private_disk_id: "disk-1".to_string(),
        public_disk_id: "disk-2".to_string(),
        website_root: None,
    };

    let doc_id = format!("channel:{}:metadata", channel.id);
    let doc = ChannelDocument::create_document(&channel.id).expect("Create doc");
    channel.update_document(&doc).expect("Update doc");

    // Should fail due to size limit
    let result = manager
        .save_document(&doc_id, "channel", &channel.id, &doc)
        .await;

    assert!(
        matches!(result, Err(CrdtError::Encoding(_))),
        "Should reject document exceeding size limit"
    );
}

/// Test that same doc_id in different entity types doesn't collide
#[tokio::test]
async fn test_entity_type_isolation() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = CrdtManager::new(temp_dir.path())
        .await
        .expect("Failed to create manager");

    // Create two documents with same suffix but different entity types
    let doc_channel = Doc::new();
    {
        let root = doc_channel.get_or_insert_map("root");
        let mut txn = doc_channel.transact_mut();
        root.insert(&mut txn, "type", "channel_data");
        root.insert(&mut txn, "value", "Channel Value");
    }

    let doc_org = Doc::new();
    {
        let root = doc_org.get_or_insert_map("root");
        let mut txn = doc_org.transact_mut();
        root.insert(&mut txn, "type", "organization_data");
        root.insert(&mut txn, "value", "Organization Value");
    }

    // Save both with entity_type in doc_id format: "entity_type:entity_id:suffix"
    manager
        .save_document("channel:123:metadata", "channel", "123", &doc_channel)
        .await
        .expect("Save channel document");

    manager
        .save_document("organization:123:metadata", "organization", "123", &doc_org)
        .await
        .expect("Save organization document");

    // Load and verify they remain separate
    let loaded_channel = manager
        .load_document("channel:123:metadata")
        .await
        .expect("Load channel document");

    let loaded_org = manager
        .load_document("organization:123:metadata")
        .await
        .expect("Load organization document");

    // Verify channel document
    {
        let root = loaded_channel.get_or_insert_map("root");
        let txn = loaded_channel.transact();
        let type_val = root
            .get(&txn, "type")
            .and_then(|v| String::try_from(v).ok())
            .expect("Channel type exists");
        let value_val = root
            .get(&txn, "value")
            .and_then(|v| String::try_from(v).ok())
            .expect("Channel value exists");

        assert_eq!(type_val, "channel_data");
        assert_eq!(value_val, "Channel Value");
    }

    // Verify organization document
    {
        let root = loaded_org.get_or_insert_map("root");
        let txn = loaded_org.transact();
        let type_val = root
            .get(&txn, "type")
            .and_then(|v| String::try_from(v).ok())
            .expect("Organization type exists");
        let value_val = root
            .get(&txn, "value")
            .and_then(|v| String::try_from(v).ok())
            .expect("Organization value exists");

        assert_eq!(type_val, "organization_data");
        assert_eq!(value_val, "Organization Value");
    }

    // Verify they are stored in different directories
    let channel_list = manager
        .list_documents("channel")
        .await
        .expect("List channel docs");
    let org_list = manager
        .list_documents("organization")
        .await
        .expect("List org docs");

    assert_eq!(channel_list.len(), 1);
    assert_eq!(org_list.len(), 1);
    assert!(channel_list.contains(&"channel:123:metadata".to_string()));
    assert!(org_list.contains(&"organization:123:metadata".to_string()));
}

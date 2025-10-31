// Copyright (c) 2025 Saorsa Labs Limited
//
// Test CRDT persistence and multi-instance synchronization

use communitas_core::legacy_crdt::EntityType;
use std::time::Duration;
use tokio::time::sleep;
use yrs::{Map, Transact};

/// Test that data written to CRDT storage persists across bridge restarts
#[tokio::test]
async fn test_crdt_persistence_across_restart() {
    // Clean up any existing test data
    let test_storage_dir = "./test-bridge-data-persistence";
    if std::path::Path::new(test_storage_dir).exists() {
        std::fs::remove_dir_all(test_storage_dir).ok();
    }

    // Save channel ID for verification in phase 2
    let saved_channel_id: String;

    // Phase 1: Create data in first instance
    {
        let core = communitas_core::CoreContext::initialize(
            "ocean-forest-mountain-river".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            communitas_core::types::DeviceType::Desktop,
            test_storage_dir.into(),
        )
        .await
        .expect("Failed to initialize core");

        // Create a channel
        let channel = core
            .entity_service
            .create_entity(
                "Test Channel".to_string(),
                EntityType::Channel,
                Some("Test channel description".to_string()),
                "test-user".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create channel");

        println!("Created channel: {} ({})", channel.name, channel.id);
        saved_channel_id = channel.id.clone();

        // Create a website
        let website_doc = yrs::Doc::new();
        let root = website_doc.get_or_insert_map("website");
        {
            let mut txn = website_doc.transact_mut();
            root.insert(&mut txn, "entity_id", saved_channel_id.clone());
            root.insert(&mut txn, "html", "<h1>Test Website</h1>".to_string());
            root.insert(&mut txn, "css", "body { color: blue; }".to_string());
            root.insert(&mut txn, "js", "console.log('test');".to_string());
            root.insert(&mut txn, "hash", "test-hash".to_string());
            root.insert(&mut txn, "published_at", 1234567890i64);
            root.insert(&mut txn, "size_bytes", 100i64);
        } // Transaction explicitly drops here

        // Drop root reference before saving
        drop(root);

        core.crdt_manager
            .save_document(
                &format!("website:{}", saved_channel_id),
                "website",
                &saved_channel_id,
                &website_doc,
            )
            .await
            .expect("Failed to save website");

        println!("Created website for entity: {}", saved_channel_id);

        // Create a file
        let file_doc = yrs::Doc::new();
        let root = file_doc.get_or_insert_map("file");
        {
            let mut txn = file_doc.transact_mut();
            root.insert(&mut txn, "entity_id", saved_channel_id.clone());
            root.insert(&mut txn, "disk_type", "private".to_string());
            root.insert(&mut txn, "path", "/test/file.txt".to_string());
            root.insert(&mut txn, "content_base64", "SGVsbG8gV29ybGQ=".to_string());
            root.insert(&mut txn, "content_type", "text/plain".to_string());
            root.insert(&mut txn, "size_bytes", 11i64);
            root.insert(&mut txn, "uploaded_at", 1234567890i64);
            root.insert(&mut txn, "file_id", "test-file-id".to_string());
            root.insert(&mut txn, "encrypted", true);
        }

        // Drop root reference before saving
        drop(root);

        let path_hash = hex::encode(blake3::hash("/test/file.txt".as_bytes()).as_bytes());
        let doc_id = format!("{}:private:{}", saved_channel_id, path_hash);

        core.crdt_manager
            .save_document(&doc_id, "file", &saved_channel_id, &file_doc)
            .await
            .expect("Failed to save file");

        println!("Created file: /test/file.txt");

        // Instance goes out of scope - simulates shutdown
    }

    // Wait a moment to ensure everything is flushed
    sleep(Duration::from_millis(100)).await;

    // Phase 2: Load data in second instance (restart simulation)
    {
        let core = communitas_core::CoreContext::initialize(
            "ocean-forest-mountain-river".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            communitas_core::types::DeviceType::Desktop,
            test_storage_dir.into(),
        )
        .await
        .expect("Failed to reinitialize core");

        // Verify channel entity document exists by loading it directly
        let channel_doc = core
            .crdt_manager
            .load_document(&format!("entity:{}:metadata", saved_channel_id))
            .await
            .expect("Failed to load channel document after restart");

        let root = channel_doc.get_or_insert_map("metadata");
        let channel_name = {
            let txn = channel_doc.transact();
            root.get(&txn, "name")
                .and_then(|v| v.to_string(&txn).into())
                .expect("Channel name not found")
        };

        println!("Found channel after restart: {channel_name} ({saved_channel_id})");
        assert_eq!(channel_name, "Test Channel");

        // Verify website exists
        let website_doc = core
            .crdt_manager
            .load_document(&format!("website:{}", saved_channel_id))
            .await
            .expect("Failed to load website after restart");

        let root = website_doc.get_or_insert_map("website");
        let html = {
            let txn = website_doc.transact();
            root.get(&txn, "html")
                .and_then(|v| v.to_string(&txn).into())
                .expect("HTML not found")
        };

        println!("Found website after restart: {}", html);
        assert_eq!(html, "<h1>Test Website</h1>");

        // Verify file exists
        let path_hash = hex::encode(blake3::hash("/test/file.txt".as_bytes()).as_bytes());
        let doc_id = format!("{}:private:{}", saved_channel_id, path_hash);

        let file_doc = core
            .crdt_manager
            .load_document(&doc_id)
            .await
            .expect("Failed to load file after restart");

        let root = file_doc.get_or_insert_map("file");
        let content = {
            let txn = file_doc.transact();
            root.get(&txn, "content_base64")
                .and_then(|v| v.to_string(&txn).into())
                .expect("Content not found")
        };

        println!("Found file after restart: {}", content);
        assert_eq!(content, "SGVsbG8gV29ybGQ=");
    }

    // Cleanup
    std::fs::remove_dir_all(test_storage_dir).ok();

    println!("✅ CRDT persistence test PASSED - all data survived restart!");
}

/// Test multi-instance CRDT synchronization
///
/// This test creates two independent bridge instances with separate storage
/// and verifies that CRDT documents can be transferred between them.
#[tokio::test]
async fn test_multi_instance_crdt_sync() {
    // Clean up any existing test data
    let instance1_dir = "./test-bridge-instance1";
    let instance2_dir = "./test-bridge-instance2";

    for dir in [instance1_dir, instance2_dir] {
        if std::path::Path::new(dir).exists() {
            std::fs::remove_dir_all(dir).ok();
        }
    }

    // Save channel IDs for verification
    let channel1_id: String;
    let channel2_id: String;

    // Phase 1: Instance 1 creates a channel
    {
        let core1 = communitas_core::CoreContext::initialize(
            "ocean-forest-mountain-river".to_string(),
            "Instance One".to_string(),
            "Device 1".to_string(),
            communitas_core::types::DeviceType::Desktop,
            instance1_dir.into(),
        )
        .await
        .expect("Failed to initialize instance 1");

        let channel = core1
            .entity_service
            .create_entity(
                "Shared Channel".to_string(),
                EntityType::Channel,
                Some("Channel created by instance 1".to_string()),
                "instance-one".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create channel in instance 1");

        println!("Instance 1 created channel: {}", channel.name);
        channel1_id = channel.id;
    }

    // Phase 2: Instance 2 creates a different channel
    {
        let core2 = communitas_core::CoreContext::initialize(
            "sunshine-rainbow-butterfly-breeze".to_string(),
            "Instance Two".to_string(),
            "Device 2".to_string(),
            communitas_core::types::DeviceType::Desktop,
            instance2_dir.into(),
        )
        .await
        .expect("Failed to initialize instance 2");

        let channel2 = core2
            .entity_service
            .create_entity(
                "Another Channel".to_string(),
                EntityType::Channel,
                Some("Channel created by instance 2".to_string()),
                "instance-two".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create channel in instance 2");

        println!("Instance 2 created channel: {}", channel2.name);
        channel2_id = channel2.id;
    }

    // Phase 3: Simulate sync by copying CRDT files between instances
    // In a real system, this would happen via gossip P2P protocol
    println!("\n📡 Simulating CRDT sync by copying files...");

    // The CRDT manager stores files in: {storage_dir}/crdt.db/crdt/{entity_type}/
    let instance1_entity_dir = format!("{}/crdt.db/crdt/entity", instance1_dir);
    let instance2_entity_dir = format!("{}/crdt.db/crdt/entity", instance2_dir);

    // Copy all files from instance 1 to instance 2
    if let Ok(entries) = std::fs::read_dir(&instance1_entity_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let source = entry.path();
            let dest = format!("{}/{}", instance2_entity_dir, file_name.to_string_lossy());

            // Copy if file doesn't exist (simulating merge)
            if !std::path::Path::new(&dest).exists() {
                std::fs::copy(&source, &dest).ok();
                println!("  Synced: {} → instance2", file_name.to_string_lossy());
            }
        }
    }

    // Copy all files from instance 2 to instance 1
    if let Ok(entries) = std::fs::read_dir(&instance2_entity_dir) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let source = entry.path();
            let dest = format!("{}/{}", instance1_entity_dir, file_name.to_string_lossy());

            if !std::path::Path::new(&dest).exists() {
                std::fs::copy(&source, &dest).ok();
                println!("  Synced: {} → instance1", file_name.to_string_lossy());
            }
        }
    }

    println!("  ✓ File sync complete (both instances now have all documents)");

    sleep(Duration::from_millis(100)).await;

    // Phase 4: Verify both instances can load both channels after sync
    {
        let core1 = communitas_core::CoreContext::initialize(
            "ocean-forest-mountain-river".to_string(),
            "Instance One".to_string(),
            "Device 1".to_string(),
            communitas_core::types::DeviceType::Desktop,
            instance1_dir.into(),
        )
        .await
        .expect("Failed to reinitialize instance 1");

        println!("\n✅ Instance 1 after sync:");

        // Verify instance 1 can load its own channel
        let channel1_doc = core1
            .crdt_manager
            .load_document(&format!("entity:{}:metadata", channel1_id))
            .await
            .expect("Instance 1 should still have its own channel");

        let root = channel1_doc.get_or_insert_map("metadata");
        let name1 = {
            let txn = channel1_doc.transact();
            root.get(&txn, "name")
                .and_then(|v| v.to_string(&txn).into())
                .expect("Name not found")
        };
        println!("  - {}", name1);

        // Verify instance 1 can load instance 2's channel (after sync)
        let channel2_doc = core1
            .crdt_manager
            .load_document(&format!("entity:{}:metadata", channel2_id))
            .await
            .expect("Instance 1 should have instance 2's channel after sync");

        let root = channel2_doc.get_or_insert_map("metadata");
        let name2 = {
            let txn = channel2_doc.transact();
            root.get(&txn, "name")
                .and_then(|v| v.to_string(&txn).into())
                .expect("Name not found")
        };
        println!("  - {}", name2);

        assert_eq!(name1, "Shared Channel");
        assert_eq!(name2, "Another Channel");
    }

    {
        let core2 = communitas_core::CoreContext::initialize(
            "sunshine-rainbow-butterfly-breeze".to_string(),
            "Instance Two".to_string(),
            "Device 2".to_string(),
            communitas_core::types::DeviceType::Desktop,
            instance2_dir.into(),
        )
        .await
        .expect("Failed to reinitialize instance 2");

        println!("\n✅ Instance 2 after sync:");

        // Verify instance 2 can load instance 1's channel (after sync)
        let channel1_doc = core2
            .crdt_manager
            .load_document(&format!("entity:{}:metadata", channel1_id))
            .await
            .expect("Instance 2 should have instance 1's channel after sync");

        let root = channel1_doc.get_or_insert_map("metadata");
        let name1 = {
            let txn = channel1_doc.transact();
            root.get(&txn, "name")
                .and_then(|v| v.to_string(&txn).into())
                .expect("Name not found")
        };
        println!("  - {}", name1);

        // Verify instance 2 can load its own channel
        let channel2_doc = core2
            .crdt_manager
            .load_document(&format!("entity:{}:metadata", channel2_id))
            .await
            .expect("Instance 2 should still have its own channel");

        let root = channel2_doc.get_or_insert_map("metadata");
        let name2 = {
            let txn = channel2_doc.transact();
            root.get(&txn, "name")
                .and_then(|v| v.to_string(&txn).into())
                .expect("Name not found")
        };
        println!("  - {}", name2);

        assert_eq!(name1, "Shared Channel");
        assert_eq!(name2, "Another Channel");
    }

    // Cleanup
    std::fs::remove_dir_all(instance1_dir).ok();
    std::fs::remove_dir_all(instance2_dir).ok();

    println!("\n✅ Multi-instance CRDT sync test PASSED!");
}

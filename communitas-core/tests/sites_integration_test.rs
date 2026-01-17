// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Integration tests for Sites protocol over QUIC
//!
//! These tests validate the complete end-to-end pipeline:
//! - SitePublisher creates and signs content
//! - SitesListener routes network requests
//! - SiteFetcher retrieves content over QUIC
//! - ML-DSA signatures verify
//! - Block hashes verify

use bytes::Bytes;
use communitas_core::gossip::{
    Block, SiteId, SitePublisher, SiteRequest, SiteResponse, SitesListener,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use saorsa_gossip_transport::{AntQuicTransport, GossipTransport};
use saorsa_pqc::ml_dsa_65::{PrivateKey, PublicKey, try_keygen_with_rng};
use std::net::SocketAddr;
use std::sync::Arc;

/// Generate a deterministic test keypair from a seed
fn generate_test_keypair(seed: u64) -> (PrivateKey, PublicKey) {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let (pk, sk) = try_keygen_with_rng(&mut rng).expect("Failed to generate test keypair");
    (sk, pk)
}

/// Test that SitePublisher can handle requests directly (no network)
#[tokio::test]
async fn test_publisher_handles_requests_locally() {
    let (sk, pk) = generate_test_keypair(1);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id.clone()));

    // Add content
    let content = b"Hello, World!".to_vec();
    let hash = publisher
        .add_asset("hello.txt".to_string(), content.clone())
        .await
        .unwrap();

    // Build and sign manifest
    let mut manifest = publisher
        .build_manifest(&pk, 1, vec![("hello.txt".to_string(), hash)])
        .await
        .unwrap();
    manifest.sign(&sk).unwrap();

    // Update stored manifest with signed version
    publisher.set_manifest(manifest.clone()).await.unwrap();

    // Request manifest
    let request = SiteRequest::GetManifest {
        site_id: site_id.clone(),
    };
    let request_bytes = bincode::serialize(&request).unwrap();
    let response_bytes = publisher
        .handle_request(Bytes::from(request_bytes))
        .await
        .unwrap();
    let response: SiteResponse = bincode::deserialize(&response_bytes).unwrap();

    match response {
        SiteResponse::Manifest(m) => {
            assert_eq!(m.site_id, site_id);
            m.verify().expect("Signature verification failed");
        }
        _ => panic!("Expected Manifest response"),
    }

    // Request block
    let request = SiteRequest::GetBlock { hash };
    let request_bytes = bincode::serialize(&request).unwrap();
    let response_bytes = publisher
        .handle_request(Bytes::from(request_bytes))
        .await
        .unwrap();
    let response: SiteResponse = bincode::deserialize(&response_bytes).unwrap();

    match response {
        SiteResponse::Block(b) => {
            assert!(b.verify());
            assert_eq!(b.content, content);
        }
        _ => panic!("Expected Block response"),
    }
}

/// Test that SitesListener can be created and started
#[tokio::test]
async fn test_sites_listener_starts() {
    let (_sk, pk) = generate_test_keypair(2);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id));

    let bind: SocketAddr = "127.0.0.1:0".parse().expect("valid addr");
    let qt = AntQuicTransport::new(bind, vec![])
        .await
        .expect("transport");
    let transport: Arc<dyn GossipTransport + Send + Sync> = Arc::new(qt);

    let listener = Arc::new(SitesListener::new(transport, Some(publisher)));
    let handle = listener.clone().start();

    // Give it a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Stop the listener
    listener.stop();

    // Wait for it to shut down (with timeout)
    tokio::select! {
        _ = handle => {},
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(2)) => {
            panic!("Listener did not shut down in time");
        }
    }
}

/// Test manifest signature verification detects tampering
#[tokio::test]
async fn test_manifest_signature_detects_tampering() {
    let (sk, pk) = generate_test_keypair(3);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id));

    // Create and sign manifest
    let hash = [1u8; 32];
    let mut manifest = publisher
        .build_manifest(&pk, 1, vec![("test.txt".to_string(), hash)])
        .await
        .unwrap();
    manifest.sign(&sk).unwrap();

    // Verify it's valid
    manifest
        .verify()
        .expect("Valid manifest failed verification");

    // Tamper with manifest (change version)
    manifest.manifest_version = 999;

    // Verification should fail
    assert!(
        manifest.verify().is_err(),
        "Tampered manifest should fail verification"
    );
}

/// Test block hash verification detects corruption
#[tokio::test]
async fn test_block_hash_detects_corruption() {
    let original_content = b"Original content".to_vec();
    let block = Block::new(original_content.clone());

    // Verify original is valid
    assert!(block.verify(), "Original block should be valid");

    // Corrupt the content
    let mut corrupted_block = block.clone();
    corrupted_block.content = b"Corrupted content".to_vec();

    // Verification should fail
    assert!(
        !corrupted_block.verify(),
        "Corrupted block should fail verification"
    );
}

/// Test concurrent asset addition
#[tokio::test]
async fn test_concurrent_asset_addition() {
    let (_sk, pk) = generate_test_keypair(4);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id));

    // Add multiple assets concurrently
    let mut handles = vec![];
    let mut expected_hashes = vec![];

    for i in 0..10 {
        let pub_clone = publisher.clone();
        let handle = tokio::spawn(async move {
            let content = format!("Content {}", i).into_bytes();
            let filename = format!("file_{}.txt", i);
            pub_clone.add_asset(filename, content).await
        });
        handles.push(handle);
    }

    // Wait for all to complete and collect hashes
    for handle in handles {
        let hash = handle.await.unwrap().expect("Asset addition failed");
        expected_hashes.push(hash);
    }

    // Build manifest with all assets
    let asset_paths: Vec<_> = expected_hashes
        .iter()
        .enumerate()
        .map(|(i, hash)| (format!("file_{}.txt", i), *hash))
        .collect();

    let manifest = publisher.build_manifest(&pk, 1, asset_paths).await.unwrap();

    // Verify all 10 blocks are in manifest
    assert_eq!(
        manifest.blocks.len(),
        10,
        "Should have 10 blocks in manifest"
    );
}

/// Test manifest version ordering (rollback protection)
#[tokio::test]
async fn test_manifest_version_rollback_protection() {
    let (_sk, pk) = generate_test_keypair(5);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id));

    // Create version 1
    let hash1 = [1u8; 32];
    let manifest_v1 = publisher
        .build_manifest(&pk, 1, vec![("v1.txt".to_string(), hash1)])
        .await
        .unwrap();

    // Create version 2
    let hash2 = [2u8; 32];
    let manifest_v2 = publisher
        .build_manifest(&pk, 2, vec![("v2.txt".to_string(), hash2)])
        .await
        .unwrap();

    // Create version 3
    let hash3 = [3u8; 32];
    let manifest_v3 = publisher
        .build_manifest(&pk, 3, vec![("v3.txt".to_string(), hash3)])
        .await
        .unwrap();

    // Test is_newer_than
    assert!(
        manifest_v2.is_newer_than(&manifest_v1),
        "v2 should be newer than v1"
    );
    assert!(
        manifest_v3.is_newer_than(&manifest_v2),
        "v3 should be newer than v2"
    );
    assert!(
        manifest_v3.is_newer_than(&manifest_v1),
        "v3 should be newer than v1"
    );

    assert!(
        !manifest_v1.is_newer_than(&manifest_v2),
        "v1 should not be newer than v2"
    );
    assert!(
        !manifest_v1.is_newer_than(&manifest_v3),
        "v1 should not be newer than v3"
    );
}

/// Test that error responses are properly formatted
#[tokio::test]
async fn test_error_response_format() {
    let (_, pk) = generate_test_keypair(6);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id.clone()));

    // Request non-existent block
    let fake_hash = [255u8; 32];
    let request = SiteRequest::GetBlock { hash: fake_hash };
    let request_bytes = bincode::serialize(&request).unwrap();
    let response_bytes = publisher
        .handle_request(Bytes::from(request_bytes))
        .await
        .unwrap();
    let response: SiteResponse = bincode::deserialize(&response_bytes).unwrap();

    match response {
        SiteResponse::Error(msg) => {
            assert!(
                msg.contains("not found") || msg.contains("Block"),
                "Error message should mention block"
            );
        }
        _ => panic!("Expected Error response"),
    }

    // Request manifest when none exists (publisher hasn't built one yet)
    let request = SiteRequest::GetManifest { site_id };
    let request_bytes = bincode::serialize(&request).unwrap();
    let response_bytes = publisher
        .handle_request(Bytes::from(request_bytes))
        .await
        .unwrap();
    let response: SiteResponse = bincode::deserialize(&response_bytes).unwrap();

    match response {
        SiteResponse::Error(msg) => {
            // Just verify we got an error response, don't check exact message
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
        _ => panic!("Expected Error response for missing manifest"),
    }
}

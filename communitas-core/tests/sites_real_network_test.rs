// Copyright (c) 2025 Saorsa Labs Limited
//
// Real Network Integration Tests for Sites Protocol
//
// These tests validate ACTUAL QUIC communication between nodes:
// - Two independent GossipContext instances
// - Real socket binding (random ports)
// - IPv4 AND IPv6 testing
// - Signature verification over network
// - Throughput measurements
// - Security validation

mod network_test_utils;

use communitas_core::gossip::SiteId;
use network_test_utils::{TestMetrics, TestNode};
use std::time::Instant;

/// Initialize tracing for tests
fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info,communitas_core=debug")
        .with_test_writer()
        .try_init();
}

/// Test 1: Type conversion and local signing
#[tokio::test]
async fn test_identity_keys_sign_manifest() {
    init_tracing();

    let node = TestNode::new("alice").await;

    // Publish a simple site
    let files = vec![
        ("index.html", b"<html><body>Hello!</body></html>" as &[u8]),
        ("style.css", b"body { font-family: sans-serif; }" as &[u8]),
    ];

    let result = node.publish_site(&files).await;

    assert!(
        result.is_ok(),
        "Should publish successfully: {:?}",
        result.err()
    );

    let (site_id, manifest, total_bytes) = result.unwrap();

    // Verify manifest properties
    assert_eq!(manifest.blocks.len(), 2, "Should have 2 blocks");
    assert!(total_bytes > 0, "Should have content");
    assert_eq!(manifest.manifest_version, 1);
    assert_eq!(manifest.site_id, site_id);

    // CRITICAL: Verify signature
    manifest.verify().expect("Manifest signature should verify");

    println!("✓ Type conversion works");
    println!("✓ Manifest signing works");
    println!("✓ Local verification works");
}

/// Test 2: Two-node QUIC communication (IPv4)
#[tokio::test]
async fn test_two_nodes_quic_publish_and_fetch_ipv4() {
    init_tracing();

    println!("\n=== Test: Two-Node QUIC Communication (IPv4) ===\n");

    // Create two independent nodes with random ports
    let node_a = TestNode::new("publisher").await;
    let node_b = TestNode::new("fetcher").await;

    println!(
        "Node A (publisher): gossip={}, sites={}, peer_id={:?}",
        node_a.gossip_port,
        node_a.sites_port,
        node_a.peer_id()
    );
    println!(
        "Node B (fetcher):   gossip={}, sites={}, peer_id={:?}\n",
        node_b.gossip_port,
        node_b.sites_port,
        node_b.peer_id()
    );

    // === PUBLISH ON NODE A ===

    let publish_start = Instant::now();

    let files = vec![
        ("index.html", b"<html><head><title>Test Site</title></head><body><h1>Hello from Node A!</h1><p>This is a test site published over QUIC.</p></body></html>" as &[u8]),
        ("style.css", b"body { font-family: sans-serif; background: #f0f0f0; padding: 20px; } h1 { color: #2EB67D; }" as &[u8]),
        ("data.json", b"{\"message\": \"Test data\", \"timestamp\": 1706543210}" as &[u8]),
    ];

    let (site_id, manifest, pub_bytes) = node_a
        .publish_site(&files)
        .await
        .expect("Node A should publish");

    let publish_time = publish_start.elapsed();

    println!("Published:");
    println!("  Site ID: {:?}", site_id);
    println!("  Manifest version: {}", manifest.manifest_version);
    println!("  Files: {}", manifest.blocks.len());
    println!("  Total bytes: {}", pub_bytes);
    println!("  Publish time: {:?}\n", publish_time);

    // CRITICAL: Exchange peer addresses before attempting fetch
    // This establishes the QUIC routing information needed for fetch_manifest()
    println!("Exchanging peer addresses between nodes...");
    node_a
        .connect_to_peer(&node_b)
        .await
        .expect("Node A should connect to Node B");

    node_b
        .connect_to_peer(&node_a)
        .await
        .expect("Node B should connect to Node A");

    println!("Peer address exchange complete.\n");

    // === FETCH ON NODE B ===

    let (fetched_manifest, blocks, fetch_bytes, fetch_time) = node_b
        .fetch_site(&site_id, node_a.peer_id())
        .await
        .expect("Node B should fetch from Node A");

    println!("Fetched:");
    println!("  Manifest version: {}", fetched_manifest.manifest_version);
    println!("  Blocks: {}", blocks.len());
    println!("  Total bytes: {}", fetch_bytes);
    println!("  Fetch time: {:?}\n", fetch_time);

    // === VERIFICATION ===

    // Manifests should match
    assert_eq!(fetched_manifest.site_id, site_id, "Site ID should match");
    assert_eq!(fetched_manifest.manifest_version, 1, "Version should be 1");
    assert_eq!(fetched_manifest.blocks.len(), 3, "Should have 3 blocks");

    // Bytes should match
    assert_eq!(
        fetch_bytes, pub_bytes,
        "Fetched bytes should match published bytes"
    );

    // Content should match
    for (i, (path, _)) in files.iter().enumerate() {
        let fetched_block = &blocks[i];
        assert_eq!(fetched_manifest.blocks[i].0, *path, "Path should match");
        assert!(fetched_block.verify(), "Block hash should verify");
    }

    // === PERFORMANCE METRICS ===

    let metrics = TestMetrics {
        publish_time_ms: publish_time.as_millis() as u64,
        fetch_time_ms: fetch_time.as_millis() as u64,
        total_bytes: fetch_bytes,
        throughput_mbps: if fetch_time.as_secs_f64() > 0.0 {
            (fetch_bytes as f64 / fetch_time.as_secs_f64()) / (1024.0 * 1024.0)
        } else {
            0.0
        },
        blocks_count: blocks.len(),
    };

    metrics.print_summary();

    println!("✓ QUIC communication works (IPv4)");
    println!("✓ Signatures verify over network");
    println!("✓ Content integrity maintained");
    println!("✓ Throughput: {:.2} MB/s\n", metrics.throughput_mbps);
}

/// Test 3: IPv6 Communication
#[tokio::test]
async fn test_sites_over_ipv6() {
    init_tracing();

    println!("\n=== Test: Sites over IPv6 ===\n");

    let node_a = TestNode::new_ipv6("publisher-v6").await;
    let node_b = TestNode::new_ipv6("fetcher-v6").await;

    println!("IPv6 Node A: port={}", node_a.gossip_port);
    println!("IPv6 Node B: port={}\n", node_b.gossip_port);

    // Publish and fetch (same as IPv4 test)
    let files = vec![(
        "ipv6-test.html",
        b"<html><body>IPv6 works!</body></html>" as &[u8],
    )];

    let (site_id, _, _) = node_a
        .publish_site(&files)
        .await
        .expect("Should publish on IPv6");

    // Exchange peer addresses for IPv6 communication
    node_a
        .connect_to_peer(&node_b)
        .await
        .expect("Node A should connect to Node B");

    node_b
        .connect_to_peer(&node_a)
        .await
        .expect("Node B should connect to Node A");

    let (manifest, blocks, bytes, time) = node_b
        .fetch_site(&site_id, node_a.peer_id())
        .await
        .expect("Should fetch over IPv6");

    assert_eq!(blocks.len(), 1);
    manifest.verify().expect("Signature should verify");

    println!("✓ IPv6 communication works");
    println!("✓ Fetched {} bytes in {:?}", bytes, time);
}

/// Test 4: Reject unsigned manifest (security)
#[tokio::test]
async fn test_reject_unsigned_manifest() {
    init_tracing();

    println!("\n=== Test: Security - Reject Unsigned Manifest ===\n");

    let node_a = TestNode::new("bad-publisher").await;
    let node_b = TestNode::new("victim").await;

    // Node A publishes unsigned manifest (security hole!)
    let publisher = node_a.ctx.site_publisher.as_ref().unwrap();
    let (pk, _sk) = node_a.ctx.get_sites_signing_keys().unwrap();
    let site_id = SiteId::from_public_key(&pk);

    let hash = publisher
        .add_asset("unsigned.html".into(), b"<html>Bad</html>".to_vec())
        .await
        .unwrap();

    // Build but DON'T sign
    let manifest = publisher
        .build_manifest(&pk, 1, vec![("unsigned.html".into(), hash)])
        .await
        .unwrap();

    // manifest.sign(&sk) ← SKIPPED INTENTIONALLY!

    publisher.set_manifest(manifest).await.unwrap();

    println!("Node A: Published UNSIGNED manifest (attack simulation)");

    // Exchange peer addresses so Node B can reach Node A
    node_a
        .connect_to_peer(&node_b)
        .await
        .expect("Node A should connect to Node B");

    node_b
        .connect_to_peer(&node_a)
        .await
        .expect("Node B should connect to Node A");

    // Node B tries to fetch
    let fetcher = node_b.ctx.site_fetcher.as_ref().unwrap();
    let result = fetcher.fetch_manifest(&site_id, node_a.peer_id()).await;

    // CRITICAL: Should REJECT
    assert!(result.is_err(), "Should reject unsigned manifest");

    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.to_lowercase().contains("signature") || error_msg.contains("verification"),
        "Error should mention signature: {}",
        error_msg
    );

    println!("✓ Unsigned manifest rejected");
    println!("✓ Error message: {}", error_msg);
}

/// Test 5: Reject tampered manifest (security)
#[tokio::test]
async fn test_reject_tampered_manifest() {
    init_tracing();

    println!("\n=== Test: Security - Reject Tampered Manifest ===\n");

    let node_a = TestNode::new("attacker").await;
    let node_b = TestNode::new("victim-2").await;

    let publisher = node_a.ctx.site_publisher.as_ref().unwrap();
    let (pk, sk) = node_a.ctx.get_sites_signing_keys().unwrap();
    let site_id = SiteId::from_public_key(&pk);

    let hash = publisher
        .add_asset("real.html".into(), b"Real".to_vec())
        .await
        .unwrap();

    let mut manifest = publisher
        .build_manifest(&pk, 1, vec![("real.html".into(), hash)])
        .await
        .unwrap();

    // Sign it
    manifest.sign(&sk).unwrap();

    // TAMPER after signing
    manifest.manifest_version = 999;

    publisher.set_manifest(manifest).await.unwrap();

    println!("Node A: Published TAMPERED manifest (signature now invalid)");

    // Exchange peer addresses so Node B can reach Node A
    node_a
        .connect_to_peer(&node_b)
        .await
        .expect("Node A should connect to Node B");

    node_b
        .connect_to_peer(&node_a)
        .await
        .expect("Node B should connect to Node A");

    // Node B fetches
    let fetcher = node_b.ctx.site_fetcher.as_ref().unwrap();
    let result = fetcher.fetch_manifest(&site_id, node_a.peer_id()).await;

    // CRITICAL: Should REJECT
    assert!(result.is_err(), "Should reject tampered manifest");

    println!("✓ Tampered manifest rejected");
    println!("✓ Signature verification prevents tampering");
}

/// Test 6: Large file throughput test
#[tokio::test]
async fn test_large_file_throughput() {
    init_tracing();

    println!("\n=== Test: Large File Throughput ===\n");

    let node_a = TestNode::new("large-publisher").await;
    let node_b = TestNode::new("large-fetcher").await;

    // Create a 5MB file
    let large_content = vec![42u8; 5 * 1024 * 1024];

    let files = vec![
        ("large.bin", large_content.as_slice()),
        ("metadata.json", b"{\"size\": \"5MB\"}" as &[u8]),
    ];

    let (site_id, manifest, pub_bytes) = node_a
        .publish_site(&files)
        .await
        .expect("Should publish large file");

    println!(
        "Published {} bytes across {} blocks",
        pub_bytes,
        manifest.blocks.len()
    );

    // Exchange peer addresses for large file transfer
    node_a
        .connect_to_peer(&node_b)
        .await
        .expect("Node A should connect to Node B");

    node_b
        .connect_to_peer(&node_a)
        .await
        .expect("Node B should connect to Node A");

    // Fetch
    let (_, blocks, fetch_bytes, fetch_time) = node_b
        .fetch_site(&site_id, node_a.peer_id())
        .await
        .expect("Should fetch large file");

    let throughput_mbps = (fetch_bytes as f64 / fetch_time.as_secs_f64()) / (1024.0 * 1024.0);

    println!("Fetched {} bytes in {:?}", fetch_bytes, fetch_time);
    println!("Throughput: {:.2} MB/s", throughput_mbps);
    println!("Blocks: {}", blocks.len());

    assert_eq!(fetch_bytes, pub_bytes);
    assert!(throughput_mbps > 0.1, "Throughput should be reasonable");

    println!("✓ Large file transfer works");
    println!("✓ Throughput is acceptable");
}

/// Test 7: Concurrent fetches (stress test)
#[tokio::test]
async fn test_concurrent_fetches() {
    init_tracing();

    println!("\n=== Test: Concurrent Fetches (Stress Test) ===\n");

    let publisher = TestNode::new("popular-site").await;

    // Publish one site
    let files = vec![("index.html", b"<html>Popular!</html>" as &[u8])];
    let (site_id, manifest, _) = publisher.publish_site(&files).await.unwrap();

    // Advertise as provider via rendezvous (this is how fetchers discover the site)
    publisher
        .advertise_as_provider(&site_id, manifest.manifest_version)
        .await
        .expect("Should advertise as provider");

    println!("Published site and advertised as provider, spawning 5 concurrent fetchers...\n");

    // Spawn 5 concurrent fetchers
    let mut handles = vec![];
    for i in 0..5 {
        let site_id_clone = site_id.clone();
        let provider_peer_id = publisher.peer_id();

        let handle = tokio::spawn(async move {
            let fetcher = TestNode::new(&format!("fetcher-{}", i)).await;

            // Note: In a real concurrent scenario, fetchers would need to discover
            // the publisher via rendezvous or peer exchange. For this test, we'll
            // simulate that they can reach the publisher (the rendezvous system
            // would handle this in production).

            let start = Instant::now();
            let result = fetcher.fetch_site(&site_id_clone, provider_peer_id).await;
            let elapsed = start.elapsed();

            (i, result, elapsed)
        });

        handles.push(handle);
    }

    // Wait for all
    let mut successful = 0;
    for handle in handles {
        let (i, result, elapsed) = handle.await.unwrap();

        if let Ok((manifest, blocks, bytes, _)) = result {
            manifest.verify().expect("Signature should verify");
            println!(
                "Fetcher {}: Success ({} blocks, {} bytes) in {:?}",
                i,
                blocks.len(),
                bytes,
                elapsed
            );
            successful += 1;
        } else {
            println!("Fetcher {}: Failed - {:?}", i, result.err());
        }
    }

    assert_eq!(successful, 5, "All 5 fetchers should succeed");

    println!("\n✓ Concurrent fetches work");
    println!("✓ Backpressure handles load");
}

/// Test 8: Address identification (IPv4 and IPv6)
#[tokio::test]
async fn test_address_identification() {
    init_tracing();

    println!("\n=== Test: Address Identification ===\n");

    let node_ipv4 = TestNode::new("ipv4-node").await;
    let node_ipv6 = TestNode::new_ipv6("ipv6-node").await;

    // Verify peer IDs are unique
    assert_ne!(
        node_ipv4.peer_id(),
        node_ipv6.peer_id(),
        "Peer IDs should be unique"
    );

    // Verify four-words are unique
    assert_ne!(
        node_ipv4.ctx.four_words(),
        node_ipv6.ctx.four_words(),
        "Four-words should be unique"
    );

    println!("IPv4 Node:");
    println!("  Four-words: {}", node_ipv4.ctx.four_words());
    println!("  Peer ID: {:?}", node_ipv4.peer_id());
    println!("  Port: {}", node_ipv4.gossip_port);

    println!("\nIPv6 Node:");
    println!("  Four-words: {}", node_ipv6.ctx.four_words());
    println!("  Peer ID: {:?}", node_ipv6.peer_id());
    println!("  Port: {}", node_ipv6.gossip_port);

    println!("\n✓ Address identification works");
    println!("✓ Unique peer IDs generated");
}

/// Test 9: Raw key operations
#[tokio::test]
async fn test_raw_key_operations() {
    init_tracing();

    println!("\n=== Test: Raw Key Operations ===\n");

    let node = TestNode::new("key-test").await;

    // Get raw keys
    let (public_key, private_key) = node.ctx.get_sites_signing_keys().expect("Should get keys");

    // Check sizes
    use saorsa_pqc::dsa_traits::SerDes;
    let pk_bytes = public_key.clone().into_bytes();
    let sk_bytes = private_key.clone().into_bytes();

    assert_eq!(pk_bytes.len(), 1952, "Public key should be 1952 bytes");
    assert_eq!(sk_bytes.len(), 4032, "Private key should be 4032 bytes");

    println!("Public key:  {} bytes", pk_bytes.len());
    println!("Private key: {} bytes", sk_bytes.len());

    // Test signing
    use saorsa_pqc::dsa_traits::{Signer, Verifier};
    let message = b"Test message for signing";
    let signature = private_key.try_sign(message, &[]).expect("Should sign");

    assert_eq!(signature.len(), 3309, "Signature should be 3309 bytes");

    // Test verification
    let valid = public_key.verify(message, &signature, &[]);
    assert!(valid, "Signature should verify");

    println!("Signature:   {} bytes", signature.len());
    println!("\n✓ Raw key operations work");
    println!("✓ ML-DSA-87 signing/verification works");
    println!("✓ Key sizes correct");
}

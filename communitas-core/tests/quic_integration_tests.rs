//! QUIC Integration Tests
//!
//! Tests real QUIC connection handling, handshake, reconnection,
//! SPKI pinning, and network resilience.

use communitas_core::test_harness::{LinkPolicy, TestHarness};
use std::time::Duration;

// ============================================================================
// QUIC Connection Handling Tests
// ============================================================================

#[tokio::test]
async fn test_handshake_success() {
    // GIVEN: 3 nodes in a mesh topology
    let harness = TestHarness::new(3).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");

    // WHEN: Nodes establish connections
    harness
        .wait_until_connected(3, Duration::from_secs(5))
        .await
        .expect("nodes should connect");

    // THEN: All nodes should be mutually connected
    let network = harness.network.read().await;
    for i in 0..3 {
        for j in (i + 1)..3 {
            assert!(
                network.are_connected(i, j).await,
                "Nodes {} and {} should be connected",
                i,
                j
            );
        }
    }

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Requires SPKI pinning implementation
async fn test_spki_pinning_reject() {
    // GIVEN: Node A with SPKI pinning enabled
    let harness = TestHarness::new(2).await.expect("harness creation failed");

    // Get node A and configure SPKI pinning
    let node_a = harness.get_node(0).await.expect("node A not found");
    // TODO: Call sync_set_quic_pinned_spki on node A with wrong SPKI

    // WHEN: Node B tries to connect (with different SPKI)
    harness.mesh().await.expect("mesh setup failed");

    // THEN: Connection should be rejected
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    let network = harness.network.read().await;
    // Connection should fail due to SPKI mismatch
    // TODO: Verify connection was rejected

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
async fn test_reconnect_after_drop() {
    // GIVEN: Two connected nodes
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(1, Duration::from_secs(5))
        .await
        .expect("initial connection failed");

    // WHEN: Network is partitioned
    harness.partition(&[0], &[1]).await.expect("partition failed");
    
    // Wait a bit to ensure disconnection
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let network = harness.network.read().await;
    assert!(!network.are_connected(0, 1).await, "Should be disconnected");
    drop(network);

    // AND: Network is healed
    harness.heal().await.expect("heal failed");

    // THEN: Nodes should reconnect
    harness
        .wait_until_connected(1, Duration::from_secs(10))
        .await
        .expect("reconnection failed");

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
async fn test_connection_with_latency() {
    // GIVEN: Two nodes with high latency link
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    
    // Set 200ms latency
    harness.set_latency(0, 1, 200).await;
    harness.mesh().await.expect("mesh setup failed");

    // WHEN: Nodes try to connect
    let start = std::time::Instant::now();
    harness
        .wait_until_connected(1, Duration::from_secs(10))
        .await
        .expect("connection should succeed despite latency");
    let elapsed = start.elapsed();

    // THEN: Connection should succeed but take longer
    assert!(
        elapsed > Duration::from_millis(200),
        "Connection should be affected by latency"
    );

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
async fn test_connection_with_packet_loss() {
    // GIVEN: Two nodes with 30% packet loss
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    
    // Set 30% packet loss
    harness.set_loss(0, 1, 0.3).await;
    harness.mesh().await.expect("mesh setup failed");

    // WHEN: Nodes try to connect
    harness
        .wait_until_connected(1, Duration::from_secs(15))
        .await
        .expect("connection should succeed despite packet loss");

    // THEN: Connection should eventually succeed (QUIC retries)
    let network = harness.network.read().await;
    assert!(
        network.are_connected(0, 1).await,
        "QUIC should handle packet loss with retries"
    );

    harness.cleanup().await.expect("cleanup failed");
}

// ============================================================================
// Multi-Node Scenarios
// ============================================================================

#[tokio::test]
async fn test_mesh_5_nodes() {
    // GIVEN: 5 nodes
    let harness = TestHarness::new(5).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");

    // WHEN: All nodes connect
    // 5 nodes = 10 connections (n*(n-1)/2)
    harness
        .wait_until_connected(10, Duration::from_secs(10))
        .await
        .expect("mesh should form");

    // THEN: Every pair should be connected
    let network = harness.network.read().await;
    for i in 0..5 {
        for j in (i + 1)..5 {
            assert!(
                network.are_connected(i, j).await,
                "Nodes {} and {} should be connected in mesh",
                i,
                j
            );
        }
    }

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
async fn test_star_topology() {
    // GIVEN: 5 nodes in star topology (node 0 is hub)
    let harness = TestHarness::new(5).await.expect("harness creation failed");
    harness.star(0).await.expect("star setup failed");

    // WHEN: Nodes connect
    // 4 connections (hub to each spoke)
    harness
        .wait_until_connected(4, Duration::from_secs(10))
        .await
        .expect("star should form");

    // THEN: Hub should be connected to all, spokes not to each other
    let network = harness.network.read().await;
    
    // Hub to spokes
    for i in 1..5 {
        assert!(
            network.are_connected(0, i).await,
            "Hub should be connected to spoke {}",
            i
        );
    }
    
    // Spokes not connected to each other
    assert!(
        !network.are_connected(1, 2).await,
        "Spokes should not be connected"
    );
    assert!(
        !network.are_connected(3, 4).await,
        "Spokes should not be connected"
    );

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
async fn test_line_topology() {
    // GIVEN: 4 nodes in line (0-1-2-3)
    let harness = TestHarness::new(4).await.expect("harness creation failed");
    harness.line().await.expect("line setup failed");

    // WHEN: Nodes connect
    // 3 connections in a line
    harness
        .wait_until_connected(3, Duration::from_secs(10))
        .await
        .expect("line should form");

    // THEN: Adjacent nodes connected, non-adjacent not
    let network = harness.network.read().await;
    
    assert!(network.are_connected(0, 1).await, "0-1 should be connected");
    assert!(network.are_connected(1, 2).await, "1-2 should be connected");
    assert!(network.are_connected(2, 3).await, "2-3 should be connected");
    
    assert!(
        !network.are_connected(0, 2).await,
        "0-2 should not be connected"
    );
    assert!(
        !network.are_connected(0, 3).await,
        "0-3 should not be connected"
    );

    harness.cleanup().await.expect("cleanup failed");
}

// ============================================================================
// Network Partition & Healing
// ============================================================================

#[tokio::test]
async fn test_partition_healing() {
    // GIVEN: 4 nodes in mesh
    let harness = TestHarness::new(4).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(6, Duration::from_secs(10))
        .await
        .expect("initial mesh failed");

    // WHEN: Network is partitioned into [0,1] and [2,3]
    harness.partition(&[0, 1], &[2, 3]).await.expect("partition failed");
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // THEN: Within-partition connections remain, cross-partition drop
    let network = harness.network.read().await;
    assert!(network.are_connected(0, 1).await, "Partition A internal");
    assert!(network.are_connected(2, 3).await, "Partition B internal");
    assert!(!network.are_connected(0, 2).await, "Cross-partition");
    assert!(!network.are_connected(1, 3).await, "Cross-partition");
    drop(network);

    // WHEN: Network heals
    harness.heal().await.expect("heal failed");
    
    // THEN: All connections should restore
    harness
        .wait_until_connected(6, Duration::from_secs(10))
        .await
        .expect("healing should restore mesh");

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
async fn test_cascading_failure() {
    // GIVEN: 5 nodes in line (0-1-2-3-4)
    let harness = TestHarness::new(5).await.expect("harness creation failed");
    harness.line().await.expect("line setup failed");
    
    harness
        .wait_until_connected(4, Duration::from_secs(10))
        .await
        .expect("line failed to form");

    // WHEN: Middle node (2) fails
    harness.partition(&[0, 1], &[2]).await.expect("partition 1 failed");
    harness.partition(&[3, 4], &[2]).await.expect("partition 2 failed");
    
    tokio::time::sleep(Duration::from_millis(500)).await;

    // THEN: Network splits into [0-1] and [3-4]
    let network = harness.network.read().await;
    assert!(network.are_connected(0, 1).await);
    assert!(network.are_connected(3, 4).await);
    assert!(!network.are_connected(1, 3).await, "Should be partitioned");

    harness.cleanup().await.expect("cleanup failed");
}

// ============================================================================
// Performance & Resilience
// ============================================================================

#[tokio::test]
#[ignore] // Slow test - run manually
async fn test_high_latency_high_loss() {
    // GIVEN: Two nodes with realistic bad network (300ms latency, 20% loss)
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    
    harness.set_latency(0, 1, 300).await;
    harness.set_loss(0, 1, 0.2).await;
    harness.mesh().await.expect("mesh setup failed");

    // WHEN: Nodes try to establish connection
    // THEN: Should eventually succeed (QUIC is designed for this)
    harness
        .wait_until_connected(1, Duration::from_secs(30))
        .await
        .expect("connection should succeed even in bad conditions");

    harness.cleanup().await.expect("cleanup failed");
}

#[tokio::test]
#[ignore] // Slow test - run manually
async fn test_flapping_connection() {
    // GIVEN: Two connected nodes
    let harness = TestHarness::new(2).await.expect("harness creation failed");
    harness.mesh().await.expect("mesh setup failed");
    
    harness
        .wait_until_connected(1, Duration::from_secs(5))
        .await
        .expect("initial connection failed");

    // WHEN: Connection flaps (disconnect/reconnect 5 times)
    for _ in 0..5 {
        harness.partition(&[0], &[1]).await.expect("partition failed");
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        harness.heal().await.expect("heal failed");
        harness
            .wait_until_connected(1, Duration::from_secs(5))
            .await
            .expect("reconnection failed");
    }

    // THEN: Connection should remain stable after flapping
    let network = harness.network.read().await;
    assert!(
        network.are_connected(0, 1).await,
        "Connection should survive flapping"
    );

    harness.cleanup().await.expect("cleanup failed");
}

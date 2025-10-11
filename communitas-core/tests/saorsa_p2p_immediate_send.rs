// saorsa-core P2P immediate send diagnostic test
//
// This test reproduces the immediate connection closure issue at the saorsa-core layer.
// Unlike the ant_quic_comprehensive tests, this tests the actual P2PNode wrapper that
// communitas uses.
//
// Issue: Connections close within microseconds after establishment, preventing immediate sends.
// Timeline: connect -> 13-35 microseconds -> connection closed -> "Endpoint closed" error

use anyhow::Result;
use saorsa_core::network::{NodeConfig, P2PNode};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::info;

/// Initialize Rustls CryptoProvider once for all tests
fn init_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Install ring crypto provider for tests
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

/// Helper: Create a P2PNode with OS-assigned ports
async fn create_test_node() -> Result<P2PNode> {
    init_crypto_provider();
    let config = NodeConfig {
        peer_id: None,
        listen_addrs: vec![
            "0.0.0.0:0".parse()?, // IPv4 on OS-assigned port
            "[::]:0".parse()?,    // IPv6 on OS-assigned port
        ],
        ..Default::default()
    };

    P2PNode::new(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create P2PNode: {}", e))
}

/// Test 1: Basic Connection Lifecycle
///
/// Validates that two nodes can connect and send messages.
/// Expected: PASS (basic saorsa-core functionality)
#[tokio::test]
async fn test_basic_connection_lifecycle() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    info!("=== Test 1: Basic Connection Lifecycle ===");

    // Create nodes
    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;

    let addrs1 = node1.listen_addrs().await;
    let addrs2 = node2.listen_addrs().await;

    info!("Node1 listening on: {:?}", addrs1);
    info!("Node2 listening on: {:?}", addrs2);

    // Connect
    let addr2 = addrs2.first().expect("Node2 should have address");
    let addr2_str = addr2.to_string();

    info!("Connecting node1 to node2 at {}", addr2_str);
    let peer2_id = node1.connect_peer(&addr2_str).await?;
    info!("✅ Connected: peer_id={}", peer2_id);

    // Wait for connection to stabilize
    sleep(Duration::from_millis(100)).await;

    // Verify connection active
    assert!(
        node1.is_connection_active(&peer2_id).await,
        "Connection should be active after 100ms"
    );

    // Send message
    let message = b"Hello from node1";
    node1
        .send_message(&peer2_id, "test", message.to_vec())
        .await?;
    info!("✅ Message sent successfully");

    info!("✅ Test 1 PASSED");
    Ok(())
}

/// Test 2: Immediate Send After Connect (CRITICAL)
///
/// Reproduces the exact issue - attempts to send immediately after connect with NO delay.
/// Expected: FAIL - reveals exact timing when connection becomes unusable
#[tokio::test]
async fn test_immediate_send_after_connect() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug,saorsa_core=debug")
        .with_test_writer()
        .try_init();

    info!("=== Test 2: Immediate Send After Connect ===");

    // Create nodes
    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;

    let addrs2 = node2.listen_addrs().await;
    let addr2_str = addrs2
        .first()
        .expect("Node2 should have address")
        .to_string();

    info!("Node1: {:?}", node1.listen_addrs().await);
    info!("Node2: {:?}", addrs2);

    // Connect with timing
    let connect_start = Instant::now();
    info!("Connecting...");
    let peer2_id = node1.connect_peer(&addr2_str).await?;
    let connect_duration = connect_start.elapsed();

    info!(
        "✅ Connected in {:?}, peer_id={}",
        connect_duration, peer2_id
    );

    // CRITICAL: Send IMMEDIATELY with NO delay
    let send_start = Instant::now();
    info!("Sending message IMMEDIATELY (no wait)...");

    let message = b"Immediate message";
    let send_result = node1
        .send_message(&peer2_id, "test", message.to_vec())
        .await;
    let send_duration = send_start.elapsed();

    match send_result {
        Ok(_) => {
            info!("✅ Immediate send succeeded in {:?}", send_duration);
            info!(
                "Total time from connect to send: {:?}",
                connect_start.elapsed()
            );
            info!("✅ Test 2 PASSED");
            Ok(())
        }
        Err(e) => {
            info!("❌ Immediate send failed after {:?}: {}", send_duration, e);
            info!("Connection closed before send could complete");
            info!(
                "Time from connect to failure: {:?}",
                connect_start.elapsed()
            );

            // Check if connection is still active
            let is_active = node1.is_connection_active(&peer2_id).await;
            info!("Connection active after failure: {}", is_active);

            Err(anyhow::anyhow!("Immediate send failed: {}", e))
        }
    }
}

/// Test 3: Send Timing Analysis
///
/// Measures success rate of sends at various delays after connect.
/// Identifies minimum safe delay required.
#[tokio::test]
async fn test_send_timing_analysis() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    info!("=== Test 3: Send Timing Analysis ===");

    // Test with various delays
    let delays_ms = vec![0, 1, 5, 10, 25, 50, 100];

    for delay_ms in delays_ms {
        info!("\n--- Testing with {} ms delay ---", delay_ms);

        // Create fresh nodes for each iteration
        let node1 = create_test_node().await?;
        let node2 = create_test_node().await?;

        let addrs2 = node2.listen_addrs().await;
        let addr2_str = addrs2
            .first()
            .expect("Node2 should have address")
            .to_string();

        // Connect
        let connect_start = Instant::now();
        let peer2_id = node1.connect_peer(&addr2_str).await?;
        let connect_duration = connect_start.elapsed();
        info!("Connected in {:?}", connect_duration);

        // Wait specified delay
        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms)).await;
        }

        // Try to send
        let send_start = Instant::now();
        let message = b"test message";
        let send_result = node1
            .send_message(&peer2_id, "test", message.to_vec())
            .await;
        let send_duration = send_start.elapsed();
        let total_duration = connect_start.elapsed();

        match send_result {
            Ok(_) => {
                info!("✅ SUCCESS with {}ms delay", delay_ms);
                info!("   Send time: {:?}", send_duration);
                info!("   Total time: {:?}", total_duration);
            }
            Err(e) => {
                info!("❌ FAILED with {}ms delay: {}", delay_ms, e);
                info!("   Send time: {:?}", send_duration);
                info!("   Total time: {:?}", total_duration);
            }
        }

        // Clean up
        drop(node1);
        drop(node2);
        sleep(Duration::from_millis(50)).await;
    }

    info!("\n✅ Test 3 completed - timing data collected");
    Ok(())
}

/// Test 4: Connection State Monitoring
///
/// Monitors connection state at each stage to understand when/why it closes.
#[tokio::test]
async fn test_connection_state_monitoring() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug,saorsa_core=debug")
        .with_test_writer()
        .try_init();

    info!("=== Test 4: Connection State Monitoring ===");

    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;

    let addrs2 = node2.listen_addrs().await;
    let addr2_str = addrs2
        .first()
        .expect("Node2 should have address")
        .to_string();

    info!("=== Before Connect ===");
    info!("Node1 addrs: {:?}", node1.listen_addrs().await);
    info!("Node2 addrs: {:?}", addrs2);

    // Connect and monitor state
    info!("\n=== Connecting ===");
    let connect_start = Instant::now();
    let peer2_id = node1.connect_peer(&addr2_str).await?;
    let connect_duration = connect_start.elapsed();

    info!(
        "\n=== Immediately After Connect ({:?}) ===",
        connect_duration
    );
    info!("Peer ID: {}", peer2_id);
    info!(
        "Is peer connected: {}",
        node1.is_peer_connected(&peer2_id).await
    );
    info!(
        "Is connection active: {}",
        node1.is_connection_active(&peer2_id).await
    );

    // Check after short delay
    sleep(Duration::from_micros(50)).await;
    info!("\n=== After 50 microseconds ===");
    info!(
        "Is peer connected: {}",
        node1.is_peer_connected(&peer2_id).await
    );
    info!(
        "Is connection active: {}",
        node1.is_connection_active(&peer2_id).await
    );

    // Attempt send
    info!("\n=== Attempting Send ===");
    let send_result = node1
        .send_message(&peer2_id, "test", b"test".to_vec())
        .await;

    info!("\n=== After Send Attempt ===");
    info!(
        "Send result: {:?}",
        send_result
            .as_ref()
            .map(|_| "OK")
            .map_err(|e| e.to_string())
    );
    info!(
        "Is peer connected: {}",
        node1.is_peer_connected(&peer2_id).await
    );
    info!(
        "Is connection active: {}",
        node1.is_connection_active(&peer2_id).await
    );

    // Final state after delay
    sleep(Duration::from_millis(100)).await;
    info!("\n=== Final State (after 100ms) ===");
    info!(
        "Is peer connected: {}",
        node1.is_peer_connected(&peer2_id).await
    );
    info!(
        "Is connection active: {}",
        node1.is_connection_active(&peer2_id).await
    );

    match send_result {
        Ok(_) => info!("✅ Send succeeded"),
        Err(e) => info!("❌ Send failed: {}", e),
    }

    info!("\n✅ Test 4 completed - state data collected");
    Ok(())
}

/// Test 5: Multiple Rapid Sends
///
/// Tests sending multiple messages rapidly to see when failure occurs.
#[tokio::test]
async fn test_multiple_rapid_sends() -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    info!("=== Test 5: Multiple Rapid Sends ===");

    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;

    let addrs2 = node2.listen_addrs().await;
    let addr2_str = addrs2
        .first()
        .expect("Node2 should have address")
        .to_string();

    // Connect
    info!("Connecting...");
    let peer2_id = node1.connect_peer(&addr2_str).await?;
    info!("✅ Connected: peer_id={}", peer2_id);

    // Try sending 10 messages immediately
    info!("\nSending 10 messages rapidly...");
    let mut success_count = 0;
    let mut first_failure: Option<(usize, String)> = None;

    for i in 0..10 {
        let message = format!("Message {}", i).into_bytes();
        let result = node1.send_message(&peer2_id, "test", message).await;

        match result {
            Ok(_) => {
                info!("✅ Message {} sent successfully", i);
                success_count += 1;
            }
            Err(e) => {
                info!("❌ Message {} failed: {}", i, e);
                if first_failure.is_none() {
                    first_failure = Some((i, e.to_string()));
                }
            }
        }

        // Small delay between sends
        sleep(Duration::from_micros(100)).await;
    }

    info!("\n=== Results ===");
    info!("Success count: {}/10", success_count);
    if let Some((msg_num, error)) = first_failure {
        info!("First failure: message {} with error: {}", msg_num, error);
    }

    info!("✅ Test 5 completed");
    Ok(())
}

/// Test Summary
#[tokio::test]
async fn test_run_diagnostics() -> Result<()> {
    println!("\n");
    println!("═══════════════════════════════════════════════════════════");
    println!("  SAORSA-CORE P2P IMMEDIATE SEND DIAGNOSTIC TESTS");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Purpose: Isolate immediate connection closure issue");
    println!();
    println!("Tests:");
    println!("  1. test_basic_connection_lifecycle");
    println!("  2. test_immediate_send_after_connect (CRITICAL - reproduces issue)");
    println!("  3. test_send_timing_analysis (identifies minimum safe delay)");
    println!("  4. test_connection_state_monitoring (state inspection)");
    println!("  5. test_multiple_rapid_sends (multiple message test)");
    println!();
    println!("Run all:");
    println!("  cargo test --package communitas-core --test saorsa_p2p_immediate_send");
    println!();
    println!("Run specific:");
    println!(
        "  cargo test --package communitas-core --test saorsa_p2p_immediate_send test_immediate_send_after_connect -- --nocapture"
    );
    println!("═══════════════════════════════════════════════════════════");
    println!();

    Ok(())
}

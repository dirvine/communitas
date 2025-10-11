// ANT-QUIC Comprehensive Test Suite
//
// This test suite systematically validates ant-quic's connection lifecycle,
// event handling, and all methods required for P2P messaging in communitas.
//
// Focus: Isolate immediate connection closure issue where connections
// close within microseconds of being established.

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// ant-quic imports
use ant_quic::{Config, Endpoint, PeerId};

// Helper trait for convenience methods on QuicP2PNode
trait QuicNodeExt {
    async fn send_to_peer(&self, peer_id: &PeerId, data: &[u8]) -> Result<()>;
    async fn receive(&self) -> Result<(PeerId, Vec<u8>)>;
    fn get_nat_endpoint(&self) -> Result<&Endpoint>;
}

// Test helper: Create configured ant-quic endpoint
async fn create_test_node() -> Result<Endpoint> {
    let config = Config::default_with_random_port()?;
    let endpoint = Endpoint::new_peer(config).await?;
    Ok(endpoint)
}

// Test helper: Wait for connection to stabilize
async fn wait_for_connection_stable(duration_ms: u64) {
    sleep(Duration::from_millis(duration_ms)).await;
}

// =============================================================================
// PHASE 1 CRITICAL TESTS - Immediate Connection Issues
// =============================================================================

/// Test 2.1.1: Single Connection Lifecycle
///
/// Validates that a connection can be established, used, and closed cleanly.
/// Expected: PASS (basic ant-quic functionality should work)
#[tokio::test]
async fn test_single_connection_lifecycle() -> Result<()> {
    println!("\n=== Test 2.1.1: Single Connection Lifecycle ===");

    // SETUP
    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;
    let addr2 = node2.local_addr()?;

    println!("Node 1 addr: {}", node1.local_addr()?);
    println!("Node 2 addr: {}", addr2);

    // CONNECT
    println!("Connecting node1 to node2...");
    let connection = node1.connect_to(&addr2).await?;
    println!("✅ Connection established");

    // Wait for connection to stabilize
    wait_for_connection_stable(100).await;

    // SEND MESSAGE
    println!("Opening bidirectional stream...");
    let (mut send, mut recv) = connection.open_bi().await?;

    let test_data = b"Hello from node1";
    println!("Sending {} bytes...", test_data.len());
    send.write_all(test_data).await?;
    send.finish()?;

    // RECEIVE MESSAGE
    println!("Receiving response...");
    let received = recv.read_to_end(1024).await?;
    println!("Received {} bytes", received.len());

    // CLOSE
    println!("Closing connection...");
    connection.close(0u32.into(), b"test complete");

    println!("✅ Test 2.1.1 PASSED");
    Ok(())
}

/// Test 2.1.3: Immediate Send After Connect
///
/// Critical test that reproduces the exact issue seen in communitas-core.
/// Attempts to send immediately after connection establishment with NO delay.
/// Expected: FAIL - reveals exact timing when connection becomes unusable
#[tokio::test]
async fn test_immediate_send_after_connect() -> Result<()> {
    println!("\n=== Test 2.1.3: Immediate Send After Connect ===");

    // SETUP
    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;
    let addr2 = node2.local_addr()?;

    println!("Node 1 addr: {}", node1.local_addr()?);
    println!("Node 2 addr: {}", addr2);

    // CONNECT
    use std::time::Instant;
    let connect_start = Instant::now();
    println!("Connecting node1 to node2...");
    let connection = node1.connect_to(&addr2).await?;
    let connect_duration = connect_start.elapsed();
    println!("✅ Connection established in {:?}", connect_duration);

    // CRITICAL: Send immediately with NO delay
    let send_start = Instant::now();
    println!("Opening stream IMMEDIATELY (no wait)...");

    let stream_result = connection.open_bi().await;
    let stream_duration = send_start.elapsed();

    match stream_result {
        Ok((mut send, _recv)) => {
            println!("✅ Stream opened in {:?}", stream_duration);

            let test_data = b"Immediate message";
            let write_start = Instant::now();
            let write_result = send.write_all(test_data).await;
            let write_duration = write_start.elapsed();

            match write_result {
                Ok(_) => {
                    println!("✅ Immediate send succeeded in {:?}", write_duration);
                    println!(
                        "Total time from connect to write: {:?}",
                        connect_start.elapsed()
                    );

                    send.finish()?;
                    println!("✅ Test 2.1.3 PASSED");
                    Ok(())
                }
                Err(e) => {
                    println!(
                        "❌ Immediate write failed after {:?}: {}",
                        write_duration, e
                    );
                    println!("Stream opened OK but write failed - connection closed?");
                    Err(e.into())
                }
            }
        }
        Err(e) => {
            println!("❌ Stream open failed after {:?}: {}", stream_duration, e);
            println!("Connection closed before stream could be opened");
            println!(
                "Time from connect to failure: {:?}",
                connect_start.elapsed()
            );
            Err(e.into())
        }
    }
}

/// Test 2.4.1: Endpoint Stays Open During Test
///
/// Validates that the QUIC endpoint remains open throughout test execution.
/// Expected: PASS (endpoint should not auto-close during active use)
#[tokio::test]
async fn test_endpoint_stays_open() -> Result<()> {
    println!("\n=== Test 2.4.1: Endpoint Stays Open ===");

    let endpoint = create_test_node().await?;

    println!("Endpoint created: {}", endpoint.local_addr()?);
    println!("Checking endpoint status...");

    // Check endpoint is open initially
    if let Some(reason) = endpoint.close_reason() {
        println!("❌ Endpoint already closed: {:?}", reason);
        return Err(anyhow::anyhow!("Endpoint closed at start"));
    }
    println!("✅ Endpoint open at start");

    // Wait and check again
    sleep(Duration::from_millis(100)).await;

    if let Some(reason) = endpoint.close_reason() {
        println!("❌ Endpoint closed after 100ms: {:?}", reason);
        return Err(anyhow::anyhow!("Endpoint auto-closed"));
    }
    println!("✅ Endpoint still open after 100ms");

    // Wait longer
    sleep(Duration::from_millis(500)).await;

    if let Some(reason) = endpoint.close_reason() {
        println!("❌ Endpoint closed after 600ms: {:?}", reason);
        return Err(anyhow::anyhow!("Endpoint auto-closed"));
    }
    println!("✅ Endpoint still open after 600ms");

    println!("✅ Test 2.4.1 PASSED");
    Ok(())
}

/// Test 2.4.2: Endpoint Closure Timing
///
/// Diagnostic test to understand when and why endpoints close during connection attempts.
/// Monitors endpoint state before, during, and after connection establishment.
/// Expected: Reveal exact timing of endpoint closure
#[tokio::test]
async fn test_endpoint_closure_timing() -> Result<()> {
    println!("\n=== Test 2.4.2: Endpoint Closure Timing ===");

    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;
    let addr2 = node2.local_addr()?;

    // Check before connect
    println!("=== Before Connect ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node1 local addr: {:?}", node1.local_addr());
    println!("Node2 endpoint closed: {:?}", node2.close_reason());

    // Connect
    use std::time::Instant;
    let connect_start = Instant::now();
    println!("\n=== Connecting ===");
    let connection = node1.connect_to(&addr2).await?;
    let connect_duration = connect_start.elapsed();

    // Check immediately after connect
    println!(
        "\n=== Immediately After Connect ({:?}) ===",
        connect_duration
    );
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node2 endpoint closed: {:?}", node2.close_reason());
    println!("Connection stable ID: {}", connection.stable_id());

    // Check after small delay
    sleep(Duration::from_micros(50)).await;
    println!("\n=== After 50 microseconds ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node2 endpoint closed: {:?}", node2.close_reason());

    // Try to open stream
    println!("\n=== Attempting Stream Open ===");
    let stream_result = connection.open_bi().await;

    println!("\n=== After Stream Attempt ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node2 endpoint closed: {:?}", node2.close_reason());
    println!(
        "Stream result: {:?}",
        stream_result
            .as_ref()
            .map(|_| "OK")
            .map_err(|e| e.to_string())
    );

    match stream_result {
        Ok(_) => println!("✅ Stream opened successfully"),
        Err(e) => println!("❌ Stream open failed: {}", e),
    }

    // Check final state
    sleep(Duration::from_millis(100)).await;
    println!("\n=== Final State (after 100ms) ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node2 endpoint closed: {:?}", node2.close_reason());

    println!("\n✅ Test 2.4.2 completed (diagnostic data collected)");
    Ok(())
}

// =============================================================================
// PHASE 2 DIAGNOSTIC TESTS
// =============================================================================

/// Test 4.1.1: Measure Connect-to-Send Timing
///
/// Measures precise timing between connection establishment and first send attempt.
/// Identifies minimum safe delay required.
/// Expected: Reveal timing requirements
#[tokio::test]
async fn test_connect_to_send_timing() -> Result<()> {
    println!("\n=== Test 4.1.1: Connect-to-Send Timing ===");

    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;
    let addr2 = node2.local_addr()?;

    use std::time::Instant;

    // Test with various delays
    let delays_ms = vec![0, 1, 5, 10, 50, 100];

    for delay_ms in delays_ms {
        println!("\n--- Testing with {} ms delay ---", delay_ms);

        let connect_start = Instant::now();
        let connection = node1.connect_to(&addr2).await?;
        let connect_duration = connect_start.elapsed();
        println!("Connection established in {:?}", connect_duration);

        // Wait specified delay
        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms)).await;
        }

        // Try to send
        let send_start = Instant::now();
        let stream_result = connection.open_bi().await;
        let stream_duration = send_start.elapsed();

        match stream_result {
            Ok((mut send, _)) => {
                let write_result = send.write_all(b"test").await;
                let total_duration = connect_start.elapsed();

                match write_result {
                    Ok(_) => {
                        println!("✅ SUCCESS with {}ms delay", delay_ms);
                        println!("   Stream open: {:?}", stream_duration);
                        println!("   Total time: {:?}", total_duration);
                        send.finish()?;
                    }
                    Err(e) => {
                        println!("❌ FAILED with {}ms delay: {}", delay_ms, e);
                    }
                }
            }
            Err(e) => {
                println!("❌ FAILED to open stream with {}ms delay: {}", delay_ms, e);
            }
        }

        // Close connection for next iteration
        connection.close(0u32.into(), b"test");
        sleep(Duration::from_millis(10)).await; // Clean up
    }

    println!("\n✅ Test 4.1.1 completed (timing data collected)");
    Ok(())
}

/// Test 4.2.1: Connection State Inspection
///
/// Deep inspection of connection and endpoint state at each stage.
/// Reveals internal state transitions and identifies root cause.
/// Expected: Show exact state when failure occurs
#[tokio::test]
async fn test_connection_state_inspection() -> Result<()> {
    println!("\n=== Test 4.2.1: Connection State Inspection ===");

    let node1 = create_test_node().await?;
    let node2 = create_test_node().await?;
    let addr2 = node2.local_addr()?;

    println!("=== Initial State ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node1 local addr: {:?}", node1.local_addr());

    // Connect and inspect
    println!("\n=== Connecting ===");
    let connection = node1.connect_to(&addr2).await?;

    println!("\n=== Connection Object State ===");
    println!("Stable ID: {}", connection.stable_id());
    println!("Remote address: {}", connection.remote_address());
    println!("Max datagram size: {:?}", connection.max_datagram_size());

    // Check stats
    let stats = connection.stats();
    println!("\n=== Connection Stats ===");
    println!("Path RTT: {:?}", stats.path.rtt);
    println!("Path congestion window: {} bytes", stats.path.cwnd);
    println!("Sent packets: {}", stats.path.sent_packets);
    println!("Lost packets: {}", stats.path.lost_packets);

    println!("\n=== Endpoint State ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!("Node2 endpoint closed: {:?}", node2.close_reason());

    // Try stream open with state monitoring
    println!("\n=== Opening Stream ===");
    let stream_result = connection.open_bi().await;

    println!("\n=== After Stream Attempt ===");
    println!("Node1 endpoint closed: {:?}", node1.close_reason());
    println!(
        "Stream result: {:?}",
        stream_result
            .as_ref()
            .map(|_| "OK")
            .map_err(|e| format!("{}", e))
    );

    // Final stats
    let final_stats = connection.stats();
    println!("\n=== Final Connection Stats ===");
    println!("Path RTT: {:?}", final_stats.path.rtt);
    println!("Sent packets: {}", final_stats.path.sent_packets);
    println!("Lost packets: {}", final_stats.path.lost_packets);

    println!("\n✅ Test 4.2.1 completed (state data collected)");
    Ok(())
}

// =============================================================================
// TEST EXECUTION SUMMARY
// =============================================================================

#[tokio::test]
async fn test_run_critical_diagnostics() -> Result<()> {
    println!("\n");
    println!("═══════════════════════════════════════════════════════════");
    println!("  ANT-QUIC COMPREHENSIVE TEST SUITE - CRITICAL DIAGNOSTICS");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("Purpose: Isolate immediate connection closure issue");
    println!();
    println!("Phase 1 Tests:");
    println!("  • 2.1.1: Basic connection lifecycle");
    println!("  • 2.1.3: Immediate send (reproduces issue)");
    println!("  • 2.4.1: Endpoint stability");
    println!("  • 2.4.2: Endpoint closure timing");
    println!();
    println!("Phase 2 Tests:");
    println!("  • 4.1.1: Connect-to-send timing analysis");
    println!("  • 4.2.1: Connection state inspection");
    println!();
    println!("Run with: cargo test --package communitas-core --test ant_quic_comprehensive");
    println!("═══════════════════════════════════════════════════════════");
    println!();

    Ok(())
}

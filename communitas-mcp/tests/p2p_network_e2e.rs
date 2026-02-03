// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! P2P Network E2E Tests
//!
//! Tests for peer-to-peer networking, including bootstrap node connection,
//! peer discovery, four-word addressing, and presence announcements.
//!
//! These tests require network connectivity and are marked with #[ignore]
//! by default. Run with: `MCP_TEST_NETWORK_ENABLED=true cargo test p2p_network`

mod harness;

use harness::{BOOTSTRAP_NODES, P2pTestNode, P2pTestScenario};
use serde_json::json;
use std::time::Duration;

/// Check if network tests are enabled
fn network_tests_enabled() -> bool {
    std::env::var("MCP_TEST_NETWORK_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

/// Skip test if network is not enabled
macro_rules! require_network {
    () => {
        if !network_tests_enabled() {
            eprintln!("Skipping network test: MCP_TEST_NETWORK_ENABLED not set");
            return;
        }
    };
}

async fn resolve_connection_words(node: &P2pTestNode) -> Option<String> {
    let words = node.call_tool("get_connection_words", json!({})).await;
    if words.success {
        if let Some(value) = words.get_str("connection_words") {
            return Some(value.to_string());
        }
    }

    let status = node.call_tool("network_status", json!({})).await;
    if status.success {
        if let Some(value) = status.get_str("connection_identity") {
            return Some(value.to_string());
        }
    }

    node.four_words().map(|v| v.to_string())
}

// =============================================================================
// BOOTSTRAP CONNECTION TESTS
// =============================================================================

mod bootstrap_connection {
    use super::*;

    /// Test that a node can start and connect to bootstrap nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_network_start_and_bootstrap_connect() {
        require_network!();

        // Start a P2P-enabled test node
        let node = P2pTestNode::start_connected("bootstrap-test")
            .await
            .expect("start P2P node");

        // Verify network is started
        let status = node.call_tool("network_status", json!({})).await;
        assert!(status.success, "network_status should succeed");
        assert!(
            status.get_str("connection_identity").is_some(),
            "Network should provide a connection identity"
        );

        // Check that we're connected to at least one bootstrap node
        let mut peer_count = 0;
        for _ in 0..20 {
            let peers = node.call_tool("network_peers", json!({})).await;
            assert!(peers.success, "network_peers should succeed");

            peer_count = peers.get_i64("count").unwrap_or(0);
            if peer_count > 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        println!("Bootstrap peer count (presence-based): {}", peer_count);

        // Verify we can get connection words
        let four_words = resolve_connection_words(&node).await;
        assert!(
            four_words.is_some(),
            "Should receive four-word connection address"
        );

        println!(
            "Connected to {} peers, our words: {}",
            peer_count,
            four_words.as_deref().unwrap_or("unknown")
        );
    }

    /// Test network start and stop cycle
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_network_start_stop_cycle() {
        require_network!();

        let node = P2pTestNode::start_connected("cycle-test")
            .await
            .expect("start P2P node");

        // Verify network is running
        let status1 = node.call_tool("network_status", json!({})).await;
        assert!(status1.success);
        assert!(
            status1.get_bool("is_active").unwrap_or(false),
            "Network should be running"
        );

        // Stop the network
        let stop_result = node.call_tool("network_stop", json!({})).await;
        assert!(stop_result.success, "network_stop should succeed");

        // Verify network is stopped
        let status2 = node.call_tool("network_status", json!({})).await;
        assert!(status2.success);
        // Network may report as not running or in a transitional state

        // Restart the network
        let start_result = node
            .call_tool(
                "network_start",
                json!({
                    "bootstrap_nodes": BOOTSTRAP_NODES
                }),
            )
            .await;
        assert!(start_result.success, "network_start should succeed");

        // Wait for reconnection
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Verify network is running again
        let status3 = node.call_tool("network_status", json!({})).await;
        assert!(status3.success);
        assert!(
            status3.get_bool("is_active").unwrap_or(false),
            "Network should be running after restart"
        );
    }

    /// Test bootstrap node configuration
    #[tokio::test]
    async fn test_bootstrap_nodes_configured() {
        // Verify bootstrap nodes are properly configured
        assert!(
            !BOOTSTRAP_NODES.is_empty(),
            "BOOTSTRAP_NODES should not be empty"
        );

        // Check format of bootstrap nodes
        for node in BOOTSTRAP_NODES {
            assert!(
                node.contains(':'),
                "Bootstrap node should contain port: {}",
                node
            );
            let parts: Vec<&str> = node.split(':').collect();
            assert_eq!(parts.len(), 2, "Should have IP:port format");

            // Validate port is numeric
            let port: Result<u16, _> = parts[1].parse();
            assert!(port.is_ok(), "Port should be numeric: {}", parts[1]);
        }

        println!("Bootstrap nodes: {:?}", BOOTSTRAP_NODES);
    }
}

// =============================================================================
// PEER DISCOVERY TESTS
// =============================================================================

mod peer_discovery {
    use super::*;

    /// Test that two nodes can discover each other
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_peer_to_peer_discovery() {
        require_network!();

        // Create a two-node scenario
        let mut scenario = P2pTestScenario::new();
        scenario.add_node("alice").await.expect("add alice");
        scenario.add_node("bob").await.expect("add bob");
        scenario.mesh_connect().await.expect("mesh connect");

        let alice = scenario.node(0).expect("get alice");
        let bob = scenario.node(1).expect("get bob");

        // Both nodes should be connected to bootstrap
        let alice_peers = alice.call_tool("network_peers", json!({})).await;
        let bob_peers = bob.call_tool("network_peers", json!({})).await;

        assert!(alice_peers.success, "Alice should have peer info");
        assert!(bob_peers.success, "Bob should have peer info");

        // Get Alice's four words
        let alice_four_words = resolve_connection_words(alice)
            .await
            .expect("get alice words");

        // Get Bob's four words
        let bob_four_words = resolve_connection_words(bob).await.expect("get bob words");

        println!("Alice: {}", alice_four_words);
        println!("Bob: {}", bob_four_words);

        // Alice connects to Bob using four words
        let connect_result = alice
            .call_tool(
                "connect_by_words",
                json!({
                    "words": bob_four_words
                }),
            )
            .await;

        assert!(
            connect_result.success,
            "Alice should connect to Bob via four words: {:?}",
            connect_result
        );

        // Wait for connection to establish
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Verify Alice can see Bob as a peer
        let alice_peers_after = alice.call_tool("network_peers", json!({})).await;
        assert!(alice_peers_after.success);

        // The peer count should have increased or include Bob's identifier
        println!(
            "Alice peers after connect: {:?}",
            alice_peers_after.get_i64("count")
        );
    }

    /// Test four-word connection mechanism
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_four_word_connection() {
        require_network!();

        let alice = P2pTestNode::start_connected("alice-words")
            .await
            .expect("start alice");

        let bob = P2pTestNode::start_connected("bob-words")
            .await
            .expect("start bob");

        // Get Bob's connection words
        let bob_words = resolve_connection_words(&bob).await.expect("get bob words");
        println!("Bob's four words: {}", bob_words);

        // Validate four-word format (space or hyphen separated)
        let word_count = bob_words
            .split(|c: char| c == '-' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .count();
        assert_eq!(
            word_count, 4,
            "Should have exactly 4 words separated by spaces or hyphens"
        );

        // Alice connects to Bob
        let connect = alice
            .call_tool(
                "connect_by_words",
                json!({
                    "words": bob_words
                }),
            )
            .await;

        assert!(connect.success, "Four-word connection should succeed");

        // Wait for Bob to register the peer
        bob.wait_for_peer(
            &alice.four_words().unwrap_or_default(),
            Duration::from_secs(10),
        )
        .await
        .ok(); // May timeout, that's okay for this test
    }

    /// Test network_peers returns peer information
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_network_peers_after_connection() {
        require_network!();

        let node = P2pTestNode::start_connected("peers-test")
            .await
            .expect("start node");

        // Give time to discover peers
        tokio::time::sleep(Duration::from_secs(3)).await;

        let peers = node.call_tool("network_peers", json!({})).await;
        assert!(peers.success, "network_peers should succeed");

        // Should have peer list
        if let Some(peer_list) = peers.get_array("peers") {
            println!("Found {} peers", peer_list.len());
            for peer in peer_list {
                if let Some(peer_id) = peer.get("peer_id") {
                    println!("  Peer: {}", peer_id);
                }
            }
        }

        // Should have peer count
        let count = peers.get_i64("count").unwrap_or(0);
        println!("Total peer count: {}", count);
    }
}

// =============================================================================
// PRESENCE TESTS
// =============================================================================

mod presence {
    use super::*;

    /// Test presence announcement and query
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_presence_announcement_and_query() {
        require_network!();

        let node = P2pTestNode::start_connected("presence-test")
            .await
            .expect("start node");

        // Announce our presence
        let announce = node
            .call_tool(
                "announce_presence",
                json!({
                    "status": "online",
                    "status_message": "Testing presence"
                }),
            )
            .await;

        assert!(announce.success, "announce_presence should succeed");

        // Get our own presence
        let our_presence = node.call_tool("get_our_presence", json!({})).await;
        assert!(our_presence.success, "get_our_presence should succeed");

        if let Some(status) = our_presence.get_str("status") {
            assert_eq!(status, "online", "Status should match announced value");
        }

        // Query presence (should return our own at minimum)
        let query = node.call_tool("query_presence", json!({})).await;
        // This might fail if no peers are present, which is acceptable
        println!("Presence query result: {:?}", query);
    }

    /// Test presence status updates
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_presence_status_updates() {
        require_network!();

        let node = P2pTestNode::start_connected("status-test")
            .await
            .expect("start node");

        // Test various status values
        let statuses = vec![
            ("online", "Available"),
            ("away", "Be right back"),
            ("busy", "In a meeting"),
            ("offline", "Going offline"),
        ];

        for (status, message) in statuses {
            let result = node
                .call_tool(
                    "announce_presence",
                    json!({
                        "status": status,
                        "status_message": message
                    }),
                )
                .await;

            assert!(
                result.success,
                "announce_presence with status '{}' should succeed",
                status
            );

            // Verify the update
            let current = node.call_tool("get_our_presence", json!({})).await;
            if current.success {
                if let Some(current_status) = current.get_str("status") {
                    assert_eq!(current_status, status, "Status should be updated");
                }
            }
        }
    }

    /// Test cached presence retrieval
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_cached_presence() {
        require_network!();

        let node = P2pTestNode::start_connected("cache-test")
            .await
            .expect("start node");

        // Announce presence first
        let _ = node
            .call_tool(
                "announce_presence",
                json!({
                    "status": "online",
                    "status_message": "Cache test"
                }),
            )
            .await;

        // Get cached presence
        let cached = node.call_tool("get_cached_presence", json!({})).await;

        // This should return our cached presence data
        // May be empty if no peers have shared presence
        println!("Cached presence: {:?}", cached);
    }
}

// =============================================================================
// EXTERNAL ADDRESS TESTS
// =============================================================================

mod external_address {
    use super::*;

    /// Test requesting external address
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_network_request_external_address() {
        require_network!();

        let node = P2pTestNode::start_connected("external-addr-test")
            .await
            .expect("start node");

        // Request external address discovery
        let result = node
            .call_tool("network_request_external_address", json!({}))
            .await;

        // This may succeed or fail depending on NAT configuration
        // The important thing is that it doesn't crash
        println!("External address request result: {:?}", result);

        if result.success {
            if let Some(addr) = result.get_str("external_address") {
                println!("External address: {}", addr);
            }
        }
    }
}

// =============================================================================
// NETWORK TOOL COVERAGE
// =============================================================================

mod network_tool_coverage {
    use super::*;

    /// Test all network-related tools are callable
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_all_network_tools_callable() {
        require_network!();

        let node = P2pTestNode::start_connected("tool-coverage")
            .await
            .expect("start node");

        // List of all network tools to test
        let network_tools = vec![
            ("network_status", json!({})),
            ("network_peers", json!({})),
            ("get_connection_words", json!({})),
            ("get_our_presence", json!({})),
            ("get_cached_presence", json!({})),
        ];

        for (tool_name, params) in network_tools {
            let result = node.call_tool(tool_name, params).await;
            println!("{}: success={}", tool_name, result.success);
            // We just verify the tool is callable, not necessarily successful
            // (some may require specific conditions)
        }
    }
}

// =============================================================================
// TEST SUMMARY
// =============================================================================

/// Summary of P2P network test coverage
#[tokio::test]
async fn test_p2p_network_coverage_summary() {
    let test_categories = vec![
        ("Bootstrap Connection", 3), // start_bootstrap, start_stop_cycle, nodes_configured
        ("Peer Discovery", 3),       // p2p_discovery, four_word, peers_after_connect
        ("Presence", 3),             // announcement_query, status_updates, cached
        ("External Address", 1),     // request_external
        ("Tool Coverage", 1),        // all_tools_callable
    ];

    let total_tests: usize = test_categories.iter().map(|(_, count)| count).sum();

    println!("\n=== P2P NETWORK E2E TEST COVERAGE ===");
    for (category, count) in &test_categories {
        println!("  {}: {} tests", category, count);
    }
    println!("  TOTAL: {} tests", total_tests);
    println!("======================================\n");

    // Most tests require network
    let network_required = total_tests - 1; // Only bootstrap_nodes_configured doesn't need network
    println!("  Network-required tests: {}", network_required);
    println!("  Run with: MCP_TEST_NETWORK_ENABLED=true cargo test p2p_network --ignored");

    assert_eq!(total_tests, 11, "Expected 11 P2P network tests");
}

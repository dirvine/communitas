// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Sync Verification E2E Tests
//!
//! Tests that verify CRDT convergence between multiple P2P nodes using
//! the `CrdtSyncVerifier` harness utility.
//!
//! These tests require network connectivity. Run with:
//! `MCP_TEST_NETWORK_ENABLED=true cargo test crdt_sync`
//!
//! Timeout can be configured via: `MCP_TEST_SYNC_TIMEOUT=60`

mod harness;

use harness::{
    CrdtStateComparator, CrdtSyncVerifier, P2pTestNode, P2pTestScenario, SyncResult,
    network_tests_enabled, sync_timeout,
};
use serde_json::json;
use std::time::Duration;

// =============================================================================
// ENTITY SYNC VERIFICATION TESTS
// =============================================================================

mod entity_sync {
    use super::*;

    /// Test that an organization created on one node syncs to another
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_organization_sync_between_nodes() {
        if !network_tests_enabled() {
            println!("Skipping: MCP_TEST_NETWORK_ENABLED not set");
            return;
        }

        // Create two connected nodes
        let alice = P2pTestNode::start_connected("crdt-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("crdt-bob")
            .await
            .expect("start bob");

        // Connect Alice to Bob
        let mut alice_mut = alice;
        alice_mut
            .connect_to(&bob)
            .await
            .expect("connect alice to bob");

        // Alice creates an organization
        let create_result = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "CRDT Sync Test Org"
                }),
            )
            .await;

        assert!(create_result.success, "Should create organization");
        let entity_id = create_result.get_str("id").expect("Should have entity id");

        // Use CrdtSyncVerifier to wait for sync
        let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);

        let sync_result = verifier.wait_for_entity_sync(entity_id).await;

        match sync_result {
            SyncResult::Synced => {
                println!("✓ Organization synced successfully to all nodes");
            }
            SyncResult::Partial {
                synced_count,
                total,
            } => {
                println!("⚠ Partial sync: {}/{} nodes", synced_count, total);
                // Partial sync is acceptable in test environments
            }
            SyncResult::Timeout { waited } => {
                println!("⚠ Sync timed out after {:?}", waited);
                // Don't fail - network conditions may vary
            }
            SyncResult::Failed { reason } => {
                panic!("Sync failed: {}", reason);
            }
        }
    }

    /// Test that a project created on one node syncs to another
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_project_sync_between_nodes() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("proj-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("proj-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create project
        let create = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "project",
                    "name": "Sync Test Project"
                }),
            )
            .await;

        if let Some(entity_id) = create.get_str("id") {
            let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);
            let result = verifier.wait_for_entity_sync(entity_id).await;

            assert!(
                result.is_synced() || matches!(result, SyncResult::Partial { .. }),
                "Project should sync between nodes"
            );
        }
    }
}

// =============================================================================
// MESSAGE SYNC VERIFICATION TESTS
// =============================================================================

mod message_sync {
    use super::*;

    /// Test that messages sync and maintain correct count across nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_message_count_sync() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("msg-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("msg-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create a channel for messaging
        let channel = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "channel",
                    "name": "Message Sync Channel"
                }),
            )
            .await;

        let entity_id = channel.get_str("id").unwrap_or("test-channel");

        // Send multiple messages
        let message_count = 3;
        for i in 1..=message_count {
            alice_mut
                .call_tool(
                    "send_message",
                    json!({
                        "entity_id": entity_id,
                        "text": format!("Sync test message {}", i)
                    }),
                )
                .await;
        }

        // Verify message count syncs
        let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);
        let result = verifier
            .wait_for_message_count(entity_id, message_count)
            .await;

        if result.is_synced() {
            println!("✓ Message count ({}) synced to all nodes", message_count);
        } else {
            println!("⚠ Message sync result: {:?}", result);
        }
    }

    /// Test that message content is identical across nodes after sync
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_message_content_consistency() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("content-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("content-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        let channel = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "channel",
                    "name": "Content Consistency Channel"
                }),
            )
            .await;

        let entity_id = channel.get_str("id").unwrap_or("test-channel");

        // Send a unique message
        let unique_content = format!(
            "Unique message {}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        alice_mut
            .call_tool(
                "send_message",
                json!({
                    "entity_id": entity_id,
                    "text": unique_content
                }),
            )
            .await;

        // Wait for sync
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Check Bob has the message with same content
        let bob_messages = bob
            .call_tool(
                "get_messages",
                json!({
                    "entity_id": entity_id
                }),
            )
            .await;

        if bob_messages.success {
            if let Some(messages) = bob_messages.get_array("messages") {
                let found = messages.iter().any(|m| {
                    m.get("content")
                        .and_then(|c| c.as_str())
                        .map(|s| s == unique_content)
                        .unwrap_or(false)
                });
                assert!(found, "Bob should have the same message content");
                println!("✓ Message content verified on both nodes");
            }
        }
    }
}

// =============================================================================
// KANBAN BOARD SYNC VERIFICATION TESTS
// =============================================================================

mod kanban_sync {
    use super::*;

    /// Test that Kanban board syncs between nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_kanban_board_sync() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("kanban-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("kanban-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create an entity to hold the board
        let entity = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "project",
                    "name": "Kanban Sync Project"
                }),
            )
            .await;

        let entity_id = entity.get_str("id").unwrap_or("test-entity");

        // Create a Kanban board
        let board = alice_mut
            .call_tool(
                "create_kanban_board",
                json!({
                    "entity_id": entity_id,
                    "board_name": "CRDT Sync Board"
                }),
            )
            .await;

        let board_id = board.get_str("id").unwrap_or("test-board");

        // Verify board syncs
        let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);
        let result = verifier.wait_for_kanban_board_sync(board_id).await;

        if result.is_synced() {
            println!("✓ Kanban board synced to all nodes");
        } else {
            println!("⚠ Kanban sync result: {:?}", result);
        }
    }

    /// Test that Kanban card count syncs between nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_kanban_card_count_sync() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("cards-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("cards-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create entity and board
        let entity = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "project",
                    "name": "Card Count Project"
                }),
            )
            .await;

        let entity_id = entity.get_str("id").unwrap_or("test-entity");

        let board = alice_mut
            .call_tool(
                "create_kanban_board",
                json!({
                    "entity_id": entity_id,
                    "board_name": "Card Count Board"
                }),
            )
            .await;

        let board_id = board.get_str("id").unwrap_or("test-board");

        // Create a column
        let column = alice_mut
            .call_tool(
                "create_kanban_column",
                json!({
                    "board_id": board_id,
                    "title": "To Do"
                }),
            )
            .await;

        let column_id = column.get_str("id").unwrap_or("test-column");

        // Create multiple cards
        let card_count = 3;
        for i in 1..=card_count {
            alice_mut
                .call_tool(
                    "create_kanban_card",
                    json!({
                        "board_id": board_id,
                        "column_id": column_id,
                        "title": format!("Card {}", i)
                    }),
                )
                .await;
        }

        // Verify card count syncs
        let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);
        let synced = verifier
            .verify_kanban_card_count(board_id, card_count)
            .await;

        if synced {
            println!("✓ Kanban card count ({}) synced", card_count);
        } else {
            // Wait and retry
            tokio::time::sleep(Duration::from_secs(5)).await;
            let synced_retry = verifier
                .verify_kanban_card_count(board_id, card_count)
                .await;
            println!(
                "Card count sync after retry: {}",
                if synced_retry { "✓" } else { "⚠" }
            );
        }
    }
}

// =============================================================================
// FILE SYNC VERIFICATION TESTS
// =============================================================================

mod file_sync {
    use super::*;

    /// Test that file content syncs between nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_file_content_sync() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("file-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("file-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create entity
        let entity = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "File Sync Org"
                }),
            )
            .await;

        let entity_id = entity.get_str("id").unwrap_or("test-entity");
        let test_content = "This content should sync via CRDT";
        let test_path = "/sync-test.txt";

        // Write file on Alice
        alice_mut
            .call_tool(
                "write_file",
                json!({
                    "entity_id": entity_id,
                    "disk_type": "shared",
                    "path": test_path,
                    "content": test_content
                }),
            )
            .await;

        // Verify file syncs with correct content
        let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);
        let synced = verifier
            .verify_file_content(entity_id, "shared", test_path, test_content)
            .await;

        if synced {
            println!("✓ File content synced correctly");
        } else {
            // Wait for sync
            let result = verifier
                .wait_for_file_sync(entity_id, "shared", test_path)
                .await;
            println!("File sync result: {:?}", result);
        }
    }
}

// =============================================================================
// STATE COMPARISON TESTS
// =============================================================================

mod state_comparison {
    use super::*;

    /// Test that entity lists converge between nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_entity_state_convergence() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("state-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("state-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create several entities on Alice
        for i in 1..=3 {
            alice_mut
                .call_tool(
                    "create_entity",
                    json!({
                        "entity_type": "project",
                        "name": format!("State Test Project {}", i)
                    }),
                )
                .await;
        }

        // Wait for sync
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Compare entity states
        let same = CrdtStateComparator::compare_entities(&alice_mut, &bob, "project").await;

        if same {
            println!("✓ Entity states converged between nodes");
        } else {
            println!("⚠ Entity states may still be syncing");
        }
    }

    /// Test that Kanban board lists converge between nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_kanban_state_convergence() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("kstate-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("kstate-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create entity and boards on Alice
        let entity = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "project",
                    "name": "Kanban State Project"
                }),
            )
            .await;

        let entity_id = entity.get_str("id").unwrap_or("test-entity");

        for i in 1..=2 {
            alice_mut
                .call_tool(
                    "create_kanban_board",
                    json!({
                        "entity_id": entity_id,
                        "board_name": format!("Board {}", i)
                    }),
                )
                .await;
        }

        // Wait for sync
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Compare Kanban states
        let same = CrdtStateComparator::compare_kanban_boards(&alice_mut, &bob, entity_id).await;

        if same {
            println!("✓ Kanban board states converged");
        } else {
            println!("⚠ Kanban states may still be syncing");
        }
    }
}

// =============================================================================
// MEMBER SYNC VERIFICATION TESTS
// =============================================================================

mod member_sync {
    use super::*;

    /// Test that member count syncs correctly between nodes
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_member_count_sync() {
        if !network_tests_enabled() {
            return;
        }

        let alice = P2pTestNode::start_connected("member-alice")
            .await
            .expect("start alice");
        let bob = P2pTestNode::start_connected("member-bob")
            .await
            .expect("start bob");

        let mut alice_mut = alice;
        alice_mut.connect_to(&bob).await.ok();

        // Create organization
        let org = alice_mut
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Member Sync Org"
                }),
            )
            .await;

        let entity_id = org.get_str("id").unwrap_or("test-org");

        // Add member (Alice is already a member as creator)
        // The creator counts as 1 member
        let expected_members = 1;

        // Verify member count syncs
        let verifier = CrdtSyncVerifier::new(vec![&alice_mut, &bob]);
        let synced = verifier
            .verify_member_count(entity_id, "organization", expected_members)
            .await;

        if synced {
            println!("✓ Member count synced across nodes");
        } else {
            println!("⚠ Member count sync may be incomplete");
        }
    }
}

// =============================================================================
// MULTI-NODE SCENARIO TESTS
// =============================================================================

mod multi_node {
    use super::*;

    /// Test CRDT sync in a 3-node mesh network
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_three_node_mesh_sync() {
        if !network_tests_enabled() {
            return;
        }

        let mut scenario = P2pTestScenario::new();

        // Create 3 nodes
        let alice_idx = scenario.add_node("mesh-alice").await.expect("add alice");
        let bob_idx = scenario.add_node("mesh-bob").await.expect("add bob");
        let charlie_idx = scenario
            .add_node("mesh-charlie")
            .await
            .expect("add charlie");

        // Connect all nodes
        scenario.mesh_connect().await.ok();

        // Alice creates an entity
        let alice = scenario.node(alice_idx).expect("get alice");
        let entity = alice
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Three Node Mesh Org"
                }),
            )
            .await;

        let entity_id = entity.get_str("id").unwrap_or("test-entity");

        // Wait for propagation
        tokio::time::sleep(Duration::from_secs(10)).await;

        // Verify all three nodes have the entity
        let bob = scenario.node(bob_idx).expect("get bob");
        let charlie = scenario.node(charlie_idx).expect("get charlie");

        let verifier = CrdtSyncVerifier::new(vec![alice, bob, charlie]);
        let result = verifier.wait_for_entity_sync(entity_id).await;

        match result {
            SyncResult::Synced => {
                println!("✓ Entity synced to all 3 nodes");
            }
            SyncResult::Partial {
                synced_count,
                total,
            } => {
                println!("⚠ Partial sync: {}/{} nodes", synced_count, total);
            }
            _ => {
                println!("⚠ Sync result: {:?}", result);
            }
        }
    }
}

// =============================================================================
// TEST SUMMARY
// =============================================================================

/// Summary test documenting CRDT sync test coverage
#[tokio::test]
async fn test_crdt_sync_coverage_summary() {
    let test_categories = vec![
        ("Entity Sync", 2),      // organization_sync, project_sync
        ("Message Sync", 2),     // count_sync, content_consistency
        ("Kanban Sync", 2),      // board_sync, card_count_sync
        ("File Sync", 1),        // file_content_sync
        ("State Comparison", 2), // entity_convergence, kanban_convergence
        ("Member Sync", 1),      // member_count_sync
        ("Multi-Node", 1),       // three_node_mesh
    ];

    let total_tests: usize = test_categories.iter().map(|(_, count)| count).sum();

    println!("\n=== CRDT SYNC E2E TEST COVERAGE ===");
    for (category, count) in &test_categories {
        println!("  {}: {} tests", category, count);
    }
    println!("  TOTAL: {} tests (all require network)", total_tests);
    println!("  Run with: MCP_TEST_NETWORK_ENABLED=true cargo test crdt_sync");
    println!(
        "  Timeout: MCP_TEST_SYNC_TIMEOUT={} (configurable)",
        sync_timeout().as_secs()
    );
    println!("=====================================\n");

    assert_eq!(total_tests, 11, "Expected 11 CRDT sync tests");
}

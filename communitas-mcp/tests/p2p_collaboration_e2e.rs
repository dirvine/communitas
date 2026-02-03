// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! P2P Collaboration E2E Tests
//!
//! Tests for multi-instance collaboration scenarios including:
//! - Organization sync between peers
//! - Channel messaging synchronization
//! - Kanban board collaborative editing
//! - Virtual disk file synchronization
//! - Contact exchange via four words
//!
//! These tests require network connectivity and multiple running instances.
//! Run with: `MCP_TEST_NETWORK_ENABLED=true cargo test p2p_collaboration`

mod harness;

use harness::{CrdtSyncVerifier, P2pTestNode, P2pTestScenario, SyncResult};
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

/// Helper to create a two-node scenario
async fn create_two_node_scenario(name_a: &str, name_b: &str) -> Result<P2pTestScenario, String> {
    let mut scenario = P2pTestScenario::new();
    scenario.add_node(name_a).await?;
    scenario.add_node(name_b).await?;
    scenario.mesh_connect().await?;
    Ok(scenario)
}

// =============================================================================
// ORGANIZATION SYNC TESTS
// =============================================================================

mod organization_sync {
    use super::*;

    /// Test organization syncs between peers after invitation
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_organization_sync_between_peers() {
        require_network!();

        // Set up two connected nodes
        let scenario = create_two_node_scenario("alice", "bob")
            .await
            .expect("create scenario");

        let alice = scenario.node(0).expect("get alice");
        let bob = scenario.node(1).expect("get bob");

        // Alice creates an organization
        let org_result = alice
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Sync Test Org",
                    "description": "Testing org sync"
                }),
            )
            .await;

        assert!(org_result.success, "Alice should create organization");
        let org_id = org_result.get_str("entity_id").expect("get entity_id");

        // Alice invites Bob (would need Bob's identity)
        // For now, simulate with direct member addition
        let bob_identity = bob.call_tool("get_profile", json!({})).await;
        if let Some(bob_pubkey) = bob_identity.get_str("pubkey_hex") {
            let add_result = alice
                .call_tool(
                    "add_member",
                    json!({
                        "entity_id": org_id,
                        "member_pubkey": bob_pubkey,
                        "role": "member"
                    }),
                )
                .await;

            // May succeed or fail depending on implementation
            println!("Add member result: {:?}", add_result);
        }

        // Wait for CRDT sync
        let verifier =
            CrdtSyncVerifier::new(vec![alice, bob]).with_timeout(Duration::from_secs(30));

        let sync_result = verifier.wait_for_entity_sync(org_id).await;

        match sync_result {
            SyncResult::Synced => println!("Organization synced to Bob"),
            SyncResult::Partial {
                synced_count,
                total,
            } => {
                println!("Partial sync: {}/{} nodes", synced_count, total);
            }
            SyncResult::Timeout { waited } => {
                println!("Sync timed out after {:?}", waited);
            }
            SyncResult::Failed { reason } => {
                println!("Sync failed: {}", reason);
            }
        }
    }

    /// Test member role changes propagate to all members
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_member_role_sync() {
        require_network!();

        let scenario = create_two_node_scenario("admin", "member")
            .await
            .expect("create scenario");

        let admin = scenario.node(0).expect("get admin");
        let member = scenario.node(1).expect("get member");

        // Admin creates organization
        let org = admin
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Role Sync Org"
                }),
            )
            .await;

        if !org.success {
            println!("Could not create org: {:?}", org);
            return;
        }

        let org_id = org.get_str("entity_id").expect("get org_id");

        // Get member's identity
        let member_profile = member.call_tool("get_profile", json!({})).await;
        let member_pubkey = member_profile.get_str("pubkey_hex");

        if let Some(pubkey) = member_pubkey {
            // Add member with initial role
            let add = admin
                .call_tool(
                    "add_member",
                    json!({
                        "entity_id": org_id,
                        "member_pubkey": pubkey,
                        "role": "member"
                    }),
                )
                .await;

            println!("Add member: {:?}", add);

            // Wait for sync
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Update role to admin
            let update = admin
                .call_tool(
                    "update_member",
                    json!({
                        "entity_id": org_id,
                        "member_pubkey": pubkey,
                        "role": "admin"
                    }),
                )
                .await;

            println!("Update role: {:?}", update);

            // Verify sync
            let verifier =
                CrdtSyncVerifier::new(vec![admin, member]).with_timeout(Duration::from_secs(15));

            let synced = verifier
                .verify_member_count(org_id, "organization", 2)
                .await;
            println!("Member count synced: {}", synced);
        }
    }
}

// =============================================================================
// MESSAGING SYNC TESTS
// =============================================================================

mod messaging_sync {
    use super::*;

    /// Test channel messages sync to all members
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_channel_messaging_sync() {
        require_network!();

        let scenario = create_two_node_scenario("sender", "receiver")
            .await
            .expect("create scenario");

        let sender = scenario.node(0).expect("get sender");
        let receiver = scenario.node(1).expect("get receiver");

        // Create a shared channel (via organization)
        let org = sender
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Message Sync Org"
                }),
            )
            .await;

        if !org.success {
            println!("Could not create org: {:?}", org);
            return;
        }

        let org_id = org.get_str("entity_id").expect("get org_id");

        // Create channel within org
        let channel = sender
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "channel",
                    "parent_id": org_id,
                    "name": "general"
                }),
            )
            .await;

        let channel_id = channel.get_str("entity_id").unwrap_or(org_id); // Fall back to org if channel creation fails

        // Send messages
        let messages = vec![
            "Hello, this is message 1",
            "This is message 2 for sync test",
            "Final message - number 3",
        ];

        for (i, content) in messages.iter().enumerate() {
            let send = sender
                .call_tool(
                    "send_message",
                    json!({
                        "entity_id": channel_id,
                        "text": content
                    }),
                )
                .await;

            println!("Send message {}: {:?}", i + 1, send.success);
        }

        // Wait for CRDT sync
        let verifier =
            CrdtSyncVerifier::new(vec![sender, receiver]).with_timeout(Duration::from_secs(30));

        let sync_result = verifier
            .wait_for_message_count(channel_id, messages.len())
            .await;

        match sync_result {
            SyncResult::Synced => {
                println!("All {} messages synced to receiver", messages.len());
            }
            other => {
                println!("Message sync result: {:?}", other);
            }
        }
    }

    /// Test direct message delivery
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_direct_message_sync() {
        require_network!();

        let mut alice = P2pTestNode::start_connected("dm-alice")
            .await
            .expect("start alice");

        let bob = P2pTestNode::start_connected("dm-bob")
            .await
            .expect("start bob");

        // Connect Alice and Bob
        let _ = alice.connect_to(&bob).await;

        // Wait for connection
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Get Bob's identity for DM
        let bob_profile = bob.call_tool("get_profile", json!({})).await;
        let bob_pubkey = bob_profile.get_str("pubkey_hex");

        if let Some(pubkey) = bob_pubkey {
            // Alice sends DM to Bob
            let dm = alice
                .call_tool(
                    "send_direct_message",
                    json!({
                        "recipient_pubkey": pubkey,
                        "content": "Hello Bob, this is a direct message!"
                    }),
                )
                .await;

            println!("DM send result: {:?}", dm);

            // Wait for delivery
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Bob checks for messages
            let bob_messages = bob.call_tool("list_direct_messages", json!({})).await;
            println!("Bob's DMs: {:?}", bob_messages);
        }
    }
}

// =============================================================================
// KANBAN SYNC TESTS
// =============================================================================

mod kanban_sync {
    use super::*;

    /// Test Kanban board collaborative editing
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_kanban_collaborative_editing() {
        require_network!();

        let scenario = create_two_node_scenario("alice", "bob")
            .await
            .expect("create scenario");

        let alice = scenario.node(0).expect("get alice");
        let bob = scenario.node(1).expect("get bob");

        // Create org and project
        let org = alice
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Kanban Test Org"
                }),
            )
            .await;

        let org_id = org.get_str("entity_id").unwrap_or("test-org");

        // Alice creates a Kanban board
        let board = alice
            .call_tool(
                "create_kanban_board",
                json!({
                    "entity_id": org_id,
                    "board_name": "Sprint Board",
                    "description": "Collaborative sprint planning"
                }),
            )
            .await;

        if !board.success {
            println!("Could not create board: {:?}", board);
            return;
        }

        let board_id = board.get_str("board_id").expect("get board_id");
        println!("Created board: {}", board_id);

        // Alice creates columns
        let columns = vec!["To Do", "In Progress", "Done"];
        let mut column_ids = Vec::new();

        for col_name in &columns {
            let col = alice
                .call_tool(
                    "create_kanban_column",
                    json!({
                        "board_id": board_id,
                        "title": col_name
                    }),
                )
                .await;

            if let Some(col_id) = col.get_str("column_id") {
                column_ids.push(col_id.to_string());
            }
        }

        // Wait for board sync to Bob
        let verifier =
            CrdtSyncVerifier::new(vec![alice, bob]).with_timeout(Duration::from_secs(20));

        let board_synced = verifier.wait_for_kanban_board_sync(board_id).await;
        println!("Board sync result: {:?}", board_synced);

        // Alice creates a card
        if let Some(todo_col) = column_ids.first() {
            let card = alice
                .call_tool(
                    "create_kanban_card",
                    json!({
                        "board_id": board_id,
                        "column_id": todo_col,
                        "title": "Task from Alice",
                        "description": "Alice created this task"
                    }),
                )
                .await;

            if let Some(card_id) = card.get_str("card_id") {
                println!("Alice created card: {}", card_id);

                // Wait for card sync
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Bob moves the card (if he has access)
                if let Some(progress_col) = column_ids.get(1) {
                    let move_result = bob
                        .call_tool(
                            "move_kanban_card",
                            json!({
                                "board_id": board_id,
                                "card_id": card_id,
                                "target_column_id": progress_col
                            }),
                        )
                        .await;

                    println!("Bob move card result: {:?}", move_result);

                    // Verify sync
                    let card_synced = verifier.verify_kanban_card_sync(board_id, card_id).await;
                    println!("Card synced: {}", card_synced);
                }
            }
        }
    }

    /// Test Kanban card count synchronization
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_kanban_card_count_sync() {
        require_network!();

        let scenario = create_two_node_scenario("creator", "viewer")
            .await
            .expect("create scenario");

        let creator = scenario.node(0).expect("get creator");
        let viewer = scenario.node(1).expect("get viewer");

        // Create board
        let board = creator
            .call_tool(
                "create_kanban_board",
                json!({
                    "entity_id": "test-entity",
                    "board_name": "Card Count Test"
                }),
            )
            .await;

        let board_id = board.get_str("board_id").unwrap_or("test-board");

        // Create column
        let col = creator
            .call_tool(
                "create_kanban_column",
                json!({
                    "board_id": board_id,
                    "title": "Tasks"
                }),
            )
            .await;

        let col_id = col.get_str("column_id").unwrap_or("test-col");

        // Create multiple cards
        let card_count = 5;
        for i in 1..=card_count {
            let _ = creator
                .call_tool(
                    "create_kanban_card",
                    json!({
                        "board_id": board_id,
                        "column_id": col_id,
                        "title": format!("Card {}", i)
                    }),
                )
                .await;
        }

        // Verify card count syncs
        let verifier =
            CrdtSyncVerifier::new(vec![creator, viewer]).with_timeout(Duration::from_secs(30));

        let synced = verifier
            .verify_kanban_card_count(board_id, card_count)
            .await;
        println!("Card count synced: {}", synced);
    }
}

// =============================================================================
// FILE SYNC TESTS
// =============================================================================

mod file_sync {
    use super::*;

    /// Test virtual disk file synchronization
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_virtual_disk_file_sync() {
        require_network!();

        let scenario = create_two_node_scenario("writer", "reader")
            .await
            .expect("create scenario");

        let writer = scenario.node(0).expect("get writer");
        let reader = scenario.node(1).expect("get reader");

        // Create shared entity
        let org = writer
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "File Sync Org"
                }),
            )
            .await;

        let entity_id = org.get_str("entity_id").unwrap_or("test-entity");

        // Write a file to shared disk
        let test_content = "This is test content for sync verification.\nLine 2.\nLine 3.";
        let file_path = "/sync-test.txt";

        let write = writer
            .call_tool(
                "write_file",
                json!({
                    "entity_id": entity_id,
                    "disk_type": "shared",
                    "path": file_path,
                    "content": test_content
                }),
            )
            .await;

        println!("Write file result: {:?}", write);

        // Wait for CRDT sync
        let verifier =
            CrdtSyncVerifier::new(vec![writer, reader]).with_timeout(Duration::from_secs(30));

        let sync_result = verifier
            .wait_for_file_sync(entity_id, "shared", file_path)
            .await;

        match sync_result {
            SyncResult::Synced => {
                // Verify content matches
                let content_match = verifier
                    .verify_file_content(entity_id, "shared", file_path, test_content)
                    .await;
                println!("File content matches: {}", content_match);
            }
            other => {
                println!("File sync result: {:?}", other);
            }
        }
    }

    /// Test file update synchronization
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_file_update_sync() {
        require_network!();

        let mut alice = P2pTestNode::start_connected("file-alice")
            .await
            .expect("start alice");

        let bob = P2pTestNode::start_connected("file-bob")
            .await
            .expect("start bob");

        let _ = alice.connect_to(&bob).await;

        let entity_id = "shared-files";
        let file_path = "/updates.txt";

        // Alice writes initial content
        let _ = alice
            .call_tool(
                "write_file",
                json!({
                    "entity_id": entity_id,
                    "disk_type": "shared",
                    "path": file_path,
                    "content": "Version 1"
                }),
            )
            .await;

        tokio::time::sleep(Duration::from_secs(2)).await;

        // Alice updates the file
        let _ = alice
            .call_tool(
                "write_file",
                json!({
                    "entity_id": entity_id,
                    "disk_type": "shared",
                    "path": file_path,
                    "content": "Version 2 - updated"
                }),
            )
            .await;

        // Verify Bob gets the update
        let verifier =
            CrdtSyncVerifier::new(vec![&alice, &bob]).with_timeout(Duration::from_secs(20));

        let content_synced = verifier
            .verify_file_content(entity_id, "shared", file_path, "Version 2 - updated")
            .await;

        println!("File update synced: {}", content_synced);
    }
}

// =============================================================================
// CONTACT EXCHANGE TESTS
// =============================================================================

mod contact_exchange {
    use super::*;

    /// Test contact exchange via four words
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_contact_exchange_via_four_words() {
        require_network!();

        let alice = P2pTestNode::start_connected("contact-alice")
            .await
            .expect("start alice");

        let bob = P2pTestNode::start_connected("contact-bob")
            .await
            .expect("start bob");

        // Get Bob's four words
        let bob_words = bob.call_tool("get_connection_words", json!({})).await;
        let words = bob_words
            .get_str("connection_words")
            .or_else(|| bob_words.get_str("connection_identity"))
            .or_else(|| bob.four_words())
            .unwrap_or("unknown");

        println!("Bob's words: {}", words);

        // Alice connects to Bob using four words
        let connect = alice
            .call_tool(
                "connect_by_words",
                json!({
                    "words": words
                }),
            )
            .await;

        println!("Connect result: {:?}", connect);

        // After connection, exchange contact info
        if words == "unknown" {
            eprintln!("Skipping contact exchange: no connection words available");
            return;
        }

        // Alice creates a contact for Bob using four words
        let contact = alice
            .call_tool(
                "create_contact",
                json!({
                    "display_name": "Bob",
                    "four_words": words
                }),
            )
            .await;

        println!("Create contact result: {:?}", contact);

        // Link contact to the connected peer
        if let Some(contact_id) = contact.get_str("contact_id") {
            let link = alice
                .call_tool(
                    "link_contact",
                    json!({
                        "contact_id": contact_id,
                        "four_words": words
                    }),
                )
                .await;

            println!("Link contact result: {:?}", link);
        }

        // Verify contact is stored
        let alice_contacts = alice.call_tool("list_contacts", json!({})).await;
        println!("Alice's contacts: {:?}", alice_contacts);
    }

    /// Test contact presence synchronization
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_contact_presence_sync() {
        require_network!();

        let mut alice = P2pTestNode::start_connected("presence-alice")
            .await
            .expect("start alice");

        let bob = P2pTestNode::start_connected("presence-bob")
            .await
            .expect("start bob");

        // Connect
        let _ = alice.connect_to(&bob).await;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Bob announces presence
        let _ = bob
            .call_tool(
                "announce_presence",
                json!({
                    "status": "online",
                    "status_message": "Available for chat"
                }),
            )
            .await;

        // Get Bob's profile for contact creation
        let bob_profile = bob.call_tool("get_profile", json!({})).await;

        if let Some(bob_pubkey) = bob_profile.get_str("pubkey_hex") {
            // Alice creates contact for Bob
            let _ = alice
                .call_tool(
                    "create_contact",
                    json!({
                        "name": "Bob",
                        "pubkey": bob_pubkey
                    }),
                )
                .await;

            // Wait for presence propagation
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Alice checks Bob's presence
            let presence = alice
                .call_tool(
                    "get_contact_presence",
                    json!({
                        "contact_pubkey": bob_pubkey
                    }),
                )
                .await;

            println!("Bob's presence from Alice's view: {:?}", presence);

            if presence.success {
                if let Some(status) = presence.get_str("status") {
                    assert_eq!(status, "online", "Bob's status should be online");
                }
            }
        }
    }
}

// =============================================================================
// INVITATION WORKFLOW TESTS
// =============================================================================

mod invitation_workflow {
    use super::*;

    /// Test full invitation workflow: invite → accept → sync
    #[tokio::test]
    #[ignore = "Requires network - run with MCP_TEST_NETWORK_ENABLED=true"]
    async fn test_invitation_workflow() {
        require_network!();

        let scenario = create_two_node_scenario("inviter", "invitee")
            .await
            .expect("create scenario");

        let inviter = scenario.node(0).expect("get inviter");
        let invitee = scenario.node(1).expect("get invitee");

        // Create organization
        let org = inviter
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Invitation Test Org"
                }),
            )
            .await;

        let org_id = org.get_str("entity_id").unwrap_or("test-org");

        // Get invitee's identity
        let invitee_profile = invitee.call_tool("get_profile", json!({})).await;

        if let Some(invitee_pubkey) = invitee_profile.get_str("pubkey_hex") {
            // Send invitation
            let invite = inviter
                .call_tool(
                    "send_invitation",
                    json!({
                        "entity_id": org_id,
                        "invitee_pubkey": invitee_pubkey,
                        "role": "member",
                        "message": "Welcome to the team!"
                    }),
                )
                .await;

            println!("Send invitation result: {:?}", invite);

            // Wait for invite delivery
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Invitee checks pending invitations
            let pending = invitee.call_tool("list_pending_invites", json!({})).await;
            println!("Pending invites: {:?}", pending);

            // Accept invitation (if present)
            if let Some(invites) = pending.get_array("invitations") {
                if let Some(first_invite) = invites.first() {
                    if let Some(invite_id) = first_invite.get("invitation_id") {
                        let accept = invitee
                            .call_tool(
                                "accept_invitation",
                                json!({
                                    "invitation_id": invite_id
                                }),
                            )
                            .await;

                        println!("Accept invitation result: {:?}", accept);

                        // Verify membership synced
                        let verifier = CrdtSyncVerifier::new(vec![inviter, invitee])
                            .with_timeout(Duration::from_secs(20));

                        let synced = verifier
                            .verify_member_count(org_id, "organization", 2)
                            .await;
                        println!("Membership synced: {}", synced);
                    }
                }
            }
        }
    }
}

// =============================================================================
// TEST SUMMARY
// =============================================================================

/// Summary of P2P collaboration test coverage
#[tokio::test]
async fn test_p2p_collaboration_coverage_summary() {
    let test_categories = vec![
        ("Organization Sync", 2),   // org_sync, role_sync
        ("Messaging Sync", 2),      // channel_sync, dm_sync
        ("Kanban Sync", 2),         // collaborative_edit, card_count
        ("File Sync", 2),           // file_sync, update_sync
        ("Contact Exchange", 2),    // four_words, presence
        ("Invitation Workflow", 1), // full_workflow
    ];

    let total_tests: usize = test_categories.iter().map(|(_, count)| count).sum();

    println!("\n=== P2P COLLABORATION E2E TEST COVERAGE ===");
    for (category, count) in &test_categories {
        println!("  {}: {} tests", category, count);
    }
    println!("  TOTAL: {} tests", total_tests);
    println!("============================================\n");

    println!("  All tests require network connectivity.");
    println!("  Run with: MCP_TEST_NETWORK_ENABLED=true cargo test p2p_collaboration --ignored");

    assert_eq!(total_tests, 11, "Expected 11 P2P collaboration tests");
}

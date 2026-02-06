// Copyright (c) 2026 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Multi-user application flow tests
//!
//! These scenarios orchestrate multiple Communitas nodes interacting together:
//! - contacts exchanged between users
//! - shared org/project/channel creation
//! - collaborative messaging verification
//! - Kanban board creation + synchronization
//!
//! The tests require the full networking stack (QUIC + gossip). Run with:
//! `MCP_TEST_NETWORK_ENABLED=true cargo test -p communitas-mcp multi_user_flows -- --ignored`

mod harness;

use harness::{CrdtSyncVerifier, P2pTestNode, P2pTestScenario, ToolAssert, ToolResult};
use serde_json::json;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Environment gating
// ---------------------------------------------------------------------------

fn network_tests_enabled() -> bool {
    std::env::var("MCP_TEST_NETWORK_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

macro_rules! require_network {
    () => {
        if !network_tests_enabled() {
            eprintln!("Skipping network test: MCP_TEST_NETWORK_ENABLED not set");
            return;
        }
    };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn create_multi_node_scenario(names: &[&str]) -> Result<P2pTestScenario, String> {
    let mut scenario = P2pTestScenario::new();
    for name in names {
        scenario.add_node(name).await?;
    }
    scenario.mesh_connect().await?;
    Ok(scenario)
}

fn extract_id(result: &ToolResult) -> Option<String> {
    [
        "id",
        "entity_id",
        "board_id",
        "column_id",
        "card_id",
        "message_id",
        "contact_id",
    ]
    .iter()
    .find_map(|key| result.get_str(key))
    .map(|s| s.to_string())
}

async fn pubkey_hex(node: &P2pTestNode) -> String {
    let profile = node.call_tool("get_profile", json!({})).await;
    profile
        .get_str("pubkey_hex")
        .unwrap_or("unknown-pubkey")
        .to_string()
}

async fn identity_four_words(node: &P2pTestNode) -> String {
    let profile = node.call_tool("get_profile", json!({})).await;
    if profile.success
        && let Some(words) = profile.get_str("four_words")
    {
        return words.to_string();
    }

    if let Some(words) = node.four_words() {
        return words.to_string();
    }

    "unknown-identity".to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "Requires QUIC networking - set MCP_TEST_NETWORK_ENABLED=true"]
async fn test_contacts_channels_messages_and_kanban_sync() {
    require_network!();

    let scenario = create_multi_node_scenario(&["alice-flow", "bob-flow", "carol-flow"])
        .await
        .expect("start scenario");

    let nodes = scenario.nodes();
    let (alice, bob, carol) = (&nodes[0], &nodes[1], &nodes[2]);
    let verifier =
        CrdtSyncVerifier::new(vec![alice, bob, carol]).with_timeout(Duration::from_secs(45));

    // Exchange pubkeys + four-word identities for contact/member operations
    let alice_key = pubkey_hex(alice).await;
    let bob_key = pubkey_hex(bob).await;
    let bob_words = identity_four_words(bob).await;
    let carol_words = identity_four_words(carol).await;

    // ---------------------------------------------------------------------
    // Contact creation + validation
    // ---------------------------------------------------------------------
    let contact = alice
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Bob Builder",
                "pubkey_hex": bob_key
            }),
        )
        .await;
    contact.assert_success();
    let contact_id = extract_id(&contact).expect("contact id");

    let contacts = alice
        .call_tool("list_contacts", json!({ "limit": 10 }))
        .await;
    contacts.assert_success().assert_contains("Bob Builder");
    assert!(
        contacts.content.contains(&contact_id),
        "Alice should list Bob contact"
    );

    // Bob reciprocates for Alice
    bob.call_tool(
        "create_contact",
        json!({
            "display_name": "Alice Flow",
            "pubkey_hex": alice_key
        }),
    )
    .await
    .assert_success();

    // ---------------------------------------------------------------------
    // Shared organization + project + channel
    // ---------------------------------------------------------------------
    let org = alice
        .call_tool(
            "create_entity",
            json!({
                "name": "Flow Org",
                "entity_type": "organization",
                "description": "End-to-end validation org"
            }),
        )
        .await;
    org.assert_success();
    let org_id = extract_id(&org).expect("org id");

    // Add Bob + Carol
    for member_words in [&bob_words, &carol_words] {
        alice
            .call_tool(
                "add_member",
                json!({
                    "entity_type": "organisation",
                    "entity_id": org_id,
                    "member_id": member_words,
                    "role": "member"
                }),
            )
            .await
            .assert_success();
    }

    verifier
        .wait_for_entity_sync(&org_id)
        .await
        .expect("org sync");

    // Project under org
    let project = alice
        .call_tool(
            "create_entity",
            json!({
                "name": "Launch Project",
                "entity_type": "project",
                "parent_id": org_id
            }),
        )
        .await;
    project.assert_success();
    let project_id = extract_id(&project).expect("project id");
    verifier
        .wait_for_entity_sync(&project_id)
        .await
        .expect("project sync");

    // Shared channel for messaging
    let channel = alice
        .call_tool(
            "create_entity",
            json!({
                "name": "Launch Chat",
                "entity_type": "channel",
                "parent_id": project_id
            }),
        )
        .await;
    channel.assert_success();
    let channel_id = extract_id(&channel).expect("channel id");
    verifier
        .wait_for_entity_sync(&channel_id)
        .await
        .expect("channel sync");

    // ---------------------------------------------------------------------
    // Messaging round-trip
    // ---------------------------------------------------------------------
    alice
        .call_tool(
            "send_message",
            json!({
                "entity_id": channel_id,
                "entity_type": "channel",
                "text": "Kick-off scheduled at 10:00 UTC."
            }),
        )
        .await
        .assert_success();

    verifier
        .wait_for_message_count(&channel_id, 1)
        .await
        .expect("message 1 sync");

    // Bob replies
    bob.call_tool(
        "send_message",
        json!({
            "entity_id": channel_id,
            "entity_type": "channel",
            "text": "Confirmed. Agenda sent to everyone."
        }),
    )
    .await
    .assert_success();

    verifier
        .wait_for_message_count(&channel_id, 2)
        .await
        .expect("message 2 sync");

    let bob_view = bob
        .call_tool(
            "list_messages",
            json!({
                "entity_id": channel_id,
                "limit": 5
            }),
        )
        .await;
    bob_view
        .assert_success()
        .assert_contains("Kick-off scheduled")
        .assert_contains("Agenda sent");

    // ---------------------------------------------------------------------
    // Kanban board collaboration
    // ---------------------------------------------------------------------
    let board = alice
        .call_tool(
            "create_kanban_board",
            json!({
                "entity_id": project_id,
                "board_name": "Launch Board"
            }),
        )
        .await;
    board.assert_success();
    let board_id = extract_id(&board).expect("board id");

    let todo_column = alice
        .call_tool(
            "create_kanban_column",
            json!({
                "board_id": board_id,
                "column_name": "To Do"
            }),
        )
        .await;
    todo_column.assert_success();
    let todo_col_id = extract_id(&todo_column).expect("column id");

    let card = alice
        .call_tool(
            "create_kanban_card",
            json!({
                "board_id": board_id,
                "column_id": todo_col_id,
                "title": "Prep press kit",
                "description": "Draft blog + release notes"
            }),
        )
        .await;
    card.assert_success();
    let card_id = extract_id(&card).expect("card id");

    verifier
        .wait_for_kanban_board_sync(&board_id)
        .await
        .expect("board sync");

    // Carol updates the card to demonstrate shared editing
    carol
        .call_tool(
            "update_kanban_card",
            json!({
                "board_id": board_id,
                "card_id": card_id,
                "title": "Prep launch press kit",
                "description": "Include screenshots + FAQ",
                "tags": ["launch", "press"]
            }),
        )
        .await
        .assert_success();

    // Confirm Alice sees Carol's edits
    let card_details = alice
        .call_tool(
            "get_kanban_card",
            json!({
                "board_id": board_id,
                "card_id": card_id
            }),
        )
        .await;
    card_details
        .assert_success()
        .assert_contains("Include screenshots")
        .assert_contains("launch");
}

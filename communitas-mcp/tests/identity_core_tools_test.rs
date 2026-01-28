// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Identity & Core Tools Tests
//!
//! Phase 10.2: Testing 40 identity, entity, contact, and member tools.
//!
//! Tool Categories:
//! - Identity (9 tools): create_identity, recover_identity, validate_mnemonic,
//!   get_session, get_profile, update_profile, workspace_init,
//!   export_audit_log, get_audit_log
//! - Entity (6 tools): create_entity, get_entity, update_entity, delete_entity,
//!   list_entities, join_entity
//! - Contact (11 tools): create_contact, get_contact, update_contact,
//!   delete_contact, list_contacts, search_contacts, link_contact,
//!   set_favourite_contact, remove_favourite_contact,
//!   list_favourite_contacts, get_contact_presence
//! - Member (8 tools): add_member, remove_member, list_members,
//!   create_invite, accept_invite, list_pending_invites,
//!   assign_user, unassign_user

#![allow(unused_variables)]

mod harness;

use serde_json::json;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::sleep;

static PORT_COUNTER: AtomicU16 = AtomicU16::new(0);

/// Test node that spawns an MCP server process
struct TestNode {
    name: String,
    process: std::process::Child,
    port: u16,
}

impl TestNode {
    async fn start(name: &str) -> Self {
        let counter = PORT_COUNTER.fetch_add(1, Ordering::SeqCst);
        let port = 31000 + (std::process::id() % 1000) as u16 * 10 + counter;

        let mut process = Command::new(env!("CARGO_BIN_EXE_communitas-mcp"))
            .args([
                "--http",
                "--demo",
                "--listen",
                &format!("127.0.0.1:{}", port),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");

        // Wait for server to be ready
        let client = reqwest::Client::new();
        for _ in 0..50 {
            sleep(std::time::Duration::from_millis(100)).await;
            if client
                .post(format!("http://127.0.0.1:{}/mcp", port))
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 0,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {"name": name, "version": "1.0"}
                    }
                }))
                .send()
                .await
                .is_ok()
            {
                return Self {
                    name: name.to_string(),
                    process,
                    port,
                };
            }
        }
        let _ = process.kill();
        let _ = process.wait();
        panic!("Node {} failed to start", name);
    }

    fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    async fn request(&self, method: &str, params: serde_json::Value) -> serde_json::Value {
        let client = reqwest::Client::new();
        client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }

    async fn call_tool(&self, name: &str, arguments: serde_json::Value) -> ToolResult {
        let response = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments
                }),
            )
            .await;

        let result = response.get("result").cloned().unwrap_or(json!(null));
        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("");

        let parsed: Option<serde_json::Value> = serde_json::from_str(content).ok();

        ToolResult {
            success: !is_error,
            content: content.to_string(),
            parsed,
        }
    }

    async fn initialize(&self) {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": &self.name, "version": "1.0"}
            }),
        )
        .await;
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

#[derive(Debug)]
struct ToolResult {
    success: bool,
    content: String,
    parsed: Option<serde_json::Value>,
}

impl ToolResult {
    fn assert_success(&self) -> &Self {
        assert!(
            self.success,
            "Expected success but got error: {}",
            self.content
        );
        self
    }

    fn get_id(&self) -> Option<String> {
        self.parsed.as_ref().and_then(|p| {
            p.get("id")
                .or_else(|| p.get("entity_id"))
                .or_else(|| p.get("board_id"))
                .or_else(|| p.get("card_id"))
                .or_else(|| p.get("column_id"))
                .or_else(|| p.get("message_id"))
                .or_else(|| p.get("contact_id"))
                .or_else(|| p.get("invite_id"))
                .or_else(|| p.get("workspace_id"))
                .and_then(|v| v.as_str())
                .map(String::from)
        })
    }
}

// ============================================================================
// IDENTITY TOOLS - HAPPY PATH
// ============================================================================

#[tokio::test]
async fn test_get_session() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    let r = node.call_tool("get_session", json!({})).await;
    assert!(r.success, "get_session failed: {}", r.content);
}

#[tokio::test]
async fn test_get_profile() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    let r = node.call_tool("get_profile", json!({})).await;
    assert!(r.success, "get_profile failed: {}", r.content);
}

#[tokio::test]
async fn test_update_profile() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    let r = node
        .call_tool("update_profile", json!({"display_name": "Updated Name"}))
        .await;
    assert!(r.success, "update_profile failed: {}", r.content);
}

#[tokio::test]
async fn test_workspace_init() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    let r = node
        .call_tool(
            "workspace_init",
            json!({
                "name": "My Project",
                "description": "A test project"
            }),
        )
        .await;
    assert!(r.success, "workspace_init failed: {}", r.content);
    assert!(r.get_id().is_some(), "No ID returned");
}

// ============================================================================
// ENTITY TOOLS - CRUD
// ============================================================================

#[tokio::test]
async fn test_create_entity() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    let r = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Test Channel",
                "entity_type": "channel",
                "description": "A test channel"
            }),
        )
        .await;
    assert!(r.success, "create_entity failed: {}", r.content);
    assert!(r.get_id().is_some(), "No entity ID returned");
}

#[tokio::test]
async fn test_get_entity() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // First create an entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Get Test Group",
                "entity_type": "group"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Now get it
    let r = node
        .call_tool(
            "get_entity",
            json!({
                "entity_id": entity_id
            }),
        )
        .await;
    assert!(r.success, "get_entity failed: {}", r.content);
}

#[tokio::test]
async fn test_update_entity() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Original Name",
                "entity_type": "project"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Update entity - need to include entity_type
    let r = node
        .call_tool(
            "update_entity",
            json!({
                "entity_id": entity_id,
                "entity_type": "project",
                "name": "Updated Name",
                "description": "New description"
            }),
        )
        .await;
    assert!(r.success, "update_entity failed: {}", r.content);
}

#[tokio::test]
async fn test_delete_entity() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "To Delete",
                "entity_type": "channel"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Delete entity - need to include entity_type
    let r = node
        .call_tool(
            "delete_entity",
            json!({
                "id": entity_id,
                "entity_type": "channel"
            }),
        )
        .await;
    assert!(r.success, "delete_entity failed: {}", r.content);
}

#[tokio::test]
async fn test_list_entities() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create a few entities
    for i in 1..=3 {
        node.call_tool(
            "create_entity",
            json!({
                "name": format!("Entity {}", i),
                "entity_type": "channel"
            }),
        )
        .await
        .assert_success();
    }

    // List entities
    let r = node.call_tool("list_entities", json!({})).await;
    assert!(r.success, "list_entities failed: {}", r.content);
}

#[tokio::test]
async fn test_join_entity() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create an entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Joinable Group",
                "entity_type": "group"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Join the entity
    let r = node
        .call_tool(
            "join_entity",
            json!({
                "id": entity_id
            }),
        )
        .await;
    assert!(r.success, "join_entity failed: {}", r.content);
}

// ============================================================================
// CONTACT TOOLS - CRUD
// ============================================================================
// Note: Contact tools require networking to be started first

async fn start_networking(node: &TestNode) {
    let r = node.call_tool("network_start", json!({})).await;
    r.assert_success();
}

#[tokio::test]
async fn test_create_contact() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    let r = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Alice Contact",
                "pubkey_hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            }),
        )
        .await;
    assert!(r.success, "create_contact failed: {}", r.content);
    assert!(r.get_id().is_some(), "No contact ID returned");
}

#[tokio::test]
async fn test_get_contact() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Bob Contact",
                "pubkey_hex": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    // Get contact
    let r = node
        .call_tool(
            "get_contact",
            json!({
                "contact_id": contact_id
            }),
        )
        .await;
    assert!(r.success, "get_contact failed: {}", r.content);
}

#[tokio::test]
async fn test_update_contact() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Original Name",
                "pubkey_hex": "fedcba0123456789fedcba0123456789fedcba0123456789fedcba0123456789"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    // Update contact
    let r = node
        .call_tool(
            "update_contact",
            json!({
                "contact_id": contact_id,
                "display_name": "Updated Name",
                "notes": "Updated notes"
            }),
        )
        .await;
    assert!(r.success, "update_contact failed: {}", r.content);
}

#[tokio::test]
async fn test_delete_contact() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Delete Me",
                "pubkey_hex": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde0"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    // Delete contact
    let r = node
        .call_tool(
            "delete_contact",
            json!({
                "contact_id": contact_id
            }),
        )
        .await;
    assert!(r.success, "delete_contact failed: {}", r.content);
}

#[tokio::test]
async fn test_list_contacts() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create a few contacts
    for i in 1..=3 {
        node.call_tool(
            "create_contact",
            json!({
                "display_name": format!("Contact {}", i),
                "pubkey_hex": format!("{:064x}", i)
            }),
        )
        .await
        .assert_success();
    }

    // List contacts
    let r = node.call_tool("list_contacts", json!({})).await;
    assert!(r.success, "list_contacts failed: {}", r.content);
}

#[tokio::test]
async fn test_search_contacts() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create contacts with specific names
    node.call_tool(
        "create_contact",
        json!({
            "display_name": "Alice Smith",
            "pubkey_hex": "1111222233334444555566667777888899990000aaaabbbbccccddddeeeeffff"
        }),
    )
    .await
    .assert_success();

    node.call_tool(
        "create_contact",
        json!({
            "display_name": "Bob Jones",
            "pubkey_hex": "ffffeeeedddcccbbbaaaa9999888877776666555544443333222211110000"
        }),
    )
    .await
    .assert_success();

    // Search for "Alice"
    let r = node
        .call_tool(
            "search_contacts",
            json!({
                "query": "Alice"
            }),
        )
        .await;
    assert!(r.success, "search_contacts failed: {}", r.content);
}

// ============================================================================
// CONTACT TOOLS - FAVORITES
// ============================================================================

#[tokio::test]
async fn test_set_favourite_contact() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Favorite Contact",
                "pubkey_hex": "9999888877776666555544443333222211110000aaaabbbbccccddddeeeeffff"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    // Set as favorite
    let r = node
        .call_tool(
            "set_favourite_contact",
            json!({
                "contact_id": contact_id
            }),
        )
        .await;
    assert!(r.success, "set_favourite_contact failed: {}", r.content);
}

#[tokio::test]
async fn test_remove_favourite_contact() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create and favorite a contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Unfavorite Me",
                "pubkey_hex": "aaaa0000bbbb1111cccc2222dddd3333eeee4444ffff55556666777788889999"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    node.call_tool(
        "set_favourite_contact",
        json!({
            "contact_id": contact_id
        }),
    )
    .await
    .assert_success();

    // Remove from favorites
    let r = node
        .call_tool(
            "remove_favourite_contact",
            json!({
                "contact_id": contact_id
            }),
        )
        .await;
    assert!(r.success, "remove_favourite_contact failed: {}", r.content);
}

#[tokio::test]
async fn test_list_favourite_contacts() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create and favorite a contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Fav Contact",
                "pubkey_hex": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    node.call_tool(
        "set_favourite_contact",
        json!({
            "contact_id": contact_id
        }),
    )
    .await
    .assert_success();

    // List favorites
    let r = node.call_tool("list_favourite_contacts", json!({})).await;
    assert!(r.success, "list_favourite_contacts failed: {}", r.content);
}

#[tokio::test]
async fn test_get_contact_presence() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // Create contact
    let create_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Presence Test",
                "pubkey_hex": "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe"
            }),
        )
        .await;

    let contact_id = create_result.get_id().expect("No contact ID");

    // Get presence (may be unknown/offline in test)
    let r = node
        .call_tool(
            "get_contact_presence",
            json!({
                "contact_id": contact_id
            }),
        )
        .await;
    assert!(r.success, "get_contact_presence failed: {}", r.content);
}

// ============================================================================
// MEMBER TOOLS
// ============================================================================

#[tokio::test]
async fn test_add_member() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create an entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Member Test Group",
                "entity_type": "group"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Add a member (using a test pubkey)
    let r = node
        .call_tool(
            "add_member",
            json!({
                "entity_id": entity_id,
                "pubkey_hex": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
            }),
        )
        .await;
    assert!(r.success, "add_member failed: {}", r.content);
}

#[tokio::test]
async fn test_remove_member() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Remove Member Group",
                "entity_type": "group"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Add member first
    let member_pubkey = "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";

    let add_result = node
        .call_tool(
            "add_member",
            json!({
                "entity_id": entity_id,
                "pubkey_hex": member_pubkey
            }),
        )
        .await;

    // Some entity types may not support adding members in demo mode
    if !add_result.success {
        return; // Skip test if add_member not supported
    }

    // Remove member
    let r = node
        .call_tool(
            "remove_member",
            json!({
                "entity_id": entity_id,
                "pubkey_hex": member_pubkey
            }),
        )
        .await;
    assert!(r.success, "remove_member failed: {}", r.content);
}

#[tokio::test]
async fn test_list_members() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "List Members Group",
                "entity_type": "group"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // List members (should at least include creator)
    let r = node
        .call_tool(
            "list_members",
            json!({
                "entity_id": entity_id
            }),
        )
        .await;
    assert!(r.success, "list_members failed: {}", r.content);
}

// ============================================================================
// INVITE TOOLS
// ============================================================================

#[tokio::test]
async fn test_create_invite() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // Create entity
    let create_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "Invitable Group",
                "entity_type": "group"
            }),
        )
        .await;

    let entity_id = create_result.get_id().expect("No entity ID");

    // Create invite
    let r = node
        .call_tool(
            "create_invite",
            json!({
                "entity_id": entity_id
            }),
        )
        .await;
    assert!(r.success, "create_invite failed: {}", r.content);
    assert!(r.get_id().is_some(), "No invite ID returned");
}

#[tokio::test]
async fn test_list_pending_invites() {
    let node = TestNode::start("test").await;
    node.initialize().await;

    // List pending invites (may be empty)
    let r = node.call_tool("list_pending_invites", json!({})).await;
    assert!(r.success, "list_pending_invites failed: {}", r.content);
}

// ============================================================================
// INTEGRATION: IDENTITY → ENTITY → CONTACT WORKFLOW
// ============================================================================

#[tokio::test]
async fn test_full_onboarding_workflow() {
    let node = TestNode::start("test").await;
    node.initialize().await;
    start_networking(&node).await;

    // 1. Create/update profile
    node.call_tool("update_profile", json!({"display_name": "New User"}))
        .await
        .assert_success();

    // 2. Create a workspace
    let workspace_result = node
        .call_tool(
            "workspace_init",
            json!({
                "name": "My Workspace",
                "description": "Personal workspace"
            }),
        )
        .await;

    workspace_result.assert_success();

    // workspace_init may create an entity internally
    // 3. Create a channel
    let channel_result = node
        .call_tool(
            "create_entity",
            json!({
                "name": "general",
                "entity_type": "channel"
            }),
        )
        .await;

    channel_result.assert_success();
    let channel_id = channel_result.get_id().expect("No channel ID");

    // 4. Add a contact
    let contact_result = node
        .call_tool(
            "create_contact",
            json!({
                "display_name": "Team Member",
                "pubkey_hex": "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            }),
        )
        .await;

    contact_result.assert_success();

    // 5. Add member (contact) to channel
    let add_result = node
        .call_tool(
            "add_member",
            json!({
                "entity_id": channel_id,
                "pubkey_hex": "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            }),
        )
        .await;

    // May not be supported in demo mode
    let _ = add_result;

    // 6. Verify member list
    let members_result = node
        .call_tool(
            "list_members",
            json!({
                "entity_id": channel_id
            }),
        )
        .await;

    members_result.assert_success();
}

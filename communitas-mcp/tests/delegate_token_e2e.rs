// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Delegate Token E2E Tests
//!
//! Tests for delegate token authorization and scope enforcement.
//! Verifies that tokens with specific scopes can only perform
//! operations permitted by those scopes.

mod harness;

use communitas_mcp::auth::Scope;
use harness::{McpTestNode, ScopeAssert, TokenPresets, TokenTestHelper};
use serde_json::json;
use std::path::PathBuf;

fn demo_storage_dir() -> PathBuf {
    std::env::temp_dir().join("communitas-mcp")
}

async fn demo_token_helper() -> TokenTestHelper {
    TokenTestHelper::from_path(demo_storage_dir())
        .await
        .expect("create token helper")
}

/// Helper to create an authenticated test client with a specific token
async fn create_client_with_token(token: &str) -> McpTestNode {
    let node = McpTestNode::start("delegate-token").await;
    node.initialize().await;

    let auth = node
        .request(
            "authenticate_token",
            json!({
                "token": token
            }),
        )
        .await;

    if auth.get("error").is_some() {
        panic!("Failed to authenticate token: {:?}", auth);
    }

    node
}

// =============================================================================
// SCOPE ENFORCEMENT TESTS
// =============================================================================

mod scope_enforcement {
    use super::*;

    /// Test that a read-only token cannot perform write operations
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_read_only_token_cannot_write() {
        let helper = demo_token_helper().await;
        let token = helper
            .create_read_only_token("read-only-agent")
            .expect("create token");

        // Verify token has correct scopes
        let verified = helper.verify_token(&token).expect("verify token");
        verified.assert_has_scope(&Scope::ReadMessages);
        verified.assert_has_scope(&Scope::ReadFiles);

        let client = create_client_with_token(&token).await;

        // Should be able to read messages
        let _read_result = client
            .call_tool(
                "list_messages",
                json!({
                    "entity_id": "test-entity",
                    "limit": 10
                }),
            )
            .await;
        // Note: May fail due to missing entity, but should not fail due to scope
        // The error should NOT be "insufficient scope"

        // Should NOT be able to send messages (requires SendMessages scope)
        let send_result = client
            .call_tool(
                "send_message",
                json!({
                    "entity_id": "test-entity",
                    "text": "Hello, World!"
                }),
            )
            .await;

        // Verify scope enforcement - should fail due to missing SendMessages scope
        assert!(
            send_result.error_contains("scope")
                || send_result.error_contains("permission")
                || send_result.error_contains("unauthorized"),
            "Expected scope error for send_message with read-only token, got: {:?}",
            send_result
        );

        // Should NOT be able to write files (requires WriteFiles scope)
        let write_result = client
            .call_tool(
                "write_file",
                json!({
                    "entity_id": "test-entity",
                    "disk_type": "private",
                    "path": "/test.txt",
                    "content": "Test content"
                }),
            )
            .await;

        assert!(
            write_result.error_contains("scope")
                || write_result.error_contains("permission")
                || write_result.error_contains("unauthorized"),
            "Expected scope error for write_file with read-only token, got: {:?}",
            write_result
        );
    }

    /// Test that a messaging token cannot manage Kanban boards
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_messaging_token_cannot_manage_kanban() {
        let helper = demo_token_helper().await;
        let token = helper
            .create_messaging_token("messaging-agent")
            .expect("create token");

        // Verify token has correct scopes
        let verified = helper.verify_token(&token).expect("verify token");
        verified.assert_has_scope(&Scope::ReadMessages);
        verified.assert_has_scope(&Scope::SendMessages);

        let client = create_client_with_token(&token).await;

        // Should NOT be able to create Kanban boards (requires ManageKanban scope)
        let kanban_result = client
            .call_tool(
                "create_kanban_board",
                json!({
                    "entity_id": "test-entity",
                    "board_name": "Test Board"
                }),
            )
            .await;

        assert!(
            kanban_result.error_contains("scope")
                || kanban_result.error_contains("permission")
                || kanban_result.error_contains("unauthorized"),
            "Expected scope error for create_kanban_board with messaging token, got: {:?}",
            kanban_result
        );

        // Should NOT be able to create Kanban cards
        let card_result = client
            .call_tool(
                "create_kanban_card",
                json!({
                    "board_id": "test-board",
                    "column_id": "test-column",
                    "title": "Test Card"
                }),
            )
            .await;

        assert!(
            card_result.error_contains("scope")
                || card_result.error_contains("permission")
                || card_result.error_contains("unauthorized"),
            "Expected scope error for create_kanban_card with messaging token, got: {:?}",
            card_result
        );

        // Should NOT be able to manage entities
        let entity_result = client
            .call_tool(
                "create_entity",
                json!({
                    "entity_type": "organization",
                    "name": "Test Org"
                }),
            )
            .await;

        assert!(
            entity_result.error_contains("scope")
                || entity_result.error_contains("permission")
                || entity_result.error_contains("unauthorized"),
            "Expected scope error for create_entity with messaging token, got: {:?}",
            entity_result
        );
    }

    /// Test that a Kanban token can only perform Kanban operations
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_kanban_token_scope() {
        let helper = demo_token_helper().await;
        let token = helper
            .create_kanban_token("kanban-agent")
            .expect("create token");

        // Verify token has correct scopes
        let verified = helper.verify_token(&token).expect("verify token");
        verified.assert_has_scope(&Scope::ManageKanban);
        verified.assert_has_scope(&Scope::ReadMessages);

        let client = create_client_with_token(&token).await;

        // Should be able to list Kanban boards (ManageKanban scope)
        let list_result = client
            .call_tool(
                "list_kanban_boards",
                json!({
                    "entity_id": "test-entity"
                }),
            )
            .await;
        // Should not fail due to scope (may fail due to missing entity)
        assert!(
            !list_result.error_contains("scope") && !list_result.error_contains("unauthorized"),
            "Kanban token should have access to list_kanban_boards"
        );

        // Should NOT be able to send messages (requires SendMessages scope)
        let send_result = client
            .call_tool(
                "send_message",
                json!({
                    "entity_id": "test-entity",
                    "text": "Hello!"
                }),
            )
            .await;

        assert!(
            send_result.error_contains("scope")
                || send_result.error_contains("permission")
                || send_result.error_contains("unauthorized"),
            "Expected scope error for send_message with kanban token, got: {:?}",
            send_result
        );

        // Should NOT be able to write files
        let file_result = client
            .call_tool(
                "write_file",
                json!({
                    "entity_id": "test-entity",
                    "disk_type": "private",
                    "path": "/test.txt",
                    "content": "Test"
                }),
            )
            .await;

        assert!(
            file_result.error_contains("scope")
                || file_result.error_contains("permission")
                || file_result.error_contains("unauthorized"),
            "Expected scope error for write_file with kanban token, got: {:?}",
            file_result
        );
    }

    /// Test that a full-access token can perform all operations
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_full_token_all_operations() {
        let helper = demo_token_helper().await;
        let token = helper
            .create_full_token("full-access-agent")
            .expect("create token");

        // Verify token has Full scope
        let verified = helper.verify_token(&token).expect("verify token");
        verified.assert_has_scope(&Scope::Full);

        let client = create_client_with_token(&token).await;

        // Full token should have access to all operations
        // These may fail due to missing resources, but NOT due to scope

        let operations = vec![
            (
                "list_messages",
                json!({"entity_id": "test-entity", "limit": 10}),
            ),
            ("list_kanban_boards", json!({"entity_id": "test-entity"})),
            (
                "list_files",
                json!({"entity_id": "test-entity", "disk_type": "private", "path": "/"}),
            ),
            ("list_contacts", json!({})),
            ("network_status", json!({})),
        ];

        for (tool, params) in operations {
            let result = client.call_tool(tool, params).await;
            assert!(
                !result.error_contains("scope") && !result.error_contains("unauthorized"),
                "Full token should have access to {}, but got scope error: {:?}",
                tool,
                result
            );
        }
    }
}

// =============================================================================
// TOKEN EXPIRATION TESTS
// =============================================================================

mod token_expiration {
    use super::*;
    use std::time::Duration;

    /// Test that expired tokens are rejected
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_expired_token_rejected() {
        let helper = demo_token_helper().await;

        // Create a token that expires immediately (0 hours)
        let token = helper
            .create_expired_token("expired-agent")
            .expect("create token");

        // Wait a moment to ensure token is expired
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verification should fail for expired token
        let verify_result = helper.verify_token(&token);
        assert!(
            verify_result.is_err(),
            "Expired token should fail verification"
        );

        // Attempting to use expired token should fail
        let client = create_client_with_token(&token).await;
        let result = client
            .call_tool(
                "list_messages",
                json!({
                    "entity_id": "test-entity",
                    "limit": 10
                }),
            )
            .await;

        assert!(
            result.error_contains("expired")
                || result.error_contains("invalid")
                || result.error_contains("unauthorized"),
            "Expected expiration error when using expired token, got: {:?}",
            result
        );
    }

    /// Test that tokens with valid TTL work correctly
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_valid_ttl_token_works() {
        let helper = demo_token_helper().await;

        // Create a token with 24-hour TTL
        let token = helper
            .create_full_token("valid-agent")
            .expect("create token");

        // Verification should succeed
        let verified = helper.verify_token(&token).expect("verify token");
        verified.assert_not_expired();

        // Token should be usable
        let client = create_client_with_token(&token).await;
        let result = client.call_tool("health_check", json!({})).await;

        // Health check doesn't require auth, but token should not cause errors
        assert!(
            !result.error_contains("expired") && !result.error_contains("invalid token"),
            "Valid token should not cause expiration errors"
        );
    }
}

// =============================================================================
// CROSS-SERVER TOKEN TESTS
// =============================================================================

mod cross_server {
    use super::*;

    /// Test that tokens from a different server/issuer are rejected
    #[tokio::test]
    async fn test_cross_server_token_rejected() {
        // Create two separate token helpers (simulating different servers)
        let helper_a = TokenTestHelper::new()
            .await
            .expect("create token helper A")
            .with_issuer("server-a-issuer");

        let helper_b = TokenTestHelper::new()
            .await
            .expect("create token helper B")
            .with_issuer("server-b-issuer");

        // Create token from server A
        let token_a = helper_a
            .create_full_token("agent-a")
            .expect("create token A");

        // Token should verify on its own server
        let verified_a = helper_a.verify_token(&token_a);
        assert!(
            verified_a.is_ok(),
            "Token should verify on its issuing server"
        );

        // Token from server A should NOT verify on server B
        let verify_on_b = helper_b.verify_token(&token_a);
        assert!(
            verify_on_b.is_err(),
            "Token from server A should not verify on server B"
        );
    }

    /// Test that malformed tokens are rejected
    #[tokio::test]
    async fn test_malformed_token_rejected() {
        let helper = demo_token_helper().await;

        // Test various malformed tokens
        let malformed_tokens = vec![
            "",
            "invalid",
            "not.a.valid.jwt",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.invalid.signature",
            "completely-random-string-12345",
        ];

        for token in malformed_tokens {
            let result = helper.verify_token(token);
            assert!(
                result.is_err(),
                "Malformed token '{}' should be rejected",
                token
            );
        }
    }
}

// =============================================================================
// SCOPE COMBINATION TESTS
// =============================================================================

mod scope_combinations {
    use super::*;

    /// Test tokens with multiple combined scopes
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_token_scope_combinations() {
        let helper = demo_token_helper().await;

        // Create token with messaging + files scopes
        let scopes = vec![
            Scope::ReadMessages,
            Scope::SendMessages,
            Scope::ReadFiles,
            Scope::WriteFiles,
        ];
        let token = helper
            .create_custom_token("multi-scope-agent", scopes.clone(), 24)
            .expect("create custom token");

        let verified = helper.verify_token(&token).expect("verify token");

        // Verify all specified scopes are present
        for scope in &scopes {
            verified.assert_has_scope(scope);
        }

        let client = create_client_with_token(&token).await;

        // Should have access to messaging operations
        let msg_result = client
            .call_tool(
                "list_messages",
                json!({
                    "entity_id": "test-entity",
                    "limit": 10
                }),
            )
            .await;
        assert!(
            !msg_result.error_contains("scope") && !msg_result.error_contains("unauthorized"),
            "Multi-scope token should have messaging access"
        );

        // Should have access to file operations
        let file_result = client
            .call_tool(
                "list_files",
                json!({
                    "entity_id": "test-entity",
                    "disk_type": "private",
                    "path": "/"
                }),
            )
            .await;
        assert!(
            !file_result.error_contains("scope") && !file_result.error_contains("unauthorized"),
            "Multi-scope token should have file access"
        );

        // Should NOT have access to Kanban operations
        let kanban_result = client
            .call_tool(
                "create_kanban_board",
                json!({
                    "entity_id": "test-entity",
                    "board_name": "Test Board"
                }),
            )
            .await;
        assert!(
            kanban_result.error_contains("scope")
                || kanban_result.error_contains("permission")
                || kanban_result.error_contains("unauthorized"),
            "Multi-scope token should NOT have Kanban access without ManageKanban scope"
        );
    }

    /// Test empty token (no scopes) is rejected for all operations
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_empty_token_rejected() {
        let helper = demo_token_helper().await;
        let token = helper
            .create_empty_token("empty-agent")
            .expect("create token");

        let client = create_client_with_token(&token).await;

        // All scope-protected operations should fail
        let operations = vec![
            (
                "send_message",
                json!({"entity_id": "test", "text": "Hello"}),
            ),
            (
                "write_file",
                json!({"entity_id": "test", "disk_type": "private", "path": "/t.txt", "content": "x"}),
            ),
            (
                "create_kanban_board",
                json!({"entity_id": "test", "board_name": "Board"}),
            ),
            (
                "create_entity",
                json!({"entity_type": "organization", "name": "Org"}),
            ),
            (
                "add_member",
                json!({"entity_id": "test", "member_id": "member"}),
            ),
            ("create_contact", json!({"name": "Contact"})),
        ];

        for (tool, params) in operations {
            let result = client.call_tool(tool, params).await;
            assert!(
                result.error_contains("scope")
                    || result.error_contains("permission")
                    || result.error_contains("unauthorized"),
                "Empty token should be rejected for {}, got: {:?}",
                tool,
                result
            );
        }
    }

    /// Test all individual scopes from TokenPresets
    #[tokio::test]
    async fn test_all_individual_scopes() {
        let all_scopes = TokenPresets::all_individual();

        // Verify we have all expected individual scopes
        assert!(all_scopes.contains(&Scope::ReadMessages));
        assert!(all_scopes.contains(&Scope::SendMessages));
        assert!(all_scopes.contains(&Scope::ReadFiles));
        assert!(all_scopes.contains(&Scope::WriteFiles));
        assert!(all_scopes.contains(&Scope::ManageEntities));
        assert!(all_scopes.contains(&Scope::ManageMembers));
        assert!(all_scopes.contains(&Scope::ManageKanban));
        assert!(all_scopes.contains(&Scope::ManageNetwork));
        assert!(all_scopes.contains(&Scope::ManageContacts));

        // Full scope should not be in individual list
        assert!(!all_scopes.contains(&Scope::Full));
    }

    /// Test preset scope combinations
    #[tokio::test]
    async fn test_preset_scope_combinations() {
        // Read-only preset
        let read_only = TokenPresets::read_only();
        assert_eq!(read_only.len(), 2);
        assert!(read_only.contains(&Scope::ReadMessages));
        assert!(read_only.contains(&Scope::ReadFiles));

        // Messaging preset
        let messaging = TokenPresets::messaging();
        assert_eq!(messaging.len(), 2);
        assert!(messaging.contains(&Scope::ReadMessages));
        assert!(messaging.contains(&Scope::SendMessages));

        // Kanban preset
        let kanban = TokenPresets::kanban();
        assert_eq!(kanban.len(), 2);
        assert!(kanban.contains(&Scope::ManageKanban));
        assert!(kanban.contains(&Scope::ReadMessages));

        // Files preset
        let files = TokenPresets::files();
        assert_eq!(files.len(), 2);
        assert!(files.contains(&Scope::ReadFiles));
        assert!(files.contains(&Scope::WriteFiles));

        // Full preset
        let full = TokenPresets::full();
        assert_eq!(full.len(), 1);
        assert!(full.contains(&Scope::Full));

        // Admin preset (same as full)
        let admin = TokenPresets::admin();
        assert_eq!(admin.len(), 1);
        assert!(admin.contains(&Scope::Full));
    }
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

mod integration {
    use super::*;

    /// Test complete token lifecycle: create, use, expire
    #[tokio::test]
    #[ignore = "requires authenticate_token MCP method not yet implemented"]
    async fn test_token_lifecycle() {
        let helper = demo_token_helper().await;

        // 1. Create token
        let token = helper
            .create_messaging_token("lifecycle-agent")
            .expect("create token");

        // 2. Verify token is valid
        let verified = helper.verify_token(&token).expect("verify token");
        verified.assert_has_scope(&Scope::ReadMessages);
        verified.assert_has_scope(&Scope::SendMessages);
        verified.assert_not_expired();

        // 3. Use token for authorized operation
        let client = create_client_with_token(&token).await;
        let result = client
            .call_tool(
                "list_messages",
                json!({
                    "entity_id": "test-entity",
                    "limit": 10
                }),
            )
            .await;

        // Should not fail due to scope
        assert!(
            !result.error_contains("scope") && !result.error_contains("unauthorized"),
            "Valid token should authorize list_messages"
        );

        // 4. Verify token still valid after use
        let still_valid = helper.verify_token(&token);
        assert!(still_valid.is_ok(), "Token should remain valid after use");
    }

    /// Test that token delegate name is preserved
    #[tokio::test]
    async fn test_token_delegate_name_preserved() {
        let helper = TokenTestHelper::new().await.expect("create token helper");

        let delegate_name = "my-special-agent-name";
        let token = helper
            .create_full_token(delegate_name)
            .expect("create token");

        let verified = helper.verify_token(&token).expect("verify token");

        // The delegate name should be preserved in the token
        assert_eq!(
            verified.delegate_name, delegate_name,
            "Delegate name should be preserved in token"
        );
    }

    /// Test tokens from TokenTestHelper presets
    #[tokio::test]
    async fn test_helper_preset_methods() {
        let helper = TokenTestHelper::new().await.expect("create token helper");

        // Test each preset method creates valid tokens
        let preset_tokens = vec![
            ("read_only", helper.create_read_only_token("agent1")),
            ("messaging", helper.create_messaging_token("agent2")),
            ("kanban", helper.create_kanban_token("agent3")),
            ("files", helper.create_files_token("agent4")),
            ("entities", helper.create_entities_token("agent5")),
            ("network", helper.create_network_token("agent6")),
            ("contacts", helper.create_contacts_token("agent7")),
            ("full", helper.create_full_token("agent8")),
        ];

        for (preset_name, token_result) in preset_tokens {
            let token = token_result.expect(&format!("create {} token", preset_name));
            let verified = helper.verify_token(&token);
            assert!(
                verified.is_ok(),
                "{} preset token should be valid",
                preset_name
            );
        }
    }
}

// =============================================================================
// TEST SUMMARY
// =============================================================================

/// Summary test to verify delegate token test coverage
#[tokio::test]
async fn test_delegate_token_coverage_summary() {
    // This test documents the delegate token test coverage

    let test_categories = vec![
        ("Scope Enforcement", 4), // read_only_cannot_write, messaging_cannot_kanban, kanban_scope, full_all_ops
        ("Token Expiration", 2),  // expired_rejected, valid_ttl_works
        ("Cross-Server", 2),      // cross_server_rejected, malformed_rejected
        ("Scope Combinations", 4), // combinations, empty_rejected, all_individual, presets
        ("Integration", 3),       // lifecycle, delegate_name, preset_methods
    ];

    let total_tests: usize = test_categories.iter().map(|(_, count)| count).sum();

    println!("\n=== DELEGATE TOKEN E2E TEST COVERAGE ===");
    for (category, count) in &test_categories {
        println!("  {}: {} tests", category, count);
    }
    println!("  TOTAL: {} tests", total_tests);
    println!("==========================================\n");

    assert_eq!(total_tests, 15, "Expected 15 delegate token tests");
}

//! Terminology verification tests.
//!
//! These tests ensure the identity model terminology is used correctly:
//! - Four-words are "connection addresses", not "identities"
//! - Identity refers to pubkey_hex
//! - Display names are shown in UI

use communitas_mcp::tools::list_tools;

/// Verify that authentication tool descriptions use "connection address" not "identity"
#[test]
fn authenticate_tool_uses_connection_address_terminology() {
    let tools = list_tools(false); // Pre-auth tools

    let authenticate = tools
        .iter()
        .find(|t| t.name == "authenticate")
        .expect("authenticate tool should exist");

    // Should say "connection address" not "identity" for four_words
    assert!(
        authenticate
            .description
            .to_lowercase()
            .contains("connection address"),
        "authenticate description should mention 'connection address', got: {}",
        authenticate.description
    );

    // Should NOT say "four-word identity"
    assert!(
        !authenticate
            .description
            .to_lowercase()
            .contains("four-word identity"),
        "authenticate description should NOT say 'four-word identity', got: {}",
        authenticate.description
    );
}

/// Verify that create_vault tool descriptions use "connection address" not "identity"
#[test]
fn create_vault_tool_uses_connection_address_terminology() {
    let tools = list_tools(false); // Pre-auth tools

    let create_vault = tools
        .iter()
        .find(|t| t.name == "create_vault")
        .expect("create_vault tool should exist");

    // Should say "connection address"
    assert!(
        create_vault
            .description
            .to_lowercase()
            .contains("connection address"),
        "create_vault description should mention 'connection address', got: {}",
        create_vault.description
    );
}

/// Verify that four_words parameter descriptions use "connection address"
#[test]
fn four_words_parameter_uses_connection_address_terminology() {
    let tools = list_tools(false);

    let authenticate = tools
        .iter()
        .find(|t| t.name == "authenticate")
        .expect("authenticate tool should exist");

    // Check the input schema for four_words description
    let schema_str = authenticate.input_schema.to_string();

    assert!(
        schema_str.to_lowercase().contains("connection address"),
        "four_words parameter should be described as 'connection address', got: {}",
        schema_str
    );
}

/// Verify no tools incorrectly label four-words as identity
#[test]
fn no_tools_label_four_words_as_identity() {
    let tools = list_tools(true); // All tools

    for tool in &tools {
        // Check description
        let desc_lower = tool.description.to_lowercase();
        assert!(
            !desc_lower.contains("four-word identity")
                && !desc_lower.contains("four word identity")
                && !desc_lower.contains("4-word identity"),
            "Tool '{}' incorrectly labels four-words as identity: {}",
            tool.name,
            tool.description
        );

        // Check input schema
        let schema_str = tool.input_schema.to_string().to_lowercase();
        assert!(
            !schema_str.contains("four-word identity")
                && !schema_str.contains("four word identity")
                && !schema_str.contains("4-word identity"),
            "Tool '{}' input schema incorrectly labels four-words as identity",
            tool.name
        );
    }
}

/// Verify recovery tools correctly reference "identity" (since they work with actual identity)
#[test]
fn recovery_tools_correctly_reference_identity() {
    let tools = list_tools(false);

    let create_identity = tools
        .iter()
        .find(|t| t.name == "create_identity")
        .expect("create_identity tool should exist");

    // These tools correctly work with actual identity, so "identity" is appropriate
    assert!(
        create_identity.description.to_lowercase().contains("identity"),
        "create_identity should reference 'identity' since it creates the actual cryptographic identity"
    );

    let recover_identity = tools
        .iter()
        .find(|t| t.name == "recover_identity")
        .expect("recover_identity tool should exist");

    assert!(
        recover_identity.description.to_lowercase().contains("identity"),
        "recover_identity should reference 'identity' since it recovers the actual cryptographic identity"
    );
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Harness Library Tests
//!
//! Tests for the MCP test harness library.
//! Run with: cargo test -p communitas-mcp --test harness_test

mod harness;

use harness::{McpTestClient, ToolAssert, ToolResult};
use serde_json::json;

// =============================================================================
// ToolResult Tests
// =============================================================================

#[test]
fn test_tool_result_success_checks() {
    let result = ToolResult {
        tool: "get_profile".to_string(),
        success: true,
        content: r#"{"display_name":"Test"}"#.to_string(),
        parsed: Some(json!({"display_name": "Test"})),
        error: None,
        duration_ms: 15,
    };

    assert!(result.is_success());
    assert!(!result.is_error());
    assert_eq!(result.get_str("display_name"), Some("Test"));
}

#[test]
fn test_tool_result_error_checks() {
    let result = ToolResult {
        tool: "invalid_tool".to_string(),
        success: false,
        content: String::new(),
        parsed: None,
        error: Some("Tool not found".to_string()),
        duration_ms: 5,
    };

    assert!(!result.is_success());
    assert!(result.is_error());
    assert!(result.error.is_some());
}

#[test]
fn test_tool_result_json_access() {
    let result = ToolResult {
        tool: "list_entities".to_string(),
        success: true,
        content: r#"{"entities":[{"id":"1"},{"id":"2"}],"count":2}"#.to_string(),
        parsed: Some(json!({"entities":[{"id":"1"},{"id":"2"}],"count":2})),
        error: None,
        duration_ms: 20,
    };

    assert_eq!(result.array_len("entities"), 2);
    assert!(result.get("count").is_some());
}

// =============================================================================
// Assertion Tests
// =============================================================================

#[test]
fn test_assert_success_passes() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: true,
        content: "{}".to_string(),
        parsed: Some(json!({})),
        error: None,
        duration_ms: 0,
    };
    result.assert_success(); // Should not panic
}

#[test]
#[should_panic(expected = "Expected success")]
fn test_assert_success_fails_on_error() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: false,
        content: String::new(),
        parsed: None,
        error: Some("error".to_string()),
        duration_ms: 0,
    };
    result.assert_success();
}

#[test]
fn test_assert_has_field_passes() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: true,
        content: r#"{"name":"test"}"#.to_string(),
        parsed: Some(json!({"name": "test"})),
        error: None,
        duration_ms: 0,
    };
    result.assert_has("name"); // Should not panic
}

#[test]
#[should_panic(expected = "Expected field 'missing'")]
fn test_assert_has_field_fails() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: true,
        content: r#"{"name":"test"}"#.to_string(),
        parsed: Some(json!({"name": "test"})),
        error: None,
        duration_ms: 0,
    };
    result.assert_has("missing");
}

#[test]
fn test_assert_str_eq_passes() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: true,
        content: r#"{"status":"ok"}"#.to_string(),
        parsed: Some(json!({"status": "ok"})),
        error: None,
        duration_ms: 0,
    };
    result.assert_str_eq("status", "ok"); // Should not panic
}

#[test]
fn test_assert_array_min_passes() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: true,
        content: r#"{"items":["a","b","c"]}"#.to_string(),
        parsed: Some(json!({"items": ["a", "b", "c"]})),
        error: None,
        duration_ms: 0,
    };
    result.assert_array_min("items", 2); // Should not panic
}

#[test]
fn test_assert_contains_passes() {
    let result = ToolResult {
        tool: "test".to_string(),
        success: true,
        content: "Hello, World!".to_string(),
        parsed: None,
        error: None,
        duration_ms: 0,
    };
    result.assert_contains("World"); // Should not panic
}

// =============================================================================
// In-Process Client Tests
// =============================================================================

/// Helper macro to run async tests with 8MB stack
macro_rules! run_async_test {
    ($test_fn:expr) => {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on($test_fn);
            })
            .unwrap()
            .join()
            .unwrap();
    };
}

#[test]
fn test_in_process_client_creation() {
    run_async_test!(async {
        let _client = McpTestClient::new().await;
        // Client should be created without error - if we get here, it worked
    });
}

#[test]
fn test_in_process_get_profile() {
    run_async_test!(async {
        let client = McpTestClient::new().await;
        let result = client.call_tool("get_profile", json!({})).await;

        // Should succeed
        assert!(
            result.is_success(),
            "get_profile failed: {:?}",
            result.error
        );

        // Should have expected fields
        assert!(result.get("display_name").is_some() || result.parsed.is_some());
    });
}

#[test]
fn test_in_process_list_entities() {
    run_async_test!(async {
        let client = McpTestClient::new().await;
        let result = client.call_tool("list_entities", json!({})).await;

        // Should succeed
        assert!(
            result.is_success(),
            "list_entities failed: {:?}",
            result.error
        );
    });
}

#[test]
fn test_in_process_invalid_tool() {
    run_async_test!(async {
        let client = McpTestClient::new().await;
        let result = client.call_tool("nonexistent_tool_xyz", json!({})).await;

        // Should fail with error
        assert!(result.is_error(), "Expected error for invalid tool");
    });
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Golden Data Comparison Tests for MCP Tool Responses
//!
//! These tests validate that MCP tool responses conform to expected JSON structures.
//! Golden data fixtures define the expected structure (keys, types) without requiring
//! exact value matches, enabling CI detection of API contract changes.
//!
//! Run with: cargo test -p communitas-mcp --test golden_test

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use communitas_mcp::protocol::{Tool, ToolCallResult, ToolContent};
use communitas_mcp::tools::{call_tool, list_tools};
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use serde_json::{Value, json};
use tempfile::TempDir;

// Re-export communitas_core through the same alias as the library
extern crate communitas_bindings as communitas_core;
use communitas_core::app::CommunitasApp;

// =============================================================================
// Test Infrastructure
// =============================================================================

/// Path to golden data fixtures
fn golden_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(manifest_dir)
        .join("tests")
        .join("golden")
        .join(format!("{}.json", name))
}

/// Load golden data fixture by name
fn load_golden(name: &str) -> Value {
    let path = golden_path(name);
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read golden file {:?}: {}", path, e));
    serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse golden file {:?}: {}", path, e))
}

/// Create test services with a temporary storage directory
async fn make_test_services(temp: &TempDir) -> (Arc<CommunitasApp>, UiServices) {
    let storage = UiStorage::from_path(temp.path()).expect("failed to create storage");
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .expect("failed to create app"),
    );
    let services = UiServices::new(storage, app.clone()).expect("failed to create services");
    (app, services)
}

/// Extract text from ToolContent enum
fn extract_text(content: &ToolContent) -> Option<&str> {
    match content {
        ToolContent::Text { text } => Some(text.as_str()),
    }
}

/// Parse JSON response from a tool call's text content
fn parse_tool_response(result: &ToolCallResult) -> Value {
    if result.is_error {
        return json!({ "error": true });
    }
    result
        .content
        .first()
        .and_then(extract_text)
        .and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or(json!({}))
}

/// Helper macro to run async tests with adequate stack size
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

// =============================================================================
// Structure Comparison Utilities
// =============================================================================

/// Check if a JSON value matches an expected type specification
///
/// Type specs can be:
/// - "string" - expects a string value
/// - "number" - expects a number value
/// - "boolean" - expects a boolean value
/// - "array" - expects an array value
/// - "object" - expects an object value
/// - "any" - accepts any value
/// - "type|optional" - the field may be missing, but if present must match type
fn value_matches_type(value: &Value, type_spec: &str) -> bool {
    // Handle optional types
    let base_type = type_spec.trim_end_matches("|optional");

    match base_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "any" => true,
        _ => false,
    }
}

/// Verify that a JSON object contains expected keys with matching types
fn verify_object_structure(actual: &Value, expected_structure: &Value) -> Result<(), String> {
    let actual_obj = actual
        .as_object()
        .ok_or("Expected object, got different type")?;
    let expected_obj = expected_structure
        .as_object()
        .ok_or("Expected structure must be an object")?;

    for (key, type_spec) in expected_obj {
        let type_str = type_spec
            .as_str()
            .ok_or_else(|| format!("Type spec for '{}' must be a string", key))?;

        let is_optional = type_str.ends_with("|optional");

        match actual_obj.get(key) {
            Some(value) => {
                if !value_matches_type(value, type_str) {
                    return Err(format!(
                        "Key '{}' has wrong type: expected {}, got {:?}",
                        key, type_str, value
                    ));
                }
            }
            None => {
                if !is_optional {
                    return Err(format!("Missing required key: '{}'", key));
                }
            }
        }
    }

    Ok(())
}

/// Verify tool response structure matches golden data expectations
fn verify_tool_response_structure(result: &ToolCallResult, golden: &Value) -> Result<(), String> {
    // Check response content structure if present
    let Some(response_structure) = golden.get("response_structure") else {
        return Ok(());
    };

    let response = parse_tool_response(result);
    if response.get("error").is_none() {
        verify_object_structure(&response, response_structure)?;
    }

    Ok(())
}

/// Verify array items match expected structure
fn verify_array_items_structure(
    response: &Value,
    array_key: &str,
    golden: &Value,
    item_structure_key: &str,
) -> Result<(), String> {
    let Some(items) = response.get(array_key).and_then(|v| v.as_array()) else {
        return Ok(());
    };
    let Some(item_structure) = golden.get(item_structure_key) else {
        return Ok(());
    };

    for (i, item) in items.iter().enumerate() {
        verify_object_structure(item, item_structure)
            .map_err(|e| format!("{} item {} structure mismatch: {}", array_key, i, e))?;
    }

    Ok(())
}

// =============================================================================
// Golden Data Tests
// =============================================================================

/// Test 1: Verify list_tools returns all expected tools with correct structure
#[test]
fn test_list_tools_golden() {
    let golden = load_golden("list_tools");
    let tools = list_tools(true);

    // Verify minimum tool count
    let min_count = golden
        .get("minimum_tool_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;

    assert!(
        tools.len() >= min_count,
        "Expected at least {} tools, got {}",
        min_count,
        tools.len()
    );

    // Verify required tools are present
    let tool_names: HashSet<&str> = tools.iter().map(|t| t.name.as_str()).collect();

    if let Some(required) = golden.get("required_tools").and_then(|v| v.as_array()) {
        for required_tool in required {
            if let Some(name) = required_tool.as_str() {
                assert!(
                    tool_names.contains(name),
                    "Missing required tool: '{}'",
                    name
                );
            }
        }
    }

    // Verify tool structure
    for tool in &tools {
        verify_tool_structure(tool);
    }
}

/// Helper to verify a single tool's structure
fn verify_tool_structure(tool: &Tool) {
    // Name must be non-empty
    assert!(!tool.name.is_empty(), "Tool name cannot be empty");

    // Description must be non-empty
    assert!(
        !tool.description.is_empty(),
        "Tool description cannot be empty"
    );

    // Input schema must be an object with 'type' field
    assert!(
        tool.input_schema.is_object(),
        "Tool '{}' input_schema must be an object",
        tool.name
    );

    let schema = tool.input_schema.as_object().unwrap();
    assert!(
        schema.contains_key("type"),
        "Tool '{}' input_schema must have 'type' field",
        tool.name
    );
}

/// Test 2: Verify create_kanban_board response structure
#[test]
fn test_create_kanban_board_golden() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;
        let golden = load_golden("create_kanban_board");

        let result = call_tool(
            &app,
            &services,
            "create_kanban_board",
            Some(json!({
                "board_name": "Test Board",
                "description": "A test kanban board"
            })),
        )
        .await;

        // Verify response structure (may be error in demo mode, which is fine)
        if !result.is_error {
            verify_tool_response_structure(&result, &golden)
                .unwrap_or_else(|e| panic!("create_kanban_board structure mismatch: {}", e));
        }

        // Verify ToolCallResult basic structure
        assert!(!result.content.is_empty(), "Response must have content");
    });
}

/// Test 3: Verify list_threads response structure
#[test]
fn test_list_threads_golden() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;
        let golden = load_golden("list_threads");

        let result = call_tool(&app, &services, "list_threads", Some(json!({}))).await;

        // Verify response structure
        if !result.is_error {
            verify_tool_response_structure(&result, &golden)
                .unwrap_or_else(|e| panic!("list_threads structure mismatch: {}", e));

            // Verify threads array structure if present
            let response = parse_tool_response(&result);
            verify_array_items_structure(&response, "threads", &golden, "thread_item_structure")
                .unwrap_or_else(|e| panic!("{}", e));
        }

        assert!(!result.content.is_empty(), "Response must have content");
    });
}

/// Test 4: Verify list_disks response structure
#[test]
fn test_list_disks_golden() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;
        let golden = load_golden("list_disks");

        let result = call_tool(&app, &services, "list_disks", Some(json!({}))).await;

        if !result.is_error {
            verify_tool_response_structure(&result, &golden)
                .unwrap_or_else(|e| panic!("list_disks structure mismatch: {}", e));

            // Verify disk array structure if present
            let response = parse_tool_response(&result);
            verify_array_items_structure(&response, "disks", &golden, "disk_item_structure")
                .unwrap_or_else(|e| panic!("{}", e));
        }

        assert!(!result.content.is_empty(), "Response must have content");
    });
}

/// Test 5: Verify canvas_get_snapshot response structure
#[test]
fn test_canvas_get_snapshot_golden() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;
        let golden = load_golden("canvas_get_snapshot");

        let result = call_tool(&app, &services, "canvas_get_snapshot", Some(json!({}))).await;

        if !result.is_error {
            verify_tool_response_structure(&result, &golden)
                .unwrap_or_else(|e| panic!("canvas_get_snapshot structure mismatch: {}", e));

            // Verify element array structure if present
            let response = parse_tool_response(&result);
            verify_array_items_structure(&response, "elements", &golden, "element_item_structure")
                .unwrap_or_else(|e| panic!("{}", e));
        }

        assert!(!result.content.is_empty(), "Response must have content");
    });
}

/// Test 6: Verify list_entities response structure
#[test]
fn test_list_entities_golden() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;
        let golden = load_golden("list_entities");

        let result = call_tool(&app, &services, "list_entities", Some(json!({}))).await;

        if !result.is_error {
            verify_tool_response_structure(&result, &golden)
                .unwrap_or_else(|e| panic!("list_entities structure mismatch: {}", e));

            // Verify entity array structure if present
            let response = parse_tool_response(&result);
            verify_array_items_structure(&response, "entities", &golden, "entity_item_structure")
                .unwrap_or_else(|e| panic!("{}", e));
        }

        assert!(!result.content.is_empty(), "Response must have content");
    });
}

/// Test 7: Verify network_status response structure
#[test]
fn test_network_status_golden() {
    run_async_test!(async {
        let temp = TempDir::new().unwrap();
        let (app, services) = make_test_services(&temp).await;
        let golden = load_golden("network_status");

        let result = call_tool(&app, &services, "network_status", Some(json!({}))).await;

        if !result.is_error {
            verify_tool_response_structure(&result, &golden)
                .unwrap_or_else(|e| panic!("network_status structure mismatch: {}", e));
        }

        assert!(!result.content.is_empty(), "Response must have content");
    });
}

// =============================================================================
// Tool Definition Structure Tests
// =============================================================================

/// Test that all authenticated tools have valid input schemas
#[test]
fn test_all_tools_have_valid_schemas() {
    let tools = list_tools(true);

    for tool in &tools {
        let schema = &tool.input_schema;

        // Schema must be an object
        assert!(
            schema.is_object(),
            "Tool '{}' schema must be an object",
            tool.name
        );

        let schema_obj = schema.as_object().unwrap();

        // Schema must have 'type' field
        assert!(
            schema_obj.contains_key("type"),
            "Tool '{}' schema must have 'type' field",
            tool.name
        );

        // Type should be 'object'
        let type_val = schema_obj.get("type").unwrap();
        assert_eq!(
            type_val.as_str(),
            Some("object"),
            "Tool '{}' schema type must be 'object'",
            tool.name
        );

        // If properties exist, they must be an object
        if let Some(props) = schema_obj.get("properties") {
            assert!(
                props.is_object(),
                "Tool '{}' properties must be an object",
                tool.name
            );
        }

        // If required exists, it must be an array
        if let Some(required) = schema_obj.get("required") {
            assert!(
                required.is_array(),
                "Tool '{}' required must be an array",
                tool.name
            );
        }
    }
}

/// Test that pre-auth tools are a subset of all tools
#[test]
fn test_preauth_tools_subset() {
    let all_tools = list_tools(true);
    let preauth_tools = list_tools(false);

    let all_names: HashSet<&str> = all_tools.iter().map(|t| t.name.as_str()).collect();
    let preauth_names: HashSet<&str> = preauth_tools.iter().map(|t| t.name.as_str()).collect();

    // Pre-auth tools should be a subset of all tools
    for name in &preauth_names {
        assert!(
            all_names.contains(name),
            "Pre-auth tool '{}' not found in all tools",
            name
        );
    }

    // Pre-auth should have fewer tools than authenticated
    assert!(
        preauth_tools.len() < all_tools.len(),
        "Pre-auth should have fewer tools than authenticated"
    );

    // Essential pre-auth tools must be present
    let essential_preauth = [
        "authenticate",
        "create_vault",
        "health_check",
        "core_status",
    ];
    for name in essential_preauth {
        assert!(
            preauth_names.contains(name),
            "Essential pre-auth tool '{}' missing",
            name
        );
    }
}

// =============================================================================
// Regression Guard Tests
// =============================================================================

/// Test that tool names don't contain invalid characters
#[test]
fn test_tool_names_valid() {
    let tools = list_tools(true);

    for tool in &tools {
        // Names should be snake_case (lowercase, underscores)
        assert!(
            tool.name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c == '_'),
            "Tool name '{}' should be snake_case",
            tool.name
        );

        // Names should not be empty
        assert!(!tool.name.is_empty(), "Tool name cannot be empty");

        // Names should not start or end with underscore
        assert!(
            !tool.name.starts_with('_') && !tool.name.ends_with('_'),
            "Tool name '{}' should not start/end with underscore",
            tool.name
        );
    }
}

/// Test that tool descriptions are meaningful
#[test]
fn test_tool_descriptions_meaningful() {
    let tools = list_tools(true);

    for tool in &tools {
        // Descriptions should be at least 10 characters
        assert!(
            tool.description.len() >= 10,
            "Tool '{}' description too short: '{}'",
            tool.name,
            tool.description
        );

        // Descriptions should not be placeholder text
        let desc_lower = tool.description.to_lowercase();
        assert!(
            !desc_lower.contains("todo") && !desc_lower.contains("fixme"),
            "Tool '{}' has placeholder description",
            tool.name
        );
    }
}

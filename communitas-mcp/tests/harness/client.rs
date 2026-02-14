// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Test Client Implementation
//!
//! Provides async test clients for both HTTP and stdio transports.

// Allow dead code in this library module - items may be used by different test files
#![allow(dead_code)]

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;

// Re-export communitas_core through the same alias as the library
extern crate communitas_bindings as communitas_core;

use communitas_core::app::CommunitasApp;
use communitas_mcp::tools::call_tool;
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;

/// Transport type for MCP communication
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    /// HTTP transport (default for E2E tests)
    Http,
    /// In-process (no HTTP, direct function calls)
    InProcess,
}

/// Result from an MCP tool call
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// Tool name that was called
    pub tool: String,
    /// Whether the call succeeded
    pub success: bool,
    /// Raw content from the response
    pub content: String,
    /// Parsed JSON value (if content was JSON)
    pub parsed: Option<Value>,
    /// Error message if any
    pub error: Option<String>,
    /// Time taken for the call (milliseconds)
    pub duration_ms: u64,
}

impl ToolResult {
    /// Check if the tool call was successful
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the tool call failed
    pub fn is_error(&self) -> bool {
        !self.success
    }

    /// Get the parsed JSON value, panics if not JSON
    pub fn json(&self) -> &Value {
        self.parsed.as_ref().expect("Response is not valid JSON")
    }

    /// Get a field from the parsed JSON
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.parsed.as_ref().and_then(|v| v.get(key))
    }

    /// Get a string field from the parsed JSON
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.as_str())
    }

    /// Get an integer field from the parsed JSON
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key).and_then(|v| v.as_i64())
    }

    /// Get a boolean field from the parsed JSON
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key).and_then(|v| v.as_bool())
    }

    /// Get an array field from the parsed JSON
    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.get(key).and_then(|v| v.as_array())
    }

    /// Get the array length for a field
    pub fn array_len(&self, key: &str) -> usize {
        self.get_array(key).map(|a| a.len()).unwrap_or(0)
    }

    /// Check if the error message contains a substring
    ///
    /// Returns true if:
    /// - The result is an error AND
    /// - The error message or content contains the given substring (case-insensitive)
    pub fn error_contains(&self, substring: &str) -> bool {
        if self.success {
            return false;
        }

        let search_str = substring.to_lowercase();

        // Check error field
        if let Some(ref err) = self.error
            && err.to_lowercase().contains(&search_str)
        {
            return true;
        }

        // Check content field (error messages are often in content)
        if self.content.to_lowercase().contains(&search_str) {
            return true;
        }

        // Check parsed JSON error field if present
        if let Some(ref parsed) = self.parsed {
            if let Some(error_obj) = parsed.get("error") {
                let error_str = error_obj.to_string().to_lowercase();
                if error_str.contains(&search_str) {
                    return true;
                }
            }
            if let Some(message) = parsed.get("message").and_then(|v| v.as_str())
                && message.to_lowercase().contains(&search_str)
            {
                return true;
            }
        }

        false
    }

    /// Create a failed result
    fn error(tool: &str, message: &str) -> Self {
        Self {
            tool: tool.to_string(),
            success: false,
            content: String::new(),
            parsed: None,
            error: Some(message.to_string()),
            duration_ms: 0,
        }
    }
}

/// Assertion helpers for tool results
pub trait ToolAssert {
    /// Assert the tool call succeeded
    fn assert_success(&self) -> &Self;
    /// Assert the tool call failed
    fn assert_error(&self) -> &Self;
    /// Assert a field exists
    fn assert_has(&self, key: &str) -> &Self;
    /// Assert a field equals a value
    fn assert_eq(&self, key: &str, expected: &Value) -> &Self;
    /// Assert a string field equals a value
    fn assert_str_eq(&self, key: &str, expected: &str) -> &Self;
    /// Assert an array has a minimum length
    fn assert_array_min(&self, key: &str, min: usize) -> &Self;
    /// Assert the response contains a pattern
    fn assert_contains(&self, pattern: &str) -> &Self;

    // === Additional assertion helpers for test completion ===

    /// Assert a field is a non-empty string (valid ID)
    fn assert_non_empty(&self, key: &str) -> &Self;
    /// Assert a field is a boolean
    fn assert_bool(&self, key: &str) -> &Self;
    /// Assert a boolean field equals expected value
    fn assert_bool_eq(&self, key: &str, expected: bool) -> &Self;
    /// Assert a numeric field is greater than N
    fn assert_num_gt(&self, key: &str, min: i64) -> &Self;
    /// Assert a numeric field is greater than or equal to N
    fn assert_num_gte(&self, key: &str, min: i64) -> &Self;
    /// Assert a string field is one of the valid values
    fn assert_one_of(&self, key: &str, valid: &[&str]) -> &Self;
    /// Assert a field is null
    fn assert_null(&self, key: &str) -> &Self;
    /// Assert a field is not null
    fn assert_not_null(&self, key: &str) -> &Self;
    /// Assert an array field is empty
    fn assert_array_empty(&self, key: &str) -> &Self;
    /// Assert an array field has exactly N items
    fn assert_array_len(&self, key: &str, len: usize) -> &Self;
}

impl ToolAssert for ToolResult {
    fn assert_success(&self) -> &Self {
        assert!(
            self.success,
            "Expected success but got error: {:?}",
            self.error
        );
        self
    }

    fn assert_error(&self) -> &Self {
        assert!(
            !self.success,
            "Expected error but got success: {}",
            self.content
        );
        self
    }

    fn assert_has(&self, key: &str) -> &Self {
        assert!(
            self.get(key).is_some(),
            "Expected field '{}' not found in response",
            key
        );
        self
    }

    fn assert_eq(&self, key: &str, expected: &Value) -> &Self {
        let actual = self.get(key);
        assert_eq!(
            actual,
            Some(expected),
            "Field '{}' mismatch: expected {:?}, got {:?}",
            key,
            expected,
            actual
        );
        self
    }

    fn assert_str_eq(&self, key: &str, expected: &str) -> &Self {
        let actual = self.get_str(key);
        assert_eq!(
            actual,
            Some(expected),
            "Field '{}' mismatch: expected {:?}, got {:?}",
            key,
            expected,
            actual
        );
        self
    }

    fn assert_array_min(&self, key: &str, min: usize) -> &Self {
        let len = self.array_len(key);
        assert!(
            len >= min,
            "Array '{}' length {} is less than minimum {}",
            key,
            len,
            min
        );
        self
    }

    fn assert_contains(&self, pattern: &str) -> &Self {
        assert!(
            self.content.contains(pattern),
            "Response does not contain '{}': {}",
            pattern,
            self.content
        );
        self
    }

    fn assert_non_empty(&self, key: &str) -> &Self {
        let val = self.get_str(key);
        assert!(
            val.map(|s| !s.is_empty()).unwrap_or(false),
            "Field '{}' should be a non-empty string, got: {:?}",
            key,
            val
        );
        self
    }

    fn assert_bool(&self, key: &str) -> &Self {
        let val = self.get(key);
        assert!(
            val.map(|v| v.is_boolean()).unwrap_or(false),
            "Field '{}' should be a boolean, got: {:?}",
            key,
            val
        );
        self
    }

    fn assert_bool_eq(&self, key: &str, expected: bool) -> &Self {
        let val = self.get(key).and_then(|v| v.as_bool());
        assert_eq!(
            val,
            Some(expected),
            "Field '{}' should be {}, got: {:?}",
            key,
            expected,
            val
        );
        self
    }

    fn assert_num_gt(&self, key: &str, min: i64) -> &Self {
        let val = self.get(key).and_then(|v| v.as_i64());
        assert!(
            val.map(|n| n > min).unwrap_or(false),
            "Field '{}' should be > {}, got: {:?}",
            key,
            min,
            val
        );
        self
    }

    fn assert_num_gte(&self, key: &str, min: i64) -> &Self {
        let val = self.get(key).and_then(|v| v.as_i64());
        assert!(
            val.map(|n| n >= min).unwrap_or(false),
            "Field '{}' should be >= {}, got: {:?}",
            key,
            min,
            val
        );
        self
    }

    fn assert_one_of(&self, key: &str, valid: &[&str]) -> &Self {
        let val = self.get_str(key);
        assert!(
            val.map(|s| valid.contains(&s)).unwrap_or(false),
            "Field '{}' should be one of {:?}, got: {:?}",
            key,
            valid,
            val
        );
        self
    }

    fn assert_null(&self, key: &str) -> &Self {
        let val = self.get(key);
        assert!(
            val.map(|v| v.is_null()).unwrap_or(true),
            "Field '{}' should be null, got: {:?}",
            key,
            val
        );
        self
    }

    fn assert_not_null(&self, key: &str) -> &Self {
        let val = self.get(key);
        assert!(
            val.map(|v| !v.is_null()).unwrap_or(false),
            "Field '{}' should not be null",
            key
        );
        self
    }

    fn assert_array_empty(&self, key: &str) -> &Self {
        let len = self.array_len(key);
        assert_eq!(len, 0, "Array '{}' should be empty, has {} items", key, len);
        self
    }

    fn assert_array_len(&self, key: &str, expected: usize) -> &Self {
        let len = self.array_len(key);
        assert_eq!(
            len, expected,
            "Array '{}' length {} != expected {}",
            key, len, expected
        );
        self
    }
}

/// MCP Test Node - spawns and manages an MCP server process
pub struct McpTestNode {
    name: String,
    process: Child,
    port: u16,
    #[allow(dead_code)]
    temp_dir: Option<TempDir>,
}

impl McpTestNode {
    /// Start a new test node with HTTP transport
    pub async fn start(name: &str) -> Self {
        Self::start_with_options(name, &[]).await
    }

    /// Start with additional CLI arguments
    pub async fn start_with_options(name: &str, extra_args: &[&str]) -> Self {
        // Use OS-assigned port to avoid collisions between concurrent test binaries
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut args = vec![
            "--http".to_string(),
            "--demo".to_string(),
            "--listen".to_string(),
            format!("127.0.0.1:{}", port),
        ];
        args.extend(extra_args.iter().map(|s| s.to_string()));

        let mut process = Command::new(env!("CARGO_BIN_EXE_communitas-mcp"))
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start MCP server");

        // Wait for server to be ready
        let client = Client::new();
        for attempt in 0..50 {
            sleep(Duration::from_millis(100)).await;
            if client
                .post(format!("http://127.0.0.1:{}/mcp", port))
                .json(&json!({
                    "jsonrpc": "2.0",
                    "id": 0,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": { "name": name, "version": "1.0" }
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
                    temp_dir: None,
                };
            }

            if attempt == 49 {
                // Clean up the process before panicking to avoid zombie
                let _ = process.kill();
                let _ = process.wait();
                panic!("Node {} failed to start after 5 seconds", name);
            }
        }

        // Clean up the process if we somehow exit the loop without returning
        let _ = process.kill();
        let _ = process.wait();
        unreachable!()
    }

    /// Get the HTTP URL for this node
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    /// Get the port number
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Send a raw JSON-RPC request
    pub async fn request(&self, method: &str, params: Value) -> Value {
        let client = Client::new();
        match client
            .post(self.url())
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": method,
                "params": params
            }))
            .send()
            .await
        {
            Ok(response) => response.json().await.unwrap_or_else(
                |e| json!({"error": {"message": format!("Failed to parse response: {}", e)}}),
            ),
            Err(e) => json!({"error": {"message": format!("Request failed: {}", e)}}),
        }
    }

    /// Call an MCP tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> ToolResult {
        let start = std::time::Instant::now();

        let response = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments
                }),
            )
            .await;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            return ToolResult {
                tool: name.to_string(),
                success: false,
                content: String::new(),
                parsed: None,
                error: Some(error.to_string()),
                duration_ms,
            };
        }

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
            .unwrap_or("")
            .to_string();

        let parsed: Option<Value> = serde_json::from_str(&content).ok();

        ToolResult {
            tool: name.to_string(),
            success: !is_error,
            content,
            parsed,
            error: if is_error {
                Some("Tool returned error".to_string())
            } else {
                None
            },
            duration_ms,
        }
    }

    /// Initialize the MCP connection
    pub async fn initialize(&self) -> Value {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": &self.name, "version": "1.0" }
            }),
        )
        .await
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<String> {
        let response = self.request("tools/list", json!({})).await;
        response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .map(|tools| {
                tools
                    .iter()
                    .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the node name
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Drop for McpTestNode {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

/// MCP Test Client - in-process client for unit testing
pub struct McpTestClient {
    app: Arc<CommunitasApp>,
    services: UiServices,
    #[allow(dead_code)]
    temp_dir: TempDir,
}

impl McpTestClient {
    /// Create a new in-process test client
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = UiStorage::from_path(temp_dir.path()).expect("Failed to create storage");

        let app = Arc::new(
            CommunitasApp::new(
                "ocean-forest-moon-star".to_string(),
                "TestUser".to_string(),
                "TestDevice".to_string(),
                temp_dir
                    .path()
                    .join("app_storage")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            .expect("Failed to create app"),
        );

        let services = UiServices::new(storage, app.clone()).expect("Failed to create services");

        Self {
            app,
            services,
            temp_dir,
        }
    }

    /// Create with custom identity words
    pub async fn with_identity(words: &str) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let storage = UiStorage::from_path(temp_dir.path()).expect("Failed to create storage");

        let app = Arc::new(
            CommunitasApp::new(
                words.to_string(),
                "TestUser".to_string(),
                "TestDevice".to_string(),
                temp_dir
                    .path()
                    .join("app_storage")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            .expect("Failed to create app"),
        );

        let services = UiServices::new(storage, app.clone()).expect("Failed to create services");

        Self {
            app,
            services,
            temp_dir,
        }
    }

    /// Call an MCP tool directly (in-process)
    pub async fn call_tool(&self, name: &str, arguments: Value) -> ToolResult {
        let start = std::time::Instant::now();

        let result = call_tool(&self.app, &self.services, name, Some(arguments)).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        let content = result
            .content
            .first()
            .map(|c| match c {
                communitas_mcp::protocol::ToolContent::Text { text } => text.clone(),
            })
            .unwrap_or_default();

        let parsed: Option<Value> = serde_json::from_str(&content).ok();

        ToolResult {
            tool: name.to_string(),
            success: !result.is_error,
            content,
            parsed,
            error: if result.is_error {
                Some("Tool returned error".to_string())
            } else {
                None
            },
            duration_ms,
        }
    }

    /// Get access to the underlying app
    pub fn app(&self) -> &Arc<CommunitasApp> {
        &self.app
    }

    /// Get access to the UI services
    pub fn services(&self) -> &UiServices {
        &self.services
    }
}

/// Configuration for multi-node test scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestScenario {
    /// Scenario name
    pub name: String,
    /// Number of nodes to spawn
    pub node_count: usize,
    /// Tools to test
    pub tools: Vec<String>,
}

/// Helper macro to run async tests with larger stack
#[macro_export]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_assertions() {
        let result = ToolResult {
            tool: "test".to_string(),
            success: true,
            content: r#"{"name":"test","value":42}"#.to_string(),
            parsed: Some(json!({"name": "test", "value": 42})),
            error: None,
            duration_ms: 10,
        };

        // These should all pass
        result.assert_success();
        result.assert_has("name");
        result.assert_str_eq("name", "test");
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
}

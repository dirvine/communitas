// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Test Generator Implementation
//!
//! Generates test stubs for untested MCP tools.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::Path;

/// Tool definition from inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
}

/// Category of tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
    pub tools: Vec<ToolDef>,
}

/// Tool inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub version: String,
    pub generated: String,
    pub total_tools: usize,
    pub categories: BTreeMap<String, Category>,
}

/// Generated test stub
#[derive(Debug, Clone)]
pub struct TestStub {
    pub tool_name: String,
    pub category: String,
    pub test_function: String,
    pub test_code: String,
}

/// Test generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Path to inventory JSON
    pub inventory_path: String,
    /// Path to directory containing existing tests
    pub tests_dir: String,
    /// Output path for generated stubs
    pub output_path: String,
    /// Whether to use in-process testing (vs HTTP)
    pub in_process: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            inventory_path: "tests/inventory/tools.json".to_string(),
            tests_dir: "tests".to_string(),
            output_path: "tests/generated_stubs.rs".to_string(),
            in_process: true,
        }
    }
}

/// Test generator
pub struct TestGenerator {
    config: GeneratorConfig,
    inventory: Option<Inventory>,
    tested_tools: HashSet<String>,
}

impl TestGenerator {
    /// Create a new test generator
    pub fn new(config: GeneratorConfig) -> Self {
        Self {
            config,
            inventory: None,
            tested_tools: HashSet::new(),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(GeneratorConfig::default())
    }

    /// Load tool inventory from JSON
    pub fn load_inventory(&mut self) -> Result<(), String> {
        let content = fs::read_to_string(&self.config.inventory_path)
            .map_err(|e| format!("Failed to read inventory: {e}"))?;

        let inventory: Inventory = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse inventory: {e}"))?;

        self.inventory = Some(inventory);
        Ok(())
    }

    /// Scan existing tests to find tested tools
    pub fn scan_tested_tools(&mut self) -> Result<(), String> {
        self.tested_tools.clear();

        let tests_dir = self.config.tests_dir.clone();
        let tests_path = Path::new(&tests_dir);
        if !tests_path.exists() {
            return Ok(());
        }

        self.scan_directory(tests_path)?;
        Ok(())
    }

    fn scan_directory(&mut self, dir: &Path) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {e}"))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip target and hidden directories
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" {
                    self.scan_directory(&path)?;
                }
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                self.scan_file(&path)?;
            }
        }
        Ok(())
    }

    fn scan_file(&mut self, path: &Path) -> Result<(), String> {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(()),
        };

        // Look for tool calls in tests
        // Pattern: call_tool("tool_name", ...)
        let tool_pattern = regex::Regex::new(r#"call_tool\s*\(\s*"([^"]+)""#)
            .map_err(|e| format!("Regex error: {e}"))?;

        for cap in tool_pattern.captures_iter(&content) {
            if let Some(tool_name) = cap.get(1) {
                self.tested_tools.insert(tool_name.as_str().to_string());
            }
        }

        // Also look for test function names that match tool names
        // Pattern: fn test_tool_name or #[test] followed by fn test_...
        let test_pattern =
            regex::Regex::new(r#"fn\s+test_([a-z_]+)"#).map_err(|e| format!("Regex error: {e}"))?;

        for cap in test_pattern.captures_iter(&content) {
            if let Some(test_name) = cap.get(1) {
                self.tested_tools.insert(test_name.as_str().to_string());
            }
        }

        Ok(())
    }

    /// Get list of untested tools
    pub fn get_untested_tools(&self) -> Vec<(String, ToolDef)> {
        let inventory = match &self.inventory {
            Some(inv) => inv,
            None => return Vec::new(),
        };

        let mut untested = Vec::new();

        for (category, cat_def) in &inventory.categories {
            for tool in &cat_def.tools {
                if !self.tested_tools.contains(&tool.name) {
                    untested.push((category.clone(), tool.clone()));
                }
            }
        }

        untested.sort_by(|a, b| a.1.name.cmp(&b.1.name));
        untested
    }

    /// Generate test stub for a single tool
    pub fn generate_stub(&self, category: &str, tool: &ToolDef) -> TestStub {
        let test_fn_name = format!("test_{}", tool.name);

        let test_code = if self.config.in_process {
            self.generate_in_process_stub(category, tool)
        } else {
            self.generate_http_stub(category, tool)
        };

        TestStub {
            tool_name: tool.name.clone(),
            category: category.to_string(),
            test_function: test_fn_name,
            test_code,
        }
    }

    fn generate_in_process_stub(&self, category: &str, tool: &ToolDef) -> String {
        let args = self.infer_arguments(&tool.name);

        format!(
            r#"/// Test: {} - {}
/// Category: {}
#[tokio::test]
async fn test_{}() {{
    let client = McpTestClient::new().await;

    let result = client.call_tool("{}", json!({{
        {}
    }})).await;

    result.assert_success();
    // TODO: Add specific assertions for {} response
}}
"#,
            tool.name, tool.description, category, tool.name, tool.name, args, tool.name
        )
    }

    fn generate_http_stub(&self, category: &str, tool: &ToolDef) -> String {
        let args = self.infer_arguments(&tool.name);

        format!(
            r#"/// Test: {} - {}
/// Category: {}
#[tokio::test]
async fn test_{}() {{
    let node = McpTestNode::start("test-{}").await;

    let result = node.call_tool("{}", json!({{
        {}
    }})).await;

    result.assert_success();
    // TODO: Add specific assertions for {} response
}}
"#,
            tool.name, tool.description, category, tool.name, tool.name, tool.name, args, tool.name
        )
    }

    /// Infer common arguments based on tool name patterns
    fn infer_arguments(&self, tool_name: &str) -> String {
        // Common patterns
        if tool_name.starts_with("get_") || tool_name.starts_with("delete_") {
            return r#""id": "test-id-123""#.to_string();
        }

        if tool_name.starts_with("list_") {
            return r#""limit": 10"#.to_string();
        }

        if tool_name.starts_with("create_") {
            if tool_name.contains("kanban_board") {
                return r#""name": "Test Board""#.to_string();
            }
            if tool_name.contains("kanban_column") {
                return r#""board_id": "board-123", "name": "Test Column""#.to_string();
            }
            if tool_name.contains("kanban_card") {
                return r#""column_id": "column-123", "title": "Test Card""#.to_string();
            }
            if tool_name.contains("contact") {
                return r#""name": "Test Contact""#.to_string();
            }
            if tool_name.contains("thread") {
                return r#""subject": "Test Thread""#.to_string();
            }
            if tool_name.contains("entity") {
                return r#""name": "Test Entity", "type": "group""#.to_string();
            }
            return r#""name": "Test Item""#.to_string();
        }

        if tool_name.starts_with("update_") {
            return r#""id": "test-id-123", "name": "Updated Name""#.to_string();
        }

        if tool_name.starts_with("move_") {
            return r#""id": "test-id-123", "target_id": "target-123""#.to_string();
        }

        if tool_name.starts_with("send_") {
            return r#""content": "Test message""#.to_string();
        }

        if tool_name.starts_with("search_") {
            return r#""query": "test""#.to_string();
        }

        if tool_name.starts_with("add_") || tool_name.starts_with("remove_") {
            return r#""id": "test-id-123""#.to_string();
        }

        // Default empty
        String::new()
    }

    /// Generate all test stubs for untested tools
    pub fn generate_all_stubs(&self) -> Vec<TestStub> {
        self.get_untested_tools()
            .iter()
            .map(|(category, tool)| self.generate_stub(category, tool))
            .collect()
    }

    /// Write generated stubs to file
    pub fn write_stubs(&self, stubs: &[TestStub]) -> Result<(), String> {
        let mut content = String::new();

        // Header
        content.push_str(
            r#"// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Generated Test Stubs for Untested MCP Tools
//!
//! These stubs were auto-generated and should be reviewed and customized.
//! Move completed tests to appropriate test files.
//!
//! Generated by: communitas-mcp/tests/generator

#![allow(dead_code)]
#![cfg(test)]

mod harness;

use harness::{McpTestClient, ToolAssert};
use serde_json::json;

"#,
        );

        // Group by category
        let mut by_category: BTreeMap<String, Vec<&TestStub>> = BTreeMap::new();
        for stub in stubs {
            by_category
                .entry(stub.category.clone())
                .or_default()
                .push(stub);
        }

        // Generate tests by category
        for (category, cat_stubs) in &by_category {
            content.push_str(&format!("\n// === {} ===\n\n", category.to_uppercase()));

            for stub in cat_stubs {
                content.push_str(&stub.test_code);
                content.push('\n');
            }
        }

        // Write file
        fs::write(&self.config.output_path, &content)
            .map_err(|e| format!("Failed to write stubs: {e}"))?;

        Ok(())
    }

    /// Generate summary report
    pub fn generate_report(&self) -> String {
        let inventory = match &self.inventory {
            Some(inv) => inv,
            None => return "No inventory loaded".to_string(),
        };

        let untested = self.get_untested_tools();
        let total = inventory.total_tools;
        let tested = total - untested.len();
        let coverage = (tested as f64 / total as f64) * 100.0;

        let mut report = String::new();

        report.push_str("# MCP Test Generator Report\n\n");
        report.push_str(&format!("**Total Tools**: {}\n", total));
        report.push_str(&format!("**Tested**: {}\n", tested));
        report.push_str(&format!("**Untested**: {}\n", untested.len()));
        report.push_str(&format!("**Coverage**: {:.1}%\n\n", coverage));

        if !untested.is_empty() {
            report.push_str("## Untested Tools by Category\n\n");

            let mut by_category: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (category, tool) in &untested {
                by_category
                    .entry(category.clone())
                    .or_default()
                    .push(tool.name.clone());
            }

            for (category, tools) in &by_category {
                report.push_str(&format!("### {}\n", category));
                for tool in tools {
                    report.push_str(&format!("- `{}`\n", tool));
                }
                report.push('\n');
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_arguments() {
        let generator = TestGenerator::with_defaults();

        assert!(generator.infer_arguments("get_profile").contains("id"));
        assert!(generator.infer_arguments("list_contacts").contains("limit"));
        assert!(
            generator
                .infer_arguments("create_kanban_board")
                .contains("name")
        );
        assert!(
            generator
                .infer_arguments("search_messages")
                .contains("query")
        );
    }

    #[test]
    fn test_generate_stub() {
        let generator = TestGenerator::with_defaults();

        let tool = ToolDef {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
        };

        let stub = generator.generate_stub("testing", &tool);

        assert_eq!(stub.tool_name, "test_tool");
        assert_eq!(stub.category, "testing");
        assert!(stub.test_code.contains("#[tokio::test]"));
        assert!(stub.test_code.contains("test_tool"));
    }
}

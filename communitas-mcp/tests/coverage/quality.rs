// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

// Allow collapsible_if for readability in nested JSON parsing
#![allow(clippy::collapsible_if)]

//! Test Quality Tracker
//!
//! Analyzes test files to determine assertion quality beyond just coverage.
//! Counts meaningful assertions vs stub tests with only `assert_success()`.

use regex::Regex;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

/// Quality metrics for a single test function
#[derive(Debug, Clone, Default, Serialize)]
pub struct TestQuality {
    /// Test function name
    pub test_name: String,
    /// Tool being tested
    pub tool_name: Option<String>,
    /// Number of assert_success() calls
    pub success_asserts: usize,
    /// Number of meaningful assertions (assert_has, assert_eq, etc.)
    pub meaningful_asserts: usize,
    /// Number of TODO comments in this test
    pub todo_count: usize,
    /// Whether this is considered a stub test (only assert_success, has TODO)
    pub is_stub: bool,
    /// Quality score (0.0 - 1.0)
    pub quality_score: f64,
}

/// Quality metrics for a category of tools
#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryQuality {
    /// Category name
    pub name: String,
    /// Total tests in category
    pub total_tests: usize,
    /// Complete tests (not stubs)
    pub complete_tests: usize,
    /// Stub tests (need work)
    pub stub_tests: usize,
    /// Total TODO comments
    pub total_todos: usize,
    /// Average quality score
    pub avg_quality: f64,
}

/// Complete quality report
#[derive(Debug, Serialize)]
pub struct QualityReport {
    /// Report generation timestamp
    pub generated: String,
    /// Total test functions analyzed
    pub total_tests: usize,
    /// Complete tests
    pub complete_tests: usize,
    /// Stub tests
    pub stub_tests: usize,
    /// Total TODO comments
    pub total_todos: usize,
    /// Overall quality score (0.0 - 1.0)
    pub overall_quality: f64,
    /// Quality by category
    pub categories: Vec<CategoryQuality>,
    /// Individual test quality (optional, can be large)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<TestQuality>,
}

/// Test quality tracker
pub struct QualityTracker {
    tests_path: PathBuf,
    /// Tool name to category mapping (from inventory)
    tool_categories: HashMap<String, String>,
}

/// Extract content between matching braces, handling nested braces
fn extract_brace_block(s: &str) -> String {
    if !s.starts_with('{') {
        return String::new();
    }

    let mut depth = 0;
    let mut end = 0;

    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end > 0 {
        s[..end].to_string()
    } else {
        String::new()
    }
}

impl QualityTracker {
    /// Create a new quality tracker
    pub fn new(tests_path: impl AsRef<Path>) -> Self {
        Self {
            tests_path: tests_path.as_ref().to_path_buf(),
            tool_categories: HashMap::new(),
        }
    }

    /// Create with inventory file for category mapping
    pub fn with_inventory(tests_path: impl AsRef<Path>, inventory_path: impl AsRef<Path>) -> Self {
        let mut tracker = Self::new(tests_path);
        if let Ok(content) = fs::read_to_string(inventory_path.as_ref()) {
            if let Ok(inventory) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(categories) = inventory.get("categories").and_then(|c| c.as_object()) {
                    for (cat_name, cat_def) in categories {
                        if let Some(tools) = cat_def.get("tools").and_then(|t| t.as_array()) {
                            for tool in tools {
                                if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                                    tracker
                                        .tool_categories
                                        .insert(name.to_string(), cat_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        tracker
    }

    /// Set the tool-to-category mapping
    #[allow(dead_code)]
    pub fn set_tool_categories(&mut self, mapping: HashMap<String, String>) {
        self.tool_categories = mapping;
    }

    /// Analyze all test files and generate quality report
    pub fn analyze(&self) -> Result<QualityReport, String> {
        let mut all_tests: Vec<TestQuality> = Vec::new();
        let mut category_map: BTreeMap<String, CategoryQuality> = BTreeMap::new();

        // Scan test files
        self.scan_directory(&self.tests_path.clone(), &mut all_tests)?;

        // Group by category
        for test in &all_tests {
            let category = test
                .tool_name
                .as_ref()
                .and_then(|t| self.tool_categories.get(t))
                .cloned()
                .unwrap_or_else(|| "uncategorized".to_string());

            let entry = category_map
                .entry(category.clone())
                .or_insert_with(|| CategoryQuality {
                    name: category,
                    ..Default::default()
                });

            entry.total_tests += 1;
            entry.total_todos += test.todo_count;
            if test.is_stub {
                entry.stub_tests += 1;
            } else {
                entry.complete_tests += 1;
            }
        }

        // Calculate average quality per category
        for cat in category_map.values_mut() {
            if cat.total_tests > 0 {
                cat.avg_quality = cat.complete_tests as f64 / cat.total_tests as f64;
            }
        }

        // Calculate overall metrics
        let total_tests = all_tests.len();
        let complete_tests = all_tests.iter().filter(|t| !t.is_stub).count();
        let stub_tests = all_tests.iter().filter(|t| t.is_stub).count();
        let total_todos: usize = all_tests.iter().map(|t| t.todo_count).sum();
        let overall_quality = if total_tests > 0 {
            complete_tests as f64 / total_tests as f64
        } else {
            1.0
        };

        Ok(QualityReport {
            generated: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            total_tests,
            complete_tests,
            stub_tests,
            total_todos,
            overall_quality,
            categories: category_map.into_values().collect(),
            tests: all_tests,
        })
    }

    fn scan_directory(&self, dir: &Path, all_tests: &mut Vec<TestQuality>) -> Result<(), String> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {e}"))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip coverage and inventory directories
                if dir_name != "coverage" && dir_name != "inventory" && dir_name != "golden" {
                    self.scan_directory(&path, all_tests)?;
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                self.scan_file(&path, all_tests)?;
            }
        }

        Ok(())
    }

    fn scan_file(&self, file_path: &Path, all_tests: &mut Vec<TestQuality>) -> Result<(), String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read {}: {e}", file_path.display()))?;

        // Regex patterns for finding test function signatures
        let test_attr_pattern = Regex::new(r"#\[(tokio::)?test\]").unwrap();
        let fn_sig_pattern = Regex::new(r"(?:async )?fn (test_\w+)\s*\(\s*\)").unwrap();
        let tool_call_pattern = Regex::new(r#"call_tool\s*\(\s*"([a-z_]+)""#).unwrap();
        let success_pattern = Regex::new(r"\.assert_success\s*\(\s*\)").unwrap();
        let meaningful_patterns = vec![
            Regex::new(r"\.assert_has\s*\(").unwrap(),
            Regex::new(r"\.assert_eq\s*\(").unwrap(),
            Regex::new(r"\.assert_str_eq\s*\(").unwrap(),
            Regex::new(r"\.assert_array_min\s*\(").unwrap(),
            Regex::new(r"\.assert_contains\s*\(").unwrap(),
            Regex::new(r"\.assert_error\s*\(").unwrap(),
            Regex::new(r"assert!\s*\(").unwrap(),
            Regex::new(r"assert_eq!\s*\(").unwrap(),
            Regex::new(r"assert_ne!\s*\(").unwrap(),
        ];
        let todo_pattern = Regex::new(r"//\s*TODO").unwrap();

        // Find test functions by looking for test attributes followed by fn signatures
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            // Look for #[test] or #[tokio::test]
            if test_attr_pattern.is_match(lines[i]) {
                // Next non-empty line should have the function signature
                let mut fn_line_idx = i + 1;
                while fn_line_idx < lines.len() && lines[fn_line_idx].trim().is_empty() {
                    fn_line_idx += 1;
                }

                if fn_line_idx < lines.len() {
                    if let Some(capture) = fn_sig_pattern.captures(lines[fn_line_idx]) {
                        let test_name = capture.get(1).map(|m| m.as_str()).unwrap_or("unknown");

                        // Extract function body - find matching braces
                        let start_idx = fn_line_idx;
                        let body_start = content
                            .find(&format!("fn {test_name}"))
                            .and_then(|pos| content[pos..].find('{').map(|p| pos + p))
                            .unwrap_or(0);

                        let test_body = if body_start > 0 {
                            extract_brace_block(&content[body_start..])
                        } else {
                            String::new()
                        };

                        // Extract tool name
                        let tool_name = tool_call_pattern
                            .captures(&test_body)
                            .and_then(|c| c.get(1))
                            .map(|m| m.as_str().to_string());

                        // Count assertions
                        let success_asserts = success_pattern.find_iter(&test_body).count();
                        let meaningful_asserts: usize = meaningful_patterns
                            .iter()
                            .map(|p| p.find_iter(&test_body).count())
                            .sum();
                        let todo_count = todo_pattern.find_iter(&test_body).count();

                        // Determine if stub
                        let is_stub = todo_count > 0 && meaningful_asserts == 0;

                        // Calculate quality score
                        let quality_score = if success_asserts + meaningful_asserts == 0 {
                            0.0
                        } else if is_stub {
                            0.25
                        } else if meaningful_asserts > 0 {
                            (meaningful_asserts as f64
                                / (success_asserts + meaningful_asserts) as f64)
                                .clamp(0.5, 1.0)
                        } else {
                            0.5
                        };

                        all_tests.push(TestQuality {
                            test_name: test_name.to_string(),
                            tool_name,
                            success_asserts,
                            meaningful_asserts,
                            todo_count,
                            is_stub,
                            quality_score,
                        });

                        i = start_idx + 1;
                        continue;
                    }
                }
            }
            i += 1;
        }

        Ok(())
    }

    /// Generate a markdown quality report
    pub fn generate_markdown(&self) -> Result<String, String> {
        let report = self.analyze()?;

        let mut md = String::new();
        md.push_str("# MCP Test Quality Report\n\n");
        md.push_str(&format!("**Generated**: {}\n\n", report.generated));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "| Metric | Value |\n|--------|-------|\n| Total Tests | {} |\n| Complete Tests | {} |\n| Stub Tests | {} |\n| TODO Comments | {} |\n| Quality Score | {:.1}% |\n\n",
            report.total_tests,
            report.complete_tests,
            report.stub_tests,
            report.total_todos,
            report.overall_quality * 100.0
        ));

        // Progress bar
        let filled = (report.overall_quality * 40.0) as usize;
        let empty = 40 - filled;
        md.push_str("```\nQuality Progress:\n");
        md.push_str(&format!(
            "[{}{}] {:.1}%\n",
            "#".repeat(filled),
            " ".repeat(empty),
            report.overall_quality * 100.0
        ));
        md.push_str("```\n\n");

        // Category breakdown
        md.push_str("## Quality by Category\n\n");
        md.push_str(
            "| Category | Total | Complete | Stubs | TODOs | Quality |\n|----------|-------|----------|-------|-------|--------|\n",
        );

        for cat in &report.categories {
            let icon = if cat.avg_quality >= 0.9 {
                "✅"
            } else if cat.avg_quality >= 0.5 {
                "⚠️"
            } else {
                "❌"
            };
            md.push_str(&format!(
                "| {} {} | {} | {} | {} | {} | {:.1}% |\n",
                icon,
                cat.name,
                cat.total_tests,
                cat.complete_tests,
                cat.stub_tests,
                cat.total_todos,
                cat.avg_quality * 100.0
            ));
        }

        md.push('\n');

        // Categories needing work
        let stubs: Vec<_> = report
            .categories
            .iter()
            .filter(|c| c.stub_tests > 0)
            .collect();
        if !stubs.is_empty() {
            md.push_str("## Categories Needing Work\n\n");
            for cat in stubs {
                md.push_str(&format!(
                    "### {} ({} stubs, {} TODOs)\n\n",
                    cat.name, cat.stub_tests, cat.total_todos
                ));
            }
        }

        // Legend
        md.push_str("\n## Legend\n\n");
        md.push_str("- ✅ 90%+ quality (complete tests)\n");
        md.push_str("- ⚠️ 50-89% quality (some stubs remain)\n");
        md.push_str("- ❌ <50% quality (mostly stubs)\n");

        Ok(md)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_tests_path() -> PathBuf {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(&manifest_dir).join("tests")
    }

    #[test]
    fn test_analyze_quality() {
        let tests_path = get_tests_path();
        if !tests_path.exists() {
            println!("Tests directory not found, skipping");
            return;
        }

        let tracker = QualityTracker::new(&tests_path);
        let report = tracker.analyze().expect("Failed to analyze");

        println!("Total tests: {}", report.total_tests);
        println!("Complete: {}", report.complete_tests);
        println!("Stubs: {}", report.stub_tests);
        println!("TODOs: {}", report.total_todos);
        println!("Quality: {:.1}%", report.overall_quality * 100.0);

        assert!(report.total_tests > 0, "Should find some tests");
    }

    #[test]
    fn test_generate_markdown() {
        let tests_path = get_tests_path();
        if !tests_path.exists() {
            println!("Tests directory not found, skipping");
            return;
        }

        let tracker = QualityTracker::new(&tests_path);
        let md = tracker
            .generate_markdown()
            .expect("Failed to generate markdown");

        assert!(md.contains("# MCP Test Quality Report"));
        assert!(md.contains("Total Tests"));
    }
}

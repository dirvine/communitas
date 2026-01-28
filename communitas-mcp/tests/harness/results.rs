// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Test Result Aggregator
//!
//! Collects and aggregates test results from cargo test output,
//! parses tool coverage from test names, and generates summary reports.

// Allow dead code since this is a library module
#![allow(dead_code)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

/// Status of an individual test
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Ignored,
    #[default]
    Unknown,
}

/// Result of a single test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name (e.g., "test_create_kanban_board")
    pub name: String,
    /// Test module path (e.g., "parity_test")
    pub module: String,
    /// Test status
    pub status: TestStatus,
    /// Duration in milliseconds (if available)
    pub duration_ms: Option<u64>,
    /// Tool being tested (extracted from test name)
    pub tool: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Aggregated results for a tool
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolTestResults {
    /// Tool name
    pub tool: String,
    /// Total tests for this tool
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Ignored tests
    pub ignored: usize,
    /// Test names
    pub tests: Vec<String>,
}

/// Complete test run summary
#[derive(Debug, Serialize, Deserialize)]
pub struct TestSummary {
    /// When the test run started
    pub timestamp: String,
    /// Total tests run
    pub total_tests: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Ignored tests
    pub ignored: usize,
    /// Pass rate percentage
    pub pass_rate: f64,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Results per tool
    pub tools: BTreeMap<String, ToolTestResults>,
    /// All individual test results
    pub tests: Vec<TestResult>,
    /// Failed test names
    pub failures: Vec<String>,
}

/// Test result aggregator
pub struct ResultAggregator {
    tests: Vec<TestResult>,
    tool_pattern: Regex,
}

impl ResultAggregator {
    /// Create a new result aggregator
    pub fn new() -> Self {
        // Pattern to extract tool name from test name
        // Matches patterns like "test_create_kanban_board" -> "create_kanban_board"
        let tool_pattern = Regex::new(
            r"(?:test_)?(create_|get_|list_|update_|delete_|add_|remove_|set_|toggle_|start_|stop_|join_|end_|send_|cancel_|resume_|move_|copy_|pin_|unpin_|mark_|tag_|untag_|search_|assign_|unassign_|share_|revoke_|export_|validate_|recover_|sync_|retry_|skip_|queue_|announce_|subscribe_|query_|accept_|resolve_|change_|stage_|upload_|canvas_|network_)([a-z_]+)"
        ).unwrap();

        Self {
            tests: Vec::new(),
            tool_pattern,
        }
    }

    /// Run cargo test and parse the output
    pub fn run_tests(&mut self, package: &str) -> Result<(), String> {
        self.tests.clear();

        let output = Command::new("cargo")
            .args([
                "test",
                "-p",
                package,
                "--",
                "--test-threads=1",
                "-Z",
                "unstable-options",
                "--format=json",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to run cargo test: {e}"))?;

        // Parse JSON output (each line is a JSON event)
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Ok(event) = serde_json::from_str::<serde_json::Value>(line) {
                self.parse_test_event(&event);
            }
        }

        Ok(())
    }

    /// Parse output from pre-captured cargo test output
    pub fn parse_output(&mut self, output: &str) {
        self.tests.clear();

        // Patterns for standard cargo test output
        let test_pattern = Regex::new(r"^test (\S+) \.\.\. (ok|FAILED|ignored)").unwrap();
        let test_json_pattern =
            Regex::new(r#""type":"test","event":"(\w+)","name":"([^"]+)""#).unwrap();

        for line in output.lines() {
            // Try JSON format first
            if let Some(caps) = test_json_pattern.captures(line) {
                let event = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let name = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let status = match event {
                    "ok" => TestStatus::Passed,
                    "failed" => TestStatus::Failed,
                    "ignored" => TestStatus::Ignored,
                    _ => continue,
                };

                self.add_test_result(name, status);
            }
            // Try standard format
            else if let Some(caps) = test_pattern.captures(line) {
                let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let result = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                let status = match result {
                    "ok" => TestStatus::Passed,
                    "FAILED" => TestStatus::Failed,
                    "ignored" => TestStatus::Ignored,
                    _ => continue,
                };

                self.add_test_result(name, status);
            }
        }
    }

    fn parse_test_event(&mut self, event: &serde_json::Value) {
        // Handle test events - check type is "test" and we have an event field
        let is_test = event.get("type").and_then(|t| t.as_str()) == Some("test");
        let test_event = event.get("event").and_then(|e| e.as_str());

        if let (true, Some(test_event)) = (is_test, test_event) {
            let name = event
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unknown");

            let status = match test_event {
                "ok" => TestStatus::Passed,
                "failed" => TestStatus::Failed,
                "ignored" => TestStatus::Ignored,
                _ => return,
            };

            self.add_test_result(name, status);
        }
    }

    fn add_test_result(&mut self, full_name: &str, status: TestStatus) {
        // Split module::test_name
        let parts: Vec<&str> = full_name.rsplitn(2, "::").collect();
        let (name, module) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            (full_name.to_string(), String::new())
        };

        // Extract tool name from test name
        let tool = self.tool_pattern.captures(&name).map(|c| {
            let prefix = c.get(1).map(|m| m.as_str()).unwrap_or("");
            let suffix = c.get(2).map(|m| m.as_str()).unwrap_or("");
            format!("{}{}", prefix, suffix)
        });

        self.tests.push(TestResult {
            name,
            module,
            status,
            duration_ms: None,
            tool,
            error: None,
        });
    }

    /// Generate a summary from collected results
    pub fn generate_summary(&self) -> TestSummary {
        let mut tools: BTreeMap<String, ToolTestResults> = BTreeMap::new();
        let mut passed = 0;
        let mut failed = 0;
        let mut ignored = 0;
        let mut failures = Vec::new();

        for test in &self.tests {
            match test.status {
                TestStatus::Passed => passed += 1,
                TestStatus::Failed => {
                    failed += 1;
                    failures.push(test.name.clone());
                }
                TestStatus::Ignored => ignored += 1,
                TestStatus::Unknown => {}
            }

            // Aggregate by tool
            if let Some(tool_name) = &test.tool {
                let tool_results =
                    tools
                        .entry(tool_name.clone())
                        .or_insert_with(|| ToolTestResults {
                            tool: tool_name.clone(),
                            ..Default::default()
                        });

                tool_results.total += 1;
                tool_results.tests.push(test.name.clone());

                match test.status {
                    TestStatus::Passed => tool_results.passed += 1,
                    TestStatus::Failed => tool_results.failed += 1,
                    TestStatus::Ignored => tool_results.ignored += 1,
                    TestStatus::Unknown => {}
                }
            }
        }

        let total = self.tests.len();
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        TestSummary {
            timestamp: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            total_tests: total,
            passed,
            failed,
            ignored,
            pass_rate,
            duration_secs: 0.0,
            tools,
            tests: self.tests.clone(),
            failures,
        }
    }

    /// Export summary to JSON file
    pub fn export_json(&self, path: &Path) -> Result<(), String> {
        let summary = self.generate_summary();
        let json = serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("Failed to serialize: {e}"))?;
        fs::write(path, json).map_err(|e| format!("Failed to write: {e}"))?;
        Ok(())
    }

    /// Generate markdown report
    pub fn generate_markdown(&self) -> String {
        let summary = self.generate_summary();
        let mut md = String::new();

        md.push_str("# Test Results Report\n\n");
        md.push_str(&format!("**Timestamp**: {}\n\n", summary.timestamp));

        md.push_str("## Summary\n\n");
        md.push_str("| Metric | Value |\n|--------|-------|\n");
        md.push_str(&format!("| Total Tests | {} |\n", summary.total_tests));
        md.push_str(&format!("| Passed | {} |\n", summary.passed));
        md.push_str(&format!("| Failed | {} |\n", summary.failed));
        md.push_str(&format!("| Ignored | {} |\n", summary.ignored));
        md.push_str(&format!("| Pass Rate | {:.1}% |\n", summary.pass_rate));

        if !summary.failures.is_empty() {
            md.push_str("\n## Failures\n\n");
            for failure in &summary.failures {
                md.push_str(&format!("- `{}`\n", failure));
            }
        }

        md.push_str("\n## Results by Tool\n\n");
        md.push_str("| Tool | Total | Passed | Failed | Status |\n");
        md.push_str("|------|-------|--------|--------|--------|\n");

        for (tool, results) in &summary.tools {
            let status = if results.failed > 0 {
                "FAIL"
            } else if results.passed > 0 {
                "PASS"
            } else {
                "SKIP"
            };
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                tool, results.total, results.passed, results.failed, status
            ));
        }

        md
    }
}

impl Default for ResultAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_output() {
        let output = r#"
running 5 tests
test test_create_kanban_board ... ok
test test_get_kanban_board ... ok
test test_delete_kanban_board ... FAILED
test test_ignored ... ignored
test test_list_entities ... ok
"#;

        let mut aggregator = ResultAggregator::new();
        aggregator.parse_output(output);

        let summary = aggregator.generate_summary();
        assert_eq!(summary.total_tests, 5);
        assert_eq!(summary.passed, 3);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.ignored, 1);
    }

    #[test]
    fn test_tool_extraction() {
        let mut aggregator = ResultAggregator::new();

        aggregator.add_test_result("test_create_kanban_board", TestStatus::Passed);
        aggregator.add_test_result("test_list_entities", TestStatus::Passed);
        aggregator.add_test_result("test_network_status", TestStatus::Passed);

        let summary = aggregator.generate_summary();

        assert!(summary.tools.contains_key("create_kanban_board"));
        assert!(summary.tools.contains_key("list_entities"));
        assert!(summary.tools.contains_key("network_status"));
    }

    #[test]
    fn test_generate_markdown() {
        let mut aggregator = ResultAggregator::new();
        aggregator.add_test_result("test_create_kanban_board", TestStatus::Passed);
        aggregator.add_test_result("test_delete_kanban_board", TestStatus::Failed);

        let md = aggregator.generate_markdown();

        assert!(md.contains("# Test Results Report"));
        assert!(md.contains("Total Tests"));
        assert!(md.contains("create_kanban_board"));
    }
}

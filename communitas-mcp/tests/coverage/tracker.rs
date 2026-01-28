// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Tool Coverage Tracker
//!
//! Parses the tool inventory and scans test files to determine
//! which MCP tools have test coverage.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

/// Tool definition from inventory.json
#[derive(Debug, Clone, Deserialize)]
pub struct ToolDef {
    pub name: String,
    #[allow(dead_code)]
    pub description: String,
}

/// Category definition from inventory.json
#[derive(Debug, Clone, Deserialize)]
pub struct CategoryDef {
    pub description: String,
    pub tools: Vec<ToolDef>,
}

/// The complete tool inventory
#[derive(Debug, Deserialize)]
pub struct ToolInventory {
    #[allow(dead_code)]
    pub version: String,
    #[allow(dead_code)]
    pub generated: String,
    pub total_tools: usize,
    pub categories: HashMap<String, CategoryDef>,
    #[allow(dead_code)]
    pub summary: HashMap<String, usize>,
}

/// Coverage information for a single tool
#[derive(Debug, Clone, Serialize)]
pub struct ToolCoverage {
    pub name: String,
    pub category: String,
    pub tested: bool,
    pub test_files: Vec<String>,
    pub test_count: usize,
}

/// Coverage statistics for a category
#[derive(Debug, Clone, Serialize)]
pub struct CategoryCoverage {
    pub name: String,
    pub description: String,
    pub total_tools: usize,
    pub tested_tools: usize,
    pub coverage_percent: f64,
    pub untested: Vec<String>,
}

/// Complete coverage report
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct CoverageReport {
    pub generated: String,
    pub total_tools: usize,
    pub tested_tools: usize,
    pub overall_coverage: f64,
    pub categories: Vec<CategoryCoverage>,
    pub tools: Vec<ToolCoverage>,
    pub untested_tools: Vec<String>,
}

/// Main coverage tracker
pub struct CoverageTracker {
    inventory_path: PathBuf,
    tests_path: PathBuf,
    tool_references: HashMap<String, HashSet<String>>,
}

impl CoverageTracker {
    /// Create a new coverage tracker
    pub fn new(inventory_path: impl AsRef<Path>, tests_path: impl AsRef<Path>) -> Self {
        Self {
            inventory_path: inventory_path.as_ref().to_path_buf(),
            tests_path: tests_path.as_ref().to_path_buf(),
            tool_references: HashMap::new(),
        }
    }

    /// Load the tool inventory from JSON
    pub fn load_inventory(&self) -> Result<ToolInventory, String> {
        let content = fs::read_to_string(&self.inventory_path)
            .map_err(|e| format!("Failed to read inventory: {e}"))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse inventory: {e}"))
    }

    /// Scan test files for tool references
    pub fn scan_tests(&mut self) -> Result<(), String> {
        self.tool_references.clear();

        // Patterns to match tool calls
        let patterns = vec![
            // call_tool(&app, &services, "tool_name", ...)
            Regex::new(r#"call_tool\s*\([^,]+,\s*[^,]+,\s*"([a-z_]+)""#).unwrap(),
            // call_tool("tool_name", ...)
            Regex::new(r#"call_tool\s*\(\s*"([a-z_]+)""#).unwrap(),
            // .call_tool("tool_name", ...)
            Regex::new(r#"\.call_tool\s*\(\s*"([a-z_]+)""#).unwrap(),
            // tool_names.contains(&"tool_name")
            Regex::new(r#"tool_names\.contains\s*\(\s*&"([a-z_]+)""#).unwrap(),
            // "tool_name" in test assertions
            Regex::new(r#""(create_|get_|list_|update_|delete_|add_|remove_|set_|toggle_|start_|stop_|join_|end_|send_|cancel_|resume_|move_|copy_|pin_|unpin_|mark_|tag_|untag_|search_|assign_|unassign_|share_|revoke_|export_|validate_|recover_|sync_|retry_|skip_|queue_|announce_|subscribe_|query_|accept_|resolve_|change_|stage_|upload_|canvas_|network_)[a-z_]*""#).unwrap(),
        ];

        // Scan all .rs files in tests directory
        self.scan_directory(&self.tests_path.clone(), &patterns)?;

        Ok(())
    }

    fn scan_directory(&mut self, dir: &Path, patterns: &[Regex]) -> Result<(), String> {
        if !dir.exists() {
            return Err(format!("Tests directory does not exist: {}", dir.display()));
        }

        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {e}"))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories (except coverage dir itself)
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name != "coverage" && dir_name != "inventory" {
                    self.scan_directory(&path, patterns)?;
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                self.scan_file(&path, patterns)?;
            }
        }

        Ok(())
    }

    fn scan_file(&mut self, file_path: &Path, patterns: &[Regex]) -> Result<(), String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read {}: {e}", file_path.display()))?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        for pattern in patterns {
            for capture in pattern.captures_iter(&content) {
                if let Some(tool_name) = capture.get(1) {
                    let tool = tool_name.as_str().to_string();
                    self.tool_references
                        .entry(tool)
                        .or_default()
                        .insert(file_name.clone());
                }
            }
        }

        Ok(())
    }

    /// Generate a complete coverage report
    pub fn generate_report(&mut self) -> Result<CoverageReport, String> {
        let inventory = self.load_inventory()?;
        self.scan_tests()?;

        let mut tools: Vec<ToolCoverage> = Vec::new();
        let mut category_coverage: BTreeMap<String, CategoryCoverage> = BTreeMap::new();
        let mut all_untested: Vec<String> = Vec::new();

        // Process each category
        for (cat_name, cat_def) in &inventory.categories {
            let mut tested_count = 0;
            let mut untested: Vec<String> = Vec::new();

            for tool_def in &cat_def.tools {
                let refs = self.tool_references.get(&tool_def.name);
                let test_files: Vec<String> = refs
                    .map(|r| r.iter().cloned().collect())
                    .unwrap_or_default();
                let tested = !test_files.is_empty();
                let test_count = test_files.len();

                if tested {
                    tested_count += 1;
                } else {
                    untested.push(tool_def.name.clone());
                    all_untested.push(tool_def.name.clone());
                }

                tools.push(ToolCoverage {
                    name: tool_def.name.clone(),
                    category: cat_name.clone(),
                    tested,
                    test_files,
                    test_count,
                });
            }

            let total = cat_def.tools.len();
            let coverage_percent = if total > 0 {
                (tested_count as f64 / total as f64) * 100.0
            } else {
                100.0
            };

            category_coverage.insert(
                cat_name.clone(),
                CategoryCoverage {
                    name: cat_name.clone(),
                    description: cat_def.description.clone(),
                    total_tools: total,
                    tested_tools: tested_count,
                    coverage_percent,
                    untested,
                },
            );
        }

        // Sort tools by category then name
        tools.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name))
        });
        all_untested.sort();

        let total_tools = inventory.total_tools;
        let tested_tools = tools.iter().filter(|t| t.tested).count();
        let overall_coverage = if total_tools > 0 {
            (tested_tools as f64 / total_tools as f64) * 100.0
        } else {
            100.0
        };

        // Convert BTreeMap to sorted Vec
        let categories: Vec<CategoryCoverage> = category_coverage.into_values().collect();

        Ok(CoverageReport {
            generated: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            total_tools,
            tested_tools,
            overall_coverage,
            categories,
            tools,
            untested_tools: all_untested,
        })
    }

    /// Generate a markdown report
    pub fn generate_markdown_report(&mut self) -> Result<String, String> {
        let report = self.generate_report()?;

        let mut md = String::new();
        md.push_str("# MCP Tool Coverage Report\n\n");
        md.push_str(&format!("**Generated**: {}\n\n", report.generated));
        md.push_str("## Summary\n\n");
        md.push_str(&format!(
            "| Metric | Value |\n|--------|-------|\n| Total Tools | {} |\n| Tested Tools | {} |\n| Coverage | {:.1}% |\n\n",
            report.total_tools,
            report.tested_tools,
            report.overall_coverage
        ));

        md.push_str("## Coverage by Category\n\n");
        md.push_str(
            "| Category | Total | Tested | Coverage |\n|----------|-------|--------|----------|\n",
        );

        for cat in &report.categories {
            md.push_str(&format!(
                "| {} | {} | {} | {:.1}% |\n",
                cat.name, cat.total_tools, cat.tested_tools, cat.coverage_percent
            ));
        }

        md.push_str("\n## Untested Tools\n\n");
        if report.untested_tools.is_empty() {
            md.push_str("All tools have test coverage!\n");
        } else {
            md.push_str(&format!(
                "**{} tools need tests:**\n\n",
                report.untested_tools.len()
            ));
            for tool in &report.untested_tools {
                md.push_str(&format!("- `{tool}`\n"));
            }
        }

        Ok(md)
    }

    /// Generate a dashboard markdown report
    pub fn generate_dashboard(&mut self) -> Result<String, String> {
        let report = self.generate_report()?;

        let mut md = String::new();

        // Header
        md.push_str("# MCP Tool Coverage Dashboard\n\n");
        md.push_str(&format!(
            "> **Last Updated**: {} | **Target**: 100% | **Threshold**: 60%\n\n",
            report.generated
        ));
        md.push_str("---\n\n");

        // Overall Status
        md.push_str("## Overall Status\n\n");

        // Progress bar (45 chars wide)
        let filled = (report.overall_coverage / 100.0 * 45.0) as usize;
        let empty = 45 - filled;
        md.push_str("```\n");
        md.push_str(&format!(
            "Coverage Progress: {:.1}%\n",
            report.overall_coverage
        ));
        md.push_str("=============================================\n");
        md.push_str(&format!(
            "[{}{}] {}/{} tools tested\n",
            "#".repeat(filled),
            " ".repeat(empty),
            report.tested_tools,
            report.total_tools
        ));
        md.push_str("```\n\n");

        // Status table
        let status_icon = if report.overall_coverage >= 100.0 {
            ":white_check_mark: Complete"
        } else if report.overall_coverage >= 60.0 {
            ":large_orange_diamond: Above threshold"
        } else {
            ":red_circle: Below threshold"
        };

        let threshold_icon = if report.overall_coverage >= 60.0 {
            ":white_check_mark: Met"
        } else {
            ":x: Not met"
        };

        md.push_str("| Metric | Value | Status |\n|--------|-------|--------|\n");
        md.push_str(&format!(
            "| **Total Tools** | {} | - |\n",
            report.total_tools
        ));
        md.push_str(&format!(
            "| **Tested Tools** | {} | - |\n",
            report.tested_tools
        ));
        md.push_str(&format!(
            "| **Untested Tools** | {} | - |\n",
            report.untested_tools.len()
        ));
        md.push_str(&format!(
            "| **Coverage** | {:.1}% | {} |\n",
            report.overall_coverage, status_icon
        ));
        md.push_str(&format!("| **Threshold** | 60.0% | {} |\n", threshold_icon));
        md.push_str("| **Target** | 100% | :construction: In progress |\n\n");
        md.push_str("---\n\n");

        // Category breakdown
        md.push_str("## Category Coverage Overview\n\n");

        // Fully covered
        md.push_str("### Fully Covered (100%)\n\n");
        md.push_str("| Category | Tools | Status |\n|----------|-------|--------|\n");
        let mut full_count = 0;
        for cat in &report.categories {
            if cat.coverage_percent >= 100.0 {
                md.push_str(&format!(
                    "| :white_check_mark: {} | {}/{} | Complete |\n",
                    cat.name, cat.tested_tools, cat.total_tools
                ));
                full_count += cat.total_tools;
            }
        }
        let full_categories = report
            .categories
            .iter()
            .filter(|c| c.coverage_percent >= 100.0)
            .count();
        md.push_str(&format!(
            "\n**Total: {} tools fully covered across {} categories**\n\n",
            full_count, full_categories
        ));

        // High coverage
        md.push_str("### High Coverage (>75%)\n\n");
        md.push_str(
            "| Category | Tools | Coverage | Gap |\n|----------|-------|----------|-----|\n",
        );
        let mut high_tested = 0;
        let mut high_gap = 0;
        for cat in &report.categories {
            if cat.coverage_percent >= 75.0 && cat.coverage_percent < 100.0 {
                let gap = cat.total_tools - cat.tested_tools;
                md.push_str(&format!(
                    "| :large_blue_diamond: {} | {}/{} | {:.1}% | {} tool{} |\n",
                    cat.name,
                    cat.tested_tools,
                    cat.total_tools,
                    cat.coverage_percent,
                    gap,
                    if gap == 1 { "" } else { "s" }
                ));
                high_tested += cat.tested_tools;
                high_gap += gap;
            }
        }
        if high_tested > 0 || high_gap > 0 {
            md.push_str(&format!(
                "\n**Total: {} tools with {} gaps**\n\n",
                high_tested, high_gap
            ));
        }

        // Medium coverage
        md.push_str("### Medium Coverage (40-75%)\n\n");
        md.push_str(
            "| Category | Tools | Coverage | Gap |\n|----------|-------|----------|-----|\n",
        );
        let mut med_tested = 0;
        let mut med_gap = 0;
        for cat in &report.categories {
            if cat.coverage_percent >= 40.0 && cat.coverage_percent < 75.0 {
                let gap = cat.total_tools - cat.tested_tools;
                md.push_str(&format!(
                    "| :large_orange_diamond: {} | {}/{} | {:.1}% | {} tool{} |\n",
                    cat.name,
                    cat.tested_tools,
                    cat.total_tools,
                    cat.coverage_percent,
                    gap,
                    if gap == 1 { "" } else { "s" }
                ));
                med_tested += cat.tested_tools;
                med_gap += gap;
            }
        }
        if med_tested > 0 || med_gap > 0 {
            md.push_str(&format!(
                "\n**Total: {} tools with {} gaps**\n\n",
                med_tested, med_gap
            ));
        }

        // Low coverage
        md.push_str("### Low Coverage (<40%)\n\n");
        md.push_str(
            "| Category | Tools | Coverage | Gap |\n|----------|-------|----------|-----|\n",
        );
        let mut low_tested = 0;
        let mut low_gap = 0;
        for cat in &report.categories {
            if cat.coverage_percent < 40.0 {
                let gap = cat.total_tools - cat.tested_tools;
                md.push_str(&format!(
                    "| :red_circle: {} | {}/{} | {:.1}% | {} tool{} |\n",
                    cat.name,
                    cat.tested_tools,
                    cat.total_tools,
                    cat.coverage_percent,
                    gap,
                    if gap == 1 { "" } else { "s" }
                ));
                low_tested += cat.tested_tools;
                low_gap += gap;
            }
        }
        if low_tested > 0 || low_gap > 0 {
            md.push_str(&format!(
                "\n**Total: {} tool{} covered with {} gaps - PRIORITY**\n\n",
                low_tested,
                if low_tested == 1 { "" } else { "s" },
                low_gap
            ));
        }

        md.push_str("---\n\n");

        // Priority gaps
        md.push_str("## Priority Gaps\n\n");
        md.push_str("### :red_circle: Critical Priority (0% Coverage Categories)\n\n");
        md.push_str("These categories have zero test coverage and should be addressed first:\n\n");

        let mut priority_num = 1;
        for cat in &report.categories {
            if cat.coverage_percent == 0.0 && !cat.untested.is_empty() {
                md.push_str(&format!(
                    "#### {}. {} ({} tools)\n",
                    priority_num,
                    cat.name,
                    cat.untested.len()
                ));
                md.push_str(&format!("{}\n```\n", cat.description));
                for tool in &cat.untested {
                    md.push_str(&format!("- {tool}\n"));
                }
                md.push_str("```\n\n");
                priority_num += 1;
            }
        }

        // Secondary priority - partial coverage categories with most gaps
        md.push_str("### :large_orange_diamond: Secondary Priority (Partial Coverage)\n\n");

        let mut partial_cats: Vec<_> = report
            .categories
            .iter()
            .filter(|c| c.coverage_percent > 0.0 && c.coverage_percent < 100.0)
            .collect();
        partial_cats.sort_by(|a, b| b.untested.len().cmp(&a.untested.len()));

        for cat in partial_cats.iter().take(5) {
            if !cat.untested.is_empty() {
                md.push_str(&format!(
                    "#### {} ({} untested tools)\n```\n",
                    cat.name,
                    cat.untested.len()
                ));
                for tool in &cat.untested {
                    md.push_str(&format!("- {tool}\n"));
                }
                md.push_str("```\n\n");
            }
        }

        md.push_str("---\n\n");

        // Quick actions
        md.push_str("## Quick Actions\n\n");
        md.push_str("### Run Coverage Check\n```bash\ncargo test -p communitas-mcp --test coverage_check -- --nocapture\n```\n\n");
        md.push_str("### Generate Test Stubs\n```bash\ncargo test -p communitas-mcp --test generate_stubs -- --nocapture\n```\n\n");
        md.push_str("### Run All MCP Tests\n```bash\ncargo test -p communitas-mcp\n```\n\n");

        md.push_str("---\n\n");

        // Legend
        md.push_str("## Legend\n\n");
        md.push_str("| Icon | Meaning |\n|------|---------|");
        md.push_str("\n| :white_check_mark: | 100% coverage |");
        md.push_str("\n| :large_blue_diamond: | 75-99% coverage |");
        md.push_str("\n| :large_orange_diamond: | 40-74% coverage |");
        md.push_str("\n| :red_circle: | <40% coverage |\n\n");

        md.push_str("---\n\n");
        md.push_str("*Dashboard auto-generated by coverage tracker. Run `cargo test -p communitas-mcp --test coverage_check` to update.*\n");

        Ok(md)
    }

    /// Get list of all tools from inventory
    pub fn get_all_tools(&self) -> Result<Vec<String>, String> {
        let inventory = self.load_inventory()?;
        let mut tools: BTreeSet<String> = BTreeSet::new();

        for cat_def in inventory.categories.values() {
            for tool_def in &cat_def.tools {
                tools.insert(tool_def.name.clone());
            }
        }

        Ok(tools.into_iter().collect())
    }

    /// Check if a specific tool has coverage
    #[allow(dead_code)]
    pub fn tool_has_coverage(&self, tool_name: &str) -> bool {
        self.tool_references.contains_key(tool_name)
    }

    /// Get the test files that reference a specific tool
    #[allow(dead_code)]
    pub fn get_tool_tests(&self, tool_name: &str) -> Vec<String> {
        self.tool_references
            .get(tool_name)
            .map(|refs| refs.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn get_test_paths() -> (PathBuf, PathBuf) {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let inventory_path = PathBuf::from(&manifest_dir).join("tests/inventory/tools.json");
        let tests_path = PathBuf::from(&manifest_dir).join("tests");
        (inventory_path, tests_path)
    }

    #[test]
    fn test_load_inventory() {
        let (inventory_path, tests_path) = get_test_paths();

        if !inventory_path.exists() {
            println!("Inventory file not found, skipping test");
            return;
        }

        let tracker = CoverageTracker::new(&inventory_path, &tests_path);
        let inventory = tracker.load_inventory().expect("Failed to load inventory");

        assert!(inventory.total_tools > 0);
        assert!(!inventory.categories.is_empty());
    }

    #[test]
    fn test_scan_tests() {
        let (inventory_path, tests_path) = get_test_paths();

        if !inventory_path.exists() || !tests_path.exists() {
            println!("Required files not found, skipping test");
            return;
        }

        let mut tracker = CoverageTracker::new(&inventory_path, &tests_path);
        tracker.scan_tests().expect("Failed to scan tests");

        // Should find at least some tool references
        assert!(
            !tracker.tool_references.is_empty(),
            "No tool references found in tests"
        );
    }

    #[test]
    fn test_generate_report() {
        let (inventory_path, tests_path) = get_test_paths();

        if !inventory_path.exists() || !tests_path.exists() {
            println!("Required files not found, skipping test");
            return;
        }

        let mut tracker = CoverageTracker::new(&inventory_path, &tests_path);
        let report = tracker
            .generate_report()
            .expect("Failed to generate report");

        assert!(report.total_tools > 0);
        assert!(!report.categories.is_empty());
        println!(
            "Coverage: {:.1}% ({}/{})",
            report.overall_coverage, report.tested_tools, report.total_tools
        );
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Tool Coverage Check Test
//!
//! Run with: cargo test -p communitas-mcp --test coverage_check -- --nocapture
//!
//! This test generates a coverage report showing which MCP tools have tests.

mod coverage;

use coverage::{CoverageTracker, QualityTracker};
use std::env;
use std::fs;
use std::path::PathBuf;

fn get_paths() -> (PathBuf, PathBuf) {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let inventory_path = PathBuf::from(&manifest_dir).join("tests/inventory/tools.json");
    let tests_path = PathBuf::from(&manifest_dir).join("tests");
    (inventory_path, tests_path)
}

#[test]
fn check_tool_coverage() {
    let (inventory_path, tests_path) = get_paths();

    if !inventory_path.exists() {
        panic!(
            "Inventory file not found at: {}. Run Task 1 first to create the inventory.",
            inventory_path.display()
        );
    }

    let mut tracker = CoverageTracker::new(&inventory_path, &tests_path);
    let report = tracker
        .generate_report()
        .expect("Failed to generate coverage report");

    // Print summary
    println!("\n===== MCP TOOL COVERAGE REPORT =====\n");
    println!("Total Tools: {}", report.total_tools);
    println!("Tested Tools: {}", report.tested_tools);
    println!("Overall Coverage: {:.1}%\n", report.overall_coverage);

    // Print category breakdown
    println!("Coverage by Category:");
    println!("{:-<60}", "");
    println!(
        "{:<25} {:>8} {:>8} {:>10}",
        "Category", "Total", "Tested", "Coverage"
    );
    println!("{:-<60}", "");

    for cat in &report.categories {
        println!(
            "{:<25} {:>8} {:>8} {:>9.1}%",
            cat.name, cat.total_tools, cat.tested_tools, cat.coverage_percent
        );
    }
    println!("{:-<60}", "");

    // Print untested tools summary
    if !report.untested_tools.is_empty() {
        println!("\nUntested Tools ({}):", report.untested_tools.len());
        for (i, tool) in report.untested_tools.iter().enumerate() {
            if i < 20 {
                println!("  - {tool}");
            } else if i == 20 {
                println!("  ... and {} more", report.untested_tools.len() - 20);
                break;
            }
        }
    } else {
        println!("\n All tools have test coverage!");
    }

    // Save full report to file
    let report_path =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
            .join("tests/coverage/report.json");

    let json = serde_json::to_string_pretty(&report).expect("Failed to serialize report");
    fs::write(&report_path, &json).expect("Failed to write report");
    println!("\nFull report saved to: {}", report_path.display());

    // Also save markdown report
    let md_report = tracker
        .generate_markdown_report()
        .expect("Failed to generate markdown report");
    let md_path =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
            .join("tests/coverage/REPORT.md");
    fs::write(&md_path, &md_report).expect("Failed to write markdown report");
    println!("Markdown report saved to: {}", md_path.display());

    // Also save dashboard
    let dashboard = tracker
        .generate_dashboard()
        .expect("Failed to generate dashboard");
    let dashboard_path =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
            .join("tests/coverage/DASHBOARD.md");
    fs::write(&dashboard_path, &dashboard).expect("Failed to write dashboard");
    println!("Dashboard saved to: {}", dashboard_path.display());
}

#[test]
fn verify_inventory_coverage_matches() {
    let (inventory_path, tests_path) = get_paths();

    if !inventory_path.exists() {
        println!("Skipping: inventory not found");
        return;
    }

    let mut tracker = CoverageTracker::new(&inventory_path, &tests_path);
    let inventory_tools = tracker.get_all_tools().expect("Failed to get tools");
    let report = tracker
        .generate_report()
        .expect("Failed to generate report");

    // Verify inventory tool count matches report
    assert_eq!(
        inventory_tools.len(),
        report.total_tools,
        "Tool count mismatch between inventory and report"
    );
}

#[test]
fn test_coverage_threshold() {
    let (inventory_path, tests_path) = get_paths();

    if !inventory_path.exists() {
        println!("Skipping: inventory not found");
        return;
    }

    let mut tracker = CoverageTracker::new(&inventory_path, &tests_path);
    let report = tracker
        .generate_report()
        .expect("Failed to generate report");

    // This test documents current coverage but doesn't fail
    // Update the threshold as coverage improves
    let min_coverage = 30.0; // Start low, increase as we add tests

    println!(
        "\nCoverage: {:.1}% (minimum required: {:.1}%)",
        report.overall_coverage, min_coverage
    );

    // Note: This is informational for now - we'll enforce after adding more tests
    if report.overall_coverage < min_coverage {
        println!(
            "WARNING: Coverage is below {:.1}% threshold. {} tools need tests.",
            min_coverage,
            report.untested_tools.len()
        );
    }
}

#[test]
fn check_test_quality() {
    let (inventory_path, tests_path) = get_paths();

    let tracker = QualityTracker::with_inventory(&tests_path, &inventory_path);
    let report = tracker.analyze().expect("Failed to analyze test quality");

    // Print summary
    println!("\n===== MCP TEST QUALITY REPORT =====\n");
    println!("Total Tests: {}", report.total_tests);
    println!("Complete Tests: {}", report.complete_tests);
    println!("Stub Tests: {}", report.stub_tests);
    println!("TODO Comments: {}", report.total_todos);
    println!("Quality Score: {:.1}%\n", report.overall_quality * 100.0);

    // Print category breakdown
    println!("Quality by Category:");
    println!("{:-<70}", "");
    println!(
        "{:<20} {:>8} {:>10} {:>8} {:>8} {:>10}",
        "Category", "Total", "Complete", "Stubs", "TODOs", "Quality"
    );
    println!("{:-<70}", "");

    for cat in &report.categories {
        let icon = if cat.avg_quality >= 0.9 {
            "✅"
        } else if cat.avg_quality >= 0.5 {
            "⚠️"
        } else {
            "❌"
        };
        println!(
            "{} {:<18} {:>8} {:>10} {:>8} {:>8} {:>9.1}%",
            icon,
            cat.name,
            cat.total_tests,
            cat.complete_tests,
            cat.stub_tests,
            cat.total_todos,
            cat.avg_quality * 100.0
        );
    }
    println!("{:-<70}", "");

    // Save quality report
    let quality_path =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()))
            .join("tests/coverage/QUALITY.md");

    let md = tracker
        .generate_markdown()
        .expect("Failed to generate markdown");
    fs::write(&quality_path, &md).expect("Failed to write quality report");
    println!("\nQuality report saved to: {}", quality_path.display());

    // Quality threshold (informational for now)
    let min_quality = 0.5; // 50% of tests should be complete (not stubs)
    if report.overall_quality < min_quality {
        println!(
            "\n⚠️ Quality below {:.0}% threshold: {} stub tests need assertions",
            min_quality * 100.0,
            report.stub_tests
        );
    }
}

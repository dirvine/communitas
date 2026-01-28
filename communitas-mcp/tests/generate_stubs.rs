// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Test stub generator runner
//!
//! Run with: cargo test -p communitas-mcp --test generate_stubs -- --nocapture

mod generator;

use generator::{GeneratorConfig, TestGenerator};
use std::fs;

#[test]
fn generate_test_stubs() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let inventory_path = format!("{}/tests/inventory/tools.json", manifest_dir);
    let tests_dir = format!("{}/tests", manifest_dir);
    let output_path = format!("{}/tests/generated_stubs.rs", manifest_dir);
    let report_path = format!("{}/tests/generator/REPORT.md", manifest_dir);

    let config = GeneratorConfig {
        inventory_path,
        tests_dir,
        output_path: output_path.clone(),
        in_process: true,
    };

    let mut generator = TestGenerator::new(config);

    // Load inventory
    if let Err(e) = generator.load_inventory() {
        println!("Warning: Failed to load inventory: {}", e);
        println!("Skipping stub generation.");
        return;
    }

    // Scan existing tests
    if let Err(e) = generator.scan_tested_tools() {
        println!("Warning: Failed to scan tests: {}", e);
    }

    // Get untested tools
    let untested = generator.get_untested_tools();

    println!("\n=== MCP Test Generator ===\n");
    println!("Untested tools: {}", untested.len());

    if untested.is_empty() {
        println!("All tools have test coverage!");
        return;
    }

    // Generate stubs
    let stubs = generator.generate_all_stubs();
    println!("Generated {} test stubs", stubs.len());

    // Write stubs
    if let Err(e) = generator.write_stubs(&stubs) {
        println!("Error writing stubs: {}", e);
        return;
    }
    println!("Stubs written to: {}", output_path);

    // Generate and save report
    let report = generator.generate_report();
    if let Err(e) = fs::write(&report_path, &report) {
        println!("Warning: Failed to write report: {}", e);
    } else {
        println!("Report written to: {}", report_path);
    }

    println!("\n{}", report);
}

#[test]
fn list_untested_tools() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let inventory_path = format!("{}/tests/inventory/tools.json", manifest_dir);
    let tests_dir = format!("{}/tests", manifest_dir);

    let config = GeneratorConfig {
        inventory_path,
        tests_dir,
        output_path: String::new(),
        in_process: true,
    };

    let mut generator = TestGenerator::new(config);

    if generator.load_inventory().is_err() {
        println!("Could not load inventory");
        return;
    }

    if generator.scan_tested_tools().is_err() {
        println!("Could not scan tests");
        return;
    }

    let untested = generator.get_untested_tools();

    println!("\n=== Untested MCP Tools ===\n");

    let mut current_category = String::new();
    for (category, tool) in &untested {
        if *category != current_category {
            println!("\n## {}", category);
            current_category = category.clone();
        }
        println!("  - {} - {}", tool.name, tool.description);
    }

    println!("\nTotal untested: {}", untested.len());
}

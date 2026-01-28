// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Tool Coverage Tracking System
//!
//! This module provides automated coverage tracking for MCP tools by:
//! - Parsing the tool inventory from inventory.json
//! - Scanning test files for tool references
//! - Generating coverage matrices and reports
//! - Tracking test quality (stub vs complete tests)
//!
//! Run coverage check: `cargo test -p communitas-mcp --test coverage_check`

mod quality;
mod tracker;

pub use quality::QualityTracker;
// QualityReport is used internally by QualityTracker
#[allow(unused_imports)]
pub use quality::QualityReport;
pub use tracker::CoverageTracker;

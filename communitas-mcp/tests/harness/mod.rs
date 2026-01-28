// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Test Harness
//!
//! Provides reusable test infrastructure for MCP tool testing:
//! - `McpTestClient`: Async client supporting HTTP and stdio transports
//! - `McpTestNode`: Spawns and manages MCP server processes
//! - Assertion helpers for validating tool responses
//!
//! # Usage
//!
//! ```rust,ignore
//! use harness::{McpTestNode, McpTestClient};
//!
//! // HTTP transport
//! let node = McpTestNode::start("test").await;
//! let result = node.call_tool("get_profile", json!({})).await;
//! assert!(result.is_success());
//!
//! // In-process testing (no HTTP)
//! let client = McpTestClient::in_process().await;
//! let result = client.call_tool("list_entities", json!({})).await;
//! ```

mod client;
mod results;

#[allow(unused_imports)]
pub use client::{McpTestClient, ToolAssert, ToolResult};

// Re-export for HTTP transport tests and result aggregation
#[allow(unused_imports)]
pub use client::{McpTestNode, Transport};
#[allow(unused_imports)]
pub use results::{ResultAggregator, TestSummary};

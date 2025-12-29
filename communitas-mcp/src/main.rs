// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license
//
//! Communitas MCP Server
//!
//! Implements the Model Context Protocol (MCP) to enable AI agents to control
//! Communitas via JSON-RPC 2.0 over stdio.
//!
//! ## Protocol
//! - Input: JSON-RPC 2.0 requests on stdin
//! - Output: JSON-RPC 2.0 responses on stdout
//! - Logging: stderr
//!
//! ## Tools
//! Commands are exposed as MCP tools that can be invoked by AI agents.
//!
//! ## Resources
//! Queries are exposed as MCP resources for reading application state.

mod protocol;
mod server;
mod tools;

use anyhow::Result;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing to stderr (stdout is for JSON-RPC)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "communitas_mcp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    info!("Starting Communitas MCP Server");

    // Run the MCP server
    server::run().await?;

    info!("MCP Server shutdown complete");
    Ok(())
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

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
//! ## Authentication Modes
//! - Default: Requires `authenticate` or `create_vault` before other tools
//! - Demo (`--demo`): Auto-initializes with temporary identity for testing
//!
//! ## Tools
//! Commands are exposed as MCP tools that can be invoked by AI agents.
//!
//! ## Resources
//! Queries are exposed as MCP resources for reading application state.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

mod http;
mod presence;
mod protocol;
mod server;
mod tls;
mod tools;
mod ui_resources;
mod ui_session;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Communitas MCP Server - AI agent interface for Communitas
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Run in demo mode with auto-initialized temporary identity.
    /// Skips authentication and starts with full access.
    /// Use for testing and development only.
    #[arg(long, default_value_t = false)]
    pub demo: bool,

    /// Storage directory for demo mode.
    /// Defaults to system temp directory.
    #[arg(long)]
    pub storage_dir: Option<String>,

    /// Four-word identity to use in demo mode.
    /// If not provided, a random identity is generated.
    #[arg(long)]
    pub four_words: Option<String>,

    /// Display name for the demo session.
    #[arg(long, default_value = "MCP Demo User")]
    pub display_name: String,

    // === HTTP Transport Options ===
    /// Run as HTTP server instead of stdio.
    /// Exposes MCP protocol over HTTP POST /mcp endpoint.
    #[arg(long, default_value_t = false)]
    pub http: bool,

    /// Enable TLS (HTTPS) with ML-DSA-65 raw public keys.
    /// Requires --http flag. Uses RFC 7250 for post-quantum TLS.
    #[arg(long, default_value_t = false)]
    pub tls: bool,

    /// Listen address for HTTP server.
    /// Default: 127.0.0.1:8080 (HTTP) or 127.0.0.1:8443 (HTTPS)
    #[arg(long)]
    pub listen: Option<String>,

    /// Disable client certificate verification (TLS only).
    /// WARNING: Only use for development/testing.
    /// In production, always verify client keys.
    #[arg(long, default_value_t = false)]
    pub no_client_auth: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // For HTTP mode, log to stderr; for stdio mode, also log to stderr (stdout is for JSON-RPC)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "communitas_mcp=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Log startup mode
    let mode = if args.demo { "DEMO" } else { "authenticated" };
    let transport = if args.http {
        if args.tls {
            "HTTPS (ML-DSA-65 raw public keys)"
        } else {
            "HTTP"
        }
    } else {
        "stdio"
    };
    info!("Starting Communitas MCP Server ({mode} mode, {transport} transport)");

    // Validate TLS flag requires HTTP
    if args.tls && !args.http {
        return Err(anyhow::anyhow!(
            "--tls flag requires --http flag (TLS is only for HTTP transport)"
        ));
    }

    // Run appropriate server based on transport mode
    if args.http {
        if args.tls {
            let tls_config = http::create_tls_config(&args)
                .map_err(|e| anyhow::anyhow!("TLS configuration failed: {e}"))?;
            http::run_https(args, tls_config).await?;
        } else {
            http::run_http(args).await?;
        }
    } else {
        // stdio mode (default)
        server::run(args).await?;
    }

    info!("MCP Server shutdown complete");
    Ok(())
}

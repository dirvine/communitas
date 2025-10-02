// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

// Security: Enforce no-panic policy in production code
#![cfg_attr(
    not(test),
    forbid(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Allow these in tests for convenience
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod app;
mod backend;
mod handlers;
mod state;
mod ui;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Communitas TUI - Terminal interface for testing Communitas backend
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Four-word identity to use (e.g., ocean-forest-moon-star)
    #[arg(short, long)]
    identity: Option<String>,

    /// Display name for the identity
    #[arg(short, long, default_value = "TUI User")]
    name: String,

    /// Device name
    #[arg(short = 'd', long, default_value = "TUI Device")]
    device: String,

    /// Data directory for storage
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Skip network initialization (offline mode)
    #[arg(long)]
    offline: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    utils::logger::init(args.debug)?;

    tracing::info!("Starting Communitas TUI v{}", env!("CARGO_PKG_VERSION"));

    // Determine data directory
    let data_dir = args.data_dir.unwrap_or_else(|| {
        dirs::data_local_dir()
            .map(|d| d.join("communitas-tui"))
            .unwrap_or_else(|| PathBuf::from(".communitas-tui"))
    });

    std::fs::create_dir_all(&data_dir)?;
    tracing::info!("Data directory: {}", data_dir.display());

    // Create and run application
    let mut app = app::App::new(data_dir, args.offline).await?;

    // Initialize identity if provided, otherwise show auth screen
    if let Some(identity) = args.identity {
        tracing::info!("Initializing with identity: {}", identity);
        app.initialize_identity(identity, args.name, args.device)
            .await?;
    } else {
        tracing::info!("No identity provided, starting with auth screen");
        app.start_with_auth();
    }

    // Run the TUI
    app.run().await?;

    Ok(())
}

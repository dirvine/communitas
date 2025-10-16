// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

// Security: CLI tools may use unwrap in controlled contexts
// Core library crates maintain strict no-unwrap policies

mod app;
mod backend;
mod control_api;
mod handlers;
mod state;
mod ui;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Try to self-update the binary using GitHub releases
fn try_self_update() -> Result<Option<String>> {
    use self_update::cargo_crate_version;
    let owner = std::env::var("COMMUNITAS_UPDATE_REPO_OWNER")
        .unwrap_or_else(|_| "dirvine".to_string());
    let name = std::env::var("COMMUNITAS_UPDATE_REPO_NAME")
        .unwrap_or_else(|_| "communitas".to_string());

    let mut cfg = self_update::backends::github::Update::configure();
    let builder = cfg
        .repo_owner(&owner)
        .repo_name(&name)
        .bin_name("communitas-tui")
        .current_version(cargo_crate_version!());

    match builder.build()?.update() {
        Ok(status) => Ok(Some(status.version().to_string())),
        Err(e) => Err(e.into()),
    }
}

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

    /// PBKDF2 iterations for testing (default: 100000)
    #[arg(long, default_value = "100000")]
    pbkdf2_iterations: u32,

    /// Disable keyring for testing
    #[arg(long)]
    no_keyring: bool,

    /// Enable HTTP control API on specified port
    #[arg(long)]
    control_port: Option<u16>,

    /// Run only HTTP control API without TUI (requires --control-port)
    #[arg(long, requires = "control_port")]
    api_only: bool,

    /// Perform self-update from GitHub Releases and exit
    #[arg(long, default_value_t = false)]
    self_update: bool,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Self-update mode: do not start TUI
    if args.self_update {
        match tokio::task::spawn_blocking(try_self_update).await {
            Ok(Ok(Some(ver))) => {
                println!("✅ Successfully updated to version: {}", ver);
                println!("Please restart the application to use the new version.");
            }
            Ok(Ok(None)) => {
                println!("Already on latest version");
            }
            Ok(Err(e)) => {
                eprintln!("❌ Self-update error: {:#}", e);
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!("❌ Spawn error: {:#}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

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

    // Start HTTP control API if requested
    if let Some(control_port) = args.control_port {
        tracing::info!("Starting HTTP control API on port {}", control_port);

        // Create separate backend for control API
        let control_data_dir = data_dir.join("control");
        std::fs::create_dir_all(&control_data_dir)?;

        let control_backend = backend::Backend::new_with_config(
            control_data_dir,
            args.pbkdf2_iterations,
            !args.no_keyring,
            args.offline,
        )
        .await?;

        // Wrap in Arc<Mutex> for shared access
        let control_state = std::sync::Arc::new(tokio::sync::Mutex::new(control_backend));

        // Spawn control server task
        let control_server = control_api::ControlServer::new(control_port, control_state);
        tokio::spawn(async move {
            if let Err(e) = control_server.run().await {
                tracing::error!("Control API server error: {}", e);
            }
        });

        tracing::info!(
            "HTTP control API started on http://localhost:{}",
            control_port
        );

        // If API-only mode, just wait forever
        if args.api_only {
            tracing::info!("Running in API-only mode (no TUI)");
            tracing::info!("Press Ctrl+C to exit");

            // Wait for Ctrl+C
            tokio::signal::ctrl_c()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to wait for Ctrl+C: {}", e))?;

            tracing::info!("Shutting down...");
            return Ok(());
        }
    }

    // Create and run application with custom configuration
    let mut app = app::App::new_with_config(
        data_dir,
        args.pbkdf2_iterations,
        !args.no_keyring,
        args.offline,
    )
    .await?;

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

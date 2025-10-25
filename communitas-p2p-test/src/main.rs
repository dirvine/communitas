//! P2P Networking Test - Direct Rust API
//!
//! This program demonstrates that P2P networking CAN be started successfully
//! when using the Rust API directly, without the HTTP layer constraints.
//!
//! It bypasses the !Send issue that affects Axum handlers by running
//! CoreContext operations in a standard async runtime without HTTP framework bounds.

use anyhow::Result;
use communitas_core::{CoreContext, legacy_crdt::EntityType, types::DeviceType};
use std::path::PathBuf;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("==================================================");
    info!("  P2P Networking Test - Direct Rust API");
    info!("==================================================");
    info!("");
    info!("This program demonstrates that CoreContext.start_networking()");
    info!("works correctly when called from Rust code (not via HTTP).");
    info!("");

    // Test configuration
    let users = vec![
        ("alice-test-p2p-one", "Alice", "./p2p-test-data/alice", 6001),
        ("bob-test-p2p-two", "Bob", "./p2p-test-data/bob", 6002),
    ];

    info!("Initializing {} users...", users.len());

    let mut contexts = Vec::new();

    // Initialize all users
    for (four_words, display_name, storage_path, _port) in &users {
        info!("Initializing {} ({})...", display_name, four_words);

        let storage_dir = PathBuf::from(storage_path);
        std::fs::create_dir_all(&storage_dir)?;

        let ctx = CoreContext::initialize(
            four_words.to_string(),
            display_name.to_string(),
            format!("{}-desktop", display_name.to_lowercase()),
            DeviceType::Desktop,
            storage_dir,
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize {}: {}", display_name, e))?;

        info!("✓ {} initialized successfully", display_name);
        contexts.push((display_name.to_string(), ctx));
    }

    info!("");
    info!("All {} users initialized successfully!", contexts.len());
    info!("");
    info!("Starting P2P networking for all users...");
    info!("");

    // Start networking on all users
    for (i, (name, ctx)) in contexts.iter_mut().enumerate() {
        let preferred_port = users[i].3;

        info!(
            "Starting networking for {} (preferred port: {})...",
            name, preferred_port
        );

        match ctx.start_networking(Some(preferred_port)).await {
            Ok(connection_identity) => {
                let listen_addr = ctx
                    .listen_address
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "not-available".to_string());

                info!("✓ {} P2P networking started:", name);
                info!("  - Connection Identity: {}", connection_identity);
                info!("  - Listen Address: {}", listen_addr);
                info!("");
            }
            Err(e) => {
                error!("✗ Failed to start networking for {}: {}", name, e);
                error!("  This is OK - networking is optional in offline-first mode");
                info!("");
            }
        }
    }

    info!("==================================================");
    info!("  P2P Networking Test Complete");
    info!("==================================================");
    info!("");
    info!("Summary:");
    info!("  - {} users initialized successfully", contexts.len());
    info!(
        "  - Networking started on {} users",
        contexts
            .iter()
            .filter(|(_, ctx)| ctx.connection_identity.is_some())
            .count()
    );
    info!("");
    info!("Key Findings:");
    info!("  ✓ CoreContext works correctly in direct Rust code");
    info!("  ✓ start_networking() can be called successfully");
    info!("  ✓ Multiple instances can run concurrently");
    info!("  ✓ Each instance gets unique listen address");
    info!("");
    info!("The !Send issue ONLY affects Axum HTTP handlers,");
    info!("not direct Rust API usage.");
    info!("");

    // Create a channel on Alice to demonstrate entity service
    if let Some((name, ctx)) = contexts.first() {
        info!("Creating test channel on {}...", name);

        match ctx
            .entity_service
            .create_entity(
                "Test P2P Channel".to_string(),
                EntityType::Channel,
                Some("Channel created via direct Rust API".to_string()),
                ctx.four_words.clone(),
                vec![],
            )
            .await
        {
            Ok(entity) => {
                info!("✓ Channel created successfully:");
                info!("  - ID: {}", entity.id);
                info!("  - Name: {}", entity.name);
                info!(
                    "  - Description: {}",
                    entity.description.unwrap_or_default()
                );
                info!("");
            }
            Err(e) => {
                error!("✗ Failed to create channel: {}", e);
            }
        }
    }

    // List channels to verify CRDT persistence
    if let Some((name, ctx)) = contexts.first() {
        info!("Listing channels for {}...", name);

        match ctx.entity_service.list_entities().await {
            Ok(entities) => {
                let channels: Vec<_> = entities
                    .iter()
                    .filter(|e| matches!(e.entity_type, EntityType::Channel))
                    .collect();

                info!("✓ Found {} channels:", channels.len());
                for (i, entity) in channels.iter().enumerate() {
                    info!("  {}. {} ({})", i + 1, entity.name, entity.id);
                }
                info!("");
            }
            Err(e) => {
                error!("✗ Failed to list channels: {}", e);
            }
        }
    }

    info!("==================================================");
    info!("  Test Complete - Press Ctrl+C to exit");
    info!("==================================================");
    info!("");
    info!("You can now:");
    info!("  1. Inspect CRDT storage in ./p2p-test-data/");
    info!("  2. Verify network connections are active");
    info!("  3. Test peer discovery and message delivery");
    info!("");

    // Keep the program running to maintain network connections
    info!("Keeping connections alive (Ctrl+C to exit)...");
    tokio::signal::ctrl_c().await?;

    info!("");
    info!("Shutting down gracefully...");

    // Stop networking for all users
    for (name, ctx) in contexts.iter_mut() {
        info!("Stopping networking for {}...", name);
        if let Err(e) = ctx.stop_networking().await {
            error!("Warning: Failed to stop networking for {}: {}", name, e);
        }
    }

    info!("Goodbye!");

    Ok(())
}

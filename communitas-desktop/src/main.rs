// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod container;
mod core_cmds;
mod core_commands;
mod core_groups;
mod network;
mod security;
mod storage_fs;
mod sync;

use communitas_core::CoreContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[tauri::command]
async fn health() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "ok",
        "saorsa_core": saorsa_core::VERSION,
        "app": env!("CARGO_PKG_VERSION"),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize PQC crypto provider
    // Note: saorsa-pqc handles its own crypto provider initialization

    // Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,communitas=debug,saorsa_core=debug".to_string()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("Starting Communitas (saorsa-core integrated)");

    let mut builder = tauri::Builder::default()
        // Shared saorsa-core context (initialized via core_initialize)
        .manage(Arc::new(RwLock::new(Option::<CoreContext>::None)))
        // Container engine state
        .manage(Arc::new(RwLock::new(
            Option::<container::EngineState>::None,
        )))
        // Sync watcher state
        .manage(Arc::new(RwLock::new(sync::TipWatcherState::default())))
        // Network runtime state
        .manage(Arc::new(RwLock::new(network::NetworkRuntime::default())))
        // Raw SPKI pinning state
        .manage(Arc::new(RwLock::new(
            security::raw_spki::RawSpkiState::default(),
        )));

    // Only include the MCP plugin in development builds
    #[cfg(debug_assertions)]
    {
        if std::env::var("TAURI_MCP_DISABLED").is_ok() {
            info!("MCP plugin disabled by TAURI_MCP_DISABLED environment variable");
        } else {
            info!("Development build detected, enabling MCP plugin");

            // Use a unique socket path to avoid conflicts
            let socket_path = if let Ok(path) = std::env::var("TAURI_MCP_SOCKET_PATH") {
                std::path::PathBuf::from(path)
            } else {
                // Generate a unique socket path using process ID
                let pid = std::process::id();
                std::path::PathBuf::from(format!("/tmp/tauri-mcp-communitas-{}.sock", pid))
            };

            info!("Using MCP socket path: {:?}", socket_path);

            builder = builder.plugin(tauri_plugin_mcp::init_with_config(
                tauri_plugin_mcp::PluginConfig::new("Communitas".to_string())
                    .start_socket_server(true)
                    .socket_path(socket_path),
            ));
        }
    }

    builder = builder.invoke_handler(tauri::generate_handler![
        // Core bindings (pointers-only DHT surface)
        core_cmds::core_claim,
        core_cmds::core_advertise,
        core_cmds::container_put,
        core_cmds::container_get,
        // Container engine
        container::container_init,
        container::container_put_object,
        container::container_get_object,
        container::container_apply_ops,
        container::container_current_tip,
        core_commands::core_initialize,
        core_commands::core_create_channel,
        core_commands::core_get_channels,
        core_commands::core_add_reaction,
        core_commands::core_send_message_to_channel,
        core_commands::core_channel_recipients,
        core_commands::core_channel_list_members,
        core_commands::core_channel_invite_by_words,
        core_commands::core_resolve_channel_members,
        core_commands::core_create_thread,
        core_commands::core_subscribe_messages,
        core_commands::core_private_put,
        core_commands::core_private_get,
        core_commands::core_send_message_to_recipients,
        core_commands::core_get_bootstrap_nodes,
        core_commands::core_update_bootstrap_nodes,
        core_groups::core_group_create,
        core_groups::core_group_add_member,
        core_groups::core_group_remove_member,
        // Storage API
        storage_fs::core_storage_list,
        storage_fs::core_storage_read,
        storage_fs::core_storage_write,
        storage_fs::core_storage_mkdir,
        storage_fs::core_storage_delete,
        storage_fs::core_storage_rename,
        storage_fs::core_storage_stats,
        // Network helpers
        network::validate_four_words,
        network::connect_via_four_words,
        network::connect_to_network,
        network::disconnect_from_network,
        network::get_endpoint_four_words,
        network::get_network_status,
        network::get_network_info,
        network::get_user_four_words,
        // Sync + Repair
        sync::sync_start_tip_watcher,
        sync::sync_stop_tip_watcher,
        sync::sync_repair_fec,
        sync::sync_fetch_deltas,
        security::raw_spki::sync_set_quic_pinned_spki,
        security::raw_spki::sync_clear_quic_pinned_spki,
        health,
    ]);

    builder
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Failed to run Tauri app: {}", e))?;

    Ok(())
}

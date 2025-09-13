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
mod security;
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

    let _ = tauri::Builder::default()
        // Shared saorsa-core context (initialized via core_initialize)
        .manage(Arc::new(RwLock::new(
            Option::<CoreContext>::None,
        )))
        // Container engine state
        .manage(Arc::new(RwLock::new(
            Option::<container::EngineState>::None,
        )))
        // Sync watcher state
        .manage(Arc::new(RwLock::new(sync::TipWatcherState::default())))
        // Raw SPKI pinning state
        .manage(Arc::new(RwLock::new(
            security::raw_spki::RawSpkiState::default(),
        )))
        .invoke_handler(tauri::generate_handler![
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
            // Sync + Repair
            sync::sync_start_tip_watcher,
            sync::sync_stop_tip_watcher,
            sync::sync_repair_fec,
            sync::sync_fetch_deltas,
            security::raw_spki::sync_set_quic_pinned_spki,
            security::raw_spki::sync_clear_quic_pinned_spki,
            health,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!());

    Ok(())
}

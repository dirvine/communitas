// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod core_context;
mod core_commands;
mod core_groups;

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
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,communitas=debug,saorsa_core=debug".to_string()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("Starting Communitas (saorsa-core integrated)");

    tauri::Builder::default()
        // Shared saorsa-core context (initialized via core_initialize)
        .manage(Arc::new(RwLock::new(Option::<core_context::CoreContext>::None)))
        .invoke_handler(tauri::generate_handler![
            core_commands::core_initialize,
            core_commands::core_create_channel,
            core_commands::core_get_channels,
            core_commands::core_add_reaction,
            core_commands::core_post_message,
            core_commands::core_send_message_to_channel,
            core_commands::core_channel_recipients,
            core_commands::core_create_thread,
            core_commands::core_subscribe_messages,
            core_commands::core_private_put,
            core_commands::core_private_get,
            core_commands::core_website_publish,
            core_commands::core_website_get_manifest,
            core_commands::core_website_publish_receipt,
            core_commands::core_website_publish_and_update_identity,
            core_commands::core_website_set_home,
            core_commands::core_disk_write,
            core_commands::core_disk_read,
            core_commands::core_disk_list,
            core_commands::core_disk_delete,
            core_commands::core_disk_sync,
            core_commands::core_identity_set_website_root,
            core_commands::core_send_message_to_recipients,
            core_groups::core_group_create,
            core_groups::core_group_add_member,
            core_groups::core_group_remove_member,
            health,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!());

    Ok(())
}

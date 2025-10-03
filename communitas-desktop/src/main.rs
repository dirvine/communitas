// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Security: Enforce no-panic policy in production code
#![cfg_attr(
    not(test),
    forbid(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Allow these in tests for convenience
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod commands;
mod container;
mod core_cmds;
mod core_commands;
mod core_groups;
mod core_storage;
mod crdt_manager;
mod network;
mod security;
mod services;
mod storage_fs;
mod sync;

use commands::{auth::AppState, org_commands::OrgState};
use communitas_core::CoreContext;
use crdt_manager::CrdtManager;
use services::{channel_service::ChannelService, issue_service::IssueService};
use std::{path::PathBuf, sync::Arc};
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

    // Initialize CRDT manager and services
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("communitas");
    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("communitas.db");
    let crdt_manager = Arc::new(CrdtManager::new(&db_path).await?);

    let channel_service = Arc::new(ChannelService::new(crdt_manager.clone()));
    let issue_service = Arc::new(IssueService::new(crdt_manager.clone()));

    let org_state = OrgState {
        channel_service,
        issue_service,
    };

    // Initialize encrypted storage app state
    let app_state = AppState::new();

    let builder = tauri::Builder::default()
        // Auth and encrypted storage state
        .manage(app_state)
        // Organization services state
        .manage(org_state)
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
        )))
        .invoke_handler(tauri::generate_handler![
        // Auth commands
        commands::auth::auth_initialize,
        commands::auth::auth_create_vault,
        commands::auth::auth_login,
        commands::auth::auth_login_password_only,
        commands::auth::auth_logout,
        commands::auth::auth_get_session,
        commands::auth::auth_list_vaults,
        commands::auth::auth_check_session,
        // Auth config commands
        commands::auth::auth_get_config,
        commands::auth::auth_try_auto_login,
        commands::auth::auth_get_recent_identities,
        commands::auth::auth_set_auto_login,
        commands::auth::auth_set_keyring_enabled,
        // Passkey commands
        commands::auth::auth_passkey_register,
        commands::auth::auth_passkey_authenticate,
        commands::auth::auth_passkey_has_passkey,
        commands::auth::auth_passkey_get_info,
        commands::auth::auth_passkey_delete,
        // Core bindings (pointers-only DHT surface)
        core_cmds::core_claim,
        core_cmds::core_advertise,
        core_cmds::container_put,
        core_cmds::container_get,
        core_cmds::generate_four_word_identity,
        core_cmds::check_dht_connection,
        core_cmds::find_group_storage_disk,
        core_cmds::store_user_identity,
        core_cmds::find_user_current_address,
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
        core_commands::core_add_bootstrap_node,
        core_commands::core_clear_custom_nodes,
        core_commands::core_get_bootstrap_stats,
        // Message management
        core_commands::core_messages_list,
        core_commands::core_messages_send,
        core_commands::core_messages_edit,
        core_commands::core_messages_delete,
        // Entity permissions and encryption
        core_commands::core_entity_get_permissions,
        core_commands::core_entity_get_encryption_status,
        // Entity management
        core_commands::core_entity_update,
        core_commands::core_entity_delete,
        // Network sync
        core_commands::get_sync_status,
        core_commands::subscribe_to_entity,
        core_commands::unsubscribe_from_entity,
        // Groups
        core_groups::core_group_create,
        core_groups::core_group_add_member,
        core_groups::core_group_remove_member,
        // Storage API
        storage_fs::core_storage_list,
        storage_fs::core_storage_read,
        storage_fs::core_storage_write,
        storage_fs::core_storage_mkdir,
        storage_fs::core_storage_fs_delete,
        storage_fs::core_storage_rename,
        storage_fs::core_storage_stats,
        // Encrypted storage
        core_storage::core_storage_initialize,
        // core_storage::core_storage_login,
        // core_storage::core_storage_password_login,
        // core_storage::core_storage_store,
        // core_storage::core_storage_retrieve,
        // core_storage::core_storage_vault_delete,
        // core_storage::core_storage_list_keys,
        // core_storage::core_storage_list_vaults,
        // core_storage::core_storage_get_sessions,
        // core_storage::core_storage_switch_vault,
        // core_storage::core_storage_logout,
        // core_storage::core_storage_export_vault,
        // core_storage::core_storage_import_vault,
        // core_storage::core_storage_store_identity,
        // core_storage::core_storage_get_stats,
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
        // Organization commands - Channels
        commands::org_commands::create_channel,
        commands::org_commands::get_channel,
        commands::org_commands::list_channels,
        commands::org_commands::send_message,
        commands::org_commands::get_messages,
        commands::org_commands::create_thread,
        commands::org_commands::get_thread_replies,
        commands::org_commands::add_channel_member,
        commands::org_commands::remove_channel_member,
        commands::org_commands::get_channel_members,
        // Organization commands - Projects
        commands::org_commands::create_project,
        commands::org_commands::get_project,
        commands::org_commands::list_projects,
        // Organization commands - Issues
        commands::org_commands::create_issue,
        commands::org_commands::get_issue,
        commands::org_commands::list_issues,
        commands::org_commands::list_issues_by_status,
        commands::org_commands::update_issue_status,
        commands::org_commands::assign_issue,
        commands::org_commands::update_issue_priority,
        commands::org_commands::add_issue_comment,
        commands::org_commands::get_issue_comments,
        // Sync commands
        commands::org_commands::get_channel_sync_update,
        commands::org_commands::apply_channel_sync_update,
        commands::org_commands::get_issue_sync_update,
        commands::org_commands::apply_issue_sync_update,
    ]);

    builder
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Failed to run Tauri app: {}", e))?;

    Ok(())
}

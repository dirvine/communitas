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
mod core_cmds;
mod core_commands;
mod core_groups;
mod core_storage;
mod crdt_error;
mod crdt_manager;
mod doc_commands;
mod gossip_commands;
mod member_commands;
pub mod member_manager;
mod message_sync_commands;
mod network;
mod security;
mod services;
mod storage_fs;
mod sync;
mod update_manager;

use commands::{auth::AppState, org_commands::OrgState};
use communitas_core::CoreContext;
use communitas_core::types::DeviceType;
use crdt_manager::CrdtManager;
use member_commands::MemberState;
use member_manager::MemberManager;
use services::{channel_service::ChannelService, issue_service::IssueService};
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;

#[tauri::command]
async fn health() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": "ok",
        "app": env!("CARGO_PKG_VERSION"),
    }))
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
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

    // Initialize member manager
    let member_manager = Arc::new(MemberManager::new(crdt_manager.clone()));
    let member_state = MemberState { member_manager };

    // Initialize encrypted storage app state
    let app_state = AppState::new();

    // Initialize update state
    let update_state = Arc::new(RwLock::new(update_manager::UpdateStatus::default()));

    // Initialize update settings state (will be loaded in setup)
    let settings_state = Arc::new(RwLock::new(update_manager::UpdateSettings::default()));

    let builder = tauri::Builder::default()
        // Auth and encrypted storage state
        .manage(app_state)
        // Organization services state
        .manage(org_state)
        // Member management state
        .manage(member_state)
        // Update manager state
        .manage(update_state.clone())
        // Update settings state
        .manage(settings_state.clone())
        // Shared saorsa-core context (initialized via core_initialize)
        .manage(Arc::new(RwLock::new(Option::<CoreContext>::None)))

        // Sync watcher state
        .manage(Arc::new(RwLock::new(sync::TipWatcherState::default())))
        // Network runtime state
        .manage(Arc::new(RwLock::new(network::NetworkRuntime::default())))
        // Raw SPKI pinning state
        .manage(Arc::new(RwLock::new(
            security::raw_spki::RawSpkiState::default(),
        )))
        // Message sync service state
        .manage(Arc::new(RwLock::new(
            Option::<communitas_core::message_sync::MessageSyncService>::None,
        )))
        // Gossip overlay state
        .manage(Arc::new(RwLock::new(
            Option::<communitas_core::gossip::GossipContext>::None,
        )) as gossip_commands::GossipState);

    let builder = builder
        // Register plugins
        .plugin(tauri_plugin_dialog::init())
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
            commands::auth::auth_remove_recent_identity,
            commands::auth::auth_delete_vault,
            commands::auth::auth_list_old_vaults,
            commands::auth::auth_set_auto_login,
            commands::auth::auth_set_keyring_enabled,
            // Passkey commands
            commands::auth::auth_passkey_register,
            commands::auth::auth_passkey_register_webauthn,
            commands::auth::auth_passkey_authenticate,
            commands::auth::auth_passkey_authenticate_webauthn,
            commands::auth::auth_passkey_has_passkey,
            commands::auth::auth_passkey_get_info,
            commands::auth::auth_passkey_delete,
            // Native Touch ID commands (macOS only)
            #[cfg(target_os = "macos")]
            commands::auth::auth_touchid_register,
            #[cfg(target_os = "macos")]
            commands::auth::auth_touchid_authenticate,
            // OS integration
            commands::auth::get_os_username,
            // Core bindings (gossip overlay surface)
            core_cmds::core_claim,
            core_cmds::core_advertise,
            core_cmds::generate_four_word_identity,
            core_cmds::check_gossip_connection,
            core_cmds::find_group_storage_disk,
            core_cmds::store_user_identity,
            core_cmds::find_user_current_address,
            core_commands::core_recover_state,
            core_commands::core_initialize,
            core_commands::core_get_peer_id,
            core_commands::core_get_user_info,
            core_commands::core_set_display_name,
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
            core_commands::core_entity_mute,
            core_commands::core_entity_block,
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
            sync::sync_fetch_deltas,
            sync::sync_get_status,
            sync::sync_force_full_sync,
            security::raw_spki::sync_set_quic_pinned_spki,
            security::raw_spki::sync_clear_quic_pinned_spki,
            // Message CRDT sync commands
            message_sync_commands::message_sync_initialize,
            message_sync_commands::message_sync_get_all_messages,
            message_sync_commands::message_sync_receive_message,
            message_sync_commands::message_sync_send_message,
            message_sync_commands::message_sync_request_sync,
            message_sync_commands::message_sync_handle_sync_response,
            message_sync_commands::message_sync_get_sync_state,
            message_sync_commands::message_sync_get_messages,
            message_sync_commands::message_sync_needs_sync,
            // Document CRDT commands (Sprint 3.3)
            doc_commands::doc_create,
            doc_commands::doc_insert_text,
            doc_commands::doc_delete_text,
            doc_commands::doc_get_text,
            doc_commands::doc_get_update,
            doc_commands::doc_apply_update,
            doc_commands::doc_list,
            doc_commands::doc_delete,
            // Gossip overlay commands
            gossip_commands::gossip_initialize,
            gossip_commands::gossip_store_message,
            gossip_commands::gossip_get_all_messages,
            gossip_commands::gossip_contains_message,
            gossip_commands::gossip_remove_message,
            gossip_commands::gossip_find_contact,
            gossip_commands::gossip_add_contact,
            gossip_commands::gossip_get_contacts,
            gossip_commands::gossip_remove_contact,
            gossip_commands::gossip_send_direct_message,
            gossip_commands::gossip_subscribe_to_entity,
            gossip_commands::gossip_publish_to_entity,
            gossip_commands::gossip_join_entity,
            gossip_commands::gossip_leave_entity,
            gossip_commands::gossip_start_presence_beacons,
            gossip_commands::gossip_stop_presence_beacons,
            gossip_commands::gossip_is_peer_online,
            gossip_commands::gossip_get_online_peers,
            gossip_commands::gossip_add_favourite_contact,
            gossip_commands::gossip_get_favourite_contacts,
            gossip_commands::gossip_replicate_to_favourites,
            gossip_commands::gossip_recover_from_favourite,
            gossip_commands::gossip_site_publish,
            gossip_commands::gossip_site_fetch,
            gossip_commands::gossip_site_list,
            gossip_commands::gossip_site_providers,
            gossip_commands::gossip_get_own_identity,
            gossip_commands::gossip_get_connection_status,
            gossip_commands::gossip_add_bootstrap_peer,
            gossip_commands::gossip_get_cached_peers,
            gossip_commands::gossip_clear_bootstrap_peers,
            gossip_commands::gossip_get_subscribed_entities,
            gossip_commands::gossip_get_entity_subscribers,
            gossip_commands::gossip_get_entity_messages,
            gossip_commands::gossip_store_entity_message,
            gossip_commands::gossip_get_peer_metadata,
            gossip_commands::gossip_store_peer_metadata,
            gossip_commands::gossip_get_own_metadata,
            gossip_commands::gossip_store_own_metadata,
            health,
            get_app_version,
            // Organization commands - Channels
            commands::org_commands::create_channel,
            commands::org_commands::get_channel,
            commands::org_commands::list_channels,
            commands::org_commands::send_message,
            commands::org_commands::edit_message,
            commands::org_commands::delete_message,
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
            // Phase 3: Efficient channel sync (state vector protocol)
            commands::org_commands::get_channel_state_vector,
            commands::org_commands::get_channel_diff,
            commands::org_commands::apply_channel_diff,
            // Member management commands
            member_commands::member_add,
            member_commands::member_list,
            member_commands::member_remove,
            member_commands::member_update_role,
            member_commands::member_prune_tombstones,
            // Update manager commands
            update_manager::check_for_updates,
            update_manager::install_update,
            update_manager::get_update_status,
            update_manager::get_update_settings,
            update_manager::set_update_settings,
            update_manager::set_auto_update,
            update_manager::set_check_frequency,
            update_manager::set_update_channel,
            
        ]);



    builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            // Load update settings and schedule automatic checks
            let app_handle = app.handle().clone();
            let settings_state_clone = settings_state.clone();
            tauri::async_runtime::spawn(async move {
                let loaded_settings = update_manager::UpdateSettings::load(&app_handle).await;
                *settings_state_clone.write().await = loaded_settings;
                info!("✅ Update settings loaded");
            });

            // Schedule automatic update checks
            update_manager::schedule_update_checks(app.handle().clone(), update_state);

            // Auto-initialize core if environment variables are set (for testing)
            if let (Ok(peer_id), Ok(user_name)) = (
                std::env::var("COMMUNITAS_PEER_ID"),
                std::env::var("COMMUNITAS_USER_NAME"),
            ) {
                info!("🔧 Auto-initializing core with peer_id={}, user_name={}", peer_id, user_name);

                // Get handle to state
                let app_handle = app.handle().clone();

                // Spawn initialization task
                tauri::async_runtime::spawn(async move {
                    // Get storage directory for user data
                    let storage_dir = dirs::data_local_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("communitas")
                        .join(&peer_id); // Per-user directory

                    match CoreContext::initialize(
                        peer_id.clone(),
                        user_name.clone(),
                        "auto-init-device".to_string(),
                        DeviceType::Desktop,
                        storage_dir,
                    )
                    .await
                    {
                        Ok(ctx) => {
                            let core_state = app_handle.state::<Arc<RwLock<Option<CoreContext>>>>();
                            let mut guard = core_state.write().await;
                            *guard = Some(ctx);
                            info!("✅ Core auto-initialized successfully");
                        }
                        Err(e) => {
                            tracing::error!("❌ Failed to auto-initialize core: {}", e);
                        }
                    }
                });
            } else {
                info!("ℹ️  No auto-init environment variables found, core will be initialized manually");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Failed to run Tauri app: {}", e))?;

    Ok(())
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Security: Enforce no-panic policy via clippy flags (-D clippy::unwrap_used, etc.)
// Note: We don't use #![forbid(...)] here because Tauri's macros may generate code
// with #[allow(...)] attributes which would conflict. Instead, we enforce via clippy.

// Allow dead code, unused imports, and unused variables in desktop binary - some helper functions are for future use
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

mod commands;
mod core_cmds;
mod core_commands;
mod core_groups;
mod core_state;
// mod core_storage;  // DELETED - dead code with all TODOs
mod doc_commands;
mod error;
mod gossip_commands;
// mod member_commands;  // DELETED - depends on deleted member_manager.rs
// mod member_manager;  // DELETED - dead code
mod message_sync_commands;
mod network;
mod network_config;
mod security;
mod services; // Contains only monitoring.rs now - all other services deleted
mod storage_fs;
mod sync;
mod update_manager;
mod webrtc_commands;

use commands::auth::AppState;
use communitas_core::CoreContext;
use communitas_core::types::DeviceType;
use core_state::CoreState;
use error::JsError;
// REMOVED: Old service architecture - now using CoreContext EntityService/MessageService
// use commands::org_commands::OrgState;
// use member_commands::MemberState;
// use member_manager::MemberManager;
// use communitas_core::CrdtManager;
// use services::{
//     channel_service::ChannelService,
//     group_service::GroupService,
//     issue_service::IssueService,
//     member_service::MemberService,
//     organization_service::OrganizationService,
//     virtual_disk_service::VirtualDiskService,
// };
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;
use tokio::sync::RwLock;
use tracing::info;

#[tauri::command]
async fn health() -> Result<serde_json::Value, JsError> {
    Ok(serde_json::json!({
        "status": "ok",
        "app": env!("CARGO_PKG_VERSION"),
    }))
}

#[tauri::command]
async fn get_app_version() -> Result<String, JsError> {
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
                .unwrap_or_else(|_| "info,communitas=debug,saorsa_gossip=debug".to_string()),
        )
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("Starting Communitas (saorsa-core integrated)");

    // Initialize encrypted storage app state
    let app_state = AppState::new();

    // Initialize update state
    let update_state = Arc::new(RwLock::new(update_manager::UpdateStatus::default()));

    // Initialize update settings state (will be loaded in setup)
    let settings_state = Arc::new(RwLock::new(update_manager::UpdateSettings::default()));

    // Initialize WebRTC state
    let webrtc_state = webrtc_commands::WebRtcState::new();

    let builder = tauri::Builder::default()
        // Auth and encrypted storage state
        .manage(app_state)
        // REMOVED: .manage(org_state) - now using CoreContext EntityService/MessageService
        // Member management now handled via CoreContext EntityService
        // Update manager state
        .manage(update_state.clone())
        // Update settings state
        .manage(settings_state.clone())
        // WebRTC state
        .manage(webrtc_state)
        // Shared saorsa-core context (initialized via core_initialize)
        .manage(Arc::new(CoreState::new()))
        // Sync watcher state
        .manage(Arc::new(RwLock::new(sync::TipWatcherState::default())))
        // Network runtime state
        .manage(Arc::new(RwLock::new(network::NetworkRuntime::default())))
        // Raw SPKI pinning state
        .manage(Arc::new(RwLock::new(
            security::raw_spki::RawSpkiState::default(),
        )))
        // Message service now handled via CoreContext MessageService
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
            // REMOVED: core_storage module deleted (dead code with all TODOs)
            // core_storage::core_storage_initialize,
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
            // WebRTC commands
            webrtc_commands::webrtc_initiate_call,
            webrtc_commands::webrtc_accept_call,
            webrtc_commands::webrtc_reject_call,
            webrtc_commands::webrtc_end_call,
            webrtc_commands::webrtc_set_video_enabled,
            webrtc_commands::webrtc_set_audio_enabled,
            webrtc_commands::webrtc_start_screen_share,
            webrtc_commands::webrtc_stop_screen_share,
            webrtc_commands::webrtc_get_media_devices,
            webrtc_commands::webrtc_subscribe_events,
            health,
            get_app_version,
            // Update manager commands
            update_manager::check_for_updates,
            update_manager::install_update,
            update_manager::get_update_status,
            update_manager::get_update_settings,
            update_manager::set_update_settings,
            update_manager::set_auto_update,
            update_manager::set_check_frequency,
            update_manager::set_update_channel,
            // Monitoring commands
            services::monitoring::monitoring_get_metrics,
            services::monitoring::monitoring_get_errors,
            services::monitoring::monitoring_get_stats,
            services::monitoring::monitoring_export_prometheus,
            // Network configuration commands
            network_config::network_config_get_bootstrap_nodes,
            network_config::network_config_is_network_enabled,
            network_config::network_config_validate,
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

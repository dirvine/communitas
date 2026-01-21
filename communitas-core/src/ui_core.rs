// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Rust-native UI API surface consumed by Dioxus/Tauri front-ends

use crate::app::CommunitasApp;
use crate::auth_service::AuthService;
use crate::command::{
    Command, ContactResponse, DiskStatsResponse, DiskTypeArg, EntityResponse, Event,
    FileInfoResponse, KanbanBoardResponse, KanbanCardResponse, KanbanColumnResponse,
    MessageResponse, PresenceResponse, Query, QueryResponse, ReactionResponse,
};
use crate::crdt::EntityType;
use crate::encrypted_storage::{EncryptedStorageManager, StorageConfig, VaultInfo};
use crate::keystore::Keystore;
use crate::peer_presence::PresenceRecord;
use crate::recovery::recover_identity;
use base64::Engine;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tracing_subscriber::EnvFilter;

/// Track whether tracing has been initialized (can only be done once)
static TRACING_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the Rust tracing system for Ui debugging.
///
/// This function sets up a tracing subscriber that outputs logs to stdout,
/// which Ui can capture and display in the debug console.
///
/// # Arguments
/// * `level` - Optional log level filter (e.g., "info", "debug", "warn").
///   If not provided, defaults to "info" or respects RUST_LOG env var.
///
/// # Returns
/// * `Ok(true)` - Tracing was successfully initialized
/// * `Ok(false)` - Tracing was already initialized (no-op)
/// * `Err(String)` - Failed to initialize tracing
///
/// # Example
/// Call this from Ui before any other Rust operations:
/// ```dart
/// await initializeTracing(level: "debug");
/// ```
pub fn initialize_tracing(level: Option<String>) -> Result<bool, String> {
    // Only initialize once
    if TRACING_INITIALIZED.swap(true, Ordering::SeqCst) {
        return Ok(false);
    }

    let filter = if let Some(level) = level {
        EnvFilter::try_new(&level).unwrap_or_else(|_| EnvFilter::new("info"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .try_init()
        .map_err(|e| format!("Failed to initialize tracing: {e}"))?;

    tracing::info!("Communitas tracing initialized");
    Ok(true)
}

/// Generate a random four-word identity
pub fn generate_id_words() -> Result<String, String> {
    crate::identity::generate_id_words().map_err(|e| e.to_string())
}

/// Opaque API wrapper exposed to UI shells
#[derive(Clone)]
pub struct CommunitasApi {
    app: Arc<CommunitasApp>,
    auth: Arc<RwLock<AuthService>>,
}

impl CommunitasApi {
    /// Initialize the API with the given identity and storage path
    pub async fn create(
        four_words: String,
        display_name: String,
        device_name: String,
        storage_path: String,
    ) -> Result<Self, String> {
        let storage_root = PathBuf::from(&storage_path);
        std::fs::create_dir_all(&storage_root)
            .map_err(|e| format!("Failed to create storage directory: {e}"))?;

        let vault_dir = storage_root.join("vaults");
        std::fs::create_dir_all(&vault_dir)
            .map_err(|e| format!("Failed to create vault directory: {e}"))?;

        let storage_config = StorageConfig {
            vault_dir,
            ..StorageConfig::default()
        };

        let storage_manager = EncryptedStorageManager::new(storage_config)
            .await
            .map_err(|e| format!("Failed to initialize storage: {e}"))?;
        let auth = AuthService::new(storage_manager);

        let app = CommunitasApp::new(
            four_words,
            display_name,
            device_name,
            storage_root
                .to_str()
                .ok_or_else(|| "Invalid storage path".to_string())?
                .to_string(),
        )
        .await
        .map_err(|e| format!("Failed to initialize app: {e}"))?;

        Ok(Self {
            app: Arc::new(app),
            auth: Arc::new(RwLock::new(auth)),
        })
    }

    // =====================
    // Auth
    // =====================

    pub async fn auth_create_vault(
        &self,
        four_words: String,
        display_name: String,
        password: String,
    ) -> Result<String, String> {
        let mut auth = self.auth.write().await;
        auth.create_vault(&four_words, &password, &display_name)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn auth_delete_vault(
        &self,
        four_words: String,
        password: String,
    ) -> Result<(), String> {
        let mut auth = self.auth.write().await;
        auth.delete_vault(&four_words, &password)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn auth_get_current_session(&self) -> Result<Option<UiSessionInfo>, String> {
        let auth = self.auth.read().await;
        Ok(auth.get_current_session().map(UiSessionInfo::from))
    }

    pub async fn auth_list_vaults(&self) -> Result<Vec<UiVaultInfo>, String> {
        let auth = self.auth.read().await;
        let vaults = auth.list_vaults().await.map_err(|e| e.to_string())?;
        Ok(vaults.into_iter().map(UiVaultInfo::from).collect())
    }

    pub async fn auth_login(
        &self,
        four_words: String,
        password: String,
    ) -> Result<UiSessionInfo, String> {
        let mut auth = self.auth.write().await;
        auth.login(&four_words, &password, None)
            .await
            .map(UiSessionInfo::from)
            .map_err(|e| e.to_string())
    }

    pub async fn auth_export_vault(&self, include_data: bool) -> Result<String, String> {
        let auth = self.auth.read().await;
        let session = auth
            .get_current_session()
            .ok_or_else(|| "No active session".to_string())?;
        let bytes = auth
            .export_vault(&session.session_id, include_data)
            .await
            .map_err(|e| e.to_string())?;
        Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub async fn auth_import_vault(
        &self,
        backup_base64: String,
        password: String,
    ) -> Result<String, String> {
        let backup_bytes = base64::engine::general_purpose::STANDARD
            .decode(backup_base64.as_bytes())
            .map_err(|e| format!("Invalid base64 backup: {e}"))?;

        let mut auth = self.auth.write().await;
        auth.import_vault(&backup_bytes, &password)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn auth_logout(&self) -> Result<(), String> {
        let mut auth = self.auth.write().await;
        auth.logout().await.map_err(|e| e.to_string())
    }

    /// Refresh the current session, extending its expiration
    pub async fn auth_refresh_session(&self) -> Result<UiSessionInfo, String> {
        let mut auth = self.auth.write().await;
        let session_info = auth.refresh_session().await.map_err(|e| e.to_string())?;
        Ok(UiSessionInfo::from(session_info))
    }

    pub async fn auth_vault_exists(&self, four_words: String) -> Result<bool, String> {
        let auth = self.auth.read().await;
        auth.vault_exists(&four_words)
            .await
            .map_err(|e| e.to_string())
    }

    // =====================
    // Multi-Identity Quick Switch
    // =====================

    /// Get list of recent identities for quick switch UI
    pub async fn auth_get_recent_identities(&self) -> Result<Vec<UiRecentIdentity>, String> {
        let auth = self.auth.read().await;
        let recent = auth
            .get_recent_identities()
            .await
            .map_err(|e| e.to_string())?;
        Ok(recent.into_iter().map(UiRecentIdentity::from).collect())
    }

    /// Switch to another identity using passkey/biometric authentication
    ///
    /// This will logout the current session and login to the new identity.
    /// Requires that the target identity has a passkey registered.
    pub async fn auth_switch_identity(&self, four_words: String) -> Result<UiSessionInfo, String> {
        let mut auth = self.auth.write().await;
        let session_info = auth
            .switch_identity(&four_words)
            .await
            .map_err(|e| e.to_string())?;
        Ok(UiSessionInfo::from(session_info))
    }

    /// Attempt auto-login using the most recent identity with passkey
    ///
    /// Returns the session info if successful, None if no auto-login available.
    pub async fn auth_try_auto_login(&self) -> Result<Option<UiSessionInfo>, String> {
        let mut auth = self.auth.write().await;
        let result = auth.try_auto_login().await.map_err(|e| e.to_string())?;
        Ok(result.map(UiSessionInfo::from))
    }

    /// Check if an identity has a passkey registered for biometric auth
    pub async fn auth_has_passkey(&self, four_words: String) -> Result<bool, String> {
        let auth = self.auth.read().await;
        auth.passkey_has_passkey(&four_words)
            .await
            .map_err(|e| e.to_string())
    }

    /// Register a passkey for the current session (enables biometric auth)
    ///
    /// This stores credentials in the platform keyring for secure biometric authentication.
    pub async fn auth_register_passkey(&self) -> Result<(), String> {
        let mut auth = self.auth.write().await;
        let session = auth
            .get_current_session()
            .ok_or_else(|| "No active session".to_string())?;
        auth.passkey_register(&session.four_words, &session.display_name)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Delete passkey for an identity (disables biometric auth)
    pub async fn auth_delete_passkey(&self, four_words: String) -> Result<(), String> {
        let mut auth = self.auth.write().await;
        auth.passkey_delete(&four_words)
            .await
            .map_err(|e| e.to_string())
    }

    /// Remove a recent identity from the list (does not delete the vault)
    pub async fn auth_remove_recent_identity(&self, four_words: String) -> Result<(), String> {
        let mut auth = self.auth.write().await;
        auth.remove_recent_identity(&four_words)
            .await
            .map_err(|e| e.to_string())
    }

    // =====================
    // Profile
    // =====================

    pub async fn get_profile(&self) -> Result<UiUserProfile, String> {
        match execute_query(&self.app, Query::GetProfile).await? {
            QueryResponse::Profile {
                four_words,
                display_name,
                device_name,
                device_type,
            } => Ok(UiUserProfile {
                four_words,
                display_name,
                device_name,
                device_type,
            }),
            _ => Err("Unexpected response for GetProfile".to_string()),
        }
    }

    pub async fn update_display_name(&self, display_name: String) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::UpdateDisplayName { display_name }).await
    }

    // =====================
    // Networking (Gossip)
    // =====================

    pub async fn gossip_start(&self, port: Option<u16>) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::StartNetworking {
                preferred_port: port,
            },
        )
        .await
    }

    pub async fn gossip_stop(&self) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::StopNetworking).await
    }

    pub async fn gossip_connect_to_peer(&self, four_words: String) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::ConnectToPeer {
                peer_four_words: four_words,
            },
        )
        .await
    }

    pub async fn gossip_get_network_info(&self) -> Result<UiNetworkInfo, String> {
        let ctx = self.app.context();
        let ctx = ctx.read().await;
        let is_active = ctx.is_networking_active();
        let bound_port = ctx.listen_address.map(|addr| addr.port());
        let external_address = ctx.external_address.map(|addr| addr.to_string());

        let peer_count = if let Some(gossip) = ctx.gossip.as_ref() {
            let membership = gossip.membership.read().await;
            membership.active_view().len() as i32
        } else {
            0
        };

        // Get bootstrap node count from peer cache
        let bootstrap_count = if let Some(gossip) = ctx.gossip.as_ref() {
            gossip.peer_cache.seed_addresses().await.len() as i32
        } else {
            0
        };

        let bootstrap_connected = peer_count > 0;

        Ok(UiNetworkInfo {
            is_active,
            bound_port,
            peer_count,
            external_address,
            bootstrap_connected,
            bootstrap_count,
        })
    }

    pub async fn gossip_get_connection_words(&self) -> Result<Option<String>, String> {
        match execute_query(&self.app, Query::GetConnectionWords).await? {
            QueryResponse::OptionalString(words) => Ok(words),
            _ => Err("Unexpected response for GetConnectionWords".to_string()),
        }
    }

    // =====================
    // Entities
    // =====================

    pub async fn entity_list(&self) -> Result<Vec<UiEntity>, String> {
        match execute_query(&self.app, Query::ListEntities).await? {
            QueryResponse::EntityList(entities) => {
                Ok(entities.into_iter().map(UiEntity::from).collect())
            }
            _ => Err("Unexpected response for ListEntities".to_string()),
        }
    }

    pub async fn entity_list_by_type(
        &self,
        entity_type: UiEntityType,
    ) -> Result<Vec<UiEntity>, String> {
        match execute_query(
            &self.app,
            Query::ListEntitiesByType {
                entity_type: entity_type.into(),
            },
        )
        .await?
        {
            QueryResponse::EntityList(entities) => {
                Ok(entities.into_iter().map(UiEntity::from).collect())
            }
            _ => Err("Unexpected response for ListEntitiesByType".to_string()),
        }
    }

    pub async fn entity_get(&self, entity_id: String) -> Result<UiEntity, String> {
        match execute_query(&self.app, Query::GetEntity { entity_id }).await? {
            QueryResponse::Entity(entity) => Ok(entity.into()),
            _ => Err("Unexpected response for GetEntity".to_string()),
        }
    }

    pub async fn entity_create(
        &self,
        name: String,
        entity_type: UiEntityType,
        description: Option<String>,
        parent_org_id: Option<String>,
    ) -> Result<Vec<UiEvent>, String> {
        let mut events = execute_command(
            &self.app,
            Command::CreateEntity {
                name: name.clone(),
                entity_type: entity_type.into(),
                description,
                initial_members: Vec::new(),
            },
        )
        .await?;

        if let Some(parent_org_id) = parent_org_id
            && let Some(entity_id) = events.iter().find_map(|event| match event {
                UiEvent::EntityCreated { entity_id } => Some(entity_id.clone()),
                _ => None,
            })
        {
            let mut parent_events = execute_command(
                &self.app,
                Command::SetParentOrganization {
                    entity_id,
                    parent_org_id,
                },
            )
            .await?;
            events.append(&mut parent_events);
        }

        Ok(events)
    }

    pub async fn entity_add_member(
        &self,
        entity_type: UiEntityType,
        entity_id: String,
        member_id: String,
        role: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::AddMember {
                entity_type: entity_type.into(),
                entity_id,
                member_id,
                role,
            },
        )
        .await
    }

    pub async fn entity_remove_member(
        &self,
        entity_type: UiEntityType,
        entity_id: String,
        member_id: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::RemoveMember {
                entity_type: entity_type.into(),
                entity_id,
                member_id,
            },
        )
        .await
    }

    // =====================
    // Messaging
    // =====================

    pub async fn message_send(
        &self,
        entity_id: String,
        entity_type: UiEntityType,
        text: String,
        reply_to_id: Option<String>,
    ) -> Result<Vec<UiEvent>, String> {
        let ctx = self.app.context();
        let ctx = ctx.read().await;
        let author = if ctx.display_name.is_empty() {
            ctx.four_words.clone()
        } else {
            ctx.display_name.clone()
        };

        drop(ctx);

        execute_command(
            &self.app,
            Command::SendMessage {
                entity_id,
                entity_type: entity_type.into(),
                text,
                author,
                reply_to_id,
                attachments: None,
            },
        )
        .await
    }

    pub async fn message_list(&self, entity_id: String) -> Result<Vec<UiMessage>, String> {
        match execute_query(&self.app, Query::GetEntityMessages { entity_id }).await? {
            QueryResponse::Messages(messages) => {
                Ok(messages.into_iter().map(UiMessage::from).collect())
            }
            _ => Err("Unexpected response for GetEntityMessages".to_string()),
        }
    }

    pub async fn message_get(
        &self,
        entity_id: String,
        message_id: String,
    ) -> Result<UiMessage, String> {
        match execute_query(
            &self.app,
            Query::GetMessage {
                entity_id,
                message_id,
            },
        )
        .await?
        {
            QueryResponse::Message(message) => Ok(message.into()),
            _ => Err("Unexpected response for GetMessage".to_string()),
        }
    }

    pub async fn message_list_thread(
        &self,
        entity_id: String,
        parent_message_id: String,
    ) -> Result<Vec<UiMessage>, String> {
        match execute_query(
            &self.app,
            Query::GetThreadMessages {
                entity_id,
                parent_message_id,
            },
        )
        .await?
        {
            QueryResponse::Messages(messages) => {
                Ok(messages.into_iter().map(UiMessage::from).collect())
            }
            _ => Err("Unexpected response for GetThreadMessages".to_string()),
        }
    }

    pub async fn message_list_direct(
        &self,
        other_peer_id: String,
    ) -> Result<Vec<UiMessage>, String> {
        match execute_query(&self.app, Query::GetDirectMessages { other_peer_id }).await? {
            QueryResponse::Messages(messages) => {
                Ok(messages.into_iter().map(UiMessage::from).collect())
            }
            _ => Err("Unexpected response for GetDirectMessages".to_string()),
        }
    }

    pub async fn message_send_direct(
        &self,
        recipients: Vec<String>,
        text: String,
    ) -> Result<Vec<UiEvent>, String> {
        let ctx = self.app.context();
        let ctx = ctx.read().await;
        let author = if ctx.display_name.is_empty() {
            ctx.four_words.clone()
        } else {
            ctx.display_name.clone()
        };
        drop(ctx);

        execute_command(
            &self.app,
            Command::SendDirectMessage {
                recipients,
                text,
                author,
            },
        )
        .await
    }

    pub async fn message_delete(
        &self,
        entity_id: String,
        entity_type: UiEntityType,
        message_id: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::DeleteMessage {
                entity_id,
                entity_type: entity_type.into(),
                message_id,
            },
        )
        .await
    }

    pub async fn message_edit(
        &self,
        entity_id: String,
        entity_type: UiEntityType,
        message_id: String,
        new_text: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::EditMessage {
                entity_id,
                entity_type: entity_type.into(),
                message_id,
                new_text,
            },
        )
        .await
    }

    pub async fn message_add_reaction(
        &self,
        entity_id: String,
        entity_type: UiEntityType,
        message_id: String,
        emoji: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::AddReaction {
                entity_id,
                entity_type: entity_type.into(),
                message_id,
                emoji,
            },
        )
        .await
    }

    pub async fn message_remove_reaction(
        &self,
        entity_id: String,
        entity_type: UiEntityType,
        message_id: String,
        emoji: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::RemoveReaction {
                entity_id,
                entity_type: entity_type.into(),
                message_id,
                emoji,
            },
        )
        .await
    }

    // =====================
    // Contacts
    // =====================

    pub async fn contacts_list(&self) -> Result<Vec<UiContact>, String> {
        match execute_query(&self.app, Query::ListContacts).await? {
            QueryResponse::ContactList(contacts) => {
                Ok(contacts.into_iter().map(UiContact::from).collect())
            }
            _ => Err("Unexpected response for ListContacts".to_string()),
        }
    }

    pub async fn contact_get(&self, contact_id: String) -> Result<UiContact, String> {
        match execute_query(&self.app, Query::GetContact { contact_id }).await? {
            QueryResponse::Contact(contact) => Ok(contact.into()),
            _ => Err("Unexpected response for GetContact".to_string()),
        }
    }

    pub async fn contacts_list_favourites(&self) -> Result<Vec<UiContact>, String> {
        match execute_query(&self.app, Query::ListFavouriteContacts).await? {
            QueryResponse::ContactList(contacts) => {
                Ok(contacts.into_iter().map(UiContact::from).collect())
            }
            _ => Err("Unexpected response for ListFavouriteContacts".to_string()),
        }
    }

    pub async fn contacts_search(&self, query: String) -> Result<Vec<UiContact>, String> {
        match execute_query(&self.app, Query::SearchContacts { query }).await? {
            QueryResponse::ContactList(contacts) => {
                Ok(contacts.into_iter().map(UiContact::from).collect())
            }
            _ => Err("Unexpected response for SearchContacts".to_string()),
        }
    }

    pub async fn contact_create(
        &self,
        display_name: String,
        four_words: Option<String>,
        is_favourite: bool,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::CreateContact {
                display_name,
                four_words,
                is_favourite,
            },
        )
        .await
    }

    pub async fn contact_update(
        &self,
        contact_id: String,
        display_name: Option<String>,
        is_favourite: Option<bool>,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::UpdateContact {
                contact_id,
                display_name,
                is_favourite,
            },
        )
        .await
    }

    pub async fn contact_delete(&self, contact_id: String) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::DeleteContact { contact_id }).await
    }

    pub async fn contact_link(
        &self,
        contact_id: String,
        four_words: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::LinkContact {
                contact_id,
                four_words,
            },
        )
        .await
    }

    pub async fn contact_set_favourite(&self, four_words: String) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::SetFavouriteContact { four_words }).await
    }

    pub async fn contact_remove_favourite(
        &self,
        four_words: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::RemoveFavouriteContact { four_words }).await
    }

    // =====================
    // Presence
    // =====================

    pub async fn presence_announce(&self) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::AnnouncePresence).await
    }

    pub async fn presence_get_our_record(&self) -> Result<Option<UiPresenceRecord>, String> {
        match execute_query(&self.app, Query::GetOurPresenceRecord).await? {
            QueryResponse::OurPresenceRecord(record) => Ok(record.map(UiPresenceRecord::from)),
            _ => Err("Unexpected response for GetOurPresenceRecord".to_string()),
        }
    }

    pub async fn presence_get_status(&self, peer_id: String) -> Result<UiPresenceStatus, String> {
        match execute_query(&self.app, Query::GetPresence { peer_id }).await? {
            QueryResponse::Presence(presence) => Ok(UiPresenceStatus::from(presence)),
            _ => Err("Unexpected response for GetPresence".to_string()),
        }
    }

    pub async fn presence_get_cached_peer(
        &self,
        pubkey_hex: String,
    ) -> Result<Option<UiPresenceRecord>, String> {
        let bytes = hex::decode(pubkey_hex).map_err(|e| format!("Invalid pubkey hex: {e}"))?;
        match execute_query(&self.app, Query::GetCachedPeerPresence { pubkey: bytes }).await? {
            QueryResponse::CachedPeerPresence(record) => Ok(record.map(UiPresenceRecord::from)),
            _ => Err("Unexpected response for GetCachedPeerPresence".to_string()),
        }
    }

    pub async fn presence_query_peer(
        &self,
        pubkey_hex: String,
    ) -> Result<Option<UiPresenceRecord>, String> {
        let bytes = hex::decode(&pubkey_hex).map_err(|e| format!("Invalid pubkey hex: {e}"))?;
        let _ = execute_command(
            &self.app,
            Command::QueryPeerPresence {
                target_pubkey: bytes.clone(),
            },
        )
        .await?;

        match execute_query(&self.app, Query::GetCachedPeerPresence { pubkey: bytes }).await? {
            QueryResponse::CachedPeerPresence(record) => Ok(record.map(UiPresenceRecord::from)),
            _ => Err("Unexpected response for GetCachedPeerPresence".to_string()),
        }
    }

    pub async fn presence_list_online_peers(&self) -> Result<Vec<String>, String> {
        match execute_query(&self.app, Query::ListOnlinePeers).await? {
            QueryResponse::PeerList(peers) => Ok(peers),
            _ => Err("Unexpected response for ListOnlinePeers".to_string()),
        }
    }

    // =====================
    // Invites
    // =====================

    pub async fn invite_create(
        &self,
        recipient_id: String,
        entity_type: UiEntityType,
        entity_id: String,
        role: String,
        message: Option<String>,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::CreateInvite {
                recipient_id,
                entity_type: entity_type.into(),
                entity_id,
                role,
                message,
                expires_in_hours: None,
            },
        )
        .await
    }

    pub async fn invite_accept(&self, invite_id: String) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::AcceptInvite { invite_id }).await
    }

    pub async fn invite_reject(&self, invite_id: String) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::RejectInvite { invite_id }).await
    }

    pub async fn invite_revoke(&self, invite_id: String) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::RevokeInvite { invite_id }).await
    }

    // =====================
    // Virtual Disk
    // =====================

    pub async fn disk_list_files(
        &self,
        entity_id: String,
        disk_type: UiDiskType,
        path: String,
    ) -> Result<Vec<UiFileInfo>, String> {
        match execute_query(
            &self.app,
            Query::ListFiles {
                entity_id,
                disk_type: disk_type.into(),
                path,
            },
        )
        .await?
        {
            QueryResponse::FileList(files) => Ok(files.into_iter().map(UiFileInfo::from).collect()),
            _ => Err("Unexpected response for ListFiles".to_string()),
        }
    }

    pub async fn disk_read_file(
        &self,
        entity_id: String,
        disk_type: UiDiskType,
        path: String,
    ) -> Result<Vec<u8>, String> {
        match execute_query(
            &self.app,
            Query::ReadFile {
                entity_id,
                disk_type: disk_type.into(),
                path,
            },
        )
        .await?
        {
            QueryResponse::FileContents(bytes) => Ok(bytes),
            _ => Err("Unexpected response for ReadFile".to_string()),
        }
    }

    pub async fn disk_write_file(
        &self,
        entity_id: String,
        disk_type: UiDiskType,
        path: String,
        data: Vec<u8>,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::WriteFile {
                entity_id,
                disk_type: disk_type.into(),
                path,
                data,
            },
        )
        .await
    }

    pub async fn disk_delete_file(
        &self,
        entity_id: String,
        disk_type: UiDiskType,
        path: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::DeleteFile {
                entity_id,
                disk_type: disk_type.into(),
                path,
            },
        )
        .await
    }

    pub async fn disk_create_directory(
        &self,
        entity_id: String,
        disk_type: UiDiskType,
        path: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::CreateDirectory {
                entity_id,
                disk_type: disk_type.into(),
                path,
            },
        )
        .await
    }

    pub async fn disk_get_stats(
        &self,
        entity_id: String,
        disk_type: UiDiskType,
    ) -> Result<UiDiskStats, String> {
        match execute_query(
            &self.app,
            Query::GetDiskStats {
                entity_id,
                disk_type: disk_type.into(),
            },
        )
        .await?
        {
            QueryResponse::DiskStats(stats) => Ok(stats.into()),
            _ => Err("Unexpected response for GetDiskStats".to_string()),
        }
    }

    // =====================
    // Kanban
    // =====================

    pub async fn kanban_list_boards(
        &self,
        entity_id: String,
    ) -> Result<Vec<UiKanbanBoard>, String> {
        match execute_query(&self.app, Query::ListKanbanBoards { entity_id }).await? {
            QueryResponse::KanbanBoardList(boards) => {
                Ok(boards.into_iter().map(UiKanbanBoard::from).collect())
            }
            _ => Err("Unexpected response for ListKanbanBoards".to_string()),
        }
    }

    pub async fn kanban_get_board(&self, board_id: String) -> Result<UiKanbanBoard, String> {
        match execute_query(&self.app, Query::GetKanbanBoard { board_id }).await? {
            QueryResponse::KanbanBoard(board) => Ok(board.into()),
            _ => Err("Unexpected response for GetKanbanBoard".to_string()),
        }
    }

    pub async fn kanban_create_board(
        &self,
        entity_id: String,
        board_name: String,
        description: Option<String>,
    ) -> Result<UiKanbanBoard, String> {
        let events = execute_command_raw(
            &self.app,
            Command::CreateKanbanBoard {
                entity_id: entity_id.clone(),
                board_name,
                description,
            },
        )
        .await?;

        let board_id = events.iter().find_map(|event| match event {
            Event::KanbanBoardCreated { board_id, .. } => Some(board_id.clone()),
            _ => None,
        });

        let board_id = board_id.ok_or_else(|| "Board creation did not return an ID".to_string())?;
        self.kanban_get_board(board_id).await
    }

    pub async fn kanban_list_columns(
        &self,
        board_id: String,
    ) -> Result<Vec<UiKanbanColumn>, String> {
        match execute_query(&self.app, Query::ListKanbanColumns { board_id }).await? {
            QueryResponse::KanbanColumns(columns) => {
                Ok(columns.into_iter().map(UiKanbanColumn::from).collect())
            }
            _ => Err("Unexpected response for ListKanbanColumns".to_string()),
        }
    }

    pub async fn kanban_create_column(
        &self,
        board_id: String,
        column_name: String,
        position: Option<u32>,
    ) -> Result<UiKanbanColumn, String> {
        let events = execute_command_raw(
            &self.app,
            Command::CreateKanbanColumn {
                board_id: board_id.clone(),
                column_name,
                position,
            },
        )
        .await?;

        let column_id = events.iter().find_map(|event| match event {
            Event::KanbanColumnCreated { column_id, .. } => Some(column_id.clone()),
            _ => None,
        });

        let column_id =
            column_id.ok_or_else(|| "Column creation did not return an ID".to_string())?;

        let columns = self.kanban_list_columns(board_id).await?;
        columns
            .into_iter()
            .find(|col| col.id == column_id)
            .ok_or_else(|| "Created column not found".to_string())
    }

    pub async fn kanban_list_cards(
        &self,
        board_id: String,
        column_id: Option<String>,
        state: Option<String>,
        assignee_id: Option<String>,
        tag_id: Option<String>,
    ) -> Result<Vec<UiKanbanCard>, String> {
        match execute_query(
            &self.app,
            Query::ListKanbanCards {
                board_id,
                column_id,
                state,
                assignee_id,
                tag_id,
            },
        )
        .await?
        {
            QueryResponse::KanbanCards(cards) => {
                Ok(cards.into_iter().map(UiKanbanCard::from).collect())
            }
            _ => Err("Unexpected response for ListKanbanCards".to_string()),
        }
    }

    pub async fn kanban_create_card(
        &self,
        board_id: String,
        column_id: String,
        title: String,
        description: Option<String>,
        assignee: Option<String>,
    ) -> Result<UiKanbanCard, String> {
        let events = execute_command_raw(
            &self.app,
            Command::CreateKanbanCard {
                board_id: board_id.clone(),
                column_id,
                title,
                description,
                assignee,
            },
        )
        .await?;

        let card_id = events.iter().find_map(|event| match event {
            Event::KanbanCardCreated { card_id, .. } => Some(card_id.clone()),
            _ => None,
        });

        let card_id = card_id.ok_or_else(|| "Card creation did not return an ID".to_string())?;

        match execute_query(&self.app, Query::GetKanbanCard { board_id, card_id }).await? {
            QueryResponse::KanbanCard(card) => Ok(card.into()),
            _ => Err("Unexpected response for GetKanbanCard".to_string()),
        }
    }

    pub async fn kanban_move_card(
        &self,
        board_id: String,
        card_id: String,
        target_column_id: String,
        position: Option<u32>,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::MoveKanbanCard {
                board_id,
                card_id,
                target_column_id,
                position,
            },
        )
        .await
    }

    pub async fn kanban_update_card(
        &self,
        board_id: String,
        card_id: String,
        title: Option<String>,
        description: Option<String>,
        assignee: Option<String>,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(
            &self.app,
            Command::UpdateKanbanCard {
                board_id,
                card_id,
                title,
                description,
                assignee,
            },
        )
        .await
    }

    pub async fn kanban_delete_card(
        &self,
        board_id: String,
        card_id: String,
    ) -> Result<Vec<UiEvent>, String> {
        execute_command(&self.app, Command::DeleteKanbanCard { board_id, card_id }).await
    }
}

// =====================
// Recovery helpers (top-level functions)
// =====================

pub fn validate_recovery_mnemonic(mnemonic: String) -> Result<bool, String> {
    match crate::recovery::validate_mnemonic(&mnemonic, crate::recovery::Language::English) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

pub fn preview_identity_from_mnemonic(
    mnemonic: String,
    passphrase: Option<String>,
) -> Result<UiRecoveredIdentity, String> {
    let keys = recover_identity(
        &mnemonic,
        crate::recovery::Language::English,
        passphrase.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    Ok(UiRecoveredIdentity::from(keys))
}

pub fn recover_identity_from_mnemonic(
    mnemonic: String,
    passphrase: Option<String>,
) -> Result<UiRecoveredIdentity, String> {
    let keys = recover_identity(
        &mnemonic,
        crate::recovery::Language::English,
        passphrase.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    let keystore = Keystore::new();
    let id_hex = blake3::hash(keys.four_words.as_bytes())
        .to_hex()
        .to_string();
    let pubkey = keys.verifying_key_bytes().to_vec();
    let privkey = keys.signing_key_bytes().to_vec();

    keystore
        .save_mldsa_keys(&id_hex, &pubkey, &privkey)
        .map_err(|e| format!("Failed to persist keys: {e}"))?;

    if let Some(words) = split_four_words(&keys.four_words) {
        let _ = keystore.save_words(&id_hex, &words);
    }

    Ok(UiRecoveredIdentity::from(keys))
}

// =====================
// Ui-facing models
// =====================

/// Entity information
#[derive(Debug, Clone)]
pub struct UiEntity {
    pub id: String,
    pub name: String,
    pub entity_type: UiEntityType,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub member_count: u64,
    pub parent_org_id: Option<String>,
    pub network_four_words: Option<String>,
    pub is_local_only: bool,
}

impl From<EntityResponse> for UiEntity {
    fn from(value: EntityResponse) -> Self {
        Self {
            id: value.id,
            name: value.name,
            entity_type: value.entity_type.into(),
            description: value.description,
            created_by: value.created_by,
            created_at: value.created_at,
            member_count: value.member_count as u64,
            parent_org_id: value.parent_org_id,
            network_four_words: value.network_four_words,
            is_local_only: value.is_local_only,
        }
    }
}

/// Entity type enumeration
#[derive(Debug, Clone, Copy)]
pub enum UiEntityType {
    Group,
    Channel,
    Project,
    Organisation,
    Person,
}

/// Disk type enumeration
#[derive(Debug, Clone, Copy)]
pub enum UiDiskType {
    Private,
    Public,
    Shared,
}

impl From<UiDiskType> for DiskTypeArg {
    fn from(value: UiDiskType) -> Self {
        match value {
            UiDiskType::Private => DiskTypeArg::Private,
            UiDiskType::Public => DiskTypeArg::Public,
            UiDiskType::Shared => DiskTypeArg::Shared,
        }
    }
}

impl From<DiskTypeArg> for UiDiskType {
    fn from(value: DiskTypeArg) -> Self {
        match value {
            DiskTypeArg::Private => UiDiskType::Private,
            DiskTypeArg::Public => UiDiskType::Public,
            DiskTypeArg::Shared => UiDiskType::Shared,
        }
    }
}

impl From<UiEntityType> for EntityType {
    fn from(value: UiEntityType) -> Self {
        match value {
            UiEntityType::Group => EntityType::Group,
            UiEntityType::Channel => EntityType::Channel,
            UiEntityType::Project => EntityType::Project,
            UiEntityType::Organisation => EntityType::Organisation,
            UiEntityType::Person => EntityType::Person,
        }
    }
}

impl From<EntityType> for UiEntityType {
    fn from(value: EntityType) -> Self {
        match value {
            EntityType::Group => UiEntityType::Group,
            EntityType::Channel => UiEntityType::Channel,
            EntityType::Project => UiEntityType::Project,
            EntityType::Organisation => UiEntityType::Organisation,
            EntityType::Person => UiEntityType::Person,
        }
    }
}

/// Event type for Ui callbacks
#[derive(Debug, Clone)]
pub enum UiEvent {
    NetworkingStarted {
        address: String,
    },
    NetworkingStopped,
    PeerConnected {
        peer_id: String,
    },
    PeerDisconnected {
        peer_id: String,
    },
    EntityCreated {
        entity_id: String,
    },
    EntityUpdated {
        entity_id: String,
    },
    MessageSent {
        message_id: String,
        entity_id: String,
    },
    MessageReceived {
        message_id: String,
        entity_id: String,
    },
    DirectMessageSent {
        message_ids: Vec<String>,
        recipients: Vec<String>,
    },
    MessageDeleted {
        message_id: String,
        entity_id: String,
    },
    MessageEdited {
        message_id: String,
        entity_id: String,
        new_text: String,
        edited_at: u64,
    },
    ReactionAdded {
        message_id: String,
        entity_id: String,
        emoji: String,
        reactor_id: String,
    },
    ReactionRemoved {
        message_id: String,
        entity_id: String,
        emoji: String,
        reactor_id: String,
    },
    InviteCreated {
        invite_id: String,
    },
    InviteAccepted {
        invite_id: String,
    },
    InviteRejected {
        invite_id: String,
    },
    InviteRevoked {
        invite_id: String,
    },
    FileWritten {
        entity_id: String,
        path: String,
    },
    FileDeleted {
        entity_id: String,
        path: String,
    },
    Error {
        code: String,
        message: String,
    },
}

/// Network status information
#[derive(Debug, Clone)]
pub struct UiNetworkInfo {
    pub is_active: bool,
    pub bound_port: Option<u16>,
    pub peer_count: i32,
    pub external_address: Option<String>,
    pub bootstrap_connected: bool,
    /// Number of bootstrap nodes in the peer cache
    pub bootstrap_count: i32,
}

/// Session information
#[derive(Debug, Clone)]
pub struct UiSessionInfo {
    pub session_id: String,
    pub four_words: String,
    pub display_name: String,
    /// Hex-encoded ML-DSA-87 public key (the user's cryptographic identity)
    pub pubkey_hex: String,
    /// Session expiration timestamp (Unix seconds)
    pub expires_at: u64,
}

impl From<crate::SessionInfo> for UiSessionInfo {
    fn from(value: crate::SessionInfo) -> Self {
        Self {
            session_id: value.session_id,
            four_words: value.four_words,
            display_name: value.display_name,
            pubkey_hex: value.pubkey_hex,
            expires_at: value.expires_at,
        }
    }
}

/// User profile information
#[derive(Debug, Clone)]
pub struct UiUserProfile {
    pub four_words: String,
    pub display_name: String,
    pub device_name: String,
    pub device_type: String,
}

/// Vault information
#[derive(Debug, Clone)]
pub struct UiVaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

/// Recent identity information for quick switch
#[derive(Debug, Clone)]
pub struct UiRecentIdentity {
    pub four_words: String,
    pub display_name: String,
    pub last_used: u64,
    pub has_passkey: bool,
}

impl From<crate::encrypted_storage::RecentIdentity> for UiRecentIdentity {
    fn from(value: crate::encrypted_storage::RecentIdentity) -> Self {
        Self {
            four_words: value.four_words,
            display_name: value.display_name,
            last_used: value.last_used,
            has_passkey: value.has_passkey,
        }
    }
}

/// Identity recovery preview/result
#[derive(Debug, Clone)]
pub struct UiRecoveredIdentity {
    pub four_words: String,
    pub pubkey_hex: String,
}

impl From<crate::recovery::IdentityKeys> for UiRecoveredIdentity {
    fn from(value: crate::recovery::IdentityKeys) -> Self {
        let pubkey_hex = hex::encode(value.verifying_key_bytes());
        Self {
            four_words: value.four_words.clone(),
            pubkey_hex,
        }
    }
}

impl From<VaultInfo> for UiVaultInfo {
    fn from(value: VaultInfo) -> Self {
        Self {
            four_words: value.four_words,
            display_name: value.display_name,
            created_at: value.created_at,
            last_accessed: value.last_accessed,
            size_bytes: value.size_bytes,
        }
    }
}

/// Message response data
#[derive(Debug, Clone)]
pub struct UiMessage {
    pub id: String,
    pub entity_id: String,
    pub author: String,
    pub text: String,
    pub timestamp: i64,
    pub reply_to_id: Option<String>,
    pub reactions: Vec<UiReaction>,
    pub edited_at: Option<u64>,
}

impl From<MessageResponse> for UiMessage {
    fn from(value: MessageResponse) -> Self {
        Self {
            id: value.id,
            entity_id: value.entity_id,
            author: value.author,
            text: value.text,
            timestamp: value.timestamp,
            reply_to_id: value.reply_to_id,
            reactions: value.reactions.into_iter().map(UiReaction::from).collect(),
            edited_at: value.edited_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiReaction {
    pub emoji: String,
    pub count: u32,
    pub user_reacted: bool,
    pub peer_ids: Vec<String>,
}

impl From<ReactionResponse> for UiReaction {
    fn from(value: ReactionResponse) -> Self {
        Self {
            emoji: value.emoji,
            count: value.count,
            user_reacted: value.user_reacted,
            peer_ids: value.peer_ids,
        }
    }
}

/// Contact response data
#[derive(Debug, Clone)]
pub struct UiContact {
    pub id: String,
    pub display_name: String,
    pub four_words: Option<String>,
    pub is_favourite: bool,
    pub is_online: bool,
    pub created_at: i64,
    pub last_seen: Option<i64>,
}

impl From<ContactResponse> for UiContact {
    fn from(value: ContactResponse) -> Self {
        Self {
            id: value.id,
            display_name: value.display_name,
            four_words: value.four_words,
            is_favourite: value.is_favourite,
            is_online: value.is_online,
            created_at: value.created_at,
            last_seen: value.last_seen,
        }
    }
}

/// File info response data
#[derive(Debug, Clone)]
pub struct UiFileInfo {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub modified_at: i64,
}

impl From<FileInfoResponse> for UiFileInfo {
    fn from(value: FileInfoResponse) -> Self {
        Self {
            path: value.path,
            name: value.name,
            is_directory: value.is_directory,
            size_bytes: value.size_bytes,
            modified_at: value.modified_at,
        }
    }
}

/// Disk stats response data
#[derive(Debug, Clone)]
pub struct UiDiskStats {
    pub entity_id: String,
    pub disk_type: UiDiskType,
    pub used_bytes: u64,
    pub file_count: u32,
    pub dir_count: u32,
}

impl From<DiskStatsResponse> for UiDiskStats {
    fn from(value: DiskStatsResponse) -> Self {
        Self {
            entity_id: value.entity_id,
            disk_type: value.disk_type.into(),
            used_bytes: value.used_bytes,
            file_count: value.file_count,
            dir_count: value.dir_count,
        }
    }
}

/// Kanban board response data
#[derive(Debug, Clone)]
pub struct UiKanbanBoard {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub description: Option<String>,
    pub column_count: u32,
}

impl From<KanbanBoardResponse> for UiKanbanBoard {
    fn from(value: KanbanBoardResponse) -> Self {
        Self {
            id: value.id,
            entity_id: value.entity_id,
            name: value.name,
            description: value.description,
            column_count: value.column_count as u32,
        }
    }
}

/// Kanban column response data
#[derive(Debug, Clone)]
pub struct UiKanbanColumn {
    pub id: String,
    pub board_id: String,
    pub name: String,
    pub position: u32,
    pub color: Option<String>,
    pub wip_limit: Option<u32>,
}

impl From<KanbanColumnResponse> for UiKanbanColumn {
    fn from(value: KanbanColumnResponse) -> Self {
        Self {
            id: value.id,
            board_id: value.board_id,
            name: value.name,
            position: value.position,
            color: value.color,
            wip_limit: value.wip_limit,
        }
    }
}

/// Kanban card response data
#[derive(Debug, Clone)]
pub struct UiKanbanCard {
    pub id: String,
    pub column_id: String,
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub position: u32,
}

impl From<KanbanCardResponse> for UiKanbanCard {
    fn from(value: KanbanCardResponse) -> Self {
        Self {
            id: value.id,
            column_id: value.column_id,
            title: value.title,
            description: value.description,
            assignee: value.assignee,
            position: value.position,
        }
    }
}

/// Presence record exposed to Ui
#[derive(Debug, Clone)]
pub struct UiPresenceRecord {
    pub pubkey_hex: String,
    pub connection_words: String,
    pub timestamp: u64,
    pub is_verified: bool,
}

impl From<PresenceRecord> for UiPresenceRecord {
    fn from(value: PresenceRecord) -> Self {
        let verified = value.verify().unwrap_or(false);
        Self {
            pubkey_hex: hex::encode(&value.pubkey),
            connection_words: value.connection_words,
            timestamp: value.timestamp,
            is_verified: verified,
        }
    }
}

/// Presence status for a peer (online/offline/unknown)
#[derive(Debug, Clone)]
pub struct UiPresenceStatus {
    pub peer_id: String,
    pub status: String,
    pub last_seen: i64,
}

impl From<PresenceResponse> for UiPresenceStatus {
    fn from(value: PresenceResponse) -> Self {
        Self {
            peer_id: value.peer_id,
            status: value.status,
            last_seen: value.last_seen,
        }
    }
}

// =====================
// Helper functions
// =====================

async fn execute_command(
    app: &Arc<CommunitasApp>,
    command: Command,
) -> Result<Vec<UiEvent>, String> {
    let events = app
        .execute(command)
        .await
        .map_err(|e| format!("{}: {}", e.code, e.message))?;

    Ok(events.into_iter().filter_map(map_event).collect())
}

async fn execute_command_raw(
    app: &Arc<CommunitasApp>,
    command: Command,
) -> Result<Vec<Event>, String> {
    app.execute(command)
        .await
        .map_err(|e| format!("{}: {}", e.code, e.message))
}

async fn execute_query(app: &Arc<CommunitasApp>, query: Query) -> Result<QueryResponse, String> {
    app.query(query)
        .await
        .map_err(|e| format!("{}: {}", e.code, e.message))
}

fn map_event(event: Event) -> Option<UiEvent> {
    match event {
        Event::NetworkingStarted {
            listen_address,
            connection_identity,
        } => Some(UiEvent::NetworkingStarted {
            address: if !connection_identity.is_empty() {
                connection_identity
            } else {
                listen_address
            },
        }),
        Event::NetworkingStopped => Some(UiEvent::NetworkingStopped),
        Event::PeerConnected { peer_four_words } => Some(UiEvent::PeerConnected {
            peer_id: peer_four_words,
        }),
        Event::ConnectionFailed { reason, .. } => Some(UiEvent::Error {
            code: "CONNECTION_FAILED".to_string(),
            message: reason,
        }),
        Event::EntityCreated { entity_id, .. } => Some(UiEvent::EntityCreated { entity_id }),
        Event::EntityUpdated { entity_id, .. } => Some(UiEvent::EntityUpdated { entity_id }),
        Event::MessageSent {
            message_id,
            entity_id,
            ..
        } => Some(UiEvent::MessageSent {
            message_id,
            entity_id,
        }),
        Event::MessageReceived {
            message_id,
            entity_id,
            ..
        } => Some(UiEvent::MessageReceived {
            message_id,
            entity_id,
        }),
        Event::DirectMessageSent {
            message_ids,
            recipients,
        } => Some(UiEvent::DirectMessageSent {
            message_ids,
            recipients,
        }),
        Event::MessageDeleted {
            message_id,
            entity_id,
            ..
        } => Some(UiEvent::MessageDeleted {
            message_id,
            entity_id,
        }),
        Event::MessageEdited {
            message_id,
            entity_id,
            new_text,
            edited_at,
            ..
        } => Some(UiEvent::MessageEdited {
            message_id,
            entity_id,
            new_text,
            edited_at,
        }),
        Event::ReactionAdded {
            message_id,
            entity_id,
            emoji,
            reactor_id,
            ..
        } => Some(UiEvent::ReactionAdded {
            message_id,
            entity_id,
            emoji,
            reactor_id,
        }),
        Event::ReactionRemoved {
            message_id,
            entity_id,
            emoji,
            reactor_id,
            ..
        } => Some(UiEvent::ReactionRemoved {
            message_id,
            entity_id,
            emoji,
            reactor_id,
        }),
        Event::InviteCreated { invite_id, .. } => Some(UiEvent::InviteCreated { invite_id }),
        Event::InviteAccepted { invite_id, .. } => Some(UiEvent::InviteAccepted { invite_id }),
        Event::InviteRejected { invite_id } => Some(UiEvent::InviteRejected { invite_id }),
        Event::InviteRevoked { invite_id } => Some(UiEvent::InviteRevoked { invite_id }),
        Event::FileWritten {
            entity_id, path, ..
        } => Some(UiEvent::FileWritten { entity_id, path }),
        Event::FileDeleted {
            entity_id, path, ..
        } => Some(UiEvent::FileDeleted { entity_id, path }),
        _ => None,
    }
}

fn split_four_words(four_words: &str) -> Option<[String; 4]> {
    let parts: Vec<String> = four_words.split('-').map(|s| s.to_string()).collect();
    if parts.len() == 4 {
        Some([
            parts[0].clone(),
            parts[1].clone(),
            parts[2].clone(),
            parts[3].clone(),
        ])
    } else {
        None
    }
}

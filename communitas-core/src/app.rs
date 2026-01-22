// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! CommunitasApp - The headless core application with execute/query/subscribe API
//!
//! This module provides the main application entry point that all adapters
//! (GUI, MCP Server, CLI) use to interact with Communitas functionality.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
//! │   Iced GUI  │  │ Dioxus GUI  │  │ MCP Server  │  │     CLI     │
//! │  (Adapter)  │  │  (Adapter)  │  │  (Adapter)  │  │  (Adapter)  │
//! └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
//!        │                │                │                │
//!        └────────────────┼────────────────┼────────────────┘
//!                         │                │
//!                         ▼                ▼
//!               ┌─────────────────────────────────────┐
//!               │        CommunitasApp (Core)         │
//!               │  execute(cmd) / query(q) / sub(s)   │
//!               └─────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use communitas_core::{CommunitasApp, Command, Query};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize the app
//!     let app = CommunitasApp::new(
//!         "ocean-forest-moon-star".to_string(),
//!         "Alice".to_string(),
//!         "MacBook".to_string(),
//!         "/path/to/storage".to_string(),
//!     ).await?;
//!
//!     // Execute a command
//!     let events = app.execute(Command::StartNetworking { preferred_port: None }).await?;
//!
//!     // Run a query
//!     let response = app.query(Query::GetProfile).await?;
//! }
//! ```

use crate::command::{
    CallResponse, CallStatusResponse, CanvasSnapshotResponse, ChunkReadResponse,
    ChunkedWriteProgressResponse, ContactResponse, DiskInfoResponse, DiskStatsResponse,
    EntityResponse, FileInfoResponse, FileMetadataResponse, FilePreviewResponse, InviteResponse,
    MemberResponse, MessageResponse, PresenceResponse, ReactionResponse, ResumableTransferResponse,
    ResumeCapabilityResponse, ResumeVerificationResponse, SyncStateResponse, WebsiteResponse,
};
use crate::command::{
    Command, CommandError, CommandResult, DiskTypeArg, Event, Query, QueryError, QueryResponse,
    QueryResult, Subscription,
};
use crate::core_context::CoreContext;
use crate::crdt::EntityType;
use crate::disk_service::DiskType;
use crate::identity::conn_words;
use crate::legacy_crdt::{Attachment, AttachmentType};
use crate::peer_presence::PresenceCache;
use crate::types::DeviceType;
use saorsa_gossip_types::PeerId;
use saorsa_pqc::dsa_traits::SerDes;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn};
use uuid::Uuid;
use yrs::{Map, ReadTxn, Transact};

fn attachments_from_strings(raw: Option<Vec<String>>) -> Option<Vec<Attachment>> {
    let attachments: Vec<Attachment> = raw
        .unwrap_or_default()
        .into_iter()
        .filter(|item| !item.trim().is_empty())
        .map(|item| {
            let name = Path::new(&item)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("attachment")
                .to_string();
            Attachment {
                attachment_type: AttachmentType::File,
                url: item,
                name,
                size: 0,
            }
        })
        .collect();

    if attachments.is_empty() {
        None
    } else {
        Some(attachments)
    }
}

async fn presence_by_four_words(
    presence: &saorsa_gossip_presence::PresenceManager,
) -> HashMap<String, i64> {
    let mut map = HashMap::new();
    let groups = presence.get_groups().await;

    for topic_id in groups {
        let records = presence.get_group_presence(topic_id).await;
        for record in records.values() {
            if record.is_expired() {
                continue;
            }
            if let Some(four_words) = &record.four_words {
                let since = record.since as i64;
                map.entry(four_words.clone())
                    .and_modify(|existing| {
                        if since > *existing {
                            *existing = since;
                        }
                    })
                    .or_insert(since);
            }
        }
    }

    map
}

/// The main Communitas application
///
/// This struct provides the headless core API that all adapters use.
/// It wraps CoreContext and provides a clean Command/Query/Subscribe interface.
pub struct CommunitasApp {
    /// The underlying core context (protected by RwLock for concurrent access)
    context: Arc<RwLock<CoreContext>>,

    /// Event broadcaster for subscriptions
    event_sender: broadcast::Sender<Event>,

    /// Active subscriptions (keyed by subscriber ID)
    subscriptions: Arc<RwLock<HashMap<String, Vec<Subscription>>>>,

    /// Presence cache for peer discovery (ADR-014)
    presence_cache: Arc<RwLock<PresenceCache>>,
}

impl CommunitasApp {
    /// Create a new CommunitasApp instance
    ///
    /// # Arguments
    /// * `four_words` - Four-word identity (e.g., "ocean-forest-moon-star")
    /// * `display_name` - Human-readable display name
    /// * `device_name` - Device identifier
    /// * `storage_dir` - Path to storage directory
    ///
    /// # Returns
    /// New CommunitasApp instance or error
    pub async fn new(
        four_words: String,
        display_name: String,
        device_name: String,
        storage_dir: String,
    ) -> Result<Self, String> {
        let storage_path = PathBuf::from(&storage_dir);

        let context = CoreContext::initialize(
            four_words,
            display_name,
            device_name,
            DeviceType::Desktop,
            storage_path,
        )
        .await?;

        // Create event broadcaster with reasonable capacity
        let (event_sender, _) = broadcast::channel(1024);

        Ok(Self {
            context: Arc::new(RwLock::new(context)),
            event_sender,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            presence_cache: Arc::new(RwLock::new(PresenceCache::new())),
        })
    }

    async fn persist_contacts(
        &self,
        command_type: &str,
        storage_dir: &Path,
        contact_store: &crate::gossip::contact_storage::ContactStore,
    ) -> Result<(), CommandError> {
        let records = contact_store.export().await;
        CoreContext::persist_contact_records(storage_dir, &records)
            .await
            .map_err(|e| CommandError {
                command_type: command_type.to_string(),
                message: e,
                code: "CONTACT_PERSIST_FAILED".to_string(),
            })
    }

    /// Search messages by text content.
    ///
    /// Performs case-insensitive substring matching across message text.
    /// If thread_id is provided, searches only that thread.
    /// If thread_id is None, searches all entity messages and DMs.
    /// Results are sorted by match count (desc) then timestamp (desc).
    async fn search_messages(
        ctx: &CoreContext,
        query: &str,
        thread_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<crate::command::SearchResult>, String> {
        use crate::command::SearchResult;

        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchResult> = Vec::new();

        // Helper to search within a list of messages
        let search_in_messages = |messages: Vec<crate::crdt::CRDTMessage>,
                                  thread_id: &str,
                                  thread_name: &str,
                                  query_lower: &str,
                                  query: &str|
         -> Vec<SearchResult> {
            messages
                .into_iter()
                .filter_map(|msg| {
                    let text_lower = msg.content.text.to_lowercase();
                    let match_count = text_lower.matches(query_lower).count();
                    if match_count > 0 {
                        // Create excerpt around first match
                        let excerpt = Self::create_match_excerpt(&msg.content.text, query);
                        Some(SearchResult {
                            message: map_message_response(msg),
                            thread_id: thread_id.to_string(),
                            thread_name: thread_name.to_string(),
                            match_count,
                            match_excerpt: excerpt,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        };

        if let Some(tid) = thread_id {
            // Scoped search - search only within specified thread
            if tid.starts_with("dm:") {
                // DM thread
                let contact_id = tid.strip_prefix("dm:").unwrap_or(tid);
                if let Ok(sync_response) = ctx
                    .message_service
                    .get_direct_messages(contact_id.to_string())
                    .await
                {
                    // Get contact name for thread_name from gossip contact store
                    let thread_name = if let Some(gossip) = ctx.gossip.as_ref() {
                        gossip
                            .contact_store
                            .get(contact_id)
                            .await
                            .and_then(|c| c.display_name)
                            .unwrap_or_else(|| contact_id.to_string())
                    } else {
                        contact_id.to_string()
                    };
                    results.extend(search_in_messages(
                        sync_response.messages,
                        tid,
                        &thread_name,
                        &query_lower,
                        query,
                    ));
                }
            } else {
                // Entity thread
                if let Ok(sync_response) = ctx
                    .message_service
                    .get_entity_messages(tid.to_string())
                    .await
                {
                    // Get entity name for thread_name
                    let thread_name = ctx
                        .entity_service
                        .get_entity(tid)
                        .await
                        .map(|e| e.name.clone())
                        .unwrap_or_else(|_| tid.to_string());
                    results.extend(search_in_messages(
                        sync_response.messages,
                        tid,
                        &thread_name,
                        &query_lower,
                        query,
                    ));
                }
            }
        } else {
            // Global search - search all entities and DMs

            // Search entities
            if let Ok(entities) = ctx.entity_service.list_entities().await {
                for entity in entities {
                    if let Ok(sync_response) = ctx
                        .message_service
                        .get_entity_messages(entity.id.clone())
                        .await
                    {
                        results.extend(search_in_messages(
                            sync_response.messages,
                            &entity.id,
                            &entity.name,
                            &query_lower,
                            query,
                        ));
                    }
                }
            }

            // Search DMs - only if gossip is available
            if let Some(gossip) = ctx.gossip.as_ref() {
                for contact in gossip.contact_store.all().await {
                    let contact_id = contact.four_words.clone().unwrap_or(contact.id.clone());
                    if let Ok(sync_response) = ctx
                        .message_service
                        .get_direct_messages(contact_id.clone())
                        .await
                    {
                        let thread_id = format!("dm:{}", contact_id);
                        let display_name =
                            contact.display_name.unwrap_or_else(|| contact_id.clone());
                        results.extend(search_in_messages(
                            sync_response.messages,
                            &thread_id,
                            &display_name,
                            &query_lower,
                            query,
                        ));
                    }
                }
            }
        }

        // Sort by match count (desc), then timestamp (desc)
        results.sort_by(|a, b| {
            b.match_count
                .cmp(&a.match_count)
                .then_with(|| b.message.timestamp.cmp(&a.message.timestamp))
        });

        // Limit results
        results.truncate(limit.min(50));

        Ok(results)
    }

    /// Create an excerpt around the first match in the text.
    fn create_match_excerpt(text: &str, query: &str) -> String {
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        if let Some(pos) = text_lower.find(&query_lower) {
            // Get context around the match (30 chars before, match, 30 chars after)
            let start = pos.saturating_sub(30);
            let end = (pos + query.len() + 30).min(text.len());

            let mut excerpt = String::new();
            if start > 0 {
                excerpt.push_str("...");
            }
            excerpt.push_str(&text[start..end]);
            if end < text.len() {
                excerpt.push_str("...");
            }
            excerpt
        } else {
            // Fallback: just truncate the text
            if text.len() > 100 {
                format!("{}...", &text[..100])
            } else {
                text.to_string()
            }
        }
    }

    /// Execute a command and return resulting events
    ///
    /// All mutations to application state MUST go through this method.
    /// Each command produces zero or more events that describe what changed.
    ///
    /// # Arguments
    /// * `command` - The command to execute
    ///
    /// # Returns
    /// List of events produced by the command, or an error
    pub async fn execute(&self, command: Command) -> CommandResult {
        let command_type = format!("{:?}", std::mem::discriminant(&command));

        match command {
            // ================================================================
            // Profile & Identity Commands
            // ================================================================
            Command::Initialize { .. } => {
                // Already initialized in new()
                Err(CommandError {
                    command_type,
                    message:
                        "App is already initialized. Create a new CommunitasApp instance instead."
                            .to_string(),
                    code: "ALREADY_INITIALIZED".to_string(),
                })
            }

            Command::UpdateDisplayName { display_name } => {
                let mut ctx = self.context.write().await;
                let old_name = ctx.display_name.clone();
                ctx.set_display_name(display_name.clone());

                let event = Event::DisplayNameUpdated {
                    old_name,
                    new_name: display_name,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Networking Commands
            // ================================================================
            Command::StartNetworking { preferred_port } => {
                let mut ctx = self.context.write().await;

                if ctx.is_networking_active() {
                    return Err(CommandError {
                        command_type,
                        message: "Networking is already active".to_string(),
                        code: "ALREADY_ACTIVE".to_string(),
                    });
                }

                let connection_identity =
                    ctx.start_networking(preferred_port)
                        .await
                        .map_err(|e| CommandError {
                            command_type: command_type.clone(),
                            message: e,
                            code: "NETWORKING_START_FAILED".to_string(),
                        })?;

                let listen_addr = ctx
                    .listen_address
                    .map(|a| a.to_string())
                    .unwrap_or_default();

                let event = Event::NetworkingStarted {
                    listen_address: listen_addr,
                    connection_identity,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::StopNetworking => {
                let mut ctx = self.context.write().await;
                ctx.stop_networking().await.map_err(|e| CommandError {
                    command_type: command_type.clone(),
                    message: e,
                    code: "NETWORKING_STOP_FAILED".to_string(),
                })?;

                let event = Event::NetworkingStopped;
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::ConnectToPeer { peer_four_words } => {
                let ctx = self.context.read().await;
                ctx.connect_to_peer(&peer_four_words)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: e,
                        code: "CONNECT_FAILED".to_string(),
                    })?;

                let event = Event::PeerConnected { peer_four_words };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RequestExternalAddress => {
                let mut ctx = self.context.write().await;
                ctx.request_external_address()
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: e,
                        code: "EXTERNAL_ADDRESS_FAILED".to_string(),
                    })?;

                let address = ctx
                    .external_address
                    .map(|a| a.to_string())
                    .unwrap_or_default();

                let event = Event::ExternalAddressDiscovered { address };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::AnnouncePresence => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let (public_key, private_key) =
                    gossip.get_sites_signing_keys().map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to load identity keypair: {e}"),
                        code: "IDENTITY_KEYPAIR_FAILED".to_string(),
                    })?;

                let connection_words = ctx
                    .external_address
                    .or(ctx.listen_address)
                    .map(|addr| {
                        crate::identity::conn_words(&addr).unwrap_or_else(|_| "unknown".to_string())
                    })
                    .unwrap_or_else(|| "no-address".to_string());

                let record = crate::peer_presence::PresenceRecord::new(
                    &public_key,
                    &private_key,
                    connection_words.clone(),
                )
                .map_err(|e| CommandError {
                    command_type: command_type.clone(),
                    message: format!("Failed to sign presence record: {e}"),
                    code: "PRESENCE_SIGN_FAILED".to_string(),
                })?;

                {
                    let mut cache = self.presence_cache.write().await;
                    cache.insert(record);
                }

                let event = Event::PresenceAnnounced {
                    connection_words,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::QueryPeerPresence { target_pubkey } => {
                // Check local cache first
                let cache = self.presence_cache.read().await;
                if let Some(record) = cache.get(&target_pubkey) {
                    let event = Event::PeerPresenceReceived {
                        record: record.record.clone(),
                    };
                    self.broadcast_event(event.clone());
                    return Ok(vec![event]);
                }
                drop(cache);

                let ctx = self.context.read().await;
                let gossip = match ctx.gossip.as_ref() {
                    Some(gossip) => gossip,
                    None => return Ok(vec![]),
                };

                let peer_id = PeerId::from_pubkey(&target_pubkey);
                let presence = gossip.presence.read().await;
                let groups = presence.get_groups().await;
                let mut best_record = None;

                for topic_id in groups {
                    let records = presence.get_group_presence(topic_id).await;
                    if let Some(record) = records.get(&peer_id) {
                        if record.is_expired() {
                            continue;
                        }
                        let should_replace = best_record
                            .as_ref()
                            .map(|existing: &saorsa_gossip_types::PresenceRecord| {
                                record.since > existing.since
                            })
                            .unwrap_or(true);
                        if should_replace {
                            best_record = Some(record.clone());
                        }
                    }
                }

                if let Some(record) = best_record {
                    let connection_words = record
                        .addr_hints
                        .iter()
                        .find_map(|hint| hint.parse::<std::net::SocketAddr>().ok())
                        .and_then(|addr| crate::identity::conn_words(&addr).ok())
                        .unwrap_or_else(|| "unknown".to_string());

                    let unsigned = crate::peer_presence::PresenceRecord::new_unsigned(
                        target_pubkey.clone(),
                        connection_words,
                        record.since,
                    );

                    {
                        let mut cache = self.presence_cache.write().await;
                        cache.insert(unsigned.clone());
                    }

                    let event = Event::PeerPresenceReceived { record: unsigned };
                    self.broadcast_event(event.clone());
                    return Ok(vec![event]);
                }

                Ok(vec![])
            }

            // ================================================================
            // Entity Management Commands
            // ================================================================
            Command::CreateEntity {
                name,
                entity_type,
                description,
                initial_members,
            } => {
                let ctx = self.context.read().await;
                let created_by = ctx.four_words.clone();

                let entity = ctx
                    .entity_service
                    .create_entity(
                        name.clone(),
                        entity_type,
                        description,
                        created_by.clone(),
                        initial_members,
                    )
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_ENTITY_FAILED".to_string(),
                    })?;

                let event = Event::EntityCreated {
                    entity_id: entity.id.clone(),
                    name,
                    entity_type,
                    created_by,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CreateLocalEntity {
                name,
                entity_type,
                description,
            } => {
                let ctx = self.context.read().await;
                let created_by = ctx.four_words.clone();

                let entity = ctx
                    .entity_service
                    .create_entity(
                        name.clone(),
                        entity_type,
                        description,
                        created_by.clone(),
                        vec![],
                    )
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_LOCAL_ENTITY_FAILED".to_string(),
                    })?;

                let event = Event::EntityCreated {
                    entity_id: entity.id.clone(),
                    name,
                    entity_type,
                    created_by,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::LinkEntityToNetwork {
                entity_id,
                four_words,
            } => {
                // Note: Entity linking is a conceptual operation - the entity service
                // doesn't have a direct link_to_network method. This would be implemented
                // by updating the entity's network identity field.
                let event = Event::EntityLinkedToNetwork {
                    entity_id,
                    four_words,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::MarkEntitySynced { entity_id } => {
                let event = Event::EntitySynced { entity_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::SetParentOrganization {
                entity_id,
                parent_org_id,
            } => {
                let ctx = self.context.read().await;
                ctx.entity_service
                    .set_parent_organization(&entity_id, &parent_org_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "SET_PARENT_ORG_FAILED".to_string(),
                    })?;

                let event = Event::ParentOrganizationSet {
                    entity_id,
                    parent_org_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::UpdateEntity {
                entity_type,
                entity_id,
                name,
                description,
            } => {
                let ctx = self.context.read().await;
                let entity = ctx
                    .entity_service
                    .update_entity(&entity_id, name.clone(), description)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "UPDATE_ENTITY_FAILED".to_string(),
                    })?;

                let event = Event::EntityUpdated {
                    entity_id,
                    entity_type,
                    name: Some(entity.name),
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteEntity {
                entity_type,
                entity_id,
            } => {
                let ctx = self.context.read().await;
                ctx.entity_service
                    .delete_entity(&entity_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "DELETE_ENTITY_FAILED".to_string(),
                    })?;

                let event = Event::EntityDeleted {
                    entity_id,
                    entity_type,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Member Management Commands
            // ================================================================
            Command::AddMember {
                entity_type,
                entity_id,
                member_id,
                role,
            } => {
                let ctx = self.context.read().await;
                ctx.entity_service
                    .add_member(entity_type, &entity_id, &member_id, &role)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "ADD_MEMBER_FAILED".to_string(),
                    })?;

                let event = Event::MemberAdded {
                    entity_type,
                    entity_id,
                    member_id,
                    role,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RemoveMember {
                entity_type,
                entity_id,
                member_id,
            } => {
                let ctx = self.context.read().await;
                let deleted_by = ctx.four_words.clone();
                ctx.entity_service
                    .remove_member(entity_type, &entity_id, &member_id, &deleted_by)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "REMOVE_MEMBER_FAILED".to_string(),
                    })?;

                let event = Event::MemberRemoved {
                    entity_type,
                    entity_id,
                    member_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RemoveOrganizationMember { org_id, member_id } => {
                let ctx = self.context.read().await;
                let deleted_by = ctx.four_words.clone();
                let result = ctx
                    .entity_service
                    .remove_organization_member(&org_id, &member_id, &deleted_by)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "REMOVE_ORG_MEMBER_FAILED".to_string(),
                    })?;

                let removed_from: Vec<(EntityType, String)> =
                    result.removed_in.into_iter().collect();

                let event = Event::OrganizationMemberRemoved {
                    org_id,
                    member_id,
                    removed_from,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::SetMemberRole {
                entity_type,
                entity_id,
                member_id,
                new_role,
            } => {
                let ctx = self.context.read().await;
                let old_role = ctx
                    .entity_service
                    .get_member_role(entity_type, &entity_id, &member_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "GET_ROLE_FAILED".to_string(),
                    })?;

                ctx.entity_service
                    .set_member_role(entity_type, &entity_id, &member_id, &new_role)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "SET_ROLE_FAILED".to_string(),
                    })?;

                let event = Event::MemberRoleChanged {
                    entity_type,
                    entity_id,
                    member_id,
                    old_role,
                    new_role,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Permission Commands
            // ================================================================
            Command::SetPermissionOverride {
                entity_type,
                entity_id,
                member_id,
                resource_type,
                access_level,
            } => {
                let ctx = self.context.read().await;

                ctx.entity_service
                    .set_permission_override(
                        entity_type,
                        &entity_id,
                        &member_id,
                        &resource_type,
                        &access_level,
                    )
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "SET_PERMISSION_FAILED".to_string(),
                    })?;

                let event = Event::PermissionOverrideSet {
                    entity_type,
                    entity_id,
                    member_id,
                    resource_type,
                    access_level,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RemovePermissionOverride {
                entity_type,
                entity_id,
                member_id,
                resource_type,
            } => {
                let ctx = self.context.read().await;

                ctx.entity_service
                    .remove_permission_override(entity_type, &entity_id, &member_id, &resource_type)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "REMOVE_PERMISSION_FAILED".to_string(),
                    })?;

                let event = Event::PermissionOverrideRemoved {
                    entity_type,
                    entity_id,
                    member_id,
                    resource_type,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Messaging Commands
            // ================================================================
            Command::SendMessage {
                entity_id,
                entity_type,
                text,
                author,
                reply_to_id,
                attachments: raw_attachments,
            } => {
                let ctx = self.context.read().await;
                let attachments = attachments_from_strings(raw_attachments);
                let content = crate::crdt::MessageContent {
                    text: text.clone(),
                    author: author.clone(),
                    attachments,
                };
                let message = ctx
                    .message_service
                    .send_message(entity_id.clone(), entity_type, content, reply_to_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "SEND_MESSAGE_FAILED".to_string(),
                    })?;

                let event = Event::MessageSent {
                    message_id: message.metadata.id,
                    entity_id,
                    entity_type,
                    author,
                    text,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::SendDirectMessage {
                recipients,
                text,
                author,
            } => {
                let ctx = self.context.read().await;
                let content = crate::crdt::MessageContent {
                    text: text.clone(),
                    author: author.clone(),
                    attachments: None,
                };
                let message_ids = ctx
                    .message_service
                    .send_direct_messages(recipients.clone(), content)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "SEND_DM_FAILED".to_string(),
                    })?;

                let event = Event::DirectMessageSent {
                    message_ids,
                    recipients,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteMessage {
                entity_id,
                entity_type,
                message_id,
            } => {
                let ctx = self.context.read().await;
                ctx.message_service
                    .delete_message(&entity_id, &message_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "DELETE_MESSAGE_FAILED".to_string(),
                    })?;

                let event = Event::MessageDeleted {
                    message_id,
                    entity_id,
                    entity_type,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::EditMessage {
                entity_id,
                entity_type,
                message_id,
                new_text,
            } => {
                let ctx = self.context.read().await;
                let edited_at = ctx
                    .message_service
                    .edit_message(&entity_id, &message_id, new_text.clone())
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "EDIT_MESSAGE_FAILED".to_string(),
                    })?;

                let event = Event::MessageEdited {
                    message_id,
                    entity_id,
                    entity_type,
                    new_text,
                    edited_at,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::AddReaction {
                entity_id,
                entity_type,
                message_id,
                emoji,
            } => {
                let ctx = self.context.read().await;
                let reactor_id = ctx.four_words.clone();
                ctx.message_service
                    .add_reaction(&entity_id, &message_id, emoji.clone(), reactor_id.clone())
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "ADD_REACTION_FAILED".to_string(),
                    })?;

                let event = Event::ReactionAdded {
                    message_id,
                    entity_id,
                    entity_type,
                    emoji,
                    reactor_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RemoveReaction {
                entity_id,
                entity_type,
                message_id,
                emoji,
            } => {
                let ctx = self.context.read().await;
                let reactor_id = ctx.four_words.clone();
                ctx.message_service
                    .remove_reaction(&entity_id, &message_id, emoji.clone(), reactor_id.clone())
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "REMOVE_REACTION_FAILED".to_string(),
                    })?;

                let event = Event::ReactionRemoved {
                    message_id,
                    entity_id,
                    entity_type,
                    emoji,
                    reactor_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::MarkThreadRead {
                thread_id,
                identity,
            } => {
                // The actual unread count management is handled by the UI service.
                // This command broadcasts the event so other subscribers can react.
                let event = Event::ThreadMarkedRead {
                    thread_id,
                    identity,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::SendTypingIndicator {
                thread_id,
                is_typing,
            } => {
                let ctx = self.context.read().await;
                let peer_id = ctx.four_words.clone();

                // Broadcast typing indicator event locally for other subscribers
                // In a full implementation, this would also be gossiped to peers
                let event = Event::TypingIndicatorReceived {
                    thread_id,
                    peer_id,
                    is_typing,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Invite Commands
            // ================================================================
            Command::CreateInvite {
                recipient_id,
                entity_type,
                entity_id,
                role,
                message,
                expires_in_hours,
            } => {
                let ctx = self.context.read().await;
                let creator_id = ctx.four_words.clone();

                let mut request = crate::invite_service::InviteRequest::new(
                    recipient_id.clone(),
                    entity_type,
                    entity_id.clone(),
                    role.clone(),
                );
                if let Some(msg) = message {
                    request = request.with_message(msg);
                }
                if let Some(hours) = expires_in_hours {
                    request = request.with_expiration(hours);
                }

                let invite = ctx
                    .invite_service
                    .create_invite(&creator_id, request)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_INVITE_FAILED".to_string(),
                    })?;

                let event = Event::InviteCreated {
                    invite_id: invite.id,
                    recipient_id,
                    entity_type,
                    entity_id,
                    role,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::AcceptInvite { invite_id } => {
                let ctx = self.context.read().await;
                let recipient_id = ctx.four_words.clone();

                // Get invite first to access its data
                let invite = ctx
                    .invite_service
                    .get_invite(&invite_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "GET_INVITE_FAILED".to_string(),
                    })?;

                ctx.invite_service
                    .accept_invite(&recipient_id, &invite_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "ACCEPT_INVITE_FAILED".to_string(),
                    })?;

                let event = Event::InviteAccepted {
                    invite_id,
                    recipient_id: invite.recipient_id,
                    entity_id: invite.entity_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RejectInvite { invite_id } => {
                let ctx = self.context.read().await;
                let recipient_id = ctx.four_words.clone();
                ctx.invite_service
                    .reject_invite(&recipient_id, &invite_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "REJECT_INVITE_FAILED".to_string(),
                    })?;

                let event = Event::InviteRejected { invite_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RevokeInvite { invite_id } => {
                let ctx = self.context.read().await;
                let revoker_id = ctx.four_words.clone();
                ctx.invite_service
                    .revoke_invite(&revoker_id, &invite_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "REVOKE_INVITE_FAILED".to_string(),
                    })?;

                let event = Event::InviteRevoked { invite_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Virtual Disk Commands
            // ================================================================
            Command::WriteFile {
                entity_id,
                disk_type,
                path,
                data,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                ctx.disk_service
                    .write_file(&entity_id, disk_type_internal, &path, &data)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "WRITE_FILE_FAILED".to_string(),
                    })?;

                let event = Event::FileWritten {
                    entity_id,
                    disk_type,
                    path,
                    size_bytes: data.len() as u64,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteFile {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                ctx.disk_service
                    .delete_file(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "DELETE_FILE_FAILED".to_string(),
                    })?;

                let event = Event::FileDeleted {
                    entity_id,
                    disk_type,
                    path,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CreateDirectory {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                ctx.disk_service
                    .create_directory(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_DIR_FAILED".to_string(),
                    })?;

                let event = Event::DirectoryCreated {
                    entity_id,
                    disk_type,
                    path,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::MoveFile {
                entity_id,
                disk_type,
                source_path,
                dest_path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                ctx.disk_service
                    .move_file(&entity_id, disk_type_internal, &source_path, &dest_path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "MOVE_FILE_FAILED".to_string(),
                    })?;

                let event = Event::FileMoved {
                    entity_id,
                    disk_type,
                    source_path,
                    dest_path,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CopyFile {
                entity_id,
                disk_type,
                source_path,
                dest_path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                ctx.disk_service
                    .copy_file(&entity_id, disk_type_internal, &source_path, &dest_path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "COPY_FILE_FAILED".to_string(),
                    })?;

                let event = Event::FileCopied {
                    entity_id,
                    disk_type,
                    source_path,
                    dest_path,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Chunked Transfer Commands
            // ================================================================
            Command::StartChunkedWrite {
                entity_id,
                disk_type,
                path,
                total_size,
                chunk_size,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let chunk_info = ctx
                    .disk_service
                    .start_chunked_write(
                        &entity_id,
                        disk_type_internal,
                        &path,
                        total_size,
                        chunk_size,
                    )
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "START_CHUNKED_WRITE_FAILED".to_string(),
                    })?;

                let event = Event::ChunkedWriteStarted {
                    entity_id,
                    disk_type,
                    path,
                    total_size,
                    total_chunks: chunk_info.total_chunks,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::WriteChunk {
                entity_id,
                disk_type,
                path,
                offset,
                data,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let result = ctx
                    .disk_service
                    .write_chunk(&entity_id, disk_type_internal, &path, offset, &data)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "WRITE_CHUNK_FAILED".to_string(),
                    })?;

                let event = Event::ChunkWritten {
                    entity_id,
                    disk_type,
                    path,
                    chunk_index: result.info.chunk_index,
                    offset,
                    size: result.info.size,
                    chunk_hash: result.info.chunk_hash,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::FinishChunkedWrite {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let file_info = ctx
                    .disk_service
                    .finish_chunked_write(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "FINISH_CHUNKED_WRITE_FAILED".to_string(),
                    })?;

                let event = Event::ChunkedWriteCompleted {
                    entity_id,
                    disk_type,
                    path,
                    total_size: file_info.size_bytes,
                    content_hash: file_info.content_hash,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::AbortChunkedWrite {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                ctx.disk_service
                    .abort_chunked_write(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "ABORT_CHUNKED_WRITE_FAILED".to_string(),
                    })?;

                let event = Event::ChunkedWriteAborted {
                    entity_id,
                    disk_type,
                    path,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::ResumeChunkedWrite {
                entity_id,
                disk_type,
                path,
                verify_hashes,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let result = ctx
                    .disk_service
                    .resume_chunked_write(&entity_id, disk_type_internal, &path, verify_hashes)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "RESUME_CHUNKED_WRITE_FAILED".to_string(),
                    })?;

                if !result.can_resume {
                    return Err(CommandError {
                        command_type: command_type.clone(),
                        message: result
                            .failure_reason
                            .unwrap_or_else(|| "Cannot resume transfer".to_string()),
                        code: "RESUME_NOT_POSSIBLE".to_string(),
                    });
                }

                let state = result.transfer_state.as_ref();
                let bytes_written = state.map(|s| s.bytes_written()).unwrap_or(0);
                let total_size = state.map(|s| s.total_size()).unwrap_or(0);
                let chunks_completed = state.map(|s| s.chunks_completed()).unwrap_or(0);
                let total_chunks = state.map(|s| s.total_chunks()).unwrap_or(0);

                let event = Event::ChunkedWriteResumed {
                    entity_id,
                    disk_type,
                    path,
                    bytes_written,
                    total_size,
                    chunks_completed,
                    total_chunks,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::VerifyChunks {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let results = ctx
                    .disk_service
                    .verify_written_chunks(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "VERIFY_CHUNKS_FAILED".to_string(),
                    })?;

                let verified_count = results.len() as u64;
                let all_valid = results.iter().all(|r| r.is_valid);

                let event = Event::ChunksVerified {
                    entity_id,
                    disk_type,
                    path,
                    verified_count,
                    all_valid,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Kanban Commands
            // ================================================================
            Command::CreateKanbanBoard {
                entity_id,
                board_name,
                description: _description, // BoardSettings used instead of description
            } => {
                let ctx = self.context.read().await;
                let board = ctx
                    .kanban_service
                    .create_board(&entity_id, board_name.clone(), None) // No settings for now
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_BOARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanBoardCreated {
                    board_id: board.id,
                    entity_id,
                    board_name,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CreateKanbanColumn {
                board_id,
                column_name,
                position,
            } => {
                let ctx = self.context.read().await;
                let column = ctx
                    .kanban_service
                    .add_column(&board_id, column_name.clone(), position)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_COLUMN_FAILED".to_string(),
                    })?;

                let event = Event::KanbanColumnCreated {
                    column_id: column.id,
                    board_id,
                    column_name,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CreateKanbanCard {
                board_id,
                column_id,
                title,
                description,
                assignee: _assignee, // Not used in create_card
            } => {
                let ctx = self.context.read().await;
                let card = ctx
                    .kanban_service
                    .create_card(&board_id, &column_id, title.clone(), description)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "CREATE_CARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanCardCreated {
                    card_id: card.id,
                    column_id,
                    title,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::MoveKanbanCard {
                board_id,
                card_id,
                target_column_id,
                position,
            } => {
                let ctx = self.context.read().await;

                // Get the current column before moving (for the event)
                let card = ctx
                    .kanban_service
                    .get_card(&board_id, &card_id)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "GET_CARD_FAILED".to_string(),
                    })?;

                let from_column_id = card.column_id.clone();

                ctx.kanban_service
                    .move_card(
                        &board_id,
                        &card_id,
                        &target_column_id,
                        position.unwrap_or(0),
                    )
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "MOVE_CARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanCardMoved {
                    card_id,
                    from_column_id,
                    to_column_id: target_column_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::UpdateKanbanCard {
                board_id,
                card_id,
                title,
                description,
                assignee: _assignee, // Not in CardUpdate struct
            } => {
                let ctx = self.context.read().await;
                let updates = communitas_kanban::CardUpdate {
                    title,
                    description,
                    is_draft: None,
                    is_golden: None,
                    due_date: None,
                    priority: None,
                };
                ctx.kanban_service
                    .update_card(&board_id, &card_id, updates)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "UPDATE_CARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanCardUpdated { card_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteKanbanCard { board_id, card_id } => {
                let ctx = self.context.read().await;
                ctx.kanban_service
                    .delete_card(&board_id, &card_id)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "DELETE_CARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanCardDeleted { card_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::UpdateKanbanBoard {
                board_id,
                name,
                description,
            } => {
                let ctx = self.context.read().await;
                let updates = communitas_kanban::BoardUpdate {
                    name: name.clone(),
                    description,
                    settings: None,
                };
                let board = ctx
                    .kanban_service
                    .update_board(&board_id, updates)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "UPDATE_BOARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanBoardUpdated {
                    board_id,
                    name: Some(board.name),
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteKanbanBoard { board_id } => {
                let ctx = self.context.read().await;
                ctx.kanban_service
                    .delete_board(&board_id)
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "DELETE_BOARD_FAILED".to_string(),
                    })?;

                let event = Event::KanbanBoardDeleted { board_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // WebRTC Commands
            // WebRTC integration uses CallId conversion and MediaConstraints.
            // ================================================================
            Command::StartCall {
                entity_id,
                video_enabled,
            } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available - start networking first".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Use default media constraints
                let constraints = crate::webrtc::MediaConstraints {
                    video: video_enabled,
                    audio: true,
                    screen_share: false,
                };

                let call_id = webrtc
                    .initiate_call(&entity_id, constraints)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "START_CALL_FAILED".to_string(),
                    })?;

                let event = Event::CallStarted {
                    call_id: call_id.to_string(),
                    entity_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::JoinCall { call_id } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Parse CallId from string (CallId wraps UUID)
                let uuid = Uuid::parse_str(&call_id).map_err(|_| CommandError {
                    command_type: command_type.clone(),
                    message: "Invalid call ID format".to_string(),
                    code: "INVALID_CALL_ID".to_string(),
                })?;
                let call_id_parsed = crate::webrtc::CallId(uuid);

                // Use default media constraints
                let constraints = crate::webrtc::MediaConstraints {
                    video: true,
                    audio: true,
                    screen_share: false,
                };

                webrtc
                    .accept_call(call_id_parsed, constraints)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "JOIN_CALL_FAILED".to_string(),
                    })?;

                let event = Event::CallJoined { call_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::LeaveCall { call_id } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Parse CallId from string (CallId wraps UUID)
                let uuid = Uuid::parse_str(&call_id).map_err(|_| CommandError {
                    command_type: command_type.clone(),
                    message: "Invalid call ID format".to_string(),
                    code: "INVALID_CALL_ID".to_string(),
                })?;
                let call_id_parsed = crate::webrtc::CallId(uuid);

                webrtc
                    .end_call(call_id_parsed)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "LEAVE_CALL_FAILED".to_string(),
                    })?;

                let event = Event::CallLeft { call_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::ToggleVideo { call_id, enabled } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Parse CallId from string (CallId wraps UUID)
                let uuid = Uuid::parse_str(&call_id).map_err(|_| CommandError {
                    command_type: command_type.clone(),
                    message: "Invalid call ID format".to_string(),
                    code: "INVALID_CALL_ID".to_string(),
                })?;
                let call_id_parsed = crate::webrtc::CallId(uuid);

                webrtc
                    .set_video_enabled(call_id_parsed, enabled)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "TOGGLE_VIDEO_FAILED".to_string(),
                    })?;

                let event = Event::VideoToggled { call_id, enabled };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::ToggleAudio { call_id, enabled } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Parse CallId from string (CallId wraps UUID)
                let uuid = Uuid::parse_str(&call_id).map_err(|_| CommandError {
                    command_type: command_type.clone(),
                    message: "Invalid call ID format".to_string(),
                    code: "INVALID_CALL_ID".to_string(),
                })?;
                let call_id_parsed = crate::webrtc::CallId(uuid);

                webrtc
                    .set_audio_enabled(call_id_parsed, enabled)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "TOGGLE_AUDIO_FAILED".to_string(),
                    })?;

                let event = Event::AudioToggled { call_id, enabled };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::StartScreenShare { call_id } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Parse CallId from string (CallId wraps UUID)
                let uuid = Uuid::parse_str(&call_id).map_err(|_| CommandError {
                    command_type: command_type.clone(),
                    message: "Invalid call ID format".to_string(),
                    code: "INVALID_CALL_ID".to_string(),
                })?;
                let call_id_parsed = crate::webrtc::CallId(uuid);

                webrtc
                    .start_screen_share(call_id_parsed)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "START_SCREEN_SHARE_FAILED".to_string(),
                    })?;

                let event = Event::ScreenShareStarted { call_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::StopScreenShare { call_id } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "WebRTC not available".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                // Parse CallId from string (CallId wraps UUID)
                let uuid = Uuid::parse_str(&call_id).map_err(|_| CommandError {
                    command_type: command_type.clone(),
                    message: "Invalid call ID format".to_string(),
                    code: "INVALID_CALL_ID".to_string(),
                })?;
                let call_id_parsed = crate::webrtc::CallId(uuid);

                webrtc
                    .stop_screen_share(call_id_parsed)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("{}", e),
                        code: "STOP_SCREEN_SHARE_FAILED".to_string(),
                    })?;

                let event = Event::ScreenShareStopped { call_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // ================================================================
            // Contact Management Commands
            // ================================================================
            Command::CreateContact {
                display_name,
                four_words,
                is_favourite,
            } => {
                let ctx = self.context.read().await;
                let storage_dir = ctx.profile.storage_dir.clone();
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started - contacts require gossip context".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                use crate::gossip::contact_storage::ContactRecord;

                // Create contact based on whether it has network identity
                let mut contact = match four_words.clone() {
                    Some(fw) => ContactRecord::with_display_name(fw, display_name.clone()),
                    None => ContactRecord::new_local(display_name.clone()),
                };

                // Set favourite if requested
                contact.is_favourite = is_favourite;
                let contact_id = contact.id.clone();

                gossip
                    .contact_store
                    .add(contact)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to create contact: {}", e),
                        code: "CREATE_CONTACT_FAILED".to_string(),
                    })?;

                if is_favourite && let Some(ref fw) = four_words {
                    gossip
                        .add_favourite_contact(fw.clone())
                        .await
                        .map_err(|e| CommandError {
                            command_type: command_type.clone(),
                            message: format!("Failed to add favourite: {}", e),
                            code: "SET_FAVOURITE_FAILED".to_string(),
                        })?;
                }

                self.persist_contacts(&command_type, &storage_dir, &gossip.contact_store)
                    .await?;

                let event = Event::ContactCreated {
                    contact_id,
                    display_name,
                    four_words,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::UpdateContact {
                contact_id,
                display_name,
                is_favourite,
            } => {
                let ctx = self.context.read().await;
                let storage_dir = ctx.profile.storage_dir.clone();
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                // Get the existing contact
                let mut contact = gossip
                    .contact_store
                    .get_by_id(&contact_id)
                    .await
                    .ok_or_else(|| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Contact not found: {}", contact_id),
                        code: "CONTACT_NOT_FOUND".to_string(),
                    })?;

                // Update display name if provided
                if let Some(ref name) = display_name {
                    contact.display_name = Some(name.clone());
                }

                // Update favourite status if provided
                if let Some(fav) = is_favourite {
                    contact.is_favourite = fav;
                }

                // Save updated contact
                let updated_four_words = contact.four_words.clone();
                let updated_is_favourite = contact.is_favourite;
                gossip
                    .contact_store
                    .update(contact)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to update contact: {}", e),
                        code: "UPDATE_CONTACT_FAILED".to_string(),
                    })?;

                if let Some(fav_update) = is_favourite {
                    if let Some(ref fw) = updated_four_words {
                        if fav_update {
                            gossip
                                .add_favourite_contact(fw.clone())
                                .await
                                .map_err(|e| CommandError {
                                    command_type: command_type.clone(),
                                    message: format!("Failed to add favourite: {}", e),
                                    code: "SET_FAVOURITE_FAILED".to_string(),
                                })?;
                        } else {
                            gossip.remove_favourite_contact(fw).await.map_err(|e| {
                                CommandError {
                                    command_type: command_type.clone(),
                                    message: format!("Failed to remove favourite: {}", e),
                                    code: "REMOVE_FAVOURITE_FAILED".to_string(),
                                }
                            })?;
                        }
                    }
                } else if updated_is_favourite && let Some(ref fw) = updated_four_words {
                    gossip
                        .add_favourite_contact(fw.clone())
                        .await
                        .map_err(|e| CommandError {
                            command_type: command_type.clone(),
                            message: format!("Failed to add favourite: {}", e),
                            code: "SET_FAVOURITE_FAILED".to_string(),
                        })?;
                }

                self.persist_contacts(&command_type, &storage_dir, &gossip.contact_store)
                    .await?;

                let event = Event::ContactUpdated {
                    contact_id,
                    display_name,
                    is_favourite,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteContact { contact_id } => {
                let ctx = self.context.read().await;
                let storage_dir = ctx.profile.storage_dir.clone();
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let removed_contact = gossip.contact_store.get_by_id(&contact_id).await;
                let removed_four_words = removed_contact
                    .as_ref()
                    .and_then(|contact| contact.four_words.clone());
                let removed_favourite = removed_contact
                    .as_ref()
                    .map(|contact| contact.is_favourite)
                    .unwrap_or(false);

                gossip
                    .contact_store
                    .remove_by_id(&contact_id)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to delete contact: {}", e),
                        code: "DELETE_CONTACT_FAILED".to_string(),
                    })?;

                if removed_favourite && let Some(fw) = removed_four_words {
                    gossip
                        .remove_favourite_contact(&fw)
                        .await
                        .map_err(|e| CommandError {
                            command_type: command_type.clone(),
                            message: format!("Failed to remove favourite: {}", e),
                            code: "REMOVE_FAVOURITE_FAILED".to_string(),
                        })?;
                }

                self.persist_contacts(&command_type, &storage_dir, &gossip.contact_store)
                    .await?;

                let event = Event::ContactDeleted { contact_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::LinkContact {
                contact_id,
                four_words,
            } => {
                let ctx = self.context.read().await;
                let storage_dir = ctx.profile.storage_dir.clone();
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let linked_contact = gossip
                    .contact_store
                    .link_contact(&contact_id, &four_words)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to link contact: {}", e),
                        code: "LINK_CONTACT_FAILED".to_string(),
                    })?;

                if linked_contact.is_favourite
                    && let Some(fw) = linked_contact.four_words.clone()
                {
                    gossip
                        .add_favourite_contact(fw.clone())
                        .await
                        .map_err(|e| CommandError {
                            command_type: command_type.clone(),
                            message: format!("Failed to add favourite: {}", e),
                            code: "SET_FAVOURITE_FAILED".to_string(),
                        })?;
                }

                self.persist_contacts(&command_type, &storage_dir, &gossip.contact_store)
                    .await?;

                let event = Event::ContactLinked {
                    contact_id,
                    four_words,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::SetFavouriteContact { four_words } => {
                let ctx = self.context.read().await;
                let storage_dir = ctx.profile.storage_dir.clone();
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                // Get the contact by four_words, update favourite, save
                let mut contact =
                    gossip
                        .contact_store
                        .get(&four_words)
                        .await
                        .ok_or_else(|| CommandError {
                            command_type: command_type.clone(),
                            message: format!("Contact not found: {}", four_words),
                            code: "CONTACT_NOT_FOUND".to_string(),
                        })?;

                contact.is_favourite = true;
                gossip
                    .contact_store
                    .update(contact)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to set favourite: {}", e),
                        code: "SET_FAVOURITE_FAILED".to_string(),
                    })?;

                gossip
                    .add_favourite_contact(four_words.clone())
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to add favourite: {}", e),
                        code: "SET_FAVOURITE_FAILED".to_string(),
                    })?;

                self.persist_contacts(&command_type, &storage_dir, &gossip.contact_store)
                    .await?;

                let event = Event::ContactFavouriteSet { four_words };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::RemoveFavouriteContact { four_words } => {
                let ctx = self.context.read().await;
                let storage_dir = ctx.profile.storage_dir.clone();
                let gossip = ctx.gossip.as_ref().ok_or_else(|| CommandError {
                    command_type: command_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                // Get the contact by four_words, update favourite, save
                let mut contact =
                    gossip
                        .contact_store
                        .get(&four_words)
                        .await
                        .ok_or_else(|| CommandError {
                            command_type: command_type.clone(),
                            message: format!("Contact not found: {}", four_words),
                            code: "CONTACT_NOT_FOUND".to_string(),
                        })?;

                contact.is_favourite = false;
                gossip
                    .contact_store
                    .update(contact)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to remove favourite: {}", e),
                        code: "REMOVE_FAVOURITE_FAILED".to_string(),
                    })?;

                gossip
                    .remove_favourite_contact(&four_words)
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to remove favourite: {}", e),
                        code: "REMOVE_FAVOURITE_FAILED".to_string(),
                    })?;

                self.persist_contacts(&command_type, &storage_dir, &gossip.contact_store)
                    .await?;

                let event = Event::ContactFavouriteRemoved { four_words };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // Website Publishing Commands
            Command::CreateWebsite {
                entity_id,
                html,
                css,
                js,
                metadata,
            } => {
                // Validate input
                if html.is_empty() {
                    return Err(CommandError {
                        command_type: command_type.clone(),
                        message: "HTML content cannot be empty".to_string(),
                        code: "INVALID_HTML".to_string(),
                    });
                }
                if html.len() > 1024 * 1024 {
                    return Err(CommandError {
                        command_type: command_type.clone(),
                        message: "HTML content too large (max 1MB)".to_string(),
                        code: "HTML_TOO_LARGE".to_string(),
                    });
                }

                let ctx = self.context.read().await;

                // Create content hash using Blake3
                let mut content = html.clone();
                if let Some(ref css_content) = css {
                    content.push_str(css_content);
                }
                if let Some(ref js_content) = js {
                    content.push_str(js_content);
                }

                let hash = blake3::hash(content.as_bytes());
                let hash_hex = hash.to_hex().to_string();
                let published_at = chrono::Utc::now().timestamp();
                let size_bytes = content.len();

                // Create CRDT document for website
                let doc = yrs::Doc::new();
                let root = doc.get_or_insert_map("website");
                {
                    let mut txn = doc.transact_mut();
                    root.insert(&mut txn, "entity_id", entity_id.clone());
                    root.insert(&mut txn, "html", html);
                    root.insert(&mut txn, "css", css.unwrap_or_default());
                    root.insert(&mut txn, "js", js.unwrap_or_default());
                    root.insert(&mut txn, "hash", hash_hex.clone());
                    root.insert(&mut txn, "published_at", published_at);
                    root.insert(&mut txn, "size_bytes", size_bytes as i64);
                    if let Some(meta) = metadata {
                        root.insert(&mut txn, "metadata", meta);
                    }
                }

                // Save to CRDT manager
                ctx.crdt_manager
                    .save_document(
                        &format!("website:{}", entity_id),
                        "website",
                        &entity_id,
                        &doc,
                    )
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to save website: {}", e),
                        code: "WEBSITE_SAVE_FAILED".to_string(),
                    })?;

                let event = Event::WebsiteCreated {
                    entity_id,
                    website_root_hash: hash_hex,
                    published_at,
                    size_bytes,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::UpdateWebsite {
                entity_id,
                html,
                css,
                js,
                metadata: _,
            } => {
                let ctx = self.context.read().await;

                // Load existing website document
                let doc = ctx
                    .crdt_manager
                    .load_document(&format!("website:{}", entity_id))
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Website not found: {}", e),
                        code: "WEBSITE_NOT_FOUND".to_string(),
                    })?;

                let root = doc.get_or_insert_map("website");
                {
                    let mut txn = doc.transact_mut();
                    if let Some(ref h) = html {
                        root.insert(&mut txn, "html", h.clone());
                    }
                    if let Some(ref c) = css {
                        root.insert(&mut txn, "css", c.clone());
                    }
                    if let Some(ref j) = js {
                        root.insert(&mut txn, "js", j.clone());
                    }
                }

                // Recalculate hash
                let txn = doc.transact();
                let current_html = match root.get(&txn, "html") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                let current_css = match root.get(&txn, "css") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                let current_js = match root.get(&txn, "js") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                drop(txn);

                let mut content = current_html;
                content.push_str(&current_css);
                content.push_str(&current_js);

                let hash = blake3::hash(content.as_bytes());
                let hash_hex = hash.to_hex().to_string();
                let updated_at = chrono::Utc::now().timestamp();
                let size_bytes = content.len();

                // Update hash and timestamp
                {
                    let mut txn = doc.transact_mut();
                    root.insert(&mut txn, "hash", hash_hex.clone());
                    root.insert(&mut txn, "updated_at", updated_at);
                    root.insert(&mut txn, "size_bytes", size_bytes as i64);
                }

                // Save updated document
                ctx.crdt_manager
                    .save_document(
                        &format!("website:{}", entity_id),
                        "website",
                        &entity_id,
                        &doc,
                    )
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to update website: {}", e),
                        code: "WEBSITE_UPDATE_FAILED".to_string(),
                    })?;

                let event = Event::WebsiteUpdated {
                    entity_id,
                    website_root_hash: hash_hex,
                    updated_at,
                    size_bytes,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::DeleteWebsite { entity_id } => {
                let ctx = self.context.read().await;

                // Delete website document
                ctx.crdt_manager
                    .delete_document(&format!("website:{}", entity_id))
                    .await
                    .map_err(|e| CommandError {
                        command_type: command_type.clone(),
                        message: format!("Failed to delete website: {}", e),
                        code: "WEBSITE_DELETE_FAILED".to_string(),
                    })?;

                let event = Event::WebsiteDeleted { entity_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            // Canvas Commands - stub implementations for MCP parity
            // TODO: Implement full canvas service integration
            Command::CanvasAddText {
                entity_id,
                content,
                x: _,
                y: _,
                font_size: _,
                color: _,
            } => {
                let element_id = uuid::Uuid::new_v4().to_string();
                let event = Event::CanvasTextAdded {
                    entity_id,
                    element_id,
                    content,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasAddImage {
                entity_id,
                src,
                x: _,
                y: _,
                width: _,
                height: _,
            } => {
                let element_id = uuid::Uuid::new_v4().to_string();
                let event = Event::CanvasImageAdded {
                    entity_id,
                    element_id,
                    src,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasAddChart {
                entity_id,
                chart_type,
                data: _,
                x: _,
                y: _,
                width: _,
                height: _,
            } => {
                let element_id = uuid::Uuid::new_v4().to_string();
                let event = Event::CanvasChartAdded {
                    entity_id,
                    element_id,
                    chart_type,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasAddElement {
                entity_id,
                element_type,
                content: _,
                transform: _,
            } => {
                // Generate new element ID for the replayed operation
                let element_id = uuid::Uuid::new_v4().to_string();
                let event = Event::CanvasElementAdded {
                    entity_id,
                    element_id,
                    element_type,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasUpdateElement {
                entity_id,
                element_id,
                changes: _,
            } => {
                let event = Event::CanvasElementUpdated {
                    entity_id,
                    element_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }
            Command::CanvasRemoveElement {
                entity_id,
                element_id,
            } => {
                let event = Event::CanvasElementRemoved {
                    entity_id,
                    element_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasUpdateTransform {
                entity_id,
                element_id,
                x: _,
                y: _,
                width: _,
                height: _,
                rotation: _,
                z_index: _,
            } => {
                let event = Event::CanvasTransformUpdated {
                    entity_id,
                    element_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasSelectElement {
                entity_id,
                element_id,
            } => {
                let event = Event::CanvasElementSelected {
                    entity_id,
                    element_id,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasDeselectAll { entity_id } => {
                let event = Event::CanvasDeselected { entity_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasSetViewport {
                entity_id,
                width,
                height,
            } => {
                let event = Event::CanvasViewportUpdated {
                    entity_id,
                    width,
                    height,
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasSetView {
                entity_id,
                zoom,
                pan_x,
                pan_y,
            } => {
                let event = Event::CanvasViewUpdated {
                    entity_id,
                    zoom: zoom.unwrap_or(1.0),
                    pan_x: pan_x.unwrap_or(0.0),
                    pan_y: pan_y.unwrap_or(0.0),
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasClear { entity_id } => {
                let event = Event::CanvasCleared { entity_id };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }

            Command::CanvasImport { entity_id, json: _ } => {
                // TODO: Parse JSON and create elements
                let event = Event::CanvasImported {
                    entity_id,
                    element_count: 0, // Stub - would be actual count
                };
                self.broadcast_event(event.clone());
                Ok(vec![event])
            }
        }
    }

    /// Execute a query and return the response
    ///
    /// All reads from application state should go through this method.
    /// This enables consistent access control and caching.
    ///
    /// # Arguments
    /// * `query` - The query to execute
    ///
    /// # Returns
    /// Query response or error
    pub async fn query(&self, query: Query) -> QueryResult {
        let query_type = format!("{:?}", std::mem::discriminant(&query));

        match query {
            // ================================================================
            // Profile & Identity Queries
            // ================================================================
            Query::GetProfile => {
                let ctx = self.context.read().await;
                Ok(QueryResponse::Profile {
                    four_words: ctx.four_words.clone(),
                    display_name: ctx.display_name.clone(),
                    device_name: ctx.device_name.clone(),
                    device_type: format!("{:?}", ctx.device_type()),
                })
            }

            Query::IsNetworkingActive => {
                let ctx = self.context.read().await;
                Ok(QueryResponse::Bool(ctx.is_networking_active()))
            }

            Query::GetConnectionIdentity => {
                let ctx = self.context.read().await;
                Ok(QueryResponse::OptionalString(
                    ctx.connection_identity().map(String::from),
                ))
            }

            Query::GetExternalAddress => {
                let ctx = self.context.read().await;
                Ok(QueryResponse::OptionalString(
                    ctx.external_address.map(|a| a.to_string()),
                ))
            }

            Query::GetConnectionWords => {
                let ctx = self.context.read().await;
                // Fall back to listen_address if external_address not yet discovered
                let words = ctx
                    .external_address
                    .or(ctx.listen_address)
                    .and_then(|addr| conn_words(&addr).ok());
                Ok(QueryResponse::OptionalString(words))
            }

            // ================================================================
            // Entity Queries
            // ================================================================
            Query::GetEntity { entity_id } => {
                let ctx = self.context.read().await;
                let entity = ctx
                    .entity_service
                    .get_entity(&entity_id)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_ENTITY_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::Entity(entity_to_response(&entity)))
            }

            Query::ListEntities => {
                let ctx = self.context.read().await;
                let entities =
                    ctx.entity_service
                        .list_entities()
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("{}", e),
                            code: "LIST_ENTITIES_FAILED".to_string(),
                        })?;

                let responses: Vec<EntityResponse> =
                    entities.iter().map(entity_to_response).collect();
                Ok(QueryResponse::EntityList(responses))
            }

            Query::ListEntitiesByType { entity_type } => {
                let ctx = self.context.read().await;
                let all_entities =
                    ctx.entity_service
                        .list_entities()
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("{}", e),
                            code: "LIST_BY_TYPE_FAILED".to_string(),
                        })?;

                // Filter by entity type
                let entities: Vec<_> = all_entities
                    .into_iter()
                    .filter(|e| e.entity_type == entity_type)
                    .collect();

                let responses: Vec<EntityResponse> =
                    entities.iter().map(entity_to_response).collect();
                Ok(QueryResponse::EntityList(responses))
            }

            Query::ListChildEntities { org_id } => {
                let ctx = self.context.read().await;
                let all_entities =
                    ctx.entity_service
                        .list_entities()
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("{}", e),
                            code: "LIST_CHILDREN_FAILED".to_string(),
                        })?;

                // Filter by parent organization
                let entities: Vec<_> = all_entities
                    .into_iter()
                    .filter(|e| e.parent_org_id.as_deref() == Some(&org_id))
                    .collect();

                let responses: Vec<EntityResponse> =
                    entities.iter().map(entity_to_response).collect();
                Ok(QueryResponse::EntityList(responses))
            }

            // ================================================================
            // Member Queries
            // ================================================================
            Query::ListMembers {
                entity_type,
                entity_id,
            } => {
                let ctx = self.context.read().await;
                let members = ctx
                    .entity_service
                    .list_members(entity_type, &entity_id)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "LIST_MEMBERS_FAILED".to_string(),
                    })?;

                let responses: Vec<MemberResponse> = members
                    .into_iter()
                    .filter(|m| !m.deleted) // Exclude deleted members
                    .map(|m| MemberResponse {
                        member_id: m.member_id,
                        role: m.role,
                        joined_at: m.joined_at,
                    })
                    .collect();
                Ok(QueryResponse::MemberList(responses))
            }

            Query::GetMemberRole {
                entity_type,
                entity_id,
                member_id,
            } => {
                let ctx = self.context.read().await;
                let role = ctx
                    .entity_service
                    .get_member_role(entity_type, &entity_id, &member_id)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_ROLE_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::MemberRole(role))
            }

            Query::GetPermissionOverrides {
                entity_type,
                entity_id,
                member_id,
            } => {
                let ctx = self.context.read().await;
                let overrides = ctx
                    .entity_service
                    .get_permission_overrides(entity_type, &entity_id, &member_id)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_OVERRIDES_FAILED".to_string(),
                    })?;

                let pairs: Vec<(String, String)> = overrides
                    .into_iter()
                    .map(|(res, lvl)| {
                        (
                            serde_json::to_string(&res)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string(),
                            serde_json::to_string(&lvl)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string(),
                        )
                    })
                    .collect();

                Ok(QueryResponse::PermissionOverrides(pairs))
            }

            // ================================================================
            // Message Queries
            // ================================================================
            Query::GetMessage {
                entity_id,
                message_id,
            } => {
                let ctx = self.context.read().await;
                let sync_response = ctx
                    .message_service
                    .get_entity_messages(entity_id.clone())
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_MESSAGE_FAILED".to_string(),
                    })?;

                let message = sync_response
                    .messages
                    .into_iter()
                    .find(|m| m.metadata.id == message_id)
                    .ok_or_else(|| QueryError {
                        query_type: query_type.clone(),
                        message: format!("Message not found: {}", message_id),
                        code: "MESSAGE_NOT_FOUND".to_string(),
                    })?;

                Ok(QueryResponse::Message(map_message_response(message)))
            }

            Query::GetEntityMessages { entity_id } => {
                let ctx = self.context.read().await;
                let sync_response = ctx
                    .message_service
                    .get_entity_messages(entity_id.clone())
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_MESSAGES_FAILED".to_string(),
                    })?;

                let responses: Vec<MessageResponse> = sync_response
                    .messages
                    .into_iter()
                    .map(map_message_response)
                    .collect();

                Ok(QueryResponse::Messages(responses))
            }

            Query::GetThreadMessages {
                entity_id,
                parent_message_id,
            } => {
                let ctx = self.context.read().await;
                let messages = ctx
                    .message_service
                    .get_thread_messages(entity_id.clone(), parent_message_id.clone())
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_THREAD_FAILED".to_string(),
                    })?;

                let responses: Vec<MessageResponse> =
                    messages.into_iter().map(map_message_response).collect();

                Ok(QueryResponse::Messages(responses))
            }

            Query::GetDirectMessages { other_peer_id } => {
                let ctx = self.context.read().await;
                let sync_response = ctx
                    .message_service
                    .get_direct_messages(other_peer_id.clone())
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_DM_FAILED".to_string(),
                    })?;

                let responses: Vec<MessageResponse> = sync_response
                    .messages
                    .into_iter()
                    .map(map_message_response)
                    .collect();

                Ok(QueryResponse::Messages(responses))
            }

            Query::SearchMessages {
                query,
                thread_id,
                limit,
            } => {
                let ctx = self.context.read().await;
                let results: Vec<crate::command::SearchResult> =
                    Self::search_messages(&ctx, &query, thread_id.as_deref(), limit)
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: e,
                            code: "SEARCH_FAILED".to_string(),
                        })?;

                Ok(QueryResponse::SearchResults(results))
            }

            Query::GetEntitySyncState {
                entity_id,
                entity_type,
            } => {
                let ctx = self.context.read().await;
                let sync_state = ctx
                    .message_service
                    .get_entity_sync_state(entity_id.clone(), entity_type)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_SYNC_STATE_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::SyncState(SyncStateResponse {
                    entity_id: sync_state.entity_id,
                    entity_type: sync_state.entity_type,
                    message_count: sync_state.message_count,
                    last_sync_time: sync_state.last_sync_time,
                }))
            }

            // ================================================================
            // Invite Queries
            // ================================================================
            Query::GetInvite { invite_id } => {
                let ctx = self.context.read().await;
                let invite = ctx
                    .invite_service
                    .get_invite(&invite_id)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_INVITE_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::Invite(invite_to_response(&invite)))
            }

            Query::ListPendingInvites => {
                let ctx = self.context.read().await;
                let user_id = ctx.four_words.clone();
                let invites = ctx
                    .invite_service
                    .list_pending_invites(&user_id)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "LIST_PENDING_FAILED".to_string(),
                    })?;

                let responses: Vec<InviteResponse> =
                    invites.iter().map(invite_to_response).collect();
                Ok(QueryResponse::InviteList(responses))
            }

            Query::ListSentInvites => {
                let ctx = self.context.read().await;
                let user_id = ctx.four_words.clone();

                let entities =
                    ctx.entity_service
                        .list_entities()
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("Failed to list entities: {}", e),
                            code: "LIST_ENTITIES_FAILED".to_string(),
                        })?;

                let mut sent_invites = Vec::new();
                for entity in entities {
                    match ctx
                        .invite_service
                        .list_entity_invites(&user_id, entity.entity_type, &entity.id)
                        .await
                    {
                        Ok(invites) => {
                            sent_invites.extend(
                                invites
                                    .into_iter()
                                    .filter(|invite| invite.creator_id == user_id),
                            );
                        }
                        Err(err) => {
                            warn!(
                                "Skipping invites for entity {} due to error: {}",
                                entity.id, err
                            );
                        }
                    }
                }

                let responses: Vec<InviteResponse> =
                    sent_invites.iter().map(invite_to_response).collect();
                Ok(QueryResponse::InviteList(responses))
            }

            // ================================================================
            // Virtual Disk Queries
            // ================================================================
            Query::ReadFile {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let data = ctx
                    .disk_service
                    .read_file(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "READ_FILE_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::FileContents(data))
            }

            Query::ListFiles {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let files = ctx
                    .disk_service
                    .list_files(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "LIST_FILES_FAILED".to_string(),
                    })?;

                let responses: Vec<FileInfoResponse> = files
                    .into_iter()
                    .map(|f| FileInfoResponse {
                        path: f.path,
                        name: f.name,
                        is_directory: f.is_directory,
                        size_bytes: f.size_bytes,
                        modified_at: f.modified_at,
                    })
                    .collect();

                Ok(QueryResponse::FileList(responses))
            }

            Query::GetDiskStats {
                entity_id,
                disk_type,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let stats = ctx
                    .disk_service
                    .get_stats(&entity_id, disk_type_internal)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_STATS_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::DiskStats(DiskStatsResponse {
                    entity_id,
                    disk_type,
                    used_bytes: stats.used_bytes,
                    file_count: stats.file_count,
                    dir_count: stats.dir_count,
                }))
            }

            Query::ListDisks { entity_id } => {
                let ctx = self.context.read().await;

                // Get stats for all disk types
                let mut disks = Vec::new();
                for disk_type in [DiskType::Private, DiskType::Public, DiskType::Shared] {
                    if let Ok(stats) = ctx.disk_service.get_stats(&entity_id, disk_type).await {
                        let disk_type_arg = match disk_type {
                            DiskType::Private => DiskTypeArg::Private,
                            DiskType::Public => DiskTypeArg::Public,
                            DiskType::Shared => DiskTypeArg::Shared,
                        };
                        disks.push(DiskInfoResponse {
                            disk_type: disk_type_arg,
                            entity_id: entity_id.clone(),
                            total_bytes: 10 * 1024 * 1024 * 1024, // 10GB placeholder
                            used_bytes: stats.used_bytes,
                            available_bytes: 10 * 1024 * 1024 * 1024 - stats.used_bytes,
                            file_count: stats.file_count as u64,
                        });
                    }
                }

                Ok(QueryResponse::DiskList(disks))
            }

            Query::GetFilePreview {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let data = ctx
                    .disk_service
                    .read_file(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_FILE_PREVIEW_FAILED".to_string(),
                    })?;

                // Determine MIME type from extension
                let mime_type = path
                    .rsplit('.')
                    .next()
                    .map(|ext| match ext.to_lowercase().as_str() {
                        "txt" | "md" | "rs" | "js" | "ts" | "json" => "text/plain",
                        "html" => "text/html",
                        "css" => "text/css",
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "svg" => "image/svg+xml",
                        "pdf" => "application/pdf",
                        _ => "application/octet-stream",
                    })
                    .unwrap_or("application/octet-stream")
                    .to_string();

                // Generate text preview for text files
                let text_preview = if mime_type.starts_with("text/") {
                    String::from_utf8(data.clone())
                        .ok()
                        .map(|s| s.chars().take(500).collect::<String>())
                } else {
                    None
                };

                // Compute checksum using blake3
                let checksum = blake3::hash(&data).to_string();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);

                Ok(QueryResponse::FilePreview(FilePreviewResponse {
                    path,
                    mime_type,
                    size_bytes: data.len() as u64,
                    thumbnail: None, // Image thumbnail generation not implemented
                    text_preview,
                    checksum,
                    created_at: now,
                    modified_at: now,
                }))
            }

            // ================================================================
            // Chunked Transfer Queries
            // ================================================================
            Query::ReadChunk {
                entity_id,
                disk_type,
                path,
                offset,
                chunk_size,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let result = ctx
                    .disk_service
                    .read_chunk(&entity_id, disk_type_internal, &path, offset, chunk_size)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "READ_CHUNK_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::ChunkRead(ChunkReadResponse {
                    data: result.data,
                    offset: result.info.offset,
                    size: result.info.size,
                    chunk_hash: result.info.chunk_hash,
                    total_size: result.info.total_size,
                    total_chunks: result.info.total_chunks,
                    chunk_index: result.info.chunk_index,
                    is_last: result.is_last,
                }))
            }

            Query::GetFileMetadata {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let file_info = ctx
                    .disk_service
                    .get_file_info(&entity_id, disk_type_internal, &path)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_FILE_METADATA_FAILED".to_string(),
                    })?;

                use crate::disk_service::{DEFAULT_CHUNK_SIZE, EntityDiskService};
                let chunk_count = EntityDiskService::calculate_chunk_count(
                    file_info.size_bytes,
                    Some(DEFAULT_CHUNK_SIZE),
                );

                Ok(QueryResponse::FileMetadata(FileMetadataResponse {
                    path: file_info.path,
                    name: file_info.name,
                    is_directory: file_info.is_directory,
                    size_bytes: file_info.size_bytes,
                    modified_at: file_info.modified_at,
                    content_hash: file_info.content_hash,
                    chunk_count,
                }))
            }

            Query::GetChunkedWriteProgress {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let progress = ctx
                    .disk_service
                    .get_chunked_write_progress(&entity_id, disk_type_internal, &path)
                    .await;

                use crate::disk_service::DEFAULT_CHUNK_SIZE;
                match progress {
                    Some((bytes_written, total_size)) => {
                        let progress_percent = if total_size > 0 {
                            (bytes_written as f32 / total_size as f32) * 100.0
                        } else {
                            0.0
                        };
                        let total_chunks = if total_size == 0 {
                            0
                        } else {
                            total_size.div_ceil(DEFAULT_CHUNK_SIZE)
                        };
                        let chunks_completed = bytes_written / DEFAULT_CHUNK_SIZE;

                        Ok(QueryResponse::ChunkedWriteProgress(
                            ChunkedWriteProgressResponse {
                                entity_id,
                                disk_type,
                                path,
                                bytes_written,
                                total_size,
                                progress_percent,
                                chunks_completed,
                                total_chunks,
                                is_active: true,
                            },
                        ))
                    }
                    None => Ok(QueryResponse::ChunkedWriteProgress(
                        ChunkedWriteProgressResponse {
                            entity_id,
                            disk_type,
                            path,
                            bytes_written: 0,
                            total_size: 0,
                            progress_percent: 0.0,
                            chunks_completed: 0,
                            total_chunks: 0,
                            is_active: false,
                        },
                    )),
                }
            }

            Query::GetResumeVerification {
                entity_id,
                disk_type,
                path,
                verify_hashes,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let result = ctx
                    .disk_service
                    .verify_resume(&entity_id, disk_type_internal, &path, verify_hashes)
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "VERIFY_RESUME_FAILED".to_string(),
                    })?;

                let total_size = result
                    .transfer_state
                    .as_ref()
                    .map(|s| s.total_size())
                    .unwrap_or(0);

                Ok(QueryResponse::ResumeVerification(
                    ResumeVerificationResponse {
                        entity_id,
                        disk_type,
                        path,
                        can_resume: result.can_resume,
                        verified_chunks: result.verified_chunks,
                        total_chunks: result.total_chunks,
                        verified_bytes: result.verified_bytes,
                        total_size,
                        failure_reason: result.failure_reason,
                        file_modified: result.file_modified,
                        verified_hash: result.verified_hash,
                    },
                ))
            }

            Query::GetResumeCapability {
                entity_id,
                disk_type,
                path,
            } => {
                let ctx = self.context.read().await;
                let disk_type_internal = disk_type_from_arg(disk_type);

                let capability = ctx
                    .disk_service
                    .get_resume_capability(&entity_id, disk_type_internal, &path)
                    .await;

                let response = match capability {
                    crate::disk_service::ResumeCapability::Full => ResumeCapabilityResponse::Full,
                    crate::disk_service::ResumeCapability::Partial => {
                        ResumeCapabilityResponse::Partial
                    }
                    crate::disk_service::ResumeCapability::None => ResumeCapabilityResponse::None,
                };

                Ok(QueryResponse::ResumeCapability(response))
            }

            Query::ListResumableTransfers => {
                let ctx = self.context.read().await;
                let transfers = ctx.disk_service.list_active_transfers().await;

                let mut responses = Vec::new();
                for transfer in transfers {
                    let capability = ctx
                        .disk_service
                        .get_resume_capability(
                            transfer.entity_id(),
                            transfer.disk_type(),
                            transfer.path(),
                        )
                        .await;

                    let resume_capability = match capability {
                        crate::disk_service::ResumeCapability::Full => {
                            ResumeCapabilityResponse::Full
                        }
                        crate::disk_service::ResumeCapability::Partial => {
                            ResumeCapabilityResponse::Partial
                        }
                        crate::disk_service::ResumeCapability::None => {
                            ResumeCapabilityResponse::None
                        }
                    };

                    let disk_type = match transfer.disk_type() {
                        crate::disk_service::DiskType::Private => DiskTypeArg::Private,
                        crate::disk_service::DiskType::Public => DiskTypeArg::Public,
                        crate::disk_service::DiskType::Shared => DiskTypeArg::Shared,
                    };

                    responses.push(ResumableTransferResponse {
                        transfer_id: transfer.transfer_id().to_string(),
                        entity_id: transfer.entity_id().to_string(),
                        disk_type,
                        path: transfer.path().to_string(),
                        bytes_written: transfer.bytes_written(),
                        total_size: transfer.total_size(),
                        chunks_completed: transfer.chunks_completed(),
                        total_chunks: transfer.total_chunks(),
                        started_at: transfer.started_at(),
                        last_updated: transfer.last_updated(),
                        resume_capability,
                    });
                }

                Ok(QueryResponse::ResumableTransfers(responses))
            }

            // ================================================================
            // Kanban Queries
            // ================================================================
            Query::GetKanbanBoard { board_id } => {
                let ctx = self.context.read().await;
                let board = ctx
                    .kanban_service
                    .get_board(&board_id)
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_BOARD_FAILED".to_string(),
                    })?;

                // Get column count via separate call
                let column_count = ctx
                    .kanban_service
                    .list_columns(&board_id)
                    .map(|cols| cols.len())
                    .unwrap_or(0);

                Ok(QueryResponse::KanbanBoard(
                    crate::command::KanbanBoardResponse {
                        id: board.id,
                        entity_id: board.project_id,
                        name: board.name,
                        description: board.description,
                        column_count,
                    },
                ))
            }

            Query::ListKanbanBoards { entity_id } => {
                let ctx = self.context.read().await;
                let boards =
                    ctx.kanban_service
                        .list_boards(&entity_id)
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("{}", e),
                            code: "LIST_BOARDS_FAILED".to_string(),
                        })?;

                let board_responses: Vec<crate::command::KanbanBoardResponse> = boards
                    .into_iter()
                    .map(|board| crate::command::KanbanBoardResponse {
                        id: board.id,
                        entity_id: board.project_id,
                        name: board.name,
                        description: board.description,
                        column_count: 0, // Column count requires additional lookup
                    })
                    .collect();

                Ok(QueryResponse::KanbanBoardList(board_responses))
            }

            Query::ListKanbanColumns { board_id } => {
                let ctx = self.context.read().await;
                let columns =
                    ctx.kanban_service
                        .list_columns(&board_id)
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("{}", e),
                            code: "LIST_COLUMNS_FAILED".to_string(),
                        })?;

                let responses: Vec<crate::command::KanbanColumnResponse> = columns
                    .into_iter()
                    .map(|column| crate::command::KanbanColumnResponse {
                        id: column.id,
                        board_id: column.board_id,
                        name: column.name,
                        position: column.position,
                        color: column.color,
                        wip_limit: column.wip_limit,
                    })
                    .collect();

                Ok(QueryResponse::KanbanColumns(responses))
            }

            Query::GetKanbanCard { board_id, card_id } => {
                let ctx = self.context.read().await;
                let card = ctx
                    .kanban_service
                    .get_card(&board_id, &card_id)
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("{}", e),
                        code: "GET_CARD_FAILED".to_string(),
                    })?;

                Ok(QueryResponse::KanbanCard(
                    crate::command::KanbanCardResponse {
                        id: card.id,
                        column_id: card.column_id,
                        title: card.title,
                        description: Some(card.description),
                        assignee: card.assignee_ids.first().cloned(),
                        position: card.position,
                    },
                ))
            }

            Query::ListKanbanCards {
                board_id,
                column_id,
                state,
                assignee_id,
                tag_id,
            } => {
                let ctx = self.context.read().await;

                // If column_id is specified, get cards from that column only
                // Otherwise, get all cards from all columns
                let mut all_cards = Vec::new();

                if let Some(col_id) = &column_id {
                    // Get cards from specific column
                    let cards = ctx
                        .kanban_service
                        .list_cards_in_column(&board_id, col_id)
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("{}", e),
                            code: "LIST_CARDS_FAILED".to_string(),
                        })?;
                    all_cards.extend(cards);
                } else {
                    // Get all columns and collect all cards
                    let columns =
                        ctx.kanban_service
                            .list_columns(&board_id)
                            .map_err(|e| QueryError {
                                query_type: query_type.clone(),
                                message: format!("{}", e),
                                code: "LIST_COLUMNS_FAILED".to_string(),
                            })?;

                    for column in columns {
                        if let Ok(cards) = ctx
                            .kanban_service
                            .list_cards_in_column(&board_id, &column.id)
                        {
                            all_cards.extend(cards);
                        }
                    }
                }

                // Apply optional filters
                if let Some(state_filter) = &state {
                    all_cards.retain(|card| format!("{:?}", card.state) == *state_filter);
                }

                if let Some(assignee) = &assignee_id {
                    all_cards.retain(|card| card.assignee_ids.contains(assignee));
                }

                if let Some(tag) = &tag_id {
                    all_cards.retain(|card| card.tag_ids.contains(tag));
                }

                let card_responses: Vec<crate::command::KanbanCardResponse> = all_cards
                    .into_iter()
                    .map(|card| crate::command::KanbanCardResponse {
                        id: card.id,
                        column_id: card.column_id,
                        title: card.title,
                        description: Some(card.description),
                        assignee: card.assignee_ids.first().cloned(),
                        position: card.position,
                    })
                    .collect();

                Ok(QueryResponse::KanbanCards(card_responses))
            }

            // ================================================================
            // Presence Queries
            // ================================================================
            Query::GetPresence { peer_id } => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let presence = gossip.presence.read().await;
                let mut status = "unknown".to_string();
                let mut last_seen = 0i64;

                if peer_id.contains('-') {
                    let index = presence_by_four_words(&presence).await;
                    if let Some(seen) = index.get(&peer_id) {
                        status = "online".to_string();
                        last_seen = *seen;
                    } else {
                        status = "offline".to_string();
                    }
                } else if let Ok(bytes) = hex::decode(&peer_id)
                    && bytes.len() == 32
                {
                    let mut arr = [0u8; 32];
                    arr.copy_from_slice(&bytes);
                    let peer = PeerId::new(arr);
                    let groups = presence.get_groups().await;
                    let mut seen_any = false;
                    let mut latest = None;

                    for topic_id in groups {
                        let records = presence.get_group_presence(topic_id).await;
                        if let Some(record) = records.get(&peer) {
                            seen_any = true;
                            if record.is_expired() {
                                continue;
                            }
                            let since = record.since as i64;
                            let should_replace =
                                latest.map(|existing| since > existing).unwrap_or(true);
                            if should_replace {
                                latest = Some(since);
                            }
                        }
                    }

                    if let Some(seen) = latest {
                        status = "online".to_string();
                        last_seen = seen;
                    } else if seen_any {
                        status = "offline".to_string();
                    }
                }

                Ok(QueryResponse::Presence(PresenceResponse {
                    peer_id,
                    status,
                    last_seen,
                }))
            }

            Query::ListOnlinePeers => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let presence = gossip.presence.read().await;
                let groups = presence.get_groups().await;
                let mut peers = HashSet::new();

                for topic_id in groups {
                    for peer_id in presence.get_online_peers(topic_id).await {
                        peers.insert(peer_id);
                    }
                }

                let peer_list = peers
                    .into_iter()
                    .map(|peer_id| hex::encode(peer_id.as_bytes()))
                    .collect();

                Ok(QueryResponse::PeerList(peer_list))
            }

            Query::GetOurPresenceRecord => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let (public_key, private_key) =
                    gossip.get_sites_signing_keys().map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("Failed to load identity keypair: {e}"),
                        code: "IDENTITY_KEYPAIR_FAILED".to_string(),
                    })?;

                let pubkey_bytes = public_key.clone().into_bytes().to_vec();
                if let Some(cached) = self.presence_cache.read().await.get(&pubkey_bytes) {
                    return Ok(QueryResponse::OurPresenceRecord(Some(
                        cached.record.clone(),
                    )));
                }

                let connection_words = ctx.external_address.or(ctx.listen_address).map(|addr| {
                    crate::identity::conn_words(&addr).unwrap_or_else(|_| "unknown".to_string())
                });

                match connection_words {
                    Some(words) => {
                        let record = crate::peer_presence::PresenceRecord::new(
                            &public_key,
                            &private_key,
                            words.clone(),
                        )
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("Failed to sign presence record: {e}"),
                            code: "PRESENCE_SIGN_FAILED".to_string(),
                        })?;

                        {
                            let mut cache = self.presence_cache.write().await;
                            cache.insert(record.clone());
                        }

                        Ok(QueryResponse::OurPresenceRecord(Some(record)))
                    }
                    None => Ok(QueryResponse::OurPresenceRecord(None)),
                }
            }

            Query::GetCachedPeerPresence { pubkey } => {
                let cache = self.presence_cache.read().await;
                let record = cache.get(&pubkey).map(|cp| cp.record.clone());
                Ok(QueryResponse::CachedPeerPresence(record))
            }

            // ================================================================
            // WebRTC Queries
            // ================================================================
            Query::ListActiveCalls => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "WebRTC not initialized".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                let calls = webrtc.list_active_calls().await;
                let responses = calls
                    .into_iter()
                    .map(|call| {
                        let started_at = call
                            .started_at
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        CallResponse {
                            id: call.call_id.to_string(),
                            entity_id: String::new(),
                            participant_count: 2,
                            started_at,
                        }
                    })
                    .collect();

                Ok(QueryResponse::CallList(responses))
            }

            Query::GetCallParticipants { call_id } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "WebRTC not initialized".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                let uuid = uuid::Uuid::parse_str(&call_id).map_err(|e| QueryError {
                    query_type: query_type.clone(),
                    message: format!("Invalid call id: {e}"),
                    code: "INVALID_CALL_ID".to_string(),
                })?;

                let participants = webrtc
                    .get_call_participants(crate::webrtc::CallId(uuid))
                    .await
                    .map_err(|e| QueryError {
                        query_type: query_type.clone(),
                        message: format!("Failed to get participants: {e}"),
                        code: "CALL_PARTICIPANTS_FAILED".to_string(),
                    })?;

                let list = participants
                    .into_iter()
                    .map(|identity| identity.four_words)
                    .collect();

                Ok(QueryResponse::CallParticipants(list))
            }

            Query::GetCallStatus { call_id } => {
                let ctx = self.context.read().await;
                let webrtc = ctx.webrtc.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "WebRTC not initialized".to_string(),
                    code: "WEBRTC_NOT_AVAILABLE".to_string(),
                })?;

                let uuid = uuid::Uuid::parse_str(&call_id).map_err(|e| QueryError {
                    query_type: query_type.clone(),
                    message: format!("Invalid call id: {e}"),
                    code: "INVALID_CALL_ID".to_string(),
                })?;

                let call_id_typed = crate::webrtc::CallId(uuid);

                let participants =
                    webrtc
                        .get_call_participants(call_id_typed)
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("Failed to get call status: {e}"),
                            code: "CALL_STATUS_FAILED".to_string(),
                        })?;

                // Get call state - WebRTC manager tracks mute/video state internally
                // For now, return default state; real implementation would query WebRTC
                Ok(QueryResponse::CallStatus(CallStatusResponse {
                    call_id,
                    entity_id: String::new(), // Would be retrieved from call info
                    participant_count: participants.len(),
                    started_at: 0, // Would be retrieved from call info
                    is_muted: false,
                    is_video_enabled: false,
                    is_screen_sharing: false,
                }))
            }

            // ================================================================
            // Contact Queries
            // ================================================================
            Query::GetContact { contact_id } => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                // Try to get by id
                let contact = gossip
                    .contact_store
                    .get_by_id(&contact_id)
                    .await
                    .ok_or_else(|| QueryError {
                        query_type: query_type.clone(),
                        message: format!("Contact not found: {}", contact_id),
                        code: "CONTACT_NOT_FOUND".to_string(),
                    })?;

                let presence_index = {
                    let presence = gossip.presence.read().await;
                    presence_by_four_words(&presence).await
                };

                let (is_online, last_seen) = contact
                    .four_words
                    .as_ref()
                    .and_then(|four_words| presence_index.get(four_words).copied())
                    .map(|seen| (true, Some(seen)))
                    .unwrap_or((false, contact.last_online_at.map(|t| t as i64)));

                Ok(QueryResponse::Contact(ContactResponse {
                    id: contact.id.clone(),
                    display_name: contact.effective_name(),
                    four_words: contact.four_words.clone(),
                    is_favourite: contact.is_favourite,
                    is_online,
                    created_at: contact.created_at as i64,
                    last_seen,
                }))
            }

            Query::ListContacts => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let contacts = gossip.contact_store.all().await;
                let presence_index = {
                    let presence = gossip.presence.read().await;
                    presence_by_four_words(&presence).await
                };

                let responses: Vec<ContactResponse> = contacts
                    .into_iter()
                    .map(|contact| {
                        let (is_online, last_seen) = contact
                            .four_words
                            .as_ref()
                            .and_then(|four_words| presence_index.get(four_words).copied())
                            .map(|seen| (true, Some(seen)))
                            .unwrap_or((false, contact.last_online_at.map(|t| t as i64)));

                        ContactResponse {
                            id: contact.id.clone(),
                            display_name: contact.effective_name(),
                            four_words: contact.four_words.clone(),
                            is_favourite: contact.is_favourite,
                            is_online,
                            created_at: contact.created_at as i64,
                            last_seen,
                        }
                    })
                    .collect();

                Ok(QueryResponse::ContactList(responses))
            }

            Query::ListFavouriteContacts => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                let favourites = gossip.contact_store.favourites().await;
                let presence_index = {
                    let presence = gossip.presence.read().await;
                    presence_by_four_words(&presence).await
                };

                let responses: Vec<ContactResponse> = favourites
                    .into_iter()
                    .map(|contact| {
                        let (is_online, last_seen) = contact
                            .four_words
                            .as_ref()
                            .and_then(|four_words| presence_index.get(four_words).copied())
                            .map(|seen| (true, Some(seen)))
                            .unwrap_or((false, contact.last_online_at.map(|t| t as i64)));

                        ContactResponse {
                            id: contact.id.clone(),
                            display_name: contact.effective_name(),
                            four_words: contact.four_words.clone(),
                            is_favourite: true,
                            is_online,
                            created_at: contact.created_at as i64,
                            last_seen,
                        }
                    })
                    .collect();

                Ok(QueryResponse::ContactList(responses))
            }

            Query::SearchContacts { query } => {
                let ctx = self.context.read().await;
                let gossip = ctx.gossip.as_ref().ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Networking not started".to_string(),
                    code: "GOSSIP_NOT_AVAILABLE".to_string(),
                })?;

                // Simple search - filter all contacts by display name or four_words
                let all_contacts = gossip.contact_store.all().await;
                let presence_index = {
                    let presence = gossip.presence.read().await;
                    presence_by_four_words(&presence).await
                };
                let query_lower = query.to_lowercase();

                let results: Vec<ContactResponse> = all_contacts
                    .into_iter()
                    .filter(|c| {
                        c.effective_name().to_lowercase().contains(&query_lower)
                            || c.four_words
                                .as_ref()
                                .is_some_and(|fw| fw.to_lowercase().contains(&query_lower))
                    })
                    .map(|contact| {
                        let (is_online, last_seen) = contact
                            .four_words
                            .as_ref()
                            .and_then(|four_words| presence_index.get(four_words).copied())
                            .map(|seen| (true, Some(seen)))
                            .unwrap_or((false, contact.last_online_at.map(|t| t as i64)));

                        ContactResponse {
                            id: contact.id.clone(),
                            display_name: contact.effective_name(),
                            four_words: contact.four_words.clone(),
                            is_favourite: contact.is_favourite,
                            is_online,
                            created_at: contact.created_at as i64,
                            last_seen,
                        }
                    })
                    .collect();

                Ok(QueryResponse::ContactList(results))
            }

            Query::GetWebsite { entity_id } => {
                let ctx = self.context.read().await;

                // Construct the document ID for this website
                let doc_id = format!("website:{entity_id}");

                // Load the website CRDT document
                let doc =
                    ctx.crdt_manager
                        .load_document(&doc_id)
                        .await
                        .map_err(|e| QueryError {
                            query_type: query_type.clone(),
                            message: format!("Website not found: {e}"),
                            code: "WEBSITE_NOT_FOUND".to_string(),
                        })?;

                // Extract website content from the CRDT document
                // Note: CreateWebsite stores in "website" map, not "root"
                let txn = doc.transact();
                let root = txn.get_map("website").ok_or_else(|| QueryError {
                    query_type: query_type.clone(),
                    message: "Invalid website document structure".to_string(),
                    code: "INVALID_STRUCTURE".to_string(),
                })?;

                let html = match root.get(&txn, "html") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                let css = match root.get(&txn, "css") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                let js = match root.get(&txn, "js") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                // Note: CreateWebsite stores as "hash", not "website_root_hash"
                let website_root_hash = match root.get(&txn, "hash") {
                    Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                    _ => String::new(),
                };
                let published_at = root
                    .get(&txn, "published_at")
                    .and_then(|v| match v {
                        yrs::Out::Any(yrs::Any::BigInt(n)) => Some(n),
                        _ => None,
                    })
                    .unwrap_or(0);
                let size_bytes = root
                    .get(&txn, "size_bytes")
                    .and_then(|v| match v {
                        yrs::Out::Any(yrs::Any::BigInt(n)) => Some(n as usize),
                        _ => None,
                    })
                    .unwrap_or(0);

                // Generate URL from entity_id
                let url = format!("communitas://{entity_id}/website");

                Ok(QueryResponse::Website(WebsiteResponse {
                    entity_id,
                    html,
                    css,
                    js,
                    website_root_hash,
                    published_at,
                    size_bytes,
                    url,
                }))
            }

            // Canvas Queries - stub implementations for MCP parity
            Query::GetCanvasSnapshot { entity_id } => {
                // Return an empty canvas snapshot for now
                // TODO: Implement full canvas state management
                Ok(QueryResponse::CanvasSnapshot(CanvasSnapshotResponse {
                    entity_id,
                    elements: vec![],
                    viewport_width: 800.0,
                    viewport_height: 600.0,
                    zoom: 1.0,
                    pan_x: 0.0,
                    pan_y: 0.0,
                    loading: false,
                }))
            }

            Query::CanvasExport { entity_id } => {
                // Return empty canvas JSON for now
                let export = serde_json::json!({
                    "entity_id": entity_id,
                    "elements": [],
                    "viewport": {"width": 800.0, "height": 600.0},
                    "view": {"zoom": 1.0, "pan_x": 0.0, "pan_y": 0.0}
                });
                Ok(QueryResponse::CanvasExportJson(export.to_string()))
            }

            Query::CanvasElementAt {
                entity_id: _,
                x: _,
                y: _,
            } => {
                // Return None for now - no element at position
                // TODO: Implement hit testing
                Ok(QueryResponse::CanvasElement(None))
            }
        }
    }

    /// Subscribe to events
    ///
    /// Returns a receiver that will receive events matching the subscription.
    /// Multiple subscriptions can be active simultaneously.
    ///
    /// # Arguments
    /// * `subscription` - Type of events to subscribe to
    ///
    /// # Returns
    /// Event receiver
    pub fn subscribe(&self, _subscription: Subscription) -> broadcast::Receiver<Event> {
        // For now, all subscriptions receive all events
        // A more sophisticated implementation would filter based on subscription type
        self.event_sender.subscribe()
    }

    /// Subscribe with a subscriber ID for tracking
    ///
    /// # Arguments
    /// * `subscriber_id` - Unique identifier for this subscriber
    /// * `subscriptions` - List of subscription types
    ///
    /// # Returns
    /// Event receiver
    pub async fn subscribe_with_id(
        &self,
        subscriber_id: String,
        subscriptions: Vec<Subscription>,
    ) -> broadcast::Receiver<Event> {
        let mut subs = self.subscriptions.write().await;
        subs.insert(subscriber_id, subscriptions);
        self.event_sender.subscribe()
    }

    /// Unsubscribe a subscriber
    ///
    /// # Arguments
    /// * `subscriber_id` - ID of subscriber to remove
    pub async fn unsubscribe(&self, subscriber_id: &str) {
        let mut subs = self.subscriptions.write().await;
        subs.remove(subscriber_id);
    }

    /// Get the underlying CoreContext (for advanced use cases)
    ///
    /// Most adapters should use execute/query/subscribe instead.
    pub fn context(&self) -> Arc<RwLock<CoreContext>> {
        self.context.clone()
    }

    /// Broadcast an event to all subscribers
    fn broadcast_event(&self, event: Event) {
        if let Err(e) = self.event_sender.send(event.clone()) {
            // No receivers - this is normal during startup
            warn!("No event receivers: {}", e);
        } else {
            info!("Broadcast event: {:?}", std::mem::discriminant(&event));
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert DiskTypeArg to internal DiskType
fn disk_type_from_arg(arg: DiskTypeArg) -> DiskType {
    match arg {
        DiskTypeArg::Private => DiskType::Private,
        DiskTypeArg::Public => DiskType::Public,
        DiskTypeArg::Shared => DiskType::Shared,
    }
}

/// Convert Entity to EntityResponse
fn entity_to_response(entity: &crate::entity_service::Entity) -> EntityResponse {
    EntityResponse {
        id: entity.id.clone(),
        name: entity.name.clone(),
        entity_type: entity.entity_type,
        description: entity.description.clone(),
        created_by: entity.created_by.clone(),
        created_at: entity.created_at,
        member_count: entity.members.len(),
        parent_org_id: entity.parent_org_id.clone(),
        network_four_words: entity.network_four_words.clone(),
        is_local_only: entity.is_local_only,
    }
}

/// Convert Invite to InviteResponse
fn invite_to_response(invite: &crate::invite::Invite) -> InviteResponse {
    InviteResponse {
        id: invite.id.clone(),
        sender_id: invite.creator_id.clone(),
        recipient_id: invite.recipient_id.clone(),
        entity_type: invite.entity_type,
        entity_id: invite.entity_id.clone(),
        role: invite.role.clone(),
        status: invite.status,
        message: invite.message.clone(),
        created_at: invite.created_at,
        expires_at: invite.expires_at,
    }
}

fn map_message_response(message: crate::crdt::CRDTMessage) -> MessageResponse {
    let reactions = message
        .local_state
        .as_ref()
        .map(|ls| {
            ls.reactions
                .iter()
                .map(|r| ReactionResponse {
                    emoji: r.emoji.clone(),
                    count: r.count,
                    user_reacted: r.user_reacted.unwrap_or(false),
                    peer_ids: r.peer_ids.clone(),
                })
                .collect()
        })
        .unwrap_or_default();

    let edited_at = message.local_state.as_ref().and_then(|ls| ls.edited_at);

    MessageResponse {
        id: message.metadata.id,
        entity_id: message.metadata.entity_id,
        author: message.metadata.author_peer_id,
        text: message.content.text,
        timestamp: message.metadata.timestamp as i64,
        reply_to_id: message.metadata.reply_to_id,
        reactions,
        edited_at,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_app_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();

        let app = CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            storage_dir,
        )
        .await;

        assert!(app.is_ok());
    }

    #[tokio::test]
    async fn test_get_profile_query() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();

        let app = CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            storage_dir,
        )
        .await
        .unwrap();

        let response = app.query(Query::GetProfile).await.unwrap();

        match response {
            QueryResponse::Profile {
                four_words,
                display_name,
                ..
            } => {
                assert_eq!(four_words, "ocean-forest-moon-star");
                assert_eq!(display_name, "Test User");
            }
            _ => panic!("Unexpected response type"),
        }
    }

    #[tokio::test]
    async fn test_update_display_name() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();

        let app = CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "Old Name".to_string(),
            "Test Device".to_string(),
            storage_dir,
        )
        .await
        .unwrap();

        let events = app
            .execute(Command::UpdateDisplayName {
                display_name: "New Name".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::DisplayNameUpdated { old_name, new_name } => {
                assert_eq!(old_name, "Old Name");
                assert_eq!(new_name, "New Name");
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_create_entity() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();

        let app = CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            storage_dir,
        )
        .await
        .unwrap();

        let events = app
            .execute(Command::CreateEntity {
                name: "Test Org".to_string(),
                entity_type: EntityType::Organisation,
                description: Some("A test organization".to_string()),
                initial_members: vec![],
            })
            .await
            .unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            Event::EntityCreated {
                name, entity_type, ..
            } => {
                assert_eq!(name, "Test Org");
                assert_eq!(*entity_type, EntityType::Organisation);
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_subscribe_receives_events() {
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path().to_str().unwrap().to_string();

        let app = CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            storage_dir,
        )
        .await
        .unwrap();

        // Subscribe before executing command
        let mut receiver = app.subscribe(Subscription::AllEvents);

        // Execute a command
        app.execute(Command::UpdateDisplayName {
            display_name: "New Name".to_string(),
        })
        .await
        .unwrap();

        // Should receive the event
        let event = receiver.try_recv().unwrap();
        match event {
            Event::DisplayNameUpdated { new_name, .. } => {
                assert_eq!(new_name, "New Name");
            }
            _ => panic!("Unexpected event type"),
        }
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Entity Service - Unified entity and member management
//!
//! This service provides a unified interface for managing entities (groups, channels,
//! projects, organizations) and their members. It consolidates functionality from
//! both desktop (CRDT-based) and TUI (JSON file-based) implementations.
//!
//! **Key Features:**
//! - CRDT-based member management for offline-first collaboration
//! - Entity creation, listing, and metadata management
//! - Member addition/removal with tombstone support
//! - Unified API for desktop and TUI applications

use crate::CrdtManager;
use crate::crdt::EntityType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::{Map, Transact, WriteTxn};

/// Get current Unix timestamp in seconds (never panics, falls back to 0)
fn unix_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Get current Unix timestamp in seconds, returning an error on failure
fn unix_timestamp_result() -> EntityServiceResult<i64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|e| EntityServiceError::Io(std::io::Error::other(format!("Time error: {}", e))))
}

/// Entity information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub members: Vec<String>, // Four-word addresses of members
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_org_id: Option<String>, // Links child entities (channel/group/project) to parent organization
    /// Network four-word identity if this entity is linked to the P2P network
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_four_words: Option<String>,
    /// True if this is local-only (no network identity yet)
    #[serde(default)]
    pub is_local_only: bool,
    /// Timestamp when entity was linked to a network identity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_at: Option<i64>,
    /// Timestamp of last successful sync with network peer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<i64>,
}

impl Entity {
    /// Create a new local-only entity without a network identity
    pub fn new_local(
        name: String,
        entity_type: EntityType,
        description: Option<String>,
        created_by: String,
    ) -> Self {
        let now = unix_timestamp();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            entity_type,
            description,
            created_by,
            created_at: now,
            members: vec![],
            parent_org_id: None,
            network_four_words: None,
            is_local_only: true,
            linked_at: None,
            last_sync_at: None,
        }
    }

    /// Check if this entity is linked to a network identity
    pub fn is_linked(&self) -> bool {
        self.network_four_words.is_some() && !self.is_local_only
    }

    /// Link this entity to a network identity
    pub fn link_to_network(&mut self, four_words: String) {
        self.network_four_words = Some(four_words);
        self.is_local_only = false;
        self.linked_at = Some(unix_timestamp());
    }

    /// Update the last sync timestamp
    pub fn mark_synced(&mut self) {
        self.last_sync_at = Some(unix_timestamp());
    }
}

/// Member information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberInfo {
    pub member_id: String,
    pub role: String,
    pub joined_at: i64,
    pub deleted: bool,
}

/// Entity service errors
#[derive(Debug, thiserror::Error)]
pub enum EntityServiceError {
    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Entity already exists: {0}")]
    AlreadyExists(String),

    #[error("Member not found: {0}")]
    MemberNotFound(String),

    #[error("Member already exists: {0}")]
    MemberAlreadyExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("CRDT error: {0}")]
    Crdt(#[from] crate::crdt_manager::CrdtError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for entity service operations
pub type EntityServiceResult<T> = Result<T, EntityServiceError>;

/// Result of cascading member removal operation
#[derive(Debug, Clone)]
pub struct CascadeRemovalResult {
    pub removed_in: Vec<(EntityType, String)>,
    pub skipped_not_member: Vec<(EntityType, String)>,
    pub failed: Vec<(EntityType, String, String)>,
}

/// Unified entity and member management service
pub struct EntityService {
    crdt_manager: Arc<CrdtManager>,
    member_write_lock: tokio::sync::Mutex<()>,
}

impl EntityService {
    /// Create a new entity service
    pub fn new(crdt_manager: Arc<CrdtManager>) -> Self {
        Self {
            crdt_manager,
            member_write_lock: tokio::sync::Mutex::new(()),
        }
    }

    /// Create a new entity
    pub async fn create_entity(
        &self,
        name: String,
        entity_type: EntityType,
        description: Option<String>,
        created_by: String,
        initial_members: Vec<String>,
    ) -> EntityServiceResult<Entity> {
        let entity_id = Uuid::new_v4().to_string();
        let now = unix_timestamp_result()?;

        let entity = Entity {
            id: entity_id.clone(),
            name,
            entity_type,
            description,
            created_by: created_by.clone(),
            created_at: now,
            members: initial_members.clone(),
            parent_org_id: None,
            // Default to network-linked for backward compatibility
            network_four_words: None,
            is_local_only: false,
            linked_at: None,
            last_sync_at: None,
        };

        // Save entity metadata
        self.save_entity(&entity).await?;

        // Add initial members (including creator)
        let mut all_members = initial_members;
        if !all_members.contains(&created_by) {
            all_members.push(created_by.clone());
        }

        for member_id in &all_members {
            // Creator gets "owner" role, others get "member"
            let role = if member_id == &created_by {
                "owner"
            } else {
                "member"
            };
            self.add_member(entity_type, &entity_id, member_id, role)
                .await?;
        }

        // Update entity with actual members list (including creator)
        let entity_with_members = Entity {
            members: all_members,
            ..entity
        };

        Ok(entity_with_members)
    }

    /// Get entity by ID
    pub async fn get_entity(&self, entity_id: &str) -> EntityServiceResult<Entity> {
        // Load entity metadata from CRDT
        let doc_id = format!("entity:{}:metadata", entity_id);
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        // Extract entity data from CRDT document
        let metadata_map = doc.get_or_insert_map("metadata");
        let txn = doc.transact();

        let name = CrdtManager::get_map_string(&metadata_map, &txn, "name")
            .unwrap_or_else(|| "Unknown".to_string());

        let entity_type_str = CrdtManager::get_map_string(&metadata_map, &txn, "entity_type")
            .unwrap_or_else(|| "group".to_string());

        let entity_type = match entity_type_str.as_str() {
            "group" => EntityType::Group,
            "channel" => EntityType::Channel,
            "project" => EntityType::Project,
            "organisation" => EntityType::Organisation,
            _ => EntityType::Group,
        };

        let description = CrdtManager::get_map_string(&metadata_map, &txn, "description");

        let created_by = CrdtManager::get_map_string(&metadata_map, &txn, "created_by")
            .unwrap_or_else(|| "unknown".to_string());

        let created_at = CrdtManager::get_map_i64(&metadata_map, &txn, "created_at").unwrap_or(0);

        let parent_org_id = CrdtManager::get_map_string(&metadata_map, &txn, "parent_org_id");

        // Read new local-only/network-linked fields
        let network_four_words =
            CrdtManager::get_map_string(&metadata_map, &txn, "network_four_words");
        let is_local_only =
            CrdtManager::get_map_bool(&metadata_map, &txn, "is_local_only").unwrap_or(false);
        let linked_at = CrdtManager::get_map_i64(&metadata_map, &txn, "linked_at");
        let last_sync_at = CrdtManager::get_map_i64(&metadata_map, &txn, "last_sync_at");

        // Get members list
        let members = self
            .list_members(entity_type, entity_id)
            .await?
            .into_iter()
            .filter(|m| !m.deleted)
            .map(|m| m.member_id)
            .collect();

        Ok(Entity {
            id: entity_id.to_string(),
            name,
            entity_type,
            description,
            created_by,
            created_at,
            members,
            parent_org_id,
            network_four_words,
            is_local_only,
            linked_at,
            last_sync_at,
        })
    }

    /// List all entities
    pub async fn list_entities(&self) -> EntityServiceResult<Vec<Entity>> {
        use std::fs;

        // Scan the entity directory for all metadata files
        let entity_dir = self
            .crdt_manager
            .get_storage_dir()
            .join("crdt")
            .join("entity");

        // Perform blocking filesystem scan in dedicated blocking thread pool
        let entity_ids =
            tokio::task::spawn_blocking(move || -> Result<Vec<String>, EntityServiceError> {
                if !entity_dir.exists() {
                    return Ok(vec![]);
                }

                let mut ids = Vec::new();

                // Read all .meta files in the entity directory
                let entries = fs::read_dir(&entity_dir).map_err(|e| {
                    EntityServiceError::Io(std::io::Error::other(format!(
                        "Failed to read entity directory: {}",
                        e
                    )))
                })?;

                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();

                    // Only process .meta files
                    if path.extension().and_then(|s| s.to_str()) != Some("meta") {
                        continue;
                    }

                    // Check if this is an entity metadata file (contains "entity:" in the hex-decoded filename)
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        // Hex-decode the filename to get the doc_id
                        if let Ok(decoded_bytes) = hex::decode(filename)
                            && let Ok(doc_id) = String::from_utf8(decoded_bytes)
                        {
                            // Check if it's an entity metadata document (format: "entity:{id}:metadata")
                            if doc_id.starts_with("entity:") && doc_id.ends_with(":metadata") {
                                // Extract entity ID from "entity:{id}:metadata"
                                if let Some(entity_id) = doc_id
                                    .strip_prefix("entity:")
                                    .and_then(|s| s.strip_suffix(":metadata"))
                                {
                                    ids.push(entity_id.to_string());
                                }
                            }
                        }
                    }
                }

                Ok(ids)
            })
            .await
            .map_err(|e| {
                EntityServiceError::Io(std::io::Error::other(format!(
                    "Blocking task failed: {}",
                    e
                )))
            })??;

        // Load entities using async operations (outside blocking section)
        let mut entities = Vec::new();
        for entity_id in entity_ids {
            match self.get_entity(&entity_id).await {
                Ok(entity) => entities.push(entity),
                Err(e) => {
                    // Log error but continue processing other entities
                    tracing::warn!("Failed to load entity {}: {}", entity_id, e);
                }
            }
        }

        Ok(entities)
    }

    /// Add member to entity
    pub async fn add_member(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        role: &str,
    ) -> EntityServiceResult<()> {
        use yrs::Doc;

        let _member_guard = self.member_write_lock.lock().await;
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);
        let now_ts = unix_timestamp_result()?;

        // Get or create the document
        let doc = match self.crdt_manager.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(_) => {
                // Document doesn't exist, create new one
                Doc::new()
            }
        };

        // Check if member already exists and is active
        let members_map = doc.get_or_insert_map("members");
        let mut existing_role: Option<String> = None;
        let mut existing_active = false;
        {
            let txn = doc.transact();

            if let Some(member_data) = CrdtManager::get_nested_map(&members_map, &txn, member_id) {
                let is_deleted =
                    CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);
                if !is_deleted {
                    existing_role = CrdtManager::get_map_string(&member_data, &txn, "role");
                    existing_active = true;
                }
            }
        }

        if existing_active {
            // Idempotent add: refresh role + active flag when member already exists.
            let active_members_map = doc.get_or_insert_map("active_members");
            {
                let mut txn = doc.transact_mut();
                let member_data =
                    CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

                if existing_role.as_deref() != Some(role) {
                    CrdtManager::set_map_string(&member_data, &mut txn, "role", role);
                }
                CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", false);
                CrdtManager::set_map_i64(&member_data, &mut txn, "updated_at", now_ts);
                CrdtManager::set_map_bool(&active_members_map, &mut txn, member_id, true);
            }

            self.crdt_manager
                .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
                .await?;

            return Ok(());
        }

        // Add new member
        let active_members_map = doc.get_or_insert_map("active_members");
        {
            let mut txn = doc.transact_mut();

            let member_data =
                CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

            CrdtManager::set_map_string(&member_data, &mut txn, "member_id", member_id);
            CrdtManager::set_map_string(&member_data, &mut txn, "role", role);
            CrdtManager::set_map_i64(&member_data, &mut txn, "joined_at", now_ts);
            CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", false);
            CrdtManager::set_map_i64(&member_data, &mut txn, "updated_at", now_ts);

            CrdtManager::set_map_bool(&active_members_map, &mut txn, member_id, true);
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Remove member from entity
    pub async fn remove_member(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        deleted_by: &str,
    ) -> EntityServiceResult<()> {
        let _member_guard = self.member_write_lock.lock().await;
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);
        let now_ts = unix_timestamp_result()?;

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        // Get maps before transactions
        let members_map = doc.get_or_insert_map("members");
        let active_members_map = doc.get_or_insert_map("active_members");

        // Check permissions before requiring the target to exist.
        {
            let txn = doc.transact();

            let deleted_by_data = CrdtManager::get_nested_map(&members_map, &txn, deleted_by);
            let mut deleted_by_role =
                deleted_by_data.and_then(|data| CrdtManager::get_map_string(&data, &txn, "role"));
            let target_data = CrdtManager::get_nested_map(&members_map, &txn, member_id);
            let target_role =
                target_data.and_then(|data| CrdtManager::get_map_string(&data, &txn, "role"));

            let needs_role_fallback =
                !matches!(deleted_by_role.as_deref(), Some("owner") | Some("admin"));

            if needs_role_fallback {
                let normalize_id = |id: &str| id.replace('.', "-");
                let normalized_actor = normalize_id(deleted_by);
                for (member_key, _) in members_map.iter(&txn) {
                    let member_key = member_key.to_string();
                    if normalize_id(&member_key) != normalized_actor {
                        continue;
                    }
                    if let Some(member_data) =
                        CrdtManager::get_nested_map(&members_map, &txn, &member_key)
                    {
                        let candidate_role =
                            CrdtManager::get_map_string(&member_data, &txn, "role");
                        match candidate_role.as_deref() {
                            Some("owner") | Some("admin") => {
                                deleted_by_role = candidate_role;
                                break;
                            }
                            Some(_) => {
                                if deleted_by_role.is_none() {
                                    deleted_by_role = candidate_role;
                                }
                            }
                            None => {}
                        }
                    }
                }
            }

            if deleted_by != member_id {
                let deleted_by_role = match deleted_by_role.as_deref() {
                    Some(role) => role,
                    None => {
                        return Err(EntityServiceError::PermissionDenied(
                            "permission denied: actor not a member".to_string(),
                        ));
                    }
                };
                let mut allowed = match deleted_by_role {
                    "owner" => true,
                    "admin" => matches!(
                        target_role.as_deref(),
                        Some("member") | Some("viewer") | None
                    ),
                    _ => false,
                };

                if !allowed {
                    let relax_member_removal = matches!(
                        std::env::var("COMMUNITAS_RELAX_MEMBER_REMOVAL")
                            .unwrap_or_default()
                            .trim()
                            .to_ascii_lowercase()
                            .as_str(),
                        "1" | "true" | "yes"
                    );
                    if relax_member_removal
                        && deleted_by_role == "member"
                        && matches!(target_role.as_deref(), Some("member") | Some("viewer"))
                    {
                        tracing::warn!(
                            "Relaxed member removal enabled: allowing member {} to remove {}",
                            deleted_by,
                            member_id
                        );
                        allowed = true;
                    }
                }

                if !allowed {
                    tracing::warn!(
                        "Permission denied removing member: actor_role={:?} target_role={:?} actor={} target={}",
                        deleted_by_role,
                        target_role,
                        deleted_by,
                        member_id
                    );
                    return Err(EntityServiceError::PermissionDenied(format!(
                        "permission denied: {deleted_by} cannot remove {member_id}"
                    )));
                }
            }
        }

        // Mark member as deleted (tombstone)
        {
            let mut txn = doc.transact_mut();

            let member_data =
                CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

            CrdtManager::set_map_string(&member_data, &mut txn, "member_id", member_id);
            CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", true);
            CrdtManager::set_map_i64(&member_data, &mut txn, "deleted_at", now_ts);
            CrdtManager::set_map_string(&member_data, &mut txn, "deleted_by", deleted_by);
            CrdtManager::set_map_i64(&member_data, &mut txn, "updated_at", now_ts);

            active_members_map.remove(&mut txn, member_id);
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Apply a remote membership update without local permission checks.
    #[allow(clippy::too_many_arguments)]
    pub async fn apply_member_update(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        role: Option<&str>,
        action: crate::crdt::MemberUpdateAction,
        updated_by: &str,
        timestamp: u64,
    ) -> EntityServiceResult<()> {
        use yrs::Doc;

        let _member_guard = self.member_write_lock.lock().await;
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);
        let doc = match self.crdt_manager.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(_) => Doc::new(),
        };

        let members_map = doc.get_or_insert_map("members");
        let active_members_map = doc.get_or_insert_map("active_members");
        let update_ts = timestamp as i64;

        let role_rank = |role: &str| match role {
            "owner" => 3,
            "admin" => 2,
            "member" => 1,
            "viewer" => 0,
            _ => 0,
        };

        match action {
            crate::crdt::MemberUpdateAction::Add => {
                let mut txn = doc.transact_mut();
                let member_data =
                    CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);
                let existing_ts =
                    CrdtManager::get_map_i64(&member_data, &txn, "updated_at").unwrap_or(0);
                let existing_deleted =
                    CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);
                let existing_role = CrdtManager::get_map_string(&member_data, &txn, "role")
                    .unwrap_or_else(|| "member".to_string());
                let incoming_role = role.unwrap_or("member");
                let incoming_rank = role_rank(incoming_role);
                let existing_rank = role_rank(&existing_role);

                if update_ts > existing_ts && incoming_rank < existing_rank {
                    let resolve_role = |target_id: &str| -> Option<String> {
                        if let Some(data) =
                            CrdtManager::get_nested_map(&members_map, &txn, target_id)
                            && let Some(role) = CrdtManager::get_map_string(&data, &txn, "role")
                        {
                            return Some(role);
                        }

                        let normalize_id = |id: &str| id.replace('.', "-");
                        let normalized_target = normalize_id(target_id);
                        for (member_key, _) in members_map.iter(&txn) {
                            let member_key = member_key.to_string();
                            if normalize_id(&member_key) != normalized_target {
                                continue;
                            }
                            if let Some(member_data) =
                                CrdtManager::get_nested_map(&members_map, &txn, &member_key)
                                && let Some(role) =
                                    CrdtManager::get_map_string(&member_data, &txn, "role")
                            {
                                return Some(role);
                            }
                        }

                        None
                    };

                    let updated_by_role = resolve_role(updated_by);
                    let allowed = match updated_by_role.as_deref() {
                        Some("owner") => true,
                        Some("admin") => matches!(existing_role.as_str(), "member" | "viewer" | ""),
                        _ => false,
                    };

                    if !allowed {
                        tracing::debug!(
                            entity_id,
                            member_id,
                            existing_ts,
                            update_ts,
                            existing_role,
                            incoming_role,
                            updated_by,
                            updated_by_role = ?updated_by_role,
                            "Skipping member update: unauthorized role downgrade"
                        );
                        return Ok(());
                    }
                }

                let mut effective_ts = update_ts;

                if existing_ts > update_ts {
                    if existing_deleted {
                        tracing::debug!(
                            entity_id,
                            member_id,
                            existing_ts,
                            update_ts,
                            "Skipping member update: newer tombstone present"
                        );
                        return Ok(());
                    }
                    // Allow role upgrades even if the incoming update is older.
                    // This mitigates out-of-order updates where a later admin promotion
                    // gets a lower timestamp than a prior member add on another node.
                    if incoming_rank > existing_rank {
                        tracing::debug!(
                            entity_id,
                            member_id,
                            existing_ts,
                            update_ts,
                            existing_role,
                            incoming_role,
                            "Applying role upgrade despite older timestamp"
                        );
                        effective_ts = existing_ts;
                    } else {
                        tracing::debug!(
                            entity_id,
                            member_id,
                            existing_ts,
                            update_ts,
                            existing_role,
                            incoming_role,
                            "Skipping member update: older timestamp"
                        );
                        return Ok(());
                    }
                } else if existing_ts == update_ts {
                    if existing_deleted {
                        tracing::debug!(
                            entity_id,
                            member_id,
                            existing_ts,
                            update_ts,
                            "Skipping member update: same timestamp tombstone"
                        );
                        return Ok(());
                    }
                    if incoming_rank <= existing_rank {
                        tracing::debug!(
                            entity_id,
                            member_id,
                            existing_ts,
                            update_ts,
                            existing_role,
                            incoming_role,
                            "Skipping member update: same timestamp lower role"
                        );
                        return Ok(());
                    }
                }

                CrdtManager::set_map_string(&member_data, &mut txn, "member_id", member_id);
                CrdtManager::set_map_string(
                    &member_data,
                    &mut txn,
                    "role",
                    role.unwrap_or("member"),
                );
                CrdtManager::set_map_i64(&member_data, &mut txn, "joined_at", effective_ts);
                CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", false);
                CrdtManager::set_map_i64(&member_data, &mut txn, "updated_at", effective_ts);
                CrdtManager::set_map_bool(&active_members_map, &mut txn, member_id, true);
            }
            crate::crdt::MemberUpdateAction::Remove => {
                let mut txn = doc.transact_mut();
                let member_data =
                    CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);
                let existing_ts =
                    CrdtManager::get_map_i64(&member_data, &txn, "updated_at").unwrap_or(0);
                let existing_deleted =
                    CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);

                if existing_ts > update_ts {
                    return Ok(());
                }
                if existing_ts == update_ts && existing_deleted {
                    return Ok(());
                }

                CrdtManager::set_map_string(&member_data, &mut txn, "member_id", member_id);
                CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", true);
                CrdtManager::set_map_i64(&member_data, &mut txn, "deleted_at", update_ts);
                CrdtManager::set_map_string(&member_data, &mut txn, "deleted_by", updated_by);
                CrdtManager::set_map_i64(&member_data, &mut txn, "updated_at", update_ts);
                active_members_map.remove(&mut txn, member_id);
            }
        }

        self.crdt_manager
            .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Build a membership snapshot as MemberUpdate entries (active + deleted).
    pub async fn get_member_updates(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        updated_by: &str,
    ) -> EntityServiceResult<Vec<crate::crdt::MemberUpdate>> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let members_map = doc.get_or_insert_map("members");
        let mut updates = Vec::new();

        let txn = doc.transact();
        for (member_id, _) in members_map.iter(&txn) {
            let member_id_string = member_id.to_string();
            let Some(member_data) =
                CrdtManager::get_nested_map(&members_map, &txn, &member_id_string)
            else {
                continue;
            };

            let deleted = CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);
            let role = CrdtManager::get_map_string(&member_data, &txn, "role");
            let stored_member_id = CrdtManager::get_map_string(&member_data, &txn, "member_id")
                .unwrap_or_else(|| member_id_string.clone());
            let deleted_by = CrdtManager::get_map_string(&member_data, &txn, "deleted_by");

            let joined_at = CrdtManager::get_map_i64(&member_data, &txn, "joined_at").unwrap_or(0);
            let updated_at =
                CrdtManager::get_map_i64(&member_data, &txn, "updated_at").unwrap_or(joined_at);
            let deleted_at =
                CrdtManager::get_map_i64(&member_data, &txn, "deleted_at").unwrap_or(0);
            let timestamp = if deleted {
                deleted_at
            } else if updated_at > 0 {
                updated_at
            } else {
                joined_at
            };
            let timestamp = if timestamp < 0 { 0 } else { timestamp as u64 };

            updates.push(crate::crdt::MemberUpdate {
                entity_id: entity_id.to_string(),
                entity_type,
                member_id: stored_member_id,
                role,
                updated_by: if deleted {
                    deleted_by.unwrap_or_else(|| updated_by.to_string())
                } else {
                    updated_by.to_string()
                },
                action: if deleted {
                    crate::crdt::MemberUpdateAction::Remove
                } else {
                    crate::crdt::MemberUpdateAction::Add
                },
                timestamp,
            });
        }

        Ok(updates)
    }

    /// List members of entity
    pub async fn list_members(
        &self,
        entity_type: EntityType,
        entity_id: &str,
    ) -> EntityServiceResult<Vec<MemberInfo>> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let mut members = Vec::new();

        // Get map before transaction
        let members_map = doc.get_or_insert_map("members");

        // Read all members
        // Add protection against excessive member counts that could cause memory issues
        {
            let txn = doc.transact();

            // Limit member count to prevent memory exhaustion (1000 members max)
            const MAX_MEMBERS: u32 = 1000;
            let member_count = members_map.len(&txn);
            if member_count > MAX_MEMBERS {
                return Err(EntityServiceError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!(
                        "Too many members: {} (max: {}) for entity: {}",
                        member_count, MAX_MEMBERS, entity_id
                    ),
                )));
            }

            for (member_id, _) in members_map.iter(&txn) {
                let member_id_string = member_id.to_string();

                if let Some(member_data) =
                    CrdtManager::get_nested_map(&members_map, &txn, &member_id_string)
                {
                    let deleted =
                        CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);

                    // Skip deleted members - they should not appear in the active member list
                    if deleted {
                        continue;
                    }

                    let member_id_str =
                        CrdtManager::get_map_string(&member_data, &txn, "member_id")
                            .unwrap_or_else(|| member_id_string.clone());

                    let role = CrdtManager::get_map_string(&member_data, &txn, "role")
                        .unwrap_or_else(|| "member".to_string());

                    let joined_at =
                        CrdtManager::get_map_i64(&member_data, &txn, "joined_at").unwrap_or(0);

                    members.push(MemberInfo {
                        member_id: member_id_str,
                        role,
                        joined_at,
                        deleted,
                    });
                }
            }
        }

        Ok(members)
    }

    /// Set parent organization for a child entity
    pub async fn set_parent_organization(
        &self,
        entity_id: &str,
        parent_org_id: &str,
    ) -> EntityServiceResult<()> {
        // Load entity
        let mut entity = self.get_entity(entity_id).await?;

        // Update parent_org_id
        entity.parent_org_id = Some(parent_org_id.to_string());

        // Save entity
        self.save_entity(&entity).await?;

        Ok(())
    }

    /// List all child entities of an organization
    async fn list_child_entities_of_org(
        &self,
        org_id: &str,
    ) -> EntityServiceResult<Vec<(EntityType, String)>> {
        // Scan all entities and find those with parent_org_id == org_id
        let entities = self.list_entities().await?;
        let children = entities
            .into_iter()
            .filter(|e| {
                e.parent_org_id.as_deref() == Some(org_id)
                    && matches!(
                        e.entity_type,
                        EntityType::Channel | EntityType::Group | EntityType::Project
                    )
            })
            .map(|e| (e.entity_type, e.id))
            .collect();
        Ok(children)
    }

    /// Remove member from organization and all child entities (channels, groups, projects)
    pub async fn remove_organization_member(
        &self,
        org_id: &str,
        member_id: &str,
        deleted_by: &str,
    ) -> EntityServiceResult<CascadeRemovalResult> {
        let mut result = CascadeRemovalResult {
            removed_in: vec![],
            skipped_not_member: vec![],
            failed: vec![],
        };

        // 1. Remove from organization itself
        match self
            .remove_member(EntityType::Organisation, org_id, member_id, deleted_by)
            .await
        {
            Ok(_) => result
                .removed_in
                .push((EntityType::Organisation, org_id.to_string())),
            Err(EntityServiceError::MemberNotFound(_)) => result
                .skipped_not_member
                .push((EntityType::Organisation, org_id.to_string())),
            Err(e) => {
                result
                    .failed
                    .push((EntityType::Organisation, org_id.to_string(), e.to_string()))
            }
        }

        // 2. Find all child entities
        let children = self.list_child_entities_of_org(org_id).await?;

        // 3. Remove from each child entity
        for (entity_type, entity_id) in children {
            match self
                .remove_member(entity_type, &entity_id, member_id, deleted_by)
                .await
            {
                Ok(_) => result.removed_in.push((entity_type, entity_id)),
                Err(EntityServiceError::MemberNotFound(_)) => {
                    result.skipped_not_member.push((entity_type, entity_id))
                }
                Err(e) => result.failed.push((entity_type, entity_id, e.to_string())),
            }
        }

        Ok(result)
    }

    /// Save entity metadata to CRDT
    async fn save_entity(&self, entity: &Entity) -> EntityServiceResult<()> {
        use yrs::Doc;

        let doc_id = format!("entity:{}:metadata", entity.id);
        let doc = Doc::new();

        {
            let mut txn = doc.transact_mut();
            let metadata_map = txn.get_or_insert_map("metadata");

            CrdtManager::set_map_string(&metadata_map, &mut txn, "name", &entity.name);
            CrdtManager::set_map_string(
                &metadata_map,
                &mut txn,
                "entity_type",
                entity.entity_type.as_str(),
            );

            if let Some(description) = &entity.description {
                CrdtManager::set_map_string(&metadata_map, &mut txn, "description", description);
            }

            CrdtManager::set_map_string(&metadata_map, &mut txn, "created_by", &entity.created_by);
            CrdtManager::set_map_i64(&metadata_map, &mut txn, "created_at", entity.created_at);

            if let Some(parent_org_id) = &entity.parent_org_id {
                CrdtManager::set_map_string(
                    &metadata_map,
                    &mut txn,
                    "parent_org_id",
                    parent_org_id,
                );
            }

            // Save local-only/network-linked fields
            if let Some(network_four_words) = &entity.network_four_words {
                CrdtManager::set_map_string(
                    &metadata_map,
                    &mut txn,
                    "network_four_words",
                    network_four_words,
                );
            }

            CrdtManager::set_map_bool(
                &metadata_map,
                &mut txn,
                "is_local_only",
                entity.is_local_only,
            );

            if let Some(linked_at) = entity.linked_at {
                CrdtManager::set_map_i64(&metadata_map, &mut txn, "linked_at", linked_at);
            }

            if let Some(last_sync_at) = entity.last_sync_at {
                CrdtManager::set_map_i64(&metadata_map, &mut txn, "last_sync_at", last_sync_at);
            }
        }

        self.crdt_manager
            .save_document(&doc_id, "entity", &entity.id, &doc)
            .await?;

        Ok(())
    }

    /// Create a new local-only entity (no network identity)
    pub async fn create_local_entity(
        &self,
        name: String,
        entity_type: EntityType,
        description: Option<String>,
        created_by: String,
    ) -> EntityServiceResult<Entity> {
        let entity = Entity::new_local(name, entity_type, description, created_by);

        // Save entity metadata
        self.save_entity(&entity).await?;

        Ok(entity)
    }

    /// Link an existing entity to a network identity
    pub async fn link_entity_to_network(
        &self,
        entity_id: &str,
        four_words: &str,
    ) -> EntityServiceResult<Entity> {
        // Load existing entity
        let mut entity = self.get_entity(entity_id).await?;

        // Link to network
        entity.link_to_network(four_words.to_string());

        // Save updated entity
        self.save_entity(&entity).await?;

        Ok(entity)
    }

    /// Update entity sync timestamp
    pub async fn mark_entity_synced(&self, entity_id: &str) -> EntityServiceResult<Entity> {
        let mut entity = self.get_entity(entity_id).await?;
        entity.mark_synced();
        self.save_entity(&entity).await?;
        Ok(entity)
    }

    pub async fn update_entity(
        &self,
        entity_id: &str,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> EntityServiceResult<Entity> {
        let mut entity = self.get_entity(entity_id).await?;

        if let Some(new_name) = name {
            entity.name = new_name;
        }

        if let Some(new_description) = description {
            entity.description = new_description;
        }

        self.save_entity(&entity).await?;
        Ok(entity)
    }

    pub async fn delete_entity(&self, entity_id: &str) -> EntityServiceResult<()> {
        let doc_id = format!("entity:{}:metadata", entity_id);
        self.crdt_manager
            .delete_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;
        Ok(())
    }

    /// Import/join an existing entity by ID (for multi-node sync)
    ///
    /// This allows a node to join an entity that was created on another node.
    /// The caller provides the entity metadata including the original ID.
    #[allow(clippy::too_many_arguments)]
    pub async fn import_entity(
        &self,
        id: String,
        name: String,
        entity_type: EntityType,
        description: Option<String>,
        created_by: String,
        created_at: i64,
        joiner_four_words: String,
        role: String,
    ) -> EntityServiceResult<Entity> {
        // Check if entity already exists
        if let Ok(existing) = self.get_entity(&id).await {
            return Ok(existing);
        }

        // Create entity with the provided ID
        let entity = Entity {
            id: id.clone(),
            name,
            entity_type,
            description,
            created_by,
            created_at,
            members: vec![joiner_four_words.clone()],
            parent_org_id: None,
            network_four_words: None,
            is_local_only: false,
            linked_at: None,
            last_sync_at: None,
        };

        // Save entity metadata
        self.save_entity(&entity).await?;

        // Add the joining member with their role
        self.add_member(entity_type, &id, &joiner_four_words, &role)
            .await?;

        Ok(entity)
    }

    // ========================================================================
    // Permission Methods (Phase 3b: CRDT Persistence)
    // ========================================================================

    /// Set a permission override for a member
    ///
    /// Stores the override in the entity's CRDT document under the member's
    /// permission_overrides map. This override takes precedence over role defaults.
    pub async fn set_permission_override(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        resource_type: &str,
        access_level: &str,
    ) -> EntityServiceResult<()> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let members_map = doc.get_or_insert_map("members");

        // Check if member exists
        {
            let txn = doc.transact();
            if CrdtManager::get_nested_map(&members_map, &txn, member_id).is_none() {
                return Err(EntityServiceError::MemberNotFound(member_id.to_string()));
            }
        }

        // Set the override
        {
            let mut txn = doc.transact_mut();
            let member_data =
                CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);
            let overrides = CrdtManager::get_or_create_nested_map(
                &member_data,
                &mut txn,
                "permission_overrides",
            );

            CrdtManager::set_map_string(&overrides, &mut txn, resource_type, access_level);
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Remove a permission override for a member
    ///
    /// Removes the override, reverting to the member's role default for that resource.
    pub async fn remove_permission_override(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        resource_type: &str,
    ) -> EntityServiceResult<()> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let members_map = doc.get_or_insert_map("members");

        // Check if member exists
        {
            let txn = doc.transact();
            if CrdtManager::get_nested_map(&members_map, &txn, member_id).is_none() {
                return Err(EntityServiceError::MemberNotFound(member_id.to_string()));
            }
        }

        // Remove the override
        {
            let mut txn = doc.transact_mut();

            if let Some(member_data) = CrdtManager::get_nested_map(&members_map, &txn, member_id)
                && let Some(overrides) =
                    CrdtManager::get_nested_map(&member_data, &txn, "permission_overrides")
            {
                overrides.remove(&mut txn, resource_type);
            }
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Get all permission overrides for a member
    ///
    /// Returns a map of resource_type -> access_level for all overrides.
    pub async fn get_permission_overrides(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
    ) -> EntityServiceResult<Vec<(String, String)>> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let members_map = doc.get_or_insert_map("members");

        let txn = doc.transact();

        // Check if member exists
        let member_data = CrdtManager::get_nested_map(&members_map, &txn, member_id)
            .ok_or_else(|| EntityServiceError::MemberNotFound(member_id.to_string()))?;

        // Get overrides map
        let mut overrides = Vec::new();
        if let Some(overrides_map) =
            CrdtManager::get_nested_map(&member_data, &txn, "permission_overrides")
        {
            for (key, _) in overrides_map.iter(&txn) {
                let resource_type = key.to_string();
                if let Some(access_level) =
                    CrdtManager::get_map_string(&overrides_map, &txn, &resource_type)
                {
                    overrides.push((resource_type, access_level));
                }
            }
        }

        Ok(overrides)
    }

    /// Update a member's role
    ///
    /// Changes the member's role in the CRDT document. Note that this does not
    /// clear existing permission overrides - they continue to take precedence.
    pub async fn set_member_role(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        new_role: &str,
    ) -> EntityServiceResult<()> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let members_map = doc.get_or_insert_map("members");

        // Check if member exists
        {
            let txn = doc.transact();
            if CrdtManager::get_nested_map(&members_map, &txn, member_id).is_none() {
                return Err(EntityServiceError::MemberNotFound(member_id.to_string()));
            }
        }

        // Update the role
        {
            let mut txn = doc.transact_mut();
            let member_data =
                CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

            CrdtManager::set_map_string(&member_data, &mut txn, "role", new_role);
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, entity_type.as_str(), entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Get a member's role from the CRDT document
    pub async fn get_member_role(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
    ) -> EntityServiceResult<String> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        let members_map = doc.get_or_insert_map("members");
        let txn = doc.transact();

        // Get member data
        let member_data = CrdtManager::get_nested_map(&members_map, &txn, member_id)
            .ok_or_else(|| EntityServiceError::MemberNotFound(member_id.to_string()))?;

        // Get role (default to "member" if not set)
        let role = CrdtManager::get_map_string(&member_data, &txn, "role")
            .unwrap_or_else(|| "member".to_string());

        Ok(role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt_manager::CrdtManager;
    use std::sync::Arc;
    use tempfile::tempdir;

    async fn create_test_service() -> EntityService {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let crdt_manager = Arc::new(CrdtManager::new(&db_path).await.unwrap());
        EntityService::new(crdt_manager)
    }

    #[tokio::test]
    async fn test_create_entity() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                Some("A test group".to_string()),
                "creator-id".to_string(),
                vec!["member1".to_string(), "member2".to_string()],
            )
            .await
            .expect("Failed to create entity");

        assert_eq!(entity.name, "Test Group");
        assert_eq!(entity.entity_type, EntityType::Group);
        assert_eq!(entity.created_by, "creator-id");
        assert!(entity.members.contains(&"creator-id".to_string()));
        assert!(entity.members.contains(&"member1".to_string()));
        assert!(entity.members.contains(&"member2".to_string()));
    }

    #[tokio::test]
    async fn test_get_entity() {
        let service = create_test_service().await;

        let created_entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                Some("A test group".to_string()),
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        let retrieved_entity = service
            .get_entity(&created_entity.id)
            .await
            .expect("Failed to get entity");

        assert_eq!(retrieved_entity.id, created_entity.id);
        assert_eq!(retrieved_entity.name, "Test Group");
        assert_eq!(retrieved_entity.entity_type, EntityType::Group);
    }

    #[tokio::test]
    async fn test_get_nonexistent_entity() {
        let service = create_test_service().await;

        let result = service.get_entity("nonexistent").await;
        assert!(matches!(result, Err(EntityServiceError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_add_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        service
            .add_member(EntityType::Group, &entity.id, "new-member", "admin")
            .await
            .expect("Failed to add member");

        let members = service
            .list_members(EntityType::Group, &entity.id)
            .await
            .expect("Failed to list members");

        assert_eq!(members.len(), 2); // creator + new member
        let new_member = members
            .iter()
            .find(|m| m.member_id == "new-member")
            .unwrap();
        assert_eq!(new_member.role, "admin");
        assert!(!new_member.deleted);
    }

    #[tokio::test]
    async fn test_add_duplicate_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Idempotent add: adding duplicate member should succeed (refresh role)
        let result = service
            .add_member(EntityType::Group, &entity.id, "member1", "member")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        service
            .remove_member(EntityType::Group, &entity.id, "member1", "creator-id")
            .await
            .expect("Failed to remove member");

        let members = service
            .list_members(EntityType::Group, &entity.id)
            .await
            .expect("Failed to list members");

        // Member should no longer be in the list (list_members filters deleted)
        assert!(
            !members.iter().any(|m| m.member_id == "member1"),
            "Deleted member should not appear in list_members"
        );
    }

    #[tokio::test]
    async fn test_remove_nonexistent_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        // Idempotent remove: removing nonexistent member creates a tombstone and succeeds
        let result = service
            .remove_member(EntityType::Group, &entity.id, "nonexistent", "creator-id")
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_members() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string(), "member2".to_string()],
            )
            .await
            .expect("Failed to create entity");

        let members = service
            .list_members(EntityType::Group, &entity.id)
            .await
            .expect("Failed to list members");

        assert_eq!(members.len(), 3); // creator + 2 members
        assert!(members.iter().any(|m| m.member_id == "creator-id"));
        assert!(members.iter().any(|m| m.member_id == "member1"));
        assert!(members.iter().any(|m| m.member_id == "member2"));
    }

    // ========================================================================
    // Permission Method Tests
    // ========================================================================

    #[tokio::test]
    async fn test_set_permission_override() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Set permission override
        service
            .set_permission_override(EntityType::Group, &entity.id, "member1", "messages", "edit")
            .await
            .expect("Failed to set permission override");

        // Verify the override was saved
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");

        assert_eq!(overrides.len(), 1);
        assert!(
            overrides
                .iter()
                .any(|(k, v)| k == "messages" && v == "edit")
        );
    }

    #[tokio::test]
    async fn test_set_multiple_permission_overrides() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Set multiple overrides
        service
            .set_permission_override(EntityType::Group, &entity.id, "member1", "messages", "edit")
            .await
            .expect("Failed to set override 1");

        service
            .set_permission_override(
                EntityType::Group,
                &entity.id,
                "member1",
                "documents",
                "read_only",
            )
            .await
            .expect("Failed to set override 2");

        service
            .set_permission_override(
                EntityType::Group,
                &entity.id,
                "member1",
                "settings",
                "not_visible",
            )
            .await
            .expect("Failed to set override 3");

        // Verify all overrides
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");

        assert_eq!(overrides.len(), 3);
        assert!(
            overrides
                .iter()
                .any(|(k, v)| k == "messages" && v == "edit")
        );
        assert!(
            overrides
                .iter()
                .any(|(k, v)| k == "documents" && v == "read_only")
        );
        assert!(
            overrides
                .iter()
                .any(|(k, v)| k == "settings" && v == "not_visible")
        );
    }

    #[tokio::test]
    async fn test_set_permission_override_nonexistent_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        let result = service
            .set_permission_override(
                EntityType::Group,
                &entity.id,
                "nonexistent",
                "messages",
                "edit",
            )
            .await;

        assert!(matches!(result, Err(EntityServiceError::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn test_remove_permission_override() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Set override
        service
            .set_permission_override(EntityType::Group, &entity.id, "member1", "messages", "edit")
            .await
            .expect("Failed to set override");

        // Verify it exists
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");
        assert_eq!(overrides.len(), 1);

        // Remove override
        service
            .remove_permission_override(EntityType::Group, &entity.id, "member1", "messages")
            .await
            .expect("Failed to remove override");

        // Verify it's gone
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");
        assert!(overrides.is_empty());
    }

    #[tokio::test]
    async fn test_remove_permission_override_nonexistent_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        let result = service
            .remove_permission_override(EntityType::Group, &entity.id, "nonexistent", "messages")
            .await;

        assert!(matches!(result, Err(EntityServiceError::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn test_get_permission_overrides_empty() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Get overrides (should be empty)
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");

        assert!(overrides.is_empty());
    }

    #[tokio::test]
    async fn test_get_permission_overrides_nonexistent_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        let result = service
            .get_permission_overrides(EntityType::Group, &entity.id, "nonexistent")
            .await;

        assert!(matches!(result, Err(EntityServiceError::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn test_set_member_role() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Change role
        service
            .set_member_role(EntityType::Group, &entity.id, "member1", "admin")
            .await
            .expect("Failed to set role");

        // Verify the role was changed
        let role = service
            .get_member_role(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get role");

        assert_eq!(role, "admin");
    }

    #[tokio::test]
    async fn test_set_member_role_nonexistent_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        let result = service
            .set_member_role(EntityType::Group, &entity.id, "nonexistent", "admin")
            .await;

        assert!(matches!(result, Err(EntityServiceError::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn test_get_member_role_default() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Get role (should default to "member")
        let role = service
            .get_member_role(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get role");

        assert_eq!(role, "member");
    }

    #[tokio::test]
    async fn test_get_member_role_nonexistent_member() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create entity");

        let result = service
            .get_member_role(EntityType::Group, &entity.id, "nonexistent")
            .await;

        assert!(matches!(result, Err(EntityServiceError::MemberNotFound(_))));
    }

    #[tokio::test]
    async fn test_permission_override_update() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Set initial override
        service
            .set_permission_override(
                EntityType::Group,
                &entity.id,
                "member1",
                "messages",
                "read_only",
            )
            .await
            .expect("Failed to set override");

        // Update override
        service
            .set_permission_override(EntityType::Group, &entity.id, "member1", "messages", "edit")
            .await
            .expect("Failed to update override");

        // Verify only one override exists with updated value
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");

        assert_eq!(overrides.len(), 1);
        assert!(
            overrides
                .iter()
                .any(|(k, v)| k == "messages" && v == "edit")
        );
    }

    #[tokio::test]
    async fn test_permission_persistence_across_operations() {
        let service = create_test_service().await;

        let entity = service
            .create_entity(
                "Test Group".to_string(),
                EntityType::Group,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create entity");

        // Set permission override
        service
            .set_permission_override(EntityType::Group, &entity.id, "member1", "messages", "edit")
            .await
            .expect("Failed to set override");

        // Change role (should not clear overrides)
        service
            .set_member_role(EntityType::Group, &entity.id, "member1", "viewer")
            .await
            .expect("Failed to set role");

        // Verify override still exists
        let overrides = service
            .get_permission_overrides(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get overrides");

        assert_eq!(overrides.len(), 1);
        assert!(
            overrides
                .iter()
                .any(|(k, v)| k == "messages" && v == "edit")
        );

        // Verify role was changed
        let role = service
            .get_member_role(EntityType::Group, &entity.id, "member1")
            .await
            .expect("Failed to get role");

        assert_eq!(role, "viewer");
    }

    #[tokio::test]
    async fn test_different_entity_types() {
        let service = create_test_service().await;

        // Create Project entity
        let project = service
            .create_entity(
                "Test Project".to_string(),
                EntityType::Project,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create project");

        // Create Channel entity
        let channel = service
            .create_entity(
                "Test Channel".to_string(),
                EntityType::Channel,
                None,
                "creator-id".to_string(),
                vec!["member1".to_string()],
            )
            .await
            .expect("Failed to create channel");

        // Set different overrides for each
        service
            .set_permission_override(
                EntityType::Project,
                &project.id,
                "member1",
                "kanban_boards",
                "edit",
            )
            .await
            .expect("Failed to set project override");

        service
            .set_permission_override(
                EntityType::Channel,
                &channel.id,
                "member1",
                "messages",
                "read_only",
            )
            .await
            .expect("Failed to set channel override");

        // Verify each entity has its own overrides
        let project_overrides = service
            .get_permission_overrides(EntityType::Project, &project.id, "member1")
            .await
            .expect("Failed to get project overrides");

        let channel_overrides = service
            .get_permission_overrides(EntityType::Channel, &channel.id, "member1")
            .await
            .expect("Failed to get channel overrides");

        assert_eq!(project_overrides.len(), 1);
        assert!(
            project_overrides
                .iter()
                .any(|(k, v)| k == "kanban_boards" && v == "edit")
        );

        assert_eq!(channel_overrides.len(), 1);
        assert!(
            channel_overrides
                .iter()
                .any(|(k, v)| k == "messages" && v == "read_only")
        );
    }
}

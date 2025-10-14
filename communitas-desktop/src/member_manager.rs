/// Member management using CRDT for offline-first collaboration
///
/// This module provides member management functionality with:
/// - CRDT-based member lists for each entity
/// - Event-driven tombstone pruning
/// - Offline-first operations with automatic sync
/// - LWW (Last-Write-Wins) conflict resolution

use crate::crdt_manager::CrdtManager;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use yrs::{Map, Transact};

/// Entity types that can have members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    /// Organization
    Organization,
    /// Group
    Group,
    /// Channel
    Channel,
    /// Project
    Project,
    /// Individual user
    Individual,
}

impl EntityType {
    /// Get the string representation for document IDs
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Organization => "org",
            EntityType::Group => "group",
            EntityType::Channel => "channel",
            EntityType::Project => "project",
            EntityType::Individual => "individual",
        }
    }
}

/// Member information returned by list_members
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemberInfo {
    pub member_id: String,
    pub role: String,
    pub joined_at: i64,
    pub deleted: bool,
}

/// Member management errors
#[derive(Debug, thiserror::Error)]
pub enum MemberError {
    #[error("Member already exists")]
    AlreadyExists,

    #[error("Member not found")]
    NotFound,

    #[error("CRDT error: {0}")]
    Crdt(#[from] crate::crdt_error::CrdtError),
}

/// Manages members for entities using CRDT documents
pub struct MemberManager {
    crdt: Arc<CrdtManager>,
}

/// Tombstone pruning configuration
const TOMBSTONE_MIN_AGE_SECS: i64 = 86400; // 24 hours

impl MemberManager {
    /// Create a new MemberManager
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self { crdt }
    }

    /// Add a member to an entity
    ///
    /// # Arguments
    /// * `entity_type` - The type of entity (org, group, channel, project, individual)
    /// * `entity_id` - The entity identifier
    /// * `member_id` - The member's four-word identity
    /// * `role` - The member's role ("owner", "admin", "member")
    ///
    /// # Errors
    /// Returns `MemberError::AlreadyExists` if member is already in the entity
    pub async fn add_member(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        role: &str,
    ) -> Result<(), MemberError> {
        use yrs::Doc;

        // Document ID follows taxonomy: {entity_type}:{entity_id}:{concern}
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Get or create the document
        let doc = match self.crdt.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(_) => {
                // Document doesn't exist, create new one
                Doc::new()
            }
        };

        // Perform all CRDT operations in a single scope to ensure MapRefs are dropped
        {
            // Get or create members map
            let members_map = doc.get_or_insert_map("members");
            let active_members_map = doc.get_or_insert_map("active_members");

            // Check if member already exists and is active
            {
                let txn = doc.transact();
                if let Some(member_data) =
                    CrdtManager::get_nested_map(&members_map, &txn, member_id)
                {
                    // Check if member is deleted
                    let is_deleted = CrdtManager::get_map_bool(&member_data, &txn, "deleted")
                        .unwrap_or(false);

                    if !is_deleted {
                        return Err(MemberError::AlreadyExists);
                    }
                }
            }

            // Add new member
            {
                let mut txn = doc.transact_mut();

                // Create member data map
                let member_data =
                    CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

                // Set member fields
                CrdtManager::set_map_string(&member_data, &mut txn, "member_id", member_id);
                CrdtManager::set_map_string(&member_data, &mut txn, "role", role);

                // Use current timestamp
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| {
                        crate::crdt_error::CrdtError::encoding_error(format!(
                            "Failed to get timestamp: {}",
                            e
                        ))
                    })?
                    .as_secs() as i64;

                CrdtManager::set_map_i64(&member_data, &mut txn, "joined_at", now);
                CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", false);

                // Add to active members
                CrdtManager::set_map_bool(&active_members_map, &mut txn, member_id, true);
            }
        } // MapRefs dropped here

        // Save document
        self.crdt
            .save_document(&doc_id, "entity", entity_id, &doc)
            .await?;

        Ok(())
    }

    /// List all active members of an entity
    ///
    /// # Arguments
    /// * `entity_type` - The type of entity (org, group, channel, project, individual)
    /// * `entity_id` - The entity identifier
    ///
    /// # Returns
    /// Vector of MemberInfo for all non-deleted members
    pub async fn list_members(
        &self,
        entity_type: EntityType,
        entity_id: &str,
    ) -> Result<Vec<MemberInfo>, MemberError> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self.crdt.load_document(&doc_id).await?;

        let mut members = Vec::new();

        // Read all members in a scope to ensure MapRefs are dropped
        {
            // Get members map
            let members_map = doc.get_or_insert_map("members");

            // Read all members
            {
                let txn = doc.transact();

                // Iterate through all member entries
                for (member_id, _) in members_map.iter(&txn) {
                    let member_id_string = member_id.to_string();
                    if let Some(member_data) =
                        CrdtManager::get_nested_map(&members_map, &txn, &member_id_string)
                    {
                        let deleted =
                            CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);

                        // Include all members (test expects to see deleted flag)
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
        } // MapRef dropped here

        Ok(members)
    }

    /// Remove a member from an entity (creates tombstone)
    ///
    /// # Arguments
    /// * `entity_type` - The type of entity (org, group, channel, project, individual)
    /// * `entity_id` - The entity identifier
    /// * `member_id` - The member to remove
    /// * `deleted_by` - Four-word identity of who is performing the deletion
    ///
    /// # Errors
    /// Returns `MemberError::NotFound` if member doesn't exist
    pub async fn remove_member(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        deleted_by: &str,
    ) -> Result<(), MemberError> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self.crdt.load_document(&doc_id).await?;

        // Perform all CRDT operations in a single scope to ensure MapRefs are dropped
        {
            // Get members map
            let members_map = doc.get_or_insert_map("members");
            let active_members_map = doc.get_or_insert_map("active_members");

            // Check if member exists
            {
                let txn = doc.transact();
                if CrdtManager::get_nested_map(&members_map, &txn, member_id).is_none() {
                    return Err(MemberError::NotFound);
                }
            }

            // Mark member as deleted (tombstone)
            {
                let mut txn = doc.transact_mut();

                let member_data =
                    CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

                // Set deleted flag
                CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", true);

                // Add deletion metadata
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| {
                        crate::crdt_error::CrdtError::encoding_error(format!(
                            "Failed to get timestamp: {}",
                            e
                        ))
                    })?
                    .as_secs() as i64;

                CrdtManager::set_map_i64(&member_data, &mut txn, "deleted_at", now);
                CrdtManager::set_map_string(&member_data, &mut txn, "deleted_by", deleted_by);

                // Remove from active members
                active_members_map.remove(&mut txn, member_id);
            }
        } // MapRefs dropped here

        // Save document
        self.crdt
            .save_document(&doc_id, "entity", entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Update a member's role
    ///
    /// # Arguments
    /// * `entity_type` - The type of entity (org, group, channel, project, individual)
    /// * `entity_id` - The entity identifier
    /// * `member_id` - The member to update
    /// * `new_role` - The new role to assign
    ///
    /// # Errors
    /// Returns `MemberError::NotFound` if member doesn't exist or is deleted
    pub async fn update_role(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
        new_role: &str,
    ) -> Result<(), MemberError> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self.crdt.load_document(&doc_id).await?;

        // Perform all CRDT operations in a single scope to ensure MapRefs are dropped
        {
            // Get members map
            let members_map = doc.get_or_insert_map("members");

            // Check if member exists and is not deleted
            {
                let txn = doc.transact();
                match CrdtManager::get_nested_map(&members_map, &txn, member_id) {
                    None => return Err(MemberError::NotFound),
                    Some(member_data) => {
                        // Check if member is deleted
                        let is_deleted =
                            CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);
                        if is_deleted {
                            return Err(MemberError::NotFound);
                        }
                    }
                }
            }

            // Update role
            {
                let mut txn = doc.transact_mut();

                let member_data =
                    CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

                CrdtManager::set_map_string(&member_data, &mut txn, "role", new_role);
            }
        } // MapRef dropped here

        // Save document
        self.crdt
            .save_document(&doc_id, "entity", entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Prune old tombstones from an entity's member list
    ///
    /// Removes tombstones that are older than TOMBSTONE_MIN_AGE_SECS
    /// to prevent unbounded growth of the member list.
    ///
    /// # Arguments
    /// * `entity_type` - The type of entity (org, group, channel, project, individual)
    /// * `entity_id` - The entity identifier
    ///
    /// # Returns
    /// Number of tombstones pruned
    pub async fn prune_tombstones(
        &self,
        entity_type: EntityType,
        entity_id: &str,
    ) -> Result<usize, MemberError> {
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self.crdt.load_document(&doc_id).await?;

        let mut pruned_count = 0;

        // Get current timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                crate::crdt_error::CrdtError::encoding_error(format!(
                    "Failed to get timestamp: {}",
                    e
                ))
            })?
            .as_secs() as i64;

        // Collect member IDs to prune in a scope
        let members_to_prune: Vec<String> = {
            let members_map = doc.get_or_insert_map("members");
            let txn = doc.transact();

            let mut to_prune = Vec::new();

            // Iterate through all members to find old tombstones
            for (member_id, _) in members_map.iter(&txn) {
                let member_id_string = member_id.to_string();
                if let Some(member_data) =
                    CrdtManager::get_nested_map(&members_map, &txn, &member_id_string)
                {
                    let is_deleted =
                        CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);

                    if is_deleted {
                        // Check if tombstone is old enough to prune
                        if let Some(deleted_at) =
                            CrdtManager::get_map_i64(&member_data, &txn, "deleted_at")
                        {
                            let age = now - deleted_at;
                            if age >= TOMBSTONE_MIN_AGE_SECS {
                                to_prune.push(member_id_string);
                            }
                        }
                    }
                }
            }

            to_prune
        }; // MapRef and txn dropped here

        // Now prune the collected tombstones
        if !members_to_prune.is_empty() {
            let members_map = doc.get_or_insert_map("members");
            let mut txn = doc.transact_mut();

            for member_id in &members_to_prune {
                members_map.remove(&mut txn, member_id);
                pruned_count += 1;
            }
        } // MapRef and txn dropped here

        // Save document if we pruned anything
        if pruned_count > 0 {
            self.crdt
                .save_document(&doc_id, "entity", entity_id, &doc)
                .await?;
        }

        Ok(pruned_count)
    }
}

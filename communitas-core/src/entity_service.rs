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

    #[error("CRDT error: {0}")]
    Crdt(#[from] crate::crdt_manager::CrdtError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for entity service operations
pub type EntityServiceResult<T> = Result<T, EntityServiceError>;

/// Unified entity and member management service
pub struct EntityService {
    crdt_manager: Arc<CrdtManager>,
}

impl EntityService {
    /// Create a new entity service
    pub fn new(crdt_manager: Arc<CrdtManager>) -> Self {
        Self { crdt_manager }
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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
                EntityServiceError::Io(std::io::Error::other(format!("Time error: {}", e)))
            })?
            .as_secs() as i64;

        let entity = Entity {
            id: entity_id.clone(),
            name,
            entity_type,
            description,
            created_by: created_by.clone(),
            created_at: now,
            members: initial_members.clone(),
        };

        // Save entity metadata
        self.save_entity(&entity).await?;

        // Add initial members (including creator)
        let mut all_members = initial_members;
        if !all_members.contains(&created_by) {
            all_members.push(created_by.clone());
        }

        for member_id in &all_members {
            self.add_member(entity_type, &entity_id, member_id, "member")
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
        })
    }

    /// List all entities
    pub async fn list_entities(&self) -> EntityServiceResult<Vec<Entity>> {
        use std::fs;

        // Scan the entity directory for all metadata files
        let entity_dir = self.crdt_manager.get_storage_dir().join("crdt").join("entity");

        // Perform blocking filesystem scan in dedicated blocking thread pool
        let entity_ids = tokio::task::spawn_blocking(move || -> Result<Vec<String>, EntityServiceError> {
            if !entity_dir.exists() {
                return Ok(vec![]);
            }

            let mut ids = Vec::new();

            // Read all .meta files in the entity directory
            let entries = fs::read_dir(&entity_dir).map_err(|e| {
                EntityServiceError::Io(std::io::Error::other(format!("Failed to read entity directory: {}", e)))
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
                        && let Ok(doc_id) = String::from_utf8(decoded_bytes) {
                            // Check if it's an entity metadata document (format: "entity:{id}:metadata")
                            if doc_id.starts_with("entity:") && doc_id.ends_with(":metadata") {
                                // Extract entity ID from "entity:{id}:metadata"
                                if let Some(entity_id) = doc_id.strip_prefix("entity:")
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
        .map_err(|e| EntityServiceError::Io(std::io::Error::other(format!("Blocking task failed: {}", e))))??;

        // Load entities using async operations (outside blocking section)
        let mut entities = Vec::new();
        for entity_id in entity_ids {
            match self.get_entity(&entity_id).await {
                Ok(entity) => entities.push(entity),
                Err(e) => {
                    // Log error but continue processing other entities
                    eprintln!("Warning: Failed to load entity {}: {}", entity_id, e);
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

        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

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
        {
            let txn = doc.transact();

            if let Some(member_data) = CrdtManager::get_nested_map(&members_map, &txn, member_id) {
                let is_deleted =
                    CrdtManager::get_map_bool(&member_data, &txn, "deleted").unwrap_or(false);
                if !is_deleted {
                    return Err(EntityServiceError::MemberAlreadyExists(
                        member_id.to_string(),
                    ));
                }
            }
        }

        // Add new member
        let active_members_map = doc.get_or_insert_map("active_members");
        {
            let mut txn = doc.transact_mut();

            let member_data =
                CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

            CrdtManager::set_map_string(&member_data, &mut txn, "member_id", member_id);
            CrdtManager::set_map_string(&member_data, &mut txn, "role", role);

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| {
                    EntityServiceError::Io(std::io::Error::other(format!("Time error: {}", e)))
                })?
                .as_secs() as i64;

            CrdtManager::set_map_i64(&member_data, &mut txn, "joined_at", now);
            CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", false);

            CrdtManager::set_map_bool(&active_members_map, &mut txn, member_id, true);
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, "entity", entity_id, &doc)
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
        let doc_id = format!("{}:{}:core", entity_type.as_str(), entity_id);

        // Load document
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| EntityServiceError::NotFound(entity_id.to_string()))?;

        // Get maps before transactions
        let members_map = doc.get_or_insert_map("members");
        let active_members_map = doc.get_or_insert_map("active_members");

        // Check if member exists
        {
            let txn = doc.transact();

            if CrdtManager::get_nested_map(&members_map, &txn, member_id).is_none() {
                return Err(EntityServiceError::MemberNotFound(member_id.to_string()));
            }
        }

        // Mark member as deleted (tombstone)
        {
            let mut txn = doc.transact_mut();

            let member_data =
                CrdtManager::get_or_create_nested_map(&members_map, &mut txn, member_id);

            CrdtManager::set_map_bool(&member_data, &mut txn, "deleted", true);

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| {
                    EntityServiceError::Io(std::io::Error::other(format!("Time error: {}", e)))
                })?
                .as_secs() as i64;

            CrdtManager::set_map_i64(&member_data, &mut txn, "deleted_at", now);
            CrdtManager::set_map_string(&member_data, &mut txn, "deleted_by", deleted_by);

            active_members_map.remove(&mut txn, member_id);
        }

        // Save document
        self.crdt_manager
            .save_document(&doc_id, "entity", entity_id, &doc)
            .await?;

        Ok(())
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
        }

        self.crdt_manager
            .save_document(&doc_id, "entity", &entity.id, &doc)
            .await?;

        Ok(())
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

        let result = service
            .add_member(EntityType::Group, &entity.id, "member1", "member")
            .await;

        assert!(matches!(
            result,
            Err(EntityServiceError::MemberAlreadyExists(_))
        ));
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

        let member1 = members.iter().find(|m| m.member_id == "member1").unwrap();
        assert!(member1.deleted);
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

        let result = service
            .remove_member(EntityType::Group, &entity.id, "nonexistent", "creator-id")
            .await;

        assert!(matches!(result, Err(EntityServiceError::MemberNotFound(_))));
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
}

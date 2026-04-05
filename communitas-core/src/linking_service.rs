// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Linking Service Module
//!
//! Provides functionality for linking local-only entities to network identities.
//!
//! With x0x integration, entities are linked to agent IDs rather than
//! four-word addresses.

use crate::entity_service::{Entity, EntityService, EntityServiceError};
use crate::identity::validate_id_words;
use crate::security::input_validation::InputValidator;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors related to linking operations
#[derive(Debug, Error)]
pub enum LinkingError {
    #[error("Invalid four-word address: {0}")]
    InvalidFourWords(String),

    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    #[error("Contact not found: {0}")]
    ContactNotFound(String),

    #[error("Entity service error: {0}")]
    EntityServiceError(#[from] EntityServiceError),

    #[error("Already linked: {0}")]
    AlreadyLinked(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type LinkingResult<T> = Result<T, LinkingError>;

/// Result of a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether sync was successful
    pub success: bool,
    /// ID of the entity/contact that was synced
    pub id: String,
    /// Number of changes pushed to remote
    pub changes_pushed: usize,
    /// Number of changes pulled from remote
    pub changes_pulled: usize,
    /// Error message if sync failed
    pub error: Option<String>,
}

impl SyncResult {
    /// Create a successful sync result
    pub fn success(id: String, pushed: usize, pulled: usize) -> Self {
        Self {
            success: true,
            id,
            changes_pushed: pushed,
            changes_pulled: pulled,
            error: None,
        }
    }

    /// Create a failed sync result
    pub fn failure(id: String, error: String) -> Self {
        Self {
            success: false,
            id,
            changes_pushed: 0,
            changes_pulled: 0,
            error: Some(error),
        }
    }
}

/// Service for linking local-only items to network identities
pub struct LinkingService {
    entity_service: Arc<RwLock<EntityService>>,
    validator: InputValidator,
}

impl LinkingService {
    /// Create a new linking service
    pub fn new(entity_service: Arc<RwLock<EntityService>>) -> Self {
        Self {
            entity_service,
            validator: InputValidator::default(),
        }
    }

    /// Validate a four-word address format and dictionary membership
    pub fn validate_four_words(&self, four_words: &str) -> LinkingResult<String> {
        let normalized = self
            .validator
            .validate_four_words(four_words)
            .map_err(|e| LinkingError::ValidationError(e.to_string()))?;

        if !validate_id_words(&normalized) {
            return Err(LinkingError::InvalidFourWords(format!(
                "'{}' contains words not in dictionary",
                normalized
            )));
        }

        Ok(normalized)
    }

    /// Check if a four-word address is valid
    pub fn is_valid_four_words(&self, four_words: &str) -> bool {
        self.validate_four_words(four_words).is_ok()
    }

    /// Link an entity to a network identity
    pub async fn link_entity(&self, entity_id: &str, four_words: &str) -> LinkingResult<Entity> {
        let normalized = self.validate_four_words(four_words)?;

        let entity_service = self.entity_service.write().await;
        let entity = entity_service
            .link_entity_to_network(entity_id, &normalized)
            .await?;

        Ok(entity)
    }

    /// Get all local-only entities
    pub async fn get_local_only_entities(&self) -> LinkingResult<Vec<Entity>> {
        let entity_service = self.entity_service.read().await;
        let all_entities = entity_service.list_entities().await?;

        Ok(all_entities
            .into_iter()
            .filter(|e| e.is_local_only)
            .collect())
    }

    /// Get all network-linked entities
    pub async fn get_linked_entities(&self) -> LinkingResult<Vec<Entity>> {
        let entity_service = self.entity_service.read().await;
        let all_entities = entity_service.list_entities().await?;

        Ok(all_entities.into_iter().filter(|e| e.is_linked()).collect())
    }

    /// Mark an entity as synced
    pub async fn mark_entity_synced(&self, entity_id: &str) -> LinkingResult<Entity> {
        let entity_service = self.entity_service.write().await;
        let entity = entity_service.mark_entity_synced(entity_id).await?;
        Ok(entity)
    }

    /// Create a local-only entity
    pub async fn create_local_entity(
        &self,
        name: String,
        entity_type: crate::EntityType,
        description: Option<String>,
        created_by: String,
    ) -> LinkingResult<Entity> {
        let entity_service = self.entity_service.write().await;
        let entity = entity_service
            .create_local_entity(name, entity_type, description, created_by)
            .await?;
        Ok(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EntityType;
    use crate::crdt_manager::CrdtManager;
    use tempfile::TempDir;

    async fn create_test_service() -> (LinkingService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();

        let crdt_manager = Arc::new(CrdtManager::new(data_dir.clone()).await.unwrap());
        let entity_service = Arc::new(RwLock::new(EntityService::new(crdt_manager)));

        let service = LinkingService::new(entity_service);
        (service, temp_dir)
    }

    async fn create_simple_test_service() -> LinkingService {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();

        let crdt_manager = Arc::new(CrdtManager::new(data_dir).await.unwrap());
        let entity_service = Arc::new(RwLock::new(EntityService::new(crdt_manager)));

        LinkingService::new(entity_service)
    }

    #[tokio::test]
    async fn test_validate_four_words_format() {
        let service = create_simple_test_service().await;

        assert!(
            service
                .validator
                .validate_four_words("hello-world-test-network")
                .is_ok()
        );

        assert!(service.validate_four_words("only-three-words").is_err());
        assert!(service.validate_four_words("").is_err());
        assert!(
            service
                .validate_four_words("too-many-words-here-now")
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_create_local_entity() {
        let (service, _temp_dir) = create_test_service().await;

        let entity = service
            .create_local_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A test organisation".to_string()),
                "creator-id".to_string(),
            )
            .await
            .unwrap();

        assert!(entity.is_local_only);
        assert!(entity.network_four_words.is_none());
        assert_eq!(entity.name, "Test Org");
        assert!(!entity.is_linked());
    }
}

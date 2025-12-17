// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Linking Service Module
//!
//! Provides functionality for linking local-only entities and contacts
//! to network identities via four-word addresses.
//!
//! ## Features
//!
//! - Four-word address validation using dictionary
//! - Entity linking to network identities
//! - Contact linking to network identities
//! - Sync status tracking

use crate::entity_service::{Entity, EntityService, EntityServiceError};
use crate::gossip::contact_storage::{ContactRecord, ContactResult, ContactStore, ContactStorageError};
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

    #[error("Contact storage error: {0}")]
    ContactStorageError(#[from] ContactStorageError),

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
    contact_store: Arc<ContactStore>,
    validator: InputValidator,
}

impl LinkingService {
    /// Create a new linking service
    pub fn new(
        entity_service: Arc<RwLock<EntityService>>,
        contact_store: Arc<ContactStore>,
    ) -> Self {
        Self {
            entity_service,
            contact_store,
            validator: InputValidator::new(),
        }
    }

    /// Validate a four-word address format and dictionary membership
    ///
    /// # Arguments
    /// * `four_words` - The four-word address to validate
    ///
    /// # Returns
    /// Ok(normalized_address) if valid, Err(LinkingError) if invalid
    pub fn validate_four_words(&self, four_words: &str) -> LinkingResult<String> {
        // First validate format and sanitize
        let normalized = self
            .validator
            .validate_four_words(four_words)
            .map_err(|e| LinkingError::ValidationError(e.to_string()))?;

        // Then validate dictionary membership
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
    ///
    /// # Arguments
    /// * `entity_id` - The local entity ID
    /// * `four_words` - The four-word network identity to link to
    ///
    /// # Returns
    /// The updated entity with network identity linked
    pub async fn link_entity(
        &self,
        entity_id: &str,
        four_words: &str,
    ) -> LinkingResult<Entity> {
        // Validate the four-word address
        let normalized = self.validate_four_words(four_words)?;

        // Get the entity service and link the entity
        let entity_service = self.entity_service.write().await;
        let entity = entity_service
            .link_entity_to_network(entity_id, &normalized)
            .await?;

        Ok(entity)
    }

    /// Link a contact to a network identity
    ///
    /// # Arguments
    /// * `contact_id` - The local contact ID
    /// * `four_words` - The four-word network identity to link to
    ///
    /// # Returns
    /// The updated contact with network identity linked
    pub async fn link_contact(
        &self,
        contact_id: &str,
        four_words: &str,
    ) -> LinkingResult<ContactRecord> {
        // Validate the four-word address
        let normalized = self.validate_four_words(four_words)?;

        // Link the contact
        let contact = self
            .contact_store
            .link_contact(contact_id, &normalized)
            .await?;

        Ok(contact)
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

        Ok(all_entities
            .into_iter()
            .filter(|e| e.is_linked())
            .collect())
    }

    /// Get all local-only contacts
    pub async fn get_local_only_contacts(&self) -> Vec<ContactRecord> {
        self.contact_store.local_only().await
    }

    /// Get all network-linked contacts
    pub async fn get_linked_contacts(&self) -> Vec<ContactRecord> {
        self.contact_store.network_linked().await
    }

    /// Mark an entity as synced
    pub async fn mark_entity_synced(&self, entity_id: &str) -> LinkingResult<Entity> {
        let entity_service = self.entity_service.write().await;
        let entity = entity_service.mark_entity_synced(entity_id).await?;
        Ok(entity)
    }

    /// Mark a contact as synced
    pub async fn mark_contact_synced(&self, contact_id: &str) -> LinkingResult<ContactRecord> {
        let contact = self
            .contact_store
            .get_by_id(contact_id)
            .await
            .ok_or_else(|| LinkingError::ContactNotFound(contact_id.to_string()))?;

        // Update the contact with sync timestamp
        let mut updated = contact;
        updated.mark_synced();

        // Re-add to store (replaces existing)
        // Note: This is a bit awkward - ideally we'd have an update method
        // For now, we just update the record in place
        let contacts = self.contact_store.all().await;
        self.contact_store.clear().await;
        for c in contacts {
            if c.id == contact_id {
                let _ = self.contact_store.add(updated.clone()).await;
            } else {
                let _ = self.contact_store.add(c).await;
            }
        }

        Ok(updated)
    }

    /// Create a local-only entity
    pub async fn create_local_entity(
        &self,
        name: String,
        entity_type: crate::legacy_crdt::EntityType,
        description: Option<String>,
        created_by: String,
    ) -> LinkingResult<Entity> {
        let entity_service = self.entity_service.write().await;
        let entity = entity_service
            .create_local_entity(name, entity_type, description, created_by)
            .await?;
        Ok(entity)
    }

    /// Create a local-only contact
    pub async fn create_local_contact(
        &self,
        display_name: String,
    ) -> ContactResult<ContactRecord> {
        let contact = ContactRecord::new_local(display_name);
        self.contact_store.add(contact.clone()).await?;
        Ok(contact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt_manager::CrdtManager;
    use crate::legacy_crdt::EntityType;
    use tempfile::TempDir;

    async fn create_test_service() -> (LinkingService, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();

        let crdt_manager = Arc::new(CrdtManager::new(data_dir.clone()).await.unwrap());
        let entity_service = Arc::new(RwLock::new(EntityService::new(crdt_manager)));
        let contact_store = Arc::new(ContactStore::new());

        let service = LinkingService::new(entity_service, contact_store);
        (service, temp_dir)
    }

    async fn create_simple_test_service() -> LinkingService {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();

        let crdt_manager = Arc::new(CrdtManager::new(data_dir).await.unwrap());
        let entity_service = Arc::new(RwLock::new(EntityService::new(crdt_manager)));
        let contact_store = Arc::new(ContactStore::new());

        LinkingService::new(entity_service, contact_store)
    }

    #[tokio::test]
    async fn test_validate_four_words_format() {
        let service = create_simple_test_service().await;

        // Valid format but may not be in dictionary
        let _result = service.validate_four_words("hello-world-test-network");
        // This depends on dictionary - may or may not be valid
        // For format testing, we just check the sanitization works
        assert!(service.validator.validate_four_words("hello-world-test-network").is_ok());

        // Invalid formats
        assert!(service.validate_four_words("only-three-words").is_err());
        assert!(service.validate_four_words("").is_err());
        assert!(service.validate_four_words("too-many-words-here-now").is_err());
    }

    #[tokio::test]
    async fn test_create_local_contact() {
        let (service, _temp_dir) = create_test_service().await;

        let contact = service
            .create_local_contact("Alice".to_string())
            .await
            .unwrap();

        assert!(contact.is_local_only);
        assert!(contact.four_words.is_none());
        assert_eq!(contact.display_name, Some("Alice".to_string()));
        assert_eq!(contact.effective_name(), "Alice");
    }

    #[tokio::test]
    async fn test_get_local_only_contacts() {
        let (service, _temp_dir) = create_test_service().await;

        // Create some contacts
        service
            .create_local_contact("Alice".to_string())
            .await
            .unwrap();
        service
            .create_local_contact("Bob".to_string())
            .await
            .unwrap();

        // Add a network-linked contact directly
        let linked = ContactRecord::new("ocean-forest-moon-star".to_string());
        service.contact_store.add(linked).await.unwrap();

        // Get local-only contacts
        let local_only = service.get_local_only_contacts().await;
        assert_eq!(local_only.len(), 2);

        // Get linked contacts
        let linked = service.get_linked_contacts().await;
        assert_eq!(linked.len(), 1);
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

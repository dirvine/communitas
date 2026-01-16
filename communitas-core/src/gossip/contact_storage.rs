// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Contact Storage Module
//!
//! Provides persistent storage for contact records with endpoint tracking.
//! Each contact stores their last-seen four-word encoded endpoint for
//! direct reconnection attempts before falling back to FOAF discovery.
//!
//! ## Features
//!
//! - Four-word identity validation against dictionary
//! - Display name separate from identity
//! - Last-seen endpoint tracking with TTL and failure backoff
//! - Persistence via encrypted storage

use crate::identity::{conn_from_words, conn_words};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors related to contact storage operations
#[derive(Debug, Error)]
pub enum ContactStorageError {
    #[error("Contact not found: {0}")]
    NotFound(String),

    #[error("Invalid four-word identity: {0}")]
    InvalidIdentity(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Endpoint encoding error: {0}")]
    EndpointEncodingError(String),
}

pub type ContactResult<T> = Result<T, ContactStorageError>;

/// Endpoint TTL in hours before considering it stale
const ENDPOINT_TTL_HOURS: u64 = 24;

/// Maximum consecutive failures before skipping endpoint
const MAX_ENDPOINT_FAILURES: u32 = 3;

/// A contact record with endpoint tracking
///
/// Stores all information about a contact including their last-seen
/// network endpoint for direct reconnection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactRecord {
    /// Unique identifier (UUID) for this contact record
    pub id: String,

    /// Four-word identity (hyphen-separated, e.g., "ocean-forest-moon-star")
    /// Optional for local-only contacts without network identity
    pub four_words: Option<String>,

    /// Display name (user-chosen, separate from identity)
    pub display_name: Option<String>,

    /// Whether this contact is marked as favourite
    pub is_favourite: bool,

    /// Whether this is local-only (no network identity yet)
    #[serde(default)]
    pub is_local_only: bool,

    /// Unix timestamp (milliseconds) when linked to network identity
    pub linked_at: Option<u64>,

    /// Unix timestamp (milliseconds) of last successful sync
    pub last_sync_at: Option<u64>,

    /// Last-seen endpoint encoded as four words (space-separated)
    /// This is the IP:port encoded via conn_words()
    pub last_seen_endpoint: Option<String>,

    /// Unix timestamp (milliseconds) when endpoint was last updated
    pub endpoint_updated_at: Option<u64>,

    /// Count of successful connections to this endpoint
    pub endpoint_success_count: u32,

    /// Count of consecutive connection failures
    pub endpoint_failure_count: u32,

    /// Unix timestamp (milliseconds) when contact was created
    pub created_at: u64,

    /// Unix timestamp (milliseconds) when contact was last seen online
    pub last_online_at: Option<u64>,
}

impl ContactRecord {
    /// Create a new contact record with a four-word identity (network-linked)
    pub fn new(four_words: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            four_words: Some(four_words),
            display_name: None,
            is_favourite: false,
            is_local_only: false,
            linked_at: None,
            last_sync_at: None,
            last_seen_endpoint: None,
            endpoint_updated_at: None,
            endpoint_success_count: 0,
            endpoint_failure_count: 0,
            created_at: now_millis(),
            last_online_at: None,
        }
    }

    /// Create a new local-only contact without a network identity
    pub fn new_local(display_name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            four_words: None,
            display_name: Some(display_name),
            is_favourite: false,
            is_local_only: true,
            linked_at: None,
            last_sync_at: None,
            last_seen_endpoint: None,
            endpoint_updated_at: None,
            endpoint_success_count: 0,
            endpoint_failure_count: 0,
            created_at: now_millis(),
            last_online_at: None,
        }
    }

    /// Create a new contact with display name
    pub fn with_display_name(four_words: String, display_name: String) -> Self {
        let mut contact = Self::new(four_words);
        contact.display_name = Some(display_name);
        contact
    }

    /// Check if this contact is linked to a network identity
    pub fn is_linked(&self) -> bool {
        self.four_words.is_some() && !self.is_local_only
    }

    /// Link this local-only contact to a network identity
    pub fn link_to_network(&mut self, four_words: String) {
        self.four_words = Some(four_words);
        self.is_local_only = false;
        self.linked_at = Some(now_millis());
    }

    /// Update the last sync timestamp
    pub fn mark_synced(&mut self) {
        self.last_sync_at = Some(now_millis());
    }

    /// Get the effective display name (display_name or four_words or "Unknown")
    pub fn effective_name(&self) -> String {
        self.display_name
            .clone()
            .or_else(|| self.four_words.clone())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Get a valid endpoint if available
    ///
    /// Returns the endpoint only if:
    /// - An endpoint is stored
    /// - It hasn't expired (within TTL)
    /// - It hasn't had too many consecutive failures
    pub fn get_valid_endpoint(&self) -> Option<SocketAddr> {
        let endpoint_words = self.last_seen_endpoint.as_ref()?;
        let updated_at = self.endpoint_updated_at?;

        // Check TTL (24 hours)
        let now = now_millis();
        let age_hours = (now.saturating_sub(updated_at)) / (1000 * 60 * 60);
        if age_hours > ENDPOINT_TTL_HOURS {
            return None;
        }

        // Skip if too many consecutive failures
        if self.endpoint_failure_count >= MAX_ENDPOINT_FAILURES {
            return None;
        }

        // Decode the four-word endpoint to SocketAddr
        conn_from_words(endpoint_words).ok()
    }

    /// Check if the stored endpoint is stale
    pub fn is_endpoint_stale(&self) -> bool {
        match self.endpoint_updated_at {
            Some(updated_at) => {
                let now = now_millis();
                let age_hours = (now.saturating_sub(updated_at)) / (1000 * 60 * 60);
                age_hours > ENDPOINT_TTL_HOURS
            }
            None => true,
        }
    }

    /// Record a successful connection to this contact
    ///
    /// Updates the endpoint and resets failure count
    pub fn record_success(&mut self, addr: SocketAddr) {
        if let Ok(words) = conn_words(&addr) {
            self.last_seen_endpoint = Some(words);
            self.endpoint_updated_at = Some(now_millis());
            self.endpoint_success_count = self.endpoint_success_count.saturating_add(1);
            self.endpoint_failure_count = 0;
            self.last_online_at = Some(now_millis());
        }
    }

    /// Record a connection failure to this contact
    ///
    /// Increments the failure count. After MAX_ENDPOINT_FAILURES,
    /// the endpoint will be skipped in get_valid_endpoint()
    pub fn record_failure(&mut self) {
        self.endpoint_failure_count = self.endpoint_failure_count.saturating_add(1);
    }

    /// Reset the endpoint failure count
    ///
    /// Call this when you want to retry an endpoint that had failures
    pub fn reset_failures(&mut self) {
        self.endpoint_failure_count = 0;
    }

    /// Update the endpoint from four-word encoded string
    pub fn update_endpoint_from_words(&mut self, endpoint_words: &str) -> ContactResult<()> {
        // Validate by attempting to decode
        conn_from_words(endpoint_words).map_err(|e| {
            ContactStorageError::EndpointEncodingError(format!(
                "Invalid endpoint words '{}': {}",
                endpoint_words, e
            ))
        })?;

        self.last_seen_endpoint = Some(endpoint_words.to_string());
        self.endpoint_updated_at = Some(now_millis());
        self.endpoint_failure_count = 0;
        Ok(())
    }

    /// Update the endpoint from a SocketAddr
    pub fn update_endpoint(&mut self, addr: &SocketAddr) -> ContactResult<()> {
        let words = conn_words(addr).map_err(|e| {
            ContactStorageError::EndpointEncodingError(format!(
                "Failed to encode endpoint {}: {}",
                addr, e
            ))
        })?;

        self.last_seen_endpoint = Some(words);
        self.endpoint_updated_at = Some(now_millis());
        self.endpoint_failure_count = 0;
        Ok(())
    }

    /// Mark this contact as online now
    pub fn mark_online(&mut self) {
        self.last_online_at = Some(now_millis());
    }

    /// Get the endpoint age in hours, if set
    pub fn endpoint_age_hours(&self) -> Option<u64> {
        self.endpoint_updated_at.map(|updated_at| {
            let now = now_millis();
            (now.saturating_sub(updated_at)) / (1000 * 60 * 60)
        })
    }
}

/// Thread-safe contact store
///
/// Manages a collection of contacts with persistence support.
/// Uses id-based primary key with a secondary index for four_words lookups.
#[derive(Debug, Clone)]
pub struct ContactStore {
    /// Primary storage: id -> ContactRecord
    contacts: Arc<RwLock<HashMap<String, ContactRecord>>>,
    /// Secondary index: four_words -> id (for network-linked contacts)
    four_words_index: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for ContactStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ContactStore {
    /// Create a new empty contact store
    pub fn new() -> Self {
        Self {
            contacts: Arc::new(RwLock::new(HashMap::new())),
            four_words_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a new contact
    pub async fn add(&self, contact: ContactRecord) -> ContactResult<()> {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;

        // Update secondary index if contact has four_words
        if let Some(ref fw) = contact.four_words {
            index.insert(fw.clone(), contact.id.clone());
        }

        contacts.insert(contact.id.clone(), contact);
        Ok(())
    }

    /// Get a contact by id
    pub async fn get_by_id(&self, id: &str) -> Option<ContactRecord> {
        let contacts = self.contacts.read().await;
        contacts.get(id).cloned()
    }

    /// Get a contact by four-word identity (backward compatible)
    pub async fn get(&self, four_words: &str) -> Option<ContactRecord> {
        self.get_by_four_words(four_words).await
    }

    /// Get a contact by four-word identity
    pub async fn get_by_four_words(&self, four_words: &str) -> Option<ContactRecord> {
        let index = self.four_words_index.read().await;
        if let Some(id) = index.get(four_words) {
            let contacts = self.contacts.read().await;
            return contacts.get(id).cloned();
        }
        None
    }

    /// Check if a contact exists by id
    pub async fn exists_by_id(&self, id: &str) -> bool {
        let contacts = self.contacts.read().await;
        contacts.contains_key(id)
    }

    /// Check if a contact exists by four-word identity (backward compatible)
    pub async fn exists(&self, four_words: &str) -> bool {
        let index = self.four_words_index.read().await;
        index.contains_key(four_words)
    }

    /// Update a contact record
    pub async fn update(&self, contact: ContactRecord) -> ContactResult<()> {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;

        if !contacts.contains_key(&contact.id) {
            return Err(ContactStorageError::NotFound(contact.id.clone()));
        }

        // Update secondary index if four_words changed
        if let Some(ref fw) = contact.four_words {
            index.insert(fw.clone(), contact.id.clone());
        }

        contacts.insert(contact.id.clone(), contact);
        Ok(())
    }

    /// Update or insert a contact
    pub async fn upsert(&self, contact: ContactRecord) {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;

        // Update secondary index if contact has four_words
        if let Some(ref fw) = contact.four_words {
            index.insert(fw.clone(), contact.id.clone());
        }

        contacts.insert(contact.id.clone(), contact);
    }

    /// Remove a contact by id
    pub async fn remove_by_id(&self, id: &str) -> ContactResult<ContactRecord> {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;

        let contact = contacts
            .remove(id)
            .ok_or_else(|| ContactStorageError::NotFound(id.to_string()))?;

        // Remove from secondary index
        if let Some(ref fw) = contact.four_words {
            index.remove(fw);
        }

        Ok(contact)
    }

    /// Remove a contact by four-word identity (backward compatible)
    pub async fn remove(&self, four_words: &str) -> ContactResult<ContactRecord> {
        let index = self.four_words_index.read().await;
        let id = index
            .get(four_words)
            .cloned()
            .ok_or_else(|| ContactStorageError::NotFound(four_words.to_string()))?;
        drop(index);

        self.remove_by_id(&id).await
    }

    /// Link a local-only contact to a network identity
    pub async fn link_contact(&self, id: &str, four_words: &str) -> ContactResult<ContactRecord> {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;

        let contact = contacts
            .get_mut(id)
            .ok_or_else(|| ContactStorageError::NotFound(id.to_string()))?;

        // Link to network
        contact.link_to_network(four_words.to_string());

        // Update secondary index
        index.insert(four_words.to_string(), id.to_string());

        Ok(contact.clone())
    }

    /// Get all local-only contacts
    pub async fn local_only(&self) -> Vec<ContactRecord> {
        let contacts = self.contacts.read().await;
        contacts
            .values()
            .filter(|c| c.is_local_only)
            .cloned()
            .collect()
    }

    /// Get all network-linked contacts
    pub async fn network_linked(&self) -> Vec<ContactRecord> {
        let contacts = self.contacts.read().await;
        contacts
            .values()
            .filter(|c| !c.is_local_only && c.four_words.is_some())
            .cloned()
            .collect()
    }

    /// Get all contacts
    pub async fn all(&self) -> Vec<ContactRecord> {
        let contacts = self.contacts.read().await;
        contacts.values().cloned().collect()
    }

    /// Get all favourite contacts
    pub async fn favourites(&self) -> Vec<ContactRecord> {
        let contacts = self.contacts.read().await;
        contacts
            .values()
            .filter(|c| c.is_favourite)
            .cloned()
            .collect()
    }

    /// Get contacts with valid endpoints
    pub async fn with_valid_endpoints(&self) -> Vec<ContactRecord> {
        let contacts = self.contacts.read().await;
        contacts
            .values()
            .filter(|c| c.get_valid_endpoint().is_some())
            .cloned()
            .collect()
    }

    /// Update endpoint for a contact by four-word identity
    pub async fn update_endpoint(&self, four_words: &str, addr: &SocketAddr) -> ContactResult<()> {
        let index = self.four_words_index.read().await;
        let id = index
            .get(four_words)
            .cloned()
            .ok_or_else(|| ContactStorageError::NotFound(four_words.to_string()))?;
        drop(index);

        let mut contacts = self.contacts.write().await;
        let contact = contacts
            .get_mut(&id)
            .ok_or_else(|| ContactStorageError::NotFound(id.clone()))?;
        contact.update_endpoint(addr)
    }

    /// Update endpoint for a contact by id
    pub async fn update_endpoint_by_id(&self, id: &str, addr: &SocketAddr) -> ContactResult<()> {
        let mut contacts = self.contacts.write().await;
        let contact = contacts
            .get_mut(id)
            .ok_or_else(|| ContactStorageError::NotFound(id.to_string()))?;
        contact.update_endpoint(addr)
    }

    /// Record successful connection for a contact
    pub async fn record_success(&self, four_words: &str, addr: SocketAddr) -> ContactResult<()> {
        let index = self.four_words_index.read().await;
        let id = index
            .get(four_words)
            .cloned()
            .ok_or_else(|| ContactStorageError::NotFound(four_words.to_string()))?;
        drop(index);

        let mut contacts = self.contacts.write().await;
        let contact = contacts
            .get_mut(&id)
            .ok_or_else(|| ContactStorageError::NotFound(id.clone()))?;
        contact.record_success(addr);
        Ok(())
    }

    /// Record connection failure for a contact
    pub async fn record_failure(&self, four_words: &str) -> ContactResult<()> {
        let index = self.four_words_index.read().await;
        let id = index
            .get(four_words)
            .cloned()
            .ok_or_else(|| ContactStorageError::NotFound(four_words.to_string()))?;
        drop(index);

        let mut contacts = self.contacts.write().await;
        let contact = contacts
            .get_mut(&id)
            .ok_or_else(|| ContactStorageError::NotFound(id.clone()))?;
        contact.record_failure();
        Ok(())
    }

    /// Get count of contacts
    pub async fn count(&self) -> usize {
        let contacts = self.contacts.read().await;
        contacts.len()
    }

    /// Clear all contacts
    pub async fn clear(&self) {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;
        contacts.clear();
        index.clear();
    }

    /// Export all contacts for persistence
    pub async fn export(&self) -> Vec<ContactRecord> {
        self.all().await
    }

    /// Import contacts from persistence
    pub async fn import(&self, records: Vec<ContactRecord>) {
        let mut contacts = self.contacts.write().await;
        let mut index = self.four_words_index.write().await;

        for record in records {
            // Update secondary index if contact has four_words
            if let Some(ref fw) = record.four_words {
                index.insert(fw.clone(), record.id.clone());
            }
            contacts.insert(record.id.clone(), record);
        }
    }
}

/// Get current time in milliseconds since UNIX epoch
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_record_new() {
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());

        assert_eq!(
            contact.four_words,
            Some("ocean-forest-moon-star".to_string())
        );
        assert!(contact.display_name.is_none());
        assert!(!contact.is_favourite);
        assert!(!contact.is_local_only);
        assert!(contact.is_linked());
        assert!(contact.last_seen_endpoint.is_none());
        assert_eq!(contact.endpoint_success_count, 0);
        assert_eq!(contact.endpoint_failure_count, 0);
    }

    #[test]
    fn test_contact_record_new_local() {
        let contact = ContactRecord::new_local("Alice".to_string());

        assert!(contact.four_words.is_none());
        assert_eq!(contact.display_name, Some("Alice".to_string()));
        assert!(contact.is_local_only);
        assert!(!contact.is_linked());
        assert_eq!(contact.effective_name(), "Alice");
    }

    #[test]
    fn test_contact_record_link_to_network() {
        let mut contact = ContactRecord::new_local("Alice".to_string());

        assert!(contact.is_local_only);
        assert!(!contact.is_linked());

        contact.link_to_network("ocean-forest-moon-star".to_string());

        assert!(!contact.is_local_only);
        assert!(contact.is_linked());
        assert_eq!(
            contact.four_words,
            Some("ocean-forest-moon-star".to_string())
        );
        assert!(contact.linked_at.is_some());
    }

    #[test]
    fn test_contact_record_effective_name() {
        // With display name and four_words
        let contact = ContactRecord::with_display_name(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
        );
        assert_eq!(contact.effective_name(), "Alice");

        // With only four_words
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        assert_eq!(contact.effective_name(), "ocean-forest-moon-star");

        // Local only with display name
        let contact = ContactRecord::new_local("Bob".to_string());
        assert_eq!(contact.effective_name(), "Bob");
    }

    #[test]
    fn test_contact_record_with_display_name() {
        let contact = ContactRecord::with_display_name(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
        );

        assert_eq!(
            contact.four_words,
            Some("ocean-forest-moon-star".to_string())
        );
        assert_eq!(contact.display_name, Some("Alice".to_string()));
    }

    #[test]
    fn test_record_success() {
        let mut contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

        contact.record_success(addr);

        assert!(contact.last_seen_endpoint.is_some());
        assert!(contact.endpoint_updated_at.is_some());
        assert_eq!(contact.endpoint_success_count, 1);
        assert_eq!(contact.endpoint_failure_count, 0);
    }

    #[test]
    fn test_record_failure() {
        let mut contact = ContactRecord::new("ocean-forest-moon-star".to_string());

        contact.record_failure();
        assert_eq!(contact.endpoint_failure_count, 1);

        contact.record_failure();
        assert_eq!(contact.endpoint_failure_count, 2);

        contact.record_failure();
        assert_eq!(contact.endpoint_failure_count, 3);
    }

    #[test]
    fn test_get_valid_endpoint_after_success() {
        let mut contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();

        contact.record_success(addr);

        let result = contact.get_valid_endpoint();
        assert_eq!(result, Some(addr));
    }

    #[test]
    fn test_get_valid_endpoint_too_many_failures() {
        let mut contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();

        contact.record_success(addr);

        // Record MAX_ENDPOINT_FAILURES failures
        for _ in 0..MAX_ENDPOINT_FAILURES {
            contact.record_failure();
        }

        // Should return None due to failures
        assert!(contact.get_valid_endpoint().is_none());
    }

    #[test]
    fn test_reset_failures() {
        let mut contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();

        contact.record_success(addr);

        // Record failures
        contact.record_failure();
        contact.record_failure();
        contact.record_failure();

        // Reset
        contact.reset_failures();
        assert_eq!(contact.endpoint_failure_count, 0);

        // Should be valid again
        assert!(contact.get_valid_endpoint().is_some());
    }

    #[tokio::test]
    async fn test_contact_store_add_get() {
        let store = ContactStore::new();
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());

        store.add(contact).await.unwrap();

        let retrieved = store.get("ocean-forest-moon-star").await;
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().four_words,
            Some("ocean-forest-moon-star".to_string())
        );
    }

    #[tokio::test]
    async fn test_contact_store_remove() {
        let store = ContactStore::new();
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());

        store.add(contact).await.unwrap();
        let removed = store.remove("ocean-forest-moon-star").await;

        assert!(removed.is_ok());
        assert!(store.get("ocean-forest-moon-star").await.is_none());
    }

    #[tokio::test]
    async fn test_contact_store_favourites() {
        let store = ContactStore::new();

        let mut contact1 = ContactRecord::new("ocean-forest-moon-star".to_string());
        contact1.is_favourite = true;

        let contact2 = ContactRecord::new("river-mountain-cloud-wind".to_string());

        store.add(contact1).await.unwrap();
        store.add(contact2).await.unwrap();

        let favourites = store.favourites().await;
        assert_eq!(favourites.len(), 1);
        assert_eq!(
            favourites[0].four_words,
            Some("ocean-forest-moon-star".to_string())
        );
    }

    #[tokio::test]
    async fn test_contact_store_with_valid_endpoints() {
        let store = ContactStore::new();
        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();

        let mut contact1 = ContactRecord::new("ocean-forest-moon-star".to_string());
        contact1.record_success(addr);

        let contact2 = ContactRecord::new("river-mountain-cloud-wind".to_string());

        store.add(contact1).await.unwrap();
        store.add(contact2).await.unwrap();

        let with_endpoints = store.with_valid_endpoints().await;
        assert_eq!(with_endpoints.len(), 1);
        assert_eq!(
            with_endpoints[0].four_words,
            Some("ocean-forest-moon-star".to_string())
        );
    }

    #[tokio::test]
    async fn test_contact_store_update_endpoint() {
        let store = ContactStore::new();
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        store.add(contact).await.unwrap();

        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();
        store
            .update_endpoint("ocean-forest-moon-star", &addr)
            .await
            .unwrap();

        let updated = store.get("ocean-forest-moon-star").await.unwrap();
        assert!(updated.last_seen_endpoint.is_some());
        assert_eq!(updated.get_valid_endpoint(), Some(addr));
    }

    #[tokio::test]
    async fn test_contact_store_record_success_failure() {
        let store = ContactStore::new();
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        store.add(contact).await.unwrap();

        let addr: SocketAddr = "192.168.1.100:9000".parse().unwrap();
        store
            .record_success("ocean-forest-moon-star", addr)
            .await
            .unwrap();

        let updated = store.get("ocean-forest-moon-star").await.unwrap();
        assert_eq!(updated.endpoint_success_count, 1);
        assert_eq!(updated.endpoint_failure_count, 0);

        store
            .record_failure("ocean-forest-moon-star")
            .await
            .unwrap();

        let updated = store.get("ocean-forest-moon-star").await.unwrap();
        assert_eq!(updated.endpoint_failure_count, 1);
    }

    #[tokio::test]
    async fn test_contact_store_export_import() {
        let store = ContactStore::new();

        let contact1 = ContactRecord::new("ocean-forest-moon-star".to_string());
        let contact2 = ContactRecord::new("river-mountain-cloud-wind".to_string());

        store.add(contact1).await.unwrap();
        store.add(contact2).await.unwrap();

        let exported = store.export().await;
        assert_eq!(exported.len(), 2);

        let new_store = ContactStore::new();
        new_store.import(exported).await;

        assert_eq!(new_store.count().await, 2);
        assert!(new_store.get("ocean-forest-moon-star").await.is_some());
        assert!(new_store.get("river-mountain-cloud-wind").await.is_some());
    }

    #[tokio::test]
    async fn test_contact_store_get_by_id() {
        let store = ContactStore::new();
        let contact = ContactRecord::new("ocean-forest-moon-star".to_string());
        let contact_id = contact.id.clone();

        store.add(contact).await.unwrap();

        // Get by ID should work
        let retrieved = store.get_by_id(&contact_id).await;
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().four_words,
            Some("ocean-forest-moon-star".to_string())
        );
    }

    #[tokio::test]
    async fn test_contact_store_local_only_contacts() {
        let store = ContactStore::new();

        // Add a network-linked contact
        let contact1 = ContactRecord::new("ocean-forest-moon-star".to_string());
        store.add(contact1).await.unwrap();

        // Add a local-only contact
        let contact2 = ContactRecord::new_local("Alice".to_string());
        let local_id = contact2.id.clone();
        store.add(contact2).await.unwrap();

        // Check local_only() returns only local contacts
        let local_contacts = store.local_only().await;
        assert_eq!(local_contacts.len(), 1);
        assert_eq!(local_contacts[0].id, local_id);
        assert!(local_contacts[0].is_local_only);

        // Check network_linked() returns only linked contacts
        let linked_contacts = store.network_linked().await;
        assert_eq!(linked_contacts.len(), 1);
        assert!(!linked_contacts[0].is_local_only);
    }

    #[tokio::test]
    async fn test_contact_store_link_contact() {
        let store = ContactStore::new();

        // Add a local-only contact
        let contact = ContactRecord::new_local("Alice".to_string());
        let contact_id = contact.id.clone();
        store.add(contact).await.unwrap();

        // Verify it's local-only
        let local_contacts = store.local_only().await;
        assert_eq!(local_contacts.len(), 1);

        // Link it to a network identity
        let linked = store
            .link_contact(&contact_id, "ocean-forest-moon-star")
            .await
            .unwrap();

        assert!(!linked.is_local_only);
        assert_eq!(
            linked.four_words,
            Some("ocean-forest-moon-star".to_string())
        );
        assert!(linked.linked_at.is_some());

        // Now it should be in network_linked, not local_only
        let local_contacts = store.local_only().await;
        assert_eq!(local_contacts.len(), 0);

        let linked_contacts = store.network_linked().await;
        assert_eq!(linked_contacts.len(), 1);

        // Should also be retrievable by four_words now
        let by_fw = store.get("ocean-forest-moon-star").await;
        assert!(by_fw.is_some());
    }

    #[tokio::test]
    async fn test_contact_store_remove_by_id() {
        let store = ContactStore::new();

        let contact = ContactRecord::new_local("Alice".to_string());
        let contact_id = contact.id.clone();
        store.add(contact).await.unwrap();

        assert_eq!(store.count().await, 1);

        // Remove by ID
        let removed = store.remove_by_id(&contact_id).await;
        assert!(removed.is_ok());

        assert_eq!(store.count().await, 0);
        assert!(store.get_by_id(&contact_id).await.is_none());
    }
}

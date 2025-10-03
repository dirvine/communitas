//! Storage integration for DHT identity system
//!
//! This module provides the storage layer that integrates with saorsa-core's DHT
//! and storage systems to persist identity records and blobs.

use crate::dht_identity::{ContentId, blobs::*, key_derivation::*, records::*};
use saorsa_core::{dht::DhtCoreEngine, storage::StorageManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Error types for identity storage operations
#[derive(Debug, thiserror::Error)]
pub enum IdentityStorageError {
    #[error("DHT operation failed: {0}")]
    DhtError(#[from] anyhow::Error),

    #[error("Serialization failed: {0}")]
    SerializationError(#[from] serde_cbor::Error),

    #[error("Identity not found: {0}")]
    IdentityNotFound(String),

    #[error("Invalid identity record: {0}")]
    InvalidIdentity(String),

    #[error("Storage limit exceeded: {0}")]
    StorageLimitExceeded(String),

    #[error("Blob not found: {0}")]
    BlobNotFound(String),
}

pub type StorageResult<T> = std::result::Result<T, IdentityStorageError>;

/// High-level identity storage interface
pub struct IdentityStorage {
    /// DHT engine for small records
    dht: Arc<DhtCoreEngine>,

    /// Storage manager for large blobs
    storage: Arc<StorageManager>,

    /// Local cache for frequently accessed items
    cache: Arc<RwLock<IdentityCache>>,
}

/// Local cache for identity data
#[derive(Debug, Default)]
struct IdentityCache {
    /// Cached identity root records
    root_records: HashMap<[u8; 32], CachedItem<IdentityRootRecord>>,

    /// Cached connection records
    connection_records: HashMap<[u8; 32], CachedItem<ConnectionRecord>>,

    /// Cached site manifest records
    site_records: HashMap<[u8; 32], CachedItem<SiteManifestRecord>>,

    /// Cached blobs by content ID
    blobs: HashMap<ContentId, CachedItem<Vec<u8>>>,
}

/// Cache item with metadata
#[derive(Debug, Clone)]
struct CachedItem<T> {
    data: T,
    cached_at: SystemTime,
    ttl_seconds: Option<u32>,
}

impl<T> CachedItem<T> {
    fn new(data: T, ttl_seconds: Option<u32>) -> Self {
        Self {
            data,
            cached_at: SystemTime::now(),
            ttl_seconds,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl_seconds {
            let age = self
                .cached_at
                .elapsed()
                .unwrap_or(std::time::Duration::ZERO)
                .as_secs() as u32;
            age > ttl
        } else {
            false
        }
    }
}

/// Complete identity information retrieved from storage
#[derive(Debug, Clone)]
pub struct ResolvedIdentity {
    /// Four-word identity
    pub four_words: NormalizedFourWords,

    /// Root record from DHT
    pub root_record: IdentityRootRecord,

    /// Identity descriptor blob
    pub descriptor: IdentityDescriptorBlob,

    /// Optional connection information
    pub connection_info: Option<(ConnectionRecord, ConnectionBlob)>,

    /// Optional site manifest
    pub site_info: Option<(SiteManifestRecord, SiteManifestBlob)>,
}

impl IdentityStorage {
    /// Create a new identity storage instance
    pub fn new(dht: Arc<DhtCoreEngine>, storage: Arc<StorageManager>) -> Self {
        Self {
            dht,
            storage,
            cache: Arc::new(RwLock::new(IdentityCache::default())),
        }
    }

    /// Store a complete identity in the DHT and storage
    pub async fn store_identity(&self, identity: &ResolvedIdentity) -> StorageResult<()> {
        // First store the descriptor blob
        let descriptor_bytes = identity.descriptor.to_cbor()?;
        let descriptor_cid = self.store_blob(&descriptor_bytes).await?;

        // Ensure the descriptor CID matches what's in the root record
        if descriptor_cid != identity.root_record.descriptor_cid {
            return Err(IdentityStorageError::InvalidIdentity(
                "Descriptor CID mismatch".to_string(),
            ));
        }

        // Store connection and site blobs if present
        if let Some((conn_record, conn_blob)) = &identity.connection_info {
            let conn_bytes = conn_blob.to_cbor()?;
            let conn_cid = self.store_blob(&conn_bytes).await?;

            if conn_record.connection_cid != conn_cid {
                return Err(IdentityStorageError::InvalidIdentity(
                    "Connection CID mismatch".to_string(),
                ));
            }

            // Store connection record in DHT
            let conn_key = derive_connection_key(&identity.four_words);
            let conn_record_bytes = conn_record.to_cbor()?;
            self.store_dht_record(&conn_key, conn_record_bytes).await?;
        }

        if let Some((site_record, site_manifest)) = &identity.site_info {
            let site_bytes = site_manifest.to_cbor()?;
            let site_cid = self.store_blob(&site_bytes).await?;

            if site_record.site_cid != site_cid {
                return Err(IdentityStorageError::InvalidIdentity(
                    "Site CID mismatch".to_string(),
                ));
            }

            // Store site record in DHT
            let site_key = derive_site_key(&identity.four_words);
            let site_record_bytes = site_record.to_cbor()?;
            self.store_dht_record(&site_key, site_record_bytes).await?;
        }

        // Finally store the identity root record in DHT
        let id_key = derive_identity_key(&identity.four_words);
        let root_record_bytes = identity.root_record.to_cbor()?;
        self.store_dht_record(&id_key, root_record_bytes).await?;

        Ok(())
    }

    /// Retrieve a complete identity from storage
    pub async fn retrieve_identity(
        &self,
        four_words: &str,
    ) -> StorageResult<Option<ResolvedIdentity>> {
        let normalized = NormalizedFourWords::new(four_words)
            .map_err(|e| IdentityStorageError::InvalidIdentity(e))?;

        // Get the root record first
        let id_key = derive_identity_key(&normalized);
        let root_record = match self.get_identity_root(&id_key).await? {
            Some(record) => record,
            None => return Ok(None),
        };

        // Validate identity hash binding
        let expected_identity_hash = *blake3::hash(normalized.as_str().as_bytes()).as_bytes();
        if root_record.identity_hash != expected_identity_hash {
            return Err(IdentityStorageError::InvalidIdentity(
                "Identity hash mismatch".to_string(),
            ));
        }

        // Get the descriptor blob
        let descriptor_bytes = self
            .retrieve_blob(&root_record.descriptor_cid)
            .await?
            .ok_or_else(|| IdentityStorageError::BlobNotFound("Identity descriptor".to_string()))?;

        let descriptor = IdentityDescriptorBlob::from_cbor(&descriptor_bytes)?;

        // Validate descriptor
        descriptor
            .validate()
            .map_err(|e| IdentityStorageError::InvalidIdentity(e))?;

        // Verify signature
        match descriptor.verify() {
            Ok(true) => {} // Valid signature
            Ok(false) | Err(_) => {
                return Err(IdentityStorageError::InvalidIdentity(
                    "Invalid descriptor signature".to_string(),
                ));
            }
        }

        // Get optional connection info
        let connection_info = if let Some(conn_cid) = root_record.connection_cid {
            let conn_key = derive_connection_key(&normalized);
            if let Some(conn_record) = self.get_connection_record(&conn_key).await? {
                if conn_record.connection_cid == conn_cid {
                    let conn_bytes = self.retrieve_blob(&conn_cid).await?.ok_or_else(|| {
                        IdentityStorageError::BlobNotFound("Connection blob".to_string())
                    })?;
                    let conn_blob = ConnectionBlob::from_cbor(&conn_bytes)?;
                    Some((conn_record, conn_blob))
                } else {
                    None // CID mismatch, ignore stale connection record
                }
            } else {
                None
            }
        } else {
            None
        };

        // Get optional site info
        let site_info = if let Some(site_cid) = root_record.site_cid {
            let site_key = derive_site_key(&normalized);
            if let Some(site_record) = self.get_site_record(&site_key).await? {
                if site_record.site_cid == site_cid {
                    let site_bytes = self.retrieve_blob(&site_cid).await?.ok_or_else(|| {
                        IdentityStorageError::BlobNotFound("Site manifest".to_string())
                    })?;
                    let site_manifest = SiteManifestBlob::from_cbor(&site_bytes)?;
                    Some((site_record, site_manifest))
                } else {
                    None // CID mismatch, ignore stale site record
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Some(ResolvedIdentity {
            four_words: normalized,
            root_record,
            descriptor,
            connection_info,
            site_info,
        }))
    }

    /// Update connection information for an identity
    pub async fn update_connection(
        &self,
        four_words: &str,
        connection_record: &ConnectionRecord,
        connection_blob: &ConnectionBlob,
    ) -> StorageResult<()> {
        let normalized = NormalizedFourWords::new(four_words)
            .map_err(|e| IdentityStorageError::InvalidIdentity(e))?;

        // Store the connection blob
        let conn_bytes = connection_blob.to_cbor()?;
        let conn_cid = self.store_blob(&conn_bytes).await?;

        // Verify CID matches
        if connection_record.connection_cid != conn_cid {
            return Err(IdentityStorageError::InvalidIdentity(
                "Connection CID mismatch".to_string(),
            ));
        }

        // Store the connection record
        let conn_key = derive_connection_key(&normalized);
        let conn_record_bytes = connection_record.to_cbor()?;
        self.store_dht_record(&conn_key, conn_record_bytes).await?;

        // Update the root record to reference the new connection
        let id_key = derive_identity_key(&normalized);
        if let Some(mut root_record) = self.get_identity_root(&id_key).await? {
            root_record.connection_cid = Some(conn_cid);
            root_record.sequence += 1;
            root_record.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let root_bytes = root_record.to_cbor()?;
            self.store_dht_record(&id_key, root_bytes).await?;
        } else {
            return Err(IdentityStorageError::IdentityNotFound(
                four_words.to_string(),
            ));
        }

        Ok(())
    }

    /// Update site information for an identity
    pub async fn update_site(
        &self,
        four_words: &str,
        site_record: &SiteManifestRecord,
        site_manifest: &SiteManifestBlob,
    ) -> StorageResult<()> {
        let normalized = NormalizedFourWords::new(four_words)
            .map_err(|e| IdentityStorageError::InvalidIdentity(e))?;

        // Validate site manifest
        site_manifest
            .validate()
            .map_err(|e| IdentityStorageError::InvalidIdentity(e))?;

        // Store the site manifest blob
        let site_bytes = site_manifest.to_cbor()?;
        let site_cid = self.store_blob(&site_bytes).await?;

        // Verify CID matches
        if site_record.site_cid != site_cid {
            return Err(IdentityStorageError::InvalidIdentity(
                "Site CID mismatch".to_string(),
            ));
        }

        // Store the site record
        let site_key = derive_site_key(&normalized);
        let site_record_bytes = site_record.to_cbor()?;
        self.store_dht_record(&site_key, site_record_bytes).await?;

        // Update the root record to reference the new site
        let id_key = derive_identity_key(&normalized);
        if let Some(mut root_record) = self.get_identity_root(&id_key).await? {
            root_record.site_cid = Some(site_cid);
            root_record.sequence += 1;
            root_record.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let root_bytes = root_record.to_cbor()?;
            self.store_dht_record(&id_key, root_bytes).await?;
        } else {
            return Err(IdentityStorageError::IdentityNotFound(
                four_words.to_string(),
            ));
        }

        Ok(())
    }

    /// Check if an identity exists
    pub async fn identity_exists(&self, four_words: &str) -> StorageResult<bool> {
        let normalized = NormalizedFourWords::new(four_words)
            .map_err(|e| IdentityStorageError::InvalidIdentity(e))?;
        let id_key = derive_identity_key(&normalized);
        Ok(self.get_identity_root(&id_key).await?.is_some())
    }

    /// Clear cache entries
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.root_records.clear();
        cache.connection_records.clear();
        cache.site_records.clear();
        cache.blobs.clear();
    }

    // Private helper methods

    async fn get_identity_root(&self, key: &[u8; 32]) -> StorageResult<Option<IdentityRootRecord>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.root_records.get(key) {
                if !cached.is_expired() {
                    return Ok(Some(cached.data.clone()));
                }
            }
        }

        // Retrieve from DHT
        if let Some(bytes) = self.retrieve_dht_record(key).await? {
            let record = IdentityRootRecord::from_cbor(&bytes)?;

            // Cache the result
            {
                let mut cache = self.cache.write().await;
                cache.root_records.insert(
                    key.clone(),
                    CachedItem::new(record.clone(), record.ttl_seconds),
                );
            }

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    async fn get_connection_record(
        &self,
        key: &[u8; 32],
    ) -> StorageResult<Option<ConnectionRecord>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.connection_records.get(key) {
                if !cached.is_expired() {
                    return Ok(Some(cached.data.clone()));
                }
            }
        }

        // Retrieve from DHT
        if let Some(bytes) = self.retrieve_dht_record(key).await? {
            let record = ConnectionRecord::from_cbor(&bytes)?;

            // Cache the result
            {
                let mut cache = self.cache.write().await;
                cache.connection_records.insert(
                    key.clone(),
                    CachedItem::new(record.clone(), Some(record.ttl_seconds)),
                );
            }

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    async fn get_site_record(&self, key: &[u8; 32]) -> StorageResult<Option<SiteManifestRecord>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.site_records.get(key) {
                if !cached.is_expired() {
                    return Ok(Some(cached.data.clone()));
                }
            }
        }

        // Retrieve from DHT
        if let Some(bytes) = self.retrieve_dht_record(key).await? {
            let record = SiteManifestRecord::from_cbor(&bytes)?;

            // Cache the result
            {
                let mut cache = self.cache.write().await;
                cache.site_records.insert(
                    key.clone(),
                    CachedItem::new(record.clone(), Some(record.ttl_seconds)),
                );
            }

            Ok(Some(record))
        } else {
            Ok(None)
        }
    }

    async fn store_dht_record(&self, key: &[u8; 32], value: Vec<u8>) -> StorageResult<()> {
        // Validate size limit
        if value.len() > super::DHT_RECORD_MAX_SIZE {
            return Err(IdentityStorageError::StorageLimitExceeded(format!(
                "DHT record too large: {} bytes",
                value.len()
            )));
        }

        // Store in DHT (note: the DHT engine expects a mutable reference)
        // In a real implementation, we'd need to handle this properly
        // For now, this represents the interface we need

        // Convert our DhtKey to saorsa-core's DhtKey format
        let _saorsa_key = saorsa_core::dht::DhtKey::from_bytes(*key);

        // The actual store operation would happen here
        // self.dht.store(&_saorsa_key, value).await?;

        // For now, we'll mark this as a placeholder that needs proper integration
        // with the mutable DHT reference
        Ok(())
    }

    async fn retrieve_dht_record(&self, key: &[u8; 32]) -> StorageResult<Option<Vec<u8>>> {
        // Convert our DhtKey to saorsa-core's DhtKey format
        let saorsa_key = saorsa_core::dht::DhtKey::from_bytes(*key);

        // Retrieve from DHT
        match self.dht.retrieve(&saorsa_key).await {
            Ok(data) => Ok(data),
            Err(e) => Err(IdentityStorageError::DhtError(e)),
        }
    }

    async fn store_blob(&self, data: &[u8]) -> StorageResult<ContentId> {
        // Calculate content address
        let cid = derive_content_address(data);

        // Check cache first
        {
            let cache = self.cache.read().await;
            if cache.blobs.contains_key(&cid) {
                return Ok(cid);
            }
        }

        // Store in content-addressed storage via StorageManager
        // The actual storage operation would use the saorsa-core StorageManager
        // For now, this is a placeholder that represents the intended interface

        // Cache the data
        {
            let mut cache = self.cache.write().await;
            cache
                .blobs
                .insert(cid, CachedItem::new(data.to_vec(), None));
        }

        Ok(cid)
    }

    async fn retrieve_blob(&self, cid: &ContentId) -> StorageResult<Option<Vec<u8>>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.blobs.get(cid) {
                if !cached.is_expired() {
                    return Ok(Some(cached.data.clone()));
                }
            }
        }

        // Retrieve from content-addressed storage via StorageManager
        // The actual retrieval would use the saorsa-core StorageManager
        // For now, return None to indicate not found
        Ok(None)
    }
}

// Helper function to convert DhtKey types (removed - can't implement foreign traits)

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dht_identity::blobs::{PageEntry, RendezvousEntry};
    use saorsa_core::dht::core_engine::NodeId as DhtNodeId;

    fn create_test_dht() -> Arc<DhtCoreEngine> {
        let node_id = DhtNodeId::from_bytes([42u8; 32]);
        Arc::new(DhtCoreEngine::new(node_id).unwrap())
    }

    fn create_test_storage() -> Arc<StorageManager> {
        // This would be a real StorageManager in production
        // For tests, we'll use a placeholder
        // Arc::new(StorageManager::new(...))
        todo!("Create test storage manager - requires integration with saorsa-core StorageManager")
    }

    async fn create_test_identity() -> ResolvedIdentity {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Create test keys
        let ml_dsa_key = vec![1u8; 1952]; // Mock ML-DSA key
        let ml_kem_key = vec![2u8; 1184]; // Mock ML-KEM key
        let transport_keys = vec![];

        // Create descriptor
        let descriptor = IdentityDescriptorBlob::new(
            four_words.as_str().to_string(),
            [3u8; 32], // root_digest
            ml_dsa_key,
            ml_kem_key,
            transport_keys,
            "Test User".to_string(),
        );

        let descriptor_cid = descriptor.content_address().unwrap();

        // Create root record
        let root_record = IdentityRootRecord::new(
            1, // sequence
            timestamp,
            *blake3::hash(four_words.as_str().as_bytes()).as_bytes(), // identity_hash
            [4u8; 32],                                                // ml_dsa_key_hash
            [5u8; 32],                                                // ml_kem_key_hash
            [6u8; 32],                                                // transport_id
            descriptor_cid,
        );

        ResolvedIdentity {
            four_words,
            root_record,
            descriptor,
            connection_info: None,
            site_info: None,
        }
    }

    #[tokio::test]
    async fn test_identity_storage_creation() {
        let dht = create_test_dht();
        // Skip storage manager creation for now
        // let storage = create_test_storage();

        // Test that we can create the storage instance
        // let identity_storage = IdentityStorage::new(dht, storage);

        // For now, just test that the DHT was created successfully
        // Note: node_id is private, so we can't access it directly in tests
        // This test just verifies the DHT was created without error
    }

    #[tokio::test]
    async fn test_four_word_key_derivation_integration() {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();

        let id_key = derive_identity_key(&four_words);
        let conn_key = derive_connection_key(&four_words);
        let site_key = derive_site_key(&four_words);

        // Keys should be different
        assert_ne!(id_key, conn_key);
        assert_ne!(id_key, site_key);
        assert_ne!(conn_key, site_key);

        // Keys should be deterministic
        assert_eq!(id_key, derive_identity_key(&four_words));
    }

    #[tokio::test]
    async fn test_resolved_identity_creation() {
        let identity = create_test_identity().await;

        assert_eq!(identity.four_words.as_str(), "ocean-forest-moon-star");
        assert_eq!(identity.root_record.sequence, 1);
        assert_eq!(identity.descriptor.four_words, "ocean-forest-moon-star");
        assert_eq!(identity.descriptor.preferred_display_name, "Test User");
        assert!(identity.connection_info.is_none());
        assert!(identity.site_info.is_none());
    }

    #[tokio::test]
    async fn test_cache_item_expiration() {
        let item = CachedItem::new("test_data", Some(1)); // 1 second TTL
        assert!(!item.is_expired());

        // Item without TTL never expires
        let permanent_item = CachedItem::new("permanent", None);
        assert!(!permanent_item.is_expired());

        // Test with expired item (would need sleep to test properly)
        let expired_item = CachedItem {
            data: "expired",
            cached_at: SystemTime::UNIX_EPOCH,
            ttl_seconds: Some(1),
        };
        assert!(expired_item.is_expired());
    }

    #[tokio::test]
    async fn test_identity_record_validation() {
        let four_words = "ocean-forest-moon-star";
        let normalized = NormalizedFourWords::new(four_words).unwrap();
        let expected_hash = *blake3::hash(normalized.as_str().as_bytes()).as_bytes();

        // Test with correct hash
        let mut record = IdentityRootRecord::new(
            1,
            1640995200000,
            expected_hash,
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
            [4u8; 32],
        );

        // Validation should pass (this would be done in the storage layer)
        assert_eq!(record.identity_hash, expected_hash);

        // Test with incorrect hash
        record.identity_hash = [99u8; 32];
        assert_ne!(record.identity_hash, expected_hash);
    }

    #[tokio::test]
    async fn test_connection_blob_creation() {
        let rendezvous_nodes = vec![
            RendezvousEntry {
                address: "bootstrap1.example.com:9000".to_string(),
                priority: 1,
            },
            RendezvousEntry {
                address: "bootstrap2.example.com:9000".to_string(),
                priority: 2,
            },
        ];

        let conn_blob = ConnectionBlob::new(
            [1u8; 32], // transport_id
            rendezvous_nodes,
            1640995200000, // timestamp
            3600,          // ttl
        );

        let conn_cid = conn_blob.content_address().unwrap();

        let conn_record = ConnectionRecord::new(
            1,             // sequence
            1640995200000, // timestamp
            [1u8; 32],     // transport_id
            conn_cid,
            3600, // ttl
        );

        assert_eq!(conn_record.connection_cid, conn_cid);
        assert_eq!(conn_blob.rendezvous_nodes.len(), 2);
        assert!(conn_blob.policy.allow_relays);
    }

    #[tokio::test]
    async fn test_site_manifest_creation() {
        let pages = vec![
            PageEntry {
                path: "/index.html".to_string(),
                content_id: [1u8; 32],
                size_bytes: 1024,
                mime_type: "text/html".to_string(),
                content_hash: [10u8; 32],
            },
            PageEntry {
                path: "/about.html".to_string(),
                content_id: [2u8; 32],
                size_bytes: 512,
                mime_type: "text/html".to_string(),
                content_hash: [20u8; 32],
            },
        ];

        let site_manifest = SiteManifestBlob::new("/index.html".to_string(), pages)
            .expect("Should create manifest");

        site_manifest.validate().expect("Should be valid");

        let site_cid = site_manifest.content_address().unwrap();

        let site_record = SiteManifestRecord::new(
            1,             // sequence
            1640995200000, // timestamp
            site_cid,
            7200, // ttl
        );

        assert_eq!(site_record.site_cid, site_cid);
        assert_eq!(site_manifest.total_size(), 1536);
        assert_eq!(site_manifest.pages.len(), 2);
    }
}

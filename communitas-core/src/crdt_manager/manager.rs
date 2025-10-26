// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use super::{CrdtError, CrdtResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use yrs::updates::decoder::Decode;
use yrs::{Any, Doc, Map, MapPrelim, MapRef, Out, ReadTxn, Transact, TransactionMut, Update};

/// Document metadata stored alongside Yrs state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentMetadata {
    doc_id: String,
    entity_type: String,
    entity_id: String,
    version: u32,
    created_at: i64,
    updated_at: i64,
}

/// Manages CRDT documents with filesystem persistence
///
/// **Storage Layout:**
/// ```text
/// storage_dir/
///   crdt/
///     {entity_type}/
///       {doc_id}.yrs      # Binary Yrs state
///       {doc_id}.meta     # JSON metadata
/// ```
pub struct CrdtManager {
    storage_dir: PathBuf,
}

impl CrdtManager {
    /// Initialize CrdtManager with filesystem storage
    ///
    /// Creates the CRDT storage directory structure if it doesn't exist.
    pub async fn new<P: AsRef<Path>>(storage_dir: P) -> CrdtResult<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        let crdt_dir = storage_dir.join("crdt");

        // Create directory structure
        fs::create_dir_all(&crdt_dir).await.map_err(|e| {
            CrdtError::FileSystem(format!("Failed to create CRDT directory: {}", e))
        })?;

        Ok(Self { storage_dir })
    }

    /// Get the storage directory path
    pub fn get_storage_dir(&self) -> &Path {
        &self.storage_dir
    }

    /// Get the directory path for a specific entity type
    fn entity_dir(&self, entity_type: &str) -> PathBuf {
        self.storage_dir.join("crdt").join(entity_type)
    }

    /// Sanitize doc_id for use as filename (Windows-safe)
    ///
    /// Doc IDs can contain colons (e.g., "entity:{id}:core") which are forbidden
    /// on Windows. We hex-encode the entire doc_id to ensure cross-platform safety.
    fn sanitize_doc_id(doc_id: &str) -> String {
        hex::encode(doc_id.as_bytes())
    }

    /// Get the file paths for a document
    fn doc_paths(&self, entity_type: &str, doc_id: &str) -> (PathBuf, PathBuf) {
        let entity_dir = self.entity_dir(entity_type);
        let safe_filename = Self::sanitize_doc_id(doc_id);
        let yrs_path = entity_dir.join(format!("{}.yrs", safe_filename));
        let meta_path = entity_dir.join(format!("{}.meta", safe_filename));
        (yrs_path, meta_path)
    }

    /// Save a Yrs document to the filesystem
    pub async fn save_document(
        &self,
        doc_id: &str,
        entity_type: &str,
        entity_id: &str,
        doc: &Doc,
    ) -> CrdtResult<()> {
        // Validate that doc_id starts with entity_type
        if !doc_id.starts_with(entity_type) {
            return Err(CrdtError::InvalidDocumentId(format!(
                "doc_id '{}' must start with entity_type '{}'",
                doc_id, entity_type
            )));
        }

        // Ensure entity directory exists
        let entity_dir = self.entity_dir(entity_type);
        fs::create_dir_all(&entity_dir).await.map_err(|e| {
            CrdtError::FileSystem(format!("Failed to create entity directory: {}", e))
        })?;

        // Encode Yrs state
        let state = doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Check encoded size (10MB limit)
        const MAX_ENCODED_SIZE: usize = 10 * 1024 * 1024;
        if state.len() > MAX_ENCODED_SIZE {
            return Err(CrdtError::encoding_error(format!(
                "Document too large: {} bytes (max: {})",
                state.len(),
                MAX_ENCODED_SIZE
            )));
        }

        let (yrs_path, meta_path) = self.doc_paths(entity_type, doc_id);

        // Load existing metadata or create new
        let mut metadata = if meta_path.exists() {
            let meta_json = fs::read_to_string(&meta_path)
                .await
                .map_err(|e| CrdtError::FileSystem(format!("Failed to read metadata: {}", e)))?;
            let mut existing: DocumentMetadata = serde_json::from_str(&meta_json)
                .map_err(|e| CrdtError::Serialization(format!("Invalid metadata JSON: {}", e)))?;
            existing.version += 1; // Increment for this save
            existing
        } else {
            let now = chrono::Utc::now().timestamp();
            DocumentMetadata {
                doc_id: doc_id.to_string(),
                entity_type: entity_type.to_string(),
                entity_id: entity_id.to_string(),
                version: 1, // First version
                created_at: now,
                updated_at: now,
            }
        };

        // Update timestamp
        metadata.updated_at = chrono::Utc::now().timestamp();

        // Write files atomically (temp + rename)
        // Create temp files in same directory to avoid cross-filesystem rename (EXDEV error)
        let yrs_temp = yrs_path.with_extension("yrs.tmp");
        let meta_temp = meta_path.with_extension("meta.tmp");

        fs::write(&yrs_temp, &state)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to write Yrs state: {}", e)))?;

        let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            CrdtError::Serialization(format!("Failed to serialize metadata: {}", e))
        })?;
        fs::write(&meta_temp, meta_json)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to write metadata: {}", e)))?;

        // Atomic rename (same filesystem - no EXDEV error)
        fs::rename(&yrs_temp, &yrs_path)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to rename Yrs file: {}", e)))?;
        fs::rename(&meta_temp, &meta_path)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to rename metadata file: {}", e)))?;

        Ok(())
    }

    /// Load a Yrs document from the filesystem
    pub async fn load_document(&self, doc_id: &str) -> CrdtResult<Doc> {
        // Parse entity_type from doc_id (format: "entity_type:entity_id:suffix")
        let parts: Vec<&str> = doc_id.split(':').collect();
        if parts.len() < 2 {
            return Err(CrdtError::InvalidDocumentId(format!(
                "Invalid doc_id format (expected 'entity_type:entity_id:...'): {}",
                doc_id
            )));
        }

        let entity_type = parts[0];
        let (yrs_path, _) = self.doc_paths(entity_type, doc_id);

        if !yrs_path.exists() {
            return Err(CrdtError::DocumentNotFound(doc_id.to_string()));
        }

        // Load and decode
        let state_bytes = fs::read(&yrs_path)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read Yrs state: {}", e)))?;

        let update = Update::decode_v1(&state_bytes).map_err(|e| {
            CrdtError::Deserialization(format!("Failed to decode Yrs state: {}", e))
        })?;

        let doc = Doc::new();
        {
            let mut txn = doc.transact_mut();
            txn.apply_update(update);
        }

        Ok(doc)
    }

    /// List all document IDs for a given entity type
    pub async fn list_documents(&self, entity_type: &str) -> CrdtResult<Vec<String>> {
        let entity_dir = self.entity_dir(entity_type);

        if !entity_dir.exists() {
            return Ok(Vec::new());
        }

        let mut doc_ids = Vec::new();

        let mut read_dir = fs::read_dir(&entity_dir).await.map_err(|e| {
            CrdtError::FileSystem(format!("Failed to read entity directory: {}", e))
        })?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read directory entry: {}", e)))?
        {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // Only process .yrs files
            if file_name_str.ends_with(".yrs") {
                let hex_id = file_name_str.trim_end_matches(".yrs");
                // Decode hex back to original doc_id
                if let Ok(bytes) = hex::decode(hex_id)
                    && let Ok(doc_id) = String::from_utf8(bytes)
                {
                    doc_ids.push(doc_id);
                }
            }
        }

        Ok(doc_ids)
    }

    /// Apply an update to an existing document
    ///
    /// This method loads a document, applies the update, and saves it back.
    /// Used for syncing updates from remote peers.
    pub async fn apply_update(&self, doc_id: &str, update_bytes: &[u8]) -> CrdtResult<()> {
        // Check update size limit
        const MAX_ENCODED_SIZE: usize = 10 * 1024 * 1024;
        if update_bytes.len() > MAX_ENCODED_SIZE {
            return Err(CrdtError::encoding_error(format!(
                "Update too large: {} bytes (max: {})",
                update_bytes.len(),
                MAX_ENCODED_SIZE
            )));
        }

        // Load the document (error if it doesn't exist)
        let doc = self.load_document(doc_id).await?;

        // Decode and apply the update
        let update = Update::decode_v1(update_bytes)
            .map_err(|e| CrdtError::Deserialization(format!("Failed to decode update: {}", e)))?;

        {
            let mut txn = doc.transact_mut();
            txn.apply_update(update);
        }

        // Extract entity_type and entity_id from doc_id
        // Expected format: "entity_type:entity_id:metadata" or similar
        let parts: Vec<&str> = doc_id.split(':').collect();
        if parts.len() < 2 {
            return Err(CrdtError::InvalidDocumentId(format!(
                "Invalid doc_id format: {}",
                doc_id
            )));
        }

        let entity_type = parts[0];
        let entity_id = parts[1];

        // Save the updated document
        self.save_document(doc_id, entity_type, entity_id, &doc)
            .await?;

        Ok(())
    }

    /// Merge updates from multiple peers into a document
    ///
    /// Takes a document ID and a list of update blobs from different peers,
    /// applies them all, and saves the merged result.
    pub async fn merge_updates(&self, doc_id: &str, updates: Vec<Vec<u8>>) -> CrdtResult<Doc> {
        // Check update size limits
        const MAX_ENCODED_SIZE: usize = 10 * 1024 * 1024;
        for (i, update_bytes) in updates.iter().enumerate() {
            if update_bytes.len() > MAX_ENCODED_SIZE {
                return Err(CrdtError::encoding_error(format!(
                    "Update {} too large: {} bytes (max: {})",
                    i,
                    update_bytes.len(),
                    MAX_ENCODED_SIZE
                )));
            }
        }

        // Load existing document (error if it doesn't exist)
        let doc = self.load_document(doc_id).await?;

        // Apply all updates
        for update_bytes in updates {
            let update = Update::decode_v1(&update_bytes).map_err(|e| {
                CrdtError::Deserialization(format!("Failed to decode update: {}", e))
            })?;

            let mut txn = doc.transact_mut();
            txn.apply_update(update);
        }

        // Extract entity info and save
        let parts: Vec<&str> = doc_id.split(':').collect();
        if parts.len() >= 2 {
            let entity_type = parts[0];
            let entity_id = parts[1];

            self.save_document(doc_id, entity_type, entity_id, &doc)
                .await?;
        }

        Ok(doc)
    }

    /// Mark a document as deleted with a tombstone
    ///
    /// Instead of physically deleting the document, this sets a "deleted" flag
    /// in the metadata. The tombstone will replicate to all peers via CRDT sync.
    pub async fn mark_deleted(&self, doc_id: &str, deleted_by: &str) -> CrdtResult<()> {
        let doc = self.load_document(doc_id).await?;

        {
            let root = doc.get_or_insert_map("root");
            let mut txn = doc.transact_mut();

            // Get or create metadata map
            let metadata = if let Some(existing) = root.get(&txn, "metadata") {
                MapRef::try_from(existing).map_err(|e| {
                    CrdtError::Operation(format!("Invalid metadata structure: {:?}", e))
                })?
            } else {
                let empty_prelim: MapPrelim = MapPrelim::from([("_", Any::Null)]);
                let m = root.insert(&mut txn, "metadata", empty_prelim);
                m.remove(&mut txn, "_");
                m
            };

            // Set tombstone fields
            metadata.insert(&mut txn, "deleted", true);
            metadata.insert(&mut txn, "deleted_at", chrono::Utc::now().timestamp());
            metadata.insert(&mut txn, "deleted_by", deleted_by);
        }

        // Extract entity info and save
        let parts: Vec<&str> = doc_id.split(':').collect();
        if parts.len() >= 2 {
            let entity_type = parts[0];
            let entity_id = parts[1];

            self.save_document(doc_id, entity_type, entity_id, &doc)
                .await?;
        }

        Ok(())
    }

    /// Check if a document is marked as deleted (tombstone check)
    pub async fn is_deleted(&self, doc_id: &str) -> CrdtResult<bool> {
        let doc = self.load_document(doc_id).await?;

        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        if let Some(metadata_val) = root.get(&txn, "metadata")
            && let Ok(metadata) = MapRef::try_from(metadata_val)
            && let Some(deleted_val) = metadata.get(&txn, "deleted")
            && let Ok(deleted) = bool::try_from(deleted_val)
        {
            return Ok(deleted);
        }

        Ok(false)
    }

    /// Delete a document from the filesystem
    pub async fn delete_document(&self, doc_id: &str) -> CrdtResult<()> {
        // Parse entity_type from doc_id
        let parts: Vec<&str> = doc_id.split(':').collect();
        if parts.len() < 2 {
            return Err(CrdtError::InvalidDocumentId(format!(
                "Invalid doc_id format (expected 'entity_type:entity_id:...'): {}",
                doc_id
            )));
        }

        let entity_type = parts[0];
        let (yrs_path, meta_path) = self.doc_paths(entity_type, doc_id);

        if !yrs_path.exists() {
            return Err(CrdtError::DocumentNotFound(doc_id.to_string()));
        }

        fs::remove_file(&yrs_path)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to delete Yrs file: {}", e)))?;

        if meta_path.exists() {
            fs::remove_file(&meta_path).await.map_err(|e| {
                CrdtError::FileSystem(format!("Failed to delete metadata file: {}", e))
            })?;
        }

        Ok(())
    }

    // === Static helper methods for root Map operations ===

    /// Get a value from the root Map of a document
    pub fn get_map_value<T>(doc: &Doc, key: &str) -> CrdtResult<Option<T>>
    where
        T: TryFrom<Out>,
    {
        // Get map before transaction to avoid nested transaction
        let map = doc.get_or_insert_map("root");
        let txn = doc.transact();
        Ok(map.get(&txn, key).and_then(|out| T::try_from(out).ok()))
    }

    /// Set a value in the root Map of a document
    pub fn set_map_value<V>(doc: &Doc, key: &str, value: V) -> CrdtResult<()>
    where
        V: Into<Any>,
    {
        // Get map before transaction to avoid nested transaction
        let map = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();
        map.insert(&mut txn, key, value);
        Ok(())
    }

    // === Static helper methods for direct Map operations ===

    /// Get a bool value from a Map
    pub fn get_map_bool(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<bool> {
        map.get(txn, key).and_then(|out| bool::try_from(out).ok())
    }

    /// Get a string value from a Map
    pub fn get_map_string(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<String> {
        map.get(txn, key).and_then(|out| String::try_from(out).ok())
    }

    /// Get an i64 value from a Map
    pub fn get_map_i64(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<i64> {
        map.get(txn, key).and_then(|out| i64::try_from(out).ok())
    }

    /// Get a nested Map from a Map
    pub fn get_nested_map(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<MapRef> {
        map.get(txn, key).and_then(|out| MapRef::try_from(out).ok())
    }

    /// Insert or update a string field in a Map
    pub fn set_map_string(
        map: &MapRef,
        txn: &mut TransactionMut,
        key: impl Into<String>,
        value: impl Into<String>,
    ) {
        map.insert(txn, key.into(), value.into());
    }

    /// Insert or update an i64 field in a Map
    pub fn set_map_i64(map: &MapRef, txn: &mut TransactionMut, key: impl Into<String>, value: i64) {
        // Explicitly wrap in BigInt to avoid automatic conversion to Number (float)
        map.insert(txn, key.into(), Any::BigInt(value));
    }

    /// Insert or update a bool field in a Map
    pub fn set_map_bool(
        map: &MapRef,
        txn: &mut TransactionMut,
        key: impl Into<String>,
        value: bool,
    ) {
        map.insert(txn, key.into(), value);
    }

    /// Get a nested Map from a parent Map, creating it if it doesn't exist
    pub fn get_or_create_nested_map(
        parent: &MapRef,
        txn: &mut TransactionMut,
        key: impl Into<String>,
    ) -> MapRef {
        let key_str = key.into();
        // Check if map already exists
        if let Some(existing) = parent.get(txn, &key_str)
            && let Ok(m) = MapRef::try_from(existing)
        {
            return m;
        }
        // Create and insert new empty map
        let empty_prelim: MapPrelim = MapPrelim::from([("_", Any::Null)]);
        let new_map: MapRef = parent.insert(txn, key_str.as_str(), empty_prelim);
        // Remove the temporary key
        new_map.remove(txn, "_");
        new_map
    }

    /// Check if a Map contains a key
    pub fn map_contains_key(map: &MapRef, txn: &impl ReadTxn, key: &str) -> bool {
        map.contains_key(txn, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load_document() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create and save a document using helper method
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Test Channel").expect("set name");
        CrdtManager::set_map_value(&doc, "count", 42i64).expect("set count");

        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Load and verify using helper methods
        let loaded_doc = manager
            .load_document("channel:ch-1:doc")
            .await
            .expect("load document");

        let name: String = CrdtManager::get_map_value(&loaded_doc, "name")
            .expect("get value")
            .expect("name exists");
        assert_eq!(name, "Test Channel");

        let count: i64 = CrdtManager::get_map_value(&loaded_doc, "count")
            .expect("get value")
            .expect("count exists");
        assert_eq!(count, 42);
    }

    #[tokio::test]
    async fn test_list_documents() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create multiple documents
        for i in 1..=3 {
            let doc = Doc::new();
            manager
                .save_document(
                    &format!("channel:ch-{}:doc", i),
                    "channel",
                    &format!("ch-{}", i),
                    &doc,
                )
                .await
                .expect("save document");
        }

        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 3);
        assert!(docs.contains(&"channel:ch-1:doc".to_string()));
        assert!(docs.contains(&"channel:ch-2:doc".to_string()));
        assert!(docs.contains(&"channel:ch-3:doc".to_string()));
    }

    #[tokio::test]
    async fn test_delete_document() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create and save a document
        let doc = Doc::new();
        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Verify it exists
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 1);

        // Delete it
        manager
            .delete_document("channel:ch-1:doc")
            .await
            .expect("delete document");

        // Verify it's gone
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 0);
    }

    #[tokio::test]
    async fn test_document_not_found() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Try to load non-existent document with invalid format
        let result = manager.load_document("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CrdtError::InvalidDocumentId(_))));

        // Try to load non-existent document with valid format
        let result = manager.load_document("channel:ch-1:doc").await;
        assert!(result.is_err());
        assert!(matches!(result, Err(CrdtError::DocumentNotFound(_))));
    }

    #[tokio::test]
    async fn test_concurrent_saves() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Save same document multiple times (simulating concurrent updates)
        for i in 1..=5 {
            let doc = Doc::new();
            CrdtManager::set_map_value(&doc, "version", i as i64).expect("set version");

            manager
                .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
                .await
                .expect("save document");
        }

        // Last write should win
        let loaded_doc = manager
            .load_document("channel:ch-1:doc")
            .await
            .expect("load document");

        let version: i64 = CrdtManager::get_map_value(&loaded_doc, "version")
            .expect("get value")
            .expect("version exists");
        assert_eq!(version, 5);
    }

    #[tokio::test]
    async fn test_metadata_persistence() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        let doc = Doc::new();
        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Verify metadata file exists (using hex-encoded filename)
        let hex_doc_id = hex::encode("channel:ch-1:doc".as_bytes());
        let meta_path = storage_path.join(format!("crdt/channel/{}.meta", hex_doc_id));
        assert!(meta_path.exists());

        // Read and verify metadata
        let meta_json = tokio::fs::read_to_string(meta_path)
            .await
            .expect("read metadata");
        let metadata: DocumentMetadata = serde_json::from_str(&meta_json).expect("parse metadata");

        assert_eq!(metadata.doc_id, "channel:ch-1:doc");
        assert_eq!(metadata.entity_type, "channel");
        assert_eq!(metadata.entity_id, "ch-1");
    }

    #[tokio::test]
    async fn test_apply_update() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create initial document
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Initial").expect("set name");

        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Simulate peer loading the document and making an update
        let doc_peer = manager
            .load_document("channel:ch-1:metadata")
            .await
            .expect("load document");

        // Peer makes a change
        CrdtManager::set_map_value(&doc_peer, "name", "Updated").expect("set name");

        // Encode the peer's state as update
        let update_bytes = doc_peer
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Apply the update to a fresh load
        manager
            .apply_update("channel:ch-1:metadata", &update_bytes)
            .await
            .expect("apply update");

        // Load and verify
        let loaded_doc = manager
            .load_document("channel:ch-1:metadata")
            .await
            .expect("load document");

        let name: String = CrdtManager::get_map_value(&loaded_doc, "name")
            .expect("get value")
            .expect("name exists");
        assert_eq!(name, "Updated");
    }

    #[tokio::test]
    async fn test_merge_updates() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create initial document first
        let doc_initial = Doc::new();
        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc_initial)
            .await
            .expect("save initial document");

        // Create three updates from different peers
        let doc1 = Doc::new();
        CrdtManager::set_map_value(&doc1, "field1", "value1").expect("set field1");
        let update1 = doc1
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        let doc2 = Doc::new();
        CrdtManager::set_map_value(&doc2, "field2", "value2").expect("set field2");
        let update2 = doc2
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        let doc3 = Doc::new();
        CrdtManager::set_map_value(&doc3, "field3", "value3").expect("set field3");
        let update3 = doc3
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Merge all updates
        let merged_doc = manager
            .merge_updates("channel:ch-1:metadata", vec![update1, update2, update3])
            .await
            .expect("merge updates");

        // Verify all fields are present
        let field1: String = CrdtManager::get_map_value(&merged_doc, "field1")
            .expect("get value")
            .expect("field1 exists");
        let field2: String = CrdtManager::get_map_value(&merged_doc, "field2")
            .expect("get value")
            .expect("field2 exists");
        let field3: String = CrdtManager::get_map_value(&merged_doc, "field3")
            .expect("get value")
            .expect("field3 exists");

        assert_eq!(field1, "value1");
        assert_eq!(field2, "value2");
        assert_eq!(field3, "value3");
    }

    #[tokio::test]
    async fn test_mark_deleted() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create a document
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Test").expect("set name");

        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Mark as deleted
        manager
            .mark_deleted("channel:ch-1:metadata", "deleter-id")
            .await
            .expect("mark deleted");

        // Verify tombstone
        let is_deleted = manager
            .is_deleted("channel:ch-1:metadata")
            .await
            .expect("check deleted");
        assert!(is_deleted);
    }

    #[tokio::test]
    async fn test_is_deleted_false_for_non_deleted() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create a normal document
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Test").expect("set name");

        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Should not be marked as deleted
        let is_deleted = manager
            .is_deleted("channel:ch-1:metadata")
            .await
            .expect("check deleted");
        assert!(!is_deleted);
    }

    #[tokio::test]
    async fn test_doc_id_validation() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        let doc = Doc::new();

        // Should fail: doc_id doesn't start with entity_type
        let result = manager
            .save_document("wrong:ch-1:doc", "channel", "ch-1", &doc)
            .await;
        assert!(matches!(result, Err(CrdtError::InvalidDocumentId(_))));

        // Should succeed: doc_id starts with entity_type
        let result = manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_apply_update_requires_existing_doc() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "field", "value").expect("set field");
        let update_bytes = doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Should fail: document doesn't exist
        let result = manager
            .apply_update("channel:ch-1:doc", &update_bytes)
            .await;
        assert!(matches!(result, Err(CrdtError::DocumentNotFound(_))));
    }

    #[tokio::test]
    async fn test_merge_updates_requires_existing_doc() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "field", "value").expect("set field");
        let update_bytes = doc
            .transact()
            .encode_state_as_update_v1(&yrs::StateVector::default());

        // Should fail: document doesn't exist
        let result = manager
            .merge_updates("channel:ch-1:doc", vec![update_bytes])
            .await;
        assert!(matches!(result, Err(CrdtError::DocumentNotFound(_))));
    }

    #[tokio::test]
    async fn test_update_size_limits() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create initial document
        let doc = Doc::new();
        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Create a large update (11MB)
        let large_update = vec![0u8; 11 * 1024 * 1024];

        // Should fail: update too large
        let result = manager
            .apply_update("channel:ch-1:doc", &large_update)
            .await;
        assert!(matches!(result, Err(CrdtError::Encoding(_))));

        // Should also fail in merge_updates
        let result = manager
            .merge_updates("channel:ch-1:doc", vec![large_update])
            .await;
        assert!(matches!(result, Err(CrdtError::Encoding(_))));
    }

    #[tokio::test]
    async fn test_metadata_version_progression() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        let doc = Doc::new();

        // First save: version should be 1
        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        let hex_doc_id = hex::encode("channel:ch-1:doc".as_bytes());
        let meta_path = storage_path.join(format!("crdt/channel/{}.meta", hex_doc_id));
        let meta_json = tokio::fs::read_to_string(&meta_path)
            .await
            .expect("read metadata");
        let metadata: DocumentMetadata = serde_json::from_str(&meta_json).expect("parse metadata");
        assert_eq!(metadata.version, 1, "First save should be version 1");

        // Second save: version should be 2
        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        let meta_json = tokio::fs::read_to_string(&meta_path)
            .await
            .expect("read metadata");
        let metadata: DocumentMetadata = serde_json::from_str(&meta_json).expect("parse metadata");
        assert_eq!(metadata.version, 2, "Second save should be version 2");

        // Third save: version should be 3
        manager
            .save_document("channel:ch-1:doc", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        let meta_json = tokio::fs::read_to_string(&meta_path)
            .await
            .expect("read metadata");
        let metadata: DocumentMetadata = serde_json::from_str(&meta_json).expect("parse metadata");
        assert_eq!(metadata.version, 3, "Third save should be version 3");
    }
}

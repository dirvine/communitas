// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use super::{CrdtError, CrdtResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
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
        fs::create_dir_all(&crdt_dir).map_err(|e| {
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
        // Ensure entity directory exists
        let entity_dir = self.entity_dir(entity_type);
        fs::create_dir_all(&entity_dir).map_err(|e| {
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
                .map_err(|e| CrdtError::FileSystem(format!("Failed to read metadata: {}", e)))?;
            serde_json::from_str(&meta_json)
                .map_err(|e| CrdtError::Serialization(format!("Invalid metadata JSON: {}", e)))?
        } else {
            let now = chrono::Utc::now().timestamp();
            DocumentMetadata {
                doc_id: doc_id.to_string(),
                entity_type: entity_type.to_string(),
                entity_id: entity_id.to_string(),
                version: 1,
                created_at: now,
                updated_at: now,
            }
        };

        // Update metadata
        metadata.updated_at = chrono::Utc::now().timestamp();
        metadata.version += 1;

        // Write files atomically (temp + rename)
        // Create temp files in same directory to avoid cross-filesystem rename (EXDEV error)
        let yrs_temp = yrs_path.with_extension("yrs.tmp");
        let meta_temp = meta_path.with_extension("meta.tmp");

        fs::write(&yrs_temp, &state)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to write Yrs state: {}", e)))?;

        let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            CrdtError::Serialization(format!("Failed to serialize metadata: {}", e))
        })?;
        fs::write(&meta_temp, meta_json)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to write metadata: {}", e)))?;

        // Atomic rename (same filesystem - no EXDEV error)
        fs::rename(&yrs_temp, &yrs_path)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to rename Yrs file: {}", e)))?;
        fs::rename(&meta_temp, &meta_path)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to rename metadata file: {}", e)))?;

        Ok(())
    }

    /// Load a Yrs document from the filesystem
    pub async fn load_document(&self, doc_id: &str) -> CrdtResult<Doc> {
        // Find the document by searching all entity type directories
        let crdt_dir = self.storage_dir.join("crdt");

        if !crdt_dir.exists() {
            return Err(CrdtError::DocumentNotFound(doc_id.to_string()));
        }

        // Search all entity type directories
        for entry in fs::read_dir(&crdt_dir)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read CRDT directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                CrdtError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;

            if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }

            let entity_type = entry.file_name().to_string_lossy().to_string();
            let (yrs_path, _) = self.doc_paths(&entity_type, doc_id);

            if yrs_path.exists() {
                // Load and decode
                let state_bytes = fs::read(&yrs_path).map_err(|e| {
                    CrdtError::FileSystem(format!("Failed to read Yrs state: {}", e))
                })?;

                let update = Update::decode_v1(&state_bytes).map_err(|e| {
                    CrdtError::Deserialization(format!("Failed to decode Yrs state: {}", e))
                })?;

                let doc = Doc::new();
                {
                    let mut txn = doc.transact_mut();
                    txn.apply_update(update);
                }

                return Ok(doc);
            }
        }

        Err(CrdtError::DocumentNotFound(doc_id.to_string()))
    }

    /// List all document IDs for a given entity type
    pub async fn list_documents(&self, entity_type: &str) -> CrdtResult<Vec<String>> {
        let entity_dir = self.entity_dir(entity_type);

        if !entity_dir.exists() {
            return Ok(Vec::new());
        }

        let mut doc_ids = Vec::new();

        for entry in fs::read_dir(&entity_dir)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read entity directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                CrdtError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;

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

    /// Delete a document from the filesystem
    pub async fn delete_document(&self, doc_id: &str) -> CrdtResult<()> {
        // Find and delete from all entity type directories
        let crdt_dir = self.storage_dir.join("crdt");

        if !crdt_dir.exists() {
            return Err(CrdtError::DocumentNotFound(doc_id.to_string()));
        }

        let mut found = false;

        for entry in fs::read_dir(&crdt_dir)
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read CRDT directory: {}", e)))?
        {
            let entry = entry.map_err(|e| {
                CrdtError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;

            if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }

            let entity_type = entry.file_name().to_string_lossy().to_string();
            let (yrs_path, meta_path) = self.doc_paths(&entity_type, doc_id);

            if yrs_path.exists() {
                fs::remove_file(&yrs_path).map_err(|e| {
                    CrdtError::FileSystem(format!("Failed to delete Yrs file: {}", e))
                })?;

                if meta_path.exists() {
                    fs::remove_file(&meta_path).map_err(|e| {
                        CrdtError::FileSystem(format!("Failed to delete metadata file: {}", e))
                    })?;
                }

                found = true;
                break;
            }
        }

        if !found {
            return Err(CrdtError::DocumentNotFound(doc_id.to_string()));
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
            .save_document("doc-1", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Load and verify using helper methods
        let loaded_doc = manager.load_document("doc-1").await.expect("load document");

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
                .save_document(&format!("doc-{}", i), "channel", &format!("ch-{}", i), &doc)
                .await
                .expect("save document");
        }

        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 3);
        assert!(docs.contains(&"doc-1".to_string()));
        assert!(docs.contains(&"doc-2".to_string()));
        assert!(docs.contains(&"doc-3".to_string()));
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
            .save_document("doc-1", "channel", "ch-1", &doc)
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
            .delete_document("doc-1")
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

        // Try to load non-existent document
        let result = manager.load_document("nonexistent").await;
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
                .save_document("doc-1", "channel", "ch-1", &doc)
                .await
                .expect("save document");
        }

        // Last write should win
        let loaded_doc = manager.load_document("doc-1").await.expect("load document");

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
            .save_document("doc-1", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Verify metadata file exists (using hex-encoded filename)
        let hex_doc_id = hex::encode("doc-1".as_bytes());
        let meta_path = storage_path.join(format!("crdt/channel/{}.meta", hex_doc_id));
        assert!(meta_path.exists());

        // Read and verify metadata
        let meta_json = fs::read_to_string(meta_path).expect("read metadata");
        let metadata: DocumentMetadata = serde_json::from_str(&meta_json).expect("parse metadata");

        assert_eq!(metadata.doc_id, "doc-1");
        assert_eq!(metadata.entity_type, "channel");
        assert_eq!(metadata.entity_id, "ch-1");
    }
}

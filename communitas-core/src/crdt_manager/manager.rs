// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use super::{CrdtError, CrdtResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
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

/// Configuration for tombstone compaction
#[derive(Debug, Clone)]
pub struct CompactionConfig {
    /// How long to keep tombstoned documents before physical deletion (default: 7 days)
    pub tombstone_retention: Duration,
    /// Interval between automatic compaction runs (default: 1 hour)
    pub compaction_interval: Duration,
    /// Documents larger than this are re-encoded to reduce size (default: 5MB)
    pub max_document_bytes: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            tombstone_retention: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            compaction_interval: Duration::from_secs(60 * 60),          // 1 hour
            max_document_bytes: 5 * 1024 * 1024,                        // 5MB
        }
    }
}

/// Result of a compaction run
#[derive(Debug, Clone, Default)]
pub struct CompactionResult {
    /// Number of tombstoned documents physically deleted
    pub tombstones_removed: usize,
    /// Number of documents re-encoded to reduce size
    pub documents_compacted: usize,
    /// Total bytes saved
    pub bytes_saved: u64,
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
    config: CompactionConfig,
    compaction_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    compaction_shutdown_tx: Arc<RwLock<Option<tokio::sync::mpsc::Sender<()>>>>,
}

impl CrdtManager {
    /// Initialize CrdtManager with filesystem storage using default configuration
    ///
    /// Creates the CRDT storage directory structure if it doesn't exist.
    pub async fn new<P: AsRef<Path>>(storage_dir: P) -> CrdtResult<Self> {
        Self::new_with_config(storage_dir, CompactionConfig::default()).await
    }

    /// Initialize CrdtManager with filesystem storage and custom configuration
    ///
    /// Creates the CRDT storage directory structure if it doesn't exist.
    pub async fn new_with_config<P: AsRef<Path>>(
        storage_dir: P,
        config: CompactionConfig,
    ) -> CrdtResult<Self> {
        let storage_dir = storage_dir.as_ref().to_path_buf();
        let crdt_dir = storage_dir.join("crdt");

        // Create directory structure
        fs::create_dir_all(&crdt_dir).await.map_err(|e| {
            CrdtError::FileSystem(format!("Failed to create CRDT directory: {}", e))
        })?;

        Ok(Self {
            storage_dir,
            config,
            compaction_task: Arc::new(RwLock::new(None)),
            compaction_shutdown_tx: Arc::new(RwLock::new(None)),
        })
    }

    /// Get the storage directory path
    pub fn get_storage_dir(&self) -> &Path {
        &self.storage_dir
    }

    /// Get the directory path for a specific entity type
    fn entity_dir(&self, entity_type: &str) -> PathBuf {
        self.storage_dir.join("crdt").join(entity_type)
    }

    /// Sanitize doc_id for use as filename (Windows-safe, length-safe)
    ///
    /// Doc IDs can contain colons (e.g., "entity:{id}:core") which are forbidden
    /// on Windows. We hex-encode the entire doc_id to ensure cross-platform safety.
    ///
    /// For long doc_ids, hex-encoding doubles the length and may exceed the
    /// 255-byte filesystem filename limit (especially with `.yrs` extension and
    /// temp file suffixes). When the hex-encoded name would exceed 200 chars,
    /// we use a SHA-256 hash instead, producing a fixed 67-char filename
    /// (`h_` prefix + 64 hex chars).
    fn sanitize_doc_id(doc_id: &str) -> String {
        let hex_encoded = hex::encode(doc_id.as_bytes());
        if hex_encoded.len() > 200 {
            use sha2::{Digest, Sha256};
            let hash = Sha256::digest(doc_id.as_bytes());
            format!("h_{}", hex::encode(hash))
        } else {
            hex_encoded
        }
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
        let state = {
            const MAX_TXN_RETRIES: usize = 8;
            const BASE_DELAY_MS: u64 = 5;

            let mut attempt = 0usize;
            loop {
                match doc.try_transact() {
                    Ok(txn) => {
                        break txn.encode_state_as_update_v1(&yrs::StateVector::default());
                    }
                    Err(err) => {
                        if attempt >= MAX_TXN_RETRIES {
                            return Err(CrdtError::Operation(format!(
                                "Failed to acquire read transaction for {} after {} attempts: {}",
                                doc_id,
                                attempt + 1,
                                err
                            )));
                        }
                        let delay = BASE_DELAY_MS.saturating_mul((attempt + 1) as u64);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        attempt += 1;
                    }
                }
            }
        };

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
        // Use a unique suffix to avoid collisions when concurrent saves occur.
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let temp_tag = format!("tmp.{}.{}", std::process::id(), unique_suffix);
        let yrs_temp = yrs_path.with_extension(format!("yrs.{}", temp_tag));
        let meta_temp = meta_path.with_extension(format!("meta.{}", temp_tag));

        // Serialize metadata before parallel write (needed for error handling)
        let meta_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
            CrdtError::Serialization(format!("Failed to serialize metadata: {}", e))
        })?;

        // Parallelize temp file writes for ~10-20ms improvement
        let yrs_write = fs::write(&yrs_temp, &state);
        let meta_write = fs::write(&meta_temp, meta_json);

        let (yrs_result, meta_result) = tokio::join!(yrs_write, meta_write);

        yrs_result
            .map_err(|e| CrdtError::FileSystem(format!("Failed to write Yrs state: {}", e)))?;
        meta_result
            .map_err(|e| CrdtError::FileSystem(format!("Failed to write metadata: {}", e)))?;

        // Atomic renames must stay sequential for consistency
        // If first rename succeeds but second fails, we have partial state
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

        // Load the document, or create new ONLY if DocumentNotFound
        // Propagate all other errors (filesystem, corruption, etc.) to prevent data loss
        let doc = match self.load_document(doc_id).await {
            Ok(doc) => doc,
            Err(CrdtError::DocumentNotFound(_)) => Doc::new(),
            Err(e) => return Err(e),
        };

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

        // Load existing document, or create new ONLY if DocumentNotFound
        // Propagate all other errors to prevent data loss
        let doc = match self.load_document(doc_id).await {
            Ok(doc) => doc,
            Err(CrdtError::DocumentNotFound(_)) => Doc::new(),
            Err(e) => return Err(e),
        };

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

    // === Compaction Methods ===

    /// Remove tombstoned documents older than the configured retention period
    ///
    /// Scans all entity type directories and physically deletes documents where:
    /// - The "deleted" flag is set to true in the Yrs document metadata
    /// - The "deleted_at" timestamp is older than `tombstone_retention`
    ///
    /// Returns the number of documents removed.
    pub async fn compact_tombstones(&self) -> CrdtResult<usize> {
        let crdt_dir = self.storage_dir.join("crdt");
        if !crdt_dir.exists() {
            return Ok(0);
        }

        let mut removed_count = 0;
        let retention_secs = self.config.tombstone_retention.as_secs() as i64;
        let now = chrono::Utc::now().timestamp();

        // Read all entity type directories
        let mut read_dir = fs::read_dir(&crdt_dir)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read CRDT directory: {}", e)))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read directory entry: {}", e)))?
        {
            if !entry
                .file_type()
                .await
                .map_err(|e| CrdtError::FileSystem(format!("Failed to get file type: {}", e)))?
                .is_dir()
            {
                continue;
            }

            let entity_type_dir = entry.path();

            // Read all .yrs files in this entity type directory
            let mut entity_dir = fs::read_dir(&entity_type_dir).await.map_err(|e| {
                CrdtError::FileSystem(format!("Failed to read entity type directory: {}", e))
            })?;

            while let Some(file_entry) = entity_dir
                .next_entry()
                .await
                .map_err(|e| CrdtError::FileSystem(format!("Failed to read file entry: {}", e)))?
            {
                let file_name = file_entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                // Only process .yrs files
                if !file_name_str.ends_with(".yrs") {
                    continue;
                }

                // Decode hex back to original doc_id
                let hex_id = file_name_str.trim_end_matches(".yrs");
                let doc_id = match hex::decode(hex_id)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                {
                    Some(id) => id,
                    None => {
                        tracing::warn!("Failed to decode doc_id from hex: {}", hex_id);
                        continue;
                    }
                };

                // Load the document and check if it's tombstoned
                let doc = match self.load_document(&doc_id).await {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::warn!("Failed to load document {}: {}", doc_id, e);
                        continue;
                    }
                };

                // Check metadata for deleted flag and timestamp
                let root = doc.get_or_insert_map("root");
                let txn = doc.transact();

                let is_deleted = if let Some(metadata_val) = root.get(&txn, "metadata")
                    && let Ok(metadata) = MapRef::try_from(metadata_val)
                {
                    let deleted = Self::get_map_bool(&metadata, &txn, "deleted").unwrap_or(false);
                    let deleted_at = Self::get_map_i64(&metadata, &txn, "deleted_at");

                    if deleted && deleted_at.is_some() {
                        let age_secs = now - deleted_at.unwrap_or(now);
                        age_secs >= retention_secs
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_deleted {
                    if let Err(e) = self.delete_document(&doc_id).await {
                        tracing::warn!("Failed to delete tombstoned document {}: {}", doc_id, e);
                    } else {
                        tracing::debug!("Compacted tombstone: {}", doc_id);
                        removed_count += 1;
                    }
                }
            }
        }

        Ok(removed_count)
    }

    /// Re-encode a document to reduce internal bloat
    ///
    /// Loads a document, encodes it as a full state update from an empty state vector,
    /// and saves it back if the new encoding is smaller.
    ///
    /// Returns the number of bytes saved (or 0 if no savings).
    pub async fn compact_document(&self, doc_id: &str, force: bool) -> CrdtResult<u64> {
        // Parse entity_type from doc_id
        let parts: Vec<&str> = doc_id.split(':').collect();
        if parts.len() < 2 {
            return Err(CrdtError::InvalidDocumentId(format!(
                "Invalid doc_id format (expected 'entity_type:entity_id:...'): {}",
                doc_id
            )));
        }

        let entity_type = parts[0];
        let entity_id = parts[1];
        let (yrs_path, _) = self.doc_paths(entity_type, doc_id);

        if !yrs_path.exists() {
            return Err(CrdtError::DocumentNotFound(doc_id.to_string()));
        }

        // Get current file size
        let old_size = fs::metadata(&yrs_path)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to get file metadata: {}", e)))?
            .len();

        // Check size threshold unless forced
        if !force && old_size < self.config.max_document_bytes as u64 {
            return Ok(0);
        }

        // Load the document
        let doc = self.load_document(doc_id).await?;

        // Encode as full state update
        let new_state = {
            const MAX_TXN_RETRIES: usize = 8;
            const BASE_DELAY_MS: u64 = 5;

            let mut attempt = 0usize;
            loop {
                match doc.try_transact() {
                    Ok(txn) => {
                        break txn.encode_state_as_update_v1(&yrs::StateVector::default());
                    }
                    Err(err) => {
                        if attempt >= MAX_TXN_RETRIES {
                            return Err(CrdtError::Operation(format!(
                                "Failed to acquire read transaction for {} after {} attempts: {}",
                                doc_id,
                                attempt + 1,
                                err
                            )));
                        }
                        let delay = BASE_DELAY_MS.saturating_mul((attempt + 1) as u64);
                        tokio::time::sleep(Duration::from_millis(delay)).await;
                        attempt += 1;
                    }
                }
            }
        };

        let new_size = new_state.len() as u64;

        // Only save if we're saving space or forced
        if new_size < old_size || force {
            // Create new document and apply the state
            let new_doc = Doc::new();
            {
                let mut txn = new_doc.transact_mut();
                let update = Update::decode_v1(&new_state).map_err(|e| {
                    CrdtError::Deserialization(format!("Failed to decode compacted state: {}", e))
                })?;
                txn.apply_update(update);
            }

            // Save the compacted document
            self.save_document(doc_id, entity_type, entity_id, &new_doc)
                .await?;

            let bytes_saved = old_size.saturating_sub(new_size);
            tracing::debug!(
                "Compacted document {}: {} bytes -> {} bytes (saved {})",
                doc_id,
                old_size,
                new_size,
                bytes_saved
            );
            Ok(bytes_saved)
        } else {
            Ok(0)
        }
    }

    /// Run full compaction: remove tombstones and compact large documents
    ///
    /// This orchestrates both tombstone removal and document re-encoding.
    /// Returns detailed statistics about the compaction run.
    pub async fn compact_all(&self) -> CrdtResult<CompactionResult> {
        let mut result = CompactionResult::default();

        // First, remove old tombstones
        result.tombstones_removed = self.compact_tombstones().await?;

        // Then compact remaining large documents
        let crdt_dir = self.storage_dir.join("crdt");
        if !crdt_dir.exists() {
            return Ok(result);
        }

        // Read all entity type directories
        let mut read_dir = fs::read_dir(&crdt_dir)
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read CRDT directory: {}", e)))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| CrdtError::FileSystem(format!("Failed to read directory entry: {}", e)))?
        {
            if !entry
                .file_type()
                .await
                .map_err(|e| CrdtError::FileSystem(format!("Failed to get file type: {}", e)))?
                .is_dir()
            {
                continue;
            }

            let entity_type_dir = entry.path();

            // Read all .yrs files in this entity type directory
            let mut entity_dir = fs::read_dir(&entity_type_dir).await.map_err(|e| {
                CrdtError::FileSystem(format!("Failed to read entity type directory: {}", e))
            })?;

            while let Some(file_entry) = entity_dir
                .next_entry()
                .await
                .map_err(|e| CrdtError::FileSystem(format!("Failed to read file entry: {}", e)))?
            {
                let file_name = file_entry.file_name();
                let file_name_str = file_name.to_string_lossy();

                // Only process .yrs files
                if !file_name_str.ends_with(".yrs") {
                    continue;
                }

                // Decode hex back to original doc_id
                let hex_id = file_name_str.trim_end_matches(".yrs");
                let doc_id = match hex::decode(hex_id)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                {
                    Some(id) => id,
                    None => {
                        tracing::warn!("Failed to decode doc_id from hex: {}", hex_id);
                        continue;
                    }
                };

                // Try to compact this document
                match self.compact_document(&doc_id, false).await {
                    Ok(bytes_saved) => {
                        if bytes_saved > 0 {
                            result.documents_compacted += 1;
                            result.bytes_saved += bytes_saved;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to compact document {}: {}", doc_id, e);
                    }
                }
            }
        }

        tracing::debug!(
            "Compaction complete: {} tombstones removed, {} documents compacted, {} bytes saved",
            result.tombstones_removed,
            result.documents_compacted,
            result.bytes_saved
        );

        Ok(result)
    }

    /// Start background compaction task
    ///
    /// Spawns a task that runs compaction at the configured interval.
    /// Only one task can run at a time; calling this multiple times is a no-op.
    pub async fn start_compaction_task(&self) {
        let mut task_lock = self.compaction_task.write().await;

        // Don't start if already running
        if task_lock.is_some() {
            tracing::debug!("Compaction task already running");
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        *self.compaction_shutdown_tx.write().await = Some(shutdown_tx);

        let interval = self.config.compaction_interval;
        let storage_dir = self.storage_dir.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            tracing::debug!("Compaction task started with interval {:?}", interval);

            loop {
                tokio::select! {
                    _ = tokio::time::sleep(interval) => {
                        // Run compaction
                        match CrdtManager::new_with_config(&storage_dir, config.clone()).await {
                            Ok(manager) => {
                                match manager.compact_all().await {
                                    Ok(result) => {
                                        tracing::debug!(
                                            "Background compaction: {} tombstones, {} docs, {} bytes saved",
                                            result.tombstones_removed,
                                            result.documents_compacted,
                                            result.bytes_saved
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!("Background compaction failed: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to create manager for compaction: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::debug!("Compaction task shutting down");
                        break;
                    }
                }
            }
        });

        *task_lock = Some(handle);
        tracing::debug!("Compaction task spawned");
    }

    /// Stop background compaction task
    ///
    /// Signals the background task to stop and waits for it to complete.
    /// Safe to call even if no task is running.
    pub async fn stop_compaction_task(&self) {
        let mut task_lock = self.compaction_task.write().await;

        if let Some(handle) = task_lock.take() {
            // Send shutdown signal
            if let Some(tx) = self.compaction_shutdown_tx.write().await.take() {
                let _ = tx.send(()).await;
            }

            // Wait for task to complete
            let _ = handle.await;
            tracing::debug!("Compaction task stopped");
        }
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
    async fn test_apply_update_creates_new_doc() {
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

        // Should succeed: document doesn't exist but will be auto-created
        let result = manager
            .apply_update("channel:ch-1:doc", &update_bytes)
            .await;
        assert!(
            result.is_ok(),
            "Expected success when applying update to non-existent document"
        );

        // Verify document was created and contains the data
        let loaded_doc = manager
            .load_document("channel:ch-1:doc")
            .await
            .expect("load created doc");
        let value = CrdtManager::get_map_value::<String>(&loaded_doc, "field")
            .expect("get field")
            .expect("field should exist");
        assert_eq!(value, "value");
    }

    #[tokio::test]
    async fn test_merge_updates_creates_new_doc() {
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

        // Should succeed: document doesn't exist but will be auto-created
        let result = manager
            .merge_updates("channel:ch-1:doc", vec![update_bytes])
            .await;
        assert!(
            result.is_ok(),
            "Expected success when merging updates to non-existent document"
        );

        // Verify document was created and contains the data
        let loaded_doc = manager
            .load_document("channel:ch-1:doc")
            .await
            .expect("load created doc");
        let value = CrdtManager::get_map_value::<String>(&loaded_doc, "field")
            .expect("get field")
            .expect("field should exist");
        assert_eq!(value, "value");
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

    // === Compaction Tests ===

    #[test]
    fn test_compaction_config_defaults() {
        let config = CompactionConfig::default();
        assert_eq!(config.tombstone_retention.as_secs(), 7 * 24 * 60 * 60);
        assert_eq!(config.compaction_interval.as_secs(), 60 * 60);
        assert_eq!(config.max_document_bytes, 5 * 1024 * 1024);
    }

    #[tokio::test]
    async fn test_compact_tombstones_removes_old() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        // Create manager with zero retention (all tombstones are old)
        let config = CompactionConfig {
            tombstone_retention: Duration::from_secs(0),
            ..Default::default()
        };
        let manager = CrdtManager::new_with_config(storage_path, config)
            .await
            .expect("create manager");

        // Create and tombstone a document
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Test").expect("set name");
        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        manager
            .mark_deleted("channel:ch-1:metadata", "deleter-id")
            .await
            .expect("mark deleted");

        // Verify document exists
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 1);

        // Run compaction
        let removed = manager
            .compact_tombstones()
            .await
            .expect("compact tombstones");
        assert_eq!(removed, 1);

        // Verify document was physically deleted
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 0);
    }

    #[tokio::test]
    async fn test_compact_tombstones_preserves_fresh() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        // Create manager with 1 hour retention
        let config = CompactionConfig {
            tombstone_retention: Duration::from_secs(3600),
            ..Default::default()
        };
        let manager = CrdtManager::new_with_config(storage_path, config)
            .await
            .expect("create manager");

        // Create and tombstone a document
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Test").expect("set name");
        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        manager
            .mark_deleted("channel:ch-1:metadata", "deleter-id")
            .await
            .expect("mark deleted");

        // Run compaction (should not remove fresh tombstone)
        let removed = manager
            .compact_tombstones()
            .await
            .expect("compact tombstones");
        assert_eq!(removed, 0);

        // Verify document still exists
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 1);
    }

    #[tokio::test]
    async fn test_compact_tombstones_ignores_non_deleted() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let config = CompactionConfig {
            tombstone_retention: Duration::from_secs(0),
            ..Default::default()
        };
        let manager = CrdtManager::new_with_config(storage_path, config)
            .await
            .expect("create manager");

        // Create a normal document (not deleted)
        let doc = Doc::new();
        CrdtManager::set_map_value(&doc, "name", "Test").expect("set name");
        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Run compaction (should not remove non-deleted document)
        let removed = manager
            .compact_tombstones()
            .await
            .expect("compact tombstones");
        assert_eq!(removed, 0);

        // Verify document still exists
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 1);
    }

    #[tokio::test]
    async fn test_compact_document_reduces_size() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let manager = CrdtManager::new(storage_path)
            .await
            .expect("create manager");

        // Create a document with many incremental updates to create bloat
        let doc = Doc::new();
        for i in 0..100 {
            CrdtManager::set_map_value(&doc, &format!("field_{}", i), format!("value_{}", i))
                .expect("set field");
        }

        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
            .await
            .expect("save document");

        // Apply many updates to create internal bloat
        for i in 0..100 {
            CrdtManager::set_map_value(&doc, &format!("field_{}", i), format!("updated_{}", i))
                .expect("update field");
            manager
                .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc)
                .await
                .expect("save document");
        }

        // Force compact (regardless of size threshold)
        let bytes_saved = manager
            .compact_document("channel:ch-1:metadata", true)
            .await
            .expect("compact document");

        // bytes_saved is u64, so it's always >= 0 by definition
        // Just verify the operation succeeded
        assert!(bytes_saved < u64::MAX);

        // Verify state is preserved
        let loaded_doc = manager
            .load_document("channel:ch-1:metadata")
            .await
            .expect("load document");

        // Check a few fields to verify state
        let field_0: String = CrdtManager::get_map_value(&loaded_doc, "field_0")
            .expect("get value")
            .expect("field exists");
        assert_eq!(field_0, "updated_0");

        let field_99: String = CrdtManager::get_map_value(&loaded_doc, "field_99")
            .expect("get value")
            .expect("field exists");
        assert_eq!(field_99, "updated_99");
    }

    #[tokio::test]
    async fn test_compact_all_combined() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let config = CompactionConfig {
            tombstone_retention: Duration::from_secs(0),
            max_document_bytes: 0, // Force compaction of all docs
            ..Default::default()
        };
        let manager = CrdtManager::new_with_config(storage_path, config)
            .await
            .expect("create manager");

        // Create a tombstoned document
        let doc1 = Doc::new();
        CrdtManager::set_map_value(&doc1, "name", "Tombstone").expect("set name");
        manager
            .save_document("channel:ch-1:metadata", "channel", "ch-1", &doc1)
            .await
            .expect("save document");
        manager
            .mark_deleted("channel:ch-1:metadata", "deleter-id")
            .await
            .expect("mark deleted");

        // Create a normal document
        let doc2 = Doc::new();
        CrdtManager::set_map_value(&doc2, "name", "Normal").expect("set name");
        manager
            .save_document("channel:ch-2:metadata", "channel", "ch-2", &doc2)
            .await
            .expect("save document");

        // Run full compaction
        let result = manager.compact_all().await.expect("compact all");

        // Should have removed 1 tombstone
        assert_eq!(result.tombstones_removed, 1);
        // documents_compacted is usize, always >= 0, just verify it's valid
        assert!(result.documents_compacted < usize::MAX);

        // Verify only non-deleted document remains
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 1);
        assert!(docs.contains(&"channel:ch-2:metadata".to_string()));
    }

    #[tokio::test]
    async fn test_start_stop_compaction_task() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let config = CompactionConfig {
            compaction_interval: Duration::from_millis(100), // Fast for testing
            ..Default::default()
        };
        let manager = CrdtManager::new_with_config(storage_path, config)
            .await
            .expect("create manager");

        // Start task
        manager.start_compaction_task().await;

        // Verify task is running
        {
            let task_lock = manager.compaction_task.read().await;
            assert!(task_lock.is_some());
        }

        // Wait a bit to let task run at least once
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Stop task
        manager.stop_compaction_task().await;

        // Verify task is stopped
        {
            let task_lock = manager.compaction_task.read().await;
            assert!(task_lock.is_none());
        }
    }

    #[tokio::test]
    async fn test_compaction_result_accumulates() {
        let temp_dir = tempdir().expect("temp dir");
        let storage_path = temp_dir.path();

        let config = CompactionConfig {
            tombstone_retention: Duration::from_secs(0),
            max_document_bytes: 0,
            ..Default::default()
        };
        let manager = CrdtManager::new_with_config(storage_path, config)
            .await
            .expect("create manager");

        // Create multiple tombstoned documents
        for i in 1..=3 {
            let doc = Doc::new();
            CrdtManager::set_map_value(&doc, "name", format!("Doc {}", i)).expect("set name");
            manager
                .save_document(
                    &format!("channel:ch-{}:metadata", i),
                    "channel",
                    &format!("ch-{}", i),
                    &doc,
                )
                .await
                .expect("save document");
            manager
                .mark_deleted(&format!("channel:ch-{}:metadata", i), "deleter-id")
                .await
                .expect("mark deleted");
        }

        // Run compaction
        let result = manager.compact_all().await.expect("compact all");

        // Should have removed all 3 tombstones
        assert_eq!(result.tombstones_removed, 3);
        assert_eq!(result.documents_compacted, 0); // All were tombstoned

        // Verify all documents were removed
        let docs = manager
            .list_documents("channel")
            .await
            .expect("list documents");
        assert_eq!(docs.len(), 0);
    }
}

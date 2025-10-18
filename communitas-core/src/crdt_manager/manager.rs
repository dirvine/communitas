// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use super::{CrdtError, CrdtResult};
use chrono::Utc;
use deadpool_sqlite::{Config, Pool, Runtime};
use rusqlite::params;
use std::path::Path;
use yrs::{Any, Doc, Map, MapPrelim, MapRef, Out, ReadTxn, Transact, TransactionMut};
use yrs::updates::encoder::Encoder;

/// SQL schema for CRDT persistence
const SCHEMA_SQL: &str = include_str!("schema.sql");

/// Manages CRDT documents with SQLite persistence via deadpool
pub struct CrdtManager {
    pool: Pool,
}

impl CrdtManager {
    /// Initialize CrdtManager with local SQLite database using connection pool
    pub async fn new<P: AsRef<Path>>(db_path: P) -> CrdtResult<Self> {
        let cfg = Config::new(db_path.as_ref());
        let pool = cfg.create_pool(Runtime::Tokio1)
            .map_err(|e| CrdtError::Pool(e.to_string()))?;

        // Initialize schema
        let conn = pool.get().await?;
        conn.interact(|conn| {
            // Enable WAL mode for better concurrency
            conn.pragma_update(None, "journal_mode", "WAL")
                .map_err(|e| CrdtError::SchemaInit(format!("WAL mode failed: {}", e)))?;

            // Set busy timeout for better handling of locked databases
            conn.pragma_update(None, "busy_timeout", 5000)
                .map_err(|e| CrdtError::SchemaInit(format!("busy_timeout failed: {}", e)))?;

            // Execute schema
            for statement in SCHEMA_SQL.split(';').filter(|s| !s.trim().is_empty()) {
                conn.execute(statement, [])
                    .map_err(|e| CrdtError::SchemaInit(e.to_string()))?;
            }

            Ok::<_, CrdtError>(())
        }).await??;

        Ok(Self { pool })
    }

    /// Save a Yrs document to the database
    pub async fn save_document(
        &self,
        doc_id: &str,
        entity_type: &str,
        entity_id: &str,
        doc: &Doc,
    ) -> CrdtResult<()> {
        // Encode state in a scope to drop the transaction before await
        // This operation can cause stack overflow for very large documents
        let state = {
            let txn = doc.transact();

            // Use encode_diff to avoid encoding the entire document state at once
            // This is safer than encode_state_as_update_v1 for large documents
            let mut encoder = yrs::updates::encoder::EncoderV1::new();
            txn.encode_diff(&yrs::StateVector::default(), &mut encoder);
            encoder.to_vec()
        };

        // Check encoded size to prevent database issues (10MB limit for encoded data)
        const MAX_ENCODED_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if state.len() > MAX_ENCODED_SIZE {
            return Err(CrdtError::encoding_error(format!(
                "Encoded document too large: {} bytes (max: {} bytes) for doc_id: {}",
                state.len(), MAX_ENCODED_SIZE, doc_id
            )));
        }
        let version = 1i64;
        let now = Utc::now().timestamp();

        let doc_id = doc_id.to_string();
        let entity_type = entity_type.to_string();
        let entity_id = entity_id.to_string();

        let conn = self.pool.get().await?;
        conn.interact(move |conn| {
            conn.execute(
                "INSERT OR REPLACE INTO crdt_documents (id, entity_type, entity_id, yrs_state, version, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![doc_id, entity_type, entity_id, state, version, now],
            )
            .map_err(CrdtError::Database)?;
            Ok::<_, CrdtError>(())
        }).await??;

        Ok(())
    }

    /// Load a Yrs document from the database
    pub async fn load_document(&self, doc_id: &str) -> CrdtResult<Doc> {
        let doc_id_str = doc_id.to_string();
        let doc_id_for_error = doc_id_str.clone(); // Clone for error message
        let conn = self.pool.get().await?;

        let result = conn.interact(move |conn| {
            let mut stmt = conn.prepare("SELECT yrs_state FROM crdt_documents WHERE id = ?1")?;
            let mut rows = stmt.query(params![doc_id_str])?;

            if let Some(row) = rows.next()? {
                let state: Vec<u8> = row.get(0)?;
                Ok::<Option<Vec<u8>>, rusqlite::Error>(Some(state))
            } else {
                Ok(None)
            }
        }).await??;

        let doc = Doc::new();
        if let Some(state) = result {
            // Check update size to prevent stack overflow during decoding
            const MAX_UPDATE_SIZE: usize = 10 * 1024 * 1024; // 10MB
            if state.len() > MAX_UPDATE_SIZE {
                return Err(CrdtError::encoding_error(format!(
                    "Update too large: {} bytes (max: {} bytes) for doc_id: {}",
                    state.len(), MAX_UPDATE_SIZE, doc_id_for_error
                )));
            }

            let mut txn = doc.transact_mut();
            use yrs::updates::decoder::Decode;
            let update = yrs::Update::decode_v1(&state)
                .map_err(|e| CrdtError::encoding_error(e.to_string()))?;
            txn.apply_update(update);
        }

        Ok(doc)
    }

    /// List all document IDs for a given entity type
    pub async fn list_documents(&self, entity_type: &str) -> CrdtResult<Vec<String>> {
        let entity_type = entity_type.to_string();
        let conn = self.pool.get().await?;
        
        conn.interact(move |conn| {
            let mut stmt = conn.prepare("SELECT id FROM crdt_documents WHERE entity_type = ?1")?;
            let rows = stmt.query_map(params![entity_type], |row| row.get::<_, String>(0))?;
            
            let mut ids = Vec::new();
            for id_result in rows {
                ids.push(id_result?);
            }
            
            Ok::<Vec<String>, rusqlite::Error>(ids)
        }).await?
        .map_err(CrdtError::Database)
    }

    /// Delete a document from the database
    pub async fn delete_document(&self, doc_id: &str) -> CrdtResult<()> {
        let doc_id = doc_id.to_string();
        let conn = self.pool.get().await?;
        
        conn.interact(move |conn| {
            conn.execute("DELETE FROM crdt_documents WHERE id = ?1", params![doc_id])?;
            Ok::<_, rusqlite::Error>(())
        }).await?
        .map_err(CrdtError::Database)
    }

    /// Get a value from a Map in the document's root
    pub fn get_map_value<T>(doc: &Doc, key: &str) -> CrdtResult<Option<T>>
    where
        T: TryFrom<Out>,
        T::Error: std::fmt::Display,
    {
        let txn = doc.transact();
        let map = doc.get_or_insert_map("root");
        
        match map.get(&txn, key) {
            Some(out) => {
                T::try_from(out)
                    .map(Some)
                    .map_err(|e| CrdtError::type_mismatch(key, std::any::type_name::<T>(), e.to_string()))
            }
            None => Ok(None),
        }
    }

    /// Set a value in a Map in the document's root
    pub fn set_map_value<V>(doc: &Doc, key: &str, value: V) -> CrdtResult<()>
    where
        V: Into<Any>,
    {
        let mut txn = doc.transact_mut();
        let map = doc.get_or_insert_map("root");
        map.insert(&mut txn, key, value);
        Ok(())
    }

    // === Static helper methods for direct Map operations ===

    /// Get a bool value from a Map
    pub fn get_map_bool(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<bool> {
        map.get(txn, key)
            .and_then(|out| bool::try_from(out).ok())
    }

    /// Get a string value from a Map
    pub fn get_map_string(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<String> {
        map.get(txn, key)
            .and_then(|out| String::try_from(out).ok())
    }

    /// Get an i64 value from a Map
    pub fn get_map_i64(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<i64> {
        map.get(txn, key)
            .and_then(|out| i64::try_from(out).ok())
    }

    /// Get a nested Map from a Map
    pub fn get_nested_map(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<MapRef> {
        map.get(txn, key)
            .and_then(|out| MapRef::try_from(out).ok())
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
    pub fn set_map_i64(
        map: &MapRef,
        txn: &mut TransactionMut,
        key: impl Into<String>,
        value: i64,
    ) {
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
        if let Some(existing) = parent.get(txn, &key_str) {
            if let Ok(m) = MapRef::try_from(existing) {
                return m;
            }
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
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let manager = CrdtManager::new(&db_path).await.unwrap();
        
        // Create and save a document
        let doc = Doc::new();
        {
            let mut txn = doc.transact_mut();
            let map = doc.get_or_insert_map("root");
            map.insert(&mut txn, "name", "Test Channel");
            map.insert(&mut txn, "count", 42i32);
        }
        
        manager.save_document("doc-1", "channel", "ch-1", &doc).await.unwrap();
        
        // Load and verify
        let loaded_doc = manager.load_document("doc-1").await.unwrap();
        let txn = loaded_doc.transact();
        let map = loaded_doc.get_or_insert_map("root");
        
        let name: String = map.get(&txn, "name").unwrap().try_into().unwrap();
        assert_eq!(name, "Test Channel");
        
        let count: i32 = map.get(&txn, "count").unwrap().try_into().unwrap();
        assert_eq!(count, 42);
    }

    #[tokio::test]
    async fn test_list_documents() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let manager = CrdtManager::new(&db_path).await.unwrap();
        
        // Create multiple documents
        for i in 1..=3 {
            let doc = Doc::new();
            manager.save_document(
                &format!("doc-{}", i),
                "channel",
                &format!("ch-{}", i),
                &doc
            ).await.unwrap();
        }
        
        let docs = manager.list_documents("channel").await.unwrap();
        assert_eq!(docs.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_document() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let manager = CrdtManager::new(&db_path).await.unwrap();
        
        let doc = Doc::new();
        manager.save_document("doc-1", "channel", "ch-1", &doc).await.unwrap();
        
        manager.delete_document("doc-1").await.unwrap();
        
        let loaded_doc = manager.load_document("doc-1").await.unwrap();
        let txn = loaded_doc.transact();
        let map = loaded_doc.get_or_insert_map("root");
        
        // Should be empty
        assert!(map.get(&txn, "any-key").is_none());
    }
}

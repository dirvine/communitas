use crate::crdt_error::{CrdtError, CrdtResult};
use chrono::Utc;
use libsql::{Builder, Connection, Database, params};
use std::path::Path;
use yrs::updates::encoder::Encoder;
use yrs::{Any, Doc, Map, MapPrelim, MapRef, ReadTxn, Transact};

/// Manages CRDT documents with Turso (libSQL) persistence
pub struct CrdtManager {
    db: Database,
}

impl CrdtManager {
    /// Initialize CrdtManager with local Turso database
    pub async fn new<P: AsRef<Path>>(db_path: P) -> CrdtResult<Self> {
        let db = Builder::new_local(db_path.as_ref())
            .build()
            .await
            .map_err(CrdtError::Database)?;

        // Initialize schema - execute each statement separately
        let schema = include_str!("schema.sql");
        let conn = db.connect().map_err(CrdtError::Database)?;

        // Enable WAL mode for better concurrency (must use query, not execute, as it returns rows)
        let _ = conn
            .query("PRAGMA journal_mode=WAL", ())
            .await
            .map_err(|e| CrdtError::SchemaInit(format!("WAL mode failed: {}", e)))?;

        // Set busy timeout for better handling of locked databases in tests
        let _ = conn
            .query("PRAGMA busy_timeout=5000", ())
            .await
            .map_err(|e| CrdtError::SchemaInit(format!("busy_timeout failed: {}", e)))?;

        // Split schema by semicolons and execute each statement
        for statement in schema.split(';').filter(|s| !s.trim().is_empty()) {
            conn.execute(statement, ())
                .await
                .map_err(|e| CrdtError::SchemaInit(e.to_string()))?;
        }

        Ok(Self { db })
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
        let state = {
            let txn = doc.transact();
            // Use encode_diff with new encoder API
            let mut encoder = yrs::updates::encoder::EncoderV1::new();
            txn.encode_diff(&yrs::StateVector::default(), &mut encoder);
            encoder.to_vec()
        };
        let version = 1i64; // Version tracking can be simplified
        let now = Utc::now().timestamp();

        let conn = self.db.connect()?;
        conn.execute(
                "INSERT OR REPLACE INTO crdt_documents (id, entity_type, entity_id, yrs_state, version, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?)",
                params![doc_id, entity_type, entity_id, state, version, now],
            )
            .await
            .map_err(CrdtError::Database)?;

        Ok(())
    }

    /// Load a Yrs document from the database
    pub async fn load_document(&self, doc_id: &str) -> CrdtResult<Doc> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT yrs_state FROM crdt_documents WHERE id = ?",
                params![doc_id],
            )
            .await
            .map_err(CrdtError::Database)?;

        if let Some(row) = rows.next().await? {
            let state: Vec<u8> = row.get(0)?;
            let doc = Doc::new();
            {
                let mut txn = doc.transact_mut();
                // Decode and apply update using correct API
                use yrs::updates::decoder::Decode;
                let update = yrs::Update::decode_v1(&state)
                    .map_err(|e| CrdtError::encoding_error(e.to_string()))?;
                txn.apply_update(update);
            }
            Ok(doc)
        } else {
            // Document doesn't exist yet, create new empty document
            Ok(Doc::new())
        }
    }

    /// Check if a document exists
    pub async fn _document_exists(&self, doc_id: &str) -> CrdtResult<bool> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM crdt_documents WHERE id = ?",
                params![doc_id],
            )
            .await
            .map_err(CrdtError::Database)?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// Merge an update into an existing document
    pub async fn merge_update(
        &self,
        doc_id: &str,
        entity_type: &str,
        entity_id: &str,
        update: &[u8],
    ) -> CrdtResult<()> {
        let doc = self.load_document(doc_id).await?;
        {
            let mut txn = doc.transact_mut();
            use yrs::updates::decoder::Decode;
            let decoded_update = yrs::Update::decode_v1(update)
                .map_err(|e| CrdtError::encoding_error(e.to_string()))?;
            txn.apply_update(decoded_update);
        }
        self.save_document(doc_id, entity_type, entity_id, &doc)
            .await
    }

    /// Get the state vector for a document (for sync)
    pub async fn _get_state_vector(&self, doc_id: &str) -> CrdtResult<yrs::StateVector> {
        let doc = self.load_document(doc_id).await?;
        let sv = {
            let txn = doc.transact();
            txn.state_vector()
        };
        Ok(sv)
    }

    /// Get the difference between two state vectors (for sync)
    pub async fn _get_diff(
        &self,
        doc_id: &str,
        remote_sv: &yrs::StateVector,
    ) -> CrdtResult<Vec<u8>> {
        let doc = self.load_document(doc_id).await?;
        let update = {
            let txn = doc.transact();
            let mut encoder = yrs::updates::encoder::EncoderV1::new();
            txn.encode_diff(remote_sv, &mut encoder);
            encoder.to_vec()
        };
        Ok(update)
    }

    /// List all documents of a specific entity type
    pub async fn _list_documents_by_type(&self, entity_type: &str) -> CrdtResult<Vec<String>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT id FROM crdt_documents WHERE entity_type = ? ORDER BY updated_at DESC",
                params![entity_type],
            )
            .await
            .map_err(CrdtError::Database)?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Delete a document
    pub async fn _delete_document(&self, doc_id: &str) -> CrdtResult<()> {
        let conn = self.db.connect()?;
        conn.execute("DELETE FROM crdt_documents WHERE id = ?", params![doc_id])
            .await
            .map_err(CrdtError::Database)?;
        Ok(())
    }

    /// Get database connection for custom queries
    pub fn connection(&self) -> CrdtResult<Connection> {
        self.db.connect().map_err(CrdtError::Database)
    }

    // =========================================================================
    // Helper Methods for CRDT Operations
    // =========================================================================

    /// Get a string value from a Map
    pub fn get_map_string(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<String> {
        map.get(txn, key).and_then(|out| String::try_from(out).ok())
    }

    /// Get an i64 value from a Map
    pub fn get_map_i64(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<i64> {
        map.get(txn, key).and_then(|out| i64::try_from(out).ok())
    }

    /// Get a bool value from a Map
    pub fn get_map_bool(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<bool> {
        map.get(txn, key).and_then(|out| bool::try_from(out).ok())
    }

    /// Get a nested Map from a Map
    pub fn get_nested_map(map: &MapRef, txn: &impl ReadTxn, key: &str) -> Option<MapRef> {
        map.get(txn, key).and_then(|out| MapRef::try_from(out).ok())
    }

    /// Insert or update a string field in a Map
    pub fn set_map_string(
        map: &MapRef,
        txn: &mut yrs::TransactionMut,
        key: impl Into<String>,
        value: impl Into<String>,
    ) {
        map.insert(txn, key.into(), value.into());
    }

    /// Insert or update an i64 field in a Map
    pub fn set_map_i64(
        map: &MapRef,
        txn: &mut yrs::TransactionMut,
        key: impl Into<String>,
        value: i64,
    ) {
        // Explicitly wrap in BigInt to avoid automatic conversion to Number (float)
        map.insert(txn, key.into(), Any::BigInt(value));
    }

    /// Insert or update a bool field in a Map
    pub fn set_map_bool(
        map: &MapRef,
        txn: &mut yrs::TransactionMut,
        key: impl Into<String>,
        value: bool,
    ) {
        map.insert(txn, key.into(), value);
    }

    /// Get a Map from a document, creating it if it doesn't exist
    #[allow(dead_code)]
    pub fn get_or_create_map(doc: &Doc, name: &str) -> MapRef {
        doc.get_or_insert_map(name)
    }

    /// Get a nested Map from a parent Map, creating it if it doesn't exist
    pub fn get_or_create_nested_map(
        parent: &MapRef,
        txn: &mut yrs::TransactionMut,
        key: impl Into<String>,
    ) -> MapRef {
        let key_str = key.into();
        // Check if map already exists
        if let Some(existing) = parent.get(txn, &key_str)
            && let Ok(m) = MapRef::try_from(existing)
        {
            return m;
        }
        // Create new map using the updated Yrs API (from syntax)
        let empty_prelim: MapPrelim = MapPrelim::from([("_", Any::Null)]);
        let new_map: MapRef = parent.insert(txn, key_str.as_str(), empty_prelim);
        // Remove the temporary key
        new_map.remove(txn, "_");
        new_map
    }

    /// Check if a Map contains a key
    #[allow(dead_code)]
    pub fn map_contains_key(map: &MapRef, txn: &impl ReadTxn, key: &str) -> bool {
        map.contains_key(txn, key)
    }

    /// Get all keys from a Map
    #[allow(dead_code)]
    pub fn get_map_keys(map: &MapRef, txn: &impl ReadTxn) -> Vec<String> {
        map.keys(txn).map(|k| k.to_string()).collect()
    }

    /// Remove a key from a Map (for tombstone deletion, prefer marking deleted=true)
    #[allow(dead_code)]
    pub fn remove_map_key(map: &MapRef, txn: &mut yrs::TransactionMut, key: &str) {
        map.remove(txn, key);
    }

    /// Materialize a Map to SQL (template method, to be implemented per entity type)
    #[allow(dead_code)]
    pub async fn materialize_to_sql(
        &self,
        _doc: &Doc,
        entity_type: &str,
        entity_id: &str,
    ) -> CrdtResult<()> {
        // This will be implemented per entity type in Phase 2
        // For now, just log
        tracing::debug!(
            "Materialization for entity_type={} entity_id={} not yet implemented",
            entity_type,
            entity_id
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use yrs::{GetString, ReadTxn, StateVector, Text};

    #[tokio::test]
    async fn test_save_and_load_document() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let manager = CrdtManager::new(&db_path).await.unwrap();

        // Create a document with text
        let doc = Doc::new();
        let text = doc.get_or_insert_text("content");
        text.push(&mut doc.transact_mut(), "Hello, world!");

        // Save
        manager
            .save_document("test-doc", "message", "msg-1", &doc)
            .await
            .unwrap();

        // Load
        let loaded_doc = manager.load_document("test-doc").await.unwrap();
        let loaded_text = loaded_doc.get_or_insert_text("content");
        let content = loaded_text.get_string(&loaded_doc.transact());

        assert_eq!(content, "Hello, world!");
    }

    #[tokio::test]
    async fn test_merge_updates() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let manager = CrdtManager::new(&db_path).await.unwrap();

        // Create initial document
        let doc1 = Doc::new();
        let text1 = doc1.get_or_insert_text("content");
        text1.push(&mut doc1.transact_mut(), "Hello");

        manager
            .save_document("test-doc", "message", "msg-1", &doc1)
            .await
            .unwrap();

        // Create second document with update
        let doc2 = Doc::new();
        let text2 = doc2.get_or_insert_text("content");
        text2.push(&mut doc2.transact_mut(), ", world!");

        let update = doc2.transact().encode_diff_v1(&StateVector::default());

        // Merge update
        manager
            .merge_update("test-doc", "message", "msg-1", &update)
            .await
            .unwrap();

        // Load and verify
        let final_doc = manager.load_document("test-doc").await.unwrap();
        let final_text = final_doc.get_or_insert_text("content");
        let content = final_text.get_string(&final_doc.transact());

        assert!(content.contains("Hello") || content.contains(", world!"));
    }

    #[tokio::test]
    async fn test_document_exists() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let manager = CrdtManager::new(&db_path).await.unwrap();

        assert!(!manager._document_exists("nonexistent").await.unwrap());

        let doc = Doc::new();
        manager
            .save_document("exists", "test", "1", &doc)
            .await
            .unwrap();

        assert!(manager._document_exists("exists").await.unwrap());
    }

    #[tokio::test]
    async fn test_map_of_maps_write_and_read() {
        // Test that we can write to nested Maps and read the data back
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let manager = CrdtManager::new(&db_path).await.unwrap();

        // Create a document with nested Maps structure
        let doc = Doc::new();

        // Get top-level Map
        let messages = doc.get_or_insert_map("messages");

        // Create nested Map and add fields
        {
            let mut txn = doc.transact_mut();
            let msg_map = CrdtManager::get_or_create_nested_map(&messages, &mut txn, "msg-1");

            CrdtManager::set_map_string(&msg_map, &mut txn, "id", "msg-1");
            CrdtManager::set_map_string(&msg_map, &mut txn, "content", "Hello");
            CrdtManager::set_map_i64(&msg_map, &mut txn, "created_at", 123456);
            CrdtManager::set_map_bool(&msg_map, &mut txn, "deleted", false);
        } // Transaction commits here

        // Save document
        manager
            .save_document("test-doc", "test", "1", &doc)
            .await
            .unwrap();

        // Read back from the SAME document (in memory)
        let messages = doc.get_or_insert_map("messages");
        {
            let txn = doc.transact();
            let msg_map =
                CrdtManager::get_nested_map(&messages, &txn, "msg-1").expect("Should find msg-1");

            let id = CrdtManager::get_map_string(&msg_map, &txn, "id").expect("Should have id");
            let content = CrdtManager::get_map_string(&msg_map, &txn, "content")
                .expect("Should have content");
            let created_at = CrdtManager::get_map_i64(&msg_map, &txn, "created_at")
                .expect("Should have created_at");
            let deleted =
                CrdtManager::get_map_bool(&msg_map, &txn, "deleted").expect("Should have deleted");

            assert_eq!(id, "msg-1");
            assert_eq!(content, "Hello");
            assert_eq!(created_at, 123456);
            assert!(!deleted);
        }

        // Load from database and read again
        let loaded_doc = manager.load_document("test-doc").await.unwrap();
        let loaded_messages = loaded_doc.get_or_insert_map("messages");
        {
            let txn = loaded_doc.transact();
            let msg_map = CrdtManager::get_nested_map(&loaded_messages, &txn, "msg-1")
                .expect("Should find msg-1 after reload");

            let id = CrdtManager::get_map_string(&msg_map, &txn, "id")
                .expect("Should have id after reload");
            let content = CrdtManager::get_map_string(&msg_map, &txn, "content")
                .expect("Should have content after reload");
            let created_at = CrdtManager::get_map_i64(&msg_map, &txn, "created_at")
                .expect("Should have created_at after reload");

            assert_eq!(id, "msg-1");
            assert_eq!(content, "Hello");
            assert_eq!(created_at, 123456);
        }
    }
}

use anyhow::{Context, Result};
use chrono::Utc;
use libsql::{params, Builder, Connection, Database};
use std::path::Path;
use yrs::{Doc, ReadTxn, Transact};

/// Manages CRDT documents with Turso (libSQL) persistence
pub struct CrdtManager {
    db: Database,
}

impl CrdtManager {
    /// Initialize CrdtManager with local Turso database
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db = Builder::new_local(db_path.as_ref())
            .build()
            .await
            .context("Failed to initialize Turso database")?;

        // Initialize schema - execute each statement separately
        let schema = include_str!("schema.sql");
        let conn = db.connect().context("Failed to get database connection")?;

        // Split schema by semicolons and execute each statement
        for statement in schema.split(';').filter(|s| !s.trim().is_empty()) {
            conn.execute(statement, ())
                .await
                .context("Failed to execute schema statement")?;
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
    ) -> Result<()> {
        // Encode state in a scope to drop the transaction before await
        let state = {
            let txn = doc.transact();
            txn.encode_state_as_update_v1(&yrs::StateVector::default())
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
            .context("Failed to save CRDT document")?;

        Ok(())
    }

    /// Load a Yrs document from the database
    pub async fn load_document(&self, doc_id: &str) -> Result<Doc> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT yrs_state FROM crdt_documents WHERE id = ?",
                params![doc_id],
            )
            .await
            .context("Failed to query CRDT document")?;

        if let Some(row) = rows.next().await? {
            let state: Vec<u8> = row.get(0)?;
            let doc = Doc::new();
            {
                let mut txn = doc.transact_mut();
                // Decode and apply update using correct API
                use yrs::updates::decoder::Decode;
                let update = yrs::Update::decode_v1(&state)
                    .context("Failed to decode Yrs state")?;
                txn.apply_update(update);
            }
            Ok(doc)
        } else {
            // Document doesn't exist yet, create new empty document
            Ok(Doc::new())
        }
    }

    /// Check if a document exists
    pub async fn document_exists(&self, doc_id: &str) -> Result<bool> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT COUNT(*) FROM crdt_documents WHERE id = ?",
                params![doc_id],
            )
            .await
            .context("Failed to check document existence")?;

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
    ) -> Result<()> {
        let doc = self.load_document(doc_id).await?;
        {
            let mut txn = doc.transact_mut();
            use yrs::updates::decoder::Decode;
            let decoded_update = yrs::Update::decode_v1(update)
                .context("Failed to decode update")?;
            txn.apply_update(decoded_update);
        }
        self.save_document(doc_id, entity_type, entity_id, &doc)
            .await
    }

    /// Get the state vector for a document (for sync)
    pub async fn get_state_vector(&self, doc_id: &str) -> Result<yrs::StateVector> {
        let doc = self.load_document(doc_id).await?;
        let sv = {
            let txn = doc.transact();
            txn.state_vector()
        };
        Ok(sv)
    }

    /// Get the difference between two state vectors (for sync)
    pub async fn get_diff(&self, doc_id: &str, remote_sv: &yrs::StateVector) -> Result<Vec<u8>> {
        let doc = self.load_document(doc_id).await?;
        let update = {
            let txn = doc.transact();
            txn.encode_state_as_update_v1(remote_sv)
        };
        Ok(update)
    }

    /// List all documents of a specific entity type
    pub async fn list_documents_by_type(&self, entity_type: &str) -> Result<Vec<String>> {
        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT id FROM crdt_documents WHERE entity_type = ? ORDER BY updated_at DESC",
                params![entity_type],
            )
            .await
            .context("Failed to list documents")?;

        let mut ids = Vec::new();
        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Delete a document
    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        conn.execute("DELETE FROM crdt_documents WHERE id = ?", params![doc_id])
            .await
            .context("Failed to delete CRDT document")?;
        Ok(())
    }

    /// Get database connection for custom queries
    pub fn connection(&self) -> Result<Connection> {
        self.db.connect().context("Failed to get database connection")
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

        assert!(!manager.document_exists("nonexistent").await.unwrap());

        let doc = Doc::new();
        manager
            .save_document("exists", "test", "1", &doc)
            .await
            .unwrap();

        assert!(manager.document_exists("exists").await.unwrap());
    }
}

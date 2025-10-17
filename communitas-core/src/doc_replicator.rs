// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

//! Document Replicator (Sprint 3.2)
//!
//! CRDT-based document synchronization with dual-storage architecture:
//!
//! ## Storage Modes
//!
//! - **Files**: SECRET storage, encrypted with ChaCha20Poly1305, group members only
//! - **Web**: PUBLIC storage, unencrypted, anyone can read
//! - **Both**: Document stored in both Files and Web (encrypted in Files, plaintext in Web)
//!
//! ## CRDT Synchronization
//!
//! Uses Yrs (Rust implementation of Yjs) for conflict-free collaborative editing.

use anyhow::{Result, anyhow};
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{debug, info};
use yrs::{
    Doc, GetString, ReadTxn, StateVector, Text, Transact, Update,
    updates::decoder::Decode,
    updates::encoder::{Encoder, EncoderV1},
};

/// Document identifier (unique ID for each document)
pub type DocumentId = String;

/// Storage mode for documents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageMode {
    /// Files storage: encrypted, group members only (SECRET)
    Files,
    /// Web storage: plaintext, public access (PUBLIC)
    Web,
    /// Both storages: encrypted in Files, public in Web
    Both,
}

/// DocReplicator configuration
#[derive(Clone)]
pub struct DocReplicatorConfig {
    pub files_storage_enabled: bool,
    pub web_storage_enabled: bool,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DocumentMetadata {
    id: DocumentId,
    name: String,
    storage_mode: StorageMode,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

/// Encrypted document wrapper for Files storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EncryptedDocument {
    nonce: [u8; 12],
    ciphertext: Vec<u8>,
}

/// Document Replicator - manages CRDT documents with dual storage
pub struct DocReplicator {
    // CRDT documents (Yrs) - using std::sync::Mutex for blocking Yrs operations
    documents: Arc<RwLock<HashMap<DocumentId, Arc<Mutex<Doc>>>>>,

    // Document metadata
    metadata: Arc<RwLock<HashMap<DocumentId, DocumentMetadata>>>,

    // Files storage: encrypted with ChaCha20Poly1305
    files_storage: Arc<RwLock<HashMap<DocumentId, EncryptedDocument>>>,
    encryption_keys: Arc<RwLock<HashMap<DocumentId, [u8; 32]>>>,

    // Web storage: plaintext, public
    web_storage: Arc<RwLock<HashMap<DocumentId, Vec<u8>>>>,

    // Configuration
    files_enabled: bool,
    web_enabled: bool,
}

impl DocReplicator {
    /// Create new DocReplicator
    pub async fn new(config: DocReplicatorConfig) -> Result<Self> {
        info!("Creating DocReplicator");

        Ok(Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            files_storage: Arc::new(RwLock::new(HashMap::new())),
            encryption_keys: Arc::new(RwLock::new(HashMap::new())),
            web_storage: Arc::new(RwLock::new(HashMap::new())),
            files_enabled: config.files_storage_enabled,
            web_enabled: config.web_storage_enabled,
        })
    }

    /// Create a new document
    pub async fn create_document(
        &self,
        name: &str,
        storage_mode: StorageMode,
    ) -> Result<DocumentId> {
        // Generate default encryption key for Files storage
        let mut default_key = [0u8; 32];
        getrandom::getrandom(&mut default_key)
            .map_err(|e| anyhow!("Failed to generate key: {}", e))?;

        self.create_document_with_key(name, storage_mode, &default_key)
            .await
    }

    /// Create a new document with specific encryption key
    ///
    /// If `name` looks like a UUID, it will be used as the document ID directly.
    /// Otherwise, a new UUID will be generated for the ID.
    pub async fn create_document_with_key(
        &self,
        name: &str,
        storage_mode: StorageMode,
        encryption_key: &[u8; 32],
    ) -> Result<DocumentId> {
        // Use name as ID if it's a valid identifier, otherwise generate UUID
        // For CRDT sync, we want to use the same ID across peers
        let doc_id = if name.contains('-') || name.len() > 30 {
            // Looks like a UUID or explicit ID - use it directly
            name.to_string()
        } else {
            // Regular name - generate UUID
            uuid::Uuid::new_v4().to_string()
        };

        let now = chrono::Utc::now();

        debug!(
            "Creating document '{}' with ID {} and mode {:?}",
            name, doc_id, storage_mode
        );

        debug!("Step 1: Creating Yrs Doc");
        // Create Yrs document
        let doc = Doc::new();
        let doc = Arc::new(Mutex::new(doc));
        debug!("Step 1: Done creating Yrs Doc");

        debug!("Step 2: Storing document in documents map");
        // Store document
        self.documents.write().await.insert(doc_id.clone(), doc);
        debug!("Step 2: Done storing document");

        debug!("Step 3: Storing metadata");
        // Store metadata
        let meta = DocumentMetadata {
            id: doc_id.clone(),
            name: name.to_string(),
            storage_mode,
            created_at: now,
            updated_at: now,
        };
        self.metadata.write().await.insert(doc_id.clone(), meta);
        debug!("Step 3: Done storing metadata");

        debug!("Step 4: Storing encryption key");
        // Store encryption key if Files storage
        if storage_mode == StorageMode::Files || storage_mode == StorageMode::Both {
            self.encryption_keys
                .write()
                .await
                .insert(doc_id.clone(), *encryption_key);
        }
        debug!("Step 4: Done storing encryption key");

        // Initialize storage based on mode
        // TEMP FIX: Skip initial save to avoid blocking on empty document
        // The document will be saved on first edit via insert_text
        debug!("Skipping initial save for empty document (will save on first edit)");

        info!("Document '{}' created with ID {}", name, doc_id);

        Ok(doc_id)
    }

    /// Get document (returns None if not found)
    pub async fn get_document(&self, doc_id: &str) -> Result<Option<Arc<Mutex<Doc>>>> {
        Ok(self.documents.read().await.get(doc_id).cloned())
    }

    /// Check if document exists in Files storage
    pub async fn document_exists_in_files(&self, doc_id: &str) -> Result<bool> {
        Ok(self.files_storage.read().await.contains_key(doc_id))
    }

    /// Check if document exists in Web storage
    pub async fn document_exists_in_web(&self, doc_id: &str) -> Result<bool> {
        Ok(self.web_storage.read().await.contains_key(doc_id))
    }

    /// Insert text at position
    pub async fn insert_text(&self, doc_id: &str, index: usize, text: &str) -> Result<()> {
        let doc = self
            .get_document(doc_id)
            .await?
            .ok_or_else(|| anyhow!("Document not found: {}", doc_id))?;

        // Clone for move into spawn_blocking
        let doc_clone = Arc::clone(&doc);
        let text_owned = text.to_string();

        // Wrap blocking Yrs operation in spawn_blocking
        tokio::task::spawn_blocking(move || {
            let doc = doc_clone
                .lock()
                .map_err(|e| anyhow!("Mutex lock failed: {}", e))?;

            // Get or create text object from Doc
            let ytext = doc.get_or_insert_text("content");
            let mut txn = doc.transact_mut();

            ytext.insert(&mut txn, index as u32, &text_owned);

            drop(txn);
            Ok::<(), anyhow::Error>(())
        })
        .await
        .map_err(|e| anyhow!("Join error: {}", e))??;

        // Update storage
        self.update_storage(doc_id).await?;

        debug!("Inserted '{}' at position {} in {}", text, index, doc_id);

        Ok(())
    }

    /// Delete text in range
    pub async fn delete_text(&self, doc_id: &str, index: usize, len: usize) -> Result<()> {
        let doc = self
            .get_document(doc_id)
            .await?
            .ok_or_else(|| anyhow!("Document not found: {}", doc_id))?;

        // Clone for move into spawn_blocking
        let doc_clone = Arc::clone(&doc);

        // Wrap blocking Yrs operation in spawn_blocking
        let actual_len = tokio::task::spawn_blocking(move || {
            let doc = doc_clone
                .lock()
                .map_err(|e| anyhow!("Mutex lock failed: {}", e))?;

            // Get or create text object from Doc
            let ytext = doc.get_or_insert_text("content");
            let mut txn = doc.transact_mut();

            // Get current length
            let current_len = ytext.len(&txn);

            // Clamp deletion to valid range
            let actual_len = if index + len > current_len as usize {
                if index >= current_len as usize {
                    return Ok(0); // Nothing to delete
                }
                current_len as usize - index
            } else {
                len
            };

            ytext.remove_range(&mut txn, index as u32, actual_len as u32);

            drop(txn);
            Ok::<usize, anyhow::Error>(actual_len)
        })
        .await
        .map_err(|e| anyhow!("Join error: {}", e))??;

        // Update storage
        self.update_storage(doc_id).await?;

        debug!(
            "Deleted {} chars at position {} in {}",
            actual_len, index, doc_id
        );

        Ok(())
    }

    /// Get text content of document
    pub async fn get_text(&self, doc_id: &str) -> Result<String> {
        let doc = self
            .get_document(doc_id)
            .await?
            .ok_or_else(|| anyhow!("Document not found: {}", doc_id))?;

        // Clone for move into spawn_blocking
        let doc_clone = Arc::clone(&doc);

        // Wrap blocking Yrs operation in spawn_blocking
        let text = tokio::task::spawn_blocking(move || {
            let doc = doc_clone
                .lock()
                .map_err(|e| anyhow!("Mutex lock failed: {}", e))?;

            // Get or create text object from Doc
            let ytext = doc.get_or_insert_text("content");
            let txn = doc.transact();
            let text = ytext.get_string(&txn);

            Ok::<String, anyhow::Error>(text)
        })
        .await
        .map_err(|e| anyhow!("Join error: {}", e))??;

        Ok(text)
    }

    /// Get CRDT update (full state from beginning)
    ///
    /// This encodes ALL changes from the document's creation, suitable for
    /// syncing to a new peer that doesn't have any prior state.
    pub async fn get_crdt_update(&self, doc_id: &str) -> Result<Vec<u8>> {
        let doc = self
            .get_document(doc_id)
            .await?
            .ok_or_else(|| anyhow!("Document not found: {}", doc_id))?;

        // Clone for move into spawn_blocking
        let doc_clone = Arc::clone(&doc);

        // Wrap blocking Yrs operation in spawn_blocking
        let update = tokio::task::spawn_blocking(move || {
            let doc = doc_clone
                .lock()
                .map_err(|e| anyhow!("Mutex lock failed: {}", e))?;
            let txn = doc.transact();

            // Use empty state vector to encode ALL changes from the beginning
            // This is the correct approach for initial sync with new peers
            let empty_state = StateVector::default();

            // Encode state as update using EncoderV1
            let mut encoder = EncoderV1::new();
            txn.encode_state_as_update(&empty_state, &mut encoder);

            Ok::<Vec<u8>, anyhow::Error>(encoder.to_vec())
        })
        .await
        .map_err(|e| anyhow!("Join error: {}", e))??;

        Ok(update)
    }

    /// Apply CRDT update from peer
    pub async fn apply_crdt_update(&self, doc_id: &str, update: &[u8]) -> Result<()> {
        // Create document if it doesn't exist (using the provided doc_id)
        if self.get_document(doc_id).await?.is_none() {
            self.create_document_with_id(doc_id, StorageMode::Files)
                .await?;
        }

        let doc = self
            .get_document(doc_id)
            .await?
            .ok_or_else(|| anyhow!("Document not found: {}", doc_id))?;

        // Clone for move into spawn_blocking
        let doc_clone = Arc::clone(&doc);
        let update_owned = update.to_vec();

        // Wrap blocking Yrs operation in spawn_blocking
        tokio::task::spawn_blocking(move || {
            let doc = doc_clone
                .lock()
                .map_err(|e| anyhow!("Mutex lock failed: {}", e))?;
            let mut txn = doc.transact_mut();

            let update_obj = Update::decode_v1(&update_owned)
                .map_err(|e| anyhow!("Failed to decode update: {:?}", e))?;

            txn.apply_update(update_obj);

            drop(txn);
            Ok::<(), anyhow::Error>(())
        })
        .await
        .map_err(|e| anyhow!("Join error: {}", e))??;

        // Update storage
        self.update_storage(doc_id).await?;

        debug!("Applied CRDT update to {}", doc_id);

        Ok(())
    }

    /// Internal: Create document with specific ID (for CRDT sync)
    async fn create_document_with_id(&self, doc_id: &str, storage_mode: StorageMode) -> Result<()> {
        let now = chrono::Utc::now();

        // Generate default encryption key if needed
        let mut default_key = [0u8; 32];
        getrandom::getrandom(&mut default_key)
            .map_err(|e| anyhow!("Failed to generate key: {}", e))?;

        debug!(
            "Creating document with ID '{}' and mode {:?}",
            doc_id, storage_mode
        );

        // Create Yrs document
        let doc = Doc::new();
        let doc = Arc::new(Mutex::new(doc));

        // Store document
        self.documents.write().await.insert(doc_id.to_string(), doc);

        // Store metadata
        let meta = DocumentMetadata {
            id: doc_id.to_string(),
            name: doc_id.to_string(),
            storage_mode,
            created_at: now,
            updated_at: now,
        };
        self.metadata.write().await.insert(doc_id.to_string(), meta);

        // Store encryption key if Files storage
        if storage_mode == StorageMode::Files || storage_mode == StorageMode::Both {
            self.encryption_keys
                .write()
                .await
                .insert(doc_id.to_string(), default_key);
        }

        // Initialize storage based on mode
        match storage_mode {
            StorageMode::Files => {
                self.save_to_files(doc_id).await?;
            }
            StorageMode::Web => {
                self.save_to_web(doc_id).await?;
            }
            StorageMode::Both => {
                self.save_to_files(doc_id).await?;
                self.save_to_web(doc_id).await?;
            }
        }

        info!("Document created with ID {}", doc_id);

        Ok(())
    }

    /// Get encrypted blob from Files storage
    pub async fn get_files_blob(&self, doc_id: &str) -> Result<Option<Vec<u8>>> {
        let storage = self.files_storage.read().await;

        let encrypted = match storage.get(doc_id) {
            Some(enc) => enc,
            None => return Ok(None),
        };

        let blob =
            bincode::serialize(encrypted).map_err(|e| anyhow!("Serialization failed: {}", e))?;

        Ok(Some(blob))
    }

    /// Get plaintext blob from Web storage
    pub async fn get_web_blob(&self, doc_id: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.web_storage.read().await.get(doc_id).cloned())
    }

    /// Get encryption key for document (fails for Web-only docs)
    pub async fn get_encryption_key(&self, doc_id: &str) -> Result<[u8; 32]> {
        let meta = self
            .metadata
            .read()
            .await
            .get(doc_id)
            .cloned()
            .ok_or_else(|| anyhow!("Document not found: {}", doc_id))?;

        if meta.storage_mode == StorageMode::Web {
            return Err(anyhow!("Web documents do not have encryption keys"));
        }

        self.encryption_keys
            .read()
            .await
            .get(doc_id)
            .copied()
            .ok_or_else(|| anyhow!("Encryption key not found"))
    }

    /// Decrypt document with specific key (for testing wrong key scenarios)
    pub async fn decrypt_with_key(&self, doc_id: &str, key: &[u8; 32]) -> Result<Vec<u8>> {
        let storage = self.files_storage.read().await;

        let encrypted = storage
            .get(doc_id)
            .ok_or_else(|| anyhow!("Document not in Files storage"))?;

        let cipher = ChaCha20Poly1305::new(key.into());
        let nonce = Nonce::from(
            *<&[u8; 12]>::try_from(encrypted.nonce.as_slice())
                .map_err(|_| anyhow!("Invalid nonce length"))?,
        );

        let plaintext = cipher
            .decrypt(&nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Save document to Files storage (encrypted)
    async fn save_to_files(&self, doc_id: &str) -> Result<()> {
        if !self.files_enabled {
            return Ok(());
        }

        let update = self.get_crdt_update(doc_id).await?;

        let key = self
            .encryption_keys
            .read()
            .await
            .get(doc_id)
            .copied()
            .ok_or_else(|| anyhow!("Encryption key not found"))?;

        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        getrandom::getrandom(&mut nonce_bytes)
            .map_err(|e| anyhow!("Nonce generation failed: {}", e))?;

        let cipher = ChaCha20Poly1305::new(&key.into());
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, update.as_ref())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        let encrypted = EncryptedDocument {
            nonce: nonce_bytes,
            ciphertext,
        };

        self.files_storage
            .write()
            .await
            .insert(doc_id.to_string(), encrypted);

        debug!("Saved {} to Files storage (encrypted)", doc_id);

        Ok(())
    }

    /// Save document to Web storage (plaintext)
    async fn save_to_web(&self, doc_id: &str) -> Result<()> {
        if !self.web_enabled {
            return Ok(());
        }

        let update = self.get_crdt_update(doc_id).await?;

        self.web_storage
            .write()
            .await
            .insert(doc_id.to_string(), update);

        debug!("Saved {} to Web storage (public)", doc_id);

        Ok(())
    }

    /// Update storage after CRDT change
    async fn update_storage(&self, doc_id: &str) -> Result<()> {
        let meta = self
            .metadata
            .read()
            .await
            .get(doc_id)
            .cloned()
            .ok_or_else(|| anyhow!("Document metadata not found"))?;

        match meta.storage_mode {
            StorageMode::Files => self.save_to_files(doc_id).await?,
            StorageMode::Web => self.save_to_web(doc_id).await?,
            StorageMode::Both => {
                self.save_to_files(doc_id).await?;
                self.save_to_web(doc_id).await?;
            }
        }

        // Update timestamp
        self.metadata
            .write()
            .await
            .entry(doc_id.to_string())
            .and_modify(|m| {
                m.updated_at = chrono::Utc::now();
            });

        Ok(())
    }

    /// List all document IDs
    pub async fn list_documents(&self) -> Result<Vec<String>> {
        debug!("list_documents: About to acquire read lock");
        let documents = self.documents.read().await;
        debug!(
            "list_documents: Acquired read lock, found {} documents",
            documents.len()
        );
        let keys: Vec<String> = documents.keys().cloned().collect();
        debug!("list_documents: Collected keys");
        Ok(keys)
    }

    /// Delete a document and all associated data
    pub async fn delete_document(&self, doc_id: &str) -> Result<()> {
        debug!("Deleting document: {}", doc_id);

        // Remove from all storage locations
        self.documents.write().await.remove(doc_id);
        self.metadata.write().await.remove(doc_id);
        self.encryption_keys.write().await.remove(doc_id);
        self.files_storage.write().await.remove(doc_id);
        self.web_storage.write().await.remove(doc_id);

        info!("Document deleted: {}", doc_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_config() -> DocReplicatorConfig {
        DocReplicatorConfig {
            files_storage_enabled: true,
            web_storage_enabled: true,
        }
    }

    #[tokio::test]
    async fn test_doc_replicator_creation() {
        let config = create_test_config().await;
        let replicator = DocReplicator::new(config).await.expect("create replicator");

        assert!(replicator.files_enabled);
        assert!(replicator.web_enabled);
    }

    #[tokio::test]
    async fn test_create_and_retrieve_document() {
        let config = create_test_config().await;
        let replicator = DocReplicator::new(config).await.expect("create replicator");

        let doc_id = replicator
            .create_document("test", StorageMode::Files)
            .await
            .expect("create doc");

        let doc = replicator.get_document(&doc_id).await.expect("get doc");

        assert!(doc.is_some());
    }

    #[tokio::test]
    async fn test_insert_and_get_text() {
        let config = create_test_config().await;
        let replicator = DocReplicator::new(config).await.expect("create replicator");

        let doc_id = replicator
            .create_document("text-test", StorageMode::Files)
            .await
            .expect("create");

        replicator
            .insert_text(&doc_id, 0, "Hello!")
            .await
            .expect("insert");

        let text = replicator.get_text(&doc_id).await.expect("get text");

        assert_eq!(text, "Hello!");
    }

    #[tokio::test]
    async fn test_dual_storage() {
        let config = create_test_config().await;
        let replicator = DocReplicator::new(config).await.expect("create replicator");

        let doc_id = replicator
            .create_document("dual-doc", StorageMode::Both)
            .await
            .expect("create");

        let files_exists = replicator
            .document_exists_in_files(&doc_id)
            .await
            .expect("check files");
        let web_exists = replicator
            .document_exists_in_web(&doc_id)
            .await
            .expect("check web");

        assert!(files_exists);
        assert!(web_exists);
    }
}

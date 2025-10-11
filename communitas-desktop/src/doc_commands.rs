// Copyright (c) 2025 Saorsa Labs Limited
//
// Document management Tauri commands (Sprint 3.3)
//
// Provides entity-scoped document storage with dual-mode support:
// - Files storage (encrypted, group members only)
// - Web storage (public, unencrypted)

use communitas_core::doc_replicator::StorageMode;
use communitas_core::CoreContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::debug;

// =============================================================================
// Request/Response Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct DocCreateRequest {
    pub entity_id: String,
    pub name: String,
    pub storage_mode: String, // "files" | "web" | "both"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocInsertTextRequest {
    pub doc_id: String,
    pub position: u32,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocDeleteTextRequest {
    pub doc_id: String,
    pub position: u32,
    pub length: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocApplyUpdateRequest {
    pub doc_id: String,
    pub update: Vec<u8>, // CRDT update bytes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocListRequest {
    pub entity_id: String,
    pub storage_mode: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocResponse {
    pub doc_id: String,
    pub entity_id: String,
    pub name: String,
    pub storage_mode: String,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse storage mode from string
fn parse_storage_mode(mode: &str) -> Result<StorageMode, String> {
    match mode.to_lowercase().as_str() {
        "files" => Ok(StorageMode::Files),
        "web" => Ok(StorageMode::Web),
        "both" => Ok(StorageMode::Both),
        _ => Err(format!("Invalid storage mode: {}. Must be 'files', 'web', or 'both'", mode)),
    }
}

/// Convert StorageMode to string
fn _storage_mode_to_string(mode: &StorageMode) -> String {
    match mode {
        StorageMode::Files => "files".to_string(),
        StorageMode::Web => "web".to_string(),
        StorageMode::Both => "both".to_string(),
    }
}

/// Create entity-scoped document ID: {entity_id}/{doc_name}
fn create_doc_id(entity_id: &str, doc_name: &str) -> String {
    format!("{}/{}", entity_id, doc_name)
}

/// Parse entity-scoped document ID back to (entity_id, doc_name)
fn parse_doc_id(doc_id: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = doc_id.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid document ID format: {}. Expected format: entity_id/doc_name", doc_id));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Create a new document in entity-scoped storage
///
/// Documents are identified by: {entity_id}/{doc_name}
/// - Files storage: Encrypted, accessible to entity members only
/// - Web storage: Public, unencrypted, for website publishing
///
/// # Example
/// ```typescript
/// await invoke('doc_create', {
///   entityId: 'channel-123',
///   name: 'meeting-notes',
///   storageMode: 'files' // or 'web' or 'both'
/// });
/// ```
#[tauri::command]
pub async fn doc_create(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    entity_id: String,
    name: String,
    storage_mode: String,
) -> Result<DocResponse, String> {
    debug!("Creating document '{}' for entity '{}' in '{}' storage", name, entity_id, storage_mode);

    let mode = parse_storage_mode(&storage_mode)?;

    // Create entity-scoped document ID
    let doc_id = create_doc_id(&entity_id, &name);

    debug!("Generated doc_id: {}", doc_id);

    debug!("BEFORE: Acquiring read lock on CoreContext");

    // Clone Arc to doc_replicator and drop the read guard ASAP
    let doc_replicator = {
        let guard = shared.read().await;
        debug!("AFTER: Acquired read lock on CoreContext");

        let ctx_option = guard.as_ref();
        debug!("AFTER: Got Option<CoreContext>, is_some: {}", ctx_option.is_some());

        let ctx = ctx_option.ok_or_else(|| "CoreContext not initialized".to_string())?;
        debug!("AFTER: Got CoreContext reference");

        let replicator = Arc::clone(&ctx.doc_replicator);
        debug!("AFTER: Cloned doc_replicator Arc");
        replicator
    }; // guard dropped here

    debug!("About to call create_document");

    // Create document using DocReplicator (without holding CoreContext lock)
    let created_id = doc_replicator
        .create_document(&doc_id, mode)
        .await
        .map_err(|e| format!("Failed to create document: {}", e))?;

    debug!("Document created with ID: {}", created_id);

    Ok(DocResponse {
        doc_id: created_id,
        entity_id,
        name,
        storage_mode,
    })
}

/// Insert text into a document at a specific position
///
/// Uses CRDT (Yrs) for conflict-free concurrent editing.
///
/// # Example
/// ```typescript
/// await invoke('doc_insert_text', {
///   docId: 'channel-123/meeting-notes',
///   position: 0,
///   text: 'Hello, World!'
/// });
/// ```
#[tauri::command]
pub async fn doc_insert_text(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    doc_id: String,
    position: u32,
    text: String,
) -> Result<(), String> {
    debug!("Inserting text '{}' at position {} in document '{}'", text, position, doc_id);

    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    ctx.doc_replicator
        .insert_text(&doc_id, position as usize, &text)
        .await
        .map_err(|e| format!("Failed to insert text: {}", e))?;

    debug!("Text inserted successfully");

    Ok(())
}

/// Delete text from a document at a specific position
///
/// # Example
/// ```typescript
/// await invoke('doc_delete_text', {
///   docId: 'channel-123/meeting-notes',
///   position: 7,
///   length: 5
/// });
/// ```
#[tauri::command]
pub async fn doc_delete_text(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    doc_id: String,
    position: u32,
    length: u32,
) -> Result<(), String> {
    debug!("Deleting {} characters at position {} in document '{}'", length, position, doc_id);

    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    ctx.doc_replicator
        .delete_text(&doc_id, position as usize, length as usize)
        .await
        .map_err(|e| format!("Failed to delete text: {}", e))?;

    debug!("Text deleted successfully");

    Ok(())
}

/// Get the full text content of a document
///
/// # Example
/// ```typescript
/// const text = await invoke('doc_get_text', {
///   docId: 'channel-123/meeting-notes'
/// });
/// ```
#[tauri::command]
pub async fn doc_get_text(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    doc_id: String,
) -> Result<String, String> {
    debug!("Getting text for document '{}'", doc_id);

    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    let text = ctx
        .doc_replicator
        .get_text(&doc_id)
        .await
        .map_err(|e| format!("Failed to get text: {}", e))?;

    Ok(text)
}

/// Get CRDT update for synchronization (full document state)
///
/// This encodes the full document state from the beginning, suitable for
/// syncing to a new peer that doesn't have any prior state.
///
/// # Example
/// ```typescript
/// const update = await invoke('doc_get_update', {
///   docId: 'channel-123/meeting-notes'
/// });
/// ```
#[tauri::command]
pub async fn doc_get_update(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    doc_id: String,
) -> Result<Vec<u8>, String> {
    debug!("Getting CRDT update for document '{}'", doc_id);

    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    let update = ctx
        .doc_replicator
        .get_crdt_update(&doc_id)
        .await
        .map_err(|e| format!("Failed to get CRDT update: {}", e))?;

    debug!("Got CRDT update ({} bytes)", update.len());

    Ok(update)
}

/// Apply CRDT update from peer
///
/// Creates the document if it doesn't exist, then applies the update.
/// This enables peer-to-peer document synchronization.
///
/// # Example
/// ```typescript
/// await invoke('doc_apply_update', {
///   docId: 'channel-123/meeting-notes',
///   update: new Uint8Array([...])
/// });
/// ```
#[tauri::command]
pub async fn doc_apply_update(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    doc_id: String,
    update: Vec<u8>,
) -> Result<(), String> {
    debug!("Applying CRDT update ({} bytes) to document '{}'", update.len(), doc_id);

    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    ctx.doc_replicator
        .apply_crdt_update(&doc_id, &update)
        .await
        .map_err(|e| format!("Failed to apply CRDT update: {}", e))?;

    debug!("CRDT update applied successfully");

    Ok(())
}

/// List all documents for an entity in a specific storage mode
///
/// # Example
/// ```typescript
/// const docs = await invoke('doc_list', {
///   entityId: 'channel-123',
///   storageMode: 'files'
/// });
/// ```
#[tauri::command]
pub async fn doc_list(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    entity_id: String,
    storage_mode: String,
) -> Result<Vec<DocResponse>, String> {
    debug!("Listing documents for entity '{}' in '{}' storage", entity_id, storage_mode);

    let _mode = parse_storage_mode(&storage_mode)?;

    debug!("doc_list: About to acquire read lock");

    // Clone Arc to doc_replicator and drop guard ASAP
    let doc_replicator = {
        let guard = shared.read().await;
        debug!("doc_list: Acquired read lock");

        let ctx_option = guard.as_ref();
        debug!("doc_list: Got Option<CoreContext>, is_some: {}", ctx_option.is_some());

        let ctx = ctx_option.ok_or_else(|| "CoreContext not initialized".to_string())?;
        debug!("doc_list: Got CoreContext");

        Arc::clone(&ctx.doc_replicator)
    };

    debug!("doc_list: About to call list_documents");

    // Get all document IDs from the replicator
    let doc_ids = doc_replicator
        .list_documents()
        .await
        .map_err(|e| format!("Failed to list documents: {}", e))?;

    debug!("doc_list: Got {} document IDs", doc_ids.len());

    // Filter for this entity and storage mode
    let entity_prefix = format!("{}/", entity_id);
    let mut docs = Vec::new();

    for doc_id in doc_ids {
        if doc_id.starts_with(&entity_prefix) {
            // Parse document ID to get name
            if let Ok((ent_id, name)) = parse_doc_id(&doc_id) {
                // Check if this document is in the requested storage mode
                // Note: For now we return all docs for the entity, filtering by storage mode
                // would require checking document metadata
                docs.push(DocResponse {
                    doc_id: doc_id.clone(),
                    entity_id: ent_id,
                    name,
                    storage_mode: storage_mode.clone(),
                });
            }
        }
    }

    debug!("Found {} documents for entity '{}'", docs.len(), entity_id);

    Ok(docs)
}

/// Delete a document
///
/// # Example
/// ```typescript
/// await invoke('doc_delete', {
///   docId: 'channel-123/meeting-notes'
/// });
/// ```
#[tauri::command]
pub async fn doc_delete(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    doc_id: String,
) -> Result<(), String> {
    debug!("Deleting document '{}'", doc_id);

    let guard = shared.read().await;
    let ctx = guard
        .as_ref()
        .ok_or_else(|| "CoreContext not initialized".to_string())?;

    ctx.doc_replicator
        .delete_document(&doc_id)
        .await
        .map_err(|e| format!("Failed to delete document: {}", e))?;

    debug!("Document deleted successfully");

    Ok(())
}

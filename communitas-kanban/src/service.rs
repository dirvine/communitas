//! KanbanService - High-level API for the Kanban system.
//!
//! This module provides [`KanbanService`], the main entry point for all
//! Kanban operations. It wraps the underlying CRDT operations with a
//! clean async API.

use std::num::NonZeroUsize;
use std::sync::Arc;

use chrono::Utc;
use lru::LruCache;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;
use yrs::observer::Subscription;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Array, ArrayRef, Doc, GetString, Map, MapRef, Out, ReadTxn, Text, TextRef, Transact,
    WriteTxn,
};

/// Maximum number of board documents to keep in the LRU cache.
/// Documents for boards accessed least recently will be evicted when
/// this limit is reached. The value of 50 is chosen to balance memory
/// usage (~20-50MB typical) against the overhead of re-loading documents.
const MAX_BOARD_DOCUMENTS: usize = 50;

use crate::error::{KanbanError, KanbanResult};
use crate::filter::CardFilter;
use crate::operations::{
    self, init_card_structure, init_column_structure, init_document_structure, keys, orset_add,
    orset_members, orset_remove,
};
use crate::state_machine::CardState;
use crate::types::{
    Board, BoardSettings, BoardUpdate, Card, CardUpdate, Column, ColumnUpdate, Comment,
    KanbanEvent, Priority, Step, Tag,
};

/// Holds the Yrs subscription for a board's CRDT observer.
struct BoardSubscription {
    /// The Yrs subscription handle. Dropping this unsubscribes.
    _subscription: Subscription,
    /// Channel sender to forward events to subscribers.
    /// Kept alive so the receiver stays open; prefixed with underscore
    /// since we send via a cloned sender in the observer closure.
    _sender: mpsc::Sender<KanbanEvent>,
}

/// High-level service for Kanban operations.
///
/// KanbanService provides all board, column, card, step, comment, and tag
/// operations. It manages Yrs documents internally and ensures all
/// operations are atomic and conflict-free.
///
/// # Example
///
/// ```ignore
/// use communitas_kanban::{KanbanService, BoardSettings};
///
/// let service = KanbanService::new("ocean-forest-moon-star");
///
/// // Create a board
/// let board = service.create_board("project-id", "Sprint 1".to_string(), None)?;
///
/// // Add columns
/// let todo = service.add_column(&board.id, "To Do".to_string(), None)?;
/// let done = service.add_column(&board.id, "Done".to_string(), None)?;
///
/// // Create and manage cards
/// let card = service.create_card(&board.id, &todo.id, "Task 1".to_string(), None)?;
/// service.move_card(&board.id, &card.id, &done.id, 0)?;
///
/// // Subscribe to CRDT changes
/// let mut rx = service.subscribe_to_changes(&board.id)?;
/// while let Some(event) = rx.recv().await {
///     println!("Board changed: {:?}", event);
/// }
/// ```
pub struct KanbanService {
    /// Current user's four-word identity
    peer_id: String,
    /// LRU cache of Yrs documents by board ID.
    /// Documents for least-recently-used boards are evicted when the cache
    /// reaches MAX_BOARD_DOCUMENTS capacity. Using Mutex instead of RwLock
    /// because LruCache updates on get() for LRU tracking.
    documents: std::sync::Mutex<LruCache<String, Arc<Doc>>>,
    /// Active subscriptions per board ID.
    subscriptions: std::sync::RwLock<std::collections::HashMap<String, BoardSubscription>>,
}

impl KanbanService {
    /// Compile-time capacity for LRU cache (guaranteed non-zero).
    const LRU_CAPACITY: NonZeroUsize = match NonZeroUsize::new(MAX_BOARD_DOCUMENTS) {
        Some(n) => n,
        None => panic!("MAX_BOARD_DOCUMENTS must be non-zero"),
    };

    /// Create a new KanbanService.
    ///
    /// # Arguments
    ///
    /// * `peer_id` - Four-word identity of the current user
    #[must_use]
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self {
            peer_id: peer_id.into(),
            documents: std::sync::Mutex::new(LruCache::new(Self::LRU_CAPACITY)),
            subscriptions: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Get the current timestamp.
    fn now() -> i64 {
        Utc::now().timestamp()
    }

    /// Generate a new UUID.
    fn new_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Get or create a document for a board.
    ///
    /// When the LRU cache is full, the least-recently-used document will
    /// be evicted and its subscriptions cleaned up.
    fn get_or_create_doc(&self, board_id: &str) -> Arc<Doc> {
        let mut docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());

        // Check if doc exists and return it (this also marks it as recently used)
        if let Some(doc) = docs.get(board_id) {
            return doc.clone();
        }

        // Doc doesn't exist - check if we need to evict
        if docs.len() >= MAX_BOARD_DOCUMENTS {
            // Get the key of the least recently used doc before adding new one
            if let Some((evicted_id, _)) = docs.peek_lru() {
                let evicted_id = evicted_id.clone();
                // Clean up subscriptions for evicted document
                self.cleanup_evicted_subscription(&evicted_id);
                info!(
                    board_id = %evicted_id,
                    cache_size = docs.len(),
                    "Evicting LRU board document from cache"
                );
            }
        }

        // Create new document
        let doc = Doc::new();
        init_document_structure(&doc);
        let doc_arc = Arc::new(doc);
        docs.put(board_id.to_string(), doc_arc.clone());
        doc_arc
    }

    /// Get a document for a board (must exist).
    ///
    /// Note: This marks the document as recently used in the LRU cache.
    fn get_doc(&self, board_id: &str) -> KanbanResult<Arc<Doc>> {
        let mut docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        docs.get(board_id)
            .cloned()
            .ok_or_else(|| KanbanError::BoardNotFound(board_id.to_string()))
    }

    /// Clean up subscription for an evicted document.
    fn cleanup_evicted_subscription(&self, board_id: &str) {
        let mut subs = self
            .subscriptions
            .write()
            .unwrap_or_else(|e| e.into_inner());
        if subs.remove(board_id).is_some() {
            debug!(
                board_id = %board_id,
                "Cleaned up subscription for evicted board document"
            );
        }
    }

    /// Get the number of documents currently in the cache.
    ///
    /// Useful for diagnostics and monitoring memory usage.
    #[must_use]
    pub fn document_count(&self) -> usize {
        let docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        docs.len()
    }

    /// Get the maximum capacity of the document cache.
    #[must_use]
    pub fn document_capacity(&self) -> usize {
        MAX_BOARD_DOCUMENTS
    }

    // ===== Board Operations =====

    /// Create a new board for a project.
    ///
    /// # Arguments
    ///
    /// * `project_id` - Four-word identity of the owning project
    /// * `name` - Display name for the board
    /// * `settings` - Optional board settings (defaults to all features enabled)
    #[instrument(skip(self, settings))]
    pub fn create_board(
        &self,
        project_id: &str,
        name: String,
        settings: Option<BoardSettings>,
    ) -> KanbanResult<Board> {
        let board_id = Self::new_id();
        let now = Self::now();
        let settings = settings.unwrap_or_else(BoardSettings::all_features);

        let doc = self.get_or_create_doc(&board_id);
        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);

        // Set board metadata
        metadata.insert(&mut txn, "id", Any::String(board_id.clone().into()));
        metadata.insert(&mut txn, "name", Any::String(name.clone().into()));
        metadata.insert(
            &mut txn,
            "project_id",
            Any::String(project_id.to_string().into()),
        );
        metadata.insert(
            &mut txn,
            "created_by",
            Any::String(self.peer_id.clone().into()),
        );
        metadata.insert(&mut txn, "created_at", Any::BigInt(now));
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));
        metadata.insert(&mut txn, "deleted", Any::Bool(false));

        // Set settings
        let settings_map: MapRef = metadata.get_or_init(&mut txn, keys::SETTINGS);
        self.write_settings(&settings_map, &mut txn, &settings);

        debug!(board_id = %board_id, name = %name, "Created board");

        Ok(Board {
            id: board_id,
            name,
            description: None,
            project_id: project_id.to_string(),
            created_by: self.peer_id.clone(),
            created_at: now,
            updated_at: now,
            settings,
            deleted: false,
            deleted_at: None,
        })
    }

    /// Write settings to a settings map.
    fn write_settings(
        &self,
        settings_map: &MapRef,
        txn: &mut yrs::TransactionMut<'_>,
        settings: &BoardSettings,
    ) {
        if let Some(ref col_id) = settings.default_column_id {
            settings_map.insert(txn, "default_column_id", Any::String(col_id.clone().into()));
        }
        settings_map.insert(txn, "enable_steps", Any::Bool(settings.enable_steps));
        settings_map.insert(
            txn,
            "enable_assignments",
            Any::Bool(settings.enable_assignments),
        );
        settings_map.insert(txn, "enable_tags", Any::Bool(settings.enable_tags));
        settings_map.insert(
            txn,
            "enable_due_dates",
            Any::Bool(settings.enable_due_dates),
        );
        settings_map.insert(
            txn,
            "auto_archive_completed",
            Any::Bool(settings.auto_archive_completed),
        );
        if let Some(days) = settings.archive_after_days {
            settings_map.insert(txn, "archive_after_days", Any::BigInt(days as i64));
        }
    }

    /// Get a board by ID.
    ///
    /// Returns an error if the board is deleted.
    #[instrument(skip(self))]
    pub fn get_board(&self, board_id: &str) -> KanbanResult<Board> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
        let metadata = match root.get(&txn, keys::METADATA) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing metadata".to_string())),
        };

        let board = self.read_board_from_map(&metadata, &txn)?;

        if board.deleted {
            return Err(KanbanError::EntityDeleted {
                entity_type: "board",
                entity_id: board_id.to_string(),
            });
        }

        Ok(board)
    }

    /// Read a board from a metadata map.
    fn read_board_from_map<T: ReadTxn>(&self, metadata: &MapRef, txn: &T) -> KanbanResult<Board> {
        let id = get_string(metadata, txn, "id")?;
        let name = get_string(metadata, txn, "name")?;
        let description = get_optional_string(metadata, txn, "description");
        let project_id = get_string(metadata, txn, "project_id")?;
        let created_by = get_string(metadata, txn, "created_by")?;
        let created_at = get_i64(metadata, txn, "created_at")?;
        let updated_at = get_i64(metadata, txn, "updated_at")?;
        let deleted = get_bool(metadata, txn, "deleted").unwrap_or(false);
        let deleted_at = get_optional_i64(metadata, txn, "deleted_at");

        // Read settings
        let settings = if let Some(Out::YMap(settings_map)) = metadata.get(txn, keys::SETTINGS) {
            BoardSettings {
                default_column_id: get_optional_string(&settings_map, txn, "default_column_id"),
                enable_steps: get_bool(&settings_map, txn, "enable_steps").unwrap_or(true),
                enable_assignments: get_bool(&settings_map, txn, "enable_assignments")
                    .unwrap_or(true),
                enable_tags: get_bool(&settings_map, txn, "enable_tags").unwrap_or(true),
                enable_due_dates: get_bool(&settings_map, txn, "enable_due_dates").unwrap_or(true),
                auto_archive_completed: get_bool(&settings_map, txn, "auto_archive_completed")
                    .unwrap_or(false),
                archive_after_days: get_optional_i64(&settings_map, txn, "archive_after_days")
                    .map(|v| v as u32),
            }
        } else {
            BoardSettings::all_features()
        };

        Ok(Board {
            id,
            name,
            description,
            project_id,
            created_by,
            created_at,
            updated_at,
            settings,
            deleted,
            deleted_at,
        })
    }

    /// Update a board.
    #[instrument(skip(self, updates))]
    pub fn update_board(&self, board_id: &str, updates: BoardUpdate) -> KanbanResult<Board> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");
            let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);

            if let Some(name) = updates.name {
                metadata.insert(&mut txn, "name", Any::String(name.into()));
            }
            if let Some(description) = updates.description {
                match description {
                    Some(desc) => {
                        metadata.insert(&mut txn, "description", Any::String(desc.into()));
                    }
                    None => {
                        let _ = metadata.remove(&mut txn, "description");
                    }
                }
            }
            if let Some(settings) = updates.settings {
                let settings_map: MapRef = metadata.get_or_init(&mut txn, keys::SETTINGS);
                self.write_settings(&settings_map, &mut txn, &settings);
            }
            metadata.insert(&mut txn, "updated_at", Any::BigInt(now));
        }

        self.get_board(board_id)
    }

    /// Soft-delete a board.
    #[instrument(skip(self))]
    pub fn delete_board(&self, board_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);

        metadata.insert(&mut txn, "deleted", Any::Bool(true));
        metadata.insert(&mut txn, "deleted_at", Any::BigInt(now));
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, "Deleted board");
        Ok(())
    }

    // ===== Column Operations =====

    /// Add a column to a board.
    #[instrument(skip(self))]
    pub fn add_column(
        &self,
        board_id: &str,
        name: String,
        position: Option<u32>,
    ) -> KanbanResult<Column> {
        let doc = self.get_doc(board_id)?;
        let column_id = Self::new_id();
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let column_order: ArrayRef = root.get_or_init(&mut txn, keys::COLUMN_ORDER);
        let columns: MapRef = root.get_or_init(&mut txn, keys::COLUMNS);

        // Determine position
        let len = column_order.len(&txn);
        let pos = position.unwrap_or(len);
        let actual_pos = pos.min(len);

        // Add to column_order
        column_order.insert(&mut txn, actual_pos, Any::String(column_id.clone().into()));

        // Create column entry
        init_column_structure(&columns, &mut txn, &column_id);
        let col: MapRef = columns.get_or_init(&mut txn, column_id.as_str());
        col.insert(&mut txn, "id", Any::String(column_id.clone().into()));
        col.insert(
            &mut txn,
            "board_id",
            Any::String(board_id.to_string().into()),
        );
        col.insert(&mut txn, "name", Any::String(name.clone().into()));
        col.insert(&mut txn, "position", Any::BigInt(actual_pos as i64));
        col.insert(&mut txn, "created_at", Any::BigInt(now));
        col.insert(&mut txn, "deleted", Any::Bool(false));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, column_id = %column_id, name = %name, "Added column");

        Ok(Column {
            id: column_id,
            board_id: board_id.to_string(),
            name,
            position: actual_pos,
            color: None,
            wip_limit: None,
            created_at: now,
            deleted: false,
        })
    }

    /// Update a column.
    #[instrument(skip(self, updates))]
    pub fn update_column(
        &self,
        board_id: &str,
        column_id: &str,
        updates: ColumnUpdate,
    ) -> KanbanResult<Column> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");
            let columns: MapRef = root.get_or_init(&mut txn, keys::COLUMNS);
            let col = match columns.get(&txn, column_id) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::ColumnNotFound(column_id.to_string())),
            };

            if let Some(name) = updates.name {
                col.insert(&mut txn, "name", Any::String(name.into()));
            }
            if let Some(color) = updates.color {
                match color {
                    Some(c) => {
                        col.insert(&mut txn, "color", Any::String(c.into()));
                    }
                    None => {
                        let _ = col.remove(&mut txn, "color");
                    }
                }
            }
            if let Some(wip_limit) = updates.wip_limit {
                match wip_limit {
                    Some(limit) => {
                        col.insert(&mut txn, "wip_limit", Any::BigInt(limit as i64));
                    }
                    None => {
                        let _ = col.remove(&mut txn, "wip_limit");
                    }
                }
            }

            // Update board timestamp
            let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
            metadata.insert(&mut txn, "updated_at", Any::BigInt(now));
        }

        self.get_column(board_id, column_id)
    }

    /// Get a column by ID.
    pub fn get_column(&self, board_id: &str, column_id: &str) -> KanbanResult<Column> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
        let columns = match root.get(&txn, keys::COLUMNS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing columns".to_string())),
        };

        let col = match columns.get(&txn, column_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::ColumnNotFound(column_id.to_string())),
        };

        self.read_column_from_map(&col, &txn)
    }

    /// Read a column from a map.
    fn read_column_from_map<T: ReadTxn>(&self, col: &MapRef, txn: &T) -> KanbanResult<Column> {
        Ok(Column {
            id: get_string(col, txn, "id")?,
            board_id: get_string(col, txn, "board_id")?,
            name: get_string(col, txn, "name")?,
            position: get_i64(col, txn, "position")? as u32,
            color: get_optional_string(col, txn, "color"),
            wip_limit: get_optional_i64(col, txn, "wip_limit").map(|v| v as u32),
            created_at: get_i64(col, txn, "created_at")?,
            deleted: get_bool(col, txn, "deleted").unwrap_or(false),
        })
    }

    /// List all columns for a board.
    #[instrument(skip(self))]
    pub fn list_columns(&self, board_id: &str) -> KanbanResult<Vec<Column>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let column_order = match root.get(&txn, keys::COLUMN_ORDER) {
            Some(Out::YArray(a)) => a,
            _ => return Err(KanbanError::InvalidData("Missing column_order".to_string())),
        };

        let columns = match root.get(&txn, keys::COLUMNS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing columns".to_string())),
        };

        let mut result = Vec::new();
        for item in column_order.iter(&txn) {
            if let Out::Any(Any::String(col_id)) = item
                && let Some(Out::YMap(col_map)) = columns.get(&txn, col_id.as_ref())
                && let Ok(col) = self.read_column_from_map(&col_map, &txn)
                && !col.deleted
            {
                result.push(col);
            }
        }

        Ok(result)
    }

    /// List all boards for an entity (project).
    ///
    /// Returns all non-deleted boards associated with the given entity_id.
    ///
    /// Note: This iterates over all documents in the cache without updating
    /// their LRU recency.
    #[instrument(skip(self))]
    pub fn list_boards(&self, entity_id: &str) -> KanbanResult<Vec<Board>> {
        let docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());
        let mut boards = Vec::new();

        // Use peek_iter to iterate without updating LRU order
        for (board_id, doc) in docs.iter() {
            let txn: yrs::TransactionMut<'_> = doc.transact_mut();
            if let Some(root) = txn.get_map("root")
                && let Some(Out::YMap(metadata)) = root.get(&txn, keys::METADATA)
                && let Ok(board) = self.read_board_from_map(&metadata, &txn)
                && !board.deleted
                && board.project_id == entity_id
            {
                // Count columns for the board
                let column_count =
                    if let Some(Out::YArray(col_order)) = root.get(&txn, keys::COLUMN_ORDER) {
                        col_order.len(&txn) as usize
                    } else {
                        0
                    };

                boards.push(Board {
                    id: board_id.clone(),
                    name: board.name,
                    description: board.description,
                    project_id: board.project_id,
                    created_by: board.created_by,
                    created_at: board.created_at,
                    updated_at: board.updated_at,
                    settings: board.settings,
                    deleted: board.deleted,
                    deleted_at: board.deleted_at,
                });
                let _ = column_count; // column_count available if needed
            }
        }

        // Sort by created_at
        boards.sort_by_key(|b| b.created_at);
        Ok(boards)
    }

    /// Delete a column.
    #[instrument(skip(self))]
    pub fn delete_column(&self, board_id: &str, column_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let columns: MapRef = root.get_or_init(&mut txn, keys::COLUMNS);

        let col = match columns.get(&txn, column_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::ColumnNotFound(column_id.to_string())),
        };

        col.insert(&mut txn, "deleted", Any::Bool(true));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, column_id = %column_id, "Deleted column");
        Ok(())
    }

    /// Move a column to a new position.
    #[instrument(skip(self))]
    pub fn move_column(
        &self,
        board_id: &str,
        column_id: &str,
        new_position: u32,
    ) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");

        // Verify column exists
        let columns: MapRef = root.get_or_init(&mut txn, keys::COLUMNS);
        if columns.get(&txn, column_id).is_none() {
            return Err(KanbanError::ColumnNotFound(column_id.to_string()));
        }

        // Get column order array
        let column_order: ArrayRef = root.get_or_init(&mut txn, keys::COLUMN_ORDER);

        // Find current position
        let mut current_pos = None;
        for (idx, item) in column_order.iter(&txn).enumerate() {
            if let Out::Any(Any::String(s)) = item
                && s.as_ref() == column_id
            {
                current_pos = Some(idx as u32);
                break;
            }
        }

        let current_pos = current_pos
            .ok_or_else(|| KanbanError::InvalidData("Column not in order array".to_string()))?;

        // Don't move if already at target position
        if current_pos == new_position {
            return Ok(());
        }

        // Remove from current position
        column_order.remove(&mut txn, current_pos);

        // Insert at new position (adjust for removal)
        let insert_pos = if new_position > current_pos {
            new_position.min(column_order.len(&txn))
        } else {
            new_position
        };
        column_order.insert(&mut txn, insert_pos, Any::String(column_id.into()));

        // Collect column IDs in order
        let col_ids: Vec<String> = column_order
            .iter(&txn)
            .filter_map(|item| {
                if let Out::Any(Any::String(s)) = item {
                    Some(s.to_string())
                } else {
                    None
                }
            })
            .collect();

        // Update position fields in column maps
        for (idx, col_id) in col_ids.iter().enumerate() {
            if let Some(Out::YMap(col)) = columns.get(&txn, col_id.as_str()) {
                col.insert(&mut txn, "position", Any::BigInt(idx as i64));
            }
        }

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, column_id = %column_id, new_position = %new_position, "Moved column");
        Ok(())
    }

    // ===== Card Operations =====

    /// Create a new card in a column.
    #[instrument(skip(self, description))]
    pub fn create_card(
        &self,
        board_id: &str,
        column_id: &str,
        title: String,
        description: Option<String>,
    ) -> KanbanResult<Card> {
        let doc = self.get_doc(board_id)?;
        let card_id = Self::new_id();
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");

        // Verify column exists
        let columns: MapRef = root.get_or_init(&mut txn, keys::COLUMNS);
        let col = match columns.get(&txn, column_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::ColumnNotFound(column_id.to_string())),
        };

        // Check WIP limit
        if let Some(wip_limit) = get_optional_i64(&col, &txn, "wip_limit")
            && let Some(Out::YArray(order)) = col.get(&txn, keys::CARD_ORDER)
        {
            let current = order.len(&txn);
            if current >= wip_limit as u32 {
                return Err(KanbanError::WipLimitExceeded {
                    column_id: column_id.to_string(),
                    limit: wip_limit as u32,
                    current,
                });
            }
        }

        // Add to column's card_order
        let card_order: ArrayRef = col.get_or_init(&mut txn, keys::CARD_ORDER);
        let position = card_order.len(&txn);
        card_order.push_back(&mut txn, Any::String(card_id.clone().into()));

        // Create card in global cards map
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        init_card_structure(&cards, &mut txn, &card_id);
        let card: MapRef = cards.get_or_init(&mut txn, card_id.as_str());

        card.insert(&mut txn, "id", Any::String(card_id.clone().into()));
        card.insert(
            &mut txn,
            "board_id",
            Any::String(board_id.to_string().into()),
        );
        card.insert(
            &mut txn,
            "column_id",
            Any::String(column_id.to_string().into()),
        );
        card.insert(&mut txn, "title", Any::String(title.clone().into()));
        card.insert(&mut txn, "position", Any::BigInt(position as i64));
        card.insert(&mut txn, "state", Any::String("Open".into()));
        card.insert(&mut txn, "is_draft", Any::Bool(false));
        card.insert(&mut txn, "is_golden", Any::Bool(false));
        card.insert(
            &mut txn,
            "created_by",
            Any::String(self.peer_id.clone().into()),
        );
        card.insert(&mut txn, "created_at", Any::BigInt(now));
        card.insert(&mut txn, "updated_at", Any::BigInt(now));
        card.insert(&mut txn, "deleted", Any::Bool(false));

        // Set description as YText
        let desc_text: TextRef = card.get_or_init(&mut txn, keys::DESCRIPTION);
        if let Some(ref desc) = description {
            desc_text.push(&mut txn, desc);
        }

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, title = %title, "Created card");

        Ok(Card {
            id: card_id,
            board_id: board_id.to_string(),
            column_id: column_id.to_string(),
            title,
            description: description.unwrap_or_default(),
            position,
            state: CardState::Open,
            priority: None,
            is_draft: false,
            is_golden: false,
            created_by: self.peer_id.clone(),
            created_at: now,
            updated_at: now,
            due_date: None,
            completed_at: None,
            assignee_ids: Vec::new(),
            tag_ids: Vec::new(),
            linked_thread_id: None,
            deleted: false,
            deleted_at: None,
            deleted_by: None,
        })
    }

    /// Get a card by ID.
    pub fn get_card(&self, board_id: &str, card_id: &str) -> KanbanResult<Card> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
        let cards = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        self.read_card_from_map(&card, &txn)
    }

    /// Read a card from a map.
    fn read_card_from_map<T: ReadTxn>(&self, card: &MapRef, txn: &T) -> KanbanResult<Card> {
        let state_str = get_string(card, txn, "state")?;
        let state = match state_str.as_str() {
            "Open" => CardState::Open,
            "Closed" => CardState::Closed,
            "Postponed" => CardState::Postponed,
            "Archived" => CardState::Archived,
            _ => CardState::Open,
        };

        let description = if let Some(Out::YText(text)) = card.get(txn, keys::DESCRIPTION) {
            text.get_string(txn)
        } else {
            String::new()
        };

        // Read assignees from OR-Set
        let assignee_ids = if let Some(Out::YMap(assignees)) = card.get(txn, keys::ASSIGNEES) {
            orset_members(&assignees, txn)
        } else {
            Vec::new()
        };

        // Read tags from OR-Set
        let tag_ids = if let Some(Out::YMap(tags)) = card.get(txn, keys::CARD_TAGS) {
            orset_members(&tags, txn)
        } else {
            Vec::new()
        };

        // Read priority from stored string
        let priority = get_optional_string(card, txn, "priority").and_then(|s| match s.as_str() {
            "Urgent" => Some(Priority::Urgent),
            "High" => Some(Priority::High),
            "Normal" => Some(Priority::Normal),
            "Low" => Some(Priority::Low),
            _ => None,
        });

        Ok(Card {
            id: get_string(card, txn, "id")?,
            board_id: get_string(card, txn, "board_id")?,
            column_id: get_string(card, txn, "column_id")?,
            title: get_string(card, txn, "title")?,
            description,
            position: get_i64(card, txn, "position")? as u32,
            state,
            priority,
            is_draft: get_bool(card, txn, "is_draft").unwrap_or(false),
            is_golden: get_bool(card, txn, "is_golden").unwrap_or(false),
            created_by: get_string(card, txn, "created_by")?,
            created_at: get_i64(card, txn, "created_at")?,
            updated_at: get_i64(card, txn, "updated_at")?,
            due_date: get_optional_i64(card, txn, "due_date"),
            completed_at: get_optional_i64(card, txn, "completed_at"),
            assignee_ids,
            tag_ids,
            linked_thread_id: get_optional_string(card, txn, "linked_thread_id"),
            deleted: get_bool(card, txn, "deleted").unwrap_or(false),
            deleted_at: get_optional_i64(card, txn, "deleted_at"),
            deleted_by: get_optional_string(card, txn, "deleted_by"),
        })
    }

    /// Update a card.
    #[instrument(skip(self, updates))]
    pub fn update_card(
        &self,
        board_id: &str,
        card_id: &str,
        updates: CardUpdate,
    ) -> KanbanResult<Card> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");
            let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
            let card = match cards.get(&txn, card_id) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
            };

            if let Some(title) = updates.title {
                card.insert(&mut txn, "title", Any::String(title.into()));
            }
            if let Some(description) = updates.description {
                let desc: TextRef = card.get_or_init(&mut txn, keys::DESCRIPTION);
                let len = desc.len(&txn);
                desc.remove_range(&mut txn, 0, len);
                desc.push(&mut txn, &description);
            }
            if let Some(is_draft) = updates.is_draft {
                card.insert(&mut txn, "is_draft", Any::Bool(is_draft));
            }
            if let Some(is_golden) = updates.is_golden {
                card.insert(&mut txn, "is_golden", Any::Bool(is_golden));
            }
            if let Some(priority) = updates.priority {
                match priority {
                    Some(p) => {
                        card.insert(&mut txn, "priority", Any::String(p.label().into()));
                    }
                    None => {
                        let _ = card.remove(&mut txn, "priority");
                    }
                }
            }
            if let Some(due_date) = updates.due_date {
                match due_date {
                    Some(date) => {
                        card.insert(&mut txn, "due_date", Any::BigInt(date));
                    }
                    None => {
                        let _ = card.remove(&mut txn, "due_date");
                    }
                }
            }
            if let Some(linked_thread_id) = updates.linked_thread_id {
                match linked_thread_id {
                    Some(thread_id) => {
                        card.insert(&mut txn, "linked_thread_id", Any::String(thread_id.into()));
                    }
                    None => {
                        let _ = card.remove(&mut txn, "linked_thread_id");
                    }
                }
            }
            card.insert(&mut txn, "updated_at", Any::BigInt(now));

            // Update board timestamp
            let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
            metadata.insert(&mut txn, "updated_at", Any::BigInt(now));
        }

        self.get_card(board_id, card_id)
    }

    /// Set the priority of a card.
    ///
    /// Pass `None` to clear the priority.
    #[instrument(skip(self))]
    pub fn set_card_priority(
        &self,
        board_id: &str,
        card_id: &str,
        priority: Option<Priority>,
    ) -> KanbanResult<Card> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");
            let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
            let card = match cards.get(&txn, card_id) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
            };

            match priority {
                Some(p) => {
                    card.insert(&mut txn, "priority", Any::String(p.label().into()));
                    debug!(board_id = %board_id, card_id = %card_id, priority = %p, "Set card priority");
                }
                None => {
                    let _ = card.remove(&mut txn, "priority");
                    debug!(board_id = %board_id, card_id = %card_id, "Cleared card priority");
                }
            }
            card.insert(&mut txn, "updated_at", Any::BigInt(now));

            // Update board timestamp
            let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
            metadata.insert(&mut txn, "updated_at", Any::BigInt(now));
        }

        self.get_card(board_id, card_id)
    }

    /// Move a card to a different column atomically.
    #[instrument(skip(self))]
    pub fn move_card(
        &self,
        board_id: &str,
        card_id: &str,
        target_column_id: &str,
        position: u32,
    ) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;

        // Get current column_id
        let source_column_id = {
            let txn = doc.transact();
            let root = txn
                .get_map("root")
                .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
            let cards = match root.get(&txn, keys::CARDS) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
            };
            let card = match cards.get(&txn, card_id) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
            };
            get_string(&card, &txn, "column_id")?
        };

        let now = Self::now();
        operations::move_card_atomic(
            &doc,
            card_id,
            &source_column_id,
            target_column_id,
            position,
            now,
        )?;

        debug!(
            board_id = %board_id,
            card_id = %card_id,
            from = %source_column_id,
            to = %target_column_id,
            "Moved card"
        );

        Ok(())
    }

    /// Change a card's state.
    #[instrument(skip(self))]
    pub fn change_card_state(
        &self,
        board_id: &str,
        card_id: &str,
        new_state: CardState,
    ) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let current_state_str = get_string(&card, &txn, "state")?;
        let current_state = match current_state_str.as_str() {
            "Open" => CardState::Open,
            "Closed" => CardState::Closed,
            "Postponed" => CardState::Postponed,
            "Archived" => CardState::Archived,
            _ => CardState::Open,
        };

        if !current_state.can_transition_to(new_state) {
            return Err(KanbanError::InvalidStateTransition {
                from: current_state,
                to: new_state,
            });
        }

        card.insert(&mut txn, "state", Any::String(new_state.label().into()));
        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Set completed_at for Closed state
        if new_state == CardState::Closed {
            card.insert(&mut txn, "completed_at", Any::BigInt(now));
        } else if current_state == CardState::Closed {
            // Clear completed_at if reopening
            let _ = card.remove(&mut txn, "completed_at");
        }

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(
            board_id = %board_id,
            card_id = %card_id,
            from = %current_state,
            to = %new_state,
            "Changed card state"
        );

        Ok(())
    }

    /// Delete a card.
    #[instrument(skip(self))]
    pub fn delete_card(&self, board_id: &str, card_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        card.insert(&mut txn, "deleted", Any::Bool(true));
        card.insert(&mut txn, "deleted_at", Any::BigInt(now));
        card.insert(
            &mut txn,
            "deleted_by",
            Any::String(self.peer_id.clone().into()),
        );
        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, "Deleted card");
        Ok(())
    }

    /// List cards in a column.
    #[instrument(skip(self))]
    pub fn list_cards_in_column(&self, board_id: &str, column_id: &str) -> KanbanResult<Vec<Card>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let columns = match root.get(&txn, keys::COLUMNS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing columns".to_string())),
        };

        let col = match columns.get(&txn, column_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::ColumnNotFound(column_id.to_string())),
        };

        let card_order = match col.get(&txn, keys::CARD_ORDER) {
            Some(Out::YArray(a)) => a,
            _ => return Err(KanbanError::InvalidData("Missing card_order".to_string())),
        };

        let cards = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        let mut result = Vec::new();
        for item in card_order.iter(&txn) {
            if let Out::Any(Any::String(card_id)) = item
                && let Some(Out::YMap(card_map)) = cards.get(&txn, card_id.as_ref())
                && let Ok(card) = self.read_card_from_map(&card_map, &txn)
                && !card.deleted
                // With concurrent moves, a card_id may exist in multiple columns' card_order arrays.
                // The card's column_id (LWW) is the authoritative source - only include cards
                // that actually belong to this column.
                && card.column_id == column_id
            {
                result.push(card);
            }
        }

        Ok(result)
    }

    /// Filter cards in a board.
    #[instrument(skip(self, filter))]
    pub fn filter_cards(&self, board_id: &str, filter: CardFilter) -> KanbanResult<Vec<Card>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let cards = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        let now = Self::now();
        let mut result = Vec::new();

        for (_, value) in cards.iter(&txn) {
            if let Out::YMap(card_map) = value
                && let Ok(card) = self.read_card_from_map(&card_map, &txn)
                && filter.matches(&card, now)
            {
                result.push(card);
            }
        }

        // Sort by position
        result.sort_by_key(|c| c.position);

        Ok(result)
    }

    // ===== Assignment Operations =====

    /// Assign a user to a card.
    #[instrument(skip(self))]
    pub fn assign_user(&self, board_id: &str, card_id: &str, user_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let assignees: MapRef = card.get_or_init(&mut txn, keys::ASSIGNEES);
        orset_add(&assignees, &mut txn, user_id, now)?;

        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, user_id = %user_id, "Assigned user");
        Ok(())
    }

    /// Unassign a user from a card.
    #[instrument(skip(self))]
    pub fn unassign_user(&self, board_id: &str, card_id: &str, user_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let assignees: MapRef = card.get_or_init(&mut txn, keys::ASSIGNEES);
        orset_remove(&assignees, &mut txn, user_id, now)?;

        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, user_id = %user_id, "Unassigned user");
        Ok(())
    }

    // ===== Tag Operations =====

    /// Create a tag for a board.
    #[instrument(skip(self))]
    pub fn create_tag(&self, board_id: &str, name: String, color: String) -> KanbanResult<Tag> {
        let doc = self.get_doc(board_id)?;
        let tag_id = Self::new_id();
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let tags: MapRef = root.get_or_init(&mut txn, keys::TAGS);

        let tag: MapRef = tags.get_or_init(&mut txn, tag_id.as_str());
        tag.insert(&mut txn, "id", Any::String(tag_id.clone().into()));
        tag.insert(
            &mut txn,
            "board_id",
            Any::String(board_id.to_string().into()),
        );
        tag.insert(&mut txn, "name", Any::String(name.clone().into()));
        tag.insert(&mut txn, "color", Any::String(color.clone().into()));
        tag.insert(&mut txn, "deleted", Any::Bool(false));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, tag_id = %tag_id, name = %name, "Created tag");

        Ok(Tag {
            id: tag_id,
            board_id: board_id.to_string(),
            name,
            color,
            deleted: false,
        })
    }

    /// Tag a card.
    #[instrument(skip(self))]
    pub fn tag_card(&self, board_id: &str, card_id: &str, tag_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");

        // Verify tag exists
        let tags: MapRef = root.get_or_init(&mut txn, keys::TAGS);
        if tags.get(&txn, tag_id).is_none() {
            return Err(KanbanError::TagNotFound(tag_id.to_string()));
        }

        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let card_tags: MapRef = card.get_or_init(&mut txn, keys::CARD_TAGS);
        orset_add(&card_tags, &mut txn, tag_id, now)?;

        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, tag_id = %tag_id, "Tagged card");
        Ok(())
    }

    /// Untag a card.
    #[instrument(skip(self))]
    pub fn untag_card(&self, board_id: &str, card_id: &str, tag_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let card_tags: MapRef = card.get_or_init(&mut txn, keys::CARD_TAGS);
        orset_remove(&card_tags, &mut txn, tag_id, now)?;

        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, tag_id = %tag_id, "Untagged card");
        Ok(())
    }

    /// List all tags for a board.
    #[instrument(skip(self))]
    pub fn list_tags(&self, board_id: &str) -> KanbanResult<Vec<Tag>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let tags = match root.get(&txn, keys::TAGS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing tags".to_string())),
        };

        let mut result = Vec::new();
        for (_, value) in tags.iter(&txn) {
            if let Out::YMap(tag_map) = value
                && !get_bool(&tag_map, &txn, "deleted").unwrap_or(false)
                && let (Ok(id), Ok(board_id), Ok(name), Ok(color)) = (
                    get_string(&tag_map, &txn, "id"),
                    get_string(&tag_map, &txn, "board_id"),
                    get_string(&tag_map, &txn, "name"),
                    get_string(&tag_map, &txn, "color"),
                )
            {
                result.push(Tag {
                    id,
                    board_id,
                    name,
                    color,
                    deleted: false,
                });
            }
        }

        Ok(result)
    }

    // ===== Step Operations =====

    /// Add a step (checklist item) to a card.
    #[instrument(skip(self))]
    pub fn add_step(
        &self,
        board_id: &str,
        card_id: &str,
        text: String,
        position: Option<u32>,
    ) -> KanbanResult<Step> {
        let doc = self.get_doc(board_id)?;
        let step_id = Self::new_id();
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let step_order: ArrayRef = card.get_or_init(&mut txn, keys::STEP_ORDER);
        let steps: MapRef = card.get_or_init(&mut txn, keys::STEPS);

        // Determine position
        let len = step_order.len(&txn);
        let pos = position.unwrap_or(len);
        let actual_pos = pos.min(len);

        // Add to step_order
        step_order.insert(&mut txn, actual_pos, Any::String(step_id.clone().into()));

        // Create step entry
        let step: MapRef = steps.get_or_init(&mut txn, step_id.as_str());
        step.insert(&mut txn, "id", Any::String(step_id.clone().into()));
        step.insert(&mut txn, "card_id", Any::String(card_id.to_string().into()));
        step.insert(&mut txn, "text", Any::String(text.clone().into()));
        step.insert(&mut txn, "completed", Any::Bool(false));
        step.insert(&mut txn, "position", Any::BigInt(actual_pos as i64));
        step.insert(&mut txn, "created_at", Any::BigInt(now));
        step.insert(&mut txn, "deleted", Any::Bool(false));

        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, step_id = %step_id, "Added step");

        Ok(Step {
            id: step_id,
            card_id: card_id.to_string(),
            text,
            completed: false,
            completed_by: None,
            completed_at: None,
            position: actual_pos,
            created_at: now,
            deleted: false,
        })
    }

    /// Toggle a step's completion status.
    #[instrument(skip(self))]
    pub fn toggle_step(&self, board_id: &str, card_id: &str, step_id: &str) -> KanbanResult<Step> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");
            let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
            let card = match cards.get(&txn, card_id) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
            };

            let steps: MapRef = card.get_or_init(&mut txn, keys::STEPS);
            let step = match steps.get(&txn, step_id) {
                Some(Out::YMap(m)) => m,
                _ => return Err(KanbanError::StepNotFound(step_id.to_string())),
            };

            let was_completed = get_bool(&step, &txn, "completed").unwrap_or(false);

            if was_completed {
                step.insert(&mut txn, "completed", Any::Bool(false));
                let _ = step.remove(&mut txn, "completed_by");
                let _ = step.remove(&mut txn, "completed_at");
            } else {
                step.insert(&mut txn, "completed", Any::Bool(true));
                step.insert(
                    &mut txn,
                    "completed_by",
                    Any::String(self.peer_id.clone().into()),
                );
                step.insert(&mut txn, "completed_at", Any::BigInt(now));
            }

            card.insert(&mut txn, "updated_at", Any::BigInt(now));

            // Update board timestamp
            let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
            metadata.insert(&mut txn, "updated_at", Any::BigInt(now));
        }

        self.get_step(board_id, card_id, step_id)
    }

    /// Get a step by ID.
    pub fn get_step(&self, board_id: &str, card_id: &str, step_id: &str) -> KanbanResult<Step> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
        let cards = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };
        let steps = match card.get(&txn, keys::STEPS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing steps".to_string())),
        };
        let step = match steps.get(&txn, step_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::StepNotFound(step_id.to_string())),
        };

        Ok(Step {
            id: get_string(&step, &txn, "id")?,
            card_id: get_string(&step, &txn, "card_id")?,
            text: get_string(&step, &txn, "text")?,
            completed: get_bool(&step, &txn, "completed").unwrap_or(false),
            completed_by: get_optional_string(&step, &txn, "completed_by"),
            completed_at: get_optional_i64(&step, &txn, "completed_at"),
            position: get_i64(&step, &txn, "position")? as u32,
            created_at: get_i64(&step, &txn, "created_at")?,
            deleted: get_bool(&step, &txn, "deleted").unwrap_or(false),
        })
    }

    /// List all steps for a card in position order.
    pub fn list_steps(&self, board_id: &str, card_id: &str) -> KanbanResult<Vec<Step>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
        let cards = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let steps = match card.get(&txn, keys::STEPS) {
            Some(Out::YMap(m)) => Some(m),
            _ => None,
        };

        let step_order = match card.get(&txn, keys::STEP_ORDER) {
            Some(Out::YArray(a)) => Some(a),
            _ => None,
        };

        let mut result = Vec::new();
        if let (Some(steps), Some(step_order)) = (steps, step_order) {
            for value in step_order.iter(&txn) {
                let Out::Any(Any::String(step_id)) = value else {
                    continue;
                };
                let Some(Out::YMap(step_map)) = steps.get(&txn, step_id.as_ref()) else {
                    continue;
                };
                let deleted = get_bool(&step_map, &txn, "deleted").unwrap_or(false);
                if deleted {
                    continue;
                }
                let (Ok(id), Ok(c_id), Ok(text), Ok(position), Ok(created_at)) = (
                    get_string(&step_map, &txn, "id"),
                    get_string(&step_map, &txn, "card_id"),
                    get_string(&step_map, &txn, "text"),
                    get_i64(&step_map, &txn, "position"),
                    get_i64(&step_map, &txn, "created_at"),
                ) else {
                    continue;
                };
                result.push(Step {
                    id,
                    card_id: c_id,
                    text,
                    completed: get_bool(&step_map, &txn, "completed").unwrap_or(false),
                    completed_by: get_optional_string(&step_map, &txn, "completed_by"),
                    completed_at: get_optional_i64(&step_map, &txn, "completed_at"),
                    position: position as u32,
                    created_at,
                    deleted: false,
                });
            }
        }

        // Sort by position to ensure consistent order
        result.sort_by_key(|s| s.position);
        Ok(result)
    }

    /// Delete a step.
    #[instrument(skip(self))]
    pub fn delete_step(&self, board_id: &str, card_id: &str, step_id: &str) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let steps: MapRef = card.get_or_init(&mut txn, keys::STEPS);
        let step = match steps.get(&txn, step_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::StepNotFound(step_id.to_string())),
        };

        step.insert(&mut txn, "deleted", Any::Bool(true));
        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, step_id = %step_id, "Deleted step");
        Ok(())
    }

    // ===== Comment Operations =====

    /// Add a comment to a card.
    #[instrument(skip(self))]
    pub fn add_comment(
        &self,
        board_id: &str,
        card_id: &str,
        content: String,
        reply_to_id: Option<String>,
    ) -> KanbanResult<Comment> {
        let doc = self.get_doc(board_id)?;
        let comment_id = Self::new_id();
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let comments: MapRef = card.get_or_init(&mut txn, keys::COMMENTS);
        let comment: MapRef = comments.get_or_init(&mut txn, comment_id.as_str());

        comment.insert(&mut txn, "id", Any::String(comment_id.clone().into()));
        comment.insert(&mut txn, "card_id", Any::String(card_id.to_string().into()));
        comment.insert(
            &mut txn,
            "author_id",
            Any::String(self.peer_id.clone().into()),
        );
        comment.insert(&mut txn, "created_at", Any::BigInt(now));
        comment.insert(&mut txn, "updated_at", Any::BigInt(now));
        comment.insert(&mut txn, "deleted", Any::Bool(false));

        if let Some(ref reply_id) = reply_to_id {
            comment.insert(
                &mut txn,
                "reply_to_id",
                Any::String(reply_id.clone().into()),
            );
        }

        // Set content as YText
        let content_text: TextRef = comment.get_or_init(&mut txn, keys::CONTENT);
        content_text.push(&mut txn, &content);

        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, comment_id = %comment_id, "Added comment");

        Ok(Comment {
            id: comment_id,
            card_id: card_id.to_string(),
            author_id: self.peer_id.clone(),
            content,
            created_at: now,
            updated_at: now,
            reply_to_id,
            deleted: false,
        })
    }

    /// List comments on a card.
    #[instrument(skip(self))]
    pub fn list_comments(&self, board_id: &str, card_id: &str) -> KanbanResult<Vec<Comment>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;
        let cards = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let comments = match card.get(&txn, keys::COMMENTS) {
            Some(Out::YMap(m)) => Some(m),
            _ => None,
        };

        let mut result = Vec::new();
        if let Some(comments) = comments {
            for (_, value) in comments.iter(&txn) {
                if let Out::YMap(comment_map) = value {
                    let deleted = get_bool(&comment_map, &txn, "deleted").unwrap_or(false);
                    if !deleted {
                        let content =
                            if let Some(Out::YText(text)) = comment_map.get(&txn, keys::CONTENT) {
                                text.get_string(&txn)
                            } else {
                                String::new()
                            };

                        if let (
                            Ok(id),
                            Ok(card_id),
                            Ok(author_id),
                            Ok(created_at),
                            Ok(updated_at),
                        ) = (
                            get_string(&comment_map, &txn, "id"),
                            get_string(&comment_map, &txn, "card_id"),
                            get_string(&comment_map, &txn, "author_id"),
                            get_i64(&comment_map, &txn, "created_at"),
                            get_i64(&comment_map, &txn, "updated_at"),
                        ) {
                            result.push(Comment {
                                id,
                                card_id,
                                author_id,
                                content,
                                created_at,
                                updated_at,
                                reply_to_id: get_optional_string(&comment_map, &txn, "reply_to_id"),
                                deleted: false,
                            });
                        }
                    }
                }
            }
        }

        // Sort by created_at
        result.sort_by_key(|c| c.created_at);

        Ok(result)
    }

    /// Delete a comment.
    #[instrument(skip(self))]
    pub fn delete_comment(
        &self,
        board_id: &str,
        card_id: &str,
        comment_id: &str,
    ) -> KanbanResult<()> {
        let doc = self.get_doc(board_id)?;
        let now = Self::now();

        let mut txn = doc.transact_mut();
        let root = txn.get_or_insert_map("root");
        let cards: MapRef = root.get_or_init(&mut txn, keys::CARDS);
        let card = match cards.get(&txn, card_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CardNotFound(card_id.to_string())),
        };

        let comments: MapRef = card.get_or_init(&mut txn, keys::COMMENTS);
        let comment = match comments.get(&txn, comment_id) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::CommentNotFound(comment_id.to_string())),
        };

        comment.insert(&mut txn, "deleted", Any::Bool(true));
        card.insert(&mut txn, "updated_at", Any::BigInt(now));

        // Update board timestamp
        let metadata: MapRef = root.get_or_init(&mut txn, keys::METADATA);
        metadata.insert(&mut txn, "updated_at", Any::BigInt(now));

        debug!(board_id = %board_id, card_id = %card_id, comment_id = %comment_id, "Deleted comment");
        Ok(())
    }

    // ===== Sync Operations =====

    /// Get the state vector for a board document.
    pub fn get_state_vector(&self, board_id: &str) -> KanbanResult<Vec<u8>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        Ok(txn.state_vector().encode_v1())
    }

    /// Get an update from a state vector.
    ///
    /// Use this to get incremental updates based on the remote peer's state.
    pub fn get_update(&self, board_id: &str, state_vector: &[u8]) -> KanbanResult<Vec<u8>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let sv = yrs::StateVector::decode_v1(state_vector)
            .map_err(|e| KanbanError::Crdt(e.to_string()))?;
        Ok(txn.encode_diff_v1(&sv))
    }

    /// Get a full update encoding all document state.
    ///
    /// Use this to sync to a new peer that has no prior state.
    pub fn get_full_update(&self, board_id: &str) -> KanbanResult<Vec<u8>> {
        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        // Use an empty state vector to get the full diff
        Ok(txn.encode_diff_v1(&yrs::StateVector::default()))
    }

    /// Apply an update to a board document.
    ///
    /// For new peers receiving their first update, this creates an empty document
    /// without initialization - the update itself contains the full structure.
    pub fn apply_update(&self, board_id: &str, update: &[u8]) -> KanbanResult<()> {
        // Get or create a bare document (no initialization) for receiving updates.
        // The update itself will contain the document structure.
        let doc = self.get_or_create_bare_doc(board_id);
        let mut txn = doc.transact_mut();
        let update =
            yrs::Update::decode_v1(update).map_err(|e| KanbanError::Crdt(e.to_string()))?;
        txn.apply_update(update);
        Ok(())
    }

    /// Get or create an empty document for a board (no initialization).
    ///
    /// Use this when receiving updates from other peers - the update will
    /// populate the document structure.
    ///
    /// When the LRU cache is full, the least-recently-used document will
    /// be evicted and its subscriptions cleaned up.
    fn get_or_create_bare_doc(&self, board_id: &str) -> Arc<Doc> {
        let mut docs = self.documents.lock().unwrap_or_else(|e| e.into_inner());

        // Check if doc exists and return it (this also marks it as recently used)
        if let Some(doc) = docs.get(board_id) {
            return doc.clone();
        }

        // Doc doesn't exist - check if we need to evict
        if docs.len() >= MAX_BOARD_DOCUMENTS
            && let Some((evicted_id, _)) = docs.peek_lru()
        {
            let evicted_id = evicted_id.clone();
            self.cleanup_evicted_subscription(&evicted_id);
            info!(
                board_id = %evicted_id,
                cache_size = docs.len(),
                "Evicting LRU board document from cache (bare doc)"
            );
        }

        // Create new document (without initialization)
        let doc_arc = Arc::new(Doc::new());
        docs.put(board_id.to_string(), doc_arc.clone());
        doc_arc
    }

    // ===== CRDT Event Subscription =====

    /// Subscribe to CRDT changes for a board.
    ///
    /// Returns a channel receiver that will emit [`KanbanEvent`]s whenever
    /// the board's CRDT document changes (from local operations or remote sync).
    ///
    /// The subscription uses `observe_after_transaction` to detect changes
    /// after each transaction commits. Events are simplified to `BoardChanged`
    /// to allow the subscriber to refresh the entire board state.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to subscribe to
    ///
    /// # Returns
    ///
    /// A channel receiver for `KanbanEvent`s. The channel has a buffer of 64
    /// events to handle burst updates.
    ///
    /// # Errors
    ///
    /// Returns an error if the board doesn't exist.
    #[instrument(skip(self))]
    pub fn subscribe_to_changes(
        &self,
        board_id: &str,
    ) -> KanbanResult<mpsc::Receiver<KanbanEvent>> {
        let doc = self.get_doc(board_id)?;

        // Create channel with buffer for bursts
        let (tx, rx) = mpsc::channel(64);

        // Clone board_id for the closure
        let board_id_owned = board_id.to_string();
        let sender = tx.clone();

        // Subscribe to after-transaction events
        let subscription = doc
            .observe_after_transaction(move |_txn| {
                // Emit a generic BoardChanged event
                // The subscriber can decide to refresh the full board state
                let event = KanbanEvent::BoardChanged {
                    board_id: board_id_owned.clone(),
                };

                // Use try_send to avoid blocking if the receiver is slow
                if sender.try_send(event).is_err() {
                    warn!(
                        target = "kanban.crdt",
                        board_id = %board_id_owned,
                        "Failed to send CRDT event - channel full or closed"
                    );
                }
            })
            .map_err(|e| {
                KanbanError::Crdt(format!("Failed to subscribe to board changes: {:?}", e))
            })?;

        // Store the subscription to keep it alive
        let board_sub = BoardSubscription {
            _subscription: subscription,
            _sender: tx,
        };

        let mut subs = self
            .subscriptions
            .write()
            .unwrap_or_else(|e| e.into_inner());
        subs.insert(board_id.to_string(), board_sub);

        debug!(board_id = %board_id, "Subscribed to CRDT changes");
        Ok(rx)
    }

    /// Unsubscribe from CRDT changes for a board.
    ///
    /// This removes the active subscription and drops the Yrs observer.
    /// The channel receiver will stop receiving events after this is called.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to unsubscribe from
    #[instrument(skip(self))]
    pub fn unsubscribe(&self, board_id: &str) {
        let mut subs = self
            .subscriptions
            .write()
            .unwrap_or_else(|e| e.into_inner());
        if subs.remove(board_id).is_some() {
            debug!(board_id = %board_id, "Unsubscribed from CRDT changes");
        }
    }

    /// Check if a board has an active subscription.
    #[must_use]
    pub fn has_subscription(&self, board_id: &str) -> bool {
        let subs = self.subscriptions.read().unwrap_or_else(|e| e.into_inner());
        subs.contains_key(board_id)
    }

    // ===== Analytics Operations =====

    /// Get analytics summary for a board.
    ///
    /// Calculates current state metrics including card counts per column,
    /// completion rates, WIP, and cycle time averages.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to analyze
    ///
    /// # Errors
    ///
    /// Returns an error if the board doesn't exist.
    #[instrument(skip(self))]
    pub fn get_board_analytics(
        &self,
        board_id: &str,
    ) -> KanbanResult<crate::analytics::BoardAnalytics> {
        use std::collections::HashMap;

        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let now = Self::now();
        let week_ago = now - 7 * 86400;
        let month_ago = now - 30 * 86400;
        let week_ahead = now + 7 * 86400;

        let cards_map = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        let columns_map = match root.get(&txn, keys::COLUMNS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing columns".to_string())),
        };

        let mut column_card_counts: HashMap<String, usize> = HashMap::new();
        let mut total_active_cards = 0usize;
        let mut completed_this_week = 0usize;
        let mut completed_this_month = 0usize;
        let mut wip_count = 0usize;
        let mut overdue_count = 0usize;
        let mut due_soon_count = 0usize;
        let mut cycle_times: Vec<i64> = Vec::new();

        // Initialize column counts
        for (col_id, _) in columns_map.iter(&txn) {
            column_card_counts.insert(col_id.to_string(), 0);
        }

        // Analyze each card
        for (_, value) in cards_map.iter(&txn) {
            if let Out::YMap(card_map) = value
                && let Ok(card) = self.read_card_from_map(&card_map, &txn)
                && !card.deleted
            {
                total_active_cards += 1;

                // Count per column
                *column_card_counts
                    .entry(card.column_id.clone())
                    .or_insert(0) += 1;

                // Count completed cards
                if let Some(completed_at) = card.completed_at {
                    if completed_at >= week_ago {
                        completed_this_week += 1;
                    }
                    if completed_at >= month_ago {
                        completed_this_month += 1;
                    }

                    // Calculate cycle time
                    let cycle_time = completed_at - card.created_at;
                    if cycle_time > 0 {
                        cycle_times.push(cycle_time);
                    }
                }

                // Count WIP (active cards)
                if card.state.is_active() {
                    wip_count += 1;
                }

                // Count overdue and due soon
                if let Some(due_date) = card.due_date
                    && card.state.is_active()
                {
                    if due_date < now {
                        overdue_count += 1;
                    } else if due_date <= week_ahead {
                        due_soon_count += 1;
                    }
                }
            }
        }

        // Calculate average cycle time
        let avg_cycle_time_seconds = if cycle_times.is_empty() {
            None
        } else {
            Some(cycle_times.iter().sum::<i64>() / cycle_times.len() as i64)
        };

        debug!(
            board_id = %board_id,
            total_active_cards = %total_active_cards,
            completed_this_week = %completed_this_week,
            "Calculated board analytics"
        );

        Ok(crate::analytics::BoardAnalytics {
            board_id: board_id.to_string(),
            calculated_at: now,
            column_card_counts,
            total_active_cards,
            completed_this_week,
            completed_this_month,
            avg_cycle_time_seconds,
            avg_time_per_column: HashMap::new(), // Would require transition tracking
            wip_count,
            overdue_count,
            due_soon_count,
        })
    }

    /// Calculate velocity metrics for a board.
    ///
    /// Velocity is measured as the number of cards completed per time period.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to analyze
    /// * `time_range` - Time range to calculate velocity over
    ///
    /// # Errors
    ///
    /// Returns an error if the board doesn't exist.
    #[instrument(skip(self))]
    pub fn calculate_velocity(
        &self,
        board_id: &str,
        time_range: crate::analytics::TimeRange,
    ) -> KanbanResult<crate::analytics::VelocityMetric> {
        use crate::analytics::{VelocityDataPoint, VelocityMetric};

        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let now = Self::now();
        let (start_time, end_time) = time_range.to_timestamps(now);
        let period_duration = time_range.period_duration_seconds();

        let cards_map = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        // Collect completed cards with their completion times
        let mut completions: Vec<i64> = Vec::new();
        for (_, value) in cards_map.iter(&txn) {
            if let Out::YMap(card_map) = value
                && let Ok(card) = self.read_card_from_map(&card_map, &txn)
                && !card.deleted
                && let Some(completed_at) = card.completed_at
                && completed_at >= start_time
                && completed_at <= end_time
            {
                completions.push(completed_at);
            }
        }

        // Sort completions by time
        completions.sort_unstable();

        // Build data points for each period
        let mut data_points: Vec<VelocityDataPoint> = Vec::new();
        let mut period_start = start_time;

        while period_start < end_time {
            let period_end = period_start + period_duration;
            let cards_completed = completions
                .iter()
                .filter(|&&t| t >= period_start && t < period_end)
                .count();

            data_points.push(VelocityDataPoint {
                period_start,
                period_end,
                cards_completed,
            });

            period_start = period_end;
        }

        debug!(
            board_id = %board_id,
            periods = %data_points.len(),
            total_completed = %completions.len(),
            "Calculated velocity"
        );

        Ok(VelocityMetric::from_data_points(
            board_id.to_string(),
            time_range,
            data_points,
        ))
    }

    /// Calculate burndown chart data for a board.
    ///
    /// Shows remaining work over time compared to an ideal burndown line.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to analyze
    /// * `period_start` - Start timestamp for the burndown period
    /// * `period_end` - End timestamp for the burndown period
    ///
    /// # Errors
    ///
    /// Returns an error if the board doesn't exist.
    #[instrument(skip(self))]
    pub fn calculate_burndown(
        &self,
        board_id: &str,
        period_start: i64,
        period_end: i64,
    ) -> KanbanResult<crate::analytics::BurndownChart> {
        use crate::analytics::{BurndownChart, BurndownDataPoint};

        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let now = Self::now();

        let cards_map = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        // Collect card data
        let mut initial_cards: Vec<(i64, Option<i64>)> = Vec::new(); // (created_at, completed_at)
        let mut added_after_start = 0usize;

        for (_, value) in cards_map.iter(&txn) {
            if let Out::YMap(card_map) = value
                && let Ok(card) = self.read_card_from_map(&card_map, &txn)
                && !card.deleted
            {
                // Card existed at start if created before period_start
                if card.created_at <= period_start {
                    initial_cards.push((card.created_at, card.completed_at));
                } else if card.created_at <= period_end {
                    // Card added during period
                    initial_cards.push((card.created_at, card.completed_at));
                    added_after_start += 1;
                }
            }
        }

        let initial_card_count = initial_cards.len().saturating_sub(added_after_start);
        let current_scope = initial_cards.len();

        // Build ideal line (linear from initial count to 0)
        let period_duration = period_end - period_start;
        let day = 86400i64;
        let mut ideal_line: Vec<(i64, f64)> = Vec::new();
        let num_days = (period_duration / day).max(1) as usize;

        for i in 0..=num_days {
            let t = period_start + (i as i64 * day);
            let remaining = if period_duration > 0 {
                initial_card_count as f64
                    * (1.0 - (t - period_start) as f64 / period_duration as f64)
            } else {
                0.0
            };
            ideal_line.push((t, remaining.max(0.0)));
        }

        // Build actual line (daily snapshots)
        let mut actual_line: Vec<BurndownDataPoint> = Vec::new();
        let effective_end = now.min(period_end);

        for i in 0..=num_days {
            let snapshot_time = period_start + (i as i64 * day);
            if snapshot_time > effective_end {
                break;
            }

            // Count cards at this snapshot
            let mut remaining = 0usize;
            let mut completed = 0usize;
            let mut added = 0usize;

            for &(created_at, completed_at) in &initial_cards {
                // Card exists if created before snapshot
                if created_at <= snapshot_time {
                    if created_at > period_start {
                        added += 1;
                    }

                    // Check if completed before snapshot
                    if let Some(ct) = completed_at {
                        if ct <= snapshot_time {
                            completed += 1;
                        } else {
                            remaining += 1;
                        }
                    } else {
                        remaining += 1;
                    }
                }
            }

            actual_line.push(BurndownDataPoint {
                timestamp: snapshot_time,
                remaining,
                completed,
                added,
            });
        }

        let scope_change = added_after_start as i32;

        debug!(
            board_id = %board_id,
            initial_card_count = %initial_card_count,
            scope_change = %scope_change,
            "Calculated burndown"
        );

        Ok(BurndownChart {
            board_id: board_id.to_string(),
            period_start,
            period_end,
            initial_card_count,
            ideal_line,
            actual_line,
            current_scope,
            scope_change,
        })
    }

    /// Calculate cycle time distribution for a board.
    ///
    /// Shows how long cards typically take to complete.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to analyze
    /// * `time_range` - Only include cards completed within this range
    ///
    /// # Errors
    ///
    /// Returns an error if the board doesn't exist.
    #[instrument(skip(self))]
    pub fn calculate_cycle_time_distribution(
        &self,
        board_id: &str,
        time_range: crate::analytics::TimeRange,
    ) -> KanbanResult<crate::analytics::CycleTimeDistribution> {
        use crate::analytics::CycleTimeDistribution;

        let doc = self.get_doc(board_id)?;
        let txn = doc.transact();
        let root = txn
            .get_map("root")
            .ok_or_else(|| KanbanError::InvalidData("Missing root map".to_string()))?;

        let now = Self::now();
        let (start_time, end_time) = time_range.to_timestamps(now);

        let cards_map = match root.get(&txn, keys::CARDS) {
            Some(Out::YMap(m)) => m,
            _ => return Err(KanbanError::InvalidData("Missing cards".to_string())),
        };

        // Collect cycle times for completed cards
        let mut cycle_times: Vec<i64> = Vec::new();
        for (_, value) in cards_map.iter(&txn) {
            if let Out::YMap(card_map) = value
                && let Ok(card) = self.read_card_from_map(&card_map, &txn)
                && !card.deleted
                && let Some(completed_at) = card.completed_at
                && completed_at >= start_time
                && completed_at <= end_time
            {
                let cycle_time = completed_at - card.created_at;
                if cycle_time > 0 {
                    cycle_times.push(cycle_time);
                }
            }
        }

        debug!(
            board_id = %board_id,
            sample_size = %cycle_times.len(),
            "Calculated cycle time distribution"
        );

        Ok(CycleTimeDistribution::from_cycle_times(
            board_id.to_string(),
            cycle_times,
        ))
    }
}

// ===== Helper Functions =====

/// Get a string from a map.
fn get_string<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> KanbanResult<String> {
    match map.get(txn, key) {
        Some(Out::Any(Any::String(s))) => Ok(s.to_string()),
        _ => Err(KanbanError::InvalidData(format!(
            "Missing string field: {}",
            key
        ))),
    }
}

/// Get an optional string from a map.
fn get_optional_string<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<String> {
    match map.get(txn, key) {
        Some(Out::Any(Any::String(s))) => Some(s.to_string()),
        _ => None,
    }
}

/// Get an i64 from a map.
fn get_i64<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> KanbanResult<i64> {
    match map.get(txn, key) {
        Some(Out::Any(Any::BigInt(i))) => Ok(i),
        Some(Out::Any(Any::Number(n))) => Ok(n as i64),
        _ => Err(KanbanError::InvalidData(format!(
            "Missing i64 field: {}",
            key
        ))),
    }
}

/// Get an optional i64 from a map.
fn get_optional_i64<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<i64> {
    match map.get(txn, key) {
        Some(Out::Any(Any::BigInt(i))) => Some(i),
        Some(Out::Any(Any::Number(n))) => Some(n as i64),
        _ => None,
    }
}

/// Get a bool from a map.
fn get_bool<T: ReadTxn>(map: &MapRef, txn: &T, key: &str) -> Option<bool> {
    match map.get(txn, key) {
        Some(Out::Any(Any::Bool(b))) => Some(b),
        _ => None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_create_board() {
        let service = KanbanService::new("ocean-forest-moon-star");
        let board = service
            .create_board("project-one", "Test Board".to_string(), None)
            .unwrap();

        assert_eq!(board.name, "Test Board");
        assert_eq!(board.project_id, "project-one");
        assert_eq!(board.created_by, "ocean-forest-moon-star");
        assert!(!board.deleted);
    }

    #[test]
    fn test_add_columns() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();

        let col1 = service
            .add_column(&board.id, "To Do".to_string(), None)
            .unwrap();
        let col2 = service
            .add_column(&board.id, "Done".to_string(), None)
            .unwrap();

        assert_eq!(col1.position, 0);
        assert_eq!(col2.position, 1);

        let columns = service.list_columns(&board.id).unwrap();
        assert_eq!(columns.len(), 2);
    }

    #[test]
    fn test_create_and_move_card() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col1 = service
            .add_column(&board.id, "To Do".to_string(), None)
            .unwrap();
        let col2 = service
            .add_column(&board.id, "Done".to_string(), None)
            .unwrap();

        let card = service
            .create_card(&board.id, &col1.id, "Task 1".to_string(), None)
            .unwrap();
        assert_eq!(card.column_id, col1.id);

        service.move_card(&board.id, &card.id, &col2.id, 0).unwrap();

        let moved_card = service.get_card(&board.id, &card.id).unwrap();
        assert_eq!(moved_card.column_id, col2.id);
    }

    #[test]
    fn test_card_state_transitions() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col = service
            .add_column(&board.id, "Col".to_string(), None)
            .unwrap();
        let card = service
            .create_card(&board.id, &col.id, "Task".to_string(), None)
            .unwrap();

        assert_eq!(card.state, CardState::Open);

        service
            .change_card_state(&board.id, &card.id, CardState::Closed)
            .unwrap();
        let card = service.get_card(&board.id, &card.id).unwrap();
        assert_eq!(card.state, CardState::Closed);
        assert!(card.completed_at.is_some());

        // Invalid transition should fail
        let result = service.change_card_state(&board.id, &card.id, CardState::Postponed);
        assert!(result.is_err());
    }

    #[test]
    fn test_assignments() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col = service
            .add_column(&board.id, "Col".to_string(), None)
            .unwrap();
        let card = service
            .create_card(&board.id, &col.id, "Task".to_string(), None)
            .unwrap();

        service.assign_user(&board.id, &card.id, "user-2").unwrap();
        service.assign_user(&board.id, &card.id, "user-3").unwrap();

        let card = service.get_card(&board.id, &card.id).unwrap();
        assert_eq!(card.assignee_ids.len(), 2);
        assert!(card.assignee_ids.contains(&"user-2".to_string()));
        assert!(card.assignee_ids.contains(&"user-3".to_string()));

        service
            .unassign_user(&board.id, &card.id, "user-2")
            .unwrap();
        let card = service.get_card(&board.id, &card.id).unwrap();
        assert_eq!(card.assignee_ids.len(), 1);
        assert!(!card.assignee_ids.contains(&"user-2".to_string()));
    }

    #[test]
    fn test_tags() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col = service
            .add_column(&board.id, "Col".to_string(), None)
            .unwrap();
        let card = service
            .create_card(&board.id, &col.id, "Task".to_string(), None)
            .unwrap();

        let tag = service
            .create_tag(&board.id, "Bug".to_string(), "#FF0000".to_string())
            .unwrap();
        service.tag_card(&board.id, &card.id, &tag.id).unwrap();

        let card = service.get_card(&board.id, &card.id).unwrap();
        assert!(card.tag_ids.contains(&tag.id));

        service.untag_card(&board.id, &card.id, &tag.id).unwrap();
        let card = service.get_card(&board.id, &card.id).unwrap();
        assert!(!card.tag_ids.contains(&tag.id));
    }

    #[test]
    fn test_steps() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col = service
            .add_column(&board.id, "Col".to_string(), None)
            .unwrap();
        let card = service
            .create_card(&board.id, &col.id, "Task".to_string(), None)
            .unwrap();

        let step = service
            .add_step(&board.id, &card.id, "Step 1".to_string(), None)
            .unwrap();
        assert!(!step.completed);

        let step = service.toggle_step(&board.id, &card.id, &step.id).unwrap();
        assert!(step.completed);
        assert!(step.completed_by.is_some());

        let step = service.toggle_step(&board.id, &card.id, &step.id).unwrap();
        assert!(!step.completed);
    }

    #[test]
    fn test_comments() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col = service
            .add_column(&board.id, "Col".to_string(), None)
            .unwrap();
        let card = service
            .create_card(&board.id, &col.id, "Task".to_string(), None)
            .unwrap();

        let comment = service
            .add_comment(&board.id, &card.id, "First comment".to_string(), None)
            .unwrap();
        assert_eq!(comment.content, "First comment");

        let reply = service
            .add_comment(
                &board.id,
                &card.id,
                "Reply".to_string(),
                Some(comment.id.clone()),
            )
            .unwrap();
        assert_eq!(reply.reply_to_id, Some(comment.id));

        let comments = service.list_comments(&board.id, &card.id).unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[test]
    fn test_filter_cards() {
        let service = KanbanService::new("user-1");
        let board = service
            .create_board("project", "Board".to_string(), None)
            .unwrap();
        let col = service
            .add_column(&board.id, "Col".to_string(), None)
            .unwrap();

        let card1 = service
            .create_card(&board.id, &col.id, "Task 1".to_string(), None)
            .unwrap();
        service
            .create_card(&board.id, &col.id, "Task 2".to_string(), None)
            .unwrap();

        service
            .change_card_state(&board.id, &card1.id, CardState::Closed)
            .unwrap();

        let filter = CardFilter::new().with_states(vec![CardState::Open]);
        let cards = service.filter_cards(&board.id, filter).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Task 2");

        let filter = CardFilter::new().with_search("Task 1".to_string());
        let cards = service.filter_cards(&board.id, filter).unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Task 1");
    }

    // ===== LRU Cache Tests =====

    #[test]
    fn test_document_count() {
        let service = KanbanService::new("user-1");
        assert_eq!(service.document_count(), 0);

        service
            .create_board("project", "Board 1".to_string(), None)
            .unwrap();
        assert_eq!(service.document_count(), 1);

        service
            .create_board("project", "Board 2".to_string(), None)
            .unwrap();
        assert_eq!(service.document_count(), 2);
    }

    #[test]
    fn test_document_capacity() {
        let service = KanbanService::new("user-1");
        assert_eq!(service.document_capacity(), MAX_BOARD_DOCUMENTS);
    }

    #[test]
    fn test_lru_eviction() {
        // Create a service with very small capacity for testing
        // We'll test the eviction logic by creating many boards
        let service = KanbanService::new("user-1");

        // Create MAX_BOARD_DOCUMENTS + 1 boards to trigger eviction
        let mut board_ids = Vec::new();
        for i in 0..=MAX_BOARD_DOCUMENTS {
            let board = service
                .create_board("project", format!("Board {}", i), None)
                .unwrap();
            board_ids.push(board.id);
        }

        // Should have exactly MAX_BOARD_DOCUMENTS in cache (one was evicted)
        assert_eq!(service.document_count(), MAX_BOARD_DOCUMENTS);

        // The first board (board_ids[0]) should have been evicted
        // Trying to get it should fail
        let result = service.get_board(&board_ids[0]);
        assert!(result.is_err());

        // The last board should still exist
        let result = service.get_board(&board_ids[MAX_BOARD_DOCUMENTS]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lru_access_updates_recency() {
        let service = KanbanService::new("user-1");

        // Create boards up to capacity
        let mut board_ids = Vec::new();
        for i in 0..MAX_BOARD_DOCUMENTS {
            let board = service
                .create_board("project", format!("Board {}", i), None)
                .unwrap();
            board_ids.push(board.id);
        }

        // Access the first board to make it "recently used"
        service.get_board(&board_ids[0]).unwrap();

        // Create one more board - should evict board_ids[1], not board_ids[0]
        let new_board = service
            .create_board("project", "New Board".to_string(), None)
            .unwrap();

        // board_ids[0] should still exist (was accessed recently)
        assert!(service.get_board(&board_ids[0]).is_ok());

        // board_ids[1] should have been evicted
        assert!(service.get_board(&board_ids[1]).is_err());

        // The new board should exist
        assert!(service.get_board(&new_board.id).is_ok());
    }

    #[test]
    fn test_eviction_cleans_up_subscriptions() {
        let service = KanbanService::new("user-1");

        // Create a board and subscribe to it
        let board = service
            .create_board("project", "Test Board".to_string(), None)
            .unwrap();
        let _rx = service.subscribe_to_changes(&board.id).unwrap();

        // Verify subscription exists
        assert!(service.has_subscription(&board.id));

        // Create enough boards to evict the first one
        for i in 0..MAX_BOARD_DOCUMENTS {
            service
                .create_board("project", format!("Board {}", i), None)
                .unwrap();
        }

        // The original board should be evicted and its subscription cleaned up
        assert!(!service.has_subscription(&board.id));
    }
}

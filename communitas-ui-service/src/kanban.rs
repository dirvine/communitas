// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kanban service for board and card operations with reactive subscriptions.

use std::sync::{Arc, Weak};

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event, Query, QueryError, QueryResponse, Subscription};
use communitas_kanban::{
    BoardUpdate as CoreBoardUpdate, CardState as CoreCardState, CardUpdate as CoreCardUpdate,
    KanbanEvent, KanbanService as CoreKanbanService, Priority as CorePriority,
};

// Re-export analytics types for UI components
pub use communitas_kanban::{
    BoardAnalytics, BurndownChart, BurndownDataPoint, CycleTimeBucket, CycleTimeDistribution,
    TimeRange, VelocityDataPoint, VelocityMetric,
};
use communitas_ui_api::{
    ActivityEntry, BoardSettings, BoardSummary, BoardView, CardDetail, CardState, CardView,
    ChecklistProgress, ColumnView, CommentView, PriorityView, StepView, SwimlaneMode, SyncState,
    TagView,
};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{RwLock, broadcast, mpsc, watch};
use tracing::{debug, info, instrument, trace, warn};

/// Direction for keyboard-based card movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveDirection {
    /// Move card to previous column (Ctrl+Left).
    Left,
    /// Move card to next column (Ctrl+Right).
    Right,
    /// Move card up within current column (Ctrl+Up).
    Up,
    /// Move card down within current column (Ctrl+Down).
    Down,
}

/// Result of a keyboard card move operation.
#[derive(Debug, Clone)]
pub struct CardMoveResult {
    /// The ID of the card that was moved.
    pub card_id: String,
    /// The title of the card that was moved.
    pub card_title: String,
    /// The ID of the column the card moved to.
    pub target_column_id: String,
    /// The name of the column the card moved to.
    pub target_column_name: String,
    /// New position within the column.
    pub new_position: u32,
}

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::directory::DirectoryService;

/// Errors returned by the kanban service.
#[derive(Debug, Error)]
pub enum KanbanError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("board not found: {0}")]
    BoardNotFound(String),
    #[error("card not found: {0}")]
    CardNotFound(String),
    #[error("column not found: {0}")]
    ColumnNotFound(String),
    #[error("step not found: {0}")]
    StepNotFound(String),
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
    #[error("storage error: {0}")]
    StorageError(String),
}

impl From<communitas_kanban::KanbanError> for KanbanError {
    fn from(err: communitas_kanban::KanbanError) -> Self {
        match err {
            communitas_kanban::KanbanError::BoardNotFound(id) => KanbanError::BoardNotFound(id),
            communitas_kanban::KanbanError::CardNotFound(id) => KanbanError::CardNotFound(id),
            communitas_kanban::KanbanError::ColumnNotFound(id) => KanbanError::ColumnNotFound(id),
            communitas_kanban::KanbanError::StepNotFound(id) => KanbanError::StepNotFound(id),
            other => KanbanError::StorageError(other.to_string()),
        }
    }
}

/// Snapshot of kanban state for reactive UI updates.
#[derive(Debug, Clone, Default)]
pub struct KanbanSnapshot {
    /// All boards for the current entity.
    pub boards: Vec<BoardSummary>,
    /// Whether boards are currently being loaded.
    pub loading: bool,
    /// Current swimlane view mode.
    pub swimlane_mode: SwimlaneMode,
    /// Active conflicts from concurrent edits.
    pub conflicts: Vec<ConflictInfo>,
}

/// Information about a detected CRDT conflict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictInfo {
    /// Unique ID for this conflict instance.
    pub id: String,
    /// ID of the board containing the conflict.
    pub board_id: String,
    /// ID of the card involved in the conflict.
    pub card_id: String,
    /// Title of the card for display.
    pub card_title: String,
    /// Description of what changed remotely.
    pub remote_change: String,
    /// When the conflict was detected (Unix timestamp ms).
    pub detected_at: i64,
}

/// Update payload for card modifications.
#[derive(Debug, Clone, Default)]
pub struct CardUpdate {
    /// New title for the card.
    pub title: Option<String>,
    /// New description for the card.
    pub description: Option<String>,
    /// New assignees (replaces existing).
    pub assignees: Option<Vec<String>>,
    /// New tags (replaces existing).
    pub tags: Option<Vec<String>>,
    /// New due date (None to clear, Some(None) to explicitly clear).
    pub due_date: Option<Option<i64>>,
    /// New linked thread ID (None to clear, Some(None) to explicitly clear).
    pub linked_thread_id: Option<Option<String>>,
}

/// Service for kanban board and card operations.
///
/// The service automatically reinitializes its CRDT backend when the user authenticates,
/// ensuring operations are attributed to the correct peer_id. When the user logs out,
/// boards are cleared and the CRDT backend resets to anonymous.
///
/// Subscribes to CRDT events for the currently loaded entity, providing reactive
/// updates when boards, columns, or cards are modified (locally or remotely).
///
/// CRDT live updates are debounced (50ms) to batch rapid changes and reduce UI update frequency.
pub struct KanbanService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    /// Directory service for resolving author names.
    directory: Arc<DirectoryService>,
    /// CRDT backend wrapped in RwLock to allow reinitialization on auth changes.
    core: Arc<RwLock<CoreKanbanService>>,
    tx: watch::Sender<KanbanSnapshot>,
    rx: watch::Receiver<KanbanSnapshot>,
    /// Currently loaded entity ID (for event subscription).
    current_entity_id: Arc<RwLock<Option<String>>>,
    /// Currently subscribed board ID for CRDT live updates.
    current_board_id: Arc<RwLock<Option<String>>>,
}

impl KanbanService {
    /// Create a new kanban service linked to the auth controller.
    ///
    /// Spawns a background task that watches for authentication state changes and
    /// reinitializes the CRDT backend with the authenticated peer_id when the user logs in.
    /// When the user logs out, the backend resets to anonymous and boards are cleared.
    ///
    /// Also subscribes to kanban CRDT events for reactive updates when an entity is loaded.
    pub fn new(
        auth: Arc<AuthController>,
        app: Arc<CommunitasApp>,
        directory: Arc<DirectoryService>,
    ) -> Self {
        let (tx, rx) = watch::channel(KanbanSnapshot::default());
        // Initialize core service with a default peer_id; will be updated on auth
        let core = Arc::new(RwLock::new(CoreKanbanService::new("anonymous")));
        let current_entity_id = Arc::new(RwLock::new(None));
        let current_board_id = Arc::new(RwLock::new(None));

        let service = Self {
            auth,
            app,
            directory,
            core,
            tx,
            rx,
            current_entity_id,
            current_board_id,
        };

        // Spawn auth state watcher using a weak reference to avoid reference cycles
        service.spawn_auth_watcher();

        service
    }

    /// Spawn a background task to watch authentication state changes.
    ///
    /// Uses a weak reference to self to avoid reference cycles. When auth state changes
    /// to Authenticated, reinitializes CoreKanbanService with the real peer_id.
    /// When auth state changes to LoggedOut, resets to anonymous and clears boards.
    fn spawn_auth_watcher(&self) {
        let mut auth_rx = self.auth.subscribe();
        let core = Arc::downgrade(&self.core);
        let tx = self.tx.clone();

        tokio::spawn(async move {
            // Process the initial state (clone to avoid holding borrow across await)
            let initial_state = auth_rx.borrow().clone();
            Self::handle_auth_change(&initial_state, &core, &tx).await;

            // Watch for changes
            loop {
                if auth_rx.changed().await.is_err() {
                    debug!("Auth channel closed, stopping kanban auth watcher");
                    break;
                }

                let state = auth_rx.borrow().clone();
                Self::handle_auth_change(&state, &core, &tx).await;
            }
        });
    }

    /// Handle an authentication state change.
    ///
    /// When transitioning to Authenticated, reinitializes the CRDT backend with the
    /// user's four_words as the peer_id. When transitioning to LoggedOut, resets
    /// to anonymous and clears the board list.
    async fn handle_auth_change(
        state: &AuthStateSnapshot,
        core_weak: &Weak<RwLock<CoreKanbanService>>,
        tx: &watch::Sender<KanbanSnapshot>,
    ) {
        let Some(core) = core_weak.upgrade() else {
            trace!("KanbanService dropped, stopping auth watcher");
            return;
        };

        match state {
            AuthStateSnapshot::Authenticated { session, .. } => {
                let peer_id = &session.four_words;
                info!(peer_id = %peer_id, "Reinitializing CoreKanbanService with authenticated peer_id");

                let mut core_guard = core.write().await;
                *core_guard = CoreKanbanService::new(peer_id);
                drop(core_guard);

                // Note: We don't clear boards here - the UI will call list_boards
                // when it needs to load boards for an entity
                trace!("CoreKanbanService reinitialized for authenticated user");
            }
            AuthStateSnapshot::LoggedOut => {
                info!("User logged out, resetting CoreKanbanService to anonymous");

                let mut core_guard = core.write().await;
                *core_guard = CoreKanbanService::new("anonymous");
                drop(core_guard);

                // Clear the board list on logout
                if tx.send(KanbanSnapshot::default()).is_err() {
                    warn!("Failed to send logout snapshot: no active subscribers");
                }
                trace!("Boards cleared on logout");
            }
            AuthStateSnapshot::Authenticating => {
                // Transitional state, nothing to do
                trace!("Auth state is Authenticating, waiting for completion");
            }
        }
    }

    /// Subscribe to CRDT events for an entity.
    ///
    /// Spawns a background task to process kanban events and update the watch channel
    /// reactively when boards, columns, or cards are modified.
    fn subscribe_to_entity_events(&self, entity_id: String) {
        let event_rx = self.app.subscribe(Subscription::KanbanEvents {
            entity_id: entity_id.clone(),
        });

        let tx = self.tx.clone();
        let app = Arc::clone(&self.app);
        let auth = Arc::clone(&self.auth);
        let entity_id_for_loop = entity_id;

        tokio::spawn(async move {
            Self::kanban_event_loop(event_rx, tx, app, auth, entity_id_for_loop).await;
        });
    }

    /// Subscribe to CRDT live updates for a specific board.
    ///
    /// This method sets up a subscription to the board's Yrs document using
    /// `observe_after_transaction`. Events are debounced (50ms) to batch rapid
    /// changes and reduce UI update frequency.
    ///
    /// When the board changes (locally or via remote sync), a `BoardChanged` event
    /// is emitted, triggering a board refresh in the UI.
    ///
    /// # Arguments
    ///
    /// * `board_id` - ID of the board to subscribe to
    ///
    /// Calling this method with a different board_id will unsubscribe from the
    /// previous board and subscribe to the new one.
    async fn subscribe_to_board_crdt(&self, board_id: &str) {
        // Check if we're already subscribed to this board
        {
            let current = self.current_board_id.read().await;
            if current.as_deref() == Some(board_id) {
                trace!(board_id = %board_id, "Already subscribed to CRDT updates");
                return;
            }
        }

        // Unsubscribe from previous board
        {
            let mut current = self.current_board_id.write().await;
            if let Some(old_board_id) = current.take() {
                let core = self.core.read().await;
                core.unsubscribe(&old_board_id);
                debug!(board_id = %old_board_id, "Unsubscribed from previous board CRDT");
            }
            *current = Some(board_id.to_string());
        }

        // Subscribe to CRDT changes for the new board
        let core = self.core.read().await;
        let event_rx = match core.subscribe_to_changes(board_id) {
            Ok(rx) => rx,
            Err(e) => {
                warn!(board_id = %board_id, error = %e, "Failed to subscribe to CRDT changes");
                return;
            }
        };
        drop(core);

        debug!(board_id = %board_id, "Subscribed to CRDT live updates");

        // Spawn debounced event processor
        let tx = self.tx.clone();
        let app = Arc::clone(&self.app);
        let auth = Arc::clone(&self.auth);
        let core_weak = Arc::downgrade(&self.core);
        let board_id_owned = board_id.to_string();

        tokio::spawn(async move {
            Self::crdt_event_loop(event_rx, tx, app, auth, core_weak, board_id_owned).await;
        });
    }

    /// Background event loop that processes CRDT events with debouncing.
    ///
    /// Events are batched within a 50ms window to reduce UI update frequency
    /// during rapid changes (e.g., drag-drop repositioning).
    async fn crdt_event_loop(
        mut event_rx: mpsc::Receiver<KanbanEvent>,
        tx: watch::Sender<KanbanSnapshot>,
        app: Arc<CommunitasApp>,
        auth: Arc<AuthController>,
        core_weak: std::sync::Weak<RwLock<CoreKanbanService>>,
        board_id: String,
    ) {
        const DEBOUNCE_MS: u64 = 50;
        let mut pending_refresh = false;
        let debounce_duration = Duration::from_millis(DEBOUNCE_MS);

        loop {
            // Wait for an event or timeout (for debounce)
            let event = if pending_refresh {
                // Use timeout to debounce
                match tokio::time::timeout(debounce_duration, event_rx.recv()).await {
                    Ok(Some(event)) => {
                        // Got another event within debounce window, continue batching
                        trace!(board_id = %board_id, ?event, "Batching CRDT event");
                        continue;
                    }
                    Ok(None) => {
                        // Channel closed
                        debug!(board_id = %board_id, "CRDT event channel closed");
                        break;
                    }
                    Err(_) => {
                        // Timeout - process the batched events
                        pending_refresh = false;
                        None
                    }
                }
            } else {
                // Wait for first event
                match event_rx.recv().await {
                    Some(event) => Some(event),
                    None => {
                        debug!(board_id = %board_id, "CRDT event channel closed");
                        break;
                    }
                }
            };

            // If we got a new event, mark pending refresh and continue batching
            if event.is_some() {
                pending_refresh = true;
                trace!(board_id = %board_id, "CRDT event received, starting debounce");
                continue;
            }

            // Debounce timeout fired - refresh the board
            let Some(core) = core_weak.upgrade() else {
                debug!(board_id = %board_id, "KanbanService dropped, stopping CRDT event loop");
                break;
            };

            // Get the current entity_id to refresh boards
            let entity_id = {
                let core_guard = core.read().await;
                match core_guard.get_board(&board_id) {
                    Ok(board) => board.project_id,
                    Err(e) => {
                        warn!(board_id = %board_id, error = %e, "Failed to get board for refresh");
                        continue;
                    }
                }
            };

            debug!(
                board_id = %board_id,
                entity_id = %entity_id,
                "Refreshing board after CRDT changes (debounced)"
            );

            // Trigger board refresh
            Self::refresh_boards_internal(&tx, &app, &auth, &entity_id).await;
        }
    }

    /// Unsubscribe from CRDT updates for the current board.
    ///
    /// Called when closing a board view or switching to a different board.
    pub async fn unsubscribe_from_board_crdt(&self) {
        let mut current = self.current_board_id.write().await;
        if let Some(board_id) = current.take() {
            let core = self.core.read().await;
            core.unsubscribe(&board_id);
            debug!(board_id = %board_id, "Unsubscribed from CRDT updates");
        }
    }

    /// Background event loop that processes kanban events and updates the watch channel.
    ///
    /// This runs continuously, refreshing boards when relevant events occur.
    async fn kanban_event_loop(
        mut event_rx: broadcast::Receiver<Event>,
        tx: watch::Sender<KanbanSnapshot>,
        app: Arc<CommunitasApp>,
        auth: Arc<AuthController>,
        entity_id: String,
    ) {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    let should_refresh = matches!(
                        event,
                        Event::KanbanBoardCreated { .. }
                            | Event::KanbanColumnCreated { .. }
                            | Event::KanbanCardCreated { .. }
                            | Event::KanbanCardMoved { .. }
                            | Event::KanbanCardUpdated { .. }
                            | Event::KanbanCardDeleted { .. }
                            | Event::KanbanBoardUpdated { .. }
                            | Event::KanbanBoardDeleted { .. }
                    );

                    if should_refresh {
                        trace!(?event, "Kanban event received, refreshing boards");
                        Self::refresh_boards_internal(&tx, &app, &auth, &entity_id).await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We missed some events, refresh to catch up
                    debug!(
                        missed_events = n,
                        "Kanban event receiver lagged, refreshing boards"
                    );
                    Self::refresh_boards_internal(&tx, &app, &auth, &entity_id).await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // Channel closed, stop the loop
                    debug!("Kanban event channel closed, stopping event loop");
                    break;
                }
            }
        }
    }

    /// Internal helper to refresh boards and update the watch channel.
    ///
    /// Used by both the event loop and explicit refresh calls.
    async fn refresh_boards_internal(
        tx: &watch::Sender<KanbanSnapshot>,
        app: &Arc<CommunitasApp>,
        auth: &Arc<AuthController>,
        entity_id: &str,
    ) {
        let is_authenticated = matches!(
            &*auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated { .. }
        );

        if !is_authenticated {
            trace!("Not authenticated, skipping board refresh");
            return;
        }

        // Query boards from CommunitasApp
        let boards = match app
            .query(Query::ListKanbanBoards {
                entity_id: entity_id.to_string(),
            })
            .await
        {
            Ok(QueryResponse::KanbanBoardList(board_list)) => {
                debug!(
                    entity_id = %entity_id,
                    count = board_list.len(),
                    "Refreshed boards from CommunitasApp"
                );
                board_list
                    .into_iter()
                    .map(|b| BoardSummary {
                        id: b.id,
                        name: b.name,
                        entity_id: b.entity_id,
                        column_count: b.column_count as u32,
                        card_count: 0, // Card count requires additional query
                        last_activity: None,
                    })
                    .collect()
            }
            Ok(other) => {
                warn!(
                    entity_id = %entity_id,
                    response_type = ?std::mem::discriminant(&other),
                    "Unexpected response type while refreshing boards"
                );
                return;
            }
            Err(e) => {
                debug!(
                    entity_id = %entity_id,
                    error = %e,
                    "Failed to refresh boards from CommunitasApp"
                );
                return;
            }
        };

        // Update the watch channel
        let mut snap = tx.borrow().clone();
        snap.boards = boards;
        snap.loading = false;
        if tx.send(snap).is_err() {
            warn!("Failed to send refresh snapshot: no active subscribers");
        }
    }

    /// Get a reference to the underlying CommunitasApp.
    pub fn app(&self) -> Arc<CommunitasApp> {
        Arc::clone(&self.app)
    }

    /// Subscribe to kanban state updates.
    pub fn subscribe(&self) -> watch::Receiver<KanbanSnapshot> {
        self.rx.clone()
    }

    /// Get the current kanban snapshot without subscribing.
    pub fn current_snapshot(&self) -> KanbanSnapshot {
        self.rx.borrow().clone()
    }

    /// Set the swimlane view mode.
    ///
    /// Updates the snapshot with the new mode, triggering UI updates.
    pub fn set_swimlane_mode(&self, mode: SwimlaneMode) {
        let mut snap = self.rx.borrow().clone();
        snap.swimlane_mode = mode;
        self.send_snapshot(snap);
        debug!(swimlane_mode = ?mode, "Swimlane mode updated");
    }

    /// Add a conflict to the current snapshot.
    ///
    /// Called when CRDT merge detects concurrent edits to the same card.
    pub fn add_conflict(&self, conflict: ConflictInfo) {
        let mut snap = self.rx.borrow().clone();
        // Avoid duplicate conflicts for the same card
        if !snap.conflicts.iter().any(|c| c.card_id == conflict.card_id) {
            info!(
                card_id = %conflict.card_id,
                change = %conflict.remote_change,
                "Conflict detected"
            );
            snap.conflicts.push(conflict);
            self.send_snapshot(snap);
        }
    }

    /// Dismiss a specific conflict by ID.
    pub fn dismiss_conflict(&self, conflict_id: &str) {
        let mut snap = self.rx.borrow().clone();
        snap.conflicts.retain(|c| c.id != conflict_id);
        self.send_snapshot(snap);
        debug!(conflict_id = %conflict_id, "Conflict dismissed");
    }

    /// Dismiss all conflicts for a specific card.
    pub fn dismiss_card_conflicts(&self, card_id: &str) {
        let mut snap = self.rx.borrow().clone();
        snap.conflicts.retain(|c| c.id != card_id);
        self.send_snapshot(snap);
        debug!(card_id = %card_id, "Card conflicts dismissed");
    }

    /// Clear all conflicts.
    pub fn clear_conflicts(&self) {
        let mut snap = self.rx.borrow().clone();
        snap.conflicts.clear();
        self.send_snapshot(snap);
        debug!("All conflicts cleared");
    }

    /// Get active conflicts for a board.
    pub fn get_conflicts(&self, board_id: &str) -> Vec<ConflictInfo> {
        self.rx
            .borrow()
            .conflicts
            .iter()
            .filter(|c| c.board_id == board_id)
            .cloned()
            .collect()
    }
    /// Load boards for an entity, setting loading state and subscribing to events.
    ///
    /// This is the primary method for UI integration when an entity is selected.
    /// It immediately sets `loading=true` in the watch channel, then fetches boards
    /// and subscribes to CRDT events for reactive updates.
    ///
    /// # Example
    /// ```ignore
    /// // In UI code when entity is selected:
    /// kanban_service.load_boards("entity-123").await?;
    /// // Subscribe to snapshot updates to see loading state and boards
    /// let mut rx = kanban_service.subscribe();
    /// ```
    #[instrument(skip(self), name = "ui.kanban.load_boards", fields(entity_id))]
    pub async fn load_boards(&self, entity_id: &str) -> Result<Vec<BoardSummary>, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Set loading state immediately so UI can show loading indicator
        {
            let mut snap = self.rx.borrow().clone();
            snap.loading = true;
            snap.boards.clear(); // Clear previous entity's boards
            self.send_snapshot(snap);
        }

        debug!(entity_id = %entity_id, "Loading boards for entity");

        // list_boards handles the query, fallback, snapshot update, and event subscription
        self.list_boards(entity_id).await
    }

    /// List all boards for an entity (project/group).
    ///
    /// Queries CommunitasApp for network-synced board data with local CRDT fallback.
    /// Also subscribes to CRDT events for the entity if not already subscribed.
    #[instrument(skip(self), name = "ui.kanban.list_boards", fields(entity_id))]
    pub async fn list_boards(&self, entity_id: &str) -> Result<Vec<BoardSummary>, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Subscribe to events if this is a new entity
        {
            let mut current = self.current_entity_id.write().await;
            if current.as_deref() != Some(entity_id) {
                debug!(
                    entity_id = %entity_id,
                    previous = ?current.as_deref(),
                    "Subscribing to CRDT events for new entity"
                );
                self.subscribe_to_entity_events(entity_id.to_string());
                *current = Some(entity_id.to_string());
            }
        }

        // Try CommunitasApp query first for network-synced data
        let summaries = match self
            .app
            .query(Query::ListKanbanBoards {
                entity_id: entity_id.to_string(),
            })
            .await
        {
            Ok(QueryResponse::KanbanBoardList(boards)) => {
                debug!(
                    entity_id = %entity_id,
                    count = boards.len(),
                    "Fetched boards from CommunitasApp"
                );
                boards
                    .into_iter()
                    .map(|b| BoardSummary {
                        id: b.id,
                        name: b.name,
                        entity_id: b.entity_id,
                        column_count: b.column_count as u32,
                        card_count: 0, // Card count requires additional query
                        last_activity: None,
                    })
                    .collect()
            }
            Ok(other) => {
                warn!(
                    entity_id = %entity_id,
                    response_type = ?std::mem::discriminant(&other),
                    "Unexpected query response, falling back to local CRDT"
                );
                self.list_boards_from_crdt(entity_id).await?
            }
            Err(e) => {
                debug!(
                    entity_id = %entity_id,
                    error = %e,
                    "CommunitasApp query failed, falling back to local CRDT"
                );
                self.list_boards_from_crdt(entity_id).await?
            }
        };

        // Update snapshot
        let mut snap = self.rx.borrow().clone();
        snap.boards = summaries.clone();
        snap.loading = false;
        self.send_snapshot(snap);

        Ok(summaries)
    }

    /// List boards from local CRDT (fallback when CommunitasApp query fails).
    async fn list_boards_from_crdt(
        &self,
        entity_id: &str,
    ) -> Result<Vec<BoardSummary>, KanbanError> {
        let core = self.core.read().await;
        let boards = core.list_boards(entity_id)?;
        let summaries: Vec<BoardSummary> = boards
            .into_iter()
            .map(|b| {
                // Count columns and cards for summary
                let column_count = core
                    .list_columns(&b.id)
                    .map(|cols| cols.len() as u32)
                    .unwrap_or(0);
                let card_count = core
                    .list_columns(&b.id)
                    .map(|cols| {
                        cols.iter()
                            .filter_map(|c| {
                                core.list_cards_in_column(&b.id, &c.id)
                                    .ok()
                                    .map(|cards| cards.len() as u32)
                            })
                            .sum()
                    })
                    .unwrap_or(0);

                BoardSummary {
                    id: b.id,
                    name: b.name,
                    entity_id: b.project_id,
                    column_count,
                    card_count,
                    last_activity: Some(b.updated_at * 1000),
                }
            })
            .collect();

        Ok(summaries)
    }

    /// Get a full board view with columns and cards.
    ///
    /// Tries CommunitasApp queries first for network-synced data, falls back to local CRDT.
    /// Automatically subscribes to CRDT live updates for the board.
    #[instrument(skip(self), name = "ui.kanban.get_board", fields(board_id))]
    pub async fn get_board(&self, board_id: &str) -> Result<BoardView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Subscribe to CRDT live updates for real-time board changes
        self.subscribe_to_board_crdt(board_id).await;

        // Try to fetch board metadata from CommunitasApp for consistency verification
        let app_board = self
            .app
            .query(Query::GetKanbanBoard {
                board_id: board_id.to_string(),
            })
            .await;

        // Try to fetch columns from CommunitasApp
        let app_columns = self
            .app
            .query(Query::ListKanbanColumns {
                board_id: board_id.to_string(),
            })
            .await;

        // Try to fetch cards from CommunitasApp
        let app_cards = self
            .app
            .query(Query::ListKanbanCards {
                board_id: board_id.to_string(),
                column_id: None,
                state: None,
                assignee_id: None,
                tag_id: None,
            })
            .await;

        // Check if we have complete data from CommunitasApp
        let use_app_data = matches!(
            (&app_board, &app_columns, &app_cards),
            (
                Ok(QueryResponse::KanbanBoard(_)),
                Ok(QueryResponse::KanbanColumns(_)),
                Ok(QueryResponse::KanbanCards(_))
            )
        );

        if use_app_data {
            debug!(board_id = %board_id, "Using CommunitasApp data for board view");
            self.build_board_view_from_app(board_id, app_board, app_columns, app_cards)
                .await
        } else {
            debug!(board_id = %board_id, "Falling back to local CRDT for board view");
            self.get_board_from_crdt(board_id).await
        }
    }

    /// Build board view from CommunitasApp query responses.
    async fn build_board_view_from_app(
        &self,
        board_id: &str,
        app_board: Result<QueryResponse, QueryError>,
        app_columns: Result<QueryResponse, QueryError>,
        app_cards: Result<QueryResponse, QueryError>,
    ) -> Result<BoardView, KanbanError> {
        let board = match app_board {
            Ok(QueryResponse::KanbanBoard(b)) => b,
            _ => return self.get_board_from_crdt(board_id).await,
        };

        let columns = match app_columns {
            Ok(QueryResponse::KanbanColumns(cols)) => cols,
            _ => return self.get_board_from_crdt(board_id).await,
        };

        let cards = match app_cards {
            Ok(QueryResponse::KanbanCards(cards)) => cards,
            _ => return self.get_board_from_crdt(board_id).await,
        };

        // Group cards by column
        let mut cards_by_column: std::collections::HashMap<String, Vec<CardView>> =
            std::collections::HashMap::new();
        for card in cards {
            let card_view = CardView {
                id: card.id,
                title: card.title,
                description: card.description,
                state: CardState::default(),
                priority: None,
                assignees: card.assignee.into_iter().collect(),
                tags: Vec::new(),
                due_date: None,
                checklist_progress: None,
                position: card.position,
                linked_thread_id: None, // Thread linking via local CRDT only
                linked_thread_name: None,
                sync_state: SyncState::Synced, // Default to synced; updated by CRDT sync
            };
            cards_by_column
                .entry(card.column_id)
                .or_default()
                .push(card_view);
        }

        // Build column views
        let column_views: Vec<ColumnView> = columns
            .into_iter()
            .map(|col| {
                let mut cards = cards_by_column.remove(&col.id).unwrap_or_default();
                cards.sort_by_key(|c| c.position);
                ColumnView {
                    id: col.id,
                    name: col.name,
                    position: col.position,
                    cards,
                    wip_limit: col.wip_limit,
                }
            })
            .collect();

        Ok(BoardView {
            id: board.id,
            name: board.name,
            entity_id: board.entity_id,
            description: board.description,
            columns: column_views,
            settings: BoardSettings::default(),
            created_at: 0,
            updated_at: 0,
        })
    }

    /// Get board view from local CRDT (fallback).
    async fn get_board_from_crdt(&self, board_id: &str) -> Result<BoardView, KanbanError> {
        let core = self.core.read().await;
        let board = core.get_board(board_id)?;
        let columns = core.list_columns(board_id)?;
        let tags = core.list_tags(board_id)?;

        // Build tag lookup
        let tag_lookup: std::collections::HashMap<String, TagView> = tags
            .into_iter()
            .map(|t| {
                (
                    t.id.clone(),
                    TagView {
                        id: t.id,
                        name: t.name,
                        color: t.color,
                    },
                )
            })
            .collect();

        let column_views: Vec<ColumnView> = columns
            .into_iter()
            .map(|col| {
                let cards = core
                    .list_cards_in_column(board_id, &col.id)
                    .unwrap_or_default();
                let card_views: Vec<CardView> = cards
                    .into_iter()
                    .map(|card| self.card_to_view(&card, &tag_lookup))
                    .collect();

                ColumnView {
                    id: col.id,
                    name: col.name,
                    position: col.position,
                    cards: card_views,
                    wip_limit: col.wip_limit,
                }
            })
            .collect();

        Ok(BoardView {
            id: board.id,
            name: board.name,
            entity_id: board.project_id,
            description: board.description,
            columns: column_views,
            settings: self.settings_to_view(&board.settings),
            created_at: board.created_at * 1000,
            updated_at: board.updated_at * 1000,
        })
    }

    /// Get detailed card information including steps, comments, and activity.
    ///
    /// Queries CommunitasApp for card verification and uses local CRDT for rich data
    /// (comments, tags, steps) which CommunitasApp query doesn't provide.
    #[instrument(skip(self), name = "ui.kanban.get_card", fields(board_id, card_id))]
    pub async fn get_card(&self, board_id: &str, card_id: &str) -> Result<CardDetail, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Query CommunitasApp for card verification
        let app_result = self
            .app
            .query(Query::GetKanbanCard {
                board_id: board_id.to_string(),
                card_id: card_id.to_string(),
            })
            .await;

        match &app_result {
            Ok(QueryResponse::KanbanCard(_)) => {
                debug!(card_id = %card_id, "Card verified via CommunitasApp");
            }
            Ok(_) => {
                warn!(card_id = %card_id, "Unexpected response type from CommunitasApp");
            }
            Err(e) => {
                debug!(card_id = %card_id, error = %e, "CommunitasApp card query failed, using local CRDT");
            }
        }

        // Get card from CRDT for complete data (comments, tags, steps, etc.)
        let core = self.core.read().await;
        let card = core.get_card(board_id, card_id)?;
        let tags = core.list_tags(board_id)?;
        let comments = core.list_comments(board_id, card_id)?;
        let core_steps = core.list_steps(board_id, card_id)?;
        drop(core);

        // Build tag lookup
        let tag_lookup: std::collections::HashMap<String, TagView> = tags
            .into_iter()
            .map(|t| {
                (
                    t.id.clone(),
                    TagView {
                        id: t.id,
                        name: t.name,
                        color: t.color,
                    },
                )
            })
            .collect();

        // Convert core steps to StepView and calculate checklist progress
        let steps: Vec<StepView> = core_steps
            .iter()
            .map(|s| StepView {
                id: s.id.clone(),
                title: s.text.clone(),
                completed: s.completed,
            })
            .collect();

        let step_count = steps.len() as u32;
        let completed_count = core_steps.iter().filter(|s| s.completed).count() as u32;

        let checklist_progress = if step_count > 0 {
            Some(ChecklistProgress {
                completed: completed_count,
                total: step_count,
            })
        } else {
            None
        };

        let tag_views: Vec<TagView> = card
            .tag_ids
            .iter()
            .filter_map(|id| tag_lookup.get(id).cloned())
            .collect();

        // Build activity log from available data (card creation + comments)
        // NOTE: Build activity BEFORE consuming comments with into_iter()
        let mut activity: Vec<ActivityEntry> = Vec::new();

        // Card creation event
        let creator_name = self.resolve_author_name(&card.created_by).await;
        activity.push(ActivityEntry {
            id: format!("activity-created-{}", card.id),
            action_type: "created".to_string(),
            actor_id: card.created_by.clone(),
            actor_name: creator_name,
            description: format!("Created card \"{}\"", card.title),
            timestamp: card.created_at * 1000,
        });

        // Comment events (iterate before consuming for comment_views)
        for comment in &comments {
            let commenter_name = self.resolve_author_name(&comment.author_id).await;
            activity.push(ActivityEntry {
                id: format!("activity-comment-{}", comment.id),
                action_type: "commented".to_string(),
                actor_id: comment.author_id.clone(),
                actor_name: commenter_name,
                description: "Added a comment".to_string(),
                timestamp: comment.created_at * 1000,
            });
        }

        // Sort by timestamp (most recent first)
        activity.sort_by_key(|item| std::cmp::Reverse(item.timestamp));

        // Build comment views (resolve names via loop since map can't be async)
        let mut comment_views: Vec<CommentView> = Vec::with_capacity(comments.len());
        for c in comments {
            let author_name = self.resolve_author_name(&c.author_id).await;
            comment_views.push(CommentView {
                id: c.id,
                author_id: c.author_id,
                author_name,
                text: c.content,
                created_at: c.created_at * 1000,
            });
        }

        Ok(CardDetail {
            id: card.id.clone(),
            title: card.title.clone(),
            description: if card.description.is_empty() {
                None
            } else {
                Some(card.description)
            },
            state: self.core_state_to_ui(card.state),
            priority: self.core_priority_to_ui(card.priority),
            assignees: card.assignee_ids,
            tags: tag_views,
            due_date: card.due_date.map(|d| d * 1000),
            checklist_progress,
            position: card.position,
            steps,
            comments: comment_views,
            attachments: Vec::new(), // TODO: Wire when attachments are available
            activity,
            linked_thread_id: card.linked_thread_id.clone(),
            linked_thread_name: None, // Thread name resolved via messaging service
            sync_state: SyncState::Synced, // CRDT cards are considered synced
        })
    }

    /// Create a new board.
    ///
    /// Creates the board both locally via CRDT and persists via CommunitasApp command.
    /// This ensures the board is available immediately in the local state and will be
    /// synchronized across the network via CRDT.
    #[instrument(skip(self), name = "ui.kanban.create_board", fields(entity_id, name))]
    pub async fn create_board(
        &self,
        entity_id: &str,
        name: &str,
        template: Option<&str>,
    ) -> Result<BoardSummary, KanbanError> {
        let _ = template; // Template support is a future enhancement
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Create board locally via CRDT for immediate availability
        let core = self.core.read().await;
        let board = core.create_board(entity_id, name.to_string(), None)?;
        let board_id = board.id.clone();
        let board_name = board.name.clone();
        let project_id = board.project_id.clone();
        let created_at = board.created_at;
        drop(core);

        // Execute command for persistence via CommunitasApp
        // This ensures the board is stored and synced across the network
        let command = Command::CreateKanbanBoard {
            entity_id: entity_id.to_string(),
            board_name: name.to_string(),
            description: None,
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                board_id = %board_id,
                error = %e,
                "Failed to persist board via CommunitasApp, board exists locally only"
            );
            // Continue - board was created locally, persistence failure is not fatal
            // The board will be available in the local CRDT state
        } else {
            debug!(board_id = %board_id, "Board created and persisted via CommunitasApp");
        }

        let summary = BoardSummary {
            id: board_id,
            name: board_name,
            entity_id: project_id,
            column_count: 0,
            card_count: 0,
            last_activity: Some(created_at * 1000),
        };

        // Update watch channel with new board
        let mut snap = self.rx.borrow().clone();
        snap.boards.push(summary.clone());
        snap.loading = false;
        self.send_snapshot(snap);

        info!(board_id = %summary.id, "Board created successfully");
        Ok(summary)
    }

    /// Create a new column in a board.
    ///
    /// Creates the column both locally via CRDT and persists via CommunitasApp command.
    #[instrument(
        skip(self),
        name = "ui.kanban.create_column",
        fields(board_id, name, position)
    )]
    pub async fn create_column(
        &self,
        board_id: &str,
        name: &str,
        position: u32,
    ) -> Result<ColumnView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Create column locally via CRDT for immediate availability
        let core = self.core.read().await;
        let col = core.add_column(board_id, name.to_string(), Some(position))?;
        let column_id = col.id.clone();
        let column_name = col.name.clone();
        let column_position = col.position;
        let wip_limit = col.wip_limit;
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::CreateKanbanColumn {
            board_id: board_id.to_string(),
            column_name: name.to_string(),
            position: Some(position),
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                column_id = %column_id,
                error = %e,
                "Failed to persist column via CommunitasApp, column exists locally only"
            );
        } else {
            debug!(column_id = %column_id, "Column created and persisted via CommunitasApp");
        }

        Ok(ColumnView {
            id: column_id,
            name: column_name,
            position: column_position,
            cards: Vec::new(),
            wip_limit,
        })
    }

    /// Create a new card in a column.
    ///
    /// Creates the card both locally via CRDT and persists via CommunitasApp command.
    #[instrument(
        skip(self),
        name = "ui.kanban.create_card",
        fields(board_id, column_id, title)
    )]
    pub async fn create_card(
        &self,
        board_id: &str,
        column_id: &str,
        title: &str,
    ) -> Result<CardView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Create card locally via CRDT for immediate availability
        let core = self.core.read().await;
        let card = core.create_card(board_id, column_id, title.to_string(), None)?;
        let card_id = card.id.clone();
        let card_title = card.title.clone();
        let card_description = card.description.clone();
        let card_state = card.state;
        let card_assignees = card.assignee_ids.clone();
        let card_due_date = card.due_date;
        let card_position = card.position;
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::CreateKanbanCard {
            board_id: board_id.to_string(),
            column_id: column_id.to_string(),
            title: title.to_string(),
            description: None,
            assignee: None,
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                card_id = %card_id,
                error = %e,
                "Failed to persist card via CommunitasApp, card exists locally only"
            );
        } else {
            debug!(card_id = %card_id, "Card created and persisted via CommunitasApp");
        }

        Ok(CardView {
            id: card_id,
            title: card_title,
            description: if card_description.is_empty() {
                None
            } else {
                Some(card_description)
            },
            state: self.core_state_to_ui(card_state),
            priority: None, // New cards start with no priority
            assignees: card_assignees,
            tags: Vec::new(),
            due_date: card_due_date.map(|d| d * 1000),
            checklist_progress: None,
            position: card_position,
            linked_thread_id: None, // New cards have no linked thread
            linked_thread_name: None,
            sync_state: SyncState::Queued, // New cards start queued for sync
        })
    }

    /// Move a card to a different column or position.
    ///
    /// Moves the card both locally via CRDT and persists via CommunitasApp command.
    #[instrument(
        skip(self),
        name = "ui.kanban.move_card",
        fields(board_id, card_id, target_column, position)
    )]
    pub async fn move_card(
        &self,
        board_id: &str,
        card_id: &str,
        target_column: &str,
        position: u32,
    ) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Move card locally via CRDT for immediate availability
        let core = self.core.read().await;
        core.move_card(board_id, card_id, target_column, position)?;
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::MoveKanbanCard {
            board_id: board_id.to_string(),
            card_id: card_id.to_string(),
            target_column_id: target_column.to_string(),
            position: Some(position),
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                card_id = %card_id,
                target_column = %target_column,
                error = %e,
                "Failed to persist card move via CommunitasApp, move exists locally only"
            );
        } else {
            debug!(card_id = %card_id, target_column = %target_column, "Card move persisted via CommunitasApp");
        }

        Ok(())
    }

    /// Move a card using keyboard shortcuts (Ctrl+Arrow).
    ///
    /// Returns information about the move for aria-live announcements.
    #[instrument(
        skip(self),
        name = "ui.kanban.move_card_keyboard",
        fields(board_id, card_id, direction)
    )]
    pub async fn move_card_keyboard(
        &self,
        board_id: &str,
        card_id: &str,
        current_column_id: &str,
        current_position: u32,
        direction: MoveDirection,
    ) -> Result<CardMoveResult, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;

        // Get board data for column list
        let columns = core.list_columns(board_id)?;
        if columns.is_empty() {
            return Err(KanbanError::InvalidOperation(
                "board has no columns".to_string(),
            ));
        }

        // Find current column index
        let current_col_idx = columns
            .iter()
            .position(|c| c.id == current_column_id)
            .ok_or_else(|| KanbanError::ColumnNotFound(current_column_id.to_string()))?;

        // Get the card for its title
        let card = core.get_card(board_id, card_id)?;

        // Calculate target column and position based on direction
        let (target_column_id, target_column_name, new_position) = match direction {
            MoveDirection::Left => {
                // Move to previous column (or stay if at first column)
                if current_col_idx == 0 {
                    return Err(KanbanError::InvalidOperation(
                        "card is already in first column".to_string(),
                    ));
                }
                let target_col = &columns[current_col_idx - 1];
                let cards_in_target = core
                    .list_cards_in_column(board_id, &target_col.id)
                    .unwrap_or_default();
                // Insert at end of target column
                (
                    target_col.id.clone(),
                    target_col.name.clone(),
                    cards_in_target.len() as u32,
                )
            }
            MoveDirection::Right => {
                // Move to next column (or stay if at last column)
                if current_col_idx >= columns.len() - 1 {
                    return Err(KanbanError::InvalidOperation(
                        "card is already in last column".to_string(),
                    ));
                }
                let target_col = &columns[current_col_idx + 1];
                let cards_in_target = core
                    .list_cards_in_column(board_id, &target_col.id)
                    .unwrap_or_default();
                // Insert at end of target column
                (
                    target_col.id.clone(),
                    target_col.name.clone(),
                    cards_in_target.len() as u32,
                )
            }
            MoveDirection::Up => {
                // Move up within current column
                if current_position == 0 {
                    return Err(KanbanError::InvalidOperation(
                        "card is already at top of column".to_string(),
                    ));
                }
                let current_col = &columns[current_col_idx];
                (
                    current_col.id.clone(),
                    current_col.name.clone(),
                    current_position - 1,
                )
            }
            MoveDirection::Down => {
                // Move down within current column
                let current_col = &columns[current_col_idx];
                let cards_in_column = core
                    .list_cards_in_column(board_id, &current_col.id)
                    .unwrap_or_default();
                let max_pos = cards_in_column.len().saturating_sub(1) as u32;
                if current_position >= max_pos {
                    return Err(KanbanError::InvalidOperation(
                        "card is already at bottom of column".to_string(),
                    ));
                }
                (
                    current_col.id.clone(),
                    current_col.name.clone(),
                    current_position + 1,
                )
            }
        };

        // Perform the move locally via CRDT
        core.move_card(board_id, card_id, &target_column_id, new_position)?;
        let card_title = card.title.clone();
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::MoveKanbanCard {
            board_id: board_id.to_string(),
            card_id: card_id.to_string(),
            target_column_id: target_column_id.clone(),
            position: Some(new_position),
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                card_id = %card_id,
                target_column = %target_column_id,
                error = %e,
                "Failed to persist keyboard card move via CommunitasApp, move exists locally only"
            );
        } else {
            debug!(card_id = %card_id, target_column = %target_column_id, "Keyboard card move persisted via CommunitasApp");
        }

        Ok(CardMoveResult {
            card_id: card_id.to_string(),
            card_title,
            target_column_id,
            target_column_name,
            new_position,
        })
    }

    /// Update a card's properties.
    ///
    /// Updates the card both locally via CRDT and persists via CommunitasApp command.
    #[instrument(
        skip(self, updates),
        name = "ui.kanban.update_card",
        fields(board_id, card_id)
    )]
    pub async fn update_card(
        &self,
        board_id: &str,
        card_id: &str,
        updates: CardUpdate,
    ) -> Result<CardView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Capture updates for CommunitasApp command before consuming
        let cmd_title = updates.title.clone();
        let cmd_description = updates.description.clone();
        let cmd_assignee = updates.assignees.as_ref().and_then(|a| a.first().cloned());

        let core = self.core.read().await;

        // Apply basic updates locally via CRDT
        let core_update = CoreCardUpdate {
            title: updates.title,
            description: updates.description,
            due_date: updates.due_date,
            linked_thread_id: updates.linked_thread_id,
            ..Default::default()
        };
        let card = core.update_card(board_id, card_id, core_update)?;

        // Handle assignee changes if specified
        if let Some(new_assignees) = updates.assignees {
            // Remove existing assignees
            for assignee in &card.assignee_ids {
                let _ = core.unassign_user(board_id, card_id, assignee);
            }
            // Add new assignees
            for assignee in &new_assignees {
                let _ = core.assign_user(board_id, card_id, assignee);
            }
        }

        // Handle tag changes if specified
        if let Some(new_tags) = updates.tags {
            // Remove existing tags
            for tag_id in &card.tag_ids {
                let _ = core.untag_card(board_id, card_id, tag_id);
            }
            // Add new tags
            for tag_id in &new_tags {
                let _ = core.tag_card(board_id, card_id, tag_id);
            }
        }

        // Fetch updated card
        let updated = core.get_card(board_id, card_id)?;
        let tags = core.list_tags(board_id)?;
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::UpdateKanbanCard {
            board_id: board_id.to_string(),
            card_id: card_id.to_string(),
            title: cmd_title,
            description: cmd_description,
            assignee: cmd_assignee,
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                card_id = %card_id,
                error = %e,
                "Failed to persist card update via CommunitasApp, update exists locally only"
            );
        } else {
            debug!(card_id = %card_id, "Card update persisted via CommunitasApp");
        }

        let tag_lookup: std::collections::HashMap<String, TagView> = tags
            .into_iter()
            .map(|t| {
                (
                    t.id.clone(),
                    TagView {
                        id: t.id,
                        name: t.name,
                        color: t.color,
                    },
                )
            })
            .collect();

        Ok(self.card_to_view(&updated, &tag_lookup))
    }

    /// Archive a card.
    ///
    /// Archives the card both locally via CRDT and persists via CommunitasApp command.
    #[instrument(skip(self), name = "ui.kanban.archive_card", fields(board_id, card_id))]
    pub async fn archive_card(&self, board_id: &str, card_id: &str) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Archive card locally via CRDT
        let core = self.core.read().await;
        core.change_card_state(board_id, card_id, CoreCardState::Archived)?;
        drop(core);

        // Execute delete command for persistence via CommunitasApp
        // (Archive is treated as delete for network persistence purposes)
        let command = Command::DeleteKanbanCard {
            board_id: board_id.to_string(),
            card_id: card_id.to_string(),
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                card_id = %card_id,
                error = %e,
                "Failed to persist card archive via CommunitasApp, archive exists locally only"
            );
        } else {
            debug!(card_id = %card_id, "Card archive persisted via CommunitasApp");
        }

        Ok(())
    }

    /// Update a board's properties.
    ///
    /// Updates the board both locally via CRDT and persists via CommunitasApp command.
    #[instrument(skip(self), name = "ui.kanban.update_board", fields(board_id))]
    pub async fn update_board(
        &self,
        board_id: &str,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Update board locally via CRDT
        let core = self.core.read().await;
        let board_update = CoreBoardUpdate {
            name: name.clone(),
            description: description.clone(),
            settings: None,
        };
        core.update_board(board_id, board_update)?;
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::UpdateKanbanBoard {
            board_id: board_id.to_string(),
            name,
            description,
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                board_id = %board_id,
                error = %e,
                "Failed to persist board update via CommunitasApp, update exists locally only"
            );
        } else {
            debug!(board_id = %board_id, "Board update persisted via CommunitasApp");
        }

        Ok(())
    }

    /// Delete a board.
    ///
    /// Deletes the board both locally via CRDT and persists via CommunitasApp command.
    #[instrument(skip(self), name = "ui.kanban.delete_board", fields(board_id))]
    pub async fn delete_board(&self, board_id: &str) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Delete board locally via CRDT
        let core = self.core.read().await;
        core.delete_board(board_id)?;
        drop(core);

        // Execute command for persistence via CommunitasApp
        let command = Command::DeleteKanbanBoard {
            board_id: board_id.to_string(),
        };

        if let Err(e) = self.app.execute(command).await {
            warn!(
                board_id = %board_id,
                error = %e,
                "Failed to persist board deletion via CommunitasApp, deletion exists locally only"
            );
        } else {
            debug!(board_id = %board_id, "Board deletion persisted via CommunitasApp");
        }

        // Update watch channel to remove the deleted board
        let mut snap = self.rx.borrow().clone();
        snap.boards.retain(|b| b.id != board_id);
        self.send_snapshot(snap);

        Ok(())
    }

    /// Add a checklist step to a card.
    #[instrument(
        skip(self),
        name = "ui.kanban.add_step",
        fields(board_id, card_id, title)
    )]
    pub async fn add_step(
        &self,
        board_id: &str,
        card_id: &str,
        title: &str,
    ) -> Result<StepView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let step = core.add_step(board_id, card_id, title.to_string(), None)?;

        Ok(StepView {
            id: step.id,
            title: step.text,
            completed: step.completed,
        })
    }

    /// Toggle a step's completion status.
    #[instrument(
        skip(self),
        name = "ui.kanban.toggle_step",
        fields(board_id, card_id, step_id)
    )]
    pub async fn toggle_step(
        &self,
        board_id: &str,
        card_id: &str,
        step_id: &str,
    ) -> Result<StepView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let step = core.toggle_step(board_id, card_id, step_id)?;

        Ok(StepView {
            id: step.id,
            title: step.text,
            completed: step.completed,
        })
    }

    /// Set the priority for a card.
    ///
    /// Updates the card's priority both locally via CRDT and persists via CommunitasApp command.
    #[instrument(
        skip(self),
        name = "ui.kanban.set_priority",
        fields(board_id, card_id, ?priority)
    )]
    pub async fn set_priority(
        &self,
        board_id: &str,
        card_id: &str,
        priority: Option<PriorityView>,
    ) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        // Convert PriorityView to CorePriority
        let core_priority = priority.map(|p| match p {
            PriorityView::Urgent => CorePriority::Urgent,
            PriorityView::High => CorePriority::High,
            PriorityView::Normal => CorePriority::Normal,
            PriorityView::Low => CorePriority::Low,
        });

        // Update priority locally via CRDT
        let core = self.core.read().await;
        let core_update = CoreCardUpdate {
            priority: Some(core_priority),
            ..Default::default()
        };
        core.update_card(board_id, card_id, core_update)?;
        drop(core);

        // Note: CommunitasApp doesn't currently have a dedicated priority command,
        // so we rely on CRDT sync. When network priority support is added,
        // we'll execute a command here.
        debug!(card_id = %card_id, ?priority, "Card priority updated via CRDT");

        Ok(())
    }

    /// Add a comment to a card.
    #[instrument(
        skip(self, text),
        name = "ui.kanban.add_comment",
        fields(board_id, card_id)
    )]
    pub async fn add_comment(
        &self,
        board_id: &str,
        card_id: &str,
        text: &str,
    ) -> Result<CommentView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let comment = core.add_comment(board_id, card_id, text.to_string(), None)?;
        drop(core);

        let author_name = self.resolve_author_name(&comment.author_id).await;
        Ok(CommentView {
            id: comment.id,
            author_id: comment.author_id,
            author_name,
            text: comment.content,
            created_at: comment.created_at * 1000,
        })
    }

    /// Link a message thread to a card.
    ///
    /// Associates a chat thread with a kanban card for discussions.
    #[instrument(
        skip(self),
        name = "ui.kanban.link_thread",
        fields(board_id, card_id, thread_id)
    )]
    pub async fn link_thread(
        &self,
        board_id: &str,
        card_id: &str,
        thread_id: &str,
    ) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let core_update = CoreCardUpdate {
            linked_thread_id: Some(Some(thread_id.to_string())),
            ..Default::default()
        };
        core.update_card(board_id, card_id, core_update)?;
        drop(core);

        info!(
            card_id = %card_id,
            thread_id = %thread_id,
            "Thread linked to card"
        );

        Ok(())
    }

    /// Unlink a message thread from a card.
    ///
    /// Removes the thread association from a kanban card.
    #[instrument(
        skip(self),
        name = "ui.kanban.unlink_thread",
        fields(board_id, card_id)
    )]
    pub async fn unlink_thread(&self, board_id: &str, card_id: &str) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let core_update = CoreCardUpdate {
            linked_thread_id: Some(None),
            ..Default::default()
        };
        core.update_card(board_id, card_id, core_update)?;
        drop(core);

        info!(card_id = %card_id, "Thread unlinked from card");

        Ok(())
    }

    // ===== Analytics Methods =====

    /// Get analytics summary for a board.
    ///
    /// Returns current metrics including card counts, WIP, overdue, and cycle time.
    #[instrument(skip(self), name = "ui.kanban.get_board_analytics", fields(board_id))]
    pub async fn get_board_analytics(
        &self,
        board_id: &str,
    ) -> Result<communitas_kanban::BoardAnalytics, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let analytics = core.get_board_analytics(board_id)?;
        drop(core);

        debug!(board_id = %board_id, "Retrieved board analytics");
        Ok(analytics)
    }

    /// Calculate velocity metrics for a board.
    ///
    /// Returns cards completed per period with average and standard deviation.
    #[instrument(skip(self), name = "ui.kanban.calculate_velocity", fields(board_id))]
    pub async fn calculate_velocity(
        &self,
        board_id: &str,
        time_range: communitas_kanban::TimeRange,
    ) -> Result<communitas_kanban::VelocityMetric, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let velocity = core.calculate_velocity(board_id, time_range)?;
        drop(core);

        debug!(board_id = %board_id, "Calculated velocity");
        Ok(velocity)
    }

    /// Calculate burndown chart data for a board.
    ///
    /// Returns ideal vs actual burndown lines for a given period.
    #[instrument(skip(self), name = "ui.kanban.calculate_burndown", fields(board_id))]
    pub async fn calculate_burndown(
        &self,
        board_id: &str,
        period_start: i64,
        period_end: i64,
    ) -> Result<communitas_kanban::BurndownChart, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let burndown = core.calculate_burndown(board_id, period_start, period_end)?;
        drop(core);

        debug!(board_id = %board_id, "Calculated burndown");
        Ok(burndown)
    }

    /// Calculate cycle time distribution for a board.
    ///
    /// Returns histogram showing how long cards typically take to complete.
    #[instrument(
        skip(self),
        name = "ui.kanban.calculate_cycle_time_distribution",
        fields(board_id)
    )]
    pub async fn calculate_cycle_time_distribution(
        &self,
        board_id: &str,
        time_range: communitas_kanban::TimeRange,
    ) -> Result<communitas_kanban::CycleTimeDistribution, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let distribution = core.calculate_cycle_time_distribution(board_id, time_range)?;
        drop(core);

        debug!(board_id = %board_id, "Calculated cycle time distribution");
        Ok(distribution)
    }

    /// Internal: set loading state.
    pub fn set_loading(&self, loading: bool) {
        let mut snap = self.rx.borrow().clone();
        snap.loading = loading;
        self.send_snapshot(snap);
    }

    /// Internal: update the board list (called by core events).
    pub fn set_boards(&self, boards: Vec<BoardSummary>) {
        let mut snap = self.rx.borrow().clone();
        snap.boards = boards;
        snap.loading = false;
        self.send_snapshot(snap);
    }

    // ===== Helper Methods =====

    /// Send a snapshot to subscribers, logging a warning if no subscribers exist.
    fn send_snapshot(&self, snap: KanbanSnapshot) {
        if self.tx.send(snap).is_err() {
            warn!("Failed to send KanbanSnapshot: no active subscribers");
        }
    }

    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated { .. }
        )
    }

    fn core_state_to_ui(&self, state: CoreCardState) -> CardState {
        match state {
            CoreCardState::Open => CardState::Todo,
            CoreCardState::Closed => CardState::Done,
            CoreCardState::Postponed => CardState::InReview,
            CoreCardState::Archived => CardState::Archived,
        }
    }

    fn core_priority_to_ui(&self, priority: Option<CorePriority>) -> Option<PriorityView> {
        priority.map(|p| match p {
            CorePriority::Urgent => PriorityView::Urgent,
            CorePriority::High => PriorityView::High,
            CorePriority::Normal => PriorityView::Normal,
            CorePriority::Low => PriorityView::Low,
        })
    }

    fn card_to_view(
        &self,
        card: &communitas_kanban::Card,
        tag_lookup: &std::collections::HashMap<String, TagView>,
    ) -> CardView {
        let tag_views: Vec<TagView> = card
            .tag_ids
            .iter()
            .filter_map(|id| tag_lookup.get(id).cloned())
            .collect();

        CardView {
            id: card.id.clone(),
            title: card.title.clone(),
            description: if card.description.is_empty() {
                None
            } else {
                Some(card.description.clone())
            },
            state: self.core_state_to_ui(card.state),
            priority: self.core_priority_to_ui(card.priority),
            assignees: card.assignee_ids.clone(),
            tags: tag_views,
            due_date: card.due_date.map(|d| d * 1000),
            checklist_progress: None, // TODO: Calculate from steps
            position: card.position,
            linked_thread_id: card.linked_thread_id.clone(),
            linked_thread_name: None, // Thread name resolved via messaging service
            sync_state: SyncState::Synced, // CRDT cards are considered synced
        }
    }

    fn settings_to_view(&self, settings: &communitas_kanban::BoardSettings) -> BoardSettings {
        BoardSettings {
            enable_wip_limits: true, // Always exposed in UI
            enable_due_dates: settings.enable_due_dates,
            enable_checklists: settings.enable_steps,
            default_column_id: settings.default_column_id.clone(),
        }
    }

    /// Resolve a four-word ID to a display name via DirectoryService.
    ///
    /// Falls back to the four-word ID if not found in the directory.
    async fn resolve_author_name(&self, author_id: &str) -> String {
        let snapshot = self.directory.snapshot().await;
        // Look for the author in the directory contacts
        snapshot
            .contacts
            .iter()
            .find(|c| c.id == author_id)
            .map(|c| c.display_name.clone())
            .unwrap_or_else(|| author_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use tempfile::TempDir;

    async fn make_service(temp: &TempDir) -> KanbanService {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());
        let directory = Arc::new(DirectoryService::new(auth.clone()));
        let app = Arc::new(
            CommunitasApp::new(
                "ocean-forest-moon-star".to_string(),
                "TestUser".to_string(),
                "TestDevice".to_string(),
                temp.path()
                    .join("app_storage")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            .unwrap(),
        );
        KanbanService::new(auth, app, directory)
    }

    #[tokio::test]
    async fn kanban_service_starts_empty() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let snap = service.current_snapshot();
        assert!(snap.boards.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn list_boards_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.list_boards("entity-1").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_board_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_board("board-1").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_board_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.create_board("entity-1", "Test Board", None).await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_card_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.create_card("board-1", "col-1", "Task 1").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn move_card_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.move_card("board-1", "card-1", "col-2", 0).await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn archive_card_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.archive_card("board-1", "card-1").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn add_step_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.add_step("board-1", "card-1", "Step 1").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn toggle_step_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.toggle_step("board-1", "card-1", "step-1").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn add_comment_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.add_comment("board-1", "card-1", "Hello").await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_priority_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .set_priority("board-1", "card-1", Some(PriorityView::High))
            .await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subscribe_returns_receiver() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let rx = service.subscribe();
        let snap = rx.borrow().clone();
        assert!(snap.boards.is_empty());
    }

    #[tokio::test]
    async fn set_boards_updates_subscribers() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let rx = service.subscribe();

        let boards = vec![BoardSummary {
            id: "b1".to_string(),
            name: "Sprint Board".to_string(),
            entity_id: "proj-1".to_string(),
            column_count: 3,
            card_count: 10,
            last_activity: Some(1234567890000),
        }];

        service.set_boards(boards);

        let snap = rx.borrow().clone();
        assert_eq!(snap.boards.len(), 1);
        assert_eq!(snap.boards[0].id, "b1");
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn set_loading_updates_subscribers() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let rx = service.subscribe();

        service.set_loading(true);
        assert!(rx.borrow().loading);

        service.set_loading(false);
        assert!(!rx.borrow().loading);
    }

    #[test]
    fn kanban_error_display() {
        let err = KanbanError::BoardNotFound("b1".to_string());
        assert_eq!(format!("{}", err), "board not found: b1");

        let err = KanbanError::NotAuthenticated;
        assert_eq!(format!("{}", err), "not authenticated");
    }

    #[test]
    fn card_update_default() {
        let update = CardUpdate::default();
        assert!(update.title.is_none());
        assert!(update.description.is_none());
        assert!(update.assignees.is_none());
        assert!(update.tags.is_none());
        assert!(update.due_date.is_none());
    }

    #[test]
    fn move_direction_variants() {
        assert_ne!(MoveDirection::Left, MoveDirection::Right);
        assert_ne!(MoveDirection::Up, MoveDirection::Down);
        assert_eq!(MoveDirection::Left, MoveDirection::Left);
    }

    #[test]
    fn card_move_result_fields() {
        let result = CardMoveResult {
            card_id: "card-1".to_string(),
            card_title: "Test Card".to_string(),
            target_column_id: "col-2".to_string(),
            target_column_name: "In Progress".to_string(),
            new_position: 0,
        };
        assert_eq!(result.card_id, "card-1");
        assert_eq!(result.card_title, "Test Card");
        assert_eq!(result.target_column_id, "col-2");
        assert_eq!(result.target_column_name, "In Progress");
        assert_eq!(result.new_position, 0);
    }

    #[tokio::test]
    async fn move_card_keyboard_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .move_card_keyboard("board-1", "card-1", "col-1", 0, MoveDirection::Right)
            .await;
        assert!(result.is_err());
        match result {
            Err(KanbanError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_swimlane_mode_updates_subscribers() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        // Subscribe to watch channel before changing mode
        let rx = service.subscribe();

        // Verify initial state is None
        let initial = rx.borrow().clone();
        assert_eq!(initial.swimlane_mode, SwimlaneMode::None);

        // Change swimlane mode
        service.set_swimlane_mode(SwimlaneMode::ByAssignee);

        // Verify subscriber received the update
        let updated = rx.borrow().clone();
        assert_eq!(updated.swimlane_mode, SwimlaneMode::ByAssignee);

        // Change again to verify multiple updates work
        service.set_swimlane_mode(SwimlaneMode::ByTag);
        let updated2 = rx.borrow().clone();
        assert_eq!(updated2.swimlane_mode, SwimlaneMode::ByTag);

        // Verify ByState mode also works
        service.set_swimlane_mode(SwimlaneMode::ByState);
        let updated3 = rx.borrow().clone();
        assert_eq!(updated3.swimlane_mode, SwimlaneMode::ByState);

        // Reset to None
        service.set_swimlane_mode(SwimlaneMode::None);
        let updated4 = rx.borrow().clone();
        assert_eq!(updated4.swimlane_mode, SwimlaneMode::None);
    }
}

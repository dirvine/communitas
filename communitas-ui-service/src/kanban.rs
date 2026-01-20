//! Kanban service for board and card operations with reactive subscriptions.

use std::sync::{Arc, Weak};

use communitas_core::app::CommunitasApp;
use communitas_core::command::Command;
use communitas_kanban::{
    CardState as CoreCardState, CardUpdate as CoreCardUpdate, KanbanService as CoreKanbanService,
};
use communitas_ui_api::{
    BoardSettings, BoardSummary, BoardView, CardDetail, CardState, CardView, ChecklistProgress,
    ColumnView, CommentView, StepView, TagView,
};
use thiserror::Error;
use tokio::sync::{RwLock, watch};
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
}

/// Service for kanban board and card operations.
///
/// The service automatically reinitializes its CRDT backend when the user authenticates,
/// ensuring operations are attributed to the correct peer_id. When the user logs out,
/// boards are cleared and the CRDT backend resets to anonymous.
pub struct KanbanService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    /// CRDT backend wrapped in RwLock to allow reinitialization on auth changes.
    core: Arc<RwLock<CoreKanbanService>>,
    tx: watch::Sender<KanbanSnapshot>,
    rx: watch::Receiver<KanbanSnapshot>,
}

impl KanbanService {
    /// Create a new kanban service linked to the auth controller.
    ///
    /// Spawns a background task that watches for authentication state changes and
    /// reinitializes the CRDT backend with the authenticated peer_id when the user logs in.
    /// When the user logs out, the backend resets to anonymous and boards are cleared.
    pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        let (tx, rx) = watch::channel(KanbanSnapshot::default());
        // Initialize core service with a default peer_id; will be updated on auth
        let core = Arc::new(RwLock::new(CoreKanbanService::new("anonymous")));

        let service = Self {
            auth,
            app,
            core,
            tx,
            rx,
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
            AuthStateSnapshot::Authenticated(session) => {
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
                let _ = tx.send(KanbanSnapshot::default());
                trace!("Boards cleared on logout");
            }
            AuthStateSnapshot::Authenticating => {
                // Transitional state, nothing to do
                trace!("Auth state is Authenticating, waiting for completion");
            }
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

    /// List all boards for an entity (project/group).
    #[instrument(skip(self), name = "ui.kanban.list_boards", fields(entity_id))]
    pub async fn list_boards(&self, entity_id: &str) -> Result<Vec<BoardSummary>, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

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
                    last_activity: Some(b.updated_at * 1000), // Convert to ms
                }
            })
            .collect();
        drop(core);

        // Update snapshot
        let mut snap = self.rx.borrow().clone();
        snap.boards = summaries.clone();
        snap.loading = false;
        let _ = self.tx.send(snap);

        Ok(summaries)
    }

    /// Get a full board view with columns and cards.
    #[instrument(skip(self), name = "ui.kanban.get_board", fields(board_id))]
    pub async fn get_board(&self, board_id: &str) -> Result<BoardView, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

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
    #[instrument(skip(self), name = "ui.kanban.get_card", fields(board_id, card_id))]
    pub async fn get_card(&self, board_id: &str, card_id: &str) -> Result<CardDetail, KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        let card = core.get_card(board_id, card_id)?;
        let tags = core.list_tags(board_id)?;
        let comments = core.list_comments(board_id, card_id)?;
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

        // Get steps (checklist items) - need to count them for progress
        // Steps are part of the card in the core, but we need to fetch via get_step
        // For now, we'll return empty steps and activity as the core doesn't expose list_steps
        let steps: Vec<StepView> = Vec::new(); // TODO: Wire when list_steps is available
        let step_count = 0u32;
        let completed_count = 0u32;

        let checklist_progress = if step_count > 0 {
            Some(ChecklistProgress {
                completed: completed_count,
                total: step_count,
            })
        } else {
            None
        };

        let comment_views: Vec<CommentView> = comments
            .into_iter()
            .map(|c| CommentView {
                id: c.id,
                author_id: c.author_id.clone(),
                author_name: c.author_id, // TODO: Resolve to display name
                text: c.content,
                created_at: c.created_at * 1000,
            })
            .collect();

        let tag_views: Vec<TagView> = card
            .tag_ids
            .iter()
            .filter_map(|id| tag_lookup.get(id).cloned())
            .collect();

        Ok(CardDetail {
            id: card.id,
            title: card.title,
            description: if card.description.is_empty() {
                None
            } else {
                Some(card.description)
            },
            state: self.core_state_to_ui(card.state),
            assignees: card.assignee_ids,
            tags: tag_views,
            due_date: card.due_date.map(|d| d * 1000),
            checklist_progress,
            position: card.position,
            steps,
            comments: comment_views,
            attachments: Vec::new(), // TODO: Wire when attachments are available
            activity: Vec::new(),    // TODO: Wire activity log
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
        let _ = self.tx.send(snap);

        info!(board_id = %summary.id, "Board created successfully");
        Ok(summary)
    }

    /// Create a new column in a board.
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

        let core = self.core.read().await;
        let col = core.add_column(board_id, name.to_string(), Some(position))?;

        Ok(ColumnView {
            id: col.id,
            name: col.name,
            position: col.position,
            cards: Vec::new(),
            wip_limit: col.wip_limit,
        })
    }

    /// Create a new card in a column.
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

        let core = self.core.read().await;
        let card = core.create_card(board_id, column_id, title.to_string(), None)?;

        Ok(CardView {
            id: card.id,
            title: card.title,
            description: if card.description.is_empty() {
                None
            } else {
                Some(card.description)
            },
            state: self.core_state_to_ui(card.state),
            assignees: card.assignee_ids,
            tags: Vec::new(),
            due_date: card.due_date.map(|d| d * 1000),
            checklist_progress: None,
            position: card.position,
        })
    }

    /// Move a card to a different column or position.
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

        let core = self.core.read().await;
        core.move_card(board_id, card_id, target_column, position)?;
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

        // Perform the move
        core.move_card(board_id, card_id, &target_column_id, new_position)?;

        Ok(CardMoveResult {
            card_id: card_id.to_string(),
            card_title: card.title,
            target_column_id,
            target_column_name,
            new_position,
        })
    }

    /// Update a card's properties.
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

        let core = self.core.read().await;

        // Apply basic updates
        let core_update = CoreCardUpdate {
            title: updates.title,
            description: updates.description,
            due_date: updates.due_date,
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
    #[instrument(skip(self), name = "ui.kanban.archive_card", fields(board_id, card_id))]
    pub async fn archive_card(&self, board_id: &str, card_id: &str) -> Result<(), KanbanError> {
        if !self.is_authenticated() {
            return Err(KanbanError::NotAuthenticated);
        }

        let core = self.core.read().await;
        core.change_card_state(board_id, card_id, CoreCardState::Archived)?;
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

        Ok(CommentView {
            id: comment.id,
            author_id: comment.author_id.clone(),
            author_name: comment.author_id, // TODO: Resolve to display name
            text: comment.content,
            created_at: comment.created_at * 1000,
        })
    }

    /// Internal: set loading state.
    pub fn set_loading(&self, loading: bool) {
        let mut snap = self.rx.borrow().clone();
        snap.loading = loading;
        let _ = self.tx.send(snap);
    }

    /// Internal: update the board list (called by core events).
    pub fn set_boards(&self, boards: Vec<BoardSummary>) {
        let mut snap = self.rx.borrow().clone();
        snap.boards = boards;
        snap.loading = false;
        let _ = self.tx.send(snap);
    }

    // ===== Helper Methods =====

    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated(_)
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
            assignees: card.assignee_ids.clone(),
            tags: tag_views,
            due_date: card.due_date.map(|d| d * 1000),
            checklist_progress: None, // TODO: Calculate from steps
            position: card.position,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use tempfile::TempDir;

    async fn make_service(temp: &TempDir) -> KanbanService {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());
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
        KanbanService::new(auth, app)
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
}

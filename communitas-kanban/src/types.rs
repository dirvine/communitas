//! Core types for the Kanban system.
//!
//! This module defines all the primary data structures used in the Kanban system:
//! - [`Board`]: A Kanban board owned by a Project
//! - [`Column`]: Workflow stages within a board
//! - [`Card`]: Work items with rich metadata
//! - [`Step`]: Checklist items within cards
//! - [`Comment`]: Threaded discussions on cards
//! - [`Tag`]: Categorization labels for cards

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::state_machine::CardState;

/// Priority level for a card.
///
/// Cards can have an optional priority to help users focus on what matters most.
/// Priority levels determine visual indicators and can be used for filtering/sorting.
///
/// # Colors
/// - Urgent: Red (#DC2626)
/// - High: Orange (#EA580C)
/// - Normal: Blue (#2563EB)
/// - Low: Gray (#6B7280)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum Priority {
    /// Highest priority - requires immediate attention
    Urgent,
    /// High priority - should be addressed soon
    High,
    /// Normal priority - standard work items
    #[default]
    Normal,
    /// Low priority - can be deferred
    Low,
}

impl Priority {
    /// Get a human-readable label for the priority.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Priority::Urgent => "Urgent",
            Priority::High => "High",
            Priority::Normal => "Normal",
            Priority::Low => "Low",
        }
    }

    /// Get the hex color code for visual indicators.
    #[must_use]
    pub fn color(&self) -> &'static str {
        match self {
            Priority::Urgent => "#DC2626",
            Priority::High => "#EA580C",
            Priority::Normal => "#2563EB",
            Priority::Low => "#6B7280",
        }
    }

    /// Get the sort order (lower value = higher priority).
    #[must_use]
    pub fn sort_order(&self) -> u8 {
        match self {
            Priority::Urgent => 0,
            Priority::High => 1,
            Priority::Normal => 2,
            Priority::Low => 3,
        }
    }

    /// Get all priority levels in order.
    #[must_use]
    pub fn all() -> &'static [Priority] {
        &[
            Priority::Urgent,
            Priority::High,
            Priority::Normal,
            Priority::Low,
        ]
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A Kanban board owned by a Project.
///
/// Boards contain columns which organize cards into workflow stages.
/// Each board is owned by a single project identified by its four-word identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// Unique identifier for the board (UUID)
    pub id: String,
    /// Display name of the board
    pub name: String,
    /// Optional description of the board's purpose
    pub description: Option<String>,
    /// Four-word identity of the owning project
    pub project_id: String,
    /// Four-word identity of the creator
    pub created_by: String,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Unix timestamp of last update
    pub updated_at: i64,
    /// Board configuration settings
    pub settings: BoardSettings,
    /// Soft delete flag
    pub deleted: bool,
    /// Unix timestamp when deleted (if deleted)
    pub deleted_at: Option<i64>,
}

/// Configuration settings for a board.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoardSettings {
    /// Default column ID for new cards
    pub default_column_id: Option<String>,
    /// Enable checklist steps on cards
    pub enable_steps: bool,
    /// Enable user assignments on cards
    pub enable_assignments: bool,
    /// Enable tags on cards
    pub enable_tags: bool,
    /// Enable due dates on cards
    pub enable_due_dates: bool,
    /// Automatically archive completed cards
    pub auto_archive_completed: bool,
    /// Days after completion to auto-archive (if enabled)
    pub archive_after_days: Option<u32>,
}

impl BoardSettings {
    /// Create settings with all features enabled (default)
    pub fn all_features() -> Self {
        Self {
            default_column_id: None,
            enable_steps: true,
            enable_assignments: true,
            enable_tags: true,
            enable_due_dates: true,
            auto_archive_completed: false,
            archive_after_days: None,
        }
    }
}

/// Update payload for board modifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoardUpdate {
    /// New name for the board
    pub name: Option<String>,
    /// New description for the board
    pub description: Option<Option<String>>,
    /// Updated settings
    pub settings: Option<BoardSettings>,
}

/// A column within a board representing a workflow stage.
///
/// Columns organize cards into stages like "To Do", "In Progress", "Done".
/// They can optionally have WIP (Work In Progress) limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Unique identifier for the column (UUID)
    pub id: String,
    /// ID of the parent board
    pub board_id: String,
    /// Display name of the column
    pub name: String,
    /// Position within the board (0-indexed)
    pub position: u32,
    /// Optional hex color code (e.g., "#FF5733")
    pub color: Option<String>,
    /// Optional WIP limit (max cards in this column)
    pub wip_limit: Option<u32>,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Soft delete flag
    pub deleted: bool,
}

/// Update payload for column modifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColumnUpdate {
    /// New name for the column
    pub name: Option<String>,
    /// New color for the column
    pub color: Option<Option<String>>,
    /// New WIP limit for the column
    pub wip_limit: Option<Option<u32>>,
}

/// A card (work item) within a column.
///
/// Cards are the primary work items in the Kanban system. They contain:
/// - Title and rich text description
/// - State (Open, Closed, Postponed, Archived)
/// - Assignees and tags (OR-Set collections)
/// - Checklist steps
/// - Threaded comments
/// - Due date tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    /// Unique identifier for the card (UUID)
    pub id: String,
    /// ID of the parent board
    pub board_id: String,
    /// ID of the containing column
    pub column_id: String,
    /// Title of the card
    pub title: String,
    /// Rich text description (stored as YText in CRDT)
    pub description: String,
    /// Position within the column (0-indexed)
    pub position: u32,

    // State
    /// Current state of the card
    pub state: CardState,
    /// Priority level for the card
    pub priority: Option<Priority>,
    /// Whether this is a draft (not yet published)
    pub is_draft: bool,
    /// Whether this card is starred/highlighted
    pub is_golden: bool,

    // Metadata
    /// Four-word identity of the creator
    pub created_by: String,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Unix timestamp of last update
    pub updated_at: i64,
    /// Optional due date (Unix timestamp)
    pub due_date: Option<i64>,
    /// Timestamp when card was completed (if closed)
    pub completed_at: Option<i64>,

    // Collections (OR-Set in CRDT)
    /// Four-word identities of assigned users
    pub assignee_ids: Vec<String>,
    /// IDs of attached tags
    pub tag_ids: Vec<String>,

    // Tombstone
    /// Soft delete flag
    pub deleted: bool,
    /// Unix timestamp when deleted (if deleted)
    pub deleted_at: Option<i64>,
    /// Four-word identity of who deleted the card
    pub deleted_by: Option<String>,
}

/// Update payload for card modifications.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardUpdate {
    /// New title for the card
    pub title: Option<String>,
    /// New description for the card
    pub description: Option<String>,
    /// New priority level (None to clear)
    pub priority: Option<Option<Priority>>,
    /// New draft status
    pub is_draft: Option<bool>,
    /// New golden/starred status
    pub is_golden: Option<bool>,
    /// New due date (None to clear)
    pub due_date: Option<Option<i64>>,
}

/// A checklist step within a card.
///
/// Steps allow breaking down cards into smaller actionable items
/// that can be checked off individually.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique identifier for the step (UUID)
    pub id: String,
    /// ID of the parent card
    pub card_id: String,
    /// Text content of the step
    pub text: String,
    /// Whether the step is completed
    pub completed: bool,
    /// Four-word identity of who completed it (if completed)
    pub completed_by: Option<String>,
    /// Unix timestamp when completed (if completed)
    pub completed_at: Option<i64>,
    /// Position within the card's steps (0-indexed)
    pub position: u32,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Soft delete flag
    pub deleted: bool,
}

/// A comment on a card.
///
/// Comments support threaded discussions via the `reply_to_id` field.
/// Content is stored as rich text (YText in CRDT).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Unique identifier for the comment (UUID)
    pub id: String,
    /// ID of the parent card
    pub card_id: String,
    /// Four-word identity of the author
    pub author_id: String,
    /// Rich text content of the comment
    pub content: String,
    /// Unix timestamp of creation
    pub created_at: i64,
    /// Unix timestamp of last update
    pub updated_at: i64,
    /// ID of parent comment for threading (None for top-level)
    pub reply_to_id: Option<String>,
    /// Soft delete flag
    pub deleted: bool,
}

/// A tag for categorizing cards.
///
/// Tags are defined at the board level and can be attached
/// to any card within that board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Unique identifier for the tag (UUID)
    pub id: String,
    /// ID of the parent board
    pub board_id: String,
    /// Display name of the tag
    pub name: String,
    /// Hex color code (e.g., "#FF5733")
    pub color: String,
    /// Soft delete flag
    pub deleted: bool,
}

/// Events emitted when the CRDT document changes.
///
/// These events are fired after each transaction commits, enabling
/// real-time UI updates when local or remote changes occur.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KanbanEvent {
    /// A card was created.
    CardCreated {
        /// ID of the board containing the card.
        board_id: String,
        /// ID of the column containing the card.
        column_id: String,
        /// ID of the newly created card.
        card_id: String,
    },
    /// A card was updated (title, description, priority, due date, etc.).
    CardUpdated {
        /// ID of the board containing the card.
        board_id: String,
        /// ID of the updated card.
        card_id: String,
    },
    /// A card was moved to a different column or position.
    CardMoved {
        /// ID of the board containing the card.
        board_id: String,
        /// ID of the moved card.
        card_id: String,
        /// ID of the source column.
        from_column_id: String,
        /// ID of the target column.
        to_column_id: String,
    },
    /// A card was deleted.
    CardDeleted {
        /// ID of the board containing the card.
        board_id: String,
        /// ID of the deleted card.
        card_id: String,
    },
    /// A column was created, updated, or deleted.
    ColumnChanged {
        /// ID of the board containing the column.
        board_id: String,
        /// ID of the affected column.
        column_id: String,
    },
    /// Board metadata was updated.
    BoardUpdated {
        /// ID of the updated board.
        board_id: String,
    },
    /// Generic change notification (used for batched/debounced updates).
    ///
    /// When multiple changes occur rapidly, they may be coalesced into
    /// a single `BoardChanged` event to reduce UI update frequency.
    BoardChanged {
        /// ID of the changed board.
        board_id: String,
    },
    /// A concurrent edit conflict was detected during CRDT merge.
    ///
    /// This occurs when a remote change modifies the same card that
    /// was recently edited locally. The CRDT automatically resolves
    /// the conflict, but the user should be notified.
    ConflictDetected {
        /// ID of the board containing the conflict.
        board_id: String,
        /// ID of the card involved in the conflict.
        card_id: String,
        /// Brief description of what changed remotely.
        remote_change: String,
    },
}

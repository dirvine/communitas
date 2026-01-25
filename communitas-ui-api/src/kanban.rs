//! Kanban DTOs for board, column, card, and related views.
//!
//! These types are designed for UI rendering and MCP tool responses.
//! They provide a simplified view of the underlying CRDT-based kanban data.

use serde::{Deserialize, Serialize};

use crate::SyncState;

/// Summary of a board for list views.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardSummary {
    /// Unique board identifier.
    pub id: String,
    /// Display name of the board.
    pub name: String,
    /// Entity (project/group) that owns this board.
    pub entity_id: String,
    /// Number of columns in the board.
    pub column_count: u32,
    /// Total number of cards across all columns.
    pub card_count: u32,
    /// Unix timestamp (ms) of last activity, if any.
    pub last_activity: Option<i64>,
}

/// Full board view with columns and cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardView {
    /// Unique board identifier.
    pub id: String,
    /// Display name of the board.
    pub name: String,
    /// Entity (project/group) that owns this board.
    pub entity_id: String,
    /// Optional description of the board.
    pub description: Option<String>,
    /// Columns in display order.
    pub columns: Vec<ColumnView>,
    /// Board configuration settings.
    pub settings: BoardSettings,
    /// Unix timestamp (ms) when the board was created.
    pub created_at: i64,
    /// Unix timestamp (ms) of last update.
    pub updated_at: i64,
}

/// A column within a board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnView {
    /// Unique column identifier.
    pub id: String,
    /// Display name of the column.
    pub name: String,
    /// Position within the board (0-indexed).
    pub position: u32,
    /// Cards in this column in display order.
    pub cards: Vec<CardView>,
    /// Optional WIP (work-in-progress) limit.
    pub wip_limit: Option<u32>,
}

/// A card (work item) in a column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardView {
    /// Unique card identifier.
    pub id: String,
    /// Card title.
    pub title: String,
    /// Optional description (may be truncated for list views).
    pub description: Option<String>,
    /// Current workflow state.
    pub state: CardState,
    /// Priority level for the card.
    pub priority: Option<PriorityView>,
    /// Four-word identities of assigned users.
    pub assignees: Vec<String>,
    /// Tags attached to this card.
    pub tags: Vec<TagView>,
    /// Optional due date as Unix timestamp (ms).
    pub due_date: Option<i64>,
    /// Checklist progress summary, if the card has steps.
    pub checklist_progress: Option<ChecklistProgress>,
    /// Position within the column (0-indexed).
    pub position: u32,
    /// Linked message thread ID for discussions.
    pub linked_thread_id: Option<String>,
    /// Display name of the linked thread (if any).
    pub linked_thread_name: Option<String>,
    /// Sync state of this card.
    pub sync_state: SyncState,
}

/// Detailed card view with full content.
///
/// Extends [`CardView`] with steps, comments, attachments, and activity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardDetail {
    /// Unique card identifier.
    pub id: String,
    /// Card title.
    pub title: String,
    /// Full description content.
    pub description: Option<String>,
    /// Current workflow state.
    pub state: CardState,
    /// Priority level for the card.
    pub priority: Option<PriorityView>,
    /// Four-word identities of assigned users.
    pub assignees: Vec<String>,
    /// Tags attached to this card.
    pub tags: Vec<TagView>,
    /// Optional due date as Unix timestamp (ms).
    pub due_date: Option<i64>,
    /// Checklist progress summary.
    pub checklist_progress: Option<ChecklistProgress>,
    /// Position within the column (0-indexed).
    pub position: u32,
    /// Checklist steps.
    pub steps: Vec<StepView>,
    /// Comments on this card.
    pub comments: Vec<CommentView>,
    /// File attachments.
    pub attachments: Vec<AttachmentView>,
    /// Activity log entries.
    pub activity: Vec<ActivityEntry>,
    /// Linked message thread ID for discussions.
    pub linked_thread_id: Option<String>,
    /// Display name of the linked thread (if any).
    pub linked_thread_name: Option<String>,
    /// Sync state of this card.
    pub sync_state: SyncState,
}

/// A tag for categorizing cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagView {
    /// Unique tag identifier.
    pub id: String,
    /// Display name of the tag.
    pub name: String,
    /// Hex color code (e.g., "#FF5733").
    pub color: String,
}

/// A checklist step within a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepView {
    /// Unique step identifier.
    pub id: String,
    /// Step text/title.
    pub title: String,
    /// Whether the step is completed.
    pub completed: bool,
}

/// A comment on a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentView {
    /// Unique comment identifier.
    pub id: String,
    /// Four-word identity of the author.
    pub author_id: String,
    /// Display name of the author.
    pub author_name: String,
    /// Comment text content.
    pub text: String,
    /// Unix timestamp (ms) when the comment was created.
    pub created_at: i64,
}

/// Summary of checklist completion progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistProgress {
    /// Number of completed steps.
    pub completed: u32,
    /// Total number of steps.
    pub total: u32,
}

/// Priority level for a card.
///
/// Used by UI layers to display visual indicators and support filtering/sorting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PriorityView {
    /// Highest priority - requires immediate attention.
    Urgent,
    /// High priority - should be addressed soon.
    High,
    /// Normal priority - standard work items.
    #[default]
    Normal,
    /// Low priority - can be deferred.
    Low,
}

impl PriorityView {
    /// Get a human-readable label for the priority.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            PriorityView::Urgent => "Urgent",
            PriorityView::High => "High",
            PriorityView::Normal => "Normal",
            PriorityView::Low => "Low",
        }
    }

    /// Get the hex color code for the priority.
    #[must_use]
    pub fn color(&self) -> &'static str {
        match self {
            PriorityView::Urgent => "#DC2626",
            PriorityView::High => "#EA580C",
            PriorityView::Normal => "#2563EB",
            PriorityView::Low => "#6B7280",
        }
    }

    /// Get sort order (lower = higher priority).
    #[must_use]
    pub fn sort_order(&self) -> u8 {
        match self {
            PriorityView::Urgent => 0,
            PriorityView::High => 1,
            PriorityView::Normal => 2,
            PriorityView::Low => 3,
        }
    }

    /// Returns all priority levels in order of importance.
    #[must_use]
    pub fn all() -> &'static [PriorityView] {
        &[
            PriorityView::Urgent,
            PriorityView::High,
            PriorityView::Normal,
            PriorityView::Low,
        ]
    }
}

impl std::fmt::Display for PriorityView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Card workflow state.
///
/// These states represent the card's position in a typical workflow.
/// UI layers may map these to column positions or visual indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CardState {
    /// Card is waiting to be started.
    #[default]
    Todo,
    /// Card is actively being worked on.
    InProgress,
    /// Card is in review/QA.
    InReview,
    /// Card is complete.
    Done,
    /// Card is archived (hidden from active views).
    Archived,
}

impl CardState {
    /// Get a human-readable label for the state.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            CardState::Todo => "To Do",
            CardState::InProgress => "In Progress",
            CardState::InReview => "In Review",
            CardState::Done => "Done",
            CardState::Archived => "Archived",
        }
    }
}

impl std::fmt::Display for CardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Board configuration settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BoardSettings {
    /// Whether WIP limits are enforced.
    pub enable_wip_limits: bool,
    /// Whether due dates are enabled.
    pub enable_due_dates: bool,
    /// Whether checklists (steps) are enabled.
    pub enable_checklists: bool,
    /// Default column for new cards.
    pub default_column_id: Option<String>,
}

/// A file attachment on a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentView {
    /// Unique attachment identifier.
    pub id: String,
    /// File name.
    pub name: String,
    /// MIME type of the file.
    pub mime_type: String,
    /// File size in bytes.
    pub size_bytes: u64,
}

/// An entry in the card activity log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityEntry {
    /// Unique activity entry identifier.
    pub id: String,
    /// Type of action (e.g., "created", "moved", "commented").
    pub action_type: String,
    /// Four-word identity of the actor.
    pub actor_id: String,
    /// Display name of the actor.
    pub actor_name: String,
    /// Human-readable description of the action.
    pub description: String,
    /// Unix timestamp (ms) when the action occurred.
    pub timestamp: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_summary_equality() {
        let b1 = BoardSummary {
            id: "board-1".to_string(),
            name: "Sprint Board".to_string(),
            entity_id: "proj-1".to_string(),
            column_count: 4,
            card_count: 12,
            last_activity: Some(1705500000000),
        };
        let b2 = b1.clone();
        assert_eq!(b1, b2);
    }

    #[test]
    fn card_state_default() {
        let state = CardState::default();
        assert_eq!(state, CardState::Todo);
    }

    #[test]
    fn card_state_display() {
        assert_eq!(format!("{}", CardState::Todo), "To Do");
        assert_eq!(format!("{}", CardState::InProgress), "In Progress");
        assert_eq!(format!("{}", CardState::InReview), "In Review");
        assert_eq!(format!("{}", CardState::Done), "Done");
        assert_eq!(format!("{}", CardState::Archived), "Archived");
    }

    #[test]
    fn checklist_progress_construction() {
        let progress = ChecklistProgress {
            completed: 3,
            total: 5,
        };
        assert_eq!(progress.completed, 3);
        assert_eq!(progress.total, 5);
    }

    #[test]
    fn card_view_with_tags() {
        let card = CardView {
            id: "card-1".to_string(),
            title: "Implement feature".to_string(),
            description: Some("Add the new feature".to_string()),
            state: CardState::InProgress,
            priority: Some(PriorityView::High),
            assignees: vec!["alice-beta-charlie-delta".to_string()],
            tags: vec![TagView {
                id: "tag-1".to_string(),
                name: "urgent".to_string(),
                color: "#FF0000".to_string(),
            }],
            due_date: Some(1705600000000),
            checklist_progress: Some(ChecklistProgress {
                completed: 2,
                total: 4,
            }),
            position: 0,
            linked_thread_id: None,
            linked_thread_name: None,
            sync_state: SyncState::Synced,
        };
        assert_eq!(card.tags.len(), 1);
        assert_eq!(card.tags[0].name, "urgent");
        assert_eq!(card.priority, Some(PriorityView::High));
    }

    #[test]
    fn board_settings_default() {
        let settings = BoardSettings::default();
        assert!(!settings.enable_wip_limits);
        assert!(!settings.enable_due_dates);
        assert!(!settings.enable_checklists);
        assert!(settings.default_column_id.is_none());
    }

    #[test]
    fn activity_entry_construction() {
        let entry = ActivityEntry {
            id: "act-1".to_string(),
            action_type: "moved".to_string(),
            actor_id: "user-1".to_string(),
            actor_name: "Alice".to_string(),
            description: "Moved card to In Progress".to_string(),
            timestamp: 1705500000000,
        };
        assert_eq!(entry.action_type, "moved");
    }
}

/// Swimlane view modes for grouping cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SwimlaneMode {
    /// Standard column view (no swimlanes).
    #[default]
    None,
    /// Group cards by assignee.
    ByAssignee,
    /// Group cards by tag.
    ByTag,
    /// Group cards by state.
    ByState,
}

impl SwimlaneMode {
    /// Returns a human-readable label for the mode.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            SwimlaneMode::None => "Standard",
            SwimlaneMode::ByAssignee => "By Assignee",
            SwimlaneMode::ByTag => "By Tag",
            SwimlaneMode::ByState => "By State",
        }
    }
}

impl std::fmt::Display for SwimlaneMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod swimlane_tests {
    use super::*;

    #[test]
    fn swimlane_mode_default() {
        let mode = SwimlaneMode::default();
        assert_eq!(mode, SwimlaneMode::None);
    }

    #[test]
    fn swimlane_mode_labels() {
        assert_eq!(SwimlaneMode::None.label(), "Standard");
        assert_eq!(SwimlaneMode::ByAssignee.label(), "By Assignee");
        assert_eq!(SwimlaneMode::ByTag.label(), "By Tag");
        assert_eq!(SwimlaneMode::ByState.label(), "By State");
    }

    #[test]
    fn swimlane_mode_display() {
        assert_eq!(format!("{}", SwimlaneMode::None), "Standard");
        assert_eq!(format!("{}", SwimlaneMode::ByAssignee), "By Assignee");
        assert_eq!(format!("{}", SwimlaneMode::ByTag), "By Tag");
        assert_eq!(format!("{}", SwimlaneMode::ByState), "By State");
    }

    #[test]
    fn swimlane_mode_equality() {
        assert_eq!(SwimlaneMode::None, SwimlaneMode::None);
        assert_ne!(SwimlaneMode::None, SwimlaneMode::ByAssignee);
        assert_ne!(SwimlaneMode::ByAssignee, SwimlaneMode::ByTag);
    }
}

#[cfg(test)]
mod priority_tests {
    use super::*;

    #[test]
    fn priority_default() {
        let priority = PriorityView::default();
        assert_eq!(priority, PriorityView::Normal);
    }

    #[test]
    fn priority_labels() {
        assert_eq!(PriorityView::Urgent.label(), "Urgent");
        assert_eq!(PriorityView::High.label(), "High");
        assert_eq!(PriorityView::Normal.label(), "Normal");
        assert_eq!(PriorityView::Low.label(), "Low");
    }

    #[test]
    fn priority_colors() {
        assert_eq!(PriorityView::Urgent.color(), "#DC2626");
        assert_eq!(PriorityView::High.color(), "#EA580C");
        assert_eq!(PriorityView::Normal.color(), "#2563EB");
        assert_eq!(PriorityView::Low.color(), "#6B7280");
    }

    #[test]
    fn priority_sort_order() {
        assert!(PriorityView::Urgent.sort_order() < PriorityView::High.sort_order());
        assert!(PriorityView::High.sort_order() < PriorityView::Normal.sort_order());
        assert!(PriorityView::Normal.sort_order() < PriorityView::Low.sort_order());
    }

    #[test]
    fn priority_all() {
        let all = PriorityView::all();
        assert_eq!(all.len(), 4);
        assert_eq!(all[0], PriorityView::Urgent);
        assert_eq!(all[3], PriorityView::Low);
    }

    #[test]
    fn priority_display() {
        assert_eq!(format!("{}", PriorityView::Urgent), "Urgent");
        assert_eq!(format!("{}", PriorityView::High), "High");
        assert_eq!(format!("{}", PriorityView::Normal), "Normal");
        assert_eq!(format!("{}", PriorityView::Low), "Low");
    }
}

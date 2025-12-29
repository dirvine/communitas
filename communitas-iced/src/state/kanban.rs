// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Kanban board state.

/// Kanban column definition.
#[derive(Debug, Clone)]
pub struct KanbanColumn {
    /// Column ID.
    pub id: String,
    /// Column name.
    pub name: String,
    /// Column color (hex).
    pub color: String,
    /// Sort order.
    pub order: u32,
}

impl KanbanColumn {
    /// Get default columns.
    #[must_use]
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: "backlog".to_string(),
                name: "Backlog".to_string(),
                color: "#6B7280".to_string(),
                order: 0,
            },
            Self {
                id: "todo".to_string(),
                name: "To Do".to_string(),
                color: "#3B82F6".to_string(),
                order: 1,
            },
            Self {
                id: "in_progress".to_string(),
                name: "In Progress".to_string(),
                color: "#F59E0B".to_string(),
                order: 2,
            },
            Self {
                id: "review".to_string(),
                name: "Review".to_string(),
                color: "#8B5CF6".to_string(),
                order: 3,
            },
            Self {
                id: "done".to_string(),
                name: "Done".to_string(),
                color: "#10B981".to_string(),
                order: 4,
            },
        ]
    }
}

/// Kanban card priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardPriority {
    /// Low priority.
    Low,
    /// Normal priority.
    #[default]
    Normal,
    /// High priority.
    High,
    /// Urgent priority.
    Urgent,
}

impl CardPriority {
    /// Get the display name for this priority.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Normal => "Normal",
            Self::High => "High",
            Self::Urgent => "Urgent",
        }
    }

    /// Get the color for this priority.
    #[must_use]
    pub fn color(&self) -> iced::Color {
        match self {
            Self::Low => iced::Color::from_rgb(0.5, 0.5, 0.5),
            Self::Normal => iced::Color::from_rgb(0.3, 0.5, 0.9),
            Self::High => iced::Color::from_rgb(0.9, 0.6, 0.2),
            Self::Urgent => iced::Color::from_rgb(0.9, 0.3, 0.3),
        }
    }

    /// Get all priority options.
    #[must_use]
    pub fn all() -> [Self; 4] {
        [Self::Low, Self::Normal, Self::High, Self::Urgent]
    }
}

/// A Kanban card.
#[derive(Debug, Clone)]
pub struct KanbanCard {
    /// Card ID.
    pub id: String,
    /// Project/entity ID.
    pub project_id: String,
    /// Column ID.
    pub column: String,
    /// Card title.
    pub title: String,
    /// Card description.
    pub description: Option<String>,
    /// Assignee four-word identity.
    pub assignee: Option<String>,
    /// Priority level.
    pub priority: CardPriority,
    /// Position within column.
    pub position: u32,
    /// Comment/discussion count.
    pub comment_count: usize,
    /// Created timestamp.
    pub created_at: i64,
    /// Whether the card is archived.
    pub is_archived: bool,
}

impl KanbanCard {
    /// Create a new card.
    #[must_use]
    pub fn new(id: String, project_id: String, column: String, title: String) -> Self {
        Self {
            id,
            project_id,
            column,
            title,
            description: None,
            assignee: None,
            priority: CardPriority::Normal,
            position: 0,
            comment_count: 0,
            created_at: chrono::Utc::now().timestamp(),
            is_archived: false,
        }
    }
}

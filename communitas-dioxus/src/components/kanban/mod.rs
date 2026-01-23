//! Kanban board UI components for Communitas.
//!
//! This module provides a complete Kanban board implementation with:
//! - Board list view with grid cards
//! - Board view with columns and cards
//! - Drag-and-drop card movement
//! - Card detail modal for editing
//! - Swimlane filters

mod board_list;
mod board_view;
pub(crate) mod card;
pub(crate) mod card_detail_modal;
pub(crate) mod column;
mod filters;

// Public exports for use in routes
pub use board_list::BoardListPage;
pub use board_view::BoardView;

/// Drag state for card drag-and-drop operations.
///
/// This is shared via Dioxus context between cards and columns to enable
/// drag-and-drop without relying on browser dataTransfer API.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DragState {
    /// The card currently being dragged, if any.
    pub dragging_card: Option<DraggingCard>,
}

/// Information about a card being dragged.
#[derive(Debug, Clone, PartialEq)]
pub struct DraggingCard {
    /// The ID of the card being dragged.
    pub card_id: String,
    /// The ID of the column the card is being dragged from.
    pub source_column_id: String,
    /// The title of the card (for announcements).
    pub card_title: String,
}

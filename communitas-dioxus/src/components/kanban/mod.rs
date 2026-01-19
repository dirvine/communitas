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

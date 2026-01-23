//! # communitas-kanban
//!
//! A CRDT-based Kanban system for Communitas.
//!
//! This crate provides offline-first, peer-to-peer collaborative project boards
//! with conflict-free synchronization via Yrs (Yjs) CRDTs.
//!
//! ## Features
//!
//! - **Boards**: Kanban boards owned by Projects
//! - **Columns**: Workflow stages with optional WIP limits
//! - **Cards**: Work items with rich text descriptions, checklists, assignments
//! - **Steps**: Checklist items within cards
//! - **Comments**: Threaded discussions on cards
//! - **Tags**: Categorization and filtering
//! - **Filtering**: Filter cards by state, assignee, tags, due dates
//!
//! ## Architecture
//!
//! - **Normalized Data Model**: Cards stored in global registry, columns store card IDs
//! - **Atomic Operations**: Card moves use single CRDT transactions
//! - **OR-Set Collections**: Conflict-free add/remove for assignees and tags
//! - **YText Rich Text**: Collaborative editing for descriptions and comments
//!
//! ## Usage
//!
//! ```ignore
//! use communitas_kanban::{KanbanService, BoardSettings};
//!
//! let service = KanbanService::new(crdt_manager, peer_id);
//!
//! // Create a board for a project
//! let board = service.create_board("project-four-words", "Sprint 1".to_string(), None).await?;
//!
//! // Add columns
//! let todo = service.add_column(&board.id, "To Do".to_string(), None).await?;
//! let done = service.add_column(&board.id, "Done".to_string(), None).await?;
//!
//! // Create and move cards
//! let card = service.create_card(&board.id, &todo.id, "Task 1".to_string(), None).await?;
//! service.move_card(&board.id, &card.id, &done.id, 0).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

mod analytics;
mod error;
mod filter;
mod operations;
mod service;
mod state_machine;
mod types;

// Re-exports
pub use analytics::{
    BoardAnalytics, BurndownChart, BurndownDataPoint, CycleTimeBucket, CycleTimeDistribution,
    TimeRange, VelocityDataPoint, VelocityMetric,
};
pub use error::{KanbanError, KanbanResult};
pub use filter::CardFilter;
pub use service::KanbanService;
pub use state_machine::CardState;
pub use types::{
    Board, BoardSettings, BoardUpdate, Card, CardUpdate, Column, ColumnUpdate, Comment,
    KanbanEvent, Priority, Step, Tag,
};

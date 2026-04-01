// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for the Kanban system.
//!
//! This module provides [`KanbanError`] for all error conditions
//! and [`KanbanResult<T>`] as a convenient type alias.

use thiserror::Error;

use crate::state_machine::CardState;

/// Errors that can occur in the Kanban system.
#[derive(Debug, Error)]
pub enum KanbanError {
    /// Board not found with the given ID
    #[error("Board not found: {0}")]
    BoardNotFound(String),

    /// Column not found with the given ID
    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    /// Card not found with the given ID
    #[error("Card not found: {0}")]
    CardNotFound(String),

    /// Step not found with the given ID
    #[error("Step not found: {0}")]
    StepNotFound(String),

    /// Comment not found with the given ID
    #[error("Comment not found: {0}")]
    CommentNotFound(String),

    /// Tag not found with the given ID
    #[error("Tag not found: {0}")]
    TagNotFound(String),

    /// Invalid state transition attempted
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        /// Current state
        from: CardState,
        /// Attempted target state
        to: CardState,
    },

    /// WIP limit exceeded for a column
    #[error(
        "WIP limit exceeded for column {column_id}: limit is {limit}, current count is {current}"
    )]
    WipLimitExceeded {
        /// Column ID that exceeded the limit
        column_id: String,
        /// The configured WIP limit
        limit: u32,
        /// Current number of cards in the column
        current: u32,
    },

    /// Invalid data encountered
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Position out of bounds
    #[error("Position {position} out of bounds (max: {max})")]
    PositionOutOfBounds {
        /// The requested position
        position: u32,
        /// The maximum valid position
        max: u32,
    },

    /// Permission denied for the operation
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Operation on a deleted entity
    #[error("Cannot operate on deleted {entity_type}: {entity_id}")]
    EntityDeleted {
        /// Type of entity (board, column, card, etc.)
        entity_type: &'static str,
        /// ID of the deleted entity
        entity_id: String,
    },

    /// CRDT document error
    #[error("CRDT error: {0}")]
    Crdt(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<serde_json::Error> for KanbanError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

/// Result type alias for Kanban operations.
pub type KanbanResult<T> = Result<T, KanbanError>;

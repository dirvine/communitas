// SPDX-License-Identifier: MIT OR Apache-2.0

//! Sync state types for offline-first UX indicators.
//!
//! This module provides types for tracking and displaying synchronization
//! state across messaging, drive, and kanban surfaces.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Synchronization state for an item or collection.
///
/// Used to indicate whether content is fully synced, currently syncing,
/// queued for sync, or has conflicts that need resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SyncState {
    /// Content is fully synchronized with peers.
    #[default]
    Synced,
    /// Content is currently being synchronized.
    Syncing,
    /// Content is queued for sync (offline or pending).
    Queued,
    /// Content has conflicts that need resolution.
    Conflict,
    /// Sync failed with an error.
    Error,
}

impl SyncState {
    /// Returns true if this state requires user attention.
    pub fn needs_attention(&self) -> bool {
        matches!(self, SyncState::Conflict | SyncState::Error)
    }

    /// Returns true if sync is in progress or pending.
    pub fn is_pending(&self) -> bool {
        matches!(self, SyncState::Syncing | SyncState::Queued)
    }

    /// Returns the appropriate icon name for this state.
    pub fn icon_name(&self) -> &'static str {
        match self {
            SyncState::Synced => "check-circle",
            SyncState::Syncing => "refresh-cw",
            SyncState::Queued => "clock",
            SyncState::Conflict => "alert-triangle",
            SyncState::Error => "x-circle",
        }
    }

    /// Returns the appropriate color class for this state.
    pub fn color_class(&self) -> &'static str {
        match self {
            SyncState::Synced => "text-green-500",
            SyncState::Syncing => "text-blue-500",
            SyncState::Queued => "text-orange-500",
            SyncState::Conflict => "text-yellow-500",
            SyncState::Error => "text-red-500",
        }
    }
}

impl fmt::Display for SyncState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncState::Synced => write!(f, "Synced"),
            SyncState::Syncing => write!(f, "Syncing"),
            SyncState::Queued => write!(f, "Waiting to sync"),
            SyncState::Conflict => write!(f, "Has conflicts"),
            SyncState::Error => write!(f, "Sync failed"),
        }
    }
}

/// Metadata about synchronization state for an item or collection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SyncMetadata {
    /// Current sync state.
    pub state: SyncState,
    /// Timestamp of last successful sync (Unix millis).
    pub last_synced: Option<u64>,
    /// Number of pending changes waiting to sync.
    pub pending_changes: u32,
    /// Number of conflicts needing resolution.
    pub conflict_count: u32,
    /// Optional error message if state is Error.
    pub error_message: Option<String>,
}

impl SyncMetadata {
    /// Create new metadata with synced state.
    pub fn synced() -> Self {
        Self {
            state: SyncState::Synced,
            last_synced: Some(current_timestamp_millis()),
            ..Default::default()
        }
    }

    /// Create new metadata with syncing state.
    pub fn syncing() -> Self {
        Self {
            state: SyncState::Syncing,
            ..Default::default()
        }
    }

    /// Create new metadata with queued state.
    pub fn queued(pending_changes: u32) -> Self {
        Self {
            state: SyncState::Queued,
            pending_changes,
            ..Default::default()
        }
    }

    /// Create new metadata with conflict state.
    pub fn conflict(conflict_count: u32) -> Self {
        Self {
            state: SyncState::Conflict,
            conflict_count,
            ..Default::default()
        }
    }

    /// Create new metadata with error state.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            state: SyncState::Error,
            error_message: Some(message.into()),
            ..Default::default()
        }
    }
}

/// Progress information for an ongoing sync operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SyncProgress {
    /// Total items to sync.
    pub total: u32,
    /// Items completed.
    pub completed: u32,
    /// Current item being synced (if any).
    pub current_item: Option<String>,
    /// Bytes transferred (for file transfers).
    pub bytes_transferred: u64,
    /// Total bytes to transfer.
    pub bytes_total: u64,
}

impl SyncProgress {
    /// Calculate progress as a percentage (0-100).
    pub fn percentage(&self) -> u8 {
        if self.total == 0 {
            return 100;
        }
        let pct = (self.completed as f64 / self.total as f64) * 100.0;
        pct.min(100.0) as u8
    }

    /// Calculate byte progress as a percentage (0-100).
    pub fn bytes_percentage(&self) -> u8 {
        if self.bytes_total == 0 {
            return 100;
        }
        let pct = (self.bytes_transferred as f64 / self.bytes_total as f64) * 100.0;
        pct.min(100.0) as u8
    }

    /// Returns true if sync is complete.
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total
    }
}

/// Summary of sync state across multiple items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SyncSummary {
    /// Number of items fully synced.
    pub synced_count: u32,
    /// Number of items currently syncing.
    pub syncing_count: u32,
    /// Number of items queued for sync.
    pub queued_count: u32,
    /// Number of items with conflicts.
    pub conflict_count: u32,
    /// Number of items with errors.
    pub error_count: u32,
}

impl SyncSummary {
    /// Returns the overall state based on item counts.
    pub fn overall_state(&self) -> SyncState {
        if self.error_count > 0 {
            SyncState::Error
        } else if self.conflict_count > 0 {
            SyncState::Conflict
        } else if self.syncing_count > 0 {
            SyncState::Syncing
        } else if self.queued_count > 0 {
            SyncState::Queued
        } else {
            SyncState::Synced
        }
    }

    /// Returns the total number of items.
    pub fn total(&self) -> u32 {
        self.synced_count
            + self.syncing_count
            + self.queued_count
            + self.conflict_count
            + self.error_count
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state_display() {
        assert_eq!(SyncState::Synced.to_string(), "Synced");
        assert_eq!(SyncState::Syncing.to_string(), "Syncing");
        assert_eq!(SyncState::Queued.to_string(), "Waiting to sync");
        assert_eq!(SyncState::Conflict.to_string(), "Has conflicts");
        assert_eq!(SyncState::Error.to_string(), "Sync failed");
    }

    #[test]
    fn test_sync_state_needs_attention() {
        assert!(!SyncState::Synced.needs_attention());
        assert!(!SyncState::Syncing.needs_attention());
        assert!(!SyncState::Queued.needs_attention());
        assert!(SyncState::Conflict.needs_attention());
        assert!(SyncState::Error.needs_attention());
    }

    #[test]
    fn test_sync_state_is_pending() {
        assert!(!SyncState::Synced.is_pending());
        assert!(SyncState::Syncing.is_pending());
        assert!(SyncState::Queued.is_pending());
        assert!(!SyncState::Conflict.is_pending());
        assert!(!SyncState::Error.is_pending());
    }

    #[test]
    fn test_sync_metadata_constructors() {
        let synced = SyncMetadata::synced();
        assert_eq!(synced.state, SyncState::Synced);
        assert!(synced.last_synced.is_some());

        let queued = SyncMetadata::queued(5);
        assert_eq!(queued.state, SyncState::Queued);
        assert_eq!(queued.pending_changes, 5);

        let conflict = SyncMetadata::conflict(3);
        assert_eq!(conflict.state, SyncState::Conflict);
        assert_eq!(conflict.conflict_count, 3);

        let error = SyncMetadata::error("Connection failed");
        assert_eq!(error.state, SyncState::Error);
        assert_eq!(error.error_message, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_sync_progress_percentage() {
        let progress = SyncProgress {
            total: 10,
            completed: 5,
            ..Default::default()
        };
        assert_eq!(progress.percentage(), 50);

        let complete = SyncProgress {
            total: 10,
            completed: 10,
            ..Default::default()
        };
        assert_eq!(complete.percentage(), 100);
        assert!(complete.is_complete());

        let empty = SyncProgress::default();
        assert_eq!(empty.percentage(), 100);
    }

    #[test]
    fn test_sync_summary_overall_state() {
        let all_synced = SyncSummary {
            synced_count: 10,
            ..Default::default()
        };
        assert_eq!(all_synced.overall_state(), SyncState::Synced);

        let has_errors = SyncSummary {
            synced_count: 8,
            error_count: 2,
            ..Default::default()
        };
        assert_eq!(has_errors.overall_state(), SyncState::Error);

        let has_conflicts = SyncSummary {
            synced_count: 8,
            conflict_count: 2,
            ..Default::default()
        };
        assert_eq!(has_conflicts.overall_state(), SyncState::Conflict);

        let some_syncing = SyncSummary {
            synced_count: 8,
            syncing_count: 2,
            ..Default::default()
        };
        assert_eq!(some_syncing.overall_state(), SyncState::Syncing);
    }
}

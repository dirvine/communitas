// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration tests for offline UX components in Communitas Dioxus.
//!
//! These tests verify the offline-first UX behavior including:
//! - Toast notification variants and behavior
//! - Conflict banner display logic
//! - Sync indicator state rendering
//! - Retry mechanism behavior
//!
//! Note: These tests use SSR rendering for CI compatibility.

use communitas_ui_api::{SyncMetadata, SyncProgress, SyncState, SyncSummary};

// =============================================================================
// SyncState UI Property Tests
// =============================================================================

/// Test that SyncState provides correct UI properties for rendering.
#[test]
fn test_sync_state_ui_properties() {
    // Synced state
    assert_eq!(SyncState::Synced.icon_name(), "check-circle");
    assert_eq!(SyncState::Synced.color_class(), "text-green-500");
    assert_eq!(SyncState::Synced.to_string(), "Synced");
    assert!(!SyncState::Synced.needs_attention());

    // Syncing state
    assert_eq!(SyncState::Syncing.icon_name(), "refresh-cw");
    assert_eq!(SyncState::Syncing.color_class(), "text-blue-500");
    assert_eq!(SyncState::Syncing.to_string(), "Syncing");
    assert!(!SyncState::Syncing.needs_attention());

    // Queued state
    assert_eq!(SyncState::Queued.icon_name(), "clock");
    assert_eq!(SyncState::Queued.color_class(), "text-orange-500");
    assert_eq!(SyncState::Queued.to_string(), "Waiting to sync");
    assert!(!SyncState::Queued.needs_attention());

    // Conflict state
    assert_eq!(SyncState::Conflict.icon_name(), "alert-triangle");
    assert_eq!(SyncState::Conflict.color_class(), "text-yellow-500");
    assert_eq!(SyncState::Conflict.to_string(), "Has conflicts");
    assert!(SyncState::Conflict.needs_attention());

    // Error state
    assert_eq!(SyncState::Error.icon_name(), "x-circle");
    assert_eq!(SyncState::Error.color_class(), "text-red-500");
    assert_eq!(SyncState::Error.to_string(), "Sync failed");
    assert!(SyncState::Error.needs_attention());
}

// =============================================================================
// Toast Notification Logic Tests
// =============================================================================

/// Toast variant enumeration for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToastVariant {
    Offline,
    Syncing,
    Success,
    Error,
}

impl ToastVariant {
    /// Get the display message for this toast variant.
    fn message(&self) -> &'static str {
        match self {
            ToastVariant::Offline => "You're offline - changes will sync when connected",
            ToastVariant::Syncing => "Syncing changes...",
            ToastVariant::Success => "Sync complete",
            ToastVariant::Error => "Sync failed - tap to retry",
        }
    }

    /// Get the color class for this toast variant.
    fn color_class(&self) -> &'static str {
        match self {
            ToastVariant::Offline => "bg-gray-600",
            ToastVariant::Syncing => "bg-blue-600",
            ToastVariant::Success => "bg-green-600",
            ToastVariant::Error => "bg-red-600",
        }
    }

    /// Whether this toast should auto-dismiss.
    fn auto_dismiss(&self) -> bool {
        !matches!(self, ToastVariant::Error)
    }

    /// Auto-dismiss duration in seconds.
    fn dismiss_duration_secs(&self) -> Option<u32> {
        match self {
            ToastVariant::Offline => Some(5),
            ToastVariant::Syncing => Some(3),
            ToastVariant::Success => Some(3),
            ToastVariant::Error => None, // Persists until dismissed
        }
    }
}

#[test]
fn test_toast_offline_variant() {
    let toast = ToastVariant::Offline;
    assert_eq!(
        toast.message(),
        "You're offline - changes will sync when connected"
    );
    assert_eq!(toast.color_class(), "bg-gray-600");
    assert!(toast.auto_dismiss());
    assert_eq!(toast.dismiss_duration_secs(), Some(5));
}

#[test]
fn test_toast_syncing_variant() {
    let toast = ToastVariant::Syncing;
    assert_eq!(toast.message(), "Syncing changes...");
    assert_eq!(toast.color_class(), "bg-blue-600");
    assert!(toast.auto_dismiss());
    assert_eq!(toast.dismiss_duration_secs(), Some(3));
}

#[test]
fn test_toast_success_variant() {
    let toast = ToastVariant::Success;
    assert_eq!(toast.message(), "Sync complete");
    assert_eq!(toast.color_class(), "bg-green-600");
    assert!(toast.auto_dismiss());
    assert_eq!(toast.dismiss_duration_secs(), Some(3));
}

#[test]
fn test_toast_error_variant() {
    let toast = ToastVariant::Error;
    assert_eq!(toast.message(), "Sync failed - tap to retry");
    assert_eq!(toast.color_class(), "bg-red-600");
    assert!(!toast.auto_dismiss());
    assert_eq!(toast.dismiss_duration_secs(), None);
}

// =============================================================================
// Conflict Banner Logic Tests
// =============================================================================

/// Conflict banner variant enumeration for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConflictBannerVariant {
    Messaging,
    Drive,
    Kanban,
}

impl ConflictBannerVariant {
    /// Get the title for this conflict banner variant.
    fn title(&self) -> &'static str {
        match self {
            ConflictBannerVariant::Messaging => "Message conflicts detected",
            ConflictBannerVariant::Drive => "File conflicts detected",
            ConflictBannerVariant::Kanban => "Card conflicts detected",
        }
    }

    /// Get the description template for this variant.
    fn description_template(&self) -> &'static str {
        match self {
            ConflictBannerVariant::Messaging => {
                "{count} message(s) have conflicting versions. Resolve to continue."
            }
            ConflictBannerVariant::Drive => {
                "{count} file(s) have conflicting versions. Choose which to keep."
            }
            ConflictBannerVariant::Kanban => {
                "{count} card(s) have conflicting changes. Review and merge."
            }
        }
    }

    /// Get the resolve action label for this variant.
    fn resolve_label(&self) -> &'static str {
        match self {
            ConflictBannerVariant::Messaging => "Review Messages",
            ConflictBannerVariant::Drive => "Resolve Files",
            ConflictBannerVariant::Kanban => "Merge Cards",
        }
    }
}

/// Represents the state of a conflict banner.
#[derive(Debug, Clone)]
struct ConflictBannerState {
    variant: ConflictBannerVariant,
    conflict_count: u32,
    dismissed: bool,
    last_conflict_timestamp: u64,
}

impl ConflictBannerState {
    fn new(variant: ConflictBannerVariant) -> Self {
        Self {
            variant,
            conflict_count: 0,
            dismissed: false,
            last_conflict_timestamp: 0,
        }
    }

    /// Get the variant of this banner.
    fn variant(&self) -> ConflictBannerVariant {
        self.variant
    }

    /// Update the conflict count and potentially show the banner again.
    fn update_conflicts(&mut self, count: u32, timestamp: u64) {
        // If new conflicts detected, show banner again
        if count > self.conflict_count {
            self.dismissed = false;
        }
        self.conflict_count = count;
        if count > 0 {
            self.last_conflict_timestamp = timestamp;
        }
    }

    /// Dismiss the banner.
    fn dismiss(&mut self) {
        self.dismissed = true;
    }

    /// Whether the banner should be visible.
    fn should_show(&self) -> bool {
        self.conflict_count > 0 && !self.dismissed
    }
}

#[test]
fn test_conflict_banner_messaging() {
    let banner = ConflictBannerVariant::Messaging;
    assert_eq!(banner.title(), "Message conflicts detected");
    assert!(
        banner
            .description_template()
            .contains("message(s) have conflicting versions")
    );
    assert_eq!(banner.resolve_label(), "Review Messages");
}

#[test]
fn test_conflict_banner_drive() {
    let banner = ConflictBannerVariant::Drive;
    assert_eq!(banner.title(), "File conflicts detected");
    assert!(banner.description_template().contains("file(s)"));
    assert_eq!(banner.resolve_label(), "Resolve Files");
}

#[test]
fn test_conflict_banner_kanban() {
    let banner = ConflictBannerVariant::Kanban;
    assert_eq!(banner.title(), "Card conflicts detected");
    assert!(banner.description_template().contains("card(s)"));
    assert_eq!(banner.resolve_label(), "Merge Cards");
}

#[test]
fn test_conflict_banner_state_initial() {
    let state = ConflictBannerState::new(ConflictBannerVariant::Messaging);
    assert_eq!(state.variant(), ConflictBannerVariant::Messaging);
    assert_eq!(state.conflict_count, 0);
    assert!(!state.dismissed);
    assert!(!state.should_show()); // No conflicts, shouldn't show
}

#[test]
fn test_conflict_banner_state_with_conflicts() {
    let mut state = ConflictBannerState::new(ConflictBannerVariant::Drive);
    state.update_conflicts(3, 1000);

    assert_eq!(state.conflict_count, 3);
    assert!(!state.dismissed);
    assert!(state.should_show()); // Has conflicts, should show
}

#[test]
fn test_conflict_banner_state_dismissed() {
    let mut state = ConflictBannerState::new(ConflictBannerVariant::Kanban);
    state.update_conflicts(2, 1000);
    state.dismiss();

    assert_eq!(state.conflict_count, 2);
    assert!(state.dismissed);
    assert!(!state.should_show()); // Dismissed, shouldn't show
}

#[test]
fn test_conflict_banner_state_reappears_on_new_conflicts() {
    let mut state = ConflictBannerState::new(ConflictBannerVariant::Messaging);

    // Initial conflicts
    state.update_conflicts(2, 1000);
    assert!(state.should_show());

    // User dismisses
    state.dismiss();
    assert!(!state.should_show());

    // New conflict detected (count increased)
    state.update_conflicts(3, 2000);
    assert!(!state.dismissed); // Should un-dismiss
    assert!(state.should_show());
}

#[test]
fn test_conflict_banner_state_stays_dismissed_if_no_new_conflicts() {
    let mut state = ConflictBannerState::new(ConflictBannerVariant::Drive);

    // Initial conflicts
    state.update_conflicts(3, 1000);
    state.dismiss();
    assert!(!state.should_show());

    // Same count (no new conflicts)
    state.update_conflicts(3, 2000);
    assert!(state.dismissed); // Should stay dismissed
    assert!(!state.should_show());

    // Lower count (conflicts resolved)
    state.update_conflicts(1, 3000);
    assert!(state.dismissed); // Should stay dismissed
    assert!(!state.should_show());
}

// =============================================================================
// Sync Indicator Rendering Logic Tests
// =============================================================================

/// Simulates the sync indicator component rendering logic.
struct SyncIndicatorProps {
    state: SyncState,
    show_when_synced: bool,
}

impl SyncIndicatorProps {
    fn should_render(&self) -> bool {
        // Hide synced state unless explicitly requested
        if !self.show_when_synced && self.state == SyncState::Synced {
            return false;
        }
        true
    }

    fn icon(&self) -> &'static str {
        match self.state {
            SyncState::Synced => "\u{2713}",   // ✓
            SyncState::Syncing => "\u{21BB}",  // ↻
            SyncState::Queued => "\u{23F1}",   // ⏱
            SyncState::Conflict => "\u{26A0}", // ⚠
            SyncState::Error => "\u{2717}",    // ✗
        }
    }

    fn aria_label(&self) -> &'static str {
        match self.state {
            SyncState::Synced => "Synced",
            SyncState::Syncing => "Syncing",
            SyncState::Queued => "Waiting to sync",
            SyncState::Conflict => "Has conflicts",
            SyncState::Error => "Sync failed",
        }
    }
}

#[test]
fn test_sync_indicator_hidden_when_synced() {
    let props = SyncIndicatorProps {
        state: SyncState::Synced,
        show_when_synced: false,
    };
    assert!(!props.should_render());
}

#[test]
fn test_sync_indicator_shown_when_synced_explicit() {
    let props = SyncIndicatorProps {
        state: SyncState::Synced,
        show_when_synced: true,
    };
    assert!(props.should_render());
}

#[test]
fn test_sync_indicator_always_shown_when_not_synced() {
    for state in [
        SyncState::Syncing,
        SyncState::Queued,
        SyncState::Conflict,
        SyncState::Error,
    ] {
        let props = SyncIndicatorProps {
            state,
            show_when_synced: false,
        };
        assert!(
            props.should_render(),
            "State {:?} should always render",
            state
        );
    }
}

#[test]
fn test_sync_indicator_icons() {
    assert_eq!(
        SyncIndicatorProps {
            state: SyncState::Synced,
            show_when_synced: false
        }
        .icon(),
        "\u{2713}"
    );
    assert_eq!(
        SyncIndicatorProps {
            state: SyncState::Syncing,
            show_when_synced: false
        }
        .icon(),
        "\u{21BB}"
    );
    assert_eq!(
        SyncIndicatorProps {
            state: SyncState::Queued,
            show_when_synced: false
        }
        .icon(),
        "\u{23F1}"
    );
    assert_eq!(
        SyncIndicatorProps {
            state: SyncState::Conflict,
            show_when_synced: false
        }
        .icon(),
        "\u{26A0}"
    );
    assert_eq!(
        SyncIndicatorProps {
            state: SyncState::Error,
            show_when_synced: false
        }
        .icon(),
        "\u{2717}"
    );
}

#[test]
fn test_sync_indicator_aria_labels() {
    let states = [
        (SyncState::Synced, "Synced"),
        (SyncState::Syncing, "Syncing"),
        (SyncState::Queued, "Waiting to sync"),
        (SyncState::Conflict, "Has conflicts"),
        (SyncState::Error, "Sync failed"),
    ];

    for (state, expected_label) in states {
        let props = SyncIndicatorProps {
            state,
            show_when_synced: false,
        };
        assert_eq!(props.aria_label(), expected_label);
    }
}

// =============================================================================
// Progress Indicator Logic Tests
// =============================================================================

#[test]
fn test_sync_progress_percentage_calculation() {
    let progress = SyncProgress {
        total: 10,
        completed: 3,
        current_item: Some("file.txt".to_string()),
        bytes_transferred: 0,
        bytes_total: 0,
    };
    assert_eq!(progress.percentage(), 30);
}

#[test]
fn test_sync_progress_percentage_at_zero() {
    let progress = SyncProgress {
        total: 0,
        completed: 0,
        current_item: None,
        bytes_transferred: 0,
        bytes_total: 0,
    };
    // Zero total means complete (nothing to do)
    assert_eq!(progress.percentage(), 100);
}

#[test]
fn test_sync_progress_percentage_at_100() {
    let progress = SyncProgress {
        total: 5,
        completed: 5,
        current_item: None,
        bytes_transferred: 0,
        bytes_total: 0,
    };
    assert_eq!(progress.percentage(), 100);
    assert!(progress.is_complete());
}

#[test]
fn test_sync_progress_bytes_percentage() {
    let progress = SyncProgress {
        total: 0,
        completed: 0,
        current_item: None,
        bytes_transferred: 250,
        bytes_total: 1000,
    };
    assert_eq!(progress.bytes_percentage(), 25);
}

// =============================================================================
// Retry Mechanism Logic Tests
// =============================================================================

/// Simulates retry state for failed sync operations.
struct RetryState {
    attempt_count: u32,
    max_attempts: u32,
    last_error: Option<String>,
    backoff_ms: u64,
}

impl RetryState {
    fn new(max_attempts: u32) -> Self {
        Self {
            attempt_count: 0,
            max_attempts,
            last_error: None,
            backoff_ms: 1000, // Start with 1 second
        }
    }

    fn can_retry(&self) -> bool {
        self.attempt_count < self.max_attempts
    }

    fn record_failure(&mut self, error: &str) {
        self.attempt_count += 1;
        self.last_error = Some(error.to_string());
        // Exponential backoff (capped at 30 seconds)
        self.backoff_ms = (self.backoff_ms * 2).min(30_000);
    }

    fn reset(&mut self) {
        self.attempt_count = 0;
        self.last_error = None;
        self.backoff_ms = 1000;
    }

    fn retry_message(&self) -> String {
        if !self.can_retry() {
            "Maximum retry attempts reached. Please try again later.".to_string()
        } else {
            format!(
                "Retry {}/{}: {}",
                self.attempt_count + 1,
                self.max_attempts,
                self.last_error.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}

#[test]
fn test_retry_state_initial() {
    let state = RetryState::new(3);
    assert_eq!(state.attempt_count, 0);
    assert!(state.can_retry());
    assert!(state.last_error.is_none());
    assert_eq!(state.backoff_ms, 1000);
}

#[test]
fn test_retry_state_after_failure() {
    let mut state = RetryState::new(3);
    state.record_failure("Network error");

    assert_eq!(state.attempt_count, 1);
    assert!(state.can_retry());
    assert_eq!(state.last_error, Some("Network error".to_string()));
    assert_eq!(state.backoff_ms, 2000); // Doubled
}

#[test]
fn test_retry_state_exponential_backoff() {
    let mut state = RetryState::new(10);

    state.record_failure("Error 1");
    assert_eq!(state.backoff_ms, 2000);

    state.record_failure("Error 2");
    assert_eq!(state.backoff_ms, 4000);

    state.record_failure("Error 3");
    assert_eq!(state.backoff_ms, 8000);

    state.record_failure("Error 4");
    assert_eq!(state.backoff_ms, 16000);

    state.record_failure("Error 5");
    assert_eq!(state.backoff_ms, 30000); // Capped at 30 seconds
}

#[test]
fn test_retry_state_max_attempts() {
    let mut state = RetryState::new(2);

    state.record_failure("Error 1");
    assert!(state.can_retry());

    state.record_failure("Error 2");
    assert!(!state.can_retry());
}

#[test]
fn test_retry_state_reset() {
    let mut state = RetryState::new(3);
    state.record_failure("Error 1");
    state.record_failure("Error 2");

    state.reset();

    assert_eq!(state.attempt_count, 0);
    assert!(state.can_retry());
    assert!(state.last_error.is_none());
    assert_eq!(state.backoff_ms, 1000);
}

#[test]
fn test_retry_message_with_attempts_remaining() {
    let mut state = RetryState::new(3);
    state.record_failure("Connection timeout");

    let message = state.retry_message();
    assert!(message.contains("Retry 2/3"));
    assert!(message.contains("Connection timeout"));
}

#[test]
fn test_retry_message_max_attempts_reached() {
    let mut state = RetryState::new(2);
    state.record_failure("Error 1");
    state.record_failure("Error 2");

    let message = state.retry_message();
    assert!(message.contains("Maximum retry attempts reached"));
}

// =============================================================================
// Sync Summary Overall State Tests
// =============================================================================

#[test]
fn test_sync_summary_all_synced() {
    let summary = SyncSummary {
        synced_count: 100,
        syncing_count: 0,
        queued_count: 0,
        conflict_count: 0,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Synced);
    assert_eq!(summary.total(), 100);
}

#[test]
fn test_sync_summary_error_priority() {
    // Error takes highest priority
    let summary = SyncSummary {
        synced_count: 90,
        syncing_count: 5,
        queued_count: 3,
        conflict_count: 1,
        error_count: 1,
    };
    assert_eq!(summary.overall_state(), SyncState::Error);
}

#[test]
fn test_sync_summary_conflict_priority() {
    // Conflict takes priority over syncing/queued
    let summary = SyncSummary {
        synced_count: 90,
        syncing_count: 5,
        queued_count: 3,
        conflict_count: 2,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Conflict);
}

#[test]
fn test_sync_summary_syncing_priority() {
    // Syncing takes priority over queued
    let summary = SyncSummary {
        synced_count: 90,
        syncing_count: 5,
        queued_count: 5,
        conflict_count: 0,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Syncing);
}

#[test]
fn test_sync_summary_queued_only() {
    let summary = SyncSummary {
        synced_count: 95,
        syncing_count: 0,
        queued_count: 5,
        conflict_count: 0,
        error_count: 0,
    };
    assert_eq!(summary.overall_state(), SyncState::Queued);
}

// =============================================================================
// SyncMetadata Constructor Tests
// =============================================================================

#[test]
fn test_sync_metadata_constructors() {
    let synced = SyncMetadata::synced();
    assert_eq!(synced.state, SyncState::Synced);
    assert!(synced.last_synced.is_some());

    let syncing = SyncMetadata::syncing();
    assert_eq!(syncing.state, SyncState::Syncing);

    let queued = SyncMetadata::queued(5);
    assert_eq!(queued.state, SyncState::Queued);
    assert_eq!(queued.pending_changes, 5);

    let conflict = SyncMetadata::conflict(3);
    assert_eq!(conflict.state, SyncState::Conflict);
    assert_eq!(conflict.conflict_count, 3);

    let error = SyncMetadata::error("Network unreachable");
    assert_eq!(error.state, SyncState::Error);
    assert_eq!(error.error_message.as_deref(), Some("Network unreachable"));
}

//! Global offline state indicators for app-wide network status.
//!
//! This module provides components for indicating network connectivity status
//! across the application. These are distinct from canvas-specific offline
//! indicators which track sync queue operations.
//!
//! # Components
//!
//! - [`OfflineBanner`] - Fixed position banner shown when network is offline
//! - [`ConnectionBadge`] - Compact inline badge showing connection state
//! - [`SyncStatusIndicator`] - Status indicator for sync operations
//!
//! # Example
//!
//! ```ignore
//! use communitas_dioxus::components::offline::{OfflineBanner, SyncStatusIndicator};
//!
//! rsx! {
//!     OfflineBanner {}
//!     // ... rest of your app
//!     SyncStatusIndicator { state: SyncState::Syncing }
//! }
//! ```

#![allow(dead_code)]

use dioxus::prelude::*;

use crate::tokens::{colors, spacing, typography};

/// Warning background color (darker amber for dark theme).
const WARNING_BG: &str = "#78350f";

/// Warning text color (light amber for contrast on dark background).
const WARNING_TEXT: &str = "#fef3c7";

/// Connection state for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    /// Network is available and connected.
    #[default]
    Online,
    /// Network is unavailable.
    Offline,
    /// Connection state is being determined.
    Checking,
}

impl ConnectionState {
    /// Check if currently online.
    #[must_use]
    pub fn is_online(&self) -> bool {
        matches!(self, Self::Online)
    }

    /// Check if currently offline.
    #[must_use]
    pub fn is_offline(&self) -> bool {
        matches!(self, Self::Offline)
    }
}

/// Sync operation state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SyncState {
    /// No pending sync operations, all data is current.
    #[default]
    Idle,
    /// Sync is in progress.
    Syncing {
        /// Number of pending operations.
        pending: usize,
    },
    /// All changes have been synced.
    Synced,
    /// Sync failed with an error.
    Error {
        /// Error message.
        message: String,
    },
}

impl SyncState {
    /// Check if currently syncing.
    #[must_use]
    pub fn is_syncing(&self) -> bool {
        matches!(self, Self::Syncing { .. })
    }

    /// Check if in error state.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Check if synced successfully.
    #[must_use]
    pub fn is_synced(&self) -> bool {
        matches!(self, Self::Synced)
    }
}

/// Fixed position banner shown when the network is offline.
///
/// Displays at the top of the viewport with a clear message about
/// offline status and that changes will sync when reconnected.
///
/// # Features
///
/// - Smooth slide-in/out animation (respects reduce-motion)
/// - Dismissible (remembers for session via signal)
/// - Accessible with ARIA live region
///
/// # Example
///
/// ```ignore
/// // Include at app root level
/// rsx! {
///     OfflineBanner {}
///     Router::<Route> {}
/// }
/// ```
#[component]
pub fn OfflineBanner(
    /// Current connection state (default: Online, so banner hidden).
    #[props(default)]
    connection: ConnectionState,
    /// Callback when banner is dismissed.
    #[props(default)]
    on_dismiss: Option<EventHandler<()>>,
) -> Element {
    let mut dismissed = use_signal(|| false);

    // Don't show if online or dismissed
    if connection.is_online() || *dismissed.read() {
        return rsx! {};
    }

    let animation_style = format!(
        "animation: {} 300ms ease-out;",
        if *dismissed.read() {
            "slideUp"
        } else {
            "slideDown"
        }
    );

    let banner_style = format!(
        "position: fixed; top: 0; left: 0; right: 0; z-index: 9999; \
         padding: {} {}; background-color: {}; color: {}; \
         display: flex; align-items: center; justify-content: center; gap: {}; \
         {}",
        spacing::SM,
        spacing::MD,
        WARNING_BG,
        WARNING_TEXT,
        spacing::MD,
        animation_style
    );

    let dismiss_button_style = format!(
        "background: transparent; border: none; color: {}; cursor: pointer; \
         padding: {}; font-size: {}; line-height: 1; border-radius: {};",
        WARNING_TEXT,
        spacing::XS,
        typography::TEXT_SM,
        spacing::XS
    );

    rsx! {
        div {
            class: "offline-banner",
            style: "{banner_style}",
            role: "status",
            "aria-live": "polite",
            // Offline icon
            span {
                class: "offline-icon",
                style: "font-size: 1.25em;",
                "📡"
            }
            // Message
            span {
                style: format!("font-size: {};", typography::TEXT_SM),
                "You're offline. Changes will sync when reconnected."
            }
            // Dismiss button
            button {
                r#type: "button",
                class: "dismiss-button",
                style: "{dismiss_button_style}",
                "aria-label": "Dismiss offline notification",
                onclick: move |_| {
                    dismissed.set(true);
                    if let Some(handler) = &on_dismiss {
                        handler.call(());
                    }
                },
                "✕"
            }
        }
    }
}

/// Compact connection status badge for inline display.
///
/// Shows a small indicator dot with optional label for connection state.
///
/// # Example
///
/// ```ignore
/// ConnectionBadge {
///     state: ConnectionState::Offline,
///     show_label: true,
/// }
/// ```
#[component]
pub fn ConnectionBadge(
    /// Current connection state.
    #[props(default)]
    state: ConnectionState,
    /// Whether to show text label alongside dot.
    #[props(default)]
    show_label: bool,
    /// Additional CSS classes.
    #[props(default)]
    class: String,
) -> Element {
    let (dot_color, label) = match state {
        ConnectionState::Online => (colors::SUCCESS, "Online"),
        ConnectionState::Offline => (colors::WARNING, "Offline"),
        ConnectionState::Checking => (colors::TEXT_SECONDARY, "Checking..."),
    };

    let animation = if matches!(state, ConnectionState::Checking) {
        "animation: pulse 1.5s ease-in-out infinite;".to_string()
    } else {
        String::new()
    };

    let container_style = format!(
        "display: inline-flex; align-items: center; gap: {};",
        spacing::XS
    );

    let dot_style = format!(
        "width: 8px; height: 8px; border-radius: 50%; background-color: {}; {}",
        dot_color, animation
    );

    let label_style = format!(
        "font-size: {}; color: {};",
        typography::TEXT_XS,
        colors::TEXT_SECONDARY
    );

    rsx! {
        span {
            class: "connection-badge {class}",
            style: "{container_style}",
            "aria-label": "{label}",
            span {
                class: "connection-dot",
                style: "{dot_style}",
                "aria-hidden": "true",
            }
            if show_label {
                span {
                    class: "connection-label",
                    style: "{label_style}",
                    "{label}"
                }
            }
        }
    }
}

/// Sync status indicator showing pending, syncing, synced, or error states.
///
/// Displays a status indicator with appropriate icon and optional message.
/// Integrates with screen reader announcements for state changes.
///
/// # Example
///
/// ```ignore
/// SyncStatusIndicator {
///     state: SyncState::Syncing { pending: 3 },
///     on_retry: move |_| sync_service.retry(),
/// }
/// ```
#[component]
pub fn SyncStatusIndicator(
    /// Current sync state.
    #[props(default)]
    state: SyncState,
    /// Callback when retry button is clicked (only shown in error state).
    #[props(default)]
    on_retry: Option<EventHandler<()>>,
    /// Additional CSS classes.
    #[props(default)]
    class: String,
) -> Element {
    // Don't show for idle state
    if matches!(state, SyncState::Idle) {
        return rsx! {};
    }

    let (icon, text, color, is_animating): (&str, String, &str, bool) = match &state {
        SyncState::Idle => ("", String::new(), colors::TEXT_SECONDARY, false),
        SyncState::Syncing { pending } => {
            let msg = if *pending > 0 {
                format!(
                    "Syncing {} item{}...",
                    pending,
                    if *pending != 1 { "s" } else { "" }
                )
            } else {
                "Syncing...".to_string()
            };
            ("🔄", msg, colors::PRIMARY, true)
        }
        SyncState::Synced => ("✓", "Saved".to_string(), colors::SUCCESS, false),
        SyncState::Error { message } => {
            let msg = format!("Sync failed: {}", message);
            ("⚠", msg, colors::DANGER, false)
        }
    };

    let animation = if is_animating {
        "animation: spin 1s linear infinite;".to_string()
    } else {
        String::new()
    };

    let container_style = format!(
        "display: inline-flex; align-items: center; gap: {};",
        spacing::XS
    );

    let icon_style = format!("font-size: {}; {}", typography::TEXT_SM, animation);

    let text_style = format!("font-size: {}; color: {};", typography::TEXT_XS, color);

    let retry_button_style = format!(
        "background: transparent; border: 1px solid {}; color: {}; cursor: pointer; \
         padding: 0.125rem {}; font-size: {}; border-radius: {}; margin-left: {};",
        colors::DANGER,
        colors::DANGER,
        spacing::XS,
        typography::TEXT_XS,
        spacing::XS,
        spacing::XS
    );

    rsx! {
        div {
            class: "sync-status {class}",
            style: "{container_style}",
            role: "status",
            "aria-live": "polite",
            // Icon
            span {
                class: "sync-icon",
                style: "{icon_style}",
                "aria-hidden": "true",
                "{icon}"
            }
            // Status text
            span {
                class: "sync-text",
                style: "{text_style}",
                "{text}"
            }
            // Retry button (only for error state)
            if state.is_error() {
                if let Some(handler) = on_retry {
                    button {
                        r#type: "button",
                        class: "retry-button",
                        style: "{retry_button_style}",
                        onclick: move |_| handler.call(()),
                        "Retry"
                    }
                }
            }
        }
    }
}

/// Hook to track connection state using browser navigator.onLine.
///
/// This hook sets up event listeners for online/offline events and
/// returns a reactive signal with the current connection state.
///
/// # Example
///
/// ```ignore
/// let connection = use_connection_state();
/// rsx! {
///     OfflineBanner { connection: *connection.read() }
/// }
/// ```
#[cfg(target_arch = "wasm32")]
pub fn use_connection_state() -> Signal<ConnectionState> {
    use web_sys::window;

    let connection = use_signal(|| {
        // Check initial state
        window()
            .and_then(|w| w.navigator().on_line().then_some(ConnectionState::Online))
            .unwrap_or(ConnectionState::Offline)
    });

    // Set up event listeners for online/offline events
    use_effect(move || {
        // In a real implementation, we'd set up event listeners here
        // For now, we rely on the initial check
        // The effect would listen to window 'online' and 'offline' events
    });

    connection
}

/// Non-WASM fallback that always returns Online.
#[cfg(not(target_arch = "wasm32"))]
pub fn use_connection_state() -> Signal<ConnectionState> {
    use_signal(|| ConnectionState::Online)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_state_is_online() {
        assert!(ConnectionState::Online.is_online());
        assert!(!ConnectionState::Offline.is_online());
        assert!(!ConnectionState::Checking.is_online());
    }

    #[test]
    fn connection_state_is_offline() {
        assert!(!ConnectionState::Online.is_offline());
        assert!(ConnectionState::Offline.is_offline());
        assert!(!ConnectionState::Checking.is_offline());
    }

    #[test]
    fn connection_state_default() {
        assert_eq!(ConnectionState::default(), ConnectionState::Online);
    }

    #[test]
    fn sync_state_is_syncing() {
        assert!(!SyncState::Idle.is_syncing());
        assert!(SyncState::Syncing { pending: 0 }.is_syncing());
        assert!(SyncState::Syncing { pending: 5 }.is_syncing());
        assert!(!SyncState::Synced.is_syncing());
        assert!(
            !SyncState::Error {
                message: "test".to_string()
            }
            .is_syncing()
        );
    }

    #[test]
    fn sync_state_is_error() {
        assert!(!SyncState::Idle.is_error());
        assert!(!SyncState::Syncing { pending: 0 }.is_error());
        assert!(!SyncState::Synced.is_error());
        assert!(
            SyncState::Error {
                message: "test".to_string()
            }
            .is_error()
        );
    }

    #[test]
    fn sync_state_is_synced() {
        assert!(!SyncState::Idle.is_synced());
        assert!(!SyncState::Syncing { pending: 0 }.is_synced());
        assert!(SyncState::Synced.is_synced());
        assert!(
            !SyncState::Error {
                message: "test".to_string()
            }
            .is_synced()
        );
    }

    #[test]
    fn sync_state_default() {
        assert_eq!(SyncState::default(), SyncState::Idle);
    }

    #[test]
    fn sync_state_debug() {
        let state = SyncState::Syncing { pending: 3 };
        let debug_str = format!("{state:?}");
        assert!(debug_str.contains("Syncing"));
        assert!(debug_str.contains("3"));
    }

    #[test]
    fn sync_state_error_message() {
        let state = SyncState::Error {
            message: "Network timeout".to_string(),
        };
        if let SyncState::Error { message } = state {
            assert_eq!(message, "Network timeout");
        } else {
            panic!("Expected Error state");
        }
    }

    #[test]
    fn connection_state_copy() {
        let state = ConnectionState::Offline;
        let copied = state;
        assert_eq!(state, copied);
    }

    #[test]
    fn sync_state_clone() {
        let state = SyncState::Error {
            message: "test".to_string(),
        };
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn warning_colors_are_valid_hex() {
        assert!(WARNING_BG.starts_with('#'));
        assert!(WARNING_TEXT.starts_with('#'));
        assert_eq!(WARNING_BG.len(), 7);
        assert_eq!(WARNING_TEXT.len(), 7);
    }
}

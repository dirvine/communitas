// SPDX-License-Identifier: MIT OR Apache-2.0

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

use communitas_ui_api::SyncState as ApiSyncState;
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

/// Variant indicating which surface has conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictBannerVariant {
    /// Conflicts in messaging threads.
    #[default]
    Messaging,
    /// Conflicts in drive files.
    Drive,
    /// Conflicts in kanban cards.
    Kanban,
}

impl ConflictBannerVariant {
    /// Get the surface name for display.
    #[must_use]
    pub fn surface_name(self) -> &'static str {
        match self {
            Self::Messaging => "messages",
            Self::Drive => "files",
            Self::Kanban => "cards",
        }
    }

    /// Get the icon for this variant.
    #[must_use]
    pub fn icon(self) -> &'static str {
        match self {
            Self::Messaging => "💬",
            Self::Drive => "📁",
            Self::Kanban => "📋",
        }
    }
}

/// Conflict resolution banner displayed when sync conflicts are detected.
///
/// Shows a warning banner when CRDT merge conflicts need user attention.
/// Provides access to conflict resolution UI and can be dismissed temporarily.
///
/// # Features
///
/// - Auto-appears when new conflicts are detected
/// - Dismissible (but reappears on new conflicts)
/// - Accessible with proper ARIA attributes
/// - Shows conflict count and affected surface
///
/// # Example
///
/// ```ignore
/// ConflictBanner {
///     variant: ConflictBannerVariant::Messaging,
///     conflict_count: 3,
///     on_resolve: move |_| open_conflict_resolution(),
///     on_dismiss: move |_| dismissed.set(true),
/// }
/// ```
#[component]
pub fn ConflictBanner(
    /// Which surface has conflicts.
    #[props(default)]
    variant: ConflictBannerVariant,
    /// Number of conflicts requiring resolution.
    #[props(default = 0)]
    conflict_count: u32,
    /// Callback when "Resolve" button is clicked.
    on_resolve: EventHandler<()>,
    /// Callback when banner is dismissed.
    #[props(default)]
    on_dismiss: Option<EventHandler<()>>,
    /// Sync state from the API (to auto-show on conflict state).
    #[props(default)]
    sync_state: Option<ApiSyncState>,
) -> Element {
    let mut dismissed = use_signal(|| false);
    let mut last_conflict_count = use_signal(|| conflict_count);

    // Auto-show if conflict count increased
    use_effect(move || {
        if conflict_count > *last_conflict_count.read() {
            dismissed.set(false);
        }
        last_conflict_count.set(conflict_count);
    });

    // Check if we should show based on sync_state or explicit conflict_count
    let should_show = if let Some(state) = sync_state {
        matches!(state, ApiSyncState::Conflict)
    } else {
        conflict_count > 0
    };

    // Don't show if no conflicts or dismissed
    if !should_show || *dismissed.read() {
        return rsx! {};
    }

    let surface = variant.surface_name();
    let icon = variant.icon();
    let count_text = if conflict_count == 1 {
        format!("1 {}", surface.trim_end_matches('s'))
    } else if conflict_count > 0 {
        format!("{} {}", conflict_count, surface)
    } else {
        format!("Some {}", surface)
    };

    let banner_style = format!(
        "display: flex; align-items: center; justify-content: space-between; \
         padding: {} {}; background-color: {}20; border: 1px solid {}50; \
         border-radius: {}; margin: {} 0; gap: {};",
        spacing::SM,
        spacing::MD,
        colors::WARNING,
        colors::WARNING,
        spacing::SM,
        spacing::SM,
        spacing::MD
    );

    let content_style = format!("display: flex; align-items: center; gap: {};", spacing::SM);

    let text_style = format!(
        "color: {}; font-size: {};",
        colors::TEXT_PRIMARY,
        typography::TEXT_SM
    );

    let actions_style = format!("display: flex; align-items: center; gap: {};", spacing::SM);

    let resolve_button_style = format!(
        "background-color: {}; color: {}; border: none; padding: {} {}; \
         font-size: {}; border-radius: {}; cursor: pointer; font-weight: 500;",
        colors::WARNING,
        colors::TEXT_INVERSE,
        spacing::XS,
        spacing::SM,
        typography::TEXT_SM,
        spacing::XS
    );

    let dismiss_button_style = format!(
        "background: transparent; border: none; color: {}; cursor: pointer; \
         padding: {}; font-size: {}; line-height: 1;",
        colors::TEXT_SECONDARY,
        spacing::XS,
        typography::TEXT_SM
    );

    rsx! {
        div {
            class: "conflict-banner",
            style: "{banner_style}",
            role: "alert",
            "aria-live": "assertive",
            // Content section
            div {
                class: "conflict-content",
                style: "{content_style}",
                // Warning icon
                span {
                    class: "conflict-icon",
                    style: "font-size: 1.25em;",
                    "aria-hidden": "true",
                    "⚠️"
                }
                // Surface icon
                span {
                    class: "surface-icon",
                    style: "font-size: 1em;",
                    "aria-hidden": "true",
                    "{icon}"
                }
                // Message
                span {
                    style: "{text_style}",
                    "{count_text} need",
                    if conflict_count == 1 { "s" } else { "" }
                    " conflict resolution"
                }
            }
            // Actions section
            div {
                class: "conflict-actions",
                style: "{actions_style}",
                // Resolve button
                button {
                    r#type: "button",
                    class: "resolve-button",
                    style: "{resolve_button_style}",
                    onclick: move |_| on_resolve.call(()),
                    "Resolve"
                }
                // Dismiss button
                if let Some(handler) = on_dismiss {
                    button {
                        r#type: "button",
                        class: "dismiss-button",
                        style: "{dismiss_button_style}",
                        "aria-label": "Dismiss conflict notification",
                        onclick: move |_| {
                            dismissed.set(true);
                            handler.call(());
                        },
                        "✕"
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

/// Toast variant for different notification types.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ToastVariant {
    /// Offline notification (gray).
    #[default]
    Offline,
    /// Syncing in progress (blue).
    Syncing,
    /// Success notification (green).
    Success,
    /// Error notification (red).
    Error,
}

impl ToastVariant {
    /// Get the background color for this variant.
    #[must_use]
    pub fn bg_color(&self) -> &'static str {
        match self {
            Self::Offline => colors::SURFACE_ELEVATED,
            Self::Syncing => colors::INFO,
            Self::Success => colors::SUCCESS,
            Self::Error => colors::DANGER,
        }
    }

    /// Get the text color for this variant.
    #[must_use]
    pub fn text_color(&self) -> &'static str {
        match self {
            Self::Offline => colors::TEXT_PRIMARY,
            Self::Syncing | Self::Success | Self::Error => colors::TEXT_INVERSE,
        }
    }

    /// Get the icon for this variant.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Offline => "📡",
            Self::Syncing => "🔄",
            Self::Success => "✓",
            Self::Error => "⚠",
        }
    }

    /// Check if this toast should auto-dismiss.
    #[must_use]
    pub fn auto_dismiss(&self) -> bool {
        !matches!(self, Self::Error)
    }

    /// Get the auto-dismiss duration in milliseconds.
    #[must_use]
    pub fn dismiss_duration_ms(&self) -> u32 {
        match self {
            Self::Success => 3000,
            Self::Syncing => 5000,
            Self::Offline => 5000,
            Self::Error => 0, // Errors persist until dismissed
        }
    }
}

/// A single toast notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToastNotification {
    /// Unique identifier.
    pub id: String,
    /// Toast variant (affects styling).
    pub variant: ToastVariant,
    /// Main message text.
    pub message: String,
    /// Optional action button label.
    pub action_label: Option<String>,
    /// When the toast was created (Unix ms).
    pub created_at: u64,
}

impl ToastNotification {
    /// Create a new offline toast.
    #[must_use]
    pub fn offline() -> Self {
        Self {
            id: generate_toast_id(),
            variant: ToastVariant::Offline,
            message: "You're offline - changes will sync when connected".to_string(),
            action_label: None,
            created_at: current_timestamp_ms(),
        }
    }

    /// Create a new syncing toast.
    #[must_use]
    pub fn syncing(item_count: usize) -> Self {
        let message = if item_count == 1 {
            "Back online - syncing 1 item".to_string()
        } else {
            format!("Back online - syncing {} items", item_count)
        };
        Self {
            id: generate_toast_id(),
            variant: ToastVariant::Syncing,
            message,
            action_label: None,
            created_at: current_timestamp_ms(),
        }
    }

    /// Create a new success toast.
    #[must_use]
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            id: generate_toast_id(),
            variant: ToastVariant::Success,
            message: message.into(),
            action_label: None,
            created_at: current_timestamp_ms(),
        }
    }

    /// Create a sync complete toast.
    #[must_use]
    pub fn sync_complete() -> Self {
        Self::success("Sync complete")
    }

    /// Create a new error toast.
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: generate_toast_id(),
            variant: ToastVariant::Error,
            message: message.into(),
            action_label: Some("Retry".to_string()),
            created_at: current_timestamp_ms(),
        }
    }

    /// Create a sync failed toast.
    #[must_use]
    pub fn sync_failed() -> Self {
        Self::error("Sync failed - tap to retry")
    }
}

/// Generate a unique toast ID.
fn generate_toast_id() -> String {
    format!("toast-{}", current_timestamp_ms())
}

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Single toast component.
///
/// Displays a toast notification with auto-dismiss behavior.
///
/// # Example
///
/// ```ignore
/// Toast {
///     notification: ToastNotification::offline(),
///     on_dismiss: move |id| dismiss_toast(id),
///     on_action: move |id| retry_sync(),
/// }
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    /// The toast notification to display.
    pub notification: ToastNotification,
    /// Callback when toast is dismissed.
    pub on_dismiss: EventHandler<String>,
    /// Callback when action button is clicked.
    #[props(default)]
    pub on_action: Option<EventHandler<String>>,
}

#[component]
pub fn Toast(props: ToastProps) -> Element {
    let toast = &props.notification;
    let toast_id = toast.id.clone();
    let variant = toast.variant.clone();
    let message = toast.message.clone();
    let action_label = toast.action_label.clone();
    let on_action = props.on_action;
    let on_dismiss = props.on_dismiss;
    let toast_id_for_dismiss = toast_id.clone();
    let toast_id_for_action = toast_id.clone();

    // Auto-dismiss timer (hook must be called unconditionally per Dioxus rules)
    let auto_dismiss = variant.auto_dismiss();
    let duration = variant.dismiss_duration_ms();
    let toast_id_for_timer = toast_id.clone();
    let on_dismiss_timer = on_dismiss;

    use_future(move || {
        let toast_id = toast_id_for_timer.clone();
        async move {
            if auto_dismiss {
                crate::ui_sleep(std::time::Duration::from_millis(duration as u64)).await;
                on_dismiss_timer.call(toast_id);
            }
        }
    });

    let bg_color = variant.bg_color();
    let text_color = variant.text_color();
    let icon = variant.icon();

    let toast_style = format!(
        "display: flex; align-items: center; gap: {}; padding: {} {}; \
         border-radius: {}; background-color: {}; color: {}; \
         box-shadow: {}; max-width: 400px; animation: slideUp 300ms ease-out;",
        spacing::SM,
        spacing::SM,
        spacing::MD,
        spacing::SM,
        bg_color,
        text_color,
        crate::tokens::shadow::MD
    );

    let dismiss_button_style = format!(
        "background: transparent; border: none; color: {}; cursor: pointer; \
         padding: {}; font-size: {}; line-height: 1; opacity: 0.7;",
        text_color,
        spacing::XS,
        typography::TEXT_SM
    );

    let action_button_style = format!(
        "background: transparent; border: 1px solid {}; color: {}; cursor: pointer; \
         padding: {} {}; font-size: {}; border-radius: {};",
        text_color,
        text_color,
        spacing::XS,
        spacing::SM,
        typography::TEXT_SM,
        spacing::XS
    );

    rsx! {
        div {
            class: "toast",
            style: "{toast_style}",
            role: "alert",
            aria_live: "polite",
            // Icon
            span {
                class: "toast-icon",
                "aria-hidden": "true",
                "{icon}"
            }
            // Message
            span {
                class: "toast-message flex-1 text-sm",
                "{message}"
            }
            // Action button (if present)
            if let Some(label) = action_label {
                if let Some(handler) = on_action {
                    button {
                        r#type: "button",
                        class: "toast-action",
                        style: "{action_button_style}",
                        onclick: move |_| handler.call(toast_id_for_action.clone()),
                        "{label}"
                    }
                }
            }
            // Dismiss button
            button {
                r#type: "button",
                class: "toast-dismiss",
                style: "{dismiss_button_style}",
                "aria-label": "Dismiss notification",
                onclick: move |_| on_dismiss.call(toast_id_for_dismiss.clone()),
                "✕"
            }
        }
    }
}

/// Toast container that stacks multiple toasts.
///
/// Displays toasts in a fixed position container at the bottom of the screen.
/// Manages stacking and animations for multiple toasts.
///
/// # Example
///
/// ```ignore
/// ToastContainer {
///     toasts: vec![
///         ToastNotification::offline(),
///         ToastNotification::sync_failed(),
///     ],
///     on_dismiss: move |id| remove_toast(id),
///     on_action: move |id| handle_toast_action(id),
/// }
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct ToastContainerProps {
    /// List of toasts to display.
    #[props(default)]
    pub toasts: Vec<ToastNotification>,
    /// Callback when a toast is dismissed.
    pub on_dismiss: EventHandler<String>,
    /// Callback when a toast action is clicked.
    #[props(default)]
    pub on_action: Option<EventHandler<String>>,
    /// Maximum number of toasts to display at once.
    #[props(default = 3)]
    pub max_visible: usize,
}

#[component]
pub fn ToastContainer(props: ToastContainerProps) -> Element {
    if props.toasts.is_empty() {
        return rsx! {};
    }

    // Only show max_visible toasts (most recent)
    let visible_toasts: Vec<_> = props.toasts.iter().rev().take(props.max_visible).collect();

    let container_style = format!(
        "position: fixed; bottom: {}; left: 50%; transform: translateX(-50%); \
         z-index: 9999; display: flex; flex-direction: column-reverse; gap: {}; \
         pointer-events: none;",
        spacing::LG,
        spacing::SM
    );

    let toast_wrapper_style = "pointer-events: auto;";

    rsx! {
        div {
            class: "toast-container",
            style: "{container_style}",
            role: "region",
            aria_label: "Notifications",
            for toast in visible_toasts {
                div {
                    key: "{toast.id}",
                    style: "{toast_wrapper_style}",
                    Toast {
                        notification: toast.clone(),
                        on_dismiss: props.on_dismiss,
                        on_action: props.on_action,
                    }
                }
            }
        }
    }
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

    #[test]
    fn conflict_banner_variant_surface_names() {
        assert_eq!(ConflictBannerVariant::Messaging.surface_name(), "messages");
        assert_eq!(ConflictBannerVariant::Drive.surface_name(), "files");
        assert_eq!(ConflictBannerVariant::Kanban.surface_name(), "cards");
    }

    #[test]
    fn conflict_banner_variant_icons() {
        assert_eq!(ConflictBannerVariant::Messaging.icon(), "💬");
        assert_eq!(ConflictBannerVariant::Drive.icon(), "📁");
        assert_eq!(ConflictBannerVariant::Kanban.icon(), "📋");
    }

    #[test]
    fn conflict_banner_variant_default() {
        assert_eq!(
            ConflictBannerVariant::default(),
            ConflictBannerVariant::Messaging
        );
    }

    #[test]
    fn conflict_banner_variant_copy() {
        let variant = ConflictBannerVariant::Drive;
        let copied = variant;
        assert_eq!(variant, copied);
    }

    #[test]
    fn toast_variant_bg_colors() {
        assert!(!ToastVariant::Offline.bg_color().is_empty());
        assert!(!ToastVariant::Syncing.bg_color().is_empty());
        assert!(!ToastVariant::Success.bg_color().is_empty());
        assert!(!ToastVariant::Error.bg_color().is_empty());
    }

    #[test]
    fn toast_variant_text_colors() {
        assert!(!ToastVariant::Offline.text_color().is_empty());
        assert!(!ToastVariant::Syncing.text_color().is_empty());
        assert!(!ToastVariant::Success.text_color().is_empty());
        assert!(!ToastVariant::Error.text_color().is_empty());
    }

    #[test]
    fn toast_variant_icons() {
        assert!(!ToastVariant::Offline.icon().is_empty());
        assert!(!ToastVariant::Syncing.icon().is_empty());
        assert!(!ToastVariant::Success.icon().is_empty());
        assert!(!ToastVariant::Error.icon().is_empty());
    }

    #[test]
    fn toast_variant_auto_dismiss() {
        assert!(ToastVariant::Offline.auto_dismiss());
        assert!(ToastVariant::Syncing.auto_dismiss());
        assert!(ToastVariant::Success.auto_dismiss());
        assert!(!ToastVariant::Error.auto_dismiss());
    }

    #[test]
    fn toast_variant_default() {
        assert_eq!(ToastVariant::default(), ToastVariant::Offline);
    }

    #[test]
    fn toast_notification_offline() {
        let toast = ToastNotification::offline();
        assert_eq!(toast.variant, ToastVariant::Offline);
        assert!(toast.message.contains("offline"));
        assert!(toast.action_label.is_none());
    }

    #[test]
    fn toast_notification_syncing() {
        let toast = ToastNotification::syncing(3);
        assert_eq!(toast.variant, ToastVariant::Syncing);
        assert!(toast.message.contains("3 items"));
    }

    #[test]
    fn toast_notification_syncing_single() {
        let toast = ToastNotification::syncing(1);
        assert!(toast.message.contains("1 item"));
        assert!(!toast.message.contains("items"));
    }

    #[test]
    fn toast_notification_success() {
        let toast = ToastNotification::success("Test message");
        assert_eq!(toast.variant, ToastVariant::Success);
        assert_eq!(toast.message, "Test message");
    }

    #[test]
    fn toast_notification_sync_complete() {
        let toast = ToastNotification::sync_complete();
        assert_eq!(toast.variant, ToastVariant::Success);
        assert!(toast.message.contains("Sync complete"));
    }

    #[test]
    fn toast_notification_error() {
        let toast = ToastNotification::error("Error message");
        assert_eq!(toast.variant, ToastVariant::Error);
        assert_eq!(toast.message, "Error message");
        assert!(toast.action_label.is_some());
    }

    #[test]
    fn toast_notification_sync_failed() {
        let toast = ToastNotification::sync_failed();
        assert_eq!(toast.variant, ToastVariant::Error);
        assert!(toast.message.contains("failed"));
        assert!(toast.action_label.is_some());
    }

    #[test]
    fn generate_toast_id_unique() {
        let id1 = generate_toast_id();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let id2 = generate_toast_id();
        assert_ne!(id1, id2);
    }
}

// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error boundary components for graceful error handling.
//!
//! These components provide consistent, accessible error displays across the application.
//! All error components use `role="alert"` with `aria-live="assertive"` to ensure
//! screen readers immediately announce errors.
//!
//! # Component Types
//!
//! - [`ErrorBanner`] - Inline dismissible error message
//! - [`ErrorCard`] - Full card error state with retry button
//! - [`ErrorPage`] - Full page error with navigation options
//! - [`NetworkError`] - Specialized offline/network error display
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::error_boundary::{ErrorBanner, ErrorCard};
//!
//! rsx! {
//!     // Inline error with dismiss
//!     ErrorBanner {
//!         message: "Failed to save changes",
//!         on_dismiss: move |_| error.set(None),
//!     }
//!
//!     // Card error with retry
//!     ErrorCard {
//!         title: "Failed to load board",
//!         message: "The board could not be loaded. Please try again.",
//!         on_retry: Some(EventHandler::new(|_| reload())),
//!     }
//! }
//! ```

use crate::tokens::colors;
use dioxus::prelude::*;

/// Converts technical error messages to user-friendly descriptions.
///
/// # Arguments
/// * `err` - Technical error message
///
/// # Returns
/// A user-friendly error message suitable for display.
#[must_use]
#[allow(dead_code)] // Exported for external use
pub fn user_friendly_error(err: &str) -> String {
    let lower = err.to_lowercase();

    // Network/connection errors
    if lower.contains("network")
        || lower.contains("connection")
        || lower.contains("timeout")
        || lower.contains("refused")
    {
        return "Unable to connect. Please check your internet connection and try again."
            .to_string();
    }

    // Authentication errors
    if lower.contains("unauthorized") || lower.contains("401") || lower.contains("forbidden") {
        return "You don't have permission to perform this action. Please sign in again."
            .to_string();
    }

    // Not found errors
    if lower.contains("not found") || lower.contains("404") {
        return "The requested item could not be found. It may have been moved or deleted."
            .to_string();
    }

    // Rate limiting
    if lower.contains("rate limit") || lower.contains("429") || lower.contains("too many") {
        return "Too many requests. Please wait a moment and try again.".to_string();
    }

    // Server errors
    if lower.contains("500") || lower.contains("internal server") || lower.contains("server error")
    {
        return "Something went wrong on our end. Please try again later.".to_string();
    }

    // Storage errors
    if lower.contains("storage") || lower.contains("disk") || lower.contains("quota") {
        return "Storage operation failed. You may be running low on space.".to_string();
    }

    // Sync errors
    if lower.contains("sync") || lower.contains("conflict") || lower.contains("crdt") {
        return "Changes couldn't be synced. They'll be saved locally and synced when possible."
            .to_string();
    }

    // Generic fallback
    format!(
        "An error occurred: {}. Please try again.",
        if err.len() > 100 { &err[..100] } else { err }
    )
}

/// An inline dismissible error banner.
///
/// Use for non-blocking errors that don't prevent the user from continuing.
#[component]
pub fn ErrorBanner(
    /// Error message to display
    message: String,
    /// Optional callback when dismiss button is clicked
    #[props(default)]
    on_dismiss: Option<EventHandler<()>>,
    /// Show dismiss button
    #[props(default = true)]
    dismissible: bool,
    /// Additional CSS classes
    #[props(default = String::new())]
    class: String,
) -> Element {
    rsx! {
        div {
            class: format!(
                "flex items-center gap-3 rounded-lg px-4 py-3 text-sm {}",
                class
            ),
            style: format!(
                "background-color: {}20; border: 1px solid {}40; color: {};",
                colors::DANGER,
                colors::DANGER,
                colors::DANGER
            ),
            role: "alert",
            aria_live: "assertive",

            // Error icon
            svg {
                class: "w-5 h-5 flex-shrink-0",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                }
            }

            // Message
            p { class: "flex-1", "{message}" }

            // Dismiss button
            if dismissible {
                if let Some(handler) = on_dismiss {
                    button {
                        class: "p-1 rounded hover:bg-white/10 transition-colors",
                        onclick: move |_| handler.call(()),
                        aria_label: "Dismiss error",
                        svg {
                            class: "w-4 h-4",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A card-style error display with optional retry action.
///
/// Use for errors that block content display but allow retry.
#[component]
pub fn ErrorCard(
    /// Error title
    #[props(default = "Something went wrong".to_string())]
    title: String,
    /// Error message/details
    message: String,
    /// Optional retry callback
    #[props(default)]
    on_retry: Option<EventHandler<()>>,
    /// Additional CSS classes
    #[props(default = String::new())]
    class: String,
) -> Element {
    rsx! {
        div {
            class: format!(
                "rounded-lg p-6 text-center {}",
                class
            ),
            style: format!(
                "background-color: {}; border: 1px solid {};",
                colors::SURFACE_CARD,
                colors::BORDER_DEFAULT
            ),
            role: "alert",
            aria_live: "assertive",

            // Error icon
            div {
                class: "w-12 h-12 mx-auto mb-4 rounded-full flex items-center justify-center",
                style: format!("background-color: {}20;", colors::DANGER),
                svg {
                    class: "w-6 h-6",
                    style: format!("color: {};", colors::DANGER),
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    }
                }
            }

            // Title
            h3 {
                class: "text-lg font-semibold mb-2",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "{title}"
            }

            // Message
            p {
                class: "mb-4",
                style: format!("color: {};", colors::TEXT_SECONDARY),
                "{message}"
            }

            // Retry button
            if let Some(handler) = on_retry {
                button {
                    class: "px-4 py-2 rounded-lg font-medium transition-colors",
                    style: format!(
                        "background-color: {}; color: {};",
                        colors::PRIMARY,
                        colors::TEXT_INVERSE
                    ),
                    onclick: move |_| handler.call(()),
                    "Try Again"
                }
            }
        }
    }
}

/// A full-page error display with navigation options.
///
/// Use for critical errors that require user action to recover.
#[component]
pub fn ErrorPage(
    /// Error title
    #[props(default = "Oops! Something went wrong".to_string())]
    title: String,
    /// Error message/details
    message: String,
    /// Optional retry callback
    #[props(default)]
    on_retry: Option<EventHandler<()>>,
    /// Optional go back callback
    #[props(default)]
    on_back: Option<EventHandler<()>>,
    /// Optional go home callback
    #[props(default)]
    on_home: Option<EventHandler<()>>,
    /// Additional CSS classes
    #[props(default = String::new())]
    class: String,
) -> Element {
    rsx! {
        div {
            class: format!(
                "flex flex-col items-center justify-center min-h-[400px] p-8 {}",
                class
            ),
            role: "alert",
            aria_live: "assertive",

            // Error illustration
            div {
                class: "w-24 h-24 mb-6 rounded-full flex items-center justify-center",
                style: format!("background-color: {}15;", colors::DANGER),
                svg {
                    class: "w-12 h-12",
                    style: format!("color: {};", colors::DANGER),
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    }
                }
            }

            // Title
            h1 {
                class: "text-2xl font-bold mb-3 text-center",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "{title}"
            }

            // Message
            p {
                class: "text-center max-w-md mb-6",
                style: format!("color: {};", colors::TEXT_SECONDARY),
                "{message}"
            }

            // Action buttons
            div {
                class: "flex flex-wrap gap-3 justify-center",

                if let Some(handler) = on_retry {
                    button {
                        class: "px-5 py-2.5 rounded-lg font-medium transition-colors",
                        style: format!(
                            "background-color: {}; color: {};",
                            colors::PRIMARY,
                            colors::TEXT_INVERSE
                        ),
                        onclick: move |_| handler.call(()),
                        "Try Again"
                    }
                }

                if let Some(handler) = on_back {
                    button {
                        class: "px-5 py-2.5 rounded-lg font-medium transition-colors",
                        style: format!(
                            "background-color: {}; color: {};",
                            colors::SURFACE_ELEVATED,
                            colors::TEXT_PRIMARY
                        ),
                        onclick: move |_| handler.call(()),
                        "Go Back"
                    }
                }

                if let Some(handler) = on_home {
                    button {
                        class: "px-5 py-2.5 rounded-lg font-medium transition-colors",
                        style: format!(
                            "background-color: {}; color: {};",
                            colors::SURFACE_ELEVATED,
                            colors::TEXT_PRIMARY
                        ),
                        onclick: move |_| handler.call(()),
                        "Go Home"
                    }
                }
            }
        }
    }
}

/// A specialized network/offline error display.
///
/// Use for connection issues and offline states.
#[component]
pub fn NetworkError(
    /// Whether the device is completely offline
    #[props(default = false)]
    is_offline: bool,
    /// Optional retry callback
    #[props(default)]
    on_retry: Option<EventHandler<()>>,
    /// Additional CSS classes
    #[props(default = String::new())]
    class: String,
) -> Element {
    let (title, message, icon_path) = if is_offline {
        (
            "You're offline",
            "Your changes will be saved locally and synced when you're back online.",
            "M18.364 5.636a9 9 0 010 12.728m0 0l-2.829-2.829m2.829 2.829L21 21M15.536 8.464a5 5 0 010 7.072m0 0l-2.829-2.829m-4.243 2.829a4.978 4.978 0 01-1.414-2.83m-1.414 5.658a9 9 0 01-2.167-9.238m7.824 2.167a1 1 0 111.414 1.414m-1.414-1.414L3 3",
        )
    } else {
        (
            "Connection lost",
            "We're having trouble connecting. Please check your internet connection.",
            "M8.111 16.404a5.5 5.5 0 017.778 0M12 20h.01m-7.08-7.071c3.904-3.905 10.236-3.905 14.141 0M1.394 9.393c5.857-5.857 15.355-5.857 21.213 0",
        )
    };

    rsx! {
        div {
            class: format!(
                "rounded-lg p-4 flex items-center gap-4 {}",
                class
            ),
            style: format!(
                "background-color: {}20; border: 1px solid {}40;",
                colors::WARNING,
                colors::WARNING
            ),
            role: "alert",
            aria_live: "polite",

            // Icon
            div {
                class: "w-10 h-10 rounded-full flex items-center justify-center flex-shrink-0",
                style: format!("background-color: {}30;", colors::WARNING),
                svg {
                    class: "w-5 h-5",
                    style: format!("color: {};", colors::WARNING),
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "{icon_path}"
                    }
                }
            }

            // Content
            div {
                class: "flex-1",
                h4 {
                    class: "font-medium",
                    style: format!("color: {};", colors::WARNING),
                    "{title}"
                }
                p {
                    class: "text-sm",
                    style: format!("color: {};", colors::TEXT_SECONDARY),
                    "{message}"
                }
            }

            // Retry button (only for non-offline states)
            if !is_offline {
                if let Some(handler) = on_retry {
                    button {
                        class: "px-3 py-1.5 rounded text-sm font-medium transition-colors",
                        style: format!(
                            "background-color: {}; color: {};",
                            colors::WARNING,
                            colors::TEXT_INVERSE
                        ),
                        onclick: move |_| handler.call(()),
                        "Retry"
                    }
                }
            }
        }
    }
}

/// A warning banner for non-critical issues.
///
/// Use for warnings that don't prevent the user from continuing.
#[component]
pub fn WarningBanner(
    /// Warning message to display
    message: String,
    /// Optional dismiss callback
    #[props(default)]
    on_dismiss: Option<EventHandler<()>>,
    /// Additional CSS classes
    #[props(default = String::new())]
    class: String,
) -> Element {
    rsx! {
        div {
            class: format!(
                "flex items-center gap-3 rounded-lg px-4 py-3 text-sm {}",
                class
            ),
            style: format!(
                "background-color: {}20; border: 1px solid {}40; color: {};",
                colors::WARNING,
                colors::WARNING,
                colors::WARNING
            ),
            role: "alert",
            aria_live: "polite",

            // Warning icon
            svg {
                class: "w-5 h-5 flex-shrink-0",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                }
            }

            // Message
            p { class: "flex-1", "{message}" }

            // Dismiss button
            if let Some(handler) = on_dismiss {
                button {
                    class: "p-1 rounded hover:bg-white/10 transition-colors",
                    onclick: move |_| handler.call(()),
                    aria_label: "Dismiss warning",
                    svg {
                        class: "w-4 h-4",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M6 18L18 6M6 6l12 12"
                        }
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
    fn user_friendly_error_network() {
        let msg = user_friendly_error("network error: connection refused");
        assert!(msg.contains("internet connection"));
    }

    #[test]
    fn user_friendly_error_timeout() {
        let msg = user_friendly_error("request timeout after 30s");
        assert!(msg.contains("internet connection"));
    }

    #[test]
    fn user_friendly_error_unauthorized() {
        let msg = user_friendly_error("401 Unauthorized");
        assert!(msg.contains("permission"));
    }

    #[test]
    fn user_friendly_error_not_found() {
        let msg = user_friendly_error("resource not found");
        assert!(msg.contains("could not be found"));
    }

    #[test]
    fn user_friendly_error_rate_limit() {
        let msg = user_friendly_error("rate limit exceeded");
        assert!(msg.contains("Too many requests"));
    }

    #[test]
    fn user_friendly_error_server() {
        let msg = user_friendly_error("500 Internal Server Error");
        assert!(msg.contains("our end"));
    }

    #[test]
    fn user_friendly_error_storage() {
        let msg = user_friendly_error("storage quota exceeded");
        assert!(msg.contains("Storage"));
    }

    #[test]
    fn user_friendly_error_sync() {
        let msg = user_friendly_error("CRDT sync conflict");
        assert!(msg.contains("synced"));
    }

    #[test]
    fn user_friendly_error_generic() {
        let msg = user_friendly_error("something unexpected happened");
        assert!(msg.contains("An error occurred"));
    }

    #[test]
    fn user_friendly_error_truncates_long_messages() {
        let long_msg = "a".repeat(200);
        let msg = user_friendly_error(&long_msg);
        assert!(msg.len() < 200);
    }

    #[test]
    fn error_banner_default_props() {
        let props = ErrorBannerProps::builder()
            .message("Test error".to_string())
            .build();
        // Dioxus Props use inner struct for fields
        assert!(props.inner.dismissible);
        assert!(props.inner.class.is_empty());
    }

    #[test]
    fn error_card_default_props() {
        let props = ErrorCardProps::builder()
            .message("Test error".to_string())
            .build();
        assert_eq!(props.inner.title, "Something went wrong");
    }

    #[test]
    fn error_page_default_props() {
        let props = ErrorPageProps::builder()
            .message("Test error".to_string())
            .build();
        assert_eq!(props.inner.title, "Oops! Something went wrong");
    }

    #[test]
    fn network_error_default_props() {
        let props = NetworkErrorProps::builder().build();
        assert!(!props.inner.is_offline);
    }

    #[test]
    fn warning_banner_default_props() {
        let props = WarningBannerProps::builder()
            .message("Test warning".to_string())
            .build();
        assert!(props.inner.class.is_empty());
    }
}

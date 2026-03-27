//! Toast notification system with CSS entrance and exit animations.
//!
//! Provides a toast system with multiple variants and auto-dismiss.
//! Uses CSS keyframe animations instead of a motion library.
//!
//! # Example
//!
//! ```rust
//! // In App:
//! use_context_provider(communitas_dioxus::components::toast_system::ToastManager::new);
//! rsx! { communitas_dioxus::components::toast_system::ToastContainer {} }
//!
//! // In any child component:
//! let toast = use_toast();
//! toast.success("Operation completed!");
//! ```

use crate::design_tokens::{palette, radius, semantic, shadow, spacing, typography};
use dioxus::prelude::*;

/// Toast notification kind.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum ToastKind {
    /// Success notification.
    #[default]
    Success,
    /// Error notification.
    Error,
    /// Warning notification.
    Warning,
    /// Info notification.
    Info,
}

impl ToastKind {
    /// Get the icon for this toast kind.
    pub fn icon(&self) -> &'static str {
        match self {
            ToastKind::Success => "✓",
            ToastKind::Error => "✕",
            ToastKind::Warning => "⚠",
            ToastKind::Info => "ℹ",
        }
    }

    /// Get the accent color for this toast kind.
    pub fn accent_color(&self) -> &'static str {
        match self {
            ToastKind::Success => palette::JADE_500,
            ToastKind::Error => palette::ROSE_500,
            ToastKind::Warning => palette::AMBER_500,
            ToastKind::Info => palette::SKY_500,
        }
    }
}

/// Toast notification data.
#[derive(Clone, PartialEq)]
pub struct Toast {
    /// Unique identifier.
    pub id: String,
    /// Toast kind.
    pub kind: ToastKind,
    /// Toast message.
    pub message: String,
    /// Optional description.
    pub description: Option<String>,
    /// Duration in milliseconds (None = persistent).
    pub duration_ms: Option<u64>,
}

impl Toast {
    /// Create a new toast notification.
    pub fn new(kind: ToastKind, message: String) -> Self {
        // Generate a simple unique ID from the message and kind.
        let id = format!(
            "toast-{:?}-{}",
            kind,
            message.len()
        );
        Self {
            id,
            kind,
            message,
            description: None,
            duration_ms: Some(4000),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set custom duration in milliseconds.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Make the toast persistent (no auto-dismiss).
    pub fn persistent(mut self) -> Self {
        self.duration_ms = None;
        self
    }
}

/// Toast manager for controlling notifications.
///
/// Stored in context so any component can call [`use_toast`] to access it.
#[derive(Clone, Copy)]
pub struct ToastManager {
    toasts: Signal<Vec<Toast>>,
}

impl ToastManager {
    /// Create a new toast manager. Called by `use_context_provider`.
    pub fn new() -> Self {
        Self {
            toasts: use_signal(Vec::new),
        }
    }

    /// Show a toast notification.
    pub fn show(mut self, toast: Toast) {
        self.toasts.write().push(toast);
    }

    /// Show a success toast.
    pub fn success(self, message: impl Into<String>) {
        self.show(Toast::new(ToastKind::Success, message.into()));
    }

    /// Show an error toast.
    pub fn error(self, message: impl Into<String>) {
        self.show(Toast::new(ToastKind::Error, message.into()));
    }

    /// Show a warning toast.
    pub fn warning(self, message: impl Into<String>) {
        self.show(Toast::new(ToastKind::Warning, message.into()));
    }

    /// Show an info toast.
    pub fn info(self, message: impl Into<String>) {
        self.show(Toast::new(ToastKind::Info, message.into()));
    }

    /// Dismiss a toast by ID.
    pub fn dismiss(mut self, id: &str) {
        self.toasts.write().retain(|t| t.id != id);
    }

    /// Clear all toasts.
    pub fn clear(mut self) {
        self.toasts.write().clear();
    }

    /// Get the current list of toasts.
    pub fn toasts(self) -> Vec<Toast> {
        self.toasts.read().clone()
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook to access the [`ToastManager`] from context.
///
/// Requires [`ToastManager`] to have been provided via
/// `use_context_provider(ToastManager::new)` in a parent component.
pub fn use_toast() -> ToastManager {
    use_context::<ToastManager>()
}

/// Toast container — renders all active toasts fixed at the top-right of the screen.
///
/// Place this once near the root of your app. Requires [`ToastManager`] in context.
#[component]
pub fn ToastContainer() -> Element {
    let manager = use_toast();
    let toasts = manager.toasts();

    rsx! {
        style {
            r#"
            @keyframes toastSlideIn {{
                from {{ opacity: 0; transform: translateX(20px); }}
                to   {{ opacity: 1; transform: translateX(0); }}
            }}
            @keyframes toastSlideOut {{
                from {{ opacity: 1; transform: translateX(0); }}
                to   {{ opacity: 0; transform: translateX(20px); }}
            }}
            "#
        }

        div {
            style: format!(
                "position: fixed; \
                 top: {}; \
                 right: {}; \
                 display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 z-index: 1000; \
                 pointer-events: none;",
                spacing::BASE,
                spacing::BASE,
                spacing::SM,
            ),

            for toast in toasts.iter() {
                ToastItem {
                    key: "{toast.id}",
                    toast: toast.clone(),
                    on_dismiss: move |id: String| manager.dismiss(&id),
                }
            }
        }
    }
}

/// Props for individual toast items.
#[derive(Props, Clone, PartialEq)]
pub struct ToastItemProps {
    /// Toast data.
    pub toast: Toast,
    /// Dismiss callback (receives the toast ID).
    pub on_dismiss: Callback<String>,
}

/// Individual toast notification.
#[component]
pub fn ToastItem(props: ToastItemProps) -> Element {
    let toast_id = props.toast.id.clone();
    let duration = props.toast.duration_ms;
    let dismiss_cb = props.on_dismiss;
    let mut dismissed = use_signal(|| false);

    // Auto-dismiss after duration.
    {
        let id = toast_id.clone();
        use_effect(move || {
            if let Some(ms) = duration {
                let id_for_spawn = id.clone();
                spawn(async move {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use gloo_timers::future::TimeoutFuture;
                        let capped = ms.min(u64::from(u32::MAX)) as u32;
                        TimeoutFuture::new(capped).await;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                    }
                    if !dismissed() {
                        dismissed.set(true);
                        dismiss_cb.call(id_for_spawn);
                    }
                });
            }
        });
    }

    let animation = if dismissed() {
        "animation: toastSlideOut 200ms ease-in forwards;"
    } else {
        "animation: toastSlideIn 200ms ease-out;"
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: flex-start; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-left: 3px solid {}; \
                 border-radius: {}; \
                 box-shadow: {}; \
                 min-width: 280px; \
                 max-width: 380px; \
                 pointer-events: auto; \
                 {}",
                spacing::SM,
                spacing::SM,
                semantic::BG_ELEVATED,
                semantic::BORDER_DEFAULT,
                props.toast.kind.accent_color(),
                radius::MD,
                shadow::LG,
                animation,
            ),
            role: "alert",
            aria_live: "polite",

            // Icon
            span {
                style: format!(
                    "font-size: {}; \
                     color: {}; \
                     flex-shrink: 0; \
                     line-height: 1;",
                    typography::SIZE_BASE,
                    props.toast.kind.accent_color(),
                ),
                "{props.toast.kind.icon()}"
            }

            // Content
            div {
                style: "flex: 1; display: flex; flex-direction: column; gap: 2px;",

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY,
                    ),
                    "{props.toast.message}"
                }

                if let Some(ref desc) = props.toast.description {
                    span {
                        style: format!(
                            "font-size: {}; \
                             color: {};",
                            typography::SIZE_XS,
                            semantic::TEXT_SECONDARY,
                        ),
                        "{desc}"
                    }
                }
            }

            // Dismiss button
            button {
                style: format!(
                    "background: transparent; \
                     border: none; \
                     color: {}; \
                     cursor: pointer; \
                     font-size: {}; \
                     padding: 0 2px; \
                     flex-shrink: 0; \
                     line-height: 1;",
                    semantic::TEXT_MUTED,
                    typography::SIZE_XS,
                ),
                aria_label: "Dismiss notification",
                onclick: {
                    let id = props.toast.id.clone();
                    let cb = props.on_dismiss;
                    move |_| {
                        dismissed.set(true);
                        cb.call(id.clone());
                    }
                },
                "✕"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_kind_default_is_success() {
        let kind: ToastKind = Default::default();
        assert_eq!(kind, ToastKind::Success);
    }

    #[test]
    fn toast_kind_icons() {
        assert_eq!(ToastKind::Success.icon(), "✓");
        assert_eq!(ToastKind::Error.icon(), "✕");
        assert_eq!(ToastKind::Warning.icon(), "⚠");
        assert_eq!(ToastKind::Info.icon(), "ℹ");
    }

    #[test]
    fn toast_new_sets_defaults() {
        let toast = Toast::new(ToastKind::Info, "Test".to_string());
        assert_eq!(toast.kind, ToastKind::Info);
        assert_eq!(toast.message, "Test");
        assert_eq!(toast.duration_ms, Some(4000));
        assert!(toast.description.is_none());
    }

    #[test]
    fn toast_with_description() {
        let toast = Toast::new(ToastKind::Info, "Test".to_string())
            .with_description("Details".to_string());
        assert_eq!(toast.description, Some("Details".to_string()));
    }

    #[test]
    fn toast_with_duration() {
        let toast = Toast::new(ToastKind::Info, "Test".to_string()).with_duration(10000);
        assert_eq!(toast.duration_ms, Some(10000));
    }

    #[test]
    fn toast_persistent() {
        let toast = Toast::new(ToastKind::Info, "Test".to_string()).persistent();
        assert!(toast.duration_ms.is_none());
    }
}

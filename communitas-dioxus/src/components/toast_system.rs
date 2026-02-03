//! Toast notification system with entrance and exit animations.
//!
//! Provides a comprehensive toast system with multiple variants,
//! auto-dismiss, and smooth animations.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::toast_system::{ToastContainer, Toast, ToastKind};
//!
//! // In your app
//! rsx! {
//!     ToastContainer {}
//! }
//!
//! // Show a toast
//! let toast = Toast::new(ToastKind::Success, "Operation completed!".to_string());
//! ```

use dioxus::prelude::*;
use dioxus_motion::prelude::*;
use std::time::Duration;

use crate::animations::springs;
use crate::animations::transitions;
use crate::design_tokens::{palette, radius, semantic, shadow, spacing, typography};

async fn async_delay(duration: Duration) {
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::future::TimeoutFuture;
        let millis = duration.as_millis().min(u128::from(u32::MAX)) as u32;
        TimeoutFuture::new(millis).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        tokio::time::sleep(duration).await;
    }
}

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

    /// Get the background color for this toast kind.
    pub fn bg_color(&self) -> &'static str {
        match self {
            ToastKind::Success => palette::JADE_500,
            ToastKind::Error => palette::ROSE_500,
            ToastKind::Warning => palette::AMBER_500,
            ToastKind::Info => palette::SKY_500,
        }
    }

    /// Get the text color for this toast kind.
    pub fn text_color(&self) -> &'static str {
        "#ffffff"
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
    /// Duration in milliseconds (None for persistent).
    pub duration_ms: Option<u64>,
}

impl Toast {
    /// Create a new toast notification.
    pub fn new(kind: ToastKind, message: String) -> Self {
        Self {
            id: format!(
                "toast-{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ),
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

    /// Set custom duration.
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
#[derive(Clone, Copy)]
pub struct ToastManager {
    toasts: Signal<Vec<Toast>>,
}

impl ToastManager {
    /// Create a new toast manager.
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

    /// Get current toasts.
    pub fn toasts(self) -> Vec<Toast> {
        self.toasts.read().clone()
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider component for toast functionality.
#[component]
pub fn ToastProvider(children: Element) -> Element {
    let manager = ToastManager::new();

    use_context_provider(|| manager);

    rsx! {
        {children}
        ToastContainer {}
    }
}

/// Hook to access the toast manager.
pub fn use_toast() -> ToastManager {
    use_context::<ToastManager>()
}

/// Toast container that displays all active notifications.
#[component]
pub fn ToastContainer() -> Element {
    let manager = use_toast();
    let toasts = manager.toasts();

    rsx! {
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
                spacing::SM
            ),

            for (index, toast) in toasts.iter().enumerate() {
                ToastItem {
                    key: "{toast.id}",
                    toast: toast.clone(),
                    index,
                    on_dismiss: move |id: String| manager.dismiss(&id),
                }
            }
        }
    }
}

/// Properties for individual toast items.
#[derive(Props, Clone, PartialEq)]
pub struct ToastItemProps {
    /// Toast data.
    pub toast: Toast,
    /// Index for stagger animation.
    pub index: usize,
    /// Dismiss callback.
    pub on_dismiss: Callback<String>,
}

/// Individual toast notification with animations.
#[component]
fn ToastItem(props: ToastItemProps) -> Element {
    let mut translate_x = use_motion(100.0f32);
    let mut opacity = use_motion(0.0f32);
    let mut scale = use_motion(0.9f32);
    let progress = use_motion(100.0f32);
    let mut is_dismissing = use_signal(|| false);

    let toast_id = props.toast.id.clone();
    let dismiss_callback = props.on_dismiss;

    // Entrance animation
    use_effect(move || {
        use std::time::Duration;
        let delay_ms = (props.index as f32 * 100.0) as u64;

        translate_x.animate_to(
            0.0,
            AnimationConfig::new(AnimationMode::Spring(springs::toast_enter()))
                .with_delay(Duration::from_millis(delay_ms)),
        );
        opacity.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::toast_enter()))
                .with_delay(Duration::from_millis(delay_ms)),
        );
        scale.animate_to(
            1.0,
            AnimationConfig::new(AnimationMode::Spring(springs::toast_enter()))
                .with_delay(Duration::from_millis(delay_ms)),
        );
    });

    // Auto-dismiss with progress bar
    {
        let duration = props.toast.duration_ms;
        let mut auto_translate_x = translate_x;
        let mut auto_opacity = opacity;
        let mut auto_scale = scale;
        let mut auto_progress = progress;
        let mut auto_is_dismissing = is_dismissing;
        let auto_toast_id = toast_id.clone();
        let auto_dismiss = dismiss_callback;
        use_effect(move || {
            if let Some(duration) = duration {
                auto_progress.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Spring(Spring {
                        stiffness: 50.0,
                        damping: 50.0,
                        mass: 1.0,
                        velocity: 0.0,
                    })),
                );

                let id = auto_toast_id.clone();
                let dismiss_cb = auto_dismiss;
                spawn(async move {
                    async_delay(Duration::from_millis(duration)).await;

                    if !auto_is_dismissing() {
                        auto_is_dismissing.set(true);
                        auto_translate_x.animate_to(
                            100.0,
                            AnimationConfig::new(AnimationMode::Spring(springs::toast_exit())),
                        );
                        auto_opacity.animate_to(
                            0.0,
                            AnimationConfig::new(AnimationMode::Spring(springs::toast_exit())),
                        );
                        auto_scale.animate_to(
                            0.9,
                            AnimationConfig::new(AnimationMode::Spring(springs::toast_exit())),
                        );
                        async_delay(Duration::from_millis(200)).await;
                        dismiss_cb.call(id);
                    }
                });
            }
        });
    }

    let handle_dismiss = {
        let toast_id_for_manual = toast_id.clone();
        let manual_dismiss = dismiss_callback;
        move |_| {
            if !is_dismissing() {
                is_dismissing.set(true);

                // Exit animation
                translate_x.animate_to(
                    100.0,
                    AnimationConfig::new(AnimationMode::Spring(springs::toast_exit())),
                );
                opacity.animate_to(
                    0.0,
                    AnimationConfig::new(AnimationMode::Spring(springs::toast_exit())),
                );
                scale.animate_to(
                    0.9,
                    AnimationConfig::new(AnimationMode::Spring(springs::toast_exit())),
                );

                // Remove after animation
                let id = toast_id_for_manual.clone();
                let on_dismiss = manual_dismiss;
                spawn(async move {
                    async_delay(Duration::from_millis(200)).await;
                    on_dismiss.call(id);
                });
            }
        }
    };

    let pointer_events = if is_dismissing() { "none" } else { "auto" };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: flex-start; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border-radius: {}; \
                 box-shadow: {}; \
                 min-width: 300px; \
                 max-width: 400px; \
                 opacity: {}; \
                 transform: translateX({}px) scale({}); \
                 pointer-events: {}; \
                 position: relative; \
                 overflow: hidden;",
                spacing::SM,
                spacing::BASE,
                semantic::BG_SECONDARY,
                radius::LG,
                shadow::LG,
                opacity.get_value(),
                translate_x.get_value(),
                scale.get_value(),
                pointer_events
            ),
            role: "alert",

            // Progress bar (if auto-dismissing)
            if props.toast.duration_ms.is_some() {
                div {
                    style: format!(
                        "position: absolute; \
                         bottom: 0; \
                         left: 0; \
                         height: 3px; \
                         width: {}%; \
                         background: {}; \
                         transition: width 0.1s linear;",
                        progress.get_value(),
                        props.toast.kind.bg_color()
                    ),
                }
            }

            // Icon
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     width: 24px; \
                     height: 24px; \
                     border-radius: {}; \
                     background: {}; \
                     color: {}; \
                     font-size: 14px; \
                     font-weight: bold; \
                     flex-shrink: 0;",
                    radius::FULL,
                    props.toast.kind.bg_color(),
                    props.toast.kind.text_color()
                ),
                "{props.toast.kind.icon()}"
            }

            // Content
            div {
                style: "flex: 1; display: flex; flex-direction: column; gap: 0.25rem;",

                // Message
                span {
                    style: format!(
                        "font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY
                    ),
                    "{props.toast.message}"
                }

                // Description
                if let Some(description) = props.toast.description.clone() {
                    span {
                        style: format!(
                            "font-family: {}; \
                             font-size: {}; \
                             color: {};",
                            typography::FONT_BODY,
                            typography::SIZE_XS,
                            semantic::TEXT_SECONDARY
                        ),
                        "{description}"
                    }
                }
            }

            // Dismiss button
            button {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     width: 20px; \
                     height: 20px; \
                     border: none; \
                     background: transparent; \
                     color: {}; \
                     cursor: pointer; \
                     border-radius: {}; \
                     font-size: 12px; \
                     flex-shrink: 0;",
                    semantic::TEXT_MUTED,
                    radius::SM
                ),
                onclick: handle_dismiss,
                aria_label: "Dismiss notification",
                "✕"
            }
        }
    }
}

/// Simple toast notification without the full system.
///
/// Use this for single, immediate notifications.
#[derive(Props, Clone, PartialEq)]
pub struct SimpleToastProps {
    /// Toast kind.
    #[props(default = ToastKind::Info)]
    pub kind: ToastKind,
    /// Toast message.
    pub message: String,
    /// Optional description.
    #[props(default = None)]
    pub description: Option<String>,
    /// Whether to show the toast.
    #[props(default = true)]
    pub visible: bool,
    /// Additional CSS classes.
    #[props(default = String::new())]
    pub class: String,
}

/// Simple standalone toast notification.
#[component]
pub fn SimpleToast(props: SimpleToastProps) -> Element {
    let mut opacity = use_motion(0.0f32);
    let mut translate_y = use_motion(-20.0f32);

    use_effect(move || {
        if props.visible {
            opacity.animate_to(1.0, transitions::overlay_fade_in());
            translate_y.animate_to(
                0.0,
                AnimationConfig::new(AnimationMode::Spring(springs::toast_enter())),
            );
        } else {
            opacity.animate_to(0.0, transitions::overlay_fade_out());
            translate_y.animate_to(-20.0, transitions::overlay_fade_out());
        }
    });

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border-radius: {}; \
                 box-shadow: {}; \
                 opacity: {}; \
                 transform: translateY({}px);",
                spacing::SM,
                spacing::BASE,
                semantic::BG_SECONDARY,
                radius::LG,
                shadow::MD,
                opacity.get_value(),
                translate_y.get_value()
            ),
            class: "{props.class}",
            role: "alert",

            // Icon
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     width: 24px; \
                     height: 24px; \
                     border-radius: {}; \
                     background: {}; \
                     color: {}; \
                     font-size: 14px; \
                     font-weight: bold;",
                    radius::FULL,
                    props.kind.bg_color(),
                    props.kind.text_color()
                ),
                "{props.kind.icon()}"
            }

            // Content
            div {
                style: "display: flex; flex-direction: column;",

                span {
                    style: format!(
                        "font-family: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY
                    ),
                    "{props.message}"
                }

                if let Some(description) = props.description {
                    span {
                        style: format!(
                            "font-family: {}; \
                             font-size: {}; \
                             color: {};",
                            typography::FONT_BODY,
                            typography::SIZE_XS,
                            semantic::TEXT_SECONDARY
                        ),
                        "{description}"
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
        let toast =
            Toast::new(ToastKind::Info, "Test".to_string()).with_description("Details".to_string());
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

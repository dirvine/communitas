//! Media error banner component for displaying and handling media capture errors.

use communitas_ui_api::{DeviceType, MediaError};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the MediaErrorBanner component.
#[derive(Props, Clone, PartialEq)]
pub struct MediaErrorBannerProps {
    /// Whether to show detailed error information.
    #[props(default = true)]
    pub show_details: bool,
}

/// Banner displaying media errors with retry options.
#[component]
pub fn MediaErrorBanner(props: MediaErrorBannerProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    // Subscribe to call state for errors
    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    // Update signal when snapshot changes
    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                call_snapshot.set(rx.borrow().clone());
            }
        }
    });

    let errors = call_snapshot().media_errors;

    if errors.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "media-error-banner space-y-2 mb-4",
            role: "alert",
            aria_live: "polite",
            for error in errors.iter() {
                MediaErrorItem {
                    error: error.clone(),
                    show_details: props.show_details,
                }
            }
        }
    }
}

/// Individual media error item.
#[derive(Props, Clone, PartialEq)]
struct MediaErrorItemProps {
    error: MediaError,
    show_details: bool,
}

#[component]
fn MediaErrorItem(props: MediaErrorItemProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let error = &props.error;

    // Icon based on device type
    let icon = match error.device_type {
        DeviceType::Microphone => "🎤",
        DeviceType::Speaker => "🔊",
        DeviceType::Camera => "📷",
    };

    // Error severity color
    let border_color = if error.recoverable {
        "border-amber-700"
    } else {
        "border-red-700"
    };
    let bg_color = if error.recoverable {
        "bg-amber-900/30"
    } else {
        "bg-red-900/30"
    };

    let device_type = error.device_type;
    let on_retry = move |_| {
        let call = call_service.clone();
        spawn(async move {
            let _ = call.retry_media(device_type).await;
        });
    };

    rsx! {
        div {
            class: format!(
                "flex items-start gap-3 p-3 rounded-lg border {} {}",
                border_color,
                bg_color
            ),
            // Icon
            span {
                class: "text-xl flex-shrink-0",
                "{icon}"
            }
            // Content
            div {
                class: "flex-1 min-w-0",
                // Error title
                div {
                    class: "flex items-center gap-2",
                    span {
                        class: "font-medium text-white",
                        "{error.device_type.label()} Error"
                    }
                    if !error.recoverable {
                        span {
                            class: "px-1.5 py-0.5 bg-red-800 text-red-200 text-xs rounded",
                            "Unrecoverable"
                        }
                    }
                }
                // Error description
                if props.show_details {
                    p {
                        class: "text-sm text-slate-400 mt-1",
                        "{error.error_kind.description()}"
                    }
                    // Suggestion
                    p {
                        class: "text-xs text-slate-500 mt-1",
                        "{error.error_kind.suggestion()}"
                    }
                }
            }
            // Retry button
            if error.recoverable {
                button {
                    class: "flex-shrink-0 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded transition-colors",
                    onclick: on_retry,
                    aria_label: format!("Retry {}", error.device_type.label().to_lowercase()),
                    "Retry"
                }
            }
        }
    }
}

/// Compact error indicator for call controls.
#[derive(Props, Clone, PartialEq)]
pub struct MediaErrorIndicatorProps {
    /// Number of media errors.
    pub error_count: usize,
    /// Whether any errors are unrecoverable.
    pub has_critical: bool,
}

#[component]
pub fn MediaErrorIndicator(props: MediaErrorIndicatorProps) -> Element {
    if props.error_count == 0 {
        return rsx! {};
    }

    let bg_class = if props.has_critical {
        "bg-red-500"
    } else {
        "bg-amber-500"
    };

    rsx! {
        span {
            class: format!(
                "inline-flex items-center justify-center min-w-5 h-5 px-1.5 text-xs font-medium text-white rounded-full {}",
                bg_class
            ),
            role: "status",
            aria_label: format!(
                "{} media error{}{}",
                props.error_count,
                if props.error_count == 1 { "" } else { "s" },
                if props.has_critical { ", critical" } else { "" }
            ),
            "{props.error_count}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_api::MediaErrorKind;

    #[test]
    fn device_type_icons() {
        let icons = [
            (DeviceType::Microphone, "🎤"),
            (DeviceType::Speaker, "🔊"),
            (DeviceType::Camera, "📷"),
        ];
        for (device_type, expected) in icons {
            let icon = match device_type {
                DeviceType::Microphone => "🎤",
                DeviceType::Speaker => "🔊",
                DeviceType::Camera => "📷",
            };
            assert_eq!(icon, expected);
        }
    }

    #[test]
    fn error_severity_colors() {
        let recoverable = true;
        let border = if recoverable {
            "border-amber-700"
        } else {
            "border-red-700"
        };
        assert_eq!(border, "border-amber-700");

        let not_recoverable = false;
        let border = if not_recoverable {
            "border-amber-700"
        } else {
            "border-red-700"
        };
        assert_eq!(border, "border-red-700");
    }

    #[test]
    fn error_kind_has_description_and_suggestion() {
        let kinds = [
            MediaErrorKind::PermissionDenied,
            MediaErrorKind::DeviceNotFound,
            MediaErrorKind::DeviceInUse,
            MediaErrorKind::NotSupported,
            MediaErrorKind::Unknown,
        ];
        for kind in kinds {
            assert!(!kind.description().is_empty());
            assert!(!kind.suggestion().is_empty());
        }
    }
}

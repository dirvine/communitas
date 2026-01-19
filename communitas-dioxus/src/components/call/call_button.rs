//! Call button component for starting and ending calls.

use communitas_ui_api::CallState;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the CallButton component.
#[derive(Props, Clone, PartialEq)]
pub struct CallButtonProps {
    /// Entity ID to call (channel, group, etc.).
    pub entity_id: String,
    /// Optional size variant (default: "md").
    #[props(default = "md")]
    pub size: &'static str,
    /// Whether to show text label.
    #[props(default = true)]
    pub show_label: bool,
}

/// Call button that toggles call state for an entity.
#[component]
pub fn CallButton(props: CallButtonProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    // Subscribe to call state
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

    let state = call_snapshot().state;
    let is_active = state.is_active();
    let is_connecting = matches!(state, CallState::Connecting | CallState::Reconnecting);

    // Determine button appearance based on state
    let (bg_class, text, icon) = if is_active {
        ("bg-red-600 hover:bg-red-700", "End", "📞")
    } else {
        ("bg-emerald-600 hover:bg-emerald-700", "Call", "📞")
    };

    let size_classes = match props.size {
        "sm" => "px-2 py-1 text-sm",
        "md" => "px-3 py-2 text-base",
        "lg" => "px-4 py-3 text-lg",
        _ => "px-3 py-2 text-base",
    };

    let entity_id = props.entity_id.clone();
    let on_click = move |_| {
        let entity = entity_id.clone();
        let call = call_service.clone();
        spawn(async move {
            if is_active {
                let _ = call.leave_call().await;
            } else {
                let _ = call.join_call(&entity).await;
            }
        });
    };

    rsx! {
        button {
            class: format!(
                "call-button inline-flex items-center gap-2 rounded-lg font-medium text-white transition-colors disabled:opacity-50 {} {}",
                bg_class,
                size_classes
            ),
            disabled: is_connecting,
            onclick: on_click,
            aria_label: if is_active { "End call" } else { "Start call" },
            // Spinner for connecting state
            if is_connecting {
                span {
                    class: "animate-spin",
                    "⏳"
                }
            } else {
                span { "{icon}" }
            }
            // Label
            if props.show_label {
                span { "{text}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_button_props_default() {
        let props = CallButtonProps {
            entity_id: "test".to_string(),
            size: "md",
            show_label: true,
        };
        assert_eq!(props.entity_id, "test");
        assert_eq!(props.size, "md");
        assert!(props.show_label);
    }

    #[test]
    fn size_variants() {
        for size in ["sm", "md", "lg"] {
            let classes = match size {
                "sm" => "px-2 py-1 text-sm",
                "md" => "px-3 py-2 text-base",
                "lg" => "px-4 py-3 text-lg",
                _ => "px-3 py-2 text-base",
            };
            assert!(!classes.is_empty());
        }
    }
}

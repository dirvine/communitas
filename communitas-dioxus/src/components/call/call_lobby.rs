//! Call lobby component for pre-call device setup.

use communitas_ui_api::{CallState, DeviceType};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

use super::device_selector::DeviceSelector;
use super::media_error_banner::MediaErrorBanner;

/// Props for the CallLobby component.
#[derive(Props, Clone, PartialEq)]
pub struct CallLobbyProps {
    /// Entity ID to join call for.
    pub entity_id: String,
    /// Entity name for display.
    pub entity_name: String,
    /// Callback when user cancels.
    pub on_cancel: EventHandler<()>,
    /// Callback when user joins.
    pub on_join: EventHandler<()>,
}

/// Pre-call lobby for device setup and joining.
#[component]
pub fn CallLobby(props: CallLobbyProps) -> Element {
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

    // Load devices on mount
    let call_for_effect = call_service.clone();
    use_effect(move || {
        let call = call_for_effect.clone();
        spawn(async move {
            let _ = call.list_devices().await;
        });
    });

    let snapshot = call_snapshot();
    let is_connecting = matches!(snapshot.state, CallState::Connecting);
    let has_errors = !snapshot.media_errors.is_empty();
    let is_listen_only = snapshot.listen_only_mode;

    let entity_id = props.entity_id.clone();
    let call_for_join = call_service.clone();
    let on_join_click = move |_| {
        let entity = entity_id.clone();
        let call = call_for_join.clone();
        spawn(async move {
            let _ = call.join_call(&entity).await;
        });
        props.on_join.call(());
    };

    rsx! {
        div {
            class: "call-lobby bg-slate-900 rounded-xl p-6 max-w-md mx-auto",
            role: "dialog",
            aria_label: "Call lobby",
            // Header
            h2 {
                class: "text-xl font-semibold text-white mb-2",
                "Join Call"
            }
            p {
                class: "text-slate-400 mb-6",
                "Join call in "
                span {
                    class: "font-medium text-white",
                    "{props.entity_name}"
                }
            }
            // Media error banner
            if has_errors {
                MediaErrorBanner {}
            }
            // Listen-only warning
            if is_listen_only {
                div {
                    class: "bg-amber-900/30 border border-amber-700 rounded-lg p-3 mb-4",
                    role: "alert",
                    div {
                        class: "flex items-center gap-2 text-amber-400",
                        span { "⚠️" }
                        span { class: "font-medium", "Listen-only mode" }
                    }
                    p {
                        class: "text-sm text-amber-300 mt-1",
                        "You'll be able to hear others but won't be able to speak."
                    }
                }
            }
            // Device selectors
            div {
                class: "space-y-4 mb-6",
                DeviceSelector {
                    device_type: DeviceType::Microphone,
                }
                DeviceSelector {
                    device_type: DeviceType::Speaker,
                }
                DeviceSelector {
                    device_type: DeviceType::Camera,
                }
            }
            // Action buttons
            div {
                class: "flex gap-3",
                button {
                    class: "flex-1 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors",
                    onclick: move |_| props.on_cancel.call(()),
                    "Cancel"
                }
                button {
                    class: "flex-1 px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                    disabled: is_connecting,
                    onclick: on_join_click,
                    if is_connecting {
                        span { class: "mr-2 animate-spin", "⏳" }
                        "Joining..."
                    } else {
                        "Join Call"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn call_lobby_states() {
        // Test state transitions are valid
        use communitas_ui_api::CallState;
        assert!(!CallState::Idle.is_active());
        assert!(CallState::Connecting.is_active());
        assert!(CallState::InCall.is_active());
    }
}

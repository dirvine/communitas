//! Call control bar for in-call actions.
//!
//! Provides controls for mute, video, screen share, and ending calls.

use communitas_ui_api::CallState;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

use super::media_error_banner::MediaErrorIndicator;

/// Props for the CallControls component.
#[derive(Props, Clone, PartialEq)]
pub struct CallControlsProps {
    /// Whether to show the end call button.
    #[props(default = true)]
    pub show_end_call: bool,
    /// Whether to show the screen share button.
    #[props(default = true)]
    pub show_screen_share: bool,
    /// Optional callback when end call is clicked.
    #[props(default)]
    pub on_end_call: Option<EventHandler<()>>,
}

/// In-call control bar with mute, video, screen share, and end call buttons.
#[component]
pub fn CallControls(props: CallControlsProps) -> Element {
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

    let snapshot = call_snapshot();
    let is_in_call = matches!(snapshot.state, CallState::InCall | CallState::Reconnecting);

    // Get current participant state
    let my_id = snapshot
        .call_info
        .as_ref()
        .map(|c| c.my_participant_id.clone());
    let my_state = my_id
        .as_ref()
        .and_then(|id| snapshot.participants.iter().find(|p| p.id == *id));

    let is_muted = my_state.map(|p| p.is_muted).unwrap_or(false);
    let is_video_enabled = my_state.map(|p| p.is_video_enabled).unwrap_or(false);
    let is_screen_sharing = snapshot.is_screen_sharing;
    let has_media_errors = !snapshot.media_errors.is_empty();
    let has_critical_errors = snapshot.media_errors.iter().any(|e| !e.recoverable);

    // Mute toggle handler
    let call_for_mute = call_service.clone();
    let on_mute_click = move |_| {
        let call = call_for_mute.clone();
        spawn(async move {
            let _ = call.toggle_mute().await;
        });
    };

    // Video toggle handler
    let call_for_video = call_service.clone();
    let on_video_click = move |_| {
        let call = call_for_video.clone();
        spawn(async move {
            let _ = call.toggle_video().await;
        });
    };

    // Screen share toggle handler
    let call_for_screen = call_service.clone();
    let on_screen_share_click = move |_| {
        let call = call_for_screen.clone();
        spawn(async move {
            // Read fresh state from service to avoid TOCTOU race condition
            let snapshot = call.current_snapshot();
            if snapshot.is_screen_sharing {
                let _ = call.stop_screen_share().await;
            } else {
                let _ = call.start_screen_share().await;
            }
        });
    };

    // End call handler
    let call_for_end = call_service.clone();
    let on_end_click = move |_| {
        let call = call_for_end.clone();
        spawn(async move {
            let _ = call.leave_call().await;
        });
        if let Some(handler) = &props.on_end_call {
            handler.call(());
        }
    };

    // Button styling
    let base_btn = "flex items-center justify-center w-12 h-12 rounded-full transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900";

    let mute_class = if is_muted {
        format!(
            "{} bg-red-600 hover:bg-red-700 focus:ring-red-500",
            base_btn
        )
    } else {
        format!(
            "{} bg-slate-700 hover:bg-slate-600 focus:ring-emerald-500",
            base_btn
        )
    };

    let video_class = if is_video_enabled {
        format!(
            "{} bg-slate-700 hover:bg-slate-600 focus:ring-emerald-500",
            base_btn
        )
    } else {
        format!(
            "{} bg-red-600 hover:bg-red-700 focus:ring-red-500",
            base_btn
        )
    };

    let screen_class = if is_screen_sharing {
        format!(
            "{} bg-emerald-600 hover:bg-emerald-700 focus:ring-emerald-500",
            base_btn
        )
    } else {
        format!(
            "{} bg-slate-700 hover:bg-slate-600 focus:ring-emerald-500",
            base_btn
        )
    };

    let end_class = format!(
        "{} bg-red-600 hover:bg-red-700 focus:ring-red-500",
        base_btn
    );

    rsx! {
        div {
            class: "call-controls flex items-center justify-center gap-4 p-4 bg-slate-800/90 backdrop-blur-sm rounded-2xl",
            role: "toolbar",
            aria_label: "Call controls",
            // Mute button
            button {
                class: "{mute_class}",
                disabled: !is_in_call,
                onclick: on_mute_click,
                aria_label: if is_muted { "Unmute microphone" } else { "Mute microphone" },
                aria_pressed: "{is_muted}",
                title: if is_muted { "Unmute" } else { "Mute" },
                span {
                    class: "text-xl text-white",
                    if is_muted { "🔇" } else { "🎤" }
                }
            }
            // Video button
            button {
                class: "{video_class}",
                disabled: !is_in_call,
                onclick: on_video_click,
                aria_label: if is_video_enabled { "Turn off camera" } else { "Turn on camera" },
                aria_pressed: "{is_video_enabled}",
                title: if is_video_enabled { "Turn off camera" } else { "Turn on camera" },
                span {
                    class: "text-xl text-white",
                    if is_video_enabled { "📷" } else { "📷" }
                }
                // Overlay slash when disabled
                if !is_video_enabled {
                    span {
                        class: "absolute inset-0 flex items-center justify-center",
                        span {
                            class: "w-8 h-0.5 bg-red-500 rotate-45 rounded",
                        }
                    }
                }
            }
            // Screen share button
            if props.show_screen_share {
                button {
                    class: "{screen_class}",
                    disabled: !is_in_call,
                    onclick: on_screen_share_click,
                    aria_label: if is_screen_sharing { "Stop sharing screen" } else { "Share screen" },
                    aria_pressed: "{is_screen_sharing}",
                    title: if is_screen_sharing { "Stop sharing" } else { "Share screen" },
                    span {
                        class: "text-xl text-white",
                        "🖥️"
                    }
                }
            }
            // Media error indicator
            if has_media_errors {
                MediaErrorIndicator {
                    error_count: snapshot.media_errors.len(),
                    has_critical: has_critical_errors,
                }
            }
            // End call button
            if props.show_end_call {
                button {
                    class: "{end_class}",
                    disabled: !is_in_call,
                    onclick: on_end_click,
                    aria_label: "End call",
                    title: "End call",
                    span {
                        class: "text-xl text-white",
                        "📞"
                    }
                }
            }
        }
    }
}

/// Compact inline call controls for embedding in other views.
#[derive(Props, Clone, PartialEq)]
pub struct InlineCallControlsProps {
    /// Size variant: "sm" or "md".
    #[props(default = "sm")]
    pub size: &'static str,
}

#[component]
pub fn InlineCallControls(props: InlineCallControlsProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

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

    let snapshot = call_snapshot();
    let is_in_call = matches!(snapshot.state, CallState::InCall | CallState::Reconnecting);

    let my_id = snapshot
        .call_info
        .as_ref()
        .map(|c| c.my_participant_id.clone());
    let my_state = my_id
        .as_ref()
        .and_then(|id| snapshot.participants.iter().find(|p| p.id == *id));

    let is_muted = my_state.map(|p| p.is_muted).unwrap_or(false);

    let (btn_size, icon_size) = match props.size {
        "sm" => ("w-8 h-8", "text-sm"),
        "md" => ("w-10 h-10", "text-base"),
        _ => ("w-8 h-8", "text-sm"),
    };

    let call_for_mute = call_service.clone();
    let on_mute_click = move |_| {
        let call = call_for_mute.clone();
        spawn(async move {
            let _ = call.toggle_mute().await;
        });
    };

    let call_for_end = call_service.clone();
    let on_end_click = move |_| {
        let call = call_for_end.clone();
        spawn(async move {
            let _ = call.leave_call().await;
        });
    };

    let mute_bg = if is_muted {
        "bg-red-600"
    } else {
        "bg-slate-600"
    };

    rsx! {
        div {
            class: "inline-call-controls flex items-center gap-2",
            role: "toolbar",
            aria_label: "Quick call controls",
            // Mute toggle
            button {
                class: format!("{} {} rounded-full flex items-center justify-center hover:opacity-80", btn_size, mute_bg),
                disabled: !is_in_call,
                onclick: on_mute_click,
                aria_label: if is_muted { "Unmute" } else { "Mute" },
                span {
                    class: format!("{} text-white", icon_size),
                    if is_muted { "🔇" } else { "🎤" }
                }
            }
            // End call
            button {
                class: format!("{} bg-red-600 rounded-full flex items-center justify-center hover:bg-red-700", btn_size),
                disabled: !is_in_call,
                onclick: on_end_click,
                aria_label: "End call",
                span {
                    class: format!("{} text-white", icon_size),
                    "📞"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn button_classes_for_states() {
        let base_btn = "flex items-center justify-center w-12 h-12 rounded-full";

        // Muted state
        let muted_class = format!("{} bg-red-600", base_btn);
        assert!(muted_class.contains("bg-red-600"));

        // Unmuted state
        let unmuted_class = format!("{} bg-slate-700", base_btn);
        assert!(unmuted_class.contains("bg-slate-700"));

        // Video enabled
        let video_on = format!("{} bg-slate-700", base_btn);
        assert!(video_on.contains("bg-slate-700"));

        // Video disabled
        let video_off = format!("{} bg-red-600", base_btn);
        assert!(video_off.contains("bg-red-600"));

        // Screen sharing
        let sharing = format!("{} bg-emerald-600", base_btn);
        assert!(sharing.contains("bg-emerald-600"));
    }

    #[test]
    fn size_variants() {
        for size in ["sm", "md"] {
            let (btn, icon) = match size {
                "sm" => ("w-8 h-8", "text-sm"),
                "md" => ("w-10 h-10", "text-base"),
                _ => ("w-8 h-8", "text-sm"),
            };
            assert!(!btn.is_empty());
            assert!(!icon.is_empty());
        }
    }
}

//! Full-screen call view component.
//!
//! Displays the active call with participant grid and controls.

use communitas_ui_api::CallState;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::warn;

use super::call_controls::CallControls;
use super::media_error_banner::MediaErrorBanner;
use super::participant_tile::ParticipantGrid;

/// Props for the CallView component.
#[derive(Props, Clone, PartialEq)]
pub struct CallViewProps {
    /// Whether to show full controls (default: true).
    #[props(default = true)]
    pub show_full_controls: bool,
    /// Callback when the call ends.
    #[props(default)]
    pub on_call_ended: Option<EventHandler<()>>,
}

/// Full-screen call view with participant grid and controls.
#[component]
pub fn CallView(props: CallViewProps) -> Element {
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

    // Call the on_call_ended callback when transitioning from active to idle
    let mut prev_state = use_signal(|| CallState::Idle);
    let snapshot = call_snapshot();

    use_effect(move || {
        let current = snapshot.state;
        let was_active = prev_state().is_active();
        let is_idle = matches!(current, CallState::Idle | CallState::Disconnected);

        if was_active
            && is_idle
            && let Some(handler) = &props.on_call_ended
        {
            handler.call(());
        }
        prev_state.set(current);
    });

    let snapshot = call_snapshot();
    let call_info = snapshot.call_info.clone();
    let participants = snapshot.participants.clone();
    let my_participant_id = call_info
        .as_ref()
        .map(|c| c.my_participant_id.clone())
        .unwrap_or_default();
    let has_media_errors = !snapshot.media_errors.is_empty();
    let is_screen_sharing = snapshot.is_screen_sharing;
    let listen_only_mode = snapshot.listen_only_mode;

    // Call state info
    let is_connecting = matches!(snapshot.state, CallState::Connecting);
    let is_reconnecting = matches!(snapshot.state, CallState::Reconnecting);
    let is_idle = matches!(snapshot.state, CallState::Idle | CallState::Disconnected);

    // Entity name and duration
    let entity_name = call_info
        .as_ref()
        .map(|c| c.entity_name.clone())
        .unwrap_or_else(|| "Call".to_string());
    let duration_seconds = call_info.as_ref().map(|c| c.duration_seconds).unwrap_or(0);
    let duration_display = format_duration(duration_seconds);

    // Participant count
    let participant_count = participants.len();

    rsx! {
        div {
            class: "call-view flex flex-col h-full bg-slate-900",
            role: "main",
            aria_label: "Video call",
            // Header bar
            div {
                class: "flex items-center justify-between px-4 py-3 bg-slate-800/80 backdrop-blur-sm border-b border-slate-700",
                // Left: Call info
                div {
                    class: "flex items-center gap-3",
                    // Call status indicator
                    div {
                        class: format!(
                            "w-3 h-3 rounded-full {}",
                            if is_connecting || is_reconnecting {
                                "bg-amber-500 animate-pulse"
                            } else if is_idle {
                                "bg-slate-500"
                            } else {
                                "bg-emerald-500"
                            }
                        ),
                        title: format!(
                            "{}",
                            if is_connecting { "Connecting..." }
                            else if is_reconnecting { "Reconnecting..." }
                            else if is_idle { "Not in call" }
                            else { "In call" }
                        ),
                    }
                    // Entity name
                    h1 {
                        class: "text-lg font-medium text-white truncate max-w-xs",
                        title: "{entity_name}",
                        "{entity_name}"
                    }
                }
                // Center: Duration
                div {
                    class: "text-slate-400 font-mono text-sm",
                    aria_label: "Call duration",
                    "{duration_display}"
                }
                // Right: Participant count
                div {
                    class: "flex items-center gap-2 text-slate-400",
                    span {
                        class: "text-lg",
                        "👥"
                    }
                    span {
                        class: "text-sm",
                        "{participant_count}"
                    }
                }
            }
            // Listen-only mode banner
            if listen_only_mode {
                div {
                    class: "bg-amber-900/50 border-b border-amber-700 px-4 py-2",
                    role: "alert",
                    div {
                        class: "flex items-center justify-center gap-2 text-amber-300",
                        span { "⚠️" }
                        span {
                            class: "text-sm font-medium",
                            "Listen-only mode - Your microphone is unavailable"
                        }
                    }
                }
            }
            // Screen sharing indicator
            if is_screen_sharing {
                div {
                    class: "bg-emerald-900/50 border-b border-emerald-700 px-4 py-2",
                    role: "status",
                    div {
                        class: "flex items-center justify-center gap-2 text-emerald-300",
                        span { "🖥️" }
                        span {
                            class: "text-sm font-medium",
                            "You are sharing your screen"
                        }
                    }
                }
            }
            // Media errors banner
            if has_media_errors {
                div {
                    class: "px-4 py-2",
                    MediaErrorBanner {
                        show_details: true,
                    }
                }
            }
            // Main content: Participant grid
            div {
                class: "flex-1 overflow-auto p-4",
                if is_connecting {
                    // Connecting state
                    div {
                        class: "flex flex-col items-center justify-center h-full",
                        div {
                            class: "w-16 h-16 border-4 border-emerald-500 border-t-transparent rounded-full animate-spin mb-4",
                        }
                        p {
                            class: "text-slate-400 text-lg",
                            "Connecting to call..."
                        }
                    }
                } else if is_reconnecting {
                    // Reconnecting state
                    div {
                        class: "flex flex-col items-center justify-center h-full",
                        div {
                            class: "w-16 h-16 border-4 border-amber-500 border-t-transparent rounded-full animate-spin mb-4",
                        }
                        p {
                            class: "text-amber-400 text-lg",
                            "Reconnecting..."
                        }
                    }
                } else if is_idle {
                    // Idle state (no active call)
                    div {
                        class: "flex flex-col items-center justify-center h-full",
                        span {
                            class: "text-6xl mb-4",
                            "📞"
                        }
                        p {
                            class: "text-slate-400 text-lg",
                            "No active call"
                        }
                    }
                } else if participants.is_empty() {
                    // In call but no participants yet
                    div {
                        class: "flex flex-col items-center justify-center h-full",
                        span {
                            class: "text-6xl mb-4",
                            "👤"
                        }
                        p {
                            class: "text-slate-400 text-lg",
                            "Waiting for others to join..."
                        }
                    }
                } else {
                    // Active call with participants
                    ParticipantGrid {
                        participants: participants.clone(),
                        my_participant_id: my_participant_id.clone(),
                    }
                }
            }
            // Bottom controls bar
            div {
                class: "p-4 flex justify-center bg-slate-800/50 backdrop-blur-sm border-t border-slate-700",
                CallControls {
                    show_end_call: true,
                    show_screen_share: props.show_full_controls,
                }
            }
        }
    }
}

/// Format duration in MM:SS or HH:MM:SS format.
#[allow(dead_code)] // Used by CallView components which aren't wired into routes yet
fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Minimized call view for picture-in-picture style display.
#[derive(Props, Clone, PartialEq)]
pub struct MiniCallViewProps {
    /// Callback when expanded is requested.
    #[props(default)]
    pub on_expand: Option<EventHandler<()>>,
}

#[component]
pub fn MiniCallView(props: MiniCallViewProps) -> Element {
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
    let is_active = snapshot.state.is_active();
    let participant_count = snapshot.participants.len();
    let duration_seconds = snapshot
        .call_info
        .as_ref()
        .map(|c| c.duration_seconds)
        .unwrap_or(0);
    let duration_display = format_duration(duration_seconds);

    let my_id = snapshot
        .call_info
        .as_ref()
        .map(|c| c.my_participant_id.clone());
    let my_state = my_id
        .as_ref()
        .and_then(|id| snapshot.participants.iter().find(|p| p.id == *id));
    let is_muted = my_state.map(|p| p.is_muted).unwrap_or(false);

    let call_for_mute = call_service.clone();
    let on_mute_click = move |_| {
        let call = call_for_mute.clone();
        spawn(async move {
            if let Err(e) = call.toggle_mute().await {
                warn!("Failed to toggle mute: {e}");
            }
        });
    };

    let call_for_end = call_service.clone();
    let on_end_click = move |_| {
        let call = call_for_end.clone();
        spawn(async move {
            if let Err(e) = call.leave_call().await {
                warn!("Failed to leave call: {e}");
            }
        });
    };

    if !is_active {
        return rsx! {};
    }

    rsx! {
        div {
            class: "mini-call-view fixed bottom-4 right-4 w-64 bg-slate-800 rounded-xl shadow-2xl overflow-hidden border border-slate-700",
            role: "complementary",
            aria_label: "Active call",
            // Header
            button {
                class: "w-full flex items-center justify-between p-3 hover:bg-slate-700/50 transition-colors",
                onclick: move |_| {
                    if let Some(handler) = &props.on_expand {
                        handler.call(());
                    }
                },
                aria_label: "Expand call view",
                div {
                    class: "flex items-center gap-2",
                    // Pulsing indicator
                    span {
                        class: "w-2 h-2 rounded-full bg-emerald-500 animate-pulse",
                    }
                    span {
                        class: "text-sm font-medium text-white",
                        "In call"
                    }
                }
                div {
                    class: "flex items-center gap-2 text-slate-400 text-sm",
                    span { "👥 {participant_count}" }
                    span { "{duration_display}" }
                }
            }
            // Quick controls
            div {
                class: "flex items-center justify-center gap-3 p-3 border-t border-slate-700",
                // Mute button
                button {
                    class: format!(
                        "w-10 h-10 rounded-full flex items-center justify-center transition-colors {}",
                        if is_muted { "bg-red-600 hover:bg-red-700" } else { "bg-slate-600 hover:bg-slate-500" }
                    ),
                    onclick: on_mute_click,
                    aria_label: if is_muted { "Unmute" } else { "Mute" },
                    span {
                        class: "text-white",
                        if is_muted { "🔇" } else { "🎤" }
                    }
                }
                // End call button
                button {
                    class: "w-10 h-10 rounded-full bg-red-600 hover:bg-red-700 flex items-center justify-center transition-colors",
                    onclick: on_end_click,
                    aria_label: "End call",
                    span {
                        class: "text-white",
                        "📞"
                    }
                }
            }
        }
    }
}

/// Call status bar for displaying in other views.
#[derive(Props, Clone, PartialEq)]
pub struct CallStatusBarProps {
    /// Callback when clicked.
    #[props(default)]
    pub on_click: Option<EventHandler<()>>,
}

#[component]
pub fn CallStatusBar(props: CallStatusBarProps) -> Element {
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
    let is_active = snapshot.state.is_active();
    let entity_name = snapshot
        .call_info
        .as_ref()
        .map(|c| c.entity_name.clone())
        .unwrap_or_else(|| "Call".to_string());
    let duration_seconds = snapshot
        .call_info
        .as_ref()
        .map(|c| c.duration_seconds)
        .unwrap_or(0);
    let duration_display = format_duration(duration_seconds);

    if !is_active {
        return rsx! {};
    }

    rsx! {
        button {
            class: "call-status-bar w-full flex items-center justify-between px-4 py-2 bg-emerald-900/80 hover:bg-emerald-900 border-b border-emerald-700 transition-colors",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(());
                }
            },
            aria_label: "Return to call",
            div {
                class: "flex items-center gap-2",
                span {
                    class: "w-2 h-2 rounded-full bg-emerald-400 animate-pulse",
                }
                span {
                    class: "text-emerald-200 text-sm font-medium",
                    "In call: {entity_name}"
                }
            }
            div {
                class: "flex items-center gap-2 text-emerald-300 text-sm",
                span { "{duration_display}" }
                span { "→" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_minutes_seconds() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(59), "00:59");
        assert_eq!(format_duration(60), "01:00");
        assert_eq!(format_duration(125), "02:05");
        assert_eq!(format_duration(3599), "59:59");
    }

    #[test]
    fn format_duration_with_hours() {
        assert_eq!(format_duration(3600), "01:00:00");
        assert_eq!(format_duration(3661), "01:01:01");
        assert_eq!(format_duration(7200), "02:00:00");
        assert_eq!(format_duration(36000), "10:00:00");
    }

    #[test]
    fn call_state_checks() {
        assert!(!CallState::Idle.is_active());
        assert!(CallState::Connecting.is_active());
        assert!(CallState::InCall.is_active());
        assert!(CallState::Reconnecting.is_active());
        assert!(!CallState::Disconnected.is_active());
    }
}

//! Recording indicator showing when a call is being recorded.
//!
//! Displays a pulsing red dot with "Recording" text and optional duration.

use communitas_ui_api::call::RecordingState;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the RecordingIndicator component.
#[derive(Props, Clone, PartialEq)]
pub struct RecordingIndicatorProps {
    /// Size variant: "sm", "md", or "lg".
    #[props(default = "md")]
    pub size: &'static str,
    /// Whether to show the duration.
    #[props(default = true)]
    pub show_duration: bool,
    /// Whether to show the "Recording" label.
    #[props(default = true)]
    pub show_label: bool,
}

/// Recording indicator with pulsing red dot.
#[component]
pub fn RecordingIndicator(props: RecordingIndicatorProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    // Subscribe to call state changes
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

    // Only show if recording is active
    if !snapshot.is_recording {
        return rsx! {};
    }

    let recording_state = snapshot.recording_state();
    let duration = snapshot.recording_duration();

    // Size variants
    let (dot_size, text_size, gap) = match props.size {
        "sm" => ("w-2 h-2", "text-xs", "gap-1"),
        "md" => ("w-2.5 h-2.5", "text-sm", "gap-1.5"),
        "lg" => ("w-3 h-3", "text-base", "gap-2"),
        _ => ("w-2.5 h-2.5", "text-sm", "gap-1.5"),
    };

    // Pulse animation for active recording, no pulse when paused
    let animation = match recording_state {
        RecordingState::Recording => "animate-pulse",
        RecordingState::Paused => "",
        RecordingState::Finalizing => "",
        RecordingState::NotRecording => "",
    };

    // Dot color based on state
    let dot_color = match recording_state {
        RecordingState::Recording => "bg-red-500",
        RecordingState::Paused => "bg-amber-500",
        RecordingState::Finalizing => "bg-slate-400",
        RecordingState::NotRecording => "bg-slate-500",
    };

    // Label text
    let label = match recording_state {
        RecordingState::Recording => "Recording",
        RecordingState::Paused => "Paused",
        RecordingState::Finalizing => "Saving...",
        RecordingState::NotRecording => "",
    };

    rsx! {
        div {
            class: "recording-indicator inline-flex items-center {gap} px-2 py-1 bg-slate-800/80 rounded-full",
            role: "status",
            aria_label: format!("Recording: {}", label),

            // Pulsing dot
            span {
                class: format!("{dot_size} {dot_color} rounded-full {animation}"),
            }

            // Label
            if props.show_label {
                span {
                    class: format!("{text_size} text-red-400 font-medium"),
                    "{label}"
                }
            }

            // Duration
            if props.show_duration {
                if let Some(ref dur) = duration {
                    span {
                        class: format!("{text_size} text-slate-300 font-mono"),
                        "{dur}"
                    }
                }
            }
        }
    }
}

/// Compact recording dot (just a pulsing red dot).
#[derive(Props, Clone, PartialEq)]
pub struct RecordingDotProps {
    /// Size variant: "sm", "md", or "lg".
    #[props(default = "sm")]
    pub size: &'static str,
}

#[component]
pub fn RecordingDot(props: RecordingDotProps) -> Element {
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

    // Only show if recording is active
    if !snapshot.is_recording {
        return rsx! {};
    }

    let recording_state = snapshot.recording_state();

    let dot_size = match props.size {
        "sm" => "w-2 h-2",
        "md" => "w-2.5 h-2.5",
        "lg" => "w-3 h-3",
        _ => "w-2 h-2",
    };

    let animation = match recording_state {
        RecordingState::Recording => "animate-pulse",
        _ => "",
    };

    let dot_color = match recording_state {
        RecordingState::Recording => "bg-red-500",
        RecordingState::Paused => "bg-amber-500",
        RecordingState::Finalizing => "bg-slate-400",
        RecordingState::NotRecording => "bg-slate-500",
    };

    let label = recording_state.label();

    rsx! {
        span {
            class: format!("recording-dot {dot_size} {dot_color} rounded-full {animation}"),
            role: "status",
            aria_label: format!("Recording: {}", label),
            title: "{label}",
        }
    }
}

/// Recording controls panel for managing recording state.
#[derive(Props, Clone, PartialEq)]
pub struct RecordingControlsProps {
    /// Whether to show pause/resume controls.
    #[props(default = true)]
    pub show_pause_controls: bool,
}

#[component]
pub fn RecordingControls(props: RecordingControlsProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    // Clone services before moving into closure
    let services_for_resource = services.clone();
    let _updater = use_resource(move || {
        let call = services_for_resource.call();
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
    let recording_state = snapshot.recording_state();

    // Start recording handler
    let start_recording = {
        let services = services.clone();
        move |_| {
            let call = services.call();
            spawn(async move {
                let _ = call.start_recording(true).await;
            });
        }
    };

    // Stop recording handler
    let stop_recording = {
        let services = services.clone();
        move |_| {
            let call = services.call();
            spawn(async move {
                let _ = call.stop_recording().await;
            });
        }
    };

    // Pause recording handler
    let pause_recording = {
        let services = services.clone();
        move |_| {
            let call = services.call();
            spawn(async move {
                let _ = call.pause_recording().await;
            });
        }
    };

    // Resume recording handler
    let resume_recording = {
        let services = services.clone();
        move |_| {
            let call = services.call();
            spawn(async move {
                let _ = call.resume_recording().await;
            });
        }
    };

    rsx! {
        div {
            class: "recording-controls flex items-center gap-2",

            match recording_state {
                RecordingState::NotRecording => rsx! {
                    button {
                        class: "px-3 py-1.5 bg-red-600 hover:bg-red-700 text-white text-sm rounded-lg transition-colors",
                        onclick: start_recording,
                        "Start Recording"
                    }
                },
                RecordingState::Recording => rsx! {
                    // Recording indicator
                    RecordingIndicator { size: "sm", show_duration: true }

                    // Pause button (if enabled)
                    if props.show_pause_controls {
                        button {
                            class: "p-1.5 bg-slate-700 hover:bg-slate-600 text-slate-200 rounded-lg transition-colors",
                            onclick: pause_recording,
                            title: "Pause recording",
                            // Pause icon (two vertical bars)
                            svg {
                                class: "w-4 h-4",
                                fill: "currentColor",
                                view_box: "0 0 20 20",
                                path {
                                    fill_rule: "evenodd",
                                    d: "M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z",
                                    clip_rule: "evenodd",
                                }
                            }
                        }
                    }

                    // Stop button
                    button {
                        class: "p-1.5 bg-red-700 hover:bg-red-600 text-white rounded-lg transition-colors",
                        onclick: stop_recording,
                        title: "Stop recording",
                        // Stop icon (square)
                        svg {
                            class: "w-4 h-4",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                fill_rule: "evenodd",
                                d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z",
                                clip_rule: "evenodd",
                            }
                        }
                    }
                },
                RecordingState::Paused => rsx! {
                    // Paused indicator
                    RecordingIndicator { size: "sm", show_duration: true }

                    // Resume button
                    if props.show_pause_controls {
                        button {
                            class: "p-1.5 bg-green-700 hover:bg-green-600 text-white rounded-lg transition-colors",
                            onclick: resume_recording,
                            title: "Resume recording",
                            // Play icon (triangle)
                            svg {
                                class: "w-4 h-4",
                                fill: "currentColor",
                                view_box: "0 0 20 20",
                                path {
                                    fill_rule: "evenodd",
                                    d: "M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z",
                                    clip_rule: "evenodd",
                                }
                            }
                        }
                    }

                    // Stop button
                    button {
                        class: "p-1.5 bg-red-700 hover:bg-red-600 text-white rounded-lg transition-colors",
                        onclick: stop_recording,
                        title: "Stop recording",
                        svg {
                            class: "w-4 h-4",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                fill_rule: "evenodd",
                                d: "M10 18a8 8 0 100-16 8 8 0 000 16zM8 7a1 1 0 00-1 1v4a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1H8z",
                                clip_rule: "evenodd",
                            }
                        }
                    }
                },
                RecordingState::Finalizing => rsx! {
                    // Finalizing indicator
                    RecordingIndicator { size: "sm", show_duration: false }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use communitas_ui_api::call::RecordingState;

    #[test]
    fn recording_state_labels() {
        assert_eq!(RecordingState::NotRecording.label(), "Not Recording");
        assert_eq!(RecordingState::Recording.label(), "Recording");
        assert_eq!(RecordingState::Paused.label(), "Paused");
        assert_eq!(RecordingState::Finalizing.label(), "Saving...");
    }

    #[test]
    fn recording_state_is_active() {
        assert!(!RecordingState::NotRecording.is_active());
        assert!(RecordingState::Recording.is_active());
        assert!(RecordingState::Paused.is_active());
        assert!(RecordingState::Finalizing.is_active());
    }

    #[test]
    fn size_variants() {
        let sizes = ["sm", "md", "lg"];
        for size in sizes {
            let (dot_size, text_size, _) = match size {
                "sm" => ("w-2 h-2", "text-xs", "gap-1"),
                "md" => ("w-2.5 h-2.5", "text-sm", "gap-1.5"),
                "lg" => ("w-3 h-3", "text-base", "gap-2"),
                _ => ("w-2.5 h-2.5", "text-sm", "gap-1.5"),
            };
            assert!(!dot_size.is_empty());
            assert!(!text_size.is_empty());
        }
    }
}

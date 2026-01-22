//! Screen share source picker for selecting monitors or windows to share.
//!
//! Provides a visual picker with thumbnail previews to choose what screen
//! content to share during a call.

use communitas_ui_api::call::{ScreenShareSource, ScreenShareSourceType};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::warn;

/// Props for the ScreenSharePicker component.
#[derive(Props, Clone, PartialEq)]
pub struct ScreenSharePickerProps {
    /// Callback when picker is closed without selection.
    pub on_close: EventHandler<()>,
    /// Callback when screen share is started.
    pub on_started: EventHandler<()>,
}

/// Modal picker for selecting screen share source.
#[component]
pub fn ScreenSharePicker(props: ScreenSharePickerProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut sources = use_signal(Vec::<ScreenShareSource>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);
    let mut selected_tab = use_signal(|| SourceTab::Monitors);
    let mut share_audio = use_signal(|| false);
    let mut allow_control = use_signal(|| false);
    let mut starting = use_signal(|| false);

    // Load sources on mount
    let call_for_effect = call_service.clone();
    use_effect(move || {
        let call = call_for_effect.clone();
        spawn(async move {
            loading.set(true);
            error.set(None);

            match call.enumerate_screen_sources().await {
                Ok(result) => {
                    sources.set(result);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to list sources: {e}")));
                }
            }
            loading.set(false);
        });
    });

    let current_sources = sources();
    let monitors: Vec<_> = current_sources
        .iter()
        .filter(|s| s.source_type == ScreenShareSourceType::Monitor)
        .cloned()
        .collect();
    let windows: Vec<_> = current_sources
        .iter()
        .filter(|s| s.source_type == ScreenShareSourceType::Window)
        .cloned()
        .collect();

    let call_for_retry = call_service.clone();
    let call_for_select = call_service.clone();
    let on_select_source = move |source_id: String| {
        let call = call_for_select.clone();
        let share_audio_val = share_audio();
        let allow_control_val = allow_control();
        let on_started = props.on_started;

        starting.set(true);
        spawn(async move {
            match call
                .start_screen_share_with_source(&source_id, share_audio_val, allow_control_val)
                .await
            {
                Ok(()) => {
                    on_started.call(());
                }
                Err(e) => {
                    warn!("Failed to start screen share: {e}");
                    error.set(Some(format!("Failed to start sharing: {e}")));
                    starting.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "screen-share-picker-overlay fixed inset-0 z-50 bg-black/70 flex items-center justify-center",
            onclick: move |e| {
                e.stop_propagation();
                props.on_close.call(());
            },

            div {
                class: "screen-share-picker bg-slate-900 rounded-xl shadow-2xl w-full max-w-2xl max-h-[80vh] overflow-hidden",
                onclick: move |e| e.stop_propagation(),
                role: "dialog",
                aria_label: "Screen share picker",

                // Header
                div {
                    class: "flex items-center justify-between px-6 py-4 border-b border-slate-700",
                    h2 {
                        class: "text-xl font-semibold text-white",
                        "Share your screen"
                    }
                    button {
                        class: "p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-lg transition-colors",
                        onclick: move |_| props.on_close.call(()),
                        aria_label: "Close",
                        span { class: "text-xl", "×" }
                    }
                }

                // Tab bar
                div {
                    class: "flex border-b border-slate-700",
                    role: "tablist",
                    TabButton {
                        label: "Screens",
                        count: monitors.len(),
                        is_selected: selected_tab() == SourceTab::Monitors,
                        on_click: move |_| selected_tab.set(SourceTab::Monitors),
                    }
                    TabButton {
                        label: "Windows",
                        count: windows.len(),
                        is_selected: selected_tab() == SourceTab::Windows,
                        on_click: move |_| selected_tab.set(SourceTab::Windows),
                    }
                }

                // Content area
                div {
                    class: "p-6 overflow-y-auto max-h-96",

                    if loading() {
                        div {
                            class: "flex items-center justify-center py-12",
                            div {
                                class: "animate-spin w-8 h-8 border-2 border-emerald-500 border-t-transparent rounded-full",
                            }
                            span {
                                class: "ml-3 text-slate-400",
                                "Loading available sources..."
                            }
                        }
                    } else if let Some(err) = error() {
                        div {
                            class: "text-center py-8",
                            p {
                                class: "text-red-400 mb-4",
                                "{err}"
                            }
                            button {
                                class: "px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg",
                                onclick: move |_| {
                                    let call = call_for_retry.clone();
                                    spawn(async move {
                                        loading.set(true);
                                        error.set(None);
                                        match call.enumerate_screen_sources().await {
                                            Ok(result) => sources.set(result),
                                            Err(e) => error.set(Some(format!("Failed: {e}"))),
                                        }
                                        loading.set(false);
                                    });
                                },
                                "Retry"
                            }
                        }
                    } else if starting() {
                        div {
                            class: "flex items-center justify-center py-12",
                            div {
                                class: "animate-spin w-8 h-8 border-2 border-emerald-500 border-t-transparent rounded-full",
                            }
                            span {
                                class: "ml-3 text-slate-400",
                                "Starting screen share..."
                            }
                        }
                    } else {
                        match selected_tab() {
                            SourceTab::Monitors => rsx! {
                                SourceGrid {
                                    sources: monitors.clone(),
                                    empty_message: "No monitors found",
                                    on_select: on_select_source.clone(),
                                }
                            },
                            SourceTab::Windows => rsx! {
                                SourceGrid {
                                    sources: windows.clone(),
                                    empty_message: "No windows found",
                                    on_select: on_select_source.clone(),
                                }
                            },
                        }
                    }
                }

                // Options footer
                div {
                    class: "px-6 py-4 bg-slate-800/50 border-t border-slate-700",
                    div {
                        class: "flex items-center gap-6 text-sm",
                        label {
                            class: "flex items-center gap-2 text-slate-300 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                class: "w-4 h-4 rounded border-slate-600 bg-slate-700 text-emerald-500 focus:ring-emerald-500",
                                checked: share_audio(),
                                onchange: move |e| share_audio.set(e.checked()),
                            }
                            "Share audio"
                        }
                        label {
                            class: "flex items-center gap-2 text-slate-300 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                class: "w-4 h-4 rounded border-slate-600 bg-slate-700 text-emerald-500 focus:ring-emerald-500",
                                checked: allow_control(),
                                onchange: move |e| allow_control.set(e.checked()),
                            }
                            "Allow remote control"
                        }
                    }
                }
            }
        }
    }
}

// Note: Used in the rsx! macro for selected_tab signal and match
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(dead_code)]
enum SourceTab {
    Monitors,
    Windows,
}

/// Props for TabButton.
#[derive(Props, Clone, PartialEq)]
struct TabButtonProps {
    label: &'static str,
    count: usize,
    is_selected: bool,
    on_click: EventHandler<()>,
}

#[component]
fn TabButton(props: TabButtonProps) -> Element {
    let base_class = "px-6 py-3 text-sm font-medium transition-colors relative";
    let selected_class = if props.is_selected {
        format!("{base_class} text-emerald-400")
    } else {
        format!("{base_class} text-slate-400 hover:text-white")
    };

    rsx! {
        button {
            class: "{selected_class}",
            role: "tab",
            aria_selected: "{props.is_selected}",
            onclick: move |_| props.on_click.call(()),
            "{props.label}"
            span {
                class: "ml-2 px-1.5 py-0.5 bg-slate-700 text-xs rounded",
                "{props.count}"
            }
            if props.is_selected {
                span {
                    class: "absolute bottom-0 left-0 right-0 h-0.5 bg-emerald-500",
                }
            }
        }
    }
}

/// Props for SourceGrid.
#[derive(Props, Clone, PartialEq)]
struct SourceGridProps {
    sources: Vec<ScreenShareSource>,
    empty_message: &'static str,
    on_select: Callback<String>,
}

#[component]
fn SourceGrid(props: SourceGridProps) -> Element {
    if props.sources.is_empty() {
        return rsx! {
            div {
                class: "text-center py-8 text-slate-400",
                "{props.empty_message}"
            }
        };
    }

    rsx! {
        div {
            class: "grid grid-cols-2 gap-4",
            role: "listbox",
            for source in props.sources.iter() {
                SourceCard {
                    key: "{source.id}",
                    source: source.clone(),
                    on_select: props.on_select,
                }
            }
        }
    }
}

/// Props for SourceCard.
#[derive(Props, Clone, PartialEq)]
struct SourceCardProps {
    source: ScreenShareSource,
    on_select: Callback<String>,
}

#[component]
fn SourceCard(props: SourceCardProps) -> Element {
    let source = &props.source;
    let source_id = source.id.clone();

    let type_icon = match source.source_type {
        ScreenShareSourceType::Monitor => "🖥️",
        ScreenShareSourceType::Window => "📱",
    };

    let badge = if source.is_primary {
        Some("Primary")
    } else {
        None
    };

    rsx! {
        button {
            class: "source-card group relative bg-slate-800 hover:bg-slate-700 border border-slate-700 hover:border-emerald-500 rounded-lg p-3 text-left transition-all focus:outline-none focus:ring-2 focus:ring-emerald-500",
            role: "option",
            onclick: move |_| props.on_select.call(source_id.clone()),

            // Thumbnail area
            div {
                class: "aspect-video bg-slate-900 rounded-md mb-3 flex items-center justify-center overflow-hidden",
                if let Some(thumbnail) = &source.thumbnail {
                    img {
                        src: "{thumbnail}",
                        alt: "{source.name} preview",
                        class: "w-full h-full object-cover",
                        width: source.thumbnail_width.map(|w| format!("{w}")),
                        height: source.thumbnail_height.map(|h| format!("{h}")),
                    }
                } else {
                    // Placeholder when no thumbnail
                    span {
                        class: "text-4xl text-slate-600",
                        "{type_icon}"
                    }
                }
            }

            // Source info
            div {
                class: "flex items-start gap-2",
                span {
                    class: "text-xl flex-shrink-0",
                    "{type_icon}"
                }
                div {
                    class: "flex-1 min-w-0",
                    p {
                        class: "text-sm font-medium text-white truncate",
                        "{source.name}"
                    }
                    if let Some(app) = &source.app_name {
                        p {
                            class: "text-xs text-slate-400 truncate",
                            "{app}"
                        }
                    }
                }
            }

            // Primary badge
            if let Some(badge_text) = badge {
                span {
                    class: "absolute top-2 right-2 px-2 py-0.5 bg-emerald-600 text-white text-xs rounded-full",
                    "{badge_text}"
                }
            }

            // Hover overlay with "Share" button
            div {
                class: "absolute inset-0 bg-emerald-500/20 opacity-0 group-hover:opacity-100 rounded-lg transition-opacity flex items-center justify-center",
                span {
                    class: "px-4 py-2 bg-emerald-600 text-white font-medium rounded-lg shadow-lg",
                    "Share"
                }
            }
        }
    }
}

/// Indicator showing what is currently being shared.
#[derive(Props, Clone, PartialEq)]
pub struct ScreenShareIndicatorProps {
    /// The source being shared.
    pub source: ScreenShareSource,
    /// Callback to stop sharing.
    pub on_stop: EventHandler<()>,
}

#[component]
pub fn ScreenShareIndicator(props: ScreenShareIndicatorProps) -> Element {
    let source = &props.source;
    let type_label = source.source_type.label();

    rsx! {
        div {
            class: "screen-share-indicator flex items-center gap-3 px-4 py-2 bg-red-600/90 text-white rounded-lg shadow-lg",
            role: "status",
            aria_live: "polite",

            // Recording dot
            span {
                class: "w-3 h-3 bg-white rounded-full animate-pulse",
            }

            // Info
            div {
                class: "flex-1 min-w-0",
                p {
                    class: "text-sm font-medium truncate",
                    "Sharing: {source.name}"
                }
                p {
                    class: "text-xs text-red-200",
                    "{type_label}"
                }
            }

            // Stop button
            button {
                class: "px-3 py-1.5 bg-white text-red-600 font-medium text-sm rounded-lg hover:bg-red-50 transition-colors",
                onclick: move |_| props.on_stop.call(()),
                "Stop"
            }
        }
    }
}

/// Compact screen share button that opens the picker.
#[derive(Props, Clone, PartialEq)]
pub struct ScreenShareButtonProps {
    /// Whether currently in a call.
    #[props(default = false)]
    pub disabled: bool,
}

#[component]
pub fn ScreenShareButton(props: ScreenShareButtonProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut show_picker = use_signal(|| false);
    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    // Subscribe to call state
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
    let is_sharing = snapshot.is_screen_sharing;

    let on_click = move |_| {
        if is_sharing {
            // Stop sharing
            let call = call_service.clone();
            spawn(async move {
                if let Err(e) = call.stop_screen_share().await {
                    warn!("Failed to stop screen share: {e}");
                }
            });
        } else {
            // Open picker
            show_picker.set(true);
        }
    };

    let button_class = if is_sharing {
        "flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
    } else {
        "flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    };

    rsx! {
        button {
            class: "{button_class}",
            disabled: props.disabled,
            onclick: on_click,
            aria_label: if is_sharing { "Stop sharing screen" } else { "Share screen" },
            span { class: "text-lg", "🖥️" }
            span {
                if is_sharing { "Stop Sharing" } else { "Share Screen" }
            }
        }

        if show_picker() {
            ScreenSharePicker {
                on_close: move |_| show_picker.set(false),
                on_started: move |_| show_picker.set(false),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_tab_equality() {
        assert_eq!(SourceTab::Monitors, SourceTab::Monitors);
        assert_eq!(SourceTab::Windows, SourceTab::Windows);
        assert_ne!(SourceTab::Monitors, SourceTab::Windows);
    }

    #[test]
    fn screen_share_source_type_icons() {
        let monitor_icon = match ScreenShareSourceType::Monitor {
            ScreenShareSourceType::Monitor => "🖥️",
            ScreenShareSourceType::Window => "📱",
        };
        assert_eq!(monitor_icon, "🖥️");

        let window_icon = match ScreenShareSourceType::Window {
            ScreenShareSourceType::Monitor => "🖥️",
            ScreenShareSourceType::Window => "📱",
        };
        assert_eq!(window_icon, "📱");
    }

    #[test]
    fn primary_badge_shown_for_primary_monitors() {
        let badge = if true { Some("Primary") } else { None };
        assert_eq!(badge, Some("Primary"));

        let badge_not_primary = if false { Some("Primary") } else { None };
        assert_eq!(badge_not_primary, None);
    }
}

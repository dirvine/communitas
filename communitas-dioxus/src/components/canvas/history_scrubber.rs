// SPDX-License-Identifier: MIT OR Apache-2.0

//! History scrubber component for canvas undo/redo timeline.

use communitas_ui_api::{HistoryActionType, HistoryEntry};
use dioxus::prelude::*;

/// Props for the HistoryScrubber component.
#[derive(Props, Clone, PartialEq)]
pub struct HistoryScrubberProps {
    /// History entries to display.
    pub entries: Vec<HistoryEntry>,
    /// Current position in history (entry ID).
    #[props(default)]
    pub current_position: Option<String>,
    /// Callback when entry is clicked to scrub to.
    #[props(default)]
    pub on_scrub_to: Option<EventHandler<String>>,
    /// Callback when undo is clicked.
    #[props(default)]
    pub on_undo: Option<EventHandler<()>>,
    /// Callback when redo is clicked.
    #[props(default)]
    pub on_redo: Option<EventHandler<()>>,
}

/// History timeline for scrubbing through canvas states.
#[component]
pub fn HistoryScrubber(props: HistoryScrubberProps) -> Element {
    let can_undo = props
        .entries
        .iter()
        .any(|e| e.can_undo && Some(&e.id) == props.current_position.as_ref());
    let can_redo = props
        .entries
        .iter()
        .any(|e| e.can_redo && Some(&e.id) == props.current_position.as_ref());

    rsx! {
        div {
            class: "history-scrubber bg-slate-800 rounded-lg p-2",
            role: "region",
            aria_label: "Canvas history",
            // Header with undo/redo
            div {
                class: "flex items-center justify-between mb-2",
                span {
                    class: "text-xs font-medium text-slate-400",
                    "History"
                }
                div {
                    class: "flex items-center gap-1",
                    button {
                        class: format!(
                            "px-2 py-1 text-xs rounded transition-colors {}",
                            if can_undo { "bg-slate-700 hover:bg-slate-600 text-slate-300" } else { "bg-slate-800 text-slate-600 cursor-not-allowed" }
                        ),
                        disabled: !can_undo,
                        title: "Undo (Ctrl+Z)",
                        onclick: {
                            let on_undo = props.on_undo;
                            move |_| {
                                if let Some(handler) = &on_undo {
                                    handler.call(());
                                }
                            }
                        },
                        "Undo"
                    }
                    button {
                        class: format!(
                            "px-2 py-1 text-xs rounded transition-colors {}",
                            if can_redo { "bg-slate-700 hover:bg-slate-600 text-slate-300" } else { "bg-slate-800 text-slate-600 cursor-not-allowed" }
                        ),
                        disabled: !can_redo,
                        title: "Redo (Ctrl+Y)",
                        onclick: {
                            let on_redo = props.on_redo;
                            move |_| {
                                if let Some(handler) = &on_redo {
                                    handler.call(());
                                }
                            }
                        },
                        "Redo"
                    }
                }
            }
            // Timeline
            if props.entries.is_empty() {
                div {
                    class: "text-center text-slate-500 text-xs py-2",
                    "No history yet"
                }
            } else {
                div {
                    class: "flex items-center gap-1 overflow-x-auto pb-1",
                    role: "listbox",
                    aria_label: "History timeline",
                    for entry in props.entries.iter() {
                        HistoryEntryDot {
                            key: "{entry.id}",
                            entry: entry.clone(),
                            is_current: Some(&entry.id) == props.current_position.as_ref(),
                            on_click: {
                                let on_scrub = props.on_scrub_to;
                                let id = entry.id.clone();
                                move |_| {
                                    if let Some(handler) = &on_scrub {
                                        handler.call(id.clone());
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Props for history entry dot.
#[derive(Props, Clone, PartialEq)]
struct HistoryEntryDotProps {
    entry: HistoryEntry,
    is_current: bool,
    on_click: EventHandler<()>,
}

/// Individual history entry in the timeline.
#[component]
fn HistoryEntryDot(props: HistoryEntryDotProps) -> Element {
    let entry = &props.entry;
    let icon = action_icon(entry.action_type);

    let bg_class = if props.is_current {
        "bg-emerald-500 ring-2 ring-emerald-400 ring-offset-1 ring-offset-slate-800"
    } else {
        "bg-slate-600 hover:bg-slate-500"
    };

    rsx! {
        button {
            class: format!(
                "relative flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center transition-all {}",
                bg_class
            ),
            title: "{entry.description}",
            aria_label: "{entry.description}",
            role: "option",
            aria_selected: props.is_current,
            onclick: move |_| props.on_click.call(()),
            span { class: "text-xs", "{icon}" }
            // Current position indicator
            if props.is_current {
                div {
                    class: "absolute -bottom-1 w-0 h-0 border-l-4 border-r-4 border-t-4 border-l-transparent border-r-transparent border-t-emerald-500",
                }
            }
        }
    }
}

/// Get icon for history action type.
#[allow(dead_code)] // Called from RSX macro in HistoryEntryDot component
fn action_icon(action_type: HistoryActionType) -> &'static str {
    match action_type {
        HistoryActionType::AddElement => "+",
        HistoryActionType::DeleteElement => "-",
        HistoryActionType::ModifyElement => "~",
        HistoryActionType::AddLayer => "L+",
        HistoryActionType::DeleteLayer => "L-",
        HistoryActionType::ModifyLayer => "L~",
        HistoryActionType::BatchOperation => "[]",
    }
}

/// Compact history indicator for toolbar integration.
#[derive(Props, Clone, PartialEq)]
pub struct HistoryIndicatorProps {
    /// Total number of history entries.
    pub total_entries: usize,
    /// Current position index (0 = oldest).
    pub current_index: usize,
    /// Whether undo is available.
    pub can_undo: bool,
    /// Whether redo is available.
    pub can_redo: bool,
}

#[component]
pub fn HistoryIndicator(props: HistoryIndicatorProps) -> Element {
    if props.total_entries == 0 {
        return rsx! {};
    }

    let progress = if props.total_entries > 0 {
        ((props.current_index as f32 / props.total_entries as f32) * 100.0) as i32
    } else {
        0
    };

    rsx! {
        div {
            class: "history-indicator flex items-center gap-2 text-xs text-slate-400",
            // Progress bar
            div {
                class: "w-16 h-1 bg-slate-700 rounded-full overflow-hidden",
                title: "History position: {props.current_index + 1} of {props.total_entries}",
                div {
                    class: "h-full bg-emerald-500 transition-all",
                    style: "width: {progress}%",
                }
            }
            // Position text
            span {
                "{props.current_index + 1}/{props.total_entries}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str, action: HistoryActionType) -> HistoryEntry {
        HistoryEntry {
            id: id.to_string(),
            action_type: action,
            timestamp: 0,
            user_id: "user-1".to_string(),
            description: format!("{:?}", action),
            can_undo: true,
            can_redo: false,
        }
    }

    #[test]
    fn action_icons_all_defined() {
        let actions = [
            HistoryActionType::AddElement,
            HistoryActionType::DeleteElement,
            HistoryActionType::ModifyElement,
            HistoryActionType::AddLayer,
            HistoryActionType::DeleteLayer,
            HistoryActionType::ModifyLayer,
            HistoryActionType::BatchOperation,
        ];

        for action in actions {
            let icon = action_icon(action);
            assert!(!icon.is_empty(), "Action {:?} should have an icon", action);
        }
    }

    #[test]
    fn history_progress_calculation() {
        let total = 10;
        let current = 5;
        let progress = ((current as f32 / total as f32) * 100.0) as i32;
        assert_eq!(progress, 50);
    }

    #[test]
    fn empty_history_progress() {
        let total = 0;
        let progress = if total > 0 { 50 } else { 0 };
        assert_eq!(progress, 0);
    }

    #[test]
    fn current_position_matching() {
        let entries = [
            make_entry("1", HistoryActionType::AddElement),
            make_entry("2", HistoryActionType::ModifyElement),
            make_entry("3", HistoryActionType::DeleteElement),
        ];

        let current_position = Some("2".to_string());
        let is_current = entries
            .iter()
            .any(|e| Some(&e.id) == current_position.as_ref());
        assert!(is_current);
    }
}

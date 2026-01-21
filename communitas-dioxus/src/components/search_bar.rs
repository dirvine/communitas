//! Message search component for finding messages across conversations.

use crate::tokens::colors;
use communitas_ui_api::SearchResult;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::error;

/// Props for the SearchBar component.
#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    /// Callback when a search result is selected.
    pub on_result_select: EventHandler<SearchResultSelection>,
    /// Optional thread ID to limit search scope.
    #[props(default)]
    pub thread_id: Option<String>,
    /// Placeholder text for the search input.
    #[props(default = "Search messages...".to_string())]
    pub placeholder: String,
}

/// Search result selection containing both thread and message info.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResultSelection {
    /// Thread ID where the message was found.
    pub thread_id: String,
    /// Message ID to scroll to.
    pub message_id: String,
}

/// Search bar with results dropdown.
#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchResult>::new);
    let mut searching = use_signal(|| false);
    let mut show_results = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Debounced search
    let thread_id = props.thread_id.clone();
    let do_search = {
        let services = services.clone();
        let thread_id = thread_id.clone();
        move || {
            let search_query = query();
            if search_query.trim().is_empty() {
                results.set(Vec::new());
                show_results.set(false);
                return;
            }

            if search_query.len() < 2 {
                // Don't search for very short queries
                return;
            }

            let services = services.clone();
            let thread_id = thread_id.clone();

            searching.set(true);
            error_msg.set(None);

            spawn(async move {
                let result = services
                    .messaging()
                    .search_messages(&search_query, thread_id.as_deref(), 20)
                    .await;

                match result {
                    Ok(search_results) => {
                        results.set(search_results);
                        show_results.set(true);
                    }
                    Err(e) => {
                        error!(target = "ui.search", event = "search_failed", error = %e);
                        error_msg.set(Some(e.to_string()));
                    }
                }
                searching.set(false);
            });
        }
    };

    let handle_input = {
        let mut do_search = do_search.clone();
        move |evt: Event<FormData>| {
            query.set(evt.value().clone());
            do_search();
        }
    };

    let handle_focus = move |_| {
        if !query().trim().is_empty() && !results().is_empty() {
            show_results.set(true);
        }
    };

    let clear_search = move |_| {
        query.set(String::new());
        results.set(Vec::new());
        show_results.set(false);
    };

    rsx! {
        div {
            class: "search-bar relative",
            // Search input container
            div {
                class: "search-input-container relative",
                // Search icon
                span {
                    class: "absolute left-3 top-1/2 -translate-y-1/2",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    aria_hidden: "true",
                    SearchIcon {}
                }
                // Input field
                input {
                    r#type: "search",
                    class: "w-full pl-10 pr-10 py-2 rounded-lg border focus:outline-none",
                    style: format!(
                        "border-color: {}; background-color: {}; color: {};",
                        colors::BORDER_DEFAULT,
                        colors::SURFACE_CARD,
                        colors::TEXT_PRIMARY
                    ),
                    placeholder: "{props.placeholder}",
                    value: "{query}",
                    oninput: handle_input,
                    onfocus: handle_focus,
                    aria_label: "Search messages",
                    aria_expanded: show_results(),
                    aria_haspopup: "listbox",
                }
                // Clear button (when has text)
                if !query().is_empty() {
                    button {
                        class: "absolute right-3 top-1/2 -translate-y-1/2 p-1 rounded-full hover:bg-opacity-10",
                        style: format!("color: {};", colors::TEXT_SECONDARY),
                        onclick: clear_search,
                        aria_label: "Clear search",
                        CloseIcon {}
                    }
                }
                // Loading spinner
                if searching() {
                    span {
                        class: "absolute right-10 top-1/2 -translate-y-1/2",
                        style: format!("color: {};", colors::TEXT_MUTED),
                        LoadingSpinner {}
                    }
                }
            }
            // Error message
            if let Some(err) = error_msg() {
                div {
                    class: "search-error mt-2 rounded-lg border px-3 py-2 text-sm",
                    style: format!(
                        "border-color: {}30; background-color: {}10; color: {};",
                        colors::DANGER,
                        colors::DANGER,
                        colors::DANGER
                    ),
                    role: "alert",
                    "{err}"
                }
            }
            // Results dropdown
            if show_results() && !results().is_empty() {
                div {
                    class: "search-results absolute top-full left-0 right-0 mt-1 max-h-80 overflow-y-auto rounded-lg border shadow-lg z-50",
                    style: format!(
                        "border-color: {}; background-color: {};",
                        colors::BORDER_DEFAULT,
                        colors::SURFACE_CARD
                    ),
                    role: "listbox",
                    aria_label: "Search results",
                    for (idx, result) in results().iter().enumerate() {
                        SearchResultItem {
                            key: "{result.message.id}",
                            result: result.clone(),
                            on_select: {
                                let thread_id = result.thread_id.clone();
                                let message_id = result.message.id.clone();
                                move |_| {
                                    props.on_result_select.call(SearchResultSelection {
                                        thread_id: thread_id.clone(),
                                        message_id: message_id.clone(),
                                    });
                                    show_results.set(false);
                                }
                            },
                            is_first: idx == 0,
                        }
                    }
                }
            }
            // No results message
            if show_results() && results().is_empty() && !searching() && !query().is_empty() {
                div {
                    class: "search-no-results absolute top-full left-0 right-0 mt-1 p-4 rounded-lg border text-center",
                    style: format!(
                        "border-color: {}; background-color: {}; color: {};",
                        colors::BORDER_DEFAULT,
                        colors::SURFACE_CARD,
                        colors::TEXT_SECONDARY
                    ),
                    "No messages found"
                }
            }
        }
    }
}

/// Props for a search result item.
#[derive(Props, Clone, PartialEq)]
struct SearchResultItemProps {
    result: SearchResult,
    on_select: EventHandler<()>,
    #[props(default)]
    is_first: bool,
}

/// Individual search result item.
#[component]
fn SearchResultItem(props: SearchResultItemProps) -> Element {
    let result = &props.result;
    let match_text = if result.match_count == 1 {
        "1 match".to_string()
    } else {
        format!("{} matches", result.match_count)
    };
    let excerpt_html = highlight_excerpt(&result.match_excerpt);
    let time_str = format_time(result.message.timestamp);

    rsx! {
        button {
            class: "search-result-item w-full text-left px-4 py-3 hover:bg-opacity-50 transition border-t first:border-t-0",
            style: format!(
                "border-color: {}; background-color: transparent;",
                colors::BORDER_DEFAULT
            ),
            onclick: move |_| props.on_select.call(()),
            role: "option",
            // Thread name
            div {
                class: "flex items-center gap-2 mb-1",
                span {
                    class: "text-xs font-medium",
                    style: format!("color: {};", colors::PRIMARY),
                    "{result.thread_name}"
                }
                span {
                    class: "text-xs",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    "{match_text}"
                }
            }
            // Message excerpt with highlighted match
            div {
                class: "text-sm",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                dangerous_inner_html: "{excerpt_html}",
            }
            // Sender and time
            div {
                class: "flex items-center gap-2 mt-1 text-xs",
                style: format!("color: {};", colors::TEXT_SECONDARY),
                span { "{result.message.sender_name}" }
                span { "·" }
                span { "{time_str}" }
            }
        }
    }
}

/// Highlight query matches in the excerpt with bold.
/// Note: Used in RSX but Rust's dead code analysis doesn't see through macro expansion.
#[allow(dead_code)]
fn highlight_excerpt(excerpt: &str) -> String {
    // The excerpt comes with ... and the query term - we can highlight the middle
    // For now, just escape HTML and show as-is
    excerpt
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format timestamp for display.
/// Note: Used in RSX but Rust's dead code analysis doesn't see through macro expansion.
#[allow(dead_code)]
fn format_time(ts: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let diff_ms = now.saturating_sub(ts);
    let diff_secs = diff_ms / 1000;
    let diff_mins = diff_secs / 60;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_mins < 1 {
        "just now".to_string()
    } else if diff_mins < 60 {
        format!("{diff_mins}m ago")
    } else if diff_hours < 24 {
        format!("{diff_hours}h ago")
    } else if diff_days < 7 {
        format!("{diff_days}d ago")
    } else {
        // Show date
        let dt = UNIX_EPOCH + Duration::from_millis(ts);
        let datetime: chrono::DateTime<chrono::Utc> = dt.into();
        datetime.format("%b %d").to_string()
    }
}

/// Search icon SVG.
#[component]
fn SearchIcon() -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "8" }
            line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
        }
    }
}

/// Close/X icon SVG.
#[component]
fn CloseIcon() -> Element {
    rsx! {
        svg {
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "18", y1: "6", x2: "6", y2: "18" }
            line { x1: "6", y1: "6", x2: "18", y2: "18" }
        }
    }
}

/// Loading spinner animation.
#[component]
fn LoadingSpinner() -> Element {
    rsx! {
        svg {
            width: "16",
            height: "16",
            view_box: "0 0 24 24",
            fill: "none",
            class: "animate-spin",
            circle {
                cx: "12",
                cy: "12",
                r: "10",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_dasharray: "31.4 31.4",
                stroke_dashoffset: "0",
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_time_just_now() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        assert_eq!(format_time(now), "just now");
    }

    #[test]
    fn test_format_time_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let five_min_ago = now - 5 * 60 * 1000;
        assert_eq!(format_time(five_min_ago), "5m ago");
    }

    #[test]
    fn test_highlight_excerpt_escapes_html() {
        let excerpt = "test <script>alert('xss')</script> test";
        let highlighted = highlight_excerpt(excerpt);
        assert!(!highlighted.contains('<'));
        assert!(highlighted.contains("&lt;"));
    }
}

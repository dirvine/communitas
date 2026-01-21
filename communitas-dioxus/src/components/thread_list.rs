//! Thread list sidebar component for messaging UI.

use crate::tokens::colors;
use communitas_ui_api::{ThreadSummary, UnifiedEntityType};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Filter for thread list display.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ThreadFilter {
    #[default]
    All,
    Entities,
    Contacts,
    Unread,
}

impl ThreadFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Entities => "Entities",
            Self::Contacts => "Contacts",
            Self::Unread => "Unread",
        }
    }
}

/// Props for the ThreadListSidebar component.
#[derive(Props, Clone, PartialEq)]
pub struct ThreadListSidebarProps {
    /// Callback when a thread is selected.
    pub on_thread_select: EventHandler<String>,
    /// Currently selected thread ID, if any.
    #[props(default)]
    pub selected_thread_id: Option<String>,
}

/// Thread list sidebar showing all conversation threads.
#[component]
pub fn ThreadListSidebar(props: ThreadListSidebarProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut threads = use_signal(Vec::<ThreadSummary>::new);
    let mut loading = use_signal(|| true);
    let mut filter = use_signal(ThreadFilter::default);

    // Subscribe to messaging updates
    let services_for_effect = services.clone();
    use_future(move || {
        let services = services_for_effect.clone();
        async move {
            let mut rx = services.messaging().subscribe();
            loop {
                {
                    let snap = rx.borrow().clone();
                    threads.set(snap.threads);
                    loading.set(snap.loading);
                }
                if rx.changed().await.is_err() {
                    break;
                }
            }
        }
    });

    let filtered = filter_threads(&threads(), filter());

    rsx! {
        div {
            class: "thread-list-sidebar flex flex-col h-full",
            role: "navigation",
            aria_label: "Conversations",
            // Filter tabs
            ThreadFilterTabs {
                current: filter(),
                on_change: move |f| filter.set(f),
            }
            // Thread list
            div {
                class: "thread-list flex-1 overflow-y-auto",
                role: "list",
                aria_label: "Thread list",
                if loading() {
                    ThreadListSkeleton {}
                } else if filtered.is_empty() {
                    EmptyThreadList { filter: filter() }
                } else {
                    for thread in filtered {
                        ThreadListItem {
                            key: "{thread.thread_id}",
                            thread: thread.clone(),
                            selected: props.selected_thread_id.as_ref() == Some(&thread.thread_id),
                            on_click: {
                                let thread_id = thread.thread_id.clone();
                                move |_| props.on_thread_select.call(thread_id.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

/// Filter tabs component.
#[derive(Props, Clone, PartialEq)]
struct ThreadFilterTabsProps {
    current: ThreadFilter,
    on_change: EventHandler<ThreadFilter>,
}

#[component]
fn ThreadFilterTabs(props: ThreadFilterTabsProps) -> Element {
    let filters = [
        ThreadFilter::All,
        ThreadFilter::Entities,
        ThreadFilter::Contacts,
        ThreadFilter::Unread,
    ];

    rsx! {
        div {
            class: "thread-filter-tabs flex gap-1 p-2 border-b",
            style: format!("border-color: {};", colors::BORDER_DEFAULT),
            role: "tablist",
            aria_label: "Filter threads",
            for f in filters {
                button {
                    class: "px-3 py-1.5 text-xs font-medium rounded-lg transition",
                    style: if props.current == f {
                        format!(
                            "background-color: {}20; color: {}; border: 1px solid {}30;",
                            colors::PRIMARY,
                            colors::PRIMARY,
                            colors::PRIMARY
                        )
                    } else {
                        format!(
                            "color: {}; background-color: transparent;",
                            colors::TEXT_SECONDARY
                        )
                    },
                    role: "tab",
                    aria_selected: props.current == f,
                    onclick: move |_| props.on_change.call(f),
                    "{f.label()}"
                }
            }
        }
    }
}

/// Props for a single thread list item.
#[derive(Props, Clone, PartialEq)]
struct ThreadListItemProps {
    thread: ThreadSummary,
    selected: bool,
    on_click: EventHandler<()>,
}

#[component]
fn ThreadListItem(props: ThreadListItemProps) -> Element {
    let thread = &props.thread;
    let type_icon = thread_type_icon(thread);
    let time_display = format_relative_time(thread.last_message_timestamp);

    rsx! {
        button {
            class: "thread-list-item w-full text-left px-3 py-3 border-b transition",
            style: format!(
                "border-color: {}50; {}",
                colors::BORDER_DEFAULT,
                if props.selected {
                    format!("background-color: {}10; border-left: 2px solid {};", colors::PRIMARY, colors::PRIMARY)
                } else {
                    "background-color: transparent;".to_string()
                }
            ),
            role: "listitem",
            aria_selected: props.selected,
            onclick: move |_| props.on_click.call(()),
            div {
                class: "flex items-start gap-3",
                // Avatar/icon
                div {
                    class: "flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center text-sm",
                    style: format!("background-color: {};", colors::SURFACE_ELEVATED),
                    "{type_icon}"
                }
                // Content
                div {
                    class: "flex-1 min-w-0",
                    div {
                        class: "flex items-center justify-between gap-2",
                        span {
                            class: "font-medium truncate",
                            style: format!(
                                "color: {};",
                                if thread.unread_count > 0 { colors::TEXT_PRIMARY } else { colors::TEXT_SECONDARY }
                            ),
                            "{thread.display_name}"
                        }
                        span {
                            class: "text-xs flex-shrink-0",
                            style: format!("color: {};", colors::TEXT_MUTED),
                            "{time_display}"
                        }
                    }
                    div {
                        class: "flex items-center justify-between gap-2 mt-0.5",
                        p {
                            class: "text-sm truncate",
                            style: format!(
                                "color: {};",
                                if thread.unread_count > 0 { colors::TEXT_SECONDARY } else { colors::TEXT_MUTED }
                            ),
                            "{thread.last_message_preview}"
                        }
                        if thread.unread_count > 0 {
                            UnreadBadge { count: thread.unread_count }
                        }
                    }
                }
            }
        }
    }
}

/// Unread message count badge.
#[derive(Props, Clone, PartialEq)]
struct UnreadBadgeProps {
    count: u32,
}

#[component]
fn UnreadBadge(props: UnreadBadgeProps) -> Element {
    let display = if props.count > 99 {
        "99+".to_string()
    } else {
        props.count.to_string()
    };

    rsx! {
        span {
            class: "flex-shrink-0 min-w-[1.25rem] h-5 px-1.5 rounded-full text-xs font-semibold flex items-center justify-center",
            style: format!("background-color: {}; color: {};", colors::PRIMARY, colors::TEXT_INVERSE),
            aria_label: format!("{} unread messages", props.count),
            "{display}"
        }
    }
}

/// Loading skeleton for thread list.
#[component]
fn ThreadListSkeleton() -> Element {
    rsx! {
        div {
            class: "animate-pulse",
            for _ in 0..5 {
                div {
                    class: "flex items-start gap-3 px-3 py-3 border-b",
                    style: format!("border-color: {}50;", colors::BORDER_DEFAULT),
                    div {
                        class: "w-10 h-10 rounded-full",
                        style: format!("background-color: {};", colors::SURFACE_ELEVATED),
                    }
                    div {
                        class: "flex-1",
                        div {
                            class: "h-4 w-32 rounded mb-2",
                            style: format!("background-color: {};", colors::SURFACE_ELEVATED),
                        }
                        div {
                            class: "h-3 w-48 rounded",
                            style: format!("background-color: {}60;", colors::SURFACE_ELEVATED),
                        }
                    }
                }
            }
        }
    }
}

/// Empty state for thread list.
#[derive(Props, Clone, PartialEq)]
struct EmptyThreadListProps {
    filter: ThreadFilter,
}

#[component]
fn EmptyThreadList(props: EmptyThreadListProps) -> Element {
    let message = match props.filter {
        ThreadFilter::All => "No conversations yet",
        ThreadFilter::Entities => "No entity conversations",
        ThreadFilter::Contacts => "No contact conversations",
        ThreadFilter::Unread => "All caught up!",
    };

    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-12 px-4 text-center",
            div {
                class: "w-16 h-16 rounded-full flex items-center justify-center mb-4",
                style: format!("background-color: {};", colors::SURFACE_ELEVATED),
                span { class: "text-2xl", "💬" }
            }
            p {
                class: "text-sm",
                style: format!("color: {};", colors::TEXT_SECONDARY),
                "{message}"
            }
            if props.filter == ThreadFilter::All {
                p {
                    class: "text-xs mt-1",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    "Start a conversation with a contact or entity"
                }
            }
        }
    }
}

/// Filter threads based on the current filter selection.
fn filter_threads(threads: &[ThreadSummary], filter: ThreadFilter) -> Vec<ThreadSummary> {
    threads
        .iter()
        .filter(|thread| match filter {
            ThreadFilter::All => true,
            ThreadFilter::Entities => thread.entity_id.is_some(),
            ThreadFilter::Contacts => thread.contact_id.is_some(),
            ThreadFilter::Unread => thread.unread_count > 0,
        })
        .cloned()
        .collect()
}

/// Get icon character for thread type.
fn thread_type_icon(thread: &ThreadSummary) -> &'static str {
    if let Some(entity_type) = thread.entity_type {
        match entity_type {
            UnifiedEntityType::Organization => "🏢",
            UnifiedEntityType::Project => "📁",
            UnifiedEntityType::Group => "👥",
            UnifiedEntityType::Channel => "#",
            UnifiedEntityType::Person => "👤",
        }
    } else if thread.contact_id.is_some() {
        "👤"
    } else {
        "💬"
    }
}

/// Format timestamp as relative time string.
fn format_relative_time(timestamp_ms: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let diff_ms = now.saturating_sub(timestamp_ms);
    let diff_secs = diff_ms / 1000;
    let diff_mins = diff_secs / 60;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_mins < 1 {
        "now".to_string()
    } else if diff_mins < 60 {
        format!("{}m", diff_mins)
    } else if diff_hours < 24 {
        format!("{}h", diff_hours)
    } else if diff_days < 7 {
        format!("{}d", diff_days)
    } else {
        // Format as date for older messages
        let secs = (timestamp_ms / 1000) as i64;
        // Simple date format - just show month/day
        let days_since_epoch = secs / 86400;
        let year = 1970 + (days_since_epoch / 365) as u32;
        let remaining = (days_since_epoch % 365) as u32;
        let month = (remaining / 30) + 1;
        let day = (remaining % 30) + 1;
        format!("{}/{}/{}", month.min(12), day.min(31), year % 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_threads_all_returns_all() {
        let threads = sample_threads();
        let filtered = filter_threads(&threads, ThreadFilter::All);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn filter_threads_entities_returns_entity_threads() {
        let threads = sample_threads();
        let filtered = filter_threads(&threads, ThreadFilter::Entities);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.entity_id.is_some()));
    }

    #[test]
    fn filter_threads_contacts_returns_contact_threads() {
        let threads = sample_threads();
        let filtered = filter_threads(&threads, ThreadFilter::Contacts);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.iter().all(|t| t.contact_id.is_some()));
    }

    #[test]
    fn filter_threads_unread_returns_unread_only() {
        let threads = sample_threads();
        let filtered = filter_threads(&threads, ThreadFilter::Unread);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.unread_count > 0));
    }

    #[test]
    fn format_relative_time_now() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        assert_eq!(format_relative_time(now), "now");
    }

    #[test]
    fn format_relative_time_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let five_min_ago = now.saturating_sub(5 * 60 * 1000);
        assert_eq!(format_relative_time(five_min_ago), "5m");
    }

    #[test]
    fn format_relative_time_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let three_hours_ago = now.saturating_sub(3 * 60 * 60 * 1000);
        assert_eq!(format_relative_time(three_hours_ago), "3h");
    }

    #[test]
    fn thread_type_icon_channel() {
        let thread = ThreadSummary {
            thread_id: "t1".to_string(),
            entity_id: Some("e1".to_string()),
            entity_type: Some(UnifiedEntityType::Channel),
            contact_id: None,
            display_name: "General".to_string(),
            last_message_preview: "Hello".to_string(),
            last_message_timestamp: 0,
            unread_count: 0,
            is_muted: false,
            is_dm: false,
            typing_users: vec![],
        };
        assert_eq!(thread_type_icon(&thread), "#");
    }

    #[test]
    fn thread_type_icon_contact() {
        let thread = ThreadSummary {
            thread_id: "t2".to_string(),
            entity_id: None,
            entity_type: None,
            contact_id: Some("alice".to_string()),
            display_name: "Alice".to_string(),
            last_message_preview: "Hi".to_string(),
            last_message_timestamp: 0,
            unread_count: 0,
            is_muted: false,
            is_dm: true,
            typing_users: vec![],
        };
        assert_eq!(thread_type_icon(&thread), "👤");
    }

    fn sample_threads() -> Vec<ThreadSummary> {
        vec![
            ThreadSummary {
                thread_id: "t1".to_string(),
                entity_id: Some("e1".to_string()),
                entity_type: Some(UnifiedEntityType::Channel),
                contact_id: None,
                display_name: "General".to_string(),
                last_message_preview: "Hello team".to_string(),
                last_message_timestamp: 1000,
                unread_count: 5,
                is_muted: false,
                is_dm: false,
                typing_users: vec![],
            },
            ThreadSummary {
                thread_id: "t2".to_string(),
                entity_id: Some("e2".to_string()),
                entity_type: Some(UnifiedEntityType::Group),
                contact_id: None,
                display_name: "Project Alpha".to_string(),
                last_message_preview: "Progress update".to_string(),
                last_message_timestamp: 2000,
                unread_count: 0,
                is_muted: false,
                is_dm: false,
                typing_users: vec![],
            },
            ThreadSummary {
                thread_id: "t3".to_string(),
                entity_id: None,
                entity_type: None,
                contact_id: Some("alice".to_string()),
                display_name: "Alice".to_string(),
                last_message_preview: "See you later!".to_string(),
                last_message_timestamp: 3000,
                unread_count: 2,
                is_muted: false,
                is_dm: true,
                typing_users: vec![],
            },
        ]
    }
}

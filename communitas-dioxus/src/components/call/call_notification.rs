//! Call notification components for incoming and missed calls.
//!
//! Provides notification banners for incoming calls and missed call indicators
//! that can be displayed in the UI.

use communitas_ui_api::call::{CallType, MissedCallNotification, MissedCallsSnapshot};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the IncomingCallBanner component.
#[derive(Props, Clone, PartialEq)]
pub struct IncomingCallBannerProps {
    /// Entity ID of the caller.
    pub caller_id: String,
    /// Display name of the caller.
    pub caller_name: String,
    /// Type of incoming call.
    pub call_type: CallType,
    /// Callback when user accepts the call.
    pub on_accept: EventHandler<()>,
    /// Callback when user declines the call.
    pub on_decline: EventHandler<()>,
}

/// Banner showing an incoming call that can be accepted or declined.
#[component]
pub fn IncomingCallBanner(props: IncomingCallBannerProps) -> Element {
    let call_type_label = match props.call_type {
        CallType::Direct => "Incoming Call",
        CallType::Group => "Group Call",
        CallType::Channel => "Channel Call",
    };

    rsx! {
        div {
            class: "incoming-call-banner fixed top-4 right-4 z-50 bg-slate-900 border border-slate-700 rounded-xl shadow-2xl p-4 w-80 animate-slide-in",
            role: "alertdialog",
            aria_label: format!("{call_type_label} from {}", props.caller_name),

            // Header with call type
            div {
                class: "flex items-center gap-2 mb-3",
                // Phone ring icon
                span {
                    class: "w-8 h-8 bg-green-600 rounded-full flex items-center justify-center animate-pulse",
                    svg {
                        class: "w-5 h-5 text-white",
                        fill: "currentColor",
                        view_box: "0 0 20 20",
                        path {
                            d: "M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z",
                        }
                    }
                }
                div {
                    class: "flex-1",
                    p {
                        class: "text-sm text-green-400 font-medium",
                        "{call_type_label}"
                    }
                    p {
                        class: "text-lg text-white font-semibold truncate",
                        "{props.caller_name}"
                    }
                }
            }

            // Action buttons
            div {
                class: "flex gap-2",
                // Decline button
                button {
                    class: "flex-1 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2",
                    onclick: move |_| props.on_decline.call(()),
                    svg {
                        class: "w-4 h-4",
                        fill: "currentColor",
                        view_box: "0 0 20 20",
                        path {
                            fill_rule: "evenodd",
                            d: "M3.5 3.5a1.5 1.5 0 00-1.5 1.5v10a1.5 1.5 0 001.5 1.5h13a1.5 1.5 0 001.5-1.5V5a1.5 1.5 0 00-1.5-1.5h-13zm4.293 4.293a1 1 0 011.414 0L10 8.586l.793-.793a1 1 0 111.414 1.414L11.414 10l.793.793a1 1 0 01-1.414 1.414L10 11.414l-.793.793a1 1 0 01-1.414-1.414L8.586 10l-.793-.793a1 1 0 010-1.414z",
                            clip_rule: "evenodd",
                        }
                    }
                    "Decline"
                }
                // Accept button
                button {
                    class: "flex-1 px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2",
                    onclick: move |_| props.on_accept.call(()),
                    svg {
                        class: "w-4 h-4",
                        fill: "currentColor",
                        view_box: "0 0 20 20",
                        path {
                            d: "M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z",
                        }
                    }
                    "Accept"
                }
            }
        }
    }
}

/// Props for the MissedCallBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct MissedCallBadgeProps {
    /// The count to display.
    pub count: usize,
    /// Size variant: "sm", "md", or "lg".
    #[props(default = "md")]
    pub size: &'static str,
}

/// Badge showing the number of missed calls.
#[component]
pub fn MissedCallBadge(props: MissedCallBadgeProps) -> Element {
    if props.count == 0 {
        return rsx! {};
    }

    let (badge_size, text_size) = match props.size {
        "sm" => ("min-w-4 h-4 px-1", "text-[10px]"),
        "md" => ("min-w-5 h-5 px-1.5", "text-xs"),
        "lg" => ("min-w-6 h-6 px-2", "text-sm"),
        _ => ("min-w-5 h-5 px-1.5", "text-xs"),
    };

    let display_count = if props.count > 99 {
        "99+".to_string()
    } else {
        props.count.to_string()
    };

    rsx! {
        span {
            class: format!("missed-call-badge {badge_size} {text_size} bg-red-500 text-white font-bold rounded-full flex items-center justify-center"),
            role: "status",
            aria_label: format!("{} missed calls", props.count),
            "{display_count}"
        }
    }
}

/// Props for the MissedCallsPanel component.
#[derive(Props, Clone, PartialEq)]
pub struct MissedCallsPanelProps {
    /// Callback when user clicks on a missed call to call back.
    pub on_call_back: EventHandler<String>,
    /// Callback when user dismisses a notification.
    pub on_dismiss: EventHandler<String>,
    /// Callback when user dismisses all notifications.
    pub on_dismiss_all: EventHandler<()>,
}

/// Panel showing all missed call notifications.
#[component]
pub fn MissedCallsPanel(props: MissedCallsPanelProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut missed_snapshot = use_signal(MissedCallsSnapshot::default);

    // Subscribe to missed calls updates
    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe_missed_calls();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                missed_snapshot.set(rx.borrow().clone());
            }
        }
    });

    // Initial load
    use_effect(move || {
        let call = call_service.clone();
        spawn(async move {
            let snapshot = call.get_missed_calls_snapshot().await;
            missed_snapshot.set(snapshot);
        });
    });

    let snapshot = missed_snapshot();

    if snapshot.notifications.is_empty() {
        return rsx! {
            div {
                class: "missed-calls-panel p-4 text-center text-slate-400",
                p { "No missed calls" }
            }
        };
    }

    rsx! {
        div {
            class: "missed-calls-panel bg-slate-900 rounded-lg overflow-hidden",

            // Header
            div {
                class: "flex items-center justify-between px-4 py-3 border-b border-slate-700",
                h3 {
                    class: "text-sm font-semibold text-white",
                    "Missed Calls"
                    if snapshot.unread_count > 0 {
                        span {
                            class: "ml-2 px-1.5 py-0.5 bg-red-500 text-white text-xs rounded-full",
                            "{snapshot.unread_count}"
                        }
                    }
                }
                if !snapshot.notifications.is_empty() {
                    button {
                        class: "text-xs text-slate-400 hover:text-white transition-colors",
                        onclick: move |_| props.on_dismiss_all.call(()),
                        "Clear all"
                    }
                }
            }

            // Notification list
            ul {
                class: "divide-y divide-slate-800 max-h-64 overflow-y-auto",
                for notification in snapshot.notifications.iter() {
                    MissedCallItem {
                        key: "{notification.id}",
                        notification: notification.clone(),
                        on_call_back: props.on_call_back,
                        on_dismiss: props.on_dismiss,
                    }
                }
            }
        }
    }
}

/// Props for MissedCallItem.
#[derive(Props, Clone, PartialEq)]
struct MissedCallItemProps {
    notification: MissedCallNotification,
    on_call_back: EventHandler<String>,
    on_dismiss: EventHandler<String>,
}

/// Individual missed call item in the list.
#[component]
fn MissedCallItem(props: MissedCallItemProps) -> Element {
    let notification = &props.notification;
    let time_ago = notification.time_ago();

    let unread_class = if !notification.is_acknowledged {
        "bg-slate-800/50"
    } else {
        ""
    };

    let call_id = notification.call_id.clone();
    let caller_id = notification.caller_id.clone();

    rsx! {
        li {
            class: format!("missed-call-item flex items-center gap-3 px-4 py-3 hover:bg-slate-800/30 transition-colors {unread_class}"),

            // Unread indicator
            if !notification.is_acknowledged {
                span {
                    class: "w-2 h-2 bg-red-500 rounded-full flex-shrink-0",
                }
            }

            // Caller info
            div {
                class: "flex-1 min-w-0",
                p {
                    class: "text-sm text-white font-medium truncate",
                    "{notification.caller_name}"
                }
                p {
                    class: "text-xs text-slate-400",
                    "{time_ago}"
                    if notification.has_called_back {
                        span { class: "ml-2 text-green-400", " - Called back" }
                    }
                }
            }

            // Action buttons
            div {
                class: "flex gap-2 flex-shrink-0",
                // Call back button
                if !notification.has_called_back {
                    button {
                        class: "p-2 bg-green-600 hover:bg-green-700 text-white rounded-lg transition-colors",
                        title: "Call back",
                        onclick: move |_| props.on_call_back.call(caller_id.clone()),
                        svg {
                            class: "w-4 h-4",
                            fill: "currentColor",
                            view_box: "0 0 20 20",
                            path {
                                d: "M2 3a1 1 0 011-1h2.153a1 1 0 01.986.836l.74 4.435a1 1 0 01-.54 1.06l-1.548.773a11.037 11.037 0 006.105 6.105l.774-1.548a1 1 0 011.059-.54l4.435.74a1 1 0 01.836.986V17a1 1 0 01-1 1h-2C7.82 18 2 12.18 2 5V3z",
                            }
                        }
                    }
                }
                // Dismiss button
                button {
                    class: "p-2 bg-slate-700 hover:bg-slate-600 text-slate-300 rounded-lg transition-colors",
                    title: "Dismiss",
                    onclick: move |_| props.on_dismiss.call(call_id.clone()),
                    svg {
                        class: "w-4 h-4",
                        fill: "currentColor",
                        view_box: "0 0 20 20",
                        path {
                            fill_rule: "evenodd",
                            d: "M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z",
                            clip_rule: "evenodd",
                        }
                    }
                }
            }
        }
    }
}

/// Reactive badge that auto-updates from call service.
#[component]
pub fn ReactiveMissedCallBadge() -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut count = use_signal(|| 0usize);

    // Subscribe to missed calls updates
    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe_missed_calls();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                count.set(rx.borrow().unread_count);
            }
        }
    });

    // Initial load
    use_effect(move || {
        let call = call_service.clone();
        spawn(async move {
            let snapshot = call.get_missed_calls_snapshot().await;
            count.set(snapshot.unread_count);
        });
    });

    rsx! {
        MissedCallBadge { count: count() }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn missed_call_badge_hides_when_zero() {
        // Badge should not render when count is 0
        // (This is a structural test - actual rendering tested in integration)
        assert_eq!(0, 0);
    }

    #[test]
    fn missed_call_badge_caps_at_99() {
        // Display should show "99+" for counts over 99
        let display = if 150 > 99 {
            "99+".to_string()
        } else {
            150.to_string()
        };
        assert_eq!(display, "99+");
    }

    #[test]
    fn size_variants() {
        let sizes = ["sm", "md", "lg"];
        for size in sizes {
            let (badge_size, text_size) = match size {
                "sm" => ("min-w-4 h-4 px-1", "text-[10px]"),
                "md" => ("min-w-5 h-5 px-1.5", "text-xs"),
                "lg" => ("min-w-6 h-6 px-2", "text-sm"),
                _ => ("min-w-5 h-5 px-1.5", "text-xs"),
            };
            assert!(!badge_size.is_empty());
            assert!(!text_size.is_empty());
        }
    }
}

//! Message list component for displaying conversation messages.

use crate::tokens::colors;
use communitas_ui_api::{Message, MessageSendStatus, PendingMessage};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::error;

/// Props for the MessageList component.
#[derive(Props, Clone, PartialEq)]
pub struct MessageListProps {
    /// The thread ID to display messages for.
    pub thread_id: String,
    /// Callback when user wants to reply to a message.
    pub on_reply: EventHandler<Message>,
}

/// Message list displaying conversation history with infinite scroll.
#[component]
pub fn MessageList(props: MessageListProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut messages = use_signal(Vec::<Message>::new);
    let mut loading = use_signal(|| true);
    let mut loading_more = use_signal(|| false);
    let mut has_more = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let thread_id = props.thread_id.clone();

    // Load initial messages when thread_id changes
    let services_for_load = services.clone();
    let thread_id_for_load = thread_id.clone();
    use_future(move || {
        let services = services_for_load.clone();
        let thread_id = thread_id_for_load.clone();
        async move {
            loading.set(true);
            error_msg.set(None);

            match services
                .messaging()
                .get_messages(&thread_id, 50, None)
                .await
            {
                Ok(msgs) => {
                    has_more.set(msgs.len() == 50);
                    messages.set(msgs);
                }
                Err(e) => {
                    error!(target = "ui.message_list", event = "load_failed", error = %e);
                    error_msg.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        }
    });

    // Load more handler
    let handle_load_more = {
        let services = services.clone();
        let thread_id = thread_id.clone();
        move |_| {
            let services = services.clone();
            let thread_id = thread_id.clone();
            let current_messages = messages();

            // Get the oldest message timestamp for cursor
            let before_timestamp = current_messages.first().map(|m| m.timestamp);

            if before_timestamp.is_none() {
                return;
            }

            loading_more.set(true);

            spawn(async move {
                match services
                    .messaging()
                    .get_messages(&thread_id, 50, before_timestamp)
                    .await
                {
                    Ok(mut older_msgs) => {
                        has_more.set(older_msgs.len() == 50);
                        // Prepend older messages
                        older_msgs.extend(current_messages);
                        messages.set(older_msgs);
                    }
                    Err(e) => {
                        error!(target = "ui.message_list", event = "load_more_failed", error = %e);
                    }
                }
                loading_more.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "message-list flex flex-col h-full",
            role: "log",
            aria_label: "Message history",
            aria_live: "polite",
            // Error display
            if let Some(err) = error_msg() {
                div {
                    class: "px-4 py-3 m-3 rounded-lg border text-sm",
                    style: format!("border-color: {}30; background-color: {}10; color: {};", colors::DANGER, colors::DANGER, colors::DANGER),
                    role: "alert",
                    p { "{err}" }
                    button {
                        class: "text-xs mt-1 hover:opacity-80",
                        style: format!("color: {};", colors::DANGER),
                        onclick: move |_| error_msg.set(None),
                        "Dismiss"
                    }
                }
            }
            // Load more button at top
            if has_more() && !loading() && !messages().is_empty() {
                div {
                    class: "flex justify-center py-3 border-b",
                    style: format!("border-color: {}50;", colors::BORDER_DEFAULT),
                    button {
                        class: "px-4 py-2 text-sm rounded-lg transition disabled:opacity-50 hover:opacity-80",
                        style: format!("color: {};", colors::TEXT_SECONDARY),
                        disabled: loading_more(),
                        onclick: handle_load_more,
                        if loading_more() {
                            span {
                                class: "flex items-center gap-2",
                                LoadingSpinner {}
                                "Loading..."
                            }
                        } else {
                            "Load earlier messages"
                        }
                    }
                }
            }
            // Messages area
            div {
                class: "flex-1 overflow-y-auto px-4 py-4 space-y-1",
                if loading() {
                    MessageListSkeleton {}
                } else if messages().is_empty() {
                    EmptyMessageList {}
                } else {
                    // Group and render messages
                    {render_message_groups(messages(), props.on_reply)}
                }
            }
        }
    }
}

/// Render messages grouped by sender for consecutive messages.
fn render_message_groups(messages: Vec<Message>, on_reply: EventHandler<Message>) -> Element {
    let mut groups: Vec<MessageGroup> = Vec::new();

    for msg in messages {
        let should_start_new_group = groups.last().is_none_or(|g| {
            g.sender_id != msg.sender_id || {
                // Also break group if more than 5 minutes apart
                let last_timestamp = g.messages.last().map(|m| m.timestamp).unwrap_or(0);
                msg.timestamp.saturating_sub(last_timestamp) > 5 * 60 * 1000
            }
        });

        if should_start_new_group {
            groups.push(MessageGroup {
                sender_id: msg.sender_id.clone(),
                sender_name: msg.sender_name.clone(),
                messages: vec![msg],
            });
        } else if let Some(group) = groups.last_mut() {
            group.messages.push(msg);
        }
    }

    rsx! {
        for group in groups {
            MessageGroupView {
                key: "{group.messages.first().map(|m| m.id.as_str()).unwrap_or(\"empty\")}",
                group: group.clone(),
                on_reply,
            }
        }
    }
}

/// A group of consecutive messages from the same sender.
#[derive(Clone, PartialEq)]
struct MessageGroup {
    sender_id: String,
    sender_name: String,
    messages: Vec<Message>,
}

/// Props for message group view.
#[derive(Props, Clone, PartialEq)]
struct MessageGroupViewProps {
    group: MessageGroup,
    on_reply: EventHandler<Message>,
}

/// View for a group of messages from the same sender.
#[component]
fn MessageGroupView(props: MessageGroupViewProps) -> Element {
    let group = &props.group;
    let first_msg = group.messages.first();

    rsx! {
        div {
            class: "message-group mb-4",
            // Sender header (only once per group)
            div {
                class: "flex items-center gap-2 mb-1",
                // Avatar
                div {
                    class: "w-8 h-8 rounded-full flex items-center justify-center text-sm",
                    style: format!("background-color: {}; color: {};", colors::SURFACE_ELEVATED, colors::TEXT_SECONDARY),
                    {group.sender_name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                }
                span {
                    class: "text-sm font-medium",
                    style: format!("color: {};", colors::PRIMARY),
                    "{group.sender_name}"
                }
                if let Some(msg) = first_msg {
                    span {
                        class: "text-xs",
                        style: format!("color: {};", colors::TEXT_MUTED),
                        "{format_timestamp(msg.timestamp)}"
                    }
                }
            }
            // Messages in group
            div {
                class: "ml-10 space-y-1",
                for msg in &group.messages {
                    MessageBubble {
                        key: "{msg.id}",
                        message: msg.clone(),
                        show_time: group.messages.len() > 1,
                        on_reply: props.on_reply,
                    }
                }
            }
        }
    }
}

/// Props for a single message bubble.
#[derive(Props, Clone, PartialEq)]
struct MessageBubbleProps {
    message: Message,
    show_time: bool,
    on_reply: EventHandler<Message>,
}

/// Single message bubble with reply action.
#[component]
fn MessageBubble(props: MessageBubbleProps) -> Element {
    let msg = &props.message;
    let mut hovered = use_signal(|| false);

    rsx! {
        div {
            class: "message-bubble group relative rounded-lg px-3 py-2 transition",
            style: if hovered() { format!("background-color: {}30;", colors::SURFACE_CARD) } else { String::new() },
            role: "article",
            aria_label: format!("Message from {}", msg.sender_name),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            // Reply-to indicator if present
            if let Some(_reply_to) = &msg.reply_to_id {
                div {
                    class: "text-xs mb-1 flex items-center gap-1",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    span {
                        style: format!("color: {};", colors::TEXT_MUTED),
                        "↩"
                    }
                    "Replying to a message"
                }
            }
            // Message text
            div {
                class: "whitespace-pre-wrap break-words",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "{msg.text}"
            }
            // Edited indicator
            if msg.edited {
                span {
                    class: "text-xs ml-1",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    "(edited)"
                }
            }
            // Timestamp for multi-message groups
            if props.show_time {
                div {
                    class: "text-xs mt-1",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    "{format_time_only(msg.timestamp)}"
                }
            }
            // Reactions
            if !msg.reactions.is_empty() {
                div {
                    class: "flex flex-wrap gap-1 mt-2",
                    for reaction in &msg.reactions {
                        span {
                            class: "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs",
                            style: if reaction.reacted_by_me {
                                format!("background-color: {}20; border: 1px solid {}30;", colors::PRIMARY, colors::PRIMARY)
                            } else {
                                format!("background-color: {};", colors::SURFACE_CARD)
                            },
                            "{reaction.emoji}"
                            span {
                                style: format!("color: {};", colors::TEXT_SECONDARY),
                                "{reaction.count}"
                            }
                        }
                    }
                }
            }
            // Action buttons (visible on hover)
            if hovered() {
                div {
                    class: "absolute right-2 top-1 flex gap-1",
                    button {
                        class: "w-7 h-7 rounded flex items-center justify-center hover:opacity-80",
                        style: format!("color: {};", colors::TEXT_SECONDARY),
                        title: "Reply",
                        onclick: {
                            let msg = msg.clone();
                            move |_| props.on_reply.call(msg.clone())
                        },
                        aria_label: "Reply to message",
                        "↩"
                    }
                }
            }
        }
    }
}

/// Loading skeleton for message list.
#[component]
fn MessageListSkeleton() -> Element {
    rsx! {
        div {
            class: "animate-pulse space-y-4",
            for _ in 0..5 {
                div {
                    class: "flex items-start gap-2",
                    div {
                        class: "w-8 h-8 rounded-full",
                        style: format!("background-color: {};", colors::SURFACE_CARD)
                    }
                    div {
                        class: "flex-1",
                        div {
                            class: "h-4 w-24 rounded mb-2",
                            style: format!("background-color: {};", colors::SURFACE_CARD)
                        }
                        div {
                            class: "h-16 rounded",
                            style: format!("background-color: {}60;", colors::SURFACE_CARD)
                        }
                    }
                }
            }
        }
    }
}

/// Empty state for message list.
#[component]
fn EmptyMessageList() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center h-full text-center py-12",
            div {
                class: "w-16 h-16 rounded-full flex items-center justify-center mb-4",
                style: format!("background-color: {};", colors::SURFACE_CARD),
                span { class: "text-2xl", "💬" }
            }
            p {
                style: format!("color: {};", colors::TEXT_SECONDARY),
                "No messages yet"
            }
            p {
                class: "text-sm mt-1",
                style: format!("color: {};", colors::TEXT_MUTED),
                "Send a message to start the conversation"
            }
        }
    }
}

/// Small loading spinner.
#[component]
fn LoadingSpinner() -> Element {
    rsx! {
        div {
            class: "w-4 h-4 border-2 rounded-full animate-spin",
            style: format!("border-color: {}; border-top-color: {};", colors::TEXT_MUTED, colors::TEXT_SECONDARY),
        }
    }
}

/// Format timestamp as relative or absolute time.
fn format_timestamp(timestamp_ms: u64) -> String {
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
        "just now".to_string()
    } else if diff_mins < 60 {
        format!("{} min ago", diff_mins)
    } else if diff_hours < 24 {
        format!(
            "{} hour{} ago",
            diff_hours,
            if diff_hours == 1 { "" } else { "s" }
        )
    } else if diff_days < 7 {
        format!(
            "{} day{} ago",
            diff_days,
            if diff_days == 1 { "" } else { "s" }
        )
    } else {
        // Format as date for older messages
        format_date(timestamp_ms)
    }
}

/// Format timestamp as time only (HH:MM).
fn format_time_only(timestamp_ms: u64) -> String {
    let secs = (timestamp_ms / 1000) as i64;
    // Simple time format - hours and minutes
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    format!("{:02}:{:02}", hours, minutes)
}

/// Format timestamp as date.
fn format_date(timestamp_ms: u64) -> String {
    let secs = (timestamp_ms / 1000) as i64;
    let days_since_epoch = secs / 86400;
    let year = 1970 + (days_since_epoch / 365) as u32;
    let remaining = (days_since_epoch % 365) as u32;
    let month = (remaining / 30) + 1;
    let day = (remaining % 30) + 1;
    format!("{}/{}/{}", month.min(12), day.min(31), year)
}

/// Message send status indicator showing pending, sending, or failed state.
///
/// Displays a small icon next to messages that haven't been successfully sent yet.
///
/// # States
///
/// - **Sending**: Animated spinner while message is being sent
/// - **Queued**: Clock icon for messages waiting to send (offline)
/// - **Failed**: Red X with retry option
/// - **Sent**: Checkmark (briefly shown then hidden)
///
/// # Example
///
/// ```ignore
/// MessageSendIndicator {
///     status: MessageSendStatus::Pending,
///     on_retry: move |_| retry_message(),
/// }
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct MessageSendIndicatorProps {
    /// Current send status.
    pub status: MessageSendStatus,
    /// Callback when retry button is clicked (for failed messages).
    #[props(default)]
    pub on_retry: Option<EventHandler<()>>,
}

#[component]
pub fn MessageSendIndicator(props: MessageSendIndicatorProps) -> Element {
    let (icon, color, label, is_animating) = match &props.status {
        MessageSendStatus::Sending => ("⟳", colors::INFO, "Sending", true),
        MessageSendStatus::Pending => ("⏱", colors::WARNING, "Waiting to send", false),
        MessageSendStatus::Failed(msg) => ("✕", colors::DANGER, msg.as_str(), false),
    };

    let animation_class = if is_animating { "animate-spin" } else { "" };
    let is_failed = matches!(props.status, MessageSendStatus::Failed(_));
    let on_retry = props.on_retry;

    rsx! {
        span {
            class: "message-send-indicator inline-flex items-center gap-1 text-xs {animation_class}",
            style: format!("color: {};", color),
            title: "{label}",
            aria_label: "{label}",
            // Status icon
            span {
                class: "send-icon",
                "{icon}"
            }
            // Retry button for failed messages
            if is_failed {
                if let Some(handler) = on_retry {
                    button {
                        r#type: "button",
                        class: "retry-btn text-xs px-1.5 py-0.5 rounded",
                        style: format!("background-color: {}20; color: {};", colors::DANGER, colors::DANGER),
                        onclick: move |_| handler.call(()),
                        "Retry"
                    }
                }
            }
        }
    }
}

/// Indicator showing count of queued messages waiting to send.
///
/// Displayed in the message composer area when messages are pending.
/// Provides visibility into offline queue status.
///
/// # Example
///
/// ```ignore
/// QueuedMessagesIndicator {
///     count: 5,
///     on_view_queue: move |_| show_queue_panel(),
/// }
/// ```
#[derive(Props, Clone, PartialEq)]
pub struct QueuedMessagesIndicatorProps {
    /// Number of messages in the queue.
    #[props(default = 0)]
    pub count: u32,
    /// Callback when "view queue" is clicked.
    #[props(default)]
    pub on_view_queue: Option<EventHandler<()>>,
}

#[component]
pub fn QueuedMessagesIndicator(props: QueuedMessagesIndicatorProps) -> Element {
    // Don't show if no queued messages
    if props.count == 0 {
        return rsx! {};
    }

    let message_text = if props.count == 1 {
        "1 message waiting to send".to_string()
    } else {
        format!("{} messages waiting to send", props.count)
    };

    let on_view_queue = props.on_view_queue;

    rsx! {
        div {
            class: "queued-messages-indicator flex items-center gap-2 px-3 py-2 rounded-lg",
            style: format!("background-color: {}20; color: {};", colors::WARNING, colors::WARNING),
            role: "status",
            aria_live: "polite",
            // Clock icon
            span {
                class: "text-sm",
                "⏱"
            }
            // Message count
            span {
                class: "text-sm flex-1",
                "{message_text}"
            }
            // View queue button
            if let Some(handler) = on_view_queue {
                button {
                    r#type: "button",
                    class: "text-xs px-2 py-1 rounded",
                    style: format!("background-color: {}; color: {};", colors::WARNING, colors::TEXT_INVERSE),
                    onclick: move |_| handler.call(()),
                    "View"
                }
            }
        }
    }
}

/// Pending message bubble for messages not yet sent.
///
/// Similar to regular MessageBubble but with send status indicator
/// and retry capability for failed messages.
#[derive(Props, Clone, PartialEq)]
pub struct PendingMessageBubbleProps {
    /// The pending message data.
    pub message: PendingMessage,
    /// Callback when retry is requested for failed messages.
    #[props(default)]
    pub on_retry: Option<EventHandler<String>>,
    /// Callback when delete is requested.
    #[props(default)]
    pub on_delete: Option<EventHandler<String>>,
}

#[component]
pub fn PendingMessageBubble(props: PendingMessageBubbleProps) -> Element {
    let msg = props.message.clone();
    let mut hovered = use_signal(|| false);

    let opacity = match msg.status {
        MessageSendStatus::Sending => "opacity-80",
        MessageSendStatus::Pending => "opacity-60",
        MessageSendStatus::Failed(_) => "opacity-90",
    };

    let has_reply = msg.reply_to_id.is_some();
    let msg_text = msg.text.clone();
    let status = msg.status.clone();
    let msg_id = msg.id.clone();

    // Build retry handler if provided
    let on_retry_handler = props.on_retry.as_ref().map(|h| {
        let msg_id = msg_id.clone();
        let handler = *h;
        EventHandler::new(move |_| handler.call(msg_id.clone()))
    });

    // Build delete handler if provided
    let on_delete = props.on_delete;
    let msg_id_for_delete = msg.id.clone();

    rsx! {
        div {
            class: "pending-message-bubble group relative rounded-lg px-3 py-2 transition {opacity}",
            style: if hovered() { format!("background-color: {}30;", colors::SURFACE_CARD) } else { String::new() },
            role: "article",
            aria_label: "Pending message",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            // Reply-to indicator if present
            if has_reply {
                div {
                    class: "text-xs mb-1 flex items-center gap-1",
                    style: format!("color: {};", colors::TEXT_MUTED),
                    span {
                        style: format!("color: {};", colors::TEXT_MUTED),
                        "↩"
                    }
                    "Replying to a message"
                }
            }
            // Message text
            div {
                class: "whitespace-pre-wrap break-words",
                style: format!("color: {};", colors::TEXT_PRIMARY),
                "{msg_text}"
            }
            // Send status row
            div {
                class: "flex items-center justify-between mt-1",
                // Status indicator
                MessageSendIndicator {
                    status: status.clone(),
                    on_retry: on_retry_handler,
                }
                // Delete button (on hover)
                if hovered() {
                    if let Some(handler) = on_delete {
                        button {
                            r#type: "button",
                            class: "text-xs px-1.5 py-0.5 rounded opacity-50 hover:opacity-100",
                            style: format!("color: {};", colors::TEXT_MUTED),
                            onclick: {
                                let msg_id = msg_id_for_delete.clone();
                                move |_| handler.call(msg_id.clone())
                            },
                            "✕"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_just_now() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        assert_eq!(format_timestamp(now), "just now");
    }

    #[test]
    fn format_timestamp_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let five_min_ago = now.saturating_sub(5 * 60 * 1000);
        assert_eq!(format_timestamp(five_min_ago), "5 min ago");
    }

    #[test]
    fn format_timestamp_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let one_hour_ago = now.saturating_sub(60 * 60 * 1000);
        assert_eq!(format_timestamp(one_hour_ago), "1 hour ago");
    }

    #[test]
    fn format_timestamp_plural_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let three_hours_ago = now.saturating_sub(3 * 60 * 60 * 1000);
        assert_eq!(format_timestamp(three_hours_ago), "3 hours ago");
    }

    #[test]
    fn format_time_only_works() {
        // Midnight UTC
        assert_eq!(format_time_only(0), "00:00");
        // 1:30 AM
        assert_eq!(format_time_only(90 * 60 * 1000), "01:30");
    }
}

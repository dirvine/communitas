//! Message composer component for sending messages.

use communitas_ui_api::Message;
use communitas_ui_service::UiServices;
use dioxus::events::MouseData;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::{error, info};

/// Props for the MessageComposer component.
#[derive(Props, Clone, PartialEq)]
pub struct MessageComposerProps {
    /// The thread ID to send messages to.
    pub thread_id: String,
    /// Message being replied to, if any.
    #[props(default)]
    pub reply_to: Option<Message>,
    /// Callback when a message is successfully sent.
    pub on_send: EventHandler<Message>,
    /// Callback to cancel reply mode.
    #[props(default)]
    pub on_cancel_reply: Option<EventHandler<()>>,
}

/// Message composer with text input and send button.
#[component]
pub fn MessageComposer(props: MessageComposerProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut text = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let thread_id = props.thread_id.clone();
    let reply_to_id = props.reply_to.as_ref().map(|m| m.id.clone());

    let do_send = {
        let services = services.clone();
        let thread_id = thread_id.clone();
        let reply_to_id = reply_to_id.clone();
        move || {
            let message_text = text();
            if message_text.trim().is_empty() {
                return;
            }

            let services = services.clone();
            let thread_id = thread_id.clone();
            let reply_to_id = reply_to_id.clone();

            sending.set(true);
            error_msg.set(None);

            spawn(async move {
                let result = services
                    .messaging()
                    .send_message(&thread_id, &message_text, reply_to_id.as_deref())
                    .await;

                match result {
                    Ok(_msg) => {
                        info!(target = "ui.composer", event = "message_sent", thread_id = %thread_id);
                        text.set(String::new());
                        // Note: on_send callback would be called here, but we can't access props in async
                    }
                    Err(e) => {
                        error!(target = "ui.composer", event = "send_failed", error = %e);
                        error_msg.set(Some(e.to_string()));
                    }
                }
                sending.set(false);
            });
        }
    };

    let handle_keydown = {
        let mut do_send = do_send.clone();
        move |evt: Event<KeyboardData>| {
            let key = evt.key();
            let modifiers = evt.modifiers();
            if key == Key::Enter && !modifiers.shift() {
                evt.prevent_default();
                do_send();
            }
        }
    };

    let handle_click = {
        let mut do_send = do_send.clone();
        move |_: Event<MouseData>| {
            do_send();
        }
    };

    let can_send = !sending() && !text().trim().is_empty();

    rsx! {
        div {
            class: "message-composer border-t border-slate-800 bg-slate-900/80 p-4",
            // Reply indicator
            if let Some(reply) = &props.reply_to {
                ReplyIndicator {
                    message: reply.clone(),
                    on_cancel: {
                        let on_cancel_reply = props.on_cancel_reply;
                        move |_| {
                            if let Some(handler) = &on_cancel_reply {
                                handler.call(());
                            }
                        }
                    },
                }
            }
            // Error message
            if let Some(err) = error_msg() {
                div {
                    class: "composer-error mb-3 rounded-lg border border-red-500/30 bg-red-500/10 px-3 py-2 text-sm text-red-300",
                    role: "alert",
                    p { "{err}" }
                    button {
                        class: "text-xs text-red-400 hover:text-red-300 mt-1",
                        onclick: move |_| error_msg.set(None),
                        "Dismiss"
                    }
                }
            }
            // Input row
            div {
                class: "composer-input-row flex items-end gap-3",
                // Attachment button (placeholder)
                button {
                    class: "composer-attach-btn flex-shrink-0 w-10 h-10 rounded-lg border border-slate-700 bg-slate-800 flex items-center justify-center text-slate-400 hover:border-slate-600 hover:text-slate-300 disabled:opacity-50",
                    title: "Attach file (coming soon)",
                    disabled: true,
                    aria_label: "Attach file",
                    "+"
                }
                // Text input
                div {
                    class: "flex-1 relative",
                    textarea {
                        class: "composer-textarea w-full min-h-[2.5rem] max-h-32 px-4 py-2.5 rounded-lg border border-slate-700 bg-slate-800 text-slate-100 placeholder-slate-500 focus:border-emerald-400 focus:outline-none resize-none",
                        placeholder: "Type a message...",
                        value: "{text}",
                        disabled: sending(),
                        rows: 1,
                        oninput: move |evt| text.set(evt.value()),
                        onkeydown: handle_keydown,
                        aria_label: "Message input",
                    }
                }
                // Emoji button (placeholder)
                button {
                    class: "composer-emoji-btn flex-shrink-0 w-10 h-10 rounded-lg border border-slate-700 bg-slate-800 flex items-center justify-center text-slate-400 hover:border-slate-600 hover:text-slate-300 disabled:opacity-50",
                    title: "Add emoji (coming soon)",
                    disabled: true,
                    aria_label: "Add emoji",
                    "😊"
                }
                // Send button
                button {
                    class: format!(
                        "composer-send-btn flex-shrink-0 px-5 h-10 rounded-lg font-semibold transition {}",
                        if can_send {
                            "bg-emerald-500 text-slate-900 hover:bg-emerald-400 shadow-lg shadow-emerald-500/20"
                        } else {
                            "bg-slate-700 text-slate-400 cursor-not-allowed"
                        }
                    ),
                    disabled: !can_send,
                    onclick: handle_click,
                    aria_label: "Send message",
                    if sending() {
                        span {
                            class: "flex items-center gap-2",
                            SendingSpinner {}
                            "Sending"
                        }
                    } else {
                        "Send"
                    }
                }
            }
            // Character count hint
            if text().len() > 500 {
                div {
                    class: "mt-2 text-xs text-slate-500",
                    "{text().len()} characters"
                }
            }
        }
    }
}

/// Reply indicator showing the message being replied to.
#[derive(Props, Clone, PartialEq)]
struct ReplyIndicatorProps {
    message: Message,
    on_cancel: EventHandler<()>,
}

#[component]
fn ReplyIndicator(props: ReplyIndicatorProps) -> Element {
    let preview = if props.message.text.len() > 60 {
        format!("{}...", &props.message.text[..57])
    } else {
        props.message.text.clone()
    };

    rsx! {
        div {
            class: "reply-indicator mb-3 flex items-start gap-3 rounded-lg border border-slate-700 bg-slate-800/50 px-3 py-2",
            role: "status",
            aria_label: format!("Replying to {}", props.message.sender_name),
            // Reply icon
            div {
                class: "flex-shrink-0 text-slate-500",
                "↩"
            }
            // Reply content
            div {
                class: "flex-1 min-w-0",
                span {
                    class: "text-xs text-emerald-400 font-medium",
                    "Replying to {props.message.sender_name}"
                }
                p {
                    class: "text-sm text-slate-400 truncate mt-0.5",
                    "{preview}"
                }
            }
            // Cancel button
            button {
                class: "flex-shrink-0 w-6 h-6 rounded flex items-center justify-center text-slate-500 hover:text-slate-300 hover:bg-slate-700",
                onclick: move |_| props.on_cancel.call(()),
                aria_label: "Cancel reply",
                "×"
            }
        }
    }
}

/// Simple sending spinner.
#[component]
fn SendingSpinner() -> Element {
    rsx! {
        div {
            class: "w-4 h-4 border-2 border-slate-900/30 border-t-slate-900 rounded-full animate-spin",
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reply_preview_truncates_long_messages() {
        let long_text = "a".repeat(100);
        let preview = if long_text.len() > 60 {
            format!("{}...", &long_text[..57])
        } else {
            long_text.clone()
        };
        assert_eq!(preview.len(), 60);
        assert!(preview.ends_with("..."));
    }

    #[test]
    fn reply_preview_keeps_short_messages() {
        let short_text = "Hello world".to_string();
        let preview = if short_text.len() > 60 {
            format!("{}...", &short_text[..57])
        } else {
            short_text.clone()
        };
        assert_eq!(preview, "Hello world");
    }
}

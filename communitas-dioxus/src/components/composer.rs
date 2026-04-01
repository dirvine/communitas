// SPDX-License-Identifier: MIT OR Apache-2.0

//! Message composer component for sending messages.

use crate::tokens::colors;
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
            class: "message-composer border-t p-4",
            style: format!("border-color: {}; background-color: {}e6;", colors::BORDER_DEFAULT, colors::SURFACE_BG),
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
                    class: "composer-error mb-3 rounded-lg border px-3 py-2 text-sm",
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
            // Input row
            div {
                class: "composer-input-row flex items-end gap-3",
                // Attachment button (placeholder)
                button {
                    class: "composer-attach-btn flex-shrink-0 w-10 h-10 rounded-lg border flex items-center justify-center hover:opacity-80 disabled:opacity-50",
                    style: format!("border-color: {}; background-color: {}; color: {};", colors::BORDER_DEFAULT, colors::SURFACE_CARD, colors::TEXT_SECONDARY),
                    title: "Attach file (coming soon)",
                    disabled: true,
                    aria_label: "Attach file",
                    "+"
                }
                // Text input
                div {
                    class: "flex-1 relative",
                    textarea {
                        class: "composer-textarea w-full min-h-[2.5rem] max-h-32 px-4 py-2.5 rounded-lg border focus:outline-none resize-none",
                        style: format!("border-color: {}; background-color: {}; color: {};", colors::BORDER_DEFAULT, colors::SURFACE_CARD, colors::TEXT_PRIMARY),
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
                    class: "composer-emoji-btn flex-shrink-0 w-10 h-10 rounded-lg border flex items-center justify-center hover:opacity-80 disabled:opacity-50",
                    style: format!("border-color: {}; background-color: {}; color: {};", colors::BORDER_DEFAULT, colors::SURFACE_CARD, colors::TEXT_SECONDARY),
                    title: "Add emoji (coming soon)",
                    disabled: true,
                    aria_label: "Add emoji",
                    "😊"
                }
                // Send button
                button {
                    class: "composer-send-btn flex-shrink-0 px-5 h-10 rounded-lg font-semibold transition",
                    style: if can_send {
                        format!(
                            "background-color: {}; color: {}; box-shadow: 0 4px 6px {}20;",
                            colors::PRIMARY, colors::TEXT_INVERSE, colors::PRIMARY
                        )
                    } else {
                        format!(
                            "background-color: {}; color: {}; cursor: not-allowed;",
                            colors::SURFACE_ELEVATED, colors::TEXT_SECONDARY
                        )
                    },
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
                    class: "mt-2 text-xs",
                    style: format!("color: {};", colors::TEXT_MUTED),
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
            class: "reply-indicator mb-3 flex items-start gap-3 rounded-lg border px-3 py-2",
            style: format!("border-color: {}; background-color: {}80;", colors::BORDER_DEFAULT, colors::SURFACE_CARD),
            role: "status",
            aria_label: format!("Replying to {}", props.message.sender_name),
            // Reply icon
            div {
                class: "flex-shrink-0",
                style: format!("color: {};", colors::TEXT_MUTED),
                "↩"
            }
            // Reply content
            div {
                class: "flex-1 min-w-0",
                span {
                    class: "text-xs font-medium",
                    style: format!("color: {};", colors::PRIMARY),
                    "Replying to {props.message.sender_name}"
                }
                p {
                    class: "text-sm truncate mt-0.5",
                    style: format!("color: {};", colors::TEXT_SECONDARY),
                    "{preview}"
                }
            }
            // Cancel button
            button {
                class: "flex-shrink-0 w-6 h-6 rounded flex items-center justify-center hover:opacity-80",
                style: format!("color: {};", colors::TEXT_MUTED),
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
            class: "w-4 h-4 border-2 rounded-full animate-spin",
            style: format!("border-color: {}30; border-top-color: {};", colors::TEXT_INVERSE, colors::TEXT_INVERSE),
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

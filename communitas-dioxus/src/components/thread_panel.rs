//! Thread panel component for threaded conversations.
//!
//! Opens as a side panel when clicking a thread indicator on a message.
//! Shows the parent message and all replies. Supports "also send to channel"
//! for broadcasting thread replies back to the main channel.

use crate::components::channel_sidebar::SelectedChannel;
use crate::design_tokens::{layout, motion, palette, radius, semantic, spacing, typography};
use crate::models::channel::ChatMessage;
use communitas_x0x_client::{X0xClient, X0xWebSocket};
use dioxus::prelude::*;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, warn};

/// Thread panel showing a parent message and its replies.
#[component]
pub fn ThreadPanel(
    /// The parent message that started the thread.
    parent_message: ChatMessage,
    /// The channel this thread belongs to.
    channel: SelectedChannel,
    /// Called when the thread panel should close.
    on_close: EventHandler<()>,
) -> Element {
    let mut replies = use_signal(Vec::<ChatMessage>::new);
    let mut composer_text = use_signal(String::new);
    let mut also_send_to_channel = use_signal(|| false);
    let mut sending = use_signal(|| false);
    let mut ws_connected = use_signal(|| false);
    let mut composer_focused = use_signal(|| false);

    let parent_msg_id = parent_message.id.clone();
    let group_id = channel.group_id.clone();
    let channel_topic = channel.topic.clone();

    // Build thread topic
    let group_id_prefix = if group_id.len() >= 16 {
        group_id[..16].to_string()
    } else {
        group_id.clone()
    };
    let thread_topic = format!("x0x.group.{group_id_prefix}.thread/{parent_msg_id}");

    // WebSocket coroutine for thread messages
    let thread_topic_ws = thread_topic.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let topic = thread_topic_ws.clone();
        async move {
            let ws = match X0xWebSocket::connect().await {
                Ok(ws) => {
                    if let Err(e) = ws.subscribe(vec![topic.clone()]) {
                        error!(target: "ui.thread_panel", "Failed to subscribe to thread: {e}");
                        return;
                    }
                    ws_connected.set(true);
                    ws
                }
                Err(e) => {
                    warn!(target: "ui.thread_panel", "WebSocket connection failed: {e}");
                    return;
                }
            };

            let mut ws = ws;

            while let Some(inbound) = ws.recv().await {
                if let communitas_x0x_client::WsInbound::Message {
                    topic: msg_topic,
                    payload,
                    ..
                } = inbound
                    && msg_topic == topic
                    && let Ok(bytes) =
                        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &payload)
                    && let Ok(msg) = serde_json::from_slice::<ChatMessage>(&bytes)
                {
                    replies.with_mut(|r| {
                        if !r.iter().any(|m| m.id == msg.id) {
                            r.push(msg);
                            r.sort_by_key(|m| m.timestamp);
                        }
                    });
                }
            }

            ws_connected.set(false);
        }
    });

    // Get own agent ID
    let mut own_agent_id = use_signal(|| Option::<String>::None);
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(agent) = client.agent().await {
            own_agent_id.set(Some(agent.agent_id));
        }
    });

    let send_reply = {
        let thread_topic = thread_topic.clone();
        let channel_topic = channel_topic.clone();
        let channel_name = channel.channel_name.clone();
        let parent_msg_id = parent_msg_id.clone();
        move || {
            let text = composer_text();
            if text.trim().is_empty() {
                return;
            }

            let thread_topic = thread_topic.clone();
            let channel_topic = channel_topic.clone();
            let channel_name = channel_name.clone();
            let parent_msg_id = parent_msg_id.clone();
            let agent_id = own_agent_id().unwrap_or_default();
            let broadcast = also_send_to_channel();

            sending.set(true);
            composer_text.set(String::new());

            spawn(async move {
                let msg = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    text: text.clone(),
                    sender_name: "Me".to_string(),
                    sender_id: agent_id,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                    channel: channel_name,
                    thread_root: Some(parent_msg_id),
                    broadcast,
                    reply_count: 0,
                    reactions: HashMap::new(),
                };

                let json_bytes = match serde_json::to_vec(&msg) {
                    Ok(b) => b,
                    Err(e) => {
                        error!(target: "ui.thread_panel", "Failed to serialize reply: {e}");
                        sending.set(false);
                        return;
                    }
                };

                let client = X0xClient::new();

                // Publish to thread topic
                if let Err(e) = client.publish(&thread_topic, &json_bytes).await {
                    error!(target: "ui.thread_panel", "Failed to publish to thread: {e}");
                } else {
                    // Add reply locally
                    replies.with_mut(|r| {
                        if !r.iter().any(|m| m.id == msg.id) {
                            r.push(msg.clone());
                        }
                    });
                }

                // If "also send to channel", publish to channel topic as well
                if broadcast && let Err(e) = client.publish(&channel_topic, &json_bytes).await {
                    warn!(target: "ui.thread_panel", "Failed to broadcast to channel: {e}");
                }

                sending.set(false);
            });
        }
    };

    rsx! {
        div {
            style: format!(
                "width: {}; \
                 height: 100%; \
                 display: flex; \
                 flex-direction: column; \
                 background: {}; \
                 border-left: 1px solid {}; \
                 flex-shrink: 0;",
                layout::THREAD_PANEL_WIDTH,
                semantic::BG_PRIMARY,
                semantic::BORDER_SUBTLE
            ),
            role: "complementary",
            aria_label: "Thread",

            // Panel header
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: space-between; \
                     padding: {} {}; \
                     border-bottom: 1px solid {}; \
                     flex-shrink: 0;",
                    spacing::MD,
                    spacing::BASE,
                    semantic::BORDER_SUBTLE
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_MD,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY
                    ),
                    "Thread"
                }

                button {
                    style: format!(
                        "width: 28px; \
                         height: 28px; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         background: none; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         cursor: pointer; \
                         color: {}; \
                         font-size: {}; \
                         transition: {};",
                        semantic::BORDER_SUBTLE,
                        radius::MD,
                        semantic::TEXT_MUTED,
                        typography::SIZE_SM,
                        motion::transition("background, border-color")
                    ),
                    aria_label: "Close thread",
                    onclick: move |_| on_close.call(()),
                    "\u{2715}" // X mark
                }
            }

            // Scrollable message area
            div {
                style: format!(
                    "flex: 1; \
                     overflow-y: auto; \
                     padding: {}; \
                     display: flex; \
                     flex-direction: column; \
                     gap: {}; \
                     scrollbar-width: thin; \
                     scrollbar-color: {} transparent;",
                    spacing::BASE,
                    spacing::SM,
                    semantic::BORDER_DEFAULT
                ),

                // Parent message (highlighted)
                ThreadParentMessage { message: parent_message.clone() }

                // Reply count divider
                if !replies().is_empty() {
                    div {
                        style: format!(
                            "display: flex; \
                             align-items: center; \
                             gap: {}; \
                             padding: {} 0;",
                            spacing::SM,
                            spacing::XS
                        ),

                        div {
                            style: format!(
                                "flex: 1; \
                                 height: 1px; \
                                 background: {};",
                                semantic::BORDER_SUBTLE
                            ),
                        }

                        span {
                            style: format!(
                                "font-size: {}; \
                                 color: {}; \
                                 white-space: nowrap;",
                                typography::SIZE_XS,
                                semantic::TEXT_MUTED
                            ),
                            {
                                let count = replies().len();
                                let label = if count == 1 { "reply" } else { "replies" };
                                format!("{count} {label}")
                            }
                        }

                        div {
                            style: format!(
                                "flex: 1; \
                                 height: 1px; \
                                 background: {};",
                                semantic::BORDER_SUBTLE
                            ),
                        }
                    }
                }

                // Replies
                for reply in replies() {
                    ThreadReplyMessage {
                        key: "{reply.id}",
                        message: reply.clone(),
                        is_own: own_agent_id().as_deref() == Some(&reply.sender_id),
                    }
                }
            }

            // Reply composer
            div {
                style: format!(
                    "padding: {}; \
                     border-top: 1px solid {}; \
                     background: {}; \
                     flex-shrink: 0;",
                    spacing::SM,
                    semantic::BORDER_SUBTLE,
                    semantic::BG_SECONDARY
                ),

                // "Also send to channel" checkbox
                label {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         margin-bottom: {}; \
                         cursor: pointer; \
                         font-size: {}; \
                         color: {};",
                        spacing::XS,
                        spacing::SM,
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED
                    ),

                    input {
                        r#type: "checkbox",
                        checked: also_send_to_channel(),
                        onchange: move |evt| {
                            also_send_to_channel.set(evt.checked());
                        },
                        style: format!(
                            "accent-color: {};",
                            semantic::PRIMARY
                        ),
                    }

                    { format!("Also send to #{}", channel.channel_name) }
                }

                // Composer input
                div {
                    style: format!(
                        "display: flex; \
                         align-items: flex-end; \
                         gap: {}; \
                         padding: {}; \
                         background: {}; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         transition: {};",
                        spacing::XS,
                        spacing::XS,
                        semantic::BG_TERTIARY,
                        if composer_focused() { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                        radius::LG,
                        motion::transition("border-color")
                    ),

                    textarea {
                        placeholder: "Reply...",
                        value: "{composer_text}",
                        disabled: sending() || !ws_connected(),
                        rows: "1",
                        aria_label: "Thread reply input. Press Enter to send.",
                        style: format!(
                            "flex: 1; \
                             background: transparent; \
                             border: none; \
                             outline: none; \
                             resize: none; \
                             color: {}; \
                             font-family: {}; \
                             font-size: {}; \
                             line-height: {}; \
                             min-height: 20px; \
                             max-height: 80px; \
                             overflow-y: auto;",
                            semantic::TEXT_PRIMARY,
                            typography::FONT_BODY,
                            typography::SIZE_SM,
                            typography::LEADING_NORMAL
                        ),
                        onfocus: move |_| composer_focused.set(true),
                        onblur: move |_| composer_focused.set(false),
                        oninput: move |evt: Event<FormData>| {
                            composer_text.set(evt.value().to_string());
                        },
                        onkeydown: {
                            let mut send = send_reply.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter && !evt.modifiers().shift() {
                                    evt.prevent_default();
                                    send();
                                }
                            }
                        },
                    }

                    button {
                        style: format!(
                            "width: 28px; \
                             height: 28px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: {}; \
                             border: none; \
                             border-radius: {}; \
                             cursor: {}; \
                             opacity: {}; \
                             transition: {};",
                            if sending() || composer_text().trim().is_empty() {
                                semantic::BG_ELEVATED
                            } else {
                                semantic::PRIMARY
                            },
                            radius::FULL,
                            if sending() { "not-allowed" } else { "pointer" },
                            if sending() || composer_text().trim().is_empty() { "0.5" } else { "1" },
                            motion::transition("background, opacity")
                        ),
                        disabled: sending() || composer_text().trim().is_empty(),
                        aria_label: "Send reply",
                        onclick: {
                            let mut send = send_reply.clone();
                            move |_| send()
                        },
                        span {
                            style: format!(
                                "font-size: {}; \
                                 color: {};",
                                typography::SIZE_XS,
                                semantic::TEXT_INVERSE
                            ),
                            "\u{2191}"
                        }
                    }
                }
            }
        }
    }
}

/// The parent (root) message in a thread, displayed with emphasis.
#[component]
fn ThreadParentMessage(message: ChatMessage) -> Element {
    let initials = message
        .sender_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    let ts_display = {
        let secs = message.timestamp / 1000;
        let mins = (secs / 60) % 60;
        let hours = (secs / 3600) % 24;
        format!("{hours:02}:{mins:02}")
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border-radius: {}; \
                 border: 1px solid {};",
                spacing::SM,
                spacing::SM,
                semantic::BG_SECONDARY,
                radius::LG,
                semantic::BORDER_SUBTLE
            ),

            // Avatar
            div {
                style: format!(
                    "width: 36px; \
                     height: 36px; \
                     border-radius: {}; \
                     background: {}; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     flex-shrink: 0;",
                    radius::FULL,
                    semantic::BG_ELEVATED,
                    typography::SIZE_XS,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY
                ),
                "{initials}"
            }

            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: format!(
                        "display: flex; \
                         align-items: baseline; \
                         gap: {}; \
                         margin-bottom: {};",
                        spacing::SM,
                        spacing::XXS
                    ),

                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_SM,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "{message.sender_name}"
                    }

                    span {
                        style: format!(
                            "font-size: {}; \
                             color: {};",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED
                        ),
                        "{ts_display}"
                    }
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         line-height: {}; \
                         margin: 0;",
                        typography::SIZE_BASE,
                        semantic::TEXT_PRIMARY,
                        typography::LEADING_NORMAL
                    ),
                    "{message.text}"
                }
            }
        }
    }
}

/// A reply message in the thread.
#[component]
fn ThreadReplyMessage(message: ChatMessage, is_own: bool) -> Element {
    let initials = message
        .sender_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    let ts_display = {
        let secs = message.timestamp / 1000;
        let mins = (secs / 60) % 60;
        let hours = (secs / 3600) % 24;
        format!("{hours:02}:{mins:02}")
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 gap: {}; \
                 padding: {} {};",
                spacing::SM,
                spacing::XS,
                spacing::XS
            ),

            // Avatar (smaller for replies)
            div {
                style: format!(
                    "width: 28px; \
                     height: 28px; \
                     border-radius: {}; \
                     background: {}; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     flex-shrink: 0;",
                    radius::FULL,
                    if is_own { palette::JADE_700 } else { semantic::BG_ELEVATED },
                    typography::SIZE_XXS,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY
                ),
                "{initials}"
            }

            div {
                style: "flex: 1; min-width: 0;",

                div {
                    style: format!(
                        "display: flex; \
                         align-items: baseline; \
                         gap: {}; \
                         margin-bottom: {};",
                        spacing::XS,
                        spacing::XXS
                    ),

                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            if is_own { palette::JADE_400 } else { semantic::TEXT_PRIMARY }
                        ),
                        "{message.sender_name}"
                    }

                    span {
                        style: format!(
                            "font-size: {}; \
                             color: {};",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED
                        ),
                        "{ts_display}"
                    }
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         line-height: {}; \
                         margin: 0;",
                        typography::SIZE_SM,
                        semantic::TEXT_PRIMARY,
                        typography::LEADING_NORMAL
                    ),
                    "{message.text}"
                }
            }
        }
    }
}

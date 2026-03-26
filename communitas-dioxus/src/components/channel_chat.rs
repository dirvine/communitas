//! Channel chat view component for real-time messaging.
//!
//! Displays messages for a selected channel, supports sending messages,
//! and shows thread indicators for messages with replies.

use crate::components::channel_sidebar::SelectedChannel;
use crate::design_tokens::{motion, palette, radius, semantic, spacing, typography};
use crate::models::channel::ChatMessage;
use base64::Engine as _;
use communitas_x0x_client::{X0xClient, X0xWebSocket};
use dioxus::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

/// Channel chat view with message list and composer.
#[component]
pub fn ChannelChatView(
    /// The currently selected channel.
    channel: SelectedChannel,
    /// Called when a thread is opened (clicking thread indicator on a message).
    on_open_thread: EventHandler<ChatMessage>,
) -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut composer_text = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut ws_connected = use_signal(|| false);

    let topic = channel.topic.clone();
    let channel_name = channel.channel_name.clone();

    // WebSocket coroutine for real-time messaging
    let topic_for_ws = topic.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let topic = topic_for_ws.clone();
        async move {
            let ws = match X0xWebSocket::connect().await {
                Ok(ws) => {
                    // Subscribe to the channel topic
                    if let Err(e) = ws.subscribe(vec![topic.clone()]) {
                        error!(target: "ui.channel_chat", "Failed to subscribe to {topic}: {e}");
                        return;
                    }
                    info!(target: "ui.channel_chat", "Subscribed to channel topic: {topic}");
                    ws_connected.set(true);
                    ws
                }
                Err(e) => {
                    warn!(target: "ui.channel_chat", "WebSocket connection failed: {e}");
                    return;
                }
            };

            let mut ws = ws;

            // Listen for incoming messages
            while let Some(inbound) = ws.recv().await {
                match inbound {
                    communitas_x0x_client::WsInbound::Message {
                        topic: msg_topic,
                        payload,
                        ..
                    } => {
                        if msg_topic == topic {
                            // Decode base64 payload to ChatMessage
                            match base64::engine::general_purpose::STANDARD.decode(&payload) {
                                Ok(bytes) => {
                                    match serde_json::from_slice::<ChatMessage>(&bytes) {
                                        Ok(msg) => {
                                            messages.with_mut(|msgs| {
                                                // Avoid duplicates by ID
                                                if !msgs.iter().any(|m| m.id == msg.id) {
                                                    msgs.push(msg);
                                                    msgs.sort_by_key(|m| m.timestamp);
                                                }
                                            });
                                        }
                                        Err(e) => {
                                            warn!(target: "ui.channel_chat", "Failed to parse message: {e}");
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!(target: "ui.channel_chat", "Failed to decode payload: {e}");
                                }
                            }
                        }
                    }
                    communitas_x0x_client::WsInbound::Error { message } => {
                        error!(target: "ui.channel_chat", "WebSocket error: {message}");
                    }
                    _ => {}
                }
            }

            ws_connected.set(false);
        }
    });

    // Get current agent ID for identifying own messages
    let mut own_agent_id = use_signal(|| Option::<String>::None);
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(agent) = client.agent().await {
            own_agent_id.set(Some(agent.agent_id));
        }
    });

    let send_message = {
        let topic = topic.clone();
        let channel_name = channel_name.clone();
        move || {
            let text = composer_text();
            if text.trim().is_empty() {
                return;
            }

            let topic = topic.clone();
            let channel_name = channel_name.clone();
            let agent_id = own_agent_id().unwrap_or_default();

            sending.set(true);
            composer_text.set(String::new());

            spawn(async move {
                let msg = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    sender_name: "Me".to_string(),
                    sender_id: agent_id,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                    channel: channel_name,
                    thread_root: None,
                    broadcast: false,
                    reply_count: 0,
                    reactions: std::collections::HashMap::new(),
                };

                match serde_json::to_vec(&msg) {
                    Ok(json_bytes) => {
                        let client = X0xClient::new();
                        if let Err(e) = client.publish(&topic, &json_bytes).await {
                            error!(target: "ui.channel_chat", "Failed to publish message: {e}");
                        } else {
                            info!(target: "ui.channel_chat", "Message published to {topic}");
                            // Add locally for immediate display
                            messages.with_mut(|msgs| {
                                if !msgs.iter().any(|m| m.id == msg.id) {
                                    msgs.push(msg);
                                }
                            });
                        }
                    }
                    Err(e) => {
                        error!(target: "ui.channel_chat", "Failed to serialize message: {e}");
                    }
                }

                sending.set(false);
            });
        }
    };

    let channel_display_name = channel.channel_name.clone();

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 height: 100%; \
                 background: {};",
                semantic::BG_PRIMARY
            ),

            // Channel header
            ChannelHeader {
                channel_name: channel_display_name.clone(),
                description: channel.meta.as_ref().map(|m| m.description.clone()).unwrap_or_default(),
                connected: ws_connected(),
            }

            // Message list
            div {
                style: format!(
                    "flex: 1; \
                     overflow-y: auto; \
                     padding: {} {}; \
                     display: flex; \
                     flex-direction: column; \
                     gap: {}; \
                     scrollbar-width: thin; \
                     scrollbar-color: {} transparent;",
                    spacing::BASE,
                    spacing::XL,
                    spacing::SM,
                    semantic::BORDER_DEFAULT
                ),
                role: "log",
                aria_label: "Channel messages",
                aria_live: "polite",

                if messages().is_empty() {
                    ChannelEmptyState { channel_name: channel_display_name.clone() }
                } else {
                    for msg in messages() {
                        ChannelMessage {
                            key: "{msg.id}",
                            message: msg.clone(),
                            is_own: own_agent_id().as_deref() == Some(&msg.sender_id),
                            on_open_thread: move |m: ChatMessage| on_open_thread.call(m),
                        }
                    }
                }
            }

            // Composer
            ChannelComposer {
                value: composer_text(),
                disabled: sending() || !ws_connected(),
                channel_name: channel.channel_name.clone(),
                oninput: move |evt: Event<FormData>| {
                    composer_text.set(evt.value().to_string());
                },
                onsubmit: {
                    let mut send = send_message.clone();
                    move |_| send()
                },
            }
        }
    }
}

/// Channel header bar with name, description, and connection status.
#[component]
fn ChannelHeader(channel_name: String, description: String, connected: bool) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 border-bottom: 1px solid {}; \
                 background: {}; \
                 flex-shrink: 0;",
                spacing::SM,
                spacing::MD,
                spacing::XL,
                semantic::BORDER_SUBTLE,
                semantic::BG_SECONDARY
            ),

            // Channel name with hash
            div {
                style: "display: flex; align-items: center; gap: 4px; flex: 1;",

                span {
                    style: format!(
                        "color: {}; \
                         font-size: {}; \
                         font-weight: {};",
                        semantic::PRIMARY,
                        typography::SIZE_LG,
                        typography::WEIGHT_NORMAL
                    ),
                    "#"
                }

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {};",
                        typography::SIZE_MD,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY
                    ),
                    "{channel_name}"
                }

                if !description.is_empty() {
                    span {
                        style: format!(
                            "color: {}; \
                             font-size: {}; \
                             margin-left: {}; \
                             padding-left: {}; \
                             border-left: 1px solid {};",
                            semantic::TEXT_MUTED,
                            typography::SIZE_SM,
                            spacing::SM,
                            spacing::SM,
                            semantic::BORDER_SUBTLE
                        ),
                        "{description}"
                    }
                }
            }

            // Connection indicator
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {};",
                    spacing::XS
                ),

                div {
                    style: format!(
                        "width: 8px; \
                         height: 8px; \
                         border-radius: {}; \
                         background: {};",
                        radius::FULL,
                        if connected { semantic::SUCCESS } else { semantic::WARNING }
                    ),
                }

                span {
                    style: format!(
                        "font-size: {}; \
                         color: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED
                    ),
                    if connected { "Connected" } else { "Connecting..." }
                }
            }
        }
    }
}

/// A single message in the channel view.
#[component]
fn ChannelMessage(
    message: ChatMessage,
    is_own: bool,
    on_open_thread: EventHandler<ChatMessage>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let initials = message
        .sender_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    // Format timestamp
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
                 padding: {} {}; \
                 border-radius: {}; \
                 transition: {}; \
                 {}",
                spacing::SM,
                spacing::XS,
                spacing::SM,
                radius::MD,
                motion::transition("background"),
                if hovered() { format!("background: {};", semantic::BG_TERTIARY) } else { String::new() }
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

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
                    if is_own { palette::JADE_700 } else { semantic::BG_ELEVATED },
                    typography::SIZE_XS,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY
                ),
                "{initials}"
            }

            // Content
            div {
                style: "flex: 1; min-width: 0;",

                // Sender name + timestamp
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

                    // Broadcast indicator
                    if message.broadcast {
                        span {
                            style: format!(
                                "font-size: {}; \
                                 color: {}; \
                                 background: {}; \
                                 padding: 1px {}; \
                                 border-radius: {};",
                                typography::SIZE_XXS,
                                semantic::TEXT_MUTED,
                                semantic::BG_ELEVATED,
                                spacing::XS,
                                radius::SM
                            ),
                            "from thread"
                        }
                    }
                }

                // Message text
                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         line-height: {}; \
                         margin: 0; \
                         word-wrap: break-word;",
                        typography::SIZE_BASE,
                        semantic::TEXT_PRIMARY,
                        typography::LEADING_NORMAL
                    ),
                    "{message.text}"
                }

                // Reactions
                if !message.reactions.is_empty() {
                    div {
                        style: format!(
                            "display: flex; \
                             flex-wrap: wrap; \
                             gap: {}; \
                             margin-top: {};",
                            spacing::XS,
                            spacing::XS
                        ),

                        for (emoji, count) in &message.reactions {
                            span {
                                key: "{emoji}",
                                style: format!(
                                    "display: inline-flex; \
                                     align-items: center; \
                                     gap: 2px; \
                                     padding: 1px {}; \
                                     background: {}; \
                                     border: 1px solid {}; \
                                     border-radius: {}; \
                                     font-size: {}; \
                                     cursor: pointer;",
                                    spacing::XS,
                                    semantic::BG_TERTIARY,
                                    semantic::BORDER_SUBTLE,
                                    radius::SM,
                                    typography::SIZE_XS
                                ),
                                "{emoji} {count}"
                            }
                        }
                    }
                }

                // Thread indicator
                if message.reply_count > 0 {
                    {
                        let msg_clone = message.clone();
                        let reply_label = if message.reply_count == 1 {
                            format!("{} reply", message.reply_count)
                        } else {
                            format!("{} replies", message.reply_count)
                        };
                        rsx! {
                            button {
                                style: format!(
                                    "display: inline-flex; \
                                     align-items: center; \
                                     gap: {}; \
                                     margin-top: {}; \
                                     padding: {} {}; \
                                     background: none; \
                                     border: 1px solid {}; \
                                     border-radius: {}; \
                                     cursor: pointer; \
                                     color: {}; \
                                     font-size: {}; \
                                     font-family: {}; \
                                     transition: {};",
                                    spacing::XS,
                                    spacing::XS,
                                    spacing::XXS,
                                    spacing::SM,
                                    semantic::BORDER_SUBTLE,
                                    radius::MD,
                                    semantic::PRIMARY,
                                    typography::SIZE_XS,
                                    typography::FONT_BODY,
                                    motion::transition("background, border-color")
                                ),
                                onclick: move |_| on_open_thread.call(msg_clone.clone()),

                                span {
                                    style: format!("color: {};", semantic::PRIMARY),
                                    "\u{1F4AC}"
                                }
                                span {
                                    "{reply_label}"
                                }
                            }
                        }
                    }
                }
            }

            // Hover actions
            if hovered() {
                div {
                    style: format!(
                        "display: flex; \
                         gap: {}; \
                         align-self: flex-start; \
                         flex-shrink: 0;",
                        spacing::XXS
                    ),

                    MessageAction {
                        icon: "\u{1F4AC}",
                        tooltip: "Reply in thread",
                        onclick: {
                            let msg = message.clone();
                            move |_| on_open_thread.call(msg.clone())
                        },
                    }
                }
            }
        }
    }
}

/// Small action button for message hover actions.
#[component]
fn MessageAction(icon: &'static str, tooltip: String, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            style: format!(
                "width: 28px; \
                 height: 28px; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: pointer; \
                 font-size: {}; \
                 transition: {};",
                semantic::BG_SECONDARY,
                semantic::BORDER_SUBTLE,
                radius::MD,
                typography::SIZE_XS,
                motion::transition("background, border-color")
            ),
            title: "{tooltip}",
            onclick: move |_| onclick.call(()),
            "{icon}"
        }
    }
}

/// Empty state shown when a channel has no messages.
#[component]
fn ChannelEmptyState(channel_name: String) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 align-items: center; \
                 justify-content: center; \
                 height: 100%; \
                 padding: {}; \
                 text-align: center;",
                spacing::HUGE
            ),

            div {
                style: format!(
                    "width: 80px; \
                     height: 80px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     background: {}; \
                     border-radius: {}; \
                     font-size: {}; \
                     margin-bottom: {};",
                    semantic::BG_TERTIARY,
                    radius::XXL,
                    typography::SIZE_4XL,
                    spacing::XL
                ),
                "#"
            }

            div {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     margin-bottom: {};",
                    typography::SIZE_LG,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY,
                    spacing::XS
                ),
                "Welcome to #{channel_name}"
            }

            div {
                style: format!(
                    "font-size: {}; \
                     color: {};",
                    typography::SIZE_SM,
                    semantic::TEXT_MUTED
                ),
                "This is the start of the channel. Send a message to get the conversation going."
            }
        }
    }
}

/// Message composer for the channel chat view.
#[component]
fn ChannelComposer(
    value: String,
    disabled: bool,
    channel_name: String,
    oninput: EventHandler<Event<FormData>>,
    onsubmit: EventHandler<()>,
) -> Element {
    let mut focused = use_signal(|| false);

    rsx! {
        div {
            style: format!(
                "padding: {}; \
                 border-top: 1px solid {}; \
                 background: {}; \
                 flex-shrink: 0;",
                spacing::BASE,
                semantic::BORDER_SUBTLE,
                semantic::BG_SECONDARY
            ),

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
                    spacing::SM,
                    spacing::SM,
                    semantic::BG_TERTIARY,
                    if focused() { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                    radius::XL,
                    motion::transition("border-color")
                ),

                textarea {
                    placeholder: "Message #{channel_name}",
                    value: "{value}",
                    disabled: disabled,
                    rows: "1",
                    aria_label: "Message input. Press Enter to send, Shift+Enter for new line.",
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
                         min-height: 24px; \
                         max-height: 120px; \
                         overflow-y: auto;",
                        semantic::TEXT_PRIMARY,
                        typography::FONT_BODY,
                        typography::SIZE_BASE,
                        typography::LEADING_NORMAL
                    ),
                    onfocus: move |_| focused.set(true),
                    onblur: move |_| focused.set(false),
                    oninput: move |evt| oninput.call(evt),
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter && !evt.modifiers().shift() {
                            evt.prevent_default();
                            onsubmit.call(());
                        }
                    },
                }

                // Send button
                button {
                    style: format!(
                        "width: 32px; \
                         height: 32px; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         background: {}; \
                         border: none; \
                         border-radius: {}; \
                         cursor: {}; \
                         opacity: {}; \
                         transition: {};",
                        if disabled || value.trim().is_empty() {
                            semantic::BG_ELEVATED.to_string()
                        } else {
                            semantic::PRIMARY.to_string()
                        },
                        radius::FULL,
                        if disabled { "not-allowed" } else { "pointer" },
                        if disabled || value.trim().is_empty() { "0.5" } else { "1" },
                        motion::transition("background, opacity")
                    ),
                    disabled: disabled || value.trim().is_empty(),
                    aria_label: "Send message",
                    onclick: move |_| onsubmit.call(()),
                    span {
                        style: format!(
                            "font-size: {}; \
                             color: {};",
                            typography::SIZE_SM,
                            semantic::TEXT_INVERSE
                        ),
                        "\u{2191}" // up arrow
                    }
                }
            }
        }
    }
}

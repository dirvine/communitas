// SPDX-License-Identifier: MIT OR Apache-2.0

//! Direct messaging view component for point-to-point communication.
//!
//! Displays a conversation with a single peer agent, supports sending and
//! receiving direct messages over the x0x `/ws/direct` WebSocket endpoint,
//! and persists history to the same local chat history file as channels.

use crate::design_tokens::{motion, palette, radius, semantic, spacing, typography};
use crate::models::channel::ChatMessage;
use crate::x0x_contract;
use base64::Engine as _;
use communitas_x0x_client::{X0xClient, X0xWebSocket};
use dioxus::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DirectMessagePayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sender_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sender_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ts: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timestamp: Option<u64>,
}

fn direct_message_id(sender_id: &str, timestamp: u64, text: &str) -> String {
    let seed = format!("{sender_id}:{timestamp}:{text}");
    format!("{:x}", Sha256::digest(seed.as_bytes()))
}

fn decode_direct_message_payload(payload: &[u8], sender_id: &str) -> Result<ChatMessage, String> {
    if let Ok(message) = serde_json::from_slice::<DirectMessagePayload>(payload) {
        let timestamp = message.ts.or(message.timestamp).unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        });
        let resolved_sender_id = message.sender_id.unwrap_or_else(|| sender_id.to_string());
        let resolved_sender_name = message
            .sender_name
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| x0x_contract::fallback_sender_name(&resolved_sender_id));
        let resolved_id = message
            .id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| direct_message_id(&resolved_sender_id, timestamp, &message.text));

        return Ok(ChatMessage {
            id: resolved_id,
            text: message.text,
            sender_name: resolved_sender_name,
            sender_id: resolved_sender_id,
            timestamp,
            channel: String::new(),
            thread_root: None,
            broadcast: false,
            is_deleted: false,
            reply_count: 0,
            reactions: HashMap::new(),
        });
    }

    match std::str::from_utf8(payload) {
        Ok(text) => {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            Ok(ChatMessage {
                id: direct_message_id(sender_id, timestamp, text),
                text: text.to_string(),
                sender_name: x0x_contract::fallback_sender_name(sender_id),
                sender_id: sender_id.to_string(),
                timestamp,
                channel: String::new(),
                thread_root: None,
                broadcast: false,
                is_deleted: false,
                reply_count: 0,
                reactions: HashMap::new(),
            })
        }
        Err(err) => Err(format!("unrecognized direct-message payload: {err}")),
    }
}

/// Truncate an agent ID for display.
fn short_agent_id(id: &str) -> String {
    if id.len() <= 12 {
        id.to_owned()
    } else {
        format!("{}..{}", &id[..6], &id[id.len() - 4..])
    }
}

/// Direct message view with message list and composer.
#[component]
pub fn DmView(
    /// The agent ID of the peer we are messaging.
    agent_id: String,
) -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut composer_text = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut ws_connected = use_signal(|| false);
    let mut peer_connected = use_signal(|| false);

    let peer_id = agent_id.clone();

    // Load local DM history on mount
    let history_peer_id = peer_id.clone();
    use_future(move || {
        let peer_id = history_peer_id.clone();
        async move {
            let history = x0x_contract::load_dm_history(&peer_id).await;
            messages.set(history);
        }
    });

    // Establish QUIC connection to the peer and check connection status
    let connect_peer_id = peer_id.clone();
    use_future(move || {
        let peer_id = connect_peer_id.clone();
        async move {
            let client = X0xClient::new();

            // Attempt to establish connection
            if let Err(e) = client.connect_agent(&peer_id).await {
                warn!(target: "ui.dm_view", "Failed to connect to agent {peer_id}: {e}");
            }

            // Poll connection status
            loop {
                match client.direct_connections().await {
                    Ok(connections) => {
                        let is_connected = connections.iter().any(|c| c.agent_id == peer_id);
                        peer_connected.set(is_connected);
                    }
                    Err(e) => {
                        warn!(target: "ui.dm_view", "Failed to check connections: {e}");
                        peer_connected.set(false);
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    // Get current agent info
    let mut own_agent_id = use_signal(|| Option::<String>::None);
    let mut own_sender_name = use_signal(|| Option::<String>::None);
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(agent) = client.agent().await {
            let fallback_name = agent
                .user_id
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent.agent_id));
            let display_name = client
                .agent_card(None, Some(false))
                .await
                .ok()
                .map(|card| card.card.display_name)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(fallback_name);

            own_agent_id.set(Some(agent.agent_id));
            own_sender_name.set(Some(display_name));
        }
    });

    // WebSocket coroutine for receiving direct messages
    let ws_peer_id = peer_id.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let peer_id = ws_peer_id.clone();
        async move {
            let mut ws = match X0xWebSocket::connect_direct().await {
                Ok(ws) => {
                    info!(target: "ui.dm_view", "Connected to /ws/direct for DMs");
                    ws_connected.set(true);
                    ws
                }
                Err(e) => {
                    warn!(target: "ui.dm_view", "WebSocket /ws/direct connection failed: {e}");
                    return;
                }
            };

            while let Some(inbound) = ws.recv().await {
                match inbound {
                    communitas_x0x_client::WsInbound::DirectMessage {
                        sender, payload, ..
                    } => {
                        // Only show messages from our conversation peer
                        if sender != peer_id {
                            continue;
                        }

                        match base64::engine::general_purpose::STANDARD.decode(&payload) {
                            Ok(bytes) => match decode_direct_message_payload(&bytes, &sender) {
                                Ok(msg) => {
                                    let history_msg = msg.clone();
                                    let history_peer = peer_id.clone();
                                    messages.with_mut(|msgs| {
                                        if !msgs.iter().any(|m| m.id == history_msg.id) {
                                            msgs.push(msg);
                                            msgs.sort_by_key(|m| m.timestamp);
                                        }
                                    });
                                    x0x_contract::append_dm_history(&history_peer, &history_msg)
                                        .await;
                                }
                                Err(e) => {
                                    warn!(target: "ui.dm_view", "Failed to parse DM payload: {e}");
                                }
                            },
                            Err(e) => {
                                warn!(target: "ui.dm_view", "Failed to decode DM base64: {e}");
                            }
                        }
                    }
                    communitas_x0x_client::WsInbound::Error { message } => {
                        error!(target: "ui.dm_view", "WebSocket error: {message}");
                    }
                    _ => {}
                }
            }

            ws_connected.set(false);
        }
    });

    // Send message handler
    let send_peer_id = peer_id.clone();
    let send_message = {
        move || {
            let text = composer_text();
            if text.trim().is_empty() {
                return;
            }

            let peer_id = send_peer_id.clone();
            let agent_id = own_agent_id().unwrap_or_default();
            let sender_name =
                own_sender_name().unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent_id));

            sending.set(true);
            composer_text.set(String::new());

            spawn(async move {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let msg = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    sender_name,
                    sender_id: agent_id,
                    timestamp,
                    channel: String::new(), // DMs don't belong to a channel
                    thread_root: None,
                    broadcast: false,
                    is_deleted: false,
                    reply_count: 0,
                    reactions: HashMap::new(),
                };
                let wire = DirectMessagePayload {
                    id: Some(msg.id.clone()),
                    text: msg.text.clone(),
                    sender_name: Some(msg.sender_name.clone()),
                    sender_id: Some(msg.sender_id.clone()),
                    ts: Some(msg.timestamp),
                    timestamp: None,
                };

                match serde_json::to_vec(&wire) {
                    Ok(json_bytes) => {
                        let client = X0xClient::new();
                        if let Err(e) = client.send_direct(&peer_id, &json_bytes).await {
                            error!(target: "ui.dm_view", "Failed to send DM to {peer_id}: {e}");
                        } else {
                            info!(target: "ui.dm_view", "DM sent to {peer_id}");
                            // Add locally for immediate display (optimistic update)
                            messages.with_mut(|msgs| {
                                if !msgs.iter().any(|m| m.id == msg.id) {
                                    msgs.push(msg.clone());
                                    msgs.sort_by_key(|entry| entry.timestamp);
                                }
                            });
                            x0x_contract::append_dm_history(&peer_id, &msg).await;
                        }
                    }
                    Err(e) => {
                        error!(target: "ui.dm_view", "Failed to serialize DM: {e}");
                    }
                }

                sending.set(false);
            });
        }
    };

    let peer_display = short_agent_id(&agent_id);

    // Look up contact label
    let label_peer_id = agent_id.clone();
    let peer_label = use_resource(move || {
        let peer_id = label_peer_id.clone();
        async move {
            let client = X0xClient::new();
            if let Ok(contacts) = client.list_contacts().await {
                for c in contacts {
                    if c.agent_id == peer_id {
                        return c.label.unwrap_or_default();
                    }
                }
            }
            String::new()
        }
    });

    let header_label = peer_label.read().as_ref().cloned().unwrap_or_default();
    let header_name = if header_label.is_empty() {
        peer_display.clone()
    } else {
        header_label
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 height: 100%; \
                 background: {};",
                semantic::BG_PRIMARY
            ),

            // Header
            DmHeader {
                peer_name: header_name,
                peer_id: peer_display,
                connected: peer_connected(),
                ws_connected: ws_connected(),
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
                aria_label: "Direct messages",
                aria_live: "polite",

                if messages().is_empty() {
                    DmEmptyState { peer_name: agent_id.clone() }
                } else {
                    for msg in messages() {
                        DmMessage {
                            key: "{msg.id}",
                            message: msg.clone(),
                            is_own: own_agent_id().as_deref() == Some(&msg.sender_id),
                        }
                    }
                }
            }

            // Composer
            DmComposer {
                value: composer_text(),
                disabled: sending() || !ws_connected(),
                peer_name: agent_id.clone(),
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

/// Header bar for the DM view.
#[component]
fn DmHeader(peer_name: String, peer_id: String, connected: bool, ws_connected: bool) -> Element {
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

            // Peer info
            div {
                style: "display: flex; align-items: center; gap: 8px; flex: 1;",

                // Avatar circle
                div {
                    style: format!(
                        "width: 32px; \
                         height: 32px; \
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
                    {
                        let initials: String = peer_name
                            .split_whitespace()
                            .take(2)
                            .filter_map(|w| w.chars().next())
                            .collect::<String>()
                            .to_uppercase();
                        if initials.is_empty() {
                            "DM".to_string()
                        } else {
                            initials
                        }
                    }
                }

                div {
                    style: "min-width: 0;",

                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_MD,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "{peer_name}"
                    }

                    div {
                        style: format!(
                            "font-size: {}; \
                             font-family: monospace; \
                             color: {};",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED
                        ),
                        "{peer_id}"
                    }
                }
            }

            // Connection indicators
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {};",
                    spacing::SM
                ),

                // QUIC connection status
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
}

/// A single DM bubble.
#[component]
fn DmMessage(message: ChatMessage, is_own: bool) -> Element {
    let mut hovered = use_signal(|| false);

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
                 padding: {} {}; \
                 border-radius: {}; \
                 transition: {}; \
                 {}{}",
                spacing::SM,
                spacing::XS,
                spacing::SM,
                radius::MD,
                motion::transition("background"),
                if is_own { "flex-direction: row-reverse; " } else { "" },
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
                style: format!(
                    "flex: 1; min-width: 0; {}",
                    if is_own { "text-align: right;" } else { "" }
                ),

                // Sender name + timestamp
                div {
                    style: format!(
                        "display: flex; \
                         align-items: baseline; \
                         gap: {}; \
                         margin-bottom: {}; \
                         {}",
                        spacing::SM,
                        spacing::XXS,
                        if is_own { "justify-content: flex-end;" } else { "" }
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
                }

                // Message text
                div {
                    style: format!(
                        "display: inline-block; \
                         padding: {} {}; \
                         border-radius: {}; \
                         background: {}; \
                         max-width: 75%;",
                        spacing::SM,
                        spacing::MD,
                        radius::XL,
                        if is_own {
                            format!("{}33", palette::JADE_700) // semi-transparent jade
                        } else {
                            semantic::BG_TERTIARY.to_string()
                        }
                    ),

                    p {
                        style: format!(
                            "font-size: {}; \
                             color: {}; \
                             line-height: {}; \
                             margin: 0; \
                             word-wrap: break-word; \
                             text-align: left;",
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
}

/// Empty state for when no DM history exists.
#[component]
fn DmEmptyState(peer_name: String) -> Element {
    let short = short_agent_id(&peer_name);
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
                "\u{1F4AC}" // speech balloon
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
                "Direct Message with {short}"
            }

            div {
                style: format!(
                    "font-size: {}; \
                     color: {};",
                    typography::SIZE_SM,
                    semantic::TEXT_MUTED
                ),
                "This is the start of your conversation. Send a message to begin."
            }
        }
    }
}

/// Message composer for the DM view.
#[component]
fn DmComposer(
    value: String,
    disabled: bool,
    peer_name: String,
    oninput: EventHandler<Event<FormData>>,
    onsubmit: EventHandler<()>,
) -> Element {
    let mut focused = use_signal(|| false);
    let short = short_agent_id(&peer_name);

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
                    placeholder: "Message {short}",
                    value: "{value}",
                    disabled: disabled,
                    rows: "1",
                    aria_label: "Direct message input. Press Enter to send, Shift+Enter for new line.",
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

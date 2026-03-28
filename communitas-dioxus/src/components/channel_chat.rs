//! Channel chat view component for real-time messaging.
//!
//! Displays messages for a selected channel, supports sending messages,
//! and shows thread indicators for messages with replies.
//!
//! # Features
//! - Auto-scroll to newest message
//! - Markdown rendering via `MarkdownContent`
//! - Typing indicators with auto-expiry
//! - Inline message search with highlight
//! - Edit / delete for own messages (delete requires confirmation)
//! - Inline reply (quote) UI
//! - @mention autocomplete
//! - Correct local-time timestamps via `chrono`

use crate::components::channel_sidebar::SelectedChannel;
use crate::components::confirm_dialog::ConfirmDialog;
use crate::components::markdown::MarkdownContent;
use crate::components::mention::{MentionAutocomplete, MentionCandidate, filter_candidates};
use crate::design_tokens::{motion, palette, radius, semantic, spacing, typography};
use crate::models::channel::ChatMessage;
use crate::x0x_contract;
use base64::Engine as _;
use chrono::{DateTime, Local};
use communitas_x0x_client::{X0xClient, X0xWebSocket};
use dioxus::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Extended wire payload (edit / delete / typing events)
// ---------------------------------------------------------------------------

/// Flat gossip envelope that is backward-compatible with old clients.
///
/// Old clients (which only know the plain `ChatMessage` schema) deserialize
/// the base `ChatMessage` fields and silently ignore the new optional control
/// fields (`msg_type`, `edit`, `delete`, `typing`).  New clients check
/// `msg_type` first; when absent or `"chat"` the message is treated as a
/// plain chat message.
///
/// **Wire format rules:**
/// - Plain chat message: populate all `ChatMessage` fields; omit `msg_type`
///   (or set to `"chat"`); omit `edit`, `delete`, `typing`.
/// - Edit event: set `msg_type = "edit"`, populate `edit`; other ChatMessage
///   fields carry dummy/empty values.
/// - Delete event: set `msg_type = "delete"`, populate `delete`.
/// - Typing event: set `msg_type = "typing"`, populate `typing`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GossipEnvelope {
    // ---- ChatMessage fields (required for backward-compat with old clients) ----
    #[serde(default)]
    id: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    sender_name: String,
    #[serde(default)]
    sender_id: String,
    #[serde(default)]
    timestamp: u64,
    #[serde(default)]
    channel: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    thread_root: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    broadcast: bool,
    // ---- Control fields (new; absent = plain chat message) ----
    /// Message type discriminant. Absent or `"chat"` = plain chat message.
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_type: Option<String>,
    /// Present when `msg_type == "edit"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    edit: Option<EditPayload>,
    /// Present when `msg_type == "delete"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<DeletePayload>,
    /// Present when `msg_type == "typing"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    typing: Option<TypingPayload>,
}

impl GossipEnvelope {
    /// Construct a plain chat-message envelope.
    fn chat(msg: &ChatMessage) -> Self {
        Self {
            id: msg.id.clone(),
            text: msg.text.clone(),
            sender_name: msg.sender_name.clone(),
            sender_id: msg.sender_id.clone(),
            timestamp: msg.timestamp,
            channel: msg.channel.clone(),
            thread_root: msg.thread_root.clone(),
            broadcast: msg.broadcast,
            msg_type: None, // omitted on wire; old clients ignore
            edit: None,
            delete: None,
            typing: None,
        }
    }

    /// Construct an edit-event envelope.
    fn edit_event(ep: EditPayload, channel: &str) -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            sender_name: String::new(),
            sender_id: ep.sender_id.clone(),
            timestamp: 0,
            channel: channel.to_string(),
            thread_root: None,
            broadcast: false,
            msg_type: Some("edit".to_string()),
            edit: Some(ep),
            delete: None,
            typing: None,
        }
    }

    /// Construct a delete-event envelope.
    fn delete_event(dp: DeletePayload, channel: &str) -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            sender_name: String::new(),
            sender_id: dp.sender_id.clone(),
            timestamp: 0,
            channel: channel.to_string(),
            thread_root: None,
            broadcast: false,
            msg_type: Some("delete".to_string()),
            edit: None,
            delete: Some(dp),
            typing: None,
        }
    }

    /// Construct a typing-event envelope.
    fn typing_event(tp: TypingPayload) -> Self {
        Self {
            id: String::new(),
            text: String::new(),
            sender_name: tp.sender_name.clone(),
            sender_id: tp.sender_id.clone(),
            timestamp: 0,
            channel: String::new(),
            thread_root: None,
            broadcast: false,
            msg_type: Some("typing".to_string()),
            edit: None,
            delete: None,
            typing: Some(tp),
        }
    }

    /// Extract the embedded chat message (only valid when `msg_type` is chat).
    fn into_chat_message(self) -> ChatMessage {
        ChatMessage {
            id: self.id,
            text: self.text,
            sender_name: self.sender_name,
            sender_id: self.sender_id,
            timestamp: self.timestamp,
            channel: self.channel,
            thread_root: self.thread_root,
            broadcast: self.broadcast,
            is_deleted: false,
            reply_count: 0,
            reactions: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct EditPayload {
    message_id: String,
    new_text: String,
    sender_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DeletePayload {
    message_id: String,
    sender_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct TypingPayload {
    sender_id: String,
    sender_name: String,
}

// ---------------------------------------------------------------------------
// Typing state
// ---------------------------------------------------------------------------

/// A typing indicator entry: (display_name, last_seen_secs).
#[derive(Clone, PartialEq)]
struct TypingEntry {
    display_name: String,
    last_seen_secs: u64,
}

/// Return monotonic seconds (seconds since UNIX_EPOCH).
fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// ChannelChatView
// ---------------------------------------------------------------------------

/// Channel chat view with message list and composer.
#[component]
pub fn ChannelChatView(
    /// The currently selected channel.
    channel: SelectedChannel,
    /// Called when a thread is opened (clicking thread indicator on a message).
    on_open_thread: EventHandler<ChatMessage>,
    /// Signal carrying a parent message ID and generation counter when a
    /// thread reply is sent. Each bump increments the reply count on the
    /// matching channel message.
    #[props(default)]
    reply_count_bump: Option<Signal<Option<(String, u64)>>>,
) -> Element {
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut composer_text = use_signal(String::new);
    let mut sending = use_signal(|| false);
    let mut ws_connected = use_signal(|| false);
    let mut discovered_agent_ids = use_signal(HashSet::<String>::new);

    // new state
    // Per-sender typing indicators; keyed by sender_id.
    let mut typing_users = use_signal(HashMap::<String, TypingEntry>::new);
    // Last time we sent a typing event (throttle to 1 per 2 s).
    let mut last_typing_sent = use_signal(|| 0u64);
    // Whether the search bar is visible.
    let mut search_active = use_signal(|| false);
    // Current search query.
    let mut search_query = use_signal(String::new);
    // Message being replied to.
    let mut reply_to = use_signal(|| Option::<ChatMessage>::None);
    // Message being edited (id + draft text).
    let mut editing_msg = use_signal(|| Option::<(String, String)>::None);
    // Message id pending deletion confirmation.
    let mut delete_confirm_id = use_signal(|| Option::<String>::None);
    // Contact candidates for @mention.
    let mut mention_candidates = use_signal(Vec::<MentionCandidate>::new);
    // Active @mention query (None = dropdown hidden).
    let mut mention_query = use_signal(|| Option::<String>::None);

    let topic = channel.topic.clone();
    let group_id = channel.group_id.clone();
    let channel_name = channel.channel_name.clone();

    // ----- history load -----
    // B2: Channel changes are handled by the parent (space_view.rs) which keys
    // ChannelChatView on "{group_id}:{channel_name}", causing a full remount when
    // the user switches channels. No need for a manual refetch trigger here.
    let history_group_id = group_id.clone();
    let history_channel_name = channel_name.clone();
    use_future(move || {
        let history_group_id = history_group_id.clone();
        let history_channel_name = history_channel_name.clone();
        async move {
            let history =
                x0x_contract::load_channel_history(&history_group_id, &history_channel_name).await;
            messages.set(history);
        }
    });

    // ----- discovered agents -----
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(agents) = client.discovered_agents().await {
            let ids: HashSet<String> = agents.into_iter().map(|a| a.agent_id).collect();
            discovered_agent_ids.set(ids);
        }
    });

    // ----- contacts for @mention -----
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(contacts) = client.list_contacts().await {
            let candidates = contacts
                .into_iter()
                .map(|c| MentionCandidate {
                    id: c.agent_id.clone(),
                    display_name: c
                        .label
                        .as_deref()
                        .filter(|l| !l.trim().is_empty())
                        .map(|l| l.to_string())
                        .unwrap_or_else(|| x0x_contract::fallback_sender_name(&c.agent_id)),
                })
                .collect::<Vec<_>>();
            mention_candidates.set(candidates);
        }
    });

    // ----- reply-count bump from thread panel -----
    let mut last_bump_gen = use_signal(|| 0u64);
    use_effect(move || {
        if let Some(bump_signal) = reply_count_bump
            && let Some((parent_id, generation)) = bump_signal()
            && generation > last_bump_gen()
        {
            last_bump_gen.set(generation);
            messages.with_mut(|msgs| {
                if let Some(msg) = msgs.iter_mut().find(|m| m.id == parent_id) {
                    msg.reply_count += 1;
                }
            });
        }
    });

    // ----- auto-scroll when message count changes -----
    // Read the signal INSIDE the closure so Dioxus tracks it as a dependency
    // and re-runs this effect every time new messages arrive.
    use_effect(move || {
        let _count = messages.read().len(); // creates the signal subscription
        let _ = document::eval(
            r#"
            var el = document.getElementById('chat-messages');
            if (el) { el.scrollTop = el.scrollHeight; }
            "#,
        );
    });

    // ----- typing indicator expiry (tick every 2 s via a cheap effect) -----
    // We purge stale entries whenever the typing map is read in the render path.
    // No separate timer needed; staleness check happens at render time.

    // ----- WebSocket coroutine -----
    let topic_for_ws = topic.clone();
    let ws_group_id = group_id.clone();
    let ws_channel_name = channel_name.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let topic = topic_for_ws.clone();
        let group_id = ws_group_id.clone();
        let channel_name = ws_channel_name.clone();
        async move {
            let ws = match X0xWebSocket::connect().await {
                Ok(ws) => {
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

            while let Some(inbound) = ws.recv().await {
                match inbound {
                    communitas_x0x_client::WsInbound::Message {
                        topic: msg_topic,
                        payload,
                        ..
                    } => {
                        if msg_topic != topic {
                            continue;
                        }
                        let decoded = match base64::engine::general_purpose::STANDARD
                            .decode(&payload)
                        {
                            Ok(b) => b,
                            Err(e) => {
                                warn!(target: "ui.channel_chat", "Failed to decode payload: {e}");
                                continue;
                            }
                        };

                        // Parse the flat GossipEnvelope. Because the envelope
                        // embeds all ChatMessage fields at the top level, old
                        // clients that only know the plain ChatMessage schema
                        // will deserialize the base fields and ignore the new
                        // control fields (msg_type, edit, delete, typing).
                        match serde_json::from_slice::<GossipEnvelope>(&decoded) {
                            Ok(env) => {
                                let msg_type = env.msg_type.as_deref().unwrap_or("chat");
                                match msg_type {
                                    "chat" | "" => {
                                        let msg = env.into_chat_message();
                                        if !msg.id.is_empty() {
                                            let history_msg = msg.clone();
                                            messages.with_mut(|msgs| {
                                                if !msgs.iter().any(|m| m.id == history_msg.id) {
                                                    msgs.push(msg);
                                                    msgs.sort_by_key(|m| m.timestamp);
                                                }
                                            });
                                            x0x_contract::append_channel_history(
                                                &group_id,
                                                &channel_name,
                                                &history_msg,
                                            )
                                            .await;
                                        }
                                    }
                                    "edit" => {
                                        // SECURITY NOTE: edit/delete authorization is UI-only.
                                        // The gossip layer does not carry ML-DSA signatures on
                                        // control messages, so `sender_id` matching here only
                                        // prevents accidental cross-user edits — it does NOT
                                        // cryptographically verify the sender. Full authorization
                                        // requires ML-DSA integration at the gossip layer.
                                        if let Some(ep) = env.edit {
                                            messages.with_mut(|msgs| {
                                                if let Some(m) =
                                                    msgs.iter_mut().find(|m| m.id == ep.message_id)
                                                    && m.sender_id == ep.sender_id
                                                {
                                                    m.text = ep.new_text;
                                                }
                                            });
                                        }
                                    }
                                    "delete" => {
                                        // SECURITY NOTE: Same limitation as "edit" above —
                                        // sender_id matching is UI-only, not cryptographic.
                                        if let Some(dp) = env.delete {
                                            messages.with_mut(|msgs| {
                                                if let Some(m) =
                                                    msgs.iter_mut().find(|m| m.id == dp.message_id)
                                                    && m.sender_id == dp.sender_id
                                                {
                                                    // Use the is_deleted flag — no string matching.
                                                    m.is_deleted = true;
                                                }
                                            });
                                        }
                                    }
                                    "typing" => {
                                        if let Some(tp) = env.typing {
                                            // M5: Per-sender inbound rate limiting — skip if the
                                            // same sender was updated less than 500 ms ago.
                                            let now = now_secs();
                                            let should_update = typing_users
                                                .read()
                                                .get(&tp.sender_id)
                                                .map(|entry| {
                                                    now.saturating_sub(entry.last_seen_secs) >= 1
                                                })
                                                .unwrap_or(true);
                                            if should_update {
                                                typing_users.with_mut(|map| {
                                                    // M2: Purge entries older than 10 s on every insert.
                                                    map.retain(|_, entry| {
                                                        now.saturating_sub(entry.last_seen_secs)
                                                            <= 10
                                                    });
                                                    map.insert(
                                                        tp.sender_id.clone(),
                                                        TypingEntry {
                                                            display_name: tp.sender_name.clone(),
                                                            last_seen_secs: now,
                                                        },
                                                    );
                                                });
                                            }
                                        }
                                    }
                                    _ => {
                                        warn!(target: "ui.channel_chat", "Unknown msg_type: {msg_type}");
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(target: "ui.channel_chat", "Failed to parse inbound message: {e}");
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

    // ----- own agent identity -----
    let mut own_agent_id = use_signal(|| Option::<String>::None);
    let mut own_sender_name = use_signal(|| Option::<String>::None);
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(agent) = client.agent().await {
            let fallback_name = agent
                .user_id
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent.agent_id));
            let display_name = client
                .agent_card(None, Some(false))
                .await
                .ok()
                .map(|card| card.card.display_name)
                .filter(|v| !v.trim().is_empty())
                .unwrap_or(fallback_name);

            own_agent_id.set(Some(agent.agent_id));
            own_sender_name.set(Some(display_name));
        }
    });

    // ----- send message closure -----
    let send_message = {
        let topic = topic.clone();
        let send_group_id = group_id.clone();
        let send_channel_name = channel_name.clone();
        move || {
            let text = composer_text();
            if text.trim().is_empty() {
                return;
            }

            let topic = topic.clone();
            let group_id = send_group_id.clone();
            let channel_name = send_channel_name.clone();
            let agent_id = own_agent_id().unwrap_or_default();
            let sender_name =
                own_sender_name().unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent_id));
            let reply_to_snapshot = reply_to();

            sending.set(true);
            // B4: Do NOT clear composer here — only clear on confirmed send.
            // mention_query is UI-only and safe to clear immediately.
            mention_query.set(None);

            spawn(async move {
                let msg_channel_name = channel_name.clone();
                let reply_to_id = reply_to_snapshot.as_ref().map(|r| r.id.clone());
                // B5: Keep thread_root for the local copy so the sender sees
                // the same inline-quote that receivers see.
                let msg = ChatMessage {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    sender_name,
                    sender_id: agent_id,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0),
                    channel: msg_channel_name,
                    thread_root: reply_to_id,
                    broadcast: false,
                    is_deleted: false,
                    reply_count: 0,
                    reactions: HashMap::new(),
                };

                // Use the flat envelope factory; thread_root is preserved on wire.
                let envelope = GossipEnvelope::chat(&msg);

                match serde_json::to_vec(&envelope) {
                    Ok(json_bytes) => {
                        let client = X0xClient::new();
                        if let Err(e) = client.publish(&topic, &json_bytes).await {
                            error!(target: "ui.channel_chat", "Failed to publish message: {e}");
                            // B4: Send failed — restore composer text so the user doesn't lose it.
                            composer_text.set(msg.text.clone());
                            reply_to.set(reply_to_snapshot);
                        } else {
                            info!(target: "ui.channel_chat", "Message published to {topic}");
                            // B4: Only clear after confirmed send.
                            composer_text.set(String::new());
                            reply_to.set(None);
                            // B3: Apply locally only after network confirmation.
                            messages.with_mut(|msgs| {
                                if !msgs.iter().any(|m| m.id == msg.id) {
                                    msgs.push(msg.clone());
                                    msgs.sort_by_key(|entry| entry.timestamp);
                                }
                            });
                            x0x_contract::append_channel_history(&group_id, &channel_name, &msg)
                                .await;
                        }
                    }
                    Err(e) => {
                        error!(target: "ui.channel_chat", "Failed to serialize message: {e}");
                        composer_text.set(msg.text.clone());
                        reply_to.set(reply_to_snapshot);
                    }
                }

                sending.set(false);
            });
        }
    };

    // ----- publish edit -----
    let publish_edit = {
        let topic = topic.clone();
        let channel_name_for_edit = channel_name.clone();
        move |msg_id: String, new_text: String| {
            let topic = topic.clone();
            let channel_name = channel_name_for_edit.clone();
            let agent_id = own_agent_id().unwrap_or_default();
            let ep = EditPayload {
                message_id: msg_id.clone(),
                new_text: new_text.clone(),
                sender_id: agent_id.clone(),
            };
            let envelope = GossipEnvelope::edit_event(ep, &channel_name);
            spawn(async move {
                match serde_json::to_vec(&envelope) {
                    Ok(bytes) => {
                        let client = X0xClient::new();
                        if let Err(e) = client.publish(&topic, &bytes).await {
                            error!(target: "ui.channel_chat", "Failed to publish edit: {e}");
                        } else {
                            // B3: Apply locally only after network confirmation.
                            messages.with_mut(|msgs| {
                                if let Some(m) = msgs.iter_mut().find(|m| m.id == msg_id)
                                    && m.sender_id == agent_id
                                {
                                    m.text = new_text;
                                }
                            });
                        }
                    }
                    Err(e) => {
                        error!(target: "ui.channel_chat", "Failed to serialize edit: {e}");
                    }
                }
            });
        }
    };

    // ----- publish delete -----
    let publish_delete = {
        let topic = topic.clone();
        let channel_name_for_delete = channel_name.clone();
        move |msg_id: String| {
            let topic = topic.clone();
            let channel_name = channel_name_for_delete.clone();
            let agent_id = own_agent_id().unwrap_or_default();
            let dp = DeletePayload {
                message_id: msg_id.clone(),
                sender_id: agent_id.clone(),
            };
            let envelope = GossipEnvelope::delete_event(dp, &channel_name);
            spawn(async move {
                match serde_json::to_vec(&envelope) {
                    Ok(bytes) => {
                        let client = X0xClient::new();
                        if let Err(e) = client.publish(&topic, &bytes).await {
                            error!(target: "ui.channel_chat", "Failed to publish delete: {e}");
                        } else {
                            // B3: Apply locally only after network confirmation.
                            // M3: Use is_deleted flag instead of string-prefix matching.
                            messages.with_mut(|msgs| {
                                if let Some(m) = msgs.iter_mut().find(|m| m.id == msg_id)
                                    && m.sender_id == agent_id
                                {
                                    m.is_deleted = true;
                                }
                            });
                        }
                    }
                    Err(e) => {
                        error!(target: "ui.channel_chat", "Failed to serialize delete: {e}");
                    }
                }
            });
        }
    };

    // ----- publish typing -----
    let publish_typing = {
        let topic = topic.clone();
        move || {
            let now = now_secs();
            if now.saturating_sub(last_typing_sent()) < 2 {
                return;
            }
            last_typing_sent.set(now);
            let topic = topic.clone();
            let agent_id = own_agent_id().unwrap_or_default();
            let sender_name =
                own_sender_name().unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent_id));
            let tp = TypingPayload {
                sender_id: agent_id,
                sender_name,
            };
            let envelope = GossipEnvelope::typing_event(tp);
            spawn(async move {
                match serde_json::to_vec(&envelope) {
                    Ok(bytes) => {
                        let client = X0xClient::new();
                        if let Err(e) = client.publish(&topic, &bytes).await {
                            error!(target: "ui.channel_chat", "Failed to publish typing: {e}");
                        }
                    }
                    Err(e) => {
                        error!(target: "ui.channel_chat", "Failed to serialize typing: {e}");
                    }
                }
            });
        }
    };

    // ----- derive filtered messages (search) -----
    // M1: Wrap in use_memo so the Vec is only recomputed when messages or the
    // query actually changes, instead of being re-cloned on every render.
    let filtered_messages = use_memo(move || {
        let q = search_query().to_lowercase();
        if q.is_empty() {
            messages()
        } else {
            messages()
                .into_iter()
                .filter(|m| m.text.to_lowercase().contains(q.as_str()))
                .collect()
        }
    });

    // ----- derive active typers (exclude self, purge stale > 3 s) -----
    let own_id = own_agent_id().unwrap_or_default();
    let active_typers: Vec<String> = {
        let now = now_secs();
        typing_users
            .read()
            .iter()
            .filter(|(id, entry)| {
                id.as_str() != own_id.as_str() && now.saturating_sub(entry.last_seen_secs) <= 3
            })
            .map(|(_, entry)| entry.display_name.clone())
            .collect()
    };

    let channel_display_name = channel.channel_name.clone();

    // M6: Snapshot own_agent_id once per render to avoid calling the signal
    // getter inside the hot message-list loop.
    let my_agent_id = own_agent_id();
    // M4: Snapshot messages for reply resolution in the render loop.
    let all_messages_snap: Vec<ChatMessage> = messages.read().clone();

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 height: 100%; \
                 background: {};",
                semantic::BG_PRIMARY
            ),

            // Delete confirmation dialog
            if let Some(pending_id) = delete_confirm_id() {
                {
                    let id_for_confirm = pending_id.clone();
                    let publish_delete = publish_delete.clone();
                    rsx! {
                        ConfirmDialog {
                            title: "Delete Message",
                            message: "Are you sure you want to delete this message? This cannot be undone.",
                            confirm_text: "Delete",
                            cancel_text: "Cancel",
                            destructive: true,
                            on_confirm: move |_| {
                                publish_delete(id_for_confirm.clone());
                                delete_confirm_id.set(None);
                            },
                            on_cancel: move |_| {
                                delete_confirm_id.set(None);
                            },
                        }
                    }
                }
            }

            // Channel header
            ChannelHeader {
                channel_name: channel_display_name.clone(),
                description: channel.meta.as_ref().map(|m| m.description.clone()).unwrap_or_default(),
                connected: ws_connected(),
                search_active: search_active(),
                search_query: search_query(),
                on_toggle_search: move |_| {
                    let was = search_active();
                    search_active.set(!was);
                    if was {
                        search_query.set(String::new());
                    }
                },
                on_search_input: move |q: String| search_query.set(q),
            }

            // Reply-to preview bar
            if let Some(ref parent_msg) = reply_to() {
                {
                    let preview_name = parent_msg.sender_name.clone();
                    let preview_text = parent_msg.text.chars().take(80).collect::<String>();
                    rsx! {
                        div {
                            style: format!(
                                "display: flex; \
                                 align-items: center; \
                                 gap: {}; \
                                 padding: {} {}; \
                                 background: {}; \
                                 border-left: 3px solid {}; \
                                 border-bottom: 1px solid {}; \
                                 flex-shrink: 0;",
                                spacing::SM,
                                spacing::XS,
                                spacing::XL,
                                semantic::BG_SECONDARY,
                                palette::JADE_500,
                                semantic::BORDER_SUBTLE
                            ),

                            div {
                                style: "flex: 1; min-width: 0;",

                                span {
                                    style: format!(
                                        "font-size: {}; font-weight: {}; color: {};",
                                        typography::SIZE_XS,
                                        typography::WEIGHT_SEMIBOLD,
                                        palette::JADE_400
                                    ),
                                    "Replying to {preview_name}"
                                }

                                p {
                                    style: format!(
                                        "margin: 0; font-size: {}; color: {}; \
                                         overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                        typography::SIZE_XS,
                                        semantic::TEXT_MUTED
                                    ),
                                    "{preview_text}"
                                }
                            }

                            button {
                                style: format!(
                                    "background: none; border: none; cursor: pointer; \
                                     color: {}; font-size: {}; padding: {};",
                                    semantic::TEXT_MUTED,
                                    typography::SIZE_SM,
                                    spacing::XS
                                ),
                                aria_label: "Cancel reply",
                                onclick: move |_| reply_to.set(None),
                                "\u{00D7}"
                            }
                        }
                    }
                }
            }

            // Message list
            div {
                id: "chat-messages",
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

                if filtered_messages().is_empty() {
                    ChannelEmptyState { channel_name: channel_display_name.clone() }
                } else {
                    for msg in filtered_messages() {
                        {
                            let is_own = my_agent_id.as_deref() == Some(&msg.sender_id);
                            let is_agent = discovered_agent_ids.read().contains(&msg.sender_id);
                            let msg_for_reply = msg.clone();
                            let msg_for_thread = msg.clone();
                            let msg_id_for_edit = msg.id.clone();
                            let msg_id_for_delete = msg.id.clone();
                            let msg_text_for_edit = msg.text.clone();
                            let publish_edit = publish_edit.clone();
                            let search_q = search_query();
                            // M4: Resolve reply sender and text from the snapshot.
                            let (reply_sender_name, reply_text_snippet) =
                                if let Some(ref rid) = msg.thread_root {
                                    if let Some(parent) = all_messages_snap.iter().find(|m| &m.id == rid) {
                                        (
                                            Some(parent.sender_name.clone()),
                                            Some(parent.text.chars().take(80).collect::<String>()),
                                        )
                                    } else {
                                        (None, None)
                                    }
                                } else {
                                    (None, None)
                                };
                            rsx! {
                                ChannelMessage {
                                    key: "{msg.id}",
                                    message: msg.clone(),
                                    is_own,
                                    is_agent,
                                    search_highlight: search_q,
                                    editing: editing_msg().as_ref().map(|(id, _)| id.clone()) == Some(msg.id.clone()),
                                    edit_draft: editing_msg().as_ref().and_then(|(id, t)| {
                                        if id == &msg.id { Some(t.clone()) } else { None }
                                    }).unwrap_or_default(),
                                    reply_sender_name,
                                    reply_text_snippet,
                                    on_open_thread: move |m: ChatMessage| on_open_thread.call(m),
                                    on_reply: move |_| reply_to.set(Some(msg_for_reply.clone())),
                                    on_start_edit: move |_| {
                                        editing_msg.set(Some((msg_id_for_edit.clone(), msg_text_for_edit.clone())));
                                    },
                                    on_edit_draft_change: move |new_text: String| {
                                        editing_msg.with_mut(|e| {
                                            if let Some((_, t)) = e {
                                                *t = new_text;
                                            }
                                        });
                                    },
                                    on_save_edit: move |_| {
                                        if let Some((id, text)) = editing_msg() {
                                            publish_edit(id, text);
                                            editing_msg.set(None);
                                        }
                                    },
                                    on_cancel_edit: move |_| editing_msg.set(None),
                                    on_delete: move |_| delete_confirm_id.set(Some(msg_id_for_delete.clone())),
                                    on_open_thread_btn: move |_| on_open_thread.call(msg_for_thread.clone()),
                                }
                            }
                        }
                    }
                }
            }

            // Typing indicator
            if !active_typers.is_empty() {
                TypingBar { typers: active_typers }
            }

            // @mention autocomplete (above composer)
            if let Some(ref mq) = mention_query() {
                {
                    let candidates_snap = mention_candidates();
                    let query_str = mq.clone();
                    let filtered_len = filter_candidates(&candidates_snap, &query_str).len();
                    if filtered_len > 0 {
                        rsx! {
                            div {
                                style: "position: relative; flex-shrink: 0;",
                                MentionAutocomplete {
                                    candidates: candidates_snap,
                                    query: query_str,
                                    on_select: move |c: MentionCandidate| {
                                        // Replace the @query in composer_text with the selected name
                                        let text = composer_text();
                                        if let Some(at_pos) = text.rfind('@') {
                                            let before = text[..at_pos].to_string();
                                            let new_text = format!("{before}@{} ", c.display_name);
                                            composer_text.set(new_text);
                                        }
                                        mention_query.set(None);
                                    },
                                    on_dismiss: move |_| mention_query.set(None),
                                }
                            }
                        }
                    } else {
                        rsx! {}
                    }
                }
            }

            // Composer
            ChannelComposer {
                value: composer_text(),
                disabled: sending() || !ws_connected(),
                channel_name: channel.channel_name.clone(),
                oninput: move |evt: Event<FormData>| {
                    let text = evt.value().to_string();
                    composer_text.set(text.clone());

                    // Detect @mention trigger
                    if let Some(at_pos) = text.rfind('@') {
                        let after_at = &text[at_pos + 1..];
                        // Only trigger if '@' is at start or preceded by space
                        let before_at = &text[..at_pos];
                        let preceded_by_space =
                            before_at.is_empty() || before_at.ends_with(' ');
                        if preceded_by_space && !after_at.contains(' ') {
                            mention_query.set(Some(after_at.to_string()));
                        } else {
                            mention_query.set(None);
                        }
                    } else {
                        mention_query.set(None);
                    }

                    // Send typing event (throttled)
                    let mut publish_typing = publish_typing.clone();
                    publish_typing();
                },
                onsubmit: {
                    let mut send = send_message.clone();
                    move |_| send()
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Channel header (with search toggle)
// ---------------------------------------------------------------------------

#[component]
fn ChannelHeader(
    channel_name: String,
    description: String,
    connected: bool,
    search_active: bool,
    search_query: String,
    on_toggle_search: EventHandler<()>,
    on_search_input: EventHandler<String>,
) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 border-bottom: 1px solid {}; \
                 background: {}; \
                 flex-shrink: 0;",
                semantic::BORDER_SUBTLE,
                semantic::BG_SECONDARY
            ),

            // Top row
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {}; \
                     padding: {} {};",
                    spacing::SM,
                    spacing::MD,
                    spacing::XL,
                ),

                // Channel name with hash
                div {
                    style: "display: flex; align-items: center; gap: 4px; flex: 1;",

                    span {
                        style: format!(
                            "color: {}; font-size: {}; font-weight: {};",
                            semantic::PRIMARY,
                            typography::SIZE_LG,
                            typography::WEIGHT_NORMAL
                        ),
                        "#"
                    }

                    span {
                        style: format!(
                            "font-size: {}; font-weight: {}; color: {};",
                            typography::SIZE_MD,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "{channel_name}"
                    }

                    if !description.is_empty() {
                        span {
                            style: format!(
                                "color: {}; font-size: {}; \
                                 margin-left: {}; padding-left: {}; \
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

                // Search toggle button
                button {
                    style: format!(
                        "width: 28px; height: 28px; \
                         display: flex; align-items: center; justify-content: center; \
                         background: {}; border: 1px solid {}; \
                         border-radius: {}; cursor: pointer; font-size: {};",
                        if search_active { semantic::BG_TERTIARY } else { "transparent" },
                        if search_active { semantic::BORDER_DEFAULT } else { "transparent" },
                        radius::MD,
                        typography::SIZE_SM
                    ),
                    title: "Search messages",
                    aria_label: "Toggle message search",
                    onclick: move |_| on_toggle_search.call(()),
                    "\u{1F50D}"
                }

                // Connection indicator
                div {
                    style: format!(
                        "display: flex; align-items: center; gap: {};",
                        spacing::XS
                    ),

                    div {
                        style: format!(
                            "width: 8px; height: 8px; border-radius: {}; background: {};",
                            radius::FULL,
                            if connected { semantic::SUCCESS } else { semantic::WARNING }
                        ),
                    }

                    span {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_XS,
                            semantic::TEXT_MUTED
                        ),
                        if connected { "Connected" } else { "Connecting..." }
                    }
                }
            }

            // Search bar (shown when search_active)
            if search_active {
                div {
                    style: format!(
                        "padding: {} {};",
                        spacing::XS,
                        spacing::XL
                    ),

                    input {
                        r#type: "text",
                        placeholder: "Search messages...",
                        value: "{search_query}",
                        autofocus: true,
                        style: format!(
                            "width: 100%; \
                             padding: {} {}; \
                             background: {}; \
                             border: 1px solid {}; \
                             border-radius: {}; \
                             color: {}; \
                             font-size: {}; \
                             font-family: {}; \
                             outline: none; \
                             box-sizing: border-box;",
                            spacing::XS,
                            spacing::SM,
                            semantic::BG_TERTIARY,
                            semantic::BORDER_DEFAULT,
                            radius::MD,
                            semantic::TEXT_PRIMARY,
                            typography::SIZE_SM,
                            typography::FONT_BODY
                        ),
                        oninput: move |evt| on_search_input.call(evt.value().to_string()),
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// TypingBar
// ---------------------------------------------------------------------------

/// Displays who is currently typing.
#[component]
fn TypingBar(typers: Vec<String>) -> Element {
    let label = match typers.len() {
        0 => return rsx! {},
        1 => format!("{} is typing…", typers[0]),
        2 => format!("{} and {} are typing…", typers[0], typers[1]),
        _ => "Several people are typing…".to_string(),
    };

    rsx! {
        div {
            style: format!(
                "padding: {} {}; \
                 font-size: {}; \
                 color: {}; \
                 font-style: italic; \
                 flex-shrink: 0;",
                spacing::XS,
                spacing::XL,
                typography::SIZE_XS,
                semantic::TEXT_MUTED
            ),
            "{label}"
        }
    }
}

// ---------------------------------------------------------------------------
// ChannelMessage
// ---------------------------------------------------------------------------

/// A single message in the channel view.
#[component]
fn ChannelMessage(
    message: ChatMessage,
    is_own: bool,
    #[props(default)] is_agent: bool,
    /// Current search query for highlighting (empty = no highlight).
    #[props(default)]
    search_highlight: String,
    /// Whether this message is being edited right now.
    #[props(default = false)]
    editing: bool,
    /// Current draft text while editing.
    #[props(default)]
    edit_draft: String,
    /// M4: Resolved reply sender name (looked up by parent from messages list).
    /// Falls back to the raw message ID when None.
    #[props(default)]
    reply_sender_name: Option<String>,
    /// M4: Resolved reply text snippet (looked up by parent from messages list).
    #[props(default)]
    reply_text_snippet: Option<String>,
    on_open_thread: EventHandler<ChatMessage>,
    on_reply: EventHandler<()>,
    on_start_edit: EventHandler<()>,
    on_edit_draft_change: EventHandler<String>,
    on_save_edit: EventHandler<()>,
    on_cancel_edit: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_open_thread_btn: EventHandler<()>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let initials = message
        .sender_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    // Format timestamp using chrono for correct local time.
    let ts_display = {
        let secs = message.timestamp / 1000;
        let nanos = ((message.timestamp % 1000) * 1_000_000) as u32;
        DateTime::from_timestamp(secs as i64, nanos)
            .map(|utc| {
                let local: DateTime<Local> = utc.with_timezone(&Local);
                local.format("%H:%M").to_string()
            })
            .unwrap_or_else(|| {
                // Fallback: derive from raw milliseconds
                let s = message.timestamp / 1000;
                let mins = (s / 60) % 60;
                let hours = (s / 3600) % 24;
                format!("{hours:02}:{mins:02}")
            })
    };

    // M3: Use the is_deleted field instead of string-prefix matching.
    let is_deleted = message.is_deleted;

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
                            "font-size: {}; font-weight: {}; color: {};",
                            typography::SIZE_SM,
                            typography::WEIGHT_SEMIBOLD,
                            if is_own { palette::JADE_400 } else { semantic::TEXT_PRIMARY }
                        ),
                        "{message.sender_name}"
                    }

                    // AI agent badge
                    if is_agent {
                        span {
                            style: format!(
                                "font-size: {}; font-weight: {}; \
                                 color: {}; background: rgba(0,150,255,0.12); \
                                 padding: 1px {}; border-radius: {}; \
                                 white-space: nowrap; flex-shrink: 0;",
                                typography::SIZE_XXS,
                                typography::WEIGHT_SEMIBOLD,
                                palette::SKY_500,
                                spacing::XS,
                                radius::FULL,
                            ),
                            "\u{1F916} AI"
                        }
                    }

                    span {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED
                        ),
                        "{ts_display}"
                    }

                    // Broadcast indicator
                    if message.broadcast {
                        span {
                            style: format!(
                                "font-size: {}; color: {}; \
                                 background: {}; padding: 1px {}; border-radius: {};",
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

                // Inline reply context
                // M4: Show resolved sender name + text snippet instead of raw UUID.
                if let Some(ref reply_id) = message.thread_root {
                    {
                        let reply_id = reply_id.clone();
                        // Use resolved info if available; fall back to the raw ID.
                        let display_name = reply_sender_name
                            .as_deref()
                            .unwrap_or(reply_id.as_str())
                            .to_string();
                        let snippet = reply_text_snippet
                            .as_deref()
                            .unwrap_or("")
                            .to_string();
                        rsx! {
                            div {
                                style: format!(
                                    "display: flex; flex-direction: column; gap: {}; \
                                     padding: {} {}; margin-bottom: {}; \
                                     background: {}; \
                                     border-left: 2px solid {}; \
                                     border-radius: 0 {} {} 0;",
                                    spacing::XXS,
                                    spacing::XXS,
                                    spacing::SM,
                                    spacing::XS,
                                    semantic::BG_SECONDARY,
                                    palette::JADE_500,
                                    radius::SM,
                                    radius::SM
                                ),
                                aria_label: "Quoted reply",

                                span {
                                    style: format!(
                                        "font-size: {}; font-weight: {}; color: {};",
                                        typography::SIZE_XS,
                                        typography::WEIGHT_SEMIBOLD,
                                        palette::JADE_400
                                    ),
                                    "\u{21A9} {display_name}"
                                }

                                if !snippet.is_empty() {
                                    span {
                                        style: format!(
                                            "font-size: {}; color: {}; \
                                             white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                            typography::SIZE_XS,
                                            semantic::TEXT_MUTED
                                        ),
                                        "{snippet}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Message body — edit mode OR markdown display
                if editing {
                    div {
                        style: format!("display: flex; flex-direction: column; gap: {};", spacing::XS),

                        textarea {
                            value: "{edit_draft}",
                            rows: "3",
                            style: format!(
                                "width: 100%; \
                                 background: {}; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {}; \
                                 font-family: {}; \
                                 font-size: {}; \
                                 padding: {}; \
                                 resize: vertical; \
                                 outline: none; \
                                 box-sizing: border-box;",
                                semantic::BG_TERTIARY,
                                semantic::PRIMARY,
                                radius::MD,
                                semantic::TEXT_PRIMARY,
                                typography::FONT_BODY,
                                typography::SIZE_BASE,
                                spacing::SM
                            ),
                            oninput: move |evt| on_edit_draft_change.call(evt.value().to_string()),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Enter && !evt.modifiers().shift() {
                                    evt.prevent_default();
                                    on_save_edit.call(());
                                } else if evt.key() == Key::Escape {
                                    on_cancel_edit.call(());
                                }
                            },
                        }

                        div {
                            style: format!("display: flex; gap: {}; justify-content: flex-end;", spacing::XS),

                            button {
                                style: format!(
                                    "padding: {} {}; background: none; \
                                     border: 1px solid {}; border-radius: {}; \
                                     cursor: pointer; color: {}; font-size: {};",
                                    spacing::XXS, spacing::SM,
                                    semantic::BORDER_SUBTLE, radius::SM,
                                    semantic::TEXT_MUTED,
                                    typography::SIZE_XS
                                ),
                                onclick: move |_| on_cancel_edit.call(()),
                                "Cancel"
                            }

                            button {
                                style: format!(
                                    "padding: {} {}; \
                                     background: {}; \
                                     border: none; border-radius: {}; \
                                     cursor: pointer; color: white; font-size: {};",
                                    spacing::XXS, spacing::SM,
                                    semantic::PRIMARY, radius::SM,
                                    typography::SIZE_XS
                                ),
                                onclick: move |_| on_save_edit.call(()),
                                "Save"
                            }
                        }
                    }
                } else if is_deleted {
                    p {
                        style: format!(
                            "font-size: {}; \
                             color: {}; \
                             line-height: {}; \
                             margin: 0; \
                             font-style: italic;",
                            typography::SIZE_BASE,
                            semantic::TEXT_MUTED,
                            typography::LEADING_NORMAL
                        ),
                        "{message.text}"
                    }
                } else {
                    MarkdownContent {
                        content: message.text.clone(),
                        is_own,
                    }
                }

                // Reactions
                if !message.reactions.is_empty() {
                    div {
                        style: format!(
                            "display: flex; flex-wrap: wrap; gap: {}; margin-top: {};",
                            spacing::XS,
                            spacing::XS
                        ),

                        for (emoji, count) in &message.reactions {
                            span {
                                key: "{emoji}",
                                style: format!(
                                    "display: inline-flex; align-items: center; gap: 2px; \
                                     padding: 1px {}; \
                                     background: {}; border: 1px solid {}; \
                                     border-radius: {}; font-size: {}; cursor: pointer;",
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
                        let reply_label = if message.reply_count == 1 {
                            format!("{} reply", message.reply_count)
                        } else {
                            format!("{} replies", message.reply_count)
                        };
                        rsx! {
                            button {
                                style: format!(
                                    "display: inline-flex; align-items: center; gap: {}; \
                                     margin-top: {}; padding: {} {}; \
                                     background: none; border: 1px solid {}; \
                                     border-radius: {}; cursor: pointer; \
                                     color: {}; font-size: {}; font-family: {}; \
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
                                onclick: move |_| on_open_thread_btn.call(()),

                                span {
                                    style: format!("color: {};", semantic::PRIMARY),
                                    "\u{1F4AC}"
                                }
                                span { "{reply_label}" }
                            }
                        }
                    }
                }
            }

            // Hover actions
            if hovered() && !editing {
                div {
                    style: format!(
                        "display: flex; gap: {}; align-self: flex-start; flex-shrink: 0;",
                        spacing::XXS
                    ),

                    // Reply (inline quote)
                    MessageAction {
                        icon: "\u{21A9}",
                        tooltip: "Reply in chat",
                        onclick: move |_| on_reply.call(()),
                    }

                    // Open thread
                    MessageAction {
                        icon: "\u{1F4AC}",
                        tooltip: "Reply in thread",
                        onclick: {
                            let msg = message.clone();
                            move |_| on_open_thread.call(msg.clone())
                        },
                    }

                    // Edit (own messages only)
                    if is_own && !is_deleted {
                        MessageAction {
                            icon: "\u{270F}",
                            tooltip: "Edit message",
                            onclick: move |_| on_start_edit.call(()),
                        }
                    }

                    // Delete (own messages only)
                    if is_own && !is_deleted {
                        MessageAction {
                            icon: "\u{1F5D1}",
                            tooltip: "Delete message",
                            onclick: move |_| on_delete.call(()),
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// MessageAction
// ---------------------------------------------------------------------------

/// Small action button for message hover actions.
#[component]
fn MessageAction(icon: &'static str, tooltip: String, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            style: format!(
                "width: 28px; height: 28px; \
                 display: flex; align-items: center; justify-content: center; \
                 background: {}; border: 1px solid {}; \
                 border-radius: {}; cursor: pointer; \
                 font-size: {}; transition: {};",
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

// ---------------------------------------------------------------------------
// ChannelEmptyState
// ---------------------------------------------------------------------------

/// Empty state shown when a channel has no messages.
#[component]
fn ChannelEmptyState(channel_name: String) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; flex-direction: column; \
                 align-items: center; justify-content: center; \
                 height: 100%; padding: {}; text-align: center;",
                spacing::HUGE
            ),

            div {
                style: format!(
                    "width: 80px; height: 80px; \
                     display: flex; align-items: center; justify-content: center; \
                     background: {}; border-radius: {}; \
                     font-size: {}; margin-bottom: {};",
                    semantic::BG_TERTIARY, radius::XXL, typography::SIZE_4XL, spacing::XL
                ),
                "#"
            }

            div {
                style: format!(
                    "font-size: {}; font-weight: {}; color: {}; margin-bottom: {};",
                    typography::SIZE_LG, typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY, spacing::XS
                ),
                "Welcome to #{channel_name}"
            }

            div {
                style: format!(
                    "font-size: {}; color: {};",
                    typography::SIZE_SM, semantic::TEXT_MUTED
                ),
                "This is the start of the channel. Send a message to get the conversation going."
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ChannelComposer
// ---------------------------------------------------------------------------

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
                    "display: flex; align-items: flex-end; gap: {}; \
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
                        "width: 32px; height: 32px; \
                         display: flex; align-items: center; justify-content: center; \
                         background: {}; border: none; border-radius: {}; \
                         cursor: {}; opacity: {}; transition: {};",
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
                            "font-size: {}; color: {};",
                            typography::SIZE_SM,
                            semantic::TEXT_INVERSE
                        ),
                        "\u{2191}"
                    }
                }
            }
        }
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Chat Screen
//!
//! Real-time messaging with entity-based channels.
//! Integrates with gossip network for P2P message sync.
//!
//! ## Features
//! - Thread replies with parent message preview
//! - Message reactions with emoji picker
//! - Message status (pending/sent/delivered/read)
//! - Typing indicators (broadcast to peers)

use crate::app::Route;
use crate::state::use_app_state;
use chrono;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

/// Message delivery status
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    #[default]
    Pending,
    Sent,
    Delivered,
    Read,
    Error,
}

/// Reaction on a message
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub user_ids: Vec<String>,
}

/// Wire protocol for chat messages sent via gossip network
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GossipChatMessage {
    id: String,
    sender: String,
    sender_display_name: String,
    channel_id: String,
    content: String,
    timestamp: i64,
    /// Optional reference to parent message for thread replies
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reply_to_id: Option<String>,
    /// Message reactions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reactions: Vec<Reaction>,
    /// Message delivery status
    #[serde(default)]
    status: MessageStatus,
}

/// Wire protocol for typing indicator events
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GossipTypingEvent {
    /// Type discriminator for wire protocol
    #[serde(rename = "type")]
    event_type: String, // "typing_start" or "typing_stop"
    sender: String,
    sender_display_name: String,
    channel_id: String,
    timestamp: i64,
}

/// Wire protocol for read receipt events
#[derive(Clone, Debug, Serialize, Deserialize)]
struct GossipReadReceiptEvent {
    /// Type discriminator for wire protocol
    #[serde(rename = "type")]
    event_type: String, // "read_receipt"
    /// Who read the message
    reader: String,
    /// Channel where the message was read
    channel_id: String,
    /// IDs of messages that were read
    message_ids: Vec<String>,
    /// When the read occurred
    timestamp: i64,
}

/// Typing user info for display
#[derive(Clone, Debug, PartialEq)]
struct TypingUser {
    sender_id: String,
    display_name: String,
    /// When this typing event expires (Unix timestamp)
    expires_at: i64,
}

/// Chat screen component
#[component]
pub fn ChatScreen(entity_id: String) -> Element {
    let navigator = use_navigator();
    let app_state = use_app_state();

    // Check authentication
    if !*app_state.is_authenticated.read() {
        let _ = navigator.push(Route::WelcomeScreen {});
        return rsx! { div { "Redirecting..." } };
    }

    let mut message_input = use_signal(String::new);
    let mut messages = use_signal(Vec::<ChatMessage>::new);
    let mut peer_count = use_signal(|| 0usize);
    let mut is_gossip_active = use_signal(|| false);
    let _last_poll_time = use_signal(|| 0i64);

    // Reply state - which message we're replying to
    let mut replying_to = use_signal(|| None::<ChatMessage>);

    // Emoji picker state - tracks which message has the picker open
    let emoji_picker_open_for = use_signal(|| None::<String>); // message_id when open

    // Typing indicator state
    let mut typing_users = use_signal(Vec::<TypingUser>::new);
    let mut last_typing_broadcast = use_signal(|| 0i64); // Debounce: last time we broadcast typing
    const TYPING_DEBOUNCE_MS: i64 = 2000; // Only broadcast typing every 2 seconds
    const TYPING_EXPIRY_SECS: i64 = 5; // Consider someone stopped typing after 5 seconds

    // Read receipt state
    // Track message IDs we've already sent read receipts for (to avoid duplicates)
    let mut sent_read_receipts = use_signal(std::collections::HashSet::<String>::new);
    // Track message IDs that have been read by others (for status update)
    let mut received_read_receipts =
        use_signal(std::collections::HashMap::<String, Vec<String>>::new); // message_id -> [reader_ids]
    const READ_RECEIPT_BATCH_DELAY_MS: u64 = 500; // Batch read receipts every 500ms

    // Message search state
    let mut search_query = use_signal(String::new);
    let mut search_is_active = use_signal(|| false);

    // Offline message queue state
    // Messages are queued here when gossip network is unavailable, then sent when network returns
    let mut offline_queue = use_signal(Vec::<GossipChatMessage>::new);
    const OFFLINE_QUEUE_RETRY_MS: u64 = 5000; // Retry every 5 seconds

    // Clone values needed in async closures
    let entity_id_for_send = entity_id.clone();
    let entity_id_for_poll = entity_id.clone();
    let entity_id_for_typing = entity_id.clone();
    let entity_id_for_read_receipts = entity_id.clone();
    let entity_id_for_offline_queue = entity_id.clone();

    // Poll for new messages periodically
    let _poll_task = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let entity_id = entity_id_for_poll.clone();
        async move {
            loop {
                // Get core service
                let core = app_state.core.read().clone();

                // Check if gossip is active
                let gossip_active = core.is_gossip_active().await;
                is_gossip_active.set(gossip_active);

                if gossip_active {
                    // Get peer count
                    let count = core.peer_count().await;
                    peer_count.set(count);

                    // Poll for new messages
                    match core.get_all_messages().await {
                        Ok(raw_messages) => {
                            let mut new_messages = Vec::new();
                            let our_four_words =
                                app_state.four_words.read().clone().unwrap_or_default();

                            // First pass: collect all messages for this channel
                            let mut channel_messages: Vec<GossipChatMessage> = Vec::new();
                            for data in raw_messages {
                                if let Ok(gossip_msg) =
                                    serde_json::from_slice::<GossipChatMessage>(&data)
                                    && gossip_msg.channel_id == entity_id
                                {
                                    channel_messages.push(gossip_msg);
                                }
                            }

                            // Count thread replies for parent messages
                            let mut thread_counts: std::collections::HashMap<String, usize> =
                                std::collections::HashMap::new();
                            for msg in &channel_messages {
                                if let Some(ref parent_id) = msg.reply_to_id {
                                    *thread_counts.entry(parent_id.clone()).or_insert(0) += 1;
                                }
                            }

                            // Build content lookup for reply previews (stores truncated preview strings)
                            let content_lookup: std::collections::HashMap<String, String> =
                                channel_messages
                                    .iter()
                                    .map(|m| {
                                        let preview = if m.content.len() > 50 {
                                            format!("{}...", &m.content[..50])
                                        } else {
                                            m.content.clone()
                                        };
                                        (m.id.clone(), preview)
                                    })
                                    .collect();

                            // Convert to ChatMessages with enhanced data
                            for gossip_msg in channel_messages {
                                let is_own = gossip_msg.sender == our_four_words;

                                // Get reply preview if this is a reply
                                let reply_preview = gossip_msg
                                    .reply_to_id
                                    .as_ref()
                                    .and_then(|parent_id| content_lookup.get(parent_id).cloned());

                                let thread_count =
                                    thread_counts.get(&gossip_msg.id).copied().unwrap_or(0);

                                // Determine message status - check if our message has been read by others
                                let status = if is_own
                                    && received_read_receipts.read().contains_key(&gossip_msg.id)
                                {
                                    MessageStatus::Read
                                } else {
                                    gossip_msg.status
                                };

                                new_messages.push(ChatMessage {
                                    id: gossip_msg.id.clone(),
                                    sender: if is_own {
                                        "You".to_string()
                                    } else {
                                        gossip_msg.sender_display_name.clone()
                                    },
                                    sender_id: gossip_msg.sender.clone(),
                                    content: gossip_msg.content,
                                    timestamp: chrono::DateTime::from_timestamp(
                                        gossip_msg.timestamp,
                                        0,
                                    )
                                    .map(|dt| dt.format("%H:%M").to_string())
                                    .unwrap_or_else(|| "??:??".to_string()),
                                    is_own,
                                    reply_to_id: gossip_msg.reply_to_id,
                                    reply_preview,
                                    reactions: gossip_msg.reactions,
                                    status,
                                    thread_count,
                                });
                            }

                            // Sort by timestamp (id contains timestamp)
                            new_messages.sort_by(|a, b| a.id.cmp(&b.id));

                            // Update messages if changed
                            let current_count = messages.read().len();
                            if new_messages.len() != current_count {
                                messages.set(new_messages);
                            }
                        }
                        Err(e) => {
                            // Gossip not active or error - silently ignore
                            if !e.contains("not started") {
                                warn!("Failed to poll messages: {}", e);
                            }
                        }
                    }
                }

                // Poll every 2 seconds
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    });

    // Poll for typing events and clean up expired ones
    let _typing_poll_task = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let entity_id = entity_id_for_typing.clone();
        async move {
            loop {
                // Get core service
                let core = app_state.core.read().clone();
                let our_four_words = app_state.four_words.read().clone().unwrap_or_default();

                // Check if gossip is active
                if core.is_gossip_active().await {
                    // Poll for typing events from gossip messages
                    if let Ok(raw_messages) = core.get_all_messages().await {
                        let now = chrono::Utc::now().timestamp();
                        let mut active_typing: Vec<TypingUser> = Vec::new();

                        for data in raw_messages {
                            // Try to parse as typing event
                            if let Ok(typing_event) =
                                serde_json::from_slice::<GossipTypingEvent>(&data)
                            {
                                // Only process typing events for this channel
                                if typing_event.channel_id == entity_id
                                    && typing_event.event_type == "typing_start"
                                    && typing_event.sender != our_four_words
                                {
                                    // Check if event is recent (within TYPING_EXPIRY_SECS)
                                    let age = now - typing_event.timestamp;
                                    if age < TYPING_EXPIRY_SECS {
                                        // Add to active typing list (avoid duplicates)
                                        if !active_typing
                                            .iter()
                                            .any(|u| u.sender_id == typing_event.sender)
                                        {
                                            active_typing.push(TypingUser {
                                                sender_id: typing_event.sender.clone(),
                                                display_name: typing_event
                                                    .sender_display_name
                                                    .clone(),
                                                expires_at: typing_event.timestamp
                                                    + TYPING_EXPIRY_SECS,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        // Update typing users if changed
                        let current = typing_users.read().clone();
                        if active_typing != current {
                            typing_users.set(active_typing);
                        }
                    }
                }

                // Also clean up expired typing users based on local expiry time
                let now = chrono::Utc::now().timestamp();
                let mut current = typing_users.read().clone();
                let before_len = current.len();
                current.retain(|u| u.expires_at > now);
                if current.len() != before_len {
                    typing_users.set(current);
                }

                // Poll every 1 second for typing updates (faster than message poll)
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    });

    // Read receipt coroutine: polls for incoming receipts and sends outgoing ones
    let _read_receipt_task = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let entity_id = entity_id_for_read_receipts.clone();
        async move {
            loop {
                let core = app_state.core.read().clone();
                let our_four_words = app_state.four_words.read().clone().unwrap_or_default();

                // Check if gossip is active
                if core.is_gossip_active().await {
                    // Step 1: Poll for read receipt events from peers
                    if let Ok(raw_messages) = core.get_all_messages().await {
                        for data in &raw_messages {
                            if let Ok(receipt_event) =
                                serde_json::from_slice::<GossipReadReceiptEvent>(data)
                            {
                                // Only process read receipts for this channel from other users
                                if receipt_event.channel_id == entity_id
                                    && receipt_event.event_type == "read_receipt"
                                    && receipt_event.reader != our_four_words
                                {
                                    // Update received_read_receipts for each message
                                    let mut receipts = received_read_receipts.write();
                                    for msg_id in &receipt_event.message_ids {
                                        receipts
                                            .entry(msg_id.clone())
                                            .or_insert_with(Vec::new)
                                            .push(receipt_event.reader.clone());
                                    }
                                }
                            }
                        }
                    }

                    // Step 2: Find messages from others that we haven't sent receipts for
                    let current_messages = messages.read().clone();
                    let mut unread_msg_ids: Vec<String> = Vec::new();

                    for msg in &current_messages {
                        // Only send receipts for messages from others that we haven't already acknowledged
                        if !msg.is_own && !sent_read_receipts.read().contains(&msg.id) {
                            unread_msg_ids.push(msg.id.clone());
                        }
                    }

                    // Step 3: Send read receipts for unread messages (batched)
                    if !unread_msg_ids.is_empty() {
                        let receipt_event = GossipReadReceiptEvent {
                            event_type: "read_receipt".to_string(),
                            reader: our_four_words.clone(),
                            channel_id: entity_id.clone(),
                            message_ids: unread_msg_ids.clone(),
                            timestamp: chrono::Utc::now().timestamp(),
                        };

                        if let Ok(bytes) = serde_json::to_vec(&receipt_event) {
                            if let Err(e) = core.store_message(bytes).await {
                                if !e.contains("not started") {
                                    info!("Failed to send read receipt: {}", e);
                                }
                            } else {
                                // Mark these as sent to avoid resending
                                for msg_id in unread_msg_ids {
                                    sent_read_receipts.write().insert(msg_id);
                                }
                            }
                        }
                    }
                }

                // Poll every 500ms for read receipt updates
                tokio::time::sleep(std::time::Duration::from_millis(
                    READ_RECEIPT_BATCH_DELAY_MS,
                ))
                .await;
            }
        }
    });

    // Offline queue processing coroutine - retry sending queued messages when network is available
    let _offline_queue_task = use_coroutine(move |_rx: UnboundedReceiver<()>| {
        let _entity_id = entity_id_for_offline_queue.clone();
        async move {
            loop {
                // Check if there are queued messages
                let queue_len = offline_queue.read().len();
                if queue_len > 0 {
                    // Check if gossip is active
                    let core = app_state.core.read().clone();
                    if core.is_gossip_active().await {
                        info!(
                            "Offline queue: {} messages queued, attempting to send",
                            queue_len
                        );

                        // Try to send each queued message
                        let queued_messages: Vec<GossipChatMessage> = offline_queue.read().clone();
                        let mut sent_indices = Vec::new();

                        for (idx, gossip_msg) in queued_messages.iter().enumerate() {
                            let msg_bytes = match serde_json::to_vec(&gossip_msg) {
                                Ok(bytes) => bytes,
                                Err(e) => {
                                    error!("Failed to serialize queued message: {}", e);
                                    continue;
                                }
                            };

                            match core.store_message(msg_bytes).await {
                                Ok(()) => {
                                    info!("Successfully sent queued message: {}", gossip_msg.id);
                                    sent_indices.push(idx);

                                    // Update message status to Sent in UI
                                    let msg_id = gossip_msg.id.clone();
                                    messages.write().iter_mut().for_each(|m| {
                                        if m.id == msg_id {
                                            m.status = MessageStatus::Sent;
                                        }
                                    });
                                }
                                Err(e) => {
                                    warn!("Failed to send queued message {}: {}", gossip_msg.id, e);
                                    // Leave in queue for next retry
                                }
                            }
                        }

                        // Remove successfully sent messages from queue (in reverse order to preserve indices)
                        if !sent_indices.is_empty() {
                            let mut queue = offline_queue.write();
                            for idx in sent_indices.into_iter().rev() {
                                if idx < queue.len() {
                                    queue.remove(idx);
                                }
                            }
                            info!("Offline queue: {} messages remaining", queue.len());
                        }
                    }
                }

                // Poll every 5 seconds
                tokio::time::sleep(std::time::Duration::from_millis(OFFLINE_QUEUE_RETRY_MS)).await;
            }
        }
    });

    // Function to broadcast typing status
    let broadcast_typing = {
        let entity_id = entity_id.clone();
        move || {
            let now_ms = chrono::Utc::now().timestamp_millis();
            let last = *last_typing_broadcast.read();

            // Debounce: only broadcast if enough time has passed
            if now_ms - last < TYPING_DEBOUNCE_MS {
                return;
            }

            last_typing_broadcast.set(now_ms);

            let entity_id = entity_id.clone();
            let our_four_words = app_state.four_words.read().clone().unwrap_or_default();
            let our_display_name = app_state
                .display_name
                .read()
                .clone()
                .unwrap_or_else(|| "Anonymous".to_string());

            let typing_event = GossipTypingEvent {
                event_type: "typing_start".to_string(),
                sender: our_four_words.clone(),
                sender_display_name: our_display_name,
                channel_id: entity_id,
                timestamp: chrono::Utc::now().timestamp(),
            };

            spawn(async move {
                let core = app_state.core.read().clone();

                if let Ok(bytes) = serde_json::to_vec(&typing_event)
                    && let Err(e) = core.store_message(bytes).await
                    // Don't warn for typing events - they're ephemeral
                    && !e.contains("not started")
                {
                    info!("Failed to broadcast typing: {}", e);
                }
            });
        }
    };

    // Handle sending a message - closure that captures needed state
    let mut send_message = {
        let entity_id = entity_id_for_send.clone();
        move || {
            let content = message_input.read().clone();
            if content.trim().is_empty() {
                return;
            }

            let entity_id = entity_id.clone();
            let our_four_words = app_state.four_words.read().clone().unwrap_or_default();
            let our_display_name = app_state
                .display_name
                .read()
                .clone()
                .unwrap_or_else(|| "Anonymous".to_string());

            // Get reply info if replying
            let reply_info = replying_to.read().clone();
            let reply_to_id = reply_info.as_ref().map(|m| m.id.clone());
            let reply_preview = reply_info.as_ref().map(|m| {
                if m.content.len() > 50 {
                    format!("{}...", &m.content[..50])
                } else {
                    m.content.clone()
                }
            });

            // Create gossip message with enhanced fields
            let msg_id = format!(
                "{}-{}",
                chrono::Utc::now().timestamp_millis(),
                uuid::Uuid::new_v4()
            );
            let gossip_msg = GossipChatMessage {
                id: msg_id.clone(),
                sender: our_four_words.clone(),
                sender_display_name: our_display_name.clone(),
                channel_id: entity_id.clone(),
                content: content.clone(),
                timestamp: chrono::Utc::now().timestamp(),
                reply_to_id: reply_to_id.clone(),
                reactions: Vec::new(),
                status: MessageStatus::Pending,
            };

            // Add to local view immediately (optimistic update)
            let local_msg = ChatMessage {
                id: msg_id,
                sender: "You".to_string(),
                sender_id: our_four_words.clone(),
                content: content.clone(),
                timestamp: chrono::Utc::now().format("%H:%M").to_string(),
                is_own: true,
                reply_to_id,
                reply_preview,
                reactions: Vec::new(),
                status: MessageStatus::Pending,
                thread_count: 0,
            };
            messages.write().push(local_msg);
            message_input.set(String::new());

            // Clear reply state
            replying_to.set(None);

            // Clone message for potential offline queueing
            let gossip_msg_for_queue = gossip_msg.clone();
            let msg_id_for_status = gossip_msg.id.clone();

            // Send via gossip network in background
            spawn(async move {
                let core = app_state.core.read().clone();

                // Serialize to JSON for wire protocol
                let msg_bytes = match serde_json::to_vec(&gossip_msg) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!("Failed to serialize message: {}", e);
                        // Queue for retry on serialization failure
                        offline_queue.write().push(gossip_msg_for_queue);
                        // Update status to indicate queued
                        messages.write().iter_mut().for_each(|m| {
                            if m.id == msg_id_for_status {
                                m.status = MessageStatus::Error;
                            }
                        });
                        return;
                    }
                };

                match core.store_message(msg_bytes).await {
                    Ok(()) => {
                        info!("Message sent via gossip network");
                        // Update message status to Sent
                        messages.write().iter_mut().for_each(|m| {
                            if m.id == msg_id_for_status {
                                m.status = MessageStatus::Sent;
                            }
                        });
                    }
                    Err(e) => {
                        warn!(
                            "Failed to send message via gossip: {}, queueing for retry",
                            e
                        );
                        // Queue for offline retry
                        offline_queue.write().push(gossip_msg_for_queue);
                        // Update status to indicate pending retry
                        messages.write().iter_mut().for_each(|m| {
                            if m.id == msg_id_for_status {
                                m.status = MessageStatus::Pending;
                            }
                        });
                    }
                }
            });
        }
    };

    // Network status indicator color
    let status_color = if *is_gossip_active.read() {
        if *peer_count.read() > 0 {
            "#34C759"
        } else {
            "#FF9500"
        }
    } else {
        "#FF3B30"
    };

    let status_text = if *is_gossip_active.read() {
        format!("{} peers", *peer_count.read())
    } else {
        "Offline".to_string()
    };

    // Button styles (precomputed for RSX)
    let button_bg = if *is_gossip_active.read() {
        "#007AFF"
    } else {
        "#ccc"
    };
    let button_cursor = if *is_gossip_active.read() {
        "pointer"
    } else {
        "not-allowed"
    };

    // Search button color (blue when active, gray when inactive)
    let search_btn_color = if *search_is_active.read() {
        "#007AFF"
    } else {
        "#86868b"
    };

    // Pre-compute filtered messages for search
    let query = search_query.read().to_lowercase();
    let is_searching = *search_is_active.read() && !query.is_empty();
    let filtered_messages: Vec<_> = messages
        .read()
        .iter()
        .filter(|message| !is_searching || message.content.to_lowercase().contains(&query))
        .cloned()
        .collect();
    let filtered_count = filtered_messages.len();

    rsx! {
        div {
            class: "chat-screen",
            style: "display: flex; height: 100vh; background: #f5f5f7;",

            // Sidebar
            div {
                style: "width: 280px; background: #1d1d1f; color: white; display: flex; flex-direction: column;",

                // Back button and network status
                div {
                    style: "padding: 16px; display: flex; justify-content: space-between; align-items: center;",

                    button {
                        style: "background: none; border: none; color: #007AFF; cursor: pointer; font-size: 14px;",
                        onclick: move |_| { navigator.push(Route::ContentScreen {}); },
                        "← Back to Home"
                    }

                    // Network status indicator
                    div {
                        style: "display: flex; align-items: center; gap: 6px;",

                        div {
                            style: "width: 8px; height: 8px; border-radius: 50%; background: {status_color};",
                        }
                        span {
                            style: "font-size: 12px; color: #86868b;",
                            "{status_text}"
                        }
                    }
                }

                // Channel list
                nav {
                    style: "flex: 1; padding: 0 16px;",

                    h3 {
                        style: "font-size: 12px; color: #86868b; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px;",
                        "Channels"
                    }

                    button {
                        style: "width: 100%; text-align: left; padding: 8px 12px; background: #007AFF; border: none; border-radius: 8px; color: white; cursor: pointer; font-size: 14px; margin-bottom: 8px;",
                        "# {entity_id}"
                    }
                }
            }

            // Chat area
            div {
                style: "flex: 1; display: flex; flex-direction: column; background: white;",

                // Channel header with search
                div {
                    style: "padding: 16px 24px; border-bottom: 1px solid #e5e5ea;",

                    // Header row with title and search button
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;",

                        h2 {
                            style: "font-size: 18px; color: #1d1d1f;",
                            "# {entity_id}"
                        }

                        // Search toggle button
                        button {
                            style: "background: none; border: none; cursor: pointer; padding: 8px; border-radius: 8px; color: {search_btn_color}; font-size: 18px;",
                            onclick: move |_| {
                                let is_active = *search_is_active.read();
                                search_is_active.set(!is_active);
                                if is_active {
                                    // Clear search when closing
                                    search_query.set(String::new());
                                }
                            },
                            "🔍"
                        }
                    }

                    // Subtitle
                    p {
                        style: "font-size: 12px; color: #86868b;",
                        "Channel conversation • P2P gossip sync"
                    }

                    // Search input (shown when active)
                    if *search_is_active.read() {
                        div {
                            style: "margin-top: 12px; display: flex; gap: 8px; align-items: center;",

                            input {
                                r#type: "text",
                                placeholder: "Search messages...",
                                value: "{search_query}",
                                style: "flex: 1; padding: 10px 14px; border: 1px solid #e5e5ea; border-radius: 8px; font-size: 14px; outline: none;",
                                oninput: move |evt| {
                                    search_query.set(evt.value().clone());
                                }
                            }

                            // Clear search button
                            if !search_query.read().is_empty() {
                                button {
                                    style: "background: #f5f5f7; border: none; cursor: pointer; padding: 8px 12px; border-radius: 8px; color: #86868b; font-size: 12px;",
                                    onclick: move |_| {
                                        search_query.set(String::new());
                                    },
                                    "Clear"
                                }
                            }
                        }

                        // Search results count
                        if is_searching {
                            p {
                                style: "font-size: 12px; color: #86868b; margin-top: 8px;",
                                "{filtered_count} message(s) found"
                            }
                        }
                    }
                }

                // Messages area
                div {
                    style: "flex: 1; overflow-y: auto; padding: 16px 24px;",

                    if messages.read().is_empty() {
                        div {
                            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #86868b;",

                            p {
                                style: "font-size: 48px; margin-bottom: 16px;",
                                "💬"
                            }
                            p {
                                style: "font-size: 16px;",
                                "No messages yet"
                            }
                            p {
                                style: "font-size: 14px;",
                                if *is_gossip_active.read() {
                                    "Start the conversation!"
                                } else {
                                    "Connect to the network to start chatting"
                                }
                            }
                        }
                    } else {
                        // Render filtered messages (pre-filtered for search above)
                        for message in filtered_messages.iter() {
                            MessageBubble {
                                key: "{message.id}",
                                message: message.clone(),
                                on_reply: move |msg: ChatMessage| {
                                    replying_to.set(Some(msg));
                                },
                                on_add_reaction: move |(msg_id, emoji): (String, String)| {
                                    info!("Adding reaction {} to message {}", emoji, msg_id);
                                    // TODO: Implement reaction storage via gossip
                                },
                                emoji_picker_open_for: emoji_picker_open_for,
                            }
                        }
                    }
                }

                // Typing indicator (shows when others are typing)
                if !typing_users.read().is_empty() {
                    div {
                        style: "padding: 8px 24px; display: flex; align-items: center; gap: 8px;",

                        // Animated typing dots
                        div {
                            style: "display: flex; gap: 3px; align-items: center;",

                            span {
                                style: "width: 6px; height: 6px; border-radius: 50%; background: #86868b; animation: typingDot 1.4s infinite; animation-delay: 0s;",
                            }
                            span {
                                style: "width: 6px; height: 6px; border-radius: 50%; background: #86868b; animation: typingDot 1.4s infinite; animation-delay: 0.2s;",
                            }
                            span {
                                style: "width: 6px; height: 6px; border-radius: 50%; background: #86868b; animation: typingDot 1.4s infinite; animation-delay: 0.4s;",
                            }
                        }

                        // Typing users text
                        span {
                            style: "font-size: 13px; color: #86868b; font-style: italic;",
                            {
                                let users = typing_users.read();
                                let names: Vec<_> = users.iter().map(|u| u.display_name.as_str()).collect();
                                match names.len() {
                                    0 => String::new(),
                                    1 => format!("{} is typing...", names[0]),
                                    2 => format!("{} and {} are typing...", names[0], names[1]),
                                    _ => format!("{} and {} others are typing...", names[0], names.len() - 1),
                                }
                            }
                        }
                    }
                }

                // Reply indicator bar (shows when replying to a message)
                if let Some(ref reply_msg) = *replying_to.read() {
                    div {
                        style: "padding: 8px 24px; background: #f0f0f0; border-top: 1px solid #e5e5ea; display: flex; align-items: center; justify-content: space-between;",

                        div {
                            style: "display: flex; align-items: center; gap: 8px;",

                            span {
                                style: "color: #007AFF; font-size: 14px;",
                                "↩ Replying to "
                            }
                            span {
                                style: "font-weight: 600; font-size: 14px; color: #1d1d1f;",
                                "{reply_msg.sender}"
                            }
                            span {
                                style: "color: #86868b; font-size: 13px; max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                "{reply_msg.content}"
                            }
                        }

                        button {
                            style: "background: none; border: none; color: #86868b; cursor: pointer; font-size: 18px; padding: 4px 8px;",
                            title: "Cancel reply",
                            onclick: move |_| replying_to.set(None),
                            "✕"
                        }
                    }
                }

                // Message input
                div {
                    style: "padding: 16px 24px; border-top: 1px solid #e5e5ea; display: flex; gap: 12px;",

                    input {
                        style: "flex: 1; padding: 12px 16px; border: 2px solid #e5e5ea; border-radius: 24px; font-size: 14px; outline: none;",
                        r#type: "text",
                        placeholder: if *is_gossip_active.read() {
                            if replying_to.read().is_some() {
                                "Write a reply..."
                            } else {
                                "Type a message..."
                            }
                        } else {
                            "Connect to network first..."
                        },
                        disabled: !*is_gossip_active.read(),
                        value: "{message_input}",
                        oninput: {
                        let mut broadcast_typing = broadcast_typing.clone();
                        move |evt: Event<FormData>| {
                            message_input.set(evt.value());
                            // Broadcast typing indicator (debounced)
                            broadcast_typing();
                        }
                    },
                        onkeypress: {
                            let mut send_message = send_message.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter && *is_gossip_active.read() {
                                    send_message();
                                }
                            }
                        },
                    }

                    button {
                        style: "padding: 12px 24px; background: {button_bg}; color: white; border: none; border-radius: 24px; font-size: 14px; cursor: {button_cursor};",
                        disabled: !*is_gossip_active.read(),
                        onclick: move |_| send_message(),
                        if replying_to.read().is_some() { "Reply" } else { "Send" }
                    }
                }
            }
        }
    }
}

/// Chat message data for UI display
#[derive(Clone, Debug, PartialEq)]
struct ChatMessage {
    id: String,
    sender: String,
    sender_id: String,
    content: String,
    timestamp: String,
    is_own: bool,
    /// Reference to parent message for thread replies
    reply_to_id: Option<String>,
    /// Preview of the parent message content (truncated)
    reply_preview: Option<String>,
    /// Message reactions
    reactions: Vec<Reaction>,
    /// Delivery status
    status: MessageStatus,
    /// Number of thread replies (if this is a parent message)
    thread_count: usize,
}

/// Common emojis for the emoji picker
const EMOJI_OPTIONS: &[&str] = &[
    "👍", "❤️", "😂", "😮", "😢", "😡", "🎉", "🙏", "👏", "🔥", "✅", "❌",
];

/// Message bubble component with thread replies, reactions, and status
#[component]
fn MessageBubble(
    message: ChatMessage,
    on_reply: EventHandler<ChatMessage>,
    on_add_reaction: EventHandler<(String, String)>, // (message_id, emoji)
    emoji_picker_open_for: Signal<Option<String>>,   // which message has picker open
) -> Element {
    let align = if message.is_own {
        "flex-end"
    } else {
        "flex-start"
    };
    let bg_color = if message.is_own { "#007AFF" } else { "#f0f0f0" };
    let text_color = if message.is_own { "white" } else { "#1d1d1f" };
    let reply_preview_bg = if message.is_own {
        "rgba(255,255,255,0.2)"
    } else {
        "rgba(0,0,0,0.05)"
    };
    let reply_preview_border = if message.is_own {
        "rgba(255,255,255,0.3)"
    } else {
        "#007AFF"
    };

    // Status indicator
    let status_icon = match message.status {
        MessageStatus::Pending => "⏳",
        MessageStatus::Sent => "✓",
        MessageStatus::Delivered => "✓✓",
        MessageStatus::Read => "✓✓", // Could use blue color
        MessageStatus::Error => "⚠️",
    };

    let status_color = match message.status {
        MessageStatus::Read => "#007AFF",
        MessageStatus::Error => "#FF3B30",
        _ => "#86868b",
    };

    // Clone message for closures
    let msg_for_reply = message.clone();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: {align}; margin-bottom: 12px;",

            // Sender name (for non-own messages)
            if !message.is_own {
                span {
                    style: "font-size: 12px; color: #86868b; margin-bottom: 4px;",
                    "{message.sender}"
                }
            }

            // Message bubble container with hover actions
            div {
                style: "max-width: 70%; position: relative;",

                // Main bubble
                div {
                    style: "padding: 12px 16px; background: {bg_color}; color: {text_color}; border-radius: 16px;",

                    // Reply preview (if this is a reply)
                    if let Some(ref preview) = message.reply_preview {
                        div {
                            style: "padding: 8px; margin-bottom: 8px; background: {reply_preview_bg}; border-left: 3px solid {reply_preview_border}; border-radius: 4px; font-size: 12px; opacity: 0.9;",
                            "↩ {preview}"
                        }
                    }

                    // Message content
                    p {
                        style: "font-size: 14px; word-break: break-word; margin: 0;",
                        "{message.content}"
                    }
                }

                // Reactions display
                if !message.reactions.is_empty() {
                    div {
                        style: "display: flex; flex-wrap: wrap; gap: 4px; margin-top: 4px;",

                        for reaction in message.reactions.iter() {
                            span {
                                style: "background: #f0f0f0; padding: 2px 6px; border-radius: 12px; font-size: 12px; cursor: pointer;",
                                title: "{reaction.user_ids.len()} reactions",
                                "{reaction.emoji} {reaction.user_ids.len()}"
                            }
                        }
                    }
                }

                // Action buttons (reply, add reaction)
                div {
                    style: "display: flex; gap: 8px; margin-top: 4px; position: relative;",

                    button {
                        style: "background: none; border: none; color: #86868b; font-size: 12px; cursor: pointer; padding: 2px 6px; border-radius: 4px;",
                        title: "Reply to this message",
                        onclick: move |_| on_reply.call(msg_for_reply.clone()),
                        "↩ Reply"
                    }

                    // Emoji button - toggles picker
                    button {
                        style: "background: none; border: none; color: #86868b; font-size: 12px; cursor: pointer; padding: 2px 6px; border-radius: 4px;",
                        title: "Add reaction",
                        onclick: {
                            let msg_id = message.id.clone();
                            move |_| {
                                let current = emoji_picker_open_for.read().clone();
                                if current.as_ref() == Some(&msg_id) {
                                    emoji_picker_open_for.set(None);
                                } else {
                                    emoji_picker_open_for.set(Some(msg_id.clone()));
                                }
                            }
                        },
                        "😊+"
                    }
                }

                // Emoji picker popup (shows when this message's picker is open)
                if emoji_picker_open_for.read().as_ref() == Some(&message.id) {
                    div {
                        style: "position: absolute; bottom: 100%; left: 0; background: white; border: 1px solid #e5e5ea; border-radius: 12px; padding: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.15); z-index: 100; display: flex; flex-wrap: wrap; gap: 4px; max-width: 200px;",

                        for emoji in EMOJI_OPTIONS.iter() {
                            button {
                                key: "{emoji}",
                                style: "background: none; border: none; font-size: 20px; cursor: pointer; padding: 4px; border-radius: 4px; transition: background 0.15s;",
                                title: "React with {emoji}",
                                onclick: {
                                    let emoji = emoji.to_string();
                                    let msg_id = message.id.clone();
                                    move |_| {
                                        on_add_reaction.call((msg_id.clone(), emoji.clone()));
                                        emoji_picker_open_for.set(None); // Close picker after selection
                                    }
                                },
                                "{emoji}"
                            }
                        }
                    }
                }
            }

            // Footer: timestamp, status, thread count
            div {
                style: "display: flex; align-items: center; gap: 8px; margin-top: 4px;",

                span {
                    style: "font-size: 10px; color: #86868b;",
                    "{message.timestamp}"
                }

                // Status indicator (for own messages)
                if message.is_own {
                    span {
                        style: "font-size: 10px; color: {status_color};",
                        "{status_icon}"
                    }
                }

                // Thread count indicator
                if message.thread_count > 0 {
                    span {
                        style: "font-size: 10px; color: #007AFF; cursor: pointer;",
                        title: "View thread",
                        "💬 {message.thread_count} replies"
                    }
                }
            }
        }
    }
}

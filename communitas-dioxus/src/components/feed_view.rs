//! Social feed view for spaces.
//!
//! A simple post-based feed per space, published and received via the x0x
//! gossip layer.

use crate::design_tokens::{motion, radius, semantic, spacing, typography};
use crate::x0x_contract;
use base64::Engine as _;
use communitas_x0x_client::{X0xClient, X0xWebSocket};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single post in the space feed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeedPost {
    /// Unique post identifier.
    pub id: String,
    /// Post body text.
    pub text: String,
    /// Display name of the author.
    pub author_name: String,
    /// Hex agent ID of the author.
    pub author_id: String,
    /// Unix-epoch milliseconds.
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// Topic helper
// ---------------------------------------------------------------------------

fn feed_topic(group_id: &str) -> String {
    format!(
        "x0x.group.{}.feed",
        x0x_contract::group_prefix(group_id)
    )
}

// ---------------------------------------------------------------------------
// Local history
// ---------------------------------------------------------------------------

/// Path for persisting feed posts to disk.
fn feed_history_path(group_id: &str) -> std::path::PathBuf {
    let prefix = x0x_contract::group_prefix(group_id);
    if let Some(base) = std::env::var_os("COMMUNITAS_X0X_HISTORY_DIR") {
        std::path::PathBuf::from(base).join(format!("x0x-feed-{prefix}.json"))
    } else {
        dirs::data_local_dir()
            .or_else(dirs::data_dir)
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("communitas")
            .join(format!("x0x-feed-{prefix}.json"))
    }
}

const FEED_LIMIT: usize = 200;

async fn load_feed_history(group_id: &str) -> Vec<FeedPost> {
    let path = feed_history_path(group_id);
    match tokio::fs::read(&path).await {
        Ok(bytes) => serde_json::from_slice::<Vec<FeedPost>>(&bytes).unwrap_or_default(),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(err) => {
            warn!(target: "ui.feed", "failed to read feed history {}: {err}", path.display());
            Vec::new()
        }
    }
}

async fn save_feed_history(group_id: &str, posts: &[FeedPost]) {
    let path = feed_history_path(group_id);
    if let Some(parent) = path.parent()
        && let Err(err) = tokio::fs::create_dir_all(parent).await
    {
        warn!(target: "ui.feed", "failed to create feed dir {}: {err}", parent.display());
        return;
    }
    match serde_json::to_vec(posts) {
        Ok(bytes) => {
            if let Err(err) = tokio::fs::write(&path, bytes).await {
                warn!(target: "ui.feed", "failed to write feed history {}: {err}", path.display());
            }
        }
        Err(err) => {
            warn!(target: "ui.feed", "failed to serialize feed history: {err}");
        }
    }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Props for [`FeedView`].
#[derive(Props, Clone, PartialEq)]
pub struct FeedViewProps {
    /// The group/space ID this feed belongs to.
    pub space_id: String,
}

/// Feed tab content -- post composer and scrollable post list.
#[component]
pub fn FeedView(props: FeedViewProps) -> Element {
    let group_id = props.space_id.clone();

    let mut posts: Signal<Vec<FeedPost>> = use_signal(Vec::new);
    let mut composer_text = use_signal(String::new);
    let mut posting = use_signal(|| false);
    let mut ws_connected = use_signal(|| false);

    // Load history on mount
    let history_group_id = group_id.clone();
    use_future(move || {
        let gid = history_group_id.clone();
        async move {
            let history = load_feed_history(&gid).await;
            posts.set(history);
        }
    });

    // Agent identity
    let mut own_agent_id = use_signal(|| Option::<String>::None);
    let mut own_agent_name = use_signal(|| Option::<String>::None);
    use_future(move || async move {
        let client = X0xClient::new();
        if let Ok(agent) = client.agent().await {
            let fallback = x0x_contract::fallback_sender_name(&agent.agent_id);
            let display = client
                .agent_card(None, Some(false))
                .await
                .ok()
                .map(|c| c.card.display_name)
                .filter(|v| !v.trim().is_empty())
                .unwrap_or(fallback);
            own_agent_id.set(Some(agent.agent_id));
            own_agent_name.set(Some(display));
        }
    });

    // WebSocket subscription
    let ws_group_id = group_id.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let group_id = ws_group_id.clone();
        async move {
            let topic = feed_topic(&group_id);

            let ws = match X0xWebSocket::connect().await {
                Ok(ws) => {
                    if let Err(e) = ws.subscribe(vec![topic.clone()]) {
                        error!(target: "ui.feed", "Failed to subscribe to feed topic: {e}");
                        return;
                    }
                    info!(target: "ui.feed", "Subscribed to feed topic: {topic}");
                    ws_connected.set(true);
                    ws
                }
                Err(e) => {
                    warn!(target: "ui.feed", "WebSocket connect failed: {e}");
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
                        if msg_topic == topic
                            && let Ok(bytes) =
                                base64::engine::general_purpose::STANDARD.decode(&payload)
                        {
                            match serde_json::from_slice::<FeedPost>(&bytes) {
                                Ok(post) => {
                                    let save_post = post.clone();
                                    posts.with_mut(|list| {
                                        if !list.iter().any(|p| p.id == save_post.id) {
                                            list.insert(0, save_post);
                                            if list.len() > FEED_LIMIT {
                                                list.truncate(FEED_LIMIT);
                                            }
                                        }
                                    });
                                    save_feed_history(&group_id, &posts()).await;
                                }
                                Err(e) => {
                                    warn!(target: "ui.feed", "Failed to parse feed post: {e}");
                                }
                            }
                        }
                    }
                    communitas_x0x_client::WsInbound::Error { message } => {
                        error!(target: "ui.feed", "WebSocket error: {message}");
                    }
                    _ => {}
                }
            }

            ws_connected.set(false);
        }
    });

    // Post handler
    let post_group_id = group_id.clone();
    let submit_post = move || {
        let text = composer_text();
        if text.trim().is_empty() {
            return;
        }

        let group_id = post_group_id.clone();
        let agent_id = own_agent_id().unwrap_or_default();
        let author_name =
            own_agent_name().unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent_id));

        posting.set(true);
        composer_text.set(String::new());

        spawn(async move {
            let topic = feed_topic(&group_id);
            let post = FeedPost {
                id: uuid::Uuid::new_v4().to_string(),
                text,
                author_name,
                author_id: agent_id,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            };

            match serde_json::to_vec(&post) {
                Ok(json_bytes) => {
                    let client = X0xClient::new();
                    if let Err(e) = client.publish(&topic, &json_bytes).await {
                        error!(target: "ui.feed", "Failed to publish feed post: {e}");
                    } else {
                        info!(target: "ui.feed", "Feed post published to {topic}");
                        posts.with_mut(|list| {
                            if !list.iter().any(|p| p.id == post.id) {
                                list.insert(0, post);
                                if list.len() > FEED_LIMIT {
                                    list.truncate(FEED_LIMIT);
                                }
                            }
                        });
                        save_feed_history(&group_id, &posts()).await;
                    }
                }
                Err(e) => {
                    error!(target: "ui.feed", "Failed to serialize feed post: {e}");
                }
            }

            posting.set(false);
        });
    };

    rsx! {
        div {
            style: format!(
                "display: flex; flex-direction: column; flex: 1; height: 100%; \
                 overflow: hidden; background: {};",
                semantic::BG_PRIMARY
            ),

            // Composer
            FeedComposer {
                value: composer_text(),
                disabled: posting() || !ws_connected(),
                connected: ws_connected(),
                oninput: move |evt: Event<FormData>| composer_text.set(evt.value().to_string()),
                onsubmit: {
                    let mut submit = submit_post.clone();
                    move |_| submit()
                },
            }

            // Post list
            div {
                style: format!(
                    "flex: 1; overflow-y: auto; padding: {} {}; display: flex; \
                     flex-direction: column; gap: {};",
                    spacing::BASE,
                    spacing::XL,
                    spacing::SM
                ),
                role: "feed",

                if posts().is_empty() {
                    div {
                        style: format!(
                            "flex: 1; display: flex; align-items: center; justify-content: center; \
                             color: {}; font-size: {};",
                            semantic::TEXT_MUTED,
                            typography::SIZE_SM
                        ),
                        "No posts yet. Share something with your space!"
                    }
                } else {
                    for post in posts() {
                        FeedPostRow {
                            key: "{post.id}",
                            post: post.clone(),
                            is_own: own_agent_id().as_deref() == Some(&post.author_id),
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/// Post composer at the top of the feed.
#[component]
fn FeedComposer(
    value: String,
    disabled: bool,
    connected: bool,
    oninput: EventHandler<Event<FormData>>,
    onsubmit: EventHandler<()>,
) -> Element {
    let mut focused = use_signal(|| false);

    rsx! {
        div {
            style: format!(
                "padding: {}; border-bottom: 1px solid {}; background: {}; flex-shrink: 0;",
                spacing::BASE,
                semantic::BORDER_SUBTLE,
                semantic::BG_SECONDARY
            ),

            div {
                style: format!(
                    "display: flex; align-items: flex-end; gap: {}; padding: {}; \
                     background: {}; border: 1px solid {}; border-radius: {}; \
                     transition: {};",
                    spacing::SM,
                    spacing::SM,
                    semantic::BG_TERTIARY,
                    if focused() { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                    radius::XL,
                    motion::transition("border-color")
                ),

                textarea {
                    placeholder: "What's happening?",
                    value: "{value}",
                    disabled: disabled,
                    rows: "2",
                    aria_label: "Post input. Press Enter to post, Shift+Enter for new line.",
                    style: format!(
                        "flex: 1; background: transparent; border: none; outline: none; \
                         resize: none; color: {}; font-family: {}; font-size: {}; \
                         line-height: {}; min-height: 40px; max-height: 120px; overflow-y: auto;",
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

                button {
                    style: format!(
                        "padding: {} {}; background: {}; color: {}; border: none; \
                         border-radius: {}; font-size: {}; font-weight: {}; cursor: {}; \
                         opacity: {}; transition: {}; flex-shrink: 0;",
                        spacing::SM,
                        spacing::BASE,
                        if disabled || value.trim().is_empty() {
                            semantic::BG_ELEVATED.to_string()
                        } else {
                            semantic::PRIMARY.to_string()
                        },
                        semantic::TEXT_INVERSE,
                        radius::MD,
                        typography::SIZE_SM,
                        typography::WEIGHT_SEMIBOLD,
                        if disabled { "not-allowed" } else { "pointer" },
                        if disabled || value.trim().is_empty() { "0.5" } else { "1" },
                        motion::transition("opacity, background")
                    ),
                    disabled: disabled || value.trim().is_empty(),
                    onclick: move |_| onsubmit.call(()),
                    "Post"
                }
            }

            // Connection indicator
            div {
                style: format!(
                    "display: flex; align-items: center; gap: {}; margin-top: {};",
                    spacing::XS,
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
                    style: format!("font-size: {}; color: {};", typography::SIZE_XS, semantic::TEXT_MUTED),
                    if connected { "Connected" } else { "Connecting..." }
                }
            }
        }
    }
}

/// A single post row in the feed.
#[component]
fn FeedPostRow(post: FeedPost, is_own: bool) -> Element {
    let mut hovered = use_signal(|| false);

    let initials = post
        .author_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    let agent_short: String = post.author_id.chars().take(8).collect();

    let ts = {
        let secs = post.timestamp / 1000;
        let mins = (secs / 60) % 60;
        let hours = (secs / 3600) % 24;
        format!("{hours:02}:{mins:02}")
    };

    rsx! {
        div {
            style: format!(
                "display: flex; gap: {}; padding: {}; border-radius: {}; \
                 transition: {}; {}",
                spacing::SM,
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
                    "width: 40px; height: 40px; border-radius: {}; background: {}; \
                     display: flex; align-items: center; justify-content: center; \
                     font-size: {}; font-weight: {}; color: {}; flex-shrink: 0;",
                    radius::FULL,
                    if is_own { "rgba(16, 185, 129, 0.25)" } else { semantic::BG_ELEVATED },
                    typography::SIZE_SM,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY
                ),
                "{initials}"
            }

            // Content
            div {
                style: "flex: 1; min-width: 0;",

                // Author line
                div {
                    style: format!(
                        "display: flex; align-items: baseline; gap: {}; margin-bottom: {};",
                        spacing::SM,
                        spacing::XXS
                    ),

                    span {
                        style: format!(
                            "font-size: {}; font-weight: {}; color: {};",
                            typography::SIZE_SM,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "{post.author_name}"
                    }

                    span {
                        style: format!(
                            "font-size: {}; color: {};",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED
                        ),
                        "{agent_short}"
                    }

                    span {
                        style: format!(
                            "font-size: {}; color: {}; margin-left: auto;",
                            typography::SIZE_XXS,
                            semantic::TEXT_MUTED
                        ),
                        "{ts}"
                    }
                }

                // Post text
                p {
                    style: format!(
                        "font-size: {}; color: {}; line-height: {}; margin: 0; \
                         word-wrap: break-word; white-space: pre-wrap;",
                        typography::SIZE_BASE,
                        semantic::TEXT_PRIMARY,
                        typography::LEADING_NORMAL
                    ),
                    "{post.text}"
                }
            }
        }
    }
}

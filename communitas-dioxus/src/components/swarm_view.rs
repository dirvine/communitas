//! Swarm task delegation view for spaces.
//!
//! Displays a task submission form, a live event feed, and an agent roster.
//! Messages are published/subscribed via the x0x gossip layer on per-space
//! swarm topics.

use crate::design_tokens::{motion, palette, radius, semantic, spacing, typography};
use crate::x0x_contract;
use base64::Engine as _;
use communitas_x0x_client::{X0xClient, X0xWebSocket};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// The type of swarm event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwarmEventKind {
    /// A new task was posted.
    Posted,
    /// An agent claimed a task.
    Claimed,
    /// An agent completed a task.
    Completed,
}

/// A swarm event transmitted on the gossip layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwarmEvent {
    /// Event type discriminator.
    #[serde(rename = "type")]
    pub kind: SwarmEventKind,
    /// Unique task identifier.
    pub task_id: String,
    /// Human-readable task description.
    pub description: String,
    /// Required capabilities for this task.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Agent that originated this event.
    pub agent_id: String,
    /// Display name of the agent.
    #[serde(default)]
    pub agent_name: String,
    /// Unix-epoch milliseconds.
    pub timestamp: u64,
    /// Optional result payload (only present for completed events).
    #[serde(default)]
    pub result: Option<String>,
}

/// Summary of an agent observed in the swarm.
#[derive(Clone, Debug)]
struct AgentSummary {
    agent_id: String,
    display_name: String,
    last_kind: SwarmEventKind,
}

// ---------------------------------------------------------------------------
// Topic helpers
// ---------------------------------------------------------------------------

fn swarm_tasks_topic(group_id: &str) -> String {
    format!(
        "x0x.group.{}.swarm/tasks",
        x0x_contract::group_prefix(group_id)
    )
}

fn swarm_results_topic(group_id: &str) -> String {
    format!(
        "x0x.group.{}.swarm/results",
        x0x_contract::group_prefix(group_id)
    )
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// Props for [`SwarmView`].
#[derive(Props, Clone, PartialEq)]
pub struct SwarmViewProps {
    /// The group/space ID this swarm belongs to.
    pub space_id: String,
}

/// Swarm tab content -- task submission, live feed, and agent roster.
#[component]
pub fn SwarmView(props: SwarmViewProps) -> Element {
    let group_id = props.space_id.clone();

    // Local state
    let mut events: Signal<Vec<SwarmEvent>> = use_signal(Vec::new);
    let mut task_desc = use_signal(String::new);
    let mut task_caps = use_signal(String::new);
    let mut posting = use_signal(|| false);
    let mut ws_connected = use_signal(|| false);

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

    // WebSocket subscription for both topics
    let ws_group_id = group_id.clone();
    use_coroutine(move |_: UnboundedReceiver<()>| {
        let group_id = ws_group_id.clone();
        async move {
            let tasks_topic = swarm_tasks_topic(&group_id);
            let results_topic = swarm_results_topic(&group_id);

            let ws = match X0xWebSocket::connect().await {
                Ok(ws) => {
                    if let Err(e) = ws.subscribe(vec![tasks_topic.clone(), results_topic.clone()]) {
                        error!(target: "ui.swarm", "Failed to subscribe to swarm topics: {e}");
                        return;
                    }
                    info!(target: "ui.swarm", "Subscribed to swarm topics for group {group_id}");
                    ws_connected.set(true);
                    ws
                }
                Err(e) => {
                    warn!(target: "ui.swarm", "WebSocket connect failed: {e}");
                    return;
                }
            };

            let mut ws = ws;
            while let Some(inbound) = ws.recv().await {
                match inbound {
                    communitas_x0x_client::WsInbound::Message {
                        topic: _, payload, ..
                    } => {
                        if let Ok(bytes) =
                            base64::engine::general_purpose::STANDARD.decode(&payload)
                        {
                            match serde_json::from_slice::<SwarmEvent>(&bytes) {
                                Ok(evt) => {
                                    events.with_mut(|list| {
                                        if !list
                                            .iter()
                                            .any(|e| e.task_id == evt.task_id && e.kind == evt.kind)
                                        {
                                            list.insert(0, evt);
                                        }
                                    });
                                }
                                Err(e) => {
                                    warn!(target: "ui.swarm", "Failed to parse swarm event: {e}");
                                }
                            }
                        }
                    }
                    communitas_x0x_client::WsInbound::Error { message } => {
                        error!(target: "ui.swarm", "WebSocket error: {message}");
                    }
                    _ => {}
                }
            }

            ws_connected.set(false);
        }
    });

    // Post task handler
    let post_group_id = group_id.clone();
    let post_task = move || {
        let desc = task_desc();
        if desc.trim().is_empty() {
            return;
        }

        let caps_raw = task_caps();
        let caps: Vec<String> = caps_raw
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();

        let group_id = post_group_id.clone();
        let agent_id = own_agent_id().unwrap_or_default();
        let agent_name =
            own_agent_name().unwrap_or_else(|| x0x_contract::fallback_sender_name(&agent_id));

        posting.set(true);
        task_desc.set(String::new());
        task_caps.set(String::new());

        spawn(async move {
            let topic = swarm_tasks_topic(&group_id);
            let evt = SwarmEvent {
                kind: SwarmEventKind::Posted,
                task_id: uuid::Uuid::new_v4().to_string(),
                description: desc,
                capabilities: caps,
                agent_id: agent_id.clone(),
                agent_name: agent_name.clone(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                result: None,
            };

            match serde_json::to_vec(&evt) {
                Ok(json_bytes) => {
                    let client = X0xClient::new();
                    if let Err(e) = client.publish(&topic, &json_bytes).await {
                        error!(target: "ui.swarm", "Failed to publish swarm task: {e}");
                    } else {
                        info!(target: "ui.swarm", "Swarm task published to {topic}");
                        events.with_mut(|list| list.insert(0, evt));
                    }
                }
                Err(e) => {
                    error!(target: "ui.swarm", "Failed to serialize swarm task: {e}");
                }
            }

            posting.set(false);
        });
    };

    // Derive agent roster from events
    let agents: Vec<AgentSummary> = {
        let mut map: HashMap<String, AgentSummary> = HashMap::new();
        for evt in events() {
            map.entry(evt.agent_id.clone())
                .and_modify(|a| a.last_kind = evt.kind.clone())
                .or_insert_with(|| AgentSummary {
                    agent_id: evt.agent_id.clone(),
                    display_name: if evt.agent_name.is_empty() {
                        x0x_contract::fallback_sender_name(&evt.agent_id)
                    } else {
                        evt.agent_name.clone()
                    },
                    last_kind: evt.kind.clone(),
                });
        }
        let mut list: Vec<AgentSummary> = map.into_values().collect();
        list.sort_by(|a, b| a.display_name.cmp(&b.display_name));
        list
    };

    rsx! {
        div {
            style: format!(
                "display: flex; flex: 1; height: 100%; overflow: hidden; background: {};",
                semantic::BG_PRIMARY
            ),

            // Left: task submission
            div {
                style: format!(
                    "width: 300px; flex-shrink: 0; border-right: 1px solid {}; \
                     display: flex; flex-direction: column; padding: {};",
                    semantic::BORDER_SUBTLE,
                    spacing::BASE
                ),

                // Section title
                div {
                    style: format!(
                        "font-size: {}; font-weight: {}; color: {}; margin-bottom: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY,
                        spacing::SM
                    ),
                    "Post a Task"
                }

                // Description textarea
                textarea {
                    placeholder: "Task description",
                    value: "{task_desc}",
                    rows: "4",
                    style: format!(
                        "width: 100%; resize: vertical; background: {}; border: 1px solid {}; \
                         border-radius: {}; padding: {}; color: {}; font-family: {}; \
                         font-size: {}; outline: none; margin-bottom: {};",
                        semantic::BG_TERTIARY,
                        semantic::BORDER_SUBTLE,
                        radius::MD,
                        spacing::SM,
                        semantic::TEXT_PRIMARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        spacing::SM
                    ),
                    oninput: move |evt: Event<FormData>| task_desc.set(evt.value().to_string()),
                }

                // Capabilities input
                input {
                    placeholder: "Required capabilities (comma-separated)",
                    value: "{task_caps}",
                    r#type: "text",
                    style: format!(
                        "width: 100%; background: {}; border: 1px solid {}; \
                         border-radius: {}; padding: {}; color: {}; font-family: {}; \
                         font-size: {}; outline: none; margin-bottom: {};",
                        semantic::BG_TERTIARY,
                        semantic::BORDER_SUBTLE,
                        radius::MD,
                        spacing::SM,
                        semantic::TEXT_PRIMARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        spacing::SM
                    ),
                    oninput: move |evt: Event<FormData>| task_caps.set(evt.value().to_string()),
                }

                // Post button
                button {
                    style: format!(
                        "padding: {} {}; background: {}; color: {}; border: none; \
                         border-radius: {}; font-size: {}; font-weight: {}; \
                         cursor: {}; opacity: {}; transition: {};",
                        spacing::SM,
                        spacing::BASE,
                        semantic::PRIMARY,
                        semantic::TEXT_INVERSE,
                        radius::MD,
                        typography::SIZE_SM,
                        typography::WEIGHT_SEMIBOLD,
                        if posting() || !ws_connected() { "not-allowed" } else { "pointer" },
                        if posting() || !ws_connected() { "0.5" } else { "1" },
                        motion::transition("opacity, background")
                    ),
                    disabled: posting() || !ws_connected(),
                    onclick: {
                        let mut post = post_task.clone();
                        move |_| post()
                    },
                    if posting() { "Posting..." } else { "Post Task" }
                }

                // Connection indicator
                div {
                    style: format!(
                        "display: flex; align-items: center; gap: {}; margin-top: {};",
                        spacing::XS,
                        spacing::SM
                    ),
                    div {
                        style: format!(
                            "width: 8px; height: 8px; border-radius: {}; background: {};",
                            radius::FULL,
                            if ws_connected() { semantic::SUCCESS } else { semantic::WARNING }
                        ),
                    }
                    span {
                        style: format!("font-size: {}; color: {};", typography::SIZE_XS, semantic::TEXT_MUTED),
                        if ws_connected() { "Connected" } else { "Connecting..." }
                    }
                }

                // Agent roster
                div {
                    style: format!(
                        "margin-top: auto; padding-top: {};",
                        spacing::BASE
                    ),

                    div {
                        style: format!(
                            "font-size: {}; font-weight: {}; color: {}; margin-bottom: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_MUTED,
                            spacing::SM
                        ),
                        "Agent Roster"
                    }

                    if agents.is_empty() {
                        div {
                            style: format!("font-size: {}; color: {};", typography::SIZE_XS, semantic::TEXT_MUTED),
                            "No agents observed yet"
                        }
                    } else {
                        div {
                            style: format!("display: flex; flex-wrap: wrap; gap: {};", spacing::XS),
                            for agent in &agents {
                                {
                                    let bg = match agent.last_kind {
                                        SwarmEventKind::Posted => palette::SKY_500,
                                        SwarmEventKind::Claimed => palette::AMBER_500,
                                        SwarmEventKind::Completed => palette::SUCCESS,
                                    };
                                    let label = format!(
                                        "{} ({})",
                                        agent.display_name,
                                        &agent.agent_id[..agent.agent_id.len().min(8)]
                                    );
                                    rsx! {
                                        span {
                                            key: "{agent.agent_id}",
                                            style: format!(
                                                "display: inline-block; padding: {} {}; \
                                                 background: {}; color: {}; border-radius: {}; \
                                                 font-size: {}; font-weight: {};",
                                                spacing::XXS,
                                                spacing::SM,
                                                bg,
                                                semantic::TEXT_INVERSE,
                                                radius::FULL,
                                                typography::SIZE_XS,
                                                typography::WEIGHT_MEDIUM
                                            ),
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Center: live event feed
            div {
                style: format!(
                    "flex: 1; overflow-y: auto; padding: {}; display: flex; flex-direction: column; gap: {};",
                    spacing::BASE,
                    spacing::SM
                ),

                div {
                    style: format!(
                        "font-size: {}; font-weight: {}; color: {}; margin-bottom: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY,
                        spacing::XS
                    ),
                    "Live Feed"
                }

                if events().is_empty() {
                    div {
                        style: format!(
                            "flex: 1; display: flex; align-items: center; justify-content: center; \
                             color: {};",
                            semantic::TEXT_MUTED
                        ),
                        "No swarm events yet. Post a task to get started."
                    }
                } else {
                    for evt in events() {
                        SwarmEventRow { key: "{evt.task_id}-{evt.kind:?}", event: evt.clone() }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

/// A single row in the live event feed.
#[component]
fn SwarmEventRow(event: SwarmEvent) -> Element {
    let mut hovered = use_signal(|| false);

    let (border_color, icon, event_label) = match event.kind {
        SwarmEventKind::Posted => (palette::SKY_500, "\u{1F4CB}", "Task posted"),
        SwarmEventKind::Claimed => (palette::AMBER_500, "\u{1F91A}", "Task claimed"),
        SwarmEventKind::Completed => (palette::SUCCESS, "\u{2705}", "Task completed"),
    };

    let agent_short = if event.agent_name.is_empty() {
        event.agent_id.chars().take(8).collect::<String>()
    } else {
        event.agent_name.clone()
    };

    let ts = {
        let secs = event.timestamp / 1000;
        let mins = (secs / 60) % 60;
        let hours = (secs / 3600) % 24;
        format!("{hours:02}:{mins:02}")
    };

    rsx! {
        div {
            style: format!(
                "border-left: 3px solid {}; padding: {} {}; border-radius: {}; \
                 background: {}; transition: {};",
                border_color,
                spacing::SM,
                spacing::SM,
                radius::MD,
                if hovered() { semantic::BG_TERTIARY } else { semantic::BG_SECONDARY },
                motion::transition("background")
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

            // Header: icon + event label + timestamp
            div {
                style: format!(
                    "display: flex; align-items: center; gap: {}; margin-bottom: {};",
                    spacing::XS,
                    spacing::XXS
                ),

                span { "{icon}" }

                span {
                    style: format!(
                        "font-size: {}; font-weight: {}; color: {};",
                        typography::SIZE_XS,
                        typography::WEIGHT_SEMIBOLD,
                        border_color
                    ),
                    "{event_label}"
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

            // Description
            div {
                style: format!(
                    "font-size: {}; color: {}; margin-bottom: {};",
                    typography::SIZE_SM,
                    semantic::TEXT_PRIMARY,
                    spacing::XXS
                ),
                "{event.description}"
            }

            // Agent + capabilities
            div {
                style: format!(
                    "display: flex; align-items: center; gap: {}; flex-wrap: wrap;",
                    spacing::XS
                ),

                span {
                    style: format!(
                        "font-size: {}; color: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_SECONDARY
                    ),
                    "{agent_short}"
                }

                for cap in &event.capabilities {
                    span {
                        key: "{cap}",
                        style: format!(
                            "display: inline-block; padding: 1px {}; background: {}; \
                             color: {}; border-radius: {}; font-size: {};",
                            spacing::XS,
                            semantic::BG_ELEVATED,
                            semantic::TEXT_MUTED,
                            radius::SM,
                            typography::SIZE_XXS
                        ),
                        "{cap}"
                    }
                }
            }

            // Result (for completed events)
            if let Some(ref result_text) = event.result {
                div {
                    style: format!(
                        "margin-top: {}; padding: {}; background: {}; border-radius: {}; \
                         font-size: {}; color: {}; font-family: {};",
                        spacing::XS,
                        spacing::SM,
                        semantic::BG_TERTIARY,
                        radius::SM,
                        typography::SIZE_XS,
                        semantic::TEXT_SECONDARY,
                        typography::FONT_MONO
                    ),
                    "{result_text}"
                }
            }
        }
    }
}

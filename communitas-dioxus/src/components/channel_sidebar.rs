// SPDX-License-Identifier: MIT OR Apache-2.0

//! Channel sidebar component for space-based messaging.
//!
//! Displays spaces (x0x groups) with their GUI-compatible channel metadata.

use crate::design_tokens::{motion, radius, semantic, spacing, typography};
use crate::models::channel::ChannelMeta;
use crate::x0x_contract;
use communitas_x0x_client::{GroupInfo, GroupSummary, X0xClient};
use dioxus::prelude::*;
use std::collections::HashMap;
use tracing::{error, warn};

/// A space (x0x named group) with its channels.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceEntry {
    /// x0x group ID.
    pub group_id: String,
    /// Human-readable space name.
    pub name: String,
    /// Canonical channel records for this space.
    pub channels: Vec<ChannelMeta>,
    /// Unread message counts per channel name.
    pub unread_counts: HashMap<String, u32>,
}

/// Currently selected channel info passed to the chat view.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectedChannel {
    /// x0x group ID of the parent space.
    pub group_id: String,
    /// Channel name.
    pub channel_name: String,
    /// Full gossip topic for this channel.
    pub topic: String,
    /// Channel metadata if available.
    pub meta: Option<ChannelMeta>,
}

/// Channel sidebar showing spaces and their channels.
#[component]
pub fn ChannelSidebar(
    /// The currently selected channel, if any.
    #[props(default)]
    selected: Option<SelectedChannel>,
    /// Forces the sidebar to reload spaces and channel metadata.
    #[props(default)]
    refresh_key: u64,
    /// Called when a channel is selected.
    on_select: EventHandler<SelectedChannel>,
    /// Called when the user wants to create a new channel.
    on_create_channel: EventHandler<String>,
    /// Opens the local identity/details screen.
    #[props(default)]
    on_open_identity: Option<EventHandler<()>>,
) -> Element {
    let mut spaces = use_signal(Vec::<SpaceEntry>::new);
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut collapsed_spaces: Signal<std::collections::HashSet<String>> =
        use_signal(std::collections::HashSet::new);

    let selected_for_load = selected.clone();
    let on_select_for_load = on_select;
    use_future(move || {
        let refresh_key = refresh_key;
        let selected = selected_for_load.clone();
        let on_select = on_select_for_load;
        async move {
            let _ = refresh_key;
            loading.set(true);
            error_msg.set(None);

            let client = X0xClient::new();
            match load_spaces(&client).await {
                Ok(loaded_spaces) => {
                    if selected.is_none()
                        && let Some(first_space) = loaded_spaces.first()
                        && let Some(first_channel) = first_space.channels.first()
                    {
                        on_select.call(SelectedChannel {
                            group_id: first_space.group_id.clone(),
                            channel_name: first_channel.name.clone(),
                            topic: first_channel.topic.clone(),
                            meta: Some(first_channel.clone()),
                        });
                    }
                    spaces.set(loaded_spaces)
                }
                Err(err) => {
                    error!(target: "ui.channel_sidebar", "Failed to load spaces: {err}");
                    error_msg.set(Some(err));
                }
            }

            loading.set(false);
        }
    });

    rsx! {
        nav {
            aria_label: "Channel navigation",
            style: format!(
                "width: 100%; \
                 height: 100%; \
                 display: flex; \
                 flex-direction: column; \
                 background: {}; \
                 overflow-y: auto; \
                 scrollbar-width: thin; \
                 scrollbar-color: {} transparent;",
                semantic::BG_PRIMARY,
                semantic::BORDER_DEFAULT
            ),

            // Header
            div {
                style: format!(
                    "padding: {} {}; \
                     border-bottom: 1px solid {}; \
                     display: flex; \
                     align-items: center; \
                     justify-content: space-between; \
                     flex-shrink: 0;",
                    spacing::MD,
                    spacing::BASE,
                    semantic::BORDER_SUBTLE
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         letter-spacing: {};",
                        typography::SIZE_XS,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_MUTED,
                        typography::TRACKING_WIDER
                    ),
                    "CHANNELS"
                }

                if let Some(on_open_identity) = on_open_identity {
                    button {
                        style: format!(
                            "padding: {} {}; \
                             border: 1px solid {}; \
                             border-radius: {}; \
                             background: transparent; \
                             color: {}; \
                             font-size: {}; \
                             cursor: pointer;",
                            spacing::XXS,
                            spacing::SM,
                            semantic::BORDER_SUBTLE,
                            radius::MD,
                            semantic::TEXT_SECONDARY,
                            typography::SIZE_XS
                        ),
                        onclick: move |_| on_open_identity.call(()),
                        "Identity"
                    }
                }
            }

            // Content
            div {
                style: format!(
                    "flex: 1; \
                     overflow-y: auto; \
                     padding: {} 0;",
                    spacing::XS
                ),

                if loading() {
                    ChannelSidebarSkeleton {}
                } else if let Some(err) = error_msg() {
                    div {
                        style: format!(
                            "padding: {}; \
                             color: {}; \
                             font-size: {};",
                            spacing::BASE,
                            semantic::ERROR,
                            typography::SIZE_SM
                        ),
                        "{err}"
                    }
                } else if spaces().is_empty() {
                    div {
                        style: format!(
                            "padding: {}; \
                             text-align: center;",
                            spacing::XL
                        ),

                        div {
                            style: format!(
                                "font-size: {}; \
                                 margin-bottom: {};",
                                typography::SIZE_2XL,
                                spacing::SM
                            ),
                            "#"
                        }
                        div {
                            style: format!(
                                "font-size: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                semantic::TEXT_MUTED
                            ),
                            "No spaces yet"
                        }
                    }
                } else {
                    for space in spaces() {
                        SpaceSection {
                            key: "{space.group_id}",
                            space: space.clone(),
                            selected: selected.clone(),
                            collapsed: collapsed_spaces().contains(&space.group_id),
                            on_toggle_collapse: {
                                let gid = space.group_id.clone();
                                move |_| {
                                    let gid = gid.clone();
                                    collapsed_spaces.with_mut(|set| {
                                        if set.contains(&gid) {
                                            set.remove(&gid);
                                        } else {
                                            set.insert(gid);
                                        }
                                    });
                                }
                            },
                            on_select: move |ch: SelectedChannel| on_select.call(ch),
                            on_create_channel: {
                                let gid = space.group_id.clone();
                                move |_| on_create_channel.call(gid.clone())
                            },
                        }
                    }
                }
            }
        }
    }
}

fn fallback_group_info(summary: &GroupSummary) -> GroupInfo {
    GroupInfo {
        group_id: summary.group_id.clone(),
        name: summary.name.clone(),
        description: summary.description.clone(),
        creator: summary.creator.clone(),
        created_at: summary.created_at,
        member_count: summary.member_count,
        chat_topic: Some(x0x_contract::channel_topic(&summary.group_id, "general")),
        metadata_topic: None,
        members: Vec::new(),
        policy: None,
    }
}

async fn load_spaces(client: &X0xClient) -> Result<Vec<SpaceEntry>, String> {
    let groups = client
        .list_groups()
        .await
        .map_err(|err| format!("Failed to load spaces: {err}"))?;

    let mut loaded_spaces = Vec::new();

    for group in groups {
        let group_info = match client.get_group(&group.group_id).await {
            Ok(group_info) => group_info,
            Err(err) => {
                warn!(
                    target: "ui.channel_sidebar",
                    "failed to load full group details for {}: {err}",
                    group.group_id
                );
                fallback_group_info(&group)
            }
        };

        loaded_spaces.push(SpaceEntry {
            group_id: group_info.group_id.clone(),
            name: group_info.name.clone(),
            channels: x0x_contract::load_group_channels(client, &group_info).await,
            unread_counts: HashMap::new(),
        });
    }

    Ok(loaded_spaces)
}

/// A single space section with its channels.
#[component]
fn SpaceSection(
    space: SpaceEntry,
    selected: Option<SelectedChannel>,
    collapsed: bool,
    on_toggle_collapse: EventHandler<()>,
    on_select: EventHandler<SelectedChannel>,
    on_create_channel: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: format!("margin-bottom: {};", spacing::XS),

            // Space header (collapsible)
            button {
                style: format!(
                    "width: 100%; \
                     display: flex; \
                     align-items: center; \
                     gap: {}; \
                     padding: {} {}; \
                     background: none; \
                     border: none; \
                     cursor: pointer; \
                     color: {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     font-family: {}; \
                     text-align: left; \
                     transition: {};",
                    spacing::XS,
                    spacing::XS,
                    spacing::BASE,
                    semantic::TEXT_SECONDARY,
                    typography::SIZE_XS,
                    typography::WEIGHT_SEMIBOLD,
                    typography::FONT_BODY,
                    motion::transition("color, background")
                ),
                aria_expanded: if collapsed { "false" } else { "true" },
                onclick: move |_| on_toggle_collapse.call(()),

                // Collapse indicator
                span {
                    style: format!(
                        "font-size: {}; \
                         transition: {}; \
                         transform: {};",
                        typography::SIZE_XXS,
                        motion::transition("transform"),
                        if collapsed { "rotate(-90deg)" } else { "rotate(0deg)" }
                    ),
                    "\u{25BC}" // down triangle
                }

                span {
                    style: format!(
                        "flex: 1; \
                         text-transform: uppercase; \
                         letter-spacing: {};",
                        typography::TRACKING_WIDER
                    ),
                    "{space.name}"
                }

                // Add channel button
                span {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         opacity: 0.6;",
                        typography::SIZE_SM,
                        semantic::TEXT_MUTED
                    ),
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_create_channel.call(());
                    },
                    "+"
                }
            }

            // Channel list (hidden when collapsed)
            if !collapsed {
                div {
                    style: format!("padding-left: {};", spacing::SM),

                    for channel in &space.channels {
                        ChannelItem {
                            key: "{channel.name}",
                            name: channel.name.clone(),
                            meta: Some(channel.clone()),
                            unread: space.unread_counts.get(&channel.name).copied().unwrap_or(0),
                            is_selected: selected.as_ref().is_some_and(|selected_channel| {
                                selected_channel.group_id == space.group_id
                                    && selected_channel.channel_name == channel.name
                            }),
                            on_click: {
                                let group_id = space.group_id.clone();
                                let meta = channel.clone();
                                move |_| {
                                    on_select.call(SelectedChannel {
                                        group_id: group_id.clone(),
                                        channel_name: meta.name.clone(),
                                        topic: meta.topic.clone(),
                                        meta: Some(meta.clone()),
                                    });
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

/// A single channel item in the sidebar.
#[component]
fn ChannelItem(
    name: String,
    meta: Option<ChannelMeta>,
    unread: u32,
    is_selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    let mut hovered = use_signal(|| false);
    let tooltip = meta.as_ref().map_or_else(
        || format!("#{name}"),
        |channel| {
            if channel.description.is_empty() {
                format!("#{}", channel.name)
            } else {
                format!("#{} - {}", channel.name, channel.description)
            }
        },
    );

    let bg = if is_selected {
        format!("background: {};", semantic::BG_ELEVATED)
    } else if hovered() {
        format!("background: {};", semantic::BG_HOVER)
    } else {
        "background: transparent;".to_string()
    };

    let text_color = if is_selected || unread > 0 {
        semantic::TEXT_PRIMARY
    } else {
        semantic::TEXT_SECONDARY
    };

    let font_weight = if unread > 0 {
        typography::WEIGHT_SEMIBOLD
    } else {
        typography::WEIGHT_NORMAL
    };

    rsx! {
        button {
            style: format!(
                "width: 100%; \
                 display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 {bg} \
                 border: none; \
                 border-radius: {}; \
                 cursor: pointer; \
                 color: {text_color}; \
                 font-size: {}; \
                 font-weight: {font_weight}; \
                 font-family: {}; \
                 text-align: left; \
                 transition: {};",
                spacing::XS,
                spacing::XS,
                spacing::SM,
                radius::MD,
                typography::SIZE_SM,
                typography::FONT_BODY,
                motion::transition("background, color")
            ),
            aria_current: if is_selected { "page" } else { "false" },
            title: "{tooltip}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |_| on_click.call(()),

            // Hash prefix
            span {
                style: format!(
                    "color: {}; \
                     font-size: {}; \
                     font-weight: {};",
                    if is_selected { semantic::PRIMARY } else { semantic::TEXT_MUTED },
                    typography::SIZE_BASE,
                    typography::WEIGHT_NORMAL
                ),
                "#"
            }

            // Channel name
            span {
                style: "flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                "{name}"
            }

            // Unread badge
            if unread > 0 {
                span {
                    style: format!(
                        "min-width: 18px; \
                         height: 18px; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         padding: 0 {}; \
                         background: {}; \
                         color: {}; \
                         font-size: {}; \
                         font-weight: {}; \
                         border-radius: {};",
                        spacing::XS,
                        semantic::PRIMARY,
                        semantic::TEXT_INVERSE,
                        typography::SIZE_XXS,
                        typography::WEIGHT_BOLD,
                        radius::FULL
                    ),
                    "{unread}"
                }
            }
        }
    }
}

/// Skeleton loading state for the channel sidebar.
#[component]
fn ChannelSidebarSkeleton() -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 padding: {};",
                spacing::SM,
                spacing::BASE
            ),
            aria_label: "Loading channels",

            for i in 0..3 {
                div {
                    key: "{i}",
                    // Space header skeleton
                    div {
                        style: format!(
                            "width: 120px; \
                             height: 12px; \
                             border-radius: {}; \
                             background: {}; \
                             margin-bottom: {}; \
                             animation: channelPulse 1.5s ease-in-out infinite;",
                            radius::SM,
                            semantic::BG_TERTIARY,
                            spacing::SM
                        ),
                    }
                    // Channel item skeletons
                    for j in 0..3 {
                        div {
                            key: "{j}",
                            style: format!(
                                "width: {}%; \
                                 height: 28px; \
                                 border-radius: {}; \
                                 background: {}; \
                                 margin-bottom: {}; \
                                 margin-left: {}; \
                                 animation: channelPulse 1.5s ease-in-out infinite;",
                                70 + (j * 5),
                                radius::MD,
                                semantic::BG_ELEVATED,
                                spacing::XXS,
                                spacing::SM
                            ),
                        }
                    }
                }
            }
        }

        style {
            r#"
            @keyframes channelPulse {{
                0%, 100% {{ opacity: 1; }}
                50% {{ opacity: 0.5; }}
            }}
            "#
        }
    }
}

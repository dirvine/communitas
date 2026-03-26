//! Channel sidebar component for space-based messaging.
//!
//! Displays spaces (x0x groups) with their channels organized by category.
//! Channels are loaded from x0x KvStore and subscribed via WebSocket.

use crate::design_tokens::{motion, radius, semantic, spacing, typography};
use crate::models::channel::{ChannelIndex, ChannelMeta};
use communitas_x0x_client::X0xClient;
use dioxus::prelude::*;
use std::collections::HashMap;
use tracing::error;

/// A space (x0x named group) with its channels.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceEntry {
    /// x0x group ID.
    pub group_id: String,
    /// Human-readable space name.
    pub name: String,
    /// Channel index for this space.
    pub channels: ChannelIndex,
    /// Channel metadata keyed by channel name.
    pub channel_meta: HashMap<String, ChannelMeta>,
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
    /// Called when a channel is selected.
    on_select: EventHandler<SelectedChannel>,
    /// Called when the user wants to create a new channel.
    on_create_channel: EventHandler<String>,
) -> Element {
    let mut spaces = use_signal(Vec::<SpaceEntry>::new);
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut collapsed_spaces: Signal<std::collections::HashSet<String>> =
        use_signal(std::collections::HashSet::new);

    // Load spaces and channels from x0x daemon
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        let client = X0xClient::new();

        // Load groups
        let groups = match client.list_groups().await {
            Ok(g) => g,
            Err(e) => {
                error!(target: "ui.channel_sidebar", "Failed to list groups: {e}");
                error_msg.set(Some(format!("Failed to load spaces: {e}")));
                loading.set(false);
                return;
            }
        };

        let mut loaded_spaces = Vec::new();

        for group in &groups {
            let group_id = &group.group_id;
            let group_id_prefix = if group_id.len() >= 16 {
                &group_id[..16]
            } else {
                group_id
            };

            // Try to load channel index from KvStore
            // First, find or create the store for this group
            let stores = client.list_stores().await.unwrap_or_default();
            let store_id = stores
                .iter()
                .find(|s| {
                    s.topic
                        .as_deref()
                        .is_some_and(|t| t.contains(group_id_prefix))
                })
                .map(|s| s.id.clone());

            let mut channel_index = ChannelIndex::default();
            let mut channel_meta_map = HashMap::new();

            if let Some(ref sid) = store_id {
                // Try loading channels_index
                if let Ok(val) = client.get(sid, "channels_index").await
                    && let Ok(decoded) = base64::Engine::decode(
                        &base64::engine::general_purpose::STANDARD,
                        &val.value,
                    )
                    && let Ok(idx) = serde_json::from_slice::<ChannelIndex>(&decoded)
                {
                    channel_index = idx;
                }

                // Load metadata for each known channel
                for ch_name in &channel_index.channels {
                    let key = format!("channel:{ch_name}");
                    if let Ok(val) = client.get(sid, &key).await
                        && let Ok(decoded) = base64::Engine::decode(
                            &base64::engine::general_purpose::STANDARD,
                            &val.value,
                        )
                        && let Ok(meta) = serde_json::from_slice::<ChannelMeta>(&decoded)
                    {
                        channel_meta_map.insert(ch_name.clone(), meta);
                    }
                }
            }

            // If no channels exist yet, create a default "general" channel entry
            if channel_index.channels.is_empty() {
                channel_index.channels.push("general".to_string());
                channel_index
                    .categories
                    .entry("General".to_string())
                    .or_default()
                    .push("general".to_string());
            }

            loaded_spaces.push(SpaceEntry {
                group_id: group_id.clone(),
                name: group.name.clone(),
                channels: channel_index,
                channel_meta: channel_meta_map,
                unread_counts: HashMap::new(),
            });
        }

        spaces.set(loaded_spaces);
        loading.set(false);
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
    let group_id_prefix = if space.group_id.len() >= 16 {
        space.group_id[..16].to_string()
    } else {
        space.group_id.clone()
    };

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

                    // Render by category if available, otherwise flat list
                    if space.channels.categories.is_empty() {
                        for ch_name in &space.channels.channels {
                            ChannelItem {
                                key: "{ch_name}",
                                name: ch_name.clone(),
                                meta: space.channel_meta.get(ch_name).cloned(),
                                unread: space.unread_counts.get(ch_name).copied().unwrap_or(0),
                                is_selected: selected.as_ref().is_some_and(|s| {
                                    s.group_id == space.group_id && s.channel_name == *ch_name
                                }),
                                on_click: {
                                    let gid = space.group_id.clone();
                                    let gid_prefix = group_id_prefix.clone();
                                    let name = ch_name.clone();
                                    let meta = space.channel_meta.get(ch_name).cloned();
                                    move |_| {
                                        let topic = meta.as_ref().map_or_else(
                                            || format!("x0x.group.{}.chat/{}", gid_prefix, name),
                                            |m| m.topic.clone(),
                                        );
                                        on_select.call(SelectedChannel {
                                            group_id: gid.clone(),
                                            channel_name: name.clone(),
                                            topic,
                                            meta: meta.clone(),
                                        });
                                    }
                                },
                            }
                        }
                    } else {
                        for (cat_name, ch_names) in &space.channels.categories {
                            div {
                                key: "{cat_name}",
                                // Category label
                                div {
                                    style: format!(
                                        "padding: {} {} {} {}; \
                                         font-size: {}; \
                                         font-weight: {}; \
                                         color: {}; \
                                         text-transform: uppercase; \
                                         letter-spacing: {};",
                                        spacing::SM,
                                        spacing::SM,
                                        spacing::XXS,
                                        spacing::BASE,
                                        typography::SIZE_XXS,
                                        typography::WEIGHT_MEDIUM,
                                        semantic::TEXT_MUTED,
                                        typography::TRACKING_WIDER
                                    ),
                                    "{cat_name}"
                                }

                                for ch_name in ch_names {
                                    ChannelItem {
                                        key: "{ch_name}",
                                        name: ch_name.clone(),
                                        meta: space.channel_meta.get(ch_name).cloned(),
                                        unread: space.unread_counts.get(ch_name).copied().unwrap_or(0),
                                        is_selected: selected.as_ref().is_some_and(|s| {
                                            s.group_id == space.group_id && s.channel_name == *ch_name
                                        }),
                                        on_click: {
                                            let gid = space.group_id.clone();
                                            let gid_prefix = group_id_prefix.clone();
                                            let name = ch_name.clone();
                                            let meta = space.channel_meta.get(ch_name).cloned();
                                            move |_| {
                                                let topic = meta.as_ref().map_or_else(
                                                    || format!("x0x.group.{}.chat/{}", gid_prefix, name),
                                                    |m| m.topic.clone(),
                                                );
                                                on_select.call(SelectedChannel {
                                                    group_id: gid.clone(),
                                                    channel_name: name.clone(),
                                                    topic,
                                                    meta: meta.clone(),
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

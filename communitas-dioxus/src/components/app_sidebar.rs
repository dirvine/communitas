// SPDX-License-Identifier: MIT OR Apache-2.0

//! Deep Space sidebar navigation component.
//!
//! Matches the x0x GUI sidebar layout exactly:
//!
//! 1. Identity header (display name, truncated agent ID, connection dot)
//! 2. SPACES section — accordion, one space expanded at a time
//!    - Collapsed spaces: presence dot + name
//!    - Expanded space: inline channels + app shortcuts + "Add channel" link
//! 3. DIRECT MESSAGES section
//! 4. SYSTEM section (People, Network, Settings, About)
//! 5. Collapse toggle

use communitas_x0x_client::{GroupInfo, GroupSummary, X0xClient};
use dioxus::prelude::*;
use std::collections::HashMap;
use tracing::warn;

use crate::models::channel::ChannelMeta;
use crate::tokens::{colors, radius, spacing, typography};
use crate::x0x_contract;

/// Props for the sidebar.
#[derive(Props, Clone, PartialEq)]
pub struct AppSidebarProps {
    /// Current route path for highlighting active item.
    pub current_path: String,
    /// Groups/spaces from x0x.
    #[props(default)]
    pub groups: Vec<GroupEntry>,
    /// Contacts list.
    #[props(default)]
    pub contacts: Vec<ContactEntry>,
    /// Agent ID (if known).
    #[props(default)]
    pub agent_id: Option<String>,
    /// Whether the daemon is connected.
    #[props(default)]
    pub connected: bool,
    /// Primary identity label shown in the local x0x header.
    #[props(default)]
    pub identity_label: Option<String>,
    /// Secondary identity label (short agent/machine details).
    #[props(default)]
    pub identity_secondary: Option<String>,
    /// Callback when the local identity header is clicked.
    #[props(default)]
    pub on_identity_click: Option<EventHandler<()>>,
    /// Callback when a navigation item is clicked.
    pub on_navigate: EventHandler<String>,
    /// Callback when a contact is clicked for profile view.
    #[props(default)]
    pub on_contact_click: Option<EventHandler<String>>,
    /// Callback to open the create/join space modal.
    #[props(default)]
    pub on_create_space: Option<EventHandler<()>>,
    /// Number of discovered agents on the network (for presence display).
    #[props(default)]
    pub discovered_agent_count: usize,
}

/// A group entry for the sidebar.
#[derive(Clone, PartialEq)]
pub struct GroupEntry {
    /// Group ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Number of members.
    pub member_count: u32,
}

/// A contact entry for the sidebar.
#[derive(Clone, PartialEq)]
pub struct ContactEntry {
    /// Agent ID of the contact.
    pub agent_id: String,
    /// Display label.
    pub label: String,
    /// Whether the contact is online.
    pub online: bool,
}

/// Truncate an ID to first 6 + last 4 chars.
fn short_id(id: &str) -> String {
    if id.len() <= 12 {
        id.to_owned()
    } else {
        format!("{}..{}", &id[..6], &id[id.len() - 4..])
    }
}

/// App shortcut tabs shown inside an expanded space (below channels).
const SPACE_TABS: &[(&str, &str, &str)] = &[
    ("board", "\u{1F4CB}", "Board"),
    ("files", "\u{1F4C4}", "Files"),
    ("swarm", "\u{1F916}", "Swarm"),
    ("feed", "\u{1F4F0}", "Feed"),
    ("wiki", "\u{1F4CA}", "Wiki"),
    ("web", "\u{1F310}", "Web"),
];

/// Fetch and cache channels for a group from the x0x KV store.
///
/// Returns an empty vec and logs a warning on failure.
async fn fetch_channels_for_group(group_id: &str) -> Vec<ChannelMeta> {
    let client = X0xClient::new();

    let group_info = match client.get_group(group_id).await {
        Ok(info) => info,
        Err(err) => {
            warn!(
                target: "ui.app_sidebar",
                "failed to load group info for {group_id}: {err}"
            );
            // Build a minimal fallback GroupInfo so we can still get the "general" channel
            GroupInfo {
                group_id: group_id.to_string(),
                name: String::new(),
                description: None,
                creator: None,
                created_at: None,
                member_count: None,
                chat_topic: Some(x0x_contract::channel_topic(group_id, "general")),
                metadata_topic: None,
                members: Vec::new(),
            }
        }
    };

    x0x_contract::load_group_channels(&client, &group_info).await
}

/// The main application sidebar.
#[component]
pub fn AppSidebar(props: AppSidebarProps) -> Element {
    let mut collapsed = use_signal(|| false);
    let is_collapsed = *collapsed.read();

    // Accordion: which space is expanded (stores the group_id string).
    let mut expanded_space_id: Signal<Option<String>> = use_signal(|| None);

    // Cache of already-fetched channels per space. Key = group_id.
    let space_channels: Signal<HashMap<String, Vec<ChannelMeta>>> = use_signal(HashMap::new);

    // When the user expands a space, load its channels if not cached yet.
    let expanded_clone = expanded_space_id;
    let mut space_channels_clone = space_channels;
    use_effect(move || {
        let Some(gid) = expanded_clone() else {
            return;
        };
        if space_channels_clone.read().contains_key(&gid) {
            return;
        }
        let gid_clone = gid.clone();
        spawn(async move {
            let channels = fetch_channels_for_group(&gid_clone).await;
            space_channels_clone.write().insert(gid_clone, channels);
        });
    });

    let sidebar_width = if is_collapsed { "72px" } else { "260px" };

    let sidebar_style = format!(
        "width: {sidebar_width}; min-width: {sidebar_width}; height: 100%; \
         display: flex; flex-direction: column; \
         background-color: {}; \
         border-right: 1px solid {}; \
         font-family: {}; \
         transition: width 200ms ease, min-width 200ms ease; \
         overflow: hidden; flex-shrink: 0;",
        colors::SURFACE_CARD,
        colors::BORDER_DEFAULT,
        typography::FONT_SANS,
    );

    let section_header_style = format!(
        "font-size: {}; color: {}; font-weight: 600; \
         text-transform: uppercase; letter-spacing: 0.08em; \
         padding: {} {}; white-space: nowrap; \
         display: flex; align-items: center; justify-content: space-between;",
        typography::TEXT_XS,
        colors::TEXT_MUTED,
        spacing::SM,
        spacing::MD,
    );

    let nav_item_base = format!(
        "display: flex; align-items: center; gap: {}; \
         padding: {} {}; margin: 0 {}; \
         border-radius: {}; cursor: pointer; \
         font-size: {}; white-space: nowrap; \
         transition: background-color 150ms ease, color 150ms ease; \
         text-decoration: none; border: none; background: none; width: calc(100% - {}); \
         text-align: left;",
        spacing::SM,
        spacing::XS,
        spacing::SM,
        spacing::XS,
        radius::MD,
        typography::TEXT_SM,
        spacing::SM,
    );

    let nav_item_style = |path: &str| -> String {
        if props.current_path == path {
            format!(
                "{nav_item_base} background-color: {}; color: {};",
                colors::PRIMARY_HOVER_BG,
                colors::PRIMARY,
            )
        } else {
            format!("{nav_item_base} color: {};", colors::TEXT_SECONDARY,)
        }
    };

    let dot_style = |online: bool| -> String {
        format!(
            "width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; \
             background-color: {};",
            if online {
                colors::SUCCESS
            } else {
                colors::TEXT_MUTED
            }
        )
    };

    // Pre-compute per-group data so we take the read lock once (Fix #7).
    let group_render_data: Vec<(String, String, bool, Vec<ChannelMeta>)> = {
        let snap = space_channels.read();
        props
            .groups
            .iter()
            .map(|g| {
                let loaded = snap.contains_key(&g.id);
                let chans = snap.get(&g.id).cloned().unwrap_or_default();
                (g.id.clone(), g.name.clone(), loaded, chans)
            })
            .collect()
    };

    let identity_dot_color = if props.connected {
        colors::SUCCESS
    } else {
        colors::DANGER
    };
    let identity_label = props
        .identity_label
        .clone()
        .unwrap_or_else(|| "Local x0x".to_string());
    let identity_secondary = props
        .identity_secondary
        .clone()
        .or_else(|| props.agent_id.as_deref().map(short_id));
    let identity_click = props.on_identity_click;
    let identity_button_style = format!(
        "display: flex; align-items: center; gap: 8px; width: 100%; padding: 0; \
         background: none; border: none; text-align: left; cursor: {}; color: inherit;",
        if identity_click.is_some() {
            "pointer"
        } else {
            "default"
        }
    );

    // System nav items (Discover, People, Network, Constitution, Settings, About)
    let system_items: &[(&str, &str, &str)] = &[
        ("Discover", "/discover", "\u{1F50E}"),
        ("People", "/people", "\u{1F465}"),
        ("Network", "/network", "\u{1F310}"),
        ("Constitution", "/constitution", "\u{1F4DC}"),
        ("Settings", "/settings", "\u{2699}"),
        ("About", "/about", "\u{2139}"),
    ];

    rsx! {
        nav {
            class: "app-sidebar",
            style: "{sidebar_style}",
            role: "navigation",
            "aria-label": "Main navigation",

            // === Identity header ===
            div {
                style: format!(
                    "padding: {} {}; border-bottom: 1px solid {}; flex-shrink: 0;",
                    spacing::MD,
                    spacing::MD,
                    colors::BORDER_DEFAULT,
                ),

                button {
                    style: "{identity_button_style}",
                    aria_label: "Open local x0x identity details",
                    onclick: move |_| {
                        if let Some(on_identity_click) = identity_click {
                            on_identity_click.call(());
                        }
                    },

                    // Connection status dot
                    span {
                        style: format!(
                            "width: 10px; height: 10px; border-radius: 50%; \
                             background-color: {}; flex-shrink: 0;",
                            identity_dot_color
                        ),
                    }

                    if !is_collapsed {
                        div {
                            style: "min-width: 0; flex: 1;",
                            // Display name
                            div {
                                style: format!(
                                    "font-size: {}; font-weight: 600; color: {}; \
                                     white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                    typography::TEXT_SM,
                                    colors::TEXT_PRIMARY,
                                ),
                                "{identity_label}"
                            }
                            // Secondary label (truncated agent ID + machine info)
                            if let Some(secondary) = identity_secondary.clone() {
                                div {
                                    style: format!(
                                        "font-size: {}; font-family: {}; color: {}; \
                                         overflow: hidden; text-overflow: ellipsis;",
                                        typography::TEXT_XS,
                                        typography::FONT_MONO,
                                        colors::TEXT_MUTED,
                                    ),
                                    "{secondary}"
                                }
                            }
                        }
                    }
                }
            }

            // === Scrollable content ===
            div {
                style: "flex: 1; overflow-y: auto; overflow-x: hidden; padding-top: 8px;",

                // === SPACES section ===
                if !is_collapsed {
                    div {
                        style: format!("margin-top: {};", spacing::XS),
                        div {
                            style: "{section_header_style}",
                            span { "Spaces" }

                            // + button: opens create/join space modal
                            if let Some(on_create) = &props.on_create_space {
                                {
                                    let on_create = *on_create;
                                    rsx! {
                                        button {
                                            style: format!(
                                                "display: flex; align-items: center; justify-content: center; \
                                                 width: 20px; height: 20px; border-radius: {}; \
                                                 background: none; border: none; cursor: pointer; \
                                                 color: {}; font-size: {}; flex-shrink: 0;",
                                                radius::SM,
                                                colors::TEXT_MUTED,
                                                typography::TEXT_SM,
                                            ),
                                            title: "Create or join a space",
                                            "aria-label": "Create or join a space",
                                            onclick: move |_| on_create.call(()),
                                            "+"
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Collapsed: show a faint spacer
                    div { style: format!("height: {};", spacing::SM) }
                }

                if props.groups.is_empty() {
                    if !is_collapsed {
                        div {
                            style: format!(
                                "padding: {} {}; font-size: {}; color: {};",
                                spacing::XS,
                                spacing::MD,
                                typography::TEXT_XS,
                                colors::TEXT_MUTED,
                            ),
                            "No spaces yet"
                        }
                    }
                } else {
                    // Iterate over pre-computed group_render_data (lock already released).
                    for (group_id, group_name, channels_loaded, channels_for_group) in &group_render_data {
                        {
                            let group_id = group_id.clone();
                            let group_name = group_name.clone();
                            let channels_loaded = *channels_loaded;
                            let channels_for_group = channels_for_group.clone();
                            let on_nav = props.on_navigate;
                            let is_expanded = expanded_space_id().as_deref() == Some(&group_id);
                            let current_path = props.current_path.clone();
                            let on_create_space = props.on_create_space;
                            let agent_count = props.discovered_agent_count;

                            rsx! {
                                SpaceAccordionItem {
                                    key: "{group_id}",
                                    group_id: group_id.clone(),
                                    group_name: group_name.clone(),
                                    is_expanded,
                                    is_collapsed,
                                    channels: channels_for_group,
                                    channels_loaded,
                                    current_path: current_path.clone(),
                                    agent_count,
                                    on_toggle: move |_| {
                                        if is_expanded {
                                            expanded_space_id.set(None);
                                        } else {
                                            expanded_space_id.set(Some(group_id.clone()));
                                        }
                                    },
                                    on_navigate: move |path: String| on_nav.call(path),
                                    on_create_channel: move |_| {
                                        // Open create/join space modal as a proxy for now;
                                        // a dedicated channel-creation modal can be wired here.
                                        if let Some(cb) = on_create_space {
                                            cb.call(());
                                        }
                                    },
                                }
                            }
                        }
                    }
                }

                // === DIRECT MESSAGES section ===
                if !is_collapsed {
                    div {
                        style: format!("margin-top: {};", spacing::MD),
                        div { style: "{section_header_style}", "Direct Messages" }
                    }
                }

                if props.contacts.is_empty() {
                    if !is_collapsed {
                        div {
                            style: format!(
                                "padding: {} {}; font-size: {}; color: {};",
                                spacing::XS,
                                spacing::MD,
                                typography::TEXT_XS,
                                colors::TEXT_MUTED,
                            ),
                            "No contacts yet"
                        }
                    }
                } else {
                    for contact in &props.contacts {
                        {
                            let path = format!("/dm/{}", contact.agent_id);
                            let on_nav = props.on_navigate;
                            let path_clone = path.clone();
                            let label = contact.label.clone();
                            let online = contact.online;
                            let contact_aid = contact.agent_id.clone();
                            let on_profile = props.on_contact_click;
                            rsx! {
                                div {
                                    key: "{path}",
                                    style: "display: flex; align-items: center;",

                                    button {
                                        style: format!(
                                            "{}flex: 1;",
                                            nav_item_style(&path),
                                        ),
                                        onclick: move |_| on_nav.call(path_clone.clone()),
                                        span { style: dot_style(online) }
                                        if !is_collapsed {
                                            span {
                                                style: "overflow: hidden; text-overflow: ellipsis;",
                                                "{label}"
                                            }
                                        }
                                    }

                                    // Profile button (only when expanded)
                                    if !is_collapsed {
                                        if let Some(on_profile) = on_profile {
                                            {
                                                let aid = contact_aid.clone();
                                                rsx! {
                                                    button {
                                                        style: format!(
                                                            "display: flex; align-items: center; justify-content: center; \
                                                             width: 20px; height: 20px; border-radius: {}; \
                                                             background: none; border: none; cursor: pointer; \
                                                             color: {}; font-size: {}; flex-shrink: 0; \
                                                             margin-right: {};",
                                                            radius::SM,
                                                            colors::TEXT_MUTED,
                                                            typography::TEXT_XS,
                                                            spacing::XS,
                                                        ),
                                                        title: "View profile",
                                                        "aria-label": "View contact profile",
                                                        onclick: move |evt| {
                                                            evt.stop_propagation();
                                                            on_profile.call(aid.clone());
                                                        },
                                                        "@"
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
            }

            // === SYSTEM section (fixed at bottom) ===
            div {
                style: format!(
                    "border-top: 1px solid {}; padding: {} 0; flex-shrink: 0;",
                    colors::BORDER_DEFAULT,
                    spacing::XS,
                ),

                if !is_collapsed {
                    div {
                        style: "{section_header_style}",
                        span { "System" }
                    }
                }

                for (label, path, icon) in system_items {
                    {
                        let on_nav = props.on_navigate;
                        let path_str = path.to_string();
                        let item_style = nav_item_style(path);
                        rsx! {
                            button {
                                key: "{path}",
                                style: "{item_style}",
                                onclick: move |_| on_nav.call(path_str.clone()),
                                span {
                                    style: format!(
                                        "font-size: {}; flex-shrink: 0; width: 20px; text-align: center;",
                                        typography::TEXT_SM,
                                    ),
                                    "{icon}"
                                }
                                if !is_collapsed {
                                    span { "{label}" }
                                }
                            }
                        }
                    }
                }

                // Collapse toggle
                button {
                    style: format!(
                        "display: flex; align-items: center; gap: {}; \
                         padding: {} {}; margin: {} {}; \
                         border-radius: {}; cursor: pointer; \
                         font-size: {}; white-space: nowrap; \
                         transition: background-color 150ms ease, color 150ms ease; \
                         text-decoration: none; border: none; background: none; \
                         width: calc(100% - {}); text-align: left; \
                         color: {};",
                        spacing::SM,
                        spacing::XS,
                        spacing::SM,
                        spacing::XS,
                        spacing::XS,
                        radius::MD,
                        typography::TEXT_SM,
                        spacing::SM,
                        colors::TEXT_MUTED,
                    ),
                    onclick: move |_| collapsed.set(!is_collapsed),
                    "aria-label": if is_collapsed { "Expand sidebar" } else { "Collapse sidebar" },
                    span {
                        style: format!(
                            "font-size: {}; flex-shrink: 0; width: 20px; text-align: center; \
                             transition: transform 200ms ease;{}",
                            typography::TEXT_SM,
                            if is_collapsed { " transform: rotate(180deg);" } else { "" }
                        ),
                        "<"
                    }
                    if !is_collapsed {
                        span { "Collapse" }
                    }
                }
            }
        }
    }
}

// ── Space accordion item ─────────────────────────────────────────────────────

/// A single space row in the sidebar accordion.
///
/// When collapsed in the sidebar, shows a dot + name.
/// When expanded, shows channels, app shortcuts, and an "Add channel" link.
#[derive(Props, Clone, PartialEq)]
struct SpaceAccordionItemProps {
    group_id: String,
    group_name: String,
    is_expanded: bool,
    /// Whether the *sidebar itself* is collapsed (icon-only mode).
    is_collapsed: bool,
    channels: Vec<ChannelMeta>,
    /// True once channels have been fetched for this group (even if the list is empty).
    channels_loaded: bool,
    current_path: String,
    on_toggle: EventHandler<()>,
    on_navigate: EventHandler<String>,
    on_create_channel: EventHandler<()>,
    /// Number of discovered agents on the network.
    #[props(default)]
    agent_count: usize,
}

#[component]
fn SpaceAccordionItem(props: SpaceAccordionItemProps) -> Element {
    let space_path = format!("/space/{}", props.group_id);

    // Whether the current path is inside this space
    let is_space_active = props.current_path.starts_with(&space_path);

    // Shared helper for channel/tab button styles (Fix #8).
    let active_item_style = |is_active: bool| -> String {
        format!(
            "display: flex; align-items: center; gap: {}; \
             padding: {} {}; \
             border-radius: {}; cursor: pointer; \
             font-size: {}; white-space: nowrap; \
             transition: background-color 150ms ease, color 150ms ease; \
             border: none; background: {}; \
             width: 100%; text-align: left; \
             color: {};",
            spacing::XS,
            spacing::XS,
            spacing::SM,
            radius::MD,
            typography::TEXT_SM,
            if is_active {
                colors::PRIMARY_HOVER_BG
            } else {
                "none"
            },
            if is_active {
                colors::PRIMARY
            } else {
                colors::TEXT_SECONDARY
            },
        )
    };

    let header_style = format!(
        "display: flex; align-items: center; gap: {}; \
         padding: {} {}; margin: 0 {}; \
         border-radius: {}; cursor: pointer; \
         font-size: {}; white-space: nowrap; \
         transition: background-color 150ms ease, color 150ms ease; \
         border: none; background: {}; \
         width: calc(100% - {}); text-align: left; \
         color: {};",
        spacing::SM,
        spacing::XS,
        spacing::SM,
        spacing::XS,
        radius::MD,
        typography::TEXT_SM,
        if is_space_active {
            "rgba(0, 212, 255, 0.08)"
        } else {
            "none"
        },
        spacing::SM,
        if is_space_active {
            colors::PRIMARY
        } else {
            colors::TEXT_SECONDARY
        },
    );

    rsx! {
        div {
            // Space header row
            button {
                style: "{header_style}",
                aria_expanded: if props.is_expanded { "true" } else { "false" },
                onclick: move |_| props.on_toggle.call(()),

                // Presence / expand indicator dot
                span {
                    style: format!(
                        "width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; \
                         background-color: {};",
                        colors::SUCCESS,
                    ),
                }

                if !props.is_collapsed {
                    // Space name
                    span {
                        style: "flex: 1; overflow: hidden; text-overflow: ellipsis;",
                        "{props.group_name}"
                    }

                    // Expand/collapse chevron
                    span {
                        style: format!(
                            "font-size: {}; color: {}; transition: transform 200ms ease; \
                             transform: {};",
                            typography::TEXT_XS,
                            colors::TEXT_MUTED,
                            if props.is_expanded { "rotate(0deg)" } else { "rotate(-90deg)" }
                        ),
                        "\u{25BC}" // ▼
                    }
                }
            }

            // Expanded section: channels + app tabs + add channel
            if props.is_expanded && !props.is_collapsed {
                div {
                    style: format!(
                        "border-left: 2px solid {}; margin-left: {}; padding-left: {};",
                        colors::BORDER_DEFAULT,
                        spacing::MD,
                        spacing::SM,
                    ),

                    // Channels list (Fix #9: distinguish loading vs truly empty)
                    if !props.channels_loaded {
                        div {
                            style: format!(
                                "padding: {} 0; font-size: {}; color: {};",
                                spacing::XS,
                                typography::TEXT_XS,
                                colors::TEXT_MUTED,
                            ),
                            "Loading channels…"
                        }
                    } else if props.channels.is_empty() {
                        div {
                            style: format!(
                                "padding: {} 0; font-size: {}; color: {};",
                                spacing::XS,
                                typography::TEXT_XS,
                                colors::TEXT_MUTED,
                            ),
                            "No channels yet"
                        }
                    } else {
                        for channel in &props.channels {
                            {
                                let chan_name = channel.name.clone();
                                let on_nav = props.on_navigate;
                                // Fix #1: include channel name in path; keep /space/{id} for "general".
                                let channel_path = if chan_name == "general" {
                                    format!("/space/{}", props.group_id)
                                } else {
                                    format!("/space/{}/{}", props.group_id, chan_name)
                                };
                                // Fix #1: match against both the exact path and the base path for "general".
                                let is_selected = props.current_path == channel_path
                                    || (chan_name == "general"
                                        && props.current_path == format!("/space/{}", props.group_id));
                                let chan_style = active_item_style(is_selected);
                                rsx! {
                                    button {
                                        key: "{chan_name}",
                                        style: "{chan_style}",
                                        aria_current: if is_selected { "page" } else { "false" },
                                        onclick: move |_| on_nav.call(channel_path.clone()),
                                        span {
                                            style: format!(
                                                "color: {}; font-size: {};",
                                                if is_selected { colors::PRIMARY } else { colors::TEXT_MUTED },
                                                typography::TEXT_BASE,
                                            ),
                                            "#"
                                        }
                                        span {
                                            style: "flex: 1; overflow: hidden; text-overflow: ellipsis;",
                                            "{chan_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // "+ Add channel" link
                    button {
                        style: format!(
                            "display: flex; align-items: center; gap: {}; \
                             padding: {} {}; \
                             border-radius: {}; cursor: pointer; \
                             font-size: {}; white-space: nowrap; \
                             transition: color 150ms ease; \
                             border: none; background: none; \
                             width: 100%; text-align: left; \
                             color: {};",
                            spacing::XS,
                            spacing::XS,
                            spacing::SM,
                            radius::MD,
                            typography::TEXT_XS,
                            colors::TEXT_MUTED,
                        ),
                        onclick: move |_| props.on_create_channel.call(()),
                        "+ Add channel"
                    }

                    // Agent presence indicator
                    if props.agent_count > 0 {
                        div {
                            style: format!(
                                "display: flex; align-items: center; gap: {}; \
                                 padding: {} {}; color: {};",
                                spacing::XS,
                                spacing::XS,
                                spacing::SM,
                                colors::TEXT_MUTED,
                            ),
                            span {
                                style: format!("font-size: {};", typography::TEXT_XS),
                                "\u{1F916}"
                            }
                            span {
                                style: format!("font-size: {}; white-space: nowrap;", typography::TEXT_XS),
                                {
                                    let count = props.agent_count;
                                    if count == 1 { "1 agent".to_string() } else { format!("{count} agents") }
                                }
                            }
                        }
                    }

                    // App shortcut tabs (Fix #8: reuse active_item_style helper)
                    div {
                        style: format!("margin-top: {};", spacing::XS),
                        for (tab_key, tab_icon, tab_label) in SPACE_TABS {
                            {
                                let tab_path = format!("/space/{}/{tab_key}", props.group_id);
                                let on_nav = props.on_navigate;
                                let tab_path_clone = tab_path.clone();
                                let is_tab_active = props.current_path == tab_path;
                                let tab_style = active_item_style(is_tab_active);
                                rsx! {
                                    button {
                                        key: "{tab_key}",
                                        style: "{tab_style}",
                                        aria_current: if is_tab_active { "page" } else { "false" },
                                        onclick: move |_| on_nav.call(tab_path_clone.clone()),
                                        span {
                                            style: format!(
                                                "font-size: {}; flex-shrink: 0; width: 20px; text-align: center;",
                                                typography::TEXT_SM,
                                            ),
                                            "{tab_icon}"
                                        }
                                        span { "{tab_label}" }
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

//! Deep Space sidebar navigation component.
//!
//! Sections:
//! 1. Identity header (agent_id, status dot)
//! 2. Spaces section (groups list)
//! 3. Direct Messages section (contacts)
//! 4. System section (People, Network, Settings)
//! 5. Collapse toggle

use dioxus::prelude::*;

use crate::tokens::{colors, radius, spacing, typography};

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
    /// Callback when a navigation item is clicked.
    pub on_navigate: EventHandler<String>,
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

/// The main application sidebar.
#[component]
pub fn AppSidebar(props: AppSidebarProps) -> Element {
    let mut collapsed = use_signal(|| false);
    let is_collapsed = *collapsed.read();

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
         padding: {} {}; white-space: nowrap;",
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

    let is_active = |path: &str| -> bool { props.current_path == path };

    let nav_item_style = |path: &str| -> String {
        if is_active(path) {
            format!(
                "{nav_item_base} background-color: rgba(0, 212, 255, 0.12); color: {};",
                colors::PRIMARY,
            )
        } else {
            format!(
                "{nav_item_base} color: {};",
                colors::TEXT_SECONDARY,
            )
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

    let identity_dot_color = if props.connected {
        colors::SUCCESS
    } else {
        colors::DANGER
    };

    // System nav items
    let system_items = vec![
        ("People", "/people", "P"),
        ("Network", "/network", "N"),
        ("Settings", "/settings", "S"),
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

                div {
                    style: "display: flex; align-items: center; gap: 8px;",

                    // Status dot
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
                            div {
                                style: format!(
                                    "font-size: {}; font-weight: 600; color: {}; \
                                     white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                                    typography::TEXT_SM,
                                    colors::TEXT_PRIMARY,
                                ),
                                "Communitas"
                            }
                            if let Some(ref id) = props.agent_id {
                                div {
                                    style: format!(
                                        "font-size: {}; font-family: {}; color: {}; \
                                         overflow: hidden; text-overflow: ellipsis;",
                                        typography::TEXT_XS,
                                        typography::FONT_MONO,
                                        colors::TEXT_MUTED,
                                    ),
                                    "{short_id(id)}"
                                }
                            }
                        }
                    }
                }
            }

            // === Scrollable content ===
            div {
                style: "flex: 1; overflow-y: auto; overflow-x: hidden; padding-top: 8px;",

                // Dashboard
                button {
                    style: nav_item_style("/"),
                    onclick: {
                        let on_nav = props.on_navigate;
                        move |_| on_nav.call("/".to_string())
                    },
                    span {
                        style: format!("font-size: {}; flex-shrink: 0; width: 20px; text-align: center;", typography::TEXT_SM),
                        "~"
                    }
                    if !is_collapsed {
                        span { "Dashboard" }
                    }
                }

                // === Spaces section ===
                if !is_collapsed {
                    div {
                        style: format!("margin-top: {};", spacing::MD),
                        div { style: "{section_header_style}", "Spaces" }
                    }
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
                    for group in &props.groups {
                        {
                            let path = format!("/space/{}", group.id);
                            let on_nav = props.on_navigate;
                            let path_clone = path.clone();
                            let group_name = group.name.clone();
                            rsx! {
                                button {
                                    key: "{path}",
                                    style: nav_item_style(&path),
                                    onclick: move |_| on_nav.call(path_clone.clone()),
                                    span {
                                        style: format!(
                                            "font-size: {}; flex-shrink: 0; width: 20px; text-align: center; color: {};",
                                            typography::TEXT_SM,
                                            colors::SECONDARY,
                                        ),
                                        "#"
                                    }
                                    if !is_collapsed {
                                        span {
                                            style: "overflow: hidden; text-overflow: ellipsis;",
                                            "{group_name}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // === DMs section ===
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
                            rsx! {
                                button {
                                    key: "{path}",
                                    style: nav_item_style(&path),
                                    onclick: move |_| on_nav.call(path_clone.clone()),
                                    span { style: dot_style(online) }
                                    if !is_collapsed {
                                        span {
                                            style: "overflow: hidden; text-overflow: ellipsis;",
                                            "{label}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // === System section (fixed at bottom) ===
            div {
                style: format!(
                    "border-top: 1px solid {}; padding: {} 0; flex-shrink: 0;",
                    colors::BORDER_DEFAULT,
                    spacing::XS,
                ),

                for (label, path, icon) in &system_items {
                    {
                        let on_nav = props.on_navigate;
                        let path_str = path.to_string();
                        rsx! {
                            button {
                                key: "{path}",
                                style: nav_item_style(path),
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
                        "{nav_item_base} color: {}; margin-top: {};",
                        colors::TEXT_MUTED,
                        spacing::XS,
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

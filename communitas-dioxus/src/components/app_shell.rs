//! Main application shell with 3-column layout.
//!
//! Layout structure:
//! - Left: Sidebar (280px) - Profile, navigation, entities
//! - Center: Main content area (flexible)
//! - Right: Thread/detail panel (360px, collapsible)

use crate::design_tokens::{
    gradients, layout, motion, palette, radius, semantic, spacing, typography,
};
use crate::styles_v2::{self, avatar, flex, presence};
use communitas_ui_api::{PresenceStatus, UnifiedContact, UnifiedEntity, UnifiedEntityType};
use dioxus::prelude::*;

/// Main app shell with 3-column layout.
#[component]
pub fn AppShell(
    /// Whether the thread panel is visible
    #[props(default = false)]
    thread_panel_open: bool,
    /// Sidebar content
    sidebar: Element,
    /// Main content area
    children: Element,
    /// Optional thread/detail panel content
    #[props(default)]
    thread_panel: Option<Element>,
) -> Element {
    rsx! {
        // Skip-to-content link for keyboard accessibility
        a {
            href: "#main-content",
            style: styles_v2::skip_link(),
            class: "skip-link",
            "Skip to main content"
        }

        div {
            style: format!(
                "display: flex; \
                 height: 100vh; \
                 background: {}; \
                 font-family: {}; \
                 overflow: hidden;",
                semantic::BG_BASE,
                typography::FONT_BODY
            ),

            // Sidebar
            aside {
                style: format!(
                    "width: {}; \
                     height: 100%; \
                     background: {}; \
                     border-right: 1px solid {}; \
                     display: flex; \
                     flex-direction: column; \
                     flex-shrink: 0; \
                     overflow: hidden;",
                    layout::SIDEBAR_WIDTH,
                    gradients::SIDEBAR_BG,
                    semantic::BORDER_SUBTLE
                ),
                {sidebar}
            }

            // Main content area
            main {
                id: "main-content",
                tabindex: "-1",
                style: format!(
                    "flex: 1; \
                     display: flex; \
                     flex-direction: column; \
                     min-width: 0; \
                     background: {}; \
                     overflow: hidden; \
                     outline: none;",
                    semantic::BG_PRIMARY
                ),
                {children}
            }

            // Thread/detail panel (collapsible)
            if thread_panel_open {
                if let Some(panel) = thread_panel {
                    aside {
                        style: format!(
                            "width: {}; \
                             height: 100%; \
                             background: {}; \
                             border-left: 1px solid {}; \
                             display: flex; \
                             flex-direction: column; \
                             flex-shrink: 0; \
                             overflow: hidden; \
                             animation: slideIn 200ms ease-out;",
                            layout::THREAD_PANEL_WIDTH,
                            semantic::BG_SECONDARY,
                            semantic::BORDER_SUBTLE
                        ),
                        {panel}
                    }
                }
            }
        }

        style {
            r#"
            @keyframes slideIn {{
                from {{
                    opacity: 0;
                    transform: translateX(20px);
                }}
                to {{
                    opacity: 1;
                    transform: translateX(0);
                }}
            }}

            .skip-link:focus {{
                top: 12px;
                outline: 2px solid rgba(16, 185, 129, 0.6);
                outline-offset: 2px;
            }}
            "#
        }
    }
}

/// Profile header in sidebar.
#[component]
pub fn ProfileHeader(
    display_name: String,
    #[props(default)] pubkey_fingerprint: Option<String>,
    #[props(default = PresenceStatus::Online)] presence: PresenceStatus,
    #[props(default = false)] is_networking: bool,
) -> Element {
    let initials = display_name
        .split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase();

    let presence_style = match presence {
        PresenceStatus::Online => presence::online(),
        PresenceStatus::Away => presence::away(),
        PresenceStatus::Busy => presence::busy(),
        PresenceStatus::Offline | PresenceStatus::Unknown => presence::offline(),
    };

    rsx! {
        div {
            style: format!(
                "padding: {} {}; \
                 border-bottom: 1px solid {};",
                spacing::XL,
                spacing::BASE,
                semantic::BORDER_SUBTLE
            ),

            div {
                style: format!(
                    "{} \
                     gap: {};",
                    flex::row(),
                    spacing::MD
                ),

                // Avatar with presence indicator
                div {
                    style: "position: relative; flex-shrink: 0;",

                    // Avatar
                    div {
                        style: format!(
                            "{} \
                             {} \
                             background: linear-gradient(135deg, {} 0%, {} 100%);",
                            avatar::md(),
                            avatar::with_bg("", semantic::TEXT_PRIMARY),
                            palette::JADE_600,
                            palette::JADE_800
                        ),
                        "{initials}"
                    }

                    // Presence dot
                    div {
                        style: format!(
                            "position: absolute; \
                             bottom: -2px; \
                             right: -2px; \
                             {}",
                            presence_style
                        ),
                    }
                }

                // Name and status
                div {
                    style: format!(
                        "{} \
                         flex: 1; \
                         min-width: 0;",
                        flex::col()
                    ),

                    // Display name
                    div {
                        style: format!(
                            "font-weight: {}; \
                             color: {}; \
                             font-size: {}; \
                             white-space: nowrap; \
                             overflow: hidden; \
                             text-overflow: ellipsis;",
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY,
                            typography::SIZE_BASE
                        ),
                        "{display_name}"
                    }

                    // Pubkey fingerprint
                    if let Some(fingerprint) = pubkey_fingerprint {
                        div {
                            style: format!(
                                "font-family: {}; \
                                 font-size: {}; \
                                 color: {}; \
                                 white-space: nowrap; \
                                 overflow: hidden; \
                                 text-overflow: ellipsis;",
                                typography::FONT_MONO,
                                typography::SIZE_XS,
                                semantic::TEXT_MUTED
                            ),
                            "{fingerprint}"
                        }
                    }
                }

                // Network status indicator
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         padding: {} {}; \
                         background: {}; \
                         border-radius: {}; \
                         font-size: {};",
                        spacing::XS,
                        spacing::XXS,
                        spacing::SM,
                        if is_networking { "rgba(34, 197, 94, 0.1)" } else { semantic::BG_ELEVATED },
                        radius::FULL,
                        typography::SIZE_XS
                    ),

                    div {
                        style: format!(
                            "width: 6px; \
                             height: 6px; \
                             border-radius: {}; \
                             background: {};",
                            radius::FULL,
                            if is_networking { semantic::PRESENCE_ONLINE } else { semantic::PRESENCE_OFFLINE }
                        ),
                    }

                    span {
                        style: format!(
                            "color: {};",
                            if is_networking { semantic::PRESENCE_ONLINE } else { semantic::TEXT_MUTED }
                        ),
                        if is_networking { "Online" } else { "Offline" }
                    }
                }
            }
        }
    }
}

/// Sidebar section header.
#[component]
pub fn SidebarSection(
    title: String,
    #[props(default)] action: Option<Element>,
    #[props(default = true)] collapsible: bool,
    #[props(default = true)] expanded: bool,
    children: Element,
) -> Element {
    let mut is_expanded = use_signal(|| expanded);

    rsx! {
        div {
            style: format!("padding: {} 0;", spacing::XS),

            // Section header
            button {
                style: format!(
                    "{} \
                     width: 100%; \
                     padding: {} {}; \
                     background: none; \
                     border: none; \
                     cursor: {}; \
                     transition: {};",
                    flex::between(),
                    spacing::SM,
                    spacing::BASE,
                    if collapsible { "pointer" } else { "default" },
                    motion::transition("background")
                ),
                onclick: move |_| {
                    if collapsible {
                        is_expanded.set(!is_expanded());
                    }
                },

                // Title with expand indicator
                div {
                    style: format!("{} gap: {};", flex::start(), spacing::SM),

                    if collapsible {
                        span {
                            style: format!(
                                "font-size: {}; \
                                 color: {}; \
                                 transform: rotate({}); \
                                 transition: {};",
                                typography::SIZE_XS,
                                semantic::TEXT_MUTED,
                                if is_expanded() { "90deg" } else { "0deg" },
                                motion::transition("transform")
                            ),
                            "▶"
                        }
                    }

                    span {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {}; \
                             text-transform: uppercase; \
                             letter-spacing: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_MUTED,
                            typography::TRACKING_WIDER
                        ),
                        "{title}"
                    }
                }

                // Action button (e.g., add)
                if let Some(act) = action {
                    {act}
                }
            }

            // Content
            if is_expanded() {
                div {
                    style: format!(
                        "animation: expandIn 150ms ease-out;",
                    ),
                    {children}
                }
            }
        }

        style {
            r#"
            @keyframes expandIn {{
                from {{
                    opacity: 0;
                    max-height: 0;
                }}
                to {{
                    opacity: 1;
                    max-height: 1000px;
                }}
            }}
            "#
        }
    }
}

/// Entity navigation item in sidebar.
#[component]
pub fn EntityNavItem(
    entity: UnifiedEntity,
    #[props(default = false)] selected: bool,
    #[props(default = 0)] unread_count: u32,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let entity_color = match entity.entity_type {
        UnifiedEntityType::Organization => semantic::ENTITY_ORG,
        UnifiedEntityType::Project => semantic::ENTITY_PROJECT,
        UnifiedEntityType::Channel => semantic::ENTITY_CHANNEL,
        UnifiedEntityType::Group => semantic::ENTITY_GROUP,
        UnifiedEntityType::Person => semantic::ENTITY_PERSON,
    };

    let icon = match entity.entity_type {
        UnifiedEntityType::Organization => "◆",
        UnifiedEntityType::Project => "◈",
        UnifiedEntityType::Channel => "#",
        UnifiedEntityType::Group => "◎",
        UnifiedEntityType::Person => "◉",
    };

    let bg_color = if selected {
        format!("background: {}20;", entity_color)
    } else if hovered() {
        format!("background: {};", semantic::BG_HOVER)
    } else {
        "background: transparent;".to_string()
    };

    let left_indicator = if selected {
        format!(
            "position: absolute; \
             left: 0; \
             top: 50%; \
             transform: translateY(-50%); \
             width: 3px; \
             height: 60%; \
             background: {}; \
             border-radius: 0 {} {} 0;",
            entity_color,
            radius::FULL,
            radius::FULL
        )
    } else {
        String::new()
    };

    let entity_type_label = match entity.entity_type {
        UnifiedEntityType::Organization => "organization",
        UnifiedEntityType::Project => "project",
        UnifiedEntityType::Channel => "channel",
        UnifiedEntityType::Group => "group",
        UnifiedEntityType::Person => "contact",
    };

    rsx! {
        button {
            style: format!(
                "{} \
                 position: relative; \
                 width: 100%; \
                 padding: {} {}; \
                 border: none; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {}; \
                 {} \
                 margin: {} 0;",
                flex::start(),
                spacing::SM,
                spacing::BASE,
                radius::MD,
                motion::transition("background"),
                bg_color,
                spacing::XXS
            ),
            aria_label: format!(
                "{} {}{}",
                entity.name,
                entity_type_label,
                if unread_count > 0 { format!(", {} unread", unread_count) } else { String::new() }
            ),
            aria_current: if selected { "page" } else { "false" },
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            // Selection indicator
            if selected {
                div {
                    style: "{left_indicator}",
                }
            }

            // Entity icon
            span {
                style: format!(
                    "width: 24px; \
                     height: 24px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     font-size: {}; \
                     color: {}; \
                     margin-right: {};",
                    typography::SIZE_BASE,
                    entity_color,
                    spacing::SM
                ),
                "{icon}"
            }

            // Entity name
            span {
                style: format!(
                    "flex: 1; \
                     text-align: left; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     white-space: nowrap; \
                     overflow: hidden; \
                     text-overflow: ellipsis;",
                    typography::SIZE_SM,
                    if selected { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_MEDIUM },
                    if selected { semantic::TEXT_PRIMARY } else { semantic::TEXT_SECONDARY }
                ),
                "{entity.name}"
            }

            // Unread badge
            if unread_count > 0 {
                span {
                    style: format!(
                        "min-width: 20px; \
                         height: 20px; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         padding: 0 {}; \
                         background: {}; \
                         color: white; \
                         font-size: {}; \
                         font-weight: {}; \
                         border-radius: {};",
                        spacing::SM,
                        semantic::PRIMARY,
                        typography::SIZE_XS,
                        typography::WEIGHT_SEMIBOLD,
                        radius::FULL
                    ),
                    "{unread_count}"
                }
            }
        }
    }
}

/// Contact navigation item in sidebar.
#[component]
pub fn ContactNavItem(
    contact: UnifiedContact,
    #[props(default = false)] selected: bool,
    #[props(default = 0)] unread_count: u32,
    #[props(default = PresenceStatus::Offline)] presence: PresenceStatus,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let initials = contact
        .display_name
        .split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase();

    let presence_style = match presence {
        PresenceStatus::Online => presence::online(),
        PresenceStatus::Away => presence::away(),
        PresenceStatus::Busy => presence::busy(),
        PresenceStatus::Offline | PresenceStatus::Unknown => presence::offline(),
    };

    let bg_color = if selected {
        format!("background: {}20;", semantic::ENTITY_PERSON)
    } else if hovered() {
        format!("background: {};", semantic::BG_HOVER)
    } else {
        "background: transparent;".to_string()
    };

    let presence_label = match presence {
        PresenceStatus::Online => "online",
        PresenceStatus::Away => "away",
        PresenceStatus::Busy => "busy",
        PresenceStatus::Offline | PresenceStatus::Unknown => "offline",
    };

    rsx! {
        button {
            style: format!(
                "{} \
                 position: relative; \
                 width: 100%; \
                 padding: {} {}; \
                 border: none; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {}; \
                 {} \
                 margin: {} 0;",
                flex::start(),
                spacing::SM,
                spacing::BASE,
                radius::MD,
                motion::transition("background"),
                bg_color,
                spacing::XXS
            ),
            aria_label: format!(
                "{}, {}{}",
                contact.display_name,
                presence_label,
                if unread_count > 0 { format!(", {} unread messages", unread_count) } else { String::new() }
            ),
            aria_current: if selected { "page" } else { "false" },
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            // Avatar with presence
            div {
                style: format!("position: relative; margin-right: {};", spacing::SM),

                // Mini avatar
                div {
                    style: format!(
                        "{} \
                         background: linear-gradient(135deg, {} 0%, {} 100%);",
                        avatar::sm(),
                        palette::CORAL_500,
                        palette::CORAL_400
                    ),
                    span {
                        style: format!(
                            "color: white; \
                             font-size: {}; \
                             font-weight: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD
                        ),
                        "{initials}"
                    }
                }

                // Presence dot
                div {
                    style: format!(
                        "position: absolute; \
                         bottom: -1px; \
                         right: -1px; \
                         {} \
                         width: 8px; \
                         height: 8px;",
                        presence_style
                    ),
                }
            }

            // Contact name
            span {
                style: format!(
                    "flex: 1; \
                     text-align: left; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     white-space: nowrap; \
                     overflow: hidden; \
                     text-overflow: ellipsis; \
                     margin-left: {};",
                    typography::SIZE_SM,
                    if selected { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_MEDIUM },
                    if selected { semantic::TEXT_PRIMARY } else { semantic::TEXT_SECONDARY },
                    spacing::SM
                ),
                "{contact.display_name}"
            }

            // Unread badge
            if unread_count > 0 {
                span {
                    style: format!(
                        "min-width: 20px; \
                         height: 20px; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         padding: 0 {}; \
                         background: {}; \
                         color: white; \
                         font-size: {}; \
                         font-weight: {}; \
                         border-radius: {};",
                        spacing::SM,
                        semantic::PRIMARY,
                        typography::SIZE_XS,
                        typography::WEIGHT_SEMIBOLD,
                        radius::FULL
                    ),
                    "{unread_count}"
                }
            }
        }
    }
}

/// Expandable entity navigation item for hierarchical display.
///
/// Used for organizations/communities that can have children (channels, projects, groups).
/// Shows a chevron that rotates when expanded, and indented children when open.
#[component]
pub fn ExpandableEntityNavItem(
    entity: UnifiedEntity,
    /// Child elements to render when expanded
    #[props(default)]
    children: Element,
    #[props(default = false)] selected: bool,
    #[props(default = 0)] unread_count: u32,
    #[props(default = false)] expanded: bool,
    /// Whether this entity has children to show
    #[props(default = false)]
    has_children: bool,
    /// Click handler for the entity row (navigation)
    onclick: EventHandler<MouseEvent>,
    /// Toggle handler for expand/collapse
    ontoggle: EventHandler<bool>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let entity_color = match entity.entity_type {
        UnifiedEntityType::Organization => semantic::ENTITY_ORG,
        UnifiedEntityType::Project => semantic::ENTITY_PROJECT,
        UnifiedEntityType::Channel => semantic::ENTITY_CHANNEL,
        UnifiedEntityType::Group => semantic::ENTITY_GROUP,
        UnifiedEntityType::Person => semantic::ENTITY_PERSON,
    };

    let icon = match entity.entity_type {
        UnifiedEntityType::Organization => "◆",
        UnifiedEntityType::Project => "◈",
        UnifiedEntityType::Channel => "#",
        UnifiedEntityType::Group => "◎",
        UnifiedEntityType::Person => "◉",
    };

    let bg_color = if selected {
        format!("background: {}20;", entity_color)
    } else if hovered() {
        format!("background: {};", semantic::BG_HOVER)
    } else {
        "background: transparent;".to_string()
    };

    let left_indicator = if selected {
        format!(
            "position: absolute; \
             left: 0; \
             top: 50%; \
             transform: translateY(-50%); \
             width: 3px; \
             height: 60%; \
             background: {}; \
             border-radius: 0 {} {} 0;",
            entity_color,
            radius::FULL,
            radius::FULL
        )
    } else {
        String::new()
    };

    let current_expanded = expanded;

    rsx! {
        div {
            style: "display: flex; flex-direction: column;",

            // Main entity row
            div {
                style: format!(
                    "{} \
                     position: relative; \
                     width: 100%; \
                     padding: {} {}; \
                     border: none; \
                     border-radius: {}; \
                     cursor: pointer; \
                     transition: {}; \
                     {} \
                     margin: {} 0;",
                    flex::start(),
                    spacing::SM,
                    spacing::BASE,
                    radius::MD,
                    motion::transition("background"),
                    bg_color,
                    spacing::XXS
                ),
                onmouseenter: move |_| hovered.set(true),
                onmouseleave: move |_| hovered.set(false),

                // Selection indicator
                if selected {
                    div {
                        style: "{left_indicator}",
                    }
                }

                // Chevron button (only if has children)
                if has_children {
                    button {
                        style: format!(
                            "width: 20px; \
                             height: 20px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: none; \
                             border: none; \
                             cursor: pointer; \
                             padding: 0; \
                             margin-right: {}; \
                             color: {}; \
                             font-size: {}; \
                             transform: rotate({}); \
                             transition: {};",
                            spacing::XXS,
                            semantic::TEXT_MUTED,
                            typography::SIZE_XS,
                            if current_expanded { "90deg" } else { "0deg" },
                            motion::transition("transform")
                        ),
                        aria_label: if current_expanded {
                            format!("Collapse {}", entity.name)
                        } else {
                            format!("Expand {}", entity.name)
                        },
                        aria_expanded: if current_expanded { "true" } else { "false" },
                        onclick: move |evt| {
                            evt.stop_propagation();
                            ontoggle.call(!current_expanded);
                        },
                        "▶"
                    }
                } else {
                    // Spacer to maintain alignment when no chevron
                    div {
                        style: format!("width: 20px; margin-right: {};", spacing::XXS),
                    }
                }

                // Entity icon (clickable for navigation)
                button {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         flex: 1; \
                         min-width: 0; \
                         background: none; \
                         border: none; \
                         cursor: pointer; \
                         padding: 0;",
                    ),
                    aria_label: format!(
                        "Navigate to {} {}{}",
                        entity.name,
                        match entity.entity_type {
                            UnifiedEntityType::Organization => "organization",
                            UnifiedEntityType::Project => "project",
                            UnifiedEntityType::Channel => "channel",
                            UnifiedEntityType::Group => "group",
                            UnifiedEntityType::Person => "contact",
                        },
                        if unread_count > 0 { format!(", {} unread", unread_count) } else { String::new() }
                    ),
                    aria_current: if selected { "page" } else { "false" },
                    onclick: move |evt| onclick.call(evt),

                    span {
                        style: format!(
                            "width: 24px; \
                             height: 24px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             font-size: {}; \
                             color: {}; \
                             margin-right: {};",
                            typography::SIZE_BASE,
                            entity_color,
                            spacing::SM
                        ),
                        "{icon}"
                    }

                    // Entity name
                    span {
                        style: format!(
                            "flex: 1; \
                             text-align: left; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {}; \
                             white-space: nowrap; \
                             overflow: hidden; \
                             text-overflow: ellipsis;",
                            typography::SIZE_SM,
                            if selected { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_MEDIUM },
                            if selected { semantic::TEXT_PRIMARY } else { semantic::TEXT_SECONDARY }
                        ),
                        "{entity.name}"
                    }
                }

                // Unread badge
                if unread_count > 0 {
                    span {
                        style: format!(
                            "min-width: 20px; \
                             height: 20px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             padding: 0 {}; \
                             background: {}; \
                             color: white; \
                             font-size: {}; \
                             font-weight: {}; \
                             border-radius: {};",
                            spacing::SM,
                            semantic::PRIMARY,
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            radius::FULL
                        ),
                        "{unread_count}"
                    }
                }
            }

            // Children container (animated)
            if current_expanded {
                div {
                    style: format!(
                        "padding-left: {}; \
                         animation: expandChildren 150ms ease-out;",
                        spacing::XL
                    ),
                    {children}
                }
            }
        }

        style {
            r#"
            @keyframes expandChildren {{
                from {{
                    opacity: 0;
                    transform: translateY(-4px);
                }}
                to {{
                    opacity: 1;
                    transform: translateY(0);
                }}
            }}
            "#
        }
    }
}

/// Quick action button for sidebar sections.
#[component]
pub fn QuickActionButton(
    icon: String,
    onclick: EventHandler<MouseEvent>,
    /// Accessible label describing the button's action
    #[props(default = "Action".to_string())]
    label: String,
) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        button {
            style: format!(
                "width: 24px; \
                 height: 24px; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: {}; \
                 border: none; \
                 border-radius: {}; \
                 color: {}; \
                 font-size: {}; \
                 cursor: pointer; \
                 transition: {};",
                if hovered() { semantic::BG_HOVER } else { "transparent" },
                radius::MD,
                if hovered() { semantic::PRIMARY } else { semantic::TEXT_MUTED },
                typography::SIZE_SM,
                motion::transition("all")
            ),
            aria_label: "{label}",
            title: "{label}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),
            "{icon}"
        }
    }
}

/// Search bar for sidebar.
#[component]
pub fn SidebarSearch(
    value: String,
    placeholder: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let mut focused = use_signal(|| false);

    rsx! {
        div {
            style: format!("padding: {} {};", spacing::SM, spacing::BASE),

            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {}; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     padding: {} {}; \
                     transition: {};",
                    spacing::SM,
                    semantic::BG_TERTIARY,
                    if focused() { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                    radius::MD,
                    spacing::SM,
                    spacing::MD,
                    motion::transition("border-color")
                ),

                // Search icon
                span {
                    style: format!(
                        "color: {}; \
                         font-size: {};",
                        semantic::TEXT_MUTED,
                        typography::SIZE_SM
                    ),
                    "⌕"
                }

                input {
                    id: "global-search-input",
                    r#type: "search",
                    placeholder: "{placeholder}",
                    value: "{value}",
                    aria_label: "Search spaces and contacts. Press Command K or Control K to focus.",
                    style: format!(
                        "flex: 1; \
                         background: transparent; \
                         border: none; \
                         outline: none; \
                         color: {}; \
                         font-family: {}; \
                         font-size: {};",
                        semantic::TEXT_PRIMARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM
                    ),
                    onfocus: move |_| focused.set(true),
                    onblur: move |_| focused.set(false),
                    oninput: move |evt| oninput.call(evt),
                }

                // Keyboard shortcut hint (shown when not focused)
                if !focused() && value.is_empty() {
                    kbd {
                        style: format!(
                            "padding: 2px {}; \
                             background: {}; \
                             border: 1px solid {}; \
                             border-radius: {}; \
                             font-family: {}; \
                             font-size: {}; \
                             color: {}; \
                             opacity: 0.7;",
                            spacing::XS,
                            semantic::BG_SECONDARY,
                            semantic::BORDER_SUBTLE,
                            radius::SM,
                            typography::FONT_MONO,
                            typography::SIZE_XS,
                            semantic::TEXT_MUTED
                        ),
                        title: "Press ⌘K or Ctrl+K to search",
                        "⌘K"
                    }
                }
            }
        }
    }
}

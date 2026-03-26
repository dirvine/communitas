//! Entity detail view with tabs for different surfaces.
//!
//! Tabs: Board (Kanban), Chat, Drive, Documents, Details

use crate::design_tokens::{motion, radius, semantic, spacing, typography};
use crate::styles_v2::flex;
use communitas_ui_api::{UnifiedEntity, UnifiedEntityType};
use dioxus::prelude::*;

/// Tab identifiers for entity views.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityTab {
    Board,
    Chat,
    Canvas,
    Drive,
    Documents,
    Details,
}

impl EntityTab {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Chat => "Chat",
            Self::Canvas => "Canvas",
            Self::Drive => "Drive",
            Self::Documents => "Docs",
            Self::Details => "Details",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Board => "☰",
            Self::Chat => "💬",
            Self::Canvas => "🎨",
            Self::Drive => "📁",
            Self::Documents => "📄",
            Self::Details => "⚙",
        }
    }

    /// Available tabs for entity type.
    ///
    /// Tab availability per entity type:
    /// - Organizations: Chat, Drive, Details
    /// - Channels: Chat, Drive
    /// - Projects: Board, Chat, Drive, Docs
    /// - Groups: Chat, Drive
    /// - Contacts: Chat only
    pub fn tabs_for_entity(entity_type: UnifiedEntityType) -> Vec<Self> {
        match entity_type {
            UnifiedEntityType::Project => vec![
                Self::Board,
                Self::Chat,
                Self::Canvas,
                Self::Drive,
                Self::Documents,
                Self::Details,
            ],
            UnifiedEntityType::Channel => {
                vec![Self::Chat, Self::Drive]
            }
            UnifiedEntityType::Organization => {
                vec![Self::Chat, Self::Drive, Self::Details]
            }
            UnifiedEntityType::Group => {
                vec![Self::Chat, Self::Drive]
            }
            UnifiedEntityType::Person => vec![Self::Chat],
        }
    }

    /// Get all tabs in order (for keyboard navigation).
    pub fn all_tabs() -> &'static [Self] {
        &[
            Self::Board,
            Self::Chat,
            Self::Canvas,
            Self::Drive,
            Self::Documents,
            Self::Details,
        ]
    }
}

/// Entity detail header with name, icon, and actions.
#[component]
pub fn EntityHeader(
    entity: UnifiedEntity,
    #[props(default)] actions: Option<Element>,
    /// Number of members currently online
    #[props(default = 0)]
    online_count: u32,
    /// Optional description for the entity
    #[props(default)]
    description: Option<String>,
    /// Parent entity name for breadcrumb (e.g., "Organization Name")
    #[props(default)]
    parent_name: Option<String>,
) -> Element {
    let mut description_expanded = use_signal(|| false);

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

    let type_label = match entity.entity_type {
        UnifiedEntityType::Organization => "Organization",
        UnifiedEntityType::Project => "Project",
        UnifiedEntityType::Channel => "Channel",
        UnifiedEntityType::Group => "Group",
        UnifiedEntityType::Person => "Contact",
    };

    // Presence summary: "X online" or "X/Y online"
    let presence_text = if entity.member_count > 0 {
        if online_count > 0 {
            format!("{online_count} online")
        } else {
            "all offline".to_string()
        }
    } else {
        String::new()
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 border-bottom: 1px solid {};",
                semantic::BORDER_SUBTLE
            ),

            // Main header row
            div {
                style: format!(
                    "{} \
                     padding: {} {};",
                    flex::between(),
                    spacing::BASE,
                    spacing::XL
                ),

                // Left: Entity info
                div {
                    style: format!("{} gap: {};", flex::start(), spacing::MD),

                    // Entity icon badge
                    div {
                        style: format!(
                            "width: 40px; \
                             height: 40px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: {}20; \
                             border-radius: {}; \
                             font-size: {}; \
                             color: {};",
                            entity_color,
                            radius::LG,
                            typography::SIZE_LG,
                            entity_color
                        ),
                        "{icon}"
                    }

                    // Name, breadcrumb, and metadata
                    div {
                        style: flex::col(),

                        // Breadcrumb (if parent exists)
                        if let Some(ref parent) = parent_name {
                            div {
                                style: format!(
                                    "display: flex; \
                                     align-items: center; \
                                     gap: {}; \
                                     font-size: {}; \
                                     color: {}; \
                                     margin-bottom: {};",
                                    spacing::XS,
                                    typography::SIZE_XS,
                                    semantic::TEXT_MUTED,
                                    spacing::XXS
                                ),

                                span { "{parent}" }
                                span {
                                    style: format!("color: {};", semantic::TEXT_MUTED),
                                    "›"
                                }
                            }
                        }

                        h1 {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {}; \
                                 margin: 0; \
                                 line-height: {};",
                                typography::SIZE_LG,
                                typography::WEIGHT_SEMIBOLD,
                                semantic::TEXT_PRIMARY,
                                typography::LEADING_TIGHT
                            ),
                            "{entity.name}"
                        }

                        div {
                            style: format!("{} gap: {};", flex::start(), spacing::SM),

                            // Type badge
                            span {
                                style: format!(
                                    "font-size: {}; \
                                     color: {}; \
                                     padding: {} {}; \
                                     background: {}15; \
                                     border-radius: {};",
                                    typography::SIZE_XS,
                                    entity_color,
                                    spacing::XXS,
                                    spacing::SM,
                                    entity_color,
                                    radius::SM
                                ),
                                "{type_label}"
                            }

                            // Member count
                            span {
                                style: format!(
                                    "font-size: {}; \
                                     color: {};",
                                    typography::SIZE_XS,
                                    semantic::TEXT_MUTED
                                ),
                                "{entity.member_count} members"
                            }

                            // Presence summary (online count)
                            if !presence_text.is_empty() {
                                span {
                                    style: format!(
                                        "font-size: {}; \
                                         color: {}; \
                                         display: flex; \
                                         align-items: center; \
                                         gap: {};",
                                        typography::SIZE_XS,
                                        semantic::PRESENCE_ONLINE,
                                        spacing::XXS
                                    ),

                                    // Online dot
                                    span {
                                        style: format!(
                                            "width: 6px; \
                                             height: 6px; \
                                             border-radius: {}; \
                                             background: {};",
                                            radius::FULL,
                                            semantic::PRESENCE_ONLINE
                                        ),
                                    }
                                    "{presence_text}"
                                }
                            }
                        }
                    }
                }

                // Right: Actions
                if let Some(acts) = actions {
                    div {
                        style: format!("{} gap: {};", flex::row(), spacing::SM),
                        {acts}
                    }
                }
            }

            // Collapsible description
            if let Some(ref desc) = description {
                if !desc.is_empty() {
                    div {
                        style: format!(
                            "padding: 0 {} {}; \
                             border-top: 1px solid {};",
                            spacing::XL,
                            spacing::BASE,
                            semantic::BORDER_SUBTLE
                        ),

                        button {
                            style: format!(
                                "display: flex; \
                                 align-items: center; \
                                 gap: {}; \
                                 width: 100%; \
                                 padding: {} 0; \
                                 background: none; \
                                 border: none; \
                                 cursor: pointer; \
                                 font-family: {}; \
                                 font-size: {}; \
                                 color: {}; \
                                 text-align: left;",
                                spacing::SM,
                                spacing::SM,
                                typography::FONT_BODY,
                                typography::SIZE_SM,
                                semantic::TEXT_SECONDARY
                            ),
                            aria_expanded: if description_expanded() { "true" } else { "false" },
                            aria_label: format!(
                                "{} description for {}",
                                if description_expanded() { "Collapse" } else { "Expand" },
                                type_label
                            ),
                            onclick: move |_| description_expanded.set(!description_expanded()),

                            span {
                                style: format!(
                                    "transform: rotate({}); \
                                     transition: {};",
                                    if description_expanded() { "90deg" } else { "0deg" },
                                    motion::transition("transform")
                                ),
                                "▶"
                            }
                            span { "About this {type_label}" }
                        }

                        if description_expanded() {
                            p {
                                style: format!(
                                    "margin: 0 0 {} 0; \
                                     padding-left: {}; \
                                     font-size: {}; \
                                     color: {}; \
                                     line-height: {}; \
                                     animation: expandDesc 150ms ease-out;",
                                    spacing::SM,
                                    spacing::XL,
                                    typography::SIZE_SM,
                                    semantic::TEXT_SECONDARY,
                                    typography::LEADING_RELAXED
                                ),
                                "{desc}"
                            }
                        }
                    }

                    style {
                        r#"
                        @keyframes expandDesc {{
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
        }
    }
}

/// Tab bar for entity views.
#[component]
pub fn EntityTabBar(
    tabs: Vec<EntityTab>,
    active_tab: EntityTab,
    on_tab_change: EventHandler<EntityTab>,
) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 gap: {}; \
                 padding: 0 {}; \
                 border-bottom: 1px solid {}; \
                 background: {};",
                spacing::XS,
                spacing::XL,
                semantic::BORDER_SUBTLE,
                semantic::BG_SECONDARY
            ),
            role: "tablist",
            aria_label: "Entity view tabs",

            for tab in tabs {
                TabButton {
                    tab: tab,
                    is_active: tab == active_tab,
                    onclick: move |_| on_tab_change.call(tab),
                }
            }
        }
    }
}

/// Individual tab button.
#[component]
fn TabButton(tab: EntityTab, is_active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);

    let color = if is_active {
        semantic::PRIMARY
    } else if hovered() {
        semantic::TEXT_PRIMARY
    } else {
        semantic::TEXT_SECONDARY
    };

    rsx! {
        button {
            style: format!(
                "position: relative; \
                 display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 background: transparent; \
                 border: none; \
                 color: {}; \
                 font-family: {}; \
                 font-size: {}; \
                 font-weight: {}; \
                 cursor: pointer; \
                 transition: {};",
                spacing::XS,
                spacing::MD,
                spacing::BASE,
                color,
                typography::FONT_BODY,
                typography::SIZE_SM,
                if is_active { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_MEDIUM },
                motion::transition("color")
            ),
            role: "tab",
            aria_selected: if is_active { "true" } else { "false" },
            aria_label: format!("{} tab", tab.label()),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            span { "{tab.icon()}" }
            span { "{tab.label()}" }

            // Active indicator
            if is_active {
                div {
                    style: format!(
                        "position: absolute; \
                         bottom: 0; \
                         left: 0; \
                         right: 0; \
                         height: 2px; \
                         background: {}; \
                         border-radius: {} {} 0 0;",
                        semantic::PRIMARY,
                        radius::FULL,
                        radius::FULL
                    ),
                }
            }
        }
    }
}

/// Entity detail container with header and tabs.
#[component]
pub fn EntityDetailView(
    entity: UnifiedEntity,
    #[props(default = EntityTab::Chat)] initial_tab: EntityTab,
    #[props(default)] header_actions: Option<Element>,
    children: Element,
) -> Element {
    let available_tabs = EntityTab::tabs_for_entity(entity.entity_type);
    let available_tabs_for_signal = available_tabs.clone();
    let mut active_tab = use_signal(move || {
        if available_tabs_for_signal.contains(&initial_tab) {
            initial_tab
        } else {
            available_tabs_for_signal
                .first()
                .copied()
                .unwrap_or(EntityTab::Chat)
        }
    });

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
            EntityHeader {
                entity: entity.clone(),
                actions: header_actions,
            }

            // Tab bar
            EntityTabBar {
                tabs: available_tabs,
                active_tab: active_tab(),
                on_tab_change: move |tab| active_tab.set(tab),
            }

            // Tab content
            div {
                style: format!(
                    "flex: 1; \
                     overflow: hidden; \
                     display: flex; \
                     flex-direction: column;",
                ),
                {children}
            }
        }
    }
}

/// Action button for entity header.
#[component]
pub fn HeaderAction(
    icon: String,
    label: String,
    #[props(default = false)] primary: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let bg = if primary {
        if hovered() {
            "linear-gradient(135deg, #34d399 0%, #10b981 100%)"
        } else {
            "linear-gradient(135deg, #10b981 0%, #059669 100%)"
        }
    } else if hovered() {
        semantic::BG_HOVER
    } else {
        "transparent"
    };

    let color = if primary {
        "white"
    } else if hovered() {
        semantic::TEXT_PRIMARY
    } else {
        semantic::TEXT_SECONDARY
    };

    rsx! {
        button {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 background: {}; \
                 border: {}; \
                 border-radius: {}; \
                 color: {}; \
                 font-family: {}; \
                 font-size: {}; \
                 font-weight: {}; \
                 cursor: pointer; \
                 transition: {};",
                spacing::XS,
                spacing::SM,
                spacing::MD,
                bg,
                if primary { "none".to_string() } else { format!("1px solid {}", semantic::BORDER_DEFAULT) },
                radius::MD,
                color,
                typography::FONT_BODY,
                typography::SIZE_SM,
                typography::WEIGHT_MEDIUM,
                motion::transition("all")
            ),
            aria_label: "{label}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            span { aria_hidden: "true", "{icon}" }
            span { "{label}" }
        }
    }
}

/// Empty state placeholder.
#[component]
pub fn EmptyState(
    icon: String,
    title: String,
    description: String,
    #[props(default)] action: Option<Element>,
) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 align-items: center; \
                 justify-content: center; \
                 padding: {}; \
                 text-align: center; \
                 flex: 1;",
                spacing::HUGE
            ),

            // Icon
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
                "{icon}"
            }

            // Title
            h3 {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     margin: 0 0 {} 0;",
                    typography::SIZE_LG,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY,
                    spacing::SM
                ),
                "{title}"
            }

            // Description
            p {
                style: format!(
                    "font-size: {}; \
                     color: {}; \
                     max-width: 320px; \
                     margin: 0 0 {} 0;",
                    typography::SIZE_BASE,
                    semantic::TEXT_SECONDARY,
                    spacing::XL
                ),
                "{description}"
            }

            // Action
            if let Some(act) = action {
                {act}
            }
        }
    }
}

/// Loading skeleton for entity content.
#[component]
pub fn EntitySkeleton() -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 padding: {};",
                spacing::MD,
                spacing::XL
            ),

            // Header skeleton
            div {
                style: format!("{} gap: {};", flex::start(), spacing::MD),

                div {
                    style: format!(
                        "width: 40px; \
                         height: 40px; \
                         background: {}; \
                         border-radius: {}; \
                         animation: pulse 1.5s ease-in-out infinite;",
                        semantic::BG_ELEVATED,
                        radius::LG
                    ),
                }

                div {
                    style: format!("{} gap: {};", flex::col(), spacing::SM),

                    div {
                        style: format!(
                            "width: 180px; \
                             height: 20px; \
                             background: {}; \
                             border-radius: {}; \
                             animation: pulse 1.5s ease-in-out infinite;",
                            semantic::BG_ELEVATED,
                            radius::MD
                        ),
                    }

                    div {
                        style: format!(
                            "width: 120px; \
                             height: 14px; \
                             background: {}; \
                             border-radius: {}; \
                             animation: pulse 1.5s ease-in-out infinite;",
                            semantic::BG_ELEVATED,
                            radius::SM
                        ),
                    }
                }
            }

            // Content skeletons
            for i in 0..5 {
                div {
                    style: format!(
                        "width: {}%; \
                         height: 48px; \
                         background: {}; \
                         border-radius: {}; \
                         animation: pulse 1.5s ease-in-out infinite; \
                         animation-delay: {}ms;",
                        90 - (i * 10),
                        semantic::BG_ELEVATED,
                        radius::MD,
                        i * 100
                    ),
                }
            }
        }

        style {
            r#"
            @keyframes pulse {{
                0%, 100% {{ opacity: 1; }}
                50% {{ opacity: 0.5; }}
            }}
            "#
        }
    }
}

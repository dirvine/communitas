//! Page components that integrate the new Digital Forest Sanctuary design system.
//!
//! These pages use the v2 components (auth_v2, app_shell, entity_view, messaging_v2)
//! to create a cohesive, stunning user experience.

use dioxus::prelude::*;
use communitas_ui_api::{PresenceStatus, UnifiedEntity, UnifiedEntityType, OrganizationCategory};
use communitas_ui_service::UiServices;
use std::sync::Arc;

use crate::components::{
    // New v2 components
    AppShell as AppShellV2, AuthLayoutV2, ContactNavItem, EmptyState, EntityDetailView,
    EntityNavItem, EntityTab, HeaderAction, PrimaryButton, ProfileHeader, QuickActionButton,
    SecondaryButton, SidebarSearch, SidebarSection,
    // Messaging v2
    ChatView, DateSeparator, MessageBubble, MessageComposerV2, MessageDisplay,
    MessageListContainer, ReactionDisplay, TypingIndicatorV2,
};
use crate::design_tokens::{motion, palette, radius, semantic, shadow, spacing, typography};

/// Main authenticated application with new design.
/// Uses `use_context` to access UiServices instead of props.
#[component]
pub fn MainAppV2(
    children: Element,
) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let _nav_snapshot = services.navigation().current_snapshot();
    let dir_snapshot = services.directory().current_snapshot();

    let identity = dir_snapshot.identity.clone();
    let display_name = identity
        .as_ref()
        .map(|i| i.display_name.clone())
        .unwrap_or_else(|| "User".to_string());
    let four_words_fingerprint = identity
        .as_ref()
        .map(|i| i.four_words.clone());

    let mut search_query = use_signal(String::new);
    let thread_panel_open = use_signal(|| false);

    // Categorize entities
    let (organizations, communities, projects, groups, _channels) = {
        let mut orgs = Vec::new();
        let mut comms = Vec::new();
        let mut projs = Vec::new();
        let mut grps = Vec::new();
        let mut chans = Vec::new();

        for entity in &dir_snapshot.entities {
            match entity.entity_type {
                UnifiedEntityType::Organization => {
                    if entity.category == Some(OrganizationCategory::Community) {
                        comms.push(entity.clone());
                    } else {
                        orgs.push(entity.clone());
                    }
                }
                UnifiedEntityType::Project => projs.push(entity.clone()),
                UnifiedEntityType::Group => grps.push(entity.clone()),
                UnifiedEntityType::Channel => chans.push(entity.clone()),
                _ => {}
            }
        }
        (orgs, comms, projs, grps, chans)
    };

    let contacts = dir_snapshot.contacts.clone();

    // Filter based on search
    let filter_entities = |entities: Vec<UnifiedEntity>| -> Vec<UnifiedEntity> {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            entities
        } else {
            entities.into_iter().filter(|e| e.name.to_lowercase().contains(&query)).collect()
        }
    };

    let filter_contacts = |contacts: Vec<communitas_ui_api::UnifiedContact>| -> Vec<communitas_ui_api::UnifiedContact> {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            contacts
        } else {
            contacts.into_iter().filter(|c| c.display_name.to_lowercase().contains(&query)).collect()
        }
    };

    let thread_panel_content = if thread_panel_open() {
        Some(rsx! {
            ThreadPanelContent {}
        })
    } else {
        None
    };

    rsx! {
        AppShellV2 {
            thread_panel_open: thread_panel_open(),
            thread_panel: thread_panel_content,
            sidebar: rsx! {
                // Profile Header
                ProfileHeader {
                    display_name: display_name.clone(),
                    pubkey_fingerprint: four_words_fingerprint,
                    presence: PresenceStatus::Online,
                    is_networking: true,
                }

                // Search
                SidebarSearch {
                    value: search_query(),
                    placeholder: "Search spaces...".to_string(),
                    oninput: move |evt: FormEvent| search_query.set(evt.value()),
                }

                // Navigation sections (scrollable)
                div {
                    style: format!(
                        "flex: 1; \
                         overflow-y: auto; \
                         padding: 0 {}; \
                         scrollbar-width: thin; \
                         scrollbar-color: {} transparent;",
                        spacing::SM,
                        semantic::BORDER_DEFAULT
                    ),

                    // My Organizations
                    SidebarSection {
                        title: "My Organizations".to_string(),
                        action: Some(rsx! {
                            QuickActionButton {
                                icon: "+".to_string(),
                                onclick: move |_| {},
                            }
                        }),
                        for entity in filter_entities(organizations.clone()) {
                            EntityNavItem {
                                entity: entity.clone(),
                                selected: false,
                                unread_count: 0,
                                onclick: move |_| {},
                            }
                        }
                    }

                    // Communities
                    if !communities.is_empty() {
                        SidebarSection {
                            title: "Communities".to_string(),
                            for entity in filter_entities(communities.clone()) {
                                EntityNavItem {
                                    entity: entity.clone(),
                                    selected: false,
                                    unread_count: 0,
                                    onclick: move |_| {},
                                }
                            }
                        }
                    }

                    // Projects
                    if !projects.is_empty() {
                        SidebarSection {
                            title: "Projects".to_string(),
                            action: Some(rsx! {
                                QuickActionButton {
                                    icon: "+".to_string(),
                                    onclick: move |_| {},
                                }
                            }),
                            for entity in filter_entities(projects.clone()) {
                                EntityNavItem {
                                    entity: entity.clone(),
                                    selected: false,
                                    unread_count: 0,
                                    onclick: move |_| {},
                                }
                            }
                        }
                    }

                    // Groups
                    if !groups.is_empty() {
                        SidebarSection {
                            title: "Groups".to_string(),
                            for entity in filter_entities(groups.clone()) {
                                EntityNavItem {
                                    entity: entity.clone(),
                                    selected: false,
                                    unread_count: 0,
                                    onclick: move |_| {},
                                }
                            }
                        }
                    }

                    // Direct Messages
                    SidebarSection {
                        title: "Direct Messages".to_string(),
                        action: Some(rsx! {
                            QuickActionButton {
                                icon: "+".to_string(),
                                onclick: move |_| {},
                            }
                        }),
                        for contact in filter_contacts(contacts.clone()) {
                            ContactNavItem {
                                contact: contact.clone(),
                                selected: false,
                                unread_count: 0,
                                onclick: move |_| {},
                            }
                        }
                    }
                }
            },

            // Main content
            {children}
        }
    }
}

/// Thread panel content.
#[component]
fn ThreadPanelContent() -> Element {
    rsx! {
        div {
            style: format!("padding: {};", spacing::BASE),

            // Header
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     justify-content: space-between; \
                     padding-bottom: {}; \
                     border-bottom: 1px solid {};",
                    spacing::BASE,
                    semantic::BORDER_SUBTLE
                ),

                h3 {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0;",
                        typography::SIZE_BASE,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY
                    ),
                    "Thread"
                }
            }

            // Thread content would go here
            EmptyState {
                icon: "💬".to_string(),
                title: "No thread selected".to_string(),
                description: "Click on a message to view its thread".to_string(),
            }
        }
    }
}

/// Enhanced Welcome/Landing page.
#[component]
pub fn WelcomePageV2() -> Element {
    rsx! {
        AuthLayoutV2 {
            title: "Welcome to Communitas".to_string(),
            subtitle: Some("Your local-first, privacy-preserving collaboration platform".to_string()),
            footer: Some(rsx! {
                p {
                    style: format!(
                        "font-size: {}; \
                         color: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED
                    ),
                    "Secure • Decentralized • Private"
                }
            }),

            div {
                style: format!("display: flex; flex-direction: column; gap: {};", spacing::MD),

                PrimaryButton {
                    onclick: Some(EventHandler::new(|_| {})),
                    "Create Identity"
                }

                SecondaryButton {
                    onclick: Some(EventHandler::new(|_| {})),
                    "Sign In"
                }

                div {
                    style: format!(
                        "text-align: center; \
                         padding-top: {}; \
                         border-top: 1px solid {};",
                        spacing::MD,
                        semantic::BORDER_SUBTLE
                    ),

                    button {
                        style: format!(
                            "background: none; \
                             border: none; \
                             color: {}; \
                             font-size: {}; \
                             cursor: pointer;",
                            semantic::TEXT_SECONDARY,
                            typography::SIZE_SM
                        ),
                        "Recover existing identity →"
                    }
                }
            }
        }
    }
}

/// Home/Dashboard page with new design.
#[component]
pub fn HomePageV2() -> Element {
    let services = use_context::<Arc<UiServices>>();
    let dir_snapshot = services.directory().current_snapshot();

    let identity = dir_snapshot.identity.clone();
    let display_name = identity
        .as_ref()
        .map(|i| i.display_name.clone())
        .unwrap_or_else(|| "Explorer".to_string());

    // Count entities
    let org_count = dir_snapshot.entities.iter().filter(|e| e.entity_type == UnifiedEntityType::Organization).count();
    let project_count = dir_snapshot.entities.iter().filter(|e| e.entity_type == UnifiedEntityType::Project).count();
    let contact_count = dir_snapshot.contacts.len();

    rsx! {
        div {
            style: format!(
                "padding: {}; \
                 max-width: 1200px; \
                 margin: 0 auto;",
                spacing::XXL
            ),

            // Welcome banner with gradient
            div {
                style: format!(
                    "position: relative; \
                     padding: {}; \
                     border-radius: {}; \
                     background: linear-gradient(135deg, {}20 0%, {}10 100%); \
                     border: 1px solid {}; \
                     margin-bottom: {}; \
                     overflow: hidden;",
                    spacing::XXL,
                    radius::XXL,
                    palette::JADE_500,
                    palette::JADE_700,
                    semantic::BORDER_DEFAULT,
                    spacing::XXL
                ),

                // Decorative glow
                div {
                    style: format!(
                        "position: absolute; \
                         top: -50%; \
                         right: -20%; \
                         width: 400px; \
                         height: 400px; \
                         background: radial-gradient(circle, {}15 0%, transparent 70%); \
                         pointer-events: none;",
                        palette::JADE_400
                    ),
                }

                // Content
                div {
                    style: "position: relative; z-index: 1;",

                    div {
                        style: format!(
                            "font-size: {}; \
                             font-weight: {}; \
                             color: {}; \
                             text-transform: uppercase; \
                             letter-spacing: {}; \
                             margin-bottom: {};",
                            typography::SIZE_XS,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::PRIMARY,
                            typography::TRACKING_WIDER,
                            spacing::SM
                        ),
                        "Communitas"
                    }

                    h1 {
                        style: format!(
                            "font-family: {}; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {}; \
                             margin: 0 0 {} 0; \
                             line-height: {};",
                            typography::FONT_DISPLAY,
                            typography::SIZE_3XL,
                            typography::WEIGHT_BOLD,
                            semantic::TEXT_PRIMARY,
                            spacing::SM,
                            typography::LEADING_TIGHT
                        ),
                        "Welcome back, {display_name}"
                    }

                    p {
                        style: format!(
                            "font-size: {}; \
                             color: {}; \
                             margin: 0;",
                            typography::SIZE_BASE,
                            semantic::TEXT_SECONDARY
                        ),
                        "Your local-first collaboration hub"
                    }
                }
            }

            // Quick stats
            div {
                style: format!(
                    "display: grid; \
                     grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); \
                     gap: {}; \
                     margin-bottom: {};",
                    spacing::XL,
                    spacing::XXL
                ),

                StatCard {
                    icon: "◆".to_string(),
                    label: "Organizations".to_string(),
                    value: org_count,
                    color: semantic::ENTITY_ORG.to_string(),
                }

                StatCard {
                    icon: "◈".to_string(),
                    label: "Projects".to_string(),
                    value: project_count,
                    color: semantic::ENTITY_PROJECT.to_string(),
                }

                StatCard {
                    icon: "◉".to_string(),
                    label: "Contacts".to_string(),
                    value: contact_count,
                    color: semantic::ENTITY_PERSON.to_string(),
                }
            }

            // Quick actions
            div {
                style: format!(
                    "display: grid; \
                     grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); \
                     gap: {};",
                    spacing::XL
                ),

                QuickActionCard {
                    icon: "🏢".to_string(),
                    title: "Create Organization".to_string(),
                    description: "Start a new team or company space".to_string(),
                    onclick: move |_| {},
                }

                QuickActionCard {
                    icon: "📋".to_string(),
                    title: "New Project".to_string(),
                    description: "Create a project with Kanban board".to_string(),
                    onclick: move |_| {},
                }

                QuickActionCard {
                    icon: "👥".to_string(),
                    title: "Add Contact".to_string(),
                    description: "Connect with someone new".to_string(),
                    onclick: move |_| {},
                }
            }
        }
    }
}

/// Stat card for dashboard.
#[component]
fn StatCard(
    icon: String,
    label: String,
    value: usize,
    color: String,
) -> Element {
    rsx! {
        div {
            style: format!(
                "padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {};",
                spacing::XL,
                semantic::BG_TERTIARY,
                semantic::BORDER_SUBTLE,
                radius::XL
            ),

            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {}; \
                     margin-bottom: {};",
                    spacing::SM,
                    spacing::MD
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         color: {};",
                        typography::SIZE_LG,
                        color
                    ),
                    "{icon}"
                }

                span {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         text-transform: uppercase; \
                         letter-spacing: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED,
                        typography::TRACKING_WIDE
                    ),
                    "{label}"
                }
            }

            span {
                style: format!(
                    "font-family: {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {};",
                    typography::FONT_DISPLAY,
                    typography::SIZE_3XL,
                    typography::WEIGHT_BOLD,
                    semantic::TEXT_PRIMARY
                ),
                "{value}"
            }
        }
    }
}

/// Quick action card for dashboard.
#[component]
fn QuickActionCard(
    icon: String,
    title: String,
    description: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        button {
            style: format!(
                "display: flex; \
                 align-items: flex-start; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: pointer; \
                 text-align: left; \
                 transition: {}; \
                 {}",
                spacing::BASE,
                spacing::XL,
                if hovered() { semantic::BG_HOVER } else { semantic::BG_TERTIARY },
                if hovered() { semantic::BORDER_STRONG } else { semantic::BORDER_SUBTLE },
                radius::XL,
                motion::transition("all"),
                if hovered() { format!("transform: translateY(-2px); box-shadow: {};", shadow::LG) } else { String::new() }
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            // Icon
            div {
                style: format!(
                    "width: 48px; \
                     height: 48px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     background: {}20; \
                     border-radius: {}; \
                     font-size: {}; \
                     flex-shrink: 0;",
                    semantic::PRIMARY,
                    radius::LG,
                    typography::SIZE_2XL
                ),
                "{icon}"
            }

            // Text
            div {
                h3 {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0 0 {} 0;",
                        typography::SIZE_BASE,
                        typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY,
                        spacing::XS
                    ),
                    "{title}"
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         margin: 0;",
                        typography::SIZE_SM,
                        semantic::TEXT_SECONDARY
                    ),
                    "{description}"
                }
            }
        }
    }
}

/// Entity chat page with new messaging UI.
#[component]
pub fn EntityChatPageV2(
    entity: UnifiedEntity,
) -> Element {
    let mut message_input = use_signal(String::new);

    // Demo messages
    let messages = [
        MessageDisplay {
            id: "1".to_string(),
            author_name: "Alice Chen".to_string(),
            author_id: "alice".to_string(),
            content: "Hey team! Just pushed the new design system updates.".to_string(),
            timestamp: "10:30 AM".to_string(),
            is_own: false,
            is_edited: false,
            reply_count: 2,
            reactions: vec![
                ReactionDisplay { emoji: "👍".to_string(), count: 3, has_reacted: true },
                ReactionDisplay { emoji: "🎉".to_string(), count: 1, has_reacted: false },
            ],
        },
        MessageDisplay {
            id: "2".to_string(),
            author_name: "You".to_string(),
            author_id: "me".to_string(),
            content: "Looks great! The new color palette is much better.".to_string(),
            timestamp: "10:32 AM".to_string(),
            is_own: true,
            is_edited: false,
            reply_count: 0,
            reactions: vec![],
        },
        MessageDisplay {
            id: "3".to_string(),
            author_name: "Bob Smith".to_string(),
            author_id: "bob".to_string(),
            content: "I love the glass morphism effects! Really gives it that modern feel.".to_string(),
            timestamp: "10:35 AM".to_string(),
            is_own: false,
            is_edited: true,
            reply_count: 0,
            reactions: vec![
                ReactionDisplay { emoji: "❤️".to_string(), count: 2, has_reacted: false },
            ],
        },
    ];

    rsx! {
        EntityDetailView {
            entity: entity.clone(),
            initial_tab: EntityTab::Chat,
            header_actions: Some(rsx! {
                HeaderAction {
                    icon: "📞".to_string(),
                    label: "Call".to_string(),
                    onclick: move |_| {},
                }
                HeaderAction {
                    icon: "📹".to_string(),
                    label: "Video".to_string(),
                    primary: true,
                    onclick: move |_| {},
                }
            }),

            ChatView {
                MessageListContainer {
                    DateSeparator { date: "Today".to_string() }

                    for (idx, msg) in messages.iter().enumerate() {
                        MessageBubble {
                            message: msg.clone(),
                            show_avatar: idx == 0 || messages.get(idx.saturating_sub(1)).map(|prev| prev.author_id != msg.author_id).unwrap_or(true),
                            on_reply: move |_id| {},
                            on_react: move |_id| {},
                        }
                    }
                }

                TypingIndicatorV2 {
                    names: vec!["Alice".to_string()],
                }

                MessageComposerV2 {
                    value: message_input(),
                    placeholder: format!("Message #{}", entity.name),
                    oninput: move |evt: FormEvent| message_input.set(evt.value()),
                    onsubmit: move |_| {
                        if !message_input().trim().is_empty() {
                            // Send message
                            message_input.set(String::new());
                        }
                    },
                }
            }
        }
    }
}

//! Page components that integrate the new Digital Forest Sanctuary design system.
//!
//! These pages use the v2 components (auth_v2, app_shell, entity_view, messaging_v2)
//! to create a cohesive, stunning user experience.

use communitas_ui_api::{PresenceStatus, UnifiedContact, UnifiedEntity, UnifiedEntityType};
use communitas_ui_service::UiServices;
use dioxus::document::eval;
use dioxus::prelude::*;
use std::sync::Arc;

use crate::Route;
use crate::components::sidebar::{ContactListSection, EntityListSection};
use crate::hooks::CategorizedEntities;

// Route helper functions
fn entity_type_segment(entity_type: UnifiedEntityType) -> &'static str {
    match entity_type {
        UnifiedEntityType::Organization => "organisation",
        UnifiedEntityType::Project => "project",
        UnifiedEntityType::Group => "group",
        UnifiedEntityType::Channel => "channel",
        UnifiedEntityType::Person => "person",
    }
}

fn entity_route(entity: &UnifiedEntity) -> Route {
    Route::EntityDetailRoute {
        entity_type: entity_type_segment(entity.entity_type).to_string(),
        entity_id: entity.id.clone(),
    }
}

fn contact_route(contact: &UnifiedContact) -> Route {
    Route::ContactDetailRoute {
        contact_id: contact.id.clone(),
    }
}

use crate::components::{
    // Accessibility announcer
    AnnouncementMode,
    // New v2 components
    AppShell as AppShellV2,
    AuthLayoutV2,
    // Messaging v2
    ChatView,
    DateSeparator,
    EmptyState,
    EntityDetailView,
    EntityTab,
    HeaderAction,
    MessageBubble,
    MessageComposerV2,
    MessageDisplay,
    MessageListContainer,
    PrimaryButton,
    ProfileHeader,
    ReactionDisplay,
    SecondaryButton,
    SidebarSearch,
    TypingIndicatorV2,
    use_announcer,
};
use crate::design_tokens::{motion, palette, radius, semantic, shadow, spacing, typography};
use std::collections::HashSet;

/// Unified entity page that handles all tabs (Chat, Board, Drive, Docs, Details).
///
/// This is the main entry point for viewing an entity. It manages tab state
/// and renders the appropriate content based on the selected tab.
#[component]
pub fn EntityPageV2(
    entity_id: String,
    #[props(default)] initial_tab: Option<EntityTab>,
) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let dir_snapshot = services.directory().current_snapshot();

    // Find the entity
    let entity = dir_snapshot
        .entities
        .iter()
        .find(|e| e.id == entity_id)
        .cloned();

    let Some(entity) = entity else {
        return rsx! {
            div {
                style: format!(
                    "flex: 1; \
                     display: flex; \
                     flex-direction: column; \
                     align-items: center; \
                     justify-content: center; \
                     padding: {};",
                    spacing::HUGE
                ),

                EmptyState {
                    icon: "🔍".to_string(),
                    title: "Entity Not Found".to_string(),
                    description: format!("Could not find entity with ID: {entity_id}"),
                }
            }
        };
    };

    // Determine available tabs for this entity type
    let available_tabs = EntityTab::tabs_for_entity(entity.entity_type);

    // Track active tab state
    let default_tab = initial_tab
        .filter(|t| available_tabs.contains(t))
        .or_else(|| available_tabs.first().copied())
        .unwrap_or(EntityTab::Chat);

    let mut active_tab = use_signal(move || default_tab);

    // Get parent name for breadcrumb
    let parent_name = entity.parent_id.as_ref().and_then(|pid| {
        dir_snapshot
            .entities
            .iter()
            .find(|e| &e.id == pid)
            .map(|e| e.name.clone())
    });

    // Calculate online count (placeholder - would come from presence service)
    let online_count = 0u32;

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 height: 100%; \
                 background: {};",
                semantic::BG_PRIMARY
            ),

            // Entity header with breadcrumb and actions
            crate::components::EntityHeader {
                entity: entity.clone(),
                online_count: online_count,
                parent_name: parent_name,
                actions: Some(rsx! {
                    HeaderAction {
                        icon: "📞".to_string(),
                        label: "Call".to_string(),
                        primary: true,
                        onclick: move |_| {
                            tracing::info!(target: "ui.entity", event = "call_clicked", entity_id = %entity_id);
                        },
                    }
                    HeaderAction {
                        icon: "⚙️".to_string(),
                        label: "Settings".to_string(),
                        primary: false,
                        onclick: move |_| {
                            active_tab.set(EntityTab::Details);
                        },
                    }
                }),
            }

            // Tab bar
            crate::components::EntityTabBar {
                tabs: available_tabs.clone(),
                active_tab: active_tab(),
                on_tab_change: move |tab| active_tab.set(tab),
            }

            // Tab content
            div {
                style: "flex: 1; overflow: hidden; display: flex; flex-direction: column;",

                match active_tab() {
                    EntityTab::Chat => rsx! { EntityChatContent { entity: entity.clone() } },
                    EntityTab::Board => rsx! { EntityBoardContent { entity: entity.clone() } },
                    EntityTab::Drive => rsx! { EntityDriveContent { entity: entity.clone() } },
                    EntityTab::Documents => rsx! { EntityDocsContent { entity: entity.clone() } },
                    EntityTab::Details => rsx! { EntityDetailsContent { entity: entity.clone() } },
                }
            }
        }
    }
}

/// Chat tab content (extracted from EntityChatPageV2).
#[component]
fn EntityChatContent(entity: UnifiedEntity) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let dir_snapshot = services.directory().current_snapshot();
    let current_user_id = dir_snapshot
        .identity
        .as_ref()
        .map(|i| i.four_words.clone())
        .unwrap_or_default();

    let mut message_input = use_signal(String::new);
    let mut messages: Signal<Vec<communitas_ui_api::Message>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut reply_to: Signal<Option<MessageDisplay>> = use_signal(|| None);
    let mut show_emoji_picker: Signal<Option<String>> = use_signal(|| None); // message_id to show picker for

    let thread_id = entity.id.clone();
    let thread_id_for_load = thread_id.clone();
    let thread_id_for_send = thread_id.clone();

    // Load messages on mount
    let services_for_load = services.clone();
    use_effect(move || {
        let thread_id = thread_id_for_load.clone();
        let services = services_for_load.clone();
        spawn(async move {
            loading.set(true);
            match services
                .messaging()
                .get_messages(&thread_id, 50, None)
                .await
            {
                Ok(msgs) => {
                    messages.set(msgs);
                    error.set(None);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load messages: {e}")));
                }
            }
            loading.set(false);
        });
    });

    let typing_users = services.messaging().get_typing_users(&thread_id);

    let msgs = messages();
    let display_messages: Vec<MessageDisplay> = msgs
        .iter()
        .map(|m| message_to_display_with_context(m, &current_user_id, &msgs))
        .collect();

    // Note: Reply and react handlers are created inline in the loop
    // to avoid closure move issues

    // Handle emoji selection from picker
    let services_for_emoji = services.clone();
    let thread_id_for_emoji = thread_id.clone();
    let on_emoji_select = move |emoji: String| {
        if let Some(message_id) = show_emoji_picker() {
            let thread_id = thread_id_for_emoji.clone();
            let services = services_for_emoji.clone();
            show_emoji_picker.set(None);

            spawn(async move {
                if let Err(e) = services
                    .messaging()
                    .add_reaction(&thread_id, &message_id, &emoji)
                    .await
                {
                    tracing::error!("Failed to add reaction: {e}");
                }
            });
        }
    };

    // Send message handler
    let services_for_send = services.clone();
    let on_send = move |_| {
        let text = message_input().trim().to_string();
        if text.is_empty() {
            return;
        }
        let thread_id = thread_id_for_send.clone();
        let services = services_for_send.clone();
        let reply_id = reply_to().map(|r| r.id.clone());
        message_input.set(String::new());
        reply_to.set(None);

        spawn(async move {
            match services
                .messaging()
                .send_message(&thread_id, &text, reply_id.as_deref())
                .await
            {
                Ok(message) => {
                    // Append sent message to local messages signal to update UI
                    let mut msgs = messages();
                    msgs.push(message);
                    messages.set(msgs);
                }
                Err(e) => {
                    tracing::error!("Failed to send message: {e}");
                }
            }
        });
    };

    rsx! {
        div {
            style: format!(
                "flex: 1; \
                 display: flex; \
                 flex-direction: column; \
                 overflow: hidden; \
                 padding: {};",
                spacing::BASE
            ),

            if loading() {
                div {
                    style: format!(
                        "flex: 1; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         color: {};",
                        semantic::TEXT_MUTED
                    ),
                    "Loading messages..."
                }
            } else if let Some(err) = error() {
                div {
                    style: format!(
                        "flex: 1; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         color: {};",
                        semantic::ERROR
                    ),
                    "{err}"
                }
            } else {
                // Message list
                MessageListContainer {
                    ChatView {
                        for msg in display_messages.iter() {
                            {
                                // Clone current_user_id for each iteration to avoid move issues
                                let user_id_for_reply = current_user_id.clone();
                                rsx! {
                                    ChatMessageItem {
                                        key: "{msg.id}",
                                        message: msg.clone(),
                                        show_picker: show_emoji_picker() == Some(msg.id.clone()),
                                        on_reply: move |msg_id: String| {
                                            let msgs = messages();
                                            if let Some(m) = msgs.iter().find(|m| m.id == msg_id) {
                                                reply_to.set(Some(message_to_display(m, &user_id_for_reply)));
                                            }
                                        },
                                        on_react: move |msg_id: String| {
                                            let current = show_emoji_picker();
                                            if current.as_ref() == Some(&msg_id) {
                                                show_emoji_picker.set(None);
                                            } else {
                                                show_emoji_picker.set(Some(msg_id));
                                            }
                                        },
                                        on_emoji_select: on_emoji_select.clone(),
                                        on_picker_close: move |_| show_emoji_picker.set(None),
                                    }
                                }
                            }
                        }
                    }
                }

                // Typing indicator
                if !typing_users.is_empty() {
                    TypingIndicatorV2 {
                        names: typing_users,
                    }
                }

                // Reply preview
                if let Some(reply) = reply_to() {
                    ReplyPreview {
                        message: reply,
                        on_cancel: move |_| reply_to.set(None),
                    }
                }

                // Composer
                MessageComposerV2 {
                    value: message_input(),
                    placeholder: format!("Message #{}", entity.name),
                    oninput: move |evt: FormEvent| message_input.set(evt.value()),
                    onsubmit: on_send,
                }
            }
        }
    }
}

/// Emoji picker for reactions.
/// Individual chat message item with reply/react handlers.
#[component]
fn ChatMessageItem(
    message: MessageDisplay,
    show_picker: bool,
    on_reply: EventHandler<String>,
    on_react: EventHandler<String>,
    on_emoji_select: EventHandler<String>,
    on_picker_close: EventHandler<()>,
) -> Element {
    let msg_id_for_reply = message.id.clone();
    let msg_id_for_react = message.id.clone();

    rsx! {
        div {
            style: "position: relative;",

            MessageBubble {
                message: message.clone(),
                on_reply: move |_| on_reply.call(msg_id_for_reply.clone()),
                on_react: move |_| on_react.call(msg_id_for_react.clone()),
            }

            // Emoji picker for this message
            if show_picker {
                EmojiPicker {
                    on_select: on_emoji_select,
                    on_close: on_picker_close,
                }
            }
        }
    }
}

#[component]
fn EmojiPicker(on_select: EventHandler<String>, on_close: EventHandler<()>) -> Element {
    let common_emojis = ["👍", "❤️", "😂", "😮", "😢", "🎉", "🔥", "👀"];

    rsx! {
        div {
            style: format!(
                "position: absolute; \
                 bottom: 100%; \
                 right: 0; \
                 margin-bottom: {}; \
                 padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 box-shadow: {}; \
                 z-index: 100; \
                 display: flex; \
                 gap: {};",
                spacing::SM,
                spacing::SM,
                semantic::BG_ELEVATED,
                semantic::BORDER_DEFAULT,
                radius::LG,
                shadow::LG,
                spacing::XS
            ),

            // Close when clicking outside (backdrop)
            div {
                style: "position: fixed; inset: 0; z-index: -1;",
                onclick: move |_| on_close.call(()),
            }

            for emoji in common_emojis {
                button {
                    style: format!(
                        "padding: {}; \
                         background: transparent; \
                         border: none; \
                         font-size: {}; \
                         cursor: pointer; \
                         border-radius: {}; \
                         transition: {}; \
                         &:hover {{ background: {}; }}",
                        spacing::XS,
                        typography::SIZE_LG,
                        radius::MD,
                        motion::transition("background"),
                        semantic::BG_HOVER
                    ),
                    onclick: {
                        let emoji = emoji.to_string();
                        move |_| on_select.call(emoji.clone())
                    },
                    "{emoji}"
                }
            }
        }
    }
}

/// Reply preview shown above the composer.
#[component]
fn ReplyPreview(message: MessageDisplay, on_cancel: EventHandler<()>) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 background: {}; \
                 border-left: 3px solid {}; \
                 border-radius: 0 {} {} 0; \
                 margin-bottom: {};",
                spacing::MD,
                spacing::SM,
                spacing::BASE,
                semantic::BG_TERTIARY,
                semantic::PRIMARY,
                radius::MD,
                radius::MD,
                spacing::SM
            ),

            // Reply indicator
            div {
                style: format!(
                    "color: {}; \
                     font-size: {};",
                    semantic::PRIMARY,
                    typography::SIZE_BASE
                ),
                "↩️"
            }

            // Reply content
            div {
                style: "flex: 1; min-width: 0;",

                p {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0 0 {} 0;",
                        typography::SIZE_XS,
                        typography::WEIGHT_MEDIUM,
                        semantic::PRIMARY,
                        spacing::XXS
                    ),
                    "Replying to {message.author_name}"
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         margin: 0; \
                         overflow: hidden; \
                         text-overflow: ellipsis; \
                         white-space: nowrap;",
                        typography::SIZE_SM,
                        semantic::TEXT_SECONDARY
                    ),
                    "{message.content}"
                }
            }

            // Cancel button
            button {
                style: format!(
                    "padding: {}; \
                     background: transparent; \
                     border: none; \
                     color: {}; \
                     font-size: {}; \
                     cursor: pointer; \
                     border-radius: {}; \
                     transition: {};",
                    spacing::XS,
                    semantic::TEXT_MUTED,
                    typography::SIZE_BASE,
                    radius::MD,
                    motion::transition("color")
                ),
                onclick: move |_| on_cancel.call(()),
                "✕"
            }
        }
    }
}

/// Board tab content (extracted from EntityBoardPageV2).
#[component]
fn EntityBoardContent(entity: UnifiedEntity) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let kanban = services.kanban();

    let mut kanban_snapshot = use_signal(|| kanban.current_snapshot());
    let entity_id = entity.id.clone();
    let entity_id_for_load = entity.id.clone();

    use_future(move || {
        let mut rx = kanban.subscribe();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                kanban_snapshot.set(rx.borrow().clone());
            }
        }
    });

    let services_for_load = services.clone();
    use_effect(move || {
        let entity_id = entity_id_for_load.clone();
        let services = services_for_load.clone();
        spawn(async move {
            if let Err(e) = services.kanban().load_boards(&entity_id).await {
                tracing::error!("Failed to load boards: {e}");
            }
        });
    });

    let snapshot = kanban_snapshot();

    rsx! {
        div {
            style: format!(
                "flex: 1; \
                 overflow: auto; \
                 padding: {};",
                spacing::BASE
            ),

            if snapshot.loading {
                div {
                    style: format!(
                        "padding: {}; \
                         text-align: center; \
                         color: {};",
                        spacing::XL,
                        semantic::TEXT_MUTED
                    ),
                    "Loading boards..."
                }
            } else if snapshot.boards.is_empty() {
                EmptyState {
                    icon: "📋".to_string(),
                    title: "No boards yet".to_string(),
                    description: "Create your first Kanban board to manage tasks".to_string(),
                    action: Some(rsx! {
                        PrimaryButton {
                            onclick: Some(EventHandler::new(|_| {})),
                            "Create Board"
                        }
                    }),
                }
            } else {
                div {
                    style: format!(
                        "display: grid; \
                         grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); \
                         gap: {};",
                        spacing::XL
                    ),

                    for board in snapshot.boards.iter().filter(|b| b.entity_id == entity_id) {
                        BoardSummaryCard {
                            key: "{board.id}",
                            board: board.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Drive tab content.
#[component]
fn EntityDriveContent(entity: UnifiedEntity) -> Element {
    rsx! {
        div {
            style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
            crate::components::DriveBrowser {
                entity_id: entity.id.clone(),
            }
        }
    }
}

/// Documents tab content.
#[component]
fn EntityDocsContent(entity: UnifiedEntity) -> Element {
    let mut docs_loading = use_signal(|| true);
    let docs: Signal<Vec<DocumentItem>> = use_signal(Vec::new);

    use_effect(move || {
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            docs_loading.set(false);
        });
    });

    rsx! {
        div {
            style: format!(
                "flex: 1; \
                 overflow: auto; \
                 padding: {};",
                spacing::XL
            ),

            if docs_loading() {
                div {
                    style: format!(
                        "text-align: center; \
                         padding: {}; \
                         color: {};",
                        spacing::HUGE,
                        semantic::TEXT_MUTED
                    ),
                    "Loading documents..."
                }
            } else if docs().is_empty() {
                EmptyState {
                    icon: "📄".to_string(),
                    title: "No documents yet".to_string(),
                    description: "Create your first document to capture ideas, notes, and specifications.".to_string(),
                    action: Some(rsx! {
                        PrimaryButton {
                            onclick: Some(EventHandler::new(|_| {})),
                            "Create Document"
                        }
                    }),
                }
            } else {
                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {};",
                        spacing::MD
                    ),

                    for doc in docs() {
                        DocumentListItem {
                            key: "{doc.id}",
                            doc: doc.clone(),
                        }
                    }
                }
            }
        }
    }
}

/// Details tab content.
#[component]
fn EntityDetailsContent(entity: UnifiedEntity) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let dir_snapshot = services.directory().current_snapshot();
    let navigator = use_navigator();

    let is_admin = true;

    let type_label = match entity.entity_type {
        UnifiedEntityType::Organization => "Organization",
        UnifiedEntityType::Project => "Project",
        UnifiedEntityType::Channel => "Channel",
        UnifiedEntityType::Group => "Group",
        UnifiedEntityType::Person => "Contact",
    };

    let members: Vec<UnifiedContact> = dir_snapshot.contacts.clone();

    // Handler for messaging a member
    let on_member_message = move |contact_id: String| {
        // Navigate to contact chat route for direct messaging
        navigator.push(Route::ContactChatRoute { contact_id });
    };

    // Handler for calling a member
    let on_member_call = move |_contact_id: String| {
        // TODO: Implement call initiation via CallService
        tracing::info!("Call requested for contact - not yet implemented");
    };

    rsx! {
        div {
            style: format!(
                "flex: 1; \
                 overflow-y: auto; \
                 padding: {};",
                spacing::XL
            ),

            // About section
            DetailSection {
                title: "About".to_string(),

                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {};",
                        spacing::MD
                    ),

                    DetailRow {
                        label: "Type".to_string(),
                        value: type_label.to_string(),
                    }

                    DetailRow {
                        label: "Name".to_string(),
                        value: entity.name.clone(),
                    }

                    DetailRow {
                        label: "ID".to_string(),
                        value: entity.id.clone(),
                    }

                    DetailRow {
                        label: "Members".to_string(),
                        value: format!("{}", entity.member_count),
                    }
                }
            }

            // Members section
            DetailSection {
                title: format!("Members ({})", members.len()),

                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {};",
                        spacing::SM
                    ),

                    if members.is_empty() {
                        p {
                            style: format!(
                                "color: {}; \
                                 font-size: {}; \
                                 padding: {};",
                                semantic::TEXT_MUTED,
                                typography::SIZE_SM,
                                spacing::MD
                            ),
                            "No members to display"
                        }
                    } else {
                        for member in members.iter().take(10) {
                            MemberRow {
                                key: "{member.id}",
                                contact: member.clone(),
                                on_message: on_member_message,
                                on_call: on_member_call,
                            }
                        }

                        if members.len() > 10 {
                            button {
                                style: format!(
                                    "padding: {} {}; \
                                     background: transparent; \
                                     border: none; \
                                     color: {}; \
                                     font-size: {}; \
                                     cursor: pointer;",
                                    spacing::SM,
                                    spacing::MD,
                                    semantic::PRIMARY,
                                    typography::SIZE_SM
                                ),
                                "View all {members.len()} members..."
                            }
                        }
                    }
                }
            }

            // Settings section (admin only)
            if is_admin {
                DetailSection {
                    title: "Settings".to_string(),

                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::MD
                        ),

                        SettingToggle {
                            label: "Allow member invites".to_string(),
                            description: "Members can invite new people to join".to_string(),
                            enabled: true,
                            on_toggle: move |_enabled| {},
                        }

                        SettingToggle {
                            label: "Public visibility".to_string(),
                            description: format!("This {type_label} can be discovered by others"),
                            enabled: false,
                            on_toggle: move |_enabled| {},
                        }
                    }
                }
            }

            // Danger zone (admin only)
            if is_admin {
                DetailSection {
                    title: "Danger Zone".to_string(),
                    danger: true,

                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::MD
                        ),

                        DangerAction {
                            icon: "🚪".to_string(),
                            label: format!("Leave {type_label}"),
                            description: format!("Remove yourself from this {type_label}"),
                            onclick: move |_| {},
                        }

                        DangerAction {
                            icon: "🗄️".to_string(),
                            label: format!("Archive {type_label}"),
                            description: format!("Archive and hide this {type_label}"),
                            onclick: move |_| {},
                        }

                        DangerAction {
                            icon: "🗑️".to_string(),
                            label: format!("Delete {type_label}"),
                            description: format!("Permanently delete this {type_label} and all data"),
                            onclick: move |_| {},
                        }
                    }
                }
            }
        }
    }
}

/// Main authenticated application with new design.
/// Uses `use_context` to access UiServices instead of props.
#[component]
pub fn MainAppV2(children: Element) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let _nav_snapshot = services.navigation().current_snapshot();
    let dir_snapshot = services.directory().current_snapshot();

    // Navigation hooks
    let navigator = use_navigator();
    let current_route: Route = use_route();

    let identity = dir_snapshot.identity.clone();
    let display_name = identity
        .as_ref()
        .map(|i| i.display_name.clone())
        .unwrap_or_else(|| "User".to_string());
    let four_words_fingerprint = identity.as_ref().map(|i| i.four_words.clone());

    let mut search_query = use_signal(String::new);
    let thread_panel_open = use_signal(|| false);
    let expanded_entities: Signal<HashSet<String>> = use_signal(HashSet::new);

    // Modal state for entity creation
    let mut show_create_modal: Signal<Option<CreateEntityType>> = use_signal(|| None);
    let mut create_modal_parent_id: Signal<Option<String>> = use_signal(|| None);

    // Screen reader announcer for navigation changes
    let announcer = use_announcer();
    let mut last_route = use_signal(|| format!("{current_route:?}"));

    // Announce navigation changes for screen readers
    {
        let route_clone = current_route.clone();
        use_effect(move || {
            let current = format!("{route_clone:?}");
            let previous = last_route();
            if current != previous {
                // Determine page name from route
                let page_name = match &route_clone {
                    Route::LoginRoute {} => "Login".to_string(),
                    Route::CreateIdentityRoute {} => "Create Identity".to_string(),
                    Route::RecoverIdentityRoute {} => "Recover Identity".to_string(),
                    Route::DashboardRoute {} => "Dashboard".to_string(),
                    Route::MessagesRoute {} => "Messages".to_string(),
                    Route::ProjectsRoute {} => "Projects".to_string(),
                    Route::ContactsRoute {} => "Contacts".to_string(),
                    Route::NetworkRoute {} => "Network".to_string(),
                    Route::MoreRoute {} => "More options".to_string(),
                    Route::EntityDetailRoute {
                        entity_type,
                        entity_id: _,
                    } => format!("{} details", entity_type),
                    Route::EntityChatRoute {
                        entity_type,
                        entity_id: _,
                    } => format!("{} chat", entity_type),
                    Route::EntityDriveRoute {
                        entity_type,
                        entity_id: _,
                    } => format!("{} drive", entity_type),
                    Route::ProjectBoardRoute { project_id: _ } => "Project board".to_string(),
                    Route::ContactDetailRoute { contact_id: _ } => "Contact details".to_string(),
                    Route::ContactChatRoute { contact_id: _ } => "Contact chat".to_string(),
                };
                announcer(
                    format!("Navigated to {page_name}"),
                    AnnouncementMode::Polite,
                );
                last_route.set(current);
            }
        });
    }

    // Global keyboard shortcuts (Cmd+K / Ctrl+K to focus search)
    use_effect(|| {
        // Set up a global keyboard listener via JavaScript
        // This is more reliable than Dioxus's onkeydown for global shortcuts
        spawn(async move {
            let _ = eval(
                r#"
                // Remove any existing handler to prevent duplicates
                if (window.__communitasKeyHandler) {
                    document.removeEventListener('keydown', window.__communitasKeyHandler);
                }

                // Create new handler for Cmd+K / Ctrl+K
                window.__communitasKeyHandler = function(evt) {
                    // Cmd+K (Mac) or Ctrl+K (Windows/Linux) to focus search
                    if ((evt.metaKey || evt.ctrlKey) && evt.key === 'k') {
                        evt.preventDefault();
                        const searchInput = document.getElementById('global-search-input');
                        if (searchInput) {
                            searchInput.focus();
                            searchInput.select();
                        }
                    }

                    // Escape to close modals (handled by modal components themselves)
                };

                document.addEventListener('keydown', window.__communitasKeyHandler);
                "#,
            );
        });
    });

    // Categorize entities using efficient single-pass hook
    let categorized = CategorizedEntities::from_snapshot(&dir_snapshot);
    let organizations = categorized.organizations;
    let communities = categorized.communities;
    let projects = categorized.projects;
    let personal_groups = categorized.personal_groups;
    let contacts = categorized.contacts;

    // Check if an entity is selected (current_route must be cloned for each use in callbacks)
    let check_entity_selected = |entity: &UnifiedEntity, route: &Route| -> bool {
        let target_route = entity_route(entity);
        match (route, &target_route) {
            (
                Route::EntityDetailRoute {
                    entity_type: et1,
                    entity_id: id1,
                },
                Route::EntityDetailRoute {
                    entity_type: et2,
                    entity_id: id2,
                },
            ) => et1 == et2 && id1 == id2,
            (
                Route::EntityChatRoute {
                    entity_type: et1,
                    entity_id: id1,
                },
                Route::EntityDetailRoute {
                    entity_type: et2,
                    entity_id: id2,
                },
            ) => et1 == et2 && id1 == id2,
            (
                Route::EntityDriveRoute {
                    entity_type: et1,
                    entity_id: id1,
                },
                Route::EntityDetailRoute {
                    entity_type: et2,
                    entity_id: id2,
                },
            ) => et1 == et2 && id1 == id2,
            _ => false,
        }
    };

    // Check if a contact is currently selected based on the route
    let check_contact_selected = |contact: &UnifiedContact, route: &Route| -> bool {
        match route {
            Route::ContactDetailRoute { contact_id } => contact_id == &contact.id,
            Route::ContactChatRoute { contact_id } => contact_id == &contact.id,
            _ => false,
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
                    {
                        let route_for_orgs = current_route.clone();
                        rsx! {
                            EntityListSection {
                                title: "My Organizations".to_string(),
                                entities: organizations.clone(),
                                all_entities: dir_snapshot.entities.clone(),
                                search_filter: search_query(),
                                expanded_ids: expanded_entities,
                                expandable: true,
                                add_button_label: Some("Create Organization".to_string()),
                                is_selected: move |entity| check_entity_selected(&entity, &route_for_orgs),
                                on_navigate: move |entity| {
                                    navigator.push(entity_route(&entity));
                                },
                                on_add: move |_| {
                                    create_modal_parent_id.set(None);
                                    show_create_modal.set(Some(CreateEntityType::Organization));
                                },
                            }
                        }
                    }

                    // Communities
                    {
                        if communities.is_empty() {
                            rsx! {}
                        } else {
                            let route_for_communities = current_route.clone();
                            rsx! {
                                EntityListSection {
                                    title: "Communities".to_string(),
                                    entities: communities.clone(),
                                    all_entities: dir_snapshot.entities.clone(),
                                    search_filter: search_query(),
                                    expanded_ids: expanded_entities,
                                    expandable: true,
                                    is_selected: move |entity| check_entity_selected(&entity, &route_for_communities),
                                    on_navigate: move |entity| {
                                        navigator.push(entity_route(&entity));
                                    },
                                }
                            }
                        }
                    }

                    // Projects
                    {
                        if projects.is_empty() {
                            rsx! {}
                        } else {
                            let route_for_projects = current_route.clone();
                            rsx! {
                                EntityListSection {
                                    title: "Projects".to_string(),
                                    entities: projects.clone(),
                                    search_filter: search_query(),
                                    add_button_label: Some("Create Project".to_string()),
                                    is_selected: move |entity| check_entity_selected(&entity, &route_for_projects),
                                    on_navigate: move |entity| {
                                        navigator.push(entity_route(&entity));
                                    },
                                    on_add: move |_| {
                                        create_modal_parent_id.set(None);
                                        show_create_modal.set(Some(CreateEntityType::Project));
                                    },
                                }
                            }
                        }
                    }

                    // Personal Groups (groups without a parent org)
                    {
                        if personal_groups.is_empty() {
                            rsx! {}
                        } else {
                            let route_for_groups = current_route.clone();
                            rsx! {
                                EntityListSection {
                                    title: "Personal Groups".to_string(),
                                    entities: personal_groups.clone(),
                                    search_filter: search_query(),
                                    add_button_label: Some("Create Group".to_string()),
                                    is_selected: move |entity| check_entity_selected(&entity, &route_for_groups),
                                    on_navigate: move |entity| {
                                        navigator.push(entity_route(&entity));
                                    },
                                    on_add: move |_| {
                                        create_modal_parent_id.set(None);
                                        show_create_modal.set(Some(CreateEntityType::Group));
                                    },
                                }
                            }
                        }
                    }

                    // Direct Messages (contacts with presence indicators)
                    {
                        let route_for_contacts = current_route.clone();
                        rsx! {
                            ContactListSection {
                                title: "Direct Messages".to_string(),
                                contacts: contacts.clone(),
                                search_filter: search_query(),
                                add_button_label: Some("Add Contact".to_string()),
                                is_selected: move |contact| check_contact_selected(&contact, &route_for_contacts),
                                on_navigate: move |contact| {
                                    navigator.push(contact_route(&contact));
                                },
                                on_add: move |_| {},
                            }
                        }
                    }
                }
            },

            // Main content
            {children}
        }

        // Create Entity Modal
        if let Some(entity_type) = show_create_modal() {
            CreateEntityModal {
                entity_type: entity_type,
                parent_id: create_modal_parent_id(),
                on_close: move |_| {
                    show_create_modal.set(None);
                    create_modal_parent_id.set(None);
                },
                on_created: move |_id| {
                    show_create_modal.set(None);
                    create_modal_parent_id.set(None);
                    // Directory service will auto-refresh via watch channels
                },
            }
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
    let org_count = dir_snapshot
        .entities
        .iter()
        .filter(|e| e.entity_type == UnifiedEntityType::Organization)
        .count();
    let project_count = dir_snapshot
        .entities
        .iter()
        .filter(|e| e.entity_type == UnifiedEntityType::Project)
        .count();
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
fn StatCard(icon: String, label: String, value: usize, color: String) -> Element {
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

/// Convert a Message from the service to MessageDisplay for the UI.
fn message_to_display(msg: &communitas_ui_api::Message, current_user_id: &str) -> MessageDisplay {
    message_to_display_with_context(msg, current_user_id, &[])
}

/// Convert a Message to MessageDisplay with context for resolving reply info.
fn message_to_display_with_context(
    msg: &communitas_ui_api::Message,
    current_user_id: &str,
    all_messages: &[communitas_ui_api::Message],
) -> MessageDisplay {
    use crate::components::messaging_v2::RepliedToDisplay;

    // Format timestamp to readable time
    let timestamp_secs = msg.timestamp / 1000;
    let datetime =
        chrono::DateTime::from_timestamp(timestamp_secs as i64, 0).unwrap_or_else(chrono::Utc::now);
    let time_str = datetime.format("%H:%M").to_string();

    // Look up replied-to message if present
    let replied_to = msg.reply_to_id.as_ref().and_then(|reply_id| {
        all_messages
            .iter()
            .find(|m| &m.id == reply_id)
            .map(|m| RepliedToDisplay {
                id: m.id.clone(),
                author_name: m.sender_name.clone(),
                content: m.text.clone(),
            })
    });

    MessageDisplay {
        id: msg.id.clone(),
        author_name: msg.sender_name.clone(),
        author_id: msg.sender_id.clone(),
        content: msg.text.clone(),
        timestamp: time_str,
        is_own: msg.sender_id == current_user_id,
        is_edited: msg.edited,
        reply_count: 0, // Not tracked per-message currently
        reactions: msg
            .reactions
            .iter()
            .map(|r| ReactionDisplay {
                emoji: r.emoji.clone(),
                count: r.count,
                has_reacted: r.reacted_by_me,
            })
            .collect(),
        replied_to,
    }
}

/// Entity chat page with messaging service integration.
#[component]
pub fn EntityChatPageV2(entity: UnifiedEntity) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let dir_snapshot = services.directory().current_snapshot();
    // Use four_words as a unique user identifier (since UnifiedIdentity doesn't have pubkey_hex)
    let current_user_id = dir_snapshot
        .identity
        .as_ref()
        .map(|i| i.four_words.clone())
        .unwrap_or_default();

    let mut message_input = use_signal(String::new);
    let mut messages: Signal<Vec<communitas_ui_api::Message>> = use_signal(Vec::new);
    let mut loading = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Thread ID is the entity ID for entity chats
    let thread_id = entity.id.clone();
    let thread_id_for_load = thread_id.clone();
    let thread_id_for_send = thread_id.clone();

    // Load messages on mount
    let services_for_load = services.clone();
    use_effect(move || {
        let thread_id = thread_id_for_load.clone();
        let services = services_for_load.clone();
        spawn(async move {
            loading.set(true);
            match services
                .messaging()
                .get_messages(&thread_id, 50, None)
                .await
            {
                Ok(msgs) => {
                    messages.set(msgs);
                    error.set(None);
                }
                Err(e) => {
                    error.set(Some(format!("Failed to load messages: {e}")));
                }
            }
            loading.set(false);
        });
    });

    // Get typing users
    let typing_users = services.messaging().get_typing_users(&thread_id);

    // Convert messages to display format
    let msgs = messages();
    let display_messages: Vec<MessageDisplay> = msgs
        .iter()
        .map(|m| message_to_display_with_context(m, &current_user_id, &msgs))
        .collect();

    // Send message handler
    let services_for_send = services.clone();
    let on_send = move |_| {
        let text = message_input().trim().to_string();
        if text.is_empty() {
            return;
        }
        let thread_id = thread_id_for_send.clone();
        let services = services_for_send.clone();
        message_input.set(String::new());

        spawn(async move {
            match services
                .messaging()
                .send_message(&thread_id, &text, None)
                .await
            {
                Ok(new_msg) => {
                    // Add to local messages immediately
                    messages.write().push(new_msg);
                }
                Err(e) => {
                    tracing::error!("Failed to send message: {e}");
                }
            }
        });
    };

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
                    if loading() {
                        div {
                            style: format!(
                                "padding: {}; \
                                 text-align: center; \
                                 color: {};",
                                spacing::XL,
                                semantic::TEXT_MUTED
                            ),
                            "Loading messages..."
                        }
                    } else if let Some(err) = error() {
                        div {
                            style: format!(
                                "padding: {}; \
                                 text-align: center; \
                                 color: {};",
                                spacing::XL,
                                semantic::ERROR
                            ),
                            "{err}"
                        }
                    } else if display_messages.is_empty() {
                        EmptyState {
                            icon: "💬".to_string(),
                            title: "No messages yet".to_string(),
                            description: "Be the first to say something!".to_string(),
                        }
                    } else {
                        DateSeparator { date: "Today".to_string() }

                        for (idx, msg) in display_messages.iter().enumerate() {
                            MessageBubble {
                                message: msg.clone(),
                                show_avatar: idx == 0 || display_messages.get(idx.saturating_sub(1)).map(|prev| prev.author_id != msg.author_id).unwrap_or(true),
                                on_reply: move |_id| {},
                                on_react: move |_id| {},
                            }
                        }
                    }
                }

                if !typing_users.is_empty() {
                    TypingIndicatorV2 {
                        names: typing_users,
                    }
                }

                MessageComposerV2 {
                    value: message_input(),
                    placeholder: format!("Message #{}", entity.name),
                    oninput: move |evt: FormEvent| message_input.set(evt.value()),
                    onsubmit: on_send,
                }
            }
        }
    }
}

/// Entity board page with Kanban service integration.
///
/// Shows the board list for a Project entity, or a specific board if one is active.
#[component]
pub fn EntityBoardPageV2(entity: UnifiedEntity) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let kanban = services.kanban();

    // Subscribe to kanban updates
    let mut kanban_snapshot = use_signal(|| kanban.current_snapshot());
    let entity_id = entity.id.clone();
    let entity_id_for_load = entity.id.clone();

    // Watch for updates
    use_future(move || {
        let mut rx = kanban.subscribe();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                kanban_snapshot.set(rx.borrow().clone());
            }
        }
    });

    // Load boards for this entity on mount
    let services_for_load = services.clone();
    use_effect(move || {
        let entity_id = entity_id_for_load.clone();
        let services = services_for_load.clone();
        spawn(async move {
            if let Err(e) = services.kanban().load_boards(&entity_id).await {
                tracing::error!("Failed to load boards: {e}");
            }
        });
    });

    let snapshot = kanban_snapshot();

    rsx! {
        EntityDetailView {
            entity: entity.clone(),
            initial_tab: EntityTab::Board,
            header_actions: Some(rsx! {
                HeaderAction {
                    icon: "➕".to_string(),
                    label: "New Board".to_string(),
                    primary: true,
                    onclick: move |_| {
                        // Would open create board modal
                    },
                }
            }),

            // Board content - use existing kanban components
            div {
                style: format!(
                    "flex: 1; \
                     overflow: auto; \
                     padding: {};",
                    spacing::BASE
                ),

                if snapshot.loading {
                    div {
                        style: format!(
                            "padding: {}; \
                             text-align: center; \
                             color: {};",
                            spacing::XL,
                            semantic::TEXT_MUTED
                        ),
                        "Loading boards..."
                    }
                } else if snapshot.boards.is_empty() {
                    EmptyState {
                        icon: "📋".to_string(),
                        title: "No boards yet".to_string(),
                        description: "Create your first Kanban board to manage tasks".to_string(),
                        action: Some(rsx! {
                            PrimaryButton {
                                onclick: Some(EventHandler::new(|_| {})),
                                "Create Board"
                            }
                        }),
                    }
                } else {
                    // Show board grid
                    div {
                        style: format!(
                            "display: grid; \
                             grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); \
                             gap: {};",
                            spacing::XL
                        ),

                        for board in snapshot.boards.iter().filter(|b| b.entity_id == entity_id) {
                            BoardSummaryCard {
                                key: "{board.id}",
                                board: board.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Summary card for a kanban board.
#[component]
fn BoardSummaryCard(board: communitas_ui_api::kanban::BoardSummary) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        div {
            style: format!(
                "padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {}; \
                 {}",
                spacing::XL,
                if hovered() { semantic::BG_HOVER } else { semantic::BG_TERTIARY },
                if hovered() { semantic::BORDER_STRONG } else { semantic::BORDER_SUBTLE },
                radius::XL,
                motion::transition("all"),
                if hovered() { format!("transform: translateY(-2px); box-shadow: {};", shadow::LG) } else { String::new() }
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

            // Board name
            h3 {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     margin: 0 0 {} 0;",
                    typography::SIZE_BASE,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY,
                    spacing::SM
                ),
                "{board.name}"
            }

            // Stats
            div {
                style: format!(
                    "display: flex; \
                     gap: {}; \
                     font-size: {}; \
                     color: {};",
                    spacing::MD,
                    typography::SIZE_SM,
                    semantic::TEXT_MUTED
                ),

                span { "{board.column_count} columns" }
                span { "·" }
                span { "{board.card_count} cards" }
            }
        }
    }
}

/// Entity drive page with file browser.
///
/// Displays the DriveBrowser component for the entity's virtual disk.
#[component]
pub fn EntityDrivePageV2(entity: UnifiedEntity) -> Element {
    rsx! {
        EntityDetailView {
            entity: entity.clone(),
            initial_tab: EntityTab::Drive,
            header_actions: Some(rsx! {
                HeaderAction {
                    icon: "⬆️".to_string(),
                    label: "Upload".to_string(),
                    primary: true,
                    onclick: move |_| {
                        // Upload handled by DriveBrowser component internally
                    },
                }
            }),

            // Drive browser content
            div {
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                crate::components::DriveBrowser {
                    entity_id: entity.id.clone(),
                }
            }
        }
    }
}

/// Entity documents page for Projects.
///
/// Shows a filtered view of documents (markdown, text files) from the entity's drive.
#[component]
pub fn EntityDocsPageV2(entity: UnifiedEntity) -> Element {
    let mut docs_loading = use_signal(|| true);
    let mut docs: Signal<Vec<DocumentItem>> = use_signal(Vec::new);

    // Simulate loading documents from /docs folder in drive
    use_effect(move || {
        // In production, this would use DriveService to list /docs directory
        spawn(async move {
            // Simulated delay
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            docs.set(vec![]);
            docs_loading.set(false);
        });
    });

    rsx! {
        EntityDetailView {
            entity: entity.clone(),
            initial_tab: EntityTab::Documents,
            header_actions: Some(rsx! {
                HeaderAction {
                    icon: "📝".to_string(),
                    label: "New Doc".to_string(),
                    primary: true,
                    onclick: move |_| {
                        // Would open new document modal
                    },
                }
            }),

            // Documents content
            div {
                style: format!(
                    "flex: 1; \
                     overflow: auto; \
                     padding: {};",
                    spacing::XL
                ),

                if docs_loading() {
                    div {
                        style: format!(
                            "text-align: center; \
                             padding: {}; \
                             color: {};",
                            spacing::HUGE,
                            semantic::TEXT_MUTED
                        ),
                        "Loading documents..."
                    }
                } else if docs().is_empty() {
                    EmptyState {
                        icon: "📄".to_string(),
                        title: "No documents yet".to_string(),
                        description: "Create your first document to capture ideas, notes, and specifications.".to_string(),
                        action: Some(rsx! {
                            PrimaryButton {
                                onclick: Some(EventHandler::new(|_| {})),
                                "Create Document"
                            }
                        }),
                    }
                } else {
                    // Document list
                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::MD
                        ),

                        for doc in docs() {
                            DocumentListItem {
                                key: "{doc.id}",
                                doc: doc.clone(),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Document item for list display.
#[derive(Clone, PartialEq)]
struct DocumentItem {
    id: String,
    name: String,
    modified: String,
    preview: String,
}

/// Document list item component.
#[component]
fn DocumentListItem(doc: DocumentItem) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {};",
                spacing::MD,
                spacing::BASE,
                if hovered() { semantic::BG_HOVER } else { semantic::BG_TERTIARY },
                if hovered() { semantic::BORDER_STRONG } else { semantic::BORDER_SUBTLE },
                radius::LG,
                motion::transition("all")
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

            // Document icon
            div {
                style: format!(
                    "width: 40px; \
                     height: 40px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     background: {}20; \
                     border-radius: {}; \
                     font-size: {};",
                    palette::JADE_500,
                    radius::MD,
                    typography::SIZE_LG
                ),
                "📄"
            }

            // Document info
            div {
                style: "flex: 1; min-width: 0;",

                h4 {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0 0 {} 0; \
                         overflow: hidden; \
                         text-overflow: ellipsis; \
                         white-space: nowrap;",
                        typography::SIZE_BASE,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY,
                        spacing::XXS
                    ),
                    "{doc.name}"
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         margin: 0; \
                         overflow: hidden; \
                         text-overflow: ellipsis; \
                         white-space: nowrap;",
                        typography::SIZE_SM,
                        semantic::TEXT_MUTED
                    ),
                    "{doc.preview}"
                }
            }

            // Modified time
            span {
                style: format!(
                    "font-size: {}; \
                     color: {}; \
                     flex-shrink: 0;",
                    typography::SIZE_XS,
                    semantic::TEXT_MUTED
                ),
                "{doc.modified}"
            }
        }
    }
}

/// Entity details page with settings and member management.
///
/// Shows entity info, members, and administrative actions.
#[component]
pub fn EntityDetailsPageV2(entity: UnifiedEntity) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let dir_snapshot = services.directory().current_snapshot();
    let navigator = use_navigator();

    // Modal state for editing and inviting
    let mut show_edit_modal = use_signal(|| false);
    let mut show_invite_modal = use_signal(|| false);

    // Determine if current user is admin (simplified - would check permissions)
    let is_admin = true; // Placeholder

    let type_label = match entity.entity_type {
        UnifiedEntityType::Organization => "Organization",
        UnifiedEntityType::Project => "Project",
        UnifiedEntityType::Channel => "Channel",
        UnifiedEntityType::Group => "Group",
        UnifiedEntityType::Person => "Contact",
    };

    // Get members (mock for now - would come from service)
    let members: Vec<UnifiedContact> = dir_snapshot.contacts.clone();

    // Handler for messaging a member
    let on_member_message = move |contact_id: String| {
        navigator.push(Route::EntityDetailRoute {
            entity_type: "person".to_string(),
            entity_id: contact_id,
        });
    };

    // Handler for calling a member
    let on_member_call = move |_contact_id: String| {
        tracing::info!("Call requested for contact - not yet implemented");
    };

    rsx! {
        EntityDetailView {
            entity: entity.clone(),
            initial_tab: EntityTab::Details,
            header_actions: if is_admin {
                Some(rsx! {
                    HeaderAction {
                        icon: "✏️".to_string(),
                        label: "Edit".to_string(),
                        primary: false,
                        onclick: move |_| {
                            show_edit_modal.set(true);
                        },
                    }
                })
            } else {
                None
            },

            // Details content
            div {
                style: format!(
                    "flex: 1; \
                     overflow-y: auto; \
                     padding: {};",
                    spacing::XL
                ),

                // About section
                DetailSection {
                    title: "About".to_string(),

                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::MD
                        ),

                        // Type
                        DetailRow {
                            label: "Type".to_string(),
                            value: type_label.to_string(),
                        }

                        // Name
                        DetailRow {
                            label: "Name".to_string(),
                            value: entity.name.clone(),
                        }

                        // ID
                        DetailRow {
                            label: "ID".to_string(),
                            value: entity.id.clone(),
                        }

                        // Member count
                        DetailRow {
                            label: "Members".to_string(),
                            value: format!("{}", entity.member_count),
                        }
                    }
                }

                // Members section
                DetailSection {
                    title: format!("Members ({})", members.len()),

                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::SM
                        ),

                        // Invite button (admin only)
                        if is_admin {
                            button {
                                style: format!(
                                    "display: flex; \
                                     align-items: center; \
                                     gap: {}; \
                                     padding: {} {}; \
                                     background: {}; \
                                     border: 1px dashed {}; \
                                     border-radius: {}; \
                                     color: {}; \
                                     font-size: {}; \
                                     cursor: pointer; \
                                     transition: {};",
                                    spacing::SM,
                                    spacing::MD,
                                    spacing::BASE,
                                    semantic::BG_TERTIARY,
                                    semantic::PRIMARY,
                                    radius::MD,
                                    semantic::PRIMARY,
                                    typography::SIZE_SM,
                                    motion::transition("background")
                                ),
                                onclick: move |_| {
                                    show_invite_modal.set(true);
                                },
                                span { "👋" }
                                span { "Invite Member" }
                            }
                        }

                        if members.is_empty() {
                            p {
                                style: format!(
                                    "color: {}; \
                                     font-size: {}; \
                                     padding: {};",
                                    semantic::TEXT_MUTED,
                                    typography::SIZE_SM,
                                    spacing::MD
                                ),
                                "No members to display"
                            }
                        } else {
                            for member in members.iter().take(10) {
                                MemberRow {
                                    key: "{member.id}",
                                    contact: member.clone(),
                                    on_message: on_member_message,
                                    on_call: on_member_call,
                                }
                            }

                            if members.len() > 10 {
                                button {
                                    style: format!(
                                        "padding: {} {}; \
                                         background: transparent; \
                                         border: none; \
                                         color: {}; \
                                         font-size: {}; \
                                         cursor: pointer;",
                                        spacing::SM,
                                        spacing::MD,
                                        semantic::PRIMARY,
                                        typography::SIZE_SM
                                    ),
                                    "View all {members.len()} members..."
                                }
                            }
                        }
                    }
                }

                // Settings section (admin only)
                if is_admin {
                    DetailSection {
                        title: "Settings".to_string(),

                        div {
                            style: format!(
                                "display: flex; \
                                 flex-direction: column; \
                                 gap: {};",
                                spacing::MD
                            ),

                            SettingToggle {
                                label: "Allow member invites".to_string(),
                                description: "Members can invite new people to join".to_string(),
                                enabled: true,
                                on_toggle: move |_enabled| {
                                    // Would update setting
                                },
                            }

                            SettingToggle {
                                label: "Public visibility".to_string(),
                                description: "This {type_label} can be discovered by others".to_string(),
                                enabled: false,
                                on_toggle: move |_enabled| {
                                    // Would update setting
                                },
                            }
                        }
                    }
                }

                // Danger zone (admin only)
                if is_admin {
                    DetailSection {
                        title: "Danger Zone".to_string(),
                        danger: true,

                        div {
                            style: format!(
                                "display: flex; \
                                 flex-direction: column; \
                                 gap: {};",
                                spacing::MD
                            ),

                            DangerAction {
                                icon: "🚪".to_string(),
                                label: "Leave {type_label}".to_string(),
                                description: "Remove yourself from this {type_label}".to_string(),
                                onclick: move |_| {
                                    // Would show leave confirmation
                                },
                            }

                            DangerAction {
                                icon: "🗄️".to_string(),
                                label: "Archive {type_label}".to_string(),
                                description: "Archive and hide this {type_label}".to_string(),
                                onclick: move |_| {
                                    // Would show archive confirmation
                                },
                            }

                            DangerAction {
                                icon: "🗑️".to_string(),
                                label: "Delete {type_label}".to_string(),
                                description: "Permanently delete this {type_label} and all data".to_string(),
                                onclick: move |_| {
                                    // Would show delete confirmation
                                },
                            }
                        }
                    }
                }
            }
        }

        // Edit Entity Modal
        if show_edit_modal() {
            EditEntityModal {
                entity: entity.clone(),
                on_close: move |_| {
                    show_edit_modal.set(false);
                },
                on_updated: move |_| {
                    show_edit_modal.set(false);
                    // Directory service will auto-refresh via watch channels
                },
            }
        }

        // Invite Member Modal
        if show_invite_modal() {
            InviteMemberModal {
                entity_id: entity.id.clone(),
                on_close: move |_| {
                    show_invite_modal.set(false);
                },
                on_invited: move |_four_words| {
                    show_invite_modal.set(false);
                    // Directory service will auto-refresh via watch channels
                },
            }
        }
    }
}

/// Section in details page.
#[component]
fn DetailSection(
    title: String,
    #[props(default = false)] danger: bool,
    children: Element,
) -> Element {
    rsx! {
        section {
            style: format!(
                "margin-bottom: {}; \
                 padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {};",
                spacing::XL,
                spacing::XL,
                semantic::BG_TERTIARY,
                if danger { semantic::ERROR } else { semantic::BORDER_SUBTLE },
                radius::XL
            ),

            h3 {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     margin: 0 0 {} 0;",
                    typography::SIZE_BASE,
                    typography::WEIGHT_SEMIBOLD,
                    if danger { semantic::ERROR } else { semantic::TEXT_PRIMARY },
                    spacing::MD
                ),
                "{title}"
            }

            {children}
        }
    }
}

/// Key-value row in details.
#[component]
fn DetailRow(label: String, value: String) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 justify-content: space-between; \
                 align-items: center; \
                 padding: {} 0; \
                 border-bottom: 1px solid {};",
                spacing::SM,
                semantic::BORDER_SUBTLE
            ),

            span {
                style: format!(
                    "font-size: {}; \
                     color: {};",
                    typography::SIZE_SM,
                    semantic::TEXT_SECONDARY
                ),
                "{label}"
            }

            span {
                style: format!(
                    "font-size: {}; \
                     color: {}; \
                     font-family: {};",
                    typography::SIZE_SM,
                    semantic::TEXT_PRIMARY,
                    typography::FONT_MONO
                ),
                "{value}"
            }
        }
    }
}

/// Skeleton loading placeholder with shimmer animation.
#[component]
pub fn Skeleton(
    /// Width of the skeleton (e.g., "100%", "200px")
    #[props(default = "100%".to_string())]
    width: String,
    /// Height of the skeleton (e.g., "1rem", "48px")
    #[props(default = "1rem".to_string())]
    height: String,
    /// Border radius (default is MD)
    #[props(default = radius::MD.to_string())]
    border_radius: String,
) -> Element {
    rsx! {
        div {
            class: "skeleton",
            style: format!(
                "width: {}; \
                 height: {}; \
                 border-radius: {};",
                width,
                height,
                border_radius
            ),
        }
    }
}

/// Skeleton text line for content loading states.
#[component]
pub fn SkeletonText(
    /// Number of lines
    #[props(default = 1)]
    lines: u8,
    /// Make the last line shorter for natural look
    #[props(default = true)]
    last_short: bool,
) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 gap: {};",
                spacing::SM
            ),

            for i in 0..lines {
                Skeleton {
                    key: "{i}",
                    width: if last_short && i == lines - 1 { "70%".to_string() } else { "100%".to_string() },
                    height: "0.875rem".to_string(),
                }
            }
        }
    }
}

/// Skeleton for user avatar with optional text.
#[component]
pub fn SkeletonAvatar(
    /// Size of the avatar
    #[props(default = "40px".to_string())]
    size: String,
    /// Show text lines next to avatar
    #[props(default = false)]
    with_text: bool,
) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {};",
                spacing::MD
            ),

            Skeleton {
                width: size.clone(),
                height: size,
                border_radius: radius::FULL.to_string(),
            }

            if with_text {
                div {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {}; \
                         flex: 1;",
                        spacing::XS
                    ),

                    Skeleton { width: "120px".to_string(), height: "0.875rem".to_string() }
                    Skeleton { width: "80px".to_string(), height: "0.75rem".to_string() }
                }
            }
        }
    }
}

/// Member row in member list with action buttons.
#[component]
fn MemberRow(
    contact: UnifiedContact,
    on_message: EventHandler<String>,
    on_call: EventHandler<String>,
) -> Element {
    let mut hovered = use_signal(|| false);
    let contact_id_for_message = contact.id.clone();
    let contact_id_for_call = contact.id.clone();

    let presence_color = match contact.presence {
        PresenceStatus::Online => semantic::PRESENCE_ONLINE,
        PresenceStatus::Away => semantic::PRESENCE_AWAY,
        PresenceStatus::Busy => semantic::PRESENCE_BUSY,
        PresenceStatus::Offline | PresenceStatus::Unknown => semantic::PRESENCE_OFFLINE,
    };

    let bg_style = if hovered() {
        format!("background: {};", semantic::BG_HOVER)
    } else {
        "background: transparent;".to_string()
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {}; \
                 border-radius: {}; \
                 transition: {}; \
                 {}",
                spacing::MD,
                spacing::SM,
                radius::MD,
                motion::transition("background"),
                bg_style
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),

            // Avatar with presence indicator
            div {
                style: "position: relative;",

                div {
                    style: format!(
                        "width: 36px; \
                         height: 36px; \
                         display: flex; \
                         align-items: center; \
                         justify-content: center; \
                         background: {}20; \
                         border-radius: {}; \
                         font-size: {}; \
                         color: {};",
                        palette::JADE_500,
                        radius::FULL,
                        typography::SIZE_BASE,
                        palette::JADE_500
                    ),
                    "👤"
                }

                // Presence dot
                div {
                    style: format!(
                        "position: absolute; \
                         bottom: 0; \
                         right: 0; \
                         width: 10px; \
                         height: 10px; \
                         background: {}; \
                         border: 2px solid {}; \
                         border-radius: {};",
                        presence_color,
                        semantic::BG_TERTIARY,
                        radius::FULL
                    ),
                }
            }

            // Name and status
            div {
                style: "flex: 1;",

                p {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0;",
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY
                    ),
                    "{contact.display_name}"
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         margin: 0;",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED
                    ),
                    "{contact.status}"
                }
            }

            // Action buttons (on hover)
            if hovered() {
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {};",
                        spacing::XS
                    ),

                    // Message button
                    ContactActionButton {
                        icon: "💬".to_string(),
                        tooltip: "Send Message".to_string(),
                        onclick: move |_| on_message.call(contact_id_for_message.clone()),
                    }

                    // Call button
                    ContactActionButton {
                        icon: "📞".to_string(),
                        tooltip: "Start Call".to_string(),
                        onclick: move |_| on_call.call(contact_id_for_call.clone()),
                    }
                }
            }
        }
    }
}

/// Small action button for contact actions.
#[component]
fn ContactActionButton(
    icon: String,
    tooltip: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let bg_style = if hovered() {
        format!("background: {};", semantic::BG_ELEVATED)
    } else {
        format!("background: {};", semantic::BG_TERTIARY)
    };

    rsx! {
        button {
            style: format!(
                "width: 28px; \
                 height: 28px; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 border: none; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {}; \
                 {}",
                radius::MD,
                motion::transition("background"),
                bg_style
            ),
            title: "{tooltip}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| {
                evt.stop_propagation();
                onclick.call(evt);
            },
            "{icon}"
        }
    }
}

/// Setting toggle component.
#[component]
fn SettingToggle(
    label: String,
    description: String,
    enabled: bool,
    on_toggle: EventHandler<bool>,
) -> Element {
    let mut local_enabled = use_signal(|| enabled);

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 justify-content: space-between; \
                 align-items: center; \
                 padding: {} 0;",
                spacing::SM
            ),

            div {
                style: "flex: 1;",

                p {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0 0 {} 0;",
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::TEXT_PRIMARY,
                        spacing::XXS
                    ),
                    "{label}"
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         margin: 0;",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED
                    ),
                    "{description}"
                }
            }

            // Toggle switch
            button {
                style: format!(
                    "width: 44px; \
                     height: 24px; \
                     border-radius: {}; \
                     border: none; \
                     background: {}; \
                     cursor: pointer; \
                     position: relative; \
                     transition: {};",
                    radius::FULL,
                    if local_enabled() { semantic::PRIMARY } else { semantic::BG_ELEVATED },
                    motion::transition("background")
                ),
                onclick: move |_| {
                    let new_val = !local_enabled();
                    local_enabled.set(new_val);
                    on_toggle.call(new_val);
                },

                // Knob
                div {
                    style: format!(
                        "position: absolute; \
                         top: 2px; \
                         left: {}; \
                         width: 20px; \
                         height: 20px; \
                         border-radius: {}; \
                         background: white; \
                         transition: {}; \
                         box-shadow: 0 1px 3px rgba(0,0,0,0.2);",
                        if local_enabled() { "22px" } else { "2px" },
                        radius::FULL,
                        motion::transition("left")
                    ),
                }
            }
        }
    }
}

/// Danger zone action button.
#[component]
fn DangerAction(
    icon: String,
    label: String,
    description: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        button {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 width: 100%; \
                 padding: {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: pointer; \
                 text-align: left; \
                 transition: {};",
                spacing::MD,
                spacing::BASE,
                if hovered() { format!("{}10", semantic::ERROR) } else { "transparent".to_string() },
                semantic::ERROR,
                radius::LG,
                motion::transition("all")
            ),
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            span {
                style: format!("font-size: {};", typography::SIZE_LG),
                "{icon}"
            }

            div {
                style: "flex: 1;",

                p {
                    style: format!(
                        "font-size: {}; \
                         font-weight: {}; \
                         color: {}; \
                         margin: 0 0 {} 0; \
                         font-family: {};",
                        typography::SIZE_SM,
                        typography::WEIGHT_MEDIUM,
                        semantic::ERROR,
                        spacing::XXS,
                        typography::FONT_BODY
                    ),
                    "{label}"
                }

                p {
                    style: format!(
                        "font-size: {}; \
                         color: {}; \
                         margin: 0; \
                         font-family: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED,
                        typography::FONT_BODY
                    ),
                    "{description}"
                }
            }
        }
    }
}

// ============================================================================
// Entity Management Modals
// ============================================================================

/// Type of entity to create.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CreateEntityType {
    Organization,
    Channel,
    Project,
    Group,
}

impl CreateEntityType {
    fn label(&self) -> &'static str {
        match self {
            Self::Organization => "Organization",
            Self::Channel => "Channel",
            Self::Project => "Project",
            Self::Group => "Group",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Organization => "🏢",
            Self::Channel => "#",
            Self::Project => "📋",
            Self::Group => "👥",
        }
    }

    fn description_placeholder(&self) -> &'static str {
        match self {
            Self::Organization => "What is this organization about?",
            Self::Channel => "What topics will be discussed here?",
            Self::Project => "What is the goal of this project?",
            Self::Group => "What brings this group together?",
        }
    }
}

/// Modal for creating new entities.
#[component]
pub fn CreateEntityModal(
    entity_type: CreateEntityType,
    /// Parent entity ID for channels/projects/groups within an org.
    #[props(default)]
    parent_id: Option<String>,
    on_close: EventHandler<()>,
    on_created: EventHandler<String>,
) -> Element {
    let mut name = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let entity_label = entity_type.label();
    let entity_icon = entity_type.icon();
    let desc_placeholder = entity_type.description_placeholder();

    let name_valid = !name().trim().is_empty();

    // Shared submit logic - called from both form submit and button click
    let mut do_submit = move || {
        if !name_valid || submitting() {
            return;
        }

        let name_value = name().trim().to_string();
        let desc_value = description().trim().to_string();
        let desc_opt = if desc_value.is_empty() {
            None
        } else {
            Some(desc_value)
        };

        submitting.set(true);
        error.set(None);

        // TODO: Call DirectoryService to create the entity
        // For now, simulate creation with a fake ID
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Simulate success - use timestamp for unique ID
            let fake_id = format!(
                "{}_{:x}",
                entity_type.label().to_lowercase(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            );
            tracing::info!(
                "Created entity: {} - {} ({:?})",
                name_value,
                desc_opt.unwrap_or_default(),
                entity_type
            );

            submitting.set(false);
            on_created.call(fake_id);
        });
    };

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        do_submit();
    };

    let on_click_submit = move |_: MouseEvent| {
        do_submit();
    };

    // Handle Escape key to close modal
    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape {
            on_close.call(());
        }
    };

    rsx! {
        // Modal backdrop
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 z-index: 1000; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: rgba(0, 0, 0, 0.7); \
                 backdrop-filter: blur(4px); \
                 animation: fadeIn {} {};",
                motion::NORMAL,
                motion::EASE_OUT
            ),
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "create-entity-modal-title",
            tabindex: "0",
            autofocus: true,
            onkeydown: on_keydown,
            onclick: move |_| on_close.call(()),

            // Modal content
            div {
                style: format!(
                    "width: 100%; \
                     max-width: 480px; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     box-shadow: {}; \
                     overflow: hidden; \
                     animation: slideInUp {} {};",
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    shadow::XL,
                    motion::SLOW,
                    motion::EASE_OUT
                ),
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         padding: {} {}; \
                         border-bottom: 1px solid {};",
                        spacing::MD,
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    span {
                        style: format!("font-size: {};", typography::SIZE_XL),
                        aria_hidden: "true",
                        "{entity_icon}"
                    }

                    h2 {
                        id: "create-entity-modal-title",
                        style: format!(
                            "flex: 1; \
                             margin: 0; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_LG,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "Create {entity_label}"
                    }

                    button {
                        style: format!(
                            "width: 32px; \
                             height: 32px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: transparent; \
                             border: none; \
                             border-radius: {}; \
                             color: {}; \
                             cursor: pointer; \
                             transition: {};",
                            radius::MD,
                            semantic::TEXT_MUTED,
                            motion::transition("background")
                        ),
                        aria_label: "Close dialog",
                        onclick: move |_| on_close.call(()),
                        span { aria_hidden: "true", "✕" }
                    }
                }

                // Form
                form {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {}; \
                         padding: {};",
                        spacing::LG,
                        spacing::XL
                    ),
                    onsubmit: on_submit,

                    // Error message
                    if let Some(err) = error() {
                        div {
                            style: format!(
                                "padding: {}; \
                                 background: {}20; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {};",
                                spacing::MD,
                                semantic::ERROR,
                                semantic::ERROR,
                                radius::MD,
                                semantic::ERROR
                            ),
                            role: "alert",
                            "{err}"
                        }
                    }

                    // Name field
                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::SM
                        ),

                        label {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_PRIMARY
                            ),
                            r#for: "entity-name",
                            "Name"
                        }

                        input {
                            id: "entity-name",
                            style: format!(
                                "width: 100%; \
                                 padding: {}; \
                                 background: {}; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {}; \
                                 font-size: {}; \
                                 outline: none; \
                                 transition: {};",
                                spacing::MD,
                                semantic::BG_TERTIARY,
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::TEXT_PRIMARY,
                                typography::SIZE_BASE,
                                motion::transition("border-color")
                            ),
                            r#type: "text",
                            placeholder: "Enter name...",
                            value: "{name()}",
                            autofocus: true,
                            oninput: move |evt| name.set(evt.value()),
                        }
                    }

                    // Description field
                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::SM
                        ),

                        label {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_PRIMARY
                            ),
                            r#for: "entity-description",
                            "Description (optional)"
                        }

                        textarea {
                            id: "entity-description",
                            style: format!(
                                "width: 100%; \
                                 min-height: 80px; \
                                 padding: {}; \
                                 background: {}; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {}; \
                                 font-size: {}; \
                                 resize: vertical; \
                                 outline: none; \
                                 transition: {};",
                                spacing::MD,
                                semantic::BG_TERTIARY,
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::TEXT_PRIMARY,
                                typography::SIZE_BASE,
                                motion::transition("border-color")
                            ),
                            placeholder: "{desc_placeholder}",
                            value: "{description()}",
                            oninput: move |evt| description.set(evt.value()),
                        }
                    }
                }

                // Actions
                div {
                    style: format!(
                        "display: flex; \
                         justify-content: flex-end; \
                         gap: {}; \
                         padding: {} {}; \
                         border-top: 1px solid {};",
                        spacing::MD,
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    SecondaryButton {
                        onclick: Some(EventHandler::new(move |_| on_close.call(()))),
                        disabled: submitting(),
                        "Cancel"
                    }

                    PrimaryButton {
                        onclick: Some(EventHandler::new(on_click_submit)),
                        disabled: !name_valid || submitting(),
                        if submitting() {
                            "Creating..."
                        } else {
                            "Create {entity_label}"
                        }
                    }
                }
            }
        }
    }
}

/// Modal for inviting members via four-words.
#[component]
pub fn InviteMemberModal(
    entity_id: String,
    on_close: EventHandler<()>,
    on_invited: EventHandler<String>,
) -> Element {
    let mut four_words = use_signal(String::new);
    let mut submitting = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Validate four-words format (4 words separated by hyphens)
    let four_words_valid = {
        let input = four_words();
        let words: Vec<&str> = input.split('-').collect();
        words.len() == 4 && words.iter().all(|w| !w.is_empty())
    };

    // Shared submit logic - called from both form submit and button click
    let mut do_submit = move || {
        if !four_words_valid || submitting() {
            return;
        }

        let words = four_words().trim().to_string();
        submitting.set(true);
        error.set(None);

        // TODO: Call DirectoryService to invite member
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            // Simulate success
            tracing::info!("Invited member with four-words: {}", words);

            submitting.set(false);
            on_invited.call(words);
        });
    };

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        do_submit();
    };

    let on_click_submit = move |_: MouseEvent| {
        do_submit();
    };

    // Handle Escape key to close modal
    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape {
            on_close.call(());
        }
    };

    rsx! {
        // Modal backdrop
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 z-index: 1000; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: rgba(0, 0, 0, 0.7); \
                 backdrop-filter: blur(4px); \
                 animation: fadeIn {} {};",
                motion::NORMAL,
                motion::EASE_OUT
            ),
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "invite-member-modal-title",
            tabindex: "0",
            autofocus: true,
            onkeydown: on_keydown,
            onclick: move |_| on_close.call(()),

            // Modal content
            div {
                style: format!(
                    "width: 100%; \
                     max-width: 420px; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     box-shadow: {}; \
                     overflow: hidden; \
                     animation: slideInUp {} {};",
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    shadow::XL,
                    motion::SLOW,
                    motion::EASE_OUT
                ),
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         padding: {} {}; \
                         border-bottom: 1px solid {};",
                        spacing::MD,
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    span {
                        style: format!("font-size: {};", typography::SIZE_XL),
                        aria_hidden: "true",
                        "👋"
                    }

                    h2 {
                        id: "invite-member-modal-title",
                        style: format!(
                            "flex: 1; \
                             margin: 0; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_LG,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "Invite Member"
                    }

                    button {
                        style: format!(
                            "width: 32px; \
                             height: 32px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: transparent; \
                             border: none; \
                             border-radius: {}; \
                             color: {}; \
                             cursor: pointer;",
                            radius::MD,
                            semantic::TEXT_MUTED
                        ),
                        aria_label: "Close dialog",
                        onclick: move |_| on_close.call(()),
                        span { aria_hidden: "true", "✕" }
                    }
                }

                // Form
                form {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {}; \
                         padding: {};",
                        spacing::LG,
                        spacing::XL
                    ),
                    onsubmit: on_submit,

                    // Error message
                    if let Some(err) = error() {
                        div {
                            style: format!(
                                "padding: {}; \
                                 background: {}20; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {};",
                                spacing::MD,
                                semantic::ERROR,
                                semantic::ERROR,
                                radius::MD,
                                semantic::ERROR
                            ),
                            role: "alert",
                            "{err}"
                        }
                    }

                    // Instructions
                    p {
                        style: format!(
                            "font-size: {}; \
                             color: {}; \
                             margin: 0;",
                            typography::SIZE_SM,
                            semantic::TEXT_MUTED
                        ),
                        "Ask the person you want to invite for their connection words. They can find this in their profile settings."
                    }

                    // Four-words field
                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::SM
                        ),

                        label {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_PRIMARY
                            ),
                            r#for: "four-words",
                            "Connection Words"
                        }

                        input {
                            id: "four-words",
                            style: format!(
                                "width: 100%; \
                                 padding: {}; \
                                 background: {}; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {}; \
                                 font-size: {}; \
                                 font-family: monospace; \
                                 text-align: center; \
                                 letter-spacing: 0.05em; \
                                 outline: none;",
                                spacing::MD,
                                semantic::BG_TERTIARY,
                                if four_words_valid { semantic::PRIMARY } else { semantic::BORDER_DEFAULT },
                                radius::LG,
                                semantic::TEXT_PRIMARY,
                                typography::SIZE_BASE
                            ),
                            r#type: "text",
                            placeholder: "ocean-forest-moon-star",
                            value: "{four_words()}",
                            autofocus: true,
                            oninput: move |evt| four_words.set(evt.value()),
                        }

                        p {
                            style: format!(
                                "font-size: {}; \
                                 color: {}; \
                                 margin: 0; \
                                 text-align: center;",
                                typography::SIZE_XS,
                                if four_words_valid { semantic::PRIMARY } else { semantic::TEXT_MUTED }
                            ),
                            if four_words_valid {
                                "✓ Valid format"
                            } else {
                                "Format: word-word-word-word"
                            }
                        }
                    }
                }

                // Actions
                div {
                    style: format!(
                        "display: flex; \
                         justify-content: flex-end; \
                         gap: {}; \
                         padding: {} {}; \
                         border-top: 1px solid {};",
                        spacing::MD,
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    SecondaryButton {
                        onclick: Some(EventHandler::new(move |_| on_close.call(()))),
                        disabled: submitting(),
                        "Cancel"
                    }

                    PrimaryButton {
                        onclick: Some(EventHandler::new(on_click_submit)),
                        disabled: !four_words_valid || submitting(),
                        if submitting() {
                            "Inviting..."
                        } else {
                            "Send Invite"
                        }
                    }
                }
            }
        }
    }
}

/// Modal for editing an existing entity.
#[component]
pub fn EditEntityModal(
    entity: UnifiedEntity,
    on_close: EventHandler<()>,
    on_updated: EventHandler<()>,
) -> Element {
    let mut name = use_signal(|| entity.name.clone());
    let mut description = use_signal(|| entity.description.clone());
    let mut submitting = use_signal(|| false);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let entity_label = match entity.entity_type {
        UnifiedEntityType::Organization => "Organization",
        UnifiedEntityType::Project => "Project",
        UnifiedEntityType::Channel => "Channel",
        UnifiedEntityType::Group => "Group",
        UnifiedEntityType::Person => "Contact",
    };

    let entity_icon = match entity.entity_type {
        UnifiedEntityType::Organization => "🏢",
        UnifiedEntityType::Project => "📋",
        UnifiedEntityType::Channel => "💬",
        UnifiedEntityType::Group => "👥",
        UnifiedEntityType::Person => "👤",
    };

    let name_valid = !name().trim().is_empty();
    let entity_id = entity.id.clone();

    let on_submit = {
        let entity_id = entity_id.clone();
        move |evt: FormEvent| {
            evt.prevent_default();

            if name().trim().is_empty() || submitting() {
                return;
            }

            let name_value = name().trim().to_string();
            let desc_value = description().trim().to_string();
            let _desc_opt = if desc_value.is_empty() {
                None
            } else {
                Some(desc_value)
            };

            submitting.set(true);
            error.set(None);

            let entity_id_for_spawn = entity_id.clone();

            // TODO: Call DirectoryService to update the entity
            spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                tracing::info!("Updated entity: {} ({})", name_value, entity_id_for_spawn);

                submitting.set(false);
                on_updated.call(());
            });
        }
    };

    let on_click_submit = move |_: MouseEvent| {
        if name().trim().is_empty() || submitting() {
            return;
        }

        let name_value = name().trim().to_string();
        let desc_value = description().trim().to_string();
        let _desc_opt = if desc_value.is_empty() {
            None
        } else {
            Some(desc_value)
        };

        submitting.set(true);
        error.set(None);

        let entity_id_for_spawn = entity_id.clone();

        // TODO: Call DirectoryService to update the entity
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            tracing::info!("Updated entity: {} ({})", name_value, entity_id_for_spawn);

            submitting.set(false);
            on_updated.call(());
        });
    };

    // Handle Escape key to close modal
    let on_keydown = move |evt: KeyboardEvent| {
        if evt.key() == Key::Escape {
            on_close.call(());
        }
    };

    rsx! {
        // Modal backdrop
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 z-index: 1000; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: rgba(0, 0, 0, 0.7); \
                 backdrop-filter: blur(4px); \
                 animation: fadeIn {} {};",
                motion::NORMAL,
                motion::EASE_OUT
            ),
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "edit-entity-modal-title",
            tabindex: "0",
            autofocus: true,
            onkeydown: on_keydown,
            onclick: move |_| on_close.call(()),

            // Modal content
            div {
                style: format!(
                    "width: 100%; \
                     max-width: 480px; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     box-shadow: {}; \
                     overflow: hidden; \
                     animation: slideInUp {} {};",
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    shadow::XL,
                    motion::SLOW,
                    motion::EASE_OUT
                ),
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         padding: {} {}; \
                         border-bottom: 1px solid {};",
                        spacing::MD,
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    span {
                        style: format!("font-size: {};", typography::SIZE_XL),
                        aria_hidden: "true",
                        "{entity_icon}"
                    }

                    h2 {
                        id: "edit-entity-modal-title",
                        style: format!(
                            "flex: 1; \
                             margin: 0; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_LG,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "Edit {entity_label}"
                    }

                    button {
                        style: format!(
                            "width: 32px; \
                             height: 32px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: transparent; \
                             border: none; \
                             border-radius: {}; \
                             color: {}; \
                             cursor: pointer; \
                             transition: {};",
                            radius::MD,
                            semantic::TEXT_MUTED,
                            motion::transition("background")
                        ),
                        aria_label: "Close dialog",
                        onclick: move |_| on_close.call(()),
                        span { aria_hidden: "true", "✕" }
                    }
                }

                // Form
                form {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {}; \
                         padding: {};",
                        spacing::LG,
                        spacing::XL
                    ),
                    onsubmit: on_submit,

                    // Error message
                    if let Some(err) = error() {
                        div {
                            style: format!(
                                "padding: {}; \
                                 background: {}20; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {};",
                                spacing::MD,
                                semantic::ERROR,
                                semantic::ERROR,
                                radius::MD,
                                semantic::ERROR
                            ),
                            role: "alert",
                            "{err}"
                        }
                    }

                    // Name field
                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::SM
                        ),

                        label {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_SECONDARY
                            ),
                            "Name"
                        }

                        input {
                            style: format!(
                                "width: 100%; \
                                 padding: {}; \
                                 background: {}; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {}; \
                                 font-size: {}; \
                                 outline: none; \
                                 transition: {};",
                                spacing::MD,
                                semantic::BG_TERTIARY,
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::TEXT_PRIMARY,
                                typography::SIZE_BASE,
                                motion::transition("border-color")
                            ),
                            r#type: "text",
                            placeholder: "Enter name...",
                            value: "{name()}",
                            autofocus: true,
                            oninput: move |evt| name.set(evt.value()),
                        }
                    }

                    // Description field
                    div {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::SM
                        ),

                        label {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_SECONDARY
                            ),
                            "Description (optional)"
                        }

                        textarea {
                            style: format!(
                                "width: 100%; \
                                 min-height: 80px; \
                                 padding: {}; \
                                 background: {}; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 color: {}; \
                                 font-size: {}; \
                                 outline: none; \
                                 resize: vertical; \
                                 transition: {};",
                                spacing::MD,
                                semantic::BG_TERTIARY,
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::TEXT_PRIMARY,
                                typography::SIZE_BASE,
                                motion::transition("border-color")
                            ),
                            placeholder: "Enter description...",
                            value: "{description()}",
                            oninput: move |evt| description.set(evt.value()),
                        }
                    }
                }

                // Actions
                div {
                    style: format!(
                        "display: flex; \
                         justify-content: flex-end; \
                         gap: {}; \
                         padding: {} {}; \
                         border-top: 1px solid {};",
                        spacing::MD,
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    SecondaryButton {
                        onclick: Some(EventHandler::new(move |_| on_close.call(()))),
                        disabled: submitting(),
                        "Cancel"
                    }

                    PrimaryButton {
                        onclick: Some(EventHandler::new(on_click_submit)),
                        disabled: !name_valid || submitting(),
                        if submitting() {
                            "Saving..."
                        } else {
                            "Save Changes"
                        }
                    }
                }
            }
        }
    }
}

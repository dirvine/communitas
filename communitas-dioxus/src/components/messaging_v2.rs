// SPDX-License-Identifier: MIT OR Apache-2.0

//! Enhanced messaging components with Digital Forest Sanctuary theme.
//!
//! Features:
//! - Message bubbles with glass effect
//! - Thread support
//! - Reactions
//! - Rich composer
//! - Typing indicators

use crate::components::emoji_picker::{EmojiPicker, QuickReactionBar};
use crate::components::markdown::MarkdownContent;
use crate::design_tokens::{motion, palette, radius, semantic, shadow, spacing, typography};
use crate::styles_v2::avatar;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Message data structure for display.
#[derive(Clone, PartialEq)]
pub struct MessageDisplay {
    pub id: String,
    pub author_name: String,
    pub author_id: String,
    pub content: String,
    pub timestamp: String,
    pub is_own: bool,
    pub is_edited: bool,
    pub reply_count: u32,
    pub reactions: Vec<ReactionDisplay>,
    /// The message this is replying to, if any.
    pub replied_to: Option<RepliedToDisplay>,
    /// Whether this message is currently pinned in the thread.
    pub is_pinned: bool,
}

/// Simplified display info for replied-to messages.
#[derive(Clone, PartialEq)]
pub struct RepliedToDisplay {
    pub id: String,
    pub author_name: String,
    pub content: String,
}

/// Reaction data for display.
#[derive(Clone, PartialEq)]
pub struct ReactionDisplay {
    pub emoji: String,
    pub count: u32,
    pub has_reacted: bool,
}

/// Chat view container with message list and composer.
#[component]
pub fn ChatView(children: Element) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 height: 100%; \
                 background: {};",
                semantic::BG_PRIMARY
            ),
            {children}
        }
    }
}

/// Message list container with loading states and auto-scroll.
#[component]
pub fn MessageListContainer(
    /// Whether messages are currently loading.
    #[props(default = false)]
    loading: bool,
    /// Whether the message list is empty (loaded but no messages).
    #[props(default = false)]
    empty: bool,
    /// Child elements (messages).
    children: Element,
) -> Element {
    rsx! {
        div {
            style: format!(
                "flex: 1; \
                 overflow-y: auto; \
                 padding: {} {}; \
                 display: flex; \
                 flex-direction: column; \
                 gap: {}; \
                 scrollbar-width: thin; \
                 scrollbar-color: {} transparent;",
                spacing::BASE,
                spacing::XL,
                spacing::SM,
                semantic::BORDER_DEFAULT
            ),
            role: "status",
            aria_busy: if loading { "true" } else { "false" },

            if loading {
                MessageListSkeleton {}
            } else if empty {
                MessageEmptyState {}
            } else {
                {children}
            }
        }
    }
}

/// Skeleton placeholder for message list during loading.
#[component]
fn MessageListSkeleton() -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; flex-direction: column; gap: {}; padding: {};",
                spacing::MD, spacing::LG
            ),
            aria_label: "Loading messages",
            for i in 0..6 {
                MessageSkeletonItem { key: "{i}", align_right: i % 3 == 0 }
            }
        }

        style {
            r#"
            @keyframes messagePulse {{
                0%, 100% {{ opacity: 1; }}
                50% {{ opacity: 0.5; }}
            }}
            "#
        }
    }
}

/// Single message skeleton item.
#[component]
fn MessageSkeletonItem(
    /// Whether the skeleton aligns to the right (own messages).
    #[props(default = false)]
    align_right: bool,
) -> Element {
    let align = if align_right {
        "flex-end"
    } else {
        "flex-start"
    };
    let width = if align_right { "60%" } else { "70%" };
    rsx! {
        div {
            style: format!(
                "display: flex; flex-direction: column; align-items: {}; max-width: {};",
                align, width
            ),
            // Avatar + name skeleton
            div {
                style: format!(
                    "display: flex; align-items: center; gap: {}; margin-bottom: {};",
                    spacing::XS, spacing::XXS
                ),
                div {
                    style: format!(
                        "width: 28px; height: 28px; border-radius: {}; background: {}; \
                         animation: messagePulse 1.5s ease-in-out infinite;",
                        radius::FULL, semantic::BG_TERTIARY
                    ),
                }
                div {
                    style: format!(
                        "width: 80px; height: 12px; border-radius: {}; background: {}; \
                         animation: messagePulse 1.5s ease-in-out infinite;",
                        radius::SM, semantic::BG_TERTIARY
                    ),
                }
            }
            // Message content skeleton
            div {
                style: format!(
                    "padding: {} {}; border-radius: {}; background: {}; \
                     animation: messagePulse 1.5s ease-in-out infinite;",
                    spacing::SM, spacing::MD, radius::LG, semantic::BG_ELEVATED
                ),
                div {
                    style: format!(
                        "width: 200px; height: 14px; border-radius: {}; background: {}; \
                         margin-bottom: {};",
                        radius::SM, semantic::BG_TERTIARY, spacing::XS
                    ),
                }
                div {
                    style: format!(
                        "width: 140px; height: 14px; border-radius: {}; background: {};",
                        radius::SM, semantic::BG_TERTIARY
                    ),
                }
            }
        }
    }
}

/// Empty state when no messages exist.
#[component]
fn MessageEmptyState() -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; flex-direction: column; align-items: center; \
                 justify-content: center; height: 100%; padding: {}; text-align: center;",
                spacing::HUGE
            ),
            div {
                style: format!(
                    "width: 80px; height: 80px; display: flex; align-items: center; \
                     justify-content: center; background: {}; border-radius: {}; \
                     font-size: {}; margin-bottom: {};",
                    semantic::BG_TERTIARY, radius::XXL, typography::SIZE_4XL, spacing::XL
                ),
                "💬"
            }
            div {
                style: format!(
                    "font-size: {}; font-weight: {}; color: {}; margin-bottom: {};",
                    typography::SIZE_LG, typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_PRIMARY, spacing::XS
                ),
                "No messages yet"
            }
            div {
                style: format!(
                    "font-size: {}; color: {};",
                    typography::SIZE_SM, semantic::TEXT_MUTED
                ),
                "Start the conversation by sending a message below"
            }
        }
    }
}

/// Individual message bubble.
#[component]
pub fn MessageBubble(
    message: MessageDisplay,
    #[props(default = false)] show_avatar: bool,
    /// The entity/thread ID this message belongs to (used for reaction backend calls).
    #[props(default)]
    thread_id: String,
    on_reply: EventHandler<String>,
    on_react: EventHandler<String>,
    /// Called when the user saves an edit. Payload is (message_id, new_text).
    #[props(default)]
    on_edit: Option<EventHandler<(String, String)>>,
    /// Called when the user deletes a message. Payload is the message_id.
    #[props(default)]
    on_delete: Option<EventHandler<String>>,
    /// Called when user clicks the inline quote to scroll to the original message.
    /// Payload is the original message ID.
    #[props(default)]
    on_scroll_to: Option<EventHandler<String>>,
    /// Called when the user pins or unpins this message.
    /// Payload is the message_id. The parent decides whether to pin or unpin
    /// based on the current `message.is_pinned` state.
    #[props(default)]
    on_pin: Option<EventHandler<String>>,
) -> Element {
    let mut hovered = use_signal(|| false);
    let mut show_actions = use_signal(|| false);
    let mut editing = use_signal(|| false);
    let mut edit_text = use_signal(String::new);
    let mut show_quick_reactions = use_signal(|| false);
    let mut show_full_picker = use_signal(|| false);

    // Optional services context — may not be available in all render contexts (e.g. tests).
    let services: Option<Arc<UiServices>> = use_context();

    // Clone message.id for use in multiple closures
    let message_id_for_reply = message.id.clone();
    let message_id_for_react = message.id.clone();
    let message_id_for_thread = message.id.clone();
    let message_id_for_edit_save = message.id.clone();
    let message_id_for_delete = message.id.clone();
    let message_id_for_pin = message.id.clone();
    let message_content_for_edit = message.content.clone();
    let message_is_pinned = message.is_pinned;

    let initials = message
        .author_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    let bubble_align = if message.is_own {
        "flex-end"
    } else {
        "flex-start"
    };
    let bubble_bg = if message.is_own {
        format!(
            "background: linear-gradient(135deg, {} 0%, {} 100%);",
            palette::JADE_600,
            palette::JADE_700
        )
    } else {
        format!("background: {};", semantic::BG_TERTIARY)
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 flex-direction: column; \
                 align-items: {}; \
                 gap: {};",
                bubble_align,
                spacing::XS
            ),

            // Message row
            div {
                style: format!(
                    "display: flex; \
                     align-items: flex-end; \
                     gap: {}; \
                     max-width: 70%; \
                     flex-direction: {};",
                    spacing::SM,
                    if message.is_own { "row-reverse" } else { "row" }
                ),
                onmouseenter: move |_| {
                    hovered.set(true);
                    show_actions.set(true);
                },
                onmouseleave: move |_| {
                    hovered.set(false);
                    show_actions.set(false);
                },

                // Avatar (for others' messages)
                if show_avatar && !message.is_own {
                    div {
                        style: format!(
                            "{} \
                             background: linear-gradient(135deg, {} 0%, {} 100%);",
                            avatar::sm(),
                            palette::FOREST_600,
                            palette::FOREST_700
                        ),
                        span {
                            style: format!("color: {}; font-size: {};", semantic::TEXT_PRIMARY, typography::SIZE_XS),
                            "{initials}"
                        }
                    }
                } else if show_avatar {
                    // Spacer for alignment when no avatar
                    div {
                        style: "width: 28px; flex-shrink: 0;",
                    }
                }

                // Message content
                div {
                    style: format!(
                        "position: relative; \
                         {} \
                         padding: {} {}; \
                         border-radius: {}; \
                         {}",
                        bubble_bg,
                        spacing::MD,
                        spacing::BASE,
                        radius::XL,
                        if message.is_own {
                            format!("border-bottom-right-radius: {};", radius::SM)
                        } else {
                            format!("border-bottom-left-radius: {};", radius::SM)
                        }
                    ),

                    // Reply context (if this message is a reply)
                    if let Some(ref replied_to) = message.replied_to {
                        {
                            let replied_to_id = replied_to.id.clone();
                            rsx! {
                                div {
                                    style: format!(
                                        "display: flex; \
                                         flex-direction: column; \
                                         gap: {}; \
                                         padding: {}; \
                                         margin-bottom: {}; \
                                         background: {}; \
                                         border-left: 2px solid {}; \
                                         border-radius: 0 {} {} 0; \
                                         cursor: pointer; \
                                         transition: {};",
                                        spacing::XXS,
                                        spacing::SM,
                                        spacing::SM,
                                        if message.is_own {
                                            "rgba(0,0,0,0.15)"
                                        } else {
                                            semantic::BG_SECONDARY
                                        },
                                        palette::JADE_500,
                                        radius::SM,
                                        radius::SM,
                                        motion::transition("background")
                                    ),
                                    role: "button",
                                    aria_label: format!("Jump to original message from {}", replied_to.author_name),
                                    onclick: move |_| {
                                        if let Some(ref handler) = on_scroll_to {
                                            handler.call(replied_to_id.clone());
                                        }
                                    },

                                    span {
                                        style: format!(
                                            "font-size: {}; \
                                             font-weight: {}; \
                                             color: {};",
                                            typography::SIZE_XS,
                                            typography::WEIGHT_SEMIBOLD,
                                            palette::JADE_400
                                        ),
                                        "{replied_to.author_name}"
                                    }

                                    p {
                                        style: format!(
                                            "margin: 0; \
                                             font-size: {}; \
                                             color: {}; \
                                             overflow: hidden; \
                                             text-overflow: ellipsis; \
                                             white-space: nowrap; \
                                             max-width: 200px;",
                                            typography::SIZE_XS,
                                            if message.is_own { "rgba(255,255,255,0.7)" } else { semantic::TEXT_MUTED }
                                        ),
                                        "{replied_to.content}"
                                    }
                                }
                            }
                        }
                    }

                    // Author name (for group messages)
                    if !message.is_own && show_avatar {
                        div {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {}; \
                                 margin-bottom: {};",
                                typography::SIZE_XS,
                                typography::WEIGHT_SEMIBOLD,
                                palette::JADE_400,
                                spacing::XS
                            ),
                            "{message.author_name}"
                        }
                    }

                    // Message text (or inline edit textarea)
                    if editing() {
                        div {
                            style: "display: flex; flex-direction: column; gap: 6px; width: 100%;",

                            textarea {
                                value: "{edit_text}",
                                rows: "3",
                                aria_label: "Edit message",
                                style: format!(
                                    "width: 100%; \
                                     background: {}; \
                                     border: 1px solid {}; \
                                     border-radius: {}; \
                                     outline: none; \
                                     resize: vertical; \
                                     color: {}; \
                                     font-family: {}; \
                                     font-size: {}; \
                                     line-height: {}; \
                                     padding: {};",
                                    semantic::BG_SECONDARY,
                                    semantic::BORDER_DEFAULT,
                                    radius::MD,
                                    semantic::TEXT_PRIMARY,
                                    typography::FONT_BODY,
                                    typography::SIZE_BASE,
                                    typography::LEADING_RELAXED,
                                    spacing::SM
                                ),
                                oninput: move |evt: FormEvent| {
                                    edit_text.set(evt.value().clone());
                                },
                                onkeydown: move |evt: KeyboardEvent| {
                                    if evt.key() == Key::Escape {
                                        editing.set(false);
                                    }
                                },
                            }

                            div {
                                style: format!(
                                    "display: flex; gap: {}; justify-content: flex-end;",
                                    spacing::XS
                                ),

                                // Cancel button
                                button {
                                    style: format!(
                                        "padding: {} {}; \
                                         background: transparent; \
                                         border: 1px solid {}; \
                                         border-radius: {}; \
                                         color: {}; \
                                         font-size: {}; \
                                         cursor: pointer; \
                                         transition: {};",
                                        spacing::XXS,
                                        spacing::SM,
                                        semantic::BORDER_DEFAULT,
                                        radius::MD,
                                        semantic::TEXT_SECONDARY,
                                        typography::SIZE_XS,
                                        motion::transition("background")
                                    ),
                                    aria_label: "Cancel editing",
                                    onclick: move |_| {
                                        editing.set(false);
                                    },
                                    "Cancel"
                                }

                                // Save button
                                button {
                                    style: format!(
                                        "padding: {} {}; \
                                         background: {}; \
                                         border: none; \
                                         border-radius: {}; \
                                         color: white; \
                                         font-size: {}; \
                                         font-weight: {}; \
                                         cursor: pointer; \
                                         transition: {};",
                                        spacing::XXS,
                                        spacing::SM,
                                        semantic::PRIMARY,
                                        radius::MD,
                                        typography::SIZE_XS,
                                        typography::WEIGHT_MEDIUM,
                                        motion::transition("background")
                                    ),
                                    aria_label: "Save edit",
                                    onclick: {
                                        let id = message_id_for_edit_save.clone();
                                        move |_| {
                                            let new_text = edit_text().trim().to_string();
                                            if !new_text.is_empty()
                                                && let Some(ref handler) = on_edit
                                            {
                                                handler.call((id.clone(), new_text));
                                            }
                                            editing.set(false);
                                        }
                                    },
                                    "Save"
                                }
                            }
                        }
                    } else {
                        MarkdownContent {
                            content: message.content.clone(),
                            is_own: message.is_own,
                        }
                    }

                    // Timestamp and edited indicator
                    div {
                        style: format!(
                            "display: flex; \
                             align-items: center; \
                             gap: {}; \
                             margin-top: {}; \
                             font-size: {}; \
                             color: {};",
                            spacing::XS,
                            spacing::XS,
                            typography::SIZE_XS,
                            if message.is_own { "rgba(255,255,255,0.7)" } else { semantic::TEXT_MUTED }
                        ),

                        if message.is_edited {
                            span { "(edited)" }
                        }

                        span { "{message.timestamp}" }
                    }
                }

                // Action buttons (on hover)
                if show_actions() {
                    div {
                        style: format!(
                            "display: flex; \
                             align-items: center; \
                             gap: {}; \
                             padding: {}; \
                             background: {}; \
                             border-radius: {}; \
                             box-shadow: {}; \
                             animation: fadeIn 100ms ease-out;",
                            spacing::XXS,
                            spacing::XXS,
                            semantic::BG_ELEVATED,
                            radius::MD,
                            shadow::MD
                        ),

                        MessageActionButton {
                            icon: "💬".to_string(),
                            tooltip: "Reply".to_string(),
                            onclick: move |_| on_reply.call(message_id_for_reply.clone()),
                        }

                        MessageActionButton {
                            icon: "😊".to_string(),
                            tooltip: "React".to_string(),
                            onclick: move |_| {
                                // Toggle quick-reaction bar; also notify parent via on_react
                                let next = !show_quick_reactions();
                                show_quick_reactions.set(next);
                                if next {
                                    on_react.call(message_id_for_react.clone());
                                }
                            },
                        }

                        // Edit button (own messages only)
                        if message.is_own && on_edit.is_some() {
                            MessageActionButton {
                                icon: "\u{270F}\u{FE0F}".to_string(),
                                tooltip: "Edit".to_string(),
                                onclick: {
                                    let content = message_content_for_edit.clone();
                                    move |_| {
                                        edit_text.set(content.clone());
                                        editing.set(true);
                                    }
                                },
                            }
                        }

                        // Delete button (own messages only)
                        if message.is_own && on_delete.is_some() {
                            MessageActionButton {
                                icon: "\u{1F5D1}\u{FE0F}".to_string(),
                                tooltip: "Delete".to_string(),
                                onclick: {
                                    let id = message_id_for_delete.clone();
                                    move |_| {
                                        if let Some(ref handler) = on_delete {
                                            handler.call(id.clone());
                                        }
                                    }
                                },
                            }
                        }

                        // Pin / Unpin button
                        if on_pin.is_some() {
                            MessageActionButton {
                                icon: if message_is_pinned { "\u{1F4CC}" } else { "\u{1F4CD}" },
                                tooltip: if message_is_pinned { "Unpin".to_string() } else { "Pin".to_string() },
                                onclick: {
                                    let id = message_id_for_pin.clone();
                                    move |_| {
                                        if let Some(ref handler) = on_pin {
                                            handler.call(id.clone());
                                        }
                                    }
                                },
                            }
                        }

                        // More button (for non-own messages or when no edit/delete)
                        if !message.is_own || (on_edit.is_none() && on_delete.is_none()) {
                            MessageActionButton {
                                icon: "\u{22EF}".to_string(),
                                tooltip: "More".to_string(),
                                onclick: move |_| {},
                            }
                        }
                    }
                }
            }

            // Quick-reaction bar and full picker (shown when react is active)
            if show_quick_reactions() || show_full_picker() {
                div {
                    style: format!(
                        "position: relative; \
                         display: flex; \
                         justify-content: {}; \
                         margin-left: {};",
                        if message.is_own { "flex-end" } else { "flex-start" },
                        if message.is_own { "0" } else { "36px" }
                    ),

                    if show_quick_reactions() && !show_full_picker() {
                        QuickReactionBar {
                            on_select: move |emoji: String| {
                                on_react.call(emoji);
                                show_quick_reactions.set(false);
                            },
                            on_more: move |_| {
                                show_quick_reactions.set(false);
                                show_full_picker.set(true);
                            },
                        }
                    }

                    if show_full_picker() {
                        EmojiPicker {
                            on_select: move |emoji: String| {
                                on_react.call(emoji);
                                show_full_picker.set(false);
                                show_quick_reactions.set(false);
                            },
                            on_close: move |_| {
                                show_full_picker.set(false);
                                show_quick_reactions.set(false);
                            },
                        }
                    }
                }
            }

            // Reactions
            if !message.reactions.is_empty() {
                div {
                    style: format!(
                        "display: flex; \
                         gap: {}; \
                         margin-left: {}; \
                         flex-wrap: wrap;",
                        spacing::XS,
                        if message.is_own { "0" } else { "36px" }
                    ),

                    for reaction in message.reactions.iter() {
                        {
                            let emoji = reaction.emoji.clone();
                            let has_reacted = reaction.has_reacted;
                            let msg_id = message.id.clone();
                            let tid = thread_id.clone();
                            let svc = services.clone();
                            rsx! {
                                ReactionChip {
                                    key: "{emoji}",
                                    emoji: emoji.clone(),
                                    count: reaction.count,
                                    has_reacted,
                                    onclick: move |_| {
                                        let emoji = emoji.clone();
                                        let msg_id = msg_id.clone();
                                        let tid = tid.clone();
                                        if let Some(svc) = svc.clone() {
                                            spawn(async move {
                                                let result = if has_reacted {
                                                    svc.messaging().remove_reaction(&tid, &msg_id, &emoji).await
                                                } else {
                                                    svc.messaging().add_reaction(&tid, &msg_id, &emoji).await
                                                };
                                                if let Err(e) = result {
                                                    tracing::warn!(
                                                        error = %e,
                                                        thread_id = %tid,
                                                        message_id = %msg_id,
                                                        emoji = %emoji,
                                                        "Reaction toggle failed"
                                                    );
                                                }
                                            });
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }

            // Reply count indicator
            if message.reply_count > 0 {
                button {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         margin-left: {}; \
                         padding: {} {}; \
                         background: transparent; \
                         border: none; \
                         color: {}; \
                         font-size: {}; \
                         cursor: pointer; \
                         transition: {};",
                        spacing::XS,
                        if message.is_own { "0" } else { "36px" },
                        spacing::XXS,
                        spacing::SM,
                        semantic::PRIMARY,
                        typography::SIZE_XS,
                        motion::transition("color")
                    ),
                    aria_label: format!("View {} repl{} in thread", message.reply_count, if message.reply_count == 1 { "y" } else { "ies" }),
                    onclick: move |_| on_reply.call(message_id_for_thread.clone()),

                    span { aria_hidden: "true", "💬" }
                    span { "{message.reply_count} replies" }
                }
            }
        }

        style {
            r#"
            @keyframes fadeIn {{
                from {{ opacity: 0; transform: scale(0.95); }}
                to {{ opacity: 1; transform: scale(1); }}
            }}
            "#
        }
    }
}

/// Small action button for message hover.
#[component]
fn MessageActionButton(
    icon: String,
    tooltip: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        button {
            style: format!(
                "width: 28px; \
                 height: 28px; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: {}; \
                 border: none; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {};",
                if hovered() { semantic::BG_HOVER } else { "transparent" },
                radius::MD,
                motion::transition("background")
            ),
            aria_label: "{tooltip}",
            title: "{tooltip}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),
            span { aria_hidden: "true", "{icon}" }
        }
    }
}

/// Reaction chip with count.
#[component]
pub fn ReactionChip(
    emoji: String,
    count: u32,
    has_reacted: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let mut hovered = use_signal(|| false);

    let bg = if has_reacted {
        format!("{}20", semantic::PRIMARY)
    } else if hovered() {
        semantic::BG_HOVER.to_string()
    } else {
        semantic::BG_TERTIARY.to_string()
    };

    let border = if has_reacted {
        semantic::PRIMARY
    } else {
        semantic::BORDER_SUBTLE
    };

    let reaction_label = format!(
        "{} reaction{}, {} {}",
        count,
        if count == 1 { "" } else { "s" },
        emoji,
        if has_reacted { "- you reacted" } else { "" }
    );

    rsx! {
        button {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {};",
                spacing::XXS,
                spacing::XXS,
                spacing::SM,
                bg,
                border,
                radius::FULL,
                motion::transition("all")
            ),
            aria_label: "{reaction_label}",
            aria_pressed: if has_reacted { "true" } else { "false" },
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            span {
                style: format!("font-size: {};", typography::SIZE_SM),
                aria_hidden: "true",
                "{emoji}"
            }

            span {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {};",
                    typography::SIZE_XS,
                    typography::WEIGHT_MEDIUM,
                    if has_reacted { semantic::PRIMARY } else { semantic::TEXT_SECONDARY }
                ),
                "{count}"
            }
        }
    }
}

/// Date separator for message groups.
#[component]
pub fn DateSeparator(date: String) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} 0;",
                spacing::MD,
                spacing::MD
            ),

            div {
                style: format!(
                    "flex: 1; \
                     height: 1px; \
                     background: {};",
                    semantic::BORDER_SUBTLE
                ),
            }

            span {
                style: format!(
                    "font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     text-transform: uppercase; \
                     letter-spacing: {};",
                    typography::SIZE_XS,
                    typography::WEIGHT_MEDIUM,
                    semantic::TEXT_MUTED,
                    typography::TRACKING_WIDE
                ),
                "{date}"
            }

            div {
                style: format!(
                    "flex: 1; \
                     height: 1px; \
                     background: {};",
                    semantic::BORDER_SUBTLE
                ),
            }
        }
    }
}

/// Rich message composer with @mention autocomplete support.
#[component]
pub fn MessageComposerV2(
    value: String,
    placeholder: String,
    #[props(default = false)] disabled: bool,
    oninput: EventHandler<FormEvent>,
    onsubmit: EventHandler<()>,
    /// Called when the user inserts an emoji from the picker (payload = emoji string).
    /// The parent should append the emoji to the current message value.
    #[props(default)]
    on_emoji_insert: Option<EventHandler<String>>,
    /// Optional reply context shown as a preview bar above the input.
    #[props(default)]
    reply_to: Option<RepliedToDisplay>,
    /// Called when the user cancels the reply by clicking the X.
    #[props(default)]
    on_cancel_reply: Option<EventHandler<()>>,
    /// Contacts available for @mention autocomplete.
    #[props(default)]
    mention_candidates: Vec<crate::components::mention::MentionCandidate>,
    /// Called when the user selects a mention candidate from the dropdown.
    /// The parent should insert `@display_name ` at the current `@query` position.
    #[props(default)]
    on_mention_select: Option<EventHandler<crate::components::mention::MentionCandidate>>,
) -> Element {
    let mut focused = use_signal(|| false);
    let mut show_emoji_picker = use_signal(|| false);
    // Active @-mention query (text after `@`, None = picker closed).
    let mut mention_query: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            style: format!(
                "padding: {}; \
                 border-top: 1px solid {}; \
                 background: {};",
                spacing::BASE,
                semantic::BORDER_SUBTLE,
                semantic::BG_SECONDARY
            ),

            // Reply preview bar (shown when replying to a message)
            if let Some(ref reply) = reply_to {
                div {
                    style: format!(
                        "display: flex; \
                         align-items: center; \
                         gap: {}; \
                         padding: {} {}; \
                         margin-bottom: {}; \
                         background: {}; \
                         border-left: 2px solid {}; \
                         border-radius: 0 {} {} 0;",
                        spacing::SM,
                        spacing::XS,
                        spacing::SM,
                        spacing::SM,
                        semantic::BG_TERTIARY,
                        palette::JADE_500,
                        radius::SM,
                        radius::SM
                    ),
                    aria_label: "Replying to message",

                    // Reply icon
                    span {
                        style: format!("color: {}; font-size: {};", palette::JADE_400, typography::SIZE_BASE),
                        aria_hidden: "true",
                        "↩"
                    }

                    // Reply content
                    div {
                        style: "flex: 1; min-width: 0;",

                        span {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {}; \
                                 display: block;",
                                typography::SIZE_XS,
                                typography::WEIGHT_SEMIBOLD,
                                palette::JADE_400
                            ),
                            "Replying to {reply.author_name}"
                        }

                        span {
                            style: format!(
                                "font-size: {}; \
                                 color: {}; \
                                 overflow: hidden; \
                                 text-overflow: ellipsis; \
                                 white-space: nowrap; \
                                 display: block;",
                                typography::SIZE_XS,
                                semantic::TEXT_MUTED
                            ),
                            "{reply.content}"
                        }
                    }

                    // Cancel button
                    button {
                        style: format!(
                            "width: 20px; \
                             height: 20px; \
                             display: flex; \
                             align-items: center; \
                             justify-content: center; \
                             background: transparent; \
                             border: none; \
                             color: {}; \
                             cursor: pointer; \
                             border-radius: {}; \
                             flex-shrink: 0;",
                            semantic::TEXT_MUTED,
                            radius::FULL
                        ),
                        aria_label: "Cancel reply",
                        title: "Cancel reply",
                        onclick: move |_| {
                            if let Some(ref handler) = on_cancel_reply {
                                handler.call(());
                            }
                        },
                        "✕"
                    }
                }
            }

            // Composer area — positioned so the mention dropdown can appear above it.
            div {
                style: "position: relative;",

                // @mention autocomplete dropdown (shown above the textarea)
                if let Some(ref query) = mention_query() {
                    crate::components::mention::MentionAutocomplete {
                        candidates: mention_candidates.clone(),
                        query: query.clone(),
                        on_select: {
                            let handler = on_mention_select;
                            move |candidate: crate::components::mention::MentionCandidate| {
                                if let Some(ref h) = handler {
                                    h.call(candidate);
                                }
                                mention_query.set(None);
                            }
                        },
                        on_dismiss: move |_| mention_query.set(None),
                    }
                }

                // Main composer row
                div {
                    style: format!(
                        "display: flex; \
                         align-items: flex-end; \
                         gap: {}; \
                         padding: {}; \
                         background: {}; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         transition: {};",
                        spacing::SM,
                        spacing::SM,
                        semantic::BG_TERTIARY,
                        if focused() { semantic::PRIMARY } else { semantic::BORDER_SUBTLE },
                        radius::XL,
                        motion::transition("border-color")
                    ),

                    // Attachment button
                    ComposerButton {
                        icon: "📎".to_string(),
                        tooltip: "Attach file".to_string(),
                        onclick: move |_| {},
                    }

                    // Text input
                    textarea {
                        placeholder: "{placeholder}",
                        value: "{value}",
                        disabled: disabled,
                        rows: "1",
                        aria_label: "Message input. Press Enter to send, Shift+Enter for new line. Type @ to mention someone.",
                        style: format!(
                            "flex: 1; \
                             background: transparent; \
                             border: none; \
                             outline: none; \
                             resize: none; \
                             color: {}; \
                             font-family: {}; \
                             font-size: {}; \
                             line-height: {}; \
                             min-height: 24px; \
                             max-height: 120px; \
                             overflow-y: auto;",
                            semantic::TEXT_PRIMARY,
                            typography::FONT_BODY,
                            typography::SIZE_BASE,
                            typography::LEADING_NORMAL
                        ),
                        onfocus: move |_| focused.set(true),
                        onblur: move |_| focused.set(false),
                        oninput: move |evt: FormEvent| {
                            // Detect @-mention query from current text.
                            // Find the last `@` that is either at the start or preceded by whitespace.
                            let text = evt.value();
                            let query = detect_mention_query(&text);
                            mention_query.set(query);
                            oninput.call(evt);
                        },
                        onkeydown: move |evt: KeyboardEvent| {
                            // When mention picker is open, let it handle nav keys.
                            // The picker's own onkeydown handles ArrowUp/Down/Enter/Escape.
                            // We still need to close the picker on Escape here as a fallback,
                            // and suppress Enter submission while picker is open.
                            if mention_query().is_some() {
                                match evt.key() {
                                    Key::Escape => {
                                        evt.prevent_default();
                                        mention_query.set(None);
                                        return;
                                    }
                                    Key::Enter if !evt.modifiers().shift() => {
                                        // Don't submit message while mention picker is open;
                                        // the picker's Enter handler selects the candidate.
                                        evt.prevent_default();
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                            if evt.key() == Key::Enter && !evt.modifiers().shift() {
                                evt.prevent_default();
                                onsubmit.call(());
                            }
                        },
                    }

                    // Emoji button — opens the emoji picker above the composer
                    ComposerButton {
                        icon: "😊".to_string(),
                        tooltip: "Emoji".to_string(),
                        onclick: move |_| show_emoji_picker.set(!show_emoji_picker()),
                    }

                    // Send button
                    SendButton {
                        disabled: disabled || value.trim().is_empty(),
                        onclick: move |_| onsubmit.call(()),
                    }
                }
            }

            // Emoji picker (appears above composer bar when emoji button is clicked)
            if show_emoji_picker() {
                div {
                    style: "position: relative;",
                    EmojiPicker {
                        on_select: move |emoji: String| {
                            if let Some(ref handler) = on_emoji_insert {
                                handler.call(emoji);
                            }
                            show_emoji_picker.set(false);
                        },
                        on_close: move |_| show_emoji_picker.set(false),
                    }
                }
            }

            // Composer toolbar (formatting, mentions, etc.)
            div {
                style: format!(
                    "display: flex; \
                     align-items: center; \
                     gap: {}; \
                     margin-top: {}; \
                     padding: 0 {};",
                    spacing::SM,
                    spacing::XS,
                    spacing::SM
                ),

                span {
                    style: format!(
                        "font-size: {}; \
                         color: {};",
                        typography::SIZE_XS,
                        semantic::TEXT_MUTED
                    ),
                    "Press Enter to send, Shift+Enter for new line, @ to mention"
                }
            }
        }
    }
}

/// Detect an active @mention query from the composer text.
///
/// Returns `Some(query)` when the caret is within a `@word` token that starts
/// at position 0 or immediately after whitespace.  Returns `None` otherwise.
///
/// The query is the text after `@` up to the end of the string (we operate on
/// the full text; a real caret-position-aware version would require JS interop,
/// so we use the last `@`-prefixed word as a practical heuristic).
pub fn detect_mention_query(text: &str) -> Option<String> {
    // Find the last `@` in the text.
    let at_pos = text.rfind('@')?;

    // Verify the `@` is either at the start or preceded by whitespace.
    if at_pos > 0 {
        let prev = text[..at_pos].chars().next_back()?;
        if !prev.is_whitespace() {
            return None;
        }
    }

    // Extract the word after `@`.
    let after = &text[at_pos + 1..];

    // If there is a space after `@word`, the mention has been committed.
    if let Some(space_pos) = after.find(|c: char| c.is_whitespace()) {
        let _ = space_pos;
        return None;
    }

    // Return the partial word (may be empty string when user just typed `@`).
    Some(after.to_string())
}

/// Composer toolbar button.
#[component]
fn ComposerButton(icon: String, tooltip: String, onclick: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);

    rsx! {
        button {
            style: format!(
                "width: 32px; \
                 height: 32px; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: {}; \
                 border: none; \
                 border-radius: {}; \
                 cursor: pointer; \
                 transition: {}; \
                 flex-shrink: 0;",
                if hovered() { semantic::BG_HOVER } else { "transparent" },
                radius::MD,
                motion::transition("background")
            ),
            aria_label: "{tooltip}",
            title: "{tooltip}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),
            span { aria_hidden: "true", "{icon}" }
        }
    }
}

/// Send button with primary styling.
#[component]
fn SendButton(disabled: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let mut hovered = use_signal(|| false);

    let bg = if disabled {
        semantic::BG_ELEVATED.to_string()
    } else if hovered() {
        "linear-gradient(135deg, #34d399 0%, #10b981 100%)".to_string()
    } else {
        "linear-gradient(135deg, #10b981 0%, #059669 100%)".to_string()
    };

    rsx! {
        button {
            style: format!(
                "width: 36px; \
                 height: 36px; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: {}; \
                 border: none; \
                 border-radius: {}; \
                 cursor: {}; \
                 transition: {}; \
                 flex-shrink: 0; \
                 {}",
                bg,
                radius::FULL,
                if disabled { "not-allowed" } else { "pointer" },
                motion::transition("all"),
                if disabled { "opacity: 0.5;" } else { "" }
            ),
            aria_label: "Send message",
            disabled: disabled,
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            span {
                style: format!(
                    "color: {}; \
                     font-size: {};",
                    if disabled { semantic::TEXT_MUTED } else { "white" },
                    typography::SIZE_BASE
                ),
                aria_hidden: "true",
                "➤"
            }
        }
    }
}

/// Typing indicator.
#[component]
pub fn TypingIndicatorV2(names: Vec<String>) -> Element {
    if names.is_empty() {
        return rsx! { Fragment {} };
    }

    let text = match names.len() {
        1 => format!("{} is typing", names[0]),
        2 => format!("{} and {} are typing", names[0], names[1]),
        _ => format!("{} and {} others are typing", names[0], names.len() - 1),
    };

    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 font-size: {}; \
                 color: {};",
                spacing::SM,
                spacing::SM,
                spacing::XL,
                typography::SIZE_XS,
                semantic::TEXT_MUTED
            ),

            // Animated dots
            div {
                style: format!(
                    "display: flex; \
                     gap: {};",
                    spacing::XXS
                ),

                for i in 0..3 {
                    div {
                        style: format!(
                            "width: 6px; \
                             height: 6px; \
                             background: {}; \
                             border-radius: {}; \
                             animation: typingDot 1.4s ease-in-out infinite; \
                             animation-delay: {}ms;",
                            semantic::PRIMARY,
                            radius::FULL,
                            i * 200
                        ),
                    }
                }
            }

            span { "{text}" }
        }

        style {
            r#"
            @keyframes typingDot {{
                0%, 60%, 100% {{ transform: translateY(0); opacity: 0.3; }}
                30% {{ transform: translateY(-4px); opacity: 1; }}
            }}
            "#
        }
    }
}

/// New message indicator.
#[component]
pub fn NewMessageIndicator(count: u32) -> Element {
    if count == 0 {
        return rsx! { Fragment {} };
    }

    let message_text = if count == 1 {
        format!("{} new message", count)
    } else {
        format!("{} new messages", count)
    };

    rsx! {
        button {
            style: format!(
                "position: fixed; \
                 bottom: {}; \
                 left: 50%; \
                 transform: translateX(-50%); \
                 display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 background: {}; \
                 border: none; \
                 border-radius: {}; \
                 color: white; \
                 font-size: {}; \
                 font-weight: {}; \
                 cursor: pointer; \
                 box-shadow: {}; \
                 animation: slideUp 200ms ease-out;",
                spacing::XXXL,
                spacing::SM,
                spacing::SM,
                spacing::BASE,
                semantic::PRIMARY,
                radius::FULL,
                typography::SIZE_SM,
                typography::WEIGHT_MEDIUM,
                shadow::LG
            ),
            aria_label: format!("Scroll to {}. Click to jump to latest messages.", message_text),

            span { aria_hidden: "true", "↓" }
            span { "{message_text}" }
        }

        style {
            r#"
            @keyframes slideUp {{
                from {{ opacity: 0; transform: translateX(-50%) translateY(20px); }}
                to {{ opacity: 1; transform: translateX(-50%) translateY(0); }}
            }}
            "#
        }
    }
}

/// Apply a mention selection to the composer text.
///
/// Finds the last `@query` fragment (as detected by [`detect_mention_query`]) and
/// replaces it with `@display_name ` (with a trailing space so the user can keep typing).
///
/// If no active mention fragment is found, appends `@display_name ` to the end.
pub fn apply_mention_selection(text: &str, display_name: &str) -> String {
    if let Some(at_pos) = text.rfind('@') {
        // Verify the `@` is at start or after whitespace.
        let valid = at_pos == 0
            || text[..at_pos]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_whitespace());
        if valid {
            let after = &text[at_pos + 1..];
            // Only replace if there's no space after `@` (i.e. query is still in-flight).
            if !after.contains(|c: char| c.is_whitespace()) {
                let prefix = &text[..at_pos];
                return format!("{prefix}@{display_name} ");
            }
        }
    }
    // Fallback: append mention at end.
    if text.ends_with(' ') || text.is_empty() {
        format!("{text}@{display_name} ")
    } else {
        format!("{text} @{display_name} ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_mention_at_start() {
        assert_eq!(detect_mention_query("@ali"), Some("ali".to_string()));
    }

    #[test]
    fn detect_mention_after_space() {
        assert_eq!(detect_mention_query("Hello @ali"), Some("ali".to_string()));
    }

    #[test]
    fn detect_mention_no_at() {
        assert_eq!(detect_mention_query("Hello world"), None);
    }

    #[test]
    fn detect_mention_completed_space_after() {
        // `@alice ` — the trailing space means the mention was committed.
        assert_eq!(detect_mention_query("@alice "), None);
    }

    #[test]
    fn detect_mention_mid_word_invalid() {
        // `foo@bar` — `@` not at start or after whitespace.
        assert_eq!(detect_mention_query("foo@bar"), None);
    }

    #[test]
    fn detect_mention_empty_query() {
        // Just `@` with nothing after.
        assert_eq!(detect_mention_query("@"), Some("".to_string()));
    }

    #[test]
    fn apply_mention_selection_basic() {
        let result = apply_mention_selection("Hello @ali", "alice");
        assert_eq!(result, "Hello @alice ");
    }

    #[test]
    fn apply_mention_selection_at_start() {
        let result = apply_mention_selection("@ali", "alice");
        assert_eq!(result, "@alice ");
    }

    #[test]
    fn apply_mention_selection_empty_query() {
        let result = apply_mention_selection("@", "alice");
        assert_eq!(result, "@alice ");
    }

    #[test]
    fn apply_mention_selection_fallback_append() {
        // No active @query — appends at end.
        let result = apply_mention_selection("Hello", "alice");
        assert_eq!(result, "Hello @alice ");
    }

    #[test]
    fn apply_mention_selection_already_ends_with_space() {
        let result = apply_mention_selection("Hello ", "alice");
        assert_eq!(result, "Hello @alice ");
    }
}

//! Enhanced messaging components with Digital Forest Sanctuary theme.
//!
//! Features:
//! - Message bubbles with glass effect
//! - Thread support
//! - Reactions
//! - Rich composer
//! - Typing indicators

use dioxus::prelude::*;
use crate::design_tokens::{motion, palette, radius, semantic, shadow, spacing, typography};
use crate::styles_v2::avatar;

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
pub fn ChatView(
    children: Element,
) -> Element {
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

/// Message list container with auto-scroll.
#[component]
pub fn MessageListContainer(
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
            {children}
        }
    }
}

/// Individual message bubble.
#[component]
pub fn MessageBubble(
    message: MessageDisplay,
    #[props(default = false)]
    show_avatar: bool,
    on_reply: EventHandler<String>,
    on_react: EventHandler<String>,
) -> Element {
    let mut hovered = use_signal(|| false);
    let mut show_actions = use_signal(|| false);

    // Clone message.id for use in multiple closures
    let message_id_for_reply = message.id.clone();
    let message_id_for_react = message.id.clone();
    let message_id_for_thread = message.id.clone();

    let initials = message.author_name
        .split_whitespace()
        .take(2)
        .filter_map(|w| w.chars().next())
        .collect::<String>()
        .to_uppercase();

    let bubble_align = if message.is_own { "flex-end" } else { "flex-start" };
    let bubble_bg = if message.is_own {
        format!("background: linear-gradient(135deg, {} 0%, {} 100%);", palette::JADE_600, palette::JADE_700)
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

                    // Message text
                    p {
                        style: format!(
                            "margin: 0; \
                             font-size: {}; \
                             line-height: {}; \
                             color: {}; \
                             word-wrap: break-word;",
                            typography::SIZE_BASE,
                            typography::LEADING_RELAXED,
                            if message.is_own { "white" } else { semantic::TEXT_PRIMARY }
                        ),
                        "{message.content}"
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
                            onclick: move |_| on_react.call(message_id_for_react.clone()),
                        }

                        MessageActionButton {
                            icon: "⋯".to_string(),
                            tooltip: "More".to_string(),
                            onclick: move |_| {},
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
                        ReactionChip {
                            emoji: reaction.emoji.clone(),
                            count: reaction.count,
                            has_reacted: reaction.has_reacted,
                            onclick: move |_| {},
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
                    onclick: move |_| on_reply.call(message_id_for_thread.clone()),

                    span { "💬" }
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
            title: "{tooltip}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),
            "{icon}"
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
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),

            span {
                style: format!("font-size: {};", typography::SIZE_SM),
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

/// Rich message composer.
#[component]
pub fn MessageComposerV2(
    value: String,
    placeholder: String,
    #[props(default = false)]
    disabled: bool,
    oninput: EventHandler<FormEvent>,
    onsubmit: EventHandler<()>,
) -> Element {
    let mut focused = use_signal(|| false);

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

            // Main composer container
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
                    oninput: move |evt| oninput.call(evt),
                    onkeydown: move |evt| {
                        if evt.key() == Key::Enter && !evt.modifiers().shift() {
                            evt.prevent_default();
                            onsubmit.call(());
                        }
                    },
                }

                // Emoji button
                ComposerButton {
                    icon: "😊".to_string(),
                    tooltip: "Emoji".to_string(),
                    onclick: move |_| {},
                }

                // Send button
                SendButton {
                    disabled: disabled || value.trim().is_empty(),
                    onclick: move |_| onsubmit.call(()),
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
                    "Press Enter to send, Shift+Enter for new line"
                }
            }
        }
    }
}

/// Composer toolbar button.
#[component]
fn ComposerButton(
    icon: String,
    tooltip: String,
    onclick: EventHandler<MouseEvent>,
) -> Element {
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
            title: "{tooltip}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |evt| onclick.call(evt),
            "{icon}"
        }
    }
}

/// Send button with primary styling.
#[component]
fn SendButton(
    disabled: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
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

            span { "↓" }
            span { "{count} new message" if count == 1 { "" } else { "s" } }
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

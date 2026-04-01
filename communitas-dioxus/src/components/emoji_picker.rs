// SPDX-License-Identifier: MIT OR Apache-2.0

//! Emoji picker and quick-reaction components.
//!
//! Provides two components:
//! - [`EmojiPicker`] — full category-tab picker with search, rendered as a floating popover.
//! - [`QuickReactionBar`] — compact row of six quick-reaction emojis plus a "more" button.

use crate::components::emoji_data::{self, EmojiCategory, QUICK_REACTIONS};
use crate::design_tokens::{motion, radius, semantic, shadow, spacing, typography};
use dioxus::prelude::*;

/// Icons used for each category tab button.
fn category_icon(cat: EmojiCategory) -> &'static str {
    match cat {
        EmojiCategory::Smileys => "😊",
        EmojiCategory::People => "👋",
        EmojiCategory::Nature => "🐶",
        EmojiCategory::Food => "🍎",
        EmojiCategory::Travel => "🚗",
        EmojiCategory::Objects => "💻",
        EmojiCategory::Symbols => "❤️",
        EmojiCategory::Flags => "🏳️",
    }
}

/// Human-readable label for a category, used as aria-label on tab buttons.
fn category_label(cat: EmojiCategory) -> &'static str {
    match cat {
        EmojiCategory::Smileys => "Smileys",
        EmojiCategory::People => "People",
        EmojiCategory::Nature => "Nature",
        EmojiCategory::Food => "Food",
        EmojiCategory::Travel => "Travel",
        EmojiCategory::Objects => "Objects",
        EmojiCategory::Symbols => "Symbols",
        EmojiCategory::Flags => "Flags",
    }
}

/// All categories in tab order.
const ALL_CATEGORIES: &[EmojiCategory] = &[
    EmojiCategory::Smileys,
    EmojiCategory::People,
    EmojiCategory::Nature,
    EmojiCategory::Food,
    EmojiCategory::Travel,
    EmojiCategory::Objects,
    EmojiCategory::Symbols,
    EmojiCategory::Flags,
];

/// Full emoji picker popover with category tabs, search, and scrollable grid.
///
/// # Props
/// - `on_select` — fires with the selected emoji string.
/// - `on_close` — fires when the picker should be dismissed (click outside / Escape).
#[component]
pub fn EmojiPicker(
    /// Called when the user selects an emoji.
    on_select: EventHandler<String>,
    /// Called when the picker should close (click outside or Escape key).
    on_close: EventHandler<()>,
) -> Element {
    let mut search_query = use_signal(String::new);
    let mut active_category = use_signal(|| EmojiCategory::Smileys);

    // Derive the emoji list: if searching, use global search; otherwise filter by category.
    let emojis = use_memo(move || {
        let q = search_query();
        if q.is_empty() {
            emoji_data::by_category(active_category())
        } else {
            emoji_data::search(&q)
        }
    });

    rsx! {
        // Transparent backdrop — clicking it closes the picker
        div {
            style: "position: fixed; inset: 0; z-index: 199;",
            onclick: move |_| on_close.call(()),
        }

        // Picker panel
        div {
            style: format!(
                "position: absolute; \
                 bottom: calc(100% + {}); \
                 right: 0; \
                 width: 320px; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 box-shadow: {}; \
                 z-index: 200; \
                 display: flex; \
                 flex-direction: column; \
                 overflow: hidden;",
                spacing::SM,
                semantic::BG_ELEVATED,
                semantic::BORDER_DEFAULT,
                radius::XL,
                shadow::LG,
            ),
            // Stop click propagation so clicks inside don't bubble to the backdrop
            onclick: move |evt| evt.stop_propagation(),
            onkeydown: move |evt| {
                if evt.key() == Key::Escape {
                    on_close.call(());
                }
            },
            role: "dialog",
            aria_label: "Emoji picker",

            // ── Search bar ──────────────────────────────────────────────────
            div {
                style: format!(
                    "padding: {} {}; \
                     border-bottom: 1px solid {};",
                    spacing::SM,
                    spacing::SM,
                    semantic::BORDER_SUBTLE
                ),
                input {
                    r#type: "text",
                    placeholder: "Search emoji…",
                    value: "{search_query}",
                    aria_label: "Search emoji",
                    autofocus: true,
                    style: format!(
                        "width: 100%; \
                         background: {}; \
                         border: 1px solid {}; \
                         border-radius: {}; \
                         color: {}; \
                         font-family: {}; \
                         font-size: {}; \
                         padding: {} {}; \
                         outline: none; \
                         box-sizing: border-box;",
                        semantic::BG_TERTIARY,
                        semantic::BORDER_DEFAULT,
                        radius::MD,
                        semantic::TEXT_PRIMARY,
                        typography::FONT_BODY,
                        typography::SIZE_SM,
                        spacing::XS,
                        spacing::SM,
                    ),
                    oninput: move |evt: FormEvent| search_query.set(evt.value()),
                }
            }

            // ── Category tabs ───────────────────────────────────────────────
            if search_query().is_empty() {
                div {
                    style: format!(
                        "display: flex; \
                         gap: {}; \
                         padding: {} {}; \
                         border-bottom: 1px solid {}; \
                         overflow-x: auto; \
                         scrollbar-width: none;",
                        spacing::XXS,
                        spacing::XS,
                        spacing::XS,
                        semantic::BORDER_SUBTLE,
                    ),
                    role: "tablist",
                    aria_label: "Emoji categories",

                    for cat in ALL_CATEGORIES {
                        {
                            let cat = *cat;
                            let is_active = active_category() == cat;
                            rsx! {
                                button {
                                    key: "{category_label(cat)}",
                                    role: "tab",
                                    aria_selected: if is_active { "true" } else { "false" },
                                    aria_label: "{category_label(cat)}",
                                    title: "{category_label(cat)}",
                                    style: format!(
                                        "flex-shrink: 0; \
                                         padding: {}; \
                                         background: {}; \
                                         border: none; \
                                         border-radius: {}; \
                                         font-size: {}; \
                                         cursor: pointer; \
                                         transition: {}; \
                                         opacity: {};",
                                        spacing::XS,
                                        if is_active { semantic::BG_HOVER } else { "transparent" },
                                        radius::MD,
                                        typography::SIZE_BASE,
                                        motion::transition("all"),
                                        if is_active { "1" } else { "0.6" },
                                    ),
                                    onclick: move |_| active_category.set(cat),
                                    "{category_icon(cat)}"
                                }
                            }
                        }
                    }
                }
            }

            // ── Emoji grid ──────────────────────────────────────────────────
            div {
                style: format!(
                    "display: grid; \
                     grid-template-columns: repeat(8, 1fr); \
                     gap: {}; \
                     padding: {}; \
                     overflow-y: auto; \
                     max-height: 220px; \
                     scrollbar-width: thin; \
                     scrollbar-color: {} transparent;",
                    spacing::XXS,
                    spacing::SM,
                    semantic::BORDER_DEFAULT,
                ),
                role: "listbox",
                aria_label: "Emoji list",

                if emojis().is_empty() {
                    div {
                        style: format!(
                            "grid-column: 1 / -1; \
                             text-align: center; \
                             padding: {}; \
                             color: {}; \
                             font-size: {};",
                            spacing::XL,
                            semantic::TEXT_MUTED,
                            typography::SIZE_SM,
                        ),
                        "No emoji found"
                    }
                }

                for entry in emojis().iter() {
                    {
                        let emoji = entry.emoji.to_string();
                        let label = entry.name.to_string();
                        rsx! {
                            button {
                                key: "{emoji}",
                                role: "option",
                                aria_label: "{label}",
                                title: "{label}",
                                style: format!(
                                    "aspect-ratio: 1; \
                                     display: flex; \
                                     align-items: center; \
                                     justify-content: center; \
                                     background: transparent; \
                                     border: none; \
                                     border-radius: {}; \
                                     font-size: {}; \
                                     cursor: pointer; \
                                     transition: {}; \
                                     padding: {};",
                                    radius::MD,
                                    typography::SIZE_LG,
                                    motion::transition("background"),
                                    spacing::XXS,
                                ),
                                onmouseenter: {
                                    // inline hover via style update not needed; CSS handles it
                                    move |_| {}
                                },
                                onclick: {
                                    let emoji = emoji.clone();
                                    move |_| on_select.call(emoji.clone())
                                },
                                "{emoji}"
                            }
                        }
                    }
                }
            }
        }

        // Inject hover style via <style> tag (can't use pseudo-selectors inline)
        style {
            r#"
            [role="listbox"] button:hover {{
                background: rgba(36, 51, 68, 0.8) !important;
            }}
            "#
        }
    }
}

/// Compact quick-reaction bar showing six common emoji and a "more" button.
///
/// Appears on message hover, above the action bar.
///
/// # Props
/// - `on_select` — fires with the selected emoji string.
/// - `on_more` — fires when the user clicks "+" to open the full picker.
#[component]
pub fn QuickReactionBar(
    /// Called when the user clicks one of the quick-reaction emoji.
    on_select: EventHandler<String>,
    /// Called when the user clicks the "+" button to open the full picker.
    on_more: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 box-shadow: {};",
                spacing::XXS,
                spacing::XXS,
                spacing::XS,
                semantic::BG_ELEVATED,
                semantic::BORDER_DEFAULT,
                radius::FULL,
                shadow::MD,
            ),
            aria_label: "Quick reactions",
            role: "toolbar",

            for emoji in QUICK_REACTIONS {
                {
                    let emoji_str = emoji.to_string();
                    rsx! {
                        QuickReactionButton {
                            key: "{emoji_str}",
                            emoji: emoji_str,
                            on_click: move |e: String| on_select.call(e),
                        }
                    }
                }
            }

            // Divider
            div {
                style: format!(
                    "width: 1px; height: 16px; background: {};",
                    semantic::BORDER_SUBTLE
                ),
            }

            // "More emojis" button
            button {
                style: format!(
                    "width: 28px; \
                     height: 28px; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     background: transparent; \
                     border: none; \
                     border-radius: {}; \
                     cursor: pointer; \
                     font-size: {}; \
                     color: {}; \
                     transition: {}; \
                     flex-shrink: 0;",
                    radius::FULL,
                    typography::SIZE_SM,
                    semantic::TEXT_SECONDARY,
                    motion::transition("background"),
                ),
                aria_label: "More emoji",
                title: "More emoji",
                onclick: move |_| on_more.call(()),
                "+"
            }
        }
    }
}

/// A single emoji button in the quick-reaction bar.
#[component]
fn QuickReactionButton(emoji: String, on_click: EventHandler<String>) -> Element {
    let mut hovered = use_signal(|| false);

    let e = emoji.clone();
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
                 font-size: {}; \
                 cursor: pointer; \
                 transition: {}; \
                 flex-shrink: 0;",
                if hovered() { semantic::BG_HOVER } else { "transparent" },
                radius::FULL,
                typography::SIZE_BASE,
                motion::transition("background"),
            ),
            aria_label: "{emoji}",
            title: "{emoji}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onclick: move |_| on_click.call(e.clone()),
            "{emoji}"
        }
    }
}

// ── Category label display workaround (used in format strings) ───────────────
// category_label and category_icon are pure functions with no runtime state,
// so they produce no dead-code lint in practice.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_categories_have_icons() {
        for cat in ALL_CATEGORIES {
            let icon = category_icon(*cat);
            assert!(!icon.is_empty(), "Missing icon for {:?}", cat);
        }
    }

    #[test]
    fn all_categories_have_labels() {
        for cat in ALL_CATEGORIES {
            let label = category_label(*cat);
            assert!(!label.is_empty(), "Missing label for {:?}", cat);
        }
    }

    #[test]
    fn quick_reactions_len_matches_constant() {
        assert_eq!(QUICK_REACTIONS.len(), 6);
    }
}

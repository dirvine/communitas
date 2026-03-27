//! @mention autocomplete dropdown for the message composer.
//!
//! Shows a filtered list of contacts when the user types `@` in the composer.
//! Supports keyboard navigation (ArrowUp/ArrowDown to move, Enter to confirm,
//! Escape to dismiss) and highlights the matching query prefix.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::mention::{MentionAutocomplete, MentionCandidate};
//!
//! let candidates = vec![
//!     MentionCandidate { id: "alice".to_string(), display_name: "Alice".to_string() },
//! ];
//! rsx! {
//!     MentionAutocomplete {
//!         candidates,
//!         query: "ali".to_string(),
//!         on_select: move |c: MentionCandidate| { /* insert mention */ },
//!         on_dismiss: move |_| { /* close picker */ },
//!     }
//! }
//! ```

use crate::design_tokens::{motion, palette, radius, semantic, shadow, spacing, typography};
use dioxus::prelude::*;

/// A candidate contact that can be @-mentioned.
#[derive(Clone, PartialEq, Debug)]
pub struct MentionCandidate {
    /// Unique identifier for this contact.
    pub id: String,
    /// Human-readable display name shown in the dropdown and inserted into the message.
    pub display_name: String,
}

/// Maximum number of candidates shown in the dropdown.
const MAX_VISIBLE: usize = 8;

/// Filter candidates whose display_name starts with `query` (case-insensitive).
///
/// Returns at most [`MAX_VISIBLE`] results.
pub fn filter_candidates<'a>(
    candidates: &'a [MentionCandidate],
    query: &str,
) -> Vec<&'a MentionCandidate> {
    let q = query.to_lowercase();
    candidates
        .iter()
        .filter(|c| c.display_name.to_lowercase().starts_with(q.as_str()))
        .take(MAX_VISIBLE)
        .collect()
}

/// Props for [`MentionAutocomplete`].
#[derive(Props, Clone, PartialEq)]
pub struct MentionAutocompleteProps {
    /// Full candidate list (will be filtered by `query`).
    pub candidates: Vec<MentionCandidate>,
    /// Current query text after the `@` character.
    pub query: String,
    /// Called when the user selects a candidate (keyboard Enter or mouse click).
    pub on_select: EventHandler<MentionCandidate>,
    /// Called when the user dismisses the dropdown (Escape key).
    pub on_dismiss: EventHandler<()>,
}

/// Autocomplete dropdown for @mention insertion.
///
/// Renders above the composer using absolute positioning relative to its
/// nearest positioned parent. The parent is responsible for placing the
/// component correctly relative to the textarea.
#[component]
pub fn MentionAutocomplete(props: MentionAutocompleteProps) -> Element {
    let filtered = filter_candidates(&props.candidates, &props.query);

    // Keyboard-controlled selected index (None = nothing highlighted yet).
    let mut selected_idx: Signal<Option<usize>> =
        use_signal(|| if filtered.is_empty() { None } else { Some(0) });

    // Re-reset selection whenever filtered list changes length.
    let filtered_len = filtered.len();
    use_effect(move || {
        if filtered_len == 0 {
            selected_idx.set(None);
        } else {
            selected_idx.set(Some(0));
        }
    });

    if filtered.is_empty() {
        return rsx! {};
    }

    let on_select = props.on_select;
    let on_dismiss = props.on_dismiss;

    rsx! {
        div {
            // Keyboard handler: arrow nav + enter + escape
            onkeydown: {
                let filtered_clone: Vec<MentionCandidate> = filtered.iter().map(|c| (*c).clone()).collect();
                move |evt: KeyboardEvent| {
                    match evt.key() {
                        Key::ArrowDown => {
                            evt.prevent_default();
                            let len = filtered_clone.len();
                            let next = match selected_idx() {
                                None => 0,
                                Some(i) => (i + 1).min(len.saturating_sub(1)),
                            };
                            selected_idx.set(Some(next));
                        }
                        Key::ArrowUp => {
                            evt.prevent_default();
                            let next = match selected_idx() {
                                None | Some(0) => 0,
                                Some(i) => i - 1,
                            };
                            selected_idx.set(Some(next));
                        }
                        Key::Enter => {
                            evt.prevent_default();
                            if let Some(idx) = selected_idx()
                                && let Some(candidate) = filtered_clone.get(idx)
                            {
                                on_select.call(candidate.clone());
                            }
                        }
                        Key::Escape => {
                            evt.prevent_default();
                            on_dismiss.call(());
                        }
                        _ => {}
                    }
                }
            },

            style: format!(
                "position: absolute; \
                 bottom: calc(100% + {}); \
                 left: 0; \
                 right: 0; \
                 background: {}; \
                 border: 1px solid {}; \
                 border-radius: {}; \
                 box-shadow: {}; \
                 overflow: hidden; \
                 z-index: 200;",
                spacing::XS,
                semantic::BG_ELEVATED,
                semantic::BORDER_DEFAULT,
                radius::MD,
                shadow::LG,
            ),
            role: "listbox",
            aria_label: "Mention suggestions",

            // Header label
            div {
                style: format!(
                    "padding: {} {}; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     border-bottom: 1px solid {};",
                    spacing::XS,
                    spacing::SM,
                    typography::SIZE_XS,
                    typography::WEIGHT_SEMIBOLD,
                    semantic::TEXT_MUTED,
                    semantic::BORDER_SUBTLE,
                ),
                "People matching @{props.query}"
            }

            // Candidate rows
            for (idx, candidate) in filtered.iter().enumerate() {
                {
                    let is_selected = selected_idx() == Some(idx);
                    let c = (*candidate).clone();
                    let c_for_click = c.clone();
                    let query_str = props.query.clone();
                    rsx! {
                        MentionCandidateRow {
                            key: "{c.id}",
                            candidate: c,
                            query: query_str,
                            is_selected,
                            on_click: move |_| on_select.call(c_for_click.clone()),
                            on_hover: move |_| selected_idx.set(Some(idx)),
                        }
                    }
                }
            }
        }
    }
}

/// Props for a single candidate row.
#[derive(Props, Clone, PartialEq)]
struct MentionCandidateRowProps {
    candidate: MentionCandidate,
    /// Current search query — used to highlight matched prefix.
    query: String,
    /// Whether this row is currently keyboard-selected.
    is_selected: bool,
    on_click: EventHandler<()>,
    on_hover: EventHandler<()>,
}

/// Single row in the mention autocomplete dropdown.
#[component]
fn MentionCandidateRow(props: MentionCandidateRowProps) -> Element {
    let bg = if props.is_selected {
        format!("background: {};", semantic::BG_HOVER)
    } else {
        "background: transparent;".to_string()
    };

    // Split display_name into matched prefix and remainder for highlighting.
    let name = props.candidate.display_name.clone();
    let q_len = props.query.len().min(name.len());
    let matched = name[..q_len].to_string();
    let rest = name[q_len..].to_string();

    rsx! {
        div {
            role: "option",
            aria_selected: if props.is_selected { "true" } else { "false" },
            style: format!(
                "display: flex; \
                 align-items: center; \
                 gap: {}; \
                 padding: {} {}; \
                 cursor: pointer; \
                 transition: background {}; \
                 {}",
                spacing::SM,
                spacing::XS,
                spacing::SM,
                motion::FAST,
                bg,
            ),
            onclick: move |_| props.on_click.call(()),
            onmouseenter: move |_| props.on_hover.call(()),

            // Avatar circle showing first letter of display name
            div {
                style: format!(
                    "width: 28px; \
                     height: 28px; \
                     border-radius: {}; \
                     background: {}20; \
                     border: 1px solid {}40; \
                     display: flex; \
                     align-items: center; \
                     justify-content: center; \
                     font-size: {}; \
                     font-weight: {}; \
                     color: {}; \
                     flex-shrink: 0; \
                     text-transform: uppercase;",
                    radius::FULL,
                    palette::JADE_500,
                    palette::JADE_500,
                    typography::SIZE_XS,
                    typography::WEIGHT_BOLD,
                    palette::JADE_400,
                ),
                "{props.candidate.display_name.chars().next().unwrap_or('?')}"
            }

            // Display name with highlighted prefix
            span {
                style: format!(
                    "font-size: {}; \
                     color: {};",
                    typography::SIZE_SM,
                    semantic::TEXT_PRIMARY,
                ),

                // Highlighted portion (matched query prefix)
                if !matched.is_empty() {
                    span {
                        style: format!(
                            "color: {}; \
                             font-weight: {};",
                            palette::JADE_400,
                            typography::WEIGHT_SEMIBOLD,
                        ),
                        "{matched}"
                    }
                }

                // Non-highlighted remainder
                "{rest}"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candidate(id: &str, name: &str) -> MentionCandidate {
        MentionCandidate {
            id: id.to_string(),
            display_name: name.to_string(),
        }
    }

    #[test]
    fn filter_by_prefix_case_insensitive() {
        let candidates = vec![
            make_candidate("1", "Alice"),
            make_candidate("2", "Bob"),
            make_candidate("3", "Alfred"),
        ];
        let result = filter_candidates(&candidates, "al");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].display_name, "Alice");
        assert_eq!(result[1].display_name, "Alfred");
    }

    #[test]
    fn filter_empty_query_returns_all_up_to_max() {
        let candidates: Vec<MentionCandidate> = (0..10)
            .map(|i| make_candidate(&i.to_string(), &format!("Person {i}")))
            .collect();
        let result = filter_candidates(&candidates, "");
        assert_eq!(result.len(), MAX_VISIBLE);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let candidates = vec![make_candidate("1", "Alice"), make_candidate("2", "Bob")];
        let result = filter_candidates(&candidates, "xyz");
        assert!(result.is_empty());
    }

    #[test]
    fn filter_exact_match() {
        let candidates = vec![make_candidate("1", "Alice"), make_candidate("2", "Bob")];
        let result = filter_candidates(&candidates, "Bob");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "2");
    }

    #[test]
    fn filter_respects_max_visible() {
        let candidates: Vec<MentionCandidate> = (0..20)
            .map(|i| make_candidate(&i.to_string(), &format!("Aardvark {i}")))
            .collect();
        let result = filter_candidates(&candidates, "A");
        assert_eq!(result.len(), MAX_VISIBLE);
    }

    #[test]
    fn mention_candidate_clone_and_eq() {
        let a = make_candidate("x", "Xavier");
        let b = a.clone();
        assert_eq!(a, b);
    }
}

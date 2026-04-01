// SPDX-License-Identifier: MIT OR Apache-2.0

//! Filter chip components for quick filtering of lists and collections.
//!
//! Displays a horizontal row of clickable filter chips with optional count badges.
//! Active filters are highlighted, and clicking an active filter deactivates it.
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::filter_chips::{FilterChips, FilterOption};
//!
//! let options = vec![
//!     FilterOption {
//!         key: "orgs".to_string(),
//!         label: "Organizations".to_string(),
//!         count: Some(5),
//!     },
//!     FilterOption {
//!         key: "projects".to_string(),
//!         label: "Projects".to_string(),
//!         count: Some(12),
//!     },
//! ];
//!
//! rsx! {
//!     FilterChips {
//!         options: options,
//!         active_key: Some("orgs".to_string()),
//!         on_filter_change: move |key| println!("Filter changed to: {:?}", key)
//!     }
//! }
//! ```

use crate::design_tokens::{palette, radius, semantic, spacing, typography};
use dioxus::prelude::*;

/// A single filter option.
#[derive(Clone, PartialEq)]
pub struct FilterOption {
    /// Unique key for this filter.
    pub key: String,
    /// Display label.
    pub label: String,
    /// Optional count to show.
    pub count: Option<usize>,
}

/// Props for the FilterChips component.
#[derive(Props, Clone, PartialEq)]
pub struct FilterChipsProps {
    /// Available filter options.
    pub options: Vec<FilterOption>,
    /// Currently active filter key (None = show all).
    pub active_key: Option<String>,
    /// Callback when a filter is selected.
    pub on_filter_change: EventHandler<Option<String>>,
}

/// Horizontal row of filter chips for quick filtering.
///
/// Displays an "All" chip followed by each filter option.
/// - Clicking an inactive chip activates it
/// - Clicking an active chip deactivates it (returns to "All")
/// - Count badges are displayed when present
#[component]
pub fn FilterChips(props: FilterChipsProps) -> Element {
    let container_style = format!(
        "display: flex; align-items: center; gap: {}; flex-wrap: wrap;",
        spacing::XS
    );

    rsx! {
        div {
            style: "{container_style}",
            role: "group",
            aria_label: "Filter options",

            // "All" chip
            {
                let is_active = props.active_key.is_none();
                let chip_style = if is_active {
                    format!(
                        "display: inline-flex; align-items: center; gap: {}; padding: {} {}; \
                         border-radius: {}; border: 1px solid {}; background: {}; color: {}; \
                         font-size: {}; cursor: pointer; transition: all 0.2s ease;",
                        spacing::XXS,
                        spacing::XS,
                        spacing::SM,
                        radius::FULL,
                        palette::JADE_600,
                        palette::JADE_900,
                        semantic::TEXT_PRIMARY,
                        typography::SIZE_XS
                    )
                } else {
                    format!(
                        "display: inline-flex; align-items: center; gap: {}; padding: {} {}; \
                         border-radius: {}; border: 1px solid {}; background: transparent; color: {}; \
                         font-size: {}; cursor: pointer; transition: all 0.2s ease;",
                        spacing::XXS,
                        spacing::XS,
                        spacing::SM,
                        radius::FULL,
                        semantic::BORDER_DEFAULT,
                        semantic::TEXT_MUTED,
                        typography::SIZE_XS
                    )
                };

                rsx! {
                    button {
                        style: "{chip_style}",
                        onclick: move |_| props.on_filter_change.call(None),
                        role: "button",
                        aria_pressed: is_active,
                        span { "All" }
                    }
                }
            }

            // Individual filter chips
            for option in &props.options {
                {
                    let key = option.key.clone();
                    let current_key = props.active_key.clone();
                    let is_active = props.active_key.as_ref() == Some(&option.key);
                    let label = option.label.clone();
                    let count = option.count;

                    let chip_style = if is_active {
                        format!(
                            "display: inline-flex; align-items: center; gap: {}; padding: {} {}; \
                             border-radius: {}; border: 1px solid {}; background: {}; color: {}; \
                             font-size: {}; cursor: pointer; transition: all 0.2s ease;",
                            spacing::XXS,
                            spacing::XS,
                            spacing::SM,
                            radius::FULL,
                            palette::JADE_600,
                            palette::JADE_900,
                            semantic::TEXT_PRIMARY,
                            typography::SIZE_XS
                        )
                    } else {
                        format!(
                            "display: inline-flex; align-items: center; gap: {}; padding: {} {}; \
                             border-radius: {}; border: 1px solid {}; background: transparent; color: {}; \
                             font-size: {}; cursor: pointer; transition: all 0.2s ease;",
                            spacing::XXS,
                            spacing::XS,
                            spacing::SM,
                            radius::FULL,
                            semantic::BORDER_DEFAULT,
                            semantic::TEXT_MUTED,
                            typography::SIZE_XS
                        )
                    };

                    let badge_style = if is_active {
                        format!(
                            "display: inline-flex; align-items: center; justify-content: center; \
                             min-width: 1.25rem; height: 1.25rem; padding: 0 {}; border-radius: {}; \
                             background: {}; color: {}; font-size: {}; font-weight: {};",
                            spacing::XXS,
                            radius::FULL,
                            palette::JADE_700,
                            semantic::TEXT_PRIMARY,
                            typography::SIZE_XXS,
                            typography::WEIGHT_MEDIUM
                        )
                    } else {
                        format!(
                            "display: inline-flex; align-items: center; justify-content: center; \
                             min-width: 1.25rem; height: 1.25rem; padding: 0 {}; border-radius: {}; \
                             background: {}; color: {}; font-size: {}; font-weight: {};",
                            spacing::XXS,
                            radius::FULL,
                            semantic::BG_TERTIARY,
                            semantic::TEXT_MUTED,
                            typography::SIZE_XXS,
                            typography::WEIGHT_MEDIUM
                        )
                    };

                    rsx! {
                        button {
                            style: "{chip_style}",
                            onclick: move |_| {
                                // Toggle: if clicking active chip, deactivate; otherwise activate
                                if current_key.as_ref() == Some(&key) {
                                    props.on_filter_change.call(None);
                                } else {
                                    props.on_filter_change.call(Some(key.clone()));
                                }
                            },
                            role: "button",
                            aria_pressed: is_active,

                            span { "{label}" }

                            if let Some(count_value) = count {
                                span {
                                    style: "{badge_style}",
                                    aria_label: "{count_value} items",
                                    "{count_value}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

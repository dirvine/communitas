// SPDX-License-Identifier: MIT OR Apache-2.0

//! Pagination components for list navigation and infinite scroll.
//!
//! Provides two variants:
//! - `Pagination`: Full page navigation with prev/next buttons and page indicator
//! - `LoadMore`: Simple button for infinite scroll patterns
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::pagination::{Pagination, LoadMore};
//!
//! rsx! {
//!     // Full pagination
//!     Pagination {
//!         current_page: 2,
//!         total_pages: 5,
//!         on_page_change: move |page| println!("Go to page {}", page)
//!     }
//!
//!     // Load more button
//!     LoadMore {
//!         has_more: true,
//!         loading: false,
//!         on_load_more: move |_| println!("Loading more...")
//!     }
//! }
//! ```

use crate::design_tokens::{radius, semantic, spacing, typography};
use dioxus::prelude::*;

/// Props for the Pagination component.
#[derive(Props, Clone, PartialEq)]
pub struct PaginationProps {
    /// Current page (1-based).
    pub current_page: usize,
    /// Total number of pages.
    pub total_pages: usize,
    /// Callback when page changes.
    pub on_page_change: EventHandler<usize>,
}

/// Page navigation with prev/next and page indicator.
///
/// Displays: [< Prev] Page 1 of 5 [Next >]
/// Prev button is disabled when on first page, Next button is disabled on last page.
#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    let is_first = props.current_page == 1;
    let is_last = props.current_page >= props.total_pages;

    let prev_button_style = if is_first {
        format!(
            "padding: {} {}; border-radius: {}; border: 1px solid {}; background: transparent; \
             color: {}; font-size: {}; cursor: not-allowed; opacity: 0.5;",
            spacing::SM,
            spacing::BASE,
            radius::MD,
            semantic::BORDER_DEFAULT,
            semantic::TEXT_MUTED,
            typography::SIZE_SM
        )
    } else {
        format!(
            "padding: {} {}; border-radius: {}; border: 1px solid {}; background: transparent; \
             color: {}; font-size: {}; cursor: pointer; transition: all 0.2s ease;",
            spacing::SM,
            spacing::BASE,
            radius::MD,
            semantic::BORDER_DEFAULT,
            semantic::TEXT_PRIMARY,
            typography::SIZE_SM
        )
    };

    let next_button_style = if is_last {
        format!(
            "padding: {} {}; border-radius: {}; border: 1px solid {}; background: transparent; \
             color: {}; font-size: {}; cursor: not-allowed; opacity: 0.5;",
            spacing::SM,
            spacing::BASE,
            radius::MD,
            semantic::BORDER_DEFAULT,
            semantic::TEXT_MUTED,
            typography::SIZE_SM
        )
    } else {
        format!(
            "padding: {} {}; border-radius: {}; border: 1px solid {}; background: transparent; \
             color: {}; font-size: {}; cursor: pointer; transition: all 0.2s ease;",
            spacing::SM,
            spacing::BASE,
            radius::MD,
            semantic::BORDER_DEFAULT,
            semantic::TEXT_PRIMARY,
            typography::SIZE_SM
        )
    };

    let page_info_style = format!(
        "padding: {} {}; color: {}; font-size: {};",
        spacing::SM,
        spacing::BASE,
        semantic::TEXT_SECONDARY,
        typography::SIZE_SM
    );

    rsx! {
        div {
            style: format!(
                "display: flex; align-items: center; justify-content: center; gap: {};",
                spacing::SM
            ),
            role: "navigation",
            aria_label: "Pagination",

            button {
                style: "{prev_button_style}",
                disabled: is_first,
                onclick: move |_| {
                    if !is_first {
                        props.on_page_change.call(props.current_page - 1);
                    }
                },
                aria_label: "Previous page",
                "‹ Prev"
            }

            span {
                style: "{page_info_style}",
                aria_current: "page",
                "Page {props.current_page} of {props.total_pages}"
            }

            button {
                style: "{next_button_style}",
                disabled: is_last,
                onclick: move |_| {
                    if !is_last {
                        props.on_page_change.call(props.current_page + 1);
                    }
                },
                aria_label: "Next page",
                "Next ›"
            }
        }
    }
}

/// Props for the LoadMore button.
#[derive(Props, Clone, PartialEq)]
pub struct LoadMoreProps {
    /// Whether more items are available.
    pub has_more: bool,
    /// Whether currently loading.
    #[props(default = false)]
    pub loading: bool,
    /// Button label.
    #[props(default = "Load more".to_string())]
    pub label: String,
    /// Callback when clicked.
    pub on_load_more: EventHandler<()>,
}

/// Simple "Load more" button for infinite scroll.
///
/// Disabled when loading or when no more items are available.
/// Shows "Loading..." text when loading state is true.
#[component]
pub fn LoadMore(props: LoadMoreProps) -> Element {
    let is_disabled = props.loading || !props.has_more;
    let button_text = if props.loading {
        "Loading..."
    } else {
        &props.label
    };

    let button_style = if is_disabled {
        format!(
            "padding: {} {}; border-radius: {}; border: 1px solid {}; background: {}; \
             color: {}; font-size: {}; cursor: not-allowed; opacity: 0.5; min-width: 120px;",
            spacing::SM,
            spacing::BASE,
            radius::MD,
            semantic::BORDER_DEFAULT,
            semantic::BG_SECONDARY,
            semantic::TEXT_MUTED,
            typography::SIZE_SM
        )
    } else {
        format!(
            "padding: {} {}; border-radius: {}; border: 1px solid {}; background: {}; \
             color: {}; font-size: {}; cursor: pointer; transition: all 0.2s ease; min-width: 120px;",
            spacing::SM,
            spacing::BASE,
            radius::MD,
            semantic::BORDER_DEFAULT,
            semantic::BG_SECONDARY,
            semantic::TEXT_PRIMARY,
            typography::SIZE_SM
        )
    };

    let container_style = format!(
        "display: flex; justify-content: center; margin-top: {};",
        spacing::BASE
    );

    let aria_label = if props.loading {
        "Loading more items"
    } else {
        &props.label
    };

    rsx! {
        div {
            style: "{container_style}",
            button {
                style: "{button_style}",
                disabled: is_disabled,
                onclick: move |_| {
                    if !is_disabled {
                        props.on_load_more.call(());
                    }
                },
                aria_label: "{aria_label}",
                "{button_text}"
            }
        }
    }
}

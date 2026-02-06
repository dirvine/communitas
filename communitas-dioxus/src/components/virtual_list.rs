//! Virtual list component for efficient rendering of large lists.
//!
//! Only renders visible items plus an overscan buffer, dramatically improving
//! performance for lists with hundreds or thousands of items.

use dioxus::prelude::*;

/// Props for the VirtualList component.
#[derive(Props, Clone, PartialEq)]
pub struct VirtualListProps {
    /// Total number of items in the list.
    pub total_count: usize,

    /// Height of each item in pixels.
    pub item_height_px: f64,

    /// Number of buffer items to render above and below the visible area.
    #[props(default = 5)]
    pub overscan: usize,

    /// CSS height value for the scrollable container.
    #[props(default = "100%".to_string())]
    pub container_height: String,

    /// Callback to render an item at the given index.
    /// Returns an Element to display.
    pub render_item: Callback<usize, Element>,
}

/// A virtualized list component that only renders visible items.
///
/// # Example
///
/// ```rust,ignore
/// rsx! {
///     VirtualList {
///         total_count: 10000,
///         item_height_px: 50.0,
///         overscan: 5,
///         container_height: "600px",
///         render_item: move |index| {
///             rsx! {
///                 div { "Item {index}" }
///             }
///         }
///     }
/// }
/// ```
#[component]
pub fn VirtualList(props: VirtualListProps) -> Element {
    let mut scroll_top = use_signal(|| 0.0_f64);
    let list_id = use_signal(|| format!("vlist-{}", generate_id()));

    // Calculate visible range
    let scroll_pos = scroll_top();
    let start_idx = if props.item_height_px > 0.0 {
        ((scroll_pos / props.item_height_px).floor() as usize).saturating_sub(props.overscan)
    } else {
        0
    };

    // Parse container height to get numeric value for visible count calculation
    let container_height_numeric = parse_css_height(&props.container_height).unwrap_or(600.0);
    let visible_count = if props.item_height_px > 0.0 {
        ((container_height_numeric / props.item_height_px).ceil() as usize)
            .saturating_add(1)
            .saturating_add(props.overscan * 2)
    } else {
        0
    };

    let end_idx = (start_idx + visible_count).min(props.total_count);

    let total_height = props.total_count as f64 * props.item_height_px;
    let list_id_val = list_id();

    rsx! {
        div {
            id: "{list_id_val}",
            class: "virtual-list-container",
            style: "position: relative; overflow-y: auto; height: {props.container_height}; width: 100%;",
            onscroll: move |_| {
                let id = list_id();
                spawn(async move {
                    let js = format!("return document.getElementById('{}').scrollTop", id);
                    match document::eval(&js).recv::<f64>().await {
                        Ok(val) => scroll_top.set(val),
                        Err(_) => {
                            // Silently ignore eval errors (e.g., element not found during unmount)
                        }
                    }
                });
            },

            // Inner spacer to establish scroll height
            div {
                class: "virtual-list-spacer",
                style: "position: relative; width: 100%; height: {total_height}px;",

                // Render only visible items
                for idx in start_idx..end_idx {
                    div {
                        key: "{idx}",
                        class: "virtual-list-item",
                        style: "position: absolute; top: {idx as f64 * props.item_height_px}px; width: 100%;",
                        {props.render_item.call(idx)}
                    }
                }
            }
        }
    }
}

/// Generate a unique ID for list instances.
fn generate_id() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u32)
        .unwrap_or(0)
}

/// Parse a CSS height string (e.g., "600px", "100%") into a numeric pixel value.
/// Returns None if the value cannot be parsed.
fn parse_css_height(height: &str) -> Option<f64> {
    let trimmed = height.trim();
    if trimmed.ends_with("px") {
        trimmed.trim_end_matches("px").trim().parse::<f64>().ok()
    } else if trimmed.ends_with('%') {
        // For percentage, we can't know the actual height without parent context.
        // Return a default estimate (600px is a reasonable viewport height).
        Some(600.0)
    } else {
        // Try parsing as raw number (assume pixels)
        trimmed.parse::<f64>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_css_height() {
        assert_eq!(parse_css_height("600px"), Some(600.0));
        assert_eq!(parse_css_height("100%"), Some(600.0)); // default estimate
        assert_eq!(parse_css_height("800"), Some(800.0));
        assert_eq!(parse_css_height("invalid"), None);
        assert_eq!(parse_css_height(""), None);
    }

    #[test]
    fn test_visible_range_calculation() {
        // Test with zero item height (edge case)
        let item_height = 0.0_f64;
        let scroll_top = 0.0_f64;
        let start_idx = if item_height > 0.0 {
            ((scroll_top / item_height).floor() as usize).saturating_sub(5)
        } else {
            0
        };
        assert_eq!(start_idx, 0);

        // Test with normal values
        let item_height = 50.0_f64;
        let scroll_top = 250.0_f64;
        let start_idx = ((scroll_top / item_height).floor() as usize).saturating_sub(5);
        assert_eq!(start_idx, 0); // 5 - 5 = 0

        let scroll_top = 1000.0_f64;
        let start_idx = ((scroll_top / item_height).floor() as usize).saturating_sub(5);
        assert_eq!(start_idx, 15); // 20 - 5 = 15
    }
}

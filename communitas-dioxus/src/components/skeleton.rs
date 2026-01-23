//! Reusable skeleton loader components for consistent loading states.
//!
//! These components provide accessible, animated placeholders while content loads.
//! All skeletons respect user's `prefers-reduced-motion` preference via Tailwind's
//! `motion-safe:` and `motion-reduce:` modifiers.
//!
//! # Accessibility
//!
//! All skeleton components include:
//! - `aria-busy="true"` to indicate loading state
//! - `aria-label="Loading"` for screen reader announcement
//! - Motion-safe animations that disable for users who prefer reduced motion
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::components::skeleton::{SkeletonLine, SkeletonCard};
//!
//! rsx! {
//!     // Simple line placeholder
//!     SkeletonLine { width: "w-48", height: "h-4" }
//!
//!     // Card placeholder
//!     SkeletonCard { lines: 3 }
//! }
//! ```

use crate::tokens::{colors, motion};
use dioxus::prelude::*;

/// Common properties for skeleton components.
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonProps {
    /// Additional Tailwind classes to apply.
    #[props(default = String::new())]
    pub class: String,

    /// Custom aria-label (defaults to "Loading").
    #[props(default = "Loading".to_string())]
    pub aria_label: String,
}

/// A simple horizontal line skeleton.
///
/// Use for text placeholders, headings, or single-line content.
#[component]
pub fn SkeletonLine(
    /// Width class (e.g., "w-48", "w-full", "w-3/4")
    #[props(default = "w-full".to_string())]
    width: String,
    /// Height class (e.g., "h-4", "h-6", "h-8")
    #[props(default = "h-4".to_string())]
    height: String,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading".to_string())]
    aria_label: String,
) -> Element {
    let animation_class = motion::animation_class("animate-pulse");

    rsx! {
        div {
            class: format!("{} {} rounded {} {}", width, height, animation_class, class),
            style: format!("background-color: {};", colors::SURFACE_ELEVATED),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",
        }
    }
}

/// A circular skeleton for avatars or icons.
#[component]
pub fn SkeletonCircle(
    /// Size class (e.g., "w-8 h-8", "w-12 h-12")
    #[props(default = "w-10 h-10".to_string())]
    size: String,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading".to_string())]
    aria_label: String,
) -> Element {
    let animation_class = motion::animation_class("animate-pulse");

    rsx! {
        div {
            class: format!("{} rounded-full {} {}", size, animation_class, class),
            style: format!("background-color: {};", colors::SURFACE_ELEVATED),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",
        }
    }
}

/// A card-shaped skeleton placeholder.
///
/// Renders a card with optional header, content lines, and footer.
#[component]
pub fn SkeletonCard(
    /// Number of content lines to show
    #[props(default = 3)]
    lines: usize,
    /// Show avatar circle in header
    #[props(default = false)]
    show_avatar: bool,
    /// Show footer section
    #[props(default = false)]
    show_footer: bool,
    /// Card height class (e.g., "h-auto", "h-32", "h-48")
    #[props(default = "h-auto".to_string())]
    height: String,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading card".to_string())]
    aria_label: String,
) -> Element {
    let animation_class = motion::animation_class("animate-pulse");

    rsx! {
        div {
            class: format!(
                "rounded-lg p-4 {} {} {}",
                height, animation_class, class
            ),
            style: format!(
                "background-color: {}; border: 1px solid {};",
                colors::SURFACE_CARD,
                colors::BORDER_DEFAULT
            ),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",

            // Header with optional avatar
            if show_avatar {
                div {
                    class: "flex items-center gap-3 mb-4",
                    SkeletonCircle { size: "w-10 h-10".to_string(), aria_label: "".to_string() }
                    div {
                        class: "flex-1 space-y-2",
                        SkeletonLine { width: "w-24".to_string(), height: "h-4".to_string(), aria_label: "".to_string() }
                        SkeletonLine { width: "w-16".to_string(), height: "h-3".to_string(), aria_label: "".to_string() }
                    }
                }
            }

            // Content lines
            div {
                class: "space-y-2",
                for i in 0..lines {
                    SkeletonLine {
                        key: "{i}",
                        width: if i == lines - 1 { "w-3/4".to_string() } else { "w-full".to_string() },
                        height: "h-4".to_string(),
                        aria_label: "".to_string(),
                    }
                }
            }

            // Optional footer
            if show_footer {
                div {
                    class: "flex justify-between mt-4 pt-4",
                    style: format!("border-top: 1px solid {};", colors::BORDER_DEFAULT),
                    SkeletonLine { width: "w-20".to_string(), height: "h-4".to_string(), aria_label: "".to_string() }
                    SkeletonLine { width: "w-16".to_string(), height: "h-4".to_string(), aria_label: "".to_string() }
                }
            }
        }
    }
}

/// A list of skeleton items for list/feed placeholders.
#[component]
pub fn SkeletonList(
    /// Number of items to show
    #[props(default = 5)]
    count: usize,
    /// Show avatar for each item
    #[props(default = true)]
    show_avatar: bool,
    /// Gap between items (Tailwind class)
    #[props(default = "gap-4".to_string())]
    gap: String,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading list".to_string())]
    aria_label: String,
) -> Element {
    let animation_class = motion::animation_class("animate-pulse");

    rsx! {
        div {
            class: format!("flex flex-col {} {} {}", gap, animation_class, class),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",

            for i in 0..count {
                div {
                    key: "{i}",
                    class: "flex items-center gap-3 p-3 rounded-lg",
                    style: format!("background-color: {};", colors::SURFACE_CARD),

                    if show_avatar {
                        SkeletonCircle { size: "w-10 h-10".to_string(), aria_label: "".to_string() }
                    }

                    div {
                        class: "flex-1 space-y-2",
                        SkeletonLine {
                            width: format!("w-{}", if i % 2 == 0 { "3/4" } else { "2/3" }),
                            height: "h-4".to_string(),
                            aria_label: "".to_string(),
                        }
                        SkeletonLine { width: "w-1/2".to_string(), height: "h-3".to_string(), aria_label: "".to_string() }
                    }
                }
            }
        }
    }
}

/// A paragraph skeleton with multiple lines of varying length.
#[component]
pub fn SkeletonText(
    /// Number of lines
    #[props(default = 4)]
    lines: usize,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading text".to_string())]
    aria_label: String,
) -> Element {
    let animation_class = motion::animation_class("animate-pulse");

    // Vary line widths for natural appearance
    let widths = ["w-full", "w-full", "w-5/6", "w-3/4", "w-4/5", "w-2/3"];

    rsx! {
        div {
            class: format!("space-y-2 {} {}", animation_class, class),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",

            for i in 0..lines {
                SkeletonLine {
                    key: "{i}",
                    width: widths[i % widths.len()].to_string(),
                    height: "h-4".to_string(),
                    aria_label: "".to_string(),
                }
            }
        }
    }
}

/// A grid of skeleton cards.
#[component]
pub fn SkeletonGrid(
    /// Number of cards
    #[props(default = 6)]
    count: usize,
    /// Grid columns (Tailwind class, e.g., "grid-cols-2", "grid-cols-3")
    #[props(default = "grid-cols-3".to_string())]
    columns: String,
    /// Gap between cards
    #[props(default = "gap-4".to_string())]
    gap: String,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading grid".to_string())]
    aria_label: String,
) -> Element {
    rsx! {
        div {
            class: format!("grid {} {} {}", columns, gap, class),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",

            for i in 0..count {
                SkeletonCard {
                    key: "{i}",
                    lines: 2,
                    aria_label: "".to_string(),
                }
            }
        }
    }
}

/// A table skeleton with rows and columns.
#[component]
pub fn SkeletonTable(
    /// Number of rows
    #[props(default = 5)]
    rows: usize,
    /// Number of columns
    #[props(default = 4)]
    columns: usize,
    /// Show header row
    #[props(default = true)]
    show_header: bool,
    /// Additional classes
    #[props(default = String::new())]
    class: String,
    /// Aria label for screen readers
    #[props(default = "Loading table".to_string())]
    aria_label: String,
) -> Element {
    let animation_class = motion::animation_class("animate-pulse");

    rsx! {
        div {
            class: format!("rounded-lg overflow-hidden {} {}", animation_class, class),
            style: format!("background-color: {}; border: 1px solid {};", colors::SURFACE_CARD, colors::BORDER_DEFAULT),
            role: "status",
            aria_busy: "true",
            aria_label: "{aria_label}",

            // Header row
            if show_header {
                div {
                    class: "flex p-3",
                    style: format!("background-color: {};", colors::SURFACE_ELEVATED),
                    for i in 0..columns {
                        div {
                            key: "header-{i}",
                            class: "flex-1 px-2",
                            SkeletonLine { width: "w-20".to_string(), height: "h-4".to_string(), aria_label: "".to_string() }
                        }
                    }
                }
            }

            // Data rows
            for row in 0..rows {
                div {
                    key: "row-{row}",
                    class: "flex p-3",
                    style: if row < rows - 1 {
                        format!("border-bottom: 1px solid {};", colors::BORDER_DEFAULT)
                    } else {
                        String::new()
                    },

                    for col in 0..columns {
                        div {
                            key: "cell-{row}-{col}",
                            class: "flex-1 px-2",
                            SkeletonLine {
                                width: if col == 0 { "w-3/4".to_string() } else { "w-1/2".to_string() },
                                height: "h-4".to_string(),
                                aria_label: "".to_string(),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_line_default_props() {
        // Just verify the component can be constructed with defaults
        let props = SkeletonLineProps::builder().build();
        assert_eq!(props.width, "w-full");
        assert_eq!(props.height, "h-4");
        assert_eq!(props.aria_label, "Loading");
    }

    #[test]
    fn skeleton_circle_default_props() {
        let props = SkeletonCircleProps::builder().build();
        assert_eq!(props.size, "w-10 h-10");
        assert_eq!(props.aria_label, "Loading");
    }

    #[test]
    fn skeleton_card_default_props() {
        let props = SkeletonCardProps::builder().build();
        assert_eq!(props.lines, 3);
        assert!(!props.show_avatar);
        assert!(!props.show_footer);
        assert_eq!(props.aria_label, "Loading card");
    }

    #[test]
    fn skeleton_list_default_props() {
        let props = SkeletonListProps::builder().build();
        assert_eq!(props.count, 5);
        assert!(props.show_avatar);
        assert_eq!(props.gap, "gap-4");
        assert_eq!(props.aria_label, "Loading list");
    }

    #[test]
    fn skeleton_text_default_props() {
        let props = SkeletonTextProps::builder().build();
        assert_eq!(props.lines, 4);
        assert_eq!(props.aria_label, "Loading text");
    }

    #[test]
    fn skeleton_grid_default_props() {
        let props = SkeletonGridProps::builder().build();
        assert_eq!(props.count, 6);
        assert_eq!(props.columns, "grid-cols-3");
        assert_eq!(props.gap, "gap-4");
        assert_eq!(props.aria_label, "Loading grid");
    }

    #[test]
    fn skeleton_table_default_props() {
        let props = SkeletonTableProps::builder().build();
        assert_eq!(props.rows, 5);
        assert_eq!(props.columns, 4);
        assert!(props.show_header);
        assert_eq!(props.aria_label, "Loading table");
    }

    #[test]
    fn all_skeletons_have_accessible_defaults() {
        // Verify all skeleton types have accessible aria_label defaults
        assert!(!SkeletonLineProps::builder().build().aria_label.is_empty());
        assert!(!SkeletonCircleProps::builder().build().aria_label.is_empty());
        assert!(!SkeletonCardProps::builder().build().aria_label.is_empty());
        assert!(!SkeletonListProps::builder().build().aria_label.is_empty());
        assert!(!SkeletonTextProps::builder().build().aria_label.is_empty());
        assert!(!SkeletonGridProps::builder().build().aria_label.is_empty());
        assert!(!SkeletonTableProps::builder().build().aria_label.is_empty());
    }
}

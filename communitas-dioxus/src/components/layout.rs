//! Responsive layout components for consistent UI structure.
//!
//! This module provides reusable layout components that help create
//! responsive, accessible layouts across the application.
//!
//! # Components
//!
//! - [`Container`] - Centered container with max-width constraints
//! - [`Stack`] - Vertical flex layout with consistent gap
//! - [`Row`] - Horizontal flex layout with consistent gap
//! - [`Grid`] - CSS grid layout with configurable columns
//! - [`Spacer`] - Flexible space filler
//! - [`Divider`] - Visual separator (horizontal or vertical)
//!
//! # Example
//!
//! ```ignore
//! use communitas_dioxus::components::layout::{Container, Stack, Row};
//!
//! rsx! {
//!     Container {
//!         Stack {
//!             h1 { "Page Title" }
//!             Row {
//!                 div { "Left content" }
//!                 div { "Right content" }
//!             }
//!         }
//!     }
//! }
//! ```

#![allow(dead_code)]

use dioxus::prelude::*;

use crate::tokens::{breakpoints, colors, spacing};

/// Breakpoint enum for responsive behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Breakpoint {
    /// Extra small screens (< 640px)
    #[default]
    Xs,
    /// Small screens (>= 640px)
    Sm,
    /// Medium screens (>= 768px)
    Md,
    /// Large screens (>= 1024px)
    Lg,
    /// Extra large screens (>= 1280px)
    Xl,
    /// 2x Extra large screens (>= 1536px)
    Xxl,
}

impl Breakpoint {
    /// Get the pixel value for this breakpoint.
    #[must_use]
    pub const fn px(&self) -> u32 {
        match self {
            Self::Xs => 0,
            Self::Sm => breakpoints::SM_PX,
            Self::Md => breakpoints::MD_PX,
            Self::Lg => breakpoints::LG_PX,
            Self::Xl => breakpoints::XL_PX,
            Self::Xxl => breakpoints::XXL_PX,
        }
    }

    /// Get the breakpoint from a width in pixels.
    #[must_use]
    pub fn from_width(width: u32) -> Self {
        if width >= breakpoints::XXL_PX {
            Self::Xxl
        } else if width >= breakpoints::XL_PX {
            Self::Xl
        } else if width >= breakpoints::LG_PX {
            Self::Lg
        } else if width >= breakpoints::MD_PX {
            Self::Md
        } else if width >= breakpoints::SM_PX {
            Self::Sm
        } else {
            Self::Xs
        }
    }

    /// Check if this breakpoint is at or above the given breakpoint.
    #[must_use]
    pub fn is_at_least(&self, other: Self) -> bool {
        self.px() >= other.px()
    }

    /// Check if this breakpoint is below the given breakpoint.
    #[must_use]
    pub fn is_below(&self, other: Self) -> bool {
        self.px() < other.px()
    }
}

/// Direction for layout components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Horizontal layout (row)
    #[default]
    Row,
    /// Vertical layout (column)
    Column,
}

/// Centered container with max-width constraints.
///
/// The container centers its content and applies appropriate max-width
/// based on the current breakpoint. Use this as a wrapper for page content.
///
/// # Props
///
/// - `max_width` - Optional custom max-width (default: responsive based on breakpoint)
/// - `class` - Additional CSS classes
/// - `children` - Child elements
///
/// # Example
///
/// ```ignore
/// Container {
///     class: "py-8",
///     div { "Centered content" }
/// }
/// ```
#[component]
pub fn Container(
    /// Optional custom max-width (e.g., "800px", "100%")
    #[props(default)]
    max_width: Option<String>,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Child elements
    children: Element,
) -> Element {
    let max_w = max_width.unwrap_or_else(|| breakpoints::container::XL.to_string());

    let style = format!(
        "width: 100%; max-width: {}; margin-left: auto; margin-right: auto; padding-left: {}; padding-right: {};",
        max_w,
        spacing::MD,
        spacing::MD
    );

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            {children}
        }
    }
}

/// Vertical or horizontal flex stack with consistent gap.
///
/// Stack is a flexible layout component that arranges children in a
/// vertical (default) or horizontal direction with consistent spacing.
///
/// # Props
///
/// - `direction` - Layout direction (Column or Row)
/// - `gap` - Gap between items (default: spacing::MD)
/// - `align` - Align items (start, center, end, stretch)
/// - `justify` - Justify content (start, center, end, between, around)
/// - `class` - Additional CSS classes
/// - `children` - Child elements
///
/// # Example
///
/// ```ignore
/// Stack {
///     gap: "1rem",
///     div { "Item 1" }
///     div { "Item 2" }
///     div { "Item 3" }
/// }
/// ```
#[component]
pub fn Stack(
    /// Layout direction
    #[props(default)]
    direction: Direction,
    /// Gap between items
    #[props(default)]
    gap: Option<String>,
    /// Align items (CSS align-items value)
    #[props(default)]
    align: Option<String>,
    /// Justify content (CSS justify-content value)
    #[props(default)]
    justify: Option<String>,
    /// Wrap content
    #[props(default)]
    wrap: bool,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Child elements
    children: Element,
) -> Element {
    let flex_direction = match direction {
        Direction::Row => "row",
        Direction::Column => "column",
    };

    let gap_value = gap.unwrap_or_else(|| spacing::MD.to_string());
    let align_value = align.unwrap_or_else(|| "stretch".to_string());
    let justify_value = justify.unwrap_or_else(|| "flex-start".to_string());
    let wrap_value = if wrap { "wrap" } else { "nowrap" };

    let style = format!(
        "display: flex; flex-direction: {}; gap: {}; align-items: {}; justify-content: {}; flex-wrap: {};",
        flex_direction, gap_value, align_value, justify_value, wrap_value
    );

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            {children}
        }
    }
}

/// Horizontal row layout with consistent gap.
///
/// Convenience component for horizontal layouts. Equivalent to
/// `Stack { direction: Direction::Row }`.
///
/// # Example
///
/// ```ignore
/// Row {
///     gap: "0.5rem",
///     align: "center",
///     button { "Cancel" }
///     button { "Save" }
/// }
/// ```
#[component]
pub fn Row(
    /// Gap between items
    #[props(default)]
    gap: Option<String>,
    /// Align items (CSS align-items value)
    #[props(default)]
    align: Option<String>,
    /// Justify content (CSS justify-content value)
    #[props(default)]
    justify: Option<String>,
    /// Wrap content
    #[props(default)]
    wrap: bool,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Child elements
    children: Element,
) -> Element {
    rsx! {
        Stack {
            direction: Direction::Row,
            gap,
            align,
            justify,
            wrap,
            class,
            {children}
        }
    }
}

/// CSS grid layout with configurable columns.
///
/// Creates a CSS grid with the specified number of columns.
/// Columns are equal width by default.
///
/// # Props
///
/// - `cols` - Number of columns (default: 1)
/// - `gap` - Gap between items (default: spacing::MD)
/// - `class` - Additional CSS classes
/// - `children` - Child elements
///
/// # Example
///
/// ```ignore
/// Grid {
///     cols: 3,
///     gap: "1.5rem",
///     div { "Card 1" }
///     div { "Card 2" }
///     div { "Card 3" }
/// }
/// ```
#[component]
pub fn Grid(
    /// Number of columns
    #[props(default = 1)]
    cols: usize,
    /// Gap between items
    #[props(default)]
    gap: Option<String>,
    /// Minimum item width for auto-fit grid (if set, cols is ignored)
    #[props(default)]
    min_item_width: Option<String>,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Child elements
    children: Element,
) -> Element {
    let gap_value = gap.unwrap_or_else(|| spacing::MD.to_string());

    let columns = if let Some(min_width) = min_item_width {
        format!("repeat(auto-fit, minmax({}, 1fr))", min_width)
    } else {
        format!("repeat({}, minmax(0, 1fr))", cols)
    };

    let style = format!(
        "display: grid; grid-template-columns: {}; gap: {};",
        columns, gap_value
    );

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            {children}
        }
    }
}

/// Flexible spacer that fills available space.
///
/// Use Spacer to push elements apart in a flex container.
/// It grows to fill available space while its siblings maintain their size.
///
/// # Example
///
/// ```ignore
/// Row {
///     div { "Left" }
///     Spacer {}
///     div { "Right" }
/// }
/// ```
#[component]
pub fn Spacer() -> Element {
    rsx! {
        div {
            style: "flex: 1 1 auto;",
            "aria-hidden": "true",
        }
    }
}

/// Visual separator line.
///
/// Renders a horizontal or vertical line to visually separate content.
/// Uses the border color token for consistent styling.
///
/// # Props
///
/// - `vertical` - If true, renders a vertical divider (default: false)
/// - `class` - Additional CSS classes
///
/// # Example
///
/// ```ignore
/// Stack {
///     div { "Section 1" }
///     Divider {}
///     div { "Section 2" }
/// }
/// ```
#[component]
pub fn Divider(
    /// Render as vertical divider
    #[props(default)]
    vertical: bool,
    /// Custom margin
    #[props(default)]
    margin: Option<String>,
    /// Additional CSS classes
    #[props(default)]
    class: String,
) -> Element {
    let margin_value = margin.unwrap_or_else(|| spacing::MD.to_string());

    let style = if vertical {
        format!(
            "width: 1px; height: 100%; background-color: {}; margin: 0 {};",
            colors::BORDER_DEFAULT,
            margin_value
        )
    } else {
        format!(
            "height: 1px; width: 100%; background-color: {}; margin: {} 0;",
            colors::BORDER_DEFAULT,
            margin_value
        )
    };

    rsx! {
        div {
            class: "{class}",
            style: "{style}",
            role: "separator",
            "aria-orientation": if vertical { "vertical" } else { "horizontal" },
        }
    }
}

/// Center content horizontally and vertically.
///
/// A convenience component that centers its children both horizontally
/// and vertically using flexbox.
///
/// # Example
///
/// ```ignore
/// Center {
///     class: "h-screen",
///     div { "Centered content" }
/// }
/// ```
#[component]
pub fn Center(
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Child elements
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "{class}",
            style: "display: flex; align-items: center; justify-content: center;",
            {children}
        }
    }
}

/// Aspect ratio container.
///
/// Maintains a specific aspect ratio for its content, useful for
/// images, videos, and embedded content.
///
/// # Props
///
/// - `ratio` - Aspect ratio as width/height (e.g., 16.0/9.0 for 16:9)
/// - `class` - Additional CSS classes
/// - `children` - Child elements
///
/// # Example
///
/// ```ignore
/// AspectRatio {
///     ratio: 16.0 / 9.0,
///     img { src: "video-thumbnail.jpg" }
/// }
/// ```
#[component]
pub fn AspectRatio(
    /// Aspect ratio (width / height)
    #[props(default = 1.0)]
    ratio: f64,
    /// Additional CSS classes
    #[props(default)]
    class: String,
    /// Child elements
    children: Element,
) -> Element {
    let padding_top = 100.0 / ratio;

    let container_style = format!(
        "position: relative; width: 100%; padding-top: {:.4}%;",
        padding_top
    );

    rsx! {
        div {
            class: "{class}",
            style: "{container_style}",
            div {
                style: "position: absolute; top: 0; left: 0; right: 0; bottom: 0;",
                {children}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breakpoint_from_width_works() {
        assert_eq!(Breakpoint::from_width(400), Breakpoint::Xs);
        assert_eq!(Breakpoint::from_width(640), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(768), Breakpoint::Md);
        assert_eq!(Breakpoint::from_width(1024), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_width(1280), Breakpoint::Xl);
        assert_eq!(Breakpoint::from_width(1536), Breakpoint::Xxl);
        assert_eq!(Breakpoint::from_width(2000), Breakpoint::Xxl);
    }

    #[test]
    fn breakpoint_px_values() {
        assert_eq!(Breakpoint::Xs.px(), 0);
        assert_eq!(Breakpoint::Sm.px(), 640);
        assert_eq!(Breakpoint::Md.px(), 768);
        assert_eq!(Breakpoint::Lg.px(), 1024);
        assert_eq!(Breakpoint::Xl.px(), 1280);
        assert_eq!(Breakpoint::Xxl.px(), 1536);
    }

    #[test]
    fn breakpoint_is_at_least() {
        assert!(Breakpoint::Lg.is_at_least(Breakpoint::Md));
        assert!(Breakpoint::Lg.is_at_least(Breakpoint::Lg));
        assert!(!Breakpoint::Md.is_at_least(Breakpoint::Lg));
    }

    #[test]
    fn breakpoint_is_below() {
        assert!(Breakpoint::Sm.is_below(Breakpoint::Md));
        assert!(!Breakpoint::Lg.is_below(Breakpoint::Md));
        assert!(!Breakpoint::Md.is_below(Breakpoint::Md));
    }

    #[test]
    fn direction_default_is_row() {
        assert_eq!(Direction::default(), Direction::Row);
    }

    #[test]
    fn breakpoint_default_is_xs() {
        assert_eq!(Breakpoint::default(), Breakpoint::Xs);
    }
}

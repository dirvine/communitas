//! Split-pane layout component for 3-column UI
//!
//! Provides a responsive 3-column layout system with:
//! - Left sidebar (navigation)
//! - Center content pane (main view)
//! - Right context panel (details)
//!
//! Features:
//! - Fixed and percentage-based widths
//! - Resizable columns (respects min/max constraints)
//! - Toggle column visibility
//! - Responsive layout calculations

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// Column identifier in the 3-column layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Column {
    /// Left sidebar (navigation, lists)
    Sidebar,
    /// Center content pane (main view)
    Content,
    /// Right context panel (details, info)
    Context,
}

/// Width specification for layout columns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnWidth {
    /// Fixed width in terminal cells
    Fixed(u16),
    /// Percentage of available width (0-100)
    Percentage(u16),
    /// Fill remaining space
    Fill,
}

impl ColumnWidth {
    /// Convert to ratatui Constraint
    fn to_constraint(self) -> Constraint {
        match self {
            ColumnWidth::Fixed(width) => Constraint::Length(width),
            ColumnWidth::Percentage(pct) => Constraint::Percentage(pct),
            ColumnWidth::Fill => Constraint::Min(0),
        }
    }

    /// Validate percentage is in range 0-100
    pub fn percentage(pct: u16) -> Result<Self, String> {
        if pct > 100 {
            Err(format!("Percentage must be 0-100, got {}", pct))
        } else {
            Ok(ColumnWidth::Percentage(pct))
        }
    }
}

/// Split-pane layout configuration for the 3-column UI
///
/// # Example
/// ```rust
/// use communitas_tui::components::split_layout::{SplitLayout, ColumnWidth};
/// use ratatui::layout::Rect;
///
/// let layout = SplitLayout::new()
///     .sidebar_width(ColumnWidth::Fixed(20))
///     .content_width(ColumnWidth::Fill)
///     .context_width(ColumnWidth::Percentage(25));
///
/// let area = Rect::new(0, 0, 100, 40);
/// let (sidebar, content, context) = layout.calculate_layout(area);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitLayout {
    sidebar_width: ColumnWidth,
    content_width: ColumnWidth,
    context_width: ColumnWidth,
    sidebar_visible: bool,
    context_visible: bool,
    min_sidebar_width: u16,
    max_sidebar_width: u16,
    min_context_width: u16,
    max_context_width: u16,
}

impl Default for SplitLayout {
    fn default() -> Self {
        Self {
            sidebar_width: ColumnWidth::Fixed(25),
            content_width: ColumnWidth::Fill,
            context_width: ColumnWidth::Fixed(30),
            sidebar_visible: true,
            context_visible: true,
            min_sidebar_width: 15,
            max_sidebar_width: 50,
            min_context_width: 20,
            max_context_width: 60,
        }
    }
}

impl SplitLayout {
    /// Create a new split layout with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set sidebar width
    pub fn sidebar_width(mut self, width: ColumnWidth) -> Self {
        self.sidebar_width = width;
        self
    }

    /// Set content width
    pub fn content_width(mut self, width: ColumnWidth) -> Self {
        self.content_width = width;
        self
    }

    /// Set context panel width
    pub fn context_width(mut self, width: ColumnWidth) -> Self {
        self.context_width = width;
        self
    }

    /// Set sidebar visibility
    pub fn sidebar_visible(mut self, visible: bool) -> Self {
        self.sidebar_visible = visible;
        self
    }

    /// Set context panel visibility
    pub fn context_visible(mut self, visible: bool) -> Self {
        self.context_visible = visible;
        self
    }

    /// Set minimum sidebar width constraint
    pub fn min_sidebar_width(mut self, width: u16) -> Self {
        self.min_sidebar_width = width;
        self
    }

    /// Set maximum sidebar width constraint
    pub fn max_sidebar_width(mut self, width: u16) -> Self {
        self.max_sidebar_width = width;
        self
    }

    /// Set minimum context panel width constraint
    pub fn min_context_width(mut self, width: u16) -> Self {
        self.min_context_width = width;
        self
    }

    /// Set maximum context panel width constraint
    pub fn max_context_width(mut self, width: u16) -> Self {
        self.max_context_width = width;
        self
    }

    /// Toggle sidebar visibility
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    /// Toggle context panel visibility
    pub fn toggle_context(&mut self) {
        self.context_visible = !self.context_visible;
    }

    /// Check if sidebar is visible
    pub fn is_sidebar_visible(&self) -> bool {
        self.sidebar_visible
    }

    /// Check if context panel is visible
    pub fn is_context_visible(&self) -> bool {
        self.context_visible
    }

    /// Resize sidebar to new width (respects min/max constraints)
    pub fn resize_sidebar(&mut self, width: u16) {
        let constrained_width = width.clamp(self.min_sidebar_width, self.max_sidebar_width);
        self.sidebar_width = ColumnWidth::Fixed(constrained_width);
    }

    /// Resize context panel to new width (respects min/max constraints)
    pub fn resize_context(&mut self, width: u16) {
        let constrained_width = width.clamp(self.min_context_width, self.max_context_width);
        self.context_width = ColumnWidth::Fixed(constrained_width);
    }

    /// Calculate layout areas for a given terminal area
    ///
    /// Returns (sidebar_area, content_area, context_area)
    /// Hidden columns will return None for their area
    pub fn calculate_layout(&self, area: Rect) -> (Option<Rect>, Rect, Option<Rect>) {
        let mut constraints = Vec::new();
        let mut has_sidebar = false;
        let mut has_context = false;

        // Sidebar
        if self.sidebar_visible {
            constraints.push(self.sidebar_width.to_constraint());
            has_sidebar = true;
        }

        // Content (always visible)
        constraints.push(self.content_width.to_constraint());

        // Context
        if self.context_visible {
            constraints.push(self.context_width.to_constraint());
            has_context = true;
        }

        let chunks = RatatuiLayout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let sidebar_area = if has_sidebar {
            Some(chunks[0])
        } else {
            None
        };

        let content_idx = if has_sidebar { 1 } else { 0 };
        let content_area = chunks[content_idx];

        let context_area = if has_context {
            Some(chunks[content_idx + 1])
        } else {
            None
        };

        (sidebar_area, content_area, context_area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ColumnWidth Tests ===

    #[test]
    fn test_column_width_fixed() {
        let width = ColumnWidth::Fixed(20);
        assert_eq!(width, ColumnWidth::Fixed(20));
        assert_eq!(width.to_constraint(), Constraint::Length(20));
    }

    #[test]
    fn test_column_width_percentage() {
        let width = ColumnWidth::Percentage(50);
        assert_eq!(width, ColumnWidth::Percentage(50));
        assert_eq!(width.to_constraint(), Constraint::Percentage(50));
    }

    #[test]
    fn test_column_width_fill() {
        let width = ColumnWidth::Fill;
        assert_eq!(width, ColumnWidth::Fill);
        assert_eq!(width.to_constraint(), Constraint::Min(0));
    }

    #[test]
    fn test_column_width_percentage_validation() {
        assert!(ColumnWidth::percentage(0).is_ok());
        assert!(ColumnWidth::percentage(50).is_ok());
        assert!(ColumnWidth::percentage(100).is_ok());
        assert!(ColumnWidth::percentage(101).is_err());
        assert!(ColumnWidth::percentage(200).is_err());
    }

    // === SplitLayout Creation Tests ===

    #[test]
    fn test_split_layout_default() {
        let layout = SplitLayout::default();
        assert_eq!(layout.sidebar_width, ColumnWidth::Fixed(25));
        assert_eq!(layout.content_width, ColumnWidth::Fill);
        assert_eq!(layout.context_width, ColumnWidth::Fixed(30));
        assert!(layout.sidebar_visible);
        assert!(layout.context_visible);
        assert_eq!(layout.min_sidebar_width, 15);
        assert_eq!(layout.max_sidebar_width, 50);
        assert_eq!(layout.min_context_width, 20);
        assert_eq!(layout.max_context_width, 60);
    }

    #[test]
    fn test_split_layout_new() {
        let layout = SplitLayout::new();
        assert_eq!(layout, SplitLayout::default());
    }

    // === Builder Pattern Tests ===

    #[test]
    fn test_layout_builder_sidebar_width() {
        let layout = SplitLayout::new().sidebar_width(ColumnWidth::Fixed(30));
        assert_eq!(layout.sidebar_width, ColumnWidth::Fixed(30));
    }

    #[test]
    fn test_layout_builder_content_width() {
        let layout = SplitLayout::new().content_width(ColumnWidth::Percentage(60));
        assert_eq!(layout.content_width, ColumnWidth::Percentage(60));
    }

    #[test]
    fn test_layout_builder_context_width() {
        let layout = SplitLayout::new().context_width(ColumnWidth::Percentage(25));
        assert_eq!(layout.context_width, ColumnWidth::Percentage(25));
    }

    #[test]
    fn test_layout_builder_sidebar_visibility() {
        let layout = SplitLayout::new().sidebar_visible(false);
        assert!(!layout.sidebar_visible);
    }

    #[test]
    fn test_layout_builder_context_visibility() {
        let layout = SplitLayout::new().context_visible(false);
        assert!(!layout.context_visible);
    }

    #[test]
    fn test_layout_builder_constraints() {
        let layout = SplitLayout::new()
            .min_sidebar_width(20)
            .max_sidebar_width(40)
            .min_context_width(25)
            .max_context_width(50);

        assert_eq!(layout.min_sidebar_width, 20);
        assert_eq!(layout.max_sidebar_width, 40);
        assert_eq!(layout.min_context_width, 25);
        assert_eq!(layout.max_context_width, 50);
    }

    #[test]
    fn test_layout_builder_chain() {
        let layout = SplitLayout::new()
            .sidebar_width(ColumnWidth::Fixed(20))
            .content_width(ColumnWidth::Fill)
            .context_width(ColumnWidth::Percentage(30))
            .sidebar_visible(true)
            .context_visible(false)
            .min_sidebar_width(15)
            .max_sidebar_width(40);

        assert_eq!(layout.sidebar_width, ColumnWidth::Fixed(20));
        assert_eq!(layout.content_width, ColumnWidth::Fill);
        assert_eq!(layout.context_width, ColumnWidth::Percentage(30));
        assert!(layout.sidebar_visible);
        assert!(!layout.context_visible);
        assert_eq!(layout.min_sidebar_width, 15);
        assert_eq!(layout.max_sidebar_width, 40);
    }

    // === Visibility Toggle Tests ===

    #[test]
    fn test_toggle_sidebar() {
        let mut layout = SplitLayout::new();
        assert!(layout.sidebar_visible);

        layout.toggle_sidebar();
        assert!(!layout.sidebar_visible);

        layout.toggle_sidebar();
        assert!(layout.sidebar_visible);
    }

    #[test]
    fn test_toggle_context() {
        let mut layout = SplitLayout::new();
        assert!(layout.context_visible);

        layout.toggle_context();
        assert!(!layout.context_visible);

        layout.toggle_context();
        assert!(layout.context_visible);
    }

    #[test]
    fn test_is_sidebar_visible() {
        let layout = SplitLayout::new().sidebar_visible(true);
        assert!(layout.is_sidebar_visible());

        let layout = SplitLayout::new().sidebar_visible(false);
        assert!(!layout.is_sidebar_visible());
    }

    #[test]
    fn test_is_context_visible() {
        let layout = SplitLayout::new().context_visible(true);
        assert!(layout.is_context_visible());

        let layout = SplitLayout::new().context_visible(false);
        assert!(!layout.is_context_visible());
    }

    // === Resize Tests ===

    #[test]
    fn test_resize_sidebar_within_bounds() {
        let mut layout = SplitLayout::new();
        layout.resize_sidebar(30);
        assert_eq!(layout.sidebar_width, ColumnWidth::Fixed(30));
    }

    #[test]
    fn test_resize_sidebar_below_min() {
        let mut layout = SplitLayout::new().min_sidebar_width(20);
        layout.resize_sidebar(10);
        assert_eq!(layout.sidebar_width, ColumnWidth::Fixed(20)); // Clamped to min
    }

    #[test]
    fn test_resize_sidebar_above_max() {
        let mut layout = SplitLayout::new().max_sidebar_width(40);
        layout.resize_sidebar(60);
        assert_eq!(layout.sidebar_width, ColumnWidth::Fixed(40)); // Clamped to max
    }

    #[test]
    fn test_resize_context_within_bounds() {
        let mut layout = SplitLayout::new();
        layout.resize_context(35);
        assert_eq!(layout.context_width, ColumnWidth::Fixed(35));
    }

    #[test]
    fn test_resize_context_below_min() {
        let mut layout = SplitLayout::new().min_context_width(25);
        layout.resize_context(15);
        assert_eq!(layout.context_width, ColumnWidth::Fixed(25)); // Clamped to min
    }

    #[test]
    fn test_resize_context_above_max() {
        let mut layout = SplitLayout::new().max_context_width(50);
        layout.resize_context(70);
        assert_eq!(layout.context_width, ColumnWidth::Fixed(50)); // Clamped to max
    }

    // === Layout Calculation Tests ===

    #[test]
    fn test_calculate_layout_all_visible() {
        let layout = SplitLayout::new()
            .sidebar_width(ColumnWidth::Fixed(20))
            .context_width(ColumnWidth::Fixed(30));

        let area = Rect::new(0, 0, 100, 40);
        let (sidebar, content, context) = layout.calculate_layout(area);

        assert!(sidebar.is_some());
        assert!(context.is_some());

        let sidebar = sidebar.unwrap();
        let context = context.unwrap();

        assert_eq!(sidebar.width, 20);
        assert_eq!(context.width, 30);
        assert_eq!(content.width, 50); // Remaining space (100 - 20 - 30)
    }

    #[test]
    fn test_calculate_layout_sidebar_hidden() {
        let layout = SplitLayout::new()
            .sidebar_visible(false)
            .context_width(ColumnWidth::Fixed(30));

        let area = Rect::new(0, 0, 100, 40);
        let (sidebar, _content, context) = layout.calculate_layout(area);

        assert!(sidebar.is_none());
        assert!(context.is_some());

        let context = context.unwrap();
        assert_eq!(context.width, 30);
        // Note: content width is 70 (100 - 30)
    }

    #[test]
    fn test_calculate_layout_context_hidden() {
        let layout = SplitLayout::new()
            .sidebar_width(ColumnWidth::Fixed(20))
            .context_visible(false);

        let area = Rect::new(0, 0, 100, 40);
        let (sidebar, content, context) = layout.calculate_layout(area);

        assert!(sidebar.is_some());
        assert!(context.is_none());

        let sidebar = sidebar.unwrap();
        assert_eq!(sidebar.width, 20);
        assert_eq!(content.width, 80); // 100 - 20
    }

    #[test]
    fn test_calculate_layout_both_hidden() {
        let layout = SplitLayout::new()
            .sidebar_visible(false)
            .context_visible(false);

        let area = Rect::new(0, 0, 100, 40);
        let (sidebar, content, context) = layout.calculate_layout(area);

        assert!(sidebar.is_none());
        assert!(context.is_none());
        assert_eq!(content.width, 100); // Full width
    }

    #[test]
    fn test_calculate_layout_percentage_widths() {
        let layout = SplitLayout::new()
            .sidebar_width(ColumnWidth::Percentage(20))
            .context_width(ColumnWidth::Percentage(30));

        let area = Rect::new(0, 0, 100, 40);
        let (sidebar, _content, context) = layout.calculate_layout(area);

        assert!(sidebar.is_some());
        assert!(context.is_some());

        // Note: ratatui percentage calculations may not be exact
        let sidebar = sidebar.unwrap();
        let context = context.unwrap();

        // Approximate percentage checks (ratatui may round)
        assert!(sidebar.width >= 18 && sidebar.width <= 22); // ~20%
        assert!(context.width >= 28 && context.width <= 32); // ~30%
    }

    #[test]
    fn test_calculate_layout_position_offsets() {
        let layout = SplitLayout::new()
            .sidebar_width(ColumnWidth::Fixed(20))
            .context_width(ColumnWidth::Fixed(30));

        let area = Rect::new(0, 0, 100, 40);
        let (sidebar, content, context) = layout.calculate_layout(area);

        let sidebar = sidebar.unwrap();
        let context = context.unwrap();

        // Check x-axis positioning
        assert_eq!(sidebar.x, 0);
        assert_eq!(content.x, 20); // After sidebar
        assert_eq!(context.x, 70); // After sidebar + content
    }
}

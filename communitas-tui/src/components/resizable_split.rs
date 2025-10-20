//! Resizable split panel component
//!
//! Provides draggable dividers for creating custom layouts with:
//! - Vertical/horizontal orientation
//! - Mouse drag to resize
//! - Double-click to reset
//! - Min/max size constraints
//! - Snap-to-grid alignment

use crate::messages::Msg;
use tuirealm::{
    Component, Frame, MockComponent, State,
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    },
};

/// Split panel orientation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    /// Vertical split (left/right panels)
    Vertical,
    /// Horizontal split (top/bottom panels)
    Horizontal,
}

/// Resizable split panel component
#[derive(Debug)]
pub struct ResizableSplit {
    /// Component props for tuirealm integration
    props: Props,

    /// Split orientation (vertical or horizontal)
    orientation: Orientation,

    /// Current divider position as percentage (0-100)
    divider_position: u16,

    /// Default position for reset (0-100)
    default_position: u16,

    /// Minimum panel size as percentage (0-100)
    min_size: u16,

    /// Maximum panel size as percentage (0-100)
    max_size: u16,

    /// Whether the divider is currently being dragged
    dragging: bool,

    /// Starting position when drag began
    drag_start_position: Option<u16>,

    /// Whether mouse is hovering over divider
    hover_over_divider: bool,

    /// Whether snap-to-grid is enabled
    snap_enabled: bool,

    /// Grid size for snapping (percentage points)
    snap_grid_size: u16,

    /// Area where the component is rendered (for hit detection)
    last_render_area: Option<Rect>,
}

impl Default for ResizableSplit {
    fn default() -> Self {
        Self::new()
    }
}

impl ResizableSplit {
    /// Create new resizable split with default settings
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            orientation: Orientation::Vertical,
            divider_position: 50,
            default_position: 50,
            min_size: 10,
            max_size: 90,
            dragging: false,
            drag_start_position: None,
            hover_over_divider: false,
            snap_enabled: true,
            snap_grid_size: 5,
            last_render_area: None,
        }
    }

    /// Create with specific orientation
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set divider position (0-100 percentage)
    pub fn with_position(mut self, position: u16) -> Self {
        self.divider_position = position.clamp(self.min_size, self.max_size);
        self.default_position = self.divider_position;
        self
    }

    /// Set minimum panel size (0-100 percentage)
    pub fn with_min_size(mut self, min_size: u16) -> Self {
        self.min_size = min_size.min(100);
        // Ensure current position respects new minimum
        self.divider_position = self.divider_position.max(self.min_size);
        self
    }

    /// Set maximum panel size (0-100 percentage)
    pub fn with_max_size(mut self, max_size: u16) -> Self {
        self.max_size = max_size.min(100);
        // Ensure current position respects new maximum
        self.divider_position = self.divider_position.min(self.max_size);
        self
    }

    /// Enable or disable snap-to-grid
    pub fn with_snap(mut self, enabled: bool, grid_size: u16) -> Self {
        self.snap_enabled = enabled;
        self.snap_grid_size = grid_size;
        self
    }

    /// Get current divider position
    pub fn position(&self) -> u16 {
        self.divider_position
    }

    /// Get split orientation
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    /// Reset divider to default position
    pub fn reset_position(&mut self) {
        self.divider_position = self.default_position;
    }

    /// Set divider position with constraints applied
    fn set_position(&mut self, position: u16) {
        let mut new_pos = position.clamp(self.min_size, self.max_size);

        // Apply snap-to-grid if enabled
        if self.snap_enabled {
            new_pos = self.snap_to_grid(new_pos);
        }

        self.divider_position = new_pos;
    }

    /// Snap position to grid
    fn snap_to_grid(&self, position: u16) -> u16 {
        let grid = self.snap_grid_size;
        if grid == 0 {
            return position;
        }
        ((position + grid / 2) / grid) * grid
    }

    /// Check if mouse coordinates are over the divider
    fn is_over_divider(&self, x: u16, y: u16, area: Rect) -> bool {
        match self.orientation {
            Orientation::Vertical => {
                // Divider is vertical, check X coordinate
                let divider_x = area.x + (area.width * self.divider_position / 100);
                let tolerance = 2; // Allow ±2 chars for easier grabbing
                x.abs_diff(divider_x) <= tolerance
            }
            Orientation::Horizontal => {
                // Divider is horizontal, check Y coordinate
                let divider_y = area.y + (area.height * self.divider_position / 100);
                let tolerance = 1; // Smaller tolerance for horizontal (fewer rows)
                y.abs_diff(divider_y) <= tolerance
            }
        }
    }

    /// Start dragging the divider
    pub fn start_drag(&mut self, x: u16, y: u16) {
        if let Some(area) = self.last_render_area
            && self.is_over_divider(x, y, area)
        {
            self.dragging = true;
            self.drag_start_position = Some(self.divider_position);
        }
    }

    /// Update drag position
    pub fn update_drag(&mut self, x: u16, y: u16) {
        if !self.dragging {
            return;
        }

        if let Some(area) = self.last_render_area {
            let new_position = match self.orientation {
                Orientation::Vertical => {
                    if area.width == 0 {
                        return;
                    }
                    let relative_x = x.saturating_sub(area.x);
                    ((relative_x as f32 / area.width as f32) * 100.0) as u16
                }
                Orientation::Horizontal => {
                    if area.height == 0 {
                        return;
                    }
                    let relative_y = y.saturating_sub(area.y);
                    ((relative_y as f32 / area.height as f32) * 100.0) as u16
                }
            };

            self.set_position(new_position);
        }
    }

    /// End dragging
    pub fn end_drag(&mut self) {
        self.dragging = false;
        self.drag_start_position = None;
    }

    /// Update hover state
    pub fn update_hover(&mut self, x: u16, y: u16) {
        if let Some(area) = self.last_render_area {
            self.hover_over_divider = self.is_over_divider(x, y, area);
        }
    }

    /// Calculate split layout (left/top and right/bottom areas)
    fn calculate_split(&self, area: Rect) -> (Rect, Rect) {
        match self.orientation {
            Orientation::Vertical => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(self.divider_position),
                        Constraint::Percentage(100 - self.divider_position),
                    ])
                    .split(area);
                (chunks[0], chunks[1])
            }
            Orientation::Horizontal => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(self.divider_position),
                        Constraint::Percentage(100 - self.divider_position),
                    ])
                    .split(area);
                (chunks[0], chunks[1])
            }
        }
    }
}

impl MockComponent for ResizableSplit {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Store render area for hit detection
        self.last_render_area = Some(area);

        // Calculate split areas
        let (left_or_top, right_or_bottom) = self.calculate_split(area);

        // Render left/top panel
        let left_block = Block::default()
            .title("Left/Top Panel")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let left_content = Paragraph::new(Line::from(vec![
            Span::raw("Panel 1\n"),
            Span::styled(
                format!("Size: {}%", self.divider_position),
                Style::default().fg(Color::Gray),
            ),
        ]))
        .block(left_block);

        frame.render_widget(left_content, left_or_top);

        // Render right/bottom panel
        let right_block = Block::default()
            .title("Right/Bottom Panel")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));

        let right_content = Paragraph::new(Line::from(vec![
            Span::raw("Panel 2\n"),
            Span::styled(
                format!("Size: {}%", 100 - self.divider_position),
                Style::default().fg(Color::Gray),
            ),
        ]))
        .block(right_block);

        frame.render_widget(right_content, right_or_bottom);

        // Render divider with visual feedback
        let divider_area = match self.orientation {
            Orientation::Vertical => {
                let divider_x = area.x + (area.width * self.divider_position / 100);
                Rect {
                    x: divider_x.saturating_sub(1),
                    y: area.y,
                    width: 1,
                    height: area.height,
                }
            }
            Orientation::Horizontal => {
                let divider_y = area.y + (area.height * self.divider_position / 100);
                Rect {
                    x: area.x,
                    y: divider_y,
                    width: area.width,
                    height: 1,
                }
            }
        };

        // Visual feedback for divider state
        let divider_style = if self.dragging {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if self.hover_over_divider {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let divider_char = match self.orientation {
            Orientation::Vertical => "│",
            Orientation::Horizontal => "─",
        };

        let divider =
            Paragraph::new(divider_char.repeat(divider_area.width as usize)).style(divider_style);

        frame.render_widget(divider, divider_area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(tuirealm::StateValue::U16(self.divider_position))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Custom("reset") => {
                self.reset_position();
                CmdResult::Submit(State::One(tuirealm::StateValue::U16(self.divider_position)))
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for ResizableSplit {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // Mouse events are handled through public methods (start_drag, update_drag, etc.)
        // This allows the parent component to coordinate mouse handling
        None
    }
}

// Re-export for convenience

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resizable_split_creation() {
        let split = ResizableSplit::new();
        assert_eq!(split.orientation(), Orientation::Vertical);
        assert_eq!(split.position(), 50);
        assert!(!split.dragging);
    }

    #[test]
    fn test_with_orientation() {
        let split = ResizableSplit::new().with_orientation(Orientation::Horizontal);
        assert_eq!(split.orientation(), Orientation::Horizontal);
    }

    #[test]
    fn test_with_position() {
        let split = ResizableSplit::new().with_position(30);
        assert_eq!(split.position(), 30);
        assert_eq!(split.default_position, 30);
    }

    #[test]
    fn test_with_position_respects_min_max() {
        let split = ResizableSplit::new()
            .with_min_size(20)
            .with_max_size(80)
            .with_position(10); // Below min
        assert_eq!(split.position(), 20); // Clamped to min

        let split = ResizableSplit::new()
            .with_min_size(20)
            .with_max_size(80)
            .with_position(90); // Above max
        assert_eq!(split.position(), 80); // Clamped to max
    }

    #[test]
    fn test_with_min_size() {
        let split = ResizableSplit::new().with_position(15).with_min_size(20);
        assert_eq!(split.min_size, 20);
        assert_eq!(split.position(), 20); // Position adjusted to meet minimum
    }

    #[test]
    fn test_with_max_size() {
        let split = ResizableSplit::new().with_position(85).with_max_size(80);
        assert_eq!(split.max_size, 80);
        assert_eq!(split.position(), 80); // Position adjusted to meet maximum
    }

    #[test]
    fn test_with_snap() {
        let split = ResizableSplit::new().with_snap(true, 10);
        assert!(split.snap_enabled);
        assert_eq!(split.snap_grid_size, 10);
    }

    #[test]
    fn test_reset_position() {
        let mut split = ResizableSplit::new().with_position(70);
        split.set_position(40);
        assert_eq!(split.position(), 40);

        split.reset_position();
        assert_eq!(split.position(), 70); // Back to default
    }

    #[test]
    fn test_snap_to_grid() {
        let split = ResizableSplit::new().with_snap(true, 10);

        assert_eq!(split.snap_to_grid(23), 20); // Snaps down
        assert_eq!(split.snap_to_grid(27), 30); // Snaps up
        assert_eq!(split.snap_to_grid(25), 30); // Rounds up on .5
        assert_eq!(split.snap_to_grid(50), 50); // Already on grid
    }

    #[test]
    fn test_snap_to_grid_disabled() {
        let split = ResizableSplit::new().with_snap(false, 10);
        let mut split_mut = split;
        split_mut.set_position(23);
        assert_eq!(split_mut.position(), 23); // No snapping
    }

    #[test]
    fn test_set_position_with_constraints() {
        let mut split = ResizableSplit::new()
            .with_min_size(20)
            .with_max_size(80)
            .with_snap(true, 5);

        split.set_position(15); // Below min
        assert_eq!(split.position(), 20); // Clamped to min

        split.set_position(85); // Above max
        assert_eq!(split.position(), 80); // Clamped to max

        split.set_position(42); // Valid but off-grid
        assert_eq!(split.position(), 40); // Snapped to grid
    }

    #[test]
    fn test_is_over_divider_vertical() {
        let mut split = ResizableSplit::new()
            .with_orientation(Orientation::Vertical)
            .with_position(50);

        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        split.last_render_area = Some(area);

        // Divider is at x=50 (50% of 100)
        assert!(split.is_over_divider(50, 25, area)); // Exactly on divider
        assert!(split.is_over_divider(51, 25, area)); // Within tolerance (+1)
        assert!(split.is_over_divider(49, 25, area)); // Within tolerance (-1)
        assert!(split.is_over_divider(52, 25, area)); // Within tolerance (+2)
        assert!(split.is_over_divider(48, 25, area)); // Within tolerance (-2)
        assert!(!split.is_over_divider(53, 25, area)); // Outside tolerance (+3)
        assert!(!split.is_over_divider(47, 25, area)); // Outside tolerance (-3)
    }

    #[test]
    fn test_is_over_divider_horizontal() {
        let mut split = ResizableSplit::new()
            .with_orientation(Orientation::Horizontal)
            .with_position(50);

        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        split.last_render_area = Some(area);

        // Divider is at y=25 (50% of 50)
        assert!(split.is_over_divider(50, 25, area)); // Exactly on divider
        assert!(split.is_over_divider(50, 26, area)); // Within tolerance (+1)
        assert!(split.is_over_divider(50, 24, area)); // Within tolerance (-1)
        assert!(!split.is_over_divider(50, 27, area)); // Outside tolerance (+2)
        assert!(!split.is_over_divider(50, 23, area)); // Outside tolerance (-2)
    }

    #[test]
    fn test_drag_lifecycle() {
        let mut split = ResizableSplit::new().with_position(50);
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        split.last_render_area = Some(area);

        // Start drag on divider
        assert!(!split.dragging);
        split.start_drag(50, 25); // On divider
        assert!(split.dragging);
        assert_eq!(split.drag_start_position, Some(50));

        // Update drag position
        split.update_drag(60, 25); // Move to 60% position
        assert_eq!(split.position(), 60);

        // End drag
        split.end_drag();
        assert!(!split.dragging);
        assert_eq!(split.drag_start_position, None);
        assert_eq!(split.position(), 60); // Position retained
    }

    #[test]
    fn test_drag_not_started_if_not_over_divider() {
        let mut split = ResizableSplit::new().with_position(50);
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        split.last_render_area = Some(area);

        split.start_drag(10, 25); // Not on divider
        assert!(!split.dragging);
    }

    #[test]
    fn test_update_drag_does_nothing_if_not_dragging() {
        let mut split = ResizableSplit::new().with_position(50);
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        split.last_render_area = Some(area);

        split.update_drag(70, 25);
        assert_eq!(split.position(), 50); // Unchanged
    }

    #[test]
    fn test_update_hover() {
        let mut split = ResizableSplit::new().with_position(50);
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        split.last_render_area = Some(area);

        assert!(!split.hover_over_divider);

        split.update_hover(50, 25); // Over divider
        assert!(split.hover_over_divider);

        split.update_hover(10, 25); // Not over divider
        assert!(!split.hover_over_divider);
    }

    #[test]
    fn test_calculate_split_vertical() {
        let split = ResizableSplit::new()
            .with_orientation(Orientation::Vertical)
            .with_position(30);

        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let (left, right) = split.calculate_split(area);

        // Left panel should be ~30% of width
        assert!(left.width >= 29 && left.width <= 31); // Allow rounding
        // Right panel should be ~70% of width
        assert!(right.width >= 69 && right.width <= 71);
    }

    #[test]
    fn test_calculate_split_horizontal() {
        let split = ResizableSplit::new()
            .with_orientation(Orientation::Horizontal)
            .with_position(40);

        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };

        let (top, bottom) = split.calculate_split(area);

        // Top panel should be ~40% of height
        assert!(top.height >= 19 && top.height <= 21); // Allow rounding
        // Bottom panel should be ~60% of height
        assert!(bottom.height >= 29 && bottom.height <= 31);
    }

    #[test]
    fn test_mock_component_state() {
        let split = ResizableSplit::new().with_position(60);
        let state = split.state();

        match state {
            State::One(tuirealm::StateValue::U16(pos)) => assert_eq!(pos, 60),
            _ => panic!("Expected State::One with U16 value"),
        }
    }

    #[test]
    fn test_mock_component_perform_reset() {
        let mut split = ResizableSplit::new().with_position(70);
        split.set_position(40);
        assert_eq!(split.position(), 40);

        let result = split.perform(Cmd::Custom("reset"));
        assert_eq!(split.position(), 70); // Reset to default

        match result {
            CmdResult::Submit(State::One(tuirealm::StateValue::U16(pos))) => {
                assert_eq!(pos, 70);
            }
            _ => panic!("Expected CmdResult::Submit with new position"),
        }
    }

    #[test]
    fn test_default_impl() {
        let split = ResizableSplit::default();
        assert_eq!(split.orientation(), Orientation::Vertical);
        assert_eq!(split.position(), 50);
    }
}

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Advanced Mouse Event Handling
//!
//! Provides comprehensive mouse interaction support including:
//! - Component hit detection
//! - Hover state tracking with tooltip support
//! - Drag and drop state management
//! - Double-click detection
//! - Scroll wheel handling

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::time::Instant;

/// Represents a rectangular area in the terminal for component hit detection
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentArea {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl ComponentArea {
    /// Create a new component area
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point (x, y) is within this area
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Get the area as a tuple (x, y, width, height)
    pub fn as_tuple(&self) -> (u16, u16, u16, u16) {
        (self.x, self.y, self.width, self.height)
    }

    /// Calculate the center point of this area
    pub fn center(&self) -> (u16, u16) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }
}

/// Tracks hover state for components with tooltip support
#[derive(Debug, Clone, PartialEq)]
pub struct HoverState {
    pub hovered: bool,
    pub hover_position: Option<(u16, u16)>,
    pub hover_duration_ms: u64,
    hover_start: Option<Instant>,
}

impl HoverState {
    /// Create a new hover state (not hovered)
    pub fn new() -> Self {
        Self {
            hovered: false,
            hover_position: None,
            hover_duration_ms: 0,
            hover_start: None,
        }
    }

    /// Called when mouse enters a component
    pub fn on_mouse_enter(&mut self, x: u16, y: u16) {
        self.hovered = true;
        self.hover_position = Some((x, y));
        self.hover_duration_ms = 0;
        self.hover_start = Some(Instant::now());
    }

    /// Called when mouse exits a component
    pub fn on_mouse_exit(&mut self) {
        self.hovered = false;
        self.hover_position = None;
        self.hover_duration_ms = 0;
        self.hover_start = None;
    }

    /// Update hover duration (call periodically)
    pub fn update_duration(&mut self) {
        if let Some(start) = self.hover_start {
            self.hover_duration_ms = start.elapsed().as_millis() as u64;
        }
    }

    /// Check if tooltip should be shown based on duration threshold
    pub fn should_show_tooltip(&self, threshold_ms: u64) -> bool {
        self.hovered && self.hover_duration_ms >= threshold_ms
    }

    /// Get current hover position
    pub fn position(&self) -> Option<(u16, u16)> {
        self.hover_position
    }
}

impl Default for HoverState {
    fn default() -> Self {
        Self::new()
    }
}

/// State machine for drag and drop operations
#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    /// No drag operation in progress
    Idle,
    /// Drag operation in progress
    Dragging {
        start_x: u16,
        start_y: u16,
        current_x: u16,
        current_y: u16,
    },
    /// Drag operation completed
    Dropped {
        start_x: u16,
        start_y: u16,
        end_x: u16,
        end_y: u16,
    },
}

impl DragState {
    /// Create a new idle drag state
    pub fn new() -> Self {
        Self::Idle
    }

    /// Start a drag operation
    pub fn start_drag(&mut self, x: u16, y: u16) {
        *self = Self::Dragging {
            start_x: x,
            start_y: y,
            current_x: x,
            current_y: y,
        };
    }

    /// Update drag position
    pub fn update_drag(&mut self, x: u16, y: u16) {
        if let Self::Dragging {
            start_x, start_y, ..
        } = *self
        {
            *self = Self::Dragging {
                start_x,
                start_y,
                current_x: x,
                current_y: y,
            };
        }
    }

    /// Complete drag operation
    pub fn end_drag(&mut self) {
        if let Self::Dragging {
            start_x,
            start_y,
            current_x,
            current_y,
        } = *self
        {
            *self = Self::Dropped {
                start_x,
                start_y,
                end_x: current_x,
                end_y: current_y,
            };
        }
    }

    /// Reset to idle state
    pub fn reset(&mut self) {
        *self = Self::Idle;
    }

    /// Check if currently dragging
    pub fn is_dragging(&self) -> bool {
        matches!(self, Self::Dragging { .. })
    }

    /// Get drag delta (current - start position)
    pub fn get_drag_delta(&self) -> Option<(i32, i32)> {
        match self {
            Self::Dragging {
                start_x,
                start_y,
                current_x,
                current_y,
            }
            | Self::Dropped {
                start_x,
                start_y,
                end_x: current_x,
                end_y: current_y,
            } => Some((
                *current_x as i32 - *start_x as i32,
                *current_y as i32 - *start_y as i32,
            )),
            Self::Idle => None,
        }
    }

    /// Get start position if dragging or dropped
    pub fn start_position(&self) -> Option<(u16, u16)> {
        match self {
            Self::Dragging {
                start_x, start_y, ..
            }
            | Self::Dropped {
                start_x, start_y, ..
            } => Some((*start_x, *start_y)),
            Self::Idle => None,
        }
    }

    /// Get current position if dragging or dropped
    pub fn current_position(&self) -> Option<(u16, u16)> {
        match self {
            Self::Dragging {
                current_x,
                current_y,
                ..
            } => Some((*current_x, *current_y)),
            Self::Dropped { end_x, end_y, .. } => Some((*end_x, *end_y)),
            Self::Idle => None,
        }
    }
}

impl Default for DragState {
    fn default() -> Self {
        Self::new()
    }
}

/// Detects double-click events
#[derive(Debug, Clone)]
pub struct DoubleClickDetector {
    last_click: Option<Instant>,
    last_position: Option<(u16, u16)>,
    threshold_ms: u64,
    position_tolerance: u16,
}

impl DoubleClickDetector {
    /// Create a new double-click detector with default thresholds
    /// - threshold_ms: Maximum time between clicks (default: 500ms)
    /// - position_tolerance: Maximum pixel distance between clicks (default: 2)
    pub fn new(threshold_ms: u64, position_tolerance: u16) -> Self {
        Self {
            last_click: None,
            last_position: None,
            threshold_ms,
            position_tolerance,
        }
    }

    /// Register a click and check if it's a double-click
    pub fn register_click(&mut self, x: u16, y: u16) -> bool {
        let now = Instant::now();
        let is_double_click = if let (Some(last_click), Some((last_x, last_y))) =
            (self.last_click, self.last_position)
        {
            let time_delta = now.duration_since(last_click).as_millis() as u64;
            let distance =
                ((x as i32 - last_x as i32).abs() + (y as i32 - last_y as i32).abs()) as u16;

            time_delta <= self.threshold_ms && distance <= self.position_tolerance
        } else {
            false
        };

        self.last_click = Some(now);
        self.last_position = Some((x, y));

        is_double_click
    }

    /// Reset the detector state
    pub fn reset(&mut self) {
        self.last_click = None;
        self.last_position = None;
    }
}

impl Default for DoubleClickDetector {
    fn default() -> Self {
        Self::new(500, 2)
    }
}

/// Tracks scroll state for components
#[derive(Debug, Clone, PartialEq)]
pub struct ScrollState {
    pub scroll_offset: i32,
    pub content_height: u32,
    pub viewport_height: u32,
}

impl ScrollState {
    /// Create a new scroll state
    pub fn new(content_height: u32, viewport_height: u32) -> Self {
        Self {
            scroll_offset: 0,
            content_height,
            viewport_height,
        }
    }

    /// Scroll by a delta amount (positive = down, negative = up)
    pub fn scroll_by(&mut self, delta: i32) {
        self.scroll_offset = (self.scroll_offset + delta).max(0);
        self.clamp_offset();
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.max_scroll_offset();
    }

    /// Get maximum scroll offset
    pub fn max_scroll_offset(&self) -> i32 {
        (self.content_height.saturating_sub(self.viewport_height)) as i32
    }

    /// Clamp scroll offset to valid range
    fn clamp_offset(&mut self) {
        let max_offset = self.max_scroll_offset();
        if self.scroll_offset > max_offset {
            self.scroll_offset = max_offset;
        }
    }

    /// Check if scrolled to top
    pub fn is_at_top(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Check if scrolled to bottom
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset >= self.max_scroll_offset()
    }

    /// Update content or viewport height
    pub fn update_dimensions(&mut self, content_height: u32, viewport_height: u32) {
        self.content_height = content_height;
        self.viewport_height = viewport_height;
        self.clamp_offset();
    }
}

/// Enhanced mouse event classification
#[derive(Debug, Clone, PartialEq)]
pub enum EnhancedMouseEvent {
    LeftClick {
        x: u16,
        y: u16,
    },
    RightClick {
        x: u16,
        y: u16,
    },
    MiddleClick {
        x: u16,
        y: u16,
    },
    DoubleClick {
        x: u16,
        y: u16,
    },
    Hover {
        x: u16,
        y: u16,
    },
    DragStart {
        x: u16,
        y: u16,
    },
    Dragging {
        x: u16,
        y: u16,
        start_x: u16,
        start_y: u16,
    },
    DragEnd {
        x: u16,
        y: u16,
        start_x: u16,
        start_y: u16,
    },
    ScrollUp {
        x: u16,
        y: u16,
    },
    ScrollDown {
        x: u16,
        y: u16,
    },
    MouseUp {
        x: u16,
        y: u16,
        button: MouseButton,
    },
}

/// Classify raw mouse events into enhanced events
pub fn classify_mouse_event(
    event: MouseEvent,
    drag_state: &DragState,
) -> Option<EnhancedMouseEvent> {
    let x = event.column;
    let y = event.row;

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => Some(EnhancedMouseEvent::LeftClick { x, y }),
        MouseEventKind::Down(MouseButton::Right) => Some(EnhancedMouseEvent::RightClick { x, y }),
        MouseEventKind::Down(MouseButton::Middle) => Some(EnhancedMouseEvent::MiddleClick { x, y }),
        MouseEventKind::Moved => {
            if drag_state.is_dragging() {
                if let Some((start_x, start_y)) = drag_state.start_position() {
                    Some(EnhancedMouseEvent::Dragging {
                        x,
                        y,
                        start_x,
                        start_y,
                    })
                } else {
                    Some(EnhancedMouseEvent::Hover { x, y })
                }
            } else {
                Some(EnhancedMouseEvent::Hover { x, y })
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let Some((start_x, start_y)) = drag_state.start_position() {
                Some(EnhancedMouseEvent::Dragging {
                    x,
                    y,
                    start_x,
                    start_y,
                })
            } else {
                Some(EnhancedMouseEvent::DragStart { x, y })
            }
        }
        MouseEventKind::Up(button) => Some(EnhancedMouseEvent::MouseUp { x, y, button }),
        MouseEventKind::ScrollUp => Some(EnhancedMouseEvent::ScrollUp { x, y }),
        MouseEventKind::ScrollDown => Some(EnhancedMouseEvent::ScrollDown { x, y }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_area_contains() {
        let area = ComponentArea::new(10, 10, 20, 10);

        // Inside
        assert!(area.contains(15, 15));
        assert!(area.contains(10, 10)); // Top-left corner
        assert!(area.contains(29, 19)); // Bottom-right corner (exclusive)

        // Outside
        assert!(!area.contains(9, 15)); // Left of area
        assert!(!area.contains(30, 15)); // Right of area
        assert!(!area.contains(15, 9)); // Above area
        assert!(!area.contains(15, 20)); // Below area
    }

    #[test]
    fn test_hover_state_transitions() {
        let mut state = HoverState::new();

        assert!(!state.hovered);
        assert_eq!(state.hover_position, None);

        state.on_mouse_enter(10, 10);
        assert!(state.hovered);
        assert_eq!(state.hover_position, Some((10, 10)));

        state.on_mouse_exit();
        assert!(!state.hovered);
        assert_eq!(state.hover_position, None);
    }

    #[test]
    fn test_drag_state_lifecycle() {
        let mut state = DragState::new();

        assert_eq!(state, DragState::Idle);
        assert!(!state.is_dragging());

        state.start_drag(10, 10);
        assert!(state.is_dragging());
        assert_eq!(state.start_position(), Some((10, 10)));

        state.update_drag(15, 15);
        assert!(state.is_dragging());
        assert_eq!(state.get_drag_delta(), Some((5, 5)));

        state.end_drag();
        assert!(!state.is_dragging());
        assert_eq!(state.get_drag_delta(), Some((5, 5)));

        state.reset();
        assert_eq!(state, DragState::Idle);
    }

    #[test]
    fn test_scroll_state_scrolling() {
        let mut state = ScrollState::new(100, 20);

        assert_eq!(state.scroll_offset, 0);
        assert!(state.is_at_top());

        state.scroll_by(10);
        assert_eq!(state.scroll_offset, 10);
        assert!(!state.is_at_top());
        assert!(!state.is_at_bottom());

        state.scroll_to_bottom();
        assert!(state.is_at_bottom());
        assert_eq!(state.scroll_offset, 80);

        state.scroll_to_top();
        assert!(state.is_at_top());
        assert_eq!(state.scroll_offset, 0);
    }
}

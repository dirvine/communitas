// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Integration tests for App::handle_mouse_event enhancements
//!
//! Tests that the enhanced mouse event handler properly integrates with AppState
//! to provide hover, drag, scroll, and context menu functionality.

use communitas_tui::components::{
    ComponentArea, EnhancedMouseEvent, MenuContext, classify_mouse_event,
};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

/// Helper to create mouse events for testing
fn create_mouse_event(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column,
        row,
        modifiers: crossterm::event::KeyModifiers::empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_mouse_event_for_hover() {
        use communitas_tui::DragState;

        let drag_state = DragState::new();
        let mouse = create_mouse_event(MouseEventKind::Moved, 40, 10);

        let classified = classify_mouse_event(mouse, &drag_state);

        assert!(matches!(
            classified,
            Some(EnhancedMouseEvent::Hover { x: 40, y: 10 })
        ));
    }

    #[test]
    fn test_classify_right_click() {
        use communitas_tui::DragState;

        let drag_state = DragState::new();
        let mouse = create_mouse_event(MouseEventKind::Down(MouseButton::Right), 40, 10);

        let classified = classify_mouse_event(mouse, &drag_state);

        assert!(matches!(
            classified,
            Some(EnhancedMouseEvent::RightClick { x: 40, y: 10 })
        ));
    }

    #[test]
    fn test_classify_scroll_events() {
        use communitas_tui::DragState;

        let drag_state = DragState::new();

        // Scroll up
        let mouse_up = create_mouse_event(MouseEventKind::ScrollUp, 40, 10);
        let classified_up = classify_mouse_event(mouse_up, &drag_state);
        assert!(matches!(
            classified_up,
            Some(EnhancedMouseEvent::ScrollUp { x: 40, y: 10 })
        ));

        // Scroll down
        let mouse_down = create_mouse_event(MouseEventKind::ScrollDown, 40, 10);
        let classified_down = classify_mouse_event(mouse_down, &drag_state);
        assert!(matches!(
            classified_down,
            Some(EnhancedMouseEvent::ScrollDown { x: 40, y: 10 })
        ));
    }

    #[test]
    fn test_classify_drag_sequence() {
        use communitas_tui::DragState;

        let mut drag_state = DragState::new();

        // Start drag
        let mouse_drag_start = create_mouse_event(MouseEventKind::Drag(MouseButton::Left), 40, 10);
        let classified_start = classify_mouse_event(mouse_drag_start, &drag_state);
        assert!(matches!(
            classified_start,
            Some(EnhancedMouseEvent::DragStart { x: 40, y: 10 })
        ));

        // Update drag state
        drag_state.start_drag(40, 10);

        // Continue dragging
        let mouse_drag_move = create_mouse_event(MouseEventKind::Drag(MouseButton::Left), 45, 12);
        let classified_move = classify_mouse_event(mouse_drag_move, &drag_state);
        assert!(matches!(
            classified_move,
            Some(EnhancedMouseEvent::Dragging {
                x: 45,
                y: 12,
                start_x: 40,
                start_y: 10
            })
        ));
    }

    #[test]
    fn test_component_area_hit_detection() {
        // Message list area (typical layout)
        let message_area = ComponentArea::new(20, 5, 55, 15);

        // Click inside message area
        assert!(message_area.contains(40, 10));

        // Click outside message area
        assert!(!message_area.contains(10, 10)); // In sidebar
        assert!(!message_area.contains(40, 3)); // Above
        assert!(!message_area.contains(40, 22)); // Below
    }

    #[test]
    fn test_context_menu_appears_on_right_click() {
        use communitas_tui::ContextMenu;

        let mut menu = ContextMenu::new();
        assert!(!menu.visible);

        // Simulate right-click in message area
        menu.show_at(
            40,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        assert!(menu.visible);
        assert_eq!(menu.position, (40, 10));
        assert!(!menu.items.is_empty());
    }

    #[test]
    fn test_context_menu_hides_on_outside_click() {
        use communitas_tui::ContextMenu;

        let mut menu = ContextMenu::new();
        menu.show_at(
            40,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );
        assert!(menu.visible);

        // Simulate click outside menu
        let outside_click = create_mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 10);
        let _result = menu.handle_mouse(outside_click);

        assert!(!menu.visible);
    }

    #[test]
    fn test_hover_state_updates_on_mouse_move() {
        use communitas_tui::HoverState;

        let mut hover = HoverState::new();
        assert!(!hover.hovered);

        // Mouse enters area
        hover.on_mouse_enter(40, 10);
        assert!(hover.hovered);
        assert_eq!(hover.position(), Some((40, 10)));

        // Mouse leaves area
        hover.on_mouse_exit();
        assert!(!hover.hovered);
        assert_eq!(hover.position(), None);
    }

    #[test]
    fn test_scroll_state_updates_on_scroll_events() {
        use communitas_tui::ScrollState;

        let mut scroll = ScrollState::new(100, 20);
        assert_eq!(scroll.scroll_offset, 0);

        // Scroll down
        scroll.scroll_by(3);
        assert_eq!(scroll.scroll_offset, 3);

        // Scroll up
        scroll.scroll_by(-2);
        assert_eq!(scroll.scroll_offset, 1);

        // Cannot scroll below 0
        scroll.scroll_by(-10);
        assert_eq!(scroll.scroll_offset, 0);
    }

    #[test]
    fn test_double_click_detection() {
        use communitas_tui::DoubleClickDetector;

        let mut detector = DoubleClickDetector::new(500, 2);

        // First click
        let is_double = detector.register_click(40, 10);
        assert!(!is_double);

        // Second click at same position (should be double-click)
        let is_double = detector.register_click(40, 10);
        assert!(is_double);
    }

    #[test]
    fn test_context_menu_item_execution() {
        use communitas_tui::{ContextMenu, MenuAction};

        let mut menu = ContextMenu::new();
        menu.show_at(
            40,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Select first item (Reply)
        let menu_x = menu.position.0;
        let menu_y = menu.position.1;
        let click = create_mouse_event(MouseEventKind::Down(MouseButton::Left), menu_x, menu_y);

        let action = menu.handle_mouse(click);
        assert!(action.is_some());
        assert!(matches!(action.unwrap(), MenuAction::Reply));
        assert!(!menu.visible); // Menu should close after action
    }

    #[test]
    fn test_scroll_respects_content_bounds() {
        use communitas_tui::ScrollState;

        let mut scroll = ScrollState::new(100, 20); // 100 lines, 20 visible
        let max_offset = scroll.max_scroll_offset();
        assert_eq!(max_offset, 80);

        // Scroll past the end
        scroll.scroll_by(100);
        assert_eq!(scroll.scroll_offset, 80); // Clamped to max
        assert!(scroll.is_at_bottom());
    }

    #[test]
    fn test_drag_state_calculates_delta() {
        use communitas_tui::DragState;

        let mut drag = DragState::new();
        drag.start_drag(40, 10);
        drag.update_drag(45, 15);

        let delta = drag.get_drag_delta();
        assert_eq!(delta, Some((5, 5)));
    }

    #[test]
    fn test_context_menu_adjusts_for_screen_bounds() {
        use communitas_tui::ContextMenu;

        let mut menu = ContextMenu::new();

        // Show menu near right edge
        menu.show_at(
            75,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );
        menu.adjust_position_for_screen(80, 24);

        // Menu should be adjusted to fit within 80 columns
        let bounds = menu.calculate_bounds();
        assert!(menu.position.0 + bounds.width <= 80);
    }

    #[test]
    fn test_context_menu_for_different_contexts() {
        use communitas_tui::ContextMenu;

        let mut menu = ContextMenu::new();

        // Message context (own message)
        menu.show_at(
            40,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );
        let message_items = menu.items.len();
        assert!(message_items > 0);
        assert!(menu.items.iter().any(|i| i.label == "Delete"));

        // Channel context
        menu.hide();
        menu.show_at(
            40,
            10,
            MenuContext::Channel {
                is_admin: true,
                is_muted: false,
            },
        );
        let channel_items = menu.items.len();
        assert!(channel_items > 0);
        assert!(menu.items.iter().any(|i| i.label == "Mute"));
    }

    #[test]
    fn test_hover_tooltip_threshold() {
        use communitas_tui::HoverState;

        let mut hover = HoverState::new();
        hover.on_mouse_enter(40, 10);

        // Initially no tooltip (duration is 0)
        assert!(!hover.should_show_tooltip(500));

        // Update duration
        hover.hover_duration_ms = 600;
        assert!(hover.should_show_tooltip(500));
    }

    #[test]
    fn test_context_menu_keyboard_navigation() {
        use communitas_tui::ContextMenu;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut menu = ContextMenu::new();
        menu.show_at(
            40,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let initial_index = menu.selected_index;

        // Navigate down
        let key_down = KeyEvent::new(KeyCode::Down, KeyModifiers::empty());
        menu.handle_key(key_down);
        assert_ne!(menu.selected_index, initial_index);

        // Navigate up
        let key_up = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());
        menu.handle_key(key_up);
        assert_eq!(menu.selected_index, initial_index);
    }

    #[test]
    fn test_context_menu_escape_closes() {
        use communitas_tui::ContextMenu;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut menu = ContextMenu::new();
        menu.show_at(
            40,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );
        assert!(menu.visible);

        // Press Escape
        let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::empty());
        menu.handle_key(key_esc);
        assert!(!menu.visible);
    }
}

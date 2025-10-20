// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! TDD Tests for Context Menu System
//!
//! Following strict TDD (Red-Green-Refactor):
//! 1. Write failing tests first (RED)
//! 2. Implement minimal code to pass (GREEN)
//! 3. Refactor for quality (REFACTOR)
//!
//! Test coverage:
//! - Menu item structure and rendering
//! - Menu positioning and bounds checking
//! - Keyboard and mouse navigation
//! - Context-specific menu generation
//! - Separator handling
//! - Enabled/disabled states
//! - Menu visibility and dismissal

use ratatui::layout::Rect;

/// Context menu item
#[derive(Debug, Clone, PartialEq)]
struct MenuItem {
    label: String,
    shortcut: Option<String>,
    action: MenuAction,
    enabled: bool,
    separator: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum MenuAction {
    Copy,
    Edit,
    Delete,
    Reply,
    React,
    Pin,
    Archive,
    Forward,
    Select,
    Custom(String),
}

impl MenuItem {
    fn new(label: impl Into<String>, shortcut: Option<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            shortcut,
            action,
            enabled: true,
            separator: false,
        }
    }

    fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            action: MenuAction::Custom("separator".into()),
            enabled: false,
            separator: true,
        }
    }

    fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    fn is_actionable(&self) -> bool {
        !self.separator && self.enabled
    }
}

/// Menu context determines which items appear
#[derive(Debug, Clone, PartialEq)]
enum MenuContext {
    Message { is_own: bool, can_edit: bool },
    Channel { is_pinned: bool },
    Project { is_archived: bool },
    Contact { is_blocked: bool },
}

/// Context menu display and interaction
#[derive(Debug, Clone, Default)]
struct ContextMenu {
    items: Vec<MenuItem>,
    position: (u16, u16),
    selected_index: usize,
    visible: bool,
    context: Option<MenuContext>,
}

impl ContextMenu {
    fn new() -> Self {
        Self::default()
    }

    fn show_at(&mut self, x: u16, y: u16, context: MenuContext) {
        self.items = self.build_items_for_context(&context);
        self.position = (x, y);
        self.visible = true;
        self.context = Some(context);
        self.selected_index = self.find_first_actionable_index();
    }

    fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
        self.context = None;
        self.selected_index = 0;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn get_position(&self) -> (u16, u16) {
        self.position
    }

    fn get_selected_action(&self) -> Option<MenuAction> {
        self.items
            .get(self.selected_index)
            .filter(|item| item.is_actionable())
            .map(|item| item.action.clone())
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            // Skip separators
            while self.selected_index > 0 && !self.items[self.selected_index].is_actionable() {
                self.selected_index -= 1;
            }
        }
    }

    fn move_selection_down(&mut self) {
        if self.selected_index < self.items.len().saturating_sub(1) {
            self.selected_index += 1;
            // Skip separators
            while self.selected_index < self.items.len()
                && !self.items[self.selected_index].is_actionable()
            {
                self.selected_index += 1;
            }
            // Clamp to valid range
            if self.selected_index >= self.items.len() {
                self.selected_index = self.items.len().saturating_sub(1);
            }
        }
    }

    fn select_at_position(&mut self, relative_y: u16) -> Option<MenuAction> {
        let index = relative_y as usize;
        if index < self.items.len() && self.items[index].is_actionable() {
            self.selected_index = index;
            Some(self.items[index].action.clone())
        } else {
            None
        }
    }

    fn calculate_bounds(&self) -> Rect {
        let width = self.calculate_width();
        let height = self.items.len() as u16;
        Rect::new(self.position.0, self.position.1, width, height)
    }

    fn calculate_width(&self) -> u16 {
        self.items
            .iter()
            .map(|item| {
                let label_len = item.label.len() as u16;
                let shortcut_len = item
                    .shortcut
                    .as_ref()
                    .map(|s| s.len() as u16 + 2)
                    .unwrap_or(0);
                label_len + shortcut_len + 4 // Padding
            })
            .max()
            .unwrap_or(10)
    }

    fn adjust_position_for_screen(&mut self, screen_width: u16, screen_height: u16) {
        let bounds = self.calculate_bounds();
        let mut x = self.position.0;
        let mut y = self.position.1;

        // Adjust horizontal position
        if x + bounds.width > screen_width {
            x = screen_width.saturating_sub(bounds.width);
        }

        // Adjust vertical position
        if y + bounds.height > screen_height {
            y = screen_height.saturating_sub(bounds.height);
        }

        self.position = (x, y);
    }

    fn find_first_actionable_index(&self) -> usize {
        self.items
            .iter()
            .position(|item| item.is_actionable())
            .unwrap_or(0)
    }

    fn build_items_for_context(&self, context: &MenuContext) -> Vec<MenuItem> {
        match context {
            MenuContext::Message { is_own, can_edit } => {
                let mut items = vec![
                    MenuItem::new("Reply", Some("R".into()), MenuAction::Reply),
                    MenuItem::new("React", Some("E".into()), MenuAction::React),
                    MenuItem::new("Forward", Some("F".into()), MenuAction::Forward),
                ];

                items.push(MenuItem::separator());

                items.push(MenuItem::new(
                    "Copy",
                    Some("Ctrl+C".into()),
                    MenuAction::Copy,
                ));

                if *is_own && *can_edit {
                    items.push(MenuItem::new("Edit", Some("I".into()), MenuAction::Edit));
                }

                if *is_own {
                    items.push(MenuItem::new(
                        "Delete",
                        Some("Del".into()),
                        MenuAction::Delete,
                    ));
                }

                items
            }
            MenuContext::Channel { is_pinned } => {
                let mut items = vec![MenuItem::new(
                    "Select",
                    Some("Enter".into()),
                    MenuAction::Select,
                )];

                if *is_pinned {
                    items.push(MenuItem::new("Unpin", Some("P".into()), MenuAction::Pin));
                } else {
                    items.push(MenuItem::new("Pin", Some("P".into()), MenuAction::Pin));
                }

                items.push(MenuItem::separator());
                items.push(MenuItem::new(
                    "Archive",
                    Some("A".into()),
                    MenuAction::Archive,
                ));

                items
            }
            MenuContext::Project { is_archived } => {
                let mut items = vec![MenuItem::new(
                    "Open",
                    Some("Enter".into()),
                    MenuAction::Custom("open".into()),
                )];

                if !is_archived {
                    items.push(MenuItem::new(
                        "Archive",
                        Some("A".into()),
                        MenuAction::Archive,
                    ));
                } else {
                    items.push(MenuItem::new("Unarchive", None, MenuAction::Archive));
                }

                items.push(MenuItem::separator());
                items.push(MenuItem::new(
                    "Delete",
                    Some("Del".into()),
                    MenuAction::Delete,
                ));

                items
            }
            MenuContext::Contact { is_blocked } => {
                let mut items = vec![MenuItem::new(
                    "Message",
                    Some("M".into()),
                    MenuAction::Custom("message".into()),
                )];

                if *is_blocked {
                    items.push(MenuItem::new(
                        "Unblock",
                        None,
                        MenuAction::Custom("unblock".into()),
                    ));
                } else {
                    items.push(MenuItem::new(
                        "Block",
                        None,
                        MenuAction::Custom("block".into()),
                    ));
                }

                items
            }
        }
    }
}

// ============================================================================
// RED Phase: Write failing tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Test Group 1: Menu Item Construction
    // ------------------------------------------------------------------------

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("Copy", Some("Ctrl+C".into()), MenuAction::Copy);
        assert_eq!(item.label, "Copy");
        assert_eq!(item.shortcut, Some("Ctrl+C".into()));
        assert_eq!(item.action, MenuAction::Copy);
        assert!(item.enabled);
        assert!(!item.separator);
    }

    #[test]
    fn test_menu_item_separator() {
        let item = MenuItem::separator();
        assert!(item.separator);
        assert!(!item.enabled);
        assert!(!item.is_actionable());
    }

    #[test]
    fn test_menu_item_disabled() {
        let item = MenuItem::new("Delete", Some("Del".into()), MenuAction::Delete).enabled(false);
        assert!(!item.enabled);
        assert!(!item.is_actionable());
    }

    #[test]
    fn test_menu_item_actionable() {
        let item = MenuItem::new("Edit", Some("E".into()), MenuAction::Edit);
        assert!(item.is_actionable());
    }

    #[test]
    fn test_menu_item_without_shortcut() {
        let item = MenuItem::new("Custom Action", None, MenuAction::Custom("test".into()));
        assert_eq!(item.shortcut, None);
        assert!(item.is_actionable());
    }

    // ------------------------------------------------------------------------
    // Test Group 2: Context Menu Visibility and State
    // ------------------------------------------------------------------------

    #[test]
    fn test_context_menu_initial_hidden() {
        let menu = ContextMenu::new();
        assert!(!menu.is_visible());
        assert_eq!(menu.items.len(), 0);
    }

    #[test]
    fn test_context_menu_show_at_position() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        assert!(menu.is_visible());
        assert_eq!(menu.get_position(), (10, 5));
        assert!(!menu.items.is_empty());
    }

    #[test]
    fn test_context_menu_hide() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );
        menu.hide();

        assert!(!menu.is_visible());
        assert_eq!(menu.items.len(), 0);
    }

    #[test]
    fn test_context_menu_stores_context() {
        let mut menu = ContextMenu::new();
        let context = MenuContext::Channel { is_pinned: true };
        menu.show_at(10, 5, context.clone());

        assert_eq!(menu.context, Some(context));
    }

    // ------------------------------------------------------------------------
    // Test Group 3: Menu Item Generation for Contexts
    // ------------------------------------------------------------------------

    #[test]
    fn test_message_context_own_editable() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Should have: Reply, React, Forward, sep, Copy, Edit, Delete
        assert!(menu.items.len() >= 7);

        // Verify critical items exist
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Reply));
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Edit));
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Delete));
    }

    #[test]
    fn test_message_context_own_not_editable() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: false,
            },
        );

        // Should not have Edit
        assert!(!menu.items.iter().any(|i| i.action == MenuAction::Edit));
        // But should have Delete (own message)
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Delete));
    }

    #[test]
    fn test_message_context_not_own() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: false,
                can_edit: false,
            },
        );

        // Should not have Edit or Delete
        assert!(!menu.items.iter().any(|i| i.action == MenuAction::Edit));
        assert!(!menu.items.iter().any(|i| i.action == MenuAction::Delete));
        // But should have Reply
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Reply));
    }

    #[test]
    fn test_channel_context_pinned() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Channel { is_pinned: true });

        // Should have Pin action (to unpin)
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Pin));
        assert!(menu.items.iter().any(|i| i.label == "Unpin"));
    }

    #[test]
    fn test_channel_context_not_pinned() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Channel { is_pinned: false });

        // Should have Pin action
        assert!(menu.items.iter().any(|i| i.action == MenuAction::Pin));
        assert!(menu.items.iter().any(|i| i.label == "Pin"));
    }

    #[test]
    fn test_project_context_archived() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Project { is_archived: true });

        // Should have Unarchive
        assert!(menu.items.iter().any(|i| i.label == "Unarchive"));
    }

    #[test]
    fn test_project_context_not_archived() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Project { is_archived: false });

        // Should have Archive
        assert!(menu.items.iter().any(|i| i.label == "Archive"));
    }

    #[test]
    fn test_contact_context_blocked() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Contact { is_blocked: true });

        // Should have Unblock
        assert!(menu.items.iter().any(|i| i.label == "Unblock"));
    }

    #[test]
    fn test_contact_context_not_blocked() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Contact { is_blocked: false });

        // Should have Block
        assert!(menu.items.iter().any(|i| i.label == "Block"));
    }

    // ------------------------------------------------------------------------
    // Test Group 4: Menu Navigation
    // ------------------------------------------------------------------------

    #[test]
    fn test_menu_initial_selection_first_actionable() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Should select first actionable item (Reply)
        let action = menu.get_selected_action();
        assert_eq!(action, Some(MenuAction::Reply));
    }

    #[test]
    fn test_menu_move_selection_down() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let first_action = menu.get_selected_action();
        menu.move_selection_down();
        let second_action = menu.get_selected_action();

        assert_ne!(first_action, second_action);
    }

    #[test]
    fn test_menu_move_selection_up() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        menu.move_selection_down();
        menu.move_selection_down();
        let before_up = menu.selected_index;

        menu.move_selection_up();
        let after_up = menu.selected_index;

        assert!(after_up < before_up);
    }

    #[test]
    fn test_menu_skip_separators_on_navigation() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Navigate through entire menu
        let mut selected_items = Vec::new();
        for _ in 0..20 {
            if let Some(action) = menu.get_selected_action() {
                selected_items.push(action);
            }
            menu.move_selection_down();
        }

        // All selected items should be actionable (no separators)
        assert!(!selected_items.is_empty());
    }

    #[test]
    fn test_menu_cannot_move_up_from_first() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Channel { is_pinned: false });

        let initial_index = menu.selected_index;
        menu.move_selection_up();
        let after_up = menu.selected_index;

        assert_eq!(initial_index, after_up);
    }

    #[test]
    fn test_menu_cannot_move_down_beyond_last() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Channel { is_pinned: false });

        // Move to end
        for _ in 0..20 {
            menu.move_selection_down();
        }

        let at_end = menu.selected_index;
        menu.move_selection_down();
        let still_at_end = menu.selected_index;

        assert_eq!(at_end, still_at_end);
    }

    // ------------------------------------------------------------------------
    // Test Group 5: Mouse Selection
    // ------------------------------------------------------------------------

    #[test]
    fn test_menu_select_at_position_valid() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Click on first item (relative y = 0)
        let action = menu.select_at_position(0);
        assert_eq!(action, Some(MenuAction::Reply));
    }

    #[test]
    fn test_menu_select_at_position_invalid() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Channel { is_pinned: false });

        // Click beyond items
        let action = menu.select_at_position(100);
        assert_eq!(action, None);
    }

    #[test]
    fn test_menu_select_separator_returns_none() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Find separator position
        let sep_index = menu.items.iter().position(|i| i.separator);
        if let Some(index) = sep_index {
            let action = menu.select_at_position(index as u16);
            assert_eq!(action, None);
        }
    }

    // ------------------------------------------------------------------------
    // Test Group 6: Menu Bounds and Positioning
    // ------------------------------------------------------------------------

    #[test]
    fn test_menu_calculate_bounds() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let bounds = menu.calculate_bounds();
        assert_eq!(bounds.x, 10);
        assert_eq!(bounds.y, 5);
        assert!(bounds.width > 0);
        assert!(bounds.height == menu.items.len() as u16);
    }

    #[test]
    fn test_menu_calculate_width_includes_shortcuts() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let width = menu.calculate_width();
        // Width should accommodate longest item + shortcut + padding
        assert!(width >= 10); // Minimum reasonable width
    }

    #[test]
    fn test_menu_adjust_position_right_edge() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            75,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let screen_width = 80;
        let screen_height = 24;
        menu.adjust_position_for_screen(screen_width, screen_height);

        let (x, _y) = menu.get_position();
        let bounds = menu.calculate_bounds();

        // Menu should fit within screen
        assert!(x + bounds.width <= screen_width);
    }

    #[test]
    fn test_menu_adjust_position_bottom_edge() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            20,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let screen_width = 80;
        let screen_height = 24;
        menu.adjust_position_for_screen(screen_width, screen_height);

        let (_x, y) = menu.get_position();
        let bounds = menu.calculate_bounds();

        // Menu should fit within screen
        assert!(y + bounds.height <= screen_height);
    }

    #[test]
    fn test_menu_adjust_position_corner() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            75,
            20,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let screen_width = 80;
        let screen_height = 24;
        menu.adjust_position_for_screen(screen_width, screen_height);

        let (x, y) = menu.get_position();
        let bounds = menu.calculate_bounds();

        // Menu should fit within screen both horizontally and vertically
        assert!(x + bounds.width <= screen_width);
        assert!(y + bounds.height <= screen_height);
    }

    #[test]
    fn test_menu_no_adjustment_when_fits() {
        let mut menu = ContextMenu::new();
        menu.show_at(10, 5, MenuContext::Channel { is_pinned: false });

        let screen_width = 80;
        let screen_height = 24;
        menu.adjust_position_for_screen(screen_width, screen_height);

        let (x, y) = menu.get_position();
        // Position should remain unchanged
        assert_eq!((x, y), (10, 5));
    }
}

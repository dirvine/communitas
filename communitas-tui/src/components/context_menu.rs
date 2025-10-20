// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Context Menu System
//!
//! Provides right-click context menus with:
//! - Context-specific menu items
//! - Keyboard navigation
//! - Mouse selection
//! - Separator support
//! - Screen bounds adjustment

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

/// Actions that can be triggered from context menus
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    Reply,
    React,
    Forward,
    Copy,
    Edit,
    Delete,
    Pin,
    Unpin,
    Mute,
    Unmute,
    Leave,
    ViewProfile,
    AddToContacts,
    Block,
    Report,
    Custom(String),
}

/// Context in which a menu is shown
#[derive(Debug, Clone, PartialEq)]
pub enum MenuContext {
    Message { is_own: bool, can_edit: bool },
    Channel { is_admin: bool, is_muted: bool },
    User { is_contact: bool, is_blocked: bool },
    ChatList { has_unread: bool, is_pinned: bool },
}

/// A single menu item
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: MenuAction,
    pub enabled: bool,
    pub separator: bool,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(label: impl Into<String>, shortcut: Option<String>, action: MenuAction) -> Self {
        Self {
            label: label.into(),
            shortcut,
            action,
            enabled: true,
            separator: false,
        }
    }

    /// Create a separator item
    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            action: MenuAction::Custom("separator".into()),
            enabled: false,
            separator: true,
        }
    }

    /// Check if this item can be selected and executed
    pub fn is_actionable(&self) -> bool {
        !self.separator && self.enabled
    }

    /// Disable this menu item
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Bounds of a menu for positioning
#[derive(Debug, Clone, PartialEq)]
pub struct MenuBounds {
    pub width: u16,
    pub height: u16,
}

/// Context menu with keyboard and mouse navigation
#[derive(Debug, Clone)]
pub struct ContextMenu {
    pub items: Vec<MenuItem>,
    pub position: (u16, u16),
    pub selected_index: usize,
    pub visible: bool,
    pub context: Option<MenuContext>,
}

impl ContextMenu {
    /// Create a new context menu (hidden by default)
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            position: (0, 0),
            selected_index: 0,
            visible: false,
            context: None,
        }
    }

    /// Show the menu at a specific position with context
    pub fn show_at(&mut self, x: u16, y: u16, context: MenuContext) {
        self.items = self.build_items_for_context(&context);
        self.position = (x, y);
        self.visible = true;
        self.context = Some(context);
        self.selected_index = self.find_first_actionable_index();
    }

    /// Hide the menu
    pub fn hide(&mut self) {
        self.visible = false;
        self.items.clear();
        self.context = None;
    }

    /// Get the currently selected item if any
    pub fn selected_item(&self) -> Option<&MenuItem> {
        if self.visible && self.selected_index < self.items.len() {
            Some(&self.items[self.selected_index])
        } else {
            None
        }
    }

    /// Select next actionable item (skipping separators and disabled items)
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let start = self.selected_index;
        loop {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            if self.items[self.selected_index].is_actionable() || self.selected_index == start {
                break;
            }
        }
    }

    /// Select previous actionable item (skipping separators and disabled items)
    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let start = self.selected_index;
        loop {
            if self.selected_index == 0 {
                self.selected_index = self.items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            if self.items[self.selected_index].is_actionable() || self.selected_index == start {
                break;
            }
        }
    }

    /// Select item at mouse position (relative to menu position)
    pub fn select_at_mouse(&mut self, mouse_x: u16, mouse_y: u16) -> bool {
        let (menu_x, menu_y) = self.position;

        // Check if mouse is within menu bounds
        let bounds = self.calculate_bounds();
        if mouse_x < menu_x
            || mouse_x >= menu_x + bounds.width
            || mouse_y < menu_y
            || mouse_y >= menu_y + bounds.height
        {
            return false;
        }

        // Calculate which item was clicked (one item per row)
        let item_index = (mouse_y - menu_y) as usize;
        if item_index < self.items.len() && self.items[item_index].is_actionable() {
            self.selected_index = item_index;
            return true;
        }

        false
    }

    /// Execute the currently selected item and return the action
    pub fn execute_selected(&mut self) -> Option<MenuAction> {
        if let Some(item) = self.selected_item()
            && item.is_actionable()
        {
            let action = item.action.clone();
            self.hide();
            return Some(action);
        }
        None
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<MenuAction> {
        match key.code {
            KeyCode::Up => {
                self.select_previous();
                None
            }
            KeyCode::Down => {
                self.select_next();
                None
            }
            KeyCode::Enter => self.execute_selected(),
            KeyCode::Esc => {
                self.hide();
                None
            }
            _ => None,
        }
    }

    /// Handle mouse events
    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Option<MenuAction> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.select_at_mouse(mouse.column, mouse.row) {
                    self.execute_selected()
                } else {
                    // Click outside menu - close it
                    self.hide();
                    None
                }
            }
            MouseEventKind::Moved => {
                // Hover over items
                self.select_at_mouse(mouse.column, mouse.row);
                None
            }
            _ => None,
        }
    }

    /// Calculate menu bounds for positioning
    pub fn calculate_bounds(&self) -> MenuBounds {
        if self.items.is_empty() {
            return MenuBounds {
                width: 0,
                height: 0,
            };
        }

        // Calculate width based on longest item (label + shortcut + padding)
        let max_width = self
            .items
            .iter()
            .map(|item| {
                let label_len = item.label.len() as u16;
                let shortcut_len = item.shortcut.as_ref().map(|s| s.len() as u16).unwrap_or(0);
                label_len + shortcut_len + 6 // Padding
            })
            .max()
            .unwrap_or(20);

        MenuBounds {
            width: max_width,
            height: self.items.len() as u16,
        }
    }

    /// Adjust menu position to fit within screen bounds
    pub fn adjust_position_for_screen(&mut self, screen_width: u16, screen_height: u16) {
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

    /// Find the first actionable item index
    fn find_first_actionable_index(&self) -> usize {
        self.items
            .iter()
            .position(|item| item.is_actionable())
            .unwrap_or(0)
    }

    /// Build menu items based on context
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
            MenuContext::Channel { is_admin, is_muted } => {
                let mut items = vec![MenuItem::new(
                    "View Info",
                    Some("I".into()),
                    MenuAction::ViewProfile,
                )];

                if *is_muted {
                    items.push(MenuItem::new(
                        "Unmute",
                        Some("M".into()),
                        MenuAction::Unmute,
                    ));
                } else {
                    items.push(MenuItem::new("Mute", Some("M".into()), MenuAction::Mute));
                }

                items.push(MenuItem::separator());

                if *is_admin {
                    items.push(MenuItem::new(
                        "Edit Channel",
                        Some("E".into()),
                        MenuAction::Edit,
                    ));
                }

                items.push(MenuItem::new(
                    "Leave Channel",
                    Some("L".into()),
                    MenuAction::Leave,
                ));

                items
            }
            MenuContext::User {
                is_contact,
                is_blocked,
            } => {
                let mut items = vec![MenuItem::new(
                    "View Profile",
                    Some("P".into()),
                    MenuAction::ViewProfile,
                )];

                if !is_contact {
                    items.push(MenuItem::new(
                        "Add to Contacts",
                        Some("A".into()),
                        MenuAction::AddToContacts,
                    ));
                }

                items.push(MenuItem::separator());

                if *is_blocked {
                    items.push(MenuItem::new(
                        "Unblock",
                        Some("U".into()),
                        MenuAction::Custom("unblock".into()),
                    ));
                } else {
                    items.push(MenuItem::new("Block", Some("B".into()), MenuAction::Block));
                }

                items.push(MenuItem::new(
                    "Report",
                    Some("R".into()),
                    MenuAction::Report,
                ));

                items
            }
            MenuContext::ChatList {
                has_unread,
                is_pinned,
            } => {
                let mut items = Vec::new();

                if *is_pinned {
                    items.push(MenuItem::new("Unpin", Some("P".into()), MenuAction::Unpin));
                } else {
                    items.push(MenuItem::new("Pin", Some("P".into()), MenuAction::Pin));
                }

                if *has_unread {
                    items.push(MenuItem::new(
                        "Mark as Read",
                        Some("M".into()),
                        MenuAction::Custom("mark_read".into()),
                    ));
                }

                items.push(MenuItem::separator());
                items.push(MenuItem::new("Mute", Some("U".into()), MenuAction::Mute));
                items.push(MenuItem::new(
                    "Delete",
                    Some("Del".into()),
                    MenuAction::Delete,
                ));

                items
            }
        }
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("Test", Some("Ctrl+T".into()), MenuAction::Copy);
        assert_eq!(item.label, "Test");
        assert_eq!(item.shortcut, Some("Ctrl+T".into()));
        assert_eq!(item.action, MenuAction::Copy);
        assert!(item.enabled);
        assert!(!item.separator);
        assert!(item.is_actionable());
    }

    #[test]
    fn test_separator_item() {
        let sep = MenuItem::separator();
        assert!(sep.separator);
        assert!(!sep.enabled);
        assert!(!sep.is_actionable());
    }

    #[test]
    fn test_context_menu_show_hide() {
        let mut menu = ContextMenu::new();
        assert!(!menu.visible);

        menu.show_at(
            10,
            10,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );
        assert!(menu.visible);
        assert_eq!(menu.position, (10, 10));
        assert!(!menu.items.is_empty());

        menu.hide();
        assert!(!menu.visible);
        assert!(menu.items.is_empty());
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            0,
            0,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        let initial_index = menu.selected_index;
        menu.select_next();
        assert_ne!(menu.selected_index, initial_index);

        menu.select_previous();
        assert_eq!(menu.selected_index, initial_index);
    }

    #[test]
    fn test_menu_skips_separators() {
        let mut menu = ContextMenu::new();
        menu.items = vec![
            MenuItem::new("Item 1", None, MenuAction::Copy),
            MenuItem::separator(),
            MenuItem::new("Item 2", None, MenuAction::Edit),
        ];
        menu.visible = true;
        menu.selected_index = 0;

        menu.select_next();
        assert_eq!(menu.selected_index, 2); // Should skip separator at index 1
    }

    #[test]
    fn test_message_context_menu_own() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            0,
            0,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Should have Reply, React, Forward, separator, Copy, Edit, Delete
        assert!(menu.items.iter().any(|i| i.label == "Edit"));
        assert!(menu.items.iter().any(|i| i.label == "Delete"));
    }

    #[test]
    fn test_message_context_menu_other() {
        let mut menu = ContextMenu::new();
        menu.show_at(
            0,
            0,
            MenuContext::Message {
                is_own: false,
                can_edit: false,
            },
        );

        // Should NOT have Edit or Delete
        assert!(!menu.items.iter().any(|i| i.label == "Edit"));
        assert!(!menu.items.iter().any(|i| i.label == "Delete"));
    }

    #[test]
    fn test_menu_bounds_calculation() {
        let mut menu = ContextMenu::new();
        menu.items = vec![
            MenuItem::new("Short", None, MenuAction::Copy),
            MenuItem::new(
                "Very Long Label",
                Some("Ctrl+Shift+X".into()),
                MenuAction::Edit,
            ),
        ];

        let bounds = menu.calculate_bounds();
        assert_eq!(bounds.height, 2);
        assert!(bounds.width > 20); // Should be wide enough for longest item
    }

    #[test]
    fn test_menu_position_adjustment() {
        let mut menu = ContextMenu::new();
        menu.items = vec![
            MenuItem::new("Item", None, MenuAction::Copy),
            MenuItem::new("Item", None, MenuAction::Copy),
        ];

        // Position near bottom-right corner
        menu.position = (70, 18);
        menu.adjust_position_for_screen(80, 20);

        // Should adjust to fit within screen
        let bounds = menu.calculate_bounds();
        assert!(menu.position.0 + bounds.width <= 80);
        assert!(menu.position.1 + bounds.height <= 20);
    }
}

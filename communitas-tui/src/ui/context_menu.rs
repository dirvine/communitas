// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Context menu rendering for TUI

use crate::components::ContextMenu;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// Render context menu as an overlay
pub fn render(f: &mut Frame, menu: &ContextMenu) {
    if !menu.visible {
        return;
    }

    let bounds = menu.calculate_bounds();
    let (x, y) = menu.position;

    // Create the menu area
    let menu_area = Rect {
        x,
        y,
        width: bounds.width,
        height: bounds.height,
    };

    // Build menu items for rendering
    let list_items: Vec<ListItem> = menu
        .items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            if item.separator {
                // Render separator as a line
                let sep_line = "─".repeat(bounds.width.saturating_sub(2) as usize);
                ListItem::new(Line::from(Span::styled(
                    sep_line,
                    Style::default().fg(Color::DarkGray),
                )))
            } else {
                // Build label with shortcut
                let mut spans = Vec::new();

                // Add indicator for selected item
                if idx == menu.selected_index {
                    spans.push(Span::styled(
                        "> ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::raw("  "));
                }

                // Add label
                let label_style = if !item.enabled {
                    Style::default().fg(Color::DarkGray)
                } else if idx == menu.selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                spans.push(Span::styled(&item.label, label_style));

                // Add shortcut if present
                if let Some(ref shortcut) = item.shortcut {
                    // Calculate padding to right-align shortcut
                    let label_len = item.label.len() + 2; // +2 for selection indicator
                    let shortcut_len = shortcut.len();
                    let total_width = bounds.width.saturating_sub(2) as usize;
                    let padding_len = total_width.saturating_sub(label_len + shortcut_len + 1);

                    if padding_len > 0 {
                        spans.push(Span::raw(" ".repeat(padding_len)));
                    }

                    let shortcut_style = if !item.enabled {
                        Style::default().fg(Color::DarkGray)
                    } else if idx == menu.selected_index {
                        Style::default().fg(Color::Black).bg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    spans.push(Span::styled(shortcut, shortcut_style));
                }

                ListItem::new(Line::from(spans))
            }
        })
        .collect();

    // Create the list widget with border
    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    // Render the menu
    f.render_widget(list, menu_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::MenuContext;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn test_render_invisible_menu() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
        let menu = ContextMenu::new();

        terminal
            .draw(|f| {
                render(f, &menu);
            })
            .expect("Failed to draw");

        // Should not crash when rendering invisible menu
    }

    #[test]
    fn test_render_visible_menu() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        terminal
            .draw(|f| {
                render(f, &menu);
            })
            .expect("Failed to draw");

        // Should successfully render visible menu
    }

    #[test]
    fn test_render_menu_with_selection() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
        let mut menu = ContextMenu::new();
        menu.show_at(
            10,
            5,
            MenuContext::Message {
                is_own: true,
                can_edit: true,
            },
        );

        // Navigate to next item
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        menu.handle_key(KeyEvent::new(KeyCode::Down, KeyModifiers::empty()));

        terminal
            .draw(|f| {
                render(f, &menu);
            })
            .expect("Failed to draw");

        // Should successfully render with different selection
    }
}

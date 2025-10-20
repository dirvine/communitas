use crate::components::CommandPalette;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Render CommandPalette as centered overlay
pub fn render(f: &mut Frame, palette: &CommandPalette) {
    // Only render if visible
    if !palette.is_visible() {
        return;
    }

    // Calculate centered overlay area (60% width, 70% height)
    let overlay_area = centered_rect(60, 70, f.area());

    // Clear the background
    f.render_widget(Clear, overlay_area);

    // Create main container block
    let block = Block::default()
        .title(" Command Palette (Ctrl+K to close) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    // Split into search input and command list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(1),    // Command list
        ])
        .split(block.inner(overlay_area));

    // Render outer block
    f.render_widget(block, overlay_area);

    // Render search input
    render_search_input(f, chunks[0], palette);

    // Render command list
    render_command_list(f, chunks[1], palette);
}

/// Render search input box
fn render_search_input(f: &mut Frame, area: Rect, palette: &CommandPalette) {
    let query = palette.query();
    let text = if query.is_empty() {
        "Search commands...".to_string()
    } else {
        query.to_string()
    };

    let style = if query.is_empty() {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input = Paragraph::new(text).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(" Search "),
    );

    f.render_widget(input, area);
}

/// Render filtered command list with category headers
fn render_command_list(f: &mut Frame, area: Rect, palette: &CommandPalette) {
    let results = palette.results();
    let all_commands = palette.commands();
    let selected = palette.selected_command();

    if results.is_empty() {
        let empty = Paragraph::new("No commands found")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Commands "));
        f.render_widget(empty, area);
        return;
    }

    // Get filtered commands from results (indices)
    let filtered: Vec<&_> = results
        .iter()
        .filter_map(|(idx, _score)| all_commands.get(*idx))
        .collect();

    // Group commands by category
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_category = String::new();

    for cmd in filtered {
        // Add category header if changed
        if cmd.category != current_category {
            current_category = cmd.category.clone();

            // Add spacing before category (except first)
            if !items.is_empty() {
                items.push(ListItem::new(""));
            }

            // Category header
            items.push(ListItem::new(Line::from(vec![Span::styled(
                format!("▼ {}", current_category),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )])));
        }

        // Check if this command is selected
        let is_selected = selected.as_ref().map(|s| s.id == cmd.id).unwrap_or(false);

        // Command item
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let shortcut_style = if is_selected {
            Style::default().fg(Color::DarkGray).bg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let mut line_spans = vec![Span::raw("  "), Span::styled(&cmd.name, style)];

        // Show first shortcut if available
        if let Some(shortcut) = cmd.shortcuts.first() {
            line_spans.push(Span::raw("  "));
            line_spans.push(Span::styled(format!("[{}]", shortcut), shortcut_style));
        }

        items.push(ListItem::new(Line::from(line_spans)));

        // Add description as sub-item
        if is_selected && !cmd.description.is_empty() {
            items.push(ListItem::new(Line::from(vec![
                Span::raw("    "),
                Span::styled(&cmd.description, Style::default().fg(Color::Gray)),
            ])));
        }
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title(format!(" Commands ({}) ", results.len())),
    );

    f.render_widget(list, area);
}

/// Create a centered rect using up certain percentage of the available rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

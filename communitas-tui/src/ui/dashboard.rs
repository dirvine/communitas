use crate::state::{AppState, EntityType};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render main dashboard with entity type selector
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(format!(
            "Communitas TUI v{}",
            env!("CARGO_PKG_VERSION")
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Split into title, content, and instructions
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Entity list
            Constraint::Length(8),  // Instructions
        ])
        .split(block.inner(area));

    f.render_widget(block, area);

    // Title
    let title = Paragraph::new(vec![
        Line::from(Span::styled(
            "Select Entity Type:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ])
    .alignment(Alignment::Left);
    f.render_widget(title, chunks[0]);

    // Entity types list
    let entities = vec![
        EntityType::Organization,
        EntityType::Project,
        EntityType::Group,
        EntityType::Contact,
    ];

    let items: Vec<ListItem> = entities
        .iter()
        .enumerate()
        .map(|(i, entity_type)| {
            let is_selected = i == state.navigation.selected_index;
            let prefix = if is_selected { "→ " } else { "  " };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let text = format!(
                "{}{} {}  (Press '{}')",
                prefix,
                entity_type.icon(),
                entity_type.name(),
                entity_type.key()
            );

            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, chunks[1]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "i",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Initialize identity (required first)"),
        ]),
        Line::from(vec![
            Span::styled(
                "n",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Check network status"),
        ]),
        Line::from(vec![
            Span::styled(
                "↑↓",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" or "),
            Span::styled(
                "k/j",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Navigate"),
        ]),
        Line::from(vec![
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Select / Open"),
        ]),
        Line::from(vec![
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Quit"),
        ]),
    ])
    .alignment(Alignment::Left);

    f.render_widget(instructions, chunks[2]);
}

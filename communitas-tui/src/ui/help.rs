use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::state::AppState;

/// Render the help screen
pub fn render(f: &mut Frame, area: Rect, _state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Help content
            Constraint::Length(3), // Footer
        ])
        .split(area);

    render_title(f, chunks[0]);
    render_help_content(f, chunks[1]);
    render_footer(f, chunks[2]);
}

fn render_title(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("? Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let title = Paragraph::new(vec![Line::from(Span::styled(
        "Communitas TUI - Keyboard Reference",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))])
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(title, area);
}

fn render_help_content(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "NAVIGATION",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  o",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Open Organizations (Channels)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  p",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Open Projects"),
        ]),
        Line::from(vec![
            Span::styled(
                "  g",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Open Groups"),
        ]),
        Line::from(vec![
            Span::styled(
                "  c",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Open Contacts"),
        ]),
        Line::from(vec![
            Span::styled(
                "  n",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Check Network Status (from Dashboard)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  q",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Go Back / Quit"),
        ]),
        Line::from(vec![
            Span::styled(
                "  ?",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Show this help screen"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "LIST NAVIGATION",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  ↑↓ or k/j",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Navigate up/down in lists"),
        ]),
        Line::from(vec![
            Span::styled(
                "  Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("      Select item / Open view"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "ENTITY MANAGEMENT",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  n",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Create new entity (context-sensitive)"),
        ]),
        Line::from("    • In Organizations: Create new channel"),
        Line::from("    • In Projects: Create new project"),
        Line::from("    • In Groups: Create new group"),
        Line::from("    • In Contacts: Add new contact"),
        Line::from(""),
        Line::from(Span::styled(
            "MESSAGING",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("      Start typing message"),
        ]),
        Line::from(vec![
            Span::styled(
                "  Esc",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("        Cancel input / Go back"),
        ]),
        Line::from(vec![
            Span::styled(
                "  t",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Create thread reply (when message selected)"),
        ]),
        Line::from(vec![
            Span::styled(
                "  r",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  Add reaction (when message selected)"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "IDENTITY",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  Four-word identities are human-readable network addresses"),
        Line::from("  Example: ocean-forest-moon-star"),
        Line::from(""),
        Line::from(Span::styled(
            "  Status Bar (Bottom)",
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  Shows: Identity | Network Status | Current Status"),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let footer = Paragraph::new(vec![Line::from(Span::styled(
        "Press 'q' or 'Esc' to close help",
        Style::default().fg(Color::Green),
    ))])
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

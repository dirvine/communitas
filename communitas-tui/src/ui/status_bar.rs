use crate::state::AppState;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

/// Render status bar at bottom of screen
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let identity_text = if let Some(id) = &state.identity {
        id.clone()
    } else {
        "Not initialized".to_string()
    };

    let network_symbol = state.network.status_symbol();
    let network_color = match state.network.status_color() {
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "red" => Color::Red,
        _ => Color::Gray,
    };

    let status_text = state
        .status_message
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("");

    let status_line = vec![
        Span::raw("Identity: "),
        Span::styled(
            identity_text,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" │ Network: "),
        Span::styled(
            network_symbol,
            Style::default()
                .fg(network_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!(" ({}) ", state.network.peer_count)),
        Span::raw("│ "),
        Span::styled(status_text, Style::default().fg(Color::Yellow)),
        Span::raw(" │ "),
        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": quit │ "),
        Span::styled("?", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(": help"),
    ];

    let paragraph = Paragraph::new(Line::from(status_line))
        .block(Block::default())
        .style(Style::default().bg(Color::DarkGray));

    f.render_widget(paragraph, area);
}

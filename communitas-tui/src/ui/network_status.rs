use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use std::time::SystemTime;

use crate::state::{AppState, ConnectionStatus};

/// Render network status view
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Connection status
            Constraint::Length(7), // Peer information
            Constraint::Min(5),    // Bootstrap nodes list
        ])
        .split(area);

    render_connection_status(f, chunks[0], state);
    render_peer_info(f, chunks[1], state);
    render_bootstrap_nodes(f, chunks[2], state);
}

/// Render connection status summary
fn render_connection_status(f: &mut Frame, area: Rect, state: &AppState) {
    let network = &state.network;

    let (status_text, status_color) = match &network.status {
        ConnectionStatus::Connected => ("Connected to P2P Network", Color::Green),
        ConnectionStatus::Connecting => ("Connecting to P2P Network...", Color::Yellow),
        ConnectionStatus::Disconnected => ("Disconnected", Color::Gray),
        ConnectionStatus::Error(err) => (err.as_str(), Color::Red),
    };

    let status_symbol = network.status_symbol();

    let block = Block::default()
        .title(" Network Connection ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let status_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                status_symbol,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                status_text,
                Style::default()
                    .fg(status_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(status_lines)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render peer information
fn render_peer_info(f: &mut Frame, area: Rect, state: &AppState) {
    let network = &state.network;

    let block = Block::default()
        .title(" Peer Statistics ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let last_update = format_time_ago(network.last_update);

    let info_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Connected Peers: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network.peer_count.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Bootstrap Nodes: ", Style::default().fg(Color::Gray)),
            Span::styled(
                network.bootstrap_nodes.len().to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Last Update: ", Style::default().fg(Color::Gray)),
            Span::styled(last_update, Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(info_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Render bootstrap nodes list
fn render_bootstrap_nodes(f: &mut Frame, area: Rect, state: &AppState) {
    let network = &state.network;

    let block = Block::default()
        .title(" Bootstrap Nodes ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if network.bootstrap_nodes.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No bootstrap nodes configured",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "The application will attempt to connect using default nodes",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    } else {
        let items: Vec<ListItem> = network
            .bootstrap_nodes
            .iter()
            .enumerate()
            .map(|(i, node)| {
                let status_indicator =
                    if matches!(network.status, ConnectionStatus::Connected) && i == 0 {
                        Span::styled("✓ ", Style::default().fg(Color::Green))
                    } else {
                        Span::raw("  ")
                    };

                ListItem::new(vec![Line::from(vec![
                    status_indicator,
                    Span::styled(node, Style::default().fg(Color::Cyan)),
                ])])
            })
            .collect();

        let list = List::new(items).block(block);
        f.render_widget(list, area);
    }
}

/// Format elapsed time since timestamp
fn format_time_ago(time: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(time).unwrap_or_default();
    let seconds = duration.as_secs();

    if seconds < 60 {
        format!("{} seconds ago", seconds)
    } else if seconds < 3600 {
        format!("{} minutes ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}

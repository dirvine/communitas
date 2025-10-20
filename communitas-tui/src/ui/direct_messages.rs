use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::state::AppState;

/// Render direct messages view for a specific contact
pub fn render(f: &mut Frame, area: Rect, state: &AppState, contact_id: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Contact header
            Constraint::Min(5),    // Messages area
            Constraint::Length(3), // Input box
        ])
        .split(area);

    render_contact_header(f, chunks[0], state, contact_id);
    render_messages_area(f, chunks[1], state, contact_id);
    render_input_box(f, chunks[2], state);
}

/// Render contact header with name and status
fn render_contact_header(f: &mut Frame, area: Rect, state: &AppState, contact_id: &str) {
    let block = Block::default()
        .title(" Direct Message ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Find contact details from state
    let contact = state.entities.contacts.iter().find(|c| c.id == contact_id);

    let (contact_name, four_words, is_online) = if let Some(contact) = contact {
        let online = is_contact_online(contact.last_seen);
        (
            contact.display_name.as_str(),
            contact.four_words.as_str(),
            online,
        )
    } else {
        // Contact not in cache - show ID
        ("Unknown Contact", contact_id, false)
    };

    let online_indicator = if is_online {
        vec![
            Span::styled("●", Style::default().fg(Color::Green)),
            Span::raw(" Online"),
        ]
    } else {
        vec![
            Span::styled("●", Style::default().fg(Color::DarkGray)),
            Span::raw(" Offline"),
        ]
    };

    let header_text = vec![Line::from({
        let mut spans = vec![
            Span::styled(
                contact_name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(four_words, Style::default().fg(Color::Cyan)),
            Span::raw("  "),
        ];
        spans.extend(online_indicator);
        spans
    })];

    let paragraph = Paragraph::new(header_text)
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render messages area
fn render_messages_area(f: &mut Frame, area: Rect, state: &AppState, contact_id: &str) {
    let block = Block::default()
        .title(" Messages ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Get messages for this contact from state cache
    let messages = state.entities.messages.get(contact_id);

    if let Some(messages) = messages {
        if messages.is_empty() {
            // No messages yet
            let empty_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No messages yet",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Press Enter to start typing...",
                    Style::default().fg(Color::Yellow),
                )),
            ];
            let paragraph = Paragraph::new(empty_text)
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(paragraph, area);
        } else {
            // Render message list
            let message_lines: Vec<Line> = messages
                .iter()
                .map(|msg| {
                    // Format timestamp
                    let time_str = format_timestamp(msg.timestamp);

                    // Determine if this message is from us or the contact
                    let current_user_id = state.identity.as_deref().unwrap_or("");

                    let is_from_us = msg.author_id == current_user_id;

                    let author_label = if is_from_us {
                        "You"
                    } else {
                        msg.author_name.as_str()
                    };

                    let author_color = if is_from_us {
                        Color::Green
                    } else {
                        Color::Yellow
                    };

                    Line::from(vec![
                        Span::styled(
                            format!("[{}] ", time_str),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            format!("{}: ", author_label),
                            Style::default()
                                .fg(author_color)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(&msg.content),
                    ])
                })
                .collect();

            let paragraph = Paragraph::new(message_lines)
                .block(block)
                .wrap(Wrap { trim: false });
            f.render_widget(paragraph, area);
        }
    } else {
        // Messages not loaded yet - show loading indicator
        let loading_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Loading messages...",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(loading_text)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// Render input box
fn render_input_box(f: &mut Frame, area: Rect, state: &AppState) {
    use crate::state::navigation::FocusedPanel;

    let is_focused = matches!(state.navigation.focused_panel, FocusedPanel::Input);

    let border_color = if is_focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(if is_focused {
            " Type message (Enter to send, Esc to cancel) "
        } else {
            " Press Enter to type message "
        })
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let text = if is_focused {
        vec![Line::from(Span::styled(
            &state.input_buffer,
            Style::default().fg(Color::White),
        ))]
    } else {
        vec![Line::from(Span::styled(
            "Press Enter to start typing...",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        ))]
    };

    let paragraph = Paragraph::new(text).block(block);

    f.render_widget(paragraph, area);
}

/// Check if contact is online (within last 5 minutes)
fn is_contact_online(last_seen: Option<i64>) -> bool {
    if let Some(timestamp) = last_seen {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        now - timestamp < 300 // 5 minutes
    } else {
        false
    }
}

/// Format Unix timestamp as HH:MM
fn format_timestamp(timestamp: i64) -> String {
    let hours = (timestamp / 3600) % 24;
    let minutes = (timestamp / 60) % 60;
    format!("{:02}:{:02}", hours, minutes)
}

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::state::AppState;

/// Render the organizations/channels list view
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_channel_list(f, chunks[0], state);
    render_channel_detail(f, chunks[1], state);
}

/// Render the channel list sidebar
fn render_channel_list(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title("📢 Channels")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Get all channels across all organizations
    let mut all_channels = Vec::new();
    for channels in state.entities.channels.values() {
        all_channels.extend(channels.clone());
    }

    if all_channels.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No channels yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to create a channel",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    // Sort by unread count, then name
    all_channels.sort_by(|a, b| {
        b.unread_count
            .cmp(&a.unread_count)
            .then_with(|| a.name.cmp(&b.name))
    });

    let items: Vec<ListItem> = all_channels
        .iter()
        .enumerate()
        .map(|(i, channel)| {
            let is_selected = i == state.navigation.selected_index;
            let prefix = if is_selected { "→ " } else { "  " };

            let unread_badge = if channel.unread_count > 0 {
                format!(" ({})", channel.unread_count)
            } else {
                String::new()
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if channel.unread_count > 0 {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!(
                "{}# {}{}  ({})",
                prefix, channel.name, unread_badge, channel.member_count
            );
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// Render the channel detail/preview panel
fn render_channel_detail(f: &mut Frame, area: Rect, state: &AppState) {
    // Get selected channel
    let mut all_channels = Vec::new();
    for channels in state.entities.channels.values() {
        all_channels.extend(channels.clone());
    }

    if all_channels.is_empty() {
        let block = Block::default()
            .title("Channel Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block, area);
        return;
    }

    all_channels.sort_by(|a, b| {
        b.unread_count
            .cmp(&a.unread_count)
            .then_with(|| a.name.cmp(&b.name))
    });

    let selected_channel = match all_channels.get(state.navigation.selected_index) {
        Some(channel) => channel,
        None => {
            let block = Block::default()
                .title("Channel Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, area);
            return;
        }
    };

    let block = Block::default()
        .title(format!("# {}", selected_channel.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::Yellow)),
            Span::raw(
                selected_channel
                    .description
                    .clone()
                    .unwrap_or_else(|| "No description".to_string()),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Members: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", selected_channel.member_count)),
        ]),
        Line::from(""),
    ];

    if selected_channel.unread_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("Unread: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} messages", selected_channel.unread_count),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "───────────────────────────",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    // Show recent messages if available
    if let Some(messages) = state.entities.messages.get(&selected_channel.id) {
        lines.push(Line::from(Span::styled(
            "Recent Messages:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for msg in messages.iter().take(5) {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{}: ", msg.author_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw(&msg.content),
            ]));
            lines.push(Line::from(""));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Enter to open channel",
        Style::default().fg(Color::Green),
    )));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Render the channel messages view
pub fn render_channel_view(f: &mut Frame, area: Rect, state: &AppState, channel_id: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    render_messages(f, chunks[0], state, channel_id);
    render_message_input(f, chunks[1], state);
}

/// Render the message list for a channel
fn render_messages(f: &mut Frame, area: Rect, state: &AppState, channel_id: &str) {
    // Get channel name for title
    let channel_name = state
        .entities
        .channels
        .values()
        .flatten()
        .find(|c| c.id == channel_id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "Unknown Channel".to_string());

    let block = Block::default()
        .title(format!("# {}", channel_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let messages = state.entities.messages.get(channel_id);

    if messages.is_none() || messages.as_ref().map_or(true, |m| m.is_empty()) {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No messages yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type a message below and press Enter",
                Style::default().fg(Color::Yellow),
            )),
        ];
        let paragraph = Paragraph::new(empty_text)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
        return;
    }

    let messages = messages.as_ref().unwrap();
    let mut lines = Vec::new();

    for msg in messages.iter() {
        // Timestamp line
        lines.push(Line::from(Span::styled(
            format!("[{}]", format_timestamp(msg.timestamp)),
            Style::default().fg(Color::DarkGray),
        )));

        // Author and content
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", msg.author_name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&msg.content),
        ]));

        // Reactions if any
        if !msg.reactions.is_empty() {
            let reactions_str = msg
                .reactions
                .iter()
                .map(|r| format!("{} {}", r.emoji, r.count))
                .collect::<Vec<_>>()
                .join("  ");
            lines.push(Line::from(Span::styled(
                format!("    {}", reactions_str),
                Style::default().fg(Color::Yellow),
            )));
        }

        // Thread indicator
        if msg.thread_count > 0 {
            lines.push(Line::from(Span::styled(
                format!("    💬 {} replies", msg.thread_count),
                Style::default().fg(Color::Blue),
            )));
        }

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Render the message input box
fn render_message_input(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title("💬 Type message (Enter to send, Esc to go back)")
        .borders(Borders::ALL)
        .border_style(if state.input_active {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        });

    let text = if state.input_active {
        state.input_buffer.clone()
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(text).block(block);
    f.render_widget(paragraph, area);

    // Show cursor if input is active
    if state.input_active && !state.input_buffer.is_empty() {
        let cursor_x = area.x + state.input_buffer.len() as u16 + 1;
        let cursor_y = area.y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

/// Render thread view (replies to a message)
pub fn render_thread_view(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    _channel_id: &str,
    thread_id: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);

    // Thread header with parent message
    render_thread_header(f, chunks[0], state, thread_id);

    // Thread replies
    render_thread_messages(f, chunks[1], state, thread_id);

    // Reply input
    render_message_input(f, chunks[2], state);
}

/// Render thread header showing parent message
fn render_thread_header(f: &mut Frame, area: Rect, state: &AppState, thread_id: &str) {
    // Find parent message
    let parent_msg = state
        .entities
        .messages
        .values()
        .flatten()
        .find(|m| m.thread_id.as_ref() == Some(&thread_id.to_string()));

    let block = Block::default()
        .title("💬 Thread")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    if let Some(msg) = parent_msg {
        let lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("{}: ", msg.author_name),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&msg.content),
            ]),
            Line::from(Span::styled(
                format!("{} replies", msg.thread_count),
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("Thread not found").block(block);
        f.render_widget(paragraph, area);
    }
}

/// Render thread replies
fn render_thread_messages(f: &mut Frame, area: Rect, state: &AppState, thread_id: &str) {
    let block = Block::default()
        .title("Replies")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    // Get all messages in this thread
    let thread_messages: Vec<_> = state
        .entities
        .messages
        .values()
        .flatten()
        .filter(|m| m.thread_id.as_ref() == Some(&thread_id.to_string()))
        .collect();

    if thread_messages.is_empty() {
        let paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No replies yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type a reply and press Enter",
                Style::default().fg(Color::Yellow),
            )),
        ])
        .block(block);
        f.render_widget(paragraph, area);
        return;
    }

    let mut lines = Vec::new();
    for msg in thread_messages.iter() {
        lines.push(Line::from(Span::styled(
            format!("[{}]", format_timestamp(msg.timestamp)),
            Style::default().fg(Color::DarkGray),
        )));

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", msg.author_name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(&msg.content),
        ]));

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Format Unix timestamp to human-readable time
fn format_timestamp(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let now = duration.as_secs() as i64;
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

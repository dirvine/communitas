use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::state::AppState;

/// Render the groups list view
pub fn render(f: &mut Frame, area: Rect, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_group_list(f, chunks[0], state);
    render_group_detail(f, chunks[1], state);
}

/// Render the group list sidebar
fn render_group_list(f: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title("👥 Groups")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let groups = &state.entities.groups;

    if groups.is_empty() {
        let empty_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No groups yet",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press 'n' to create a group",
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
    let mut sorted_groups = groups.clone();
    sorted_groups.sort_by(|a, b| {
        b.unread_count
            .cmp(&a.unread_count)
            .then_with(|| a.name.cmp(&b.name))
    });

    let items: Vec<ListItem> = sorted_groups
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let is_selected = i == state.navigation.selected_index;
            let prefix = if is_selected { "→ " } else { "  " };

            let unread_badge = if group.unread_count > 0 {
                format!(" ({})", group.unread_count)
            } else {
                String::new()
            };

            let last_msg_preview = if let Some(ref last_msg) = group.last_message {
                format!("\n    {}", last_msg)
            } else {
                String::new()
            };

            let style = if is_selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else if group.unread_count > 0 {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let text = format!(
                "{}{} ({} members){}{}",
                prefix, group.name, group.member_count, unread_badge, last_msg_preview
            );
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

/// Render the group detail/preview panel
fn render_group_detail(f: &mut Frame, area: Rect, state: &AppState) {
    let groups = &state.entities.groups;

    if groups.is_empty() {
        let block = Block::default()
            .title("Group Details")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        f.render_widget(block, area);
        return;
    }

    let mut sorted_groups = groups.clone();
    sorted_groups.sort_by(|a, b| {
        b.unread_count
            .cmp(&a.unread_count)
            .then_with(|| a.name.cmp(&b.name))
    });

    let selected_group = match sorted_groups.get(state.navigation.selected_index) {
        Some(group) => group,
        None => {
            let block = Block::default()
                .title("Group Details")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            f.render_widget(block, area);
            return;
        }
    };

    let block = Block::default()
        .title(format!("👥 {}", selected_group.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Members: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}", selected_group.member_count)),
        ]),
        Line::from(""),
    ];

    if selected_group.unread_count > 0 {
        lines.push(Line::from(vec![
            Span::styled("Unread: ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} messages", selected_group.unread_count),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
    }

    if let Some(ref last_msg) = selected_group.last_message {
        lines.push(Line::from(Span::styled(
            "───────────────────────────",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Last Message:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::raw(last_msg)));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Enter to open group messages",
        Style::default().fg(Color::Green),
    )));

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

/// Render the group messages view
pub fn render_messages(f: &mut Frame, area: Rect, state: &AppState, group_id: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    render_group_message_list(f, chunks[0], state, group_id);
    render_message_input(f, chunks[1], state);
}

/// Render the message list for a group
fn render_group_message_list(f: &mut Frame, area: Rect, state: &AppState, group_id: &str) {
    // Get group name for title
    let group_name = state
        .entities
        .groups
        .iter()
        .find(|g| g.id == group_id)
        .map(|g| g.name.clone())
        .unwrap_or_else(|| "Unknown Group".to_string());

    let block = Block::default()
        .title(format!("👥 {}", group_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let messages = state.entities.messages.get(group_id);

    let Some(messages) = messages.as_ref() else {
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
    };

    if messages.is_empty() {
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
